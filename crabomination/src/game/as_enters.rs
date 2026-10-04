//! CR 614.12 — the "as this permanent enters" replacements, and the one
//! funnel every battlefield entry runs them through.
//!
//! CR 614.12a puts the choice these replacements ask for *before* the
//! permanent enters, so each applier has to run inside the battlefield hop
//! rather than off an `EntersBattlefield` trigger. The engine has four such
//! hops — the cast path (`stack.rs`), the universal move
//! (`effects/movement.rs`), the land drop (`actions.rs::play_land`) and the
//! token mint (`mod.rs::mint_token_with_counters`) — and until this funnel
//! each applier was wired to a different subset of them:
//!
//! | applier | cast | move | land drop | token |
//! |---|---|---|---|---|
//! | `as_enters_effect` | ✓ | ✓ | — | — |
//! | `enters_as_choice` | ✓ | — | — | — |
//! | `enter_modes` | ✓ | — | — | — |
//!
//! So a reanimated Corrupted Shapeshifter chose no body, a Cavern of Souls
//! *played* named no creature type, and CR 614.12's own worked example — a
//! token copy of Voice of All choosing its colour as the token is created —
//! had nothing to run at all.

use crate::card::CardId;
use crate::game::GameState;

impl GameState {
    /// CR 614.12 — apply every as-enters replacement that asks a question or
    /// rewrites the entering permanent's characteristics, in one order, from
    /// one place. Called by all four battlefield-entry paths.
    ///
    /// Ordered as the cast path ordered them: the free-form one-shot
    /// (`as_enters_effect`) first, then the two mode pickers, which only
    /// rewrite the permanent's own printed line and so cannot be read by the
    /// other two.
    ///
    /// Not in here: `enters_as_copy` (CR 707.2), which needs the entering
    /// permanent's controller and an event sink, and whose two wirings differ
    /// in *position* rather than presence — the token mint applies it before
    /// this funnel, because a token copy's copiable values are what the mint
    /// establishes; the cast path applies it after. See the backlog for the
    /// two paths that still lack it.
    #[doc(hidden)] // reachable from the out-of-crate test suite, like `actions`/`stack`
    pub fn apply_as_enters_replacements(&mut self, card_id: CardId) {
        if self.has_as_enters_replacement(card_id) {
            self.apply_as_enters_effect(card_id);
            self.apply_as_enters_mode_pickers(card_id);
        }
        self.apply_unleash(card_id);
        self.apply_riot(card_id);
    }

    /// How many instances of `kw` the entering permanent has, printed or
    /// granted by a static already in effect (CR 614.12). The computed read
    /// only runs for a permanent that prints it or while a grant is in scope.
    fn entering_keyword_count(&self, card_id: CardId, kw: &crate::card::Keyword) -> usize {
        use crate::card::KeywordSlice;
        let Some(c) = self.battlefield.find_by_id(card_id) else { return 0 };
        if !c.definition.keywords.has_kw(kw) && !self.keyword_grant_in_scope(|k| k == kw) {
            return 0;
        }
        self.computed_permanent(card_id).map_or(0, |cp| cp.keywords().iter().filter(|k| *k == kw).count())
    }

    /// Ask `decision` of `seat` off the stack, as `drive_suspensions` does:
    /// a prompting seat has nowhere to park it, so its policy answers.
    fn ask_entering(&mut self, seat: usize, decision: &crate::decision::Decision) -> crate::decision::DecisionAnswer {
        if self.seat_prompts(seat) {
            crate::server::bot::decide_pending_policy(
                self,
                seat,
                &crate::server::bot::EvalWeights::default(),
                decision,
                false,
            )
        } else {
            self.decider.decide(decision)
        }
    }

    /// CR 702.98a — unleash, asked before the permanent enters (614.12a). A
    /// yes rides `pending_etb_counters`, which every entry path places with
    /// its other entering counters.
    fn apply_unleash(&mut self, card_id: CardId) {
        let Some(ctrl) = self.battlefield.find_by_id(card_id).map(|c| c.controller) else { return };
        if self.entering_keyword_count(card_id, &crate::card::Keyword::Unleash) == 0 {
            return;
        }
        let decision = crate::decision::Decision::OptionalTrigger {
            source: card_id,
            description: "Unleash — enter with a +1/+1 counter?".into(),
            kind: crate::decision::OptionalKind::FreeUpside,
        };
        if matches!(self.ask_entering(ctrl, &decision), crate::decision::DecisionAnswer::Bool(true))
            && let Some(c) = self.battlefield.find_by_id_mut(card_id)
        {
            c.pending_etb_counters.push((crate::card::CounterType::PlusOnePlusOne, 1));
        }
    }

    /// CR 702.136a — riot: a +1/+1 counter (riding `pending_etb_counters`)
    /// or haste for as long as it stays on the battlefield, once per instance
    /// (702.136b). Mode 0 is haste, mode 1 the counter.
    fn apply_riot(&mut self, card_id: CardId) {
        use crate::card::Keyword;
        let Some(ctrl) = self.battlefield.find_by_id(card_id).map(|c| c.controller) else { return };
        for _ in 0..self.entering_keyword_count(card_id, &Keyword::Riot) {
            let counter = if self.seat_prompts(ctrl) {
                !self.riot_prefers_haste(card_id, ctrl)
            } else {
                let decision = crate::decision::Decision::ChooseMode {
                    source: card_id,
                    num_modes: 2,
                    mode_texts: vec!["Haste".into(), "+1/+1 counter".into()],
                };
                matches!(self.decider.decide(&decision), crate::decision::DecisionAnswer::Mode(1))
            };
            if counter {
                if let Some(c) = self.battlefield.find_by_id_mut(card_id) {
                    c.pending_etb_counters.push((crate::card::CounterType::PlusOnePlusOne, 1));
                }
            } else {
                let ctx = crate::game::effects::EffectContext::for_ability(card_id, ctrl, None);
                let haste = crate::effect::Effect::GrantKeyword {
                    what: crate::effect::Selector::This,
                    keyword: Keyword::Haste,
                    duration: crate::effect::Duration::Permanent,
                };
                let _ = self.resolve_as_enters_driven(&haste, &ctx);
            }
        }
    }

    /// A seat's riot pick: haste only when it can still attack with the
    /// creature this turn and needs the haste to.
    fn riot_prefers_haste(&self, card_id: CardId, ctrl: usize) -> bool {
        use crate::game::types::TurnStep;
        self.active_player_idx == ctrl
            && self.step < TurnStep::DeclareAttackers
            && self.battlefield.find_by_id(card_id).is_some_and(|c| {
                c.summoning_sick
                    && !c.has_keyword(&crate::card::Keyword::Haste)
                    && self.computed_permanent(card_id).is_some_and(|cp| cp.power > 0 && !cp.keywords().contains(&crate::card::Keyword::Haste))
            })
    }

    /// One battlefield lookup answering "does any of the three appliers have
    /// anything to do?". Every entry path calls the funnel and almost no
    /// permanent carries one of these, so the negative is the hot case and it
    /// costs one lookup rather than three.
    fn has_as_enters_replacement(&self, card_id: CardId) -> bool {
        self.battlefield.find_by_id(card_id).is_some_and(|c| {
            c.definition.as_enters_effect.is_some()
                || c.definition.enters_as_choice.is_some()
                || c.definition.enter_modes.is_some()
        })
    }

    /// The two mode pickers, which answer through the decider directly and so
    /// never suspend. Split out so the land drop can run them *after* its
    /// `as_enters_effect` resumes.
    pub(crate) fn apply_as_enters_mode_pickers(&mut self, card_id: CardId) {
        self.apply_enters_as_choice(card_id);
        self.apply_enters_mode_choice(card_id);
    }

    /// CR 614.12a on the land drop: as [`apply_as_enters_replacements`], but
    /// the free-form replacement's ask is left in `suspend_signal` rather than
    /// driven through the decider. Returns `true` when it suspended, in which
    /// case the caller owns parking it — the mode pickers have **not** run and
    /// the entry is unfinished.
    ///
    /// The land drop is the one entry path that needs this. The cast and the
    /// mint run inside a resolution, which has a stack item to park the
    /// continuation on; the land drop is an action, and unlike the other
    /// suspending actions it cannot be replayed — the land is on the
    /// battlefield and the drop is already spent.
    pub(crate) fn apply_as_enters_replacements_suspending(&mut self, card_id: CardId) -> bool {
        if !self.has_as_enters_replacement(card_id) {
            return false;
        }
        self.apply_as_enters_effect_suspending(card_id);
        if self.suspend_signal.is_some() {
            return true;
        }
        self.apply_as_enters_mode_pickers(card_id);
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::decision::DecisionAnswer;

    /// CR 614.12 — an as-enters replacement applied off the stack (a
    /// state-based move while a vote waits on its next voter) resolves on its
    /// own answer channel: the parked vote's ballot is still there after.
    #[test]
    fn cr_614_12_off_stack_as_enters_keeps_a_parked_resolutions_answers() {
        let mut g = crate::game::multi_player_game(3);
        let cavern = g.add_card_to_battlefield(0, crate::catalog::cavern_of_souls());
        g.scratch.resolution_answer_log = vec![DecisionAnswer::Amount(0)];
        g.apply_as_enters_replacements(cavern);
        assert_eq!(g.scratch.resolution_answer_log, vec![DecisionAnswer::Amount(0)]);
    }
}
