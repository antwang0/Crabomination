//! Pure-query helpers over `GameState`: `evaluate_value` (numeric expressions),
//! `evaluate_predicate` (boolean conditions), `evaluate_requirement_static`
//! and `evaluate_requirement_on_card` (selection-requirement matching).
//!
//! These are read-only and called from the resolver match arms in
//! `mod.rs` (and from `auto_target_for_effect_avoiding` in `targeting.rs`).

use super::{EffectContext, EntityRef};
use crate::card::{CardId, CardInstance, CardType, SelectionRequirement, Supertype};
use crate::effect::{Predicate, Value};
use crate::mana::ManaSymbol;
use crate::game::{GameState, StackItem, Target};

impl GameState {
    /// CR 700.5 — `player`'s devotion to `colors`: the number of mana
    /// symbols matching any listed color among the mana costs of
    /// permanents they control. A hybrid / Phyrexian / mono-hybrid pip
    /// counts once if it contains any of the colors.
    pub(crate) fn devotion_to(&self, player: usize, colors: &[crate::mana::Color]) -> i32 {
                let matches = |c: &crate::mana::Color| colors.contains(c);
        self.battlefield
            .iter()
            .filter(|card| card.controller == player)
            .flat_map(|card| card.definition.cost.symbols.iter())
            .filter(|sym| match sym {
                ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) | ManaSymbol::MonoHybrid(_, c) => {
                    matches(c)
                }
                ManaSymbol::Hybrid(a, b) => matches(a) || matches(b),
                _ => false,
            })
            .count() as i32
    }

    pub(crate) fn evaluate_value(&self, v: &Value, ctx: &EffectContext) -> i32 {
        match v {
            Value::HalfLibrarySizeRoundedUp(who) => self
                .resolve_player(who, ctx)
                .map(|p| ((self.players[p].library.len() as i32) + 1) / 2)
                .unwrap_or(0),
            Value::GreatestManaValueInExile => self
                .exile
                .iter()
                .map(|c| c.definition.cost.cmc() as i32)
                .max()
                .unwrap_or(0),
            Value::Const(n) => *n,
            Value::CountOf(s) => self.resolve_selector(s, ctx).len() as i32,
            Value::CountMatching { sel, filter } => self
                .resolve_selector(sel, ctx)
                .into_iter()
                .filter(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => {
                        self.evaluate_requirement_static(filter, &Target::Permanent(*cid), ctx.controller, ctx.source)
                    }
                    EntityRef::Player(_) => matches!(filter, SelectionRequirement::Player),
                })
                .count() as i32,
            // CR-spec: "the power of X" returns the total across all
            // entities X resolves to. Single-entity selectors (Target,
            // This, TriggerSource) return that entity's power; fan-out
            // selectors (`EachPermanent(filter)`) return the sum across
            // every match — unblocking "total power among creatures you
            // control" cards (Orysa Tide Choreographer's "total toughness
            // ≥ 10" alt-cost gate, etc.). Same fan-out convention as
            // `CountersOn`.
            Value::BlockersOf(s) => self
                .resolve_selector(s, ctx)
                .iter()
                .filter_map(|e| e.as_permanent_id())
                .map(|id| self.block_map.values().filter(|&&a| a == id).count() as i32)
                .sum(),
            Value::PowerOf(s) => self.resolve_selector(s, ctx).iter()
                .filter_map(|e| {
                    // `as_card_id` (not `as_permanent_id`): a dies-trigger
                    // subject arrives as `EntityRef::Card` once the creature
                    // is in the graveyard (Anax's "if its power was 4+").
                    let cid = e.as_card_id()?;
                    // CR 603.10 — a leaves-battlefield trigger ("when this
                    // dies, deals damage equal to its power") reads the
                    // dying object's last-known power, counters/pumps
                    // included, in preference to the graveyard's printed P/T.
                    if self.resolving_lki_source == Some(cid)
                        && self.battlefield_find(cid).is_none()
                        && let Some(snap) = self.leaves_bf_lki.get(&cid)
                    {
                        return Some(snap.power());
                    }
                    // CR 121 / Lorehold Excavation: read power from the
                    // battlefield first (live `power()` includes
                    // counters), then fall through to graveyard / exile /
                    // hand for cards that have changed zones but whose
                    // power is still being read (e.g. Lorehold
                    // Excavation's "X = its power" rider where the
                    // target is in graveyard at evaluation time, before
                    // it gets exiled). Non-battlefield zones return the
                    // printed power from `CardDefinition.power` since
                    // counters don't apply off the battlefield.
                    if let Some(c) = self.battlefield_find(cid) {
                        return Some(c.power());
                    }
                    if let Some(c) = self.exile.iter().find(|c| c.id == cid) {
                        return Some(c.definition.power);
                    }
                    for p in &self.players {
                        if let Some(c) = p.graveyard.iter().find(|c| c.id == cid) {
                            return Some(c.definition.power);
                        }
                        if let Some(c) = p.hand.iter().find(|c| c.id == cid) {
                            return Some(c.definition.power);
                        }
                    }
                    None
                })
                .sum(),
            Value::ToughnessOf(s) => self.resolve_selector(s, ctx).iter()
                .filter_map(|e| {
                    let cid = e.as_permanent_id()?;
                    if self.resolving_lki_source == Some(cid)
                        && self.battlefield_find(cid).is_none()
                        && let Some(snap) = self.leaves_bf_lki.get(&cid)
                    {
                        return Some(snap.toughness());
                    }
                    if let Some(c) = self.battlefield_find(cid) {
                        return Some(c.toughness());
                    }
                    if let Some(c) = self.exile.iter().find(|c| c.id == cid) {
                        return Some(c.definition.toughness);
                    }
                    for p in &self.players {
                        if let Some(c) = p.graveyard.iter().find(|c| c.id == cid) {
                            return Some(c.definition.toughness);
                        }
                        if let Some(c) = p.hand.iter().find(|c| c.id == cid) {
                            return Some(c.definition.toughness);
                        }
                    }
                    None
                })
                .sum(),
            Value::LifeOf(p) => self.resolve_player(p, ctx).map(|p| self.players[p].life).unwrap_or(0),
            Value::HandSizeOf(p) => self.resolve_player(p, ctx).map(|p| self.players[p].hand.len() as i32).unwrap_or(0),
            Value::LifeGainedThisTurn(p) => self.resolve_player(p, ctx).map(|p| self.players[p].life_gained_this_turn as i32).unwrap_or(0),
            // Max over the resolved set, so `EachOpponent` reads "the most
            // life any opponent lost this turn" (Spinerock Knoll).
            Value::LifeLostThisTurn(p) => self
                .resolve_players(p, ctx)
                .iter()
                .map(|&p| self.players[p].life_lost_this_turn as i32)
                .max()
                .unwrap_or(0),
            Value::CreaturesAttackedWithThisTurn(p) => self
                .resolve_players(p, ctx)
                .iter()
                .map(|&p| self.players[p].creatures_attacked_this_turn as i32)
                .max()
                .unwrap_or(0),
            Value::DistinctPowerYouControl => {
                let mut powers: Vec<i32> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == ctx.controller && c.definition.is_creature())
                    .map(|c| c.power())
                    .collect();
                powers.sort_unstable();
                powers.dedup();
                powers.len() as i32
            }
            Value::GraveyardSizeOf(p) => self.resolve_player(p, ctx).map(|p| self.players[p].graveyard.len() as i32).unwrap_or(0),
            Value::MaxGraveyardSize => self
                .players
                .iter()
                .filter(|p| p.is_alive())
                .map(|p| p.graveyard.len() as i32)
                .max()
                .unwrap_or(0),
            Value::LibrarySizeOf(p) => self.resolve_player(p, ctx).map(|p| self.players[p].library.len() as i32).unwrap_or(0),
            Value::XFromCost => ctx.x_value as i32,
            Value::TriggerEventAmount => ctx.event_amount as i32,
            Value::LastDieRoll => self.last_die_roll as i32,
            Value::StormCount => self.spells_cast_this_turn.saturating_sub(1) as i32,
            Value::DevotionTo(colors) => self.devotion_to(ctx.controller, colors),
            Value::CountersOn { what, kind } => self
                .resolve_selector(what, ctx)
                .into_iter()
                .filter_map(|e| {
                    let cid = match e {
                        EntityRef::Permanent(c) | EntityRef::Card(c) => c,
                        _ => return None,
                    };
                    // CR 122 — counters persist on a card when it moves
                    // between zones. So a die-trigger that reads "its
                    // +1/+1 counters" needs to be able to find the
                    // freshly-died card in its new graveyard zone. The
                    // battlefield lookup stays first (the common case),
                    // then we fall through to graveyards and exile —
                    // matching the cross-zone search shape of
                    // `evaluate_requirement_static` for WithCounter.
                    self.battlefield_find(cid)
                        .or_else(|| self.players.iter().find_map(
                            |p| p.graveyard.iter().find(|c| c.id == cid)))
                        .or_else(|| self.exile.iter().find(|c| c.id == cid))
                        .map(|c| c.counter_count(*kind) as i32)
                })
                // CR-spec: "the number of [counter type] on X" returns the
                // total across all entities X resolves to. Single-entity
                // selectors (`Target(0)`, `This`) still return that entity's
                // count; fan-out selectors (`EachPermanent(filter)`) now sum
                // — unblocking "total +1/+1 counters across all creatures
                // you control" cards (Reflective Anatomy). Lock-in test:
                // `tests::stx::reflective_anatomy_pumps_target_by_total_counters`.
                .sum(),
            Value::Sum(vs) => vs.iter().map(|v| self.evaluate_value(v, ctx)).sum(),
            Value::Diff(a, b) => self.evaluate_value(a, ctx) - self.evaluate_value(b, ctx),
            Value::Times(a, b) => self.evaluate_value(a, ctx) * self.evaluate_value(b, ctx),
            Value::Min(a, b) => self.evaluate_value(a, ctx).min(self.evaluate_value(b, ctx)),
            Value::Max(a, b) => self.evaluate_value(a, ctx).max(self.evaluate_value(b, ctx)),
            Value::NonNeg(v) => self.evaluate_value(v, ctx).max(0),
            Value::HalvedRoundUp(v) => (self.evaluate_value(v, ctx).max(0) + 1) / 2,
            Value::IfAtLeast { value, threshold, then, else_ } => {
                if self.evaluate_value(value, ctx) >= *threshold {
                    self.evaluate_value(then, ctx)
                } else {
                    self.evaluate_value(else_, ctx)
                }
            }
            Value::SacrificedPower => self.sacrificed_power.unwrap_or(0),
            Value::SacrificedToughness => self.sacrificed_toughness.unwrap_or(0),
            Value::CardsDiscardedThisEffect => self.cards_discarded_this_resolution as i32,
            Value::MaxCardsDiscardedThisEffectByAnyPlayer => self
                .cards_discarded_per_player_this_resolution
                .values()
                .copied()
                .max()
                .unwrap_or(0) as i32,
            Value::CreatureCardsDiscardedThisEffect => {
                self.creature_cards_discarded_this_resolution as i32
            }
            Value::DistinctManaValuesInExileWithCounter { counter } => {
                let p = ctx.controller;
                let mut mvs: Vec<u32> = self.exile.iter()
                    .filter(|c| c.owner == p
                        && !c.definition.is_land()
                        && c.counter_count(*counter) > 0)
                    .map(|c| c.definition.cost.cmc())
                    .collect();
                mvs.sort_unstable();
                mvs.dedup();
                mvs.len() as i32
            }
            Value::PermanentsDestroyedThisResolution => {
                self.permanents_destroyed_this_resolution as i32
            }
            Value::ConvergedValue => ctx.converged_value as i32,
            Value::SquadCount => ctx
                .source
                .and_then(|s| self.battlefield_find(s))
                .map(|c| c.squad_count as i32)
                .unwrap_or(0),
            Value::TimesKicked => ctx
                .source
                .and_then(|s| self.battlefield_find(s))
                .map(|c| c.kick_count as i32)
                .unwrap_or(0),
            Value::CastSpellManaSpent => {
                // Prefer the spell stack item's stored `mana_spent` when
                // the just-cast spell is still on the stack (trigger
                // evaluation at cast-time). Falls back to the trigger
                // context's `mana_spent` (set when
                // `fire_spell_cast_triggers` pushes the trigger, or when
                // the spell itself is resolving and reading from its own
                // resolution context).
                if let Some(EntityRef::Card(cid)) = ctx.trigger_source
                    && let Some(ms) = self.stack.iter().find_map(|si| match si {
                        StackItem::Spell { card, mana_spent, .. } if card.id == cid => {
                            Some(*mana_spent as i32)
                        }
                        _ => None,
                    })
                {
                    return ms;
                }
                ctx.mana_spent as i32
            }
            Value::LoyaltyOf(s) => self
                .resolve_selector(s, ctx)
                .into_iter()
                .find_map(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => self
                        .battlefield_find(cid)
                        .or_else(|| {
                            self.players.iter().find_map(|p| {
                                p.graveyard.iter().find(|c| c.id == cid)
                            })
                        })
                        .or_else(|| self.exile.iter().find(|c| c.id == cid))
                        .map(|c| {
                            c.counter_count(crate::card::CounterType::Loyalty) as i32
                        }),
                    EntityRef::Player(_) => None,
                })
                .unwrap_or(0),
            Value::ManaValueOf(s) => self
                .resolve_selector(s, ctx)
                .into_iter()
                .find_map(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => self
                        .battlefield_find(cid)
                        .or_else(|| {
                            self.players.iter().find_map(|p| {
                                p.graveyard
                                    .iter()
                                    .find(|c| c.id == cid)
                                    .or_else(|| p.hand.iter().find(|c| c.id == cid))
                                    .or_else(|| p.library.iter().find(|c| c.id == cid))
                            })
                        })
                        .or_else(|| self.exile.iter().find(|c| c.id == cid))
                        // Walk the stack last so a SpellCast trigger's
                        // filter predicate can read the mana value of the
                        // spell that just went on the stack but hasn't
                        // resolved yet (Up the Beanstalk, Mind's Desire,
                        // etc.).
                        .or_else(|| self.stack.iter().find_map(|si| match si {
                            StackItem::Spell { card, .. } if card.id == cid => Some(&**card),
                            _ => None,
                        }))
                        .map(|c| c.definition.cost.cmc() as i32),
                    EntityRef::Player(_) => None,
                })
                .unwrap_or(0),
            Value::DistinctTypesInTopOfLibrary { who, count } => {
                let Some(p) = self.resolve_player(who, ctx) else { return 0; };
                let n = self.evaluate_value(count, ctx).max(0) as usize;
                let mut seen: std::collections::HashSet<CardType> =
                    std::collections::HashSet::new();
                for card in self.players[p].library.iter().take(n) {
                    for t in &card.definition.card_types {
                        seen.insert(t.clone());
                    }
                }
                seen.len() as i32
            }
            Value::DistinctTypesInGraveyard { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return 0; };
                let mut seen: std::collections::HashSet<CardType> =
                    std::collections::HashSet::new();
                for card in &self.players[p].graveyard {
                    for t in &card.definition.card_types {
                        seen.insert(t.clone());
                    }
                }
                seen.len() as i32
            }
            Value::DistinctCardTypesExiledWith => {
                let Some(src) = ctx.source else { return 0; };
                let mut seen: std::collections::HashSet<CardType> =
                    std::collections::HashSet::new();
                for card in self.exile.iter().filter(|c| c.exiled_with == Some(src)) {
                    for t in &card.definition.card_types {
                        seen.insert(t.clone());
                    }
                }
                seen.len() as i32
            }
            Value::CardsDrawnThisTurn(p) => self
                .resolve_player(p, ctx)
                .map(|p| self.players[p].cards_drawn_this_turn as i32)
                .unwrap_or(0),
            Value::CardsDrawnThisStep(p) => self
                .resolve_player(p, ctx)
                .map(|p| self.players[p].cards_drawn_this_step as i32)
                .unwrap_or(0),
            Value::LandsPlayedThisTurn(p) => self
                .resolve_player(p, ctx)
                .map(|p| self.players[p].lands_played_this_turn as i32)
                .unwrap_or(0),
            Value::CreaturesDiedThisTurn(p) => self
                .resolve_player(p, ctx)
                .map(|p| self.players[p].creatures_died_this_turn as i32)
                .unwrap_or(0),
            Value::CreaturesDiedThisTurnTotal => self
                .players
                .iter()
                .map(|p| p.creatures_died_this_turn as i32)
                .sum(),
            Value::LowestLifeTotal => self
                .players
                .iter()
                .map(|p| p.life)
                .min()
                .unwrap_or(0),
            Value::HighestLifeTotal => self
                .players
                .iter()
                .filter(|p| p.is_alive())
                .map(|p| p.life)
                .max()
                .unwrap_or(0),
            Value::Pow2(inner) => {
                let exp = self.evaluate_value(inner, ctx).clamp(0, 30);
                1i32.checked_shl(exp as u32).unwrap_or(i32::MAX)
            }
            Value::HalfDown(inner) => self.evaluate_value(inner, ctx) / 2,
            Value::PermanentCountControlledBy(p) => self
                .resolve_player(p, ctx)
                .map(|seat| {
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == seat)
                        .count() as i32
                })
                .unwrap_or(0),
            Value::CreatureCountControlledBy(p) => self
                .resolve_player(p, ctx)
                .map(|seat| {
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == seat && c.definition.is_creature())
                        .count() as i32
                })
                .unwrap_or(0),
            Value::NonbasicLandCountControlledBy(p) => self
                .resolve_player(p, ctx)
                .map(|seat| {
                    self.battlefield
                        .iter()
                        .filter(|c| {
                            c.controller == seat
                                && c.definition.is_land()
                                && !c.definition.is_basic()
                        })
                        .count() as i32
                })
                .unwrap_or(0),
            Value::SnowPermanentCountControlledBy(p) => self
                .resolve_player(p, ctx)
                .map(|seat| {
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == seat && c.definition.is_snow())
                        .count() as i32
                })
                .unwrap_or(0),
            Value::DomainCount(p) => self
                .resolve_player(p, ctx)
                .map(|seat| self.domain_count(seat) as i32)
                .unwrap_or(0),
            Value::SameNamedInAllGraveyards => {
                let Some(name) = ctx.source_name.filter(|n| !n.is_empty()) else { return 0 };
                self.players
                    .iter()
                    .flat_map(|p| p.graveyard.iter())
                    .filter(|c| c.definition.name == name)
                    .count() as i32
            }
            Value::IfPred { pred, then, else_ } => {
                if self.evaluate_predicate(pred, ctx) {
                    self.evaluate_value(then, ctx)
                } else {
                    self.evaluate_value(else_, ctx)
                }
            }
        }
    }

    /// CR 702.43 — distinct basic land types among lands `seat` controls (0–5).
    pub(crate) fn domain_count(&self, seat: usize) -> usize {
        use crate::card::LandType::*;
        [Plains, Island, Swamp, Mountain, Forest]
            .into_iter()
            .filter(|lt| {
                self.battlefield.iter().any(|c| {
                    c.controller == seat
                        && c.definition.is_land()
                        && c.definition.subtypes.land_types.contains(lt)
                })
            })
            .count()
    }

    pub(crate) fn evaluate_predicate(&self, p: &Predicate, ctx: &EffectContext) -> bool {
        match p {
            Predicate::True => true,
            Predicate::False => false,
            Predicate::Not(q) => !self.evaluate_predicate(q, ctx),
            Predicate::All(qs) => qs.iter().all(|q| self.evaluate_predicate(q, ctx)),
            Predicate::Any(qs) => qs.iter().any(|q| self.evaluate_predicate(q, ctx)),
            Predicate::SelectorExists(s) => !self.resolve_selector(s, ctx).is_empty(),
            Predicate::SelectorCountAtLeast { sel, n } => {
                self.resolve_selector(sel, ctx).len() as i32 >= self.evaluate_value(n, ctx)
            }
            Predicate::ValueAtLeast(a, b) => self.evaluate_value(a, ctx) >= self.evaluate_value(b, ctx),
            Predicate::ValueAtMost(a, b) => self.evaluate_value(a, ctx) <= self.evaluate_value(b, ctx),
            Predicate::ValueEquals(a, b) => self.evaluate_value(a, ctx) == self.evaluate_value(b, ctx),
            Predicate::ValueIsOdd(v) => self.evaluate_value(v, ctx).rem_euclid(2) == 1,
            Predicate::PlayerSacrificedThisResolution(pref) => self
                .resolve_player(pref, ctx)
                .is_some_and(|p| self.players_sacrificed_this_resolution.contains(&p)),
            Predicate::IsTurnOf(pref) => self.resolve_player(pref, ctx) == Some(self.active_player_idx),
            Predicate::CurrentStepIs(step) => self.step == *step,
            Predicate::EntityMatches { what, filter } => self
                .resolve_selector(what, ctx)
                .into_iter()
                .all(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => {
                        self.evaluate_requirement_static(filter, &Target::Permanent(cid), ctx.controller, ctx.source)
                    }
                    EntityRef::Player(_) => matches!(filter, SelectionRequirement::Player),
                }),
            Predicate::LifeGainedThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].life_gained_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::ExpendReached(n) => {
                // CR 700.14 — fired only on the cost-payment that pushes the
                // turn's spell-mana total from below `n` up to at least `n`.
                self.expend_prev_total < *n && ctx.event_amount >= *n
            }
            Predicate::HasCityBlessing { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].city_blessing),
            Predicate::IsDay => self.day_night == Some(crate::game::types::DayNight::Day),
            Predicate::IsNight => self.day_night == Some(crate::game::types::DayNight::Night),
            Predicate::IsMonarch { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.monarch == Some(p)),
            Predicate::PlayerDamagedThisTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].was_dealt_damage_this_turn),
            Predicate::PlayerLostLifeThisTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].lost_life_this_turn),
            Predicate::PlayerLifeAtMost { who, life } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.effective_life(p) <= *life),
            Predicate::PlayerHasMostLife { who } => {
                let max_life = (0..self.players.len())
                    .filter(|&p| !self.players[p].eliminated)
                    .map(|p| self.effective_life(p))
                    .max()
                    .unwrap_or(i32::MIN);
                self.resolve_players(who, ctx)
                    .into_iter()
                    .any(|p| self.effective_life(p) >= max_life)
            }
            Predicate::PlayerHasLessLifeThanOpponent { who } => {
                self.resolve_players(who, ctx).into_iter().any(|p| {
                    let my_life = self.effective_life(p);
                    (0..self.players.len()).any(|o| {
                        o != p && !self.players[o].eliminated && self.effective_life(o) > my_life
                    })
                })
            }
            Predicate::SourceAttackedThisTurn => ctx
                .source
                .and_then(|cid| self.battlefield.iter().find(|c| c.id == cid))
                .map(|c| c.attacked_this_turn)
                .unwrap_or(false),
            Predicate::SourceIsMonstrous => ctx
                .source
                .and_then(|cid| self.battlefield.iter().find(|c| c.id == cid))
                .map(|c| c.monstrous)
                .unwrap_or(false),
            Predicate::SourceIsSuspected => ctx
                .source
                .and_then(|cid| self.battlefield.iter().find(|c| c.id == cid))
                .map(|c| c.suspected)
                .unwrap_or(false),
            Predicate::SourceIsCreature => ctx
                .source
                .and_then(|cid| self.computed_permanent(cid))
                .map(|c| c.card_types.contains(&crate::card::CardType::Creature))
                .unwrap_or(false),
            Predicate::SourceSaddled => ctx
                .source
                .and_then(|cid| self.battlefield.iter().find(|c| c.id == cid))
                .map(|c| c.saddled)
                .unwrap_or(false),
            Predicate::SourceCastFromEscape => ctx
                .source
                .and_then(|cid| self.battlefield.iter().find(|c| c.id == cid))
                .map(|c| c.cast_from_escape)
                .unwrap_or(false),
            Predicate::SourceChampionedSomething => ctx.source.is_some_and(|cid| {
                self.exile.iter().any(|c| c.exiled_by.as_ref().is_some_and(|l| l.source == cid))
            }),
            Predicate::TriggerBlocksSource => match (ctx.trigger_source, ctx.source) {
                (Some(EntityRef::Permanent(blocker)), Some(src)) => {
                    self.block_map.get(&blocker) == Some(&src)
                }
                _ => false,
            },
            Predicate::TriggerObjectNameMatchesNamedCard => {
                let named = ctx
                    .source
                    .and_then(|cid| self.find_card_anywhere(cid))
                    .and_then(|c| c.named_card.clone());
                let cast_name = match ctx.trigger_source {
                    Some(EntityRef::Card(id)) => {
                        self.find_card_anywhere(id).map(|c| c.definition.name.to_string())
                    }
                    _ => None,
                };
                matches!((named, cast_name), (Some(n), Some(c)) if n == c)
            }
            Predicate::PlayerAttackedThisTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].attacked_this_turn),
            Predicate::AnotherCreatureEnteredControlLastTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| {
                    self.players[p]
                        .creatures_entered_last_turn
                        .iter()
                        .any(|&cid| Some(cid) != ctx.source)
                }),
            Predicate::CastBlueOrBlackThisTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].cast_blue_or_black_this_turn),
            Predicate::DiscardedNonlandThisEffect { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| {
                    self.nonland_cards_discarded_per_player_this_resolution
                        .get(&p)
                        .copied()
                        .unwrap_or(0)
                        > 0
                }),
            Predicate::CardsLeftGraveyardThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].cards_left_graveyard_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::SearchedLibraryThisTurn { who } => self
                .resolve_players(who, ctx)
                .iter()
                .any(|&p| self.players[p].searched_library_this_turn),
            Predicate::ProwlTypeDealtCombatDamage { types } => {
                let pl = &self.players[ctx.controller];
                pl.prowl_any_type_this_turn
                    || types.iter().any(|t| pl.prowl_types_this_turn.contains(t))
            }
            Predicate::CardsToGraveyardThisTurnAtLeast { who, at_least } => self
                .resolve_players(who, ctx)
                .iter()
                .any(|&p| self.players[p].cards_to_graveyard_this_turn >= *at_least),
            Predicate::SpellsCastThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].spells_cast_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::SpellsCastThisTurnEquals { who, count } => {
                let n = self.evaluate_value(count, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].spells_cast_this_turn == n)
                    .unwrap_or(false)
            }
            Predicate::NoSpellsCastLastTurn => self.spells_cast_last_turn == 0,
            Predicate::TwoOrMoreSpellsCastLastTurn => self.spells_cast_last_turn >= 2,
            Predicate::CreaturesDiedThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].creatures_died_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                let total: u32 = self
                    .players
                    .iter()
                    .map(|p| p.creatures_died_this_turn)
                    .sum();
                total >= n
            }
            Predicate::CardsExiledThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].cards_exiled_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::InstantsOrSorceriesCastThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].instants_or_sorceries_cast_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::CreaturesCastThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].creatures_cast_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::CastSpellTargetsMatch(filter) => {
                // Find the cast spell on the stack via the trigger source.
                // `fire_spell_cast_triggers` sets `ctx.trigger_source` to
                // `EntityRef::Card(cast_card_id)` so we can locate the
                // `StackItem::Spell` that just got pushed.
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                let target = self.stack.iter().find_map(|si| match si {
                    StackItem::Spell { card, target, .. } if card.id == cid => Some(target.clone()),
                    _ => None,
                });
                match target {
                    Some(Some(t)) => self.evaluate_requirement_static(filter, &t, ctx.controller, ctx.source),
                    _ => false,
                }
            }
            Predicate::CastSpellMatches(filter) => {
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cid => {
                        self.evaluate_requirement_on_card(filter, card, ctx.controller)
                    }
                    _ => false,
                })
            }
            Predicate::CastSpellWasKicked => {
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cid => card.kicked,
                    _ => false,
                })
            }
            Predicate::CastSpellHasX => {
                // Locate the just-cast spell via the trigger source and
                // peek at its printed mana cost. Used by "whenever you
                // cast a spell with {X} in its cost" Quandrix triggers.
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cid => {
                        card.definition.cost.has_x()
                    }
                    _ => false,
                })
            }
            Predicate::CastSpellManaSpentAtLeast(min) => {
                // First try the most precise read: the just-cast spell's
                // `StackItem::Spell.mana_spent`. Falls back to
                // `ctx.mana_spent` (set when this filter runs at
                // cast-trigger-push time, when the spell hasn't been
                // popped from the stack yet) so Opus filters at
                // `fire_spell_cast_triggers` time also see the right
                // value.
                if let Some(EntityRef::Card(cid)) = ctx.trigger_source
                    && let Some(ms) = self.stack.iter().find_map(|si| match si {
                        StackItem::Spell { card, mana_spent, .. } if card.id == cid => {
                            Some(*mana_spent)
                        }
                        _ => None,
                    })
                {
                    return ms >= *min;
                }
                ctx.mana_spent >= *min
            }
            Predicate::SourceGainedCounterThisTurn => {
                ctx.source
                    .map(|cid| self.permanents_gained_counter_this_turn.contains(&cid))
                    .unwrap_or(false)
            }
            Predicate::CastSpellFromExile => {
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cid => card.cast_from_exile,
                    _ => false,
                })
            }
            Predicate::CastSpellNotOwnedByYou => {
                // Owner ≠ controller test against the just-cast spell.
                // Resolution: walk the stack for the trigger source's
                // `StackItem::Spell.card.owner` and compare to
                // `ctx.controller` (the triggered-ability controller =
                // the spell's caster). Falls back to `false` when the
                // spell can't be located (defensive — should not happen
                // during normal CastSpell trigger dispatch).
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cid => {
                        card.owner != ctx.controller
                    }
                    _ => false,
                })
            }
            Predicate::SameNamedInZoneAtLeast { who, zone, at_least } => {
                // Read the resolving spell's printed name from
                // `ctx.source_name` (stamped by `for_spell_with_source`).
                // During spell resolution the card is in transient
                // ownership and not present in any visible zone, so
                // `source_name` is the reliable channel. Fall back to
                // `ctx.source` (the source permanent's battlefield
                // entry) for activated-ability resolution paths where
                // `source_name` isn't stamped — Page, Loose Leaf's
                // Grandeur cost gate ("Discard another card named
                // Page, Loose Leaf") uses this fallback.
                let target_name = ctx.source_name.or_else(|| {
                    ctx.source.and_then(|cid| {
                        self.battlefield
                            .iter()
                            .find(|c| c.id == cid)
                            .map(|c| c.definition.name)
                    })
                });
                let Some(target_name) = target_name else {
                    return false;
                };
                let Some(seat) = self.resolve_player(who, ctx) else {
                    return false;
                };
                let n = self.evaluate_value(at_least, ctx).max(0) as usize;
                let count = match zone {
                    crate::card::Zone::Graveyard => self.players[seat]
                        .graveyard
                        .iter()
                        .filter(|c| c.definition.name == target_name)
                        .count(),
                    crate::card::Zone::Hand => self.players[seat]
                        .hand
                        .iter()
                        .filter(|c| c.definition.name == target_name)
                        .count(),
                    crate::card::Zone::Library => self.players[seat]
                        .library
                        .iter()
                        .filter(|c| c.definition.name == target_name)
                        .count(),
                    crate::card::Zone::Exile => self
                        .exile
                        .iter()
                        .filter(|c| c.owner == seat && c.definition.name == target_name)
                        .count(),
                    crate::card::Zone::Battlefield => self
                        .battlefield
                        .iter()
                        .filter(|c| c.controller == seat && c.definition.name == target_name)
                        .count(),
                    crate::card::Zone::Stack | crate::card::Zone::Command => 0,
                };
                count >= n
            }
            Predicate::CastFromGraveyard => {
                // Read directly off the resolution context. Stamped by
                // `for_spell_with_source` from the resolving
                // `CardInstance.cast_from_hand` flag. Non-spell
                // contexts default `cast_from_hand` to true, so this
                // predicate is `False` for triggers and activated
                // abilities — which matches the printed wording
                // ("cast from a graveyard" is a spell-only concept).
                !ctx.cast_from_hand
            }
            Predicate::CastFromHand => {
                // Inverse of CastFromGraveyard. Triggers / activated
                // abilities default `cast_from_hand` to `true` which
                // matches their non-spell-resolution context.
                ctx.cast_from_hand
            }
            Predicate::SpellWasKicked => {
                // CR 702.32 — true iff the kicker cost was paid at cast
                // time. Stamped onto `ctx.kicked` from the resolving
                // `CardInstance.kicked` flag.
                ctx.kicked
            }
            Predicate::SpellWasBargained => {
                // CR 702.176 — true iff the Bargain cost was paid (an
                // artifact/enchantment/token sacrificed) at cast time.
                ctx.bargained
            }
            Predicate::OpponentControlsMoreLandsThanYou => {
                // Walk the battlefield, count lands per seat. True iff
                // any opponent of `ctx.controller` has strictly more
                // lands than the controller. Skips eliminated players
                // and shares seat ↔ team semantics via the helper.
                let you = ctx.controller;
                let mut your_lands = 0usize;
                let mut max_opp_lands = 0usize;
                for c in &self.battlefield {
                    if !c.definition.is_land() {
                        continue;
                    }
                    if c.controller == you {
                        your_lands += 1;
                    } else if !self.same_team(c.controller, you)
                        && !self.players[c.controller].eliminated
                    {
                        // Track the largest opponent land count so we
                        // compare against the most-ahead opponent.
                        // (Tracking a per-opp sum and taking the max
                        // would require a HashMap; the same effect is
                        // achieved by counting each opp's lands.)
                        let opp_lands = self
                            .battlefield
                            .iter()
                            .filter(|p| {
                                p.controller == c.controller && p.definition.is_land()
                            })
                            .count();
                        if opp_lands > max_opp_lands {
                            max_opp_lands = opp_lands;
                        }
                    }
                }
                max_opp_lands > your_lands
            }
            Predicate::AnOpponentHasMoreLife => {
                let you = ctx.controller;
                let your_life = self.players[you].life;
                self.players.iter().enumerate().any(|(i, p)| {
                    i != you && !p.eliminated && !self.same_team(i, you) && p.life > your_life
                })
            }
            Predicate::AnOpponentControlsMoreCreatures => {
                let you = ctx.controller;
                let count_creatures = |seat: usize, g: &Self| {
                    g.battlefield
                        .iter()
                        .filter(|c| c.controller == seat && c.definition.is_creature())
                        .count()
                };
                let your_creatures = count_creatures(you, self);
                (0..self.players.len()).any(|i| {
                    i != you
                        && !self.players[i].eliminated
                        && !self.same_team(i, you)
                        && count_creatures(i, self) > your_creatures
                })
            }
            Predicate::AttackingAlone => self.attacking.len() == 1,
            Predicate::AttackingWithAtLeast(n) => self.attacking.len() as u32 >= *n,
            Predicate::RevoltActive { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].permanent_left_battlefield_this_turn),
            Predicate::DeliriumActive { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                let mut kinds: std::collections::HashSet<&crate::card::CardType> =
                    std::collections::HashSet::new();
                for c in &self.players[p].graveyard {
                    for t in &c.definition.card_types {
                        kinds.insert(t);
                    }
                }
                kinds.len() >= 4
            }
            Predicate::IncrementSatisfied => {
                // SOS Increment: "Whenever you cast a spell, if the
                // amount of mana you spent is greater than this
                // creature's power or toughness, put a +1/+1 counter on
                // this creature." Both clauses (P and T) are OR'd —
                // pumps fire whenever mana_spent strictly exceeds
                // *either* stat. We evaluate against the listening
                // permanent (the source whose triggered ability we're
                // gating).
                let Some(source_id) = ctx.source else {
                    return false;
                };
                let Some(source_card) = self.battlefield_find(source_id) else {
                    // If the Increment-bearing creature already left
                    // the battlefield (e.g. countered cast that resolved
                    // a removal spell first), the trigger no-ops.
                    return false;
                };
                // Resolve mana_spent the same way as
                // `CastSpellManaSpentAtLeast` — prefer the stack item
                // if the spell hasn't resolved yet, otherwise fall back
                // to `ctx.mana_spent`.
                let mana_spent = if let Some(EntityRef::Card(cid)) = ctx.trigger_source {
                    self.stack
                        .iter()
                        .find_map(|si| match si {
                            StackItem::Spell { card, mana_spent, .. } if card.id == cid => {
                                Some(*mana_spent)
                            }
                            _ => None,
                        })
                        .unwrap_or(ctx.mana_spent)
                } else {
                    ctx.mana_spent
                };
                let p = source_card.power();
                let t = source_card.toughness();
                (mana_spent as i32 > p) || (mana_spent as i32 > t)
            }
        }
    }

    // ── Requirement evaluation (unchanged API) ──────────────────────────────

    pub(crate) fn evaluate_requirement_static(
        &self,
        req: &SelectionRequirement,
        target: &Target,
        controller: usize,
        source: Option<CardId>,
    ) -> bool {
        use SelectionRequirement as R;
        match req {
            R::Any => true,
            R::Player => matches!(target, Target::Player(_)),
            R::OpponentPlayer => {
                matches!(target, Target::Player(p) if !self.same_team(*p, controller))
            }
            R::And(a, b) => self.evaluate_requirement_static(a, target, controller, source)
                && self.evaluate_requirement_static(b, target, controller, source),
            R::Or(a, b) => self.evaluate_requirement_static(a, target, controller, source)
                || self.evaluate_requirement_static(b, target, controller, source),
            R::Not(inner) => !self.evaluate_requirement_static(inner, target, controller, source),
            R::ControlledByYou => match target {
                Target::Permanent(cid) => self.battlefield_find(*cid).map(|c| c.controller == controller).unwrap_or(false),
                Target::Player(p) => *p == controller,
            },
            R::ControlledByOpponent => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| !self.same_team(c.controller, controller))
                    .unwrap_or(false),
                Target::Player(p) => !self.same_team(*p, controller),
            },
            R::DealtDamageToControllerThisTurn => match target {
                Target::Permanent(cid) => self.players[controller]
                    .creatures_that_damaged_me_this_turn
                    .contains(cid),
                Target::Player(_) => false,
            },
            _ => {
                let Target::Permanent(cid) = target else { return false; };
                // Look on the battlefield first; fall through to graveyards,
                // exile, and the stack so reanimate-style spells (Goryo's
                // Vengeance, Reanimate, Animate Dead) can validate their
                // targets, and so counter-style spells (Mystical Dispute,
                // Force of Negation) can read the colors of a target stack
                // spell.
                let stack_card = self.stack.iter().find_map(|si| match si {
                    StackItem::Spell { card, .. } if card.id == *cid => Some(&**card),
                    _ => None,
                });
                let card = self
                    .battlefield_find(*cid)
                    .or_else(|| self.players.iter().find_map(|p| p.graveyard.iter().find(|c| c.id == *cid)))
                    .or_else(|| self.exile.iter().find(|c| c.id == *cid))
                    .or(stack_card)
                    // Library / hand: needed by "look at top of library"
                    // predicates (Lurking Predators: "if it's a creature
                    // card, …"), discard-from-hand pickers, and any future
                    // hidden-zone filter check. Library cards have hidden
                    // info for opponents in real play, but the engine is
                    // permission-checked at the call site (effects target
                    // the controller's own library).
                    .or_else(|| self.players.iter().find_map(|p| p.library.iter().find(|c| c.id == *cid)))
                    .or_else(|| self.players.iter().find_map(|p| p.hand.iter().find(|c| c.id == *cid)))
                    // Dying-card snapshot, populated at SBA die-time and
                    // cleared after trigger dispatch.
                    // Lets predicates like
                    // `EntityMatches { TriggerSource, HasCreatureType(Pest) }`
                    // read the dying card's printed types even when the
                    // card vanished from every zone via CR 111.7c
                    // (token-ceases-to-exist).
                    .or_else(|| self.died_card_snapshots.get(cid));
                let Some(card) = card else { return false; };
                // Layer-4-aware card types for battlefield permanents
                // (CR 613.2): an artifact-ized creature (Phyrexian
                // Scriptures I), an animated land, or a devotion-gated god
                // must filter by its *computed* types, not the printed ones.
                // Off-battlefield cards keep the printed definition.
                let computed_types: Option<Vec<crate::card::CardType>> =
                    if self.in_layer_gather.load(std::sync::atomic::Ordering::Relaxed) {
                        None // mid-recompute: printed types (reentrancy guard)
                    } else {
                        self.battlefield_find(*cid)
                            .and_then(|_| self.computed_permanent(*cid))
                            .map(|cp| cp.card_types.clone())
                    };
                let has_type = |t: crate::card::CardType| match &computed_types {
                    Some(ts) => ts.contains(&t),
                    None => card.definition.card_types.contains(&t),
                };
                use crate::card::CardType as CT;
                match req {
                    R::Creature => has_type(CT::Creature),
                    R::Artifact => has_type(CT::Artifact),
                    R::Enchantment => has_type(CT::Enchantment),
                    R::Planeswalker => has_type(CT::Planeswalker),
                    R::Permanent => card.definition.is_permanent(),
                    R::Land => has_type(CT::Land),
                    R::Nonland => !has_type(CT::Land),
                    R::Noncreature => !has_type(CT::Creature),
                    R::Tapped => card.tapped,
                    R::Untapped => !card.tapped,
                    // CR 105.2/202.2 — hybrid/Phyrexian pips count via
                    // `ManaCost::colors()` (bare `Colored` scan missed them).
                    R::HasColor(c) => card.definition.cost.colors().contains(c),
                    R::HasKeyword(kw) => card.has_keyword(kw),
                    R::HasCyclingAbility => card.definition.keywords.iter().any(|k| matches!(
                        k,
                        crate::card::Keyword::Cycling(_)
                            | crate::card::Keyword::CyclingLife(_)
                            | crate::card::Keyword::Landcycling(_, _)
                            | crate::card::Keyword::Typecycling(_)
                    )),
                    R::PowerAtMost(n) => card.definition.is_creature() && card.power() <= *n,
                    R::ToughnessAtMost(n) => card.definition.is_creature() && card.toughness() <= *n,
                    R::PowerAtLeast(n) => card.definition.is_creature() && card.power() >= *n,
                    R::ToughnessAtLeast(n) => card.definition.is_creature() && card.toughness() >= *n,
                    R::PowerPlusToughnessAtMost(n) => {
                        card.definition.is_creature() && card.power() + card.toughness() <= *n
                    }
                    R::PowerLessThanSource => {
                        source
                            .and_then(|s| self.battlefield_find(s))
                            .is_some_and(|src| {
                                card.definition.is_creature() && card.power() < src.power()
                            })
                    }
                    R::GreaterPowerOrToughnessThanSource => {
                        source
                            .and_then(|s| self.battlefield_find(s))
                            .is_some_and(|src| {
                                card.definition.is_creature()
                                    && (card.power() > src.power()
                                        || card.toughness() > src.toughness())
                            })
                    }
                    R::PowerGreaterThanSource => {
                        source
                            .and_then(|s| self.battlefield_find(s))
                            .is_some_and(|src| {
                                card.definition.is_creature() && card.power() > src.power()
                            })
                    }
                    R::WithCounter(k) => card.counter_count(*k) > 0,
                    R::HasNoCounters => {
                        card.counters.values().all(|&n| n == 0)
                            && card.keyword_counters.values().all(|&n| n == 0)
                    }
                    R::HasSupertype(st) => card.definition.supertypes.contains(st),
                    R::HasCreatureType(ct) => card.definition.subtypes.creature_types.contains(ct)
                        || card.has_keyword(&crate::card::Keyword::Changeling),
                    R::HasLandType(lt) => card.definition.subtypes.land_types.contains(lt),
                    R::HasArtifactSubtype(a) => card.definition.subtypes.artifact_subtypes.contains(a),
                    R::HasEnchantmentSubtype(e) => card.definition.subtypes.enchantment_subtypes.contains(e),
                    R::IsToken => card.is_token,
                    R::NotToken => !card.is_token,
                    R::IsBasicLand => card.definition.is_land() && card.definition.supertypes.contains(&Supertype::Basic),
                    R::IsNonbasicLand => card.definition.is_land() && !card.definition.supertypes.contains(&Supertype::Basic),
                    R::IsAttacking => self.attacking.iter().any(|a| a.attacker == card.id),
                    R::IsBlocking => self.block_map.contains_key(&card.id),
                    R::AttackedThisTurn => card.attacked_this_turn,
                    // CR 603.4 — entered this turn (stamped on every ETB).
                    R::EnteredThisTurn => card.entered_turn == Some(self.turn_number),
                    R::EnteredFromGraveyardThisTurn => {
                        self.entered_from_graveyard_this_turn.contains(cid)
                    }
                    // CR 303 — "enchanted" = an Aura is attached. Equipment also
                    // sets `attached_to`, so require the attachment be an
                    // enchantment to exclude it.
                    R::IsEnchanted => self.battlefield.iter().any(|o| {
                        o.attached_to == Some(*cid) && o.definition.is_enchantment()
                    }),
                    // CR 301.5 — "equipped" = an Equipment is attached.
                    R::IsEquipped => self.battlefield.iter().any(|o| {
                        o.attached_to == Some(*cid) && o.definition.is_equipment()
                    }),
                    // CR 700.9 — counters, equipped, or enchanted by an Aura
                    // the permanent's own controller controls.
                    R::IsModified => {
                        !card.counters.is_empty()
                            || self.battlefield.iter().any(|o| {
                                o.attached_to == Some(*cid)
                                    && (o.definition.is_artifact()
                                        || (o.definition.is_enchantment()
                                            && o.controller == card.controller))
                            })
                    }
                    // CR 506.5: attacking alone = card is in attacking AND
                    // there is exactly one declared attacker.
                    R::IsAttackingAlone => {
                        self.attacking.len() == 1
                            && self.attacking.iter().any(|a| a.attacker == card.id)
                    }
                    // CR 506.5: blocking alone = card is in block_map keys
                    // AND there is exactly one declared blocker.
                    R::IsBlockingAlone => {
                        self.block_map.len() == 1 && self.block_map.contains_key(&card.id)
                    }
                    R::IsSpellOnStack => self.stack.iter().any(|si| matches!(si, StackItem::Spell { card: c, .. } if c.id == card.id)),
                    R::HasAbilityOnStack => self.stack.iter().any(|si| matches!(
                        si,
                        StackItem::Trigger { source, .. } if *source == card.id
                    )),
                    R::ManaValueAtMost(n) => card.definition.cost.cmc() <= *n,
                    R::ManaValueAtMostYourCount(inner) => {
                        let n = self
                            .battlefield
                            .iter()
                            .filter(|c| self.evaluate_requirement_on_card(inner, c, controller))
                            .count() as u32;
                        card.definition.cost.cmc() <= n
                    }
                    // Unresolved X-relative filter (no X in scope here).
                    R::ManaValueAtMostXFromCost | R::ManaValueAtMostConverged => false,
                    R::ManaValueAtLeast(n) => card.definition.cost.cmc() >= *n,
                    R::ManaValueExactly(n) => card.definition.cost.cmc() == *n,
                    R::ManaValueEqualsSacrificedPlus(off) => {
                        card.definition.cost.cmc()
                            == self.sacrificed_mana_value.unwrap_or(0) + *off
                    }
                    R::ManaValueLessThanEventAmount => {
                        card.definition.cost.cmc() < self.trigger_event_amount_scratch
                    }
                    R::HasCardType(ct) => {
                        // CR 715 — an adventuring card is its instant/sorcery
                        // half while on the stack, so report the adventure types.
                        if card.adventuring {
                            card.definition.adventure.as_ref()
                                .map(|a| a.card_types.contains(ct)).unwrap_or(false)
                        } else {
                            card.definition.card_types.contains(ct)
                        }
                    }
                    R::Multicolored => card.definition.cost.distinct_colors() >= 2,
                    // CR 702.114 — Devoid CDA: colorless despite colored pips.
                    R::Colorless => card.definition.keywords.contains(&crate::card::Keyword::Devoid)
                        || card.definition.cost.distinct_colors() == 0,
                    R::Monocolored => card.definition.cost.distinct_colors() == 1,
                    R::HasXInCost => card.definition.cost.has_x(),
                    // OtherThanSource: enforce "different from the source"
                    // when a source CardId is threaded into this call (effect
                    // resolvers pass `ctx.source`, cast-time validators pass
                    // `None`). Without source context, falls through to
                    // permissive (matches the old behavior, leaving the
                    // static-ability `applies_to` pipeline to handle the
                    // "Other …" half via `AffectedPermanents.exclude_source`).
                    R::OtherThanSource => match source {
                        Some(src_id) => *cid != src_id,
                        None => true,
                    },
                    R::InGraveyard => self
                        .players
                        .iter()
                        .any(|p| p.graveyard.iter().any(|c| c.id == *cid)),
                    R::InYourGraveyard => self
                        .players
                        .get(controller)
                        .is_some_and(|p| p.graveyard.iter().any(|c| c.id == *cid)),
                    R::InExile => self.exile.iter().any(|c| c.id == *cid),
                    // CR-spec: "the greatest mana value among [filter] they
                    // control" — the candidate must (a) match `inner` and
                    // (b) have an MV ≥ every other matching permanent under
                    // the same controller. Used by SOS End of the Hunt;
                    // ties pass permissively so the auto-target picks among
                    // all max-MV matches.
                    R::HasGreatestManaValueAmongControlled(inner) => {
                        // Candidate must be a battlefield permanent that
                        // matches the inner filter.
                        let Some(cand) = self.battlefield_find(*cid) else {
                            return false;
                        };
                        if !self.evaluate_requirement_static(inner, target, controller, source) {
                            return false;
                        }
                        let cand_mv = cand.definition.cost.cmc();
                        let cand_ctrl = cand.controller;
                        // Walk the same controller's permanents matching
                        // inner; reject if any has a strictly greater MV.
                        !self.battlefield.iter().any(|other| {
                            other.controller == cand_ctrl
                                && other.id != *cid
                                && self.evaluate_requirement_static(
                                    inner,
                                    &Target::Permanent(other.id),
                                    controller,
                                    source,
                                )
                                && other.definition.cost.cmc() > cand_mv
                        })
                    }
                    R::HasName(name) => card.definition.name == name.as_str(),
                    R::ManaValueAtMostControlledCount(inner) => {
                        let count = self
                            .battlefield
                            .iter()
                            .filter(|c| {
                                self.evaluate_requirement_static(
                                    inner,
                                    &Target::Permanent(c.id),
                                    controller,
                                    source,
                                )
                            })
                            .count() as u32;
                        card.definition.cost.cmc() <= count
                    }
                    R::ManaValueAtMostControllerGraveyard => {
                        let count = self.players[card.controller].graveyard.len() as u32;
                        card.definition.cost.cmc() <= count
                    }
                    R::HasBackFace => card.definition.back_face.is_some(),
                    R::HasPrepareSpell => card.definition.prepare_spell.is_some(),
                    R::NameDiffersFromLastMoved => !self.last_moved_cards.iter().any(|id| {
                        self.find_card_anywhere(*id)
                            .is_some_and(|c| c.definition.name == card.definition.name)
                    }),
                    R::NamedBySource => source
                        .and_then(|sid| self.battlefield_find(sid))
                        .and_then(|s| s.named_card.as_deref())
                        .is_some_and(|n| n == card.definition.name),
                    _ => unreachable!("handled above"),
                }
            }
        }
    }

    /// Evaluate a `SelectionRequirement` directly against a `CardInstance`
    /// without requiring it to be on the battlefield. Used for library searches.
    /// Battlefield-only predicates (Tapped, IsAttacking, etc.) return false.
    pub(crate) fn evaluate_requirement_on_card(
        &self,
        req: &SelectionRequirement,
        card: &CardInstance,
        controller: usize,
    ) -> bool {
        use SelectionRequirement as R;
        match req {
            R::Any => true,
            R::Player | R::OpponentPlayer => false,
            R::And(a, b) => {
                self.evaluate_requirement_on_card(a, card, controller)
                    && self.evaluate_requirement_on_card(b, card, controller)
            }
            R::Or(a, b) => {
                self.evaluate_requirement_on_card(a, card, controller)
                    || self.evaluate_requirement_on_card(b, card, controller)
            }
            R::Not(inner) => !self.evaluate_requirement_on_card(inner, card, controller),
            R::ControlledByYou => card.controller == controller,
            R::ControlledByOpponent => !self.same_team(card.controller, controller),
            R::Creature => card.definition.is_creature(),
            R::Artifact => card.definition.is_artifact(),
            R::Enchantment => card.definition.is_enchantment(),
            R::Planeswalker => card.definition.is_planeswalker(),
            R::Permanent => card.definition.is_permanent(),
            R::Land => card.definition.is_land(),
            R::Nonland => !card.definition.is_land(),
            R::Noncreature => !card.definition.is_creature(),
            // A card's color is set by every colored symbol in its cost,
            // including hybrid ({R/W}), Phyrexian ({R/P}) and mono-hybrid
            // ({2/W}) pips — so a {R/W} card is both red and white. Defer to
            // the shared `ManaCost::colors()` helper (the same expansion the
            // battlefield path uses via `colors_from_card`); a bare
            // `Colored`-pip scan would mis-read hybrid/Phyrexian cards as
            // colorless in hidden zones.
            R::HasColor(c) => card.definition.cost.colors().contains(c),
            R::HasKeyword(kw) => card.has_keyword(kw),
            R::HasCyclingAbility => card.definition.keywords.iter().any(|k| matches!(
                k,
                crate::card::Keyword::Cycling(_)
                    | crate::card::Keyword::CyclingLife(_)
                    | crate::card::Keyword::Landcycling(_, _)
                    | crate::card::Keyword::Typecycling(_)
            )),
            R::PowerAtMost(n) => card.definition.is_creature() && card.power() <= *n,
            R::PowerAtLeast(n) => card.definition.is_creature() && card.power() >= *n,
            // No source/battlefield context in the on-card evaluator (used
            // for hidden-zone cards); the source-relative Mentor check only
            // makes sense for battlefield targets, so it's vacuously false.
            R::PowerLessThanSource => false,
            R::GreaterPowerOrToughnessThanSource => false,
            R::PowerGreaterThanSource => false,
            R::ToughnessAtMost(n) => card.definition.is_creature() && card.toughness() <= *n,
            R::ToughnessAtLeast(n) => card.definition.is_creature() && card.toughness() >= *n,
            R::PowerPlusToughnessAtMost(n) => {
                card.definition.is_creature() && card.power() + card.toughness() <= *n
            }
            R::HasSupertype(st) => card.definition.supertypes.contains(st),
            R::HasCreatureType(ct) => card.definition.subtypes.creature_types.contains(ct)
                        || card.has_keyword(&crate::card::Keyword::Changeling),
            R::HasLandType(lt) => card.definition.subtypes.land_types.contains(lt),
            R::HasArtifactSubtype(a) => card.definition.subtypes.artifact_subtypes.contains(a),
            R::HasEnchantmentSubtype(e) => card.definition.subtypes.enchantment_subtypes.contains(e),
            R::IsToken => card.is_token,
            R::NotToken => !card.is_token,
            // CR 603.4 — entered this turn (hidden-zone cards are never
            // stamped, so this is false off the battlefield).
            R::EnteredThisTurn => card.entered_turn == Some(self.turn_number),
            R::EnteredFromGraveyardThisTurn => {
                self.entered_from_graveyard_this_turn.contains(&card.id)
            }
            R::IsBasicLand => card.definition.is_land() && card.definition.supertypes.contains(&Supertype::Basic),
            R::IsNonbasicLand => card.definition.is_land() && !card.definition.supertypes.contains(&Supertype::Basic),
            R::ManaValueAtMost(n) => card.definition.cost.cmc() <= *n,
            R::ManaValueAtMostYourCount(inner) => {
                let n = self
                    .battlefield
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(inner, c, controller))
                    .count() as u32;
                card.definition.cost.cmc() <= n
            }
            // Unresolved X-relative filter (callers concretize via `resolve_x`).
            R::ManaValueAtMostXFromCost | R::ManaValueAtMostConverged => false,
            R::ManaValueAtLeast(n) => card.definition.cost.cmc() >= *n,
            R::ManaValueExactly(n) => card.definition.cost.cmc() == *n,
            // Unresolved source-counter MV gate (concretized at resolution
            // via `resolve_source_counters`).
            R::ManaValueEqualsSourceCounters(_) => false,
            R::ManaValueEqualsSacrificedPlus(off) => {
                card.definition.cost.cmc() == self.sacrificed_mana_value.unwrap_or(0) + *off
            }
            R::ManaValueLessThanEventAmount => {
                card.definition.cost.cmc() < self.trigger_event_amount_scratch
            }
            R::HasCardType(ct) => {
                        // CR 715 — an adventuring card is its instant/sorcery
                        // half while on the stack, so report the adventure types.
                        if card.adventuring {
                            card.definition.adventure.as_ref()
                                .map(|a| a.card_types.contains(ct)).unwrap_or(false)
                        } else {
                            card.definition.card_types.contains(ct)
                        }
                    }
            R::Multicolored => card.definition.cost.distinct_colors() >= 2,
            // CR 702.114 — Devoid CDA: colorless despite colored pips.
            R::Colorless => card.definition.keywords.contains(&crate::card::Keyword::Devoid)
                || card.definition.cost.distinct_colors() == 0,
            R::Monocolored => card.definition.cost.distinct_colors() == 1,
            R::HasXInCost => card.definition.cost.has_x(),
            // OtherThanSource is `applies_to`-pipeline-only — see the
            // companion arm in `evaluate_requirement_static`. For
            // library/zone searches we don't filter on this; the
            // candidate set already excludes the source's current zone
            // (a card in a graveyard search can't be the source on the
            // battlefield).
            R::OtherThanSource => true,
            R::InGraveyard => self
                .players
                .iter()
                .any(|p| p.graveyard.iter().any(|c| c.id == card.id)),
            R::InYourGraveyard => self
                .players
                .get(controller)
                .is_some_and(|p| p.graveyard.iter().any(|c| c.id == card.id)),
            R::InExile => self.exile.iter().any(|c| c.id == card.id),
            // Battlefield-only ("greatest MV among controlled" walks the
            // battlefield in the static variant; library searches don't
            // surface this filter).
            R::HasGreatestManaValueAmongControlled(_) => false,
            // Name match works in any zone — used by Grandeur
            // activations that walk a hand for a same-named card.
            R::HasName(name) => card.definition.name == name.as_str(),
            // Resolved to a concrete `HasName` by callers that have the
            // source in hand (RevealUntilFind); vacuously false otherwise.
            R::NamedBySource => false,
            // Count walks the battlefield for the evaluating controller's
            // matching permanents; the candidate's own zone is irrelevant.
            R::ManaValueAtMostControlledCount(inner) => {
                let count = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        self.evaluate_requirement_static(
                            inner,
                            &Target::Permanent(c.id),
                            controller,
                            None,
                        )
                    })
                    .count() as u32;
                card.definition.cost.cmc() <= count
            }
            R::ManaValueAtMostControllerGraveyard => {
                let count = self.players[card.controller].graveyard.len() as u32;
                card.definition.cost.cmc() <= count
            }
            // Back-face / prepare-spell checks are static properties of
            // the card definition — same answer in any zone.
            R::HasBackFace => card.definition.back_face.is_some(),
            R::HasPrepareSpell => card.definition.prepare_spell.is_some(),
            R::HasNoCounters => {
                card.counters.values().all(|&n| n == 0)
                    && card.keyword_counters.values().all(|&n| n == 0)
            }
            // "With different names" — excludes anything sharing a name with
            // a card already moved this resolution (Saheeli Rai -7).
            R::NameDiffersFromLastMoved => !self.last_moved_cards.iter().any(|id| {
                self.find_card_anywhere(*id)
                    .is_some_and(|c| c.definition.name == card.definition.name)
            }),
            // Battlefield-state predicates can't be evaluated for library cards.
            R::Tapped | R::Untapped | R::WithCounter(_)
            | R::IsAttacking | R::IsBlocking | R::IsAttackingAlone | R::IsBlockingAlone
            | R::AttackedThisTurn | R::HasAbilityOnStack
            | R::IsSpellOnStack | R::DealtDamageToControllerThisTurn | R::IsEnchanted
            | R::IsEquipped | R::IsModified => false,
        }
    }
}
