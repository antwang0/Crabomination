//! Pure-query helpers over `GameState`: `evaluate_value` (numeric expressions),
//! `evaluate_predicate` (boolean conditions), `evaluate_requirement_static`
//! and `evaluate_requirement_on_card` (selection-requirement matching).
//!
//! These are read-only and called from the resolver match arms in
//! `mod.rs` (and from `auto_target_for_effect_avoiding` in `targeting.rs`).

use crate::game::KeywordSlice;

use super::{EffectContext, EntityRef};
use crate::card::{CardId, CardInstance, CardType, SelectionRequirement, Supertype};
use crate::effect::{Predicate, Value};
use crate::mana::ManaSymbol;
use crate::game::{GameState, PresenceGate, StackItem, Target};

/// One requirement walk's memo of the three layer-4 presence gates the
/// printed evaluator consults — see [`GameState::requirement_on_permanent`].
/// Built once per requirement by the caller and read per permanent; `false`
/// from a gate is authoritative, `true` sends that leaf to the walker.
///
/// Each read is the gate's *body*, not its `GameState` wrapper: the wrappers'
/// one caller is the walker, which inlines them whole, and a second caller
/// de-inlines them there (PERF `(-182)`, two builds). A closure of this
/// module's own is a separate `presence_gate` monomorphization, so the
/// walker's copies are untouched.
#[derive(Default)]
pub(crate) struct PrintedGates {
    card: std::cell::Cell<Option<bool>>,
    creature: std::cell::Cell<Option<bool>>,
    land: std::cell::Cell<Option<bool>>,
}

impl PrintedGates {
    #[inline]
    fn card(&self, g: &GameState) -> bool {
        if let Some(v) = self.card.get() {
            return v;
        }
        let v = g.presence_gate(PresenceGate::Card, || g.card_type_change_unscoped());
        self.card.set(Some(v));
        v
    }
    #[inline]
    fn creature(&self, g: &GameState) -> bool {
        if let Some(v) = self.creature.get() {
            return v;
        }
        let v = g.battlefield.has_creature_type_changer(crate::game::card_can_change_creature_types)
            || g.presence_gate(PresenceGate::Creature, || {
                g.continuous_effects.iter().any(|e| {
                    matches!(
                        e.modification,
                        crate::game::Modification::AddCreatureType(_)
                            | crate::game::Modification::SetCreatureTypes(_)
                    )
                })
            });
        self.creature.set(Some(v));
        v
    }
    #[inline]
    fn land(&self, g: &GameState) -> bool {
        if let Some(v) = self.land.get() {
            return v;
        }
        let v = g.battlefield.has_land_type_changer(crate::game::card_can_change_land_types)
            || g.presence_gate(PresenceGate::Land, || {
                g.continuous_effects.iter().any(|e| {
                    matches!(
                        e.modification,
                        crate::game::Modification::AddLandType(_)
                            | crate::game::Modification::SetLandTypes(_)
                            | crate::game::Modification::ReplaceBasicLandType(..)
                    )
                })
            });
        self.land.set(Some(v));
        v
    }
}

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

/// Equinox — `(targeted, mass)`: does this spell's effect destroy lands by
/// pointing at one, and/or by sweeping them?
pub(crate) fn effect_destroys_lands(e: &crate::effect::Effect) -> (bool, bool) {
    use crate::effect::{Effect, Selector};
    let or = |a: (bool, bool), b: (bool, bool)| (a.0 || b.0, a.1 || b.1);
    match e {
        Effect::Seq(v) => v.iter().map(effect_destroys_lands).fold((false, false), or),
        Effect::MayDo { body, .. } => effect_destroys_lands(body),
        Effect::Destroy { what } | Effect::DestroyNoRegen { what } | Effect::DestroyAndRemember { what } => {
            match what {
                Selector::Target(_) | Selector::TargetFiltered { .. } => (true, false),
                Selector::EachPermanent(f) => (false, filter_can_match_land(f)),
                _ => (false, false),
            }
        }
        Effect::DestroyTargets { filter } | Effect::DestroyTargetsPolymorph { filter } => {
            (filter_can_match_land(filter), false)
        }
        Effect::DestroyEachMatchingWithManaValue { filter, .. } => {
            (false, filter_can_match_land(filter))
        }
        Effect::DestroyLandOfEachBasicType => (false, true),
        _ => (false, false),
    }
}

/// Can a selection requirement match a land at all? Conservative: only
/// filters that positively admit lands count.
fn filter_can_match_land(f: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match f {
        R::Any | R::Permanent | R::Land | R::IsNonbasicLand | R::HasLandType(_) => true,
        R::And(a, b) => filter_can_match_land(a) && filter_can_match_land(b),
        R::Or(a, b) => filter_can_match_land(a) || filter_can_match_land(b),
        _ => false,
    }
}

impl GameState {
    /// One of the five board/resource tallies the EXO Keeper and Oath cycles
    /// compare between two seats.
    pub fn player_tally(&self, seat: usize, what: crate::card::PlayerTally) -> i64 {
        use crate::card::PlayerTally;
        match what {
            PlayerTally::Life => self.players[seat].life as i64,
            PlayerTally::CardsInHand => self.players[seat].hand.len() as i64,
            PlayerTally::CreaturesControlled => self
                .battlefield
                .iter()
                .filter(|c| c.controller == seat && c.definition.is_creature())
                .count() as i64,
            PlayerTally::LandsControlled => self
                .battlefield
                .iter()
                .filter(|c| c.controller == seat && c.definition.is_land())
                .count() as i64,
            PlayerTally::NonbasicLandsControlled => self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == seat
                        && c.definition.is_land()
                        && !c.definition.supertypes.contains(&crate::card::Supertype::Basic)
                })
                .count() as i64,
            PlayerTally::CreatureCardsInGraveyard => self.players[seat]
                .graveyard
                .iter()
                .filter(|c| c.definition.is_creature())
                .count() as i64,
        }
    }

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
            // CR 105.4 — a permanent that is exactly two colors contributes
            // its unordered pair; count the distinct pairs (Niv-Mizzet).
            Value::DistinctTwoColorPairsControlled(who) => {
                let Some(p) = self.resolve_player(who, ctx) else { return 0 };
                let mut pairs: Vec<crate::mana::ColorSet> = Vec::new();
                for c in self.battlefield.iter().filter(|c| c.controller == p) {
                    let Some(cp) = self.computed_permanent(c.id) else { continue };
                    if cp.colors.len() != 2 {
                        continue;
                    }
                    if !pairs.contains(&cp.colors) {
                        pairs.push(cp.colors);
                    }
                }
                pairs.len() as i32
            }
            Value::ArtifactsToGraveyardFromBattlefieldThisTurn => self
                .players
                .iter()
                .flat_map(|p| p.graveyard.iter())
                .filter(|c| {
                    c.definition.is_artifact()
                        && self.graveyard_from_battlefield_this_turn.contains(&c.id)
                })
                .count() as i32,
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
                        let changeling = cp.keywords().has_kw(&Keyword::Changeling);
                        std::array::from_fn(|i| {
                            changeling || cp.subtypes().creature_types.contains(&roles[i])
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
            Value::OpponentCount => self.opponents_of(ctx.controller).len() as i32,
            Value::CreaturesExiledFromControlThisTurn(who) => self
                .resolve_players(who, ctx)
                .into_iter()
                .map(|p| self.players[p].creatures_exiled_from_control_this_turn as i32)
                .sum(),
            Value::UnlockedDoorsControlled(who) => self
                .resolve_player(who, ctx)
                .map(|p| {
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == p && c.definition.room.is_some())
                        .map(|c| c.unlocked_doors.count_ones() as i32)
                        .sum()
                })
                .unwrap_or(0),
            // CR 700.2 — how many modes the resolved spell chose. A plain
            // `ChooseMode` spell records one; modal-cast spells record the set.
            Value::ModesChosenOf(sel) => self
                .resolve_selector(sel, ctx)
                .into_iter()
                .find_map(|e| e.as_card_id())
                .and_then(|cid| {
                    self.stack.iter().find_map(|si| match si {
                        crate::game::types::StackItem::Spell { card, mode, .. }
                            if card.id == cid =>
                        {
                            Some(card.modes_chosen.len().max(usize::from(mode.is_some())) as i32)
                        }
                        _ => None,
                    })
                })
                .unwrap_or(0),
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
            Value::CombatDamageTakenThisTurn(p) => self
                .resolve_players(p, ctx)
                .iter()
                .map(|&p| self.players[p].combat_damage_taken_this_turn as i32)
                .max()
                .unwrap_or(0),
            Value::DamageDealtToSourceThisTurn => ctx
                .source
                .and_then(|id| {
                    self.battlefield_find(id).or_else(|| self.leaves_bf_lki.get(&id))
                })
                .map(|c| c.damage_dealt_to_this_turn as i32)
                .unwrap_or(0),
            Value::DamageToSourceThisTurnFromOthersNamedSame => ctx
                .source
                .and_then(|id| {
                    self.battlefield_find(id).or_else(|| self.leaves_bf_lki.get(&id))
                })
                .map(|c| {
                    c.damage_by_source_name_this_turn
                        .iter()
                        .filter(|(name, _)| *name == c.definition.name)
                        .map(|(_, n)| *n as i32)
                        .sum()
                })
                .unwrap_or(0),
            Value::CreaturesDiedThisResolution => self.creatures_died_this_resolution as i32,
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
                let mut seats = crate::fxhash::HashSet::default();
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
            Value::ArtifactDamageToPlayerThisTurn { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return 0 };
                self.artifact_damage_to_players_this_turn
                    .iter()
                    .find(|(seat, _)| *seat == p)
                    .map_or(0, |(_, n)| *n as i32)
            }
            Value::HalfGreatestSorceryDamageThisTurn { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return 0 };
                self.sorcery_damage_this_turn
                    .iter()
                    .filter(|(_, caster, _)| *caster == p)
                    .map(|(_, _, dmg)| (dmg / 2) as i32)
                    .max()
                    .unwrap_or(0)
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
            Value::DistinctNamesControlledMatching(filter) => {
                let mut names: Vec<&str> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == ctx.controller
                            && self.evaluate_requirement_static(
                                filter,
                                &crate::game::types::Target::Permanent(c.id),
                                ctx.controller,
                                None,
                            )
                    })
                    .map(|c| c.definition.name)
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
            Value::TurnNumber => self.turn_number as i32,
            Value::DraftNoteNumber { agg } => {
                let notes = &self.players[ctx.controller].draft_notes;
                // A resolving instant/sorcery is already off the stack, so
                // fall back to the cast's stamped name.
                ctx.source
                    .and_then(|id| self.find_card_anywhere(id))
                    .map(|c| c.definition.name)
                    .or(ctx.source_name)
                    .map(|name| match agg {
                        crate::effect::DraftNoteAgg::Max => notes.max_number(name),
                        crate::effect::DraftNoteAgg::Sum => notes.sum_numbers(name),
                    })
                    .unwrap_or(0) as i32
            }
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
                use crate::fxhash::HashMap;
                let mut counts: HashMap<crate::card::CreatureType, i32> = HashMap::default();
                let mut changelings = 0i32;
                for c in self.battlefield.iter().filter(|c| c.controller == ctx.controller) {
                    let Some(cp) = self.computed_permanent(c.id) else { continue };
                    if !cp.card_types().contains(&crate::card::CardType::Creature) {
                        continue;
                    }
                    if cp.keywords().has_kw(&crate::card::Keyword::Changeling) {
                        changelings += 1;
                        continue;
                    }
                    for t in &cp.subtypes().creature_types {
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
                        // CR 901.7 — a face-up plane can carry counters.
                        .or_else(|| self.command_card(cid))
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
                    // sacrificed as a cost (Twitching Doll) or a dead
                    // die-trigger source (Ambitious Augmenter) reads its last
                    // counter total (CR 603.10 / 608.2). Keyword counters
                    // count toward "counters on it" too (CR 122.1).
                    self.battlefield_find(cid)
                        .or_else(|| self.lki_snapshot(cid))
                        .or_else(|| self.died_card_snapshots.get(&cid))
                        .or_else(|| {
                            self.players
                                .iter()
                                .find_map(|p| p.graveyard.iter().find(|c| c.id == cid))
                        })
                        .or_else(|| self.exile.iter().find(|c| c.id == cid))
                        .map(|inst| {
                            (inst.counters.values().sum::<u32>()
                                + inst.keyword_counters.values().sum::<u32>())
                                as i32
                        })
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
            Value::Negate(inner) => -self.evaluate_value(inner, ctx),
            Value::CountersRemovedThisEffect => self.counters_removed_this_effect as i32,
            Value::CountersRemovedAsCost => self.counters_removed_as_cost as i32,
            Value::ExiledForCostManaValue => self.exiled_for_cost_mana_value.unwrap_or(0),
            Value::CardsNamedLikeSourceInAllGraveyards => {
                let Some(name) = ctx
                    .source
                    .and_then(|id| self.find_card_anywhere(id))
                    .map(|c| c.definition.name)
                    .or(ctx.source_name)
                else {
                    return 0;
                };
                self.graveyard_cards_named(name)
            }
            // The Odyssey Shrine cycle — the *cast spell* is the trigger's
            // subject, so read its name rather than the enchantment's.
            Value::CardsNamedLikeTriggerSpellInAllGraveyards => {
                let Some(name) = ctx
                    .trigger_source
                    .and_then(|e| e.as_card_id())
                    .and_then(|id| self.find_card_anywhere(id).map(|c| c.definition.name))
                    .or_else(|| {
                        ctx.trigger_source.and_then(|e| e.as_card_id()).and_then(|id| {
                            self.stack.iter().find_map(|si| match si {
                                crate::game::StackItem::Spell { card, .. } if card.id == id => {
                                    Some(card.definition.name)
                                }
                                _ => None,
                            })
                        })
                    })
                else {
                    return 0;
                };
                self.graveyard_cards_named(name)
            }
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
            Value::CardsDrawnThisEffect => self.cards_drawn_this_resolution as i32,
            Value::EnergyPaidThisEffect => self.energy_paid_this_resolution as i32,
            Value::PermanentsReturnedThisEffect => self.permanents_returned_this_resolution as i32,
            Value::PermanentsTappedThisEffect => self.permanents_tapped_this_resolution as i32,
            Value::CardsRevealedThisEffect => self.cards_revealed_this_resolution as i32,
            Value::LastExiledManaValue => self
                .scratch.exiled_card_ids_this_resolution
                .last()
                .and_then(|id| self.find_card_anywhere(*id))
                .map(|c| c.definition.cost.cmc() as i32)
                .unwrap_or(0),
            Value::MaxCardsDiscardedThisEffectByAnyPlayer => self
                .scratch.cards_discarded_per_player_this_resolution
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
            Value::CardsMilledThisEffectMatching { filter } => self
                .scratch.last_moved_cards
                .iter()
                .filter(|&&cid| {
                    self.players.iter().enumerate().any(|(seat, p)| {
                        p.graveyard.iter().any(|c| {
                            c.id == cid && self.evaluate_requirement_on_card(filter, c, seat)
                        })
                    })
                })
                .count() as i32,
            Value::CreatureCardsMilledThisEffect => self
                .scratch.last_moved_cards
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
            Value::GreatestPowerControlled { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return 0 };
                self.battlefield
                    .iter()
                    .filter(|c| c.controller == p && c.definition.is_creature())
                    .filter_map(|c| self.computed_permanent(c.id).map(|cp| cp.power))
                    .max()
                    .unwrap_or(0)
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
            Value::GreatestSameStoredResult => ctx
                .source
                .and_then(|id| self.battlefield_find(id))
                .map(|c| {
                    let mut best = 0;
                    for face in 1..=20u8 {
                        let n = c.stored_die_results.iter().filter(|r| **r == face).count();
                        best = best.max(n);
                    }
                    best as i32
                })
                .unwrap_or(0),
            // Mana Echoes — "creatures you control that share a creature type
            // with it". Changelings share with everything (CR 702.73a).
            Value::CreaturesSharingTypeWith(subject) => {
                let Some(sid) = self
                    .resolve_selector(subject, ctx)
                    .into_iter()
                    .find_map(|e| e.as_card_id())
                else {
                    return 0;
                };
                let Some(subj) = self.find_card_anywhere(sid) else { return 0 };
                let types = subj.definition.subtypes.creature_types.clone();
                let wild = subj.definition.keywords.has_kw(&crate::card::Keyword::Changeling);
                self.battlefield
                    .iter()
                    .filter(|c| c.controller == ctx.controller && c.definition.is_creature())
                    .filter(|c| {
                        wild || c.definition.keywords.has_kw(&crate::card::Keyword::Changeling)
                            || c.definition
                                .subtypes
                                .creature_types
                                .iter()
                                .any(|t| types.contains(t))
                    })
                    .count() as i32
            }
            Value::LastRevealedManaValue => {
                self.last_revealed_from_hand.map(|(_, mv)| mv).unwrap_or(0) as i32
            }
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
                    // Any zone: a graveyard/exile card has a mana value too
                    // (Necropolis reads the card it exiled as a cost).
                    EntityRef::Permanent(cid) | EntityRef::Card(cid) => {
                        self.find_card_anywhere(cid).map(|c| c.definition.cost.cmc() as i32)
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
                let mut seen: crate::fxhash::HashSet<crate::mana::Color> =
                    crate::fxhash::HashSet::default();
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
                let mut seen: crate::fxhash::HashSet<CardType> =
                    crate::fxhash::HashSet::default();
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
            Value::CardsInAllGraveyardsMatching { filter } => {
                let ids: Vec<CardId> = self
                    .players
                    .iter()
                    .flat_map(|p| p.graveyard.iter())
                    .map(|c| c.id)
                    .collect();
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
            Value::CardsInOpponentsGraveyardsMatching { filter } => {
                let ids: Vec<CardId> = self
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !self.same_team(*i, ctx.controller))
                    .flat_map(|(_, p)| p.graveyard.iter())
                    .map(|c| c.id)
                    .collect();
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
                let mut seen: crate::fxhash::HashSet<CardType> =
                    crate::fxhash::HashSet::default();
                for card in &self.players[p].graveyard {
                    for t in &card.definition.card_types {
                        seen.insert(t.clone());
                    }
                }
                seen.len() as i32
            }
            Value::DistinctCardTypesExiledWith => {
                let Some(src) = ctx.source else { return 0; };
                let mut seen: crate::fxhash::HashSet<CardType> =
                    crate::fxhash::HashSet::default();
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
                                || c.definition.keywords.has_kw(&crate::card::Keyword::Changeling)
                        })
                    })
                    .count() as i32
            }
            // Selvala, Eager Trailblazer — distinct computed powers.
            Value::DistinctPowersAmongCreaturesControlled(p) => {
                let Some(seat) = self.resolve_player(p, ctx) else { return 0 };
                let powers: crate::fxhash::HashSet<i32> = self
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
            Value::OpponentsWhoLostLifeThisTurn => self
                .opponents_of(ctx.controller)
                .into_iter()
                .filter(|p| self.players[*p].lost_life_this_turn)
                .count() as i32,
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
                        c.keywords()
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
            Value::PermanentCountControlledByMatching(p, filter) => self
                .resolve_player(p, ctx)
                .map(|seat| {
                    // `evaluate_requirement_static` (not `..._on_card`) so
                    // battlefield-state filters like `Untapped` are live.
                    self.battlefield
                        .iter()
                        .filter(|c| {
                            c.controller == seat
                                && self.evaluate_requirement_static(
                                    filter,
                                    &crate::game::types::Target::Permanent(c.id),
                                    seat,
                                    None,
                                )
                        })
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
                self.greatest_shared_type_count(ctx.controller) as i32
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
            Value::RememberedAmountOfSource => {
                let Some(src) = ctx.source else { return 0 };
                self.find_card_anywhere(src)
                    .or_else(|| self.died_card_snapshots.get(&src))
                    .and_then(|c| c.remembered_amount)
                    .unwrap_or(0)
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

    /// The printed colours of the object in target slot `slot`, wherever it
    /// currently is (battlefield, stack, or a terminal zone). `None` when the
    /// slot is empty or holds a player.
    fn target_colors(&self, ctx: &EffectContext, slot: u8) -> Option<Vec<crate::mana::Color>> {
        let Some(crate::game::types::Target::Permanent(cid)) = ctx.targets.get(slot as usize)
        else {
            return None;
        };
        self.find_card_anywhere(*cid)
            .or_else(|| {
                self.stack.iter().find_map(|si| match si {
                    crate::game::types::StackItem::Spell { card, .. } if card.id == *cid => {
                        Some(&**card)
                    }
                    _ => None,
                })
            })
            .map(|c| c.definition.printed_colors())
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
            Predicate::AllMatchingShareAColor(filter) => {
                // Common Cause — intersect the colours of every match; a
                // colourless one empties the set immediately.
                let mut shared: Option<Vec<crate::mana::Color>> = None;
                for c in &self.battlefield {
                    if !self.evaluate_requirement_static(
                        filter,
                        &crate::game::types::Target::Permanent(c.id),
                        c.controller,
                        ctx.source,
                    ) {
                        continue;
                    }
                    // Printed colours, not computed: this predicate gates an
                    // anthem, and asking the layer system here would recurse.
                    let colors = c.definition.printed_colors();
                    shared = Some(match shared {
                        None => colors,
                        Some(prev) => prev.into_iter().filter(|k| colors.contains(k)).collect(),
                    });
                    if shared.as_ref().is_some_and(|s| s.is_empty()) {
                        return false;
                    }
                }
                true
            }
            Predicate::ColorIsMostCommonAmongPermanents(k) => {
                self.most_common_permanent_colors().contains(k)
            }
            Predicate::TappedLandForManaThisTurn(who) => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].tapped_land_for_mana_this_turn),
            Predicate::ControlsLandOfEachBasicType(who) => {
                use crate::card::LandType::*;
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                [Plains, Island, Swamp, Mountain, Forest].iter().all(|t| {
                    self.battlefield.iter().any(|c| {
                        c.controller == p
                            && self
                                .computed_permanent(c.id)
                                .is_some_and(|cp| cp.subtypes().land_types.contains(t))
                    })
                })
            }
            Predicate::ControlsCreatureOfEachColor(who) => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                crate::mana::Color::ALL.iter().all(|k| {
                    self.battlefield.iter().any(|c| {
                        c.controller == p
                            && self.computed_permanent(c.id).is_some_and(|cp| {
                                cp.card_types().contains(&crate::card::CardType::Creature)
                                    && cp.colors.contains(k)
                            })
                    })
                })
            }
            Predicate::ValueAtLeast(a, b) => self.evaluate_value(a, ctx) >= self.evaluate_value(b, ctx),
            Predicate::ValueAtMost(a, b) => self.evaluate_value(a, ctx) <= self.evaluate_value(b, ctx),
            Predicate::ValueEquals(a, b) => self.evaluate_value(a, ctx) == self.evaluate_value(b, ctx),
            Predicate::ValueIsOdd(v) => self.evaluate_value(v, ctx).rem_euclid(2) == 1,
            Predicate::ValueIsPrime(v) => {
                let n = self.evaluate_value(v, ctx) as i64;
                n >= 2 && (2..=((n as f64).sqrt() as i64)).all(|d| n % d != 0)
            }
            // CR 105 — Dead Ringers' "unless either one is a color the other
            // isn't": both slots must carry exactly the same colour set.
            Predicate::TargetsHaveIdenticalColors(a, b) => {
                match (self.target_colors(ctx, *a), self.target_colors(ctx, *b)) {
                    (Some(x), Some(y)) => {
                        x.iter().all(|c| y.contains(c)) && y.iter().all(|c| x.contains(c))
                    }
                    _ => false,
                }
            }
            Predicate::TargetSharesColorWithControlled { slot, filter } => {
                let Some(colors) = self.target_colors(ctx, *slot) else { return false };
                !colors.is_empty()
                    && self.battlefield.iter().any(|c| {
                        c.controller == ctx.controller
                            && self.evaluate_requirement_on_card(filter, c, ctx.controller)
                            && c.definition.printed_colors().iter().any(|x| colors.contains(x))
                    })
            }
            Predicate::PlayerSacrificedThisResolution(pref) => self
                .resolve_player(pref, ctx)
                .is_some_and(|p| self.scratch.players_sacrificed_this_resolution.contains(&p)),
            Predicate::ExcessDamageDealtThisResolution => self.excess_damage_this_resolution > 0,
            Predicate::IsTurnOf(pref) => self.resolve_player(pref, ctx) == Some(self.active_player_idx),
            Predicate::SamePlayer(a, b) => {
                match (self.resolve_player(a, ctx), self.resolve_player(b, ctx)) {
                    (Some(x), Some(y)) => x == y,
                    _ => false,
                }
            }
            Predicate::PlayerIsOpponent { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| !self.same_team(p, ctx.controller)),
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
            // An **unbound target slot** is "no entity", not "every entity".
            // `all` over an empty selector is vacuously true, so
            // `If { cond: EntityMatches { Target(0), … } }` on an ability
            // whose target was never chosen ran its `then` branch — Eagle of
            // Deliverance drew a card off a counter it had put on nothing.
            // Scoped to the slot: this predicate is also written over
            // `EachPermanent(…)` as a plain existence test, where the empty
            // set is a *separate* open defect (see ENGINE_BACKLOG).
            Predicate::EntityMatches { what, filter } => {
                let ents = self.resolve_selector(what, ctx);
                !ents.is_empty()
                    && ents.into_iter().all(|e| match e {
                        EntityRef::Permanent(cid) | EntityRef::Card(cid) => self
                            .evaluate_requirement_static(
                                filter,
                                &Target::Permanent(cid),
                                ctx.controller,
                                ctx.source,
                            ),
                        EntityRef::Player(_) => matches!(filter, SelectionRequirement::Player),
                    })
            }
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
            Predicate::WasMonarchAtTurnStart { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.monarch_at_turn_start == Some(p)),
            Predicate::HasInitiative { who } => self
                .resolve_players(who, ctx)
                .into_iter()
                .any(|p| self.initiative == Some(p)),
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
            // "a permanent OTHER than this one" — Last Laugh's self-gate.
            Predicate::TriggerSourceIsSelf => {
                match (ctx.source, ctx.trigger_source) {
                    (Some(me), Some(e)) => e.as_card_id() == Some(me),
                    _ => false,
                }
            }
            Predicate::TriggerSourceIsSourceHost => {
                let host = ctx
                    .source
                    .and_then(|s| self.battlefield_find(s))
                    .and_then(|c| c.attached_to);
                match (host, ctx.trigger_source) {
                    (Some(want), Some(e)) => e.as_card_id() == Some(want),
                    _ => false,
                }
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
                .and_then(|cid| self.battlefield.find_by_id(cid))
                .map(|c| c.attacked_this_turn)
                .unwrap_or(false),
            Predicate::IsExtraTurn => self.current_turn_is_extra,
            Predicate::SourceIsMonstrous => ctx
                .source
                .and_then(|cid| self.battlefield.find_by_id(cid))
                .map(|c| c.monstrous)
                .unwrap_or(false),
            Predicate::SourceIsRenowned => ctx
                .source
                .and_then(|cid| self.battlefield.find_by_id(cid))
                .map(|c| c.renowned)
                .unwrap_or(false),
            Predicate::SourceIsEquipped => {
                ctx.source.is_some_and(|cid| self.attached_equipment_count(cid) > 0)
            }
            Predicate::SourceIsSuspected => ctx
                .source
                .and_then(|cid| self.battlefield.find_by_id(cid))
                .map(|c| c.suspected)
                .unwrap_or(false),
            Predicate::SourceIsBestowedAura => ctx
                .source
                .and_then(|cid| self.battlefield.find_by_id(cid))
                .map(|c| c.bestowed)
                .unwrap_or(false),
            Predicate::SourceOnBattlefield => {
                ctx.source.is_some_and(|cid| self.battlefield_find(cid).is_some())
            }
            // Wall of Caltrops — every blocker on the creature the source is
            // blocking matches `filter`, and at least one of them isn't the
            // source.
            Predicate::SourceCoBlockersAllMatch { filter } => {
                let Some(src) = ctx.source else { return false };
                let Some(attackers) = self.block_map.get(&src) else { return false };
                attackers.iter().any(|atk| {
                    let peers: Vec<CardId> = self
                        .block_map
                        .iter()
                        .filter(|(_, blocked)| blocked.contains(atk))
                        .map(|(b, _)| *b)
                        .collect();
                    peers.iter().any(|b| *b != src)
                        && peers.iter().all(|b| {
                            self.battlefield_find(*b).is_some_and(|c| {
                                self.evaluate_requirement_on_card(filter, c, c.controller)
                            })
                        })
                })
            }
            Predicate::SourceIsCreature => ctx
                .source
                .and_then(|cid| self.computed_permanent(cid))
                .map(|c| c.card_types().contains(&crate::card::CardType::Creature))
                .unwrap_or(false),
            Predicate::SourceSaddled => ctx
                .source
                .and_then(|cid| self.battlefield.find_by_id(cid))
                .map(|c| c.saddled)
                .unwrap_or(false),
            Predicate::SourceCastFromEscape => ctx
                .source
                .and_then(|cid| self.battlefield.find_by_id(cid))
                .map(|c| c.cast_from_escape)
                .unwrap_or(false),
            Predicate::SourceWasCast => ctx
                .source
                .and_then(|cid| self.battlefield.find_by_id(cid))
                .map(|c| {
                    !c.is_token
                        && (c.cast_from_hand
                            || c.cast_from_exile
                            || c.cast_via_flashback
                            || c.cast_from_suspend
                            || c.cast_from_escape)
                })
                .unwrap_or(false),
            Predicate::SourceCastFromOwnersHand => ctx
                .source
                .and_then(|cid| self.battlefield.find_by_id(cid))
                .is_some_and(|c| !c.is_token && c.cast_from_hand),
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
                    self.scratch.nonland_cards_discarded_per_player_this_resolution
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
                    self.scratch.cards_discarded_per_player_this_resolution
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
            Predicate::OpponentCastSpellSinceYourTurn { who } => self
                .resolve_players(who, ctx)
                .iter()
                .any(|&p| self.opponent_cast_since_your_turn & crate::game::seat_bit(p) != 0),
            Predicate::SearchedLibraryThisTurn { who } => self
                .resolve_players(who, ctx)
                .iter()
                .any(|&p| self.players[p].searched_library_this_turn),
            Predicate::ProwlTypeDealtCombatDamage { types } => {
                let pl = &self.players[ctx.controller];
                pl.prowl_any_type_this_turn
                    || types.iter().any(|t| pl.prowl_types_this_turn.contains(t))
            }
            Predicate::CreatureCardsToGraveyardThisTurnAtLeast(n) => {
                self.players.iter().map(|p| p.creature_cards_to_graveyard_this_turn).sum::<u32>()
                    >= *n
            }
            Predicate::SourcesYouControlledDealtDamageThisTurnAtLeast(n) => {
                self.damage_sources_this_turn
                    .iter()
                    .filter(|(seat, _)| *seat == ctx.controller)
                    .count() as u32
                    >= *n
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
            Predicate::CreatureDiedThisTurnMatching { filter } => self
                .creature_deaths_this_turn
                .iter()
                .any(|c| self.evaluate_requirement_on_card(filter, c, ctx.controller)),
            Predicate::DistinctCounterKindsAmongCreaturesAtLeast { who, at_least } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                let mut kinds = crate::fxhash::HashSet::default();
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
            Predicate::CardsInExileAtLeast(n) => self.exile.len() as u32 >= *n,
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
            Predicate::CastSpellTargetsOnlyOneMatching(filter) => {
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                let Some((target, extra)) = self.stack.iter().find_map(|si| match si {
                    StackItem::Spell { card, target, additional_targets, .. } if card.id == cid => {
                        Some((target.clone(), additional_targets.clone()))
                    }
                    _ => None,
                }) else {
                    return false;
                };
                match (target, extra.is_empty()) {
                    (Some(t), true) => {
                        self.evaluate_requirement_static(filter, &t, ctx.controller, ctx.source)
                    }
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
            Predicate::CastSpellFirstMatchingThisTurn(filter) => {
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                let caster = self
                    .stack
                    .iter()
                    .find_map(|si| match si {
                        StackItem::Spell { card, .. } if card.id == cid => Some(card.controller),
                        _ => None,
                    })
                    .unwrap_or(ctx.controller);
                self.players[caster]
                    .spell_ids_cast_this_turn
                    .iter()
                    .find(|id| {
                        self.find_card_anywhere(**id).is_some_and(|c| {
                            self.evaluate_requirement_on_card(filter, c, caster)
                        })
                    })
                    == Some(&cid)
            }
            Predicate::AuraHostIsCheaperOpponentPermanent => {
                let Some(EntityRef::Permanent(aura_id)) = ctx.trigger_source else {
                    return false;
                };
                let Some(aura) = self.battlefield_find(aura_id) else { return false };
                let Some(host) = aura.attached_to.and_then(|h| self.battlefield_find(h)) else {
                    return false;
                };
                !host.definition.is_land()
                    && self.opponents_of(ctx.controller).contains(&host.controller)
                    && host.definition.cost.cmc() <= aura.definition.cost.cmc()
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
            Predicate::CastSpellNotOwnedByCaster => {
                // The caster-relative sibling of `CastSpellNotOwnedByYou`:
                // both sides are read off the stack item, so an `AnyPlayer`
                // listener (Gonti, Night Minister) judges the real caster
                // rather than its own controller.
                let Some(EntityRef::Card(cid)) = ctx.trigger_source else {
                    return false;
                };
                self.stack.iter().any(|si| match si {
                    StackItem::Spell { card, caster, .. } if card.id == cid => {
                        card.owner != *caster
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
                    crate::card::Zone::Ante => self.players[seat]
                        .ante
                        .iter()
                        .filter(|c| c.definition.name == target_name)
                        .count(),
                    crate::card::Zone::Stack | crate::card::Zone::Command => 0,
                };
                count >= n
            }
            Predicate::CausedByOpponentSpellOrAbility => self
                .resolution_causer
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
            Predicate::SpellWasKickedWith(n) => ctx.kicked_options.contains(n),
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
                let powers: crate::fxhash::HashSet<i32> = self
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
            Predicate::DistinctUnlockedDoorNamesAtLeast { who, count } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                let mut names: Vec<&str> = Vec::new();
                for c in self.battlefield.iter().filter(|c| c.controller == p) {
                    let Some(doors) = c.definition.room.as_ref() else { continue };
                    for (bit, door) in [(1u8, &doors.left), (2u8, &doors.right)] {
                        if c.unlocked_doors & bit != 0 && !names.contains(&door.name.as_str()) {
                            names.push(door.name.as_str());
                        }
                    }
                }
                names.len() as u32 >= *count
            }
            Predicate::ControlsOutlaw { who } => {
                let Some(p) = self.resolve_player(who, ctx) else { return false };
                self.battlefield.iter().any(|c| c.controller == p && card_is_outlaw(c))
            }
            Predicate::RevoltActive { who } => self
                .resolve_player(who, ctx)
                .is_some_and(|p| self.players[p].permanent_left_battlefield_this_turn),
            Predicate::AnyPlayerControlsNoCreatures => (0..self.players.len()).any(|seat| {
                !self.battlefield.iter().any(|c| c.controller == seat && c.definition.is_creature())
            }),
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
                // CR 613 — "three or more artifacts" counts the *computed*
                // type line, so Mycosynth Lattice turns Metalcraft on. The
                // layer read is second and gated: the printed answer is
                // already `true` for a real artifact, and
                // `card_type_change_unscoped` (memo-backed) is `false` on
                // almost every board, where this is the printed scan it
                // replaces plus one load. Counts up to three and stops.
                let typed = self.card_type_change_unscoped();
                let mut n = 0;
                for c in self.battlefield.iter().filter(|c| c.controller == p) {
                    let artifact = c.definition.is_artifact()
                        || (typed
                            && self.computed_permanent(c.id).is_some_and(|cp| {
                                cp.card_types().contains(&crate::card::CardType::Artifact)
                            }));
                    if artifact {
                        n += 1;
                        if n >= 3 {
                            break;
                        }
                    }
                }
                n >= 3
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

    /// CR 601.2c — check the `SameControllerAsTargetSlot` constraints in `req`
    /// against an explicit slot vector, for the `&self` auto-target walkers
    /// that can't stamp `target_slots_scratch`.
    pub(crate) fn cross_slot_targets_ok(
        &self,
        req: &SelectionRequirement,
        target: &Target,
        slots: &[Option<Target>],
    ) -> bool {
        use SelectionRequirement as R;
        match req {
            R::And(a, b) => {
                self.cross_slot_targets_ok(a, target, slots)
                    && self.cross_slot_targets_ok(b, target, slots)
            }
            R::SameControllerAsTargetSlot(slot) => {
                let ctrl_of = |t: &Target| match t {
                    Target::Permanent(cid) => self
                        .battlefield_find(*cid)
                        .map(|c| c.controller)
                        .or_else(|| self.stack_spell_caster(*cid)),
                    Target::Player(p) => Some(*p),
                };
                match slots.get(*slot as usize).and_then(|t| t.as_ref()) {
                    Some(other) => ctrl_of(other).is_some() && ctrl_of(other) == ctrl_of(target),
                    None => true,
                }
            }
            _ => true,
        }
    }

    /// CR 400.7 — does `def` share a card type with a card this source
    /// exiled (`exiled_with == source`)? Holistic Wisdom's return gate.
    pub(crate) fn shares_card_type_with_exiled_by(
        &self,
        source: Option<crate::card::CardId>,
        def: &crate::card::CardDefinition,
    ) -> bool {
        let Some(src) = source else { return false };
        self.exile
            .iter()
            .filter(|c| c.exiled_with == Some(src))
            .any(|c| c.definition.card_types.iter().any(|t| def.card_types.contains(t)))
    }

    /// Cards in every graveyard counting as named `name` — the card's own
    /// name, or its `counts_as_named_in_graveyard` alias (Pardic Firecat
    /// counting as Flame Burst).
    fn graveyard_cards_named(&self, name: &str) -> i32 {
        self.players
            .iter()
            .flat_map(|p| p.graveyard.iter())
            .filter(|c| {
                c.definition.name == name
                    || c.definition.counts_as_named_in_graveyard.is_some_and(|a| a == name)
            })
            .count() as i32
    }

    /// Whether `seat` controls at least one land of `lt` — the reading behind
    /// `SelectionRequirement::ControllerControlsLandType` and the Homelands
    /// Island-gated combat riders.
    pub(crate) fn seat_controls_land_type(&self, seat: usize, lt: crate::card::LandType) -> bool {
        self.battlefield
            .iter()
            .any(|c| c.controller == seat && c.definition.subtypes.land_types.contains(&lt))
    }

    /// CR 702.121 — how many distinct players are being attacked this combat.
    /// Shared by melee's pump and Custodi Soulcaller's mana-value gate.
    pub(crate) fn opponents_attacked_this_combat(&self) -> u32 {
        use crate::game::types::AttackTarget;
        let mut seats = crate::fxhash::HashSet::default();
        for atk in &self.attacking {
            let defender = match atk.target {
                AttackTarget::Player(p) => Some(p),
                AttackTarget::Planeswalker(id) => self.battlefield_find(id).map(|c| c.controller),
                AttackTarget::Battle(id) => self.battlefield_find(id).and_then(|c| c.protected_by),
            };
            if let Some(seat) = defender {
                seats.insert(seat);
            }
        }
        seats.len() as u32
    }

    pub fn evaluate_requirement_static(
        &self,
        req: &SelectionRequirement,
        target: &Target,
        controller: usize,
        source: Option<CardId>,
    ) -> bool {
        self.evaluate_requirement_static_hinted(req, target, controller, source, None)
    }

    /// Same question with the battlefield permanent already in hand.
    ///
    /// Every `Target::Permanent` arm below opens by locating the object, and
    /// the located-on-the-battlefield case is a linear `battlefield_find`.
    /// The callers that dominate the traffic are *walking the battlefield*
    /// when they ask — `auto_targets_for_effect_all_slots`' candidate loop,
    /// the counting filters — so they hand the permanent over instead of
    /// making this re-find it by id.
    pub fn evaluate_requirement_static_on(
        &self,
        req: &SelectionRequirement,
        card: &CardInstance,
        controller: usize,
        source: Option<CardId>,
    ) -> bool {
        // The hint is only equivalent to the walk if `card` *is* the
        // battlefield permanent with that id — a graveyard or hand card
        // handed in here would take the battlefield branch (and the layer
        // view with it). Debug-only, so the 18.7k-test suite is the proof.
        debug_assert!(
            self.battlefield_find(card.id).is_some_and(|c| std::ptr::eq(&**c, &**card)),
            "evaluate_requirement_static_on: hint is not the battlefield permanent it names",
        );
        self.evaluate_requirement_static_hinted(
            req,
            &Target::Permanent(card.id),
            controller,
            source,
            Some(card),
        )
    }

    /// [`evaluate_requirement_static_on`](Self::evaluate_requirement_static_on)
    /// for a caller that asks one requirement of many battlefield permanents:
    /// the printed-line evaluator answers first and the walker only the
    /// shapes it declines. `gates` is the caller's per-walk memo of the three
    /// presence gates; build one per requirement, never per permanent.
    ///
    /// The walker's frame is the cost — ~150-300 Ir a call for what a target
    /// filter usually is (a card type, a controller, a subtype), and the
    /// targeting enumerator and `resolve_selector`'s `EachPermanent` arm ask
    /// it 170 k times a six-game `cube` run (PERF `(-183)`). Every fast
    /// answer is checked against the walker under `debug_assertions`, so the
    /// suite and `scripts/robustness_grid.sh` are the ratchet that keeps
    /// [`printed_requirement`](Self::printed_requirement) in step with it.
    #[inline]
    pub(crate) fn requirement_on_permanent(
        &self,
        req: &SelectionRequirement,
        card: &CardInstance,
        controller: usize,
        source: Option<CardId>,
        gates: &PrintedGates,
    ) -> bool {
        match self.printed_requirement(req, card, controller, source, gates) {
            Some(p) => {
                debug_assert_eq!(
                    p,
                    self.evaluate_requirement_static_on(req, card, controller, source),
                    "printed_requirement disagrees with the requirement walker"
                );
                p
            }
            None => self.evaluate_requirement_static_on(req, card, controller, source),
        }
    }

    /// The same entry for a card the caller holds in a graveyard. The walker
    /// asked about a graveyard card pays a `battlefield_find` miss and then
    /// `requirement_card_off_battlefield`'s chain to re-find it; this reads
    /// the same object the chain would return — a death snapshot or the
    /// leaves-battlefield LKI first, the graveyard card itself otherwise — and
    /// evaluates it in the off-battlefield zone (PERF `(-185)`).
    pub(crate) fn requirement_on_graveyard_card(
        &self,
        req: &SelectionRequirement,
        card: &CardInstance,
        controller: usize,
        source: Option<CardId>,
    ) -> bool {
        let cid = card.id;
        let walk = || self.evaluate_requirement_static(req, &Target::Permanent(cid), controller, source);
        // The walker's lookup order, minus the battlefield miss: an id names
        // one object and this one is in a graveyard.
        let obj = match self.died_card_snapshots.get(&cid) {
            Some(snap) => snap,
            None => match self.leaves_bf_lki.get(&cid) {
                Some(lki) if self.resolving_lki_source == Some(cid) => lki,
                _ => card,
            },
        };
        match self.printed_requirement_impl::<true>(req, obj, controller, source, &PrintedGates::default()) {
            Some(p) => {
                debug_assert_eq!(p, walk(), "printed_requirement (off battlefield) disagrees with the walker");
                p
            }
            None => walk(),
        }
    }

    /// The printed-line twin of the walker's `Target::Permanent` block, for a
    /// permanent the caller holds: three-valued, `None` meaning "ask the
    /// walker". A leaf is answered here only where the walker itself reads
    /// the printed line — the card-type family behind the layer-4 presence
    /// gate and `bestowed`, the subtype families behind theirs — or an
    /// instance field / zone scan it would read the same way. `And`/`Or`
    /// short-circuit on a known side, so `And(IsSpellOnStack, …)` is `false`
    /// on the battlefield without touching the unknown half.
    ///
    /// **Every arm here is a copy of a walker arm and must read what it
    /// reads.** `requirement_on_permanent`'s `debug_assert_eq!` is the audit;
    /// a leaf that is not a bit-for-bit restatement of its walker arm stays
    /// out (`PowerAtMost` reads the computed view; `HasKeyword` the instance
    /// grants — both walker-only).
    pub(crate) fn printed_requirement(
        &self,
        req: &SelectionRequirement,
        card: &CardInstance,
        controller: usize,
        source: Option<CardId>,
        gates: &PrintedGates,
    ) -> Option<bool> {
        self.printed_requirement_impl::<false>(req, card, controller, source, gates)
    }

    /// The body of [`printed_requirement`](Self::printed_requirement). `OFF`
    /// is which of the walker's two `Target::Permanent` paths it stands in
    /// for: the battlefield permanent (layer gates, `bestowed`, the
    /// controller off the object) or a card the walker finds off the
    /// battlefield (printed types unconditionally, a controller only through
    /// the stack or a death snapshot). A const parameter and not a value:
    /// the evaluator recurses per node, and a seventh argument spilled on
    /// every call (+5.1 M Ir on `cube`) while a runtime flag cost six
    /// instructions a call across the 465 k calls the battlefield sites
    /// already made (`(-185)`).
    fn printed_requirement_impl<const OFF: bool>(
        &self,
        req: &SelectionRequirement,
        card: &CardInstance,
        controller: usize,
        source: Option<CardId>,
        gates: &PrintedGates,
    ) -> Option<bool> {
        use SelectionRequirement as R;
        let cid = card.id;
        let on_bf = !OFF;
        // The walker's `has_type`: on the battlefield, printed unless a
        // layer-4 card-type source is in scope or the permanent is bestowed;
        // off it, printed unconditionally.
        let card_type = |t: CardType| -> Option<bool> {
            if on_bf && (card.bestowed || gates.card(self)) {
                return None;
            }
            Some(card.definition.card_types.contains(&t))
        };
        match req {
            R::Any => Some(true),
            R::Player => Some(false),
            R::And(a, b) => {
                let x = self.printed_requirement_impl::<OFF>(a, card, controller, source, gates);
                if x == Some(false) {
                    return Some(false);
                }
                let y = self.printed_requirement_impl::<OFF>(b, card, controller, source, gates);
                match (x, y) {
                    (Some(true), y) => y,
                    (None, Some(false)) => Some(false),
                    _ => None,
                }
            }
            R::Or(a, b) => {
                let x = self.printed_requirement_impl::<OFF>(a, card, controller, source, gates);
                if x == Some(true) {
                    return Some(true);
                }
                let y = self.printed_requirement_impl::<OFF>(b, card, controller, source, gates);
                match (x, y) {
                    (Some(false), y) => y,
                    (None, Some(true)) => Some(true),
                    _ => None,
                }
            }
            R::Not(inner) => self
                .printed_requirement_impl::<OFF>(inner, card, controller, source, gates)
                .map(|v| !v),
            // Off the battlefield the walker knows a controller only through
            // the stack (a spell's caster) or the CR 603.10 death snapshot.
            R::ControlledByYou => Some(if on_bf {
                card.controller == controller
            } else {
                self.stack_spell_caster(cid)
                    .or_else(|| self.died_card_snapshots.get(&cid).map(|c| c.controller))
                    == Some(controller)
            }),
            R::ControlledByOpponent => Some(if on_bf {
                !self.same_team(card.controller, controller)
            } else {
                self.stack_spell_caster(cid).is_some_and(|c| !self.same_team(c, controller))
            }),
            R::ControlledByActivePlayer => Some(on_bf && card.controller == self.active_player_idx),
            // Ownership is stable across zones; the walker's `find_card_anywhere`
            // lands on this object.
            R::OwnedByYou => Some(card.owner == controller),
            R::Permanent => Some(card.definition.is_permanent()),
            // CR 604.3 — Grist's `creature_off_battlefield` clause is the
            // walker's `Creature` arm only (`Noncreature` is `!has_type`):
            // `true` off the battlefield, the walker's question on it.
            R::Creature => {
                let grist = card.definition.creature_off_battlefield;
                if on_bf && grist {
                    return None;
                }
                card_type(CardType::Creature).map(|v| v || (grist && !on_bf))
            }
            R::Land => card_type(CardType::Land),
            R::Artifact => card_type(CardType::Artifact),
            R::Enchantment => card_type(CardType::Enchantment),
            R::Planeswalker => card_type(CardType::Planeswalker),
            R::Nonland => card_type(CardType::Land).map(|v| !v),
            R::Noncreature => card_type(CardType::Creature).map(|v| !v),
            // CR 715 / 702.183 — the walker reads the adventure/omen half's
            // types here and never the layer view.
            R::HasCardType(ct) => Some(match card.alt_spell_half() {
                Some(half) => half.card_types.contains(ct),
                None => card.definition.card_types.contains(ct),
            }),
            // Off the battlefield both subtype arms end on the printed line
            // whatever the gate says: `computed()` is `None` there and the
            // shallow read needs a battlefield permanent too.
            R::HasCreatureType(ct) => {
                if on_bf && gates.creature(self) {
                    return None;
                }
                Some(
                    card.definition.subtypes.creature_types.contains(ct)
                        || card.has_keyword(&crate::card::Keyword::Changeling),
                )
            }
            R::HasLandType(lt) => {
                if on_bf && gates.land(self) {
                    return None;
                }
                Some(card.definition.subtypes.land_types.contains(lt))
            }
            R::HasColor(c) => Some(card.definition.printed_colors().contains(c)),
            R::IsToken => Some(card.is_token),
            R::NotToken => Some(!card.is_token),
            R::Tapped => Some(card.tapped),
            R::Untapped => Some(!card.tapped),
            R::EnteredThisTurn => Some(card.entered_turn == Some(self.turn_number)),
            R::IsAttacking => Some(self.attacking.iter().any(|a| a.attacker == cid)),
            R::OtherThanSource => Some(source.is_none_or(|s| cid != s)),
            R::IsSource => Some(source == Some(cid)),
            R::IsSpellOnStack => Some(self.stack.iter().any(
                |si| matches!(si, StackItem::Spell { card: c, .. } if c.id == cid),
            )),
            R::InYourGraveyard => Some(
                self.players.get(controller).is_some_and(|p| p.graveyard.iter().any(|c| c.id == cid)),
            ),
            R::InOpponentGraveyard => Some(
                self.players
                    .iter()
                    .enumerate()
                    .any(|(i, p)| i != controller && p.graveyard.iter().any(|c| c.id == cid)),
            ),
            _ => None,
        }
    }

    /// The hint stands in for `battlefield_find` only when it names the id
    /// being asked about; a mismatch falls back to the walk, so a wrong hint
    /// costs nothing but the compare.
    #[inline]
    fn bf_hint_or_find<'a>(
        &'a self,
        cid: CardId,
        hint: Option<&'a CardInstance>,
    ) -> Option<&'a CardInstance> {
        match hint {
            Some(c) if c.id == cid => Some(c),
            _ => self.battlefield_find(cid),
        }
    }

    /// The off-battlefield half of the requirement walker's card lookup:
    /// the two LKI caches, then graveyards, exile, the stack, libraries and
    /// hands. Out of line and behind the battlefield answer — the walk stops
    /// at the battlefield on almost every call, and the chain that looks past
    /// it is eight `Option::or_else` calls, none of them inlined here.
    ///
    /// Dies-trigger filters (Felisa's "with a +1/+1 counter on it") read the
    /// dying object's last-known battlefield state, not the counter-stripped
    /// graveyard copy (CR 603.10 LKI; CR 122.2 cleared the counters on the
    /// zone change), so the LKI caches come before the terminal zones.
    /// Library / hand are needed by "look at top of library" predicates
    /// (Lurking Predators) and discard-from-hand pickers; hidden-zone reads
    /// are permission-checked at the call site.
    #[inline(never)]
    fn requirement_card_off_battlefield(&self, cid: CardId) -> Option<&CardInstance> {
        if let Some(c) = self.died_card_snapshots.get(&cid) {
            return Some(c);
        }
        if self.resolving_lki_source == Some(cid)
            && let Some(c) = self.leaves_bf_lki.get(&cid)
        {
            return Some(c);
        }
        if let Some(c) = self.players.iter().find_map(|p| p.graveyard.iter().find(|c| c.id == cid)) {
            return Some(c);
        }
        if let Some(c) = self.exile.iter().find(|c| c.id == cid) {
            return Some(c);
        }
        if let Some(c) = self.stack.iter().find_map(|si| match si {
            StackItem::Spell { card, .. } if card.id == cid => Some(&**card),
            _ => None,
        }) {
            return Some(c);
        }
        if let Some(c) = self.players.iter().find_map(|p| p.library.iter().find(|c| c.id == cid)) {
            return Some(c);
        }
        self.players.iter().find_map(|p| p.hand.iter().find(|c| c.id == cid))
    }

    fn evaluate_requirement_static_hinted<'a>(
        &'a self,
        req: &SelectionRequirement,
        target: &Target,
        controller: usize,
        source: Option<CardId>,
        hint: Option<&'a CardInstance>,
    ) -> bool {
        use SelectionRequirement as R;
        #[cfg(feature = "trig-census")]
        if crate::zone::trig_census::on() {
            crate::zone::req_census::call();
            if matches!(req, R::And(..) | R::Or(..) | R::Not(..)) {
                crate::zone::req_census::combinator();
            }
        }
        // PERF (-126): tag each recursive call by whether the child is itself
        // a combinator. `nested` is what a flattened `All`/`Any` could
        // collapse; its complement is what an inline leaf path could serve.
        let tick = |_child: &SelectionRequirement| {
            #[cfg(feature = "trig-census")]
            if crate::zone::trig_census::on() {
                crate::zone::req_census::child(matches!(
                    _child,
                    R::And(..) | R::Or(..) | R::Not(..)
                ));
            }
        };
        match req {
            R::Any => true,
            R::Player => matches!(target, Target::Player(_)),
            // Fire and Brimstone — "target player who attacked this turn".
            R::PlayerAttackedThisTurn => {
                matches!(target, Target::Player(p) if self.players[*p].attacked_this_turn)
            }
            R::OpponentPlayer => {
                matches!(target, Target::Player(p) if !self.same_team(*p, controller))
            }
            R::YouPlayer => matches!(target, Target::Player(p) if *p == controller),
            R::OpponentTallyDiffers { what, by, fewer } => {
                let Target::Player(p) = target else { return false };
                if self.same_team(*p, controller) {
                    return false;
                }
                let diff = if *fewer {
                    self.player_tally(controller, *what) - self.player_tally(*p, *what)
                } else {
                    self.player_tally(*p, *what) - self.player_tally(controller, *what)
                };
                diff >= *by as i64
            }
            R::And(a, b) => {
                tick(a);
                self.evaluate_requirement_static_hinted(a, target, controller, source, hint) && {
                    tick(b);
                    self.evaluate_requirement_static_hinted(b, target, controller, source, hint)
                }
            }
            R::Or(a, b) => {
                tick(a);
                self.evaluate_requirement_static_hinted(a, target, controller, source, hint) || {
                    tick(b);
                    self.evaluate_requirement_static_hinted(b, target, controller, source, hint)
                }
            }
            R::Not(inner) => {
                tick(inner);
                !self.evaluate_requirement_static_hinted(inner, target, controller, source, hint)
            }
            R::ControlledByYou => match target {
                // A `Target::Permanent` can also address a spell on the stack
                // (a "copy target spell you control" ability); its caster is
                // its controller. When the object has left the battlefield
                // (a die-trigger reading "a creature you control dies" off a
                // graveyard source), fall back to the CR 603.10 last-known
                // controller in `died_card_snapshots`.
                // The battlefield answers first and the two fallbacks are
                // out-of-line `or_else` calls in this build, so they sit
                // behind a branch rather than in the chain.
                Target::Permanent(cid) => {
                    let ctrl = match self.bf_hint_or_find(*cid, hint) {
                        Some(c) => Some(c.controller),
                        None => self
                            .stack_spell_caster(*cid)
                            .or_else(|| self.died_card_snapshots.get(cid).map(|c| c.controller)),
                    };
                    ctrl == Some(controller)
                }
                Target::Player(p) => *p == controller,
            },
            // CR 601.2c — "controlled by the same player as slot N". The
            // already-chosen slots live in `target_slots_scratch`, stamped by
            // the cast/activation validator; an unstamped slot passes.
            R::SameControllerAsTargetSlot(slot) => {
                let ctrl_of = |t: &Target| match t {
                    Target::Permanent(cid) => match self.bf_hint_or_find(*cid, hint) {
                        Some(c) => Some(c.controller),
                        None => self.stack_spell_caster(*cid),
                    },
                    Target::Player(p) => Some(*p),
                };
                match self.scratch.target_slots_scratch.get(*slot as usize).and_then(|t| t.as_ref()) {
                    Some(other) => ctrl_of(other).is_some() && ctrl_of(other) == ctrl_of(target),
                    None => true,
                }
            }
            // Backdraft — "choose a player who cast one or more sorcery
            // spells this turn".
            R::CastSorceryThisTurn => matches!(target, Target::Player(p)
                if self.players.get(*p).is_some_and(|pl| pl.sorceries_cast_this_turn > 0)),
            R::ControlledByActivePlayer => match target {
                Target::Permanent(cid) => {
                    self.bf_hint_or_find(*cid, hint).is_some_and(|c| c.controller == self.active_player_idx)
                }
                Target::Player(p) => *p == self.active_player_idx,
            },
            R::ControlledByOpponent => match target {
                Target::Permanent(cid) => match self.bf_hint_or_find(*cid, hint) {
                    Some(c) => !self.same_team(c.controller, controller),
                    None => self
                        .stack_spell_caster(*cid)
                        .is_some_and(|ctrl| !self.same_team(ctrl, controller)),
                },
                Target::Player(p) => !self.same_team(*p, controller),
            },
            R::ControlledByTriggerPlayer => {
                let Some(who) = self.trigger_event_player_scratch else { return false };
                match target {
                    Target::Permanent(cid) => {
                        let ctrl = match self.bf_hint_or_find(*cid, hint) {
                            Some(c) => Some(c.controller),
                            None => self
                                .stack_spell_caster(*cid)
                                // CR 108.4 — a card in a hidden/terminal zone
                                // has no controller; its owner stands in
                                // (Wrexial's "target instant or sorcery card
                                // in that player's graveyard").
                                .or_else(|| self.find_card_anywhere(*cid).map(|c| c.owner)),
                        };
                        ctrl == Some(who)
                    }
                    Target::Player(p) => *p == who,
                }
            }
            R::OwnedByDefendingPlayer => {
                // The source may be the attacker itself or an Aura/Equipment
                // riding one, so fall through to the attachment host.
                let attacker = source.filter(|id| self.attack_for(*id).is_some()).or_else(|| {
                    source
                        .and_then(|id| self.battlefield_find(id))
                        .and_then(|c| c.attached_to)
                        .filter(|id| self.attack_for(*id).is_some())
                });
                let Some(defender) =
                    attacker.and_then(|id| self.attack_for(id)).and_then(|a| self.defender_for(a.target))
                else {
                    return false;
                };
                match target {
                    Target::Permanent(cid) => {
                        self.find_card_anywhere(*cid).is_some_and(|c| c.owner == defender)
                    }
                    Target::Player(p) => *p == defender,
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
                        self.bf_hint_or_find(*cid, hint).map(|c| c.controller).unwrap_or(controller)
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
                        self.bf_hint_or_find(*cid, hint).map(|c| c.controller).unwrap_or(controller)
                    }
                    Target::Player(p) => *p,
                };
                self.players[owner].cards_drawn_this_turn >= *n
            }
            R::ControllerSacrificedArtifactThisTurn => {
                let owner = match target {
                    Target::Permanent(cid) => {
                        self.bf_hint_or_find(*cid, hint).map(|c| c.controller).unwrap_or(controller)
                    }
                    Target::Player(p) => *p,
                };
                self.players[owner].artifacts_sacrificed_this_turn > 0
            }
            R::ControllersTurn => {
                let owner = match target {
                    Target::Permanent(cid) => {
                        self.bf_hint_or_find(*cid, hint).map(|c| c.controller).unwrap_or(controller)
                    }
                    Target::Player(p) => *p,
                };
                self.active_player_idx == owner
            }
            R::ControllerCorrupted => {
                let owner = match target {
                    Target::Permanent(cid) => {
                        self.bf_hint_or_find(*cid, hint).map(|c| c.controller).unwrap_or(controller)
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
                // spell. Every leg is lazy, the stack included: the walk stops
                // at the battlefield on almost every call, and the stack scan
                // used to run before it on all of them.
                let bf_card = self.bf_hint_or_find(*cid, hint);
                // The seven other places a card can be are one branch away,
                // not eight `Option::or_else` links: this build has no LTO, so
                // each link is an out-of-line call paid on the path where
                // `bf_card` has already answered.
                let card = match bf_card {
                    Some(c) => Some(c),
                    None => self.requirement_card_off_battlefield(*cid),
                };
                let Some(card) = card else { return false; };
                // Layer-4-aware card types for battlefield permanents
                // (CR 613.2): an artifact-ized creature (Phyrexian
                // Scriptures I), an animated land, or a devotion-gated god
                // must filter by its *computed* types, not the printed ones.
                // Off-battlefield cards keep the printed definition.
                // Taken lazily: most arms below never look at the layer view,
                // and this one is 40 % of the program's `computed_permanent`
                // calls when it is taken eagerly.
                type Computed = Option<std::sync::Arc<crate::game::layers::ComputedPermanent>>;
                // `computed()` is `None` exactly when the card isn't a live
                // battlefield permanent — `computed_permanent` returns `None`
                // off the battlefield and takes the printed view mid-gather —
                // so the arms that only need to know *that* ask this instead
                // and never force the cell.
                let computed_absent = || bf_card.is_none() || self.layer_reads_are_printed();
                // **No `OnceCell`, and none of the three this block used to
                // build.** One call evaluates one `req`, the arms below are
                // exclusive, and a composite requirement recurses into a fresh
                // frame — so `computed()`, the card-type gate and the shallow
                // creature-type read each run **at most once** per invocation
                // and a cell is pure construction. That is the fifty-fifth
                // pass's refutation (a cell around a once-per-call gate is
                // +1.24 M on `fixed`), which this block recorded for one of
                // the three and then kept for the other two.
                let computed = || -> Computed {
                    if self.layer_reads_are_printed() {
                        // Mid-recompute: printed types. A fast path, not
                        // the guard — `computed_permanent` enforces the
                        // reentrancy rule for every caller and answers the
                        // same printed view a layer pass slower.
                        //
                        // CR 613.8: the gather's condition-gated tail installs
                        // the effects built so far, and while it does this is
                        // *not* a mid-recompute read — a layer-7 condition
                        // asking about a layer-4 type change has to see it.
                        // `layer_reads_are_printed` is where the two
                        // conditions live.
                        None
                    } else {
                        bf_card.and_then(|_| self.computed_permanent(*cid))
                    }
                };
                // The card-type family is most of the requirement traffic
                // (`Creature` / `Land` / `Nonland` / `Noncreature` / `Artifact`)
                // and the gather behind `computed()` is ~2.7k Ir. A board with
                // no layer-4 card-type source in scope gives the same answer
                // from the printed line, so ask the presence gate — one
                // battlefield walk over printed shapes — before gathering.
                // `bestowed` (CR 702.103d) rewrites the type line without a
                // `Modification`, so it joins the gate off the card itself.
                let has_type = |t: crate::card::CardType| {
                    let gathered =
                        !computed_absent() && (card.bestowed || self.card_type_change_in_scope());
                    if !gathered {
                        return card.definition.card_types.contains(&t);
                    }
                    match computed() {
                        Some(cp) => cp.card_types().contains(&t),
                        None => card.definition.card_types.contains(&t),
                    }
                };
                // CR 613.2 layer-4 — a creature that gained a type from a
                // continuous effect (Jenova's Mutant grant) matches by its
                // *computed* subtypes on the battlefield; off-battlefield cards
                // (incl. die-snapshots, whose grants were stamped in at death)
                // fall back to the definition.
                // Mid-gather the full computed view is off-limits, but stored
                // layer-4 type changes are already in `continuous_effects` —
                // read them shallowly so a "as long as it's a Wall" gate sees a
                // retyped permanent (CR 613.8; Mistform Wall).
                // Presence-gated like `has_type`, and this arm is where the
                // traffic was: at the fifty-fifth pass's base the subtype
                // arms forced 413,844 of a cube run's 680,960
                // `computed_permanent` calls against the card-type gate's
                // 13,052 walks. `AddCreatureType` / `SetCreatureTypes` are the
                // only two modifications that write `subtypes.creature_types`,
                // and `shallow_creature_types` reads those same two off the
                // stored set, so with neither in scope all three paths give
                // the printed line.
                //
                // The gate here has never had a cell, for the reason the
                // block above now applies to all three.
                let has_ctype = |ct: &crate::card::CreatureType| {
                    if !self.creature_type_change_in_scope() {
                        return card.definition.subtypes.creature_types.contains(ct);
                    }
                    match computed() {
                        Some(cp) => cp.subtypes().creature_types.contains(ct),
                        None => match self.shallow_creature_types(*cid) {
                            Some(types) => types.contains(ct),
                            None => card.definition.subtypes.creature_types.contains(ct),
                        },
                    }
                };
                // CR 613.2 layer-4 — subtypes/supertypes a permanent gained (or
                // lost) from a continuous effect (Vraska's Treasure, Song of the
                // Dryads' Forest, Sugar Coat's Food, the Ring-bearer's Legendary)
                // read from the *computed* type line on the battlefield.
                let has_atype = |a: &crate::card::ArtifactSubtype| match computed() {
                    Some(cp) => cp.subtypes().artifact_subtypes.contains(a),
                    None => card.definition.subtypes.artifact_subtypes.contains(a),
                };
                // Same gate, same argument: `AddLandType` / `SetLandTypes` /
                // `ReplaceBasicLandType` are the only three modifications that
                // write `subtypes.land_types`.
                let has_ltype = |lt: &crate::card::LandType| {
                    if !self.land_type_change_in_scope() {
                        return card.definition.subtypes.land_types.contains(lt);
                    }
                    match computed() {
                        Some(cp) => cp.subtypes().land_types.contains(lt),
                        None => card.definition.subtypes.land_types.contains(lt),
                    }
                };
                let has_stype = |st: &Supertype| match computed() {
                    Some(cp) => cp.supertypes().contains(st),
                    None => card.definition.supertypes.contains(st),
                };
                use crate::card::CardType as CT;
                match req {
                    // CR 604.3 — Grist is a creature everywhere but the
                    // battlefield.
                    R::Creature => {
                        has_type(CT::Creature)
                            || (card.definition.creature_off_battlefield && computed_absent())
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
                    // Sentinel — "blocking or blocked by this creature",
                    // read off `block_map` in both directions.
                    R::BlockingOrBlockedBySource => source.is_some_and(|s| {
                        self.block_map.get(&card.id).is_some_and(|atk| atk.contains(&s))
                            || self.block_map.get(&s).is_some_and(|atk| atk.contains(&card.id))
                    }),
                    R::BlockedBySourceThisTurn => source
                        .and_then(|s| self.battlefield_find(s))
                        .is_some_and(|src| src.blocked_attackers_this_turn.contains(&card.id)),
                    // The game-level pair log, not the candidate's own field:
                    // the body that asks is an end-of-combat one and
                    // `resolve_combat` has already dropped `block_map`.
                    R::BlockedSourceThisTurn => source.is_some_and(|s| {
                        self.blocks_declared_this_turn.contains(&(card.id, s))
                    }),
                    // Brine Hag fires from the graveyard, so the source's own
                    // damage log comes off its leaves-battlefield LKI.
                    R::DealtDamageToSourceThisTurn => source
                        .and_then(|s| {
                            self.battlefield_find(s)
                                .or_else(|| self.leaves_bf_lki.get(&s))
                                .or_else(|| self.died_card_snapshots.get(&s))
                        })
                        .is_some_and(|src| src.damaged_by_this_turn.contains(&card.id)),
                    // CR 105.2/202.2 — color is the union of the mana cost's
                    // colors and the color indicator (tokens, DFC backs), and
                    // empty under Devoid. `printed_colors` folds all three in.
                    R::HasColor(c) => card.definition.printed_colors().contains(c),
                    R::HasKeyword(kw) => card.has_keyword(kw),
                    R::HasToxic => card.has_toxic(),
                    R::HasModular => card.has_modular(),
                    R::HasMutate => card.definition.mutate.is_some(),
                    R::HasMorphAbility => card.definition.keywords.iter().any(|k| matches!(
                        k,
                        crate::card::Keyword::Morph(_)
                            | crate::card::Keyword::MorphCost(_)
                            | crate::card::Keyword::Megamorph(_)
                            | crate::card::Keyword::Disguise(_)
                    )),
                    R::HasNoAbilities => card.definition.has_no_abilities(),
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
                    R::HasFlashback => card.definition.keywords.iter().any(|k| {
                        matches!(
                            k,
                            crate::card::Keyword::Flashback(_)
                                | crate::card::Keyword::FlashbackTap { .. }
                        )
                    }),
                    R::SharesCardTypeWithExiledBySource => self
                        .shares_card_type_with_exiled_by(source, &card.definition),
                    // CR 613 — the P/T thresholds read the *computed* view, so
                    // an anthem or a shrink (The Hippodrome's -5/-0) counts.
                    R::PowerAtMost(n) => {
                        card.definition.is_creature() && self.effective_power(card) <= *n
                    }
                    R::PowerAtMostSourcePower => {
                        card.definition.is_creature()
                            && source
                                .and_then(|s| self.battlefield_find(s))
                                .is_some_and(|src| card.power() <= src.power())
                    }
                    R::ToughnessAtMost(n) => {
                        card.definition.is_creature() && self.effective_toughness(card) <= *n
                    }
                    R::PowerAtLeast(n) => {
                        card.definition.is_creature() && self.effective_power(card) >= *n
                    }
                    R::ToughnessAtLeast(n) => {
                        card.definition.is_creature() && self.effective_toughness(card) >= *n
                    }
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
                    R::WithCounterAtLeast(k, n) => card.counter_count(*k) >= *n,
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
                    R::ControllerControlsLandType(lt) => {
                        self.seat_controls_land_type(card.controller, *lt)
                    }
                    R::HasArtifactSubtype(a) => has_atype(a),
                    R::HasEnchantmentSubtype(e) => card.definition.subtypes.enchantment_subtypes.contains(e),
                    R::HasPlaneswalkerType(pw) => card.definition.subtypes.planeswalker_subtypes.contains(pw),
                    R::IsToken => card.is_token,
                    R::NotToken => !card.is_token,

                    R::Warped => card.warped,
                    R::HasPhyrexianManaInCost => card.definition.cost.has_phyrexian(),
                    // Mitotic Manipulation — "if it has the same name as a permanent".
                    R::SameNameAsAPermanent => {
                        self.battlefield.iter().any(|p| p.definition.name == card.definition.name)
                    }
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
                    R::WasBlockedThisTurn => {
                        self.blocks_declared_this_turn.iter().any(|(_, a)| *a == card.id)
                    }
                    R::FaceDown => card.face_down,
                    R::PutIntoGraveyardThisTurn => {
                        self.players.iter().any(|p| p.graveyard_ids_this_turn.contains(cid))
                    }
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
                        // `values().any(n > 0)`, not `!is_empty()`: a stored
                        // zero is not a counter (CR 122.1). The accessors keep
                        // the invariant, this keeps the reader honest if a
                        // direct `entry()` site ever breaks it again.
                        card.counters.values().any(|&n| n > 0)
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
                    R::SpellTargetsMatching(inner) => self.stack.iter().any(|si| {
                        let StackItem::Spell { card: c, target, additional_targets, .. } = si else { return false };
                        c.id == card.id
                            && target.iter().chain(additional_targets.iter()).any(|t| {
                                self.evaluate_requirement_static(inner, t, controller, source)
                            })
                    }),
                    // Equinox — a stack spell whose effect destroys lands, and
                    // that either sweeps them or points at one of yours.
                    R::SpellWouldDestroyALandYouControl => self.stack.iter().any(|si| {
                        let StackItem::Spell { card: c, target, additional_targets, .. } = si
                        else {
                            return false;
                        };
                        if c.id != card.id {
                            return false;
                        }
                        let (targeted, mass) = effect_destroys_lands(&c.definition.effect);
                        if mass {
                            return self
                                .battlefield
                                .iter()
                                .any(|o| o.controller == controller && o.definition.is_land());
                        }
                        targeted
                            && target.iter().chain(additional_targets.iter()).any(|t| {
                                matches!(t, crate::game::types::Target::Permanent(pid)
                                    if self.battlefield.iter().any(|o| {
                                        o.id == *pid
                                            && o.controller == controller
                                            && o.definition.is_land()
                                    }))
                            })
                    }),
                    // Teferi's Response — a stack item (spell or ability) with
                    // a land the evaluating player controls among its targets.
                    R::TargetsALandYouControl => {
                        let hits_your_land = |t: &crate::game::types::Target| {
                            matches!(t, crate::game::types::Target::Permanent(pid)
                                if self.battlefield.iter().any(|o| {
                                    o.id == *pid
                                        && o.controller == controller
                                        && o.definition.is_land()
                                }))
                        };
                        self.stack.iter().any(|si| match si {
                            StackItem::Spell { card: c, target, additional_targets, .. } => {
                                c.id == card.id
                                    && target
                                        .iter()
                                        .chain(additional_targets.iter())
                                        .any(hits_your_land)
                            }
                            StackItem::Trigger { source: sid, target, .. } => {
                                *sid == card.id && target.iter().any(hits_your_land)
                            }
                        })
                    }
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
                    R::SaddledSourceThisTurn => source.is_some_and(|s| {
                        self.battlefield_find(s).is_some_and(|m| m.saddled_by.contains(cid))
                    }),
                    R::SpellNotCastFromHand => self.stack.iter().any(|si| matches!(
                        si,
                        StackItem::Spell { card: c, .. } if c.id == card.id && !c.cast_from_hand
                    )),
                    R::HasAbilityOnStack => self.stack.iter().any(|si| matches!(
                        si,
                        StackItem::Trigger { source, .. } if *source == card.id
                    )),
                    R::ManaValueAtMost(n) => card.definition.cost.cmc() <= *n,
                    R::ManaValueAtMostOpponentsAttackedThisCombat => {
                        card.definition.cost.cmc() <= self.opponents_attacked_this_combat()
                    }
                    R::ManaValueAtMostDevotion(color) => {
                        card.definition.cost.cmc() <= self.devotion_to(controller, &[*color]).max(0) as u32
                    }
                    // Battlefield permanents, so the *battlefield-aware*
                    // walker: `evaluate_requirement_on_card` answers `false`
                    // for every battlefield-state predicate (`Tapped`, the
                    // "greatest among" superlatives) by design — it is the
                    // library/hand-search path — so counting through it made
                    // a `Tapped` inner filter count zero. `_static_on` takes
                    // the instance, so this costs no lookup.
                    R::ManaValueAtMostYourCount(inner) => {
                        let n = self
                            .battlefield
                            .iter()
                            .filter(|c| self.evaluate_requirement_static_on(inner, c, controller, None))
                            .count() as u32;
                        card.definition.cost.cmc() <= n
                    }
                    R::ToughnessAtMostYourCount(inner) => {
                        let n = self
                            .battlefield
                            .iter()
                            .filter(|c| self.evaluate_requirement_static_on(inner, c, controller, None))
                            .count() as i32;
                        card.definition.is_creature()
                            && self
                                .computed_permanent(card.id)
                                .map(|cp| cp.toughness)
                                .unwrap_or_else(|| card.toughness())
                                <= n
                    }
                    // Ghastly Demise — "toughness ≤ cards in your graveyard".
                    R::ToughnessAtMostGraveyardCount => {
                        let n = self.players[controller].graveyard.len() as i32;
                        card.definition.is_creature()
                            && self
                                .computed_permanent(card.id)
                                .map(|cp| cp.toughness)
                                .unwrap_or_else(|| card.toughness())
                                <= n
                    }
                    // Temporary Insanity — "power < cards in your graveyard".
                    R::PowerLessThanYourGraveyardCount => {
                        let n = self.players[controller].graveyard.len() as i32;
                        card.definition.is_creature()
                            && self
                                .computed_permanent(card.id)
                                .map(|cp| cp.power)
                                .unwrap_or_else(|| card.power())
                                < n
                    }
                    R::PowerAtMostSourceCounters(kind) => {
                        // CR 608.2b — the source may have paid itself as the
                        // ability's cost (Legacy's Allure sacrifices itself),
                        // so fall back to its last-known battlefield counters.
                        let n = source
                            .and_then(|sid| {
                                self.battlefield_find(sid)
                                    .or_else(|| self.died_card_snapshots.get(&sid))
                                    .or_else(|| self.leaves_bf_lki.get(&sid))
                            })
                            .map(|c| c.counter_count(*kind))
                            .unwrap_or(0) as i32;
                        card.definition.is_creature()
                            && self
                                .computed_permanent(card.id)
                                .map(|cp| cp.power)
                                .unwrap_or_else(|| card.power())
                                <= n
                    }
                    R::PowerAtMostYourCount(inner) => {
                        let n = self
                            .battlefield
                            .iter()
                            .filter(|c| self.evaluate_requirement_static_on(inner, c, controller, None))
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
                    R::ManaValueEqualsCountersOnSource(kind) => source.is_some_and(|s| {
                        self.find_card_anywhere(s)
                            .is_some_and(|src| card.definition.cost.cmc() == src.counter_count(*kind))
                    }),
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
                    R::SharesMostCommonColor => {
                        let top = self.most_common_permanent_colors();
                        card.definition.printed_colors().iter().any(|k| top.contains(k))
                    }
                    R::ManaValueEqualsChosenNumber => {
                        card.definition.cost.cmc() == self.chosen_number_this_resolution
                    }
                    R::HasNonManaActivatedAbility => card
                        .definition
                        .activated_abilities
                        .iter()
                        .any(|a| !crate::game::actions::is_mana_ability_public(&a.effect)),
                    R::SharesNameWithAnotherPermanent => self
                        .battlefield
                        .iter()
                        .any(|c| c.id != card.id && c.definition.name == card.definition.name),
                    R::NameNotSharedWithYourPermanents => !self.battlefield.iter().any(|c| {
                        c.controller == controller && c.definition.name == card.definition.name
                    }),
                    // CR 702.114 — Devoid CDA: colorless despite colored pips.
                    R::Colorless => card.definition.keywords.has_kw(&crate::card::Keyword::Devoid)
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
                    R::NotSourcesChosenPermanent => source
                        .and_then(|s| self.battlefield_find(s))
                        .and_then(|s| s.chosen_permanent)
                        .is_none_or(|chosen| *cid != chosen),
                    R::IsSource => source == Some(*cid),
                    R::NotSacrificedThisResolution => {
                        !self.scratch.cards_sacrificed_this_resolution.contains(cid)
                    }
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
                        let Some(cand) = self.bf_hint_or_find(*cid, hint) else {
                            return false;
                        };
                        if !self.evaluate_requirement_static_hinted(inner, target, controller, source, hint) {
                            return false;
                        }
                        let cand_mv = cand.definition.cost.cmc();
                        let cand_ctrl = cand.controller;
                        // Walk the same controller's permanents matching
                        // inner; reject if any has a strictly greater MV.
                        !self.battlefield.iter().any(|other| {
                            other.controller == cand_ctrl
                                && other.id != *cid
                                && self.evaluate_requirement_static_on(
                                    inner,
                                    other,
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
                        let Some(cand) = self.bf_hint_or_find(*cid, hint) else {
                            return false;
                        };
                        if !self.evaluate_requirement_static_hinted(inner, target, controller, source, hint) {
                            return false;
                        }
                        let cand_pow = cand.power();
                        let cand_ctrl = cand.controller;
                        !self.battlefield.iter().any(|other| {
                            other.controller == cand_ctrl
                                && other.id != *cid
                                && self.evaluate_requirement_static_on(
                                    inner,
                                    other,
                                    controller,
                                    source,
                                )
                                && other.power() > cand_pow
                        })
                    }
                    R::HasGreatestPowerAmongAllCreatures => {
                        let Some(cand) = self.bf_hint_or_find(*cid, hint) else { return false };
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
                    R::HasName(name) => {
                        card.definition.name == name.as_str()
                            || self.has_all_creature_names(card.id, name)
                    }
                    R::OriginallyPrintedIn(set) => set.contains(card.definition.name),
                    R::ManaValueAtMostControllerHand => {
                        card.definition.cost.cmc() as usize <= self.players[controller].hand.len()
                    }
                    R::ManaValueAtMostControlledCount(inner) => {
                        let count = self
                            .battlefield
                            .iter()
                            .filter(|c| {
                                self.evaluate_requirement_static_on(
                                    inner,
                                    c,
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
                    R::NameDiffersFromLastMoved => !self.scratch.last_moved_cards.iter().any(|id| {
                        self.find_card_anywhere(*id)
                            .is_some_and(|c| c.definition.name == card.definition.name)
                    }),
                    R::NamedBySource => source
                        .and_then(|sid| self.static_source(sid))
                        .and_then(|s| s.named_card.as_deref())
                        // A resolving spell that named a card is off-zone, so
                        // fall back to the per-resolution scratchpad (Predict).
                        .or(self.scratch.named_card_this_resolution.as_deref())
                        .is_some_and(|n| n == card.definition.name),
                    R::NamedByEitherAgendaOfSource => source
                        .and_then(|sid| self.static_source(sid))
                        .is_some_and(|s| {
                            [s.named_card.as_deref(), s.named_card_2.as_deref()]
                                .iter()
                                .flatten()
                                .any(|n| *n == card.definition.name)
                        }),
                    R::NameNotedForSource => source
                        .and_then(|sid| self.find_card_anywhere(sid))
                        .is_some_and(|s| {
                            self.players[card.controller]
                                .draft_notes
                                .has_name(s.definition.name, card.definition.name)
                        }),
                    R::PowerAtMostDraftNoteMax => source
                        .and_then(|sid| self.find_card_anywhere(sid))
                        .is_some_and(|s| {
                            let cap = self.players[controller]
                                .draft_notes
                                .max_number(s.definition.name);
                            self.computed_permanent(card.id)
                                .map(|c| c.power)
                                .unwrap_or(card.definition.power)
                                <= cap as i32
                        }),
                    // The source is looked up by name, so a resolving spell
                    // (already off the stack) falls back to its stamped name.
                    R::HasDraftNotedColorOfSource => {
                        let name = source
                            .and_then(|sid| self.find_card_anywhere(sid))
                            .map(|s| s.definition.name)
                            .or(self.source_name_scratch);
                        name.is_some_and(|name| {
                            let noted = self
                                .players[controller]
                                .draft_notes
                                .colors
                                .get(name)
                                .cloned()
                                .unwrap_or_default();
                            card.definition.printed_colors().iter().any(|c| noted.contains(c))
                        })
                    }
                    R::HasDraftNotedCreatureTypeOfSource => {
                        let name = source
                            .and_then(|sid| self.find_card_anywhere(sid))
                            .map(|s| s.definition.name)
                            .or(self.source_name_scratch);
                        name.is_some_and(|name| {
                            let noted =
                                self.players[controller].draft_notes.noted_creature_types(name);
                            // Printed types — the anthem this gates is gathered
                            // inside the layer walk, so a computed lookup here
                            // would recurse.
                            card.definition
                                .subtypes
                                .creature_types
                                .iter()
                                .any(|t| noted.contains(t))
                        })
                    }
                    R::IsSourceChosenCardType => source
                        .and_then(|sid| self.battlefield_find(sid))
                        .and_then(|s| s.chosen_card_type.clone())
                        .is_some_and(|t| card.definition.card_types.contains(&t)),
                    R::SameNameAsTarget => false,
                    R::IsSourceChosenCreatureType => source
                        .and_then(|sid| self.find_card_anywhere(sid))
                        .and_then(|s| s.chosen_creature_type)
                        .or(self.chosen_creature_type_scratch)
                        .is_some_and(|ct| {
                            card.definition.subtypes.creature_types.contains(&ct)
                                || card.has_keyword(&crate::card::Keyword::Changeling)
                        }),
                    // Extraplanar Lens — "a land with the same name as the
                    // exiled card".
                    // Either link is "exiled with": `exiled_with` (imprint,
                    // `ExileWithSource`) or the until-leaves `exiled_by`
                    // (Circle of Confinement's Vampire name check).
                    R::SameNameAsExiledWithSource => source.is_some_and(|sid| {
                        self.exile
                            .iter()
                            .any(|c| {
                                (c.exiled_with == Some(sid)
                                    || c.exiled_by.as_ref().is_some_and(|l| l.source == sid))
                                    && c.definition.name == card.definition.name
                            })
                    }),
                    // Story Circle — "a source of the chosen colour".
                    R::HasChosenColorOfSource => source
                        .and_then(|sid| self.find_card_anywhere(sid))
                        .and_then(|src| src.chosen_color)
                        .is_some_and(|color| card.definition.printed_colors().contains(&color)),
                    // Roots of Life — "a land of the chosen type".
                    R::HasChosenLandTypeOfSource => source
                        .and_then(|sid| self.find_card_anywhere(sid))
                        .and_then(|src| src.chosen_land_type)
                        .is_some_and(|lt| card.definition.subtypes.land_types.contains(&lt)),
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
                            // CR 603.10 — an Aura sacrificed as its own
                            // ability's cost is already gone when the body
                            // resolves, so fall back to its leave snapshot
                            // for the host link (the ONS Crown cycle).
                            .and_then(|sid| {
                                self.battlefield_find(sid)
                                    .or_else(|| self.died_card_snapshots.get(&sid))
                                    .or_else(|| self.leaves_bf_lki.get(&sid))
                            })
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
    /// Takes `impl Into<Arc<_>>` so a caller holding a `CardInstance`'s
    /// definition hands over a refcount. It used to take `&CardDefinition` and
    /// `clone()` it into the scratch instance — a **deep** copy of an
    /// 8,232-byte struct and every `Vec` in it, to answer a predicate.
    pub fn definition_matches_requirement(
        &self,
        def: impl Into<std::sync::Arc<crate::card::CardDefinition>>,
        req: &SelectionRequirement,
        controller: usize,
    ) -> bool {
        let scratch = CardInstance::new(CardId(u32::MAX), def, controller);
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
            // Source-less path: the counter count needs the ability's source,
            // which only `evaluate_requirement_static` carries.
            R::ManaValueEqualsCountersOnSource(_) | R::PowerAtMostSourceCounters(_) => false,
            R::Player
            | R::OpponentPlayer
            | R::YouPlayer
            | R::OpponentTallyDiffers { .. }
            | R::PlayerAttackedThisTurn => false,
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
            R::ControlledByActivePlayer => card.controller == self.active_player_idx,
            R::HasAwaken => card.definition.alternative_cost.as_ref().is_some_and(|a| a.awaken),
            R::PutIntoGraveyardFromBattlefieldThisTurn => {
                self.graveyard_from_battlefield_this_turn.contains(&card.id)
            }
            R::PutIntoGraveyardThisTurn => {
                self.players.iter().any(|p| p.graveyard_ids_this_turn.contains(&card.id))
            }
            R::ControlledByTriggerPlayer => {
                self.trigger_event_player_scratch == Some(card.controller)
            }
            R::OwnedByYou => card.owner == controller,
            // Source-less path: the defending player needs the ability's
            // source, which only `evaluate_requirement_static` carries.
            R::OwnedByDefendingPlayer => false,
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
            R::HasMorphAbility => card.definition.keywords.iter().any(|k| matches!(
                k,
                crate::card::Keyword::Morph(_)
                    | crate::card::Keyword::MorphCost(_)
                    | crate::card::Keyword::Megamorph(_)
                    | crate::card::Keyword::Disguise(_)
            )),
            R::HasNoAbilities => card.definition.has_no_abilities(),
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
            R::HasFlashback => card.definition.keywords.iter().any(|k| {
                matches!(
                    k,
                    crate::card::Keyword::Flashback(_)
                        | crate::card::Keyword::FlashbackTap { .. }
                )
            }),
            // No source context in the on-card evaluator; the exile-linked
            // share check only resolves through `evaluate_requirement_static`.
            R::SharesCardTypeWithExiledBySource => false,
            R::PowerAtMost(n) => card.definition.is_creature() && card.power() <= *n,
            R::PowerAtMostSourcePower => false,
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
            R::ControllerControlsLandType(lt) => {
                self.seat_controls_land_type(card.controller, *lt)
            }
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
            R::SameNameAsAPermanent => {
                self.battlefield.iter().any(|p| p.definition.name == card.definition.name)
            }
            R::HasPhyrexianManaInCost => card.definition.cost.has_phyrexian(),
            R::IsBasicLand => card.definition.is_land() && card.definition.supertypes.contains(&Supertype::Basic),
            R::IsNonbasicLand => card.definition.is_land() && !card.definition.supertypes.contains(&Supertype::Basic),
            R::ProducesColorless => card.definition.produces_colorless(),
            R::IsSnow => card.definition.is_snow(),
            R::ManaValueAtMost(n) => card.definition.cost.cmc() <= *n,
            R::ManaValueAtMostOpponentsAttackedThisCombat => {
                card.definition.cost.cmc() <= self.opponents_attacked_this_combat()
            }
            // Need the ability's source or the live trigger context, which
            // only the static walker carries.
            R::HasChosenColorOfSource
            | R::HasChosenLandTypeOfSource
            | R::SharesColorWithExiledBySource
            | R::SameNameAsExiledWithSource
            | R::SharesColorWithAttachedHost
            | R::SharesCreatureTypeWithAttachedHost => false,
            // Empty-Shrine Kannushi — printed colours on both sides, since
            // this is consulted from inside the layer gather.
            R::SharesColorWithSacrificed => {
                let colors = card.definition.printed_colors();
                self.sacrificed_colors
                    .as_ref()
                    .is_some_and(|cs| cs.iter().any(|c| colors.contains(c)))
            }
            // Endemic Plague — "share a creature type with the sacrificed
            // creature". Changeling matches everything (CR 702.73a).
            R::SharesCreatureTypeWithSacrificed => {
                let mine = &card.definition.subtypes.creature_types;
                let wild = card.definition.keywords.has_kw(&crate::card::Keyword::Changeling);
                self.sacrificed_card
                    .and_then(|id| {
                        self.died_card_snapshots.get(&id).or_else(|| self.find_card_anywhere(id))
                    })
                    .is_some_and(|s| {
                        wild || s.definition.keywords.has_kw(&crate::card::Keyword::Changeling)
                            || s.definition
                                .subtypes
                                .creature_types
                                .iter()
                                .any(|t| mine.contains(t))
                    })
            }
            // Harsh Mercy — "of a type chosen this way".
            R::IsTypeChosenThisWay => {
                card.definition.keywords.has_kw(&crate::card::Keyword::Changeling)
                    || self
                        .scratch.chosen_creature_types_scratch
                        .iter()
                        .any(|t| card.definition.subtypes.creature_types.contains(t))
            }
            // Cryptic Gateway — "shares a creature type with EACH creature
            // tapped this way". Changeling matches everything (CR 702.73a).
            R::SharesCreatureTypeWithTapped => {
                let mine = &card.definition.subtypes.creature_types;
                let wild = card.definition.keywords.has_kw(&crate::card::Keyword::Changeling);
                !self.tapped_for_cost.is_empty()
                    && self.tapped_for_cost.iter().all(|id| {
                        self.battlefield_find(*id).is_some_and(|t| {
                            wild || t.definition.keywords.has_kw(&crate::card::Keyword::Changeling)
                                || t.definition
                                    .subtypes
                                    .creature_types
                                    .iter()
                                    .any(|ct| mine.contains(ct))
                        })
                    })
            }
            // Concretized before the walk (`choose_damage_prevention_source`)
            // and slot-aware (`cross_slot_targets_ok`) respectively.
            R::SharesColorWithManaSpent | R::SameControllerAsTargetSlot(_) => true,
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
            // Same battlefield-aware walk as the static side — see the
            // note on its `ManaValueAtMostYourCount` arm.
            R::ManaValueAtMostYourCount(inner) => {
                let n = self
                    .battlefield
                    .iter()
                    .filter(|c| self.evaluate_requirement_static_on(inner, c, controller, None))
                    .count() as u32;
                card.definition.cost.cmc() <= n
            }
            R::ToughnessAtMostGraveyardCount => {
                let n = self.players[controller].graveyard.len() as i32;
                card.definition.is_creature()
                    && self
                        .computed_permanent(card.id)
                        .map(|cp| cp.toughness)
                        .unwrap_or_else(|| card.toughness())
                        <= n
            }
            R::PowerLessThanYourGraveyardCount => {
                let n = self.players[controller].graveyard.len() as i32;
                card.definition.is_creature()
                    && self
                        .computed_permanent(card.id)
                        .map(|cp| cp.power)
                        .unwrap_or_else(|| card.power())
                        < n
            }
            R::ToughnessAtMostYourCount(inner) => {
                let n = self
                    .battlefield
                    .iter()
                    .filter(|c| self.evaluate_requirement_static_on(inner, c, controller, None))
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
                    .filter(|c| self.evaluate_requirement_static_on(inner, c, controller, None))
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
            // Corrosion — the candidate's own counters, so it reads off a
            // `CardInstance` directly.
            R::ManaValueAtMostOwnCounters(kind) => {
                card.definition.cost.cmc() <= card.counter_count(*kind)
            }
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
            R::SharesMostCommonColor => {
                let top = self.most_common_permanent_colors();
                card.definition.printed_colors().iter().any(|k| top.contains(k))
            }
            R::ManaValueEqualsChosenNumber => {
                card.definition.cost.cmc() == self.chosen_number_this_resolution
            }
            R::HasNonManaActivatedAbility => card
                .definition
                .activated_abilities
                .iter()
                .any(|a| !crate::game::actions::is_mana_ability_public(&a.effect)),
            R::SharesNameWithAnotherPermanent => self
                .battlefield
                .iter()
                .any(|c| c.id != card.id && c.definition.name == card.definition.name),
            R::NameNotSharedWithYourPermanents => !self.battlefield.iter().any(|c| {
                c.controller == controller && c.definition.name == card.definition.name
            }),
            // CR 702.114 — Devoid CDA: colorless despite colored pips.
            R::Colorless => card.definition.keywords.has_kw(&crate::card::Keyword::Devoid)
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
            // Source-less path: no `chosen_permanent` slot to consult.
            R::NotSourcesChosenPermanent => true,
            // A card in another zone is never the battlefield source.
            R::IsSource => false,
            R::NotSacrificedThisResolution => {
                !self.scratch.cards_sacrificed_this_resolution.contains(&card.id)
            }
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
            R::OriginallyPrintedIn(set) => set.contains(card.definition.name),
            // Resolved to a concrete `HasName` by callers that have the
            // source in hand (RevealUntilFind); vacuously false otherwise.
            // No source context here; the per-resolution scratchpad is the
            // only readable stamp (Predict's milled-card check).
            R::NamedBySource => self
                .scratch.named_card_this_resolution
                .as_deref()
                .is_some_and(|n| n == card.definition.name),
            R::IsSourceChosenCardType => false,
            R::NameNotedForSource | R::NamedByEitherAgendaOfSource => false,
            R::PowerAtMostDraftNoteMax
            | R::HasDraftNotedColorOfSource
            | R::HasDraftNotedCreatureTypeOfSource => false,
            R::IsSourceChosenCreatureType => false,
            R::SameNameAsTarget | R::TargetsALandYouControl => false,
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
            R::NameDiffersFromLastMoved => !self.scratch.last_moved_cards.iter().any(|id| {
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
            R::WasBlockedThisTurn => {
                self.blocks_declared_this_turn.iter().any(|(_, a)| *a == card.id)
            }
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
            // Counters live on the instance, so they read in any zone (a
            // library/graveyard card simply has none) — Eluge's flood-counter
            // land scope resolves through this path.
            R::WithCounter(k) => card.counter_count(*k) > 0,
            R::WithCounterAtLeast(k, n) => card.counter_count(*k) >= *n,
            R::WithAnyCounter => card.counters.values().any(|&n| n > 0),
            // Battlefield-state predicates can't be evaluated for library cards.
            R::Tapped | R::Untapped
            | R::IsUnblocked | R::IsBlocked | R::IsBlocking | R::InCombatWithSource
            | R::IsAttackingAlone | R::IsBlockingAlone
            | R::FaceDown | R::HasAbilityOnStack
            | R::IsSpellOnStack | R::SpellNotCastFromHand
            | R::SpellTargetsControllerOrControlled
            | R::SpellTargetsCreature
            | R::SpellTargetsMatching(_)
            | R::SpellWouldDestroyALandYouControl
            | R::CastSorceryThisTurn
            | R::SpellTargetsOnlySource
            | R::SpellWithSingleTarget
            | R::DealtDamageToControllerThisTurn | R::IsBestowed
            | R::EquippedByAtLeast(_) | R::IsModified | R::DealtDamageThisTurn
            | R::DamagedBySourceThisTurn | R::DealtDamageToSourceThisTurn
            | R::BlockingOrBlockedBySource
            | R::BlockedBySourceThisTurn
            | R::BlockedSourceThisTurn
            | R::PlayerDamagedBySourceThisTurn
            | R::SaddledSourceThisTurn => false,
        }
    }
}
