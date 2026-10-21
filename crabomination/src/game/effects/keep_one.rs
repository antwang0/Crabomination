//! "Each player chooses a [filter] permanent they control, then sacrifices
//! (or: destroy) the rest" — Single Combat, Divine Reckoning, Deadly Vanity —
//! and its one-per-card-type sibling (Cataclysmic Gearhulk, Cataclysm, Tragic
//! Arrogance) — and Unstable Glyphbridge's "for each player, [you] choose a
//! creature with power 2 or less that player controls; destroy the rest".

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
    #[allow(clippy::too_many_arguments)]
    pub(super) fn each_player_keeps_one(
        &mut self,
        who: &Selector,
        filter: &SelectionRequirement,
        destroy: bool,
        bounce: bool,
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
            if bounce {
                let owner = self.battlefield_find(id).map_or(p, |c| c.owner);
                self.move_card_to(id, &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::Seat(owner)), ctx, events);
            } else if destroy {
                self.destroy_permanent(id, false, events);
            } else {
                self.sacrifice_one(id, p, events);
            }
        }
        Ok(())
    }

    /// `Effect::EachPlayerKeepsCreaturesTotalPower` (Destined Confrontation):
    /// each living seat, APNAP (CR 101.4), keeps any creatures whose total
    /// power is at most `max_power`; the rest are sacrificed at once. A
    /// headless seat keeps greedily by mana value; a pick over the cap is cut
    /// back in the order given.
    pub(super) fn each_player_keeps_creatures_total_power(
        &mut self,
        max_power: i32,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let source = ctx.source.unwrap_or(CardId(0));
        let seats = self.apnap_sort(self.living_seats().collect());
        let mut cursor = 0;
        let mut doomed: Vec<(usize, CardId)> = Vec::new();
        for p in seats {
            let mut mine: Vec<(u32, i32, CardId, String)> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && self.computed_is_creature(c))
                .map(|c| (c.definition.cost.cmc(), self.effective_power(c), c.id, c.definition.name.to_string()))
                .collect();
            if mine.is_empty() {
                continue;
            }
            mine.sort_by_key(|(mv, _, id, _)| (std::cmp::Reverse(*mv), *id));
            let fit = |order: &mut dyn Iterator<Item = CardId>| -> Vec<CardId> {
                let mut total = 0;
                let mut kept = Vec::new();
                for id in order {
                    let Some(&(_, pw, _, _)) = mine.iter().find(|m| m.2 == id) else { continue };
                    if total + pw <= max_power && !kept.contains(&id) {
                        total += pw;
                        kept.push(id);
                    }
                }
                kept
            };
            let auto = fit(&mut mine.iter().map(|m| m.2));
            let n = mine.len() as u32;
            let Some(picked) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                format!("Choose creatures with total power {max_power} or less to keep"),
                source,
                mine.iter().map(|m| (m.2, m.3.clone())).collect(),
                0,
                n,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            let kept = fit(&mut picked.into_iter());
            doomed.extend(mine.iter().map(|m| m.2).filter(|id| !kept.contains(id)).map(|id| (p, id)));
        }
        self.clear_answer_log();
        for (p, id) in doomed {
            self.sacrifice_one(id, p, events);
        }
        Ok(())
    }

    /// `Effect::GainChosenBasicLandType` — the controller picks a basic land
    /// type (the mana it wants, as Terraformer picks), and each land `what`
    /// names gains it in addition to its other types (CR 305.7 doesn't apply:
    /// nothing is replaced).
    pub(super) fn gain_chosen_basic_land_type(
        &mut self,
        what: &Selector,
        duration: crate::effect::Duration,
        ctx: &EffectContext,
    ) -> Result<(), GameError> {
        use crate::game::layers::{AffectedPermanents, ContinuousEffect, Layer, Modification};
        let source = ctx.source.unwrap_or(CardId(0));
        let lands: Vec<CardId> = self.resolve_selector(what, ctx).into_iter().filter_map(|e| e.as_permanent_id()).collect();
        if lands.is_empty() {
            return Ok(());
        }
        // A type the controller's lands lack first: it is new mana and new
        // domain; with all five already there, any.
        let lacked: Vec<crate::mana::Color> = crate::mana::Color::ALL
            .into_iter()
            .filter(|&c| {
                let lt = super::basic_land_type_for(c);
                !self.battlefield.iter().any(|p| {
                    p.controller == ctx.controller
                        && self.computed_permanent(p.id).is_some_and(|cp| cp.subtypes().land_types.contains(&lt))
                })
            })
            .collect();
        let options: &[crate::mana::Color] = if lacked.is_empty() { &crate::mana::Color::ALL } else { &lacked };
        let land_type = super::basic_land_type_for(self.chosen_mana_color(ctx.controller, options, Some(source)));
        let duration_kind = self.effect_duration_for(duration, ctx.controller);
        for cid in lands {
            let ts = self.next_timestamp();
            self.add_continuous_effect(ContinuousEffect {
                timestamp: ts,
                source,
                affected: AffectedPermanents::just(cid),
                layer: Layer::L4Type,
                sublayer: None,
                duration: duration_kind.clone(),
                modification: Modification::AddLandType(land_type),
            });
        }
        Ok(())
    }

    /// `Effect::EachPlayerChoosesOpponentPermanentExile` (The Legend of
    /// Yangchen): starting with the controller, each living seat chooses up
    /// to one `filter` permanent an opponent of the controller controls; the
    /// chosen ones are exiled together. A headless seat names the highest
    /// mana value among those it doesn't control itself.
    pub(super) fn each_player_chooses_opponent_permanent_exile(
        &mut self,
        filter: &SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut chosen: Vec<CardId> = Vec::new();
        for p in self.seats_in_turn_order_from(me) {
            let pool: Vec<&crate::card::CardInstance> = self
                .battlefield
                .iter()
                .filter(|c| c.controller != me && self.evaluate_requirement_on_card(filter, c, me))
                .collect();
            if pool.is_empty() {
                break;
            }
            let auto = pool
                .iter()
                .filter(|c| c.controller != p)
                .max_by_key(|c| c.definition.cost.cmc())
                .map(|c| c.id);
            let candidates: Vec<(CardId, String)> =
                pool.iter().map(|c| (c.id, c.definition.name.to_string())).collect();
            let Some(picked) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                "Choose up to one permanent to exile".into(),
                source,
                candidates,
                0,
                1,
                PickValue::Cost,
                effect,
                auto.into_iter().collect(),
            ) else {
                return Ok(());
            };
            if let Some(id) = picked.first().copied()
                && !chosen.contains(&id)
            {
                chosen.push(id);
            }
        }
        self.clear_answer_log();
        for id in chosen {
            if self.battlefield_find(id).is_some() {
                self.move_card_to(id, &crate::effect::ZoneDest::Exile, ctx, events);
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

    /// `Effect::ChooseOnePerPlayerDestroyRest` — the controller picks, for
    /// each living player in APNAP order (CR 101.4), one `keep` permanent that
    /// player controls; then every `of` permanent not picked is destroyed at
    /// once (CR 608.2c). Headless: its own priciest, an opponent's cheapest.
    pub(super) fn choose_one_per_player_destroy_rest(
        &mut self,
        keep: &SelectionRequirement,
        of: &SelectionRequirement,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let source = ctx.source.unwrap_or(CardId(0));
        let seats = self.apnap_sort(self.living_seats().collect());
        let mut cursor = 0;
        let mut kept: Vec<CardId> = Vec::new();
        for p in seats {
            let mine: Vec<(u32, CardId, String)> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p)
                .filter(|c| self.evaluate_requirement_static(keep, &crate::game::Target::Permanent(c.id), me, ctx.source))
                .map(|c| (c.definition.cost.cmc(), c.id, c.definition.name.to_string()))
                .collect();
            let auto = if p == me {
                mine.iter().max_by_key(|(mv, id, _)| (*mv, std::cmp::Reverse(*id)))
            } else {
                mine.iter().min_by_key(|(mv, id, _)| (*mv, *id))
            }
            .map(|(_, id, _)| *id);
            let Some(auto) = auto else { continue };
            let Some(picked) = self.ask_seat_cards_logged(
                &mut cursor,
                me,
                format!("Choose the creature seat {p} keeps"),
                source,
                mine.iter().map(|(_, id, name)| (*id, name.clone())).collect(),
                1,
                1,
                PickValue::Gain,
                effect,
                vec![auto],
            ) else {
                return Ok(());
            };
            kept.push(picked.first().copied().unwrap_or(auto));
        }
        self.clear_answer_log();
        let doomed: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| !kept.contains(&c.id))
            .filter(|c| self.evaluate_requirement_static(of, &crate::game::Target::Permanent(c.id), me, ctx.source))
            .map(|c| c.id)
            .collect();
        for id in doomed {
            self.destroy_permanent(id, false, events);
        }
        Ok(())
    }
}
