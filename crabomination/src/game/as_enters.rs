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
    pub(crate) fn apply_as_enters_replacements(&mut self, card_id: CardId) {
        self.apply_as_enters_effect(card_id);
        self.apply_enters_as_choice(card_id);
        self.apply_enters_mode_choice(card_id);
    }
}
