//! CR 603.3b — the triggers a declaration of attackers fires go on the stack
//! in APNAP order: the active player's first (resolving last), then each
//! other player's in turn order. The defender-side walk (`combat.rs`) pushes
//! "whenever a creature attacks you" triggers per attacker, as declared, and
//! the dispatcher pushes the attacking side's afterward, so a defender's
//! trigger sat UNDER the attacker's and resolved last (Lulu stunned an
//! attacker only after Hero of Bladehold's Soldiers had already arrived).

use super::GameState;
use super::types::StackItem;

impl GameState {
    /// Stable-sort the triggers pushed since `mark` by APNAP rank, keeping
    /// each controller's own order. A no-op unless two controllers pushed.
    pub(crate) fn apnap_order_triggers_since(&mut self, mark: usize) {
        if mark >= self.stack.len() {
            return;
        }
        let controller = |si: &StackItem| match si {
            StackItem::Trigger { controller, .. } => Some(*controller),
            StackItem::Spell { .. } => None,
        };
        let first = self.stack[mark..].iter().find_map(controller);
        let mixed = self.stack[mark..].iter().filter_map(controller).any(|c| Some(c) != first);
        let all_triggers = self.stack[mark..].iter().all(|si| controller(si).is_some());
        if !mixed || !all_triggers {
            return;
        }
        let rank: Vec<usize> = {
            let order = self.apnap_sort((0..self.players.len()).collect());
            self.stack[mark..]
                .iter()
                .map(|si| controller(si).and_then(|c| order.iter().position(|&s| s == c)).unwrap_or(usize::MAX))
                .collect()
        };
        if rank.windows(2).all(|w| w[0] <= w[1]) {
            return;
        }
        let mut items: Vec<(usize, StackItem)> = rank.into_iter().zip(self.stack.drain(mark..)).collect();
        items.sort_by_key(|(r, _)| *r);
        for (_, si) in items {
            self.stack.push(si);
        }
    }
}
