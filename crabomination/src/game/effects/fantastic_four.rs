//! The Fantastic Four (MSC, Invisible Woman) primitives: a forced attack on a
//! player for a duration (Silver Surfer — CR 508.1d), a pump for the other
//! creatures attacking the same player (Namor), and a random linked exile
//! from a graveyard (Power Pack).

use super::{EffectContext, EntityRef};
use crate::card::{CardId, SelectionRequirement};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::game::GameState;
use crate::game::types::{AttackTarget, GameError, GameEvent};
use rand::RngExt;

impl GameState {
    /// CR 508.1d — each creature `attacker` resolves to attacks the player
    /// `defender` resolves to each combat if able, for `duration`: the named
    /// player is stamped on the creature and the requirement is
    /// `MustAttackChosenPlayer`'s.
    pub(super) fn must_attack_player(
        &mut self,
        attacker: &Selector,
        defender: &Selector,
        duration: Duration,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(q) = self.resolve_selector(defender, ctx).into_iter().find_map(|e| match e {
            EntityRef::Player(q) => Some(q),
            _ => None,
        }) else {
            return Ok(());
        };
        let ids: Vec<CardId> =
            self.resolve_selector(attacker, ctx).into_iter().filter_map(|e| e.as_permanent_id()).collect();
        for cid in &ids {
            if let Some(c) = self.battlefield_find_mut(*cid) {
                c.chosen_player = Some(q);
            }
        }
        if ids.is_empty() {
            return Ok(());
        }
        self.run_effect(
            &Effect::GrantKeyword {
                what: Selector::ExactObjects(ids),
                keyword: crate::card::Keyword::MustAttackChosenPlayer,
                duration,
            },
            ctx,
            events,
        )
    }

    /// "Other creatures you control attacking that player get +P/+T until end
    /// of turn", where that player is the one the source attacks (Namor).
    pub(super) fn pump_other_attackers_on_same_player(
        &mut self,
        power: &Value,
        toughness: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(src) = ctx.source else { return Ok(()) };
        let Some(AttackTarget::Player(p)) =
            self.attacking.iter().find(|a| a.attacker == src).map(|a| a.target)
        else {
            return Ok(());
        };
        let ids: Vec<CardId> = self
            .attacking
            .iter()
            .filter(|a| a.attacker != src && a.target == AttackTarget::Player(p))
            .map(|a| a.attacker)
            .filter(|id| self.battlefield_find(*id).is_some_and(|c| c.controller == ctx.controller))
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        self.run_effect(
            &Effect::PumpPT {
                what: Selector::ExactObjects(ids),
                power: power.clone(),
                toughness: toughness.clone(),
                duration: Duration::EndOfTurn,
            },
            ctx,
            events,
        )
    }

    /// Exile a card matching `filter` chosen at random from `who`'s graveyard,
    /// linked to the source (`exiled_with`) — Power Pack.
    pub(super) fn exile_random_from_graveyard_with_source(
        &mut self,
        who: &PlayerRef,
        filter: &SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let Some(p) = self.resolve_player(who, ctx) else { return };
        let picks: Vec<CardId> = self.players[p]
            .graveyard
            .iter()
            .filter(|c| self.evaluate_requirement_on_card(filter, c, ctx.controller))
            .map(|c| c.id)
            .collect();
        if picks.is_empty() {
            return;
        }
        let pick = picks[self.rng.draw().random_range(0..picks.len())];
        self.move_card_to(pick, &crate::effect::ZoneDest::ExileWithSourceStamp, ctx, events);
    }
}
