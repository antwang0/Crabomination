//! Commander-specific effect resolution (CR 903).
//!
//! Its own module rather than another arm body in `effects/mod.rs`, which is
//! 30k lines: the arms here call in, so the Commander rules stay readable and
//! the big match keeps one line per variant.

use crate::card::{CardId, CardInstance, CreatureType, Keyword, KeywordSlice};
use crate::decision::{Decision, DecisionAnswer};
use crate::effect::{Effect, ManaPayload, PlayerRef, Value};
use crate::game::GameState;
use crate::game::types::TriggerPush;
use crate::mana::{SpellKind, SpendRestriction};

impl GameState {
    /// Geode Golem — cast one of `seat`'s commanders from the command zone
    /// without paying its mana cost (the tax is still paid; see
    /// `cast_from_command_zone_as`). With two in the zone (Partner) the one
    /// with the greater mana value goes, the one the discount is worth most
    /// on. A cast that fails leaves the commander where it was.
    pub(crate) fn cast_commander_without_paying(
        &mut self,
        seat: usize,
        events: &mut Vec<crate::game::types::GameEvent>,
    ) {
        let Some(id) = self.players.get(seat).and_then(|p| {
            p.command
                .iter()
                .filter(|c| p.commanders.contains(&c.id))
                .max_by_key(|c| c.definition.cost.cmc())
                .map(|c| c.id)
        }) else {
            return;
        };
        if let Ok(mut evs) = self.cast_from_command_zone_as(seat, id, None, vec![], None, None, true) {
            events.append(&mut evs);
        }
    }

    /// Saheeli, the Gifted's +1 — the next spell `seat` casts this turn has
    /// affinity for artifacts. Keyed, like `pending_spell_discounts`, on the
    /// spells-cast tally so it lapses with the next spell; the count itself is
    /// taken at cast time by `cost_reduction_for_spell`.
    /// Dance with Calamity — exile `seat`'s top card while the exiled total
    /// mana value is below `stop_at`; true when that total is `limit` or
    /// less. `place_card_in_dest` records each card for
    /// `Selector::ExiledThisResolution`.
    pub(crate) fn exile_top_pushing_luck(
        &mut self,
        seat: usize,
        stop_at: u32,
        limit: u32,
        events: &mut Vec<crate::game::types::GameEvent>,
    ) -> bool {
        let mut total = 0u32;
        while total < stop_at && !self.players[seat].library.is_empty() {
            let card = self.players[seat].library.remove(0);
            total += card.definition.cost.cmc();
            self.place_card_in_dest(card, seat, &crate::effect::ZoneDest::Exile, events);
        }
        total <= limit
    }

    /// Bell Borca's note: raise the turn's greatest exiled mana value.
    pub(crate) fn note_exiled_mana_value(&mut self, mv: u32) {
        let mv = mv.min(u8::MAX as u32) as u8;
        if mv > self.greatest_exiled_mv_this_turn {
            self.greatest_exiled_mv_this_turn = mv;
        }
    }

    pub(crate) fn grant_next_spell_affinity(&mut self, seat: usize) {
        if let Some(p) = self.players.get_mut(seat) {
            let at = p.spells_cast_this_turn;
            p.pending_affinity_next_spell.push(at);
        }
    }

    /// CR 903.3 / 202.3 — the greatest printed mana value among `seat`'s
    /// commanders, in whatever zone each is; 0 with none.
    pub(crate) fn greatest_commander_mana_value(&self, seat: usize) -> u32 {
        self.players.get(seat).map_or(0, |p| {
            p.commanders
                .iter()
                .filter_map(|&id| self.find_card_anywhere(id))
                .map(|c| c.definition.cost.cmc())
                .max()
                .unwrap_or(0)
        })
    }

    /// CR 903.8 / Command Beacon — move one of `seat`'s commanders from the
    /// command zone to their hand.
    ///
    /// The CR 903.9b replacement ("if a commander would be put into its
    /// owner's hand … that player may put it into the command zone instead")
    /// is optional, and a player using this ability always declines it —
    /// applying it would make the ability do nothing — so the move is direct.
    ///
    /// With two commanders (Partner) the controller chooses which one; a
    /// headless seat takes the first, which is what `ChooseCards`'s forced
    /// `min == 1` default does.
    ///
    /// No event: the engine has no generic zone-change event, and the two it
    /// does have for reaching a hand (`PermanentReturnedToHand`) describe a
    /// permanent leaving the battlefield, which this is not.
    pub(crate) fn commander_to_hand_from_command_zone(
        &mut self,
        seat: usize,
        source: Option<CardId>,
    ) {
        let Some(chosen) =
            self.choose_commander_in_command_zone(seat, source, "Put which commander into your hand?")
        else {
            return;
        };
        let Some(pos) = self.players[seat].command.iter().position(|c| c.id == chosen) else {
            return;
        };
        let card = self.players[seat].command.remove(pos);
        self.players[seat].hand.push(card);
        self.offboard_keyword_grants = true;
    }

    /// One of `seat`'s commanders that is in the command zone: the only one,
    /// or — with two (Partner) — the one the decider picks (`ChooseCards`,
    /// `min == 1`; a headless seat takes the first). `None` when neither is
    /// there.
    fn choose_commander_in_command_zone(
        &mut self,
        seat: usize,
        source: Option<CardId>,
        prompt: &str,
    ) -> Option<CardId> {
        let player = self.players.get(seat)?;
        let candidates: Vec<(CardId, String)> = player
            .command
            .iter()
            .filter(|c| player.commanders.contains(&c.id))
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        match candidates.len() {
            0 => None,
            1 => Some(candidates[0].0),
            _ => {
                let answer = self.decider.decide(&Decision::ChooseCards {
                    source: source.unwrap_or(CardId(0)),
                    prompt: prompt.into(),
                    candidates: candidates.clone(),
                    min: 1,
                    max: 1,
                    eligible: None,
                    value: crate::decision::PickValue::Gain,
                });
                Some(match answer {
                    DecisionAnswer::Cards(ids)
                        if ids.first().is_some_and(|id| {
                            candidates.iter().any(|(c, _)| c == id)
                        }) =>
                    {
                        ids[0]
                    }
                    _ => candidates[0].0,
                })
            }
        }
    }

    /// `Effect::PutCommanderOntoBattlefield` — Hellkite Courser's "put a
    /// commander you own from the command zone onto the battlefield. It gains
    /// haste. Return it to the command zone at the beginning of the next end
    /// step."
    ///
    /// Not a cast (CR 903.8): the commander tax and `commander_cast_count` are
    /// untouched. The entrant goes on `Selector::LastMoved`.
    ///
    /// The return is a CR 603.7a delayed trigger sourced on the commander
    /// itself, gated on `SourceOnBattlefield`: a commander that already left
    /// (and went home through CR 903.9) isn't chased into another zone.
    /// ⚠ The engine keeps one `CardId` across zone changes, so a commander
    /// that left and was recast the same turn is still returned — the
    /// CR 400.7 new-object rule is not modelled (as for Sneak Attack's
    /// sacrifice).
    pub(crate) fn put_commander_onto_battlefield(
        &mut self,
        seat: usize,
        source: Option<CardId>,
        haste: bool,
        return_at_end_step: bool,
        controller: usize,
        events: &mut Vec<crate::game::GameEvent>,
    ) {
        use crate::effect::{Predicate, Selector, ZoneDest};
        let Some(chosen) = self.choose_commander_in_command_zone(
            seat,
            source,
            "Put which commander onto the battlefield?",
        ) else {
            return;
        };
        let Some(card) = Self::take_card(&mut self.players[seat].command, chosen) else {
            return;
        };
        self.offboard_keyword_grants = true;
        self.place_card_in_dest(
            card,
            seat,
            &ZoneDest::Battlefield { controller: PlayerRef::Seat(seat), tapped: false },
            events,
        );
        if self.battlefield_find(chosen).is_none() {
            return;
        }
        self.scratch.last_moved_cards.push(chosen);
        if haste {
            self.grant_keyword_eot(chosen, Keyword::Haste);
        }
        if return_at_end_step {
            // CR 400.7 — "return it": only the object that entered here, not
            // the same card back on the battlefield after a trip elsewhere.
            // Bound to its entry stamp when this batch's entry is stamped.
            let battlefield_timestamp = crate::effect::UNBOUND_OBJECT_STAMP;
            self.delayed_triggers.push(crate::game::types::DelayedTrigger {
                controller,
                source: chosen,
                kind: crate::game::types::DelayedKind::NextEndStep,
                effect: Effect::If {
                    cond: Predicate::SourceIsSameObjectOnBattlefield { battlefield_timestamp },
                    then: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Command }),
                    else_: Box::new(Effect::Noop),
                },
                target: None,
                bound_token: None,
                bound_subject: None,
                fires_once: true,
                expires_after_turn: None,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Mana provenance — CR 106.6's "when that mana is spent to …" (Path of
// Ancestry's delayed trigger, Opal Palace's additional effect). The pool
// doesn't remember which permanent produced a pip; what it does remember is
// the pip's `SpendRestriction`, and a *rider* is a restriction that permits
// everything and only cares what it funded. The payment already reports the
// riders it spent through `PaymentSideEffects::spent_restrictions`, so
// provenance is that report read by the cast path, exactly like Cavern of
// Souls' uncounterable rider.
//
// ⚠ Approximation, CR 106.6a: `spent_restrictions` is a set, so two rider
// pips spent on one cast (Mana Reflection doubling the source) fire one
// trigger where the rule makes one per mana produced. Counting them would
// widen `PaymentSideEffects` for an interaction no target deck has.
// ---------------------------------------------------------------------------

/// The scry triggers a cast's commander-mana riders owe, put on the stack
/// once the spell is: Path of Ancestry's pips (scry 1 each, if the spell
/// shares a creature type with the commander) and Study Hall's `(pips, X)`.
#[derive(Default, Clone, Copy)]
pub(crate) struct CommanderManaScry {
    path_pips: u32,
    study: (u32, u32),
    /// Pyromancer's Goggles — rider pips that funded a red instant or
    /// sorcery, and that spell.
    copies: (u32, Option<CardId>),
    /// Sunken Palace — rider pips that funded any spell, and that spell.
    palace_copies: (u32, Option<CardId>),
}

impl GameState {
    /// `CardDefinition::spell_kind` plus the one field only the game can fill:
    /// CR 903.3 — whether `card` is `seat`'s own commander — and whether a
    /// graveyard-cast permission hopped it into hand (CR 601.2a).
    pub fn spell_kind_for(&self, seat: usize, card: &CardInstance) -> SpellKind {
        let mut kind = card.definition.spell_kind();
        kind.from_graveyard =
            self.casting_hop == Some((card.id, crate::game::HopFrom::Graveyard));
        kind.commander = self
            .players
            .get(seat)
            .is_some_and(|p| !p.commanders.is_empty() && p.commanders.contains(&card.id));
        kind
    }

    /// Apply the cast-time half of the commander mana riders and report how
    /// many Path of Ancestry triggers still have to go on the stack.
    ///
    /// Opal Palace's counters are stamped on the cast card's own
    /// `pending_etb_counters` rather than the caster's shared
    /// `pending_creature_etb_counters`: the rider names *that* object, and a
    /// creature cast in response would otherwise eat them.
    ///
    /// Call after the command-zone cast counter has been bumped — the ruling
    /// (2020-11-10) counts the cast in progress.
    #[must_use]
    pub(crate) fn note_commander_mana_riders(
        &mut self,
        spent: &crate::mana::PaymentSideEffects,
        kind: &SpellKind,
        card: &mut CardInstance,
    ) -> CommanderManaScry {
        if spent.spent_restrictions.is_empty() {
            return CommanderManaScry::default();
        }
        // CR 106.6a — "a separate effect … once for each mana produced", so
        // two doubled Opal Palace pips are two counters per prior cast.
        let pips = spent.spent_count(SpendRestriction::CommanderCastCounters);
        if kind.commander && pips > 0 {
            let casts = self.commander_cast_count.get(&card.id).copied().unwrap_or(0);
            let n = casts.saturating_mul(pips);
            if n > 0 {
                card.pending_etb_counters
                    .push((crate::card::CounterType::PlusOnePlusOne, n));
            }
        }
        // Forger's Foundry — the same provenance hook: the rider names the
        // cast object (CR 106.6), so it rides the card like Opal Palace's.
        if kind.instant_or_sorcery
            && card.definition.cost.cmc() <= 3
            && spent.spent(SpendRestriction::SmallInstantSorceryExileInstead)
        {
            card.exile_with_on_resolve = self.restricted_mana_source(
                self.priority.player_with_priority,
                SpendRestriction::SmallInstantSorceryExileInstead,
            );
        }
        // Pyromancer's Goggles — one copy trigger per rider pip that funded a
        // red instant or sorcery (CR 106.6: the rider names that spell).
        let goggles = spent.spent_count(SpendRestriction::RedInstantSorceryCopy);
        let goggles = if kind.instant_or_sorcery && kind.colors.contains(crate::mana::Color::Red) {
            goggles
        } else {
            0
        };
        // Sunken Palace — its rider copies whatever spell it funded.
        let palace = spent.spent_count(SpendRestriction::SpellOrAbilityCopy);
        let copies = if goggles > 0 { (goggles, Some(card.id)) } else { (0, None) };
        let palace_copies = if palace > 0 { (palace, Some(card.id)) } else { (0, None) };
        // Study Hall — one scry-X trigger per rider pip that funded the
        // commander, X counting the cast in progress (like Opal Palace).
        let study_pips = spent.spent_count(SpendRestriction::CommanderCastScry);
        let study = if kind.commander && study_pips > 0 {
            (study_pips, self.commander_cast_count.get(&card.id).copied().unwrap_or(0))
        } else {
            (0, 0)
        };
        CommanderManaScry {
            path_pips: if kind.creature {
                spent.spent_count(SpendRestriction::CommanderTypeScry)
            } else {
                0
            },
            study,
            copies,
            palace_copies,
        }
    }

    /// CR 603.3 — Path of Ancestry's trigger, put on the stack once the spell
    /// it funded is already there, so it resolves first.
    ///
    /// The shared-type check reads the commander's types *now* (2023-07-28
    /// ruling: they're checked immediately after the cast, not at activation),
    /// layer-aware while it's on the battlefield.
    /// CR 106.6a — one trigger per rider pip spent, not one per cast.
    pub(crate) fn push_commander_mana_scry(
        &mut self,
        seat: usize,
        kind: &SpellKind,
        riders: CommanderManaScry,
    ) {
        // CR 707.10 — Pyromancer's Goggles: "copy that spell and you may
        // choose new targets for the copy", above the spell it funded.
        if let (copy_pips @ 1.., Some(spell)) = riders.copies {
            let source = self
                .restricted_mana_source(seat, SpendRestriction::RedInstantSorceryCopy)
                .unwrap_or(CardId(0));
            for _ in 0..copy_pips {
                self.stack.push(
                    TriggerPush::new(
                        source,
                        seat,
                        Effect::CopySpellMayChooseTargets {
                            what: crate::effect::Selector::TriggerSource,
                            count: Value::ONE,
                        },
                    )
                    .trigger_source(Some(crate::game::effects::EntityRef::Card(spell)))
                    .build(),
                );
            }
        }
        if let (copy_pips @ 1.., Some(spell)) = riders.palace_copies {
            let source = self
                .restricted_mana_source(seat, SpendRestriction::SpellOrAbilityCopy)
                .unwrap_or(CardId(0));
            for _ in 0..copy_pips {
                self.stack.push(
                    TriggerPush::new(
                        source,
                        seat,
                        Effect::CopySpellMayChooseTargets {
                            what: crate::effect::Selector::TriggerSource,
                            count: Value::ONE,
                        },
                    )
                    .trigger_source(Some(crate::game::effects::EntityRef::Card(spell)))
                    .build(),
                );
            }
        }
        let (study_pips, x) = riders.study;
        if study_pips > 0 && x > 0 {
            let source = self
                .restricted_mana_source(seat, SpendRestriction::CommanderCastScry)
                .unwrap_or(CardId(0));
            for _ in 0..study_pips {
                self.stack.push(
                    TriggerPush::new(
                        source,
                        seat,
                        Effect::Scry { who: PlayerRef::You, amount: Value::Const(x as i32) },
                    )
                    .build(),
                );
            }
        }
        let pips = riders.path_pips;
        if pips == 0 || !self.commander_shares_creature_type(seat, kind) {
            return;
        }
        let source = self
            .restricted_mana_source(seat, SpendRestriction::CommanderTypeScry)
            .unwrap_or(CardId(0));
        for _ in 0..pips {
            self.stack.push(
                TriggerPush::new(
                    source,
                    seat,
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                )
                .build(),
            );
        }
    }

    /// True iff a creature spell with `kind`'s types shares one with any of
    /// `seat`'s commanders. Changeling on either side matches everything
    /// (CR 702.73); a commander with no creature types shares nothing.
    fn commander_shares_creature_type(&self, seat: usize, kind: &SpellKind) -> bool {
        let Some(player) = self.players.get(seat) else { return false };
        let ids: Vec<CardId> = player.commanders.to_vec();
        ids.into_iter().any(|id| {
            let (types, changeling): (Vec<CreatureType>, bool) =
                match self.computed_permanent(id) {
                    Some(cp) => (
                        cp.subtypes().creature_types.clone(),
                        cp.keywords().contains(&Keyword::Changeling),
                    ),
                    None => match self.find_card_anywhere(id) {
                        Some(c) => (
                            c.definition.subtypes.creature_types.clone(),
                            c.definition.keywords.has_kw(&Keyword::Changeling),
                        ),
                        None => return false,
                    },
                };
            if types.is_empty() {
                return false;
            }
            (changeling && kind.creature)
                || kind.changeling
                || types.iter().any(|t| kind.creature_types.contains(t))
        })
    }

    /// The battlefield permanent `seat` controls whose mana ability carries
    /// `r`. The pool doesn't record which permanent made a pip, so a rider's
    /// trigger recovers its source from the ability that could have produced
    /// it — exact in the ordinary case (the producer was tapped for mana this
    /// turn and is still there), `None` once it has left, where the trigger
    /// still happens with no source object.
    fn restricted_mana_source(&self, seat: usize, r: SpendRestriction) -> Option<CardId> {
        self.battlefield
            .iter()
            .find(|c| {
                c.controller == seat
                    && c.definition.activated_abilities.iter().any(|ab| {
                        matches!(
                            &ab.effect,
                            Effect::AddMana { pool: ManaPayload::Restricted(_, got), .. }
                                if *got == r
                        )
                    })
            })
            .map(|c| c.id)
    }
}

// ---------------------------------------------------------------------------
// CR 601.2c — per-opponent targeting. "For each opponent, [verb] up to one
// target X that player controls" is a target *per opponent*, and a target slot
// is declared statically by the card literal, so the arity a pod needs is not
// expressible as slots. `Effect::ForEachOpponentTarget` says it as a
// constraint on the chosen set instead: at most one target per controller, at
// most one per opponent.
// ---------------------------------------------------------------------------

impl GameState {
    /// The seat a target "belongs to" for the per-opponent constraint: the
    /// controller of a permanent, the owner of a card in a non-battlefield
    /// zone (Sepulchral and Diluvian Primordial target graveyard cards), or a
    /// player target itself. `None` when the target can't be located, which
    /// drops it from the set rather than letting it collide with everything.
    pub(crate) fn target_controller_key(
        &self,
        t: &crate::game::types::Target,
    ) -> Option<usize> {
        use crate::game::types::Target;
        match t {
            Target::Player(p) => (*p < self.players.len()).then_some(*p),
            Target::Permanent(id) => self
                .battlefield_find(*id)
                .map(|c| c.controller)
                .or_else(|| self.find_card_anywhere(*id).map(|c| c.owner)),
        }
    }
}
