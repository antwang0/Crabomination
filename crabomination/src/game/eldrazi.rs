//! Eldrazi Incursion's multiplayer-wide effects: Selective Obliteration's
//! per-player color, Ulalek's copy-everything, Benthic Anomaly's summed copy.

use crate::card::CardId;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, StackItem};
use crate::game::{GameError, GameState};
use crate::mana::Color;

impl GameState {
    /// The color most common among `seat`'s permanents (WUBRG on ties), or
    /// `None` when they control nothing colored.
    fn most_common_color_among_permanents(&self, seat: usize) -> Option<Color> {
        let mut tally = [0u32; 5];
        for c in self.battlefield.iter().filter(|c| c.controller == seat) {
            for col in self.source_colors(c.id) {
                tally[col as usize] += 1;
            }
        }
        let best = (0..5).max_by_key(|&i| (tally[i], std::cmp::Reverse(i)))?;
        (tally[best] > 0).then(|| Color::ALL[best])
    }

    /// Selective Obliteration: exile each permanent unless it's colorless or
    /// exactly its controller's chosen color.
    pub(crate) fn each_player_chooses_color_exile_others(&mut self, ctx: &EffectContext, events: &mut Vec<GameEvent>) {
        let chosen: Vec<Option<Color>> =
            (0..self.players.len()).map(|s| self.most_common_color_among_permanents(s)).collect();
        let doomed: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| {
                let colors = self.source_colors(c.id);
                let kept = colors.is_empty() || (colors.len() == 1 && Some(colors[0]) == chosen[c.controller]);
                !kept
            })
            .map(|c| c.id)
            .collect();
        for id in doomed {
            self.move_card_to(id, &ZoneDest::Exile, ctx, events);
        }
    }

    /// Ulalek: copy each spell `ctx.controller` controls, then each of their
    /// activated and triggered abilities on the stack (CR 707.10).
    pub(crate) fn copy_all_spells_and_abilities(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let spells: Vec<CardId> = self
            .stack
            .iter()
            .filter_map(|si| match si {
                StackItem::Spell { card, caster, .. } if *caster == me => Some(card.id),
                _ => None,
            })
            .collect();
        let abilities: Vec<StackItem> = self
            .stack
            .iter()
            .filter(|si| matches!(si, StackItem::Trigger { controller, .. } if *controller == me))
            .cloned()
            .collect();
        if !spells.is_empty() {
            self.run_effect(
                &Effect::CopySpell { what: Selector::ExactObjects(spells), count: crate::effect::Value::ONE },
                ctx,
                events,
            )?;
        }
        for item in abilities {
            self.stack.push(item);
        }
        Ok(())
    }

    /// Benthic Anomaly: one creature per opponent (their greatest power), a
    /// token copy of the one with the greatest mana value carrying the
    /// chosen creatures' summed power and toughness, colorless and Eldrazi.
    pub(crate) fn copy_one_per_opponent_with_total_stats(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let mut chosen: Vec<CardId> = Vec::new();
        for opp in self.opponents_of(me) {
            let best = self
                .battlefield
                .iter()
                .filter(|c| c.controller == opp && self.computed_permanent(c.id).is_some_and(|cp| cp.card_types().contains(&crate::card::CardType::Creature)))
                .max_by_key(|c| (self.effective_power(c), std::cmp::Reverse(c.id)))
                .map(|c| c.id);
            chosen.extend(best);
        }
        let Some(&model) = chosen.iter().max_by_key(|&&id| {
            let mv = self.battlefield_find(id).map(|c| c.definition.cost.cmc()).unwrap_or(0);
            (mv, std::cmp::Reverse(id))
        }) else {
            return Ok(());
        };
        let (mut p, mut t) = (0i32, 0i32);
        for &id in &chosen {
            if let Some(c) = self.battlefield_find(id) {
                p = p.saturating_add(self.effective_power(c));
                t = t.saturating_add(self.effective_toughness(c));
            }
        }
        self.run_effect(
            &Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: crate::effect::Value::ONE,
                source: Selector::ExactObjects(vec![model]),
                extra_creature_types: vec![crate::card::CreatureType::Eldrazi],
                extra_card_types: vec![],
                override_pt: Some((p, t)),
                override_colors: Some(vec![]),
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
            ctx,
            events,
        )
    }
}
