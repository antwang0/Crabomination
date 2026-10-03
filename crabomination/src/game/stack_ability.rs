//! Abilities on the stack as objects of their own (CR 113.7, 115.1).
//!
//! Every `StackItem::Trigger` pushed through [`GameState::push_stack`] gets an
//! `ability_id` from a separate id space ([`ABILITY_ID_BASE`]), so "target
//! activated or triggered ability" names one ability — not "the topmost one
//! from that source". Two abilities of one permanent on the stack are two
//! targets (Strionic Resonator, Stifle).

use super::GameState;
use super::types::{ABILITY_ID_BASE, StackItem, Target, is_stack_ability_id};
use crate::card::{CardId, SelectionRequirement};

impl GameState {
    /// Push onto the stack, stamping a trigger/ability with a fresh id. A copy
    /// is a new object (CR 707.10), so a cloned item is restamped too.
    pub fn push_stack(&mut self, mut item: StackItem) {
        self.stamp_ability_id(&mut item);
        self.stack.push(item);
    }

    /// One past every ability id on the stack or named by a stack target, so
    /// an id is never reused while anything can still name it. Stateless: a
    /// counter would cost `GameState` eight bytes it has no room for
    /// (`cow::tests::game_state_stays_small`), and never moves a card id.
    fn stamp_ability_id(&self, item: &mut StackItem) {
        let StackItem::Trigger { ability_id, .. } = item else { return };
        let named = |t: &Target| match t {
            Target::Permanent(c) if is_stack_ability_id(*c) => c.0,
            _ => 0,
        };
        let top = self
            .stack
            .iter()
            .map(|si| match si {
                StackItem::Trigger { ability_id, target, additional_targets, .. } => target
                    .iter()
                    .chain(additional_targets.iter())
                    .map(named)
                    .fold(*ability_id, u32::max),
                StackItem::Spell { target, additional_targets, .. } => {
                    target.iter().chain(additional_targets.iter()).map(named).max().unwrap_or(0)
                }
            })
            .max()
            .unwrap_or(0);
        *ability_id = top.max(ABILITY_ID_BASE - 1) + 1;
    }

    /// The stack index of the ability `cid` names: the ability with that id,
    /// or — for a source card id (unstamped test fixtures, the bot's legacy
    /// source targets) — the topmost ability from that source, `controller`'s
    /// first when given.
    pub(crate) fn stack_ability_pos(&self, cid: CardId, controller: Option<usize>) -> Option<usize> {
        if is_stack_ability_id(cid) {
            return self.stack.iter().position(
                |si| matches!(si, StackItem::Trigger { ability_id, .. } if *ability_id == cid.0),
            );
        }
        let find = |mine: bool| {
            self.stack.iter().enumerate().rev().find_map(|(i, si)| match si {
                StackItem::Trigger { source, controller: c, .. }
                    if *source == cid && (!mine || Some(*c) == controller) =>
                {
                    Some(i)
                }
                _ => None,
            })
        };
        if controller.is_some() { find(true).or_else(|| find(false)) } else { find(false) }
    }

    /// The id of `source`'s topmost ability on the stack — the target a
    /// caller holding only the source passes (CR 115.1).
    pub fn top_ability_of(&self, source: CardId) -> Option<CardId> {
        self.stack.iter().rev().find_map(|si| match si {
            StackItem::Trigger { source: s, ability_id, .. } if *s == source && *ability_id != 0 => {
                Some(CardId(*ability_id))
            }
            _ => None,
        })
    }

    /// The source of the ability `cid` names, or `cid` itself for a card id.
    pub(crate) fn stack_ability_source(&self, cid: CardId) -> Option<CardId> {
        if !is_stack_ability_id(cid) {
            return Some(cid);
        }
        self.stack_ability_pos(cid, None).and_then(|i| match &self.stack[i] {
            StackItem::Trigger { source, .. } => Some(*source),
            StackItem::Spell { .. } => None,
        })
    }

    /// Does the stack hold an *unstamped* ability from `source` (one that can
    /// only be named by its source)? Every engine push stamps, so this is only
    /// true of hand-built fixtures.
    pub(crate) fn has_unstamped_ability_from(&self, source: CardId, triggered_only: bool) -> bool {
        self.stack.iter().any(|si| matches!(si,
            StackItem::Trigger { source: s, ability_id: 0, activated, .. }
                if *s == source && (!triggered_only || !*activated)))
    }

    /// `req` asked of the stack ability `aid` (CR 115.1): stack-object atoms
    /// read the ability, controller atoms its controller, every other atom
    /// its source ("an ability of an artifact source" — Tawnos).
    pub(crate) fn evaluate_requirement_on_ability(
        &self,
        req: &SelectionRequirement,
        aid: CardId,
        controller: usize,
        source: Option<CardId>,
    ) -> bool {
        use SelectionRequirement as R;
        let Some(StackItem::Trigger {
            source: from,
            controller: owner,
            activated,
            target,
            additional_targets,
            ..
        }) = self.stack_ability_pos(aid, None).map(|i| &self.stack[i])
        else {
            return false;
        };
        match req {
            R::And(a, b) => {
                self.evaluate_requirement_on_ability(a, aid, controller, source)
                    && self.evaluate_requirement_on_ability(b, aid, controller, source)
            }
            R::Or(a, b) => {
                self.evaluate_requirement_on_ability(a, aid, controller, source)
                    || self.evaluate_requirement_on_ability(b, aid, controller, source)
            }
            R::Not(a) => !self.evaluate_requirement_on_ability(a, aid, controller, source),
            R::Any | R::HasAbilityOnStack => true,
            R::HasTriggeredAbilityOnStack => !*activated,
            // CR 115.5 — not a spell; never the resolving object itself.
            R::IsSpellOnStack | R::Player | R::OpponentPlayer => false,
            R::ControlledByYou => *owner == controller,
            // Emissary of Grudges: the ability's controller is the chosen one.
            R::ControlledByChosenPlayerOfSource => source
                .and_then(|s| self.battlefield_find(s))
                .and_then(|c| c.chosen_player)
                .is_some_and(|p| p == *owner),
            // "targets you or a permanent you control" (CR 115.7) — the
            // ability's targets, read as `SpellTargetsControllerOrControlled`
            // reads a spell's.
            R::SpellTargetsControllerOrControlled => target.iter().chain(additional_targets.iter()).any(|t| match t {
                Target::Player(p) => *p == controller,
                Target::Permanent(id) => self.battlefield_find(*id).is_some_and(|o| o.controller == controller),
            }),
            R::ControlledByOpponent => !self.same_team(*owner, controller),
            R::AbilityTargetsMatching(inner) => target
                .iter()
                .chain(additional_targets.iter())
                .any(|t| self.evaluate_requirement_static(inner, t, controller, source)),
            other => self.evaluate_requirement_static(other, &Target::Permanent(*from), controller, source),
        }
    }

    /// Every stack ability `req` accepts, topmost first — the candidates a
    /// "target activated or triggered ability" slot offers.
    pub(crate) fn stack_ability_targets(
        &self,
        req: &SelectionRequirement,
        controller: usize,
        source: Option<CardId>,
    ) -> Vec<Target> {
        self.stack
            .iter()
            .rev()
            .filter_map(|si| match si {
                StackItem::Trigger { ability_id, .. } if *ability_id != 0 => Some(CardId(*ability_id)),
                _ => None,
            })
            .filter(|aid| self.evaluate_requirement_on_ability(req, *aid, controller, source))
            .map(Target::Permanent)
            .collect()
    }

    /// CR 115.7d — `chooser` chooses new targets for the stack ability `aid`,
    /// each declared slot against the ability's own slot filter.
    pub(crate) fn choose_new_targets_for_ability(&mut self, aid: CardId, chooser: usize) {
        let Some(pos) = self.stack_ability_pos(aid, None) else { return };
        let StackItem::Trigger { effect, source, target, additional_targets, .. } = &self.stack[pos] else { return };
        if target.is_none() {
            return;
        }
        let (effect, source, orig, extra) = ((**effect).clone(), *source, target.clone(), additional_targets.clone());
        let name = self.find_card_anywhere(source).map_or("ability", |c| c.definition.name);
        let first = self.retarget_slot(&effect, name, chooser, 0, &orig, &[]);
        let mut taken: Vec<Target> = first.iter().cloned().collect();
        for (i, t) in extra.iter().enumerate() {
            let pick = self
                .retarget_slot(&effect, name, chooser, (i + 1) as u8, &Some(t.clone()), &taken)
                .unwrap_or_else(|| t.clone());
            taken.push(pick);
        }
        let new_extra = taken.split_off(first.iter().len());
        if let StackItem::Trigger { target, additional_targets, .. } = &mut self.stack[pos] {
            *target = first;
            *additional_targets = new_extra;
        }
    }
}
