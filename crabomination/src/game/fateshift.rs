//! Subjective Reality's effects: manifest-and-attach, Jeskai Infiltrator's
//! re-manifest, Sower of Discord's chosen pair, Aminatou's permanent
//! rotation, Aminatou's Augury's one-per-type casts, Yennett and Djinn of
//! Wishes' top-card casts.

use crate::card::{CardId, CardType, Zone};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState};

impl GameState {
    pub(crate) fn manifest_top_attach_source(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        // "It becomes an Aura with enchant creature" — even when there is
        // nothing to manifest, in which case CR 704.5m sweeps it. Swapped in
        // through `temporary_copies` so leaving the battlefield restores the
        // printed plain enchantment.
        if let Some(src) = ctx.source {
            self.become_aura(src);
        }
        let Some(top) = self.players[p].library.first().map(|c| c.id) else { return Ok(()) };
        self.manifest_card(top, p, ctx, events);
        if self.battlefield_find(top).is_none() {
            return Ok(());
        }
        self.run_effect(&Effect::Attach { what: Selector::This, to: Selector::ExactObjects(vec![top]) }, ctx, events)
    }

    fn become_aura(&mut self, src: CardId) {
        use crate::card::EnchantmentSubtype;
        let Some(orig) = self.battlefield_find(src).map(|c| c.definition.arc()) else { return };
        if orig.is_aura() {
            return;
        }
        let mut aura = (*orig).clone();
        aura.subtypes.enchantment_subtypes.push(EnchantmentSubtype::Aura);
        self.temporary_copies.push(crate::game::TempCopy {
            until_turn_of: None,
            card: src,
            original: Some(orig.clone()),
            original_name: orig.name.to_string(),
            duration: crate::effect::Duration::Permanent,
            source: None,
            shapeshifter: false,
        });
        if let Some(c) = self.battlefield_find_mut(src) {
            c.set_copiable_definition(std::sync::Arc::new(aura));
        }
    }

    pub(crate) fn exile_source_and_top_then_manifest(&mut self, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        use rand::seq::SliceRandom;
        let p = ctx.controller;
        let mut pile: Vec<CardId> = Vec::new();
        if let Some(src) = ctx.source.filter(|s| self.battlefield_find(*s).is_some()) {
            self.move_card_to(src, &ZoneDest::Exile, ctx, events);
            pile.push(src);
        }
        if let Some(top) = self.players[p].library.first().map(|c| c.id) {
            self.move_card_to(top, &ZoneDest::Exile, ctx, events);
            pile.push(top);
        }
        pile.shuffle(&mut self.rng.draw());
        for cid in pile {
            let Some(c) = self.exile.iter_mut().find(|c| c.id == cid) else { continue };
            c.turn_face_down();
            let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: false };
            self.move_card_to(cid, &dest, ctx, events);
        }
    }

    pub(crate) fn choose_two_players_for_source(&mut self, ctx: &EffectContext) {
        let Some(src) = ctx.source else { return };
        let me = ctx.controller;
        let mut opps = self.opponents_of(me);
        opps.sort_by_key(|&o| (self.players[o].life, o));
        let pair = match opps.as_slice() {
            [a, b, ..] => (*a, *b),
            [a] => (*a, me),
            [] => return,
        };
        self.chosen_player_pairs.retain(|(s, ..)| *s != src);
        self.chosen_player_pairs.push((src, pair.0, pair.1));
    }

    pub(crate) fn other_chosen_player_loses_life(
        &mut self,
        ctx: &EffectContext,
        n: i32,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let (Some(src), Some(hit)) = (ctx.source, self.resolve_player(&PlayerRef::TriggerEventPlayer, ctx)) else {
            return Ok(());
        };
        let Some(&(_, a, b)) = self.chosen_player_pairs.iter().find(|(s, ..)| *s == src) else { return Ok(()) };
        let other = if hit == a {
            b
        } else if hit == b {
            a
        } else {
            return Ok(());
        };
        self.run_effect(
            &Effect::LoseLife { who: Selector::Player(PlayerRef::Seat(other)), amount: Value::Const(n) },
            ctx,
            events,
        )
    }

    pub(crate) fn rotate_nonland_permanents(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use crate::decision::{Decision, DecisionAnswer};
        let n = self.players.len();
        if n < 2 {
            return Ok(());
        }
        let answer = self.decider.decide(&Decision::ChooseMode {
            source: ctx.source.unwrap_or(CardId(0)),
            num_modes: 2,
            mode_texts: vec!["Left".into(), "Right".into()],
        });
        let step: isize = if matches!(answer, DecisionAnswer::Mode(1)) { -1 } else { 1 };
        // Each seat receives the permanents of the next live seat that way.
        let next = |s: usize| -> Option<usize> {
            (1..n as isize)
                .map(|k| (s as isize + step * k).rem_euclid(n as isize) as usize)
                .find(|&t| self.players[t].is_alive())
        };
        let moves: Vec<(CardId, usize)> = self
            .battlefield
            .iter()
            .filter(|c| Some(c.id) != ctx.source && !c.definition.is_land())
            .filter_map(|c| {
                let giver = c.controller;
                // The receiver is the seat whose "next player" is `giver`.
                (0..n)
                    .find(|&r| r != giver && self.players[r].is_alive() && next(r) == Some(giver))
                    .map(|r| (c.id, r))
            })
            .collect();
        for (id, to) in moves {
            self.run_effect(
                &Effect::GainControl {
                    what: Selector::ExactObjects(vec![id]),
                    to: Some(PlayerRef::Seat(to)),
                    duration: crate::effect::Duration::Permanent,
                },
                ctx,
                events,
            )?;
        }
        Ok(())
    }

    pub(crate) fn grant_free_cast_one_per_card_type(&mut self, ids: &[CardId], ctx: &EffectContext) {
        const TYPES: [CardType; 6] = [
            CardType::Creature,
            CardType::Artifact,
            CardType::Enchantment,
            CardType::Planeswalker,
            CardType::Instant,
            CardType::Sorcery,
        ];
        let mut cards: Vec<(CardId, u32, Vec<CardType>)> = ids
            .iter()
            .filter_map(|id| self.exile.iter().find(|c| c.id == *id))
            .filter(|c| !c.definition.is_land())
            .map(|c| (c.id, c.definition.cost.cmc(), c.definition.card_types.clone()))
            .collect();
        cards.sort_by_key(|(id, mv, _)| (std::cmp::Reverse(*mv), *id));
        let mut picked: Vec<CardId> = Vec::new();
        for t in TYPES {
            if let Some((id, ..)) = cards.iter().find(|(id, _, ts)| ts.contains(&t) && !picked.contains(id)) {
                picked.push(*id);
            }
        }
        let turn = self.turn_number;
        for id in picked {
            if let Some(c) = self.exile.iter_mut().find(|c| c.id == id) {
                c.may_play_until = Some(crate::card::MayPlayPermission {
                    player: ctx.controller,
                    granted_turn: turn,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    exile_after: false,
                    miracle: false,
                    pay_life: false,
                });
                c.granted_alt_cast_cost_eot = Some(crate::mana::ManaCost::new(vec![]));
            }
        }
    }

    pub(crate) fn cast_top_free_if_else_draw(
        &mut self,
        filter: &crate::card::SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let Some(top) = self.players[p].library.first().map(|c| c.id) else { return Ok(()) };
        let castable = self.players[p].library.first().is_some_and(|c| {
            !c.definition.is_land() && self.evaluate_requirement_on_card(filter, c, p)
        });
        if castable {
            self.run_effect(
                &Effect::CastWithoutPayingImmediate {
                    what: Selector::ExactObjects(vec![top]),
                    source_zone: Zone::Library,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
                ctx,
                events,
            )?;
        }
        if self.players[p].library.first().map(|c| c.id) == Some(top) {
            self.run_effect(&Effect::Draw { who: Selector::You, amount: Value::ONE }, ctx, events)?;
        }
        Ok(())
    }

    pub(crate) fn play_top_free_else_exile(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let Some((top, land)) = self.players[p].library.first().map(|c| (c.id, c.definition.is_land())) else {
            return Ok(());
        };
        if land {
            let pl = &self.players[p];
            if self.active_player_idx == p && pl.lands_played_this_turn < 1 + pl.extra_land_plays {
                self.players[p].lands_played_this_turn += 1;
                let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: false };
                self.move_card_to(top, &dest, ctx, events);
            }
        } else {
            self.run_effect(
                &Effect::CastWithoutPayingImmediate {
                    what: Selector::ExactObjects(vec![top]),
                    source_zone: Zone::Library,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
                ctx,
                events,
            )?;
        }
        if self.players[p].library.first().map(|c| c.id) == Some(top) {
            self.move_card_to(top, &ZoneDest::Exile, ctx, events);
        }
        Ok(())
    }
}
