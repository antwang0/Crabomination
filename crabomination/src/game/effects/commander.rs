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
        let Some(player) = self.players.get(seat) else { return };
        let candidates: Vec<(CardId, String)> = player
            .command
            .iter()
            .filter(|c| player.commanders.contains(&c.id))
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        let chosen = match candidates.len() {
            0 => return,
            1 => candidates[0].0,
            _ => {
                let answer = self.decider.decide(&Decision::ChooseCards {
                    source: source.unwrap_or(CardId(0)),
                    prompt: "Put which commander into your hand?".into(),
                    candidates: candidates.clone(),
                    min: 1,
                    max: 1,
                    eligible: None,
                    value: crate::decision::PickValue::Gain,
                });
                match answer {
                    DecisionAnswer::Cards(ids)
                        if ids.first().is_some_and(|id| {
                            candidates.iter().any(|(c, _)| c == id)
                        }) =>
                    {
                        ids[0]
                    }
                    _ => candidates[0].0,
                }
            }
        };
        let Some(pos) = self.players[seat].command.iter().position(|c| c.id == chosen) else {
            return;
        };
        let card = self.players[seat].command.remove(pos);
        self.players[seat].hand.push(card);
        self.offboard_keyword_grants = true;
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

impl GameState {
    /// `CardDefinition::spell_kind` plus the one field only the game can fill:
    /// CR 903.3 — whether `card` is `seat`'s own commander.
    pub fn spell_kind_for(&self, seat: usize, card: &CardInstance) -> SpellKind {
        let mut kind = card.definition.spell_kind();
        kind.commander = self
            .players
            .get(seat)
            .is_some_and(|p| !p.commanders.is_empty() && p.commanders.contains(&card.id));
        kind
    }

    /// Apply the cast-time half of the commander mana riders and report
    /// whether Path of Ancestry's trigger still has to go on the stack.
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
        spent: &[SpendRestriction],
        kind: &SpellKind,
        card: &mut CardInstance,
    ) -> bool {
        if spent.is_empty() {
            return false;
        }
        if kind.commander && spent.contains(&SpendRestriction::CommanderCastCounters) {
            let n = self.commander_cast_count.get(&card.id).copied().unwrap_or(0);
            if n > 0 {
                card.pending_etb_counters
                    .push((crate::card::CounterType::PlusOnePlusOne, n));
            }
        }
        kind.creature && spent.contains(&SpendRestriction::CommanderTypeScry)
    }

    /// CR 603.3 — Path of Ancestry's trigger, put on the stack once the spell
    /// it funded is already there, so it resolves first.
    ///
    /// The shared-type check reads the commander's types *now* (2023-07-28
    /// ruling: they're checked immediately after the cast, not at activation),
    /// layer-aware while it's on the battlefield.
    pub(crate) fn push_commander_mana_scry(&mut self, seat: usize, kind: &SpellKind) {
        if !self.commander_shares_creature_type(seat, kind) {
            return;
        }
        let source = self
            .restricted_mana_source(seat, SpendRestriction::CommanderTypeScry)
            .unwrap_or(CardId(0));
        self.stack.push(
            TriggerPush::new(
                source,
                seat,
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            )
            .build(),
        );
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
