//! Reselecting what an attacking creature attacks (Misleading Signpost).
//!
//! Its own module rather than another arm body in `effects/mod.rs`.

use super::EffectContext;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::AttackTarget;

impl GameState {
    /// "You may reselect which player or permanent [what] is attacking."
    /// The new choice must be something the creature could attack — an
    /// opponent of its controller, or a planeswalker one controls (CR
    /// 508.1b) — but restrictions and costs are ignored and nothing
    /// triggers (the rulings). The chooser is `ctx.controller`; a headless
    /// seat takes the first option, so the list opens with the targets it
    /// would rather see hit (its ranked hostile opponent first, never
    /// itself), then "keep", then its own player and planeswalkers.
    pub(crate) fn reselect_attack_target(&mut self, what: &Selector, ctx: &EffectContext, effect: &Effect) {
        let Some(attacker) = self.resolve_selector(what, ctx).into_iter().find_map(|e| e.as_permanent_id())
        else {
            return;
        };
        let Some(slot) = self.attacking.iter().position(|a| a.attacker == attacker) else { return };
        let Some(owner) = self.battlefield_find(attacker).map(|c| c.controller) else { return };
        let chooser = ctx.controller;
        let current = self.attacking[slot].target;

        let mut seats = self.opponents_of(owner);
        let hostile = self.default_hostile_opponent(chooser);
        // Ranked: the chooser's hostile opponent, then seat order; the chooser last.
        seats.sort_by_key(|&s| (s == chooser, Some(s) != hostile, s));
        let mut theirs: Vec<(AttackTarget, String)> = Vec::new();
        let mut mine: Vec<(AttackTarget, String)> = Vec::new();
        for s in seats {
            let bucket = if s == chooser { &mut mine } else { &mut theirs };
            bucket.push((AttackTarget::Player(s), format!("Player {}", s + 1)));
            for pw in self.battlefield.iter().filter(|c| c.controller == s && c.definition.is_planeswalker()) {
                bucket.push((AttackTarget::Planeswalker(pw.id), pw.definition.name.to_string()));
            }
        }
        theirs.retain(|(t, _)| *t != current);
        mine.retain(|(t, _)| *t != current);
        if theirs.is_empty() && mine.is_empty() {
            return;
        }
        let keep = theirs.len();
        let mut options: Vec<(Option<AttackTarget>, String)> =
            theirs.into_iter().map(|(t, n)| (Some(t), n)).collect();
        options.push((None, "Keep its attack".to_string()));
        options.extend(mine.into_iter().map(|(t, n)| (Some(t), n)));
        debug_assert!(options[keep].0.is_none());

        let mut cursor = 0;
        let labels = options.iter().map(|(_, n)| n.clone()).collect();
        let Some(idx) = self.ask_seat_option(
            &mut cursor,
            chooser,
            "Reselect what it attacks?".to_string(),
            ctx.source.unwrap_or(attacker),
            labels,
            effect,
        ) else {
            return;
        };
        self.clear_answer_log();
        if let Some((Some(target), _)) = options.into_iter().nth(idx)
            && let Some(a) = self.attacking.get_mut(slot)
        {
            a.target = target;
        }
    }
}
