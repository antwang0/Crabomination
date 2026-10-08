//! Eldrazi Incursion's multiplayer-wide effects: Selective Obliteration's
//! per-player color, Ulalek's copy-everything, Benthic Anomaly's summed copy.

use crate::card::CardId;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, StackItem};
use crate::game::{GameError, GameState};
use crate::mana::Color;

impl GameState {
    /// `seat`'s five colors, the one that saves most of their permanents (a
    /// mono-colored one survives only its own color) first, WUBRG on ties.
    fn colors_by_survivors(&self, seat: usize) -> Vec<Color> {
        let mut tally = [0u32; 5];
        for c in self.battlefield.iter().filter(|c| c.controller == seat) {
            if let [col] = self.source_colors(c.id)[..] {
                tally[col as usize] += 1;
            }
        }
        let mut order: Vec<usize> = (0..5).collect();
        order.sort_by_key(|&i| (std::cmp::Reverse(tally[i]), i));
        order.into_iter().map(|i| Color::ALL[i]).collect()
    }

    /// Selective Obliteration: each player chooses a color in APNAP order
    /// (CR 101.4), then each permanent is exiled unless it's colorless or
    /// exactly its controller's chosen color.
    pub(crate) fn each_player_chooses_color_exile_others(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) {
        let source = ctx.source.unwrap_or(CardId(0));
        let mut chosen: Vec<Option<Color>> = vec![None; self.players.len()];
        let mut cursor = 0;
        for seat in self.apnap_sort(self.living_seats().collect()) {
            let order = self.colors_by_survivors(seat);
            let options = order.iter().map(|c| format!("{c:?}")).collect();
            let Some(i) = self.ask_seat_option(&mut cursor, seat, "Choose a color".into(), source, options, effect)
            else {
                return;
            };
            chosen[seat] = order.get(i).copied();
        }
        self.clear_answer_log();
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
            self.push_stack(item);
        }
        Ok(())
    }

    /// Benthic Anomaly: one creature per opponent (their greatest power), a
    /// token copy of the one with the greatest mana value carrying the
    /// chosen creatures' summed power and toughness, colorless and Eldrazi.
    /// Benthic Anomaly: the controller chooses one creature per opponent,
    /// then which of them to copy (headless: each opponent's greatest power,
    /// the copy of the greatest mana value). All asks come first.
    pub(crate) fn copy_one_per_opponent_with_total_stats(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        use crate::game::types::Target;
        let me = ctx.controller;
        let asks = self.seat_prompts(me) || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto);
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut chosen: Vec<CardId> = Vec::new();
        for opp in self.opponents_of(me) {
            let mut theirs: Vec<&crate::card::CardInstance> =
                self.battlefield.iter().filter(|c| c.controller == opp && self.computed_is_creature(c)).collect();
            // The headless pick first (greatest power, the earliest on ties).
            theirs.sort_by_key(|c| (std::cmp::Reverse(self.effective_power(c)), c.id));
            let legal: Vec<Target> = theirs.iter().map(|c| Target::Permanent(c.id)).collect();
            let Some(first) = legal.first().cloned() else { continue };
            let pick = if asks && legal.len() > 1 {
                match self.ask_seat_target_logged(&mut cursor, me, format!("Choose a creature seat {opp} controls"), source, legal, effect) {
                    None => return Ok(()),
                    Some(t) => t,
                }
            } else {
                first
            };
            if let Target::Permanent(id) = pick {
                chosen.push(id);
            }
        }
        let mut by_mv = chosen.clone();
        by_mv.sort_by_key(|&id| {
            (std::cmp::Reverse(self.battlefield_find(id).map(|c| c.definition.cost.cmc()).unwrap_or(0)), id)
        });
        let Some(&first) = by_mv.first() else { return Ok(()) };
        let model = if asks && by_mv.len() > 1 {
            let legal = by_mv.iter().map(|&id| Target::Permanent(id)).collect();
            match self.ask_seat_target_logged(&mut cursor, me, "Choose the creature to copy".into(), source, legal, effect) {
                None => return Ok(()),
                Some(Target::Permanent(id)) => id,
                Some(_) => first,
            }
        } else {
            first
        };
        self.clear_answer_log();
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
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            },
            ctx,
            events,
        )
    }
}
