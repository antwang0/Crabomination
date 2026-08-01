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

/// OTJ — a card is an outlaw if it is a creature that's an Assassin, Mercenary,
/// Pirate, Rogue, or Warlock (Changeling satisfies any type).
pub(crate) fn card_is_outlaw(card: &CardInstance) -> bool {
    use crate::card::CreatureType::*;
    card.definition.is_creature()
        && (card.has_keyword(&crate::card::Keyword::Changeling)
            || [Assassin, Mercenary, Pirate, Rogue, Warlock]
                .iter()
                .any(|t| card.definition.subtypes.creature_types.contains(t)))
}

impl GameState {
    /// CR 700.5 — `player`'s devotion to `colors`: the number of mana
    /// symbols matching any listed color among the mana costs of
    /// permanents they control. A hybrid / Phyrexian / mono-hybrid pip
    /// counts once if it contains any of the colors.
    pub fn devotion_to(&self, player: usize, colors: &[crate::mana::Color]) -> i32 {
        let matches = |c: &crate::mana::Color| colors.contains(c);
        let pips = self
            .battlefield
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
            .count() as i32;
        // CR 700.5 — Altar of the Pantheon adds 1 to every non-empty devotion
        // query (to each color and combination).
        let bonus = if colors.is_empty() {
            0
        } else {
            self.battlefield
                .iter()
                .filter(|card| card.controller == player)
                .filter(|card| {
                    card.definition.static_abilities.iter().any(|s| {
                        matches!(s.effect, crate::effect::StaticEffect::DevotionBonus)
                    })
                })
                .count() as i32
        };
        pips + bonus
    }

    pub(crate) fn evaluate_value(&self, v: &Value, ctx: &EffectContext) -> i32 {
        match v {
            Value::HalfLibrarySizeRoundedUp(who) => self
                .resolve_player(who, ctx)
                .map(|p| ((self.players[p].library.len() as i32) + 1) / 2)
                .unwrap_or(0),
            Value::HalfLifeRoundedUp(who) => self
                .resolve_player(who, ctx)
                .map(|p| (self.players[p].life.max(0) + 1) / 2)
                .unwrap_or(0),
            Value::GreatestManaValueInExile => self
                .exile
                .iter()
                .map(|c| c.definition.cost.cmc() as i32)
                .max()
                .unwrap_or(0),
            Value::GreatestManaValueInGraveyard(who) => self
                .resolve_player(who, ctx)
                .map(|p| {
                    self.players[p]
                        .graveyard
                        .iter()
                        .map(|c| c.definition.cost.cmc() as i32)
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0),
            Value::Const(n) => *n,
            Value::CountOf(s) => self.resolve_selector(s, ctx).len() as i32,
            Value::PartyCount => {
                use crate::card::{CreatureType as CT, Keyword};
                let roles = [CT::Cleric, CT::Rogue, CT::Warrior, CT::Wizard];
                // Per creature you control, which of the four roles it can fill
                // (a Changeling fills all — CR 702.73). One creature fills at
                // most one slot, so party size is a max bipartite matching.
                let creatures: Vec<[bool; 4]> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == ctx.controller && c.definition.is_creature())
                    .filter_map(|c| self.computed_permanent(c.id))
                    .map(|cp| {
                        let changeling = cp.keywords.contains(&Keyword::Changeling);
                        std::array::from_fn(|i| {
                            changeling || cp.subtypes.creature_types.contains(&roles[i])
                        })
                    })
                    .collect();
                // Kuhn's algorithm: match each role to a distinct creature.
                fn augment(
                    role: usize,
                    creatures: &[[bool; 4]],
                    seen: &mut [bool],
                    to_role: &mut [Option<usize>],
                ) -> bool {
                    for (ci, has) in creatures.iter().enumerate() {
                        if has[role] && !seen[ci] {
                            seen[ci] = true;
                            if to_role[ci].is_none()
                                || augment(to_role[ci].unwrap(), creatures, seen, to_role)
                            {
                                to_role[ci] = Some(role);
                                return true;
                            }
                        }
                    }
                    false
                }
                let mut to_role: Vec<Option<usize>> = vec![None; creatures.len()];
                (0..4)
                    .filter(|&role| {
                        let mut seen = vec![false; creatures.len()];
                        augment(role, &creatures, &mut seen, &mut to_role)
                    })
                    .count() as i32
            }
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
                .map(|id| self.blocker_count_of(id) as i32)
                .sum(),
            Value::CreaturesBlockedBy(s) => self
                .resolve_selector(s, ctx)
                .iter()
                .filter_map(|e| e.as_permanent_id())
                .map(|id| self.attackers_blocked_by(id).len() as i32)
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
                    // Covers both the trigger's source and its dead subject.
                    if let Some(snap) = self.lki_snapshot(cid) {
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
                    // `as_card_id` (mirrors `PowerOf`): a dies-trigger subject
                    // arrives as `EntityRef::Card` once it's in the graveyard,
                    // so "gain life equal to its toughness" (Proper Burial) can
                    // read the dead creature's last-known toughness (CR 603.10).
                    let cid = e.as_card_id()?;
                    if let Some(snap) = self.lki_snapshot(cid) {
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
            Value::MarkedDamageOn(s) => self
                .resolve_selector(s, ctx)
                .iter()
                .filter_map(|e| {
                    let cid = e.as_card_id()?;
                    // CR 603.10 LKI first (a dies-trigger reads the damage that
                    // was marked when it left), then the live permanent.
                    if let Some(snap) = self.lki_snapshot(cid) {
                        return Some(snap.damage as i32);
                    }
                    self.battlefield_find(cid).map(|c| c.damage as i32)
                })
                .sum(),
            Value::LifeOf(p) => self.resolve_player(p, ctx).map(|p| self.players[p].life).unwrap_or(0),
            Value::PlayerSpeed(p) => self.resolve_player(p, ctx).map(|p| self.players[p].speed as i32).unwrap_or(0),
            Value::HandSizeOf(p) => self.resolve_player(p, ctx).map(|p| self.players[p].hand.len() as i32).unwrap_or(0),
            Value::OpponentsWithHandSizeAtMost(n) => {
                let me = ctx.controller;
                let teammates = self.teammates(me);
                self.players
                    .iter()
                    .enumerate()
                    .filter(|(i, pl)| {
                        *i != me && !teammates.contains(i) && pl.hand.len() <= *n as usize
                    })
                    .count() as i32
            }
            Value::LifeGainedThisTurn(p) => self.resolve_player(p, ctx).map(|p| self.players[p].life_gained_this_turn as i32).unwrap_or(0),
            // Max over the resolved set, so `EachOpponent` reads "the most
            // life any opponent lost this turn" (Spinerock Knoll).
            Value::LifeLostThisTurn(p) => self
                .resolve_players(p, ctx)
                .iter()
                .map(|&p| self.players[p].life_lost_this_turn as i32)
                .max()
                .unwrap_or(0),
            // Summed across the resolved set: "damage dealt to your opponents
            // this turn" counts every opponent's total (Petrified Wood-Kin).
            Value::DamageTakenThisTurn(p) => self
                .resolve_players(p, ctx)
                .iter()
                .map(|&p| self.players[p].damage_taken_this_turn as i32)
                .sum(),
            Value::DamageDealtToSourceThisTurn => ctx
                .source
                .and_then(|id| {
                    self.battlefield_find(id).or_else(|| self.leaves_bf_lki.get(&id))
                })
                .map(|c| c.damage_dealt_to_this_turn as i32)
                .unwrap_or(0),
            Value::CardsInExileOwnedBy(p) => self
                .resolve_players(p, ctx)
                .iter()
                .map(|&seat| self.exile.iter().filter(|c| c.owner == seat).count() as i32)
                .sum(),
            Value::CreaturesAttackedWithThisTurn(p) => self
                .resolve_players(p, ctx)
                .iter()
                .map(|&p| self.players[p].creatures_attacked_this_turn as i32)
                .max()
                .unwrap_or(0),
            Value::NoncreatureSpellsCastThisTurn(p) => self
                .resolve_players(p, ctx)
                .iter()
                .map(|&p| self.players[p].noncreature_spells_cast_this_game_turn as i32)
                .max()
                .unwrap_or(0),
            Value::SpellsCastThisTurn(p) => self
                .resolve_players(p, ctx)
                .iter()
                .map(|&p| self.players[p].spells_cast_this_turn as i32)
                .max()
                .unwrap_or(0),
            Value::SpellsCastThisTurnTotal => {
                self.players.iter().map(|p| p.spells_cast_this_turn as i32).sum()
            }
            Value::OtherSpellsCastThisTurn(p) => self
                .resolve_players(p, ctx)
                .iter()
                .map(|&p| (self.players[p].spells_cast_this_turn as i32 - 1).max(0))
                .max()
                .unwrap_or(0),
            Value::OpponentsAttackedThisCombat => {
                use crate::game::types::AttackTarget;
                let mut seats = std::collections::HashSet::new();
                for atk in &self.attacking {
                    let defender = match atk.target {
                        AttackTarget::Player(p) => Some(p),
                        AttackTarget::Planeswalker(id) => {
                            self.battlefield_find(id).map(|c| c.controller)
                        }
                        AttackTarget::Battle(id) => {
                            self.battlefield_find(id).and_then(|c| c.protected_by)
                        }
                    };
                    if let Some(seat) = defender {
                        seats.insert(seat);
                    }
                }
                seats.len() as i32
            }
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
            Value::DifferentlyNamedLandsControlled => {
                let mut names: Vec<String> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == ctx.controller && c.definition.is_land())
                    .map(|c| c.definition.name.to_string())
                    .collect();
                names.sort_unstable();
                names.dedup();
                names.len() as i32
            }
            Value::DistinctlyNamedGatesControlled => {
                let mut names: Vec<String> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == ctx.controller
                            && c.definition.has_land_type(crate::card::LandType::Gate)
                    })
                    .map(|c| c.definition.name.to_string())
                    .collect();
                names.sort_unstable();
                names.dedup();
                names.len() as i32
            }
            Value::DifferentlyNamedCreatureTokensControlled => {
                let mut names: Vec<String> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == ctx.controller
                            && c.is_token
                            && c.definition.is_creature()
                    })
                    .map(|c| c.definition.name.to_string())
                    .collect();
                names.sort_unstable();
                names.dedup();
                names.len() as i32
            }
            Value::OozesInExileAndGraveyard => {
                let p = ctx.controller;
                let is_ooze = |c: &crate::card::CardInstance| {
                    c.owner == p
                        && (c.definition.name == "Slime Against Humanity"
                            || c.definition
                                .subtypes
                                .creature_types
                                .contains(&crate::card::CreatureType::Ooze))
                };
                let gy = self.players[p].graveyard.iter().filter(|c| is_ooze(c)).count();
                let ex = self.exile.iter().filter(|c| is_ooze(c)).count();
                (gy + ex) as i32
            }
            Value::TotalToughnessControlled => {
                let ids: Vec<_> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == ctx.controller && c.definition.is_creature())
                    .map(|c| c.id)
                    .collect();
                ids.iter()
                    .filter_map(|id| self.computed_permanent(*id))
                    .map(|cp| cp.toughness.max(0))
                    .sum()
            }
            Value::TotalPowerControlled => {
                let ids: Vec<_> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == ctx.controller && c.definition.is_creature())
                    .map(|c| c.id)
                    .collect();
                ids.iter()
                    .filter_map(|id| self.computed_permanent(*id))
                    .map(|cp| cp.power.max(0))
                    .sum()
            }
            Value::SourceCrewerCount => ctx
                .source
                .and_then(|s| self.battlefield_find(s))
                .map(|c| c.crewed_by.len() as i32)
                .unwrap_or(0),
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
            Value::DungeonsCompleted => {
                self.players[ctx.controller].dungeons_completed as i32
            }
            Value::ControllerExperience => self.players[ctx.controller].experience as i32,
            Value::MutateCount => ctx
                .trigger_source
                .and_then(|e| e.as_card_id())
                .or(ctx.source)
                .and_then(|id| self.battlefield_find(id))
                .map(|c| c.mutate_stack.len().saturating_sub(1) as i32)
                .unwrap_or(0),
            Value::DevotionTo(colors) => self.devotion_to(ctx.controller, colors),
            Value::LargestCreatureTypeCount => {
                // Greatest number of the controller's creatures sharing a
                // creature type; changelings count for every type.
                use std::collections::HashMap;
                let mut counts: HashMap<crate::card::CreatureType, i32> = HashMap::new();
                let mut changelings = 0i32;
                for c in self.battlefield.iter().filter(|c| c.controller == ctx.controller) {
                    let Some(cp) = self.computed_permanent(c.id) else { continue };
                    if !cp.card_types.contains(&crate::card::CardType::Creature) {
                        continue;
                    }
                    if cp.keywords.contains(&crate::card::Keyword::Changeling) {
                        changelings += 1;
                        continue;
                    }
                    for t in &cp.subtypes.creature_types {
                        *counts.entry(*t).or_default() += 1;
                    }
                }
                counts.values().max().copied().unwrap_or(0) + changelings
            }
            Value::AurasYouControlledOnDyingSubject => ctx
                .trigger_source
                .and_then(|e| e.as_card_id())
                .and_then(|host| self.auras_at_death.get(&host))
                .map(|auras| auras.iter().filter(|(_, c)| *c == ctx.controller).count() as i32)
                .unwrap_or(0),
            Value::CountersOn { what, kind } => self
                .resolve_selector(what, ctx)
                .into_iter()
                .filter_map(|e| {
                    let cid = match e {
                        EntityRef::Permanent(c) | EntityRef::Card(c) => c,
                        _ => return None,
                    };
                    // CR 122.2 strips counters on zone change, so a
                    // die-trigger that reads "its +1/+1 counters"
                    // (Ambitious Augmenter's transfer) consults the
                    // leaves-battlefield LKI snapshot (CR 603.10); the
                    // dispatch-time `died_card_snapshots` cache covers
                    // filter evaluation before the trigger resolves.
                    self.battlefield_find(cid)
                        // Covers both the resolving trigger's source and its
                        // dead subject (Sporogenesis' "for each fungus counter
                        // on that creature").
                        .or_else(|| self.lki_snapshot(cid))
                        .or_else(|| self.died_card_snapshots.get(&cid))
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
            Value::TotalCountersOn { what } => self
                .resolve_selector(what, ctx)
                .into_iter()
                .filter_map(|e| {
                    let cid = match e {
                        EntityRef::Permanent(c) | EntityRef::Card(c) => c,
                        _ => return None,
                    };
                    // Same LKI fallback chain as `CountersOn` so a source
                    // sacrificed as a cost (Twitching Doll) reads its last
                    // counter total (CR 603.10 / 608.2).
                    self.battlefield_find(cid)
                        .or_else(|| {
                            self.resolving_lki_source
                                .filter(|s| *s == cid)
                                .and_then(|_| self.leaves_bf_lki.get(&cid))
                        })
                        .or_else(|| self.died_card_snapshots.get(&cid))
                        .map(|inst| inst.counters.values().sum::<u32>() as i32)
                })
                .sum(),
            Value::ExcessDamageDealtThisResolution => self.excess_damage_this_resolution as i32,
            Value::DamageDealtThisResolution => self.damage_dealt_this_resolution as i32,
            Value::FaceDownCreatures => self
                .battlefield
                .iter()
                .filter(|c| c.face_down && c.definition.is_creature())
                .count() as i32,
            Value::CounteredSpellManaSpent => self.countered_spell_mana_spent as i32,
            Value::CounteredSpellManaValue => self.countered_spell_mana_value as i32,
            Value::ChosenNumber => self.chosen_number_this_resolution as i32,
            Value::ChosenNumberOfSource => ctx
                .source
                .and_then(|s| self.battlefield_find(s))
                .and_then(|c| c.chosen_number)
                .unwrap_or(0) as i32,
            Value::NonlandCardsExiledThisEffect => self.nonland_cards_exiled_this_effect as i32,
            Value::Sum(vs) => vs.iter().map(|v| self.evaluate_value(v, ctx)).sum(),
            Value::Diff(a, b) => self.evaluate_value(a, ctx) - self.evaluate_value(b, ctx),
            Value::Times(a, b) => self.evaluate_value(a, ctx) * self.evaluate_value(b, ctx),
            Value::Min(a, b) => self.evaluate_value(a, ctx).min(self.evaluate_value(b, ctx)),
            Value::Max(a, b) => self.evaluate_value(a, ctx).max(self.evaluate_value(b, ctx)),
            Value::NonNeg(v) => self.evaluate_value(v, ctx).max(0),
            Value::HalvedRoundUp(v) => (self.evaluate_value(v, ctx).max(0) + 1) / 2,
            Value::HalvedRoundDown(v) => self.evaluate_value(v, ctx).max(0) / 2,
            Value::IfAtLeast { value, threshold, then, else_ } => {
                if self.evaluate_value(value, ctx) >= *threshold {
                    self.evaluate_value(then, ctx)
                } else {
                    self.evaluate_value(else_, ctx)
                }
            }
            Value::SacrificedPower => self.sacrificed_power.unwrap_or(0),
            Value::RevealedForCostPower => self.revealed_for_cost_power.unwrap_or(0),
            Value::GreatestManaValueAmongPermanents(who) => self
                .resolve_player(who, ctx)
                .map(|p| {
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == p)
                        .map(|c| c.definition.cost.cmc() as i32)
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0),
            Value::SacrificedTotalPower => self.sacrificed_total_power,
            Value::SacrificedCount => self.sacrificed_count as i32,
            Value::TappedForCostPower => self.tapped_for_cost_power.unwrap_or(0),
            Value::SacrificedToughness => self.sacrificed_toughness.unwrap_or(0),
            Value::SacrificedManaValue => self.sacrificed_mana_value.unwrap_or(0) as i32,
            Value::CardsDiscardedThisEffect => self.cards_discarded_this_resolution as i32,
            Value::EnergyPaidThisEffect => self.energy_paid_this_resolution as i32,
            Value::PermanentsReturnedThisEffect => self.permanents_returned_this_resolution as i32,
            Value::PermanentsTappedThisEffect => self.permanents_tapped_this_resolution as i32,
            Value::CardsRevealedThisEffect => self.cards_revealed_this_resolution as i32,
            Value::LastExiledManaValue => self
                .exiled_card_ids_this_resolution
                .last()
                .and_then(|id| self.find_card_anywhere(*id))
                .map(|c| c.definition.cost.cmc() as i32)
                .unwrap_or(0),
            Value::MaxCardsDiscardedThisEffectByAnyPlayer => self
                .cards_discarded_per_player_this_resolution
                .values()
                .copied()
                .max()
                .unwrap_or(0) as i32,
            Value::CreatureCardsDiscardedThisEffect => {
                self.creature_cards_discarded_this_resolution as i32
            }
            Value::GreatestDiscardedManaValueThisEffect => {
                self.greatest_discarded_mv_this_resolution as i32
            }
            Value::CreatureCardsMilledThisEffect => self
                .last_moved_cards
                .iter()
                .filter(|&&cid| {
                    self.players.iter().any(|p| {
                        p.graveyard
                            .iter()
                            .any(|c| c.id == cid && c.definition.is_creature())
                    })
                })
                .count() as i32,
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
            Value::DistinctManaValuesAmongControlledNonland => {
                let p = ctx.controller;
                let mut mvs: Vec<u32> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == p && !c.definition.is_land())
                    .map(|c| c.definition.cost.cmc())
                    .collect();
                mvs.sort_unstable();
                mvs.dedup();
                mvs.len() as i32
            }
            Value::DistinctManaValuesInGraveyard(who) => {
                let Some(p) = self.resolve_player(who, ctx) else { return 0 };
                let mut mvs: Vec<u32> = self.players[p]
                    .graveyard
                    .iter()
                    .map(|c| c.definition.cost.cmc())
                    .collect();
                mvs.sort_unstable();
                mvs.dedup();
                mvs.len() as i32
            }
            Value::GreatestPowerControlledAndGraveyard => {
                let p = ctx.controller;
                let bf = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == p && c.definition.is_creature())
                    .filter_map(|c| self.computed_permanent(c.id).map(|cp| cp.power));
                let gy = self.players[p]
                    .graveyard
                    .iter()
                    .filter(|c| c.definition.is_creature())
                    .map(|c| c.definition.power);
                bf.chain(gy).max().unwrap_or(0)
            }
            Value::PermanentsDestroyedThisResolution => {
                self.permanents_destroyed_this_resolution as i32
            }
            Value::ConvergedValue => ctx.converged_value as i32,
            Value::CardTypesInGraveyard(who) => self
                .resolve_player(who, ctx)
                .map(|p| self.distinct_card_types_in_graveyard(p) as i32)
                .unwrap_or(0),
            Value::CardTypesInAllGraveyards => {
                self.distinct_card_types_in_all_graveyards() as i32
            }
            Value::LastDiscardedCardTypes => self.last_discarded_card_types as i32,
            Value::LastDiscardedManaValue => self
                .last_discarded_mana_value
                .or(self.cost_discarded_mana_value)
                .unwrap_or(0) as i32,
            Value::CardsDiscardedThisTurn(who) => self
                .resolve_players(who, ctx)
                .into_iter()
                .map(|p| self.players[p].cards_discarded_this_turn as i32)
                .max()
                .unwrap_or(0),
            Value::SquadCount => ctx
                .source
                .and_then(|s| self.battlefield_find(s))
                .map(|c| c.squad_count as i32)
                .unwrap_or(0),
            // A permanent reads its own stamped count; a resolving spell
            // (Spell Contortion) reads the count threaded onto the context.
            Value::TimesKicked => ctx
                .source
                .and_then(|s| self.battlefield_find(s))
                .map(|c| c.kick_count as i32)
                .unwrap_or(ctx.kick_count as i32),
            Value::CastSpellTimesKicked => ctx
                .trigger_source
                .and_then(|e| e.as_card_id())
                .and_then(|cid| {
                    self.stack.iter().find_map(|si| match si {
                        StackItem::Spell { card, .. } if card.id == cid => {
                            Some(card.kick_count as i32)
                        }
                        _ => None,
                    })
                })
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
                if ctx.mana_spent > 0 {
                    return ctx.mana_spent as i32;
                }
                // ETB rider reading the cost after the spell left the stack:
                // the value was stamped onto the entered permanent.
                ctx.source
                    .and_then(|s| self.battlefield_find(s))
                    .map(|c| c.cast_mana_spent as i32)
                    .unwrap_or(ctx.mana_spent as i32)
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
            Value::TotalManaValueOf(s) => self
                .resolve_selector(s, ctx)
                .into_iter()
                .filter_map(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => {
                        self.battlefield_find(cid).map(|c| c.definition.cost.cmc() as i32)
                    }
                    EntityRef::Player(_) => None,
                })
                .sum(),
            Value::HighestManaValueAmong(s) => self
                .resolve_selector(s, ctx)
                .into_iter()
                .filter_map(|e| match e {
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => {
                        self.battlefield_find(cid).map(|c| c.definition.cost.cmc() as i32)
                    }
                    EntityRef::Player(_) => None,
                })
                .max()
                .unwrap_or(0),
            Value::ColorCountOf(s) => self
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
                            })
                        })
                        .or_else(|| self.exile.iter().find(|c| c.id == cid))
                        .map(|c| c.definition.printed_colors().len() as i32),
                    EntityRef::Player(_) => None,
                })
                .unwrap_or(0),
            Value::DistinctColorsAmong(s) => {
                let mut seen: std::collections::HashSet<crate::mana::Color> =
                    std::collections::HashSet::new();
                for ent in self.resolve_selector(s, ctx) {
                    if let Some(cid) = ent.as_permanent_id()
                        && let Some(c) = self.battlefield_find(cid)
                    {
                        seen.extend(c.definition.printed_colors());
                    }
                }
                seen.len() as i32
            }
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
            Value::CardsInGraveyardMatching { who, filter } => {
                let Some(p) = self.resolve_player(who, ctx) else { return 0; };
                let ids: Vec<CardId> = self.players[p].graveyard.iter().map(|c| c.id).collect();
                ids.into_iter()
                    .filter(|id| {
                        self.evaluate_requirement_static(
                            filter,
                            &crate::game::Target::Permanent(*id),
                            ctx.controller,
                            ctx.source,
                        )
                    })
                    .count() as i32
            }
            Value::CardsInHandMatching { who, filter } => {
                let Some(p) = self.resolve_player(who, ctx) else { return 0; };
                let ids: Vec<CardId> = self.players[p].hand.iter().map(|c| c.id).collect();
                ids.into_iter()
                    .filter(|id| {
                        self.evaluate_requirement_static(
                            filter,
                            &crate::game::Target::Permanent(*id),
                            ctx.controller,
                            ctx.source,
                        )
                    })
                    .count() as i32
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
            Value::CardsExiledWithSourceCount => {
                let Some(src) = ctx.source else { return 0; };
                self.exile.iter().filter(|c| c.exiled_with == Some(src)).count() as i32
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
            Value::ArtifactsEnteredThisTurn(p) => self
                .resolve_player(p, ctx)
                .map(|p| self.players[p].artifacts_entered_this_turn as i32)
                .unwrap_or(0),
            Value::MountsVehiclesEnteredThisTurn(p) => self
                .resolve_player(p, ctx)
                .map(|p| self.players[p].mounts_vehicles_entered_this_turn as i32)
                .unwrap_or(0),
            // Geralf, the Fleshwright — count this turn's arrivals of a type,
            // excluding the trigger's own subject. The entrant may have left,
            // so look it up anywhere rather than on the battlefield.
            Value::OtherCreaturesOfTypeEnteredThisTurn(ct) => {
                let subject = ctx.trigger_source.and_then(|e| e.as_permanent_id());
                self.players[ctx.controller]
                    .creatures_entered_this_turn
                    .iter()
                    .filter(|id| Some(**id) != subject)
                    .filter(|id| {
                        self.find_card_anywhere(**id).is_some_and(|c| {
                            c.definition.subtypes.creature_types.contains(ct)
                                || c.definition.keywords.contains(&crate::card::Keyword::Changeling)
                        })
                    })
                    .count() as i32
            }
            // Selvala, Eager Trailblazer — distinct computed powers.
            Value::DistinctPowersAmongCreaturesControlled(p) => {
                let Some(seat) = self.resolve_player(p, ctx) else { return 0 };
                let powers: std::collections::HashSet<i32> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == seat && c.definition.is_creature())
                    .filter_map(|c| self.computed_permanent(c.id).map(|cp| cp.power))
                    .collect();
                powers.len() as i32
            }
            Value::PoisonCountersOf(p) => self
                .resolve_player(p, ctx)
                .map(|p| self.players[p].poison_counters as i32)
                .unwrap_or(0),
            Value::MulticoloredSpellsCastThisTurn(p) => self
                .resolve_player(p, ctx)
                .map(|p| self.players[p].multicolored_spells_cast_this_turn as i32)
                .unwrap_or(0),
            Value::GreatestToxicAmongControlled(p) => {
                let Some(p) = self.resolve_player(p, ctx) else { return 0 };
                self.compute_battlefield()
                    .iter()
                    .filter(|c| c.controller == p)
                    .map(|c| {
                        c.keywords
                            .iter()
                            .map(|k| match k {
                                crate::card::Keyword::Toxic(n) => *n as i32,
                                _ => 0,
                            })
                            .sum::<i32>()
                    })
                    .max()
                    .unwrap_or(0)
            }
            Value::CreaturesDiedThisTurn(p) => self
                .resolve_player(p, ctx)
                .map(|p| self.players[p].creatures_died_this_turn as i32)
                .unwrap_or(0),
            Value::PermanentsSacrificedThisTurn(p) => self
                .resolve_player(p, ctx)
                .map(|p| self.players[p].permanents_sacrificed_this_turn as i32)
                .unwrap_or(0),
            Value::CreaturesDiedThisTurnTotal => self
                .players
                .iter()
                .map(|p| p.creatures_died_this_turn as i32)
                .sum(),
            Value::ControllerCreaturesDiedThisTurn => {
                self.players[ctx.controller].creatures_died_this_turn as i32
            }
            Value::ZuberasDiedThisTurnTotal => self
                .players
                .iter()
                .map(|p| p.zuberas_died_this_turn as i32)
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
            Value::StartingLifeTotal => self.players[ctx.controller].starting_life,
            Value::HalfDown(inner) => self.evaluate_value(inner, ctx) / 2,
            Value::DivDown(inner, by) => {
                if *by == 0 { 0 } else { self.evaluate_value(inner, ctx) / *by as i32 }
            }
            Value::PermanentCountControlledBy(p) => self
                .resolve_player(p, ctx)
                .map(|seat| {
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == seat)
                        .count() as i32
                })
                .unwrap_or(0),
            Value::PlayerCount => self.alive_count() as i32,
            Value::CreatureCountControlledBy(p) => self
                .resolve_player(p, ctx)
                .map(|seat| {
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == seat && c.definition.is_creature())
                        .count() as i32
                })
                .unwrap_or(0),
            Value::GreatestSharedCreatureTypeCount => {
                // Tally each creature type across the controller's creatures
                // (changelings count for every type) and take the largest.
                use crate::card::CreatureType;
                let mut tally: std::collections::HashMap<CreatureType, i32> =
                    std::collections::HashMap::new();
                let mut changelings = 0i32;
                for cp in self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == ctx.controller)
                    .filter_map(|c| self.computed_permanent(c.id))
                    .filter(|cp| cp.card_types.contains(&crate::card::CardType::Creature))
                {
                    if cp.keywords.contains(&crate::card::Keyword::Changeling) {
                        changelings += 1;
                        continue;
                    }
                    for t in &cp.subtypes.creature_types {
                        *tally.entry(*t).or_insert(0) += 1;
                    }
                }
                tally.values().copied().max().unwrap_or(0) + changelings
            }
            Value::TimesDescendedThisTurn => self
                .players
                .get(ctx.controller)
                .map(|pl| pl.descend_count_this_turn as i32)
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

    pub fn evaluate_predicate(&self, p: &Predicate, ctx: &EffectContext) -> bool {
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
            Predicate::ValueIsPrime(v) => {
                let n = self.evaluate_value(v, ctx) as i64;
                n >= 2 && (2..=((n as f64).sqrt() as i64)).all(|d| n % d != 0)
            }
            Predicate::PlayerSacrificedThisResolution(pref) => self
                .resolve_player(pref, ctx)
                .is_some_and(|p| self.players_sacrificed_this_resolution.contains(&p)),
            Predicate::ExcessDamageDealtThisResolution => self.excess_damage_this_resolution > 0,
            Predicate::IsTurnOf(pref) => self.resolve_player(pref, ctx) == Some(self.active_player_idx),
            Predicate::YourMainPhase => {
                self.step.is_main_phase() && self.active_player_idx == ctx.controller
            }
            Predicate::ActivePlayerControls(sel) => self
                .resolve_selector(sel, ctx)
                .into_iter()
                .filter_map(|e| e.as_permanent_id())
                .any(|cid| {
                    self.battlefield_find(cid).map(|c| c.controller) == Some(self.active_player_idx)
                }),
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
            Predicate::EntityMatchesAny { what, filter } => self
                .resolve_selector(what, ctx)
                .into_iter()
                .any(|e| match e {
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
            Predicate::FirstLifeGainThisTurn { who } => self
                .resolve_player(who, ctx)
                .map(|p| !self.players[p].gained_life_earlier_this_turn)
                .unwrap_or(false),
            Predicate::ChoseModesAtLeast(n) => ctx.spree_modes.len() >= *n as usize,
            Predicate::CastOwnNameThisGameAtLeast(n) => ctx
                .source
                .and_then(|s| self.find_card_anywhere(s))
                .map(|c| c.definition.name)
                .or(ctx.source_name)
                .is_some_and(|name| {
                    self.players[ctx.controller]
                        .spells_cast_by_name_this_game
                        .get(name)
                        .copied()
                        .unwrap_or(0)
                        >= *n
                }),
            Predicate::ExpendReached(n) => {
                // CR 700.14 — fired only on the cost-payment that pushes the
                // turn's spell-mana total from below `n` up to at least `n`.
                self.expend_prev_total < *n && ctx.event_amount >= *n
            }
            Predicate::DiscardedThisTurn { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| !self.players[p].discarded_this_turn.is_empty()),
            Predicate::HasCityBlessing { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].city_blessing),
            Predicate::IsDay => self.day_night == Some(crate::game::types::DayNight::Day),
            Predicate::IsNight => self.day_night == Some(crate::game::types::DayNight::Night),
            Predicate::DieResultAtLeast(n) => ctx.event_amount >= *n as u32,
            Predicate::IsMonarch { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.monarch == Some(p)),
            Predicate::SpeedAtLeast { who, speed } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].speed >= *speed),
            Predicate::PlayerDamagedThisTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].was_dealt_damage_this_turn),
            Predicate::PlayerLostLifeThisTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].lost_life_this_turn),
            Predicate::PlayerGainedLifeThisTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].life_gained_this_turn > 0),
            Predicate::PlayerDrewAtLeastThisTurn { who, n } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].cards_drawn_this_turn >= *n),
            Predicate::PlayerLifeAtMost { who, life } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.effective_life(p) <= *life),
            Predicate::PlayerLifeExactly { who, life } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.effective_life(p) == *life),
            Predicate::PlayerLifeAtLeast { who, life } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.effective_life(p) >= *life),
            Predicate::PlayerLifeAtLeastAboveStarting { who, delta } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.effective_life(p) >= self.players[p].starting_life + *delta),
            Predicate::PlayerLifeAtMostHalfStarting { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.effective_life(p) <= self.players[p].starting_life / 2),
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
            Predicate::PlayerControlsMostOf { who, filter } => {
                let count = |p: usize| {
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == p)
                        .filter(|c| {
                            self.evaluate_requirement_static(
                                filter,
                                &Target::Permanent(c.id),
                                p,
                                ctx.source,
                            )
                        })
                        .count()
                };
                self.resolve_players(who, ctx).into_iter().any(|p| {
                    let mine = count(p);
                    (0..self.players.len())
                        .filter(|&q| q != p && !self.players[q].eliminated)
                        .all(|q| count(q) < mine)
                })
            }
            Predicate::PlayerHasLessLifeThanOpponent { who } => {
                self.resolve_players(who, ctx).into_iter().any(|p| {
                    let my_life = self.effective_life(p);
                    (0..self.players.len()).any(|o| {
                        o != p && !self.players[o].eliminated && self.effective_life(o) > my_life
                    })
                })
            }
            Predicate::TriggerSourceIsSourcesChosenPermanent => {
                let stamped = ctx
                    .source
                    .and_then(|s| {
                        self.battlefield_find(s)
                            .or_else(|| self.died_card_snapshots.get(&s))
                            .or_else(|| self.leaves_bf_lki.get(&s))
                    })
                    .and_then(|c| c.chosen_permanent);
                match (stamped, ctx.trigger_source) {
                    (Some(want), Some(e)) => e.as_card_id() == Some(want),
                    _ => false,
                }
            }
            Predicate::SourceAttackedThisTurn => ctx
                .source
                .and_then(|cid| self.battlefield.iter().find(|c| c.id == cid))
                .map(|c| c.attacked_this_turn)
                .unwrap_or(false),
            Predicate::IsExtraTurn => self.current_turn_is_extra,
            Predicate::SourceIsMonstrous => ctx
                .source
                .and_then(|cid| self.battlefield.iter().find(|c| c.id == cid))
                .map(|c| c.monstrous)
                .unwrap_or(false),
            Predicate::SourceIsRenowned => ctx
                .source
                .and_then(|cid| self.battlefield.iter().find(|c| c.id == cid))
                .map(|c| c.renowned)
                .unwrap_or(false),
            Predicate::SourceIsEquipped => {
                ctx.source.is_some_and(|cid| self.attached_equipment_count(cid) > 0)
            }
            Predicate::SourceIsSuspected => ctx
                .source
                .and_then(|cid| self.battlefield.iter().find(|c| c.id == cid))
                .map(|c| c.suspected)
                .unwrap_or(false),
            Predicate::SourceIsBestowedAura => ctx
                .source
                .and_then(|cid| self.battlefield.iter().find(|c| c.id == cid))
                .map(|c| c.bestowed)
                .unwrap_or(false),
            Predicate::SourceOnBattlefield => {
                ctx.source.is_some_and(|cid| self.battlefield_find(cid).is_some())
            }
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
            Predicate::SourceWasCast => ctx
                .source
                .and_then(|cid| self.battlefield.iter().find(|c| c.id == cid))
                .map(|c| {
                    !c.is_token
                        && (c.cast_from_hand
                            || c.cast_from_exile
                            || c.cast_via_flashback
                            || c.cast_from_suspend
                            || c.cast_from_escape)
                })
                .unwrap_or(false),
            Predicate::SourceChampionedSomething => ctx.source.is_some_and(|cid| {
                self.exile.iter().any(|c| c.exiled_by.as_ref().is_some_and(|l| l.source == cid))
            }),
            Predicate::TriggerBlocksSource => match (ctx.trigger_source, ctx.source) {
                (Some(EntityRef::Permanent(blocker)), Some(src)) => {
                    self.blocks(blocker, src)
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
            Predicate::TriggerObjectIsChosenType => {
                let chosen = ctx
                    .source
                    .and_then(|cid| self.find_card_anywhere(cid))
                    .and_then(|c| c.chosen_creature_type);
                let obj = ctx
                    .trigger_source
                    .and_then(|e| e.as_card_id())
                    .and_then(|id| self.find_card_anywhere(id));
                match (chosen, obj) {
                    (Some(ct), Some(card)) => {
                        card.has_keyword(&crate::card::Keyword::Changeling)
                            || card.definition.subtypes.creature_types.contains(&ct)
                    }
                    _ => false,
                }
            }
            Predicate::PlayerAttackedThisTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].attacked_this_turn),
            Predicate::EnergyPaidThisTurnAtLeast { who, n } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].energy_spent_this_turn >= *n),
            Predicate::DealtCombatDamageToPlayerThisTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].dealt_combat_damage_to_player_this_turn),
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
            Predicate::CastSpellThisTurnWith { who, colors, types } => {
                self.resolve_players(who, ctx).into_iter().any(|p| {
                    self.players[p].spell_casts_this_turn.iter().any(|c| {
                        (colors.is_empty() || colors.iter().any(|k| c.colors.contains(k)))
                            && (types.is_empty()
                                || types.iter().any(|t| c.card_types.contains(t)))
                    })
                })
            }
            Predicate::DamagedByCreaturesThisTurnAtLeast { who, at_least } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| {
                    self.players[p].creatures_that_damaged_me_this_turn.len() >= *at_least as usize
                }),
            Predicate::LandsEnteredThisTurnAtLeast { who, at_least } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].lands_entered_this_turn >= *at_least),
            Predicate::CreatureSpellCounteredByOpponentThisTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].creature_spell_countered_by_opponent_this_turn),
            Predicate::NoncreaturePermanentDestroyedByOpponentThisTurn { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.players[p].noncreature_destroyed_by_opponent_this_turn),
            Predicate::CreatureEnteredThisTurnMatching { who, filter } => {
                self.resolve_players(who, ctx).into_iter().any(|p| {
                    self.players[p].creatures_entered_this_turn.iter().any(|&cid| {
                        self.evaluate_requirement_static(
                            filter,
                            &crate::game::types::Target::Permanent(cid),
                            p,
                            ctx.source,
                        )
                    })
                })
            }
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
            Predicate::LastDiscardedHasCreatureType(ct) => {
                self.last_discarded_creature_types.contains(ct)
            }
            Predicate::DiscardedThisEffect { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| {
                    self.cards_discarded_per_player_this_resolution
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
            Predicate::NoSpellCastFromHandThisTurn { who } => self
                .resolve_player(who, ctx)
                .map(|p| self.players[p].spells_cast_from_hand_this_turn == 0)
                .unwrap_or(false),
            Predicate::FirstNoncreatureSpellThisTurn => self.noncreature_spells_cast_this_turn == 1,
            Predicate::NoSpellsCastLastTurn => self.spells_cast_last_turn == 0,
            Predicate::TwoOrMoreSpellsCastLastTurn => self.spells_cast_last_turn >= 2,
            Predicate::CreaturesDiedThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].creatures_died_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::DistinctCounterKindsAmongCreaturesAtLeast { who, at_least } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                let mut kinds = std::collections::HashSet::new();
                for c in self.battlefield.iter().filter(|c| {
                    c.controller == p && c.definition.card_types.contains(&crate::card::CardType::Creature)
                }) {
                    for (kind, n) in &c.counters {
                        if *n > 0 {
                            kinds.insert(*kind);
                        }
                    }
                }
                kinds.len() as u32 >= *at_least
            }
            Predicate::PermanentsSacrificedThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].permanents_sacrificed_this_turn >= n)
                    .unwrap_or(false)
            }
            Predicate::SacrificedArtifactThisTurn { who } => self
                .resolve_player(who, ctx)
                .map(|p| self.players[p].artifacts_sacrificed_this_turn > 0)
                .unwrap_or(false),
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
                // `EachPlayer` = the player-agnostic printed wording ("if
                // one or more cards were put into exile this turn" — Ennis,
                // Debate Moderator): sum the per-player tallies.
                if matches!(who, crate::effect::PlayerRef::EachPlayer) {
                    let total: u32 =
                        self.players.iter().map(|p| p.cards_exiled_this_turn).sum();
                    total >= n
                } else {
                    self.resolve_player(who, ctx)
                        .map(|p| self.players[p].cards_exiled_this_turn >= n)
                        .unwrap_or(false)
                }
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
            Predicate::NoncreatureSpellsCastThisTurnAtLeast { who, at_least } => {
                let n = self.evaluate_value(at_least, ctx).max(0) as u32;
                self.resolve_player(who, ctx)
                    .map(|p| self.players[p].noncreature_spells_cast_this_game_turn >= n)
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
            Predicate::CastSpellIsAdventure => {
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cid => card.adventuring,
                    _ => false,
                })
            }
            // Thought Prison — the cast spell shares a colour or mana value
            // with the imprinted card.
            Predicate::CastSharesColorOrManaValueWithExiledBySource => {
                let Some(src) = ctx.source else { return false };
                let Some(imprint) = self.exile.iter().find(|c| c.exiled_with == Some(src)) else {
                    return false;
                };
                let (colors, mv) =
                    (imprint.definition.printed_colors(), imprint.definition.cost.cmc());
                let cid = match ctx.trigger_source {
                    Some(EntityRef::Card(c)) | Some(EntityRef::Permanent(c)) => c,
                    _ => return false,
                };
                self.find_card_anywhere(cid).is_some_and(|trig| {
                    trig.definition.cost.cmc() == mv
                        || trig.definition.printed_colors().iter().any(|c| colors.contains(c))
                })
            }
            // Soul Foundry — "{X}, {T}: … X is the mana value of that card."
            Predicate::ExiledWithSourceManaValueIsX => {
                let Some(src) = ctx.source else { return false };
                self.exile
                    .iter()
                    .find(|c| c.exiled_with == Some(src))
                    .is_some_and(|c| c.definition.cost.cmc() == ctx.x_value)
            }
            Predicate::SharesCardTypeWithExiledBySource => {
                let Some(src) = ctx.source else { return false };
                // Card types of whatever this source exiled (CR — the
                // "exiled card"). No exiled card → the clause is false.
                let exiled_types: Vec<CardType> = self
                    .exile
                    .iter()
                    .filter(|c| c.exiled_with == Some(src))
                    .flat_map(|c| c.definition.card_types.clone())
                    .collect();
                if exiled_types.is_empty() {
                    return false;
                }
                // The triggering card (cast spell on the stack, or the played
                // land now on the battlefield).
                let cid = match ctx.trigger_source {
                    Some(EntityRef::Card(c)) | Some(EntityRef::Permanent(c)) => c,
                    _ => return false,
                };
                self.find_card_anywhere(cid).is_some_and(|trig| {
                    trig.definition
                        .card_types
                        .iter()
                        .any(|t| exiled_types.contains(t))
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
            Predicate::ManaSpentOfColorAtLeast { color, at_least } => {
                ctx.mana_spent_by_color
                    .iter()
                    .find(|(c, _)| c == color)
                    .is_some_and(|(_, n)| *n >= *at_least)
            }
            Predicate::SourceCastWithColorSpent { color, at_least } => {
                ctx.source
                    .and_then(|s| self.find_card_anywhere(s))
                    .map(|c| &c.cast_mana_spent_by_color)
                    .and_then(|bc| bc.iter().find(|(c, _)| c == color))
                    .is_some_and(|(_, n)| *n >= *at_least)
            }
            Predicate::CastSpellSharesChosenColorOfSource => {
                let Some(color) = ctx.source.and_then(|s| self.find_card_anywhere(s)).and_then(|c| c.chosen_color) else {
                    return false;
                };
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cid => {
                        card.definition.printed_colors().contains(&color)
                    }
                    _ => false,
                })
            }
            Predicate::CastSpellNoColoredManaSpent => {
                // Read the just-cast spell's per-color payment off the stack.
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cid => {
                        card.cast_mana_spent_by_color.iter().all(|(_, n)| *n == 0)
                    }
                    _ => false,
                })
            }
            Predicate::CastSpellColorlessManaSpent { spent } => {
                // Colorless {C} spent = total mana spent − the colored breakdown
                // (whatever's left was paid with {C}, incl. for generic pips).
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                let c_spent = self.stack.iter().find_map(|si| match si {
                    StackItem::Spell { card, mana_spent, .. } if card.id == cid => {
                        let colored: u32 =
                            card.cast_mana_spent_by_color.iter().map(|(_, n)| *n).sum();
                        Some(mana_spent.saturating_sub(colored) > 0)
                    }
                    _ => None,
                });
                c_spent.unwrap_or(false) == *spent
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
            Predicate::SourceHasCountersAtLeast { counter, n } => ctx
                .source
                .and_then(|cid| self.battlefield_find(cid))
                .map(|c| c.counter_count(*counter) >= *n)
                .unwrap_or(false),
            Predicate::SourceClassLevelIs(n) => ctx
                .source
                .and_then(|cid| self.battlefield_find(cid))
                .map(|c| c.class_level == *n)
                .unwrap_or(false),
            Predicate::SourceClassLevelAtLeast(n) => ctx
                .source
                .and_then(|cid| self.battlefield_find(cid))
                .map(|c| c.class_level >= *n)
                .unwrap_or(false),
            Predicate::CastSpellFromExile => {
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cid => card.cast_from_exile,
                    _ => false,
                })
            }
            Predicate::CastSpellFromLibrary => {
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cid => card.cast_from_library,
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
            Predicate::DiscardCausedByOpponent => self
                .discard_causer
                .is_some_and(|c| self.opponents_of(ctx.controller).contains(&c)),
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
            Predicate::IsFirstCombatPhaseThisTurn => self.combat_phases_this_turn <= 1,
            Predicate::IsFirstEndStepThisTurn => self.end_steps_this_turn <= 1,
            Predicate::IsFirstUpkeepThisTurn => self.upkeep_steps_this_turn <= 1,
            Predicate::CastFromHand => {
                // Inverse of CastFromGraveyard. Triggers / activated
                // abilities default `cast_from_hand` to `true` which
                // matches their non-spell-resolution context.
                ctx.cast_from_hand
            }
            Predicate::SacrificedWasArtifact => {
                self.sacrificed_was_artifact.unwrap_or(false)
            }
            Predicate::SacrificedWasOutlaw => {
                self.sacrificed_was_outlaw.unwrap_or(false)
            }
            Predicate::SacrificedWasVehicle => {
                self.sacrificed_was_vehicle.unwrap_or(false)
            }
            Predicate::SacrificedWasColor(color) => self
                .sacrificed_colors
                .as_ref()
                .is_some_and(|cs| cs.contains(color)),
            Predicate::LastDiscardedWasMulticolored => {
                self.last_discarded_was_multicolored.unwrap_or(false)
            }
            Predicate::LastDiscardedWasColor(c) => self.last_discarded_colors.contains(c),
            Predicate::TriggerSourceEnteredFromGraveyard => {
                let cid = match ctx.trigger_source {
                    Some(EntityRef::Card(c)) | Some(EntityRef::Permanent(c)) => c,
                    _ => return false,
                };
                self.entered_from_graveyard_this_turn.contains(&cid)
                    || self.battlefield_find(cid).is_some_and(|c| !c.cast_from_hand)
            }
            Predicate::TriggerSourceEnteredByCast => {
                let cid = match ctx.trigger_source {
                    Some(EntityRef::Card(c)) | Some(EntityRef::Permanent(c)) => c,
                    _ => return false,
                };
                self.battlefield_find(cid).is_some_and(|c| c.entered_by_cast)
            }
            Predicate::YouControlACommander => {
                let ids = &self.players[ctx.controller].commanders;
                self.battlefield
                    .iter()
                    .any(|c| c.controller == ctx.controller && ids.contains(&c.id))
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
            Predicate::SpellWasMayhem => {
                // CR 702.187 — true iff this spell was cast from the graveyard
                // for its Mayhem cost.
                ctx.cast_via_mayhem
            }
            Predicate::SpellWasWaterbend => {
                // CR 701.67 — true iff this spell's optional waterbend cost was paid.
                ctx.cast_via_waterbend
            }
            Predicate::SpellCollectedEvidence => {
                // CR 701.59 — true iff this spell's "collect evidence" cost was paid.
                ctx.cast_collected_evidence
            }
            Predicate::SourceGiftPromised => {
                // CR 702.165 — read the source permanent's persisted gift flag.
                ctx.source
                    .and_then(|s| self.battlefield_find(s))
                    .is_some_and(|c| c.gift_promised)
            }
            Predicate::LastDiscardedManaValueAtMost(n) => {
                // A discard must have happened this resolution and its MV ≤ n.
                self.last_discarded_mana_value.is_some_and(|mv| mv <= *n)
            }
            Predicate::OwnExiledAdventureCard => {
                // CR 715 — the controller owns a card in exile on an Adventure.
                let owner = ctx.controller;
                self.exile.iter().any(|c| c.owner == owner && c.on_adventure)
            }
            Predicate::CastSpellTargetsSource => {
                // CR 702.85 — Heroic. The just-cast spell (trigger source,
                // a card on the stack) targets this trigger's own source.
                match (ctx.source, ctx.trigger_source) {
                    (Some(src), Some(EntityRef::Card(spell_id))) => {
                        self.stack.iter().any(|si| match si {
                            StackItem::Spell { card, target, additional_targets, .. }
                                if card.id == spell_id =>
                            {
                                target
                                    .iter()
                                    .chain(additional_targets.iter())
                                    .any(|t| matches!(t, Target::Permanent(p) if *p == src))
                            }
                            _ => false,
                        })
                    }
                    _ => false,
                }
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
            Predicate::AnOpponentHasMoreCardsInHand => {
                let you = ctx.controller;
                let your_hand = self.players[you].hand.len();
                self.players.iter().enumerate().any(|(i, p)| {
                    i != you && !p.eliminated && !self.same_team(i, you) && p.hand.len() > your_hand
                })
            }
            Predicate::CovenActive { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                let powers: std::collections::HashSet<i32> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == p && c.definition.is_creature())
                    .filter_map(|c| self.computed_permanent(c.id).map(|cp| cp.power))
                    .collect();
                powers.len() >= 3
            }
            Predicate::OilActivityThisTurn { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].oil_activity_this_turn),
            Predicate::CorruptedActive { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                self.players.iter().enumerate().any(|(i, pl)| {
                    i != p && !pl.eliminated && !self.same_team(i, p) && pl.poison_counters >= 3
                })
            }
            Predicate::ControlsGreatestPowerCreature { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                let powers: Vec<(usize, i32)> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.definition.is_creature())
                    .filter_map(|c| {
                        self.computed_permanent(c.id).map(|cp| (c.controller, cp.power))
                    })
                    .collect();
                let Some(max) = powers.iter().map(|(_, pow)| *pow).max() else { return false };
                powers.iter().any(|(ctrl, pow)| *ctrl == p && *pow >= max)
            }
            Predicate::AttackingAlone => self.attacking.len() == 1,
            Predicate::AttackingWithAtLeast(n) => self.attacking.len() as u32 >= *n,
            Predicate::AttackedWithTotalPowerAtLeast { who, at_least } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                let total: i32 = self
                    .attacking
                    .iter()
                    .filter_map(|a| self.battlefield_find(a.attacker).map(|c| (a.attacker, c.controller)))
                    .filter(|(_, ctrl)| *ctrl == p)
                    .filter_map(|(id, _)| self.computed_permanent(id).map(|cp| cp.power.max(0)))
                    .sum();
                total as u32 >= *at_least
            }
            Predicate::AttackedWithCountAtLeast { who, at_least } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                let count = self
                    .attacking
                    .iter()
                    .filter(|a| self.battlefield_find(a.attacker).is_some_and(|c| c.controller == p))
                    .count();
                count as u32 >= *at_least
            }
            Predicate::AttackedWithCreatureMatching { who, filter } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                self.attacking.iter().any(|a| {
                    self.battlefield_find(a.attacker).is_some_and(|c| c.controller == p)
                        && self.evaluate_requirement_static(
                            filter,
                            &Target::Permanent(a.attacker),
                            ctx.controller,
                            ctx.source,
                        )
                })
            }
            Predicate::CommittedCrimeThisTurn { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].committed_crime_this_turn),
            Predicate::FaceDownActivityThisTurn { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].face_down_activity_this_turn),
            Predicate::UnlockedDoorsControlledAtLeast { who, count } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                let total: u32 = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == p && c.definition.room.is_some())
                    .map(|c| c.unlocked_doors.count_ones())
                    .sum();
                total >= *count
            }
            Predicate::ControlsOutlaw { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                self.battlefield.iter().any(|c| c.controller == p && card_is_outlaw(c))
            }
            Predicate::RevoltActive { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].permanent_left_battlefield_this_turn),
            Predicate::VoidActive { who } => {
                self.nonland_permanent_left_bf_this_turn
                    || self
                        .resolve_player(who, ctx)
                        .is_some_and(|p| self.players[p].warped_spell_this_turn)
            }
            Predicate::DeliriumActive { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                self.delirium_active(p)
            }
            Predicate::DescendActive { who, count } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                self.players[p]
                    .graveyard
                    .iter()
                    .filter(|c| c.definition.is_permanent())
                    .count()
                    >= *count as usize
            }
            Predicate::DescendedThisTurn { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].descended_this_turn),
            Predicate::ArtifactEnteredThisTurn { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].artifacts_entered_this_turn > 0),
            Predicate::OwnsSourceNamedCardInEveryZone { who } => {
                let (Some(seat), Some(src)) = (self.resolve_player(who, ctx), ctx.source) else {
                    return false;
                };
                let Some(name) = self.find_card_anywhere(src).map(|c| c.definition.name) else {
                    return false;
                };
                let owned = |c: &crate::card::CardInstance| {
                    c.owner == seat && c.definition.name == name
                };
                self.exile.iter().any(owned)
                    && self.players[seat].hand.iter().any(owned)
                    && self.players[seat].graveyard.iter().any(owned)
                    && self.battlefield.iter().any(owned)
            }
            Predicate::PlaneswalkerEnteredThisTurn { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].planeswalkers_entered_this_turn > 0),
            Predicate::CreatureEnteredThisTurn { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| !self.players[p].creatures_entered_this_turn.is_empty()),
            Predicate::AnotherCreatureEnteredThisTurn { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| {
                    self.players[p]
                        .creatures_entered_this_turn
                        .iter()
                        .any(|id| Some(*id) != ctx.source)
                }),
            Predicate::CelebrationActive { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].nonland_permanents_entered_this_turn >= 2),
            Predicate::ThresholdActive { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].graveyard.len() >= 7),
            Predicate::MetalcraftActive { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                self.battlefield
                    .iter()
                    .filter(|c| c.controller == p && c.definition.is_artifact())
                    .count()
                    >= 3
            }
            Predicate::FerociousActive { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                self.battlefield
                    .iter()
                    .filter(|c| c.controller == p && c.definition.is_creature())
                    .any(|c| self.computed_permanent(c.id).is_some_and(|cp| cp.power >= 4))
            }
            Predicate::HellbentActive { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].hand.is_empty()),
            Predicate::FormidableActive { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                let total: i32 = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == p && c.definition.is_creature())
                    .filter_map(|c| self.computed_permanent(c.id).map(|cp| cp.power))
                    .sum();
                total >= 8
            }
            Predicate::ControlsEachGreatestPowerCreature { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                let powers: Vec<(usize, i32)> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.definition.is_creature())
                    .filter_map(|c| self.computed_permanent(c.id).map(|cp| (c.controller, cp.power)))
                    .collect();
                let Some(best) = powers.iter().map(|(_, pw)| *pw).max() else { return true };
                powers.iter().all(|(ctrl, pw)| *pw < best || *ctrl == p)
            }
            Predicate::ActivatedLoyaltyThisTurn { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].activated_loyalty_this_turn),
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
                // Use the fully-computed stats so continuous effects
                // (anthems, static pumps from other permanents) are
                // reflected — `CardInstance::power()` only sees counters
                // and self-pumps.
                let (p, t) = self
                    .computed_permanent(source_id)
                    .map(|cp| (cp.power, cp.toughness))
                    .unwrap_or_else(|| (source_card.power(), source_card.toughness()));
                (mana_spent as i32 > p) || (mana_spent as i32 > t)
            }
        }
    }

    // ── Requirement evaluation (unchanged API) ──────────────────────────────

    /// Last-known-information-aware power of `src`: its live battlefield power
    /// if present, else the `leaves_bf_lki` snapshot of a just-died source.
    /// Powers source-power-relative filters that resolve in a death trigger.
    pub(crate) fn source_power_lki(&self, src: CardId) -> Option<i32> {
        if let Some(c) = self.battlefield_find(src) {
            return Some(c.power());
        }
        self.leaves_bf_lki.get(&src).map(|snap| snap.power())
    }

    /// The caster (controller) of the spell with id `cid` if it's on the stack.
    pub(crate) fn stack_spell_caster(&self, cid: CardId) -> Option<usize> {
        self.stack.iter().find_map(|si| match si {
            StackItem::Spell { card, caster, .. } if card.id == cid => Some(*caster),
            _ => None,
        })
    }

    pub fn evaluate_requirement_static(
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
                // A `Target::Permanent` can also address a spell on the stack
                // (a "copy target spell you control" ability); its caster is
                // its controller. When the object has left the battlefield
                // (a die-trigger reading "a creature you control dies" off a
                // graveyard source), fall back to the CR 603.10 last-known
                // controller in `died_card_snapshots`.
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.controller)
                    .or_else(|| self.stack_spell_caster(*cid))
                    .or_else(|| self.died_card_snapshots.get(cid).map(|c| c.controller))
                    .map(|ctrl| ctrl == controller)
                    .unwrap_or(false),
                Target::Player(p) => *p == controller,
            },
            R::ControlledByOpponent => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.controller)
                    .or_else(|| self.stack_spell_caster(*cid))
                    .map(|ctrl| !self.same_team(ctrl, controller))
                    .unwrap_or(false),
                Target::Player(p) => !self.same_team(*p, controller),
            },
            R::ControlledByTriggerPlayer => {
                let Some(who) = self.trigger_event_player_scratch else { return false };
                match target {
                    Target::Permanent(cid) => self
                        .battlefield_find(*cid)
                        .map(|c| c.controller)
                        .or_else(|| self.stack_spell_caster(*cid))
                        // CR 108.4 — a card in a hidden/terminal zone has no
                        // controller; its owner stands in (Wrexial's "target
                        // instant or sorcery card in that player's graveyard").
                        .or_else(|| self.find_card_anywhere(*cid).map(|c| c.owner))
                        .is_some_and(|ctrl| ctrl == who),
                    Target::Player(p) => *p == who,
                }
            }
            R::OwnedByYou => match target {
                // Ownership is stable across zones — look the card up anywhere
                // (battlefield / graveyard / exile / stack) so "target creature
                // card in your graveyard" (Meren) resolves.
                Target::Permanent(cid) => self
                    .find_card_anywhere(*cid)
                    .map(|c| c.owner == controller)
                    .unwrap_or(false),
                Target::Player(_) => false,
            },
            R::DealtDamageToControllerThisTurn => match target {
                Target::Permanent(cid) => self.players[controller]
                    .creatures_that_damaged_me_this_turn
                    .contains(cid),
                Target::Player(_) => false,
            },
            R::PlayerDamagedBySourceThisTurn => match target {
                Target::Player(p) => source.is_some_and(|s| {
                    self.players[*p].creatures_that_damaged_me_this_turn.contains(&s)
                }),
                Target::Permanent(_) => false,
            },
            R::ControllerDescend(n) => {
                // Count permanent cards in the candidate's controller's
                // graveyard (CR 701.x — LCI Descend). For a `SelfHasKeywordWhile`
                // condition the controller arg already is the source's
                // controller, so read it directly.
                let owner = match target {
                    Target::Permanent(cid) => {
                        self.battlefield_find(*cid).map(|c| c.controller).unwrap_or(controller)
                    }
                    Target::Player(p) => *p,
                };
                let count = self.players[owner]
                    .graveyard
                    .iter()
                    .filter(|c| c.definition.is_permanent())
                    .count();
                count >= *n as usize
            }
            R::ControllerDrewAtLeastThisTurn(n) => {
                let owner = match target {
                    Target::Permanent(cid) => {
                        self.battlefield_find(*cid).map(|c| c.controller).unwrap_or(controller)
                    }
                    Target::Player(p) => *p,
                };
                self.players[owner].cards_drawn_this_turn >= *n
            }
            R::ControllerSacrificedArtifactThisTurn => {
                let owner = match target {
                    Target::Permanent(cid) => {
                        self.battlefield_find(*cid).map(|c| c.controller).unwrap_or(controller)
                    }
                    Target::Player(p) => *p,
                };
                self.players[owner].artifacts_sacrificed_this_turn > 0
            }
            R::ControllersTurn => {
                let owner = match target {
                    Target::Permanent(cid) => {
                        self.battlefield_find(*cid).map(|c| c.controller).unwrap_or(controller)
                    }
                    Target::Player(p) => *p,
                };
                self.active_player_idx == owner
            }
            R::ControllerCorrupted => {
                let owner = match target {
                    Target::Permanent(cid) => {
                        self.battlefield_find(*cid).map(|c| c.controller).unwrap_or(controller)
                    }
                    Target::Player(p) => *p,
                };
                self.players[owner].poison_counters >= 3
            }
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
                    // Dies-trigger filters (Felisa's "with a +1/+1 counter on
                    // it") read the dying object's last-known battlefield
                    // state, not the counter-stripped graveyard copy
                    // (CR 603.10 LKI; CR 122.2 cleared the counters on the
                    // zone change). Consulted before the terminal zones; the
                    // cache only holds cards from the in-flight die batch.
                    .or_else(|| self.died_card_snapshots.get(cid))
                    .or_else(|| {
                        self.resolving_lki_source
                            .filter(|s| s == cid)
                            .and_then(|_| self.leaves_bf_lki.get(cid))
                    })
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
                    .or_else(|| self.players.iter().find_map(|p| p.hand.iter().find(|c| c.id == *cid)));
                let Some(card) = card else { return false; };
                // Layer-4-aware card types for battlefield permanents
                // (CR 613.2): an artifact-ized creature (Phyrexian
                // Scriptures I), an animated land, or a devotion-gated god
                // must filter by its *computed* types, not the printed ones.
                // Off-battlefield cards keep the printed definition.
                let computed: Option<crate::game::layers::ComputedPermanent> =
                    if self.in_layer_gather.load(std::sync::atomic::Ordering::Relaxed) {
                        None // mid-recompute: printed types (reentrancy guard)
                    } else {
                        self.battlefield_find(*cid).and_then(|_| self.computed_permanent(*cid))
                    };
                let has_type = |t: crate::card::CardType| match &computed {
                    Some(cp) => cp.card_types.contains(&t),
                    None => card.definition.card_types.contains(&t),
                };
                // CR 613.2 layer-4 — a creature that gained a type from a
                // continuous effect (Jenova's Mutant grant) matches by its
                // *computed* subtypes on the battlefield; off-battlefield cards
                // (incl. die-snapshots, whose grants were stamped in at death)
                // fall back to the definition.
                let has_ctype = |ct: &crate::card::CreatureType| match &computed {
                    Some(cp) => cp.subtypes.creature_types.contains(ct),
                    None => card.definition.subtypes.creature_types.contains(ct),
                };
                // CR 613.2 layer-4 — subtypes/supertypes a permanent gained (or
                // lost) from a continuous effect (Vraska's Treasure, Song of the
                // Dryads' Forest, Sugar Coat's Food, the Ring-bearer's Legendary)
                // read from the *computed* type line on the battlefield.
                let has_atype = |a: &crate::card::ArtifactSubtype| match &computed {
                    Some(cp) => cp.subtypes.artifact_subtypes.contains(a),
                    None => card.definition.subtypes.artifact_subtypes.contains(a),
                };
                let has_ltype = |lt: &crate::card::LandType| match &computed {
                    Some(cp) => cp.subtypes.land_types.contains(lt),
                    None => card.definition.subtypes.land_types.contains(lt),
                };
                let has_stype = |st: &Supertype| match &computed {
                    Some(cp) => cp.supertypes.contains(st),
                    None => card.definition.supertypes.contains(st),
                };
                use crate::card::CardType as CT;
                match req {
                    // CR 604.3 — Grist is a creature everywhere but the
                    // battlefield.
                    R::Creature => {
                        has_type(CT::Creature)
                            || (card.definition.creature_off_battlefield && computed.is_none())
                    }
                    R::Artifact => has_type(CT::Artifact),
                    R::Enchantment => has_type(CT::Enchantment),
                    R::Planeswalker => has_type(CT::Planeswalker),
                    R::Permanent => card.definition.is_permanent(),
                    R::Land => has_type(CT::Land),
                    R::Nonland => !has_type(CT::Land),
                    R::Noncreature => !has_type(CT::Creature),
                    R::Tapped => card.tapped,
                    R::Untapped => !card.tapped,
                    R::DealtDamageThisTurn => card.dealt_damage_this_turn,
                    R::DamagedBySourceThisTurn => {
                        source.is_some_and(|s| card.damaged_by_this_turn.contains(&s))
                    }
                    // CR 105.2/202.2 — color is the union of the mana cost's
                    // colors and the color indicator (tokens, DFC backs), and
                    // empty under Devoid. `printed_colors` folds all three in.
                    R::HasColor(c) => card.definition.printed_colors().contains(c),
                    R::HasKeyword(kw) => card.has_keyword(kw),
                    R::HasToxic => card.has_toxic(),
                    R::HasModular => card.has_modular(),
                    R::HasMutate => card.definition.mutate.is_some(),
                    R::HasCyclingAbility => card.definition.keywords.iter().any(|k| matches!(
                        k,
                        crate::card::Keyword::Cycling(_)
                            | crate::card::Keyword::CyclingLife(_)
                            | crate::card::Keyword::Landcycling(_, _)
                            | crate::card::Keyword::Typecycling(_)
                    )),
                    R::HasDisturb => card
                        .definition
                        .keywords
                        .iter()
                        .any(|k| matches!(k, crate::card::Keyword::Disturb(_))),
                    R::PowerAtMost(n) => card.definition.is_creature() && card.power() <= *n,
                    R::ToughnessAtMost(n) => card.definition.is_creature() && card.toughness() <= *n,
                    R::PowerAtLeast(n) => card.definition.is_creature() && card.power() >= *n,
                    R::ToughnessAtLeast(n) => card.definition.is_creature() && card.toughness() >= *n,
                    R::ToughnessGreaterThanPower => {
                        card.definition.is_creature() && card.toughness() > card.power()
                    }
                    R::PowerGreaterThanBasePower => {
                        card.definition.is_creature() && card.power() > card.definition.power
                    }
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
                    R::WithAnyCounter => {
                        card.counters.values().any(|&n| n > 0)
                            || card.keyword_counters.values().any(|&n| n > 0)
                    }
                    R::HasNoCounters => {
                        card.counters.values().all(|&n| n == 0)
                            && card.keyword_counters.values().all(|&n| n == 0)
                    }
                    R::HasSupertype(st) => has_stype(st),
                    R::HasCreatureType(ct) => has_ctype(ct)
                        || card.has_keyword(&crate::card::Keyword::Changeling),
                    R::IsOutlaw => card_is_outlaw(card),
                    R::HasLandType(lt) => has_ltype(lt),
                    R::HasArtifactSubtype(a) => has_atype(a),
                    R::HasEnchantmentSubtype(e) => card.definition.subtypes.enchantment_subtypes.contains(e),
                    R::HasPlaneswalkerType(pw) => card.definition.subtypes.planeswalker_subtypes.contains(pw),
                    R::IsToken => card.is_token,
                    R::NotToken => !card.is_token,

                    R::Warped => card.warped,
                    R::IsBasicLand => card.definition.is_land() && card.definition.supertypes.contains(&Supertype::Basic),
                    R::HasAwaken => card.definition.alternative_cost.as_ref().is_some_and(|a| a.awaken),
                    R::IsNonbasicLand => card.definition.is_land() && !card.definition.supertypes.contains(&Supertype::Basic),
                    R::ProducesColorless => card.definition.produces_colorless(),
                    R::IsSnow => card.definition.is_snow(),
                    R::IsAttacking => self.attacking.iter().any(|a| a.attacker == card.id),
                    R::IsUnblocked => {
                        self.attacking.iter().any(|a| a.attacker == card.id)
                            && !self.blocked_attackers.contains(&card.id)
                    }
                    R::IsBlocked => self.blocked_attackers.contains(&card.id),
                    R::IsBlocking => self.block_map.contains_key(&card.id),
                    // Symmetric: the candidate blocks the source, or the source
                    // blocks the candidate (Gomazoa's "each creature it's
                    // blocking").
                    R::InCombatWithSource => source.is_some_and(|src| {
                        self.blockers_of(src).contains(cid)
                            || self.attackers_blocked_by(*cid).contains(&src)
                            || self.attackers_blocked_by(src).contains(cid)
                            || self.blockers_of(*cid).contains(&src)
                    }),
                    R::AttackedThisTurn => card.attacked_this_turn,
                    R::BlockedThisTurn => card.blocked_this_turn,
                    R::FaceDown => card.face_down,
                    // CR 603.4 — entered this turn (stamped on every ETB).
                    R::EnteredThisTurn => card.entered_turn == Some(self.turn_number),
                    R::EnteredFromGraveyardThisTurn => {
                        self.entered_from_graveyard_this_turn.contains(cid)
                    }
                    R::EnteredFromExileThisTurn => {
                        self.entered_from_exile_this_turn.contains(cid)
                    }
                    // CR 303 — "enchanted" = an Aura is attached. Equipment also
                    // sets `attached_to`, so require the attachment be an
                    // enchantment to exclude it.
                    R::IsEnchanted => self.battlefield.iter().any(|o| {
                        o.attached_to == Some(*cid) && o.definition.is_enchantment()
                    }),
                    R::PutIntoGraveyardFromBattlefieldThisTurn => {
                        self.graveyard_from_battlefield_this_turn.contains(cid)
                    }
                    // CR 301.5 — "equipped" = an Equipment is attached.
                    R::IsEquipped => self.attached_equipment_count(*cid) > 0,
                    // CR 701.60 — suspected.
                    R::IsSuspected => card.suspected,
                    // CR 702.103 — on the battlefield as a bestowed Aura.
                    R::IsBestowed => card.bestowed,
                    // CR 301.5 — equipped by at least `n` Equipment (Balan).
                    R::EquippedByAtLeast(n) => {
                        self.attached_equipment_count(*cid) as u32 >= *n
                    }
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
                    // CR 115 — a stack spell that targets the chooser or a
                    // permanent they control (Hindering Light).
                    R::SpellTargetsControllerOrControlled => self.stack.iter().any(|si| {
                        let StackItem::Spell { card: c, target, additional_targets, .. } = si else { return false };
                        if c.id != card.id { return false; }
                        target.iter().chain(additional_targets.iter()).any(|t| match t {
                            crate::game::types::Target::Player(p) => *p == controller,
                            crate::game::types::Target::Permanent(id) => self
                                .battlefield
                                .iter()
                                .any(|o| o.id == *id && o.controller == controller),
                        })
                    }),
                    R::SpellTargetsCreature => self.stack.iter().any(|si| {
                        let StackItem::Spell { card: c, target, additional_targets, .. } = si else { return false };
                        c.id == card.id
                            && target.iter().chain(additional_targets.iter()).any(|t| {
                                matches!(t, crate::game::types::Target::Permanent(id)
                                    if self.battlefield.iter().any(|o| o.id == *id && o.definition.is_creature()))
                            })
                    }),
                    R::SpellTargetsOnlySource => source.is_some_and(|src| {
                        self.stack.iter().any(|si| {
                            let StackItem::Spell { card: c, target, additional_targets, .. } = si
                            else {
                                return false;
                            };
                            c.id == card.id
                                && additional_targets.is_empty()
                                && *target
                                    == Some(crate::game::types::Target::Permanent(src))
                        })
                    }),
                    // CR 115.7 — "target spell with a single target": exactly
                    // one filled slot (Ricochet Trap).
                    R::SpellWithSingleTarget => self.stack.iter().any(|si| {
                        matches!(
                            si,
                            StackItem::Spell { card: c, target: Some(_), additional_targets, .. }
                                if c.id == card.id && additional_targets.is_empty()
                        )
                    }),
                    // Wash Away's base mode: a stack spell cast from
                    // anywhere but its owner's hand (CR 702.148 bracket).
                    R::SpellNotCastFromHand => self.stack.iter().any(|si| matches!(
                        si,
                        StackItem::Spell { card: c, .. } if c.id == card.id && !c.cast_from_hand
                    )),
                    R::HasAbilityOnStack => self.stack.iter().any(|si| matches!(
                        si,
                        StackItem::Trigger { source, .. } if *source == card.id
                    )),
                    R::ManaValueAtMost(n) => card.definition.cost.cmc() <= *n,
                    R::ManaValueAtMostDevotion(color) => {
                        card.definition.cost.cmc() <= self.devotion_to(controller, &[*color]).max(0) as u32
                    }
                    R::ManaValueAtMostYourCount(inner) => {
                        let n = self
                            .battlefield
                            .iter()
                            .filter(|c| self.evaluate_requirement_on_card(inner, c, controller))
                            .count() as u32;
                        card.definition.cost.cmc() <= n
                    }
                    R::ToughnessAtMostYourCount(inner) => {
                        let n = self
                            .battlefield
                            .iter()
                            .filter(|c| self.evaluate_requirement_on_card(inner, c, controller))
                            .count() as i32;
                        card.definition.is_creature()
                            && self
                                .computed_permanent(card.id)
                                .map(|cp| cp.toughness)
                                .unwrap_or_else(|| card.toughness())
                                <= n
                    }
                    R::PowerAtMostYourCount(inner) => {
                        let n = self
                            .battlefield
                            .iter()
                            .filter(|c| self.evaluate_requirement_on_card(inner, c, controller))
                            .count() as i32;
                        card.definition.is_creature()
                            && self
                                .computed_permanent(card.id)
                                .map(|cp| cp.power)
                                .unwrap_or_else(|| card.power())
                                <= n
                    }
                    R::ManaValueAtMostPermanentsInYourGraveyard => {
                        let n = self.players[controller]
                            .graveyard
                            .iter()
                            .filter(|c| c.definition.is_permanent())
                            .count() as u32;
                        card.definition.cost.cmc() <= n
                    }
                    // Unresolved X-relative filter (no X in scope here).
                    R::ManaValueAtMostXFromCost | R::ManaValueExactlyXFromCost | R::PowerAtMostXFromCost | R::ToughnessAtMostXFromCost | R::ManaValueAtMostConverged => false,
                    R::ManaValueAtLeast(n) => card.definition.cost.cmc() >= *n,
                    R::ManaValueExactly(n) => card.definition.cost.cmc() == *n,
                    R::ManaValueEqualsTriggerAmount => {
                        card.definition.cost.cmc() == self.trigger_event_amount_scratch
                    }
                    R::ManaValueParity { odd } => (card.definition.cost.cmc() % 2 == 1) == *odd,
                    R::ManaValueEqualsSacrificedPlus(off) => {
                        card.definition.cost.cmc()
                            == self.sacrificed_mana_value.unwrap_or(0) + *off
                    }
                    R::ManaValueAtMostSacrificedPlus(off) => {
                        card.definition.cost.cmc()
                            <= self.sacrificed_mana_value.unwrap_or(0) + *off
                    }
                    R::ManaValueLessThanEventAmount => {
                        card.definition.cost.cmc() < self.trigger_event_amount_scratch
                    }
                    R::HasCardType(ct) => {
                        // CR 715 / 702.183 — an Adventure/Omen card is its
                        // instant/sorcery half on the stack; report those types.
                        if let Some(half) = card.alt_spell_half() {
                            half.card_types.contains(ct)
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
                    R::IsSource => source == Some(*cid),
                    R::ManaValueAtMostCastManaSpent => source
                        .and_then(|s| self.battlefield_find(s))
                        .is_some_and(|s| card.definition.cost.cmc() <= s.cast_mana_spent),
                    R::ManaValueAtMostLifeGainedThisTurn => self
                        .players
                        .get(controller)
                        .is_some_and(|p| card.definition.cost.cmc() <= p.life_gained_this_turn),
                    R::ManaValueAtMostSourcePower => source
                        .and_then(|s| self.source_power_lki(s))
                        .is_some_and(|pw| card.definition.cost.cmc() as i32 <= pw),
                    R::InGraveyard => self
                        .players
                        .iter()
                        .any(|p| p.graveyard.iter().any(|c| c.id == *cid)),
                    R::InYourGraveyard => self
                        .players
                        .get(controller)
                        .is_some_and(|p| p.graveyard.iter().any(|c| c.id == *cid)),
                    R::InOpponentGraveyard => self
                        .players
                        .iter()
                        .enumerate()
                        .any(|(i, p)| i != controller && p.graveyard.iter().any(|c| c.id == *cid)),
                    R::InExile => self.exile.iter().any(|c| c.id == *cid),
                    R::ExiledWithSource => source.is_some_and(|s| {
                        self.exile.iter().any(|c| c.id == *cid && c.exiled_with == Some(s))
                    }),
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
                    // Power sibling of the MV variant above — "the greatest
                    // power among [filter] they control" (Professor Onyx −3).
                    // Ties pass permissively; battlefield-only.
                    R::HasGreatestPowerAmongControlled(inner) => {
                        let Some(cand) = self.battlefield_find(*cid) else {
                            return false;
                        };
                        if !self.evaluate_requirement_static(inner, target, controller, source) {
                            return false;
                        }
                        let cand_pow = cand.power();
                        let cand_ctrl = cand.controller;
                        !self.battlefield.iter().any(|other| {
                            other.controller == cand_ctrl
                                && other.id != *cid
                                && self.evaluate_requirement_static(
                                    inner,
                                    &Target::Permanent(other.id),
                                    controller,
                                    source,
                                )
                                && other.power() > cand_pow
                        })
                    }
                    R::HasGreatestPowerAmongAllCreatures => {
                        let Some(cand) = self.battlefield_find(*cid) else { return false };
                        if !cand.definition.is_creature() {
                            return false;
                        }
                        let cand_pow = cand.power();
                        !self.battlefield.iter().any(|other| {
                            other.id != *cid
                                && other.definition.is_creature()
                                && other.power() > cand_pow
                        })
                    }
                    R::HasName(name) => card.definition.name == name.as_str(),
                    R::ManaValueAtMostControllerHand => {
                        card.definition.cost.cmc() as usize <= self.players[controller].hand.len()
                    }
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
                    R::IsSourceChosenCardType => source
                        .and_then(|sid| self.battlefield_find(sid))
                        .and_then(|s| s.chosen_card_type.clone())
                        .is_some_and(|t| card.definition.card_types.contains(&t)),
                    // Extraplanar Lens — "a land with the same name as the
                    // exiled card".
                    R::SameNameAsExiledWithSource => source.is_some_and(|sid| {
                        self.exile
                            .iter()
                            .any(|c| {
                                c.exiled_with == Some(sid)
                                    && c.definition.name == card.definition.name
                            })
                    }),
                    // Mourner's Shield — "shares a color with the exiled card".
                    R::SharesColorWithExiledBySource => source.is_some_and(|sid| {
                        self.exile
                            .iter()
                            .find(|c| c.exiled_with == Some(sid))
                            .is_some_and(|imp| {
                                let colors = imp.definition.printed_colors();
                                card.definition
                                    .printed_colors()
                                    .iter()
                                    .any(|c| colors.contains(c))
                            })
                    }),
                    // "The permanent this source is attached to" — the
                    // source-precise half of `AttachedToSource` (Necrotic
                    // Plague's granted upkeep sacrifice). A source-blind
                    // caller still answers `true`.
                    R::IsHostOfSource => {
                        match source.and_then(|sid| self.battlefield_find(sid)) {
                            Some(src) => src.attached_to == Some(card.id),
                            None => true,
                        }
                    }
                    // Konda's Banner — "creatures that share a color / a
                    // creature type with equipped creature". Read off printed
                    // characteristics: this filter is evaluated *inside* the
                    // layer gather, so consulting the computed view would
                    // recurse.
                    R::SharesColorWithAttachedHost | R::SharesCreatureTypeWithAttachedHost => {
                        let by_color = matches!(req, R::SharesColorWithAttachedHost);
                        source
                            .and_then(|sid| self.battlefield_find(sid))
                            .and_then(|src| src.attached_to)
                            .and_then(|host| self.battlefield_find(host))
                            .is_some_and(|host| {
                                if by_color {
                                    let hc = host.definition.printed_colors();
                                    card.definition.printed_colors().iter().any(|c| hc.contains(c))
                                } else {
                                    host.definition
                                        .subtypes
                                        .creature_types
                                        .iter()
                                        .any(|t| {
                                            card.definition.subtypes.creature_types.contains(t)
                                        })
                                }
                            })
                    }
                    // Zone-agnostic atoms (spell/enchantment subtype, token
                    // flags, …) the battlefield walker doesn't special-case
                    // are evaluated against the located card. Keeps the two
                    // requirement walkers from drifting apart (TODO P3).
                    _ => self.evaluate_requirement_on_card(req, card, controller),
                }
            }
        }
    }

    /// CR 201.4a — evaluate a `SelectionRequirement` against a bare
    /// `CardDefinition` (a *name*, not an object in any zone), for the
    /// namespace restriction on "choose a [kind] card name".
    pub fn definition_matches_requirement(
        &self,
        def: &crate::card::CardDefinition,
        req: &SelectionRequirement,
        controller: usize,
    ) -> bool {
        let scratch = CardInstance::new(CardId(u32::MAX), def.clone(), controller);
        self.evaluate_requirement_on_card(req, &scratch, controller)
    }

    /// Evaluate a `SelectionRequirement` directly against a `CardInstance`
    /// without requiring it to be on the battlefield. Used for library searches.
    /// Battlefield-only predicates (Tapped, IsAttacking, etc.) return false.
    pub fn evaluate_requirement_on_card(
        &self,
        req: &SelectionRequirement,
        card: &CardInstance,
        controller: usize,
    ) -> bool {
        use SelectionRequirement as R;
        match req {
            R::Any => true,
            R::ManaValueEqualsTriggerAmount => {
                card.definition.cost.cmc() == self.trigger_event_amount_scratch
            }
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
            R::HasAwaken => card.definition.alternative_cost.as_ref().is_some_and(|a| a.awaken),
            R::PutIntoGraveyardFromBattlefieldThisTurn => {
                self.graveyard_from_battlefield_this_turn.contains(&card.id)
            }
            R::ControlledByTriggerPlayer => {
                self.trigger_event_player_scratch == Some(card.controller)
            }
            R::OwnedByYou => card.owner == controller,
            R::Creature => {
                // CR 604.3 — Grist is a creature everywhere but the battlefield.
                card.definition.is_creature()
                    || (card.definition.creature_off_battlefield
                        && self.battlefield_find(card.id).is_none())
            }
            R::Artifact => card.definition.is_artifact(),
            R::Enchantment => card.definition.is_enchantment(),
            R::Planeswalker => card.definition.is_planeswalker(),
            R::Permanent | R::PermanentCard => card.definition.is_permanent(),
            R::ControllerDescend(n) => {
                self.players[controller]
                    .graveyard
                    .iter()
                    .filter(|c| c.definition.is_permanent())
                    .count()
                    >= *n as usize
            }
            R::SharesNameWithControllerGraveyardCard => self.players[controller]
                .graveyard
                .iter()
                .any(|c| c.id != card.id && c.definition.name == card.definition.name),
            R::ControllerDrewAtLeastThisTurn(n) => {
                self.players[controller].cards_drawn_this_turn >= *n
            }
            R::ControllerSacrificedArtifactThisTurn => {
                self.players[card.controller].artifacts_sacrificed_this_turn > 0
            }
            R::ControllersTurn => self.active_player_idx == card.controller,
            R::ControllerCorrupted => self.players[card.controller].poison_counters >= 3,
            R::Land => card.definition.is_land(),
            R::Nonland => !card.definition.is_land(),
            R::Noncreature => !card.definition.is_creature(),
            // CR 105.2/202.2 — color is the union of the mana cost's colors
            // (incl. hybrid {R/W}, Phyrexian {R/P}, mono-hybrid {2/W}) and the
            // color indicator (tokens, DFC backs), and empty under Devoid.
            // `printed_colors` folds all three in — matching the battlefield
            // path (`colors_from_card`) for cards in hidden zones too.
            R::HasColor(c) => card.definition.printed_colors().contains(c),
            R::HasKeyword(kw) => card.has_keyword(kw),
            R::HasToxic => card.has_toxic(),
            R::HasModular => card.has_modular(),
            R::HasMutate => card.definition.mutate.is_some(),
            R::HasCyclingAbility => card.definition.keywords.iter().any(|k| matches!(
                k,
                crate::card::Keyword::Cycling(_)
                    | crate::card::Keyword::CyclingLife(_)
                    | crate::card::Keyword::Landcycling(_, _)
                    | crate::card::Keyword::Typecycling(_)
            )),
            R::HasDisturb => card
                .definition
                .keywords
                .iter()
                .any(|k| matches!(k, crate::card::Keyword::Disturb(_))),
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
            R::ToughnessGreaterThanPower => {
                card.definition.is_creature() && card.toughness() > card.power()
            }
            R::PowerGreaterThanBasePower => {
                card.definition.is_creature() && card.power() > card.definition.power
            }
            R::PowerPlusToughnessAtMost(n) => {
                card.definition.is_creature() && card.power() + card.toughness() <= *n
            }
            R::HasSupertype(st) => card.definition.supertypes.contains(st),
            R::HasCreatureType(ct) => card.definition.subtypes.creature_types.contains(ct)
                        || card.has_keyword(&crate::card::Keyword::Changeling)
                        || self.graveyard_type_grants(card).contains(ct),
            R::IsOutlaw => card_is_outlaw(card),
            R::HasLandType(lt) => card.definition.subtypes.land_types.contains(lt),
            R::HasArtifactSubtype(a) => card.definition.subtypes.artifact_subtypes.contains(a),
            R::HasEnchantmentSubtype(e) => card.definition.subtypes.enchantment_subtypes.contains(e),
            R::HasPlaneswalkerType(pw) => card.definition.subtypes.planeswalker_subtypes.contains(pw),
            R::HasSpellSubtype(s) => card.definition.subtypes.spell_subtypes.contains(s),
            R::IsToken => card.is_token,
            R::NotToken => !card.is_token,

            R::Warped => card.warped,
            // CR 603.4 — entered this turn (hidden-zone cards are never
            // stamped, so this is false off the battlefield).
            R::EnteredThisTurn => card.entered_turn == Some(self.turn_number),
            R::EnteredFromGraveyardThisTurn => {
                self.entered_from_graveyard_this_turn.contains(&card.id)
            }
            R::EnteredFromExileThisTurn => {
                self.entered_from_exile_this_turn.contains(&card.id)
            }
            R::IsBasicLand => card.definition.is_land() && card.definition.supertypes.contains(&Supertype::Basic),
            R::IsNonbasicLand => card.definition.is_land() && !card.definition.supertypes.contains(&Supertype::Basic),
            R::ProducesColorless => card.definition.produces_colorless(),
            R::IsSnow => card.definition.is_snow(),
            R::ManaValueAtMost(n) => card.definition.cost.cmc() <= *n,
            // Need the ability's source or the live trigger context, which
            // only the static walker carries.
            R::SharesColorWithExiledBySource
            | R::SameNameAsExiledWithSource
            | R::SharesColorWithAttachedHost
            | R::SharesCreatureTypeWithAttachedHost => false,
            // Empty-Shrine Kannushi — printed colours on both sides, since
            // this is consulted from inside the layer gather.
            R::SharesColorWithPermanentYouControl => {
                let colors = card.definition.printed_colors();
                !colors.is_empty()
                    && self.battlefield.iter().any(|c| {
                        c.controller == controller
                            && c.definition.printed_colors().iter().any(|x| colors.contains(x))
                    })
            }
            // Glissa Sunseeker — "if its mana value is equal to the amount of
            // unspent mana you have".
            R::ManaValueEqualsYourUnspentMana => {
                card.definition.cost.cmc() == self.players[controller].mana_pool.total()
            }
            // CR — "with the lowest mana value" among nonland permanents; a tie
            // leaves every tied permanent legal (Culling Scales).
            R::LowestManaValueAmongNonland => {
                !card.definition.is_land()
                    && self
                        .battlefield
                        .iter()
                        .filter(|c| !c.definition.is_land())
                        .all(|c| c.definition.cost.cmc() >= card.definition.cost.cmc())
            }
            R::ManaValueAtMostDevotion(color) => {
                card.definition.cost.cmc() <= self.devotion_to(controller, &[*color]).max(0) as u32
            }
            R::ManaValueAtMostYourCount(inner) => {
                let n = self
                    .battlefield
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(inner, c, controller))
                    .count() as u32;
                card.definition.cost.cmc() <= n
            }
            R::ToughnessAtMostYourCount(inner) => {
                let n = self
                    .battlefield
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(inner, c, controller))
                    .count() as i32;
                card.definition.is_creature()
                    && self
                        .computed_permanent(card.id)
                        .map(|cp| cp.toughness)
                        .unwrap_or_else(|| card.toughness())
                        <= n
            }
            R::PowerAtMostYourCount(inner) => {
                let n = self
                    .battlefield
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(inner, c, controller))
                    .count() as i32;
                card.definition.is_creature()
                    && self
                        .computed_permanent(card.id)
                        .map(|cp| cp.power)
                        .unwrap_or_else(|| card.power())
                        <= n
            }
            R::ManaValueAtMostPermanentsInYourGraveyard => {
                let n = self.players[controller]
                    .graveyard
                    .iter()
                    .filter(|c| c.definition.is_permanent())
                    .count() as u32;
                card.definition.cost.cmc() <= n
            }
            // Unresolved X-relative filter (callers concretize via `resolve_x`).
            // `CastManaSpent` is source-relative; no source here, so vacuous.
            R::ManaValueAtMostXFromCost | R::ManaValueExactlyXFromCost | R::PowerAtMostXFromCost | R::ToughnessAtMostXFromCost | R::ManaValueAtMostConverged | R::ManaValueAtMostCastManaSpent | R::ManaValueAtMostSourcePower | R::ManaValueAtMostLifeGainedThisTurn => false,
            R::ManaValueAtLeast(n) => card.definition.cost.cmc() >= *n,
            R::ManaValueExactly(n) => card.definition.cost.cmc() == *n,
            R::ManaValueParity { odd } => (card.definition.cost.cmc() % 2 == 1) == *odd,
            // Unresolved source-counter MV gate (concretized at resolution
            // via `resolve_source_counters`).
            R::ManaValueEqualsSourceCounters(_) => false,
            R::ManaValueAtMostDiscardedThisEffect => {
                card.definition.cost.cmc() <= self.last_discarded_mana_value.unwrap_or(0)
            }
            R::ManaValueEqualsDiscardedThisEffect => {
                self.last_discarded_mana_value
                    .is_some_and(|mv| card.definition.cost.cmc() == mv)
            }
            R::ManaValueEqualsSacrificedPlus(off) => {
                card.definition.cost.cmc() == self.sacrificed_mana_value.unwrap_or(0) + *off
            }
            R::ManaValueAtMostSacrificedPlus(off) => {
                card.definition.cost.cmc() <= self.sacrificed_mana_value.unwrap_or(0) + *off
            }
            R::ManaValueLessThanEventAmount => {
                card.definition.cost.cmc() < self.trigger_event_amount_scratch
            }
            R::HasCardType(ct) => {
                        // CR 715 / 702.183 — an Adventure/Omen card is its
                        // instant/sorcery half on the stack; report those types.
                        if let Some(half) = card.alt_spell_half() {
                            half.card_types.contains(ct)
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
            // A card in another zone is never the battlefield source.
            R::IsSource => false,
            R::InGraveyard => self
                .players
                .iter()
                .any(|p| p.graveyard.iter().any(|c| c.id == card.id)),
            R::InYourGraveyard => self
                .players
                .get(controller)
                .is_some_and(|p| p.graveyard.iter().any(|c| c.id == card.id)),
            R::InOpponentGraveyard => self
                .players
                .iter()
                .enumerate()
                .any(|(i, p)| i != controller && p.graveyard.iter().any(|c| c.id == card.id)),
            R::InExile => self.exile.iter().any(|c| c.id == card.id),
            // Source-relative; this card-only path has no source id.
            R::ExiledWithSource => card.exiled_with.is_some(),
            // Battlefield-only ("greatest MV among controlled" walks the
            // battlefield in the static variant; library searches don't
            // surface this filter).
            R::HasGreatestManaValueAmongControlled(_) => false,
            R::HasGreatestPowerAmongControlled(_) => false,
            R::HasGreatestPowerAmongAllCreatures => false,
            // Name match works in any zone — used by Grandeur
            // activations that walk a hand for a same-named card.
            R::HasName(name) => card.definition.name == name.as_str(),
            // Resolved to a concrete `HasName` by callers that have the
            // source in hand (RevealUntilFind); vacuously false otherwise.
            R::NamedBySource => false,
            R::IsSourceChosenCardType => false,
            // Count walks the battlefield for the evaluating controller's
            // matching permanents; the candidate's own zone is irrelevant.
            R::ManaValueAtMostControllerHand => {
                card.definition.cost.cmc() as usize <= self.players[controller].hand.len()
            }
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
            // "Attached to something" — the source-precise intersection (the
            // permanent must be attached to *this* source) happens in the
            // sac-cost path, which knows the source id.
            R::AttachedToSource => card.attached_to.is_some(),
            R::IsHostOfSource => true,
            // `self.attacking` keys by card id, so a card not on the battlefield
            // is never listed — this stays false there (Static Snare's affinity
            // "for each attacking creature" reads it from the affinity counter).
            R::IsAttacking => self.attacking.iter().any(|a| a.attacker == card.id),
            // A battlefield instance carries this flag directly (Rowdy Research's
            // "{1} less for each creature that attacked this turn" affinity).
            R::AttackedThisTurn => card.attacked_this_turn,
            R::BlockedThisTurn => card.blocked_this_turn,
            // CR 701.60 — the suspected flag lives on the instance.
            R::IsSuspected => card.suspected,
            // Answerable off live state even for a card that has left the
            // battlefield: the Aura's `attached_to` still points at it during
            // the death replacement (Necromancer's Magemark).
            R::IsEnchanted => self
                .battlefield
                .iter()
                .any(|o| o.attached_to == Some(card.id) && o.definition.is_enchantment()),
            // CR 301.5 — same reasoning for "equipped" (Rakdos Riteknife's
            // tap-an-equipped-creature cost).
            R::IsEquipped => self.attached_equipment_count(card.id) > 0,
            // Battlefield-state predicates can't be evaluated for library cards.
            R::Tapped | R::Untapped | R::WithCounter(_) | R::WithAnyCounter
            | R::IsUnblocked | R::IsBlocked | R::IsBlocking | R::InCombatWithSource
            | R::IsAttackingAlone | R::IsBlockingAlone
            | R::FaceDown | R::HasAbilityOnStack
            | R::IsSpellOnStack | R::SpellNotCastFromHand
            | R::SpellTargetsControllerOrControlled
            | R::SpellTargetsCreature
            | R::SpellTargetsOnlySource
            | R::SpellWithSingleTarget
            | R::DealtDamageToControllerThisTurn | R::IsBestowed
            | R::EquippedByAtLeast(_) | R::IsModified | R::DealtDamageThisTurn
            | R::DamagedBySourceThisTurn | R::PlayerDamagedBySourceThisTurn => false,
        }
    }
}
