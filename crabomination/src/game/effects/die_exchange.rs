//! Vedalken Squirrel-Whacker's dice. `Effect::RollDiceSetBasePt` — "as this
//! creature enters, roll a d`sides` twice; its base power becomes the first
//! result and its base toughness the second" — and
//! `StaticEffect::MayExchangeD6WithSelfBasePt`, the replacement on each d6
//! its controller rolls: "you may exchange one result with this creature's
//! base power or base toughness" (CR 706.6). The base P/T is one layer-7b
//! set the creature owns, rewritten in place by an exchange. Every seat takes
//! the headless exchange: a 1 or 2 swapped for a base stat of 5 or more.

use crate::card::{CardId, StaticEffect};
use crate::game::effects::EffectContext;
use crate::game::layers::{AffectedPermanents, ContinuousEffect, EffectDuration, Layer, Modification, PtSublayer};
use crate::game::{GameError, GameEvent, GameState};

/// A roll at most this low is worth trading away.
const LOW_ROLL: u8 = 2;
/// A base stat at least this high is worth trading for a low roll.
const HIGH_STAT: i32 = 5;

impl GameState {
    pub(super) fn roll_dice_set_base_pt(
        &mut self,
        sides: u8,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(source) = ctx.source else { return Ok(()) };
        let sides = sides.max(2);
        let a = self.roll_one_die(ctx.controller, sides);
        let b = self.roll_one_die(ctx.controller, sides);
        events.push(GameEvent::DiceRolled { player: ctx.controller, count: 2, high: a.max(b) });
        for face in [a, b] {
            events.push(GameEvent::DieResult { player: ctx.controller, result: face });
            if face == sides {
                events.push(GameEvent::RolledNaturalMax { player: ctx.controller });
            }
        }
        let timestamp = self.next_timestamp();
        self.add_continuous_effect(ContinuousEffect {
            timestamp,
            source,
            affected: AffectedPermanents::just(source),
            layer: Layer::L7PowerTough,
            sublayer: Some(PtSublayer::SetValue),
            duration: EffectDuration::Indefinite,
            modification: Modification::SetPowerToughness(a as i32, b as i32),
        });
        Ok(())
    }

    /// CR 706.6 — a d6 `player` just rolled, after any exchange with the base
    /// P/T of a Squirrel-Whacker they control.
    pub(crate) fn exchange_d6_with_base_pt(&mut self, player: usize, face: u8) -> u8 {
        if face > LOW_ROLL {
            return face;
        }
        let whackers: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == player
                    && c.definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(sa.effect, StaticEffect::MayExchangeD6WithSelfBasePt))
            })
            .map(|c| c.id)
            .collect();
        for id in whackers {
            let set = self.continuous_effects.iter_mut().rev().find(|e| {
                e.source == id
                    && e.sublayer == Some(PtSublayer::SetValue)
                    && matches!(e.modification, Modification::SetPowerToughness(..))
            });
            let Some(e) = set else { continue };
            let Modification::SetPowerToughness(p, t) = &mut e.modification else { continue };
            let stat = if *p >= *t { p } else { t };
            if *stat >= HIGH_STAT {
                let traded = *stat;
                *stat = face as i32;
                return traded.clamp(1, 6) as u8;
            }
        }
        face
    }

    /// CR 706.6 — Monitor Monitor's paid reroll of a low die `player` rolled,
    /// once each turn per Monitor (headless: a face in the bottom third).
    pub(crate) fn paid_die_reroll(&mut self, player: usize, sides: u8, face: u8) -> u8 {
        if u32::from(face) * 3 > u32::from(sides) {
            return face;
        }
        let slot = crate::game::DONE_THIS_TURN_SLOT;
        let monitor = self.battlefield.iter().find_map(|c| {
            (c.controller == player && !self.triggered_once_per_turn_used.contains(&(c.id, slot)))
                .then(|| {
                    c.definition.static_abilities.iter().find_map(|sa| match sa.effect {
                        StaticEffect::MayPayToRerollDieOncePerTurn { generic } => Some((c.id, generic)),
                        _ => None,
                    })
                })
                .flatten()
        });
        let Some((id, generic)) = monitor else { return face };
        if !self.pay_generic(player, generic) {
            return face;
        }
        self.triggered_once_per_turn_used.insert((id, slot));
        let rolled = rand::RngExt::random_range(&mut self.rng.draw(), 1..=sides);
        match self.decider.decide(&crate::decision::Decision::DieRoll { player, sides, rolled }) {
            crate::decision::DecisionAnswer::DieRoll(f) => f.clamp(1, sides),
            _ => rolled,
        }
    }
}
