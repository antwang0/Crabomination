//! "Each player chooses a [filter] permanent they control, then sacrifices
//! (or: destroy) the rest" — Single Combat, Divine Reckoning, Deadly Vanity —
//! and its one-per-card-type sibling (Cataclysmic Gearhulk, Cataclysm, Tragic
//! Arrogance).

use crate::card::{CardId, SelectionRequirement};
use crate::decision::PickValue;
use crate::effect::{Effect, Selector};
use crate::game::effects::{EffectContext, EntityRef};
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::EachPlayerKeepsOneSacrificeRest` — every resolved player
    /// picks their keeper in APNAP order (CR 101.4; a bot keeps its highest
    /// mana value), then the rest go at once (CR 608.2c).
    pub(super) fn each_player_keeps_one(
        &mut self,
        who: &Selector,
        filter: &SelectionRequirement,
        destroy: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let seats: Vec<usize> = self
            .resolve_selector(who, ctx)
            .into_iter()
            .filter_map(|e| match e {
                EntityRef::Player(p) => Some(p),
                _ => None,
            })
            .collect();
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut doomed: Vec<(usize, CardId)> = Vec::new();
        for p in seats {
            let mine: Vec<&crate::card::CardInstance> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && self.evaluate_requirement_on_card(filter, c, p))
                .collect();
            if mine.is_empty() {
                continue;
            }
            let auto = mine.iter().max_by_key(|c| c.definition.cost.cmc()).map(|c| c.id);
            let candidates: Vec<(CardId, String)> =
                mine.iter().map(|c| (c.id, c.definition.name.to_string())).collect();
            let ids: Vec<CardId> = mine.iter().map(|c| c.id).collect();
            let Some(picked) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                "Choose the permanent you keep".into(),
                source,
                candidates,
                1,
                1,
                PickValue::Gain,
                effect,
                auto.into_iter().collect(),
            ) else {
                return Ok(());
            };
            // The choice is mandatory: an empty answer keeps the bot's pick.
            let keep = picked.first().copied().or(auto);
            doomed.extend(ids.into_iter().filter(|id| Some(*id) != keep).map(|id| (p, id)));
        }
        self.clear_answer_log();
        for (p, id) in doomed {
            if destroy {
                self.destroy_permanent(id, false, events);
            } else {
                self.sacrifice_one(id, p, events);
            }
        }
        Ok(())
    }

    /// `Effect::SacrificeAllButOnePerType{,YouChoose}` — for each resolved
    /// player in turn, one permanent of each listed card type is chosen to
    /// keep: by that player, or (Tragic Arrogance, `you_choose`) by the
    /// resolving controller. A permanent of two types may be chosen as both
    /// (the 2015/2016 rulings); lands are never chosen nor sacrificed unless
    /// `include_land` (Cataclysm). Every other permanent of theirs goes at
    /// once (CR 608.2c). Headless: a chooser keeps its own highest mana value
    /// of each type, a fresh permanent where it can; it spares an opponent's
    /// lowest, reusing one already spared where it can.
    pub(super) fn each_player_keeps_one_per_type(
        &mut self,
        who: &Selector,
        include_land: bool,
        you_choose: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        use crate::card::CardType;
        let types: &[CardType] = if include_land {
            &[CardType::Artifact, CardType::Creature, CardType::Enchantment, CardType::Land]
        } else {
            &[CardType::Artifact, CardType::Creature, CardType::Enchantment, CardType::Planeswalker]
        };
        let seats: Vec<usize> = self
            .resolve_selector(who, ctx)
            .into_iter()
            .filter_map(|e| match e {
                EntityRef::Player(p) => Some(p),
                _ => None,
            })
            .collect();
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut doomed: Vec<(usize, CardId)> = Vec::new();
        for p in seats {
            let chooser = if you_choose { ctx.controller } else { p };
            let friendly = self.same_team(chooser, p);
            let in_scope = |g: &GameState, c: &crate::card::CardInstance| {
                c.controller == p && (include_land || !g.computed_has_card_type(c, CardType::Land))
            };
            let mut keep: Vec<CardId> = Vec::new();
            for ty in types {
                let of_type: Vec<(CardId, String, u32)> = self
                    .battlefield
                    .iter()
                    .filter(|c| in_scope(self, c) && self.computed_has_card_type(c, ty.clone()))
                    .map(|c| (c.id, c.definition.name.to_string(), c.definition.cost.cmc()))
                    .collect();
                if of_type.is_empty() {
                    continue;
                }
                let auto = if friendly {
                    of_type.iter().filter(|(id, ..)| !keep.contains(id)).max_by_key(|(.., mv)| *mv)
                } else {
                    of_type.iter().filter(|(id, ..)| keep.contains(id)).max_by_key(|(.., mv)| *mv)
                }
                .or_else(|| {
                    if friendly {
                        of_type.iter().max_by_key(|(.., mv)| *mv)
                    } else {
                        of_type.iter().min_by_key(|(.., mv)| *mv)
                    }
                })
                .map(|(id, ..)| *id);
                let Some(picked) = self.ask_seat_cards_logged(
                    &mut cursor,
                    chooser,
                    format!("Choose the {} to keep", format!("{ty:?}").to_lowercase()),
                    source,
                    of_type.iter().map(|(id, name, _)| (*id, name.clone())).collect(),
                    1,
                    1,
                    if friendly { PickValue::Gain } else { PickValue::Cost },
                    effect,
                    auto.into_iter().collect(),
                ) else {
                    return Ok(());
                };
                // The choice is mandatory: an empty answer keeps the default.
                if let Some(id) = picked.first().copied().or(auto)
                    && !keep.contains(&id)
                {
                    keep.push(id);
                }
            }
            doomed.extend(
                self.battlefield
                    .iter()
                    .filter(|c| in_scope(self, c) && !keep.contains(&c.id))
                    .map(|c| (p, c.id)),
            );
        }
        self.clear_answer_log();
        for (p, id) in doomed {
            self.sacrifice_one(id, p, events);
        }
        Ok(())
    }
}
