//! CR 613 layer 6 — "[filter] have '[static ability]'" (a Background's
//! "Commander creatures you own have …"). A `GrantStaticAbility` source writes
//! the static into each matching permanent's definition (`bake_grant`), so
//! every reader of `definition.static_abilities` sees it as that permanent's
//! own, "you" being its controller; it is no copiable value and ends with the
//! object (CR 707.2 / 400.7). Synced at each state-based-action check, so a
//! grant reaches a permanent from the first check after it starts matching.

use super::GameState;
use crate::card::CardId;
use crate::effect::{StaticAbility, StaticEffect};
use crate::game::types::Target;

type Grants = Vec<(CardId, StaticAbility)>;

impl GameState {
    /// Bring every permanent's granted statics in line with the live
    /// `GrantStaticAbility` sources. True when a definition changed.
    pub(crate) fn sync_granted_statics(&mut self) -> bool {
        let mut desired: Vec<(CardId, Grants)> = Vec::new();
        for src in self.battlefield.iter() {
            for sa in &src.definition.static_abilities {
                let Some(StaticEffect::GrantStaticAbility { filter, ability }) = self.active_static(&sa.effect, src)
                else {
                    continue;
                };
                for p in self.battlefield.iter() {
                    if !self.evaluate_requirement_static(filter, &Target::Permanent(p.id), src.controller, Some(src.id)) {
                        continue;
                    }
                    let grant = (src.id, (**ability).clone());
                    match desired.iter_mut().find(|(id, _)| *id == p.id) {
                        Some((_, v)) => v.push(grant),
                        None => desired.push((p.id, vec![grant])),
                    }
                }
            }
        }
        let touched: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| !c.granted_statics.is_empty() || desired.iter().any(|(id, _)| *id == c.id))
            .map(|c| c.id)
            .collect();
        let mut changed = false;
        let mut live = false;
        for id in touched {
            let want = desired.iter().find(|(d, _)| *d == id).map(|(_, v)| v.clone()).unwrap_or_default();
            let Some(c) = self.battlefield.find_by_id_mut(id) else { continue };
            // A copy effect or a face change replaced the definition since
            // the last bake: what the list says is there no longer is.
            let present =
                c.granted_statics.iter().all(|(_, sa)| c.definition.static_abilities.contains(sa));
            if present && c.granted_statics == want {
                live |= !want.is_empty();
                continue;
            }
            let current = if present { c.granted_statics.clone() } else { Vec::new() };
            if !current.is_empty() || !want.is_empty() {
                let def = c.bake_grant();
                for (_, sa) in &current {
                    if let Some(i) = def.static_abilities.iter().rposition(|x| x == sa) {
                        def.static_abilities.remove(i);
                    }
                }
                def.static_abilities.extend(want.iter().map(|(_, sa)| sa.clone()));
                changed = true;
            }
            live |= !want.is_empty();
            c.granted_statics = want;
        }
        if self.static_grants_live != live {
            self.static_grants_live = live;
        }
        changed
    }
}
