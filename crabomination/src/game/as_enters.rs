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
        if !self.has_as_enters_replacement(card_id) {
            return;
        }
        self.apply_as_enters_effect(card_id);
        self.apply_as_enters_mode_pickers(card_id);
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
