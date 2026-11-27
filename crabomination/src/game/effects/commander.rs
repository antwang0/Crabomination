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
    /// Whether `id` sits in any player's command zone (CR 408).
    pub(crate) fn in_a_command_zone(&self, id: CardId) -> bool {
        self.players.iter().any(|p| p.command.iter().any(|c| c.id == id))
    }

    /// Geode Golem — cast one of `seat`'s commanders from the command zone
    /// without paying its mana cost (the tax is still paid; see
    /// `cast_from_command_zone_as`). With two in the zone (Partner) the seat
    /// picks which. A cast that fails leaves the commander where it was.
    pub(crate) fn cast_commander_without_paying(
        &mut self,
        seat: usize,
        source: Option<CardId>,
        events: &mut Vec<crate::game::types::GameEvent>,
    ) {
        let Some(id) = self.choose_commander_in_command_zone(seat, source, "Cast which commander?") else {
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
    /// Dance with Calamity — "as many times as you choose, you may exile the
    /// top card": `seat` is asked before each card (headless: while the total
    /// is below `stop_at`). True when the exiled total is `limit` or less;
    /// `None` is a suspend. The asks walk the library by index and the cards
    /// move after the last, so a re-run replays the answers (Ad Nauseam's
    /// shape). `place_card_in_dest` records each for
    /// `Selector::ExiledThisResolution`.
    pub(crate) fn exile_top_pushing_luck(
        &mut self,
        seat: usize,
        stop_at: u32,
        limit: u32,
        source: CardId,
        effect: &Effect,
        events: &mut Vec<crate::game::types::GameEvent>,
    ) -> Option<bool> {
        let ask = self.seat_prompts(seat) || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto);
        let mut total = 0u32;
        let mut n = 0usize;
        let mut cursor = 0;
        while let Some(card) = self.players[seat].library.get(n) {
            let mv = card.definition.cost.cmc();
            let more = if ask {
                self.ask_seat_bool(
                    &mut cursor,
                    seat,
                    format!("Exile the top card of your library? (exiled so far: mana value {total})"),
                    source,
                    effect,
                    crate::decision::OptionalKind::PushYourLuck { total, limit },
                )?
            } else {
                total < stop_at
            };
            if !more {
                break;
            }
            total += mv;
            n += 1;
        }
        self.clear_answer_log();
        for _ in 0..n {
            let card = self.players[seat].library.remove(0);
            self.place_card_in_dest(card, seat, &crate::effect::ZoneDest::Exile, events);
        }
        Some(total <= limit)
    }

    /// A card was put into exile: raise each noting permanent's own note
    /// (Bell Borca — CR 400.7: only what it saw while on the battlefield,
    /// `CardCold::exile_note`).
    pub(crate) fn note_exiled_mana_value(&mut self, mv: u32) {
        let mv = mv.min(u8::MAX as u32) as u8;
        let turn = self.turn_number;
        let noters: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| notes_exiled_mana_values(&c.definition))
            .filter(|c| {
                let held = c.exile_note.filter(|&(t, s, _)| t == turn && s == c.battlefield_timestamp);
                held.is_none_or(|(_, _, m)| mv > m)
            })
            .map(|c| c.id)
            .collect();
        for id in noters {
            if let Some(c) = self.battlefield_find_mut(id) {
                let stamp = c.battlefield_timestamp;
                c.exile_note = Some((turn, stamp, mv));
            }
        }
    }

    /// Bell Borca's own note (`Value::GreatestManaValueNotedForSource`).
    pub(crate) fn noted_exiled_mana_value(&self, source: Option<CardId>) -> u8 {
        source
            .and_then(|s| self.battlefield_find(s))
            .and_then(|c| {
                c.exile_note.filter(|&(t, s, _)| t == self.turn_number && s == c.battlefield_timestamp)
            })
            .map_or(0, |(_, _, m)| m)
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

    /// [`Self::greatest_commander_mana_value`] over the commanders `seat` owns
    /// on the battlefield (under anyone's control) or in their command zone.
    pub(crate) fn greatest_commander_mana_value_in_play_or_command_zone(&self, seat: usize) -> u32 {
        self.players.get(seat).map_or(0, |p| {
            p.commanders
                .iter()
                .filter_map(|&id| {
                    p.command.iter().find(|c| c.id == id).or_else(|| self.battlefield_find(id))
                })
                .map(|c| c.definition.cost.cmc())
                .max()
                .unwrap_or(0)
        })
    }

    /// CR 903.8 / Command Beacon — move one of `seat`'s commanders from the
    /// command zone to their hand.
    ///
    /// The optional CR 903.9b replacement ("may put it into the command zone
    /// instead") is always declined here — applying it would make the ability
    /// do nothing — so the move is direct. With two commanders (Partner) the
    /// controller chooses; a headless seat takes the first (`ChooseCards`'s
    /// forced `min == 1` default).
    ///
    /// No event: there's no generic zone-change event, and
    /// `PermanentReturnedToHand` describes a permanent leaving the battlefield.
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

    /// Tevesh Szat's −10: every commander in any command zone enters under
    /// `controller`'s control, seat order (deterministic).
    pub(crate) fn put_commanders_onto_battlefield(&mut self, controller: usize, events: &mut Vec<crate::game::GameEvent>) {
        let dest = crate::effect::ZoneDest::Battlefield { controller: PlayerRef::Seat(controller), tapped: false };
        for p in 0..self.players.len() {
            let ids: Vec<CardId> = self.players[p]
                .command
                .iter()
                .filter(|c| self.players[p].commanders.contains(&c.id))
                .map(|c| c.id)
                .collect();
            for id in ids {
                let Some(pos) = self.players[p].command.iter().position(|c| c.id == id) else { continue };
                let card = self.players[p].command.remove(pos);
                self.place_card_in_dest(card, p, &dest, events);
            }
        }
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
// Ancestry, Opal Palace). The pool doesn't remember which permanent made a
// pip, but it does remember the pip's `SpendRestriction`; a *rider* is a
// restriction that permits everything and only cares what it funded. The
// payment reports spent riders via `PaymentSideEffects::spent_restrictions`,
// which the cast path reads, like Cavern of Souls' uncounterable rider.
//
// ⚠ Approximation, CR 106.6a: `spent_restrictions` is a set, so two rider
// pips on one cast (Mana Reflection) fire one trigger where the rule makes
// one per mana. Counting them would widen `PaymentSideEffects` for an
// interaction no target deck has.
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
    /// Scaled Nurturer — rider pips that funded a Dragon creature spell.
    dragon_life: u32,
    /// Lapis Orb of Dragonkind — rider pips that funded a Dragon creature spell.
    dragon_scry: u32,
    /// Tecutlan — rider pips that funded a permanent spell, and its mana value.
    discover: (u32, u32),
}

impl GameState {
    /// `CardDefinition::spell_kind` plus the one field only the game can fill:
    /// CR 903.3 — whether `card` is `seat`'s own commander — and whether a
    /// graveyard-cast permission hopped it into hand or it is being cast out
    /// of exile (CR 601.2a).
    pub fn spell_kind_for(&self, seat: usize, card: &CardInstance) -> SpellKind {
        let mut kind = card.definition.spell_kind();
        kind.from_graveyard =
            self.casting_hop == Some((card.id, crate::game::HopFrom::Graveyard));
        kind.from_exile = self.exile.iter().any(|c| c.id == card.id);
        kind.not_owned = card.owner != seat;
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
        // Biophagus — any creature spell, one counter per funding pip.
        let bio = spent.spent_count(SpendRestriction::CreatureCastCounter);
        if kind.creature && bio > 0 {
            card.pending_etb_counters.push((crate::card::CounterType::PlusOnePlusOne, bio));
        }
        // Domri, Chaos Bringer — a funded creature spell gains riot: one
        // queued instance per pip, read by `apply_riot` as it enters.
        let riot = spent.spent_count(SpendRestriction::CreatureGainsRiot);
        if kind.creature {
            for _ in 0..riot {
                self.pending_etb_keyword_grants.push((card.id, Keyword::Riot, crate::effect::Duration::Permanent));
            }
        }
        // Animal Attendant — the same, for a non-Human creature spell.
        let attendant = spent.spent_count(SpendRestriction::NonHumanCreatureCastCounter);
        if kind.creature
            && attendant > 0
            && !kind.changeling
            && !kind.creature_types.contains(&CreatureType::Human)
        {
            card.pending_etb_counters.push((crate::card::CounterType::PlusOnePlusOne, attendant));
        }
        // Guildmages' Forum — the same, for a multicolored creature spell.
        let forum = spent.spent_count(SpendRestriction::MulticoloredCreatureCastCounter);
        if kind.creature && kind.multicolored && forum > 0 {
            card.pending_etb_counters.push((crate::card::CounterType::PlusOnePlusOne, forum));
        }
        // Jade Orb of Dragonkind — a Dragon creature spell enters with a
        // counter per pip and hexproof until its caster's next turn.
        let dragon = kind.creature
            && (kind.changeling || kind.creature_types.contains(&CreatureType::Dragon));
        let jade = spent.spent_count(SpendRestriction::DragonCreatureCounterHexproof);
        if dragon && jade > 0 {
            card.pending_etb_counters.push((crate::card::CounterType::PlusOnePlusOne, jade));
            self.pending_etb_keyword_grants.push((
                card.id,
                Keyword::Hexproof,
                crate::effect::Duration::UntilNextTurn,
            ));
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
        // Primal Wellspring — the same copy for any instant or sorcery.
        let goggles = goggles
            + if kind.instant_or_sorcery { spent.spent_count(SpendRestriction::InstantSorceryCopy) } else { 0 };
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
            dragon_life: if dragon { spent.spent_count(SpendRestriction::DragonCreatureGainTwo) } else { 0 },
            dragon_scry: if dragon { spent.spent_count(SpendRestriction::DragonCreatureScryTwo) } else { 0 },
            discover: if card.definition.is_permanent() {
                (spent.spent_count(SpendRestriction::PermanentSpellDiscover), card.definition.cost.cmc())
            } else {
                (0, 0)
            },
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
                .or_else(|| self.restricted_mana_source(seat, SpendRestriction::InstantSorceryCopy))
                .unwrap_or(CardId(0));
            for _ in 0..copy_pips {
                self.push_stack(
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
                self.push_stack(
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
        // Scaled Nurturer — "you gain 2 life", one trigger per pip.
        if riders.dragon_life > 0 {
            let source = self
                .restricted_mana_source(seat, SpendRestriction::DragonCreatureGainTwo)
                .unwrap_or(CardId(0));
            for _ in 0..riders.dragon_life {
                self.push_stack(
                    TriggerPush::new(
                        source,
                        seat,
                        Effect::GainLife { who: crate::effect::Selector::You, amount: Value::Const(2) },
                    )
                    .build(),
                );
            }
        }
        // Lapis Orb of Dragonkind — "scry 2", one trigger per pip.
        if riders.dragon_scry > 0 {
            let source = self
                .restricted_mana_source(seat, SpendRestriction::DragonCreatureScryTwo)
                .unwrap_or(CardId(0));
            for _ in 0..riders.dragon_scry {
                self.push_stack(
                    TriggerPush::new(source, seat, Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })
                        .build(),
                );
            }
        }
        // Tecutlan — "discover X, where X is that spell's mana value".
        let (discover_pips, mv) = riders.discover;
        if discover_pips > 0 {
            let source = self
                .restricted_mana_source(seat, SpendRestriction::PermanentSpellDiscover)
                .unwrap_or(CardId(0));
            for _ in 0..discover_pips {
                self.push_stack(
                    TriggerPush::new(source, seat, Effect::Discover { n: Value::Const(mv as i32), filter: None }).build(),
                );
            }
        }
        let (study_pips, x) = riders.study;
        if study_pips > 0 && x > 0 {
            let source = self
                .restricted_mana_source(seat, SpendRestriction::CommanderCastScry)
                .unwrap_or(CardId(0));
            for _ in 0..study_pips {
                self.push_stack(
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
            self.push_stack(
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
    pub(crate) fn restricted_mana_source(&self, seat: usize, r: SpendRestriction) -> Option<CardId> {
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

/// Does this permanent note exiled mana values for itself (Bell Borca)?
fn notes_exiled_mana_values(def: &crate::card::CardDefinition) -> bool {
    def.static_abilities.iter().any(|sa| {
        matches!(
            &sa.effect,
            crate::effect::StaticEffect::SelfBasePtFromValue {
                power: crate::effect::Value::GreatestManaValueNotedForSource,
                ..
            }
        )
    })
}

impl GameState {
    /// `Effect::ShuffleSelfRevealUntilSameNameToBattlefield` (Mirror-Mad
    /// Phantasm). CR 701.20 — the revealed misses go to the graveyard; the
    /// first card with the source's name enters under its owner's control.
    pub(crate) fn shuffle_self_reveal_until_same_name(
        &mut self,
        ctx: &crate::game::effects::EffectContext,
        events: &mut Vec<crate::game::types::GameEvent>,
    ) {
        use crate::effect::{LibraryPosition, ZoneDest};
        let Some(src) = ctx.source else { return };
        let Some((owner, name)) = self.battlefield_find(src).map(|c| (c.owner, c.definition.name)) else { return };
        self.move_card_to(src, &ZoneDest::Library { who: PlayerRef::Seat(owner), pos: LibraryPosition::Shuffled }, ctx, events);
        if !self.players[owner].library.iter().any(|c| c.id == src) {
            return;
        }
        while let Some(top) = self.players[owner].library.first() {
            let (id, hit, card_name, is_land) =
                (top.id, top.definition.name == name, top.definition.name, top.definition.is_land());
            events.push(crate::game::types::GameEvent::TopCardRevealed { player: owner, card_name, is_land });
            if hit {
                self.move_card_to(id, &ZoneDest::Battlefield { controller: PlayerRef::Seat(owner), tapped: false }, ctx, events);
                return;
            }
            self.move_card_to(id, &ZoneDest::Graveyard, ctx, events);
        }
    }

    /// `Effect::EachPlayerGainsChosenFromRightNeighbor` (Inniaz, the Gale
    /// Force). For each living seat in turn order, the controller picks one
    /// matching permanent its right neighbour (the previous living seat)
    /// controls; then each pick moves to that seat at once. A headless
    /// controller takes the best permanent for itself and hands each other
    /// seat the cheapest.
    pub(crate) fn each_player_gains_chosen_from_right(
        &mut self,
        filter: &crate::card::SelectionRequirement,
        ctx: &crate::game::effects::EffectContext,
        effect: &crate::effect::Effect,
    ) {
        use crate::game::types::Target;
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(crate::card::CardId(0));
        let n = self.players.len();
        let mut cursor = 0;
        let mut moves: Vec<(crate::card::CardId, usize)> = Vec::new();
        for seat in self.seats_in_turn_order_from(me) {
            let Some(right) = (1..n).map(|k| (seat + n - k) % n).find(|&q| self.players[q].is_alive()) else {
                continue;
            };
            if right == seat {
                continue;
            }
            let mut legal: Vec<(u32, crate::card::CardId)> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == right)
                .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), me, Some(source)))
                .map(|c| (c.definition.cost.cmc(), c.id))
                .collect();
            if legal.is_empty() {
                continue;
            }
            // Best first for the controller's own take, cheapest first for
            // what the others get; the decider's default is the head.
            legal.sort_by_key(|&(mv, id)| (if seat == me { u32::MAX - mv } else { mv }, id.0));
            let legal: Vec<Target> = legal.into_iter().map(|(_, id)| Target::Permanent(id)).collect();
            let Some(Target::Permanent(id)) = self.ask_seat_target_logged(
                &mut cursor,
                me,
                format!("Choose a permanent for P{seat} from P{right}"),
                source,
                legal,
                effect,
            ) else {
                return;
            };
            moves.push((id, seat));
        }
        self.clear_answer_log();
        for (id, seat) in moves {
            self.change_control(id, seat);
        }
    }

    /// `Effect::EachPlayerMayExileHandDraw` (Wheel of Potential). Each living
    /// seat from the controller on (CR 101.4) chooses; then each one that
    /// said yes exiles its hand and draws. Past the threshold the controller
    /// may play their own exiled cards until the end of their next turn.
    pub(crate) fn each_player_may_exile_hand_draw(
        &mut self,
        amount: &Value,
        play_if_at_least: u32,
        ctx: &crate::game::effects::EffectContext,
        events: &mut Vec<crate::game::types::GameEvent>,
        effect: &Effect,
    ) -> Result<(), crate::game::GameError> {
        use crate::effect::{Selector, ZoneDest};
        let n = self.evaluate_value(amount, ctx).max(0) as usize;
        let seats = self.players.len();
        let order: Vec<usize> = (0..seats)
            .map(|i| (ctx.controller + i) % seats)
            .filter(|&s| !self.players[s].eliminated)
            .collect();
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut wheel = Vec::new();
        for seat in order {
            let yes = if self.seat_prompts(seat) {
                let Some(b) = self.ask_seat_bool(
                    &mut cursor,
                    seat,
                    format!("Exile your hand and draw {n}?"),
                    source,
                    effect,
                    crate::decision::OptionalKind::MayBody,
                ) else {
                    return Ok(());
                };
                b
            } else {
                self.players[seat].hand.len() < n
            };
            if yes {
                wheel.push(seat);
            }
        }
        self.clear_answer_log();
        for seat in wheel {
            let hand = Selector::CardsInZone {
                who: PlayerRef::Seat(seat),
                zone: crate::card::Zone::Hand,
                filter: crate::card::SelectionRequirement::Any,
            };
            let exile = Effect::Move { what: hand, to: ZoneDest::Exile };
            let body = if seat == ctx.controller && n >= play_if_at_least as usize {
                Effect::Seq(vec![
                    exile,
                    Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ])
            } else {
                exile
            };
            self.run_effect(&body, ctx, events)?;
            self.draw_n(seat, n, events);
        }
        Ok(())
    }
}

impl GameState {
    /// `SelectionRequirement::ChosenPermanentOfSource`: `cid` is the
    /// permanent `source` remembers (Havengul Mystery).
    pub(crate) fn is_chosen_permanent_of(&self, cid: crate::card::CardId, source: Option<crate::card::CardId>) -> bool {
        source
            .and_then(|s| self.battlefield_find(s))
            .is_some_and(|c| c.chosen_permanent == Some(cid))
    }
}

impl GameState {
    /// CR 702.82 — Devour `per` as `cid` enters: its controller may sacrifice
    /// any number of other creatures; it enters with `per` +1/+1 counters for
    /// each (`per == 0`: Devour X, X the number devoured — that many squared).
    /// A prompting seat picks; a headless one feeds it its tokens of power 1
    /// or less.
    pub(crate) fn apply_devour_etb(&mut self, cid: CardId, per: u32, events: &mut Vec<crate::game::types::GameEvent>) {
        use crate::card::CounterType;
        let Some(me) = self.battlefield_find(cid) else { return };
        let ctrl = me.controller;
        let candidates: Vec<(CardId, String)> = self
            .battlefield
            .iter()
            .filter(|c| c.id != cid && c.controller == ctrl && self.computed_is_creature(c) && self.can_be_sacrificed(c.id))
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        if candidates.is_empty() {
            return;
        }
        let chosen: Vec<CardId> = if self.seat_prompts(ctrl) {
            match self.decider.decide(&Decision::ChooseCards {
                source: cid,
                prompt: "Devour: sacrifice any number of creatures".to_string(),
                candidates: candidates.clone(),
                min: 0,
                max: candidates.len() as u32,
                eligible: None,
                value: crate::decision::PickValue::Cost,
            }) {
                DecisionAnswer::Cards(c) => c.into_iter().filter(|id| candidates.iter().any(|(k, _)| k == id)).collect(),
                _ => Vec::new(),
            }
        } else {
            candidates
                .iter()
                .map(|(id, _)| *id)
                .filter(|id| {
                    self.battlefield_find(*id).is_some_and(|c| c.is_token)
                        && self.computed_permanent(*id).is_some_and(|c| c.power <= 1)
                })
                .collect()
        };
        let mut eaten = Vec::new();
        for id in chosen {
            if let Some(c) = self.battlefield_find(id).cloned() {
                self.sacrifice_one(id, ctrl, events);
                eaten.push(c);
            }
        }
        let devoured = eaten.len() as u32;
        if !eaten.is_empty()
            && let Some(me) = self.battlefield_find_mut(cid)
        {
            me.devoured = eaten;
        }
        let n = if per == 0 { devoured * devoured } else { devoured * per };
        if n == 0 {
            return;
        }
        let n = self.place_counters(cid, CounterType::PlusOnePlusOne, n);
        if n > 0 {
            events.push(crate::game::types::GameEvent::CounterAdded {
                card_id: cid,
                counter_type: CounterType::PlusOnePlusOne,
                count: n,
                placer: self.resolution_causer,
            });
        }
    }
}

impl GameState {
    /// CR 701.15 — `c` would regenerate instead of being destroyed: a shield,
    /// or Mossbridge Troll's "if this creature would be destroyed, regenerate
    /// it" — unless it can't be regenerated this turn.
    pub(crate) fn has_regeneration(c: &CardInstance) -> bool {
        !c.cant_regenerate_this_turn
            && (c.regeneration_shields > 0
                || c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, crate::effect::StaticEffect::RegeneratesInsteadOfDestroyed)))
    }
}

impl GameState {
    /// `Effect::OpponentsBidDamageForFreeCast` (Itazura, Lingering Wick). Every
    /// ask comes first on one cursor — the bids in APNAP order, the tie, the
    /// winner's cast pick, the controller's hand pick — so a suspend re-runs
    /// them from the log; nothing moves until the last is answered. A bid is a
    /// life payment capped at the best spell's mana value (a bot keeps its
    /// buffer); the controller breaks a tie toward the lowest life.
    pub(crate) fn opponents_bid_damage_for_free_cast(
        &mut self,
        what: &crate::effect::Selector,
        ctx: &crate::game::effects::EffectContext,
        effect: &Effect,
        events: &mut Vec<crate::game::types::GameEvent>,
    ) -> Result<(), crate::game::GameError> {
        use crate::decision::{AmountKind, PickValue};
        use crate::effect::{Selector, ZoneDest};
        use crate::game::types::Target;
        let me = ctx.controller;
        let src = ctx.source.unwrap_or(CardId(0));
        let pool: Vec<(CardId, String, u32, bool)> = self
            .resolve_selector(what, ctx)
            .into_iter()
            .filter_map(|e| e.as_card_id())
            .filter_map(|id| self.exile.iter().find(|c| c.id == id))
            .map(|c| (c.id, c.definition.name.to_string(), c.definition.cost.cmc(), c.definition.is_land()))
            .collect();
        if pool.is_empty() {
            return Ok(());
        }
        let best = pool.iter().filter(|c| !c.3).map(|c| c.2).max().unwrap_or(0);
        let opps = {
            let o = self.opponents_of(me);
            self.apnap_sort(o)
        };
        if opps.is_empty() {
            return Ok(());
        }
        let mut cursor = 0;
        let mut bids: Vec<(usize, u32)> = Vec::with_capacity(opps.len());
        for &o in &opps {
            let n = if best == 0 {
                0
            } else {
                let prompt = "Secretly choose a number: the highest takes that much damage and a free spell".to_string();
                let Some(n) = self.ask_seat_amount(&mut cursor, o, prompt, src, best, AmountKind::Life, effect) else {
                    return Ok(());
                };
                n
            };
            bids.push((o, n));
        }
        let high = bids.iter().map(|b| b.1).max().unwrap_or(0);
        let mut tied: Vec<usize> = bids.iter().filter(|b| b.1 == high).map(|b| b.0).collect();
        tied.sort_by_key(|&s| self.effective_life(s));
        let winner = if tied.len() > 1 {
            let names = tied.iter().map(|&s| format!("Player {}", s + 1)).collect();
            let Some(i) = self.ask_seat_option(&mut cursor, me, "Choose an opponent with the highest number".into(), src, names, effect)
            else {
                return Ok(());
            };
            tied[i.min(tied.len() - 1)]
        } else {
            tied[0]
        };
        let spells: Vec<(CardId, String)> = pool.iter().filter(|c| !c.3).map(|c| (c.0, c.1.clone())).collect();
        let cast = if spells.is_empty() {
            None
        } else {
            let auto = pool.iter().filter(|c| !c.3).max_by_key(|c| c.2).map(|c| c.0).into_iter().collect();
            let Some(pick) = self.ask_seat_cards_logged(
                &mut cursor,
                winner,
                "Cast a spell from among the exiled cards without paying its mana cost?".into(),
                src,
                spells,
                0,
                1,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            pick.first().copied()
        };
        let rest: Vec<(CardId, String)> =
            pool.iter().filter(|c| Some(c.0) != cast).map(|c| (c.0, c.1.clone())).collect();
        let keep = if rest.is_empty() {
            None
        } else {
            let auto = pool.iter().filter(|c| Some(c.0) != cast).max_by_key(|c| (!c.3, c.2)).map(|c| c.0);
            let Some(pick) = self.ask_seat_cards_logged(
                &mut cursor,
                me,
                "Put a card from among them into your hand".into(),
                src,
                rest.clone(),
                1,
                1,
                PickValue::Gain,
                effect,
                auto.into_iter().collect(),
            ) else {
                return Ok(());
            };
            pick.first().copied().or(rest.first().map(|r| r.0))
        };
        self.clear_answer_log();
        if high > 0 {
            let damage = Effect::DealDamage {
                to: Selector::Player(PlayerRef::Seat(winner)),
                amount: Value::Const(high as i32),
            };
            self.run_effect(&damage, ctx, events)?;
        }
        // The hand pick moves before the cast so a parked target pick can't
        // strand it; the spell is on the stack either way.
        if let Some(id) = keep {
            let c = crate::game::effects::EffectContext { targets: vec![Target::Permanent(id)], ..ctx.clone() };
            self.run_effect(&Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }, &c, events)?;
        }
        if let Some(id) = cast.filter(|id| self.exile.iter().any(|c| c.id == *id))
            && self.players.get(winner).is_some_and(|p| p.is_alive())
        {
            let c = crate::game::effects::EffectContext {
                controller: winner,
                targets: vec![Target::Permanent(id)],
                ..ctx.clone()
            };
            let free = Effect::CastWithoutPayingImmediate {
                what: Selector::Target(0),
                source_zone: crate::card::Zone::Exile,
                exile_after: false,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: false,
            };
            self.run_effect(&free, &c, events)?;
        }
        Ok(())
    }
}

impl GameState {
    /// Miss Highwater — `lost` is leaving the game holding contract counters:
    /// each beneficiary still in the game gets a trigger creating a token copy
    /// of every artifact and creature `lost` controlled. Read before the
    /// departing seat's objects go (CR 800.4a); their last-known objects ride
    /// `captured_token_lki` for the copies (CR 608.2h).
    pub(crate) fn queue_contract_copies(&mut self, lost: usize) {
        use crate::card::CardType;
        if self.players.get(lost).is_none_or(|p| p.contracts.is_empty()) {
            return;
        }
        let mut contracts = std::mem::take(&mut self.players[lost].contracts);
        contracts.dedup_by_key(|c| c.0);
        let lki: Vec<CardInstance> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == lost)
            .filter(|c| self.computed_has_card_type(c, CardType::Artifact) || self.computed_has_card_type(c, CardType::Creature))
            .cloned()
            .collect();
        if lki.is_empty() {
            return;
        }
        let ids: Vec<CardId> = lki.iter().map(|c| c.id).collect();
        for c in lki {
            if !self.captured_token_lki.iter().any(|k| k.id == c.id) {
                self.captured_token_lki.push(c);
            }
        }
        for (beneficiary, source) in contracts {
            if beneficiary == lost || !self.players.get(beneficiary).is_some_and(|p| p.is_alive()) {
                continue;
            }
            let body = Effect::Seq(
                ids.iter()
                    .map(|&id| {
                        crate::effect::shortcut::token_copy_of(
                            PlayerRef::You,
                            Value::ONE,
                            crate::effect::Selector::ExactObjects(vec![id]),
                        )
                    })
                    .collect(),
            );
            self.push_stack(TriggerPush::new(source, beneficiary, body).build());
        }
    }
}
