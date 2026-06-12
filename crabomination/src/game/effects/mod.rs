//! Resolver for the unified `Effect` tree.
//!
//! A single entry point — [`GameState::resolve_effect`] — walks the effect
//! tree against an [`EffectContext`] describing the casting/activating player,
//! chosen target(s), etc. Combinators (`Seq`, `If`, `ForEach`, `Repeat`,
//! `ChooseMode`) recurse; leaf mutations perform game-state changes and emit
//! [`GameEvent`]s.

mod delayed;
mod eval;
mod events;
mod movement;
mod targeting;

// Token factories now live in `crabomination_base`; re-exported here so the
// engine's `game::effects::*_token` paths keep working.
pub use crabomination_base::tokens::{
    blood_token, clue_token, detective_token, food_token, token_to_card_definition, treasure_token,
};
pub(crate) use delayed::delayed_kind_from_effect;
pub(crate) use events::{emblem_event_matches, event_matches_spec, event_subject};

use super::*;
use crate::card::{
    CardId, CardInstance, CounterType,
    Keyword, SelectionRequirement, Zone,
};
use crate::effect::{
    AttackingTokenCleanup, Duration, Effect, ManaPayload, PlayerRef,
    Selector, ZoneDest, ZoneRef,
};
use crate::game::layers::EffectDuration;
use crate::mana::Color;

/// Continuation for per-player decision loops that suspend mid-iteration:
/// re-runs `make(seat)` for each not-yet-processed seat once the suspended
/// seat's answer resolves, so a symmetric effect ("each player discards…")
/// doesn't silently stop at the first `wants_ui` player.
pub(crate) fn per_seat_continuation(rest: &[usize], make: impl Fn(usize) -> Effect) -> Effect {
    let mut effs: Vec<Effect> = rest.iter().map(|&q| make(q)).collect();
    match effs.len() {
        0 => Effect::Noop,
        1 => effs.pop().unwrap(),
        _ => Effect::Seq(effs),
    }
}

/// Translate the cast-site `effect::Duration` into the runtime
/// `layers::EffectDuration` used by the continuous-effect layer system.
///
/// CR 511.2: `Duration::EndOfCombat` maps to the dedicated
/// `UntilEndOfCombat` variant (cleared in `do_combat_end`) so that
/// "until end of combat" effects don't linger into the post-combat
/// main phase.
pub(crate) fn map_effect_duration(
    duration: crate::effect::Duration,
) -> EffectDuration {
    match duration {
        crate::effect::Duration::EndOfTurn => EffectDuration::UntilEndOfTurn,
        crate::effect::Duration::EndOfCombat => EffectDuration::UntilEndOfCombat,
        crate::effect::Duration::UntilNextTurn
        | crate::effect::Duration::UntilYourNextUntap => {
            EffectDuration::UntilNextTurn
        }
        crate::effect::Duration::Permanent => EffectDuration::Indefinite,
    }
}

/// Rank card names by frequency (descending, ties by first appearance),
/// deduped — the heuristic feed for `Decision::NameCard` suggestions.
pub(crate) fn rank_names_by_frequency<'a>(
    names: impl IntoIterator<Item = &'a str>,
) -> Vec<String> {
    let mut order: Vec<(&str, u32)> = Vec::new();
    for n in names {
        match order.iter_mut().find(|(name, _)| *name == n) {
            Some((_, c)) => *c += 1,
            None => order.push((n, 1)),
        }
    }
    order.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    order.into_iter().map(|(n, _)| n.to_string()).collect()
}

/// Runtime context threaded through effect resolution.
#[derive(Debug, Clone)]
pub struct EffectContext {
    pub controller: usize,
    pub source: Option<CardId>,
    /// Targets chosen at cast/activation time (typically 0 or 1 entries).
    pub targets: Vec<Target>,
    /// The entity that caused the current trigger to fire (`Selector::TriggerSource`).
    pub trigger_source: Option<EntityRef>,
    /// Modal choice index (for `Effect::ChooseMode`).
    pub mode: usize,
    pub x_value: u32,
    /// Number of distinct colors of mana spent on the spell's cost
    /// (Pest Control, Prismatic Ending). Computed at cast time and
    /// threaded through `StackItem::Spell`.
    pub converged_value: u32,
    /// Total mana spent paying the originating spell's cost
    /// (Increment / Opus payoffs). Computed at cast time and
    /// threaded through `StackItem::Spell` / `StackItem::Trigger`.
    pub mana_spent: u32,
    /// The resolving spell's printed name. Stamped by
    /// `for_spell_with_source` so predicates that need to introspect the
    /// spell's name (e.g. `Predicate::SameNamedInZoneAtLeast` for
    /// Dragon's Approach's "same name in your graveyard" rider) can
    /// read it directly — the card itself is in transient ownership
    /// during spell resolution and isn't present in any visible zone,
    /// so a zone-walking lookup wouldn't find it.
    pub source_name: Option<&'static str>,
    /// True if the resolving spell was cast from its caster's hand.
    /// False for flashback / cast-from-graveyard / cast-from-exile
    /// paths. Stamped by `for_spell_with_source` from the resolving
    /// `CardInstance.cast_from_hand` flag, read by
    /// `Predicate::CastFromGraveyard` (Increasing Vengeance "if cast
    /// from graveyard, copy twice instead"). Defaults to `true` for
    /// non-spell contexts (triggers, activated abilities) since those
    /// don't have a "cast zone" concept.
    pub cast_from_hand: bool,
    /// Per-event amount of the firing event (life gained, life lost,
    /// damage dealt, cards drawn, …). Set on trigger resolutions from
    /// the event payload (`StackItem::Trigger.event_amount`) so trigger
    /// bodies can read it via `Value::TriggerEventAmount`. Used by
    /// Light of Promise's "Whenever you gain life, put that many
    /// +1/+1 counters on target creature you control." Defaults to 0
    /// for non-trigger contexts (spells, activated abilities).
    pub event_amount: u32,
    /// True if the resolving spell was kicked (CR 702.32). Stamped from
    /// the resolving `CardInstance.kicked` flag; read by
    /// `Predicate::SpellWasKicked`. Defaults to `false` for non-spell
    /// contexts.
    pub kicked: bool,
    /// True if the resolving spell was bargained (CR 702.176). Stamped from
    /// the resolving `CardInstance.bargained` flag; read by
    /// `Predicate::SpellWasBargained`. Defaults to `false`.
    pub bargained: bool,
    /// True if the resolving spell was entwined (CR 702.41): its
    /// `ChooseMode` runs every mode in order. Defaults to `false`.
    pub entwined: bool,
}

impl EffectContext {
    pub fn for_spell(caster: usize, target: Option<Target>, mode: usize, x_value: u32) -> Self {
        Self {
            controller: caster,
            source: None,
            targets: target.into_iter().collect(),
            trigger_source: None,
            mode,
            x_value,
            converged_value: 0,
            mana_spent: 0,
            source_name: None,
            cast_from_hand: true,
            event_amount: 0,
            kicked: false,
            bargained: false,
            entwined: false,
        }
    }
    /// Spell-resolution context with the resolving spell's
    /// `CardId` + printed name stamped onto `ctx.source` /
    /// `ctx.source_name` and all cast-time scalars (X, Converge, total
    /// mana spent) threaded in. Lets predicates that introspect the
    /// resolving spell (e.g. `Predicate::SameNamedInZoneAtLeast` for
    /// Dragon's Approach's "same name in your graveyard" rider) read
    /// the spell's identity without needing to find the card in any
    /// game zone — during spell resolution the card is in transient
    /// ownership (popped from the stack, not yet placed in graveyard),
    /// so a zone-walking lookup would fail.
    #[allow(clippy::too_many_arguments)]
    pub fn for_spell_with_source(
        spell_card: CardId,
        spell_name: &'static str,
        caster: usize,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        mana_spent: u32,
    ) -> Self {
        Self::for_spell_with_source_and_origin(
            spell_card,
            spell_name,
            caster,
            target,
            additional_targets,
            mode,
            x_value,
            converged_value,
            mana_spent,
            true,
        )
    }

    /// Variant of `for_spell_with_source` that also stamps the
    /// `cast_from_hand` flag so the resolution-time predicate
    /// `Predicate::CastFromGraveyard` (Increasing Vengeance) can read
    /// whether the spell came from hand vs graveyard / flashback. The
    /// no-origin sibling defaults to `cast_from_hand = true` since the
    /// vast majority of spell resolutions are hand casts.
    #[allow(clippy::too_many_arguments)]
    pub fn for_spell_with_source_and_origin(
        spell_card: CardId,
        spell_name: &'static str,
        caster: usize,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        mana_spent: u32,
        cast_from_hand: bool,
    ) -> Self {
        // Merge slot 0 (`target`) + slots 1+ (`additional_targets`) into
        // a single `targets` Vec so `Selector::Target(n)` reads slot n
        // for any n. Single-target spells leave `additional_targets`
        // empty.
        let mut targets: Vec<Target> = target.into_iter().collect();
        targets.extend(additional_targets);
        Self {
            controller: caster,
            source: Some(spell_card),
            targets,
            trigger_source: None,
            mode,
            x_value,
            converged_value,
            mana_spent,
            source_name: Some(spell_name),
            cast_from_hand,
            event_amount: 0,
            kicked: false,
            bargained: false,
            entwined: false,
        }
    }
    pub fn for_trigger(
        source: CardId,
        controller: usize,
        target: Option<Target>,
        mode: usize,
    ) -> Self {
        Self {
            controller,
            source: Some(source),
            targets: target.into_iter().collect(),
            trigger_source: Some(EntityRef::Permanent(source)),
            mode,
            x_value: 0,
            converged_value: 0,
            mana_spent: 0,
            source_name: None,
            cast_from_hand: true,
            event_amount: 0,
            kicked: false,
            bargained: false,
            entwined: false,
        }
    }
    pub fn for_ability(
        source: CardId,
        controller: usize,
        target: Option<Target>,
    ) -> Self {
        Self {
            controller,
            source: Some(source),
            targets: target.into_iter().collect(),
            trigger_source: Some(EntityRef::Permanent(source)),
            mode: 0,
            x_value: 0,
            converged_value: 0,
            mana_spent: 0,
            source_name: None,
            cast_from_hand: true,
            event_amount: 0,
            kicked: false,
            bargained: false,
            entwined: false,
        }
    }
}

/// A resolved reference to something in the game (used internally for selector
/// resolution and `ForEach` iteration).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EntityRef {
    Player(usize),
    Permanent(CardId),
    /// A card in a non-battlefield zone (library/graveyard/exile/hand).
    Card(CardId),
}

impl EntityRef {
    /// Extract the `CardId` if this entity is a battlefield permanent.
    pub fn as_permanent_id(&self) -> Option<CardId> {
        match *self {
            EntityRef::Permanent(c) => Some(c),
            _ => None,
        }
    }
    /// The `CardId` behind a card-or-permanent reference. Unlike
    /// `as_permanent_id`, this also unwraps `EntityRef::Card` — used by the
    /// counter-class effects, which match a spell on the stack. A cast
    /// spell's `Selector::TriggerSource` resolves to `EntityRef::Card`
    /// (see `event_subject`), so a SpellCast-triggered "counter that spell"
    /// (Chalice of the Void) needs this broader unwrap to find its target.
    pub fn as_card_id(&self) -> Option<CardId> {
        match *self {
            EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
            _ => None,
        }
    }
}

impl GameState {
    /// Ask `seat` a yes/no question inside a (possibly multi-question)
    /// resolution effect, replaying `resolution_answer_log` first. Returns
    /// `None` when the effect must suspend for a `wants_ui` seat (the
    /// originating `effect` is re-queued; the resume re-runs it with the
    /// log one answer longer). `cursor` is the caller's per-run ask index —
    /// start it at 0 and pass the same variable to every ask in the arm.
    /// The caller must keep all side effects *after* its final ask (the
    /// re-run repeats everything before the suspension point) and call
    /// `clear_answer_log()` on every completing path.
    pub(crate) fn ask_seat_bool(
        &mut self,
        cursor: &mut usize,
        seat: usize,
        description: String,
        source: CardId,
        effect: &Effect,
    ) -> Option<bool> {
        use crate::decision::{Decision, DecisionAnswer};
        if let Some(a) = self.resolution_answer_log.get(*cursor) {
            *cursor += 1;
            return Some(matches!(a, DecisionAnswer::Bool(true)));
        }
        let decision = Decision::OptionalTrigger { source, description };
        if self.players.get(seat).is_some_and(|p| p.wants_ui) {
            self.suspend_signal = Some((
                decision,
                PendingEffectState::SeatBoolAnswerPending { player: seat },
                effect.clone(),
            ));
            return None;
        }
        let b = matches!(self.decider.decide(&decision), DecisionAnswer::Bool(true));
        // Log synchronous answers too, so a later suspend's re-run replays
        // them instead of re-asking the decider.
        self.resolution_answer_log.push(DecisionAnswer::Bool(b));
        *cursor += 1;
        Some(b)
    }

    /// Drop the multi-question replay log — call when a log-using effect
    /// reaches any completing path (see `ask_seat_bool`).
    pub(crate) fn clear_answer_log(&mut self) {
        self.resolution_answer_log.clear();
    }

    /// CR 707.10 — push `n` copies of the spell `cid` (if it's on the
    /// stack and copyable) directly above it. Copies inherit the
    /// original's target / mode / x / converged value and are flagged
    /// uncounterable + token (so they cease to exist off the stack).
    /// When `choose_new_targets` is set (Reverberate / Fork), the copy's
    /// controller may repoint the primary target via `Decision::ChooseTarget`
    /// (the original target is offered first, so AutoDecider keeps it).
    pub(crate) fn copy_stack_spell(
        &mut self,
        cid: CardId,
        n: usize,
        choose_new_targets: bool,
        events: &mut Vec<GameEvent>,
    ) {
        self.copy_stack_spell_controlled(cid, n, choose_new_targets, None, events);
    }

    /// As `copy_stack_spell`, but `controller` (when `Some`) controls the
    /// copies and chooses their new targets — CR 702.150 Demonstrate hands a
    /// copy to an opponent.
    pub(crate) fn copy_stack_spell_controlled(
        &mut self,
        cid: CardId,
        n: usize,
        choose_new_targets: bool,
        controller: Option<usize>,
        events: &mut Vec<GameEvent>,
    ) {
        use crate::game::types::StackItem;
        // Locate the matching Spell on the stack (topmost wins).
        let Some(idx) = self.stack.iter().rposition(|s| {
            matches!(s, StackItem::Spell { card, .. } if card.id == cid)
        }) else {
            return;
        };
        // CR 707 — a spell with `Keyword::CantBeCopied` is skipped.
        if let StackItem::Spell { card, .. } = &self.stack[idx]
            && card.definition.keywords.contains(&crate::card::Keyword::CantBeCopied)
        {
            return;
        }
        let (orig_card_def, orig_caster, target, additional_targets, mode, x_value, converged_value) =
            if let StackItem::Spell {
                card, caster, target, additional_targets, mode, x_value, converged_value, ..
            } = &self.stack[idx]
            {
                (
                    card.definition.clone(),
                    *caster,
                    target.clone(),
                    additional_targets.clone(),
                    *mode,
                    *x_value,
                    *converged_value,
                )
            } else {
                return;
            };
        let caster = controller.unwrap_or(orig_caster);
        for _ in 0..n {
            // Per copy, optionally let the controller choose a new primary
            // target (CR 115.7). Legal targets are enumerated against the
            // copy's own effect; the original is offered first so the
            // default (AutoDecider) keeps it.
            let copy_target = if choose_new_targets && target.is_some() {
                self.repoint_copy_target(&orig_card_def, caster, &target)
            } else {
                target.clone()
            };
            let new_id = self.next_id();
            let mut copy_inst = crate::card::CardInstance::new(new_id, orig_card_def.clone(), caster);
            // CR 707.10a — a copy of a spell ceases to exist off the stack.
            copy_inst.is_token = true;
            self.stack.push(StackItem::Spell {
                card: Box::new(copy_inst),
                caster,
                target: copy_target,
                additional_targets: additional_targets.clone(),
                mode,
                x_value,
                converged_value,
                mana_spent: 0,
                uncounterable: true, // copies can't be countered
            });
        }
        events.push(GameEvent::SpellsCopied { original: cid, count: n as u32 });
    }

    /// Helper for `copy_stack_spell`: enumerate legal targets for the
    /// copied spell's primary effect and ask `caster`'s decider to pick
    /// one (original offered first). Returns the chosen target, or the
    /// original when no choice is made / no legal alternative exists.
    fn repoint_copy_target(
        &mut self,
        def: &crate::card::CardDefinition,
        caster: usize,
        original: &Option<crate::game::types::Target>,
    ) -> Option<crate::game::types::Target> {
        use crate::decision::{Decision, DecisionAnswer};
        let mut legal = self.enumerate_legal_targets(&def.effect, caster);
        if legal.is_empty() {
            return original.clone();
        }
        // Offer the original first so the conservative default keeps it —
        // but only when it's actually legal for *this* chooser. A
        // Demonstrate copy handed to an opponent (CR 702.150 / 707.12) often
        // can't keep the caster's target ("…you control"), so forcing it
        // would pick an illegal target; in that case leave the legal set as
        // enumerated so the default lands on a legal object.
        if let Some(orig) = original
            && legal.contains(orig)
        {
            legal.retain(|t| t != orig);
            legal.insert(0, orig.clone());
        }
        let source = match original {
            Some(crate::game::types::Target::Permanent(c)) => *c,
            _ => crate::card::CardId(0),
        };
        let answer = self.decider.decide(&Decision::ChooseTarget {
            source,
            legal: legal.clone(),
            source_name: def.name.to_string(),
            description: "choose new targets for the copy".to_string(),
        });
        match answer {
            DecisionAnswer::Target(t) if legal.contains(&t) => Some(t),
            _ => original.clone(),
        }
    }

    /// CR 707 — apply a permanent's `enters_as_copy` replacement as it
    /// enters the battlefield. Auto-picks the highest-power matching
    /// permanent (excluding the copier itself); no-op when nothing
    /// matches. Called from the spell-resolution ETB path before SBA.
    pub(crate) fn apply_enters_as_copy(
        &mut self,
        card_id: CardId,
        controller: usize,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        let spec = self
            .battlefield
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| c.definition.enters_as_copy.clone());
        let Some(spec) = spec else { return false };
        // Capture the copier's own printed name before the copy rewrite, for
        // the CR 707.2 name-retention exception (Mockingbird).
        let original_name: &'static str = self
            .battlefield
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| c.definition.name)
            .unwrap_or("");
        // Best legal copy source: highest power among matching permanents,
        // never the copier itself.
        let source = self
            .battlefield
            .iter()
            .filter(|c| c.id != card_id)
            .filter(|c| {
                self.evaluate_requirement_static(
                    &spec.filter,
                    &Target::Permanent(c.id),
                    controller,
                    None,
                )
            })
            .max_by_key(|c| c.definition.power)
            .map(|c| c.id);
        let Some(source) = source else { return false };
        let ctx = EffectContext::for_trigger(
            card_id,
            controller,
            Some(Target::Permanent(source)),
            0,
        );
        if let Ok(evs) = self.resolve_effect(
            &Effect::BecomeCopyOf {
                what: crate::effect::Selector::This,
                source: crate::effect::Selector::Target(0),
                extra_creature_types: spec.extra_creature_types.clone(),
            },
            &ctx,
        ) {
            events.extend(evs);
        }
        // Layer the copy-exception abilities/keywords on top of the
        // copiable characteristics (e.g. Phantasmal Image's sacrifice rider),
        // and restore the copier's own name for the CR 707.2 name-retention
        // exception (Mockingbird). `original_name` was captured before the
        // copy rewrite stamped the source's name over it.
        if (!spec.extra_triggered.is_empty()
            || !spec.extra_keywords.is_empty()
            || !spec.extra_card_types.is_empty()
            || spec.keep_name)
            && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id)
        {
            let def = std::sync::Arc::make_mut(&mut c.definition);
            def.triggered_abilities
                .extend(spec.extra_triggered.iter().cloned());
            for kw in &spec.extra_keywords {
                if !def.keywords.contains(kw) {
                    def.keywords.push(kw.clone());
                }
            }
            for t in &spec.extra_card_types {
                if !def.card_types.contains(t) {
                    def.card_types.push(t.clone());
                }
            }
            if spec.keep_name {
                def.name = original_name;
            }
        }
        true
    }

    // ── Entry points ─────────────────────────────────────────────────────────

    /// Heuristic for `Effect::Punisher`: would the chooser (`ctx.controller`)
    /// be willing and able to perform `opt` to dodge the punisher's payoff?
    /// `LoseLife` is affordable only while it leaves the chooser alive;
    /// `Sacrifice` needs a matching permanent; `Seq` needs all parts
    /// affordable. Everything else is treated as freely doable.
    fn punisher_option_affordable(&self, opt: &Effect, ctx: &EffectContext) -> bool {
        match opt {
            Effect::LoseLife { who, amount } => {
                let cost = self.evaluate_value(amount, ctx).max(0);
                self.resolve_selector(who, ctx).into_iter().all(|e| match e {
                    EntityRef::Player(p) => (self.players[p].life as i64) > cost as i64,
                    _ => true,
                })
            }
            Effect::Sacrifice { who, filter, .. }
            | Effect::SacrificeGreatestMV { who, filter, .. } => {
                self.resolve_selector(who, ctx).into_iter().all(|e| match e {
                    EntityRef::Player(p) => self.battlefield.iter().any(|c| {
                        c.controller == p
                            && self.evaluate_requirement_static(
                                filter,
                                &Target::Permanent(c.id),
                                p,
                                ctx.source,
                            )
                    }),
                    _ => true,
                })
            }
            Effect::Seq(v) => v.iter().all(|e| self.punisher_option_affordable(e, ctx)),
            _ => true,
        }
    }

    pub(crate) fn resolve_effect(
        &mut self,
        effect: &Effect,
        ctx: &EffectContext,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Reset sacrificed-power / sacrificed-toughness / sacrificed-mana-value
        // scratch for this independent resolution.
        self.sacrificed_power = None;
        self.sacrificed_toughness = None;
        self.sacrificed_mana_value = None;
        // Reset last-created-token scratch — `Selector::LastCreatedToken`
        // (singular) and `Selector::LastCreatedTokens` (plural) only refer
        // to tokens created by *this* resolution.
        self.last_created_token = None;
        self.last_created_tokens.clear();
        // Reset last-moved-cards scratch — `Selector::LastMoved` only
        // refers to cards moved by *this* resolution (Practiced
        // Scrollsmith's ETB chains Move → GrantMayPlay on the same
        // moved card via this scratch).
        self.last_moved_cards.clear();
        // Reset cards-discarded scratch — `Value::CardsDiscardedThisEffect`
        // only counts discards from *this* resolution (Borrowed Knowledge
        // mode 1's "draw cards equal to the number discarded this way").
        self.cards_discarded_this_resolution = 0;
        self.creature_cards_discarded_this_resolution = 0;
        self.cards_discarded_per_player_this_resolution.clear();
        self.nonland_cards_discarded_per_player_this_resolution.clear();
        self.discarded_card_ids_this_resolution.clear();
        self.permanents_destroyed_this_resolution = 0;
        self.players_sacrificed_this_resolution.clear();
        self.named_card_this_resolution = None;
        let mut events = vec![];
        self.run_effect(effect, ctx, &mut events)?;
        Ok(events)
    }

    /// Sacrifice one permanent `id` controlled by `who` (CR 701.16): emit the
    /// sacrifice/death events (creature-specific ones first, then the generic
    /// `PermanentSacrificed`) and route it to the graveyard, firing dies/LTB
    /// triggers. Shared by the auto-pick path and the interactive
    /// `SacrificePending` resume so both behave identically.
    pub(crate) fn sacrifice_one(&mut self, id: CardId, who: usize, events: &mut Vec<GameEvent>) {
        let is_creature = self
            .battlefield_find(id)
            .map(|c| c.definition.is_creature())
            .unwrap_or(false);
        if is_creature {
            // Cache snapshot for AnotherOfYours / death-matters triggers.
            if let Some(c) = self.battlefield_find(id) {
                self.died_card_snapshots.insert(id, c.clone());
            }
            events.push(GameEvent::CreatureSacrificed { card_id: id, who });
            events.push(GameEvent::CreatureDied { card_id: id });
        }
        events.push(GameEvent::PermanentSacrificed { card_id: id, who });
        self.players_sacrificed_this_resolution.insert(who);
        let mut die_evs = self.remove_to_graveyard_with_triggers(id);
        events.append(&mut die_evs);
    }

    /// The permanents `player` controls that satisfy a sacrifice `filter`
    /// (CR 701.16), as a list of ids. `source` is the effect's source for
    /// filter predicates that key off it ("another", self-exclusion).
    pub(crate) fn sacrifice_candidates(
        &self,
        player: usize,
        filter: &crate::card::SelectionRequirement,
        source: Option<CardId>,
    ) -> Vec<CardId> {
        self.battlefield
            .iter()
            .filter(|c| {
                c.controller == player
                    && self.evaluate_requirement_static(
                        filter,
                        &Target::Permanent(c.id),
                        player,
                        source,
                    )
            })
            .map(|c| c.id)
            .collect()
    }

    /// Pair each battlefield `CardId` with its display name, for building a
    /// `Decision::ChooseCards` candidate list. Ids not on the battlefield are
    /// dropped.
    pub(crate) fn card_id_names(&self, ids: &[CardId]) -> Vec<(CardId, String)> {
        ids.iter()
            .filter_map(|id| {
                self.battlefield_find(*id)
                    .map(|c| (*id, c.definition.name.to_string()))
            })
            .collect()
    }

    /// Auto-pick `count` permanents for `player` to sacrifice from `candidates`
    /// (the AutoDecider heuristic used for bots / tests / multi-sacrifice):
    /// non-source first (honours "another"), then tokens, then by mana value,
    /// then by power. With `greatest`, the mana-value (or `by_power`) key is
    /// reversed to pick the *largest* match (Soul Shatter / Crackling Doom).
    pub(crate) fn auto_pick_sacrifices(
        &self,
        candidates: &[CardId],
        count: usize,
        source: Option<CardId>,
        greatest: bool,
        by_power: bool,
    ) -> Vec<CardId> {
        let mut cands: Vec<&CardInstance> = candidates
            .iter()
            .filter_map(|id| self.battlefield_find(*id))
            .collect();
        cands.sort_by_key(|c| {
            let mv = c.definition.cost.cmc() as i64;
            let pw = c.power() as i64;
            // For `greatest`, negate the primary metric so the largest sorts
            // first; ties fall back to lowest power (matching prior behaviour).
            let primary = if greatest {
                if by_power { -pw } else { -mv }
            } else if by_power {
                pw
            } else {
                mv
            };
            // The "tokens first" key only applies to the cheapest-pick
            // (`Sacrifice`) heuristic — the greatest-pick (`SacrificeGreatestMV`)
            // never used it, so keep it constant there to preserve ordering.
            let token_key = if greatest { false } else { !c.is_token };
            (Some(c.id) == source, token_key, primary, pw)
        });
        cands.into_iter().take(count).map(|c| c.id).collect()
    }

    fn run_effect(
        &mut self,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        match effect {
            Effect::Noop => Ok(()),

            Effect::Seq(steps) => {
                for (idx, step) in steps.iter().enumerate() {
                    self.run_effect(step, ctx, events)?;
                    // A child effect signalled suspension — append the rest of
                    // this Seq after whatever remaining effects it already
                    // saved (the child's own continuation runs first).
                    if let Some((_, _, remaining)) = self.suspend_signal.as_mut() {
                        let tail: Vec<Effect> = steps[idx + 1..].to_vec();
                        if !tail.is_empty() {
                            let carried = std::mem::replace(remaining, Effect::Noop);
                            let mut combined = Vec::with_capacity(tail.len() + 1);
                            if !matches!(carried, Effect::Noop) {
                                combined.push(carried);
                            }
                            combined.extend(tail);
                            *remaining = Effect::seq(combined);
                        }
                        return Ok(());
                    }
                }
                Ok(())
            }

            Effect::If { cond, then, else_ } => {
                if self.evaluate_predicate(cond, ctx) {
                    self.run_effect(then, ctx, events)
                } else {
                    self.run_effect(else_, ctx, events)
                }
            }

            Effect::ForEach { selector, body } => {
                let entities = self.resolve_selector(selector, ctx);
                for ent in entities {
                    let mut sub_ctx = ctx.clone();
                    sub_ctx.trigger_source = Some(ent);
                    self.run_effect(body, &sub_ctx, events)?;
                }
                Ok(())
            }

            Effect::Repeat { count, body } => {
                let n = self.evaluate_value(count, ctx).max(0);
                for _ in 0..n {
                    self.run_effect(body, ctx, events)?;
                }
                Ok(())
            }

            // CR 705 — Flip a coin `count` times; for each flip ask the
            // controller's decider for heads/tails and dispatch to
            // `on_heads` / `on_tails`. AutoDecider flips a real random coin;
            // ScriptedDecider scripts per-flip outcomes for tests.
            Effect::FlipCoin { count, on_heads, on_tails } => {
                // CR 705.3 — `flip_one_coin` applies Krark's-Thumb advantage
                // (replay + treat as heads if any replay is heads).
                let n = self.evaluate_value(count, ctx).max(0);
                for _ in 0..n {
                    if self.flip_one_coin(ctx.controller) {
                        // CR 705.1 — the controller won this flip; fire any
                        // "Whenever you win a coin flip" triggers.
                        events.push(GameEvent::CoinFlipWon { player: ctx.controller });
                        self.run_effect(on_heads, ctx, events)?;
                    } else {
                        // CR 705.1 — the controller lost this flip.
                        events.push(GameEvent::CoinFlipLost { player: ctx.controller });
                        self.run_effect(on_tails, ctx, events)?;
                    }
                }
                Ok(())
            }

            Effect::ManaClash { opponent } => {
                // CR 705.2 — both players flip each round; whoever is tails
                // takes 1 damage; repeat until both heads on the same flip.
                let me = ctx.controller;
                let Some(opp) = self.resolve_selector(opponent, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    })
                else {
                    return Ok(());
                };
                let src = ctx.source;
                // Cap the loop so a degenerate decider can't spin forever.
                for _ in 0..1000 {
                    let my_flip = self.flip_one_coin(me);
                    let opp_flip = self.flip_one_coin(opp);
                    // CR 705.1 — each player who won this round's flip.
                    if my_flip { events.push(GameEvent::CoinFlipWon { player: me }); }
                    else { events.push(GameEvent::CoinFlipLost { player: me }); }
                    if opp_flip { events.push(GameEvent::CoinFlipWon { player: opp }); }
                    else { events.push(GameEvent::CoinFlipLost { player: opp }); }
                    if !my_flip {
                        self.deal_damage_to_from(EntityRef::Player(me), 1, src, events);
                    }
                    if !opp_flip {
                        self.deal_damage_to_from(EntityRef::Player(opp), 1, src, events);
                    }
                    if my_flip && opp_flip {
                        break;
                    }
                }
                Ok(())
            }

            // CR 706 — Roll `count` N-sided dice. For each die, ask the
            // controller's decider for `Decision::DieRoll { sides }`
            // (which returns `DecisionAnswer::DieRoll(rolled)`), then
            // walk `results` and run the FIRST arm whose [low, high]
            // range covers `rolled`. AutoDecider returns the midpoint;
            // ScriptedDecider can script any face. Mirrors FlipCoin's
            // resolver shape.
            Effect::RollDie { sides, count, modifier, reroll_at_most, results, on_doubles } => {
                let n = self.evaluate_value(count, ctx).max(0);
                let sides = (*sides).max(2);
                // CR 706.6 — "whenever a player rolls one or more dice" fires
                // once for the whole roll. Emitted before the per-die result
                // dispatch so the roll-count is known to listeners.
                if n > 0 {
                    events.push(GameEvent::DiceRolled {
                        player: ctx.controller,
                        count: n as u32,
                    });
                }
                // CR 706.2 — the flat result modifier applied to every die
                // this resolution.
                let modifier = self.evaluate_value(modifier, ctx);
                let roll_one = |s: &mut Self| -> u8 {
                    match s.decider.decide(&crate::decision::Decision::DieRoll {
                        player: ctx.controller,
                        sides,
                    }) {
                        crate::decision::DecisionAnswer::DieRoll(face) => face.clamp(1, sides),
                        // Decider returned the wrong shape — degrade to
                        // midpoint rather than panicking. Real clients
                        // should always return DieRoll(n).
                        _ => (sides as u32).div_ceil(2) as u8,
                    }
                };
                // CR 706.5 — track natural faces to detect "doubles".
                let mut naturals: Vec<u8> = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    let mut natural = roll_one(self);
                    // CR 706.2b — reroll a low natural result exactly once.
                    if *reroll_at_most > 0 && natural <= *reroll_at_most {
                        natural = roll_one(self);
                    }
                    naturals.push(natural);
                    // CR 706.2 — add the modifier, flooring the modified
                    // result at 1 (a die result is never reduced below 1).
                    // The result may exceed `sides`, letting a top "N+"
                    // arm catch boosted rolls.
                    let rolled = (natural as i32 + modifier).max(1).min(u8::MAX as i32) as u8;
                    // CR 706.3a — first matching arm fires. If no arm
                    // matches the roll, the die has no result-table
                    // effect (per CR 706.3a "If the result was in this
                    // range, [effect]" — silent on out-of-range rolls).
                    if let Some((_, _, effect)) =
                        results.iter().find(|(lo, hi, _)| rolled >= *lo && rolled <= *hi)
                    {
                        // CR 706.4 — expose the rolled face to the arm's
                        // effect via `Value::LastDieRoll`.
                        self.last_die_roll = rolled;
                        self.run_effect(effect, ctx, events)?;
                    }
                }
                // CR 706.5 — "if any of the dice show the same number"
                // (doubles): fires once after the per-die dispatch when two
                // or more natural faces match.
                if let Some(doubles_effect) = on_doubles {
                    let mut sorted = naturals.clone();
                    sorted.sort_unstable();
                    if sorted.windows(2).any(|w| w[0] == w[1]) {
                        self.run_effect(doubles_effect, ctx, events)?;
                    }
                }
                Ok(())
            }

            Effect::ChooseMode(modes) => {
                // CR 702.41c — an entwined spell runs every mode in order;
                // each target-bearing mode owns the next target slot. A
                // suspending mode carries the remaining modes as its
                // continuation (same shape as Seq above).
                if ctx.entwined {
                    let mut slot = 0usize;
                    for (idx, m) in modes.iter().enumerate() {
                        if m.requires_target() {
                            let mut sub_ctx = ctx.clone();
                            sub_ctx.targets =
                                ctx.targets.get(slot).cloned().into_iter().collect();
                            slot += 1;
                            self.run_effect(m, &sub_ctx, events)?;
                        } else {
                            self.run_effect(m, ctx, events)?;
                        }
                        if let Some((_, _, remaining)) = self.suspend_signal.as_mut() {
                            let tail: Vec<Effect> = modes[idx + 1..].to_vec();
                            if !tail.is_empty() {
                                let carried = std::mem::replace(remaining, Effect::Noop);
                                let mut combined = Vec::with_capacity(tail.len() + 1);
                                if !matches!(carried, Effect::Noop) {
                                    combined.push(carried);
                                }
                                combined.extend(tail);
                                *remaining = Effect::seq(combined);
                            }
                            return Ok(());
                        }
                    }
                    return Ok(());
                }
                let idx = if ctx.mode == crate::game::types::MODE_PICK_DEFERRED {
                    // A wants_ui trigger controller's pick was deferred from
                    // push time to resolution (see `pick_trigger_mode`):
                    // consume the stashed modal answer, or suspend for one.
                    use crate::decision::{Decision, DecisionAnswer};
                    let decision = Decision::ChooseMode {
                        source: ctx.source.unwrap_or(CardId(0)),
                        num_modes: modes.len(),
                        mode_texts: modes.iter().map(|m| m.effect_short_text()).collect(),
                    };
                    match self.stashed_resolution_answer.take() {
                        Some(DecisionAnswer::Mode(i)) => i.min(modes.len().saturating_sub(1)),
                        Some(_) => 0,
                        None if self.players[ctx.controller].wants_ui => {
                            self.suspend_signal = Some((
                                decision,
                                PendingEffectState::ModeAnswerPending { num_modes: modes.len() },
                                effect.clone(),
                            ));
                            return Ok(());
                        }
                        // Defensive: a deferred pick reaching a non-UI
                        // controller (control changed mid-stack) falls back
                        // to the decider.
                        None => match self.decider.decide(&decision) {
                            DecisionAnswer::Mode(i) => i.min(modes.len().saturating_sub(1)),
                            _ => 0,
                        },
                    }
                } else {
                    ctx.mode
                };
                if let Some(m) = modes.get(idx) {
                    self.run_effect(m, ctx, events)
                } else {
                    Err(GameError::ModeOutOfBounds(idx))
                }
            }

            Effect::ChooseN { picks, modes } => {
                // CR 700.2d — let the controller's decider choose `picks.len()`
                // distinct modes. `AutoDecider` returns the card's `picks`
                // default unchanged; a UI/scripted decider can pick any
                // distinct set. The answer is sanitised: out-of-range and
                // duplicate indices are dropped, and it falls back to `picks`
                // if the result is empty.
                use crate::decision::{Decision, DecisionAnswer};
                let source = ctx.source.unwrap_or(CardId(0));
                let decision = Decision::ChooseModes {
                    source,
                    num_modes: modes.len(),
                    count: picks.len(),
                    default: picks.clone(),
                    mode_texts: modes.iter().map(|m| m.effect_short_text()).collect(),
                };
                // Stash-and-rerun suspend: a `wants_ui` controller answers
                // through the client modal; the resume re-runs this effect
                // with the sanitised answer stashed.
                let answer = match self.stashed_resolution_answer.take() {
                    Some(a) => a,
                    None if self.players[ctx.controller].wants_ui => {
                        self.suspend_signal = Some((
                            decision,
                            PendingEffectState::ModesAnswerPending { num_modes: modes.len() },
                            effect.clone(),
                        ));
                        return Ok(());
                    }
                    None => self.decider.decide(&decision),
                };
                let in_range = |v: &[u8]| -> Vec<u8> {
                    v.iter().copied().filter(|&i| (i as usize) < modes.len()).collect()
                };
                let run: Vec<u8> = match answer {
                    // The unmodified card default is honored verbatim, so
                    // "you may choose the same mode more than once" cards
                    // (Eldrazi / Mystic Confluence, picks=[1,1,1]) run the
                    // repeats their author intended.
                    DecisionAnswer::Modes(ref v) if v == picks => in_range(picks),
                    // A real override is sanitised per CR 700.2d: drop
                    // out-of-range + duplicate indices.
                    DecisionAnswer::Modes(v) => {
                        let mut seen = Vec::new();
                        for &i in &v {
                            if (i as usize) < modes.len() && !seen.contains(&i) {
                                seen.push(i);
                            }
                        }
                        if seen.is_empty() { in_range(picks) } else { seen }
                    }
                    _ => in_range(picks),
                };
                // Each target-bearing mode owns one cast-time target slot,
                // assigned by its position among the target-bearing modes in
                // the card's *default* `picks` (not the run order). Keying off
                // the default picks keeps the slot stable so the targets the
                // caster supplied (validated against the same default-picks
                // ordering in `target_filter_for_slot_in_mode`) line up at
                // resolution even when the decider runs only a subset. This
                // lets "choose one or both" spells whose modes target
                // different things — Steal the Show's mode 0 (target player) +
                // mode 1 (target creature) — run each with the right target.
                // Single-target-mode cards (the Strixhaven Commands) are
                // unaffected since only one picked mode consumes slot 0.
                let mut slot_of_mode: std::collections::HashMap<u8, usize> =
                    std::collections::HashMap::new();
                let mut next_slot = 0usize;
                for &i in picks {
                    if modes.get(i as usize).is_some_and(|m| m.requires_target()) {
                        slot_of_mode.entry(i).or_insert_with(|| {
                            let s = next_slot;
                            next_slot += 1;
                            s
                        });
                    }
                }
                for &i in &run {
                    if let Some(m) = modes.get(i as usize) {
                        if m.requires_target() {
                            let slot = slot_of_mode.get(&i).copied().unwrap_or(0);
                            let mut sub_ctx = ctx.clone();
                            sub_ctx.targets =
                                ctx.targets.get(slot).cloned().into_iter().collect();
                            self.run_effect(m, &sub_ctx, events)?;
                        } else {
                            self.run_effect(m, ctx, events)?;
                        }
                    }
                }
                Ok(())
            }

            Effect::Escalate { modes, cost } => {
                use crate::decision::{Decision, DecisionAnswer};
                let source = ctx.source.unwrap_or(CardId(0));
                // The cast-time `mode` is the base (always-chosen) mode; the
                // decider may escalate to additional distinct modes. AutoDecider
                // keeps just the base — no escalate cost, mirroring a plain
                // modal cast (so existing single-mode casts are unaffected).
                let base = (ctx.mode as u8).min(modes.len().saturating_sub(1) as u8);
                let decision = Decision::ChooseModes {
                    source,
                    num_modes: modes.len(),
                    count: modes.len(),
                    default: vec![base],
                    mode_texts: modes.iter().map(|m| m.effect_short_text()).collect(),
                };
                let answer = match self.stashed_resolution_answer.take() {
                    Some(a) => a,
                    None if self.players[ctx.controller].wants_ui => {
                        self.suspend_signal = Some((
                            decision,
                            PendingEffectState::ModesAnswerPending { num_modes: modes.len() },
                            effect.clone(),
                        ));
                        return Ok(());
                    }
                    None => self.decider.decide(&decision),
                };
                let chosen: Vec<u8> = match answer {
                    DecisionAnswer::Modes(v) => v,
                    _ => vec![base],
                };
                // Sanitize: distinct, in range; the base mode always runs.
                let mut run: Vec<u8> = vec![base];
                for &i in &chosen {
                    if (i as usize) < modes.len() && !run.contains(&i) {
                        run.push(i);
                    }
                }
                // Cap chosen modes by how many escalate costs the controller can
                // actually pay (a "discard a card" cost is bounded by hand size;
                // other costs are uncapped) — escalate stays honest.
                let affordable_extra = match &**cost {
                    Effect::Discard { who: Selector::You, amount, random: false } => {
                        let per = self.evaluate_value(amount, ctx).max(0) as usize;
                        self.players[ctx.controller].hand.len().checked_div(per).unwrap_or(usize::MAX)
                    }
                    _ => usize::MAX,
                };
                run.truncate(1 + affordable_extra);
                // Pay the escalate cost once per mode beyond the first.
                for _ in 1..run.len() {
                    self.run_effect(cost, ctx, events)?;
                }
                // Per-mode target slots assigned in run order: the first
                // target-bearing chosen mode reads ctx.targets[0], the next
                // ctx.targets[1] (additional_targets), and so on.
                let mut next_slot = 0usize;
                for &i in &run {
                    if let Some(m) = modes.get(i as usize) {
                        if m.requires_target() {
                            let mut sub_ctx = ctx.clone();
                            sub_ctx.targets =
                                ctx.targets.get(next_slot).cloned().into_iter().collect();
                            next_slot += 1;
                            self.run_effect(m, &sub_ctx, events)?;
                        } else {
                            self.run_effect(m, ctx, events)?;
                        }
                    }
                }
                Ok(())
            }

            Effect::MayDo { description, body } => {
                // Yes/no decision via `Decision::OptionalTrigger`, asked of
                // the *controller* of the effect (`ctx.controller`). A
                // `wants_ui` controller gets the client's yes/no modal via
                // the stash-and-rerun suspend; bots / tests answer through
                // the installed `Decider` (`AutoDecider` declines,
                // `ScriptedDecider` injects `Bool(true)` to exercise the
                // body).
                use crate::decision::{Decision, DecisionAnswer};
                let source = ctx.source.unwrap_or(CardId(0));
                let decision = Decision::OptionalTrigger {
                    source,
                    description: description.clone(),
                };
                let answer = match self.stashed_resolution_answer.take() {
                    Some(a) => a,
                    None if self.players[ctx.controller].wants_ui => {
                        self.suspend_signal = Some((
                            decision,
                            PendingEffectState::MayDoAnswerPending,
                            effect.clone(),
                        ));
                        return Ok(());
                    }
                    None => self.decider.decide(&decision),
                };
                let yes = matches!(answer, DecisionAnswer::Bool(true));
                if yes {
                    self.run_effect(body, ctx, events)?;
                }
                Ok(())
            }

            Effect::Process { count, then } => {
                // Process N exile cards an opponent owns → their graveyards;
                // run `then` only if at least one was processed.
                let opponents = self.opponents_of(ctx.controller);
                let eligible: Vec<CardId> = self
                    .exile
                    .iter()
                    .filter(|c| opponents.contains(&c.owner))
                    .map(|c| c.id)
                    .take(*count as usize)
                    .collect();
                if eligible.is_empty() {
                    return Ok(());
                }
                let source = ctx.source.unwrap_or(CardId(0));
                let mut cursor = 0;
                let Some(yes) = self.ask_seat_bool(
                    &mut cursor,
                    ctx.controller,
                    format!("Process up to {count} card(s) from exile?"),
                    source,
                    effect,
                ) else {
                    return Ok(());
                };
                self.clear_answer_log();
                if !yes {
                    return Ok(());
                }
                for id in eligible {
                    if let Some(pos) = self.exile.iter().position(|c| c.id == id) {
                        let card = self.exile.remove(pos);
                        // CR 614.6 — graveyard-hate redirects apply.
                        self.route_to_graveyard(card, events);
                    }
                }
                self.run_effect(then, ctx, events)?;
                Ok(())
            }

            Effect::IfRevealFromHand { filter, then, else_ } => {
                // Peek at the controller's hand for a card matching `filter`.
                // If any match exists, run `then` (the implicit "yes, I
                // reveal" branch — AutoDecider always accepts since the
                // alternative is the printed downside, e.g. enters-tapped
                // for STX Snarl lands). If no match, run `else_`. The
                // reveal itself is information-only and isn't modeled as
                // a separate game event today; a future enhancement could
                // emit `GameEvent::CardRevealed { player, card_id }` for
                // replay/log purposes.
                let has_match = self.players[ctx.controller]
                    .hand
                    .iter()
                    .any(|c| self.evaluate_requirement_on_card(filter, c, ctx.controller));
                if has_match {
                    self.run_effect(then, ctx, events)?;
                } else {
                    self.run_effect(else_, ctx, events)?;
                }
                Ok(())
            }

            Effect::MayPay {
                description,
                mana_cost,
                body,
                else_,
            } => {
                // Sibling to `MayDo`: ask yes/no, then *attempt* to pay
                // mana. If the controller declines or can't afford the
                // cost the body is skipped and `else_` (if any) runs.
                // The cost is deducted from the controller's already-
                // floated mana pool — we don't auto-tap lands inside an
                // effect (mana abilities aren't activatable mid-resolve
                // by default). A `wants_ui` controller gets the yes/no
                // modal via the seat-routed suspend.
                let source = ctx.source.unwrap_or(CardId(0));
                let mut cursor = 0;
                let Some(yes) = self.ask_seat_bool(
                    &mut cursor,
                    ctx.controller,
                    description.clone(),
                    source,
                    effect,
                ) else {
                    return Ok(());
                };
                self.clear_answer_log();
                let pay = yes
                    && self.players[ctx.controller].mana_pool.pay(mana_cost).is_ok();
                if pay {
                    self.run_effect(body, ctx, events)?;
                } else if let Some(e) = else_ {
                    self.run_effect(e, ctx, events)?;
                }
                Ok(())
            }

            Effect::DealDamage { to, amount } => {
                let amt = self.evaluate_value(amount, ctx).max(0) as u32;
                if amt == 0 { return Ok(()); }
                for ent in self.resolve_selector(to, ctx) {
                    // CR 702.90b — pass the source so infect routes
                    // player-target damage to poison counters.
                    self.deal_damage_to_from(ent, amt, ctx.source, events);
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            // CR 601.2d — deal `total` damage divided among the chosen
            // targets. Targets were collected into `ctx.targets` across
            // slots `0..max_targets` at cast time; the split is decided
            // here so a wants-UI controller / scripted test can choose it
            // (AutoDecider spreads evenly).
            Effect::DealDamageDivided { total, .. } => {
                let amt = self.evaluate_value(total, ctx).max(0) as u32;
                if amt == 0 { return Ok(()); }
                // Only still-present targets receive damage.
                let targets: Vec<Target> = ctx
                    .targets
                    .iter()
                    .filter(|t| match t {
                        Target::Player(p) => *p < self.players.len(),
                        Target::Permanent(id) => self.battlefield_find(*id).is_some(),
                    })
                    .cloned()
                    .collect();
                if targets.is_empty() { return Ok(()); }
                let decision = Decision::DivideDamage {
                    source: ctx.source.unwrap_or(CardId(0)),
                    total: amt,
                    targets: targets.clone(),
                };
                let answer = match self.stashed_resolution_answer.take() {
                    Some(a) => a,
                    None if self.players[ctx.controller].wants_ui => {
                        self.suspend_signal = Some((
                            decision,
                            PendingEffectState::DivisionAnswerPending,
                            effect.clone(),
                        ));
                        return Ok(());
                    }
                    None => self.decider.decide(&decision),
                };
                let mut division = match answer {
                    crate::decision::DecisionAnswer::DamageDivision(v) => v,
                    _ => vec![],
                };
                // Renormalize a malformed answer (wrong length / sum) to an
                // even split so a buggy decider can't drop or duplicate damage.
                if division.len() != targets.len()
                    || division.iter().sum::<u32>() != amt
                {
                    division = crate::decision::even_damage_split(amt, targets.len());
                }
                for (t, n) in targets.iter().zip(division) {
                    if n == 0 { continue; }
                    self.deal_damage_to_from(target_to_entity(t), n, ctx.source, events);
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::SupportCounters { .. } => {
                // CR 701.32 — put one +1/+1 counter on each still-present
                // target creature (up to N targets, supplied per slot).
                use crate::card::CounterType;
                let targets: Vec<CardId> = ctx.targets.iter()
                    .filter_map(|t| match t {
                        Target::Permanent(id) => Some(*id),
                        _ => None,
                    })
                    .filter(|id| self.battlefield_find(*id).is_some_and(|c| c.definition.is_creature()))
                    .collect();
                for id in targets {
                    let ctrl = self.battlefield_find(id).map(|c| c.controller);
                    let mut n = 1u32;
                    if let Some(ctrl) = ctrl {
                        for _ in 0..self.counter_doublers_for(ctrl) {
                            n = n.saturating_mul(2);
                        }
                    }
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.add_counters(CounterType::PlusOnePlusOne, n);
                        events.push(GameEvent::CounterAdded {
                            card_id: id, counter_type: CounterType::PlusOnePlusOne, count: n,
                        });
                    }
                    self.permanents_gained_counter_this_turn.insert(id);
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::Fight { attacker, defender } => {
                // Two creatures simultaneously deal damage equal to
                // their power to each other. Snapshot powers up-front
                // so post-damage stats don't affect the back-swing.
                // Either side resolving to no permanent (target left
                // the battlefield, defender selector matches nothing)
                // no-ops the whole fight, matching MTG's "if either
                // is no longer a creature, no damage is dealt".
                let atk_id = self
                    .resolve_selector(attacker, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Permanent(c) => Some(c),
                        _ => None,
                    });
                let def_id = self
                    .resolve_selector(defender, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Permanent(c) => Some(c),
                        _ => None,
                    });
                let (Some(atk_id), Some(def_id)) = (atk_id, def_id) else {
                    return Ok(());
                };
                // Layer-computed powers; each half carries its own source so
                // the funnel applies lifelink / deathtouch / wither / infect
                // and protection-based prevention (CR 701.12b).
                let atk_power = self.computed_permanent(atk_id).map(|cp| cp.power).unwrap_or(0);
                let def_power = self.computed_permanent(def_id).map(|cp| cp.power).unwrap_or(0);
                if atk_power > 0 {
                    self.deal_damage_to_from(
                        EntityRef::Permanent(def_id),
                        atk_power as u32,
                        Some(atk_id),
                        events,
                    );
                }
                if def_power > 0 {
                    self.deal_damage_to_from(
                        EntityRef::Permanent(atk_id),
                        def_power as u32,
                        Some(def_id),
                        events,
                    );
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::GainLife { who, amount } => {
                let amt = self.evaluate_value(amount, ctx).max(0) as u32;
                if amt == 0 { return Ok(()); }
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        // CR 119.10 — the event carries the APPLIED amount
                        // (post replacement/bonus); a fully-replaced gain
                        // fires no lifegain trigger.
                        let applied = self.adjust_life_applied(p, amt as i32);
                        if applied > 0 {
                            events.push(GameEvent::LifeGained { player: p, amount: applied as u32 });
                        } else if applied < 0 {
                            // Tainted Remedy turned the gain into a loss.
                            events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                        }
                    }
                }
                Ok(())
            }

            Effect::LoseLife { who, amount } => {
                let amt = self.evaluate_value(amount, ctx).max(0) as u32;
                if amt == 0 { return Ok(()); }
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        let applied = self.adjust_life_applied(p, -(amt as i32));
                        if applied < 0 {
                            events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                        }
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::DoubleLife { who } => {
                // CR 701.10d — gain life equal to current total (20 → 40).
                let seats: Vec<usize> = self
                    .resolve_selector(who, ctx)
                    .into_iter()
                    .filter_map(|e| if let EntityRef::Player(p) = e { Some(p) } else { None })
                    .collect();
                for p in seats {
                    let life = self.players[p].life;
                    if life <= 0 { continue; }
                    self.adjust_life(p, life);
                    // adjust_life may be capped (e.g. "can't gain life"); emit
                    // the actual gain.
                    let gained = (self.players[p].life - life).max(0);
                    if gained > 0 {
                        events.push(GameEvent::LifeGained { player: p, amount: gained as u32 });
                    }
                }
                Ok(())
            }

            Effect::LoseHalfLife { who, rounded_up } => {
                // Per-player: each loses half of their *own* total.
                let seats: Vec<usize> = self
                    .resolve_selector(who, ctx)
                    .into_iter()
                    .filter_map(|e| if let EntityRef::Player(p) = e { Some(p) } else { None })
                    .collect();
                for p in seats {
                    let life = self.players[p].life.max(0);
                    let amt = if *rounded_up { (life + 1) / 2 } else { life / 2 } as u32;
                    if amt == 0 { continue; }
                    let applied = self.adjust_life_applied(p, -(amt as i32));
                    if applied < 0 {
                        events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::MillHalf { who, rounded_up } => {
                // Per-player: each mills half of their *own* library.
                let seats: Vec<usize> = self
                    .resolve_selector(who, ctx)
                    .into_iter()
                    .filter_map(|e| if let EntityRef::Player(p) = e { Some(p) } else { None })
                    .collect();
                for p in seats {
                    let lib = self.players[p].library.len();
                    let half = if *rounded_up { lib.div_ceil(2) } else { lib / 2 };
                    let n = self.mill_count_for(p, half);
                    for _ in 0..n {
                        if self.players[p].library.is_empty() { break; }
                        let card = self.players[p].library.remove(0);
                        let cid = card.id;
                        if !self.route_to_graveyard(card, events) {
                            events.push(GameEvent::CardMilled { player: p, card_id: cid });
                        }
                        self.last_moved_cards.push(cid);
                    }
                }
                Ok(())
            }

            Effect::DiscardHalf { who, rounded_up } => {
                // Per-player: each discards half of their *own* hand (pick-first,
                // matching the random-discard bot harness in `Effect::Discard`).
                let seats: Vec<usize> = self
                    .resolve_selector(who, ctx)
                    .into_iter()
                    .filter_map(|e| if let EntityRef::Player(p) = e { Some(p) } else { None })
                    .collect();
                for p in seats {
                    let hand = self.players[p].hand.len();
                    let n = if *rounded_up { hand.div_ceil(2) } else { hand / 2 };
                    for _ in 0..n {
                        let Some(cid) = self.players[p].hand.first().map(|c| c.id) else { break };
                        self.discard_card(p, cid, events);
                    }
                }
                Ok(())
            }

            Effect::SacrificeHalf { who, filter, rounded_up } => {
                // Per-player: each sacrifices half of the permanents they
                // control matching `filter` (weakest/cheapest first, like
                // `Effect::Sacrifice`'s AutoDecider heuristic).
                let seats: Vec<usize> = self
                    .resolve_selector(who, ctx)
                    .into_iter()
                    .filter_map(|e| if let EntityRef::Player(p) = e { Some(p) } else { None })
                    .collect();
                for p in seats {
                    let mut candidates: Vec<&CardInstance> = self
                        .battlefield
                        .iter()
                        .filter(|c| c.controller == p)
                        .filter(|c| {
                            let t = Target::Permanent(c.id);
                            self.evaluate_requirement_static(filter, &t, p, ctx.source)
                        })
                        .collect();
                    candidates.sort_by_key(|c| (!c.is_token, c.definition.cost.cmc(), c.power()));
                    let total = candidates.len();
                    let n = if *rounded_up { total.div_ceil(2) } else { total / 2 };
                    let ids: Vec<CardId> = candidates.into_iter().take(n).map(|c| c.id).collect();
                    for id in ids {
                        let is_creature = self
                            .battlefield_find(id)
                            .map(|c| c.definition.is_creature())
                            .unwrap_or(false);
                        if is_creature {
                            if let Some(c) = self.battlefield_find(id) {
                                self.died_card_snapshots.insert(id, c.clone());
                            }
                            events.push(GameEvent::CreatureSacrificed { card_id: id, who: p });
                            events.push(GameEvent::CreatureDied { card_id: id });
                        }
                        events.push(GameEvent::PermanentSacrificed { card_id: id, who: p });
                        let mut die_evs = self.remove_to_graveyard_with_triggers(id);
                        events.append(&mut die_evs);
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::SetLifeTotal { who, amount } => {
                let new_total = self.evaluate_value(amount, ctx);
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        // CR 119.7 — setting a life total is a gain/loss of
                        // the difference, so replacements (cannot-gain,
                        // Tainted Remedy) apply through the life funnel.
                        let delta = new_total - self.effective_life(p);
                        let applied = self.adjust_life_applied(p, delta);
                        if applied > 0 {
                            events.push(GameEvent::LifeGained { player: p, amount: applied as u32 });
                        } else if applied < 0 {
                            events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                        }
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::ExchangeLifeTotals { a, b } => {
                // CR 701.12c — capture both previous totals, then move each
                // player to the other's previous total.
                let pa = self.resolve_selector(a, ctx).into_iter().find_map(|e| {
                    if let EntityRef::Player(p) = e { Some(p) } else { None }
                });
                let pb = self.resolve_selector(b, ctx).into_iter().find_map(|e| {
                    if let EntityRef::Player(p) = e { Some(p) } else { None }
                });
                if let (Some(pa), Some(pb)) = (pa, pb)
                    && pa != pb
                {
                    let la = self.effective_life(pa);
                    let lb = self.effective_life(pb);
                    for (p, new_total, old) in [(pa, lb, la), (pb, la, lb)] {
                        // CR 119.7 — the exchange is a gain/loss of the
                        // difference; route it through the life funnel.
                        let applied = self.adjust_life_applied(p, new_total - old);
                        if applied > 0 {
                            events.push(GameEvent::LifeGained { player: p, amount: applied as u32 });
                        } else if applied < 0 {
                            events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                        }
                    }
                    let mut sba = self.check_state_based_actions();
                    events.append(&mut sba);
                }
                Ok(())
            }

            Effect::LifeGainLockThisTurn { who } => {
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].cannot_gain_life_this_turn = true;
                    }
                }
                Ok(())
            }

            Effect::GrantSpellsUncounterableThisTurn { who } => {
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].spells_uncounterable_this_turn = true;
                    }
                }
                Ok(())
            }

            Effect::MakeSpellUncounterable { what } => {
                use crate::game::types::StackItem;
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_card_id() else { continue };
                    for item in self.stack.iter_mut().rev() {
                        if let StackItem::Spell { card, uncounterable, .. } = item
                            && card.id == cid
                        {
                            *uncounterable = true;
                            break;
                        }
                    }
                }
                Ok(())
            }

            Effect::CantCastNoncreatureThisTurn { who } => {
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].cant_cast_noncreature_this_turn = true;
                    }
                }
                Ok(())
            }

            Effect::Drain { from, to, amount } => {
                let amt = self.evaluate_value(amount, ctx).max(0) as u32;
                if amt == 0 { return Ok(()); }
                for ent in self.resolve_selector(from, ctx) {
                    if let EntityRef::Player(p) = ent {
                        let applied = self.adjust_life_applied(p, -(amt as i32));
                        if applied < 0 {
                            events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                        }
                    }
                }
                for ent in self.resolve_selector(to, ctx) {
                    if let EntityRef::Player(p) = ent {
                        let applied = self.adjust_life_applied(p, amt as i32);
                        if applied > 0 {
                            events.push(GameEvent::LifeGained { player: p, amount: applied as u32 });
                        } else if applied < 0 {
                            events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                        }
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::AddEnergy(amount) => {
                let amt = self.evaluate_value(amount, ctx).max(0) as u32;
                if amt == 0 { return Ok(()); }
                let p = ctx.controller;
                self.players[p].energy = self.players[p].energy.saturating_add(amt);
                events.push(GameEvent::EnergyGained { player: p, amount: amt });
                Ok(())
            }

            Effect::PayEnergy { amount, then } => {
                let p = ctx.controller;
                if self.players[p].energy >= *amount {
                    self.players[p].energy -= *amount;
                    self.run_effect(then, ctx, events)?;
                }
                Ok(())
            }

            Effect::PayEnergyOrElse { amount, otherwise } => {
                // CR 107.16 — "sacrifice/return unless you pay {E}…". Pay when
                // able (AutoDecider keeps the permanent); otherwise resolve the
                // fallback (sacrifice / bounce).
                let p = ctx.controller;
                if self.players[p].energy >= *amount {
                    self.players[p].energy -= *amount;
                } else {
                    self.run_effect(otherwise, ctx, events)?;
                }
                Ok(())
            }

            Effect::PayManaOrElse { mana_cost, otherwise } => {
                // Mana sibling of PayEnergyOrElse — pay from the floating
                // pool when able (AutoDecider keeps the permanent),
                // otherwise resolve the fallback (sacrifice / bounce).
                let p = ctx.controller;
                if self.players[p].mana_pool.pay(mana_cost).is_err() {
                    self.run_effect(otherwise, ctx, events)?;
                }
                Ok(())
            }

            Effect::ExileTopMayPayEnergyToCast { energy } => {
                use crate::card::Zone;
                use crate::decision::{Decision, DecisionAnswer};
                use crate::effect::ZoneDest;
                let p = ctx.controller;
                let Some(top_id) = self.players[p].library.first().map(|c| c.id) else {
                    return Ok(());
                };
                self.move_card_to(top_id, &ZoneDest::Exile, ctx, events);
                // CR 107.16 — only offer the pay-and-cast if the controller
                // can actually afford the energy.
                if self.players[p].energy < *energy {
                    return Ok(());
                }
                let src = ctx.source.unwrap_or(CardId(0));
                let answer = self.decider.decide(&Decision::OptionalTrigger {
                    source: src,
                    description: format!(
                        "Pay {{E}}×{energy} to cast the exiled card without paying its mana cost?"
                    ),
                });
                if !matches!(answer, DecisionAnswer::Bool(true)) {
                    return Ok(());
                }
                self.players[p].energy -= *energy;
                let card_def = self.find_card_anywhere(top_id).map(|c| c.definition.clone());
                if let Some(card_def) = card_def {
                    let auto_target =
                        self.auto_target_for_effect_avoiding(&card_def.effect, p, Some(top_id));
                    let cast_events = self.cast_card_for_free(
                        p,
                        top_id,
                        Zone::Exile,
                        auto_target,
                        vec![],
                        None,
                        None,
                        false,
                    )?;
                    events.extend(cast_events);
                }
                Ok(())
            }

            Effect::Draw { who, amount } => {
                let n = self.evaluate_value(amount, ctx).max(0) as usize;
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        // CR 121.2b — a per-turn draw cap truncates the draw.
                        let n = match self.draw_cap_for(p) {
                            Some(cap) => {
                                let remaining = (cap as usize)
                                    .saturating_sub(self.players[p].cards_drawn_this_turn as usize);
                                n.min(remaining)
                            }
                            None => n,
                        };
                        for _ in 0..n {
                            // `draw_one` applies the Dredge replacement
                            // (CR 702.52) before falling back to a normal
                            // draw; AutoDecider declines dredge by default.
                            if !self.draw_one(p, events) {
                                // CR 104.3c (or the Lab-Man win override);
                                // the SBA pass decides game-over.
                                self.lose_to_empty_draw(p);
                                return Ok(());
                            }
                        }
                    }
                }
                Ok(())
            }

            Effect::Learn { who } => {
                // CR 701.45 — Learn. Reveal a Lesson from the sideboard into
                // hand, or discard a card to draw a card. When no Lesson is
                // available (no sideboard configured), fall back to the
                // legacy `Draw 1` approximation so existing games are
                // unaffected.
                use crate::card::SpellSubtype;
                use crate::decision::{Decision, DecisionAnswer, LearnChoice};
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                // Retriever Phoenix (CR replacement): a card in p's graveyard
                // with `MayReturnFromGraveyardInsteadOfLearn` lets p return it
                // to the battlefield instead of learning.
                let returnable: Option<crate::card::CardId> = self.players[p]
                    .graveyard
                    .iter()
                    .find(|c| c.definition.static_abilities.iter().any(|sa| matches!(
                        sa.effect,
                        crate::effect::StaticEffect::MayReturnFromGraveyardInsteadOfLearn
                    )))
                    .map(|c| c.id);
                if let Some(card_id) = returnable {
                    let name = self.players[p]
                        .graveyard
                        .iter()
                        .find(|c| c.id == card_id)
                        .map(|c| c.definition.name.to_string())
                        .unwrap_or_default();
                    let answer = self.decider.decide(&Decision::OptionalTrigger {
                        source: card_id,
                        description: format!("Return {name} to the battlefield instead of learning?"),
                    });
                    if matches!(answer, DecisionAnswer::Bool(true)) {
                        self.move_card_to(
                            card_id,
                            &ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: false },
                            ctx,
                            events,
                        );
                        return Ok(());
                    }
                }
                let lessons: Vec<(crate::card::CardId, String)> = self.players[p]
                    .sideboard
                    .iter()
                    .filter(|c| {
                        c.definition.subtypes.spell_subtypes.contains(&SpellSubtype::Lesson)
                    })
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                if lessons.is_empty() {
                    if !self.draw_one(p, events) {
                        self.lose_to_empty_draw(p);
                    }
                    return Ok(());
                }
                let hand: Vec<(crate::card::CardId, String)> = self.players[p]
                    .hand
                    .iter()
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                let decision = Decision::Learn { player: p, lessons, hand };
                // UI players answer asynchronously: suspend resolution and
                // surface the decision; `apply_pending_effect_answer` resumes
                // via `PendingEffectState::LearnPending`.
                if self.players[p].wants_ui {
                    self.suspend_signal = Some((
                        decision,
                        crate::game::types::PendingEffectState::LearnPending { player: p },
                        Effect::Noop,
                    ));
                    return Ok(());
                }
                let choice = match self.decider.decide(&decision) {
                    DecisionAnswer::Learn(c) => c,
                    _ => LearnChoice::Decline,
                };
                self.apply_learn_choice(p, choice, events);
                Ok(())
            }

            Effect::Discard { who, amount, random } => {
                use crate::decision::Decision;
                let n = self.evaluate_value(amount, ctx).max(0) as usize;
                if n == 0 { return Ok(()); }
                let seats: Vec<usize> = self
                    .resolve_selector(who, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                for (i, p) in seats.iter().copied().enumerate() {
                    // Tamiyo: opponents' spells/abilities can't make you discard.
                    if !self.same_team(p, ctx.controller)
                        && self.player_cant_be_made_to_discard(p)
                    {
                        continue;
                    }
                    if *random {
                        // Random-discard semantics: deterministic-pick-first
                        // for the in-process tests; a real client would seed
                        // an RNG, but the bot harness doesn't care which
                        // card gets dumped. The discard itself (zone move +
                        // CardDiscarded + counters + Madness, CR 702.35) is
                        // centralized in `discard_card`.
                        for _ in 0..n {
                            let Some(cid) = self.players[p].hand.first().map(|c| c.id) else {
                                break;
                            };
                            self.discard_card(p, cid, events);
                        }
                        continue;
                    }
                    // Player-chosen discard: surface a `Decision::Discard` so
                    // the discarding player picks N cards from their own
                    // hand. Reuses the `DiscardChosenPending` resume context
                    // (the resume logic only cares about which player loses
                    // the chosen cards, not who picked them).
                    if self.players[p].hand.is_empty() { continue; }
                    let candidates: Vec<(crate::card::CardId, String)> = self
                        .players[p]
                        .hand
                        .iter()
                        .map(|c| (c.id, c.definition.name.to_string()))
                        .collect();
                    let count = (candidates.len().min(n)) as u32;
                    let decision = Decision::Discard {
                        player: p,
                        count,
                        hand: candidates,
                    };
                    let pending = PendingEffectState::DiscardChosenPending { target_player: p };
                    if self.players[p].wants_ui {
                        // Suspend for this seat; the continuation re-runs
                        // the discard for every seat not yet processed so a
                        // symmetric discard doesn't stop at the first human.
                        let rest = per_seat_continuation(&seats[i + 1..], |q| Effect::Discard {
                            who: Selector::Player(crate::effect::PlayerRef::Seat(q)),
                            amount: crate::effect::Value::Const(n as i32),
                            random: *random,
                        });
                        self.suspend_signal = Some((decision, pending, rest));
                        return Ok(());
                    }
                    let answer = self.decider.decide(&decision);
                    let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                    events.append(&mut applied);
                }
                Ok(())
            }

            Effect::Mill { who, amount } => {
                let base = self.evaluate_value(amount, ctx).max(0) as usize;
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        let n = self.mill_count_for(p, base);
                        for _ in 0..n {
                            if self.players[p].library.is_empty() { break; }
                            let card = self.players[p].library.remove(0);
                            let cid = card.id;
                            if !self.route_to_graveyard(card, events) {
                                events.push(GameEvent::CardMilled { player: p, card_id: cid });
                            }
                            // Stash the milled id so a follow-up
                            // `Selector::LastMoved` in the same Seq can
                            // target the milled card (Tablet of
                            // Discovery, Ark of Hunger's "you may play
                            // that card this turn" rider).
                            self.last_moved_cards.push(cid);
                        }
                    }
                }
                Ok(())
            }

            Effect::MillUntilLands { who, lands } => self.resolve_mill_until_lands(who, lands, ctx, events),


            Effect::MillTwoRepeatSharedColor { who } => self.resolve_mill_two_repeat_shared_color(who, ctx, events),


            Effect::ExileTopOfLibrary { who, amount, link_to_source, face_down } => {
                // CR 702.115 Ingest etc. — like Mill but routes to exile.
                let n = self.evaluate_value(amount, ctx).max(0) as usize;
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        for _ in 0..n {
                            if self.players[p].library.is_empty() { break; }
                            let card = self.players[p].library.remove(0);
                            let cid = card.id;
                            self.place_card_in_dest(card, p, &ZoneDest::Exile, events);
                            self.last_moved_cards.push(cid);
                            if (*link_to_source || *face_down)
                                && let Some(c) = self.exile.iter_mut().find(|c| c.id == cid)
                            {
                                if *link_to_source {
                                    c.exiled_with = ctx.source;
                                }
                                c.face_down = *face_down;
                            }
                        }
                    }
                }
                Ok(())
            }

            Effect::SetNoMaxHandSize { who } => {
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].max_hand_size = None;
                    }
                }
                Ok(())
            }

            Effect::SetMaxHandSize { who, size } => {
                let n = self.evaluate_value(size, ctx).max(0) as usize;
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].max_hand_size = Some(n);
                    }
                }
                Ok(())
            }

            Effect::DiscardAnyNumber { who } => {
                use crate::decision::Decision;
                let seats: Vec<usize> = self
                    .resolve_selector(who, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                for (i, p) in seats.iter().copied().enumerate() {
                    if self.players[p].hand.is_empty() { continue; }
                    let candidates: Vec<(crate::card::CardId, String)> = self
                        .players[p]
                        .hand
                        .iter()
                        .map(|c| (c.id, c.definition.name.to_string()))
                        .collect();
                    // "Any number" — count = hand size; the decider's
                    // `Discard(picked_ids)` answer can return 0..=hand.len()
                    // entries. AutoDecider picks 0 by default (it returns
                    // `iter().take(count).take(0)` semantics — but our
                    // AutoDecider uses `count` directly so we tell it 0 by
                    // surfacing `count: 0` with the full hand). For UI seats
                    // the full hand is surfaced so the player can pick any
                    // subset.
                    let count = if self.players[p].wants_ui {
                        candidates.len() as u32
                    } else {
                        0
                    };
                    let decision = Decision::Discard {
                        player: p,
                        count,
                        hand: candidates,
                    };
                    let pending = PendingEffectState::DiscardChosenPending { target_player: p };
                    if self.players[p].wants_ui {
                        let rest = per_seat_continuation(&seats[i + 1..], |q| {
                            Effect::DiscardAnyNumber {
                                who: Selector::Player(crate::effect::PlayerRef::Seat(q)),
                            }
                        });
                        self.suspend_signal = Some((decision, pending, rest));
                        return Ok(());
                    }
                    let answer = self.decider.decide(&decision);
                    let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                    events.append(&mut applied);
                }
                Ok(())
            }

            Effect::Scry { who, amount }
            | Effect::Surveil { who, amount }
            | Effect::LookAtTop { who, amount }
            | Effect::RearrangeTop { who, amount } => {
                use crate::decision::Decision;
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(amount, ctx).max(0) as usize;
                // CR 701.22b: "If a player is instructed to scry 0, no
                // scry event occurs. Abilities that trigger whenever a
                // player scries won't trigger." (Same wording covers
                // Surveil 0 via 701.42 by reference.) An instruction
                // of `n == 0` short-circuits at the top so no decision
                // / event flows downstream.
                if n == 0 {
                    return Ok(());
                }
                let peek: Vec<(CardId, String)> = self.players[p]
                    .library
                    .iter()
                    .take(n)
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                let actual = peek.len();
                // Per CR 701.22a, if the library has fewer cards than
                // requested, the player looks at and may rearrange the
                // available cards — the scry instruction still
                // executes. Only return early when there are literally
                // no cards to peek at (e.g. empty library + Scry N>0
                // is still a vacuous-but-real scry; we model that by
                // proceeding to emit the bookkeeping path). For
                // simplicity if `actual == 0` we still skip the
                // decision (no cards to reorder) but acknowledge the
                // event happened; future scry-counting payoffs would
                // need an explicit event emission here.
                if actual == 0 {
                    return Ok(());
                }

                let scry_mode = match effect {
                    Effect::Surveil { .. } => crate::decision::ScryMode::Surveil,
                    Effect::RearrangeTop { .. } => crate::decision::ScryMode::Rearrange,
                    _ => crate::decision::ScryMode::Scry,
                };
                let decision = Decision::Scry { player: p, cards: peek.clone(), mode: scry_mode };
                let pending_state = match effect {
                    Effect::Surveil { .. } => PendingEffectState::SurveilPeeked { count: actual, player: p },
                    Effect::RearrangeTop { .. } => PendingEffectState::RearrangePeeked { count: actual, player: p },
                    _ => PendingEffectState::ScryPeeked { count: actual, player: p },
                };

                // If the acting player wants UI input, suspend — the outer
                // resolver will convert `suspend_signal` into `pending_decision`
                // and `submit_decision` will apply the answer + run any
                // remaining Seq effects.
                if self.players[p].wants_ui {
                    self.suspend_signal = Some((decision, pending_state, Effect::Noop));
                    return Ok(());
                }

                // Otherwise resolve synchronously via the decider (bot / tests).
                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending_state, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::Monstrosity { n } => {
                // CR 701.31 — if the source isn't monstrous, add N +1/+1
                // counters and mark it monstrous (firing BecameMonstrous).
                let base = self.evaluate_value(n, ctx).max(0) as u32;
                let Some(src) = ctx.source else { return Ok(()); };
                let already = self.battlefield_find(src).map(|c| c.monstrous).unwrap_or(true);
                if already {
                    return Ok(());
                }
                // CR 614.16 — counter doublers scale monstrosity counters too.
                let ctrl = self.battlefield_find(src).map(|c| c.controller);
                let mut count = base;
                if let Some(ctrl) = ctrl {
                    for _ in 0..self.counter_doublers_for(ctrl) {
                        count = count.saturating_mul(2);
                    }
                }
                if let Some(c) = self.battlefield_find_mut(src) {
                    c.monstrous = true;
                    if count > 0 {
                        c.add_counters(CounterType::PlusOnePlusOne, count);
                        events.push(GameEvent::CounterAdded {
                            card_id: src,
                            counter_type: CounterType::PlusOnePlusOne,
                            count,
                        });
                        self.permanents_gained_counter_this_turn.insert(src);
                    }
                }
                events.push(GameEvent::BecameMonstrous { card_id: src });
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::Goad { what } => {
                // CR 701.38 — add the resolving controller to each target
                // creature's goaded_by list. The grant expires when the
                // goader's next turn begins (cleared in `do_untap`).
                let goader = ctx.controller;
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    if let Some(c) = self.battlefield_find_mut(cid)
                        && c.definition.is_creature()
                        && !c.goaded_by.contains(&goader)
                    {
                        c.goaded_by.push(goader);
                    }
                }
                Ok(())
            }

            Effect::Suspect { what } => {
                // CR 701.60 — mark each target creature as suspected (menace +
                // can't block, injected as computed keywords).
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    if let Some(c) = self.battlefield_find_mut(cid)
                        && c.definition.is_creature()
                    {
                        c.suspected = true;
                    }
                }
                Ok(())
            }

            Effect::Detain { what } => {
                // CR 701.35 — stamp each target permanent with the detaining
                // player so it can't attack/block/activate until the detainer's
                // next turn (cleared in `do_untap` / turn start).
                let detainer = ctx.controller;
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        c.detained_by = Some(detainer);
                    }
                }
                Ok(())
            }

            Effect::Fateseal { who, amount } => {
                use crate::decision::{Decision, DecisionAnswer};
                // CR 701.29 — look at the top N of the targeted opponent's
                // library; the controller may put any of them on the bottom.
                let Some(opp) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(amount, ctx).max(0) as usize;
                if n == 0 { return Ok(()); }
                let peek: Vec<(CardId, String)> = self.players[opp]
                    .library
                    .iter()
                    .take(n)
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                if peek.is_empty() { return Ok(()); }
                let source = ctx.source.unwrap_or(CardId(0));
                // Decided by the fatesealer (ctx.controller), not the library's
                // owner. A wants_ui controller is served the same inline pick
                // (a dedicated suspend path is a TODO.md follow-up).
                let answer = self.decider.decide(&Decision::ChooseCards {
                    source,
                    prompt: "Fateseal: put which cards on the bottom?".to_string(),
                    candidates: peek,
                    min: 0,
                    max: n as u32,
                });
                if let DecisionAnswer::Cards(to_bottom) = answer {
                    for cid in to_bottom {
                        if let Some(pos) = self.players[opp].library.iter().position(|c| c.id == cid) {
                            let card = self.players[opp].library.remove(pos);
                            self.players[opp].library.push(card);
                        }
                    }
                }
                Ok(())
            }

            Effect::DigToHandLoseLife { count, life_per_card } => {
                use crate::decision::{Decision, DecisionAnswer};
                let p = ctx.controller;
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 { return Ok(()); }
                let per = self.evaluate_value(life_per_card, ctx).max(0);
                let top: Vec<(CardId, String)> = self.players[p]
                    .library
                    .iter()
                    .take(n)
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                if top.is_empty() { return Ok(()); }
                let source = ctx.source.unwrap_or(CardId(0));
                // Controller chooses any subset to keep (a `wants_ui` seat is
                // served the same inline pick; a dedicated suspend path is a
                // TODO.md follow-up).
                let answer = self.decider.decide(&Decision::ChooseCards {
                    source,
                    prompt: "Put which cards into your hand?".to_string(),
                    candidates: top.clone(),
                    min: 0,
                    max: top.len() as u32,
                });
                let chosen: Vec<CardId> = match answer {
                    DecisionAnswer::Cards(v) => v,
                    _ => vec![],
                };
                let mut taken = 0i32;
                // Take chosen → hand (top-down), rest of the revealed set →
                // graveyard. Putting a card into hand this way is NOT a draw
                // (CR 121.5 — no CardDrawn event, no draw-trigger fire), and
                // the leftovers honor graveyard replacements (CR 614.6).
                for (cid, _) in &top {
                    if chosen.contains(cid) {
                        if let Some(pos) = self.players[p].library.iter().position(|c| c.id == *cid) {
                            let card = self.players[p].library.remove(pos);
                            self.players[p].hand.push(card);
                            taken += 1;
                        }
                    } else if let Some(pos) = self.players[p].library.iter().position(|c| c.id == *cid) {
                        let card = self.players[p].library.remove(pos);
                        self.route_to_graveyard(card, events);
                    }
                }
                let life = taken * per;
                if life > 0 {
                    let applied = self.adjust_life_applied(p, -life);
                    if applied < 0 {
                        events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                    }
                }
                Ok(())
            }

            Effect::DiscardUnlessKind { who, count, instead } => {
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
                let pick = self.players[p]
                    .hand
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(instead, c, p))
                    .min_by_key(|c| c.definition.cost.cmc())
                    .map(|c| c.id);
                if let Some(id) = pick {
                    self.discard_card(p, id, events);
                    return Ok(());
                }
                self.run_effect(
                    &Effect::Discard {
                        who: Selector::Player(who.clone()),
                        amount: count.clone(),
                        random: false,
                    },
                    ctx,
                    events,
                )
            }

            Effect::RevealTopToHandLoseLifeRepeat => {
                use crate::decision::{Decision, DecisionAnswer};
                let p = ctx.controller;
                let source = ctx.source.unwrap_or(CardId(0));
                while let Some(top) = self.players[p].library.first() {
                    let (name, mv) = (top.definition.name, top.definition.cost.cmc() as i32);
                    let answer = self.decider.decide(&Decision::OptionalTrigger {
                        source,
                        description: format!("Reveal the top card ({name}, lose {mv} life)?"),
                    });
                    if !matches!(answer, DecisionAnswer::Bool(true)) {
                        break;
                    }
                    let card = self.players[p].library.remove(0);
                    self.players[p].hand.push(card);
                    if mv > 0 {
                        let applied = self.adjust_life_applied(p, -mv);
                        if applied < 0 {
                            events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                        }
                    }
                }
                Ok(())
            }

            Effect::Provoke { what } => {
                // CR 702.39 — untap the target creature and force it to block
                // the source attacker this combat if able.
                let source = ctx.source;
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    if let Some(c) = self.battlefield_find_mut(cid)
                        && c.definition.is_creature()
                    {
                        c.tapped = false;
                        c.must_block = source;
                    }
                }
                Ok(())
            }

            Effect::Explore { who } => {
                // CR 701.40 — each resolved permanent explores: reveal the
                // top card of its controller's library; a land goes to hand,
                // otherwise the permanent gets a +1/+1 counter (the revealed
                // nonland card stays on top — the optional graveyard choice
                // is collapsed). An empty library still explores (counter,
                // no card). Emits `Explored` so payoff triggers can fire.
                for ent in self.resolve_selector(who, ctx) {
                    // `as_card_id` so a reanimated explorer reached via
                    // `Selector::LastMoved` (an `EntityRef::Card`) is honored;
                    // the `battlefield_find` below still gates to permanents.
                    let Some(cid) = ent.as_card_id() else { continue };
                    let Some(controller) =
                        self.battlefield_find(cid).map(|c| c.controller)
                    else {
                        continue;
                    };
                    let top = self.players[controller].library.first();
                    let is_land = top.map(|c| c.definition.is_land());
                    if let Some(name) = top.map(|c| c.definition.name) {
                        events.push(GameEvent::TopCardRevealed {
                            player: controller,
                            card_name: name,
                            is_land: is_land.unwrap_or(false),
                        });
                    }
                    if is_land == Some(true) {
                        let card = self.players[controller].library.remove(0);
                        self.players[controller].hand.push(card);
                    } else {
                        // Nonland revealed (or empty library): +1/+1 counter.
                        // CR 614.1c — Hardened Scales-style doublers apply.
                        let mut n = 1u32;
                        for _ in 0..self.counter_doublers_for(controller) {
                            n = n.saturating_mul(2);
                        }
                        if let Some(c) = self.battlefield_find_mut(cid) {
                            c.add_counters(CounterType::PlusOnePlusOne, n);
                            events.push(GameEvent::CounterAdded {
                                card_id: cid,
                                counter_type: CounterType::PlusOnePlusOne,
                                count: n,
                            });
                            self.permanents_gained_counter_this_turn.insert(cid);
                        }
                    }
                    events.push(GameEvent::Explored { card_id: cid, controller });
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::AddMana { who, pool } => {
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                // Unwrap a spend-restriction wrapper. The inner payload
                // resolves exactly as normal; each colored pip it produces
                // is tagged with `restriction` so `pay_for_spell` can gate
                // its use. Colorless pips stay unrestricted (no card needs
                // restricted colorless mana).
                let (pool, restriction) = match pool {
                    ManaPayload::Restricted(inner, r) => (inner.as_ref(), Some(*r)),
                    // Cavern of Souls: tag pips with the source's chosen
                    // creature type; no chosen type → unrestricted.
                    ManaPayload::RestrictedToChosenType(inner) => {
                        let chosen = ctx
                            .source
                            .and_then(|cid| self.battlefield_find(cid))
                            .and_then(|c| c.chosen_creature_type);
                        (
                            inner.as_ref(),
                            chosen.map(crate::mana::SpendRestriction::CreatureOfTypeUncounterable),
                        )
                    }
                    // Unclaimed Territory: same chosen-type tagging without
                    // the uncounterable rider.
                    ManaPayload::RestrictedToChosenTypePlain(inner) => {
                        let chosen = ctx
                            .source
                            .and_then(|cid| self.battlefield_find(cid))
                            .and_then(|c| c.chosen_creature_type);
                        (
                            inner.as_ref(),
                            chosen.map(crate::mana::SpendRestriction::CreatureOfType),
                        )
                    }
                    other => (other, None),
                };
                // CR 701.10f / 614.5 — mana-production multiplier (Mana
                // Reflection, Nyxbloom Ancient). 1× outside a tapped mana
                // ability; 0 (serde default) also reads as 1×.
                let mult = self.mana_production_multiplier.max(1);
                let add_one = |state: &mut Self, p: usize, c: Color| match restriction {
                    Some(r) => state.players[p].mana_pool.add_restricted(c, mult, r),
                    None => state.players[p].mana_pool.add(c, mult),
                };
                match pool {
                    ManaPayload::Colors(colors) => {
                        for c in colors {
                            add_one(self, p, *c);
                            events.push(GameEvent::ManaAdded { player: p, color: *c, source: ctx.source });
                        }
                    }
                    ManaPayload::Colorless(v) => {
                        let n = self.evaluate_value(v, ctx).max(0) as u32;
                        for _ in 0..n {
                            self.players[p].mana_pool.add_colorless(mult);
                            events.push(GameEvent::ColorlessManaAdded { player: p, source: ctx.source });
                        }
                    }
                    ManaPayload::OfColor(color, v) => {
                        // Fixed-color, value-scaled mana adder. No player
                        // choice — just N pips of `color`. Used by
                        // power-scaled mana abilities (Topiary Lecturer,
                        // Rofellos when promoted to per-Forest scaling).
                        let n = self.evaluate_value(v, ctx).max(0) as u32;
                        for _ in 0..n {
                            add_one(self, p, *color);
                            events.push(GameEvent::ManaAdded { player: p, color: *color, source: ctx.source });
                        }
                    }
                    ManaPayload::ChosenColorOfSource => {
                        // Coldsteel Heart-style: tap for the color stamped at
                        // ETB. Falls back to colorless if none was chosen.
                        let source = ctx.source.unwrap_or(CardId(0));
                        match self.battlefield_find(source).and_then(|c| c.chosen_color) {
                            Some(c) => {
                                add_one(self, p, c);
                                events.push(GameEvent::ManaAdded { player: p, color: c, source: ctx.source });
                            }
                            None => {
                                self.players[p].mana_pool.add_colorless(mult);
                                events.push(GameEvent::ColorlessManaAdded { player: p, source: ctx.source });
                            }
                        }
                    }
                    ManaPayload::AnyOneColor(v) => {
                        // ONE color choice; add `n` mana of that color (Black
                        // Lotus, Birds of Paradise, Mox Diamond, etc.).
                        let n = self.evaluate_value(v, ctx).max(0) as u32;
                        if n == 0 { return Ok(()); }
                        let source = ctx.source.unwrap_or(CardId(0));
                        let legal = vec![
                            Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
                        ];
                        if self.players[p].wants_ui {
                            // Surface a `ChooseColor` decision to the UI.
                            // After the player answers, `apply_pending_effect_answer`
                            // adds `n` mana of the chosen color.
                            self.suspend_signal = Some((
                                crate::decision::Decision::ChooseColor {
                                    source,
                                    legal,
                                },
                                PendingEffectState::AnyOneColorPending { player: p, count: n, restriction },
                                Effect::Noop,
                            ));
                            return Ok(());
                        }
                        let answer = self.decider.decide(
                            &crate::decision::Decision::ChooseColor {
                                source,
                                legal,
                            },
                        );
                        let color = match answer {
                            crate::decision::DecisionAnswer::Color(c) => c,
                            _ => Color::White,
                        };
                        for _ in 0..n {
                            add_one(self, p, color);
                            events.push(GameEvent::ManaAdded { player: p, color, source: ctx.source });
                        }
                    }
                    ManaPayload::ImprintedCardColor => {
                        // Chrome Mox — add one mana of a color of the card
                        // imprinted on the source (exile, exiled_with == source).
                        let source = ctx.source.unwrap_or(CardId(0));
                        let legal: Vec<Color> = self
                            .exile
                            .iter()
                            .find(|c| c.exiled_with == Some(source))
                            .map(|c| c.definition.cost.colors())
                            .unwrap_or_default();
                        if legal.is_empty() {
                            return Ok(()); // no imprint / colorless → no mana
                        }
                        if self.players[p].wants_ui {
                            // Surface the color choice to the UI (mirrors the
                            // AnyOneColor arm); resume adds 1 of the chosen color.
                            self.suspend_signal = Some((
                                crate::decision::Decision::ChooseColor { source, legal },
                                PendingEffectState::AnyOneColorPending { player: p, count: 1, restriction },
                                Effect::Noop,
                            ));
                            return Ok(());
                        }
                        let answer = self.decider.decide(
                            &crate::decision::Decision::ChooseColor { source, legal: legal.clone() },
                        );
                        let color = match answer {
                            crate::decision::DecisionAnswer::Color(c) if legal.contains(&c) => c,
                            _ => legal[0],
                        };
                        add_one(self, p, color);
                        events.push(GameEvent::ManaAdded { player: p, color, source: ctx.source });
                    }
                    ManaPayload::DevotionOfChosenColor => {
                        // Nykthos — choose a color, then add mana of that
                        // color equal to your devotion to it (CR 700.5).
                        let source = ctx.source.unwrap_or(CardId(0));
                        let legal = vec![
                            Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
                        ];
                        if self.players[p].wants_ui {
                            self.suspend_signal = Some((
                                crate::decision::Decision::ChooseColor { source, legal },
                                PendingEffectState::DevotionColorPending { player: p },
                                Effect::Noop,
                            ));
                            return Ok(());
                        }
                        let answer = self.decider.decide(
                            &crate::decision::Decision::ChooseColor { source, legal },
                        );
                        let color = match answer {
                            crate::decision::DecisionAnswer::Color(c) => c,
                            _ => Color::White,
                        };
                        let n = self.devotion_to(p, &[color]).max(0) as u32;
                        for _ in 0..n {
                            self.players[p].mana_pool.add(color, mult);
                            events.push(GameEvent::ManaAdded { player: p, color, source: ctx.source });
                        }
                    }
                    ManaPayload::AnyColorAmongLegendaries => {
                        // Mox Amber — union of colors among legendary
                        // creatures/planeswalkers you control; no mana if none.
                        use crate::card::Supertype;
                        let mut legal: Vec<Color> = Vec::new();
                        for c in self.battlefield.iter().filter(|c| {
                            c.controller == p
                                && c.definition.supertypes.contains(&Supertype::Legendary)
                                && (c.definition.is_creature() || c.definition.is_planeswalker())
                        }) {
                            for col in c.definition.cost.colors() {
                                if !legal.contains(&col) {
                                    legal.push(col);
                                }
                            }
                        }
                        if !legal.is_empty() {
                            let color = if legal.len() == 1 {
                                legal[0]
                            } else {
                                match self.decider.decide(
                                    &crate::decision::Decision::ChooseColor {
                                        source: ctx.source.unwrap_or(CardId(0)),
                                        legal: legal.clone(),
                                    },
                                ) {
                                    crate::decision::DecisionAnswer::Color(c)
                                        if legal.contains(&c) => c,
                                    _ => legal[0],
                                }
                            };
                            add_one(self, p, color);
                            events.push(GameEvent::ManaAdded { player: p, color, source: ctx.source });
                        }
                    }
                    ManaPayload::AnyColorOpponentCouldProduce
                    | ManaPayload::AnyColorYouCouldProduce => {
                        // Fellwar Stone (opponent) / Star Compass (self) —
                        // scan the relevant side's battlefield for basic-typed
                        // lands and build the legal-color set from those land
                        // types. Falls back to colorless if none (so the
                        // activation produces *something* — matches the
                        // engine's "never silently no-op" convention for
                        // mana abilities).
                        use crate::card::LandType;
                        let own_side =
                            matches!(pool, ManaPayload::AnyColorYouCouldProduce);
                        let mut legal: Vec<Color> = Vec::new();
                        let push_unique = |c: Color, v: &mut Vec<Color>| {
                            if !v.contains(&c) { v.push(c); }
                        };
                        for opp in self.battlefield.iter()
                            .filter(|c| (c.controller == p) == own_side)
                        {
                            for lt in &opp.definition.subtypes.land_types {
                                match lt {
                                    LandType::Plains => push_unique(Color::White, &mut legal),
                                    LandType::Island => push_unique(Color::Blue, &mut legal),
                                    LandType::Swamp => push_unique(Color::Black, &mut legal),
                                    LandType::Mountain => push_unique(Color::Red, &mut legal),
                                    LandType::Forest => push_unique(Color::Green, &mut legal),
                                    _ => {} // Non-basic land types (Desert,
                                            // Gate, Locus, etc.) don't produce
                                            // a fixed color.
                                }
                            }
                        }
                        if legal.is_empty() {
                            self.players[p].mana_pool.add_colorless(mult);
                            events.push(GameEvent::ColorlessManaAdded { player: p, source: ctx.source });
                        } else {
                            let source = ctx.source.unwrap_or(CardId(0));
                            let answer = self.decider.decide(
                                &crate::decision::Decision::ChooseColor {
                                    source,
                                    legal: legal.clone(),
                                },
                            );
                            let color = match answer {
                                crate::decision::DecisionAnswer::Color(c)
                                    if legal.contains(&c) => c,
                                _ => legal[0],
                            };
                            add_one(self, p, color);
                            events.push(GameEvent::ManaAdded { player: p, color, source: ctx.source });
                        }
                    }
                    ManaPayload::AnyColors(v) => {
                        // N independent color choices (one per pip). Currently
                        // resolves synchronously via the installed decider — a
                        // UI prompt per pip would require a multi-step pending
                        // state and isn't needed by any catalog card today.
                        let n = self.evaluate_value(v, ctx).max(0) as u32;
                        let source = ctx.source.unwrap_or(CardId(0));
                        let legal = vec![
                            Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
                        ];
                        for _ in 0..n {
                            let answer = self.decider.decide(&crate::decision::Decision::ChooseColor {
                                source,
                                legal: legal.clone(),
                            });
                            let color = match answer {
                                crate::decision::DecisionAnswer::Color(c) => c,
                                _ => Color::White,
                            };
                            add_one(self, p, color);
                            events.push(GameEvent::ManaAdded { player: p, color, source: ctx.source });
                        }
                    }
                    ManaPayload::Restricted(..)
                    | ManaPayload::RestrictedToChosenType(..)
                    | ManaPayload::RestrictedToChosenTypePlain(..) => {
                        // One unwrap above already stripped the restriction;
                        // no card nests wrappers, so a doubly-wrapped payload
                        // is malformed — ignore it rather than panic.
                    }
                    ManaPayload::OfColors(colors, v) => {
                        // N pips, each chosen from the restricted palette
                        // (Culling Ritual: {B} or {G} per permanent destroyed).
                        let n = self.evaluate_value(v, ctx).max(0) as u32;
                        let source = ctx.source.unwrap_or(CardId(0));
                        let fallback = colors.first().copied().unwrap_or(Color::White);
                        for _ in 0..n {
                            let answer = self.decider.decide(&crate::decision::Decision::ChooseColor {
                                source,
                                legal: colors.clone(),
                            });
                            let color = match answer {
                                crate::decision::DecisionAnswer::Color(c)
                                    if colors.contains(&c) => c,
                                _ => fallback,
                            };
                            add_one(self, p, color);
                            events.push(GameEvent::ManaAdded { player: p, color, source: ctx.source });
                        }
                    }
                }
                Ok(())
            }

            Effect::Destroy { what } | Effect::DestroyNoRegen { what } => {
                // CR 701.15g — `DestroyNoRegen` ("can't be regenerated")
                // bypasses regeneration shields; everything else (the
                // Indestructible check, Shield-counter replacement) is
                // identical to plain `Destroy`.
                let no_regen = matches!(effect, Effect::DestroyNoRegen { .. });
                let entities = self.resolve_selector(what, ctx);
                for ent in entities {
                    if let Some(cid) = ent.as_permanent_id() {
                        let indestructible = self.battlefield_find(cid)
                            .map(|c| c.is_indestructible())
                            .unwrap_or(true);
                        if indestructible {
                            continue;
                        }
                        // CR 122.1c — Shield counters create a single
                        // replacement: "If this permanent would be
                        // destroyed as the result of an effect, instead
                        // remove a shield counter from it."
                        let has_shield = self
                            .battlefield_find(cid)
                            .map(|c| c.counter_count(crate::card::CounterType::Shield) > 0)
                            .unwrap_or(false);
                        if has_shield {
                            if let Some(c) = self.battlefield_find_mut(cid) {
                                c.remove_counters(crate::card::CounterType::Shield, 1);
                                // No 0-count residue (CR 700.9 IsModified).
                                if c.counter_count(crate::card::CounterType::Shield) == 0 {
                                    c.counters.remove(&crate::card::CounterType::Shield);
                                }
                            }
                            continue;
                        }
                        // CR 701.15 — regeneration shield replaces destruction:
                        // remove a shield, tap the permanent, remove it from
                        // combat, and heal marked damage instead of dying.
                        // Skipped entirely for `DestroyNoRegen` (CR 701.15g).
                        if !no_regen
                            && self
                                .battlefield_find(cid)
                                .map(|c| c.regeneration_shields > 0)
                                .unwrap_or(false)
                        {
                            self.apply_regeneration(cid);
                            continue;
                        }
                        // CR 702.89 — umbra armor replaces destruction.
                        if self.apply_umbra_armor(cid, events) {
                            continue;
                        }
                        let is_creature = self.battlefield_find(cid)
                            .map(|c| c.definition.is_creature())
                            .unwrap_or(false);
                        if is_creature {
                            // Cache the dying card's snapshot for
                            // AnotherOfYours-scope triggers and type
                            // filter predicates (token deaths in
                            // particular vanish before dispatch).
                            if let Some(c) = self.battlefield_find(cid) {
                                self.died_card_snapshots.insert(cid, c.clone());
                            }
                            events.push(GameEvent::CreatureDied { card_id: cid });
                        }
                        let mut dies = self.remove_to_graveyard_with_triggers(cid);
                        events.append(&mut dies);
                        self.permanents_destroyed_this_resolution =
                            self.permanents_destroyed_this_resolution.saturating_add(1);
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::Regenerate { what } => {
                // CR 701.15 — add one regeneration shield per resolved
                // permanent. The shield is consumed by the next destruction
                // (SBA lethal damage / Effect::Destroy) this turn.
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid)
                    {
                        c.regeneration_shields = c.regeneration_shields.saturating_add(1);
                    }
                }
                Ok(())
            }

            Effect::ExileIfWouldDieThisTurn { what } => {
                // Install an until-end-of-turn death replacement on each
                // resolved permanent. `remove_from_battlefield_to_graveyard_raw`
                // consults `dies_to_exile_eot` and redirects to exile — the
                // same path the finality counter uses; the set is cleared at
                // cleanup.
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id() {
                        self.dies_to_exile_eot.insert(cid);
                    }
                }
                Ok(())
            }

            Effect::GrantMiracle { what, cost } => {
                // Stamp an until-end-of-turn may-play permission plus the
                // miracle alt-cost on each resolved card, so the controller
                // may cast it this turn for `cost` (Lorehold, the Historian).
                let granter = ctx.controller;
                let granted_turn = self.turn_number;
                for ent in self.resolve_selector(what, ctx) {
                    if let EntityRef::Card(cid) = ent
                        && let Some(card) = self.find_card_anywhere_mut(cid)
                    {
                        card.may_play_until = Some(crate::card::MayPlayPermission {
                            player: granter,
                            granted_turn,
                            duration: crate::card::MayPlayDuration::EndOfThisTurn,
                            exile_after: false,
                        });
                        card.granted_alt_cast_cost_eot = Some(cost.clone());
                    }
                }
                Ok(())
            }

            Effect::GrantFlashbackThisTurn { what } => {
                // Grant until-end-of-turn flashback (cost = the card's own
                // mana cost) to each resolved graveyard card, so it can be
                // recast this turn via the normal flashback path (pay the
                // cost, exile on resolve). Cleared at cleanup.
                for ent in self.resolve_selector(what, ctx) {
                    if let EntityRef::Card(cid) = ent
                        && let Some(card) = self.find_card_anywhere_mut(cid)
                    {
                        let fb_cost = card.definition.cost.clone();
                        card.granted_flashback_eot = Some(fb_cost);
                    }
                }
                Ok(())
            }

            Effect::Exile { what } => {
                // Exile accepts both `EntityRef::Permanent` (battlefield)
                // and `EntityRef::Card` (any other zone). Battlefield exits
                // emit `PermanentExiled` and walk through the standard
                // remove-from-battlefield path so leaves-the-battlefield
                // hooks fire; non-battlefield zones (graveyards, hand,
                // exile→exile re-routes) just relocate via `move_card_to`.
                for ent in self.resolve_selector(what, ctx) {
                    match ent {
                        // A `Permanent` ref whose object already left the
                        // battlefield (died mid-trigger — Kaldra Compleat's
                        // "exile that creature") is exiled from wherever it
                        // went instead of silently no-oping.
                        EntityRef::Permanent(cid)
                            if !self.battlefield.iter().any(|c| c.id == cid) =>
                        {
                            self.move_card_to(cid, &ZoneDest::Exile, ctx, events);
                        }
                        EntityRef::Permanent(cid) => {
                            self.remove_from_battlefield_to_exile(cid);
                            // Bump the controller's per-turn exile tally
                            // for Ennis-style "if a card was put into
                            // exile this turn" payoffs.
                            if ctx.controller < self.players.len() {
                                self.players[ctx.controller].cards_exiled_this_turn =
                                    self.players[ctx.controller].cards_exiled_this_turn.saturating_add(1);
                            }
                            events.push(GameEvent::PermanentExiled { card_id: cid });
                        }
                        EntityRef::Card(cid) => {
                            self.move_card_to(cid, &ZoneDest::Exile, ctx, events);
                        }
                        _ => {}
                    }
                }
                Ok(())
            }

            Effect::ExileReturnNextEndStep { what } => {
                // Exile each resolved permanent now; register a per-card
                // NextEndStep delayed trigger that returns it under its owner's
                // control with an extra +1/+1 (creature) or loyalty (PW)
                // counter. Semester's End.
                use crate::card::CounterType;
                use crate::game::types::{DelayedKind, DelayedTrigger};
                let source = ctx.source.unwrap_or(CardId(0));
                for ent in self.resolve_selector(what, ctx) {
                    let EntityRef::Permanent(cid) = ent else { continue; };
                    let is_pw = self
                        .battlefield_find(cid)
                        .is_some_and(|c| c.definition.is_planeswalker());
                    self.remove_from_battlefield_to_exile(cid);
                    if ctx.controller < self.players.len() {
                        self.players[ctx.controller].cards_exiled_this_turn =
                            self.players[ctx.controller].cards_exiled_this_turn.saturating_add(1);
                    }
                    events.push(GameEvent::PermanentExiled { card_id: cid });
                    let counter_kind = if is_pw {
                        CounterType::Loyalty
                    } else {
                        CounterType::PlusOnePlusOne
                    };
                    let body = Effect::Seq(vec![
                        Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                        Effect::AddCounter {
                            what: Selector::Target(0),
                            kind: counter_kind,
                            amount: crate::effect::Value::Const(1),
                        },
                    ]);
                    self.delayed_triggers.push(DelayedTrigger {
                        controller: ctx.controller,
                        source,
                        kind: DelayedKind::NextEndStep,
                        effect: body,
                        target: Some(Target::Permanent(cid)),
                        bound_token: None,
                        fires_once: true,
                    });
                }
                Ok(())
            }

            Effect::ExileTaggedWithSource { what } => {
                let source = ctx.source;
                for ent in self.resolve_selector(what, ctx) {
                    let cid = match ent {
                        EntityRef::Permanent(c) | EntityRef::Card(c) => c,
                        _ => continue,
                    };
                    // Route battlefield exits through the LTB path; anything
                    // in another zone (the common graveyard-hate case) just
                    // relocates via `move_card_to`.
                    if self.battlefield.iter().any(|c| c.id == cid) {
                        self.remove_from_battlefield_to_exile(cid);
                        events.push(GameEvent::PermanentExiled { card_id: cid });
                    } else {
                        self.move_card_to(cid, &ZoneDest::Exile, ctx, events);
                    }
                    if let Some(c) = self.exile.iter_mut().find(|c| c.id == cid) {
                        c.exiled_with = source;
                    }
                }
                Ok(())
            }

            Effect::ExileAnyNumberFromGraveyards { filter } => {
                use crate::decision::{Decision, DecisionAnswer};
                // Gather every graveyard card matching `filter`, across all
                // players. Each is offered as a `ChooseCards` candidate.
                let mut candidates: Vec<(CardId, String)> = Vec::new();
                for p in 0..self.players.len() {
                    for c in &self.players[p].graveyard {
                        if self.evaluate_requirement_static(
                            filter, &Target::Permanent(c.id), p, ctx.source,
                        ) {
                            candidates.push((c.id, c.definition.name.to_string()));
                        }
                    }
                }
                if candidates.is_empty() { return Ok(()); }
                let max = candidates.len() as u32;
                let source = ctx.source.unwrap_or(CardId(0));
                let answer = self.decider.decide(&Decision::ChooseCards {
                    source,
                    prompt: "Exile which cards from graveyards?".to_string(),
                    candidates: candidates.clone(),
                    min: 0,
                    max,
                });
                let valid: std::collections::HashSet<CardId> =
                    candidates.iter().map(|(id, _)| *id).collect();
                let chosen: Vec<CardId> = match answer {
                    DecisionAnswer::Cards(ids) => ids
                        .into_iter()
                        .filter(|id| valid.contains(id))
                        .take(max as usize)
                        .collect(),
                    _ => vec![],
                };
                for cid in chosen {
                    self.move_card_to(cid, &ZoneDest::Exile, ctx, events);
                }
                Ok(())
            }

            Effect::ExileAllGraveyards { filter, opponents_only } => {
                // Rest in Peace ETB — move every (matching) graveyard card
                // to exile (Sanctifier en-Vec passes a color filter;
                // Phyrexian Scriptures III skips the controller's own).
                for p in 0..self.players.len() {
                    if *opponents_only && self.same_team(p, ctx.controller) {
                        continue;
                    }
                    let cards: Vec<CardInstance> =
                        std::mem::take(&mut self.players[p].graveyard);
                    for card in cards {
                        let matches = filter
                            .as_ref()
                            .is_none_or(|f| self.evaluate_requirement_on_card(f, &card, p));
                        if !matches {
                            self.players[p].graveyard.push(card);
                            continue;
                        }
                        let cid = card.id;
                        self.exile.push(card);
                        events.push(GameEvent::PermanentExiled { card_id: cid });
                        self.note_left_graveyard(p, cid, events);
                    }
                }
                Ok(())
            }

            Effect::LivingEnd => {
                // Each player exiles all creature cards from their graveyard…
                let mut returning: Vec<CardId> = Vec::new();
                for p in 0..self.players.len() {
                    let gy = std::mem::take(&mut self.players[p].graveyard);
                    for card in gy {
                        if card.definition.is_creature() {
                            returning.push(card.id);
                            let cid = card.id;
                            self.exile.push(card);
                            events.push(GameEvent::PermanentExiled { card_id: cid });
                            self.note_left_graveyard(p, cid, events);
                        } else {
                            self.players[p].graveyard.push(card);
                        }
                    }
                }
                // …then sacrifices all creatures they control…
                let dying: Vec<(CardId, usize)> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.definition.is_creature())
                    .map(|c| (c.id, c.controller))
                    .collect();
                for (id, who) in dying {
                    // CR 700.4 — the swept board still dies properly: dies
                    // triggers, Persist/Undying, snapshots, tallies.
                    if let Some(c) = self.battlefield_find(id) {
                        self.died_card_snapshots.insert(id, c.clone());
                    }
                    events.push(GameEvent::CreatureSacrificed { card_id: id, who });
                    events.push(GameEvent::CreatureDied { card_id: id });
                    events.push(GameEvent::PermanentSacrificed { card_id: id, who });
                    let mut die = self.remove_to_graveyard_with_triggers(id);
                    events.append(&mut die);
                }
                // …then puts the exiled cards onto the battlefield.
                for id in returning {
                    if let Some(pos) = self.exile.iter().position(|c| c.id == id) {
                        let card = self.exile.remove(pos);
                        let owner = card.owner;
                        self.place_card_in_dest(
                            card,
                            owner,
                            &ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                            events,
                        );
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::ExilePlayerGraveyard { who } => {
                // Go Blank / Ashiok −10 — move graveyards to exile.
                for p in self.resolve_players(who, ctx) {
                    let cards: Vec<CardInstance> = std::mem::take(&mut self.players[p].graveyard);
                    for card in cards {
                        let cid = card.id;
                        self.exile.push(card);
                        events.push(GameEvent::PermanentExiled { card_id: cid });
                        self.note_left_graveyard(p, cid, events);
                    }
                }
                Ok(())
            }

            Effect::ExileHand { who } => {
                for p in self.resolve_players(who, ctx) {
                    let cards: Vec<CardInstance> = std::mem::take(&mut self.players[p].hand);
                    for card in cards {
                        let cid = card.id;
                        self.exile.push(card);
                        events.push(GameEvent::PermanentExiled { card_id: cid });
                    }
                }
                Ok(())
            }

            Effect::ExileSameNameAsTarget { what } => {
                // Crumble to Dust / Surgical Extraction: exile the anchor
                // (battlefield permanent or graveyard card), then exile every
                // same-named card from its owner's graveyard, hand, and
                // library, and shuffle that library.
                let anchor = self.resolve_selector(what, ctx).into_iter().find_map(|e| match e {
                    EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                    _ => None,
                });
                let Some(anchor_id) = anchor else { return Ok(()); };
                let Some((name, owner)) = self
                    .find_card_anywhere(anchor_id)
                    .map(|c| (c.definition.name.to_string(), c.owner))
                else { return Ok(()); };

                if self.battlefield_find(anchor_id).is_some() {
                    self.remove_from_battlefield_to_exile(anchor_id);
                    events.push(GameEvent::PermanentExiled { card_id: anchor_id });
                }

                // Sweep the owner's hidden/graveyard zones for same-named
                // cards. Graveyard exiles get leaves-graveyard bookkeeping.
                let pl = &mut self.players[owner];
                let mut swept: Vec<CardInstance> = Vec::new();
                let mut from_gy: Vec<CardId> = Vec::new();
                for (zi, zone) in [&mut pl.graveyard, &mut pl.hand, &mut pl.library]
                    .into_iter()
                    .enumerate()
                {
                    let mut i = 0;
                    while i < zone.len() {
                        if zone[i].definition.name == name.as_str() {
                            if zi == 0 {
                                from_gy.push(zone[i].id);
                            }
                            swept.push(zone.remove(i));
                        } else {
                            i += 1;
                        }
                    }
                }
                for c in swept {
                    let cid = c.id;
                    self.exile.push(c);
                    events.push(GameEvent::PermanentExiled { card_id: cid });
                }
                for cid in from_gy {
                    self.note_left_graveyard(owner, cid, events);
                }
                use rand::seq::SliceRandom;
                self.players[owner].library.shuffle(&mut rand::rng());
                Ok(())
            }

            Effect::ExileUntilSourceLeaves { what, return_to } => {
                // CR 603.6e — exile the resolved card(s) and link each to
                // the ability's source. When that source leaves the
                // battlefield, `return_linked_exiles` brings them back.
                let Some(source) = ctx.source else {
                    return Ok(());
                };
                for ent in self.resolve_selector(what, ctx) {
                    let cid = match ent {
                        EntityRef::Permanent(cid) => {
                            self.remove_from_battlefield_to_exile(cid);
                            events.push(GameEvent::PermanentExiled { card_id: cid });
                            cid
                        }
                        EntityRef::Card(cid) => {
                            self.move_card_to(cid, &ZoneDest::Exile, ctx, events);
                            cid
                        }
                        _ => continue,
                    };
                    if ctx.controller < self.players.len() {
                        self.players[ctx.controller].cards_exiled_this_turn =
                            self.players[ctx.controller].cards_exiled_this_turn.saturating_add(1);
                    }
                    if let Some(c) = self.exile.iter_mut().find(|c| c.id == cid) {
                        c.exiled_by = Some(crate::card::ExileLink {
                            source,
                            return_to: *return_to,
                        });
                    }
                    // Chainable: a follow-up `GrantMayPlay(LastMoved)` lets
                    // the controller cast the exiled card (Hostage Taker).
                    self.last_moved_cards.push(cid);
                }
                Ok(())
            }

            Effect::Tap { what } => {
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid)
                        && !c.tapped {
                            c.tapped = true;
                            events.push(GameEvent::PermanentTapped { card_id: cid });
                        }
                }
                Ok(())
            }

            Effect::PhaseOut { what } => {
                // CR 702.26 — collect the targeted permanents (and anything
                // attached to them) and move them to the phased-out zone.
                let mut ids: std::collections::HashSet<crate::card::CardId> = self
                    .resolve_selector(what, ctx)
                    .iter()
                    .filter_map(|e| e.as_permanent_id())
                    .collect();
                if !ids.is_empty() {
                    let attached: Vec<crate::card::CardId> = self
                        .battlefield
                        .iter()
                        .filter(|c| c.attached_to.is_some_and(|h| ids.contains(&h)))
                        .map(|c| c.id)
                        .collect();
                    ids.extend(attached);
                    let mut idx = 0;
                    while idx < self.battlefield.len() {
                        if ids.contains(&self.battlefield[idx].id) {
                            let c = self.battlefield.remove(idx);
                            // CR 702.26e — a phased-out permanent is treated as
                            // though it doesn't exist, so remove it from combat
                            // if `Effect::PhaseOut` fires mid-combat (the
                            // untap-step `do_phasing` path can't, but a cast /
                            // ETB phase-out can — Talon Gates, Reality Ripple).
                            self.remove_from_combat(c.id);
                            events.push(GameEvent::PermanentPhasedOut { card_id: c.id });
                            self.phased_out.push(c);
                        } else {
                            idx += 1;
                        }
                    }
                }
                Ok(())
            }

            Effect::Untap { what, up_to } => {
                let cap = up_to
                    .as_ref()
                    .map(|v| self.evaluate_value(v, ctx).max(0) as usize);
                let mut count = 0usize;
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(c) = cap
                        && count >= c
                    {
                        break;
                    }
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid)
                        && c.tapped {
                            // CR 122.1c: stun counter replaces untap.
                            let stun = c
                                .counters
                                .get(&crate::card::CounterType::Stun)
                                .copied()
                                .unwrap_or(0);
                            if stun > 0 {
                                *c.counters
                                    .entry(crate::card::CounterType::Stun)
                                    .or_insert(0) -= 1;
                            } else {
                                c.tapped = false;
                                events.push(GameEvent::PermanentUntapped { card_id: cid });
                            }
                            count += 1;
                        }
                }
                Ok(())
            }

            Effect::PumpPT { what, power, toughness, duration } => {
                let p = self.evaluate_value(power, ctx);
                let t = self.evaluate_value(toughness, ctx);
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    if self.battlefield_find(cid).is_none() {
                        continue;
                    }
                    match duration {
                        // Fast path: the EOT fields are wiped at Cleanup.
                        Duration::EndOfTurn => {
                            if let Some(c) = self.battlefield_find_mut(cid) {
                                c.power_bonus += p;
                                c.toughness_bonus += t;
                            }
                        }
                        // Permanent pumps survive Cleanup (Wall of Roots).
                        Duration::Permanent => {
                            if let Some(c) = self.battlefield_find_mut(cid) {
                                c.perm_power_bonus += p;
                                c.perm_toughness_bonus += t;
                            }
                        }
                        // Mid durations ride the layer system, which knows
                        // how to expire EndOfCombat / UntilNextTurn effects.
                        _ => {
                            use crate::game::layers::{
                                AffectedPermanents, ContinuousEffect, Layer, Modification,
                                PtSublayer,
                            };
                            let duration_kind = map_effect_duration(*duration);
                            let source = ctx.source.unwrap_or(CardId(0));
                            let ts = self.next_timestamp();
                            self.add_continuous_effect(ContinuousEffect {
                                timestamp: ts,
                                source,
                                affected: AffectedPermanents::Specific(vec![cid]),
                                layer: Layer::L7PowerTough,
                                sublayer: Some(PtSublayer::Modify),
                                duration: duration_kind,
                                modification: Modification::ModifyPowerToughness(p, t),
                            });
                        }
                    }
                    events.push(GameEvent::PumpApplied { card_id: cid, power: p, toughness: t });
                }
                Ok(())
            }

            Effect::DoublePower { what, times, duration: _ } => {
                // Double each resolved creature's current power `times` times
                // (Exponential Growth). Adds power*(2^times - 1) as an EOT pump
                // so the live power ends at power * 2^times.
                let n = self.evaluate_value(times, ctx).max(0);
                if n > 0 {
                    let factor = 1i32.checked_shl(n as u32).unwrap_or(i32::MAX); // 2^n
                    for ent in self.resolve_selector(what, ctx) {
                        if let Some(cid) = ent.as_permanent_id()
                            && let Some(c) = self.battlefield_find(cid) {
                                let cur = c.power();
                                let delta = cur.saturating_mul(factor - 1);
                                if let Some(c) = self.battlefield_find_mut(cid) {
                                    c.power_bonus += delta;
                                    events.push(GameEvent::PumpApplied { card_id: cid, power: delta, toughness: 0 });
                                }
                            }
                    }
                }
                Ok(())
            }

            Effect::LoseAllAbilities { what, duration } => {
                // Layer-6 strip-abilities continuous effect (CR 113.10b).
                // Installs `Modification::RemoveAllAbilities` against each
                // resolved permanent so the trigger dispatcher and
                // activated-ability resolver skip the target's printed
                // abilities while the effect is in scope. Used by Mercurial
                // Transformation / Turn to Frog / Lignify "becomes 1/1
                // creature and loses all abilities" patterns.
                use crate::game::layers::{
                    AffectedPermanents, ContinuousEffect, Layer, Modification,
                };
                let duration_kind = map_effect_duration(*duration);
                let source = ctx.source.unwrap_or(CardId(0));
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id() {
                        let ts = self.next_timestamp();
                        self.add_continuous_effect(ContinuousEffect {
                            timestamp: ts,
                            source,
                            affected: AffectedPermanents::Specific(vec![cid]),
                            layer: Layer::L6Ability,
                            sublayer: None,
                            duration: duration_kind.clone(),
                            modification: Modification::RemoveAllAbilities,
                        });
                    }
                }
                Ok(())
            }

            Effect::SetBasePT { what, power, toughness, duration } => {
                // Layer-7b SetPT continuous effect — installs a real
                // `Modification::SetPowerToughness(p, t)` against the
                // resolved target permanent, with the given duration
                // mapped onto `EffectDuration`. The layer system
                // applies the set *before* counters / +N/+M bonuses
                // (CR 613.7b vs c/f), so a +1/+1 counter on top of
                // Square Up's 0/4 yields 1/5 — matching the printed
                // rules exactly.
                use crate::game::layers::{
                    AffectedPermanents, ContinuousEffect, Layer, Modification,
                    PtSublayer,
                };
                let p = self.evaluate_value(power, ctx);
                let t = self.evaluate_value(toughness, ctx);
                let duration_kind = map_effect_duration(*duration);
                let source = ctx.source.unwrap_or(CardId(0));
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id() {
                        let ts = self.next_timestamp();
                        self.add_continuous_effect(ContinuousEffect {
                            timestamp: ts,
                            source,
                            affected: AffectedPermanents::Specific(vec![cid]),
                            layer: Layer::L7PowerTough,
                            sublayer: Some(PtSublayer::SetValue),
                            duration: duration_kind.clone(),
                            modification: Modification::SetPowerToughness(p, t),
                        });
                        events.push(GameEvent::PumpApplied {
                            card_id: cid,
                            power: p,
                            toughness: t,
                        });
                    }
                }
                Ok(())
            }

            Effect::SwitchPT { what, duration } => {
                // Layer-7d P/T switch (CR 613.7d) — applied after every other
                // P/T change; two switches cancel.
                use crate::game::layers::{
                    AffectedPermanents, ContinuousEffect, Layer, Modification, PtSublayer,
                };
                let duration_kind = map_effect_duration(*duration);
                let source = ctx.source.unwrap_or(CardId(0));
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id() {
                        let ts = self.next_timestamp();
                        self.add_continuous_effect(ContinuousEffect {
                            timestamp: ts,
                            source,
                            affected: AffectedPermanents::Specific(vec![cid]),
                            layer: Layer::L7PowerTough,
                            sublayer: Some(PtSublayer::Switch),
                            duration: duration_kind.clone(),
                            modification: Modification::SwitchPowerToughness,
                        });
                    }
                }
                Ok(())
            }

            Effect::BecomeCreature {
                what,
                power,
                toughness,
                creature_types,
                keywords,
                duration,
            } => {
                use crate::game::layers::{
                    AffectedPermanents, ContinuousEffect, Layer, Modification, PtSublayer,
                };
                let p = self.evaluate_value(power, ctx);
                let t = self.evaluate_value(toughness, ctx);
                let duration_kind = map_effect_duration(*duration);
                let source = ctx.source.unwrap_or(CardId(0));
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    let affected = AffectedPermanents::Specific(vec![cid]);
                    // Layer 4: add the Creature card type + any subtypes.
                    let ts = self.next_timestamp();
                    self.add_continuous_effect(ContinuousEffect {
                        timestamp: ts,
                        source,
                        affected: affected.clone(),
                        layer: Layer::L4Type,
                        sublayer: None,
                        duration: duration_kind.clone(),
                        modification: Modification::AddCardType(crate::card::CardType::Creature),
                    });
                    for ct in creature_types {
                        let ts = self.next_timestamp();
                        self.add_continuous_effect(ContinuousEffect {
                            timestamp: ts,
                            source,
                            affected: affected.clone(),
                            layer: Layer::L4Type,
                            sublayer: None,
                            duration: duration_kind.clone(),
                            modification: Modification::AddCreatureType(*ct),
                        });
                    }
                    // Layer 7b: set the animated body's base P/T.
                    let ts = self.next_timestamp();
                    self.add_continuous_effect(ContinuousEffect {
                        timestamp: ts,
                        source,
                        affected: affected.clone(),
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::SetValue),
                        duration: duration_kind.clone(),
                        modification: Modification::SetPowerToughness(p, t),
                    });
                    // Layer 6: keyword grants (flying, vigilance, etc.).
                    for kw in keywords {
                        let ts = self.next_timestamp();
                        self.add_continuous_effect(ContinuousEffect {
                            timestamp: ts,
                            source,
                            affected: affected.clone(),
                            layer: Layer::L6Ability,
                            sublayer: None,
                            duration: duration_kind.clone(),
                            modification: Modification::AddKeyword(kw.clone()),
                        });
                    }
                }
                Ok(())
            }

            Effect::GrantTriggeredAbility { what, trigger, duration: _ } => {
                // Currently only EOT-duration grants are honored; the
                // entry is cleared in `do_cleanup`. Permanent-duration
                // grants would need a separate map keyed off the
                // granting permanent (so the grant retires when the
                // granter leaves) — tracked as future engine work.
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id() {
                        self.granted_triggers_eot
                            .entry(cid)
                            .or_default()
                            .push((**trigger).clone());
                    }
                }
                Ok(())
            }

            Effect::GrantKeyword { what, keyword, duration } => {
                // Per-instance granted keyword. Previously mutated
                // `definition.keywords` directly with no cleanup, so an
                // "EOT haste" grant would persist forever. Now distinguishes
                // EOT vs Permanent: EOT grants enter the `granted_keywords_eot`
                // bag (cleared at Cleanup along with `power_bonus`), while
                // Permanent grants still mutate the printed keyword list
                // (a leak-free no-op since indefinite grants don't expire).
                use crate::effect::Duration as EffectDur;
                let is_eot = matches!(
                    duration,
                    EffectDur::EndOfTurn | EffectDur::EndOfCombat
                );
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id() {
                        if is_eot {
                            self.grant_keyword_eot(cid, keyword.clone());
                        } else if let Some(c) = self.battlefield_find_mut(cid)
                            && !c.definition.keywords.contains(keyword)
                        {
                            std::sync::Arc::make_mut(&mut c.definition)
                                .keywords
                                .push(keyword.clone());
                        }
                    }
                }
                Ok(())
            }

            Effect::LoseKeywordThisTurn { what, keyword } => {
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid)
                        && !c.removed_keywords_eot.contains(keyword)
                    {
                        c.removed_keywords_eot.push(keyword.clone());
                    }
                }
                Ok(())
            }

            Effect::SkipNextUntap { what } => {
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid)
                    {
                        c.skip_next_untap = true;
                    }
                }
                Ok(())
            }

            Effect::ReturnSelfAsEnchantment => {
                use crate::card::CardType;
                let Some(src) = ctx.source else { return Ok(()); };
                // Locate the source in some graveyard; only return it if it
                // was a creature card (the printed-or-current type still
                // carries Creature on the first death; the enchantment side
                // we mint below has it stripped, so a second death no-ops).
                let owner = self.players.iter().position(|p| {
                    p.graveyard.iter().any(|c| {
                        c.id == src && c.definition.card_types.contains(&CardType::Creature)
                    })
                });
                let Some(owner) = owner else { return Ok(()); };
                let dest = ZoneDest::Battlefield {
                    controller: PlayerRef::Seat(owner),
                    tapped: false,
                };
                let ret_ctx = EffectContext::for_ability(src, owner, None);
                self.move_card_to(src, &dest, &ret_ctx, events);
                // Strip the Creature type so it returns as an enchantment.
                if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == src) {
                    let def = std::sync::Arc::make_mut(&mut c.definition);
                    def.card_types.retain(|t| *t != CardType::Creature);
                }
                Ok(())
            }

            Effect::Transform { what } => {
                // CR 712 — toggle each targeted DFC permanent to its other face
                // in place (same object: counters / tapped / attachments persist).
                let ids: Vec<CardId> = self
                    .resolve_selector(what, ctx)
                    .into_iter()
                    .filter_map(|e| e.as_permanent_id())
                    .collect();
                for id in ids {
                    self.transform_permanent(id, events);
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::Meld { partner, into } => {
                // CR 701.37 — meld the source with the named partner. Both
                // must be owned AND controlled by the resolving player
                // (701.37b: otherwise nothing happens — the source stays).
                let Some(source) = ctx.source else { return Ok(()) };
                let p = ctx.controller;
                let owns_and_controls = |c: &crate::card::CardInstance| {
                    c.owner == p && c.controller == p
                };
                let src_ok = self
                    .battlefield
                    .iter()
                    .any(|c| c.id == source && owns_and_controls(c));
                let partner_id = self
                    .battlefield
                    .iter()
                    .find(|c| c.definition.name == partner.as_str() && owns_and_controls(c))
                    .map(|c| c.id);
                let (Some(partner_id), true) = (partner_id, src_ok) else { return Ok(()) };
                let Some(melded_def) = crate::card_registry::lookup_by_name(into) else {
                    return Ok(());
                };
                // Lift both components off the battlefield (continuous
                // effects / combat / linked exiles all release), exile them,
                // then stash them inside the new melded object.
                let mut parts = Vec::new();
                for id in [source, partner_id] {
                    let Some(pos) = self.battlefield.iter().position(|c| c.id == id) else {
                        continue;
                    };
                    let card = self.battlefield.remove(pos);
                    self.remove_effects_from_source(id);
                    self.remove_from_combat(id);
                    events.push(GameEvent::PermanentExiled { card_id: id });
                    self.on_left_battlefield(id, events);
                    parts.push(card);
                }
                if parts.len() != 2 {
                    // One half vanished mid-resolution — leave the survivor
                    // in exile rather than melding (CR 701.37b).
                    for card in parts {
                        self.exile.push(card);
                    }
                    return Ok(());
                }
                let id = self.next_id();
                let mut melded =
                    crate::card::CardInstance::new(id, std::sync::Arc::new(melded_def), p);
                melded.meld_parts = parts;
                self.place_card_in_dest(
                    melded,
                    p,
                    &ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    events,
                );
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::SpellsCostLessThisTurn { filter, amount } => {
                self.players[ctx.controller]
                    .turn_spell_discounts
                    .push((filter.clone(), *amount));
                Ok(())
            }

            Effect::BecomeChosenColor { what, duration } => {
                use crate::decision::{Decision, DecisionAnswer};
                use crate::mana::Color;
                let duration_kind = map_effect_duration(*duration);
                let source = ctx.source.unwrap_or(CardId(0));
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    let legal = vec![
                        Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
                    ];
                    let answer = self.decider.decide(&Decision::ChooseColor { source: cid, legal });
                    let color = match answer {
                        DecisionAnswer::Color(c) => c,
                        _ => Color::Green,
                    };
                    let ts = self.next_timestamp();
                    self.add_continuous_effect(ContinuousEffect {
                        timestamp: ts,
                        source,
                        affected: AffectedPermanents::Specific(vec![cid]),
                        layer: Layer::L5Color,
                        sublayer: None,
                        duration: duration_kind.clone(),
                        modification: Modification::SetColors(vec![color]),
                    });
                }
                Ok(())
            }

            Effect::ReplaceColorWord { what, duration } => {
                // CR 612 — two ChooseColor prompts pick the word to replace
                // and its replacement; applied as a layer-3 text change.
                use crate::decision::{Decision, DecisionAnswer};
                use crate::mana::Color;
                let duration_kind = map_effect_duration(*duration);
                let source = ctx.source.unwrap_or(CardId(0));
                let legal = vec![
                    Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
                ];
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    let from = match self.decider.decide(&Decision::ChooseColor {
                        source: cid,
                        legal: legal.clone(),
                    }) {
                        DecisionAnswer::Color(c) => c,
                        _ => Color::White,
                    };
                    let to = match self.decider.decide(&Decision::ChooseColor {
                        source: cid,
                        legal: legal.clone(),
                    }) {
                        DecisionAnswer::Color(c) => c,
                        _ => Color::Blue,
                    };
                    let ts = self.next_timestamp();
                    self.add_continuous_effect(ContinuousEffect {
                        timestamp: ts,
                        source,
                        affected: AffectedPermanents::Specific(vec![cid]),
                        layer: Layer::L3Text,
                        sublayer: None,
                        duration: duration_kind.clone(),
                        modification: Modification::ReplaceColorWord(from, to),
                    });
                }
                Ok(())
            }

            Effect::ReplaceBasicLandType { what, duration } => {
                // CR 612 / 305.7 — the from/to basic land types ride the
                // ChooseColor decision (basics map 1:1 onto colors).
                use crate::card::LandType;
                use crate::decision::{Decision, DecisionAnswer};
                use crate::mana::Color;
                let land_for = |c: Color| match c {
                    Color::White => LandType::Plains,
                    Color::Blue => LandType::Island,
                    Color::Black => LandType::Swamp,
                    Color::Red => LandType::Mountain,
                    Color::Green => LandType::Forest,
                };
                let duration_kind = map_effect_duration(*duration);
                let source = ctx.source.unwrap_or(CardId(0));
                let legal = vec![
                    Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
                ];
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    let from = match self.decider.decide(&Decision::ChooseColor {
                        source: cid,
                        legal: legal.clone(),
                    }) {
                        DecisionAnswer::Color(c) => land_for(c),
                        _ => LandType::Forest,
                    };
                    let to = match self.decider.decide(&Decision::ChooseColor {
                        source: cid,
                        legal: legal.clone(),
                    }) {
                        DecisionAnswer::Color(c) => land_for(c),
                        _ => LandType::Island,
                    };
                    let ts = self.next_timestamp();
                    self.add_continuous_effect(ContinuousEffect {
                        timestamp: ts,
                        source,
                        affected: AffectedPermanents::Specific(vec![cid]),
                        layer: Layer::L3Text,
                        sublayer: None,
                        duration: duration_kind.clone(),
                        modification: Modification::ReplaceBasicLandType(from, to),
                    });
                }
                Ok(())
            }

            Effect::BecomeColor { what, colors, duration } => {
                let duration_kind = map_effect_duration(*duration);
                let source = ctx.source.unwrap_or(CardId(0));
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    let ts = self.next_timestamp();
                    self.add_continuous_effect(ContinuousEffect {
                        timestamp: ts,
                        source,
                        affected: AffectedPermanents::Specific(vec![cid]),
                        layer: Layer::L5Color,
                        sublayer: None,
                        duration: duration_kind.clone(),
                        modification: Modification::SetColors(colors.clone()),
                    });
                }
                Ok(())
            }

            Effect::ChooseColorForSelf => {
                use crate::decision::{Decision, DecisionAnswer};
                use crate::mana::Color;
                let Some(source) = ctx.source else { return Ok(()); };
                let legal = vec![
                    Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
                ];
                let color = match self.decider.decide(&Decision::ChooseColor { source, legal }) {
                    DecisionAnswer::Color(c) => c,
                    _ => Color::White,
                };
                if let Some(c) = self.battlefield_find_mut(source) {
                    c.chosen_color = Some(color);
                }
                Ok(())
            }

            Effect::GrantProtectionFromChosenColor { what, duration } => {
                // The controller picks a color; each target gains
                // protection from it for `duration` (EOT today). Mother of
                // Runes / Gods Willing.
                use crate::decision::{Decision, DecisionAnswer};
                use crate::mana::Color;
                let source = ctx.source.unwrap_or(CardId(0));
                let legal = vec![
                    Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
                ];
                let color = match self.decider.decide(&Decision::ChooseColor { source, legal }) {
                    DecisionAnswer::Color(c) => c,
                    _ => Color::White,
                };
                let kw = Keyword::Protection(color);
                let is_eot = matches!(
                    duration,
                    crate::effect::Duration::EndOfTurn | crate::effect::Duration::EndOfCombat
                );
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id() {
                        if is_eot {
                            self.grant_keyword_eot(cid, kw.clone());
                        } else if let Some(c) = self.battlefield_find_mut(cid)
                            && !c.definition.keywords.contains(&kw)
                        {
                            std::sync::Arc::make_mut(&mut c.definition).keywords.push(kw.clone());
                        }
                    }
                }
                Ok(())
            }

            // CR 701.63 — endure N: controller of `target` either grows it
            // with N +1/+1 counters or mints an N/N white Spirit. AutoDecider
            // (Bool(false)) keeps the counters; Bool(true) takes the token.
            Effect::Endure { target, n } => {
                use crate::decision::{Decision, DecisionAnswer};
                let amt = self.evaluate_value(n, ctx).max(0);
                if amt == 0 { return Ok(()); }
                let source = ctx.source.unwrap_or(CardId(0));
                let make_token = matches!(
                    self.decider.decide(&Decision::OptionalTrigger {
                        source,
                        description: "Endure: create an N/N Spirit instead of counters?".into(),
                    }),
                    DecisionAnswer::Bool(true)
                );
                if make_token {
                    let token = crate::card::TokenDefinition {
                        name: "Spirit".into(),
                        power: amt,
                        toughness: amt,
                        card_types: vec![crate::card::CardType::Creature],
                        colors: vec![crate::mana::Color::White],
                        subtypes: crate::card::Subtypes {
                            creature_types: vec![crate::card::CreatureType::Spirit],
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    self.run_effect(
                        &Effect::CreateToken {
                            who: crate::effect::PlayerRef::You,
                            count: crate::effect::Value::Const(1),
                            definition: token,
                        },
                        ctx,
                        events,
                    )?;
                } else {
                    self.run_effect(
                        &Effect::AddCounter {
                            what: target.clone(),
                            kind: crate::card::CounterType::PlusOnePlusOne,
                            amount: crate::effect::Value::Const(amt),
                        },
                        ctx,
                        events,
                    )?;
                }
                Ok(())
            }

            Effect::AddCounter { what, kind, amount } => {
                let base = self.evaluate_value(amount, ctx).max(0) as u32;
                if base == 0 { return Ok(()); }
                // CR 122.1 — Solemnity: counters can't be put on anything.
                if self.counters_locked() { return Ok(()); }
                for ent in self.resolve_selector(what, ctx) {
                    match ent {
                        EntityRef::Permanent(cid) => {
                            // CR 614.16 counter-doubling replacement: each
                            // `StaticEffect::DoubleCounters` permanent the
                            // affected permanent's *controller* has on the
                            // battlefield doubles the count. Stacking
                            // doublers multiply (2^k where k is the number
                            // of active doublers). Looked up per-target
                            // since a fan-out (`ForEach`) could span
                            // controllers.
                            let target_ctrl = self.battlefield_find(cid).map(|c| c.controller);
                            let n = if let Some(ctrl) = target_ctrl {
                                let doublers = self.counter_doublers_for(ctrl);
                                let mut scaled = base;
                                for _ in 0..doublers {
                                    scaled = scaled.saturating_mul(2);
                                }
                                // CR 614.5 — Vizier of Remedies: each copy
                                // shaves one -1/-1 counter bound for a
                                // creature its controller controls.
                                if *kind == CounterType::MinusOneMinusOne
                                    && self
                                        .battlefield_find(cid)
                                        .is_some_and(|c| c.definition.is_creature())
                                {
                                    scaled = scaled
                                        .saturating_sub(self.minus_counter_reduction_for(ctrl));
                                }
                                scaled
                            } else {
                                base
                            };
                            if let Some(c) = self.battlefield_find_mut(cid) {
                                // SOS Prepare — "A creature can't become
                                // prepared if it's already prepared":
                                // the designation is a count-1 flag, so
                                // clamp instead of stacking (you never
                                // get a second copy of the spell).
                                let n = if *kind == CounterType::Prepared {
                                    1u32.saturating_sub(c.counter_count(CounterType::Prepared))
                                } else {
                                    n
                                };
                                if n == 0 { continue; }
                                c.add_counters(*kind, n);
                                events.push(GameEvent::CounterAdded { card_id: cid, counter_type: *kind, count: n });
                            }
                            // Track per-turn "this permanent gained counters"
                            // for Fractal Tender's end-step trigger and any
                            // future "if you put a counter on this creature
                            // this turn" payoff.
                            self.permanents_gained_counter_this_turn.insert(cid);
                        }
                        EntityRef::Player(p) if *kind == CounterType::Poison => {
                            // Poison counters on players also scale per
                            // CR 614.16 (Doubling Season / Vorinclex would
                            // double poison too); use the affected player's
                            // own counter-doubler count.
                            let doublers = self.counter_doublers_for(p);
                            let mut n = base;
                            for _ in 0..doublers {
                                n = n.saturating_mul(2);
                            }
                            self.players[p].poison_counters += n;
                            events.push(GameEvent::PoisonAdded { player: p, amount: n });
                        }
                        _ => {}
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::DoubleCountersOnEach { what, kind } => {
                // CR 122.1 — Solemnity locks counter placement entirely.
                if self.counters_locked() { return Ok(()); }
                // Snapshot the (permanent, current-count) pairs first so the
                // additions don't compound across the fan-out.
                let targets: Vec<(CardId, u32)> = self
                    .resolve_selector(what, ctx)
                    .into_iter()
                    .filter_map(|e| e.as_permanent_id())
                    .filter_map(|cid| {
                        let c = self.battlefield_find(cid)?;
                        let cur = c.counter_count(*kind);
                        (cur > 0).then_some((cid, cur))
                    })
                    .collect();
                for (cid, cur) in targets {
                    // Adding `cur` doubles the total (N → 2N), routed through the
                    // CR 614.16 counter-doubling replacement like any other add.
                    let doublers = self
                        .battlefield_find(cid)
                        .map(|c| self.counter_doublers_for(c.controller))
                        .unwrap_or(0);
                    let mut add = cur;
                    for _ in 0..doublers {
                        add = add.saturating_mul(2);
                    }
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        c.add_counters(*kind, add);
                        events.push(GameEvent::CounterAdded { card_id: cid, counter_type: *kind, count: add });
                    }
                    self.permanents_gained_counter_this_turn.insert(cid);
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::RemoveCounter { what, kind, amount } => {
                let n = self.evaluate_value(amount, ctx).max(0) as u32;
                if n == 0 { return Ok(()); }
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid) {
                            let removed = c.remove_counters(*kind, n);
                            if removed > 0 {
                                events.push(GameEvent::CounterRemoved { card_id: cid, counter_type: *kind, count: removed });
                            }
                        }
                }
                Ok(())
            }

            Effect::RemoveAllCounters { what } => {
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid)
                    {
                        let kinds: Vec<(crate::card::CounterType, u32)> =
                            c.counters.iter().map(|(k, v)| (*k, *v)).collect();
                        c.counters.clear();
                        // CR 122.1b — keyword counters are counters too.
                        c.keyword_counters.clear();
                        for (kind, count) in kinds {
                            events.push(GameEvent::CounterRemoved { card_id: cid, counter_type: kind, count });
                        }
                    }
                }
                Ok(())
            }

            // CR 606 — set loyalty outright (loyalty-set effect). Overwrites
            // the Loyalty counter to `value`, emitting a balancing
            // CounterAdded / CounterRemoved so listeners see the delta.
            Effect::SetLoyalty { what, value } => {
                let target = self.evaluate_value(value, ctx).max(0) as u32;
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid)
                    {
                        let cur = c.counter_count(CounterType::Loyalty);
                        c.counters.insert(CounterType::Loyalty, target);
                        if target > cur {
                            events.push(GameEvent::CounterAdded { card_id: cid, counter_type: CounterType::Loyalty, count: target - cur });
                        } else if cur > target {
                            events.push(GameEvent::CounterRemoved { card_id: cid, counter_type: CounterType::Loyalty, count: cur - target });
                        }
                    }
                }
                Ok(())
            }

            // CR 122.1b — Add a keyword counter to `what`. The host gains
            // the named keyword while at least one counter of this kind
            // is present (applied as a layer-6 grant in
            // `compute_battlefield`). Push (modern_decks batch 183).
            Effect::AddKeywordCounter { what, keyword, amount } => {
                let base = self.evaluate_value(amount, ctx).max(0) as u32;
                if base == 0 { return Ok(()); }
                // CR 614.16 counter-doubling replacement effects (Doubling
                // Season, Vorinclex, Pir, Hardened Scales-style scalers)
                // also apply to keyword counters: they're counters per
                // CR 122.1b. Each `StaticEffect::DoubleCounters` permanent
                // the affected permanent's controller has on the
                // battlefield doubles the count.
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id() {
                        let target_ctrl = self.battlefield_find(cid).map(|c| c.controller);
                        let n = if let Some(ctrl) = target_ctrl {
                            let doublers = self.counter_doublers_for(ctrl);
                            let mut scaled = base;
                            for _ in 0..doublers {
                                scaled = scaled.saturating_mul(2);
                            }
                            scaled
                        } else {
                            base
                        };
                        if let Some(c) = self.battlefield_find_mut(cid) {
                            *c.keyword_counters.entry(keyword.clone()).or_insert(0) += n;
                            // "This permanent gained a counter this turn"
                            // payoffs count keyword counters too (CR 122.1b).
                            self.permanents_gained_counter_this_turn.insert(cid);
                            events.push(GameEvent::KeywordCounterAdded {
                                card_id: cid,
                                keyword: keyword.clone(),
                                count: n,
                            });
                        }
                    }
                }
                Ok(())
            }

            // CR 122.1b — Remove keyword counters from `what`. Clamped at
            // the source's actual count; the host loses the keyword
            // (assuming no other source) when the last counter is
            // removed. Counterpart to AddKeywordCounter. Doubling-
            // counter replacements (CR 614.16) do NOT apply to removes
            // — they only scale puts (the rule is about "if an effect
            // would put one or more counters…").
            Effect::RemoveKeywordCounter { what, keyword, amount } => {
                let request = self.evaluate_value(amount, ctx).max(0) as u32;
                if request == 0 { return Ok(()); }
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid) {
                            let entry = c.keyword_counters.entry(keyword.clone()).or_insert(0);
                            let remove = (*entry).min(request);
                            *entry -= remove;
                            // If the counter is now 0, drop the entry to
                            // keep the map sparse (so layer-6 doesn't
                            // grant a phantom 0-count keyword).
                            if *entry == 0 {
                                c.keyword_counters.remove(keyword);
                            }
                        }
                }
                Ok(())
            }

            Effect::MoveAllCounters { from, to } => {
                // CR 122.5 — relocation, not creation (no doublers).
                let src = self.resolve_selector(from, ctx).into_iter().find_map(|e| e.as_permanent_id());
                let dst = self.resolve_selector(to, ctx).into_iter().find_map(|e| e.as_permanent_id());
                if let (Some(src), Some(dst)) = (src, dst)
                    && src != dst
                {
                    let taken = self
                        .battlefield_find_mut(src)
                        .map(|c| std::mem::take(&mut c.counters))
                        .unwrap_or_default();
                    if let Some(d) = self.battlefield_find_mut(dst) {
                        for (kind, n) in taken {
                            if n > 0 {
                                d.add_counters(kind, n);
                                events.push(GameEvent::CounterAdded {
                                    card_id: dst,
                                    counter_type: kind,
                                    count: n,
                                });
                            }
                        }
                    }
                }
                Ok(())
            }

            Effect::MoveCounter { from, to, kind, amount } => {
                // CR 122.5: moving counters is a single zone-internal
                // transfer, not a remove-then-add (DoubleCounters does
                // NOT apply). The actual move is clamped at the source's
                // current counter pool.
                let request = self.evaluate_value(amount, ctx).max(0) as u32;
                if request == 0 { return Ok(()); }
                // Pick the source (singular — moves typically target one
                // permanent; if multiple, take the first).
                let source_cids: Vec<_> = self.resolve_selector(from, ctx)
                    .into_iter()
                    .filter_map(|e| e.as_permanent_id())
                    .collect();
                let Some(src_cid) = source_cids.first().copied() else { return Ok(()); };
                let removed = if let Some(s) = self.battlefield_find_mut(src_cid) {
                    s.remove_counters(*kind, request)
                } else {
                    0
                };
                if removed == 0 { return Ok(()); }
                events.push(GameEvent::CounterRemoved {
                    card_id: src_cid, counter_type: *kind, count: removed,
                });
                // Pick the first destination and add the removed counter
                // count (no doubling per CR 122.5 — moves preserve the
                // counter identity, they're not "put counters").
                for ent in self.resolve_selector(to, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(d) = self.battlefield_find_mut(cid) {
                            d.add_counters(*kind, removed);
                            events.push(GameEvent::CounterAdded {
                                card_id: cid, counter_type: *kind, count: removed,
                            });
                            break;
                        }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::Proliferate => {
                // CR 122.1 — Solemnity locks out all counter placement.
                if self.counters_locked() { return Ok(()); }
                // CR 701.34a — "To proliferate means to choose any number
                // of permanents and/or players that have a counter, then
                // give each one additional counter of each kind that
                // permanent or player already has." The proliferating
                // player picks which permanents and which players. The
                // engine's auto-decider implements a strategic baseline:
                //  - Pick each friendly permanent for every counter kind
                //    that benefits the controller (+1/+1, Charge, Loyalty,
                //    Page, Stun on enemy-bound permanents). Skip
                //    MinusOneMinusOne on friendlies (would shrink them).
                //  - Pick each enemy permanent for MinusOneMinusOne only
                //    (would shrink them); skip +1/+1 on enemies (would
                //    pump them).
                //  - For poison: each opponent gets +1 poison; the
                //    proliferating player declines their own poison
                //    counter.
                let proliferating = ctx.controller;
                let updates: Vec<(CardId, Vec<CounterType>)> = self
                    .battlefield
                    .iter()
                    .map(|c| {
                        let friendly = c.controller == proliferating;
                        let kinds: Vec<CounterType> = c.counters.iter()
                            .filter(|(_, n)| **n > 0)
                            .filter(|(k, _)| match **k {
                                // Bad-for-friendly: skip on your stuff,
                                // proliferate on enemy stuff.
                                CounterType::MinusOneMinusOne => !friendly,
                                CounterType::Stun => !friendly,
                                // Good-for-friendly: proliferate yours,
                                // skip opponent's.
                                CounterType::PlusOnePlusOne
                                | CounterType::Loyalty
                                | CounterType::Charge
                                | CounterType::Page => friendly,
                                // Other kinds: proliferate by default
                                // (the controller can always elect to
                                // proliferate any counter under the
                                // printed rule).
                                _ => true,
                            })
                            .map(|(k, _)| *k)
                            .collect();
                        (c.id, kinds)
                    })
                    .filter(|(_, kinds)| !kinds.is_empty())
                    .collect();
                for (cid, kinds) in updates {
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        for k in kinds {
                            c.add_counters(k, 1);
                            events.push(GameEvent::CounterAdded { card_id: cid, counter_type: k, count: 1 });
                        }
                    }
                }
                for i in 0..self.players.len() {
                    if self.players[i].poison_counters > 0 && i != proliferating {
                        self.players[i].poison_counters += 1;
                        events.push(GameEvent::PoisonAdded { player: i, amount: 1 });
                    }
                }
                Ok(())
            }

            Effect::GainControl { what, to, duration } => {
                use crate::effect::Duration;
                let new_ctrl = match to {
                    Some(pref) => match self.resolve_player(pref, ctx) {
                        Some(p) => p,
                        None => return Ok(()),
                    },
                    None => ctx.controller,
                };
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    let prev = match self.battlefield_find_mut(cid) {
                        Some(c) if c.controller != new_ctrl => {
                            let prev = c.controller;
                            c.controller = new_ctrl;
                            // CR 302.6 — it hasn't been under the new
                            // controller's control since their turn began.
                            c.summoning_sick = true;
                            prev
                        }
                        _ => continue,
                    };
                    // For non-permanent steals, remember the pre-steal
                    // controller so control reverts when the duration ends
                    // (CR 800.4). Keep the earliest entry if the permanent is
                    // re-stolen so it unwinds all the way back.
                    if !matches!(duration, Duration::Permanent)
                        && !self.temporary_control.iter().any(|t| t.card == cid)
                    {
                        self.temporary_control.push(crate::game::TempControl {
                            card: cid,
                            original_controller: prev,
                            duration: *duration,
                            source: None,
                        });
                    }
                }
                Ok(())
            }

            Effect::GainControlWhileSourceRemains { what } => {
                // CR 611.2c — the steal lasts while the source stays on the
                // battlefield; `on_left_battlefield` unwinds it.
                let Some(src) = ctx.source else { return Ok(()) };
                let new_ctrl = ctx.controller;
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    if let Some(c) = self.battlefield_find_mut(cid)
                        && c.controller != new_ctrl
                    {
                        let prev = c.controller;
                        c.controller = new_ctrl;
                        c.summoning_sick = true;
                        if !self.temporary_control.iter().any(|t| t.card == cid) {
                            self.temporary_control.push(crate::game::TempControl {
                                card: cid,
                                original_controller: prev,
                                duration: crate::effect::Duration::Permanent,
                                source: Some(src),
                            });
                        }
                    }
                }
                Ok(())
            }

            Effect::ExchangeControl { a, b } => {
                // CR 701.12 — swap the controllers of the two resolved
                // permanents simultaneously. No-op unless both still exist.
                let ca = self.resolve_selector(a, ctx).into_iter().find_map(|e| e.as_permanent_id());
                let cb = self.resolve_selector(b, ctx).into_iter().find_map(|e| e.as_permanent_id());
                if let (Some(ca), Some(cb)) = (ca, cb)
                    && ca != cb
                {
                    let ctrl_a = self.battlefield_find(ca).map(|c| c.controller);
                    let ctrl_b = self.battlefield_find(cb).map(|c| c.controller);
                    if let (Some(ctrl_a), Some(ctrl_b)) = (ctrl_a, ctrl_b)
                        && ctrl_a != ctrl_b
                    {
                        // CR 302.6 — both sides pick up summoning sickness
                        // under their new controller.
                        if let Some(c) = self.battlefield_find_mut(ca) {
                            c.controller = ctrl_b;
                            c.summoning_sick = true;
                        }
                        if let Some(c) = self.battlefield_find_mut(cb) {
                            c.controller = ctrl_a;
                            c.summoning_sick = true;
                        }
                    }
                }
                Ok(())
            }

            Effect::ExchangeControlChoosing { filter, with } => self.resolve_exchange_control_choosing(filter, with, ctx, events),


            Effect::CreateToken { who, count, definition } => {
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let mut n = self.evaluate_value(count, ctx).max(0) as u32;
                // CR 614.13 token-doubling replacement: each
                // `StaticEffect::DoubleTokens` permanent the controller has on
                // the battlefield (Adrix and Nev, Twincasters; Doubling Season
                // for the token half) doubles the count. Stacking doublers
                // multiply (2^k where k is the number of active doublers).
                let doublers = self.token_doublers_for(p);
                for _ in 0..doublers {
                    n = n.saturating_mul(2);
                }
                // Mint-time dynamic P/T (Shark Typhoon's X/X): resolved once
                // against this resolution's context, stamped as the token's
                // printed P/T (a mint-time rider, CR 707.2-stable).
                let dyn_pt = definition.dynamic_pt.as_ref().map(|(pv, tv)| {
                    (self.evaluate_value(pv, ctx), self.evaluate_value(tv, ctx))
                });
                for _ in 0..n {
                    let mut def = token_to_card_definition(definition);
                    if let Some((pw, tn)) = dyn_pt {
                        def.power = pw;
                        def.toughness = tn;
                    }
                    self.mint_token_onto_battlefield(def, p, false, events);
                }
                Ok(())
            }

            Effect::Amass { who, count, extra_type } => {
                use crate::card::{CardType, CreatureType, CounterType};
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                // CR 701.43a — grow an existing Army you control, else mint a
                // 0/0 black Army token first.
                let army = self.battlefield.iter().find(|c| {
                    c.controller == p
                        && c.definition.is_creature()
                        && c.definition.subtypes.creature_types.contains(&CreatureType::Army)
                }).map(|c| c.id);
                let army = match army {
                    Some(id) => id,
                    None => {
                        let mut types = vec![CreatureType::Army];
                        if let Some(t) = extra_type { types.push(*t); }
                        let def = token_to_card_definition(&crate::card::TokenDefinition {
                            name: "Army".into(),
                            power: 0,
                            toughness: 0,
                            card_types: vec![CardType::Creature],
                            colors: vec![Color::Black],
                            subtypes: crate::card::Subtypes {
                                creature_types: types,
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                        self.mint_token_onto_battlefield(def, p, false, events)
                    }
                };
                // CR 614.16 — counter doubling applies to the amassed counters.
                if n > 0 && self.battlefield.iter().any(|c| c.id == army) {
                    let mut scaled = n;
                    for _ in 0..self.counter_doublers_for(p) {
                        scaled = scaled.saturating_mul(2);
                    }
                    if let Some(c) = self.battlefield_find_mut(army) {
                        c.add_counters(CounterType::PlusOnePlusOne, scaled);
                    }
                    events.push(GameEvent::CounterAdded {
                        card_id: army, counter_type: CounterType::PlusOnePlusOne, count: scaled,
                    });
                    self.permanents_gained_counter_this_turn.insert(army);
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::CreateTokenAttacking { who, count, definition, cleanup } => {
                use crate::game::types::{Attack, AttackTarget};
                // Only meaningful while a combat is in progress.
                if self.attacking.is_empty() {
                    return Ok(());
                }
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                // Attack the same defender the source is attacking; else the
                // controller's first opponent.
                let target = ctx
                    .source
                    .and_then(|src| self.attacking.iter().find(|a| a.attacker == src))
                    .map(|a| a.target)
                    .or_else(|| {
                        (0..self.players.len())
                            .find(|&q| !self.same_team(q, p))
                            .map(AttackTarget::Player)
                    });
                let Some(target) = target else { return Ok(()); };
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                for _ in 0..n {
                    let def = token_to_card_definition(definition);
                    let id = self.mint_token_onto_battlefield(def, p, true, events);
                    // Join combat tapped + attacking (CR 508.3a) — bypasses the
                    // declare-attackers timing/sickness gates, like Ninjutsu.
                    if self.battlefield.iter().any(|c| c.id == id) {
                        self.attacking.push(Attack { attacker: id, target });
                        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id) {
                            c.attacked_this_turn = true;
                        }
                        events.push(GameEvent::AttackerDeclared(id));
                        // Mobilize/Myriad temporary tokens leave at end of combat.
                        if !matches!(cleanup, AttackingTokenCleanup::None) {
                            self.attacking_token_cleanup.push((id, *cleanup));
                        }
                    }
                }
                Ok(())
            }

            Effect::Myriad => {
                use crate::game::types::{Attack, AttackTarget};
                // Source must currently be attacking a player.
                let Some(src) = ctx.source else { return Ok(()); };
                let Some(src_attack) = self.attacking.iter().find(|a| a.attacker == src) else {
                    return Ok(());
                };
                let defending = match src_attack.target {
                    AttackTarget::Player(p) => p,
                    AttackTarget::Planeswalker(pw) => {
                        self.battlefield_find(pw).map(|c| c.controller).unwrap_or(usize::MAX)
                    }
                };
                let Some(ctrl) = self.battlefield_find(src).map(|c| c.controller) else {
                    return Ok(());
                };
                let def = self.battlefield_find(src).map(|c| (*c.definition).clone());
                let Some(def) = def else { return Ok(()); };
                // CR 702.115b — one copy per opponent other than the defender.
                let opps: Vec<usize> = (0..self.players.len())
                    .filter(|&q| !self.same_team(q, ctrl) && q != defending)
                    .collect();
                for opp in opps {
                    let id = self.mint_token_onto_battlefield(def.clone(), ctrl, true, events);
                    if self.battlefield.iter().any(|c| c.id == id) {
                        self.attacking.push(Attack { attacker: id, target: AttackTarget::Player(opp) });
                        if let Some(c) = self.battlefield_find_mut(id) {
                            c.attacked_this_turn = true;
                        }
                        events.push(GameEvent::AttackerDeclared(id));
                        self.attacking_token_cleanup
                            .push((id, AttackingTokenCleanup::ExileAtEndOfCombat));
                    }
                }
                Ok(())
            }

            Effect::GrantNextInstantOrSorceryDiscountThisTurn { amount } => {
                // Stamp the discount with the controller's current IS tally so
                // it applies only to the *next* instant/sorcery they cast.
                let p = ctx.controller;
                let granted_at = self.players[p].instants_or_sorceries_cast_this_turn;
                self.players[p].pending_is_discounts.push((*amount, granted_at));
                Ok(())
            }

            Effect::Enlist => {
                // CR 702.151 — tap a nonattacking, non-sick creature you
                // control and add its power to the attacker until end of turn.
                let Some(src) = ctx.source else { return Ok(()); };
                let Some(ctrl) = self.battlefield_find(src).map(|c| c.controller) else {
                    return Ok(());
                };
                let attacking_ids: Vec<CardId> = self.attacking.iter().map(|a| a.attacker).collect();
                // Highest-power eligible creature, only if its power is positive.
                let best = self.battlefield.iter()
                    .filter(|c| c.controller == ctrl
                        && c.id != src
                        && c.definition.is_creature()
                        && !c.tapped
                        && !c.summoning_sick
                        && !attacking_ids.contains(&c.id))
                    .max_by_key(|c| c.power())
                    .filter(|c| c.power() > 0)
                    .map(|c| (c.id, c.power()));
                if let Some((helper, power)) = best {
                    if let Some(c) = self.battlefield_find_mut(helper) {
                        c.tapped = true;
                    }
                    events.push(GameEvent::PermanentTapped { card_id: helper });
                    if let Some(c) = self.battlefield_find_mut(src) {
                        c.power_bonus += power;
                        events.push(GameEvent::PumpApplied { card_id: src, power, toughness: 0 });
                    }
                }
                Ok(())
            }

            Effect::CreateTokenCopyOf {
                who,
                count,
                source,
                extra_creature_types,
                override_pt,
                non_legendary,
            } => {
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let mut n = self.evaluate_value(count, ctx).max(0) as u32;
                let doublers = self.token_doublers_for(p);
                for _ in 0..doublers {
                    n = n.saturating_mul(2);
                }
                // Resolve the source permanent. Walk battlefield first;
                // fall back to graveyard / hand / exile via the same
                // sequence `move_card_to` uses, so a fresh copy can be
                // minted off a card that left the bf mid-resolution.
                let source_id = self
                    .resolve_selector(source, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                        _ => None,
                    });
                let Some(src_id) = source_id else { return Ok(()); };
                // Source def: battlefield first, then graveyard / exile so an
                // Embalm/Eternalize copy (CR 702.88/702.91) can be minted off
                // the card after it's been exiled as the activation cost.
                let source_def = self
                    .battlefield
                    .iter()
                    .find(|c| c.id == src_id)
                    .or_else(|| self.exile.iter().find(|c| c.id == src_id))
                    .or_else(|| {
                        self.players
                            .iter()
                            .find_map(|pl| pl.graveyard.iter().find(|c| c.id == src_id))
                    })
                    .map(|c| (*c.definition).clone());
                let Some(mut def) = source_def else { return Ok(()); };
                // Apply extra creature types & P/T override.
                let mut extra_types = def.subtypes.creature_types.clone();
                for t in extra_creature_types.iter() {
                    if !extra_types.contains(t) {
                        extra_types.push(*t);
                    }
                }
                def.subtypes.creature_types = extra_types;
                if let Some((p_o, t_o)) = override_pt {
                    def.power = *p_o;
                    def.toughness = *t_o;
                }
                if *non_legendary {
                    def.supertypes.clear();
                }
                for _ in 0..n {
                    self.mint_token_onto_battlefield(def.clone(), p, false, events);
                }
                Ok(())
            }

            Effect::Populate { who } => {
                // CR 701.32 — copy one creature token the player controls.
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                // Pick the controller's highest-power creature token (AutoDecider
                // heuristic; deterministic for tests).
                let pick = self
                    .battlefield
                    .iter()
                    .filter(|c| c.is_token && c.controller == p && c.definition.is_creature())
                    .max_by_key(|c| c.power())
                    .map(|c| c.id);
                let Some(src_id) = pick else { return Ok(()); };
                let Some(def) = self
                    .battlefield
                    .iter()
                    .find(|c| c.id == src_id)
                    .map(|c| (*c.definition).clone())
                else {
                    return Ok(());
                };
                // Token doublers (Doubling Season / Parallel Lives) apply.
                let mut n: u32 = 1;
                for _ in 0..self.token_doublers_for(p) {
                    n = n.saturating_mul(2);
                }
                for _ in 0..n {
                    self.mint_token_onto_battlefield(def.clone(), p, false, events);
                }
                Ok(())
            }

            Effect::CounterSpell { what } => {
                // With only a single stack target, we pop the top of the
                // stack if it's a spell (matching by target id when
                // available). Spells flagged `uncounterable` (Cavern of
                // Souls) are skipped — the counter has no effect on them.
                let targets = self.resolve_selector(what, ctx);
                let mut to_remove: Vec<usize> = Vec::new();
                for t in &targets {
                    if let Some(cid) = t.as_card_id()
                        && let Some(pos) = self.stack.iter().position(|si| matches!(
                            si,
                            StackItem::Spell { card, uncounterable: false, .. }
                                if card.id == cid
                        ))
                    {
                        to_remove.push(pos);
                    }
                }
                to_remove.sort_unstable_by(|a, b| b.cmp(a));
                for pos in to_remove {
                    if let StackItem::Spell { card, .. } = self.stack.remove(pos) {
                        self.route_to_graveyard(*card, events);
                    }
                }
                Ok(())
            }

            Effect::CounterSpellToZone { what, zone } => {
                // Counter target spell and route the lifted card to a
                // non-graveyard zone. Overrides CR 701.6a's default
                // (countered spell -> owner's graveyard) via the spell's
                // printed "instead" clause (CR 608.2c — later text on a
                // card may modify earlier text). Memory Lapse routes to
                // top of owner's library; Spell Crumple to exile; Remand
                // to owner's hand. Spells flagged `uncounterable` (Cavern
                // of Souls) are skipped — the counter does nothing.
                use crate::effect::CounteredSpellZone;
                let targets = self.resolve_selector(what, ctx);
                let mut to_remove: Vec<usize> = Vec::new();
                for t in &targets {
                    if let Some(cid) = t.as_card_id()
                        && let Some(pos) = self.stack.iter().position(|si| matches!(
                            si,
                            StackItem::Spell { card, uncounterable: false, .. }
                                if card.id == cid
                        ))
                    {
                        to_remove.push(pos);
                    }
                }
                to_remove.sort_unstable_by(|a, b| b.cmp(a));
                for pos in to_remove {
                    if let StackItem::Spell { card, .. } = self.stack.remove(pos) {
                        let owner = card.owner;
                        match zone {
                            // Index 0 is the top (draw = `library.remove(0)`).
                            CounteredSpellZone::OwnerLibraryTop => {
                                self.players[owner].library.insert(0, *card);
                            }
                            CounteredSpellZone::OwnerLibraryBottom => {
                                self.players[owner].library.push(*card);
                            }
                            CounteredSpellZone::OwnerLibraryTopOrBottom => {
                                // CR 701.5g — the spell's owner chooses top or
                                // bottom (Subtlety). Ask via OptionalTrigger
                                // (true = top, false = bottom); AutoDecider
                                // bottoms it.
                                let cid = card.id;
                                let on_top = matches!(
                                    self.decider.decide(&crate::decision::Decision::OptionalTrigger {
                                        source: cid,
                                        description: "Put countered spell on top of library? (no = bottom)".into(),
                                    }),
                                    crate::decision::DecisionAnswer::Bool(true)
                                );
                                if on_top {
                                    self.players[owner].library.insert(0, *card);
                                } else {
                                    self.players[owner].library.push(*card);
                                }
                            }
                            CounteredSpellZone::OwnerHand => {
                                self.players[owner].hand.push(*card);
                            }
                            CounteredSpellZone::Exile => {
                                self.exile.push(*card);
                            }
                        }
                    }
                }
                Ok(())
            }

            Effect::CounterUnlessPaid { what, mana_cost, exile, extra_generic } => {
                // Counter target spell unless its controller pays `mana_cost`.
                // Auto-pays on behalf of the spell's controller via the
                // existing `auto_tap_for_cost` + `mana_pool.pay` path: if
                // affordable, the spell stays; otherwise it's countered (and
                // exiled instead of binned when `exile` is set — Reject).
                let targets = self.resolve_selector(what, ctx);
                let target_id = targets.into_iter().find_map(|t| match t {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => Some(cid),
                    _ => None,
                });
                let Some(cid) = target_id else { return Ok(()); };
                let pos = self.stack.iter().position(|si| matches!(
                    si,
                    StackItem::Spell { card, uncounterable: false, .. } if card.id == cid
                ));
                let Some(pos) = pos else { return Ok(()); };
                let StackItem::Spell { caster: spell_caster, .. } = &self.stack[pos]
                    else { unreachable!("filtered above") };
                let spell_caster = *spell_caster;

                // Try to auto-pay on behalf of the spell's controller. We
                // override priority temporarily so `auto_tap_for_cost`
                // taps that player's lands. `try_pay_with_auto_tap` rolls
                // back the pool + tap state on payment failure.
                let mut cost = mana_cost.clone();
                if let Some(v) = extra_generic {
                    let x = self.evaluate_value(v, ctx).max(0) as u32;
                    if x > 0 {
                        cost.symbols.push(crate::mana::ManaSymbol::Generic(x));
                    }
                }
                let saved_priority = self.priority.player_with_priority;
                self.priority.player_with_priority = spell_caster;
                let paid = self.try_pay_with_auto_tap(spell_caster, &cost).is_ok();
                self.priority.player_with_priority = saved_priority;

                if !paid
                    && let StackItem::Spell { card, .. } = self.stack.remove(pos)
                {
                    if *exile {
                        self.exile.push(*card);
                    } else {
                        self.route_to_graveyard(*card, events);
                    }
                }
                Ok(())
            }

            Effect::CounterUnless { what, cost } => {
                // CR 702.21 — Ward body. Resolve `what` to a card-id, walk
                // the stack for the topmost matching `Spell` (by `card.id`)
                // or `Trigger` (by `source`), and try to auto-pay `cost`
                // on the affected controller's behalf. If they can't pay,
                // the stack item is removed (spells fall into their
                // owner's graveyard; abilities just vanish).
                use crate::card::WardCost;

                let targets = self.resolve_selector(what, ctx);
                let target_id = targets.into_iter().find_map(|t| match t {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => Some(cid),
                    _ => None,
                });
                let Some(cid) = target_id else { return Ok(()); };

                // Find the topmost matching stack item — prefer Spell, then
                // Trigger. CR 702.21a: Ward fires on "spell or ability";
                // the stack carries spells as `StackItem::Spell` and
                // activated/triggered abilities as `StackItem::Trigger`.
                let mut spell_pos: Option<usize> = None;
                let mut trigger_pos: Option<usize> = None;
                for (i, si) in self.stack.iter().enumerate().rev() {
                    match si {
                        StackItem::Spell { card, uncounterable: false, .. }
                            if card.id == cid && spell_pos.is_none() =>
                        {
                            spell_pos = Some(i);
                        }
                        StackItem::Trigger { source, .. }
                            if *source == cid && trigger_pos.is_none() =>
                        {
                            trigger_pos = Some(i);
                        }
                        _ => {}
                    }
                    if spell_pos.is_some() && trigger_pos.is_some() {
                        break;
                    }
                }
                // Prefer Spell if both exist — a card on the stack as a
                // spell can't simultaneously be the source of an ability
                // (the permanent doesn't exist yet).
                let (pos, affected_controller, is_spell) = match (spell_pos, trigger_pos) {
                    (Some(p), _) => {
                        let StackItem::Spell { caster, .. } = &self.stack[p] else {
                            unreachable!()
                        };
                        (p, *caster, true)
                    }
                    (None, Some(p)) => {
                        let StackItem::Trigger { controller, .. } = &self.stack[p] else {
                            unreachable!()
                        };
                        (p, *controller, false)
                    }
                    (None, None) => return Ok(()),
                };

                // Attempt auto-pay on the affected controller's behalf.
                let paid = match cost {
                    WardCost::Mana(mc) => {
                        let saved_priority = self.priority.player_with_priority;
                        self.priority.player_with_priority = affected_controller;
                        let ok = self.try_pay_with_auto_tap(affected_controller, mc).is_ok();
                        self.priority.player_with_priority = saved_priority;
                        ok
                    }
                    WardCost::Life(n) => {
                        // Ward—Pay N life. CR 119.4 forbids paying more
                        // life than you have, so insufficient life means
                        // payment fails.
                        let n = *n as i32;
                        if self.effective_life(affected_controller) >= n {
                            self.pay_life_cost(affected_controller, n as u32);
                            true
                        } else {
                            false
                        }
                    }
                    WardCost::Discard(n) => {
                        // Ward—Discard N cards. Payable only if the
                        // controller has ≥ N cards in hand. Auto-pay
                        // picks the first N cards. An interactive
                        // surface should prompt.
                        let n = *n as usize;
                        if self.players[affected_controller].hand.len() >= n {
                            // Through the shared discard funnel so
                            // CardDiscarded fires and Madness applies
                            // (CR 702.35).
                            for _ in 0..n {
                                let Some(card_id) = self
                                    .players[affected_controller]
                                    .hand
                                    .first()
                                    .map(|c| c.id)
                                else {
                                    break;
                                };
                                self.discard_card(affected_controller, card_id, events);
                            }
                            true
                        } else {
                            false
                        }
                    }
                    WardCost::SacrificeCreature => {
                        let pick = self
                            .battlefield
                            .iter()
                            .find(|c| {
                                c.controller == affected_controller && c.definition.is_creature()
                            })
                            .map(|c| c.id);
                        if let Some(sac_id) = pick {
                            // Real sacrifice (CR 701.16): dies/sac events
                            // and the die snapshot, not a bare removal.
                            self.sacrifice_one(sac_id, affected_controller, events);
                            true
                        } else {
                            false
                        }
                    }
                    WardCost::SacrificePermanents(n) => {
                        let picks: Vec<CardId> = self
                            .battlefield
                            .iter()
                            .filter(|c| c.controller == affected_controller)
                            .take(*n as usize)
                            .map(|c| c.id)
                            .collect();
                        if picks.len() == *n as usize {
                            for sac_id in picks {
                                self.sacrifice_one(sac_id, affected_controller, events);
                            }
                            true
                        } else {
                            false
                        }
                    }
                    // Not a printed ward cost (UnlessPlayerPays-only).
                    WardCost::GenericSourcePower => false,
                };

                if !paid {
                    let removed = self.stack.remove(pos);
                    if is_spell
                        && let StackItem::Spell { card, .. } = removed
                    {
                        self.route_to_graveyard(*card, events);
                    }
                    // Trigger items just drop off — nothing else to clean up.
                }
                Ok(())
            }

            Effect::UnlessPlayerPays { who, cost, then } => {
                // Rhystic-tax rider: resolve the taxed player, ask them yes/no
                // whether to pay `cost`, and resolve `then` if they don't (or
                // can't). `then` runs in this same context, so its
                // `PlayerRef::You` is the rider's controller. The question is
                // seat-routed: a `wants_ui` payer (who may be an opponent of
                // the resolving controller) gets the yes/no modal.
                use crate::card::WardCost;
                let Some(payer) = self.resolve_player(who, ctx) else {
                    return self.run_effect(then, ctx, events);
                };
                // AutoDecider declines (false) — let the effect resolve.
                let mut cursor = 0;
                let Some(wants_to_pay) = self.ask_seat_bool(
                    &mut cursor,
                    payer,
                    "Pay the tax to prevent the triggered effect?".to_string(),
                    ctx.source.unwrap_or(CardId(0)),
                    effect,
                ) else {
                    return Ok(());
                };
                self.clear_answer_log();
                let paid = wants_to_pay
                    && match cost {
                        WardCost::Mana(mc) => {
                            let saved = self.priority.player_with_priority;
                            self.priority.player_with_priority = payer;
                            let ok = self.try_pay_with_auto_tap(payer, mc).is_ok();
                            self.priority.player_with_priority = saved;
                            ok
                        }
                        WardCost::Life(n) => {
                            let n = *n as i32;
                            if self.effective_life(payer) >= n {
                                self.pay_life_cost(payer, n as u32);
                                true
                            } else {
                                false
                            }
                        }
                        // "{X} where X is this creature's power" (Esper
                        // Sentinel). Read the source's computed power and
                        // auto-tap-pay that much generic.
                        WardCost::GenericSourcePower => {
                            let x = ctx
                                .source
                                .and_then(|sid| self.computed_permanent(sid))
                                .map(|c| c.power.max(0) as u32)
                                .unwrap_or(0);
                            if x == 0 {
                                true
                            } else {
                                let mc = crate::mana::cost(&[crate::mana::generic(x)]);
                                let saved = self.priority.player_with_priority;
                                self.priority.player_with_priority = payer;
                                let ok = self.try_pay_with_auto_tap(payer, &mc).is_ok();
                                self.priority.player_with_priority = saved;
                                ok
                            }
                        }
                        // Discard / sacrifice costs aren't used by the
                        // tax-rider cards; treat as unpaid so the effect runs.
                        _ => false,
                    };
                if !paid {
                    self.run_effect(then, ctx, events)?;
                }
                Ok(())
            }

            Effect::CounterAbility { what } => {
                // Counter target activated/triggered ability. The selector
                // resolves to a permanent (the ability's source); we remove
                // the topmost `StackItem::Trigger` whose `source` matches.
                // Used by Consign to Memory.
                let targets = self.resolve_selector(what, ctx);
                let mut to_remove: Vec<usize> = Vec::new();
                for t in &targets {
                    if let Some(cid) = t.as_permanent_id() {
                        // Walk top-down so we counter the most recent
                        // matching trigger (the one the player most likely
                        // intends to cancel).
                        if let Some(pos) = self
                            .stack
                            .iter()
                            .enumerate()
                            .rev()
                            .find_map(|(i, si)| match si {
                                StackItem::Trigger { source, .. } if *source == cid => Some(i),
                                _ => None,
                            })
                        {
                            to_remove.push(pos);
                        }
                    }
                }
                to_remove.sort_unstable_by(|a, b| b.cmp(a));
                for pos in to_remove {
                    self.stack.remove(pos);
                }
                Ok(())
            }

            Effect::SacrificeAllMatching { who, filter } => {
                for ent in self.resolve_selector(who, ctx) {
                    let EntityRef::Player(p) = ent else { continue };
                    let ids: Vec<crate::card::CardId> = self
                        .battlefield
                        .iter()
                        .filter(|c| {
                            c.controller == p
                                && self.evaluate_requirement_static(
                                    filter,
                                    &Target::Permanent(c.id),
                                    p,
                                    ctx.source,
                                )
                        })
                        .map(|c| c.id)
                        .collect();
                    for id in ids {
                        if self.battlefield_find(id).is_some() {
                            self.sacrifice_one(id, p, events);
                        }
                    }
                }
                Ok(())
            }

            Effect::Sacrifice { who, count, filter } => {
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 {
                    return Ok(());
                }
                // The source permanent (if any) — Daemogoth Titan-style
                // "When this attacks, sacrifice another creature" triggers
                // bind `ctx.source` to themselves. The auto-picker prefers NOT
                // picking the source when other legal candidates exist, so the
                // printed "another" intent is honored.
                let source_id = ctx.source;
                // CR 701.16 — the player doing the sacrificing chooses which
                // permanent(s). For a `wants_ui` player with a genuine choice
                // (more legal candidates than required) we suspend: a *single*
                // sacrifice uses the in-scene `ChooseTarget` cursor, a
                // multi-sacrifice uses the `ChooseCards` modal. Bots and the
                // "no real choice" case keep the auto-pick (cheapest/weakest
                // non-source). The suspend's continuation re-runs the
                // sacrifice for every seat not yet processed, so a symmetric
                // edict completes for all players in APNAP order.
                let seats: Vec<usize> = self
                    .resolve_selector(who, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                for (i, p) in seats.iter().copied().enumerate() {
                    // Sigarda / Tamiyo — "spells and abilities your opponents
                    // control can't cause you to sacrifice." Skip a player the
                    // opponent-controlled effect would force.
                    if p != ctx.controller && self.player_cant_be_made_to_sacrifice(p) {
                        continue;
                    }
                    let candidates = self.sacrifice_candidates(p, filter, source_id);
                    if candidates.is_empty() {
                        continue;
                    }
                    if candidates.len() > n && self.players[p].wants_ui {
                        let source = source_id.unwrap_or(crate::card::CardId(0));
                        let decision = if n == 1 {
                            crate::decision::Decision::ChooseTarget {
                                source,
                                legal: candidates.iter().map(|id| Target::Permanent(*id)).collect(),
                                source_name: ctx.source_name.unwrap_or("").to_string(),
                                description: "choose a permanent to sacrifice".into(),
                            }
                        } else {
                            crate::decision::Decision::ChooseCards {
                                source,
                                prompt: format!("Choose {n} permanents to sacrifice"),
                                candidates: self.card_id_names(&candidates),
                                min: n as u32,
                                max: n as u32,
                            }
                        };
                        let rest = per_seat_continuation(&seats[i + 1..], |q| Effect::Sacrifice {
                            who: Selector::Player(crate::effect::PlayerRef::Seat(q)),
                            count: crate::effect::Value::Const(n as i32),
                            filter: filter.clone(),
                        });
                        self.suspend_signal = Some((
                            decision,
                            PendingEffectState::SacrificePending { player: p },
                            rest,
                        ));
                        return Ok(());
                    }
                    let ids = self.auto_pick_sacrifices(&candidates, n, source_id, false, false);
                    for id in ids {
                        self.sacrifice_one(id, p, events);
                    }
                }
                Ok(())
            }

            Effect::PlayerExilesPermanents { who, count, filter } => {
                // Exile analogue of Annihilator (Bane of Bala Ged). The
                // affected player auto-picks the weakest N matching permanents;
                // a human-defender chooser is a follow-up (tracked in TODO.md).
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 {
                    return Ok(());
                }
                let players: Vec<usize> = self.resolve_players(who, ctx);
                for p in players {
                    let candidates = self.sacrifice_candidates(p, filter, ctx.source);
                    let ids = self.auto_pick_sacrifices(&candidates, n, ctx.source, false, false);
                    for id in ids {
                        self.move_card_to(id, &ZoneDest::Exile, ctx, events);
                    }
                }
                Ok(())
            }

            Effect::SacrificeSource => {
                if let Some(id) = ctx.source
                    && let Some(c) = self.battlefield_find(id)
                {
                    let p = c.controller;
                    let is_creature = c.definition.is_creature();
                    if is_creature {
                        self.died_card_snapshots.insert(id, c.clone());
                        events.push(GameEvent::CreatureSacrificed { card_id: id, who: p });
                        events.push(GameEvent::CreatureDied { card_id: id });
                    }
                    events.push(GameEvent::PermanentSacrificed { card_id: id, who: p });
                    let mut die_evs = self.remove_to_graveyard_with_triggers(id);
                    events.append(&mut die_evs);
                }
                Ok(())
            }

            Effect::SacrificeSourceUnlessSacrifice { filter } => {
                // CR 701.16 — "Sacrifice this permanent unless you sacrifice a
                // [filter]" (The Gitrog Monster's upkeep). The controller may
                // spare the source by sacrificing one matching permanent; with
                // no candidate (or a UI seat declining) the source is
                // sacrificed instead.
                let Some(src) = ctx.source else { return Ok(()); };
                let p = ctx.controller;
                let candidates = self.sacrifice_candidates(p, filter, Some(src));
                let spare = if candidates.is_empty() {
                    false
                } else if self.players[p].wants_ui {
                    matches!(
                        self.decider.decide(&crate::decision::Decision::OptionalTrigger {
                            source: src,
                            description: "Sacrifice a permanent to keep this one?".into(),
                        }),
                        crate::decision::DecisionAnswer::Bool(true)
                    )
                } else {
                    // Bots keep the source by paying the weakest candidate.
                    true
                };
                if spare {
                    let pick = self.auto_pick_sacrifices(&candidates, 1, Some(src), false, false);
                    for id in pick {
                        self.sacrifice_one(id, p, events);
                    }
                } else {
                    self.run_effect(&Effect::SacrificeSource, ctx, events)?;
                }
                Ok(())
            }

            Effect::SacrificeGreatestMV { who, count, filter, by_power } => {
                // Pick the greatest match by mana value (or power, with
                // `by_power`). Used by Soul Shatter ("greatest mana value") and
                // Crackling Doom ("greatest power"). The sacrificing player
                // only has a choice among permanents *tied* at the greatest
                // metric — so a `wants_ui` player making a single sacrifice
                // with a real tie is offered the pick; otherwise auto-pick.
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 {
                    return Ok(());
                }
                let source_id = ctx.source;
                let players: Vec<usize> = self
                    .resolve_selector(who, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                let by_power = *by_power;
                let metric = |this: &Self, id: CardId| -> i64 {
                    this.battlefield_find(id)
                        .map(|c| {
                            if by_power {
                                c.power() as i64
                            } else {
                                c.definition.cost.cmc() as i64
                            }
                        })
                        .unwrap_or(i64::MIN)
                };
                let mut deferred_ui: Option<(usize, Vec<CardId>)> = None;
                for p in players {
                    let candidates = self.sacrifice_candidates(p, filter, source_id);
                    if candidates.is_empty() {
                        continue;
                    }
                    if n == 1 && self.players[p].wants_ui && deferred_ui.is_none() {
                        let best = candidates.iter().map(|id| metric(self, *id)).max();
                        if let Some(best) = best {
                            let tied: Vec<CardId> = candidates
                                .iter()
                                .copied()
                                .filter(|id| metric(self, *id) == best)
                                .collect();
                            if tied.len() > 1 {
                                deferred_ui = Some((p, tied));
                                continue;
                            }
                        }
                    }
                    let ids = self.auto_pick_sacrifices(&candidates, n, source_id, true, by_power);
                    for id in ids {
                        self.sacrifice_one(id, p, events);
                    }
                }
                if let Some((p, tied)) = deferred_ui {
                    let options: Vec<Target> =
                        tied.iter().map(|id| Target::Permanent(*id)).collect();
                    let decision = crate::decision::Decision::ChooseTarget {
                        source: source_id.unwrap_or(crate::card::CardId(0)),
                        legal: options,
                        source_name: ctx.source_name.unwrap_or("").to_string(),
                        description: "choose a permanent to sacrifice".into(),
                    };
                    self.suspend_signal = Some((
                        decision,
                        PendingEffectState::SacrificePending { player: p },
                        Effect::Noop,
                    ));
                    return Ok(());
                }
                Ok(())
            }

            Effect::Punisher { chooser, options, otherwise } => {
                // Resolve the set of choosing players up front (the borrow of
                // `self` from resolve_selector must end before we mutate).
                let choosers: Vec<usize> = self
                    .resolve_selector(chooser, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                for p in choosers {
                    // The chooser evaluates the options with themselves as the
                    // effect controller (so `PlayerRef::You` = the chooser).
                    let opt_ctx = EffectContext { controller: p, ..ctx.clone() };
                    let picked = options
                        .iter()
                        .find(|opt| self.punisher_option_affordable(opt, &opt_ctx));
                    match picked {
                        Some(opt) => self.run_effect(opt, &opt_ctx, events)?,
                        // No affordable option — the ability's controller
                        // gets the payoff (uses the original ctx).
                        None => self.run_effect(otherwise, ctx, events)?,
                    }
                }
                Ok(())
            }

            Effect::AddPoison { who, amount } => {
                let n = self.evaluate_value(amount, ctx).max(0) as u32;
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].poison_counters += n;
                        events.push(GameEvent::PoisonAdded { player: p, amount: n });
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::AddRadCounters { who, amount } => {
                let n = self.evaluate_value(amount, ctx).max(0) as u32;
                if n == 0 { return Ok(()); }
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].rad_counters =
                            self.players[p].rad_counters.saturating_add(n);
                    }
                }
                Ok(())
            }

            Effect::Move { what, to } => {
                for ent in self.resolve_selector(what, ctx) {
                    let cid = match ent {
                        EntityRef::Permanent(c) | EntityRef::Card(c) => c,
                        _ => continue,
                    };
                    self.move_card_to(cid, to, ctx, events);
                    // Stash the moved id so a downstream
                    // `Selector::LastMoved` in the same Seq can target
                    // it (Practiced Scrollsmith's Move → GrantMayPlay
                    // chain).
                    self.last_moved_cards.push(cid);
                }
                Ok(())
            }

            Effect::PutFromHandOntoBattlefield { who, filter, count, tapped, haste, sacrifice_eot } => {
                use crate::decision::{Decision, DecisionAnswer};
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let max = self.evaluate_value(count, ctx).max(0) as u32;
                if max == 0 { return Ok(()); }
                // Concretize source-counter-relative MV gates (Aether Vial's
                // "mana value equal to the number of charge counters").
                let src_counts = |kind: CounterType| -> u32 {
                    ctx.source
                        .and_then(|sid| self.battlefield_find(sid))
                        .map(|c| c.counter_count(kind))
                        .unwrap_or(0)
                };
                let filter = filter.resolve_source_counters(&src_counts);
                let candidates: Vec<(CardId, String)> = self.players[p]
                    .hand
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(&filter, c, p))
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                if candidates.is_empty() { return Ok(()); }
                let source = ctx.source.unwrap_or(CardId(0));
                // Always optional ("you may"): min 0.
                let answer = self.decider.decide(&Decision::ChooseCards {
                    source,
                    prompt: "Put which card(s) from your hand onto the battlefield?".to_string(),
                    candidates,
                    min: 0,
                    max,
                });
                let chosen: Vec<CardId> = match answer { DecisionAnswer::Cards(v) => v, _ => vec![] };
                let dest = ZoneDest::Battlefield { controller: PlayerRef::You, tapped: *tapped };
                for cid in chosen {
                    // Only move cards that are still in the hand and match.
                    if !self.players[p].hand.iter().any(|c| c.id == cid) { continue; }
                    self.move_card_to(cid, &dest, ctx, events);
                    if *haste {
                        self.grant_keyword_eot(cid, Keyword::Haste);
                    }
                    if *sacrifice_eot {
                        self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                            controller: ctx.controller,
                            source: cid,
                            kind: crate::game::types::DelayedKind::NextEndStep,
                            effect: Effect::SacrificeSource,
                            target: None,
                            bound_token: None,
                            fires_once: true,
                        });
                    }
                }
                Ok(())
            }

            Effect::ManifestFromHand { who, count, controller_draws } => self.resolve_manifest_from_hand(who, count, *controller_draws, ctx, events),


            Effect::CreateTokenAttachedTo { target, definition } => self.resolve_create_token_attached_to(target, definition, ctx, events),


            Effect::DestroyTargetsPolymorph { filter } => self.resolve_destroy_targets_polymorph(filter, ctx, events),

            Effect::DestroyTargets { filter } => {
                // Destroy the X chosen targets (slots 0..X) matching `filter`;
                // without an {X} in the cost, every given target.
                let x = if ctx.x_value > 0 { ctx.x_value as usize } else { ctx.targets.len() };
                let mut seen = std::collections::HashSet::new();
                for (i, t) in ctx.targets.iter().enumerate().take(x) {
                    if let Target::Permanent(id) = t
                        && seen.insert(*id)
                        && self.evaluate_requirement_static(filter, t, ctx.controller, ctx.source)
                    {
                        self.run_effect(
                            &Effect::Destroy { what: crate::effect::Selector::Target(i as u8) },
                            ctx,
                            events,
                        )?;
                    }
                }
                Ok(())
            }

            Effect::Champion { filter } => self.resolve_champion(filter, ctx, events),

            Effect::ExileUpToNFromGraveyards { count } => {
                self.resolve_exile_up_to_n_from_graveyards(count, ctx, events)
            }

            Effect::ExileTopMintPerChosenColor { who, amount, token } => {
                self.resolve_exile_top_mint_per_chosen_color(who, amount, token, ctx, events)
            }

            Effect::SpellTaxUntilYourNextTurn { amount, filter } => {
                self.turn_scoped_spell_taxes.push(crate::game::TurnScopedSpellTax {
                    controller: ctx.controller,
                    amount: *amount,
                    filter: filter.clone(),
                });
                Ok(())
            }

            Effect::SacrificeAllButOnePerType { who } => self.resolve_sacrifice_all_but_one_per_type(who, ctx, events),


            Effect::WishToHand { filter } => self.resolve_wish_to_hand(filter, ctx, events),


            Effect::Manifest { who, amount } => {
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(amount, ctx).max(0) as u32;
                for _ in 0..n {
                    let Some(top_id) = self.players[p].library.first().map(|c| c.id) else { break };
                    self.manifest_card(top_id, p, ctx, events);
                }
                Ok(())
            }

            Effect::Cloak { who, amount } => {
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(amount, ctx).max(0) as u32;
                for _ in 0..n {
                    let Some(top_id) = self.players[p].library.first().map(|c| c.id) else { break };
                    if let Some(c) = self.players[p].library.iter_mut().find(|c| c.id == top_id) {
                        c.cloaked = true;
                    }
                    self.manifest_card(top_id, p, ctx, events);
                }
                Ok(())
            }

            Effect::ManifestDread { who } => {
                use crate::decision::{Decision, DecisionAnswer};
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                // Look at the top two cards; choose one to manifest, the other
                // goes to the graveyard. With one card left, manifest it.
                let top: Vec<(CardId, String)> = self.players[p]
                    .library
                    .iter()
                    .take(2)
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                if top.is_empty() { return Ok(()); }
                let chosen = if top.len() == 1 {
                    top[0].0
                } else {
                    let source = ctx.source.unwrap_or(CardId(0));
                    let answer = self.decider.decide(&Decision::ChooseCards {
                        source,
                        prompt: "Manifest which card? (the other goes to your graveyard)".to_string(),
                        candidates: top.clone(),
                        min: 1,
                        max: 1,
                    });
                    match answer {
                        DecisionAnswer::Cards(v) if !v.is_empty() => v[0],
                        _ => top[0].0,
                    }
                };
                let other = top.iter().map(|(id, _)| *id).find(|id| *id != chosen);
                self.manifest_card(chosen, p, ctx, events);
                if let Some(other_id) = other {
                    self.move_card_to(other_id, &ZoneDest::Graveyard, ctx, events);
                }
                Ok(())
            }

            e @ (Effect::Search { .. } | Effect::SearchPickedBy { .. }) => {
                use crate::decision::Decision;
                let (who, picker_ref, filter, to) = match e {
                    Effect::Search { who, filter, to } => (who, None, filter, to),
                    Effect::SearchPickedBy { who, picker, filter, to } => {
                        (who, Some(picker), filter, to)
                    }
                    _ => unreachable!(),
                };
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                // CR 701.19a — the picker (when distinct) makes the pick;
                // the searched library is still `p`'s.
                let picker = picker_ref
                    .and_then(|pr| self.resolve_player(pr, ctx))
                    .unwrap_or(p);

                // Leonin Arbiter — an unpayable search tax means the search
                // happens but finds nothing (CR 701.19d).
                if !self.pay_search_tax(p) {
                    return Ok(());
                }
                // CR 701.19 — `p` searched their library this turn (Archive Trap).
                self.players[p].searched_library_this_turn = true;
                // Aven Mindcensor — an opponent's search only sees the top N.
                let limit = self.search_top_limit_for(p).unwrap_or(usize::MAX);

                // Collect candidates from the library using definition-level evaluation
                // (cards are not on the battlefield so battlefield_find would fail).
                // X-dependent filters concretize against the paid X (Chord of Calling).
                let filter = filter.resolve_x(ctx.x_value).resolve_converge(ctx.converged_value);
                let candidates: Vec<(crate::card::CardId, String)> = self.players[p]
                    .library
                    .iter()
                    .take(limit)
                    .filter(|c| self.evaluate_requirement_on_card(&filter, c, p))
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();

                let eligible: Option<Vec<crate::card::CardId>> =
                    Some(candidates.iter().map(|(id, _)| *id).collect());
                let decision = Decision::SearchLibrary {
                    player: picker,
                    candidates,
                    eligible: eligible.clone(),
                };
                let pending =
                    PendingEffectState::SearchPending { player: p, to: to.clone(), eligible };

                if self.players[picker].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }

                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::RedirectSpellTargetToSelf { what } => {
                let Some(src) = ctx.source else { return Ok(()) };
                // Locate the targeted spell on the stack.
                let spell_id = self
                    .resolve_selector(what, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Card(cid) | EntityRef::Permanent(cid) => Some(cid),
                        _ => None,
                    });
                let Some(spell_id) = spell_id else { return Ok(()) };
                let Some(idx) = self.stack.iter().rposition(|si| {
                    matches!(si, StackItem::Spell { card, .. } if card.id == spell_id)
                }) else {
                    // CR 115.7 — "spell or ability": retarget a targeted
                    // triggered/activated ability whose source is the
                    // selected permanent (topmost if several).
                    if let Some(tidx) = self.stack.iter().rposition(|si| matches!(
                        si,
                        StackItem::Trigger { source, target: Some(_), .. } if *source == spell_id
                    )) {
                        let legal = if let StackItem::Trigger { effect, controller, .. } =
                            &self.stack[tidx]
                        {
                            effect
                                .target_filter_for_slot(0)
                                .is_none_or(|f| {
                                    self.evaluate_requirement_static(
                                        f,
                                        &Target::Permanent(src),
                                        *controller,
                                        Some(spell_id),
                                    )
                                })
                                && self
                                    .check_target_legality_with_source(
                                        &Target::Permanent(src),
                                        ctx.controller,
                                        Some(spell_id),
                                    )
                                    .is_ok()
                        } else {
                            false
                        };
                        if legal
                            && let StackItem::Trigger { target, .. } = &mut self.stack[tidx]
                        {
                            *target = Some(Target::Permanent(src));
                        }
                    }
                    return Ok(());
                };
                // CR 115.7 — the new target must be legal for that spell.
                let (legal, caster) = if let StackItem::Spell { card, caster, .. } = &self.stack[idx] {
                    let filter_ok = card
                        .definition
                        .effect
                        .target_filter_for_slot_in_mode_kicked(0, None, card.kicked)
                        .is_none_or(|f| {
                            self.evaluate_requirement_static(
                                f,
                                &Target::Permanent(src),
                                *caster,
                                Some(card.id),
                            )
                        });
                    (
                        filter_ok
                            && self
                                .check_target_legality_with_source(
                                    &Target::Permanent(src),
                                    *caster,
                                    Some(card.id),
                                )
                                .is_ok(),
                        *caster,
                    )
                } else {
                    return Ok(());
                };
                let _ = caster;
                if legal
                    && let StackItem::Spell { target, .. } = &mut self.stack[idx]
                    && target.is_some()
                {
                    *target = Some(Target::Permanent(src));
                }
                Ok(())
            }

            Effect::SearchSplitOpponentChooses {
                opponent, count, opponent_picks, chosen_to, rest_to,
            } => {
                use crate::decision::Decision;
                let p = ctx.controller;
                let opp = self
                    .resolve_selector(opponent, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Player(o) => Some(o),
                        _ => None,
                    })
                    .or_else(|| self.opponents_of(p).into_iter().next());
                let Some(opp) = opp else { return Ok(()) };
                // CR 701.19 — the searcher picks up to `count` cards with
                // different names.
                let candidates: Vec<(crate::card::CardId, String)> = self.players[p]
                    .library
                    .iter()
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                let answer = self.decider.decide(&Decision::ChooseCards {
                    source: ctx.source.unwrap_or(CardId(0)),
                    prompt: format!("Search for up to {count} cards with different names"),
                    candidates: candidates.clone(),
                    min: 0,
                    max: *count,
                });
                let mut picked: Vec<crate::card::CardId> = Vec::new();
                let mut seen_names: Vec<String> = Vec::new();
                if let crate::decision::DecisionAnswer::Cards(ids) = answer {
                    for id in ids {
                        let Some((_, name)) = candidates.iter().find(|(c, _)| *c == id) else {
                            continue;
                        };
                        if seen_names.contains(name) || picked.len() >= *count as usize {
                            continue;
                        }
                        seen_names.push(name.clone());
                        picked.push(id);
                    }
                }
                self.players[p].searched_library_this_turn = true;
                if picked.is_empty() {
                    use rand::seq::SliceRandom;
                    self.players[p].library.shuffle(&mut rand::rng());
                    return Ok(());
                }
                // The opponent splits the revealed pile.
                let n_chosen = (*opponent_picks as usize).min(picked.len());
                let revealed: Vec<(crate::card::CardId, String)> = picked
                    .iter()
                    .map(|id| {
                        (*id, candidates.iter().find(|(c, _)| c == id).unwrap().1.clone())
                    })
                    .collect();
                let answer = self.decider.decide(&Decision::ChooseCards {
                    source: ctx.source.unwrap_or(CardId(0)),
                    prompt: format!("Opponent: choose {n_chosen} to put into the graveyard"),
                    candidates: revealed,
                    min: n_chosen as u32,
                    max: n_chosen as u32,
                });
                let mut chosen: Vec<crate::card::CardId> = match answer {
                    crate::decision::DecisionAnswer::Cards(ids) => ids
                        .into_iter()
                        .filter(|id| picked.contains(id))
                        .take(n_chosen)
                        .collect(),
                    _ => Vec::new(),
                };
                let _ = opp; // opponent's choice rides the shared decider
                while chosen.len() < n_chosen {
                    let next = picked.iter().find(|id| !chosen.contains(id)).copied();
                    match next {
                        Some(id) => chosen.push(id),
                        None => break,
                    }
                }
                for id in picked {
                    let dest = if chosen.contains(&id) { chosen_to } else { rest_to };
                    self.move_card_to(id, dest, ctx, events);
                }
                {
                    use rand::seq::SliceRandom;
                    self.players[p].library.shuffle(&mut rand::rng());
                }
                Ok(())
            }

            Effect::LookPickToHand { who, count, rest_to_graveyard, pick_filter, take, to_battlefield } => {
                use crate::decision::Decision;
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let take = take
                    .as_ref()
                    .map(|v| self.evaluate_value(v, ctx).max(1) as usize)
                    .unwrap_or(1);
                let top_ids: Vec<crate::card::CardId> =
                    self.players[p].library.iter().take(n).map(|c| c.id).collect();
                if top_ids.is_empty() {
                    return Ok(());
                }
                // Eligible-to-take set: filtered by `pick_filter` when present
                // (Satyr Wayfinder — lands only). `revealed` keeps all top-N
                // for the rest-to-graveyard sweep.
                let eligible: Option<Vec<crate::card::CardId>> = pick_filter.as_ref().map(|f| {
                    top_ids
                        .iter()
                        .copied()
                        .filter(|id| {
                            self.evaluate_requirement_static(f, &Target::Permanent(*id), p, ctx.source)
                        })
                        .collect()
                });
                // Show every revealed card; the UI greys the non-eligible
                // ones (only `eligible` picks are accepted).
                let candidates: Vec<(crate::card::CardId, String)> = top_ids
                    .iter()
                    .filter_map(|id| {
                        self.players[p]
                            .library
                            .iter()
                            .find(|c| c.id == *id)
                            .map(|c| (*id, c.definition.name.to_string()))
                    })
                    .collect();
                let decision = Decision::SearchLibrary {
                    player: p,
                    candidates,
                    eligible: eligible.clone(),
                };
                let pending = PendingEffectState::ImpulsePending {
                    player: p,
                    revealed: top_ids,
                    rest_to_graveyard: *rest_to_graveyard,
                    eligible,
                    take,
                    to_battlefield: *to_battlefield,
                };
                if self.players[p].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }
                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::Hideaway { count } => {
                // CR 702.76 — look at the top N, exile the best (highest-MV)
                // face down linked to the source, bottom the rest at random.
                let Some(src) = ctx.source else { return Ok(()); };
                let p = ctx.controller;
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let top: Vec<crate::card::CardId> =
                    self.players[p].library.iter().take(n).map(|c| c.id).collect();
                if top.is_empty() {
                    return Ok(());
                }
                let pick = top
                    .iter()
                    .copied()
                    .max_by_key(|id| {
                        self.players[p]
                            .library
                            .iter()
                            .find(|c| c.id == *id)
                            .map(|c| c.definition.cost.cmc())
                            .unwrap_or(0)
                    })
                    .unwrap();
                if let Some(pos) = self.players[p].library.iter().position(|c| c.id == pick) {
                    let mut card = self.players[p].library.remove(pos);
                    card.face_down = true;
                    card.exiled_with = Some(src);
                    self.exile.push(card);
                    events.push(crate::game::GameEvent::PermanentExiled { card_id: pick });
                }
                // Bottom the remaining looked-at cards in a random order.
                use rand::seq::SliceRandom;
                let mut rest: Vec<crate::card::CardId> =
                    top.iter().copied().filter(|id| *id != pick).collect();
                rest.shuffle(&mut rand::rng());
                for id in rest {
                    if let Some(pos) = self.players[p].library.iter().position(|c| c.id == id) {
                        let card = self.players[p].library.remove(pos);
                        self.players[p].library.push(card);
                    }
                }
                Ok(())
            }

            Effect::RemoveAllCountersDiscountNextSpell { what } => {
                let mut removed = 0u32;
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        removed += c.counters.values().sum::<u32>();
                        removed += c.keyword_counters.len() as u32;
                        c.counters.clear();
                        c.keyword_counters.clear();
                    }
                }
                if removed > 0 {
                    let cast_so_far = self.players[ctx.controller].spells_cast_this_turn;
                    self.players[ctx.controller]
                        .pending_spell_discounts
                        .push((removed, cast_so_far));
                }
                Ok(())
            }

            Effect::ExileWithSource { what } => {
                let ids: Vec<CardId> = self
                    .resolve_selector(what, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        EntityRef::Card(id) | EntityRef::Permanent(id) => Some(id),
                        _ => None,
                    })
                    .collect();
                for id in ids {
                    self.move_card_to(id, &ZoneDest::Exile, ctx, events);
                    if let Some(c) = self.exile.iter_mut().find(|c| c.id == id) {
                        c.exiled_with = ctx.source;
                    }
                }
                Ok(())
            }

            Effect::TemptingOffer { body } => {
                // Offer each opponent a copy (seat-routed yes/no, so a
                // networked human actually chooses), then run the body for
                // the controller, each acceptor, and once more per acceptor.
                // All asks precede every body run so the suspend re-run is
                // idempotent; opponents therefore answer before seeing the
                // controller's result — a minor ordering approximation.
                let source = ctx.source.unwrap_or(crate::card::CardId(0));
                let mut cursor = 0;
                let mut acceptors = Vec::new();
                for opp in self.resolve_players(&crate::effect::PlayerRef::EachOpponent, ctx) {
                    let Some(yes) = self.ask_seat_bool(
                        &mut cursor,
                        opp,
                        "Accept the tempting offer?".to_string(),
                        source,
                        effect,
                    ) else {
                        return Ok(());
                    };
                    if yes {
                        acceptors.push(opp);
                    }
                }
                self.clear_answer_log();
                self.run_effect(body, ctx, events)?;
                for &opp in &acceptors {
                    let opp_ctx = EffectContext { controller: opp, ..ctx.clone() };
                    self.run_effect(body, &opp_ctx, events)?;
                }
                for _ in 0..acceptors.len() {
                    self.run_effect(body, ctx, events)?;
                }
                Ok(())
            }

            Effect::PlayersMayAccept { who, description, on_accept, otherwise } => {
                // Ask each resolved player in APNAP order (seat-routed —
                // each player answers their own offer); the first to
                // accept runs `on_accept` with themselves in slot 0 and the
                // offer closes. Nobody accepting runs `otherwise`.
                let source = ctx.source.unwrap_or(crate::card::CardId(0));
                let mut cursor = 0;
                for p in self.resolve_players(who, ctx) {
                    let Some(yes) =
                        self.ask_seat_bool(&mut cursor, p, description.clone(), source, effect)
                    else {
                        return Ok(());
                    };
                    if yes {
                        self.clear_answer_log();
                        let mut acc_ctx = ctx.clone();
                        if acc_ctx.targets.is_empty() {
                            acc_ctx.targets.push(Target::Player(p));
                        } else {
                            acc_ctx.targets[0] = Target::Player(p);
                        }
                        return self.run_effect(on_accept, &acc_ctx, events);
                    }
                }
                self.clear_answer_log();
                self.run_effect(otherwise, ctx, events)
            }

            Effect::StealCreatureEtbThisTurn => {
                let p = ctx.controller;
                if !self.creature_etb_steal_this_turn.contains(&p) {
                    self.creature_etb_steal_this_turn.push(p);
                }
                Ok(())
            }

            Effect::PutFromHandOrGraveyardOntoBattlefield { filter } => {
                use crate::decision::Decision;
                let p = ctx.controller;
                let candidates: Vec<(crate::card::CardId, String)> = self.players[p]
                    .hand
                    .iter()
                    .chain(self.players[p].graveyard.iter())
                    .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                if candidates.is_empty() {
                    return Ok(());
                }
                let decision = Decision::SearchLibrary { player: p, candidates: candidates.clone(), eligible: None };
                let pending = PendingEffectState::PutFromZonesPending { player: p };
                if self.players[p].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }
                // Auto-pick: the highest-MV match ("may" never declined).
                let pick = candidates
                    .iter()
                    .map(|(id, _)| *id)
                    .max_by_key(|id| {
                        self.find_card_anywhere(*id).map(|c| c.definition.cost.cmc()).unwrap_or(0)
                    });
                let answer = crate::decision::DecisionAnswer::Search(pick);
                let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::RevealTopToHandOpponentsLoseMv => {
                // Sorin +1: top card to hand; each opponent loses its MV.
                let p = ctx.controller;
                if self.players[p].library.is_empty() {
                    return Ok(());
                }
                let card = self.players[p].library.remove(0);
                let mv = card.definition.cost.cmc() as i32;
                events.push(GameEvent::TopCardRevealed {
                    player: p,
                    card_name: card.definition.name,
                    is_land: card.definition.is_land(),
                });
                self.players[p].hand.push(card);
                if mv > 0 {
                    for opp in self.resolve_players(&crate::effect::PlayerRef::EachOpponent, ctx) {
                        let applied = self.adjust_life_applied(opp, -mv);
                        if applied < 0 {
                            events.push(GameEvent::LifeLost { player: opp, amount: (-applied) as u32 });
                        }
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::LookTopExileOneMayPlay { count } => {
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let opp = self
                    .resolve_player(&crate::effect::PlayerRef::Target(0), ctx)
                    .or_else(|| {
                        (0..self.players.len()).find(|s| !self.same_team(*s, ctx.controller))
                    });
                let Some(opp) = opp else { return Ok(()) };
                let top: Vec<crate::card::CardId> =
                    self.players[opp].library.iter().take(n).map(|c| c.id).collect();
                if top.is_empty() {
                    return Ok(());
                }
                let pick = top
                    .iter()
                    .copied()
                    .max_by_key(|id| {
                        self.players[opp]
                            .library
                            .iter()
                            .find(|c| c.id == *id)
                            .map(|c| c.definition.cost.cmc())
                            .unwrap_or(0)
                    })
                    .unwrap();
                let pos = self.players[opp].library.iter().position(|c| c.id == pick).unwrap();
                let mut card = self.players[opp].library.remove(pos);
                card.exiled_with = ctx.source;
                card.face_down = true;
                card.may_play_until = Some(crate::card::MayPlayPermission {
                    player: ctx.controller,
                    granted_turn: self.turn_number,
                    duration: crate::card::MayPlayDuration::WhileExiled,
                    exile_after: false,
                });
                // Gonti's "spend mana as though it were mana of any type"
                // (CR 609.4b) — the pay-to-cast cost is the MV as generic.
                card.granted_alt_cast_cost_eot = Some(crate::mana::ManaCost::new(vec![
                    crate::mana::generic(card.definition.cost.cmc()),
                ]));
                let cid = card.id;
                self.exile.push(card);
                events.push(GameEvent::PermanentExiled { card_id: cid });
                // Bottom the rest in a random order.
                use rand::seq::SliceRandom;
                let mut rest: Vec<crate::card::CardId> =
                    top.into_iter().filter(|id| *id != pick).collect();
                rest.shuffle(&mut rand::rng());
                for id in rest {
                    if let Some(pos) = self.players[opp].library.iter().position(|c| c.id == id) {
                        let card = self.players[opp].library.remove(pos);
                        self.players[opp].library.push(card);
                    }
                }
                Ok(())
            }

            Effect::NameCardTargetDiscardsMatching => {
                use crate::decision::Decision;
                let who = self
                    .resolve_player(&crate::effect::PlayerRef::Target(0), ctx)
                    .or_else(|| {
                        (0..self.players.len()).find(|s| !self.same_team(*s, ctx.controller))
                    });
                let Some(who) = who else { return Ok(()) };
                // Heuristic feed: the target's nonland hand names, most
                // common first, so bots strip the densest stack.
                let suggestions = rank_names_by_frequency(
                    self.players[who]
                        .hand
                        .iter()
                        .filter(|c| !c.definition.is_land())
                        .map(|c| c.definition.name),
                );
                let decision = Decision::NameCard {
                    source: ctx.source.unwrap_or(crate::card::CardId(0)),
                    source_name: ctx.source_name.unwrap_or_default().to_string(),
                    suggestions,
                };
                let pending = PendingEffectState::NameDiscardMatchingPending { who };
                if self.players[ctx.controller].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }
                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::NameCardRevealTop { count } => {
                // Tamiyo +1: choose a nonland card name, reveal the top N —
                // matching names to hand, the rest to the graveyard.
                use crate::decision::Decision;
                let p = ctx.controller;
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let source_name = ctx
                    .source
                    .and_then(|cid| self.find_card_anywhere(cid))
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                // Heuristic feed: nonland names among the top N that will be
                // revealed, most common first.
                let suggestions = rank_names_by_frequency(
                    self.players[p]
                        .library
                        .iter()
                        .take(n)
                        .filter(|c| !c.definition.is_land())
                        .map(|c| c.definition.name),
                );
                let decision = Decision::NameCard {
                    source: ctx.source.unwrap_or(crate::card::CardId(0)),
                    source_name,
                    suggestions,
                };
                let pending = PendingEffectState::NameRevealTopPending { player: p, count: n };
                if self.players[p].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }
                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::PutExiledCreatureOntoBattlefield { mv } => {
                // Ashiok −X: a creature card with MV X exiled with the source
                // enters under your control as a Nightmare.
                let x = self.evaluate_value(mv, ctx).max(0) as u32;
                let pick = self
                    .exile
                    .iter()
                    .find(|c| {
                        ctx.source.is_some()
                            && c.exiled_with == ctx.source
                            && c.definition.is_creature()
                            && c.definition.cost.cmc() == x
                    })
                    .map(|c| c.id);
                if let Some(id) = pick {
                    let dest = ZoneDest::Battlefield {
                        controller: crate::effect::PlayerRef::Seat(ctx.controller),
                        tapped: false,
                    };
                    self.move_card_to(id, &dest, ctx, events);
                    if let Some(c) = self.battlefield_find_mut(id) {
                        let def = std::sync::Arc::make_mut(&mut c.definition);
                        if !def.subtypes.creature_types.contains(&crate::card::CreatureType::Nightmare) {
                            def.subtypes.creature_types.push(crate::card::CreatureType::Nightmare);
                        }
                    }
                }
                Ok(())
            }

            Effect::RevealOpponentTopPutOntoBattlefield { count, filter } => {
                // Lonis, Genetics Expert: target opponent (slot 0, else the
                // lowest-seat opponent) reveals the top N; put one matching
                // nonland permanent card with MV ≤ N onto the battlefield
                // under your control (auto-pick: highest MV); they shuffle.
                let n = self.evaluate_value(count, ctx).max(0);
                let opp = self
                    .resolve_player(&crate::effect::PlayerRef::Target(0), ctx)
                    .or_else(|| {
                        (0..self.players.len()).find(|s| !self.same_team(*s, ctx.controller))
                    });
                let Some(opp) = opp else { return Ok(()) };
                let revealed: Vec<crate::card::CardId> = self.players[opp]
                    .library
                    .iter()
                    .take(n as usize)
                    .map(|c| c.id)
                    .collect();
                let pick = revealed
                    .iter()
                    .copied()
                    .filter(|id| {
                        self.players[opp].library.iter().find(|c| c.id == *id).is_some_and(|c| {
                            (c.definition.cost.cmc() as i32) <= n
                        }) && self.evaluate_requirement_static(
                            filter,
                            &Target::Permanent(*id),
                            ctx.controller,
                            ctx.source,
                        )
                    })
                    .max_by_key(|id| {
                        self.players[opp]
                            .library
                            .iter()
                            .find(|c| c.id == *id)
                            .map(|c| c.definition.cost.cmc())
                            .unwrap_or(0)
                    });
                if let Some(id) = pick {
                    let dest = ZoneDest::Battlefield {
                        controller: crate::effect::PlayerRef::Seat(ctx.controller),
                        tapped: false,
                    };
                    self.move_card_to(id, &dest, ctx, events);
                }
                use rand::seq::SliceRandom;
                self.players[opp].library.shuffle(&mut rand::rng());
                Ok(())
            }

            Effect::RevealTopTakeOnePerType { who, count } => {
                use crate::card::CardType;
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let revealed: Vec<crate::card::CardId> =
                    self.players[p].library.iter().take(n).map(|c| c.id).collect();
                if revealed.is_empty() {
                    return Ok(());
                }
                // For each card type, take the first revealed (not-yet-taken)
                // card bearing that type.
                const TYPES: [CardType; 8] = [
                    CardType::Artifact, CardType::Battle, CardType::Creature,
                    CardType::Enchantment, CardType::Instant, CardType::Land,
                    CardType::Planeswalker, CardType::Sorcery,
                ];
                let mut taken: Vec<crate::card::CardId> = Vec::new();
                for ty in TYPES {
                    if let Some(id) = revealed.iter().copied().find(|id| {
                        !taken.contains(id)
                            && self.players[p].library.iter()
                                .find(|c| c.id == *id)
                                .is_some_and(|c| c.definition.card_types.contains(&ty))
                    }) {
                        taken.push(id);
                    }
                }
                // Pull taken cards into hand (preserve library order otherwise).
                for id in &taken {
                    if let Some(pos) = self.players[p].library.iter().position(|c| c.id == *id) {
                        let card = self.players[p].library.remove(pos);
                        self.players[p].hand.push(card);
                    }
                }
                // Bottom the remaining revealed cards in a random order (CR 401.4).
                use rand::seq::SliceRandom;
                let mut rest: Vec<crate::card::CardId> =
                    revealed.iter().copied().filter(|id| !taken.contains(id)).collect();
                rest.shuffle(&mut rand::rng());
                for id in rest {
                    if let Some(pos) = self.players[p].library.iter().position(|c| c.id == id) {
                        let card = self.players[p].library.remove(pos);
                        self.players[p].library.push(card);
                    }
                }
                Ok(())
            }

            Effect::RevealTopTakeMatchingToHand { who, count, filter } => {
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let revealed: Vec<crate::card::CardId> =
                    self.players[p].library.iter().take(n).map(|c| c.id).collect();
                if revealed.is_empty() { return Ok(()); }
                // Take every revealed card matching the filter into hand.
                let taken: Vec<crate::card::CardId> = revealed.iter().copied().filter(|id| {
                    self.evaluate_requirement_static(filter, &Target::Permanent(*id), p, ctx.source)
                }).collect();
                for id in &taken {
                    if let Some(pos) = self.players[p].library.iter().position(|c| c.id == *id) {
                        let card = self.players[p].library.remove(pos);
                        self.players[p].hand.push(card);
                    }
                }
                // Bottom the rest in a random order (CR 401.4).
                use rand::seq::SliceRandom;
                let mut rest: Vec<crate::card::CardId> =
                    revealed.iter().copied().filter(|id| !taken.contains(id)).collect();
                rest.shuffle(&mut rand::rng());
                for id in rest {
                    if let Some(pos) = self.players[p].library.iter().position(|c| c.id == id) {
                        let card = self.players[p].library.remove(pos);
                        self.players[p].library.push(card);
                    }
                }
                Ok(())
            }

            Effect::ExileLibraryExceptBottom { who, keep } => {
                let keep = self.evaluate_value(keep, ctx).max(0) as usize;
                for p in self.resolve_players(who, ctx) {
                    let take = self.players[p].library.len().saturating_sub(keep);
                    let ids: Vec<crate::card::CardId> =
                        self.players[p].library.iter().take(take).map(|c| c.id).collect();
                    for id in ids {
                        self.move_card_to(id, &ZoneDest::Exile, ctx, events);
                    }
                }
                Ok(())
            }

            Effect::ShuffleGraveyardIntoLibrary { who } => {
                use rand::seq::SliceRandom;
                if let Some(p) = self.resolve_player(who, ctx) {
                    let cards = std::mem::take(&mut self.players[p].graveyard);
                    self.players[p].library.extend(cards);
                    self.players[p].library.shuffle(&mut rand::rng());
                }
                Ok(())
            }

            Effect::ShuffleHandAndGraveyardIntoLibrary { who } => {
                use rand::seq::SliceRandom;
                for p in self.resolve_players(who, ctx) {
                    let hand = std::mem::take(&mut self.players[p].hand);
                    let gy = std::mem::take(&mut self.players[p].graveyard);
                    self.players[p].library.extend(hand);
                    self.players[p].library.extend(gy);
                    self.players[p].library.shuffle(&mut rand::rng());
                }
                Ok(())
            }

            Effect::ExchangeHandAndGraveyard { who } => {
                if let Some(p) = self.resolve_player(who, ctx) {
                    let (hand, gy) = (
                        std::mem::take(&mut self.players[p].hand),
                        std::mem::take(&mut self.players[p].graveyard),
                    );
                    // Hand cards → graveyard; graveyard cards → hand.
                    self.players[p].graveyard = hand;
                    self.players[p].hand = gy;
                }
                Ok(())
            }

            Effect::ShuffleLibrary { who } => {
                use rand::seq::SliceRandom;
                if let Some(p) = self.resolve_player(who, ctx) {
                    self.players[p].library.shuffle(&mut rand::rng());
                }
                Ok(())
            }

            Effect::ShuffleSelfIntoLibrary => {
                // Flag the post-resolution routing (resolve_spell) to send the
                // resolving spell to its owner's library + shuffle, rather than
                // the graveyard. No-op for non-spell sources.
                self.shuffle_resolving_spell_into_library = true;
                Ok(())
            }

            Effect::ReturnResolvingSpellToHand => {
                self.return_resolving_spell_to_hand = true;
                Ok(())
            }

            Effect::ExileResolvingSpell => {
                self.exile_resolving_spell = true;
                Ok(())
            }

            Effect::EndTheTurn => {
                self.end_turn_requested = true;
                Ok(())
            }

            Effect::SilencePlayersThisTurn { who } => {
                for p in self.resolve_players(who, ctx) {
                    self.players[p].silenced_this_turn = true;
                }
                Ok(())
            }

            Effect::RevealTopOpponentChoosesToHand { count, counter } => {
                let p = ctx.controller;
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let revealed: Vec<crate::card::CardId> =
                    self.players[p].library.iter().take(n).map(|c| c.id).collect();
                if revealed.is_empty() { return Ok(()); }
                // The opponent chooses which card goes to the controller's
                // hand. Heuristic (no interactive prompt, like `Punisher`):
                // give the controller the lowest-mana-value card and exile
                // the rest.
                let to_hand = revealed
                    .iter()
                    .copied()
                    .min_by_key(|id| {
                        self.players[p].library.iter()
                            .find(|c| c.id == *id)
                            .map(|c| c.definition.cost.cmc())
                            .unwrap_or(0)
                    })
                    .unwrap();
                self.move_card_to(to_hand, &ZoneDest::Hand(PlayerRef::You), ctx, events);
                for id in revealed.into_iter().filter(|id| *id != to_hand) {
                    self.move_card_to(id, &ZoneDest::Exile, ctx, events);
                    if let Some(kind) = counter
                        && let Some(c) = self.exile.iter_mut().find(|c| c.id == id) {
                            c.add_counters(*kind, 1);
                        }
                }
                Ok(())
            }

            Effect::BecomeMonarch { who } => {
                if let Some(p) = self.resolve_player(who, ctx) {
                    self.set_monarch(p, events);
                }
                Ok(())
            }

            Effect::BecomeDay => {
                self.set_day_night(crate::game::types::DayNight::Day, events);
                Ok(())
            }
            Effect::BecomeNight => {
                self.set_day_night(crate::game::types::DayNight::Night, events);
                Ok(())
            }

            Effect::Ascend { who } => {
                // CR 702.131 — get the city's blessing if `who` controls ten
                // or more permanents (once obtained it's permanent).
                if let Some(p) = self.resolve_player(who, ctx)
                    && !self.players[p].city_blessing
                    && self.battlefield.iter().filter(|c| c.controller == p).count() >= 10 {
                        self.players[p].city_blessing = true;
                        events.push(GameEvent::CityBlessingGained { player: p });
                    }
                Ok(())
            }

            Effect::ReturnFromExileWithCounter { counter } => {
                let p = ctx.controller;
                // Highest-value qualifying card (owned by p, has the counter).
                let pick = self.exile.iter()
                    .filter(|c| c.owner == p && c.counter_count(*counter) > 0)
                    .max_by_key(|c| c.definition.cost.cmc())
                    .map(|c| c.id);
                if let Some(id) = pick {
                    if let Some(c) = self.exile.iter_mut().find(|c| c.id == id) {
                        c.remove_counters(*counter, u32::MAX);
                    }
                    self.move_card_to(id, &ZoneDest::Hand(PlayerRef::You), ctx, events);
                }
                Ok(())
            }

            Effect::StudyTopCard { counter } => {
                let p = ctx.controller;
                let Some(top) = self.players[p].library.first().map(|c| c.id) else {
                    return Ok(());
                };
                let is_land = self.players[p].library.first()
                    .map(|c| c.definition.is_land())
                    .unwrap_or(false);
                if is_land {
                    self.move_card_to(top, &ZoneDest::Hand(PlayerRef::You), ctx, events);
                } else {
                    self.move_card_to(top, &ZoneDest::Exile, ctx, events);
                    if let Some(c) = self.exile.iter_mut().find(|c| c.id == top) {
                        c.add_counters(*counter, 1);
                    }
                }
                Ok(())
            }

            Effect::ExileTopWithCounters { count, counter } => {
                let p = ctx.controller;
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let ids: Vec<crate::card::CardId> =
                    self.players[p].library.iter().take(n).map(|c| c.id).collect();
                for id in ids {
                    self.move_card_to(id, &ZoneDest::Exile, ctx, events);
                    if let Some(c) = self.exile.iter_mut().find(|c| c.id == id) {
                        c.add_counters(*counter, 1);
                    }
                }
                Ok(())
            }

            Effect::ExileFromHandTaxed { from, count, filter, extra_cost } => {
                use crate::decision::Decision;
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 { return Ok(()); }
                let picker = ctx.controller;
                let seats: Vec<usize> = self
                    .resolve_selector(from, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                for (i, target_player) in seats.iter().copied().enumerate() {
                    let candidates: Vec<(crate::card::CardId, String)> = self.players[target_player]
                        .hand
                        .iter()
                        .filter(|c| self.evaluate_requirement_on_card(filter, c, picker))
                        .map(|c| (c.id, c.definition.name.to_string()))
                        .collect();
                    if candidates.is_empty() { continue; }
                    let decision = Decision::Discard { player: picker, count: n as u32, hand: candidates };
                    let pending = PendingEffectState::ExileFromHandTaxedPending {
                        target_player,
                        extra_cost: *extra_cost,
                    };
                    if self.players[picker].wants_ui {
                        let rest = per_seat_continuation(&seats[i + 1..], |q| {
                            Effect::ExileFromHandTaxed {
                                from: Selector::Player(crate::effect::PlayerRef::Seat(q)),
                                count: crate::effect::Value::Const(n as i32),
                                filter: filter.clone(),
                                extra_cost: *extra_cost,
                            }
                        });
                        self.suspend_signal = Some((decision, pending, rest));
                        return Ok(());
                    }
                    let answer = self.decider.decide(&decision);
                    let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                    events.append(&mut applied);
                }
                Ok(())
            }

            Effect::NthResolutionThisTurn { branches } => {
                let p = ctx.controller;
                let n = self.players[p].escalating_resolutions_this_turn as usize;
                self.players[p].escalating_resolutions_this_turn =
                    self.players[p].escalating_resolutions_this_turn.saturating_add(1);
                if let Some(branch) = branches.get(n) {
                    let branch = branch.clone();
                    self.run_effect(&branch, ctx, events)?;
                }
                Ok(())
            }

            Effect::CatchUpBasicLands => {
                use crate::card::Supertype;
                use rand::seq::SliceRandom;
                let n = self.players.len();
                let land_counts: Vec<usize> = (0..n)
                    .map(|p| {
                        self.battlefield
                            .iter()
                            .filter(|c| c.controller == p && c.definition.is_land())
                            .count()
                    })
                    .collect();
                let max = land_counts.iter().copied().max().unwrap_or(0);
                for (p, &count) in land_counts.iter().enumerate() {
                    let deficit = max.saturating_sub(count);
                    if deficit == 0 {
                        continue;
                    }
                    self.players[p].searched_library_this_turn = true;
                    let ids: Vec<crate::card::CardId> = self.players[p]
                        .library
                        .iter()
                        .filter(|c| {
                            c.definition.is_land()
                                && c.definition.supertypes.contains(&Supertype::Basic)
                        })
                        .take(deficit)
                        .map(|c| c.id)
                        .collect();
                    for id in ids {
                        self.move_card_to(
                            id,
                            &ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: true },
                            ctx,
                            events,
                        );
                    }
                    // Searching a library always shuffles it (CR 701.19c).
                    self.players[p].library.shuffle(&mut rand::rng());
                }
                Ok(())
            }

            Effect::HoneFromHand { count } => {
                // Uvilda — the owner may exile an instant/sorcery from hand
                // with `count` hone counters. Reuses the Discard decision shape.
                use crate::decision::Decision;
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                let p = ctx.controller;
                let candidates: Vec<(crate::card::CardId, String)> = self.players[p]
                    .hand
                    .iter()
                    .filter(|c| c.definition.is_instant() || c.definition.is_sorcery())
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                if candidates.is_empty() { return Ok(()); }
                // "You may" — count 0..=1; the auto-decider may decline.
                let decision = Decision::Discard { player: p, count: 1, hand: candidates };
                let pending = PendingEffectState::HoneFromHandPending { target_player: p, count: n };
                if self.players[p].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }
                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::Attach { what, to } => {
                let anchor = self.resolve_selector(to, ctx)
                    .into_iter()
                    .find_map(|e| e.as_permanent_id());
                for ent in self.resolve_selector(what, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find_mut(cid) {
                            c.attached_to = anchor;
                            events.push(GameEvent::AttachmentMoved { attachment: cid, attached_to: anchor });
                        }
                }
                Ok(())
            }

            Effect::PutOnLibraryFromHand { who, count } => {
                use crate::decision::Decision;
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let hand_snapshot: Vec<(crate::card::CardId, String)> =
                    self.players[p].hand.iter().map(|c| (c.id, c.definition.name.to_string())).collect();
                let actual = n.min(hand_snapshot.len());
                if actual == 0 { return Ok(()); }

                let decision = Decision::PutOnLibrary { player: p, count: actual, hand: hand_snapshot.clone() };
                let pending = PendingEffectState::PutOnLibraryPending { player: p, count: actual };

                if self.players[p].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }
                // Bot: auto-pick first N cards.
                let chosen: Vec<crate::card::CardId> =
                    hand_snapshot.iter().take(actual).map(|(id, _)| *id).collect();
                self.execute_put_on_library(p, &chosen, events);
                Ok(())
            }

            Effect::RevealTopAndDrawIf { who, reveal_filter, may_graveyard_miss } => {
                // Each resolved player reveals the top card of their library;
                // if it matches `reveal_filter`, that player puts it into
                // their hand (otherwise it stays on top).
                //
                // CR 121.5 compliance — "If an effect moves cards from a
                // player's library to that player's hand without using
                // the word 'draw,' the player has not drawn those cards.
                // This makes a difference for abilities that trigger on
                // drawing and effects that count cards drawn." Goblin
                // Guide and its kin say "puts it into their hand", not
                // "draws"; so we do NOT emit a `CardDrawn` event here
                // and do NOT increment `cards_drawn_this_turn`. A
                // dedicated `CardPutIntoHand` event would let cards
                // listen to *all* library→hand moves, but no current
                // card needs that; if/when one lands, add the event
                // here in front of the silent move.
                let mut cursor = 0;
                for p in self.resolve_players(who, ctx) {
                    let Some(top) = self.players[p].library.first() else {
                        continue;
                    };
                    let card_name = top.definition.name;
                    let is_land = top.definition.is_land();
                    // `evaluate_requirement_on_card` works on any
                    // `CardInstance` (the battlefield-only variant would fail
                    // here since the card is in the library).
                    let matches =
                        self.evaluate_requirement_on_card(reveal_filter, top, ctx.controller);
                    // On a miss the revealer may bin the card instead of
                    // leaving it on top (Nylea, Keen-Eyed). Seat-routed; the
                    // only carded users resolve a single player, so the
                    // pre-ask reveal event can't double-emit on re-run.
                    let bin = !matches
                        && *may_graveyard_miss
                        && match self.ask_seat_bool(
                            &mut cursor,
                            p,
                            format!("Revealed {card_name} — put it into your graveyard?"),
                            ctx.source.unwrap_or(CardId(0)),
                            effect,
                        ) {
                            Some(b) => b,
                            None => return Ok(()),
                        };
                    events.push(GameEvent::TopCardRevealed {
                        player: p,
                        card_name,
                        is_land,
                    });
                    if matches {
                        let card = self.players[p].library.remove(0);
                        self.players[p].hand.push(card);
                        // Intentionally no CardDrawn event (CR 121.5).
                    } else if bin {
                        let card = self.players[p].library.remove(0);
                        self.route_to_graveyard(card, events);
                    }
                }
                self.clear_answer_log();
                Ok(())
            }

            Effect::RevealTopCard { who } => {
                for p in self.resolve_players(who, ctx) {
                    let Some(top) = self.players[p].library.first() else { continue };
                    events.push(GameEvent::TopCardRevealed {
                        player: p,
                        card_name: top.definition.name,
                        is_land: top.definition.is_land(),
                    });
                }
                Ok(())
            }

            Effect::RevealTopPutPermanentOntoBattlefield { who } => {
                for p in self.resolve_players(who, ctx) {
                    let Some(top) = self.players[p].library.first() else { continue };
                    let (cid, name, is_land, is_perm) = (
                        top.id, top.definition.name, top.definition.is_land(),
                        top.definition.is_permanent(),
                    );
                    events.push(GameEvent::TopCardRevealed { player: p, card_name: name, is_land });
                    if is_perm {
                        self.move_card_to(
                            cid,
                            &ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: false },
                            ctx,
                            events,
                        );
                    }
                }
                Ok(())
            }

            Effect::RevealTopPutPermanentMvElseHand { who, max_mv } => {
                let cap = self.evaluate_value(max_mv, ctx).max(0) as u32;
                for p in self.resolve_players(who, ctx) {
                    let Some(top) = self.players[p].library.first() else { continue };
                    let (cid, name, is_land, is_perm, mv) = (
                        top.id, top.definition.name, top.definition.is_land(),
                        top.definition.is_permanent(), top.definition.cost.cmc(),
                    );
                    events.push(GameEvent::TopCardRevealed { player: p, card_name: name, is_land });
                    let dest = if is_perm && mv <= cap {
                        ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: false }
                    } else {
                        ZoneDest::Hand(PlayerRef::Seat(p))
                    };
                    self.move_card_to(cid, &dest, ctx, events);
                }
                Ok(())
            }

            Effect::RevealTopLandToBattlefieldElseHand { who } => {
                for p in self.resolve_players(who, ctx) {
                    let Some(top) = self.players[p].library.first() else { continue };
                    let (cid, name, is_land) =
                        (top.id, top.definition.name, top.definition.is_land());
                    events.push(GameEvent::TopCardRevealed { player: p, card_name: name, is_land });
                    let dest = if is_land {
                        ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: false }
                    } else {
                        ZoneDest::Hand(PlayerRef::Seat(p))
                    };
                    self.move_card_to(cid, &dest, ctx, events);
                }
                Ok(())
            }

            Effect::DiscardChosen { from, count, filter } => {
                // Resolve target player(s) — usually one opponent. For each,
                // the **caster** picks `count` cards matching `filter`
                // from that player's hand. When the caster has `wants_ui`,
                // the engine suspends with a `Decision::Discard` so the
                // human picks; otherwise the decider is invoked
                // synchronously (AutoDecider takes the first matching).
                use crate::decision::Decision;
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 { return Ok(()); }
                let picker = ctx.controller;
                let seats: Vec<usize> = self
                    .resolve_selector(from, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                for (i, target_player) in seats.iter().copied().enumerate() {
                    let candidates: Vec<(crate::card::CardId, String)> = self
                        .players[target_player]
                        .hand
                        .iter()
                        .filter(|c| self.evaluate_requirement_on_card(filter, c, picker))
                        .map(|c| (c.id, c.definition.name.to_string()))
                        .collect();
                    if candidates.is_empty() {
                        continue;
                    }
                    let decision = Decision::Discard {
                        player: picker,
                        count: n as u32,
                        hand: candidates,
                    };
                    let pending = PendingEffectState::DiscardChosenPending { target_player };

                    if self.players[picker].wants_ui {
                        let rest = per_seat_continuation(&seats[i + 1..], |q| Effect::DiscardChosen {
                            from: Selector::Player(crate::effect::PlayerRef::Seat(q)),
                            count: crate::effect::Value::Const(n as i32),
                            filter: filter.clone(),
                        });
                        self.suspend_signal = Some((decision, pending, rest));
                        return Ok(());
                    }
                    let answer = self.decider.decide(&decision);
                    let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                    events.append(&mut applied);
                }
                Ok(())
            }

            Effect::ExileChosenUntilSourceLeaves { from, count, filter, return_to } => {
                // CR 603.6e — same caster-picks-from-hand shape as
                // DiscardChosen, but the chosen card is exiled and linked to
                // the ability's source instead of discarded.
                use crate::decision::Decision;
                let Some(source) = ctx.source else { return Ok(()); };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 { return Ok(()); }
                let picker = ctx.controller;
                let seats: Vec<usize> = self
                    .resolve_selector(from, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                for (i, target_player) in seats.iter().copied().enumerate() {
                    let candidates: Vec<(crate::card::CardId, String)> = self
                        .players[target_player]
                        .hand
                        .iter()
                        .filter(|c| self.evaluate_requirement_on_card(filter, c, picker))
                        .map(|c| (c.id, c.definition.name.to_string()))
                        .collect();
                    if candidates.is_empty() { continue; }
                    let decision = Decision::Discard {
                        player: picker,
                        count: n as u32,
                        hand: candidates,
                    };
                    let pending = PendingEffectState::ExileChosenUntilSourceLeavesPending {
                        target_player,
                        source,
                        return_to: *return_to,
                    };
                    if self.players[picker].wants_ui {
                        let rest = per_seat_continuation(&seats[i + 1..], |q| Effect::ExileChosenUntilSourceLeaves {
                            from: Selector::Player(crate::effect::PlayerRef::Seat(q)),
                            count: crate::effect::Value::Const(n as i32),
                            filter: filter.clone(),
                            return_to: *return_to,
                        });
                        self.suspend_signal = Some((decision, pending, rest));
                        return Ok(());
                    }
                    let answer = self.decider.decide(&decision);
                    let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                    events.append(&mut applied);
                }
                Ok(())
            }

            Effect::ExileChosenFromHand { from, count, filter } => {
                // Same caster-picks-from-hand shape as DiscardChosen, but the
                // chosen card is exiled permanently (Thought-Knot Seer).
                use crate::decision::Decision;
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 { return Ok(()); }
                let picker = ctx.controller;
                let seats: Vec<usize> = self
                    .resolve_selector(from, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                for (i, target_player) in seats.iter().copied().enumerate() {
                    let candidates: Vec<(crate::card::CardId, String)> = self
                        .players[target_player]
                        .hand
                        .iter()
                        .filter(|c| self.evaluate_requirement_on_card(filter, c, picker))
                        .map(|c| (c.id, c.definition.name.to_string()))
                        .collect();
                    if candidates.is_empty() { continue; }
                    let decision = Decision::Discard {
                        player: picker,
                        count: n as u32,
                        hand: candidates,
                    };
                    let pending = PendingEffectState::ExileChosenFromHandPending { target_player };
                    if self.players[picker].wants_ui {
                        let rest = per_seat_continuation(&seats[i + 1..], |q| Effect::ExileChosenFromHand {
                            from: Selector::Player(crate::effect::PlayerRef::Seat(q)),
                            count: crate::effect::Value::Const(n as i32),
                            filter: filter.clone(),
                        });
                        self.suspend_signal = Some((decision, pending, rest));
                        return Ok(());
                    }
                    let answer = self.decider.decide(&decision);
                    let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                    events.append(&mut applied);
                }
                Ok(())
            }

            Effect::WithSacrificedPt { power, toughness, body } => {
                self.sacrificed_power = Some(*power);
                self.sacrificed_toughness = Some(*toughness);
                self.run_effect(body, ctx, events)
            }

            Effect::SacrificeAndRemember { who, filter } => {
                // Resolve `who` to a single player; pick one of their
                // controlled permanents matching `filter`; sacrifice it and
                // record its power + toughness on `state.sacrificed_power`
                // / `state.sacrificed_toughness` so a subsequent
                // `Value::SacrificedPower` / `Value::SacrificedToughness`
                // can reference it (Thud, Tribute to Hunger).
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let candidate = self
                    .battlefield
                    .iter()
                    .find(|c| {
                        c.controller == p
                            && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, ctx.source)
                    })
                    .map(|c| (c.id, c.power(), c.toughness(), c.definition.cost.cmc()));
                if let Some((cid, power, toughness, mv)) = candidate {
                    self.sacrificed_power = Some(power);
                    self.sacrificed_toughness = Some(toughness);
                    self.sacrificed_mana_value = Some(mv);
                    let is_creature = self
                        .battlefield_find(cid)
                        .map(|c| c.definition.is_creature())
                        .unwrap_or(false);
                    if is_creature {
                        if let Some(c) = self.battlefield_find(cid) {
                            self.died_card_snapshots.insert(cid, c.clone());
                        }
                        events.push(GameEvent::CreatureSacrificed { card_id: cid, who: p });
                        events.push(GameEvent::CreatureDied { card_id: cid });
                    }
                    events.push(GameEvent::PermanentSacrificed { card_id: cid, who: p });
                    // CR 700.4 — through the rich funnel so dies/LTB triggers
                    // and Persist/Undying fire (Kitchen Finks sacrificed to
                    // Tribute to Hunger).
                    let mut die = self.remove_to_graveyard_with_triggers(cid);
                    events.append(&mut die);
                }
                Ok(())
            }

            Effect::SacrificeAnyNumber { who, filter, per_each } => {
                use crate::decision::{Decision, DecisionAnswer};
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                // Candidate set, cheapest/weakest first (the player keeps their
                // best creatures when sacrificing for value).
                let mut candidates: Vec<CardId> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == p
                        && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, ctx.source))
                    .map(|c| (c.id, c.is_token, c.definition.cost.cmc(), c.power()))
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|(id, ..)| id)
                    .collect();
                candidates.sort_by_key(|id| {
                    let c = self.battlefield_find(*id);
                    (c.map(|c| !c.is_token).unwrap_or(true),
                     c.map(|c| c.definition.cost.cmc()).unwrap_or(0),
                     c.map(|c| c.power()).unwrap_or(0))
                });
                let max = candidates.len() as u32;
                if max == 0 { return Ok(()); }
                let source = ctx.source.unwrap_or(CardId(0));
                let decision = Decision::ChooseAmount {
                    source,
                    prompt: "Sacrifice how many?".to_string(),
                    max,
                };
                let answer = match self.stashed_resolution_answer.take() {
                    Some(a) => a,
                    None if self.players[p].wants_ui => {
                        self.suspend_signal = Some((
                            decision,
                            PendingEffectState::AmountAnswerPending { max },
                            effect.clone(),
                        ));
                        return Ok(());
                    }
                    None => self.decider.decide(&decision),
                };
                let n = match answer {
                    DecisionAnswer::Amount(v) => v.min(max),
                    _ => 0,
                } as usize;
                for &cid in candidates.iter().take(n) {
                    let is_creature = self.battlefield_find(cid)
                        .map(|c| c.definition.is_creature()).unwrap_or(false);
                    if is_creature {
                        if let Some(c) = self.battlefield_find(cid) {
                            self.died_card_snapshots.insert(cid, c.clone());
                        }
                        events.push(GameEvent::CreatureSacrificed { card_id: cid, who: p });
                        events.push(GameEvent::CreatureDied { card_id: cid });
                    }
                    events.push(GameEvent::PermanentSacrificed { card_id: cid, who: p });
                    let mut die_evs = self.remove_to_graveyard_with_triggers(cid);
                    events.append(&mut die_evs);
                    // Run the per-sacrifice payoff once (GainLife 3 → 3 × count).
                    self.run_effect(per_each, ctx, events)?;
                }
                Ok(())
            }

            Effect::PayLifeDraw { who } => {
                use crate::decision::{Decision, DecisionAnswer};
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let life = self.players[p].life.max(0) as u32;
                if life == 0 { return Ok(()); }
                let decision = Decision::ChooseAmount {
                    source: ctx.source.unwrap_or(CardId(0)),
                    prompt: "Pay how much life? (draw that many)".to_string(),
                    max: life,
                };
                let answer = match self.stashed_resolution_answer.take() {
                    Some(a) => a,
                    None if self.players[p].wants_ui => {
                        self.suspend_signal = Some((
                            decision,
                            PendingEffectState::AmountAnswerPending { max: life },
                            effect.clone(),
                        ));
                        return Ok(());
                    }
                    None => self.decider.decide(&decision),
                };
                let x = match answer {
                    DecisionAnswer::Amount(v) => v.min(life),
                    _ => 0,
                };
                if x == 0 { return Ok(()); }
                let applied = self.adjust_life_applied(p, -(x as i32));
                if applied < 0 {
                    events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                }
                for _ in 0..x {
                    if !self.draw_one(p, events) {
                        self.lose_to_empty_draw(p);
                        break;
                    }
                }
                Ok(())
            }

            Effect::OnAttackedUntilYourNextTurn { body } => {
                self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                    controller: ctx.controller,
                    source: ctx.source.unwrap_or(CardId(0)),
                    kind: crate::game::types::DelayedKind::CreatureAttacksYouUntilYourNextTurn,
                    effect: (**body).clone(),
                    target: None,
                    bound_token: None,
                    fires_once: false,
                });
                Ok(())
            }

            Effect::ClashWithOpponent { on_win } => {
                // CR 701.30 — both reveal the top card; each may bottom it
                // (seat-routed yes/no, so a networked human gets the prompt;
                // AutoDecider keeps it on top). The controller wins on a
                // strictly higher mana value of the *revealed* cards. All
                // asks precede the zone moves / reveal events so the suspend
                // re-run is idempotent.
                let me = ctx.controller;
                let Some(opp) = self.opponents_of(me).first().copied() else {
                    return Ok(());
                };
                let source = ctx.source.unwrap_or(CardId(0));
                let mut cursor = 0;
                let mut mv = [0i64; 2];
                let mut bottoms = [false; 2];
                for (i, p) in [me, opp].into_iter().enumerate() {
                    let Some(top) = self.players[p].library.first() else { continue };
                    mv[i] = top.definition.cost.cmc() as i64;
                    let prompt = format!(
                        "Clash: you revealed {}. Put it on the bottom?",
                        top.definition.name
                    );
                    let Some(b) = self.ask_seat_bool(&mut cursor, p, prompt, source, effect)
                    else {
                        return Ok(());
                    };
                    bottoms[i] = b;
                }
                self.clear_answer_log();
                for (i, p) in [me, opp].into_iter().enumerate() {
                    let Some(top) = self.players[p].library.first() else { continue };
                    events.push(GameEvent::TopCardRevealed {
                        player: p,
                        card_name: top.definition.name,
                        is_land: top.definition.is_land(),
                    });
                    if bottoms[i] {
                        let card = self.players[p].library.remove(0);
                        self.players[p].library.push(card);
                    }
                }
                if mv[0] > mv[1] {
                    self.run_effect(on_win, ctx, events)?;
                }
                Ok(())
            }

            Effect::RevealUntilLandDamage { to, double_if } => {
                let p = ctx.controller;
                let mut nonland = 0u32;
                let mut doubled = false;
                let mut revealed: Vec<CardInstance> = Vec::new();
                while !self.players[p].library.is_empty() {
                    let card = self.players[p].library.remove(0);
                    let is_land = card.definition.is_land();
                    if is_land {
                        doubled = double_if
                            .is_some_and(|lt| card.definition.subtypes.land_types.contains(&lt));
                    } else {
                        nonland += 1;
                    }
                    revealed.push(card);
                    if is_land {
                        break;
                    }
                }
                for c in revealed {
                    self.players[p].library.push(c);
                }
                let dmg = if doubled { nonland * 2 } else { nonland };
                if dmg > 0 {
                    for ent in self.resolve_selector(to, ctx) {
                        self.deal_damage_to_from(ent, dmg, ctx.source, events);
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::PayLifeLookTake { who } => {
                use crate::decision::{Decision, DecisionAnswer};
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let life = self.players[p].life.max(0) as u32;
                if life == 0 { return Ok(()); }
                let source = ctx.source.unwrap_or(CardId(0));
                let decision = Decision::ChooseAmount {
                    source,
                    prompt: "Pay how much life?".to_string(),
                    max: life,
                };
                let answer = match self.stashed_resolution_answer.take() {
                    Some(a) => a,
                    None if self.players[p].wants_ui => {
                        self.suspend_signal = Some((
                            decision,
                            PendingEffectState::AmountAnswerPending { max: life },
                            effect.clone(),
                        ));
                        return Ok(());
                    }
                    None => self.decider.decide(&decision),
                };
                let x = match answer {
                    DecisionAnswer::Amount(v) => v.min(life),
                    _ => 0,
                };
                if x == 0 { return Ok(()); }
                let applied = self.adjust_life_applied(p, -(x as i32));
                if applied < 0 {
                    events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                }
                // Look at top X; pick one to hand, exile the rest.
                let revealed: Vec<CardId> =
                    self.players[p].library.iter().take(x as usize).map(|c| c.id).collect();
                if revealed.is_empty() { return Ok(()); }
                let candidates: Vec<(CardId, String)> = revealed.iter().filter_map(|id| {
                    self.players[p].library.iter().find(|c| c.id == *id)
                        .map(|c| (*id, c.definition.name.to_string()))
                }).collect();
                let decision = Decision::SearchLibrary { player: p, candidates, eligible: None };
                let pending = PendingEffectState::PayLifeLookPending { player: p, revealed };
                if self.players[p].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }
                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::DelayUntil { kind, body } => {
                // Capture the current target slot so the delayed body can
                // reference it via `Selector::Target(0)` later (e.g. Goryo's
                // wants to exile the same creature it reanimated).
                //
                // Fall back to the cast spell's slot-0 target when our own
                // resolution context has no target — that's the Repartee /
                // triggered-ability shape used by Conciliator's Duelist
                // ("Repartee → exile the cast spell's target, return at
                // next end step"). The trigger resolves with empty
                // `ctx.targets` but the cast-spell `StackItem::Spell` is
                // still below us on the stack, so we can pull its target.
                let target = ctx.targets.first().cloned().or_else(|| {
                    let cid = match ctx.trigger_source {
                        Some(EntityRef::Card(c)) | Some(EntityRef::Permanent(c)) => c,
                        _ => return None,
                    };
                    self.stack.iter().rev().find_map(|si| match si {
                        StackItem::Spell { card, target, .. } if card.id == cid => {
                            target.clone()
                        }
                        _ => None,
                    })
                });
                let source = ctx.source.unwrap_or(crate::card::CardId(0));
                self.delayed_triggers.push(DelayedTrigger {
                    controller: ctx.controller,
                    source,
                    kind: delayed_kind_from_effect(*kind),
                    effect: (**body).clone(),
                    target,
                    // Bind a token minted earlier in this resolution so the
                    // delayed body's `LastCreatedToken` still finds it.
                    bound_token: self.last_created_token,
                    fires_once: true,
                });
                Ok(())
            }

            Effect::WhenLastCreatedTokenLeaves { body } => {
                // Watch the token minted earlier in this resolution; capture
                // the current trigger source (e.g. the card just exiled) as
                // the body's Target(0).
                if let Some(token) = self.last_created_token {
                    let captured = match ctx.trigger_source {
                        Some(EntityRef::Card(c)) | Some(EntityRef::Permanent(c)) => {
                            Some(crate::game::Target::Permanent(c))
                        }
                        _ => None,
                    };
                    self.delayed_triggers.push(DelayedTrigger {
                        controller: ctx.controller,
                        source: ctx.source.unwrap_or(CardId(0)),
                        kind: crate::game::types::DelayedKind::WhenCardLeavesBattlefield(token),
                        effect: (**body).clone(),
                        target: captured,
                        bound_token: Some(token),
                        fires_once: true,
                    });
                }
                Ok(())
            }

            Effect::WhenTargetDiesThisTurn { body, slot } => {
                // Watch the targeted creature's death; capture its controller
                // as the body's Target(0) so it survives the creature leaving
                // play. No-op if there's no permanent target (the creature
                // already left, or none was chosen).
                if let Some(crate::game::Target::Permanent(cid)) = ctx.targets.get(*slot).cloned()
                    && let Some(controller) = self.battlefield_find(cid).map(|c| c.controller)
                {
                    let source = ctx.source.unwrap_or(crate::card::CardId(0));
                    self.delayed_triggers.push(DelayedTrigger {
                        controller: ctx.controller,
                        source,
                        kind: crate::game::types::DelayedKind::WhenCardDies(cid),
                        effect: (**body).clone(),
                        target: Some(crate::game::Target::Player(controller)),
                        bound_token: None,
                        fires_once: true,
                    });
                }
                Ok(())
            }

            Effect::CreaturesYouControlEnteringThisTurn { body } => {
                let source = ctx.source.unwrap_or(crate::card::CardId(0));
                self.delayed_triggers.push(DelayedTrigger {
                    controller: ctx.controller,
                    source,
                    kind: crate::game::types::DelayedKind::CreatureYouControlEntersThisTurn,
                    effect: (**body).clone(),
                    target: None,
                    bound_token: None,
                    fires_once: false,
                });
                Ok(())
            }

            Effect::SearchSplitWithOpponent { count } => {
                use crate::card::CardType;
                use crate::effect::ZoneDest;
                let p = ctx.controller;
                let cap = ctx.x_value;
                self.players[p].searched_library_this_turn = true;
                // Candidates: creature cards in the controller's library +
                // graveyard, MV ≤ X, distinct names; take the biggest `count`.
                let mut candidates: Vec<(CardId, &'static str, u32)> = self.players[p]
                    .library
                    .iter()
                    .chain(self.players[p].graveyard.iter())
                    .filter(|c| {
                        c.definition.card_types.contains(&CardType::Creature)
                            && c.definition.cost.cmc() <= cap
                    })
                    .map(|c| (c.id, c.definition.name, c.definition.cost.cmc()))
                    .collect();
                candidates.sort_by_key(|&(_, _, mv)| std::cmp::Reverse(mv));
                let mut picked: Vec<(CardId, u32)> = Vec::new();
                let mut names: Vec<&str> = Vec::new();
                for (cid, name, mv) in candidates {
                    if picked.len() as u32 >= *count {
                        break;
                    }
                    if !names.contains(&name) {
                        names.push(name);
                        picked.push((cid, mv));
                    }
                }
                // The opponent denies the two biggest (picked is MV-sorted).
                let denied: Vec<CardId> = picked.iter().take(2).map(|(c, _)| *c).collect();
                for (cid, _) in picked.iter().skip(2) {
                    self.move_card_to(
                        *cid,
                        &ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: false },
                        ctx,
                        events,
                    );
                }
                for cid in denied {
                    self.move_card_to(
                        cid,
                        &ZoneDest::Library { who: PlayerRef::Seat(p), pos: crate::effect::LibraryPosition::Shuffled },
                        ctx,
                        events,
                    );
                }
                Ok(())
            }

            Effect::OnEachSpellCastThisTurn { body } => {
                let Some(source) = ctx.source else { return Ok(()) };
                self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                    controller: ctx.controller,
                    source,
                    kind: crate::game::types::DelayedKind::YourNextSpellCastThisTurn,
                    effect: (**body).clone(),
                    target: None,
                    bound_token: None,
                    fires_once: false,
                });
                Ok(())
            }

            Effect::OnYourNextSpellCastThisTurn { body } => {
                let source = ctx.source.unwrap_or(crate::card::CardId(0));
                self.delayed_triggers.push(DelayedTrigger {
                    controller: ctx.controller,
                    source,
                    kind: crate::game::types::DelayedKind::YourNextSpellCastThisTurn,
                    effect: (**body).clone(),
                    target: None,
                    bound_token: None,
                    fires_once: true,
                });
                Ok(())
            }

            Effect::PayOrLoseGame { mana_cost, life_cost } => {
                let p = ctx.controller;
                // Try to pay mana via auto-tap, then deduct life. If any of
                // those fail, the controller loses the game. Roll back any
                // partial payment on failure.
                let cost_subbed = if mana_cost.has_x() {
                    mana_cost.with_x_value(0)
                } else {
                    mana_cost.clone()
                };
                // We can't go through `try_pay_with_auto_tap` directly:
                // even on a successful pay, we may need to roll back if
                // `life_cost` would lose the game. Snapshot manually,
                // commit on success, restore on either failure path.
                let snapshot = self.snapshot_payment_state(p);
                let mut paid_events = self.auto_tap_for_cost(p, &cost_subbed);
                let mana_paid = self.players[p].mana_pool.pay(&cost_subbed).is_ok();
                let life_ok = self.players[p].life > *life_cost as i32;
                if mana_paid && life_ok {
                    if *life_cost > 0 {
                        let applied = self.adjust_life_applied(p, -(*life_cost as i32));
                        if applied < 0 {
                            paid_events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                        }
                    }
                    events.append(&mut paid_events);
                } else {
                    self.restore_payment_state(p, snapshot);
                    self.players[p].eliminated = true;
                    let mut sba = self.check_state_based_actions();
                    events.append(&mut sba);
                }
                Ok(())
            }

            Effect::AddFirstSpellTax { who, count } => {
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                if n == 0 {
                    return Ok(());
                }
                for p in self.resolve_players(who, ctx) {
                    self.players[p].first_spell_tax_charges =
                        self.players[p].first_spell_tax_charges.saturating_add(n);
                }
                Ok(())
            }

            Effect::GrantSorceriesAsFlash { who } => {
                for p in self.resolve_players(who, ctx) {
                    self.players[p].sorceries_as_flash = true;
                }
                Ok(())
            }

            Effect::RevealUntilFind {
                who,
                find,
                to,
                cap,
                life_per_revealed,
                miss_dest,
            } => {
                // Walk the top of `who`'s library until we either find a
                // matching card or hit the cap. Route misses according to
                // `miss_dest` (graveyard by default; bottom-of-library
                // for SOS Strixhaven "rest on the bottom in a random
                // order" cards).
                let Some(p) = self.resolve_player(who, ctx) else { return Ok(()); };
                let cap_n = self.evaluate_value(cap, ctx).max(0) as usize;
                if cap_n == 0 {
                    return Ok(());
                }
                let resolved_dest = self.resolve_zonedest_player(to, ctx);
                // `NamedBySource` reads the dynamic name the source stamped
                // via `Effect::NameCard` (Spoils of the Vault). Resolve it to
                // a concrete `HasName` once, up front.
                let dynamic_find = if matches!(find, SelectionRequirement::NamedBySource) {
                    match self.named_card_this_resolution.clone()
                        .or_else(|| ctx.source.and_then(|s| self.find_card_anywhere(s))
                            .and_then(|c| c.named_card.clone()))
                    {
                        Some(name) => Some(SelectionRequirement::HasName(name)),
                        // No card named → nothing can match; reveal to cap.
                        None => Some(SelectionRequirement::And(
                            Box::new(SelectionRequirement::Any),
                            Box::new(SelectionRequirement::Not(Box::new(SelectionRequirement::Any))),
                        )),
                    }
                } else {
                    None
                };
                let find: &SelectionRequirement = dynamic_find.as_ref().unwrap_or(find);
                let mut revealed = 0usize;
                let mut found_idx: Option<usize> = None;
                for i in 0..cap_n.min(self.players[p].library.len()) {
                    revealed += 1;
                    let card = &self.players[p].library[i];
                    if self.evaluate_requirement_on_card(find, card, ctx.controller) {
                        found_idx = Some(i);
                        break;
                    }
                }
                // Move the misses (everything before `found_idx`, or
                // everything if no match) into the configured miss zone.
                let miss_count = found_idx.unwrap_or(revealed);
                for _ in 0..miss_count {
                    if self.players[p].library.is_empty() {
                        break;
                    }
                    let card = self.players[p].library.remove(0);
                    let cid = card.id;
                    match miss_dest {
                        crate::effect::RevealMissDest::Graveyard => {
                            if !self.route_to_graveyard(card, events) {
                                events.push(GameEvent::CardMilled { player: p, card_id: cid });
                            }
                        }
                        crate::effect::RevealMissDest::BottomRandom => {
                            // No RNG hook in the engine yet — push to
                            // the back of the library deterministically.
                            // From a gameplay standpoint this is
                            // indistinguishable from a "random bottom"
                            // since no card knows the bottom ordering
                            // before the next shuffle / reveal.
                            self.players[p].library.push(card);
                        }
                        crate::effect::RevealMissDest::ShuffleIntoLibrary => {
                            // Re-shuffled below once the find is placed.
                            self.players[p].library.push(card);
                        }
                    }
                }
                // If we found a match, take it off the (now-shifted) top
                // and place it via the requested destination. Push the
                // matched id onto `last_moved_cards` so a downstream
                // `Selector::LastMoved` in the same Seq can target it
                // — used by Velomachus Lorehold's
                // `RevealUntilFind + GrantMayPlay` chain.
                if found_idx.is_some() && !self.players[p].library.is_empty() {
                    let card = self.players[p].library.remove(0);
                    let cid = card.id;
                    self.place_card_in_dest(card, p, &resolved_dest, events);
                    self.last_moved_cards.push(cid);
                }
                if matches!(miss_dest, crate::effect::RevealMissDest::ShuffleIntoLibrary) {
                    use rand::seq::SliceRandom;
                    self.players[p].library.shuffle(&mut rand::rng());
                }
                // Lose 1 life per revealed card (Spoils of the Vault rider).
                let life = (revealed as u32).saturating_mul(*life_per_revealed);
                if life > 0 {
                    let applied = self.adjust_life_applied(p, -(life as i32));
                    if applied < 0 {
                        events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
                Ok(())
            }

            Effect::BecomeCopyOf {
                what,
                source,
                extra_creature_types,
            } => {
                // CR 707.2 — `what` becomes a copy of `source`'s copiable
                // characteristics. One-shot definition rewrite: clone the
                // source's current definition Arc and stamp it onto each
                // resolved `what`, preserving instance state. Locked in at
                // resolution; later changes to the source don't propagate.
                let src_def = self
                    .resolve_selector(source, ctx)
                    .into_iter()
                    .find_map(|e| e.as_permanent_id())
                    .and_then(|id| self.battlefield.iter().find(|c| c.id == id))
                    .map(|c| c.definition.clone());
                if let Some(src_def) = src_def {
                    for ent in self.resolve_selector(what, ctx) {
                        let Some(cid) = ent.as_permanent_id() else { continue };
                        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == cid) {
                            let mut new_def = (*src_def).clone();
                            for t in extra_creature_types {
                                if !new_def.subtypes.creature_types.contains(t) {
                                    new_def.subtypes.creature_types.push(*t);
                                }
                            }
                            let original =
                                std::mem::replace(&mut c.definition, std::sync::Arc::new(new_def));
                            // CR 400.7 — in its next zone the object is its
                            // printed card again; `revert_copy_on_leave`
                            // restores this (a dead Clone is a Clone in the
                            // graveyard, so Vizier's embalm stays available).
                            self.temporary_copies.push(crate::game::TempCopy {
                                card: cid,
                                original_name: original.name.to_string(),
                                original: Some(original),
                                duration: crate::effect::Duration::Permanent,
                            });
                        }
                    }
                }
                Ok(())
            }

            Effect::BecomeCopyOfFor { what, source, duration, non_legendary } => {
                // CR 707.2 — continuous copy: swap the definition now and
                // schedule the revert (`temporary_copies`), mirroring the
                // Act-of-Treason control plumbing. Battlefield-leave reverts
                // via `revert_copy_on_leave`.
                let src = self
                    .resolve_selector(source, ctx)
                    .into_iter()
                    .find_map(|e| e.as_permanent_id());
                // The copy source may be a battlefield permanent or a card
                // in another zone (Shifting Woodland copies a graveyard
                // permanent *card*).
                let src_def = src
                    .and_then(|id| self.find_card_anywhere(id))
                    .map(|c| c.definition.clone());
                let Some(src_def) = src_def else { return Ok(()) };
                let copy_def = if *non_legendary {
                    let mut d = (*src_def).clone();
                    d.supertypes.retain(|s| !matches!(s, crate::card::Supertype::Legendary));
                    std::sync::Arc::new(d)
                } else {
                    src_def
                };
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    // The copied permanent itself never re-copies ("each
                    // *other* creature"; self-copy is identity anyway).
                    if Some(cid) == src {
                        continue;
                    }
                    let Some(c) = self.battlefield.iter_mut().find(|c| c.id == cid) else {
                        continue;
                    };
                    let original = std::mem::replace(&mut c.definition, copy_def.clone());
                    self.temporary_copies.push(crate::game::TempCopy {
                        card: cid,
                        original_name: original.name.to_string(),
                        original: Some(original),
                        duration: *duration,
                    });
                }
                Ok(())
            }

            Effect::ResetCreature {
                what,
                power,
                toughness,
                creature_types,
                duration,
            } => {
                // CR 613 — target becomes a creature with the given P/T and
                // creature types, losing all other card types, abilities, and
                // printed creature subtypes. Layer 4 sets the type line, layer
                // 6 strips abilities, layer 7b sets base P/T. Oko's "3/3 Elk",
                // Turn to Frog's "0/1 blue Frog with no abilities", etc.
                use crate::game::layers::{
                    AffectedPermanents, ContinuousEffect, Layer, Modification, PtSublayer,
                };
                let p = self.evaluate_value(power, ctx);
                let t = self.evaluate_value(toughness, ctx);
                let duration_kind = map_effect_duration(*duration);
                let source = ctx.source.unwrap_or(CardId(0));
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    let affected = AffectedPermanents::Specific(vec![cid]);
                    let mut push = |layer, sublayer, modification| {
                        let ts = self.next_timestamp();
                        self.add_continuous_effect(ContinuousEffect {
                            timestamp: ts,
                            source,
                            affected: affected.clone(),
                            layer,
                            sublayer,
                            duration: duration_kind.clone(),
                            modification,
                        });
                    };
                    push(
                        Layer::L4Type,
                        None,
                        Modification::SetCardTypes(vec![crate::card::CardType::Creature]),
                    );
                    push(
                        Layer::L4Type,
                        None,
                        Modification::SetCreatureTypes(creature_types.clone()),
                    );
                    push(Layer::L6Ability, None, Modification::RemoveAllAbilities);
                    push(
                        Layer::L7PowerTough,
                        Some(PtSublayer::SetValue),
                        Modification::SetPowerToughness(p, t),
                    );
                    events.push(GameEvent::PumpApplied {
                        card_id: cid,
                        power: p,
                        toughness: t,
                    });
                }
                Ok(())
            }

            Effect::BecomeBasicLand { what, land_type, duration } => {
                // CR 305.7 / 613 — target becomes a basic land of `land_type`:
                // lose all other card/land types, abilities, and colors; gain
                // the basic's intrinsic "{T}: Add {C}" mana ability. Spreading
                // Seas / Blood Moon family. The intrinsic mana ability is
                // derived from the land type at activation time (see
                // `intrinsic_land_mana_ability`), so no ability grant is
                // installed here.
                use crate::game::layers::{
                    AffectedPermanents, ContinuousEffect, Layer, Modification,
                };
                let duration_kind = map_effect_duration(*duration);
                let source = ctx.source.unwrap_or(CardId(0));
                for ent in self.resolve_selector(what, ctx) {
                    let Some(cid) = ent.as_permanent_id() else { continue };
                    let affected = AffectedPermanents::Specific(vec![cid]);
                    let mut push = |layer, modification| {
                        let ts = self.next_timestamp();
                        self.add_continuous_effect(ContinuousEffect {
                            timestamp: ts,
                            source,
                            affected: affected.clone(),
                            layer,
                            sublayer: None,
                            duration: duration_kind.clone(),
                            modification,
                        });
                    };
                    push(
                        Layer::L4Type,
                        Modification::SetCardTypes(vec![crate::card::CardType::Land]),
                    );
                    push(Layer::L4Type, Modification::SetLandTypes(vec![*land_type]));
                    push(Layer::L5Color, Modification::LoseAllColors);
                    push(Layer::L6Ability, Modification::RemoveAllAbilities);
                }
                Ok(())
            }

            Effect::CopySpell { what, count } => {
                // Resolve which spell to copy. We support two main patterns:
                // 1. `Selector::TriggerSource` — the spell that fired this
                //    trigger (Magecraft / Storm / Aziza-style "whenever you
                //    cast X, copy that spell"). The trigger_source carries
                //    the cast spell's CardId.
                // 2. `Selector::Target(n)` / `CastSpellTarget(n)` — a
                //    specifically-targeted spell on the stack
                //    (Reverberate / Twincast / Choreographed Sparks).
                //
                // For each, we locate the matching `StackItem::Spell` and
                // clone it `count` times with a fresh CardId per copy. The
                // copies inherit the original's target / mode / x_value /
                // converged_value. Auto-retargeting for friendly-fire
                // (choose new targets for the copy) is left to the
                // existing auto_target_for_effect picker, which runs at
                // resolution time. The original spell still resolves
                // afterward — copies always end up above it on the stack.
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 {
                    return Ok(());
                }
                // Find candidate stack indices to copy.
                let candidate_ids: Vec<CardId> = match what {
                    Selector::TriggerSource => ctx
                        .trigger_source
                        .into_iter()
                        .filter_map(|e| match e {
                            EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                            _ => None,
                        })
                        .collect(),
                    Selector::This => ctx.source.into_iter().collect(),
                    _ => self
                        .resolve_selector(what, ctx)
                        .into_iter()
                        .filter_map(|e| match e {
                            EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                            _ => None,
                        })
                        .collect(),
                };
                for cid in candidate_ids {
                    self.copy_stack_spell(cid, n, false, events);
                }
                Ok(())
            }

            Effect::CopySpellMayChooseTargets { what, count } => {
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 {
                    return Ok(());
                }
                let candidate_ids: Vec<CardId> = match what {
                    Selector::TriggerSource => ctx
                        .trigger_source
                        .into_iter()
                        .filter_map(|e| match e {
                            EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                            _ => None,
                        })
                        .collect(),
                    Selector::This => ctx.source.into_iter().collect(),
                    _ => self
                        .resolve_selector(what, ctx)
                        .into_iter()
                        .filter_map(|e| match e {
                            EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                            _ => None,
                        })
                        .collect(),
                };
                for cid in candidate_ids {
                    self.copy_stack_spell(cid, n, true, events);
                }
                Ok(())
            }

            Effect::Demonstrate => {
                // CR 702.150 — copy this spell for its caster, then an
                // opponent of the caster's choosing also copies it; every
                // copy may choose new targets. Fired from a self-cast
                // trigger, so `ctx.source` is the spell still on the stack.
                let spell_id = match ctx.source {
                    Some(c) => c,
                    None => return Ok(()),
                };
                self.copy_stack_spell(spell_id, 1, true, events);
                // Pick an opponent (the lowest-seat opponent by default; a
                // single-opponent game has exactly one).
                if let Some(&opp) = self.opponents_of(ctx.controller).first() {
                    self.copy_stack_spell_controlled(spell_id, 1, true, Some(opp), events);
                }
                Ok(())
            }

            Effect::ChooseNewTargetsForSpell { what } => {
                // CR 115.7 — repoint the targeted spell's primary target in
                // place. The controller of *this* effect (Redirect's caster)
                // chooses; the original target is offered first.
                let chooser = ctx.controller;
                let spell_ids: Vec<CardId> = self
                    .resolve_selector(what, ctx)
                    .into_iter()
                    .filter_map(|e| match e {
                        EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                        _ => None,
                    })
                    .collect();
                for sid in spell_ids {
                    let Some(idx) = self.stack.iter().rposition(|s| {
                        matches!(s, crate::game::types::StackItem::Spell { card, .. }
                            if card.id == sid)
                    }) else {
                        continue;
                    };
                    let (def, orig_target) =
                        if let crate::game::types::StackItem::Spell { card, target, .. } =
                            &self.stack[idx]
                        {
                            (card.definition.clone(), target.clone())
                        } else {
                            continue;
                        };
                    if orig_target.is_none() {
                        continue;
                    }
                    let new_target = self.repoint_copy_target(&def, chooser, &orig_target);
                    if let crate::game::types::StackItem::Spell { target, .. } =
                        &mut self.stack[idx]
                    {
                        *target = new_target;
                    }
                }
                Ok(())
            }

            Effect::CopySpellUnlessPaid { what, mana_cost, count } => {
                // Wandering Archaic shape: the *caster of the spell being
                // copied* may pay `mana_cost` to avoid being copied. We:
                // (1) locate the matching `StackItem::Spell`; (2) ask the
                // *caster* yes/no via `Decision::OptionalTrigger`; (3) on
                // yes + affordable pool, deduct + skip copy; (4) on no
                // or unaffordable, fall through to the same copy path as
                // `Effect::CopySpell`.
                use crate::decision::{Decision, DecisionAnswer};
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 {
                    return Ok(());
                }
                let candidate_ids: Vec<CardId> = match what {
                    Selector::TriggerSource => ctx
                        .trigger_source
                        .into_iter()
                        .filter_map(|e| match e {
                            EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                            _ => None,
                        })
                        .collect(),
                    Selector::This => ctx.source.into_iter().collect(),
                    _ => self
                        .resolve_selector(what, ctx)
                        .into_iter()
                        .filter_map(|e| match e {
                            EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                            _ => None,
                        })
                        .collect(),
                };
                for cid in candidate_ids {
                    let stack_idx = self.stack.iter().rposition(|s| {
                        matches!(s, crate::game::types::StackItem::Spell { card, .. }
                            if card.id == cid)
                    });
                    let Some(idx) = stack_idx else { continue; };
                    // Snapshot the caster (the affected payer for the
                    // optional pay) up-front. The spell snapshot for the
                    // copy is taken inside the "unpaid" branch.
                    let caster_for_pay = if let crate::game::types::StackItem::Spell {
                        caster, ..
                    } = &self.stack[idx]
                    {
                        *caster
                    } else {
                        continue;
                    };
                    // Ask the *caster* of the spell whether they want to
                    // pay the tax. Bot's AutoDecider defaults to false
                    // (let the copy happen — saves the {2}).
                    let answer = self.decider.decide(&Decision::OptionalTrigger {
                        source: ctx.source.unwrap_or(CardId(0)),
                        description: "Pay {2} to prevent Wandering Archaic's copy?"
                            .to_string(),
                    });
                    if matches!(answer, DecisionAnswer::Bool(true)) {
                        // Try to deduct from the payer's pool.
                        let pool = &mut self.players[caster_for_pay].mana_pool;
                        if pool.pay(mana_cost).is_ok() {
                            // Paid — skip the copy.
                            continue;
                        }
                        // Couldn't afford; fall through to copy.
                    }
                    // Unpaid (declined or unaffordable) → copy `n` times
                    // through the shared copy funnel (CR 707 CantBeCopied
                    // guard, copy bookkeeping, SpellsCopied event).
                    self.copy_stack_spell(cid, n, false, events);
                }
                Ok(())
            }

            Effect::NameCreatureType { what } => {
                // Cavern of Souls "as it enters, choose a creature type".
                // The chooser is the source's controller. Suspend with a
                // `ChooseCreatureType` decision so a UI player can pick;
                // bots / AutoDecider resolve synchronously.
                use crate::decision::Decision;
                let candidate = self
                    .resolve_selector(what, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Permanent(c) => Some(c),
                        _ => None,
                    });
                let Some(target_id) = candidate else { return Ok(()); };
                let chooser = ctx.controller;
                let decision = Decision::ChooseCreatureType {
                    source: target_id,
                    suggestions: self.creature_type_suggestions(chooser),
                };
                let pending =
                    PendingEffectState::ChooseCreatureTypePending { target_id };
                if self.players[chooser].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }
                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::NameCard { what } => {
                // CR 201.3 — "as this enters, choose a card name." Mirrors
                // NameCreatureType: stamp the chosen name onto the source
                // permanent's `named_card`. Suspends for UI players; bots /
                // AutoDecider resolve synchronously (and name nothing).
                use crate::decision::Decision;
                let candidate = self
                    .resolve_selector(what, ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Permanent(c) | EntityRef::Card(c) => Some(c),
                        _ => None,
                    })
                    // A resolving instant/sorcery (Spoils of the Vault) is the
                    // source itself, not a battlefield permanent.
                    .or(ctx.source);
                let Some(target_id) = candidate else { return Ok(()); };
                let source_name = self
                    .find_card_anywhere(target_id)
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                // Heuristic feed: a battlefield namer (Pithing Needle) wants
                // an opponent's activated-ability permanent; a resolving
                // spell (Spoils of the Vault) wants its controller's most
                // common library name.
                let suggestions = if self.battlefield.iter().any(|c| c.id == target_id) {
                    rank_names_by_frequency(
                        self.battlefield
                            .iter()
                            .filter(|c| {
                                !self.same_team(c.controller, ctx.controller)
                                    && !c.definition.activated_abilities.is_empty()
                            })
                            .map(|c| c.definition.name),
                    )
                } else {
                    rank_names_by_frequency(
                        self.players[ctx.controller].library.iter().map(|c| c.definition.name),
                    )
                };
                let decision = Decision::NameCard { source: target_id, source_name, suggestions };
                let pending = PendingEffectState::NameCardPending { target_id };
                let chooser = ctx.controller;
                if self.players[chooser].wants_ui {
                    self.suspend_signal = Some((decision, pending, Effect::Noop));
                    return Ok(());
                }
                let answer = self.decider.decide(&decision);
                let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
                events.append(&mut applied);
                Ok(())
            }

            Effect::PreventAllCombatDamageThisTurn => {
                // CR 615.1 — set the engine-wide flag the combat damage
                // resolver consults. Cleared in `do_cleanup` (CR 514.2).
                self.prevent_combat_damage_this_turn = true;
                Ok(())
            }

            Effect::PreventAllCombatDamageInvolving { target } => {
                // CR 614.9 — Maze of Ith: prevent all combat damage to and by
                // the target creature for the rest of the turn.
                for ent in self.resolve_selector(target, ctx) {
                    if let EntityRef::Permanent(id) | EntityRef::Card(id) = ent
                        && !self.combat_damage_prevented_creatures.contains(&id)
                    {
                        self.combat_damage_prevented_creatures.push(id);
                    }
                }
                Ok(())
            }

            Effect::PreventAllDamageFromChosenSourceThisTurn { filter } => {
                let Some(chosen) = self.choose_damage_prevention_source(filter, ctx) else {
                    return Ok(());
                };
                if !self.damage_prevented_sources.contains(&chosen) {
                    self.damage_prevented_sources.push(chosen);
                }
                Ok(())
            }

            Effect::PreventNextDamageFromChosenSource { filter } => {
                // CR 615.7 — Circle of Protection: a one-event shield around
                // the controller, restricted to the chosen source.
                let Some(chosen) = self.choose_damage_prevention_source(filter, ctx) else {
                    return Ok(());
                };
                self.prevention_shields.push(crate::game::types::PreventionShield {
                    target: crate::game::types::PreventionTarget::Player(ctx.controller),
                    remaining: None,
                    gain_life: false,
                    source: Some(chosen),
                    one_event: true,
                });
                Ok(())
            }

            Effect::ExileSelfReturnTransformed => {
                // CR 714.4 — exile this Saga, then return it transformed
                // under its controller's control. Routed through
                // `place_card_in_dest` so new-object state (counters clear,
                // summoning sickness) and the back face's ETB fire normally.
                let Some(id) = ctx.source else { return Ok(()); };
                let Some(pos) = self.battlefield.iter().position(|c| c.id == id) else {
                    return Ok(());
                };
                if self.battlefield[pos].definition.back_face.is_none() {
                    return Ok(());
                }
                let mut card = self.battlefield.remove(pos);
                events.push(GameEvent::PermanentExiled { card_id: id });
                let back = card.definition.back_face.as_ref().map(|b| (**b).clone()).unwrap();
                card.front_face = Some(card.definition.clone());
                card.definition = std::sync::Arc::new(back);
                card.transformed = true;
                events.push(GameEvent::Transformed { card_id: id });
                self.place_card_in_dest(
                    card,
                    ctx.controller,
                    &ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    events,
                );
                Ok(())
            }

            Effect::CantBlockSourceThisTurn { target } => {
                // Record (blocker, attacker=source) so the declare-blockers
                // validator bars only this pairing (Kozilek's Pathfinder).
                let Some(source) = ctx.source else { return Ok(()); };
                for ent in self.resolve_selector(target, ctx) {
                    if let EntityRef::Permanent(id) | EntityRef::Card(id) = ent {
                        let pair = (id, source);
                        if !self.cant_block_pairs.contains(&pair) {
                            self.cant_block_pairs.push(pair);
                        }
                    }
                }
                Ok(())
            }

            Effect::PreventNextDamage { target, amount } => {
                // CR 615.7 — push a "prevent the next N damage to target"
                // shield consumed by `apply_prevention_shields`.
                let n = self.evaluate_value(amount, ctx).max(0) as u32;
                if n > 0 {
                    for s in self.prevention_targets(target, ctx) {
                        self.prevention_shields.push(crate::game::types::PreventionShield {
                            target: s,
                            remaining: Some(n),
                            gain_life: false,
                            source: None,
                            one_event: false,
                        });
                    }
                }
                Ok(())
            }

            Effect::PreventNextDamageAndGainLife { target, amount } => {
                // CR 615.1 — prevent the next N damage to target; the
                // protected player gains that much life (Reverse Damage).
                let n = self.evaluate_value(amount, ctx).max(0) as u32;
                if n > 0 {
                    for s in self.prevention_targets(target, ctx) {
                        self.prevention_shields.push(crate::game::types::PreventionShield {
                            target: s,
                            remaining: Some(n),
                            gain_life: true,
                            source: None,
                            one_event: false,
                        });
                    }
                }
                Ok(())
            }

            Effect::PreventAllDamageThisTurn { target } => {
                // CR 615 — a fog scoped to one player/permanent.
                for s in self.prevention_targets(target, ctx) {
                    self.prevention_shields.push(crate::game::types::PreventionShield {
                        target: s,
                        remaining: None,
                        gain_life: false,
                        source: None,
                        one_event: false,
                    });
                }
                Ok(())
            }

            Effect::DamageCantBePreventedThisTurn => {
                // CR 615.12 — suppress every prevention shield for the turn.
                self.damage_cant_be_prevented_this_turn = true;
                Ok(())
            }

            Effect::PlayerProtectionUntilNextTurn { who } => {
                if let Some(p) = self.resolve_player(who, ctx) {
                    self.players[p].protected_from_everything = true;
                }
                Ok(())
            }

            Effect::DiminishCreaturesExceptChosenType { power, toughness } => {
                // Crippling Fear-style "Choose a creature type. Creatures
                // other than creatures of the chosen type get -P/-T EOT."
                // A `wants_ui` controller picks through the client modal
                // (stash-and-rerun suspend); bots / tests answer via
                // `self.decider` (AutoDecider picks Demon, ScriptedDecider
                // can override). The pump is applied per-creature via the
                // standard `power_bonus` / `toughness_bonus` mutation path
                // (same code shape as Effect::PumpPT).
                use crate::decision::{Decision, DecisionAnswer};
                let p = self.evaluate_value(power, ctx);
                let t = self.evaluate_value(toughness, ctx);
                let source_id = ctx.source.unwrap_or(CardId(0));
                let decision = Decision::ChooseCreatureType {
                    source: source_id,
                    suggestions: self.creature_type_suggestions(ctx.controller),
                };
                let answer = match self.stashed_resolution_answer.take() {
                    Some(a) => a,
                    None if self.players[ctx.controller].wants_ui => {
                        self.suspend_signal = Some((
                            decision,
                            PendingEffectState::CreatureTypeAnswerPending,
                            effect.clone(),
                        ));
                        return Ok(());
                    }
                    None => self.decider.decide(&decision),
                };
                let DecisionAnswer::CreatureType(ct) = answer else {
                    return Err(crate::game::GameError::DecisionAnswerMismatch);
                };
                let card_ids: Vec<CardId> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.definition.is_creature()
                            && !c.definition.subtypes.creature_types.contains(&ct)
                    })
                    .map(|c| c.id)
                    .collect();
                for cid in card_ids {
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        c.power_bonus += p;
                        c.toughness_bonus += t;
                        events.push(GameEvent::PumpApplied {
                            card_id: cid,
                            power: p,
                            toughness: t,
                        });
                    }
                }
                Ok(())
            }

            Effect::ExileTopAndGrantMayPlay { who, count, duration, pay_any_color, uncast_penalty } => {
                // Atomic helper: move the top `count` cards of `who`'s library
                // to exile and stamp `may_play_until` on each in one step.
                // The top of the library is index 0 (see `Player::draw_top`
                // and `Selector::TopOfLibrary`), so this reads `.first()`,
                // not `.last()` (which is the bottom card).
                let targets = self.resolve_players(who, ctx);
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let granted_turn = self.turn_number;
                // Plural `who` (EachOpponent) peels from every resolved
                // library — Nassari, Dean of Expression exiles the top card
                // of each opponent's library, all castable by Nassari's
                // controller until end of turn.
                for p in targets {
                    for _ in 0..n {
                        let Some(top_id) = self.players[p].library.first().map(|c| c.id) else { break; };
                        let mut local_events = Vec::new();
                        self.move_card_to(top_id, &crate::effect::ZoneDest::Exile, ctx, &mut local_events);
                        events.extend(local_events);
                        if let Some(card) = self.find_card_anywhere_mut(top_id) {
                            card.may_play_until = Some(crate::card::MayPlayPermission {
                                player: ctx.controller,
                                granted_turn,
                                duration: *duration,
                                exile_after: false,
                            });
                            // Pay-to-cast rider (CR 609.4b any-type spend):
                            // the cast costs the card's MV as generic.
                            if *pay_any_color {
                                card.granted_alt_cast_cost_eot =
                                    Some(crate::mana::ManaCost::new(vec![crate::mana::generic(
                                        card.definition.cost.cmc(),
                                    )]));
                            }
                        }
                        // "If you don't [cast it], …" — check at the next
                        // end step whether the card is still in exile.
                        if let Some(pen) = uncast_penalty {
                            self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                                controller: ctx.controller,
                                source: ctx.source.unwrap_or(CardId(0)),
                                kind: crate::game::types::DelayedKind::NextEndStep,
                                effect: Effect::If {
                                    cond: crate::effect::Predicate::EntityMatches {
                                        what: crate::effect::Selector::Target(0),
                                        filter: crate::card::SelectionRequirement::InExile,
                                    },
                                    then: pen.clone(),
                                    else_: Box::new(Effect::Noop),
                                },
                                target: Some(Target::Permanent(top_id)),
                                bound_token: None,
                                fires_once: true,
                            });
                        }
                    }
                }
                Ok(())
            }

            Effect::AddCardTypeIndefinitely { what, card_type } => {
                use crate::game::layers::{
                    AffectedPermanents, ContinuousEffect, EffectDuration, Layer, Modification,
                };
                for ent in self.resolve_selector(what, ctx) {
                    let Some(id) = ent.as_permanent_id() else { continue };
                    let ts = self.next_timestamp();
                    // Anchored to the affected permanent itself so the type
                    // grant dies with it (remove_effects_from_source).
                    self.add_continuous_effect(ContinuousEffect {
                        timestamp: ts,
                        source: id,
                        affected: AffectedPermanents::Specific(vec![id]),
                        layer: Layer::L4Type,
                        sublayer: None,
                        duration: EffectDuration::Indefinite,
                        modification: Modification::AddCardType(card_type.clone()),
                    });
                }
                Ok(())
            }

            Effect::GainActivatedAbility { what, ability } => {
                for ent in self.resolve_selector(what, ctx) {
                    let (EntityRef::Permanent(id) | EntityRef::Card(id)) = ent else { continue };
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.granted_activated_abilities.push((**ability).clone());
                    }
                }
                Ok(())
            }

            Effect::GrantMayPlay {
                what,
                duration,
                to_owner,
                exile_after,
                pay_own_cost,
                any_color,
            } => {
                // Resolve `what` to a set of cards and stamp each with a
                // `MayPlayPermission`. The selector can match cards in
                // any zone — graveyard (Practiced Scrollsmith's target
                // from your graveyard), exile (Suspend Aggression's
                // exiled cards), or even hand. The cast-from-zone game
                // action consults `may_play_until` regardless of zone.
                let entities = self.resolve_selector(what, ctx);
                let granter_player = ctx.controller;
                let granted_turn = self.turn_number;
                for ent in entities {
                    let cid = match ent {
                        EntityRef::Card(id) | EntityRef::Permanent(id) => id,
                        _ => continue,
                    };
                    // Determine recipient before we take a mut borrow.
                    let recipient = if *to_owner {
                        self.find_card_owner(cid).unwrap_or(granter_player)
                    } else {
                        granter_player
                    };
                    if let Some(card) = self.find_card_anywhere_mut(cid) {
                        card.may_play_until = Some(crate::card::MayPlayPermission {
                            player: recipient,
                            granted_turn,
                            duration: *duration,
                            exile_after: *exile_after,
                        });
                        if *pay_own_cost {
                            // "Spend mana as though it were mana of any type"
                            // (CR 609.4b) — pay the MV as generic.
                            card.granted_alt_cast_cost_eot = Some(if *any_color {
                                crate::mana::ManaCost::new(vec![crate::mana::generic(
                                    card.definition.cost.cmc(),
                                )])
                            } else {
                                card.definition.cost.clone()
                            });
                        }
                    }
                }
                Ok(())
            }

            Effect::Cascade { max_mv } => {
                // CR 702.85: exile cards from the top of the controller's
                // library until a nonland card with MV < max_mv is exiled;
                // the controller may cast it for free; the rest go to the
                // bottom of the library.
                use crate::card::{CardType, Zone};
                use crate::effect::{LibraryPosition, ZoneDest};
                let p = ctx.controller;
                let cap = self.evaluate_value(max_mv, ctx).max(0) as u32;
                let mut exiled: Vec<crate::card::CardId> = Vec::new();
                let mut hit: Option<crate::card::CardId> = None;
                while !self.players[p].library.is_empty() {
                    // Inspect the top card (index 0) before moving it.
                    let top = &self.players[p].library[0];
                    let cid = top.id;
                    let is_land = top.definition.card_types.contains(&CardType::Land);
                    let mv = top.definition.cost.cmc();
                    self.move_card_to(cid, &ZoneDest::Exile, ctx, events);
                    exiled.push(cid);
                    if !is_land && mv < cap {
                        hit = Some(cid);
                        break;
                    }
                }
                // Offer the matched card for a free cast.
                if let Some(cid) = hit {
                    use crate::decision::{Decision, DecisionAnswer};
                    let card_def = self.find_card_anywhere(cid).map(|c| c.definition.clone());
                    if let Some(card_def) = card_def {
                        let src = ctx.source.unwrap_or(CardId(0));
                        let answer = self.decider.decide(&Decision::OptionalTrigger {
                            source: src,
                            description: "Cascade: cast the exiled card without paying its mana cost?"
                                .to_string(),
                        });
                        if matches!(answer, DecisionAnswer::Bool(true)) {
                            let auto_target = self.auto_target_for_effect_avoiding(
                                &card_def.effect,
                                p,
                                Some(cid),
                            );
                            let cast_events = self.cast_card_for_free(
                                p,
                                cid,
                                Zone::Exile,
                                auto_target,
                                vec![],
                                None,
                                None,
                                false,
                            )?;
                            events.extend(cast_events);
                            // The matched card left exile — don't bottom it.
                            exiled.retain(|&x| x != cid);
                        }
                    }
                }
                // Bottom the remaining exiled cards (random order ≈ bottom).
                for cid in exiled {
                    if self.exile.iter().any(|c| c.id == cid) {
                        self.move_card_to(
                            cid,
                            &ZoneDest::Library {
                                who: PlayerRef::Seat(p),
                                pos: LibraryPosition::Bottom,
                            },
                            ctx,
                            events,
                        );
                    }
                }
                Ok(())
            }

            Effect::Discover { n, filter } => {
                // CR 701.57: exile cards from the top of the controller's
                // library until a nonland card with MV ≤ n is exiled; the
                // controller casts it for free or puts it into hand. The rest
                // go to the bottom of the library in a random order. With a
                // `filter`, the stop condition is "matches filter with MV ≤ n"
                // (Codie's instant-or-sorcery impulse).
                use crate::card::{CardType, Zone};
                use crate::effect::{LibraryPosition, ZoneDest};
                let p = ctx.controller;
                let cap = self.evaluate_value(n, ctx).max(0) as u32;
                let mut exiled: Vec<crate::card::CardId> = Vec::new();
                let mut hit: Option<crate::card::CardId> = None;
                while !self.players[p].library.is_empty() {
                    let top = &self.players[p].library[0];
                    let cid = top.id;
                    let matches = match filter {
                        Some(f) => crate::game::layers::requirement_matches_card(f, top, p),
                        None => !top.definition.card_types.contains(&CardType::Land),
                    };
                    let mv = top.definition.cost.cmc();
                    self.move_card_to(cid, &ZoneDest::Exile, ctx, events);
                    exiled.push(cid);
                    if matches && mv <= cap {
                        hit = Some(cid);
                        break;
                    }
                }
                if let Some(cid) = hit {
                    use crate::decision::{Decision, DecisionAnswer};
                    let card_def = self.find_card_anywhere(cid).map(|c| c.definition.clone());
                    if let Some(card_def) = card_def {
                        let src = ctx.source.unwrap_or(CardId(0));
                        let answer = self.decider.decide(&Decision::OptionalTrigger {
                            source: src,
                            description:
                                "Discover: cast the exiled card without paying its mana cost? \
                                 (Otherwise put it into your hand.)"
                                    .to_string(),
                        });
                        let cast = matches!(answer, DecisionAnswer::Bool(true));
                        exiled.retain(|&x| x != cid);
                        if cast {
                            let auto_target = self.auto_target_for_effect_avoiding(
                                &card_def.effect,
                                p,
                                Some(cid),
                            );
                            let cast_events = self.cast_card_for_free(
                                p, cid, Zone::Exile, auto_target, vec![], None, None, false,
                            )?;
                            events.extend(cast_events);
                        } else {
                            // Decline → put the matched card into hand.
                            self.move_card_to(cid, &ZoneDest::Hand(PlayerRef::Seat(p)), ctx, events);
                        }
                    }
                }
                // Bottom the remaining exiled cards (random order ≈ bottom).
                for cid in exiled {
                    if self.exile.iter().any(|c| c.id == cid) {
                        self.move_card_to(
                            cid,
                            &ZoneDest::Library {
                                who: PlayerRef::Seat(p),
                                pos: LibraryPosition::Bottom,
                            },
                            ctx,
                            events,
                        );
                    }
                }
                Ok(())
            }

            Effect::CollectEvidence { amount, then } => {
                // CR 701.59 — the controller may exile cards with total MV ≥
                // `amount` from their graveyard; if they do, the reflexive
                // `then` payoff resolves. Engine auto-picks the cheapest
                // qualifying set.
                use crate::decision::{Decision, DecisionAnswer};
                use crate::effect::ZoneDest;
                let p = ctx.controller;
                let need = self.evaluate_value(amount, ctx).max(0) as u32;
                let mut gy: Vec<(CardId, u32)> = self.players[p]
                    .graveyard
                    .iter()
                    .map(|c| (c.id, c.definition.cost.cmc()))
                    .collect();
                gy.sort_by_key(|&(_, mv)| mv);
                let total: u32 = gy.iter().map(|&(_, mv)| mv).sum();
                if total < need {
                    return Ok(()); // can't collect enough evidence
                }
                let src = ctx.source.unwrap_or(CardId(0));
                let to_exile: Vec<CardId> = if self.players[p].wants_ui {
                    // A human picks exactly which cards to exile; the engine
                    // validates the chosen set clears the MV threshold and
                    // otherwise treats the answer as a decline.
                    let mv: std::collections::HashMap<CardId, u32> =
                        gy.iter().copied().collect();
                    let candidates: Vec<(CardId, String)> = self.players[p]
                        .graveyard
                        .iter()
                        .map(|c| (c.id, c.definition.name.to_string()))
                        .collect();
                    let answer = self.decider.decide(&Decision::ChooseCards {
                        source: src,
                        prompt: format!(
                            "Collect evidence {need}: exile cards from your \
                             graveyard with total mana value {need}+"
                        ),
                        candidates,
                        min: 0,
                        max: mv.len() as u32,
                    });
                    let chosen: Vec<CardId> = match answer {
                        DecisionAnswer::Cards(ids) => {
                            ids.into_iter().filter(|id| mv.contains_key(id)).collect()
                        }
                        _ => vec![],
                    };
                    let picked: u32 = chosen.iter().map(|id| mv[id]).sum();
                    if picked < need {
                        return Ok(()); // declined or insufficient evidence
                    }
                    chosen
                } else {
                    let answer = self.decider.decide(&Decision::OptionalTrigger {
                        source: src,
                        description: format!(
                            "Collect evidence {need}? (exile cards from your graveyard \
                             with total mana value {need} or greater)"
                        ),
                    });
                    if !matches!(answer, DecisionAnswer::Bool(true)) {
                        return Ok(());
                    }
                    let mut acc = 0u32;
                    let mut auto = Vec::new();
                    for (cid, mv) in gy {
                        if acc >= need {
                            break;
                        }
                        acc += mv;
                        auto.push(cid);
                    }
                    auto
                };
                for cid in to_exile {
                    self.move_card_to(cid, &ZoneDest::Exile, ctx, events);
                }
                // The "when you do" payoff is a reflexive trigger: its targets
                // are chosen now, after collecting. Auto-target `then` and
                // thread the picks through a derived context.
                let (slot0, additional) =
                    self.auto_targets_for_effect_all_slots(then, p, None);
                let mut then_ctx = ctx.clone();
                then_ctx.targets = slot0.into_iter().chain(additional).collect();
                self.run_effect(then, &then_ctx, events)?;
                Ok(())
            }

            Effect::Forage { then } => {
                // CR 701.61 — exile three cards from your graveyard or sacrifice
                // a Food. Optional: ask the controller; on yes, pay (prefer
                // exiling three graveyard cards, else sacrifice a Food) and run
                // the reflexive payoff.
                use crate::card::ArtifactSubtype;
                use crate::decision::{Decision, DecisionAnswer};
                use crate::effect::ZoneDest;
                let p = ctx.controller;
                let gy_ids: Vec<CardId> =
                    self.players[p].graveyard.iter().map(|c| c.id).collect();
                let food = self.battlefield.iter().find(|c| {
                    c.controller == p
                        && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Food)
                }).map(|c| c.id);
                let can_exile = gy_ids.len() >= 3;
                if !can_exile && food.is_none() {
                    return Ok(()); // can't forage
                }
                let src = ctx.source.unwrap_or(CardId(0));
                let answer = self.decider.decide(&Decision::OptionalTrigger {
                    source: src,
                    description: "Forage? (exile three cards from your graveyard \
                        or sacrifice a Food)".into(),
                });
                if !matches!(answer, DecisionAnswer::Bool(true)) {
                    return Ok(());
                }
                if can_exile {
                    for cid in gy_ids.into_iter().take(3) {
                        self.move_card_to(cid, &ZoneDest::Exile, ctx, events);
                    }
                } else if let Some(fid) = food {
                    events.push(GameEvent::PermanentSacrificed { card_id: fid, who: p });
                    let mut die = self.remove_to_graveyard_with_triggers(fid);
                    events.append(&mut die);
                }
                let (slot0, additional) =
                    self.auto_targets_for_effect_all_slots(then, p, None);
                let mut then_ctx = ctx.clone();
                then_ctx.targets = slot0.into_iter().chain(additional).collect();
                self.run_effect(then, &then_ctx, events)?;
                Ok(())
            }

            Effect::CastWithoutPayingImmediate {
                what,
                source_zone,
                exile_after,
            } => {
                // Resolve `what` to a single card in `source_zone`, ask
                // the controller via OptionalTrigger, and on yes hand
                // off to the free-cast helper. The helper auto-targets /
                // auto-modes the card; ScriptedDecider can override.
                let entities = self.resolve_selector(what, ctx);
                // A targeted graveyard/exile card resolves to `Permanent` (the
                // only `Target` card variant); a `LastMoved`/zone selector may
                // yield `Card`. Accept either.
                let card_id = entities.into_iter().find_map(|e| match e {
                    EntityRef::Card(id) | EntityRef::Permanent(id) => Some(id),
                    _ => None,
                });
                let Some(card_id) = card_id else { return Ok(()); };
                // Confirm the card is actually in the named zone — the
                // selector may have read a stale target.
                if self.find_card_zone(card_id) != Some(*source_zone) {
                    return Ok(());
                }
                // Lands are played, not cast — skip them silently. This
                // makes `ForEach LastMoved → CastWithoutPayingImmediate`
                // safe to use over a mixed top-of-library exile (e.g.
                // Improvisation Capstone exiles whatever's on top).
                let is_land = self
                    .find_card_anywhere(card_id)
                    .map(|c| c.definition.card_types.contains(&crate::card::CardType::Land))
                    .unwrap_or(false);
                if is_land { return Ok(()); }
                use crate::decision::{Decision, DecisionAnswer};
                let source_for_ask = ctx.source.unwrap_or(CardId(0));
                let answer = self.decider.decide(&Decision::OptionalTrigger {
                    source: source_for_ask,
                    description: "Cast without paying?".to_string(),
                });
                let yes = matches!(answer, DecisionAnswer::Bool(true));
                if !yes {
                    return Ok(());
                }
                // Auto-pick a target for the freshly-cast spell. Targets
                // are picked from the controller's perspective (avoiding
                // the cast card itself).
                let card_def = self
                    .find_card_anywhere(card_id)
                    .map(|c| c.definition.clone());
                let Some(card_def) = card_def else { return Ok(()); };
                let auto_target = self.auto_target_for_effect_avoiding(
                    &card_def.effect,
                    ctx.controller,
                    Some(card_id),
                );
                let cast_events = self.cast_card_for_free(
                    ctx.controller,
                    card_id,
                    *source_zone,
                    auto_target,
                    vec![],
                    None,
                    None,
                    *exile_after,
                )?;
                events.extend(cast_events);
                Ok(())
            }

            Effect::CastFromHandWithoutPaying { filter } => {
                use crate::decision::{Decision, DecisionAnswer};
                let p = ctx.controller;
                let candidates: Vec<(CardId, String)> = self.players[p]
                    .hand
                    .iter()
                    .filter(|c| !c.definition.is_land())
                    .filter(|c| {
                        filter
                            .as_ref()
                            .is_none_or(|f| self.evaluate_requirement_on_card(f, c, p))
                    })
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                if candidates.is_empty() {
                    return Ok(());
                }
                let answer = self.decider.decide(&Decision::ChooseCards {
                    source: ctx.source.unwrap_or(CardId(0)),
                    prompt: "Cast which spell without paying its mana cost?".to_string(),
                    candidates: candidates.clone(),
                    min: 0,
                    max: 1,
                });
                let DecisionAnswer::Cards(picked) = answer else { return Ok(()) };
                let Some(card_id) = picked
                    .into_iter()
                    .find(|id| candidates.iter().any(|(c, _)| c == id))
                else {
                    return Ok(());
                };
                let card_def = self
                    .find_card_anywhere(card_id)
                    .map(|c| c.definition.clone());
                let Some(card_def) = card_def else { return Ok(()) };
                let auto_target =
                    self.auto_target_for_effect_avoiding(&card_def.effect, p, Some(card_id));
                let cast_events = self.cast_card_for_free(
                    p,
                    card_id,
                    crate::card::Zone::Hand,
                    auto_target,
                    vec![],
                    None,
                    None,
                    false,
                )?;
                events.extend(cast_events);
                Ok(())
            }

            Effect::Tribute { n, otherwise } => {
                // CR 702.104 — an opponent may put N +1/+1 counters on the
                // source; if they decline, the "if tribute wasn't paid"
                // trigger half runs. Seat-routed to the opponent, so a
                // networked human answers their own tribute prompt.
                let Some(source) = ctx.source else { return Ok(()) };
                let opp = self
                    .opponents_of(ctx.controller)
                    .first()
                    .copied()
                    .unwrap_or(ctx.controller);
                let mut cursor = 0;
                let Some(yes) = self.ask_seat_bool(
                    &mut cursor,
                    opp,
                    format!("Pay tribute: put {n} +1/+1 counter(s) on it?"),
                    source,
                    effect,
                ) else {
                    return Ok(());
                };
                self.clear_answer_log();
                if yes {
                    self.run_effect(
                        &Effect::AddCounter {
                            what: Selector::This,
                            kind: crate::card::CounterType::PlusOnePlusOne,
                            amount: crate::effect::Value::Const(*n as i32),
                        },
                        ctx,
                        events,
                    )?;
                } else {
                    self.run_effect(otherwise, ctx, events)?;
                }
                Ok(())
            }

            Effect::RegisterParadigm => {
                // Register a recurring `YourNextMainPhase` delayed
                // trigger whose body is `Effect::CastFreeParadigmCopy`.
                // The trigger's `source` is the resolving Paradigm
                // card's id; combined with `CardDefinition.exile_on_resolve
                // = true`, the card lands in exile and stays reachable
                // for the recurrence. `fires_once: false` so the trigger
                // survives each firing and fans out again next turn.
                use crate::game::types::{DelayedKind, DelayedTrigger};
                let Some(source) = ctx.source else { return Ok(()); };
                self.delayed_triggers.push(DelayedTrigger {
                    controller: ctx.controller,
                    source,
                    kind: DelayedKind::YourNextMainPhase,
                    effect: Effect::CastFreeParadigmCopy,
                    target: None,
                    bound_token: None,
                    fires_once: false,
                });
                Ok(())
            }

            Effect::CastFreeParadigmCopy => {
                // Find the original Paradigm-exiled card by id (the
                // trigger's `source`). If it's missing from exile (e.g.
                // someone Bojuka-Bogged the exile zone — unlikely in
                // SOS but defensive), drop the trigger.
                use crate::card::Zone;
                let source = ctx.source.unwrap_or(CardId(0));
                let original_def = self
                    .exile
                    .iter()
                    .find(|c| c.id == source)
                    .map(|c| c.definition.clone());
                let Some(def) = original_def else { return Ok(()); };
                // Ask the controller "cast a copy?"
                use crate::decision::{Decision, DecisionAnswer};
                let answer = self.decider.decide(&Decision::OptionalTrigger {
                    source,
                    description: format!("Cast a copy of {}?", def.name),
                });
                if !matches!(answer, DecisionAnswer::Bool(true)) {
                    return Ok(());
                }
                // Mint a tokenized copy of the exiled card in exile.
                let new_id = self.next_id();
                let mut copy = crate::card::CardInstance::new(new_id, def.clone(), ctx.controller);
                copy.is_token = true;
                self.exile.push(copy);
                // Auto-target for the copy.
                let auto_target = self.auto_target_for_effect_avoiding(
                    &def.effect,
                    ctx.controller,
                    Some(new_id),
                );
                // Free-cast from exile. Drop the result events into our
                // surrounding events buffer.
                let cast_events = self.cast_card_for_free(
                    ctx.controller,
                    new_id,
                    Zone::Exile,
                    auto_target,
                    vec![],
                    None,
                    None,
                    false,
                )?;
                events.extend(cast_events);
                Ok(())
            }

            Effect::Cipher => {
                // CR 702.46 — the controller may exile this spell card encoded
                // on a creature they control. Auto-pick the highest-power
                // untapped creature (the likeliest attacker); a wants_ui seat is
                // offered a yes/no. On yes, flag the post-resolution routing.
                use crate::decision::{Decision, DecisionAnswer};
                let ctrl = ctx.controller;
                let mut creatures: Vec<(CardId, i32)> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == ctrl && c.definition.is_creature())
                    .map(|c| (c.id, c.power()))
                    .collect();
                if creatures.is_empty() {
                    return Ok(());
                }
                creatures.sort_by_key(|(_, p)| -*p);
                let source = ctx.source.unwrap_or(CardId(0));
                let answer = self.decider.decide(&Decision::OptionalTrigger {
                    source,
                    description: "Cipher: encode this spell on a creature you control?".into(),
                });
                if matches!(answer, DecisionAnswer::Bool(true)) {
                    self.cipher_encode_pending = Some(creatures[0].0);
                }
                Ok(())
            }

            Effect::CreateEmblem { who, name, triggered } => {
                for ent in self.resolve_selector(&Selector::Player(who.clone()), ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].emblems.push(crate::player::Emblem {
                            name: name.clone(),
                            triggered: triggered.clone(),
                        });
                    }
                }
                Ok(())
            }

            Effect::SkipTurns { who, count } => {
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                if n == 0 { return Ok(()); }
                for ent in self.resolve_selector(&Selector::Player(who.clone()), ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].skip_turns =
                            self.players[p].skip_turns.saturating_add(n);
                    }
                }
                Ok(())
            }

            Effect::TakeExtraTurn { who, count } => {
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                if n == 0 { return Ok(()); }
                for ent in self.resolve_selector(&Selector::Player(who.clone()), ctx) {
                    if let EntityRef::Player(p) = ent {
                        self.players[p].extra_turns =
                            self.players[p].extra_turns.saturating_add(n);
                    }
                }
                Ok(())
            }

            Effect::AdditionalCombatPhase { count } => {
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                self.additional_combat_phases =
                    self.additional_combat_phases.saturating_add(n);
                Ok(())
            }

            Effect::AdditionalCombatPhaseAfterMain { count } => {
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                self.additional_post_main_combats =
                    self.additional_post_main_combats.saturating_add(n);
                Ok(())
            }

            Effect::WinGame { who } => {
                // CR 104.2a — "you win the game". Resolve `who` to a single
                // player and eliminate every other (non-eliminated) player.
                // The SBA pass after this resolution will pick up the
                // 1-alive-player state and promote it to
                // `game_over = Some(winner)`. We don't directly set
                // `game_over` here so that anything resolving after this
                // step (in the same Seq) can still observe normal state;
                // the SBA loop is the canonical "the game ends" gate.
                let winner = self
                    .resolve_selector(&Selector::Player(who.clone()), ctx)
                    .into_iter()
                    .find_map(|e| match e {
                        EntityRef::Player(p) => Some(p),
                        _ => None,
                    });
                if let Some(w) = winner {
                    for (idx, pl) in self.players.iter_mut().enumerate() {
                        if idx != w {
                            pl.eliminated = true;
                        }
                    }
                }
                Ok(())
            }
            Effect::LoseGame { who } => {
                // CR 104.3a — eliminate the named player; the SBA loop
                // promotes the last player standing.
                if let Some(loser) = self.resolve_player(who, ctx) {
                    self.players[loser].eliminated = true;
                    let mut sba = self.check_state_based_actions();
                    events.append(&mut sba);
                }
                Ok(())
            }
            Effect::GrantExtraLandPlay { who, count } => {
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                if let Some(target) = self.resolve_player(who, ctx) {
                    self.players[target].extra_land_plays =
                        self.players[target].extra_land_plays.saturating_add(n);
                }
                Ok(())
            }
        }
    }

    pub(crate) fn execute_put_on_library(
        &mut self,
        player: usize,
        chosen: &[crate::card::CardId],
        _events: &mut Vec<crate::game::GameEvent>,
    ) {
        // Remove chosen cards from hand (in reverse order to preserve indices).
        let mut cards_to_insert: Vec<crate::card::CardInstance> = Vec::new();
        for &id in chosen {
            if let Some(pos) = self.players[player].hand.iter().position(|c| c.id == id) {
                cards_to_insert.push(self.players[player].hand.remove(pos));
            }
        }
        // Insert in reverse so that chosen[0] ends up on top.
        for card in cards_to_insert.into_iter().rev() {
            self.players[player].library.insert(0, card);
        }
    }

    // ── Selector / Value / Predicate resolution ─────────────────────────────

    /// CR 615.7 / 609.7b — pick a damage source matching `filter` among
    /// stack spells and battlefield permanents, asking the controller when
    /// there's more than one (AutoDecider keeps the first candidate —
    /// stack spells offered before permanents). Shared by the
    /// chosen-source prevention effects (Burrenton Forge-Tender, Circle of
    /// Protection).
    fn choose_damage_prevention_source(
        &mut self,
        filter: &crate::card::SelectionRequirement,
        ctx: &EffectContext,
    ) -> Option<CardId> {
        use crate::decision::{Decision, DecisionAnswer};
        let p = ctx.controller;
        let mut candidates: Vec<(CardId, String)> = self
            .stack
            .iter()
            .filter_map(|item| match item {
                crate::game::types::StackItem::Spell { card, .. }
                    if self.evaluate_requirement_on_card(filter, card, p) =>
                {
                    Some((card.id, card.definition.name.to_string()))
                }
                _ => None,
            })
            .collect();
        candidates.extend(
            self.battlefield
                .iter()
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| (c.id, c.definition.name.to_string())),
        );
        if candidates.is_empty() {
            return None;
        }
        if candidates.len() == 1 {
            return Some(candidates[0].0);
        }
        let source = ctx.source.unwrap_or(CardId(0));
        let answer = self.decider.decide(&Decision::ChooseCards {
            source,
            prompt: "Prevent damage from which source?".to_string(),
            candidates: candidates.clone(),
            min: 1,
            max: 1,
        });
        match answer {
            DecisionAnswer::Cards(v) if !v.is_empty() => Some(v[0]),
            _ => Some(candidates[0].0),
        }
    }

    /// Map a selector to the `PreventionTarget`s it designates (players and
    /// permanents only). Used by the prevention-shield effects.
    fn prevention_targets(
        &self,
        sel: &Selector,
        ctx: &EffectContext,
    ) -> Vec<crate::game::types::PreventionTarget> {
        use crate::game::types::PreventionTarget;
        self.resolve_selector(sel, ctx)
            .into_iter()
            .filter_map(|e| match e {
                EntityRef::Player(p) => Some(PreventionTarget::Player(p)),
                EntityRef::Permanent(c) => Some(PreventionTarget::Permanent(c)),
                EntityRef::Card(_) => None,
            })
            .collect()
    }

    pub(crate) fn resolve_selector(&self, sel: &Selector, ctx: &EffectContext) -> Vec<EntityRef> {
        // Multi-candidate selectors run layer-aware filters per permanent;
        // freeze the gather so the whole resolution shares one effect set.
        // Hand-rolled (no closure/guard) — this sits inside the effect
        // recursion, where debug-build frame size is at a premium.
        self.freeze_layers_push();
        let out = self.resolve_selector_inner(sel, ctx);
        self.freeze_layers_pop();
        out
    }

    fn resolve_selector_inner(&self, sel: &Selector, ctx: &EffectContext) -> Vec<EntityRef> {
        match sel {
            Selector::None => vec![],
            Selector::This => ctx.source.map(EntityRef::Permanent).into_iter().collect(),
            Selector::You => vec![EntityRef::Player(ctx.controller)],
            Selector::Target(idx) | Selector::TargetFiltered { slot: idx, .. } => ctx
                .targets
                .get(*idx as usize)
                .map(target_to_entity)
                .into_iter()
                .collect(),
            Selector::TriggerSource => ctx.trigger_source.into_iter().collect(),
            Selector::BlockedAttacker => ctx
                .source
                .and_then(|blocker| self.block_map.get(&blocker).copied())
                .filter(|aid| self.battlefield.iter().any(|c| c.id == *aid))
                .map(EntityRef::Permanent)
                .into_iter()
                .collect(),
            Selector::CardExiledWithSource => self
                .exile
                .iter()
                .filter(|c| ctx.source.is_some() && c.exiled_with == ctx.source)
                .map(|c| EntityRef::Permanent(c.id))
                .collect(),
            Selector::LastCreatedToken => self
                .last_created_token
                // Delayed-trigger fire: the scheduling resolution's token
                // rides in as `trigger_source` (DelayedTrigger.bound_token).
                .or(match ctx.trigger_source {
                    Some(EntityRef::Permanent(id))
                        if self
                            .battlefield
                            .iter()
                            .any(|c| c.id == id && c.is_token) =>
                    {
                        Some(id)
                    }
                    _ => None,
                })
                .filter(|id| self.battlefield.iter().any(|c| c.id == *id))
                .map(EntityRef::Permanent)
                .into_iter()
                .collect(),
            Selector::LastCreatedTokens => self
                .last_created_tokens
                .iter()
                .copied()
                .filter(|id| self.battlefield.iter().any(|c| c.id == *id))
                .map(EntityRef::Permanent)
                .collect(),
            Selector::LastMoved => self
                .last_moved_cards
                .iter()
                .copied()
                // A card moved onto the battlefield (Search/reveal → bf) is now
                // a permanent — surface it as one so battlefield-only effects
                // (AddCounter, BecomeCreature) act on it (Emergent Sequence).
                .map(|cid| {
                    if self.battlefield_find(cid).is_some() {
                        EntityRef::Permanent(cid)
                    } else {
                        EntityRef::Card(cid)
                    }
                })
                .collect(),
            Selector::CastSpellTarget(slot) => {
                // Walk the stack for the spell whose SpellCast event fired
                // this trigger — that's the topmost matching `Spell` whose
                // card id matches `ctx.trigger_source` (set by
                // `fire_spell_cast_triggers`). Pull the target slot off it.
                let cast_id = match ctx.trigger_source {
                    Some(EntityRef::Card(cid)) | Some(EntityRef::Permanent(cid)) => Some(cid),
                    _ => None,
                };
                let Some(cid) = cast_id else { return vec![]; };
                let target = self.stack.iter().rev().find_map(|si| match si {
                    StackItem::Spell { card, target, .. } if card.id == cid => Some(target.clone()),
                    _ => None,
                });
                match target {
                    Some(Some(t)) if *slot == 0 => vec![target_to_entity(&t)],
                    _ => vec![],
                }
            }

            Selector::SharingNameWith(inner) => {
                // Resolve the anchor, read its printed name, then collect
                // every battlefield permanent (anchor included) with that
                // name. The anchor's name is read from the battlefield so a
                // freshly-resolved target still matches.
                let anchor = self
                    .resolve_selector(inner, ctx)
                    .into_iter()
                    .find_map(|e| e.as_permanent_id());
                let Some(anchor_id) = anchor else { return vec![] };
                let Some(name) = self.battlefield_find(anchor_id).map(|c| c.definition.name)
                else {
                    return vec![];
                };
                self.battlefield
                    .iter()
                    .filter(|c| c.definition.name == name)
                    .map(|c| EntityRef::Permanent(c.id))
                    .collect()
            }

            Selector::EachMatching { zone, filter } => self.entities_in_zone(zone, filter, ctx),
            Selector::EachPermanent(filter) => self
                .battlefield
                .iter()
                .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller, ctx.source))
                .map(|c| EntityRef::Permanent(c.id))
                .collect(),

            Selector::ControlledBy { who, filter } => {
                let Some(p) = self.resolve_player(who, ctx) else { return vec![]; };
                self.battlefield
                    .iter()
                    .filter(|c| c.controller == p)
                    .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller, ctx.source))
                    .map(|c| EntityRef::Permanent(c.id))
                    .collect()
            }

            // CR 701.21 — the controller's least-toughness creature.
            Selector::LeastToughnessYouControl => self
                .battlefield
                .iter()
                .filter(|c| c.controller == ctx.controller && c.definition.is_creature())
                .min_by_key(|c| c.toughness())
                .map(|c| EntityRef::Permanent(c.id))
                .into_iter()
                .collect(),

            Selector::GreatestPowerYouControl => self
                .battlefield
                .iter()
                .filter(|c| c.controller == ctx.controller && c.definition.is_creature())
                .max_by_key(|c| c.power())
                .map(|c| EntityRef::Permanent(c.id))
                .into_iter()
                .collect(),

            // The least-power creature among ALL players' (Porphyry Nodes).
            // Battlefield-order tie-break stands in for "you choose one".
            Selector::LeastPowerAmongAll => self
                .battlefield
                .iter()
                .filter(|c| c.definition.is_creature())
                .min_by_key(|c| c.power())
                .map(|c| EntityRef::Permanent(c.id))
                .into_iter()
                .collect(),

            Selector::AttachedTo(inner) => self
                .resolve_selector(inner, ctx)
                .into_iter()
                .filter_map(|e| {
                    let EntityRef::Permanent(cid) = e else { return None; };
                    // Last-known-info fallback (CR 603.10): an Aura's
                    // "when this leaves, do X to enchanted creature" trigger
                    // resolves after the Aura is already in the graveyard, so
                    // read `attached_to` from the die-time snapshot when the
                    // source is no longer on the battlefield (Parallax Dementia).
                    self.battlefield_find(cid)
                        .or_else(|| self.died_card_snapshots.get(&cid))
                        .and_then(|c| c.attached_to)
                        .map(EntityRef::Permanent)
                })
                .collect(),

            Selector::AttachedToMe(inner) => {
                let anchors: Vec<CardId> = self
                    .resolve_selector(inner, ctx)
                    .into_iter()
                    .filter_map(|e| e.as_permanent_id())
                    .collect();
                self.battlefield
                    .iter()
                    .filter(|c| c.attached_to.is_some_and(|a| anchors.contains(&a)))
                    .map(|c| EntityRef::Permanent(c.id))
                    .collect()
            }

            Selector::TopOfLibrary { who, count } => {
                let Some(p) = self.resolve_player(who, ctx) else { return vec![]; };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                self.players[p]
                    .library
                    .iter()
                    .take(n)
                    .map(|c| EntityRef::Card(c.id))
                    .collect()
            }
            Selector::TopOfLibraryUntilMvAtLeast { who, threshold } => {
                let Some(p) = self.resolve_player(who, ctx) else { return vec![]; };
                let cap = self.evaluate_value(threshold, ctx).max(0);
                let mut sum: i32 = 0;
                let mut out: Vec<EntityRef> = Vec::new();
                for c in self.players[p].library.iter() {
                    out.push(EntityRef::Card(c.id));
                    sum += c.definition.cost.cmc() as i32;
                    if sum >= cap {
                        break;
                    }
                }
                out
            }
            Selector::BottomOfLibrary { who, count } => {
                let Some(p) = self.resolve_player(who, ctx) else { return vec![]; };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let lib = &self.players[p].library;
                let total = lib.len();
                if n >= total {
                    lib.iter().map(|c| EntityRef::Card(c.id)).collect()
                } else {
                    lib.iter().skip(total - n).map(|c| EntityRef::Card(c.id)).collect()
                }
            }
            Selector::CardsInZone { who, zone, filter } => {
                // Use the multi-player resolver so EachPlayer / EachOpponent
                // aggregate cards from every matching seat (Soul-Guide
                // Lantern's mass-graveyard exile, Bojuka Bog–style effects,
                // future Windfall-shape primitives all need this).
                let players = self.resolve_players(who, ctx);
                let mut out: Vec<EntityRef> = Vec::new();
                for p in players {
                    let cards: Vec<&CardInstance> = match zone {
                        Zone::Hand => self.players[p].hand.iter().collect(),
                        Zone::Graveyard => self.players[p].graveyard.iter().collect(),
                        Zone::Library => self.players[p].library.iter().collect(),
                        Zone::Exile => self.exile.iter().filter(|c| c.owner == p).collect(),
                        Zone::Battlefield => self.battlefield.iter().filter(|c| c.controller == p).collect(),
                        Zone::Stack | Zone::Command => vec![],
                    };
                    // For battlefield-resident cards we use the
                    // permanent-state-aware `evaluate_requirement_static`
                    // (Tapped, IsAttacking, etc. resolve correctly). For
                    // hand/library/graveyard/exile we use the card-level
                    // evaluator since those zones don't have permanent
                    // state — `evaluate_requirement_static` only walks
                    // battlefield-then-graveyard-then-exile-then-stack
                    // and would silently miss hand-resident cards
                    // (push XVI fix — was breaking Embrace the Paradox's
                    // MayDo land sub).
                    let on_bf = matches!(zone, Zone::Battlefield);
                    // CR — honor `OtherThanSource` in hidden-zone searches too
                    // (the on-card evaluator has no source context, so it
                    // can't drop the source itself — e.g. Ichorid exiling a
                    // black creature *other than itself* from its own gy).
                    let exclude_source = requirement_mentions_other_than_source(filter);
                    out.extend(
                        cards
                            .into_iter()
                            .filter(|c| {
                                if !on_bf && exclude_source && ctx.source == Some(c.id) {
                                    return false;
                                }
                                if on_bf {
                                    self.evaluate_requirement_static(
                                        filter, &Target::Permanent(c.id), ctx.controller, ctx.source,
                                    )
                                } else {
                                    self.evaluate_requirement_on_card(filter, c, ctx.controller)
                                }
                            })
                            .map(|c| if on_bf {
                                EntityRef::Permanent(c.id)
                            } else {
                                EntityRef::Card(c.id)
                            }),
                    );
                }
                out
            }

            Selector::DiscardedThisResolution { filter } => {
                // Walk the IDs captured in `discarded_card_ids_this_resolution`
                // and look them up in their owner's graveyard (where
                // `Effect::Discard` moves them). Filter via the card-level
                // evaluator since the discarded cards aren't on the
                // battlefield.
                let ids = self.discarded_card_ids_this_resolution.clone();
                let mut out: Vec<EntityRef> = Vec::new();
                for cid in ids {
                    let card = self.players.iter()
                        .find_map(|p| p.graveyard.iter().find(|c| c.id == cid));
                    if let Some(c) = card
                        && self.evaluate_requirement_on_card(filter, c, ctx.controller)
                    {
                        out.push(EntityRef::Card(cid));
                    }
                }
                out
            }

            Selector::Player(p) => self
                .resolve_players(p, ctx)
                .into_iter()
                .map(EntityRef::Player)
                .collect(),

            Selector::Take { inner, count } => {
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                if n == 0 {
                    return vec![];
                }
                let mut all = self.resolve_selector(inner, ctx);
                all.truncate(n);
                all
            }

            Selector::TakeWithSumCap { inner, cap, value_of_each } => {
                let cap_n = self.evaluate_value(cap, ctx).max(0);
                if cap_n == 0 {
                    return vec![];
                }
                let candidates = self.resolve_selector(inner, ctx);
                let mut running_total: i32 = 0;
                let mut kept: Vec<EntityRef> = Vec::new();
                for ent in candidates {
                    // Bind the candidate to `ctx.trigger_source` so that
                    // `value_of_each` can reference it via
                    // `Selector::TriggerSource` (mirrors `Effect::ForEach`'s
                    // binding convention). Per-iteration sub-ctx clone keeps
                    // outer ctx untouched after evaluation.
                    let mut sub_ctx = ctx.clone();
                    sub_ctx.trigger_source = Some(ent);
                    let v = self.evaluate_value(value_of_each, &sub_ctx).max(0);
                    if running_total + v <= cap_n {
                        running_total += v;
                        kept.push(ent);
                    }
                    // Otherwise skip this candidate; iteration continues so
                    // smaller items can still fit. Greedy walk gives the
                    // AutoDecider a deterministic pick.
                }
                kept
            }
        }
    }

    /// Multi-player resolver. `EachPlayer` and `EachOpponent` return all
    /// matching alive seats so effects like Wheel of Fortune actually hit
    /// every player. Non-collective `PlayerRef` variants resolve to a single
    /// seat (or empty if the reference can't be resolved).
    pub(crate) fn resolve_players(&self, pref: &PlayerRef, ctx: &EffectContext) -> Vec<usize> {
        match pref {
            // CR 101.4 / 121.2c — "each player"/"each opponent" fan-outs
            // resolve in APNAP order (active player first, then turn order),
            // not raw seat index.
            PlayerRef::EachOpponent => self.apnap_sort(
                self.opponents_of(ctx.controller)
                    .into_iter()
                    .filter(|i| self.players[*i].is_alive())
                    .collect(),
            ),
            PlayerRef::EachPlayer => self.apnap_sort(
                (0..self.players.len())
                    .filter(|i| self.players[*i].is_alive())
                    .collect(),
            ),
            PlayerRef::EachPlayerExceptControllerOf(sel) => {
                let excl = self.resolve_selector(sel, ctx).into_iter().find_map(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => {
                        self.battlefield_find(cid).map(|c| c.controller)
                    }
                    EntityRef::Player(p) => Some(p),
                });
                self.apnap_sort(
                    (0..self.players.len())
                        .filter(|i| self.players[*i].is_alive() && Some(*i) != excl)
                        .collect(),
                )
            }
            _ => self.resolve_player(pref, ctx).into_iter().collect(),
        }
    }

    /// Apply a resolved `LearnChoice` (CR 701.45): reveal a Lesson from the
    /// sideboard into hand, rummage (discard then draw), or decline. Shared
    /// by the synchronous `Effect::Learn` path and the UI resume in
    /// `apply_pending_effect_answer`.
    pub(crate) fn apply_learn_choice(
        &mut self,
        p: usize,
        choice: crate::decision::LearnChoice,
        events: &mut Vec<GameEvent>,
    ) {
        use crate::decision::LearnChoice;
        match choice {
            LearnChoice::FetchLesson(cid) => {
                if let Some(pos) = self.players[p].sideboard.iter().position(|c| c.id == cid) {
                    let card = self.players[p].sideboard.remove(pos);
                    self.players[p].hand.push(card);
                }
            }
            LearnChoice::Rummage { discard } => {
                if self.players[p].hand.iter().any(|c| c.id == discard) {
                    self.discard_card(p, discard, events);
                    if !self.draw_one(p, events) {
                        self.lose_to_empty_draw(p);
                    }
                }
            }
            LearnChoice::Decline => {}
        }
    }

    pub(crate) fn resolve_player(&self, pref: &PlayerRef, ctx: &EffectContext) -> Option<usize> {
        match pref {
            PlayerRef::You => Some(ctx.controller),
            PlayerRef::Seat(p) => Some(*p),
            PlayerRef::ActivePlayer => Some(self.active_player_idx),
            PlayerRef::Triggerer => ctx.trigger_source.and_then(|e| match e {
                EntityRef::Player(p) => Some(p),
                // A card trigger-source (e.g. a SpellCast trigger) resolves to
                // the caster of the spell on the stack, falling back to the
                // permanent's controller. Lets "whenever a player casts their
                // Nth spell" read the *caster's* tally (Ledger Shredder).
                EntityRef::Card(cid) | EntityRef::Permanent(cid) => self
                    .stack
                    .iter()
                    .find_map(|s| match s {
                        StackItem::Spell { card, caster, .. } if card.id == cid => Some(*caster),
                        _ => None,
                    })
                    .or_else(|| self.battlefield_find(cid).map(|c| c.controller))
                    // A just-drawn / discarded card lives in a player's
                    // hand/graveyard/etc. — bind Triggerer to that player
                    // (Sheoldred's "whenever an opponent draws a card, they
                    // lose 2 life"; Strict Tutelage).
                    .or_else(|| {
                        self.players.iter().position(|pl| {
                            pl.hand.iter().chain(&pl.graveyard).any(|c| c.id == cid)
                        })
                    }),
            }),
            PlayerRef::Target(idx) => ctx.targets.get(*idx as usize).and_then(|t| match t {
                Target::Player(p) => Some(*p),
                _ => None,
            }),
            PlayerRef::EachOpponent => {
                // Singular fallback — `resolve_players` returns the full set.
                self.opponents_of(ctx.controller)
                    .into_iter()
                    .find(|i| self.players[*i].is_alive())
            }
            PlayerRef::EachPlayer => (0..self.players.len()).find(|i| self.players[*i].is_alive()),
            PlayerRef::EachPlayerExceptControllerOf(_) => {
                self.resolve_players(pref, ctx).into_iter().next()
            }
            PlayerRef::DefendingPlayer => ctx
                .source
                .and_then(|src| self.attack_for(src).map(|a| a.target))
                .and_then(|target| self.defender_for(target))
                // "Whenever a [creature] you control attacks" triggers fire
                // off *another* permanent, so `ctx.source` (the ability's
                // owner) isn't the attacker — read the defending player from
                // the triggering attacker instead. (Leeching Sliver.)
                .or_else(|| match ctx.trigger_source {
                    Some(EntityRef::Permanent(att)) => self
                        .attack_for(att)
                        .map(|a| a.target)
                        .and_then(|target| self.defender_for(target)),
                    _ => None,
                })
                // Fallback for post-combat-damage triggers: by the time a
                // `DealsCombatDamageToPlayer` body resolves, the attack
                // record is gone, so `attack_for` returns nothing. The
                // dispatcher stamps the damaged player as the trigger's
                // `Target::Player`, so read it back here.
                .or_else(|| {
                    ctx.targets.iter().find_map(|t| match t {
                        Target::Player(p) => Some(*p),
                        _ => None,
                    })
                }),
            PlayerRef::OwnerOf(sel) => self
                .resolve_selector(sel, ctx)
                .into_iter()
                .find_map(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => {
                        self.find_card_owner(cid)
                    }
                    _ => None,
                }),
            // Resolved per-card inside `place_card_in_dest`; meaningless here.
            PlayerRef::OwnerOfMoved => None,
            PlayerRef::ControllerOf(sel) => self
                .resolve_selector(sel, ctx)
                .into_iter()
                .find_map(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => self
                        .battlefield_find(cid)
                        .map(|c| c.controller)
                        .or_else(|| self.stack_caster_for_card(cid))
                        .or_else(|| self.find_card_owner(cid)),
                    _ => None,
                }),
        }
    }

    fn entities_in_zone(
        &self,
        zone: &ZoneRef,
        filter: &SelectionRequirement,
        ctx: &EffectContext,
    ) -> Vec<EntityRef> {
        match zone {
            ZoneRef::Battlefield => self
                .battlefield
                .iter()
                .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller, ctx.source))
                .map(|c| EntityRef::Permanent(c.id))
                .collect(),
            ZoneRef::Stack => self
                .stack
                .iter()
                .filter_map(|si| match si {
                    StackItem::Spell { card, .. } => Some(EntityRef::Permanent(card.id)),
                    _ => None,
                })
                .collect(),
            ZoneRef::Library(who) | ZoneRef::Hand(who) | ZoneRef::Graveyard(who) => {
                // Multi-player aware: an `EachPlayer` / `EachOpponent` ref
                // expands to every matching player's zone, not just the
                // first. Previously this resolved to a single player only,
                // silently dropping the remaining graveyards / hands.
                // Powers Devious Cover-Up's "exile any number of target
                // cards from graveyards" (across all gys), Necrogenesis's
                // total-creature-count payoff, and any other cross-player
                // zone iteration.
                let players = self.resolve_players(who, ctx);
                let mut out = Vec::new();
                for p in players {
                    let cards: Vec<&CardInstance> = match zone {
                        ZoneRef::Library(_) => self.players[p].library.iter().collect(),
                        ZoneRef::Hand(_) => self.players[p].hand.iter().collect(),
                        ZoneRef::Graveyard(_) => self.players[p].graveyard.iter().collect(),
                        _ => vec![],
                    };
                    out.extend(
                        cards
                            .into_iter()
                            .filter(|c| {
                                self.evaluate_requirement_static(
                                    filter,
                                    &Target::Permanent(c.id),
                                    ctx.controller,
                                    ctx.source,
                                )
                            })
                            .map(|c| EntityRef::Card(c.id)),
                    );
                }
                out
            }
            ZoneRef::Exile => self
                .exile
                .iter()
                .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), ctx.controller, ctx.source))
                .map(|c| EntityRef::Card(c.id))
                .collect(),
            ZoneRef::Command => vec![],
        }
    }

}

fn target_to_entity(t: &Target) -> EntityRef {
    match t {
        Target::Player(p) => EntityRef::Player(*p),
        Target::Permanent(c) => EntityRef::Permanent(*c),
    }
}

/// True if a requirement tree contains `OtherThanSource` anywhere. Used to
/// apply source-exclusion in hidden-zone searches where the on-card
/// evaluator can't see the source.
fn requirement_mentions_other_than_source(req: &crate::card::SelectionRequirement) -> bool {
    use crate::card::SelectionRequirement as R;
    match req {
        R::OtherThanSource => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_mentions_other_than_source(a)
                || requirement_mentions_other_than_source(b)
        }
        R::Not(inner) => requirement_mentions_other_than_source(inner),
        _ => false,
    }
}


// ── Out-of-line resolver arm bodies ──────────────────────────────────────────
// These rarely-hit arms live in their own functions so their locals don't
// inflate `run_effect`'s (already enormous) debug-build stack frame —
// deep effect recursion overflows the default test stack otherwise.
impl GameState {

    #[inline(never)]
    fn resolve_exchange_control_choosing(
        &mut self,
        filter: &crate::card::SelectionRequirement,
        with: &crate::effect::Selector,
        ctx: &EffectContext,
        _events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
                // CR 701.12 — the controller picks one of their own matching
                // permanents, then swaps controllers with the resolved
                // target (Vedalken Plotter).
                use crate::decision::{Decision, DecisionAnswer};
                let Some(target) = self
                    .resolve_selector(with, ctx)
                    .into_iter()
                    .find_map(|e| e.as_permanent_id())
                else {
                    return Ok(());
                };
                let mine = crate::effect::Selector::EachPermanent(
                    filter.clone().and(crate::card::SelectionRequirement::ControlledByYou),
                );
                let candidates: Vec<CardId> = self
                    .resolve_selector(&mine, ctx)
                    .into_iter()
                    .filter_map(|e| e.as_permanent_id())
                    .filter(|&id| id != target)
                    .collect();
                let Some(first) = candidates.first().copied() else { return Ok(()) };
                let chosen = if candidates.len() > 1 {
                    let cands = candidates
                        .iter()
                        .filter_map(|&id| {
                            self.battlefield_find(id)
                                .map(|c| (id, c.definition.name.to_string()))
                        })
                        .collect();
                    match self.decider.decide(&Decision::ChooseCards {
                        source: ctx.source.unwrap_or(CardId(0)),
                        prompt: "Exchange control of which permanent?".into(),
                        candidates: cands,
                        min: 1,
                        max: 1,
                    }) {
                        DecisionAnswer::Cards(ids) => {
                            ids.into_iter().find(|id| candidates.contains(id)).unwrap_or(first)
                        }
                        _ => first,
                    }
                } else {
                    first
                };
                // CR 701.12 / 302.6 — simultaneous swap; both sides pick up
                // summoning sickness under their new controller.
                let ctrl_a = self.battlefield_find(chosen).map(|c| c.controller);
                let ctrl_b = self.battlefield_find(target).map(|c| c.controller);
                if let (Some(ctrl_a), Some(ctrl_b)) = (ctrl_a, ctrl_b)
                    && ctrl_a != ctrl_b
                {
                    if let Some(c) = self.battlefield_find_mut(chosen) {
                        c.controller = ctrl_b;
                        c.summoning_sick = true;
                    }
                    if let Some(c) = self.battlefield_find_mut(target) {
                        c.controller = ctrl_a;
                        c.summoning_sick = true;
                    }
                }
                Ok(())
            }


    #[inline(never)]
    fn resolve_mill_until_lands(
        &mut self,
        who: &crate::effect::Selector,
        lands: &crate::effect::Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
                // Reveal from the top until `lands` land cards, then put all
                // revealed cards into the graveyard (Mind Grind).
                let want = self.evaluate_value(lands, ctx).max(0) as usize;
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        let mut found = 0usize;
                        while found < want && !self.players[p].library.is_empty() {
                            let card = self.players[p].library.remove(0);
                            if card.definition.is_land() {
                                found += 1;
                            }
                            let cid = card.id;
                            if !self.route_to_graveyard(card, events) {
                                events.push(GameEvent::CardMilled { player: p, card_id: cid });
                            }
                            self.last_moved_cards.push(cid);
                        }
                    }
                }
                Ok(())
            }


    #[inline(never)]
    fn resolve_mill_two_repeat_shared_color(
        &mut self,
        who: &crate::effect::Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
                // Mill two; repeat while two nonland milled cards share a
                // color (Sphinx's Tutelage). Capped at the library size so a
                // degenerate loop always terminates.
                for ent in self.resolve_selector(who, ctx) {
                    if let EntityRef::Player(p) = ent {
                        loop {
                            let mut milled_colors: Vec<Vec<crate::mana::Color>> = Vec::new();
                            for _ in 0..2 {
                                if self.players[p].library.is_empty() {
                                    break;
                                }
                                let card = self.players[p].library.remove(0);
                                if !card.definition.is_land() {
                                    milled_colors.push(card.definition.cost.colors());
                                }
                                let cid = card.id;
                                if !self.route_to_graveyard(card, events) {
                                    events.push(GameEvent::CardMilled { player: p, card_id: cid });
                                }
                            }
                            let repeat = milled_colors.len() == 2
                                && milled_colors[0].iter().any(|c| milled_colors[1].contains(c));
                            if !repeat || self.players[p].library.is_empty() {
                                break;
                            }
                        }
                    }
                }
                Ok(())
            }


    #[inline(never)]
    fn resolve_manifest_from_hand(
        &mut self,
        who: &crate::effect::Selector,
        count: &crate::effect::Value,
        controller_draws: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
                // CR 701.34 from the hand (Kozilek): the resolved player
                // manifests `count` cards from their hand (their choice —
                // auto-picks from the front for bots/tests); the effect's
                // controller draws one per card manifested.
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                for ent in self.resolve_selector(who, ctx) {
                    let EntityRef::Player(p) = ent else { continue };
                    let mut manifested = 0u32;
                    for _ in 0..n {
                        let Some(cid) = self.players[p].hand.first().map(|c| c.id) else { break };
                        self.manifest_card(cid, p, ctx, events);
                        manifested += 1;
                    }
                    if controller_draws {
                        for _ in 0..manifested {
                            self.draw_one(ctx.controller, events);
                        }
                    }
                }
                Ok(())
            }


    #[inline(never)]
    fn resolve_create_token_attached_to(
        &mut self,
        target: &crate::effect::Selector,
        definition: &crate::card::TokenDefinition,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
                // CR 111.10 Role-style token Aura: mint, then attach.
                let Some(tid) = self
                    .resolve_selector(target, ctx)
                    .into_iter()
                    .find_map(|e| e.as_permanent_id())
                else {
                    return Ok(());
                };
                if self.battlefield_find(tid).is_none() {
                    return Ok(());
                }
                let def = token_to_card_definition(definition);
                let minted = self.mint_token_onto_battlefield(def, ctx.controller, false, events);
                if let Some(c) = self.battlefield_find_mut(minted) {
                    c.attached_to = Some(tid);
                }
                Ok(())
            }


    #[inline(never)]
    fn resolve_destroy_targets_polymorph(
        &mut self,
        filter: &crate::card::SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
                // Destroy up to X chosen targets, then each destroyed
                // permanent's controller reveals to the first artifact or
                // creature card, battlefields it, and shuffles the rest in.
                use crate::effect::RevealMissDest;
                let x = ctx.x_value as usize;
                let mut victims: Vec<(u8, usize)> = Vec::new();
                let mut seen = std::collections::HashSet::new();
                for (i, t) in ctx.targets.iter().enumerate().take(x) {
                    if let Target::Permanent(id) = t
                        && seen.insert(*id)
                        && self.evaluate_requirement_static(filter, t, ctx.controller, ctx.source)
                        && let Some(c) = self.battlefield_find(*id)
                    {
                        victims.push((i as u8, c.controller));
                    }
                }
                let mut owed: Vec<usize> = Vec::new();
                for (slot, controller) in victims {
                    let before = self.permanents_destroyed_this_resolution;
                    self.run_effect(
                        &Effect::Destroy { what: crate::effect::Selector::Target(slot) },
                        ctx,
                        events,
                    )?;
                    if self.permanents_destroyed_this_resolution > before {
                        owed.push(controller);
                    }
                }
                let find = crate::card::SelectionRequirement::Artifact
                    .or(crate::card::SelectionRequirement::Creature);
                for p in owed {
                    self.run_effect(
                        &Effect::RevealUntilFind {
                            who: crate::effect::PlayerRef::Seat(p),
                            find: find.clone(),
                            to: crate::effect::ZoneDest::Battlefield {
                                controller: crate::effect::PlayerRef::Seat(p),
                                tapped: false,
                            },
                            cap: crate::effect::Value::Const(500),
                            life_per_revealed: 0,
                            miss_dest: RevealMissDest::ShuffleIntoLibrary,
                        },
                        ctx,
                        events,
                    )?;
                }
                Ok(())
            }

    /// CR 702.77 — Champion a [filter]: exile another matching permanent you
    /// control linked to the source (returned by `return_linked_exiles` when
    /// the source leaves), or sacrifice the source if nothing was exiled.
    fn resolve_champion(
        &mut self,
        filter: &crate::card::SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(source) = ctx.source else { return Ok(()) };
        let p = ctx.controller;
        // Lowest-power match keeps the better body on the battlefield.
        let pick = self
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == p
                    && c.id != source
                    && self.evaluate_requirement_on_card(filter, c, p)
            })
            .min_by_key(|c| (c.power(), c.id))
            .map(|c| c.id);
        match pick {
            Some(cid) => {
                self.remove_from_battlefield_to_exile(cid);
                events.push(GameEvent::PermanentExiled { card_id: cid });
                if let Some(c) = self.exile.iter_mut().find(|c| c.id == cid) {
                    c.exiled_by = Some(crate::card::ExileLink {
                        source,
                        return_to: crate::card::ExileReturnZone::Battlefield,
                    });
                }
            }
            None => self.sacrifice_one(source, p, events),
        }
        Ok(())
    }

    /// Exile up to `count` cards from any graveyards, chosen by the
    /// controller's decider (Faerie Macabre). Optional — bots take none.
    fn resolve_exile_up_to_n_from_graveyards(
        &mut self,
        count: &crate::effect::Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use crate::decision::{Decision, DecisionAnswer};
        let n = self.evaluate_value(count, ctx).max(0) as u32;
        if n == 0 {
            return Ok(());
        }
        let candidates: Vec<(CardId, String)> = self
            .players
            .iter()
            .flat_map(|p| p.graveyard.iter())
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        let answer = self.decider.decide(&Decision::ChooseCards {
            source: ctx.source.unwrap_or(CardId(0)),
            prompt: format!("Exile up to {n} cards from graveyards"),
            candidates: candidates.clone(),
            min: 0,
            max: n,
        });
        if let DecisionAnswer::Cards(ids) = answer {
            for cid in ids.into_iter().take(n as usize) {
                if candidates.iter().any(|(c, _)| *c == cid) {
                    self.move_card_to(cid, &ZoneDest::Exile, ctx, events);
                }
            }
        }
        Ok(())
    }

    /// Oona — choose a color, exile the top `amount` cards of the resolved
    /// player's library, and mint one `token` per exiled card of that color.
    fn resolve_exile_top_mint_per_chosen_color(
        &mut self,
        who: &crate::effect::Selector,
        amount: &crate::effect::Value,
        token: &crate::card::TokenDefinition,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use crate::decision::{Decision, DecisionAnswer};
        use crate::mana::Color;
        let Some(EntityRef::Player(victim)) = self.resolve_selector(who, ctx).into_iter().next()
        else {
            return Ok(());
        };
        let n = self.evaluate_value(amount, ctx).max(0) as usize;
        let answer = self.decider.decide(&Decision::ChooseColor {
            source: ctx.source.unwrap_or(CardId(0)),
            legal: vec![Color::White, Color::Blue, Color::Black, Color::Red, Color::Green],
        });
        let color = match answer {
            DecisionAnswer::Color(c) => c,
            _ => Color::Blue,
        };
        let mut matches = 0;
        for _ in 0..n {
            if self.players[victim].library.is_empty() {
                break;
            }
            let card = self.players[victim].library.remove(0);
            if card.definition.cost.colors().contains(&color) {
                matches += 1;
            }
            let cid = card.id;
            self.place_card_in_dest(card, victim, &ZoneDest::Exile, events);
            self.last_moved_cards.push(cid);
        }
        if matches > 0 {
            self.run_effect(
                &Effect::CreateToken {
                    who: crate::effect::PlayerRef::You,
                    count: crate::effect::Value::Const(matches),
                    definition: token.clone(),
                },
                ctx,
                events,
            )?;
        }
        Ok(())
    }

    #[inline(never)]
    fn resolve_sacrifice_all_but_one_per_type(
        &mut self,
        who: &crate::effect::Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
                use crate::card::CardType;
                for ent in self.resolve_selector(who, ctx) {
                    let EntityRef::Player(p) = ent else { continue };
                    let mut keep: Vec<CardId> = Vec::new();
                    for ty in [
                        CardType::Artifact,
                        CardType::Creature,
                        CardType::Enchantment,
                        CardType::Planeswalker,
                    ] {
                        let pick = self
                            .battlefield
                            .iter()
                            .filter(|c| {
                                c.controller == p
                                    && !c.definition.is_land()
                                    && c.definition.card_types.contains(&ty)
                                    && !keep.contains(&c.id)
                            })
                            .max_by_key(|c| c.definition.cost.cmc())
                            .map(|c| c.id);
                        if let Some(id) = pick {
                            keep.push(id);
                        }
                    }
                    let to_sac: Vec<CardId> = self
                        .battlefield
                        .iter()
                        .filter(|c| {
                            c.controller == p && !c.definition.is_land() && !keep.contains(&c.id)
                        })
                        .map(|c| c.id)
                        .collect();
                    for id in to_sac {
                        self.sacrifice_one(id, p, events);
                    }
                }
                Ok(())
            }


    #[inline(never)]
    fn resolve_wish_to_hand(
        &mut self,
        filter: &crate::card::SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
                // Sideboard first ("outside the game"), then own exiled
                // cards; a wants_ui controller picks via ChooseCards.
                use crate::decision::{Decision, DecisionAnswer};
                let p = ctx.controller;
                let mut candidates: Vec<(CardId, bool)> = self.players[p]
                    .sideboard
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                    .map(|c| (c.id, true))
                    .collect();
                candidates.extend(
                    self.exile
                        .iter()
                        .filter(|c| {
                            c.owner == p && self.evaluate_requirement_on_card(filter, c, p)
                        })
                        .map(|c| (c.id, false)),
                );
                let Some(&(first, _)) = candidates.first() else { return Ok(()) };
                let chosen = if candidates.len() > 1 && self.players[p].wants_ui {
                    let cands = candidates
                        .iter()
                        .filter_map(|&(id, sb)| {
                            let name = if sb {
                                self.players[p].sideboard.iter().find(|c| c.id == id)
                            } else {
                                self.exile.iter().find(|c| c.id == id)
                            }
                            .map(|c| c.definition.name.to_string())?;
                            Some((id, name))
                        })
                        .collect();
                    match self.decider.decide(&Decision::ChooseCards {
                        source: ctx.source.unwrap_or(CardId(0)),
                        prompt: "Put which card into your hand?".into(),
                        candidates: cands,
                        min: 1,
                        max: 1,
                    }) {
                        DecisionAnswer::Cards(ids) => ids
                            .first()
                            .copied()
                            .filter(|id| candidates.iter().any(|(c, _)| c == id))
                            .unwrap_or(first),
                        _ => first,
                    }
                } else {
                    first
                };
                let card = if let Some(pos) =
                    self.players[p].sideboard.iter().position(|c| c.id == chosen)
                {
                    Some(self.players[p].sideboard.remove(pos))
                } else {
                    self.exile
                        .iter()
                        .position(|c| c.id == chosen)
                        .map(|pos| self.exile.remove(pos))
                };
                if let Some(card) = card {
                    let cid = card.id;
                    self.players[p].hand.push(card);
                    events.push(GameEvent::CardDrawn { player: p, card_id: cid });
                }
                Ok(())
            }

}
