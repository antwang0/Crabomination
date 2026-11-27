//! CR 702.132a — Assist: before the caster pays a spell's total cost, another
//! player they choose may pay any amount of its generic mana. Made where no
//! cast can suspend (like a gift, CR 702.174a): a scripted decider answers
//! both asks, every other seat takes the headless policy below.

use super::GameState;
use super::actions::PaymentSnapshot;
use crate::card::{CardId, CardInstance, Keyword, KeywordSlice};
use crate::decision::{AmountKind, Decision, DecisionAnswer, DeciderKind};
use crate::effect::{EventKind, Predicate, StaticEffect};
use crate::mana::ManaCost;

/// An assister's payment, held until the caster's own payment settles.
pub(crate) struct AssistPaid {
    pub seat: usize,
    pub snapshot: PaymentSnapshot,
    pub events: Vec<crate::game::GameEvent>,
}

impl GameState {
    /// CR 702.132a — does `card`, as `p` casts it now, have assist? Printed,
    /// or granted by a `GrantAssistToSpells` static `p` controls.
    pub(crate) fn spell_has_assist(&self, p: usize, card: &CardInstance) -> bool {
        card.definition.keywords.has_kw(&Keyword::Assist)
            || self.battlefield.iter().any(|c| {
                c.controller == p
                    && c.definition.static_abilities.iter().any(|sa| {
                        match self.active_static_where(&sa.effect, c, |e| matches!(e, StaticEffect::GrantAssistToSpells { .. })) {
                            Some(StaticEffect::GrantAssistToSpells { filter }) => {
                                crate::game::layers::requirement_matches_card(filter, card, p)
                            }
                            _ => false,
                        }
                    })
            })
    }

    /// Something `p` controls pays the assister for helping (Yume's "you and
    /// that player each draw"): a `SpellCast` trigger gated on the spell
    /// having been assisted.
    fn assist_rewards_assister(&self, p: usize) -> bool {
        let gated = |f: &Predicate| match f {
            Predicate::SpellWasAssisted { .. } => true,
            Predicate::All(v) => v.iter().any(|x| matches!(x, Predicate::SpellWasAssisted { .. })),
            _ => false,
        };
        self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition
                    .triggered_abilities
                    .iter()
                    .any(|t| t.event.kind == EventKind::SpellCast && t.event.filter.as_ref().is_some_and(gated))
        })
    }

    /// The headless assister policy: how much of `generic` seat `q` pays for
    /// `p`'s spell. A teammate pays all it can; an opponent pays {1} only for
    /// a reward and only while keeping a source spare.
    fn assist_offer(&self, p: usize, q: usize, generic: u32, rewarded: bool) -> u32 {
        if self.same_team(q, p) {
            (1..=generic).rev().find(|&n| self.could_pay_generic(q, n)).unwrap_or(0)
        } else {
            u32::from(rewarded && self.could_pay_generic(q, 2))
        }
    }

    /// CR 702.132a — offer `p`'s spell's generic mana to an assister, who pays
    /// with their own mana; `cost` loses what they paid. `None` when the spell
    /// has no assist, no generic component, or nobody pays.
    pub(crate) fn take_assist(&mut self, p: usize, card: &CardInstance, cost: &mut ManaCost) -> Option<AssistPaid> {
        let generic = cost.generic_total();
        if generic == 0 || !self.spell_has_assist(p, card) {
            return None;
        }
        let seats: Vec<usize> =
            self.seats_in_turn_order_from(p).into_iter().filter(|&q| q != p && self.players[q].is_alive()).collect();
        if seats.is_empty() {
            return None;
        }
        let (seat, amount) = if matches!(self.decider.kind(), DeciderKind::Scripted { .. }) {
            // Option 0 is "no one"; option i names seats[i - 1].
            let mut options = vec!["No one".to_string()];
            options.extend(seats.iter().map(|q| format!("Player {}", q + 1)));
            let pick = match self.decider.decide(&Decision::ChooseOption {
                source: card.id,
                prompt: format!("Choose a player to assist with {}", card.definition.name),
                options,
            }) {
                DecisionAnswer::Amount(i) => i as usize,
                _ => 0,
            };
            let seat = *seats.get(pick.checked_sub(1)?)?;
            let amount = match self.decider.decide(&Decision::ChooseAmount {
                source: card.id,
                prompt: format!("Pay how much of {}'s generic mana?", card.definition.name),
                max: generic,
                kind: AmountKind::Mana,
            }) {
                DecisionAnswer::Amount(n) => n.min(generic),
                _ => 0,
            };
            (seat, amount)
        } else {
            // Teammates first, then opponents in turn order.
            let rewarded = self.assist_rewards_assister(p);
            let mut order = seats.clone();
            order.sort_by_key(|&q| !self.same_team(q, p));
            order.into_iter().find_map(|q| Some((q, self.assist_offer(p, q, generic, rewarded))).filter(|x| x.1 > 0))?
        };
        if amount == 0 {
            return None;
        }
        let snapshot = self.snapshot_payment_state(seat);
        let receipt = self.try_pay_with_auto_tap(seat, &crate::mana::cost(&[crate::mana::generic(amount)])).ok()?;
        cost.reduce_generic(amount);
        Some(AssistPaid { seat, snapshot, events: receipt.auto_events })
    }

    /// The caster's payment failed: give the assister back what they paid.
    pub(crate) fn undo_assist(&mut self, a: Option<AssistPaid>) {
        if let Some(a) = a {
            self.restore_payment_state(a.seat, a.snapshot);
        }
    }

    /// The cast went through: remember who assisted (`PlayerRef::AssisterOfTriggerSpell`).
    pub(crate) fn record_assist(&mut self, spell: CardId, a: Option<AssistPaid>) {
        if let Some(a) = a {
            self.queue_payment_events(a.events);
            self.turn.assisted_spells.push((spell, a.seat as u8));
        }
    }

    /// The seat that assisted `spell` this turn.
    pub(crate) fn assister_of(&self, spell: CardId) -> Option<usize> {
        self.turn.assisted_spells.iter().find(|(id, _)| *id == spell).map(|&(_, q)| q as usize)
    }
}
