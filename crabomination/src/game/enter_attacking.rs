//! CR 508.4 — a permanent put onto the battlefield attacking (Mobilize,
//! Myriad, Ninjutsu, "create a token that's tapped and attacking") was never
//! declared as an attacker: it is an attacking creature, but "whenever ~
//! attacks" abilities don't trigger for it. Every such site funnels through
//! here so none emits `GameEvent::AttackerDeclared` — the event the Attacks
//! triggers read. A General Kreat with a Goblin army otherwise minted a
//! token per token per attack, without bound.
//!
//! Also the CR 506.3 read of an attack's defender ("creatures attacking you").

use super::GameState;
use super::effects::EffectContext;
use super::types::{Attack, AttackTarget, GameError, Target};
use crate::card::{CardId, CardType};
use crate::effect::{Effect, Selector};

impl GameState {
    /// `Effect::JoinCombatAttackingChosen` — CR 508.4: each of `what` not yet
    /// attacking joins combat tapped, attacking the defending player (one
    /// being attacked this combat) or planeswalker of theirs its controller
    /// chooses. Headless, and the default, is the source's defender, else the
    /// first defending player. The asks come first and are replayed by a
    /// re-run, which names the movers it resolved.
    pub(crate) fn join_combat_attacking_chosen(
        &mut self,
        what: &Selector,
        cleanup: crate::effect::AttackingTokenCleanup,
        ctx: &EffectContext,
        effect: &Effect,
    ) -> Result<(), GameError> {
        if self.attacking.is_empty() {
            return Ok(());
        }
        let movers: Vec<(CardId, usize)> = self
            .resolve_selector(what, ctx)
            .into_iter()
            .filter_map(|e| e.as_card_id())
            .filter(|id| !self.attacking.iter().any(|a| a.attacker == *id))
            .filter_map(|id| self.battlefield_find(id).map(|c| (id, c.controller)))
            .collect();
        let mut seats: Vec<usize> =
            self.attacking.iter().filter_map(|a| self.defender_for(a.target)).filter(|&p| self.players[p].is_alive()).collect();
        seats.sort_unstable();
        seats.dedup();
        let source_target = ctx
            .source
            .and_then(|src| self.attacking.iter().find(|a| a.attacker == src))
            .map(|a| a.target)
            .filter(|t| self.defender_for(*t).is_some());
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut picks: Vec<(CardId, AttackTarget)> = Vec::new();
        for &(id, controller) in &movers {
            let mut legal: Vec<Target> = Vec::new();
            for &p in seats.iter().filter(|&&p| !self.same_team(p, controller)) {
                legal.push(Target::Player(p));
                legal.extend(
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == p && self.computed_has_card_type(c, crate::card::CardType::Planeswalker))
                        .map(|c| Target::Permanent(c.id)),
                );
            }
            let default = match source_target {
                Some(AttackTarget::Player(p)) => Some(Target::Player(p)),
                Some(AttackTarget::Planeswalker(pw)) => Some(Target::Permanent(pw)),
                _ => None,
            }
            .filter(|t| legal.contains(t))
            .or_else(|| legal.first().cloned());
            let Some(default) = default else { continue };
            legal.retain(|t| *t != default);
            legal.insert(0, default.clone());
            let ask = legal.len() > 1
                && (self.seat_prompts(controller) || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto));
            let pick = if ask {
                let ids: Vec<CardId> = movers.iter().map(|m| m.0).collect();
                let Some(t) = self.ask_seat_target_logged(
                    &mut cursor,
                    controller,
                    "Choose what it's attacking".into(),
                    source,
                    legal,
                    effect,
                ) else {
                    super::effects::rewrap_parked(&mut self.suspend_signal, |_| Effect::JoinCombatAttackingChosen {
                        what: Selector::ExactObjects(ids),
                        cleanup,
                    });
                    return Ok(());
                };
                t
            } else {
                default
            };
            let target = match pick {
                Target::Player(p) => AttackTarget::Player(p),
                Target::Permanent(pw) => AttackTarget::Planeswalker(pw),
            };
            picks.push((id, target));
        }
        self.clear_answer_log();
        for (id, target) in picks {
            if let Some(c) = self.battlefield.find_by_id_mut(id) {
                c.tapped = true;
            }
            if self.put_into_combat_attacking(id, target) && cleanup != crate::effect::AttackingTokenCleanup::None {
                self.attacking_token_cleanup.push((id, cleanup));
            }
        }
        Ok(())
    }

    /// Mark `id` (already on the battlefield) as attacking `target`, without
    /// declaring it (CR 508.4). Returns false if it isn't on the battlefield,
    /// or it entered but is never an attacking creature (CR 506.3a-c): it isn't
    /// a creature, an attacking player doesn't control it (an Akroan Horse
    /// copy entering under an opponent's control), or its defender left.
    pub(crate) fn put_into_combat_attacking(&mut self, id: CardId, target: AttackTarget) -> bool {
        let Some(ctrl) = self.battlefield.find_by_id(id).map(|c| c.controller) else { return false };
        if !self.same_team(ctrl, self.active_player_idx)
            || !self.computed_permanent(id).is_some_and(|cp| cp.card_types().contains(&CardType::Creature))
            || match target {
                AttackTarget::Player(p) => !self.players.get(p).is_some_and(|pl| pl.is_alive()),
                AttackTarget::Planeswalker(pw) | AttackTarget::Battle(pw) => self.battlefield.find_by_id(pw).is_none(),
            }
        {
            return false;
        }
        let Some(c) = self.battlefield.find_by_id_mut(id) else { return false };
        c.attacked_this_turn = true;
        self.note_attack_defender(target);
        self.attacking.push(Attack { attacker: id, target });
        true
    }

    /// Whether `id` is attacking the player `seat` itself (CR 506.3) — not
    /// a planeswalker or battle of theirs.
    pub(crate) fn creature_is_attacking_seat(&self, id: CardId, seat: usize) -> bool {
        self.attacking.iter().any(|a| a.attacker == id && a.target == AttackTarget::Player(seat))
    }

    /// `id` attacks the player `source` last chose (its `chosen_player`).
    pub(crate) fn attacking_chosen_player_of(&self, id: CardId, source: Option<CardId>) -> bool {
        source
            .and_then(|s| self.battlefield_find(s))
            .and_then(|c| c.chosen_player)
            .is_some_and(|p| self.creature_is_attacking_seat(id, p))
    }

    /// `id` (a permanent, or a spell by its caster) is controlled by the
    /// player `source` chose.
    pub(crate) fn controlled_by_chosen_player_of(&self, id: CardId, source: Option<CardId>) -> bool {
        let Some(p) = source.and_then(|s| self.battlefield_find(s)).and_then(|c| c.chosen_player) else {
            return false;
        };
        match self.battlefield_find(id) {
            Some(c) => c.controller == p,
            None => self.stack_spell_caster(id) == Some(p),
        }
    }

    /// `id` attacks the player the firing trigger event named.
    pub(crate) fn attacking_trigger_player(&self, id: CardId) -> bool {
        self.trigger_event_player_scratch.is_some_and(|p| self.creature_is_attacking_seat(id, p))
    }

    /// `id` attacks `seat` or a planeswalker `seat` controls.
    pub(crate) fn creature_is_attacking_seat_or_its_planeswalker(&self, id: CardId, seat: usize) -> bool {
        self.attacking.iter().any(|a| {
            a.attacker == id
                && match a.target {
                    AttackTarget::Player(p) => p == seat,
                    AttackTarget::Planeswalker(pw) => self.battlefield_find(pw).is_some_and(|c| c.controller == seat),
                    AttackTarget::Battle(_) => false,
                }
        })
    }

    /// `id` attacks an opponent of `seat`, or a planeswalker one controls.
    pub(crate) fn creature_is_attacking_an_opponent_of(&self, id: CardId, seat: usize) -> bool {
        self.attacking.iter().any(|a| {
            a.attacker == id
                && match a.target {
                    AttackTarget::Player(p) => !self.same_team(p, seat),
                    AttackTarget::Planeswalker(pw) => self
                        .battlefield_find(pw)
                        .is_some_and(|c| !self.same_team(c.controller, seat)),
                    AttackTarget::Battle(_) => false,
                }
        })
    }

    /// Attacking an opponent of `seat` directly (not a planeswalker or
    /// battle of theirs).
    pub(crate) fn creature_is_attacking_opponent_player(&self, id: CardId, seat: usize) -> bool {
        self.attacking
            .iter()
            .any(|a| a.attacker == id && matches!(a.target, AttackTarget::Player(p) if !self.same_team(p, seat)))
    }
}
