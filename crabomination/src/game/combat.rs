use super::*;
use crate::card::Keyword;
use crate::effect::{Effect, EventKind, Selector, Value};
use crate::game::layers::ComputedPermanent;
use smallvec::SmallVec;

/// One combat/noncombat damage trigger gathered by `fire_combat_damage_
/// triggers`, before its intervening-'if' runs: `(source, effect,
/// controller, intervening-if, bind_dealer)`. The last flag binds the damage
/// dealer as the body's `TriggerSource` (the Phase-1.5 listener pattern —
/// Kaito's "return one of them to hand").
type DamageTrigger = (CardId, Effect, usize, Option<crate::card::Predicate>, bool);

/// Static prohibitions `declare_attackers_banded` checks against the whole
/// battlefield. Same device as [`crate::game::actions::cast_static`] and
/// `prevent_static`: one walk up front, a bit per family, and each gated
/// block still runs its own controller / filter / amount tests unchanged, so
/// a set bit costs a walk and a clear bit skips work that was a no-op.
///
/// **Sound by construction, and only for these four.** Every block gated here
/// tests `matches!` on one `StaticEffect` variant read straight off a
/// battlefield card's `static_abilities`, which is exactly what the scan
/// reads. The declaration's other two static walks — Magnetic Web's
/// `AttackTogether` and Arboria's
/// `PlayersCantBeAttackedUnlessTheyActedLastTurn` — go through
/// `active_static`, which *peels* `WhileYourTurn`-style wrappers, so a raw
/// variant scan would miss a wrapped one and skip work that was not a no-op.
/// **They are deliberately left ungated**; widening the scan means peeling
/// the same wrappers `active_static` does, and a second hand-written copy of
/// that list is the walker-drift bug class this repo keeps closing.
pub(crate) mod attack_static {
    /// `AttackerCapAgainstController` (Silent Arbiter-style per-defender cap).
    pub const ATTACKER_CAP: u32 = 1 << 0;
    /// `AttackPowerCapByControllerHand` (Ghostly Prison-adjacent power caps).
    pub const POWER_CAP: u32 = 1 << 1;
    /// `CreaturesCantAttackController` (Propaganda's prohibition half).
    pub const CANT_ATTACK_CONTROLLER: u32 = 1 << 2;
    /// `AttackTaxToController` (Propaganda, Elephant Grass, Norn's Annex).
    pub const ATTACK_TAX: u32 = 1 << 3;
}

/// Tag which of `declare_attackers_banded`'s rejections fired, under
/// `CRAB_SIM_REJECTS=names`.
///
/// The function gates an attacker on thirteen prohibitions and returns the
/// same three error kinds from twenty-eight places, so the census in PERF
/// (-55) could name the *card* and never the *rule*. `line!()` at the return
/// site is the cheapest unique tag there is and cannot drift from the code it
/// names. Off by default: one atomic load and a branch, and the value is
/// returned unchanged either way.
#[inline]
fn attack_reject(line: u32, e: GameError) -> GameError {
    if crate::game::reject_trace_level() >= 2 {
        eprintln!("attack_reject combat.rs:{line} {e:?}");
    }
    e
}

/// [`attack_reject`] for `declare_blockers`, which has forty-four rejection
/// returns and the same problem: `CannotBlock(id)` names the blocker and
/// never which of the ~twenty prohibitions barred it, and the batch-level
/// ones (`MustBeBlockedIfAble`, the block cost) name a card that is not the
/// one at fault at all.
#[inline]
fn block_reject(line: u32, e: GameError) -> GameError {
    if crate::game::reject_trace_level() >= 2 {
        eprintln!("block_reject combat.rs:{line} {e:?}");
    }
    e
}

/// See [`attack_static`]. One battlefield walk; `u32::MAX` is the ungated
/// reading every gated site `debug_assert!`s against.
pub(crate) fn attack_static_scan(state: &GameState) -> u32 {
    use crate::effect::StaticEffect as SE;
    let mut m = 0u32;
    for card in state.battlefield.iter() {
        for sa in &card.definition.static_abilities {
            m |= match sa.effect {
                SE::AttackerCapAgainstController { .. } => attack_static::ATTACKER_CAP,
                SE::AttackPowerCapByControllerHand => attack_static::POWER_CAP,
                SE::CreaturesCantAttackController { .. } => {
                    attack_static::CANT_ATTACK_CONTROLLER
                }
                SE::AttackTaxToController { .. } => attack_static::ATTACK_TAX,
                _ => 0,
            };
        }
    }
    m
}

/// The `AttackBlockCostTapAnother` filters carried by a computed keyword list
/// (Hollow Warrior) — one helper must be tapped per entry.
fn tap_another_filters(kws: &[Keyword]) -> Vec<crate::card::SelectionRequirement> {
    // Asked once per declared attacker and once per declared blocker, and
    // empty on every board that plays none of the handful of cards with the
    // keyword — but an empty `collect()` still calls `Vec::from_iter`. The
    // presence scan is over the same slice the `filter_map` would walk.
    if !kws.iter().any(|k| matches!(k, Keyword::AttackBlockCostTapAnother(_))) {
        return Vec::new();
    }
    kws.iter()
        .filter_map(|k| match k {
            Keyword::AttackBlockCostTapAnother(f) => Some((**f).clone()),
            _ => None,
        })
        .collect()
}

/// CR 509.1b — how many attackers this creature may block. One by default;
/// `CanBlockAdditional(n)` adds n (they stack), `CanBlockAnyNumber` lifts the
/// cap entirely. `SelfCanBlockAdditionalPerAttachedEquipment` (Kemba's Legion)
/// adds one more per attached Equipment — see `max_blocks_on`.
fn max_blocks_for(kws: &[Keyword]) -> usize {
    if kws.has_kw(&Keyword::CanBlockAnyNumber) {
        return usize::MAX;
    }
    1 + kws
        .iter()
        .filter_map(|k| match k {
            Keyword::CanBlockAdditional(n) => Some(*n as usize),
            _ => None,
        })
        .sum::<usize>()
}

impl GameState {
    /// CR 702.22d/j — the qualities named by "bands with other [quality]" on
    /// any of `ids`, read from the computed keyword set so granted instances
    /// (Adventurers' Guildhouse) count.
    pub(crate) fn bands_with_other_qualities(
        &self,
        ids: &[CardId],
    ) -> Vec<crate::card::SelectionRequirement> {
        // The band is 2-3 cards; computing all ~20 permanents to read their
        // keywords was the whole cost. Walk the battlefield for order, but
        // compute only the members — under one freeze so they share a gather.
        self.with_frozen_layers(|g| {
            g.battlefield
                .iter()
                .filter(|c| ids.contains(&c.id))
                .filter_map(|c| g.computed_permanent(c.id))
                .flat_map(|c| {
                    c.keywords()
                        .iter()
                        .filter_map(|k| match k {
                            Keyword::BandsWithOther(q) => Some((**q).clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        })
    }

    /// [`bands_with_other_qualities`](Self::bands_with_other_qualities) read
    /// off a computed set the caller already holds. Every combat-damage caller
    /// has one — the resolver's `combat_damage_computed` covers exactly the
    /// attackers and declared blockers — so the band question costs a
    /// battlefield walk instead of a gather.
    fn bands_with_other_qualities_of(
        &self,
        ids: &[CardId],
        computed: &[ComputedPermanent],
    ) -> Vec<crate::card::SelectionRequirement> {
        // Plain loops, same reason as `fire_combat_damage_to_player_triggers`.
        let mut out = Vec::new();
        for c in self.battlefield.iter() {
            if !ids.contains(&c.id) {
                continue;
            }
            let Some(cp) = computed.iter().find(|p| p.id == c.id) else { continue };
            for k in cp.keywords().iter() {
                if let Keyword::BandsWithOther(q) = k {
                    out.push((**q).clone());
                }
            }
        }
        out
    }

    /// CR 702.22j — is this set of blockers a "bands with other [quality]"
    /// band? True when at least two of them match a quality one of them bands
    /// with, which hands the damage division to the defending player.
    fn quality_band_assigner(&self, ids: &[CardId], computed: &[ComputedPermanent]) -> Option<usize> {
        let qualities = self.bands_with_other_qualities_of(ids, computed);
        qualities.iter().find(|q| {
            ids.iter()
                .filter(|id| {
                    self.evaluate_requirement_static(q, &Target::Permanent(**id), 0, None)
                })
                .count()
                >= 2
        })?;
        ids.iter().find_map(|id| self.battlefield_find(*id)).map(|c| c.controller)
    }

    /// `max_blocks_for` plus the board-dependent riders keyed on the blocker
    /// itself ("…for each Equipment attached to this creature").
    /// CR 506.2 / 509.1b — the tightest "no more than N creatures can
    /// attack/block each combat" cap in play (Silent Arbiter), or `None` when
    /// nothing caps participation.
    pub(crate) fn combat_participation_cap(&self, blocking: bool) -> Option<u32> {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match (&sa.effect, blocking) {
                (StaticEffect::MaxAttackersPerCombat(n), false) => Some(*n),
                (StaticEffect::MaxBlockersPerCombat(n), true) => Some(*n),
                _ => None,
            })
            .min()
    }

    pub(crate) fn max_blocks_on(&self, blocker: CardId, kws: &[Keyword]) -> usize {
        use crate::effect::StaticEffect;
        let base = max_blocks_for(kws);
        if base == usize::MAX {
            return base;
        }
        let per_equipment = self
            .battlefield_find(blocker)
            .filter(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        StaticEffect::SelfCanBlockAdditionalPerAttachedEquipment
                    )
                })
            })
            .map(|_| {
                self.battlefield
                    .iter()
                    .filter(|c| c.attached_to == Some(blocker) && c.definition.is_equipment())
                    .count()
            })
            .unwrap_or(0);
        base + per_equipment
    }

    /// True if `card` carries a `CanAttackIgnoringDefenderWhile` static whose
    /// condition currently holds — it may attack despite Defender
    /// (Drowsing Tyrannodon).
    pub(crate) fn ignores_defender_for_attack(&self, card: &CardInstance) -> bool {
        use crate::effect::StaticEffect;
        // CR 508.1a — a turn-scoped grant (Krotiq Nestguard's activated ability).
        if self.attack_despite_defender_this_turn.contains(&card.id) {
            return true;
        }
        // CR 508.1a — a team-wide static (High Alert, Assault Formation): any
        // permanent the attacker's controller has granting "creatures you
        // control can attack as though they didn't have defender".
        if self.battlefield.iter().any(|c| {
            c.controller == card.controller
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::YourCreaturesCanAttackAsThoughNoDefender)
                })
        }) {
            return true;
        }
        let mut ctx =
            crate::game::effects::EffectContext::for_ability(card.id, card.controller, None);
        // CR 702.32 — the gate may be "if this creature was kicked" (Prison
        // Barricade), which reads the permanent's own cast-time flag.
        ctx.kicked = card.kicked;
        card.definition.static_abilities.iter().any(|sa| {
            if let StaticEffect::CanAttackIgnoringDefenderWhile { condition } = &sa.effect {
                self.evaluate_predicate(condition, &ctx)
            } else {
                false
            }
        })
    }

    /// CR 509.1a — true if `controller` has a permanent granting "tapped
    /// creatures you control can block as though they were untapped" (Masako
    /// the Humorless).
    pub(crate) fn tapped_creatures_can_block(&self, controller: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == controller
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::TappedCreaturesCanBlock))
        })
    }

    /// The seat that declares attackers this turn — the active player unless a
    /// `combat_chooser` (Master Warcraft) is set.
    pub fn attack_declarer(&self) -> usize {
        self.combat_chooser.unwrap_or(self.active_player_idx)
    }

    /// The single seat that must submit the block declaration, if any:
    /// Master Warcraft's one-shot `combat_chooser`, else the active player
    /// while an `AttackingPlayerChoosesBlocks` static is out (Invasion Plans).
    pub fn block_chooser(&self) -> Option<usize> {
        if let Some(chooser) = self.combat_chooser {
            return Some(chooser);
        }
        self.battlefield
            .iter()
            .any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::AttackingPlayerChoosesBlocks
                    )
                })
            })
            .then_some(self.active_player_idx)
    }

    /// May `seat` submit the block declaration? Normally any non-active seat
    /// (a defending player declares its own blocks); with a `block_chooser`
    /// set, only that seat.
    pub fn may_declare_blocks(&self, seat: usize) -> bool {
        match self.block_chooser() {
            Some(chooser) => chooser == seat,
            None => seat != self.active_player_idx,
        }
    }

    // ── Declare attackers ─────────────────────────────────────────────────────

    /// CR 508.1a — the restrictions on one attacker that depend only on the
    /// creature and the board: not on which defender it is aimed at (see
    /// [`Self::attacker_target_block`]) and not on the rest of the batch
    /// (Okk's bigger partner, the participation cap).
    ///
    /// One walker, so [`Self::declare_attackers_banded`]'s declaration gate
    /// and its three CR 508.1d "attacks each combat if able" requirement
    /// loops cannot disagree about which creatures are *able*. Those loops
    /// used to spell out five of these families by hand, so a creature that
    /// had to attack and carried any of the other twenty was **required to
    /// attack and then rejected for attacking** — an unsatisfiable
    /// declaration that costs the seat its whole combat, with no legal move
    /// out of it.
    ///
    /// Returns the rejecting site's line alongside the error so the per-site
    /// census (`CRAB_SIM_REJECTS=names`) still names the rule, not the card.
    pub(crate) fn attacker_self_block(
        &self,
        p: usize,
        card: &crate::card::CardInstance,
        cp: Option<&ComputedPermanent>,
        power_caps: &[usize],
    ) -> Option<(u32, GameError)> {
        let id = card.id;
        let kws: &[Keyword] = cp.map(|c| c.keywords()).unwrap_or(&[]);
        // The instance reads first — they need no keyword walk at all, and the
        // cascade's order is the one `declare_attackers_banded` reported
        // before the two walkers were merged.
        if card.tapped {
            return Some((line!(), GameError::CardIsTapped(id)));
        }
        let is_creature_now = cp
            .map(|c| c.card_types().contains(&crate::card::CardType::Creature))
            .unwrap_or_else(|| card.definition.is_creature());
        // CR 701.35 — detain; CR 508.1a — Wall of Dust's one-turn ban. Both
        // report `CannotAttack`, as does a permanent that is not a creature
        // right now: a bestowed Aura (Kestia) or a de-animated Vehicle, which
        // used to fall through this cascade and come out labelled *summoning
        // sick* on a card whose `summoning_sick` is false.
        if !is_creature_now
            || card.detained_by.is_some()
            || card.attack_ban == crate::card::AttackBan::Active
        {
            return Some((line!(), GameError::CannotAttack(id)));
        }
        // Ensnaring Bridge cap (computed power, CR 613).
        if !power_caps.is_empty()
            && let power = cp.map(|c| c.power).unwrap_or(0)
            && power_caps.iter().any(|cap| power > *cap as i32)
        {
            return Some((line!(), GameError::CannotAttack(id)));
        }
        // **One walk over the computed keywords.** Twenty `has_kw` /
        // `iter().any()` scans of the same short slice is twenty times the
        // loop, and this runs once per attack candidate inside the bot's
        // search — asking the families one at a time read `fixed` +0.17 %.
        let mut hasted = false;
        for k in kws {
            match k {
                Keyword::Haste => hasted = true,
                Keyword::CantAttack => {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                Keyword::Defender if !self.ignores_defender_for_attack(card) => {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Goblin Cohort: unless you cast a creature spell
                // this turn.
                Keyword::CantAttackUnlessCastCreatureThisTurn
                    if self.players[p].creatures_cast_this_turn == 0 =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Hazoret-class: unless the hand is small.
                Keyword::CantAttackOrBlockUnlessHandSizeAtMost(n)
                    if self.players[p].hand.len() as u32 > *n =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Patchwork Beastie's delirium gate.
                Keyword::CantAttackOrBlockUnlessDelirium if !self.delirium_active(p) => {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Bontu the Glorified: a creature died this turn.
                Keyword::CantAttackOrBlockUnlessCreatureDiedThisTurn
                    if self.players[p].creatures_died_this_turn == 0 =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — The Ancient One's descend gate.
                Keyword::CantAttackOrBlockUnlessDescend(n)
                    if self.descend_count(p) < *n as usize =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Wayward Swordtooth's city's blessing.
                Keyword::CantAttackOrBlockUnlessCityBlessing
                    if !self.players[p].city_blessing =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Glacial Crasher: a land of the named type is on
                // the battlefield (anyone's).
                Keyword::CantAttackUnlessLandTypeOnBattlefield(lt)
                    if !self
                        .battlefield
                        .iter()
                        .any(|c| c.definition.subtypes.land_types.contains(lt)) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Harbor Serpent: five or more Islands.
                Keyword::CantAttackUnlessLandCount(lt, n)
                    if (self
                        .battlefield
                        .iter()
                        .filter(|c| c.definition.subtypes.land_types.contains(lt))
                        .count() as u32)
                        < *n =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Bloodcrazed Goblin: an opponent took damage.
                Keyword::CantAttackUnlessOpponentDamaged
                    if !self
                        .players
                        .iter()
                        .enumerate()
                        .any(|(i, pl)| !self.same_team(i, p) && pl.was_dealt_damage_this_turn) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Giant Turtle: not if it attacked last turn.
                Keyword::CantAttackIfAttackedLastTurn if card.attacked_last_turn => {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Ketramose: seven or more cards in exile.
                Keyword::CantAttackOrBlockUnlessCardsInExile(n)
                    if (self.exile.len() as u32) < *n =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // "Even number of counters" gate (Sab-Sunen). Zero is even.
                Keyword::CantAttackOrBlockUnlessEvenCounters
                    if card.counters.values().sum::<u32>() % 2 != 0 =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // "Can't attack unless you control a [filter]" (Lovestruck
                // Beast).
                Keyword::CanAttackOnlyIfYouControl(req)
                    if !self.battlefield.iter().any(|c| {
                        c.controller == p && self.evaluate_requirement_on_card(req, c, p)
                    }) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // "Can't attack unless you control N+ [filter]" (Topiary
                // Stomper — seven or more lands).
                Keyword::CantAttackOrBlockUnlessYouControlCount {
                    filter,
                    min,
                    block_only: false,
                    exclude_self,
                    ..
                } if (self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == p
                            && !(*exclude_self && c.id == id)
                            && self.evaluate_requirement_on_card(filter, c, p)
                    })
                    .count() as u32)
                    < *min =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                _ => {}
            }
        }
        if card.summoning_sick && !hasted {
            return Some((line!(), GameError::SummoningSickness(id)));
        }
        None
    }

    /// CR 508.1a — the restrictions on one attacker that depend on *which*
    /// defender it is aimed at. Split from [`Self::attacker_self_block`] so
    /// [`Self::attacker_is_able`] can ask whether **any** defender would do
    /// without re-deriving the six families by hand.
    pub(crate) fn attacker_target_block(
        &self,
        p: usize,
        id: CardId,
        kws: &[Keyword],
        defender: Option<usize>,
    ) -> Option<(u32, GameError)> {
        // One walk, same reason as `attacker_self_block`.
        for k in kws {
            match k {
                // "Can't attack unless defending player controls a [filter]"
                // (Dandân).
                Keyword::CanAttackOnlyIfDefenderControls(req)
                    if !defender.is_some_and(|d| {
                        self.battlefield.iter().any(|c| {
                            c.controller == d && self.evaluate_requirement_on_card(req, c, d)
                        })
                    }) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 725 — Crown-Hunter Hireling: only the monarch.
                Keyword::CantAttackUnlessDefenderIsMonarch if defender != self.monarch => {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Merchant Ship: the defender controls a land of
                // the named type.
                Keyword::CantAttackUnlessDefenderControlsLandType(lt)
                    if defender.is_some_and(|d| {
                        !self.battlefield.iter().any(|c| {
                            c.controller == d && c.definition.subtypes.land_types.contains(lt)
                        })
                    }) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Branded Brawlers: any untapped land locks it.
                Keyword::CantAttackIfDefenderHasUntappedLand
                    if defender.is_some_and(|d| {
                        self.battlefield
                            .iter()
                            .any(|c| c.controller == d && c.definition.is_land() && !c.tapped)
                    }) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Mogg Toady: strictly more creatures.
                Keyword::CantAttackUnlessMoreCreaturesThanDefender
                    if defender
                        .is_some_and(|d| self.creature_count(p) <= self.creature_count(d)) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Monstrous Hound: more lands than the defender.
                Keyword::CantAttackUnlessMoreLandsThanDefender
                    if defender.is_some_and(|d| {
                        self.player_tally(p, crate::card::PlayerTally::LandsControlled)
                            <= self.player_tally(d, crate::card::PlayerTally::LandsControlled)
                    }) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                _ => {}
            }
        }
        None
    }

    /// Does `kws` carry any family [`Self::attacker_target_block`] answers?
    /// The requirement loops walk the whole battlefield, so the "any legal
    /// defender" search stays off boards that cannot ask the question.
    fn has_defender_dependent_restriction(kws: &[Keyword]) -> bool {
        kws.iter().any(|k| {
            matches!(
                k,
                Keyword::CanAttackOnlyIfDefenderControls(_)
                    | Keyword::CantAttackUnlessDefenderIsMonarch
                    | Keyword::CantAttackUnlessDefenderControlsLandType(_)
                    | Keyword::CantAttackIfDefenderHasUntappedLand
                    | Keyword::CantAttackUnlessMoreCreaturesThanDefender
                    | Keyword::CantAttackUnlessMoreLandsThanDefender
            )
        })
    }

    /// The Ensnaring Bridge-style caps in scope, one per static (CR 613 —
    /// each reads its own controller's hand). `statics` is
    /// [`attack_static_scan`]'s bitmask, so a board without the family pays
    /// one bit test.
    pub(crate) fn attack_power_caps(&self, statics: u32) -> Vec<usize> {
        if statics & attack_static::POWER_CAP == 0 {
            return Vec::new();
        }
        self.battlefield
            .iter()
            .filter(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::AttackPowerCapByControllerHand
                    )
                })
            })
            .map(|c| self.players[c.controller].hand.len())
            .collect()
    }

    /// The engine's own answer to "may `card` be declared as an attacker
    /// against `defender`", batch-independent.
    ///
    /// The bot's attack picker calls this instead of re-deriving the gate, so
    /// its candidate filter cannot drift from
    /// [`Self::declare_attackers_banded`] — the picker/engine disagreement
    /// class PERF (-55) is made of. Batch-level rules (Okk's bigger partner,
    /// the participation cap, attacks-alone) are the caller's, because they
    /// are not properties of one creature.
    pub(crate) fn may_declare_attacker(
        &self,
        p: usize,
        card: &crate::card::CardInstance,
        cp: Option<&ComputedPermanent>,
        power_caps: &[usize],
        defender: Option<usize>,
    ) -> bool {
        self.attacker_self_block(p, card, cp, power_caps).is_none()
            && self
                .attacker_target_block(
                    p,
                    card.id,
                    cp.map(|c| c.keywords()).unwrap_or(&[]),
                    defender,
                )
                .is_none()
    }

    /// CR 508.1d — is `card` *able* to attack at all? The predicate the three
    /// "attacks each combat if able" requirement loops share: nothing about
    /// the creature or the board blocks it, and at least one legal defending
    /// player survives the defender-dependent restrictions.
    ///
    /// Deliberately the same walkers the declaration gate runs, because a
    /// requirement the gate then rejects has no legal answer.
    pub(crate) fn attacker_is_able(
        &self,
        p: usize,
        card: &crate::card::CardInstance,
        cp: Option<&ComputedPermanent>,
        power_caps: &[usize],
        statics: u32,
    ) -> bool {
        if self.attacker_self_block(p, card, cp, power_caps).is_some() {
            return false;
        }
        let kws: &[Keyword] = cp.map(|c| c.keywords()).unwrap_or(&[]);
        let taxed = self.attack_tax_possible(statics);
        if !Self::has_defender_dependent_restriction(kws) && !taxed {
            return true;
        }
        (0..self.players.len()).any(|d| {
            !self.same_team(p, d)
                && self.players[d].is_alive()
                && self.player_in_range_of(p, d)
                && self.attacker_target_block(p, card.id, kws, Some(d)).is_none()
                && (!taxed || self.attack_cost_payable(p, card.id, kws, d, statics))
        })
    }

    /// CR 508.1g — can attacking cost anything at all on this board? Three
    /// sources, and `attack_static_scan`'s bitmask covers only the first: the
    /// `AttackTaxToController` statics (Propaganda, Sphere of Safety), War
    /// Tax's turn-scoped symmetric tax, and Forbidding Spirit's per-seat one.
    pub(crate) fn attack_tax_possible(&self, statics: u32) -> bool {
        statics & attack_static::ATTACK_TAX != 0
            || self.attack_tax_this_turn > 0
            || self.players.iter().any(|pl| pl.attack_tax_until_your_turn > 0)
    }

    /// CR 508.1g — could `p` pay what it costs for `id` to attack `d`?
    ///
    /// **A creature whose attack cost cannot be paid is not *able* to attack,
    /// so CR 508.1d must not require it.** Without this, a Juggernaut behind a
    /// Propaganda its controller has no mana for is required to attack by the
    /// requirement loops and rejected for attacking by the tax gate below
    /// them — the seat has no legal declaration in either direction and loses
    /// its whole combat. `build_cube_state_seeded(3637)` is that board.
    ///
    /// The probe is a state clone plus an auto-tap ([`could_pay_generic`]), so
    /// every caller reaches it behind [`attack_tax_possible`] *and* behind a
    /// must-attack presence gate. It prices this attacker alone; the batch's
    /// total is the declaration gate's business, and asking the narrower
    /// question here is the safe direction — a requirement that is weaker than
    /// the gate leaves a legal declaration, and one that is stronger does not.
    fn attack_cost_payable(
        &self,
        p: usize,
        id: CardId,
        kws: &[Keyword],
        d: usize,
        statics: u32,
    ) -> bool {
        let atk = Attack { attacker: id, target: crate::game::types::AttackTarget::Player(d) };
        let tax = self.attack_tax_for(std::slice::from_ref(&atk), statics, |_| {
            self.attack_block_keyword_tax(id, kws, true)
        });
        tax == 0 || self.could_pay_generic(p, tax)
    }


    pub fn declare_attackers(
        &mut self,
        attacks: Vec<Attack>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.declare_attackers_banded(attacks, vec![])
    }

    /// [`declare_attackers`] with CR 702.22c attacking bands announced in the
    /// same step. Each `bands` entry lists one band's members.
    ///
    /// [`declare_attackers`]: Self::declare_attackers
    pub fn declare_attackers_banded(
        &mut self,
        attacks: Vec<Attack>,
        bands: Vec<Vec<CardId>>,
    ) -> Result<Vec<GameEvent>, GameError> {
        if self.step != TurnStep::DeclareAttackers {
            return Err(GameError::WrongStep { actual: self.step });
        }
        // Master Warcraft — an outside chooser declares in the active
        // player's place; the attackers are still the active player's
        // creatures, so only the *submitter* changes.
        if self.priority.player_with_priority != self.attack_declarer() {
            return Err(GameError::NotYourPriority);
        }
        // Peace Talks — CR 508.1a, nobody attacks for its two turns.
        if self.truce_active() && !attacks.is_empty() {
            return Err(attack_reject(line!(), GameError::CannotAttack(attacks[0].attacker)));
        }
        let p = self.active_player_idx;
        // One battlefield walk for the four static prohibitions this
        // declaration checks; see [`attack_static`]. Two of the four are
        // asked *per attacker*, so on a board carrying none of them this
        // replaces (attackers x battlefield x static abilities) with one
        // pass.
        let statics = attack_static_scan(self);

        // CR 803.1a/b — under the attack-left / attack-right option, the only
        // legal defending player is the nearest living opponent in that
        // direction (and "more than one seat away" means you can't attack at
        // all, which falls out of the same walk).
        let seat_restriction = self.attack_left_right_defender();

        // Validate every attack target up-front. The defender must be an
        // *opponent* — not self, not a teammate. `same_team` returns true
        // for `a == b`, so this single check rules out both cases. In
        // 1v1 / FFA it behaves identically to the old `target != active`
        // check; in 2HG / team formats it correctly rejects targeting a
        // teammate's life total or planeswalker.
        for atk in &attacks {
            match atk.target {
                AttackTarget::Player(target_player) => {
                    if target_player >= self.players.len()
                        || self.same_team(self.active_player_idx, target_player)
                        || !self.players[target_player].is_alive()
                        || seat_restriction.is_some_and(|only| only != Some(target_player))
                        // CR 801.3 — only opponents inside the attacker's range.
                        || !self.player_in_range_of(p, target_player)
                        // CR 809.3c — Emperor: only the seats either side.
                        || !self.seat_attackable_from(p, target_player)
                        // "Creatures they control can't attack you this turn"
                        // (Web of Inertia).
                        || self
                            .cant_attack_player_this_turn
                            .contains(&(self.active_player_idx, target_player))
                    {
                        return Err(GameError::InvalidAttackTarget(target_player));
                    }
                }
                AttackTarget::Planeswalker(pw_id) => {
                    let pw = self
                        .battlefield_find(pw_id)
                        .ok_or(GameError::InvalidPlaneswalkerAttackTarget(pw_id))?;
                    if !pw.definition.is_planeswalker()
                        // CR 506.2 — "can't be attacked" (The Aetherspark
                        // while attached to a creature).
                        || self.permanent_cant_be_attacked(pw_id)
                        || self.same_team(self.active_player_idx, pw.controller)
                        || !self.players[pw.controller].is_alive()
                        || seat_restriction.is_some_and(|only| only != Some(pw.controller))
                        || !self.player_in_range_of(p, pw.controller)
                        || !self.seat_attackable_from(p, pw.controller)
                    {
                        return Err(GameError::InvalidPlaneswalkerAttackTarget(pw_id));
                    }
                }
                // CR 508.4 — a battle is a legal target as long as the active
                // player isn't its protector (you can attack your own Siege,
                // which a teammate-checked planeswalker arm would reject).
                AttackTarget::Battle(b_id) => {
                    let b = self
                        .battlefield_find(b_id)
                        .ok_or(GameError::InvalidPlaneswalkerAttackTarget(b_id))?;
                    let protector = b.protected_by;
                    if !b.definition.is_battle()
                        || self.permanent_cant_be_attacked(b_id)
                        || protector == Some(self.active_player_idx)
                        || protector.is_none_or(|pr| !self.players[pr].is_alive())
                    {
                        return Err(GameError::InvalidPlaneswalkerAttackTarget(b_id));
                    }
                }
            }
        }

        // CR 508.1d — "attacks each combat if able" (Juggernaut, goaded
        // creatures). Any creature the active player controls that carries
        // MustAttack and *can* legally attack (untapped, not sick / has
        // Haste, not Defender / CantAttack) must be in the declared batch
        // while at least one opponent is in range. Reject an incomplete
        // declaration so the requirement is honored.
        let has_legal_target = self
            .players
            .iter()
            .enumerate()
            .any(|(i, pl)| !self.same_team(p, i) && pl.is_alive());
        // CR 508.1d — Magnetic Web: once one of the group attacks, every
        // able member of the group has to join it. Built before the layer
        // pass because an empty group list is what lets the pass stay small
        // — it reads only the battlefield and the active statics, so the
        // hoist is mechanical.
        // Two `for` loops rather than `flat_map(..).filter_map(..).collect()`:
        // this is a whole-board walk executed once per declaration, and
        // `FlatMap::next` costs ~20 Ir a permanent whether or not any card's
        // `static_abilities` list has anything in it (PERF (-78)).
        let mut groups: Vec<crate::card::SelectionRequirement> = Vec::new();
        if has_legal_target {
            for c in self.battlefield.iter() {
                for sa in &c.definition.static_abilities {
                    if let Some(crate::effect::StaticEffect::AttackTogether { filter }) =
                        self.active_static(&sa.effect, c)
                    {
                        groups.push(filter.clone());
                    }
                }
            }
        }
        // CR 508.1a/d — Oracle en-Vec's mandate: only the chosen creatures may
        // attack, and each of them that can attack must.
        let mandate = self.armed_attack_mandate_for(p);

        // One layer pass for the whole declaration. Everything from here to
        // the trigger collection is validation — nothing mutates a layer
        // input — and the band, attacks-alone, can't-attack-alone and trigger
        // passes were each taking their own, up to four per call.
        //
        // Only two consumers below read the *whole* board: the CR 508.1d
        // requirement loop and the `AttackTogether` loop. Both are no-ops
        // unless a permanent can carry an attack requirement, so when none
        // can, the pass covers just the declared attackers, the band members
        // and the creatures already attacking — a handful instead of ~23.
        // One freeze scope so the gate and the pass share a single gather.
        let (computed, attack_requirement) = self.with_frozen_layers(|g| {
            let requirement = has_legal_target
                && (g.battlefield.iter().any(|c| !c.goaded_by.is_empty())
                    || g.board_keyword_in_scope(&[
                        Keyword::MustAttack,
                        Keyword::MustAttackOrBlock,
                        Keyword::MustAttackIfAnotherAttacks,
                    ]));
            if requirement || !groups.is_empty() {
                return (g.compute_battlefield(), requirement);
            }
            let mut ids: SmallVec<[CardId; 8]> = SmallVec::new();
            let mut want = |id: CardId| {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            };
            attacks.iter().for_each(|a| want(a.attacker));
            bands.iter().flatten().for_each(|&m| want(m));
            g.attacking.iter().for_each(|a| want(a.attacker));
            mandate.iter().flatten().for_each(|&id| want(id));
            (g.compute_permanents(&ids), requirement)
        });
        // A whole-board consumer that slipped past the gate above would
        // silently read `&[]` for every permanent outside the subset and drop
        // its restriction. Panic on it across the suite instead.
        #[cfg(debug_assertions)]
        let computed_ids: Vec<CardId> = computed.iter().map(|c| c.id).collect();
        #[cfg(debug_assertions)]
        let battlefield_ids: Vec<CardId> = self.battlefield.iter().map(|c| c.id).collect();

        // CR 702.22c-d — band legality: every member must be attacking, at
        // most one may lack banding, at least one must have it, and they must
        // all attack the same defender. Read from the *computed* keyword set
        // so a granted banding counts.
        {
            let has_banding = |id: CardId| {
                computed.iter().any(|c| c.id == id && c.keywords().has_kw(&Keyword::Banding))
            };
            for members in &bands {
                let Some(&first) = members.first() else { continue };
                let targets: Vec<AttackTarget> = members
                    .iter()
                    .map(|m| {
                        attacks
                            .iter()
                            .find(|a| a.attacker == *m)
                            .map(|a| a.target)
                            .ok_or(GameError::CannotAttack(first))
                    })
                    .collect::<Result<_, _>>()?;
                if targets.iter().any(|t| *t != targets[0]) {
                    return Err(attack_reject(line!(), GameError::CannotAttack(first)));
                }
                let unbanded = members.iter().filter(|m| !has_banding(**m)).count();
                let plain_band_ok = unbanded <= 1 && unbanded < members.len();
                // CR 702.22d — the "bands with other [quality]" alternative:
                // every member matches the quality and at least one of them
                // has the ability (Adventurers' Guildhouse).
                let quality_band_ok = !plain_band_ok
                    && self.bands_with_other_qualities(members).iter().any(|q| {
                        members.iter().all(|m| {
                            self.evaluate_requirement_static(q, &Target::Permanent(*m), 0, None)
                        })
                    });
                if !plain_band_ok && !quality_band_ok {
                    return Err(attack_reject(line!(), GameError::CannotAttack(first)));
                }
            }
        }

        // CR 508.1 — Crawlspace: no more than N creatures can attack a player
        // who controls an `AttackerCapAgainstController` permanent.
        debug_assert!(
            statics & attack_static::ATTACKER_CAP != 0
                || !self.battlefield.iter().any(|c| c
                    .definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(
                        sa.effect,
                        crate::effect::StaticEffect::AttackerCapAgainstController { .. }
                    ))),
            "attack_static_scan missed an attacker cap",
        );
        for p in 0..self.players.len() {
            if statics & attack_static::ATTACKER_CAP == 0 {
                break; // no cap on the board — the per-seat walks below are all `None`
            }
            let Some(cap) = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p)
                .filter_map(|c| {
                    c.definition.static_abilities.iter().find_map(|sa| match sa.effect {
                        crate::effect::StaticEffect::AttackerCapAgainstController { n } => Some(n),
                        _ => None,
                    })
                })
                .min()
            else {
                continue;
            };
            let against =
                attacks.iter().filter(|a| a.target == AttackTarget::Player(p)).count();
            if against > cap {
                return Err(GameError::InvalidAttackTarget(p));
            }
        }

        // CR 701.15b — a goaded creature "attacks a player other than the
        // controller of the [goad source] if able." Enforce the player-target
        // half: a goaded attacker may not attack one of its own goaders while
        // an unattacked, alive non-goader opponent it could instead attack
        // exists. (Planeswalker/battle redirection is not modeled here.)
        for atk in &attacks {
            if let AttackTarget::Player(target_player) = atk.target
                && let Some(c) = self.battlefield_find(atk.attacker)
                && c.goaded_by.contains(&target_player)
            {
                let has_nongoader_option = (0..self.players.len()).any(|q| {
                    q != self.active_player_idx
                        && !self.same_team(self.active_player_idx, q)
                        && self.players[q].is_alive()
                        && !c.goaded_by.contains(&q)
                });
                if has_nongoader_option {
                    return Err(GameError::InvalidAttackTarget(target_player));
                }
            }
        }

        // Angelic Arbiter — an opponent who cast a spell this turn can't
        // attack with creatures at all.
        if !attacks.is_empty() {
            let p = self.active_player_idx;
            if self.players[p].spells_cast_this_turn > 0
                && self.opponent_has_static(p, |e| {
                    matches!(e, crate::effect::StaticEffect::OpponentsWhoCastCantAttack)
                })
            {
                return Err(attack_reject(line!(), GameError::CannotAttack(attacks[0].attacker)));
            }
        }

        // CR 508.0 — "attacks only alone" (Master of Cruelties). If any
        // declared attacker carries AttacksAlone, the batch must be a
        // single attacker. Read from the computed keyword set so granted
        // variants count.
        if attacks.len() > 1
            && attacks.iter().any(|atk| {
                computed
                    .iter()
                    .find(|c| c.id == atk.attacker)
                    .is_some_and(|c| c.keywords().has_kw(&Keyword::AttacksAlone))
            })
        {
            return Err(attack_reject(line!(), GameError::CannotAttack(attacks[0].attacker)));
        }

        // CR 508.0 — "can't attack alone" (Militia Rallier). A lone attacker
        // carrying CantAttackAlone makes the batch illegal.
        if attacks.len() == 1
            && computed.iter().find(|c| c.id == attacks[0].attacker).is_some_and(|c| {
                c.keywords().has_kw(&Keyword::CantAttackAlone)
                    || c.keywords().has_kw(&Keyword::CantAttackOrBlockAlone)
            })
        {
            return Err(attack_reject(line!(), GameError::CannotAttack(attacks[0].attacker)));
        }

        // CR 506.2 — Silent Arbiter: "No more than N creatures can attack each
        // combat." The cap covers the whole combat, so count attackers already
        // declared this combat alongside the incoming batch.
        if let Some(cap) = self.combat_participation_cap(false)
            && let Some(first) = attacks.first()
            && self.attacking.len() + attacks.len() > cap as usize
        {
            return Err(attack_reject(line!(), GameError::CannotAttack(first.attacker)));
        }

        let mut events = vec![];
        // Per CR 506.5, the Attacks trigger filter must be evaluated
        // post-batch, so we carry the optional filter alongside each
        // queued trigger.
        let mut triggers: Vec<(
            CardId,
            Effect,
            usize,
            Option<crate::effect::Predicate>,
        )> = vec![];
        let computed_kw = |id: CardId| -> &[Keyword] {
            #[cfg(debug_assertions)]
            debug_assert!(
                computed_ids.contains(&id) || !battlefield_ids.contains(&id),
                "computed_kw({id:?}) read a battlefield permanent outside the gated \
                 subset — a whole-board consumer needs the gate widened"
            );
            computed
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.keywords())
                .unwrap_or(&[])
        };

        // Hoisted above the CR 508.1d requirement loops: an Ensnaring Bridge
        // cap is one of the restrictions that make a must-attack creature
        // *unable*, so those loops need it too. Gated by the
        // `attack_static_scan` bitmask, so a board without the family pays
        // one bit test.
        let attack_power_caps = self.attack_power_caps(statics);
        // CR 508.1d — one *able* predicate for all three requirement loops
        // below and for the declaration gate under them. Asking a narrower
        // question here than the gate asks is what leaves a seat with no
        // legal declaration in either direction: see `attacker_is_able`.
        let able_to_attack = |c: &crate::card::CardInstance| {
            self.attacker_is_able(
                p,
                c,
                computed.iter().find(|x| x.id == c.id),
                &attack_power_caps,
                statics,
            )
        };

        if let Some(chosen) = mandate {
            if let Some(bad) = attacks.iter().find(|a| !chosen.contains(&a.attacker)) {
                return Err(attack_reject(line!(), GameError::CannotAttack(bad.attacker)));
            }
            for id in chosen {
                let Some(c) = self.battlefield.iter().find(|c| c.id == id && c.controller == p)
                else {
                    continue;
                };
                if able_to_attack(c)
                    && has_legal_target
                    && !attacks.iter().any(|a| a.attacker == id)
                {
                    return Err(attack_reject(line!(), GameError::CannotAttack(id)));
                }
            }
        }

        if attack_requirement {
            for c in &self.battlefield {
                // A creature must be declared if it carries MustAttack
                // (Juggernaut) or is goaded (CR 701.38 — "attacks each
                // combat if able").
                let must = computed_kw(c.id).has_kw(&Keyword::MustAttack)
                    || computed_kw(c.id).has_kw(&Keyword::MustAttackOrBlock)
                    // CR 508.1d — Ekundu Cyclops only has to join an attack
                    // someone else already started.
                    || (computed_kw(c.id).has_kw(&Keyword::MustAttackIfAnotherAttacks)
                        && attacks.iter().any(|a| a.attacker != c.id))
                    || !c.goaded_by.is_empty();
                if c.controller != p || !must {
                    continue;
                }
                if able_to_attack(c) && !attacks.iter().any(|atk| atk.attacker == c.id) {
                    return Err(attack_reject(line!(), GameError::CannotAttack(c.id)));
                }
            }
        }
        // CR 508.1d — Magnetic Web (`groups`, built above the layer pass).
        for filter in groups {
            let matches = |id: CardId| {
                self.evaluate_requirement_static(&filter, &Target::Permanent(id), p, None)
            };
            if !attacks.iter().any(|atk| matches(atk.attacker)) {
                continue;
            }
            for c in &self.battlefield {
                if c.controller != p || !matches(c.id) {
                    continue;
                }
                if able_to_attack(c) && !attacks.iter().any(|atk| atk.attacker == c.id) {
                    return Err(attack_reject(line!(), GameError::CannotAttack(c.id)));
                }
            }
        }

        // CR 601.2h-style atomicity: validate the entire declaration before
        // any state is spent (attack tax) or mutated (tapping attackers) —
        // one illegal attacker must not corrupt the batch.
        {
            debug_assert!(
                statics & attack_static::POWER_CAP != 0
                    || !self.battlefield.iter().any(|c| c
                        .definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(
                            sa.effect,
                            crate::effect::StaticEffect::AttackPowerCapByControllerHand
                        ))),
                "attack_static_scan missed an attack power cap",
            );
            let mut seen = crate::fxhash::HashSet::default();
            for atk in &attacks {
                let id = atk.attacker;
                if !seen.insert(id) {
                    return Err(attack_reject(line!(), GameError::CannotAttack(id)));
                }
                // Controller (not owner) must be the active player.
                let card = self
                    .battlefield
                    .iter()
                    .find(|c| c.id == id && c.controller == p)
                    .ok_or(GameError::CardNotOnBattlefield(id))?;
                let cp = computed.iter().find(|c| c.id == id);
                let kws = computed_kw(id);
                // CR 508.1a — every restriction that reads only the creature
                // and the board, in the one walker the CR 508.1d requirement
                // loops above share. Two hand-written copies of this list is
                // the drift that made a must-attack creature unable to attack
                // and illegal not to.
                if let Some((line, e)) = self.attacker_self_block(p, card, cp, &attack_power_caps)
                {
                    return Err(attack_reject(line, e));
                }
                // CR 508.1a — and every restriction that reads *this* attack's
                // defender.
                if let Some((line, e)) =
                    self.attacker_target_block(p, id, kws, self.defender_for(atk.target))
                {
                    return Err(attack_reject(line, e));
                }
                // CR 508.1a — Okk: needs a strictly bigger partner in the same
                // declared batch (already-declared attackers count too). The
                // one per-attacker restriction that depends on the rest of the
                // batch, so it cannot move into the shared walker.
                if kws.has_kw(&Keyword::CantAttackUnlessGreaterPowerAttacks) {
                    let mine = cp.map(|c| c.power).unwrap_or(0);
                    if !attacks
                        .iter()
                        .map(|a| a.attacker)
                        .chain(self.attacking.iter().map(|a| a.attacker))
                        .filter(|other| *other != id)
                        .any(|other| {
                            computed.iter().find(|c| c.id == other).is_some_and(|c| c.power > mine)
                        })
                    {
                        return Err(attack_reject(line!(), GameError::CannotAttack(id)));
                    }
                }
            }
        }

        // CR 508.1 — absolute "creatures can't attack you" prohibition (Blazing
        // Archon). Reject the whole declaration if any attacker targets a player
        // (or protected planeswalker) whose controller has the static.
        for atk in &attacks {
            let (defender, at_planeswalker) = match atk.target {
                crate::game::types::AttackTarget::Player(d) => (Some(d), false),
                crate::game::types::AttackTarget::Planeswalker(pw) => {
                    (self.battlefield_find(pw).map(|c| c.controller), true)
                }
                crate::game::types::AttackTarget::Battle(b) => {
                    (self.battlefield_find(b).and_then(|c| c.protected_by), false)
                }
            };
            let Some(d) = defender else { continue };
            // CR 508.1 — Arboria: a player who did nothing on their own last
            // turn can't be attacked at all.
            if !self.acted_on_their_last_turn(d)
                && self.battlefield.iter().any(|c| {
                    c.definition.static_abilities.iter().any(|sa| {
                        matches!(
                            self.active_static(&sa.effect, c),
                            Some(crate::effect::StaticEffect::
                                PlayersCantBeAttackedUnlessTheyActedLastTurn)
                        )
                    })
                })
            {
                return Err(attack_reject(line!(), GameError::CannotAttack(atk.attacker)));
            }
            debug_assert!(
                statics & attack_static::CANT_ATTACK_CONTROLLER != 0
                    || !self.battlefield.iter().any(|c| c
                        .definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(
                            sa.effect,
                            crate::effect::StaticEffect::CreaturesCantAttackController { .. }
                        ))),
                "attack_static_scan missed an attack prohibition",
            );
            // Each live prohibition, asked in one pass: the old form collected
            // `(source id, cloned filter)` into a `Vec` and then ran `any` over
            // it, i.e. one allocation and one `SelectionRequirement` clone per
            // lock per attacker for a question that short-circuits.
            let barred = statics & attack_static::CANT_ATTACK_CONTROLLER != 0
                && self
                .battlefield
                .iter()
                .filter(|c| c.controller == d)
                .flat_map(|c| c.definition.static_abilities.iter().map(move |sa| (c.id, sa)))
                .any(|(src, sa)| match &sa.effect {
                    crate::effect::StaticEffect::CreaturesCantAttackController {
                        protect_planeswalkers,
                        filter,
                    } if !at_planeswalker || *protect_planeswalkers => {
                        filter.as_ref().is_none_or(|f| {
                            self.evaluate_requirement_static(
                                f,
                                &Target::Permanent(atk.attacker),
                                d,
                                Some(src),
                            )
                        })
                    }
                    _ => false,
                });
            if barred {
                return Err(attack_reject(line!(), GameError::CannotAttack(atk.attacker)));
            }
        }

        // CR 508.1g — the attack tax, computed by the one walker the bot's
        // picker also calls so the two cannot drift (`attack_tax_for`).
        let total_tax = self.attack_tax_for(&attacks, statics, |id| {
            self.attack_block_keyword_tax(id, computed_kw(id), true)
        });
        if total_tax > 0 {
            // Pay from the floating pool, auto-tapping mana sources for any
            // shortfall (rolled back atomically if unpayable).
            let tax_cost = crate::mana::cost(&[crate::mana::generic(total_tax)]);
            if self.try_pay_with_auto_tap(p, &tax_cost).is_err() {
                return Err(attack_reject(line!(), GameError::CannotAttack(attacks[0].attacker)));
            }
        }

        // CR 508.1g — Floodtide Serpent's attack cost: each such attacker
        // returns one matching permanent its controller controls to hand. The
        // pool is checked and consumed together so two Serpents need two
        // enchantments.
        {
            let mut spent: Vec<CardId> = Vec::new();
            for atk in &attacks {
                // The `collect` exists only so the keyword borrow ends before
                // the `&mut self` below; almost no attacker carries the
                // keyword, and an empty `collect()` still calls
                // `Vec::from_iter`. Ask the borrowed slice first — this and
                // its `AttackCostSacrifice` twin are two `from_iter`s per
                // declared attacker.
                let filters: Vec<crate::card::SelectionRequirement> = if !computed_kw(
                    atk.attacker,
                )
                .iter()
                .any(|k| matches!(k, Keyword::AttackCostBounce(_)))
                {
                    Vec::new()
                } else {
                    computed_kw(atk.attacker)
                        .iter()
                        .filter_map(|k| match k {
                            Keyword::AttackCostBounce(f) => Some((**f).clone()),
                            _ => None,
                        })
                        .collect()
                };
                for f in filters {
                    let pick = self.battlefield.iter().find(|c| {
                        c.controller == p
                            && !spent.contains(&c.id)
                            && self.evaluate_requirement_static(
                                &f,
                                &crate::game::types::Target::Permanent(c.id),
                                p,
                                Some(atk.attacker),
                            )
                    });
                    match pick {
                        Some(c) => spent.push(c.id),
                        None => return Err(GameError::CannotAttack(atk.attacker)),
                    }
                }
            }
            let mut events = Vec::new();
            let ctx = crate::game::effects::EffectContext::for_ability(CardId(0), p, None);
            for id in spent {
                self.move_card_to(
                    id,
                    &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::OwnerOfMoved),
                    &ctx,
                    &mut events,
                );
            }
            self.dispatch_triggers_for_events(&events);
        }

        // CR 508.1g — Leviathan's attack cost: sacrifice N matching permanents
        // per such attacker. The pool is shared, so two Leviathans need four
        // Islands.
        {
            let mut spent: Vec<CardId> = Vec::new();
            for atk in &attacks {
                let costs: Vec<(crate::card::SelectionRequirement, u32)> = if !computed_kw(
                    atk.attacker,
                )
                .iter()
                .any(|k| matches!(k, Keyword::AttackCostSacrifice(..)))
                {
                    Vec::new()
                } else {
                    computed_kw(atk.attacker)
                        .iter()
                        .filter_map(|k| match k {
                            Keyword::AttackCostSacrifice(f, n) => Some(((**f).clone(), *n)),
                            _ => None,
                        })
                        .collect()
                };
                for (f, n) in costs {
                    for _ in 0..n {
                        let pick = self.battlefield.iter().find(|c| {
                            c.controller == p
                                && !spent.contains(&c.id)
                                && self.evaluate_requirement_static(
                                    &f,
                                    &crate::game::types::Target::Permanent(c.id),
                                    p,
                                    Some(atk.attacker),
                                )
                        });
                        match pick {
                            Some(c) => spent.push(c.id),
                            None => return Err(GameError::CannotAttack(atk.attacker)),
                        }
                    }
                }
            }
            let mut events = Vec::new();
            for id in spent {
                self.sacrifice_one(id, p, &mut events);
            }
            self.dispatch_triggers_for_events(&events);
        }

        // CR 508.1g — Hollow Warrior's attack cost: tap an untapped matching
        // permanent that isn't itself attacking. One helper per such attacker.
        // Same shape: `declared` is built for `find_tap_helper`, and no
        // attacker carries a tap-another filter on any board the bench or the
        // sealed pool plays.
        if attacks.iter().any(|atk| {
            computed_kw(atk.attacker)
                .iter()
                .any(|k| matches!(k, Keyword::AttackBlockCostTapAnother(_)))
        }) {
            let declared: Vec<CardId> = attacks.iter().map(|a| a.attacker).collect();
            let mut tapped: SmallVec<[CardId; 4]> = SmallVec::new();
            for atk in &attacks {
                for f in tap_another_filters(computed_kw(atk.attacker)) {
                    match self.find_tap_helper(p, &f, atk.attacker, &declared, &tapped) {
                        Some(id) => tapped.push(id),
                        None => return Err(GameError::CannotAttack(atk.attacker)),
                    }
                }
            }
            for id in tapped {
                if let Some(c) = self.battlefield_find_mut(id) {
                    c.tapped = true;
                }
            }
        }

        let any_attackers = !attacks.is_empty();
        // CR 702.121 — Melee counts the distinct opponents this player attacked
        // this combat (a player targeted directly, or the controller of a
        // planeswalker/battle attacked). Computed over the whole batch up front.
        let melee_opponents: i32 = {
            let mut seats: crate::fxhash::HashSet<usize> = crate::fxhash::HashSet::default();
            for atk in &attacks {
                let seat = match atk.target {
                    AttackTarget::Player(s) => Some(s),
                    AttackTarget::Planeswalker(cid) | AttackTarget::Battle(cid) => {
                        self.battlefield_find(cid).map(|c| c.controller)
                    }
                };
                if let Some(s) = seat.filter(|s| !self.same_team(*s, p)) {
                    seats.insert(s);
                }
            }
            seats.len() as i32
        };
        // Statics-granted triggers ("Slivers you control have '…attacks…'" —
        // Thorncaster) and equipment-granted triggers (CR 702.6e — "whenever
        // equipped creature attacks, …") both come off whole-board scans —
        // `trigger_grant_sources` and `equip_granted_trigger_sources`. The
        // scans' answer is the same for every attacker in this batch, so
        // walk the board once here rather than per attacker inside the loop
        // below. Consumed by value in the main loop so its `&mut self`
        // borrow of `battlefield.iter_mut()` is unblocked.
        let attacker_grants: Vec<(
            Vec<crate::card::TriggeredAbility>,
            Vec<crate::card::TriggeredAbility>,
        )> = {
            let trigger_grants = self.trigger_grant_sources();
            let equip_grants = self.equip_granted_trigger_sources();
            attacks
                .iter()
                .map(|atk| {
                    self.battlefield
                        .iter()
                        .find(|c| c.id == atk.attacker)
                        .map(|c| {
                            (
                                self.statics_granted_triggers_with(c, &trigger_grants),
                                self.equip_granted_triggers_with(c, &equip_grants),
                            )
                        })
                        .unwrap_or_default()
                })
                .collect()
        };
        for (atk, (static_granted, equip_granted)) in attacks.into_iter().zip(attacker_grants) {
            let id = atk.attacker;
            // Validated above — commit only. Filter by *controller*, not
            // *owner*: a stolen creature (Threaten / Mind Control) attacks
            // for its current controller.
            // Both of these read the cold group, whose `Deref` borrows the
            // whole state — take them before the battlefield `&mut`.
            let decayed = computed_kw(id).has_kw(&Keyword::Decayed);
            let granted: Vec<crate::card::TriggeredAbility> =
                self.granted_triggers_eot.get(&id).cloned().unwrap_or_default();
            let card = self
                .battlefield
                .iter_mut()
                .find(|c| c.id == id && c.controller == p)
                .ok_or(GameError::CardNotOnBattlefield(id))?;
            if !computed_kw(id).has_kw(&Keyword::Vigilance) {
                card.tapped = true;
                // CR 508.1f — attacking taps the creature; surface a
                // "becomes tapped" event so Tapped triggers fire (Magda).
                events.push(GameEvent::PermanentTapped { card_id: id, actor: None, as_attacker: true });
            }
            // CR 702.83 — Exert. We auto-exert any attacking creature with
            // the keyword (the "you may" choice is collapsed; the AutoDecider
            // would have no policy and a real exert is almost always taken for
            // its bonus). The creature won't untap next untap step. Its exert
            // bonus rides its normal SelfSource Attacks trigger.
            if computed_kw(id).has_kw(&Keyword::Exert) {
                card.skip_next_untap = true;
            }
            // CR 702.121 — Melee: +1/+1 until end of turn per opponent attacked.
            if melee_opponents > 0 && computed_kw(id).has_kw(&Keyword::Melee) {
                card.power_bonus += melee_opponents;
                card.toughness_bonus += melee_opponents;
            }
            // CR 702.142 — record that this creature attacked (gates Boast).
            card.attacked_this_turn = true;
            card.attacked_own_turn = true;
            self.attacking.push(atk);
            // Raid (CR 702.108 ability word): the controller attacked this turn.
            self.players[p].attacked_this_turn = true;
            self.players[p].creatures_attacked_this_turn += 1;
            events.push(GameEvent::AttackerDeclared(id));
            // Walk printed Attacks triggers + any transient granted
            // Attacks triggers (Root Manipulation's "gain 1 life when
            // this attacks" grant lands in `granted_triggers_eot`).
            for t in card
                .definition
                .triggered_abilities
                .iter()
                .chain(granted.iter())
                .chain(static_granted.iter())
                .chain(equip_granted.iter())
            {
                // Only SelfSource Attacks triggers are hardcoded here.
                // YourControl-scoped Attacks triggers (Exalted via
                // `Predicate::AttackingAlone`, Battle Banner, …) are
                // routed through the unified `dispatch_triggers_for_events`
                // path off the `AttackerDeclared` event — pushing them
                // here too would double-fire the ability.
                if t.event.kind == EventKind::Attacks
                    && t.event.scope == crate::effect::EventScope::SelfSource
                {
                    // Capture the trigger's optional filter so we can
                    // re-evaluate it AFTER the entire attacker batch is
                    // declared (CR 506.5 "attacking alone" semantics
                    // require the post-batch view).
                    triggers.push((id, t.effect.clone(), p, t.event.filter.clone()));
                }
            }
            // CR 702.147 — Decayed. "When it attacks, sacrifice it at end of
            // combat." Reuse the attacking-token cleanup queue (CR 511.3).
            if decayed {
                self.attacking_token_cleanup.push((
                    id,
                    crate::effect::AttackingTokenCleanup::SacrificeAtEndOfCombat,
                ));
            }
            // Annihilator N — CR 702.85a: "Whenever this creature attacks,
            // defending player sacrifices N permanents." Translate the
            // keyword to an Attacks-trigger that fires
            // `Effect::Sacrifice { who: defender, count: N, filter: Any }`.
            // The defender comes from `atk.target`; for a planeswalker
            // attack, that's the planeswalker's controller (CR 506.4a).
            let annihilator_n = computed_kw(id).iter().find_map(|kw| {
                if let Keyword::Annihilator(n) = kw {
                    Some(*n)
                } else {
                    None
                }
            });
            if let Some(n) = annihilator_n
                && let Some(defender) = self.defender_for(atk.target)
            {
                let sac_effect = Effect::Sacrifice {
                    who: Selector::Player(crate::effect::PlayerRef::Seat(defender)),
                    count: Value::Const(n as i32),
                    filter: crate::card::SelectionRequirement::Permanent,
                };
                triggers.push((id, sac_effect, p, None));
            }
            // Firebending N — CR 702.189a: a triggered mana ability (resolves
            // without the stack, CR 605.3b). Add N {R} now; the mana survives
            // step/phase emptying until end of combat (`firebending_kept_red`).
            let firebend_n = computed_kw(id).iter().find_map(|kw| match kw {
                Keyword::Firebending(n) => Some(*n),
                // Firebending X = this creature's power (clamped at 0).
                Keyword::FirebendingPower => Some(
                    self.computed_permanent(id)
                        .map(|c| c.power.max(0) as u32)
                        .unwrap_or(0),
                ),
                Keyword::FirebendingCreaturesYouControl => Some(
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == p && c.definition.is_creature())
                        .count() as u32,
                ),
                _ => None,
            });
            if let Some(n) = firebend_n
                && n > 0
            {
                self.players[p].mana_pool.add(crate::mana::Color::Red, n);
                self.players[p].firebending_kept_red =
                    self.players[p].firebending_kept_red.saturating_add(n);
            }
            // ControllerAttackedByOpponent (CR 508.1g listeners): permanents
            // the defending player controls that fire "when a creature an
            // opponent attacks me" — Coveted Jewel's control-flip. The
            // attacking creature's controller is bound to the trigger's
            // target slot ("that creature's controller").
            if let Some(defender) = self.defender_for(atk.target)
                && defender != p
            {
                // The planeswalker-only sibling scope fires only when the
                // attack is aimed at a planeswalker (Mila) — not the player.
                let is_pw_attack =
                    matches!(atk.target, crate::game::types::AttackTarget::Planeswalker(_));
                // Plain loops, same reason as `groups` above: a whole-board
                // walk per attacker, and the `flat_map` was ~20 Ir a
                // permanent before the filter saw a single trigger.
                let mut listeners: Vec<(CardId, Effect)> = Vec::new();
                for c in self.battlefield.iter() {
                    if c.controller != defender {
                        continue;
                    }
                    for t in &c.definition.triggered_abilities {
                        if t.event.kind == EventKind::Attacks
                            && (t.event.scope
                                == crate::effect::EventScope::ControllerAttackedByOpponent
                                || (is_pw_attack
                                    && t.event.scope
                                        == crate::effect::EventScope::ControllerPlaneswalkerAttackedByOpponent))
                        {
                            listeners.push((c.id, t.effect.clone()));
                        }
                    }
                }
                for (src, effect) in listeners {
                    self.stack.push(
                        TriggerPush::new(src, defender, effect)
                            .target(Some(Target::Player(p)))
                            // Bind the attacking creature as the trigger source so
                            // "those creatures get -1/-0" (Sabotage Strategist) can
                            // address it via `Selector::TriggerSource`.
                            .trigger_source(Some(crate::game::effects::EntityRef::Permanent(id)))
                            .build(),
                    );
                }
            }
        }
        // CR 702.22e — a declared band lasts for the rest of combat regardless
        // of later banding loss; drop members that never made it into combat.
        // Guarded: `attack_bands` is a `ColdState` field, so writing it
        // deep-copies the whole cold group — once per declare-attackers, and
        // banding is empty on every board without a bander. Compare first;
        // the comparison is a `&self` read and takes nothing.
        let declared: Vec<Vec<CardId>> = bands
            .into_iter()
            .map(|b| {
                b.into_iter().filter(|m| self.attacking.iter().any(|a| a.attacker == *m)).collect()
            })
            .filter(|b: &Vec<CardId>| b.len() > 1)
            .collect();
        if self.attack_bands != declared {
            self.attack_bands = declared;
        }

        // YourControl-scoped Attacks triggers (e.g. Battle Banner,
        // Sparring Regimen) are NOT walked here — the unified
        // `dispatch_triggers_for_events` path in `mod.rs` picks them up
        // off the `AttackerDeclared` event(s) and routes them through the
        // same trigger pipeline. Walking them here additionally would
        // double-fire the trigger (one push from combat.rs + one from
        // the dispatcher). The hardcoded `is_event_hardcoded` check only
        // marks SelfSource Attacks as already handled.

        for (source, effect, controller, filter) in triggers {
            // CR 603.2 + CR 506.5: evaluate the trigger's optional filter
            // predicate at fire-time, which for Attacks is "after the
            // entire declare attackers step batch is resolved".
            if let Some(predicate) = filter {
                let ctx = crate::game::effects::EffectContext {
                    controller,
                    source: Some(source),
                    targets: vec![],
                    // The attacker is a battlefield permanent — bind it as such
                    // so `ToughnessOf(TriggerSource)`-style gates (Young Hero
                    // Role) resolve its P/T (CR 506.5 post-batch view).
                    trigger_source: Some(crate::game::effects::EntityRef::Permanent(source)),
                    mode: 0,
                    x_value: 0,
                    converged_value: 0,
                    mana_spent: 0,
                    mana_spent_by_color: Vec::new(),
                    source_name: None,
                    cast_from_hand: true,
                    event_amount: 0,
                    kicked: false,
                    kicked_options: Vec::new(),
                    kick_count: 0,
                    bargained: false,
                    cast_via_mayhem: false,
                    cast_via_waterbend: false,
                    cast_collected_evidence: false,
                    entwined: false,
                    spree_modes: Vec::new(),
                };
                if !self.evaluate_predicate(&predicate, &ctx) {
                    continue;
                }
            }
            let auto_target =
                self.auto_target_for_effect_avoiding(&effect, controller, Some(source));
            // CR 115.1c — fill any additional "up to N target" slots (Lagorin's
            // "put a +1/+1 counter on each of up to two target Mounts and/or
            // Vehicles"), same as the self-source ETB path.
            let additional =
                self.auto_extra_targets_for(&effect, source, controller, auto_target.clone());
            // Isshin / Windcrag Siege (Mardu): a self-source attack trigger of a
            // permanent you control fires an additional time per doubler.
            let fires = 1
                + self.attack_trigger_extra_fires(controller)
                + crate::game::actions::ally_trigger_extra_fires(self, controller, source);
            for _ in 0..fires {
                self.stack.push(
                    TriggerPush::new(source, controller, effect.clone())
                        .target(auto_target.clone())
                        .additional_targets(additional.clone())
                        .build(),
                );
            }
        }

        // CR 508 — "Whenever you attack" fires once for the attacking player
        // when one or more attackers are declared. Active-player permanents fire
        // for the SelfSource/YourControl scopes (the event is player-wide);
        // `AnyPlayer`-scoped triggers are observers ("whenever [a player]
        // attacks", Argent Dais) and fire from every controller, once per
        // combat, with the ability's controller as the fired-for player.
        if any_attackers {
            let ap = self.active_player_idx;
            #[allow(clippy::type_complexity)]
            // Plain loops, same reason as `groups` and `listeners` above.
            let mut you_attack: Vec<(CardId, usize, Effect, Option<crate::effect::Predicate>)> =
                Vec::new();
            for c in self.battlefield.iter() {
                let ctrl = c.controller;
                for t in &c.definition.triggered_abilities {
                    if t.event.kind == EventKind::YouAttack
                        && (ctrl == ap
                            || t.event.scope == crate::effect::EventScope::AnyPlayer)
                    {
                        you_attack.push((c.id, ctrl, t.effect.clone(), t.event.filter.clone()));
                    }
                }
            }
            for (src, ctrl, effect, filter) in you_attack {
                // CR 603.2 — the "whenever you attack with …" rider is a
                // trigger-time gate read off the finished attack declaration.
                if let Some(predicate) = filter {
                    let ctx = crate::game::effects::EffectContext::for_ability(src, ctrl, None);
                    if !self.evaluate_predicate(&predicate, &ctx) {
                        continue;
                    }
                }
                let auto_target = self.auto_target_for_effect_avoiding(&effect, ctrl, Some(src));
                // Isshin / Fractured Realm: an attack-caused trigger of a
                // permanent you control fires an additional time per doubler.
                let fires = 1
                    + self.attack_trigger_extra_fires(ctrl)
                    + crate::game::actions::ally_trigger_extra_fires(self, ctrl, src);
                for _ in 0..fires {
                    self.stack.push(
                        TriggerPush::new(src, ctrl, effect.clone())
                            .target(auto_target.clone())
                            .build(),
                    );
                }
            }
        }

        self.give_priority_to_active();
        Ok(events)
    }

    // ── Declare blockers ──────────────────────────────────────────────────────

    pub fn declare_blockers(
        &mut self,
        assignments: Vec<(CardId, CardId)>,
    ) -> Result<Vec<GameEvent>, GameError> {
        if self.step != TurnStep::DeclareBlockers {
            return Err(block_reject(line!(), GameError::WrongStep { actual: self.step }));
        }
        // Master Warcraft / Invasion Plans — only the chooser may submit.
        // (Without one the engine keeps trusting the caller; blocker
        // ownership is validated per assignment below.)
        if let Some(chooser) = self.block_chooser()
            && self.priority.player_with_priority != chooser
        {
            return Err(block_reject(line!(), GameError::NotYourPriority));
        }

        // One layer pass for the whole declaration. Every consumer below is a
        // `find(id)` on the declared blockers and their attackers, the
        // creatures already attacking, or the blockers already declared —
        // except the five CR 509.1 requirement loops, which ask the whole
        // board who *has* to block. Four of them are keyed on a keyword and
        // the fifth on `must_block`, so when no permanent can carry one, the
        // pass covers the participants instead of the ~23-permanent board.
        // See `board_keyword_in_scope` for why `false` is authoritative.
        let (computed, block_requirement) = self.with_frozen_layers(|g| {
            let requirement = g.battlefield.iter().any(|c| c.must_block.is_some())
                || g.board_keyword_in_scope(&[
                    Keyword::MustBlock,
                    Keyword::MustAttackOrBlock,
                    Keyword::MustBeBlocked,
                    Keyword::AllMustBlock,
                    Keyword::CantBeBlockedUnlessAllBlock,
                ]);
            if requirement {
                return (g.compute_battlefield(), requirement);
            }
            let mut ids: SmallVec<[CardId; 8]> = SmallVec::new();
            let mut want = |id: CardId| {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            };
            for &(blocker, attacker) in &assignments {
                want(blocker);
                want(attacker);
            }
            g.attacking.iter().for_each(|a| want(a.attacker));
            // `block_map`'s key order is a `HashMap`'s, but it only decides
            // which permanents get computed; every reader looks up by id.
            g.block_map.keys().for_each(|&b| want(b));
            (g.compute_permanents(&ids), requirement)
        });
        // A whole-board consumer that slipped past the gate would silently
        // read `None` / `&[]` for every permanent outside the subset and drop
        // its restriction. Panic on it across the suite instead.
        #[cfg(debug_assertions)]
        let computed_ids: Vec<CardId> = computed.iter().map(|c| c.id).collect();
        #[cfg(debug_assertions)]
        let battlefield_ids: Vec<CardId> = self.battlefield.iter().map(|c| c.id).collect();
        #[cfg(debug_assertions)]
        let in_subset = |id: CardId| computed_ids.contains(&id) || !battlefield_ids.contains(&id);
        let cp_of = |id: CardId| {
            #[cfg(debug_assertions)]
            debug_assert!(in_subset(id), "cp_of({id:?}) read outside the gated subset");
            computed.iter().find(|c| c.id == id)
        };
        let kws_of = |id: CardId| -> &[Keyword] {
            #[cfg(debug_assertions)]
            debug_assert!(in_subset(id), "kws_of({id:?}) read outside the gated subset");
            computed
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.keywords())
                .unwrap_or(&[])
        };

        // Validate ALL assignments before mutating any state. Each blocker's
        // controller must equal the defender of the attacker it's blocking.
        // CR 509.1b — a creature blocks one attacker unless an effect lets it
        // block more (`CanBlockAdditional` / `CanBlockAnyNumber`); count the
        // merged set (already-declared blocks plus this batch) against the
        // allowance, and reject a repeat of the same pair.
        let mut batch_blocks: crate::fxhash::HashMap<CardId, SmallVec<[CardId; 4]>> =
            crate::fxhash::HashMap::default();
        // CR 509.1b — Silent Arbiter: "No more than N creatures can block each
        // combat." Count distinct blockers across already-declared blocks and
        // this batch.
        if let Some(cap) = self.combat_participation_cap(true)
            && let Some(&(first, _)) = assignments.first()
        {
            let mut distinct: crate::fxhash::HashSet<CardId> =
                self.block_map.keys().copied().collect();
            distinct.extend(assignments.iter().map(|(b, _)| *b));
            if distinct.len() > cap as usize {
                return Err(block_reject(line!(), GameError::CannotBlock(first)));
            }
        }
        for &(blocker_id, attacker_id) in &assignments {
            let taken = batch_blocks.entry(blocker_id).or_default();
            if taken.contains(&attacker_id) || self.blocks(blocker_id, attacker_id) {
                return Err(block_reject(line!(), GameError::CannotBlock(blocker_id)));
            }
            taken.push(attacker_id);
            let total = taken.len() + self.attackers_blocked_by(blocker_id).len();
            if total > self.max_blocks_on(blocker_id, kws_of(blocker_id)) {
                return Err(block_reject(line!(), GameError::CannotBlock(blocker_id)));
            }
            let atk = self
                .attack_for(attacker_id)
                .ok_or(GameError::CardNotOnBattlefield(attacker_id))?;
            let defender_idx = self
                .defender_for(atk.target)
                .ok_or(GameError::CardNotOnBattlefield(attacker_id))?;

            let blocker = self
                .battlefield
                .iter()
                .find(|c| c.id == blocker_id)
                .ok_or(GameError::CardNotOnBattlefield(blocker_id))?;

            // CR 509.1a: any creature controlled by the defending player
            // (or, in team formats, a teammate of the defending player)
            // may block. In 1v1 / FFA `same_team(a, b)` collapses to
            // `a == b`, so this preserves the historical behavior.
            if !self.same_team(blocker.controller, defender_idx) {
                return Err(block_reject(line!(), GameError::BlockerWrongDefender { blocker: blocker_id }));
            }

            // CR 509.1a/b — everything about the blocker alone, in the one
            // walker the bot's planner and the requirement loops also read.
            // See `blocker_self_block` for what each of them used to miss.
            if let Some((line, e)) = self.blocker_self_block(blocker, cp_of(blocker_id)) {
                return Err(block_reject(line, e));
            }

            // CR 509.1b — everything about *this* pair, in the same walker
            // the planner and the requirement loops read. Four families used
            // to live only here (`cant_block_pairs`, Burden of Proof,
            // Ironclaw Curse, Monstrous Hound); see `blocker_pair_block`.
            let blocker_cp = cp_of(blocker_id).ok_or(GameError::CannotBlock(blocker_id))?;
            let attacker = self
                .battlefield_find(attacker_id)
                .ok_or(GameError::CardNotOnBattlefield(attacker_id))?;
            if let Some((line, e)) =
                self.blocker_pair_block(blocker, blocker_cp, attacker, cp_of(attacker_id), defender_idx)
            {
                return Err(block_reject(line, e));
            }

        }


        // CR 509.1c — "can't block alone". A creature blocks alone if it's the
        // only creature blocking this combat; count the merged block set
        // (this batch plus any earlier-declared blockers).
        {
            let mut all_blockers: crate::fxhash::HashSet<CardId> =
                self.block_map.keys().copied().collect();
            all_blockers.extend(assignments.iter().map(|(b, _)| *b));
            if all_blockers.len() == 1 {
                for &(blocker_id, _) in &assignments {
                    if kws_of(blocker_id).has_kw(&Keyword::CantAttackOrBlockAlone) {
                        return Err(block_reject(line!(), GameError::CannotBlock(blocker_id)));
                    }
                }
            }
            // CR 509.1b — Okk's blocking half: needs a strictly bigger partner
            // among the whole combat's blockers. Nothing above mutates the
            // board, so this reads the pass taken at the top.
            let computed_pow = &computed;
            let power_of = |id: CardId| {
                computed_pow.iter().find(|c| c.id == id).map(|c| c.power).unwrap_or(0)
            };
            for &(blocker_id, _) in &assignments {
                if kws_of(blocker_id).has_kw(&Keyword::CantBlockUnlessGreaterPowerBlocks) {
                    let mine = power_of(blocker_id);
                    if !all_blockers.iter().any(|&o| o != blocker_id && power_of(o) > mine) {
                        return Err(block_reject(line!(), GameError::CannotBlock(blocker_id)));
                    }
                }
            }
        }

        // CR 509.1d — block tax (Archangel of Tithes). Sum every active
        // `BlockTaxToController` amount (an `only_while_attacking` source counts
        // only while it's attacking this combat) and pay {tax} per declared
        // blocker from that blocker's controller's pool, auto-tapping for any
        // shortfall. Reject the whole declaration if a player can't cover it.
        // The tax is summed per blocker, since a source may narrow itself to
        // some of them (Heat Wave taxes only nonblue blockers) and may charge
        // life rather than mana.
        // The spend is deferred to after every block-legality check so a
        // rejected declaration never costs mana (CR 601.2h-style atomicity).
        let mut block_tax_by_controller: crate::fxhash::HashMap<usize, (u32, u32)> =
            crate::fxhash::HashMap::default();
        for &(blocker_id, _) in &assignments {
            let (mana, life) = self.block_tax_for(blocker_id);
            if mana == 0 && life == 0 {
                continue;
            }
            if let Some(b) = self.battlefield_find(blocker_id) {
                let e = block_tax_by_controller.entry(b.controller).or_insert((0, 0));
                e.0 += mana;
                e.1 += life;
            }
        }

        // Menace: attackers with Menace must be blocked by 2+ creatures or
        // not at all (CR 702.110b). Counts the merged block set (existing
        // blocks plus this batch) so incremental multi-defender submissions
        // compose, same as the CantBeBlockedExceptByN check below.
        for atk in &self.attacking {
            let has_menace = kws_of(atk.attacker).has_kw(&Keyword::Menace);
            if has_menace {
                let blocker_count = assignments
                    .iter()
                    .filter(|(_, aid)| *aid == atk.attacker)
                    .count()
                    + self.blocker_count_of(atk.attacker);
                if blocker_count == 1 {
                    return Err(block_reject(line!(), GameError::MenaceRequiresTwoBlockers(atk.attacker)));
                }
            }
        }

        // CR 509.1b — "can't block unless its controller pays {N}"
        // (Oppressive Rays). Charged once per declared blocker, paid from
        // that blocker's controller's pool with auto-tap for the shortfall.
        {
            let mut owed: crate::fxhash::HashMap<usize, u32> = Default::default();
            for &(blocker_id, _) in &assignments {
                let Some(seat) = self.battlefield_find(blocker_id).map(|c| c.controller) else {
                    continue;
                };
                *owed.entry(seat).or_default() +=
                    self.attack_block_keyword_tax(blocker_id, kws_of(blocker_id), false);
            }
            for (seat, amount) in owed {
                let tax = crate::mana::cost(&[crate::mana::generic(amount)]);
                if self.try_pay_with_auto_tap(seat, &tax).is_err() {
                    return Err(block_reject(line!(), GameError::CannotBlock(assignments[0].0)));
                }
            }
        }

        // CR 509.1b — the blocking half of Hollow Warrior's tap cost. Helpers
        // may be neither an attacker nor one of the declared blockers.
        // The presence gate is the attack side's (`AttackBlockCostTapAnother`
        // is one keyword on a handful of cards, and `tap_another_filters` is
        // already gated on it) — without it the `declared` list below is built
        // on every declaration for a keyword no board plays.
        if assignments.iter().any(|&(b, _)| {
            kws_of(b).iter().any(|k| matches!(k, Keyword::AttackBlockCostTapAnother(_)))
        }) {
            let declared: Vec<CardId> = assignments
                .iter()
                .map(|(b, _)| *b)
                .chain(self.attacking.iter().map(|a| a.attacker))
                .collect();
            let mut tapped: SmallVec<[CardId; 4]> = SmallVec::new();
            for &(blocker_id, _) in &assignments {
                let Some(seat) = self.battlefield_find(blocker_id).map(|c| c.controller) else {
                    continue;
                };
                for f in tap_another_filters(kws_of(blocker_id)) {
                    match self.find_tap_helper(seat, &f, blocker_id, &declared, &tapped) {
                        Some(id) => tapped.push(id),
                        None => return Err(block_reject(line!(), GameError::CannotBlock(blocker_id))),
                    }
                }
            }
            for id in tapped {
                if let Some(c) = self.battlefield_find_mut(id) {
                    c.tapped = true;
                }
            }
        }

        // "Can't be blocked except by N or more creatures" (Pathrazer of
        // Ulamog). Generalized Menace: 0 or >= N blockers, never 1..N-1.
        for atk in &self.attacking {
            for kw in kws_of(atk.attacker) {
                if let Keyword::CantBeBlockedExceptByN(n) = kw {
                    let blocker_count = assignments
                        .iter()
                        .filter(|(_, aid)| *aid == atk.attacker)
                        .count()
                        + self.blocker_count_of(atk.attacker);
                    if blocker_count > 0 && (blocker_count as u32) < *n {
                        return Err(block_reject(line!(), GameError::MenaceRequiresTwoBlockers(atk.attacker)));
                    }
                }
            }
        }

        // CR 509.1g — "can't be blocked by more than one creature" (Charging
        // Rhino). At most one blocker may be assigned (the inverse of Menace).
        for atk in &self.attacking {
            if kws_of(atk.attacker).has_kw(&Keyword::CantBeBlockedByMoreThanOne) {
                let blocker_count = assignments
                    .iter()
                    .filter(|(_, aid)| *aid == atk.attacker)
                    .count()
                    + self.blocker_count_of(atk.attacker);
                if blocker_count > 1 {
                    return Err(block_reject(line!(), GameError::CannotBeBlockedByMoreThanOne(atk.attacker)));
                }
            }
        }

        // CR 509.1c — "must be blocked if able" (Lure / Academic Dispute).
        // If such an attacker is left unblocked while the defender controls
        // an idle creature that could legally block it, reject the
        // declaration. Considers the merged block set (already-declared
        // blocks plus this batch) so independent multiplayer submissions
        // compose. Single-requirement model; full CR maximization across
        // multiple simultaneous requirements is approximated.
        for atk in &self.attacking {
            if !kws_of(atk.attacker).has_kw(&Keyword::MustBeBlocked)
                || !self.block_requirement_binds(atk.attacker)
            {
                continue;
            }
            let already = self.blocker_count_of(atk.attacker) > 0;
            let in_batch = assignments.iter().any(|(_, aid)| *aid == atk.attacker);
            if already || in_batch {
                continue;
            }
            let Some(defender_idx) = self.defender_for(atk.target) else { continue };
            if self.battlefield_find(atk.attacker).is_none() {
                continue;
            }
            let idle_able_blocker = self.battlefield.iter().any(|b| {
                self.same_team(b.controller, defender_idx)
                    && !self.is_blocking(b.id)
                    && !assignments.iter().any(|(bid, _)| *bid == b.id)
                    && self.block_requirement_able(b, atk.attacker)
                    && !self.block_spoken_for_elsewhere(b, atk.attacker, &assignments)
            });
            if idle_able_blocker {
                return Err(block_reject(line!(), GameError::MustBeBlockedIfAble(atk.attacker)));
            }
        }

        // CR 509.1c — true Lure ("all creatures able to block this do so").
        // Every idle defender creature that *can* legally block such an
        // attacker must be assigned to it in the merged block set.
        for atk in &self.attacking {
            if !kws_of(atk.attacker).has_kw(&Keyword::AllMustBlock)
                || !self.block_requirement_binds(atk.attacker)
            {
                continue;
            }
            let Some(defender_idx) = self.defender_for(atk.target) else { continue };
            if self.battlefield_find(atk.attacker).is_none() {
                continue;
            }
            let unmet = self.battlefield.iter().any(|b| {
                self.same_team(b.controller, defender_idx)
                    && self.block_requirement_able(b, atk.attacker)
                    // Able to block it but not assigned to it (here or earlier).
                    && !self.blocks(b.id, atk.attacker)
                    && !assignments.iter().any(|(bid, aid)| *bid == b.id && *aid == atk.attacker)
                    && !self.block_spoken_for_elsewhere(b, atk.attacker, &assignments)
            });
            if unmet {
                return Err(block_reject(line!(), GameError::MustBeBlockedIfAble(atk.attacker)));
            }
        }

        // CR 702.39 — Provoke: a creature provoked this combat (`must_block`
        // set to an attacker still in this combat) must be assigned to block
        // that attacker if it's able to. It was untapped by the provoke
        // resolution, so "able" reduces to the normal can-block checks.
        for b in &self.battlefield {
            let Some(required) = b.must_block else { continue };
            // The provoker must still be attacking for the requirement to
            // bind, and a CR 509.1b count it cannot reach un-binds it too.
            if !self.attacking.iter().any(|a| a.attacker == required)
                || !self.block_requirement_binds(required)
            {
                continue;
            }
            if !self.block_requirement_able(b, required) {
                continue;
            }
            let assigned = self.blocks(b.id, required)
                || assignments.iter().any(|(bid, aid)| *bid == b.id && *aid == required);
            if !assigned && !self.block_spoken_for_elsewhere(b, required, &assignments) {
                return Err(block_reject(line!(), GameError::MustBeBlockedIfAble(required)));
            }
        }

        // CR 509.1b — Tromokratis: once such an attacker is blocked at all,
        // *every* untapped defending creature able to block it must also be
        // assigned to it. Checked against the merged block set.
        for atk in &self.attacking {
            if !kws_of(atk.attacker).has_kw(&Keyword::CantBeBlockedUnlessAllBlock) {
                continue;
            }
            let blocked = self.blocker_count_of(atk.attacker) > 0
                || assignments.iter().any(|(_, aid)| *aid == atk.attacker);
            if !blocked {
                continue;
            }
            let Some(defender_idx) = self.defender_for(atk.target) else { continue };
            if self.battlefield_find(atk.attacker).is_none() {
                continue;
            }
            let unmet = self.battlefield.iter().any(|b| {
                self.same_team(b.controller, defender_idx)
                    && !self.blocks(b.id, atk.attacker)
                    && !assignments
                        .iter()
                        .any(|(bid, aid)| *bid == b.id && *aid == atk.attacker)
                    && self.block_requirement_able(b, atk.attacker)
            });
            if unmet {
                return Err(block_reject(line!(), GameError::CannotBlock(atk.attacker)));
            }
        }

        // CR 509.1c — "blocks each combat if able" (`MustBlock`). A creature
        // carrying the keyword that can legally block at least one declared
        // attacker must be assigned to block one of them. Unlike the four
        // loops above, this one asks the computed view *before* the keyword
        // that decides it, so it takes the gate directly.
        let must_block_scan: &[crate::card::CardInstance] =
            if block_requirement { &self.battlefield } else { &[] };
        for b in must_block_scan {
            if !(kws_of(b.id).has_kw(&Keyword::MustBlock)
                || kws_of(b.id).has_kw(&Keyword::MustAttackOrBlock))
            {
                continue;
            }
            let already = self.is_blocking(b.id)
                || assignments.iter().any(|(bid, _)| *bid == b.id);
            if already { continue; }
            // Could it have blocked any declared attacker?
            let could_block = self.attacking.iter().any(|atk| {
                self.defender_for(atk.target)
                    .is_some_and(|d| self.same_team(b.controller, d))
                    && self.block_requirement_able(b, atk.attacker)
                    && self.block_requirement_binds(atk.attacker)
            });
            if could_block {
                return Err(block_reject(line!(), GameError::MustBeBlockedIfAble(b.id)));
            }
        }

        // All validation passed — pay the block tax (CR 509.1d) from each
        // blocking player's pool, auto-tapping for any shortfall. Restore
        // every payment if any player can't cover theirs.
        if !block_tax_by_controller.is_empty() {
            let mut snapshots = Vec::new();
            let mut payers: Vec<(usize, (u32, u32))> =
                block_tax_by_controller.into_iter().collect();
            payers.sort_by_key(|(p, _)| *p);
            let mut life_paid: SmallVec<[(usize, u32); 2]> = SmallVec::new();
            for (player, (mana, life)) in payers {
                let snap = self.snapshot_payment_state(player);
                // CR 119.4 — a life cost is unpayable below the amount owed.
                let ok = self.players[player].life >= life as i32
                    && (mana == 0 || {
                        let cost = crate::mana::cost(&[crate::mana::generic(mana)]);
                        self.try_pay_with_auto_tap(player, &cost).is_ok()
                    });
                if !ok {
                    for (p, s) in snapshots {
                        self.restore_payment_state(p, s);
                    }
                    for (p, n) in life_paid {
                        self.players[p].life += n as i32;
                    }
                    return Err(block_reject(line!(), GameError::CannotBlock(assignments[0].0)));
                }
                if life > 0 {
                    self.players[player].life -= life as i32;
                    life_paid.push((player, life));
                }
                snapshots.push((player, snap));
            }
        }

        // Combat-keyword P/T adjustments applied on block declaration:
        // Flanking (CR 702.25), Bushido (CR 702.45), Rampage (CR 702.23).
        // Snapshot the +/-N deltas (same value on power and toughness)
        // before mutating so the borrow of `assignments` stays clean.
        // Only the assignments' own blockers and attackers are read below, so
        // compute those instead of the whole board: the freeze scope pays one
        // gather and one layer pass per participant, against ~23 passes for a
        // bench board.
        let mut ids: SmallVec<[CardId; 8]> = SmallVec::new();
        for &(b, a) in &assignments {
            if !ids.contains(&b) {
                ids.push(b);
            }
            if !ids.contains(&a) {
                ids.push(a);
            }
        }
        let computed: SmallVec<[(CardId, Vec<Keyword>); 8]> = self.with_frozen_layers(|g| {
            ids.iter()
                .map(|&id| {
                    let kws =
                        g.computed_permanent(id).map(|c| c.keywords().to_vec()).unwrap_or_default();
                    (id, kws)
                })
                .collect()
        });
        let kws_for = |id: CardId| -> &[Keyword] {
            computed.iter().find(|(i, _)| *i == id).map_or(&[][..], |(_, k)| k)
        };
        let sum_n = |kws: &[Keyword], pick: fn(&Keyword) -> Option<i32>| -> i32 {
            kws.iter().filter_map(pick).sum()
        };
        let mut pt_deltas: SmallVec<[(CardId, i32); 8]> = SmallVec::new();
        let mut blocked: crate::fxhash::HashMap<CardId, usize> = crate::fxhash::HashMap::default();
        for &(b, a) in &assignments {
            *blocked.entry(a).or_insert(0) += 1;
            let bk = kws_for(b);
            let ak = kws_for(a);
            // Flanking: nonflanking blocker shrinks once per flanking instance.
            let flank = ak.iter().filter(|k| **k == Keyword::Flanking).count() as i32;
            if flank > 0 && !bk.has_kw(&Keyword::Flanking) {
                pt_deltas.push((b, -flank));
            }
            // Bushido on the blocker (it blocks).
            let bn = sum_n(bk, |k| if let Keyword::Bushido(x) = k { Some(*x as i32) } else { None });
            if bn > 0 { pt_deltas.push((b, bn)); }
        }
        for (a, count) in blocked {
            let ak = kws_for(a);
            // Bushido on the attacker (it becomes blocked — once).
            let bn = sum_n(ak, |k| if let Keyword::Bushido(x) = k { Some(*x as i32) } else { None });
            if bn > 0 { pt_deltas.push((a, bn)); }
            // Rampage: +N for each blocker beyond the first.
            let rn = sum_n(ak, |k| if let Keyword::Rampage(x) = k { Some(*x as i32) } else { None });
            let extra = count.saturating_sub(1) as i32;
            if rn > 0 && extra > 0 { pt_deltas.push((a, rn * extra)); }
        }

        // All valid — apply (merge into existing block_map so multiple
        // defenders can submit independently in multiplayer).
        self.blockers_declared = true;
        // CR 702.22h — blocking one member of a band blocks every other member
        // by that same blocker. Expanded before the apply loop so the added
        // pairs go through the same bookkeeping and events.
        let assignments: Vec<(CardId, CardId)> = {
            let mut out = assignments;
            for i in 0..out.len() {
                let (blocker, attacker) = out[i];
                let banded: Vec<CardId> = self
                    .attack_bands
                    .iter()
                    .find(|b| b.contains(&attacker))
                    .map(|b| b.iter().copied().filter(|m| *m != attacker).collect())
                    .unwrap_or_default();
                for m in banded {
                    if !out.contains(&(blocker, m)) && !self.blocks(blocker, m) {
                        out.push((blocker, m));
                    }
                }
            }
            out
        };
        let mut events = vec![];
        for (blocker_id, attacker_id) in assignments {
            self.add_block(blocker_id, attacker_id);
            if let Some(b) = self.battlefield_find_mut(blocker_id) {
                b.blocked_this_turn = true;
                if !b.blocked_attackers_this_turn.contains(&attacker_id) {
                    b.blocked_attackers_this_turn.push(attacker_id);
                }
            }
            if !self.blocks_declared_this_turn.contains(&(blocker_id, attacker_id)) {
                self.blocks_declared_this_turn.push((blocker_id, attacker_id));
            }
            // CR 510.1c — once blocked, the attacker stays blocked for this
            // combat even if every blocker later leaves combat.
            if !self.blocked_attackers.contains(&attacker_id) {
                self.blocked_attackers.push(attacker_id);
            }
            events.push(GameEvent::BlockerDeclared {
                blocker: blocker_id,
                attacker: attacker_id,
            });
            // Noxious Assault's turn rider: each declared block poisons the
            // blocker's controller.
            if self.block_poison_this_turn > 0 {
                let ctrl = self
                    .battlefield_find(blocker_id)
                    .map(|c| c.controller);
                if let Some(ctrl) = ctrl {
                    let n = self.block_poison_this_turn;
                    self.add_poison(ctrl, n, &mut events);
                }
            }
        }
        for (id, d) in pt_deltas {
            if let Some(c) = self.battlefield_find_mut(id) {
                c.power_bonus += d;
                c.toughness_bonus += d;
            }
        }
        // CR 509.3g — emit `AttackerWentUnblocked` for each attacker
        // with no blockers assigned. Trigger source is the unblocked
        // attacker; consumers can read it via `Selector::TriggerSource`.
        let mut frenzy_deltas: SmallVec<[(CardId, i32); 8]> = SmallVec::new();
        // One gather for the whole sweep. `computed_permanent` is `&self` and
        // this loop runs at depth 0, so unfrozen it rebuilds the full effect
        // set once per unblocked attacker to read one keyword.
        self.with_frozen_layers(|g| {
            for atk in &g.attacking {
                let blocked = g.blocker_count_of(atk.attacker) > 0;
                if !blocked {
                    events.push(GameEvent::AttackerWentUnblocked { attacker: atk.attacker });
                    // CR 702.35 — Frenzy N: an unblocked attacker gets +N/+0.
                    // Read computed keywords so statically-granted Frenzy
                    // (Frenzy Sliver) counts too.
                    if let Some(cp) = g.computed_permanent(atk.attacker) {
                        let fn_: i32 = cp
                            .keywords()
                            .iter()
                            .filter_map(
                                |k| if let Keyword::Frenzy(x) = k { Some(*x as i32) } else { None },
                            )
                            .sum();
                        if fn_ > 0 {
                            frenzy_deltas.push((atk.attacker, fn_));
                        }
                    }
                }
            }
        });
        for (id, d) in frenzy_deltas {
            if let Some(c) = self.battlefield_find_mut(id) {
                c.power_bonus += d;
            }
        }
        self.give_priority_to_active();
        Ok(events)
    }

    // ── Combat resolution ─────────────────────────────────────────────────────

    pub(crate) fn has_first_strikers(&self) -> bool {
        // Asked once per combat-damage step about the 2-6 combat
        // participants; computing the whole board for it was the cost.
        if self.attacking.is_empty() && self.block_map.is_empty() {
            return false;
        }
        if !self.first_strike_possible() {
            return false;
        }
        self.with_frozen_layers(|g| {
            let strikes_first = |id: CardId| {
                g.computed_permanent(id).is_some_and(|c| {
                    c.keywords().has_kw(&Keyword::FirstStrike)
                        || c.keywords().has_kw(&Keyword::DoubleStrike)
                })
            };
            g.attacking.iter().any(|atk| strikes_first(atk.attacker))
                || g.block_map.keys().any(|&id| strikes_first(id))
        })
    }

    /// Can any combat participant's *computed* keywords carry first or double
    /// strike, answered without gathering? `false` is authoritative; `true`
    /// only means [`has_first_strikers`](Self::has_first_strikers) has to
    /// gather.
    ///
    /// [`card_keyword_possible`] per participant with its one expensive leg —
    /// the board-wide grant scan — hoisted out of the loop, because the two
    /// or six participants are asked about the same board. The whole step
    /// transition rides on this: a board printing neither keyword and holding
    /// no source that can grant one skips the first-strike damage step, and
    /// under the old shape that skip cost a full gather plus one layer pass
    /// per participant.
    ///
    /// [`card_keyword_possible`]: crate::game::GameState::card_keyword_possible
    fn first_strike_possible(&self) -> bool {
        let strikes =
            |k: &Keyword| matches!(k, Keyword::FirstStrike | Keyword::DoubleStrike);
        let printed = |id: CardId| {
            self.battlefield_find(id).is_some_and(|c| {
                c.definition.keywords.iter().any(strikes)
                    || c.granted_keywords_eot.iter().any(strikes)
                    || c.keyword_counters.iter().any(|(k, n)| *n > 0 && strikes(k))
            })
        };
        let hit = self.attacking.iter().any(|atk| printed(atk.attacker))
            || self.block_map.keys().any(|&id| printed(id))
            || self.keyword_grant_in_scope(strikes);
        // The gate has no shared choke point to hang an enumeration audit on,
        // so it audits against its own outcome: when it says no, the guarded
        // body must agree. Runs on every combat-damage step the suite plays.
        #[cfg(debug_assertions)]
        if !hit {
            let computed_hit = self.with_frozen_layers(|g| {
                let strikes_first = |id: CardId| {
                    g.computed_permanent(id)
                        .is_some_and(|c| c.keywords().iter().any(strikes))
                };
                g.attacking.iter().any(|atk| strikes_first(atk.attacker))
                    || g.block_map.keys().any(|&id| strikes_first(id))
            });
            debug_assert!(
                !computed_hit,
                "the first-strike presence gate said no, but a participant strikes first"
            );
        }
        hit
    }

    /// The computed views [`resolve_combat_damage_with_filter`] reads: every
    /// attacker and every declared blocker, from one gather.
    ///
    /// Everything under that resolver looks its permanents up by id — the
    /// attacker infos, the banding assigner, the lethal table, the blocker
    /// scans. The one whole-board consumer is [`free_division_targets`]'
    /// second half (Butcher Orgg divides among *any* of the defending
    /// player's creatures), and it is gated on a keyword the attacker itself
    /// carries, so the whole board is computed only when such an attacker is
    /// actually in combat. **The gate is checked, not assumed**:
    /// `butcher_orgg_divides_damage_among_defenders` assigns damage to a
    /// creature that never blocked, so it fails if this falls back to the
    /// subset.
    ///
    /// [`free_division_targets`]: Self::free_division_targets
    /// [`resolve_combat_damage_with_filter`]: Self::resolve_combat_damage_with_filter
    fn combat_damage_computed(&self) -> Vec<ComputedPermanent> {
        let mut ids: SmallVec<[CardId; 8]> = SmallVec::new();
        for atk in &self.attacking {
            if !ids.contains(&atk.attacker) {
                ids.push(atk.attacker);
            }
        }
        // `block_map`'s key order is a `HashMap`'s, but it only decides which
        // permanents get computed; every reader looks up by id, and the one
        // that collects (`free_division_targets`) sorts.
        for &bid in self.block_map.keys() {
            if !ids.contains(&bid) {
                ids.push(bid);
            }
        }
        let subset = self.compute_permanents(&ids);
        if subset.iter().any(|c| c.keywords().has_kw(&Keyword::DividesCombatDamageAmongDefenders)) {
            return self.compute_battlefield();
        }
        subset
    }

    pub fn resolve_first_strike_damage(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let computed = self.combat_damage_computed();
        // CR 510.4: in the first-strike combat damage step, only creatures
        // with first strike or double strike deal combat damage. The same
        // gate applies to attackers (who deals?) and blockers (who strikes
        // back at the attacker?).
        let fs_or_ds = |kws: &[Keyword]| {
            kws.has_kw(&Keyword::FirstStrike) || kws.has_kw(&Keyword::DoubleStrike)
        };
        let mut events = self.resolve_combat_damage_with_filter(&computed, fs_or_ds, fs_or_ds)?;
        // Suspended on a `wants_ui` player's combat-damage choice — no damage
        // has been dealt yet; `submit_decision` re-enters this step.
        if self.pending_decision.is_some() {
            return Ok(events);
        }
        self.check_state_based_actions_into(&mut events);
        events.push(GameEvent::FirstStrikeDamageResolved);
        Ok(events)
    }

    pub fn resolve_combat(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let computed = self.combat_damage_computed();
        // CR 510.5: in the regular combat damage step, every attacking and
        // blocking creature that didn't deal damage in the first-strike step
        // deals damage now — i.e. anyone without first strike, plus double
        // strikers (who strike in both steps).
        let regular_or_ds = |kws: &[Keyword]| {
            !kws.has_kw(&Keyword::FirstStrike) || kws.has_kw(&Keyword::DoubleStrike)
        };
        let mut events =
            self.resolve_combat_damage_with_filter(&computed, regular_or_ds, regular_or_ds)?;

        // Suspended on a `wants_ui` player's combat-damage choice — no damage
        // dealt yet; combat is not torn down. `submit_decision` re-enters.
        if self.pending_decision.is_some() {
            return Ok(events);
        }

        self.check_state_based_actions_into(&mut events);

        self.attacking.clear();
        // Dropped, not cleared — a cleared `HashMap` keeps its table and
        // every later `GameState::clone` re-allocates it (see `resolve_effect`'s
        // per-resolution reset).
        self.block_map = Default::default();
        // Both are `ColdState` fields; an unguarded `clear` on a combat
        // boundary deep-copies the whole cold group (PERF, twenty-eighth
        // pass's rule, restated in the thirty-third's Log block).
        if !self.blocked_attackers.is_empty() {
            self.blocked_attackers.clear();
        }
        if !self.attack_bands.is_empty() {
            clear_cold!(self.attack_bands);
        }
        self.clear_combat_damage_plan();
        self.blockers_declared = false;
        // CR 702.39 — provoke's "block this combat" requirement ends here.
        // Gated: the write is a `DerefMut` on a CoW `CardData`, so clearing
        // the `None` almost every permanent already holds deep-copied the
        // whole battlefield once per combat.
        for c in &mut self.battlefield {
            if c.must_block.is_some() {
                c.must_block = None;
            }
        }

        events.push(GameEvent::CombatResolved);
        Ok(events)
    }

    /// CR 702.15 — does `defender` control a land with the given land type?
    /// Reads printed land subtypes (Forest/Island/…), so dual lands and
    /// nonbasics with the type count.
    /// CR 802 / 803 / 801.3 — the seats `seat` may legally declare attacks
    /// against right now. Empty when it isn't `seat`'s combat, and narrowed to
    /// one entry (or none) by the attack-left / attack-right option; under a
    /// limited range of influence, only opponents inside `seat`'s range.
    pub fn attackable_players_for(&self, seat: usize) -> Vec<usize> {
        if self.active_player_idx != seat {
            return Vec::new();
        }
        let restriction = self.attack_left_right_defender();
        (0..self.players.len())
            .filter(|p| {
                self.players[*p].is_alive()
                    && !self.same_team(seat, *p)
                    && restriction.is_none_or(|only| only == Some(*p))
                    && self.player_in_range_of(seat, *p)
                    && self.seat_attackable_from(seat, *p)
                    && !self.player_cant_be_attacked_at_all(seat, *p)
            })
            .collect()
    }

    /// CR 508.1 — `defender` can't be attacked by `seat` no matter which
    /// creature is declared: an unfiltered "creatures can't attack you"
    /// (Blazing Archon), Arboria's did-nothing-last-turn lock, or a
    /// turn-scoped Web of Inertia ban. Attacker-*filtered* prohibitions are
    /// left to `declare_attackers`, which sees the actual batch.
    pub(crate) fn player_cant_be_attacked_at_all(&self, seat: usize, defender: usize) -> bool {
        use crate::effect::StaticEffect;
        if self.cant_attack_player_this_turn.contains(&(seat, defender)) {
            return true;
        }
        self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| {
                match self.active_static(&sa.effect, c) {
                    Some(StaticEffect::CreaturesCantAttackController { filter: None, .. }) => {
                        c.controller == defender
                    }
                    Some(StaticEffect::PlayersCantBeAttackedUnlessTheyActedLastTurn) => {
                        !self.acted_on_their_last_turn(defender)
                    }
                    _ => false,
                }
            })
        })
    }

    /// CR 809.3c — under the Emperor variant's attack option, `defender` must
    /// be seated immediately next to `seat` (measured over living seats).
    /// Always true when the option is off.
    pub(crate) fn seat_attackable_from(&self, seat: usize, defender: usize) -> bool {
        if !self.attack_adjacent_only {
            return true;
        }
        let living: Vec<usize> =
            (0..self.players.len()).filter(|p| self.players[*p].is_alive()).collect();
        let (Some(here), Some(there)) = (
            living.iter().position(|p| *p == seat),
            living.iter().position(|p| *p == defender),
        ) else {
            return false;
        };
        let n = living.len() as isize;
        let d = (here as isize - there as isize).rem_euclid(n);
        d == 1 || d == n - 1
    }

    /// CR 803.1a/b — the single seat the active player may attack under the
    /// attack-left / attack-right option. `None` means no restriction (CR 802);
    /// `Some(None)` means "no legal defender" (the nearest living opponent is
    /// more than one seat away, so this player can't attack at all).
    fn attack_left_right_defender(&self) -> Option<Option<usize>> {
        let step: isize = match self.attack_option {
            crate::game::AttackOption::MultiplePlayers => return None,
            crate::game::AttackOption::AttackLeft => 1,
            crate::game::AttackOption::AttackRight => -1,
        };
        let n = self.players.len() as isize;
        if n < 2 {
            return Some(None);
        }
        let me = self.active_player_idx as isize;
        let neighbor = ((me + step).rem_euclid(n)) as usize;
        // "More than one seat away" — the adjacent seat has to be a living
        // opponent, otherwise this player has no legal defender.
        (self.players[neighbor].is_alive() && !self.same_team(self.active_player_idx, neighbor))
            .then_some(Some(neighbor))
            .or(Some(None))
    }

    pub(crate) fn defender_controls_land_type(
        &self,
        defender: usize,
        lt: &crate::card::LandType,
    ) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller == defender && c.definition.has_land_type(*lt)
        })
    }

    /// CR 509.1b — is this landwalk flavor blanked for everyone by a
    /// `LandwalkIgnored` static in play (Great Wall, Deadfall, Quagmire,
    /// Crevasse, Gosta Dirk, Lord Magnus)?
    pub(crate) fn landwalk_ignored(&self, lt: crate::card::LandType) -> bool {
        self.battlefield
            .iter()
            .flat_map(|c| &c.definition.static_abilities)
            .any(|sa| matches!(sa.effect, crate::effect::StaticEffect::LandwalkIgnored(t) if t == lt))
    }

    /// CR 510.1c — build the `Decision::CombatDamageOrder` asking the
    /// attacking player to order `default_order` (deterministic CardId order)
    /// for combat-damage assignment.
    fn combat_damage_order_decision(
        &self,
        attacker: CardId,
        default_order: &[CardId],
    ) -> crate::decision::Decision {
        let blockers: Vec<(CardId, String)> = default_order
            .iter()
            .map(|id| {
                let name = self
                    .battlefield_find(*id)
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                (*id, name)
            })
            .collect();
        crate::decision::Decision::CombatDamageOrder { attacker, blockers }
    }

    /// Validate a `DamageOrder` answer into a concrete blocker order. Ids the
    /// answer omits are appended in their default position, and unknown ids
    /// are ignored — so a partial or empty answer is always legal and keeps
    /// `default_order`.
    fn resolve_damage_order(
        &self,
        default_order: &[CardId],
        answer: &crate::decision::DecisionAnswer,
    ) -> Vec<CardId> {
        use crate::decision::DecisionAnswer;
        let DecisionAnswer::DamageOrder(chosen) = answer else {
            return default_order.to_vec();
        };
        let mut ordered: Vec<CardId> = Vec::with_capacity(default_order.len());
        for id in chosen {
            if default_order.contains(id) && !ordered.contains(id) {
                ordered.push(*id);
            }
        }
        for id in default_order {
            if !ordered.contains(id) {
                ordered.push(*id);
            }
        }
        ordered
    }

    /// CR 510.1c-d — divide an attacker's `total_power` combat damage among
    /// its blockers (given in assignment order with their `lethal` amounts).
    /// Returns `(blocker_id, amount)` pairs in the same order; any power not
    /// assigned to a blocker is the trample-over leftover.
    ///
    /// The default (and `AutoDecider`) split assigns lethal to each blocker
    /// in order until the power runs out. A `wants_ui` / scripted decider may
    /// answer `CombatDamageAssignment` to over-assign (e.g. deny trample),
    /// subject to CR 510.1c: a blocker may receive damage only after every
    /// earlier blocker has been assigned at least its lethal, and the total
    /// can't exceed `total_power`. A malformed answer falls back to default.
    fn default_damage_split(
        &self,
        total_power: u32,
        lethals: &[(CardId, u32)],
        has_trample: bool,
    ) -> Vec<(CardId, u32)> {
        let mut remaining = total_power;
        let mut split: Vec<(CardId, u32)> = lethals
            .iter()
            .map(|&(id, lethal)| {
                let a = lethal.min(remaining);
                remaining -= a;
                (id, a)
            })
            .collect();
        // CR 510.1d — ALL the attacker's damage must be assigned; without
        // trample the excess goes to the blockers (default: the last one)
        // rather than vanishing. With trample the remainder is left for the
        // caller's trample-over-to-player path (CR 702.19g).
        if !has_trample && remaining > 0 && let Some(last) = split.last_mut() {
            last.1 += remaining;
        }
        split
    }

    /// Build the `Decision::AssignCombatDamage` for dividing `total_power`
    /// among `lethals` (in assignment order).
    fn assign_combat_damage_decision(
        &self,
        attacker: CardId,
        total_power: u32,
        lethals: &[(CardId, u32)],
    ) -> crate::decision::Decision {
        let blockers: Vec<(CardId, String, u32)> = lethals
            .iter()
            .map(|&(id, lethal)| {
                let name = self
                    .battlefield_find(id)
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                (id, name, lethal)
            })
            .collect();
        crate::decision::Decision::AssignCombatDamage {
            attacker,
            attacker_power: total_power,
            blockers,
        }
    }

    /// Validate a `CombatDamageAssignment` answer into a concrete split. An
    /// empty or rule-violating answer falls back to `default_damage_split`.
    fn resolve_damage_assignment(
        &self,
        total_power: u32,
        lethals: &[(CardId, u32)],
        has_trample: bool,
        answer: &crate::decision::DecisionAnswer,
    ) -> Vec<(CardId, u32)> {
        use crate::decision::DecisionAnswer;
        let DecisionAnswer::CombatDamageAssignment(pairs) = answer else {
            return self.default_damage_split(total_power, lethals, has_trample);
        };
        if pairs.is_empty() {
            return self.default_damage_split(total_power, lethals, has_trample);
        }
        // Re-key the answer into blocker order (missing entries = 0).
        let amounts: Vec<u32> = lethals
            .iter()
            .map(|&(id, _)| {
                pairs
                    .iter()
                    .find(|(pid, _)| *pid == id)
                    .map(|(_, a)| *a)
                    .unwrap_or(0)
            })
            .collect();
        let assigned: u32 = amounts.iter().sum();
        if assigned > total_power {
            return self.default_damage_split(total_power, lethals, has_trample);
        }
        // Ordering rule: once a blocker is under-assigned, no later blocker
        // (nor trample-over) may receive damage.
        let mut earlier_all_lethal = true;
        for (i, &(_, lethal)) in lethals.iter().enumerate() {
            if !earlier_all_lethal && amounts[i] > 0 {
                return self.default_damage_split(total_power, lethals, has_trample);
            }
            if amounts[i] < lethal {
                earlier_all_lethal = false;
            }
        }
        if total_power > assigned && !earlier_all_lethal {
            // A trample-over leftover requires every blocker to be at lethal.
            return self.default_damage_split(total_power, lethals, has_trample);
        }
        // CR 510.1d — all of the attacker's power must be assigned; without
        // trample there is no player to soak the leftover, so an
        // under-assignment (even with every blocker at lethal) is illegal.
        if !has_trample && assigned < total_power {
            return self.default_damage_split(total_power, lethals, has_trample);
        }
        lethals.iter().map(|&(id, _)| id).zip(amounts).collect()
    }

    /// CR 510.1c-d — gather (and cache) the active player's combat-damage
    /// ordering and assignment choices for every multi-blocker attacker,
    /// before any damage is applied. Returns `true` if it suspended on a
    /// `wants_ui` player's pending decision (the caller must return early and
    /// re-enter the damage step after the answer); `false` once every choice
    /// is settled. Pure w.r.t. the battlefield — only the decision caches and
    /// `pending_decision` are written.
    fn gather_combat_damage_decisions(
        &mut self,
        attacker_infos: &[AttackerInfo],
        computed: &[ComputedPermanent],
        blocker_filter: &impl Fn(&[Keyword]) -> bool,
    ) -> bool {
        use crate::game::types::{CombatDecisionKind, PendingDecision, ResumeContext};
        // Reset the caches once when entering a new damage step (first-strike
        // vs regular), but never on a mid-step decision resume.
        if self.combat_damage_plan_step != Some(self.step) {
            self.combat_damage_order.clear();
            self.combat_damage_assignment.clear();
            self.combat_damage_plan_step = Some(self.step);
        }
        let active = self.active_player_idx;
        for atk in attacker_infos.iter().filter(|a| a.should_deal) {
            let blocker_ids = self.blockers_of(atk.id);
            // A free divider (Butcher Orgg) always gets an assignment choice —
            // it divides over the defending player's creatures, not its
            // blockers, so the multi-blocker gate doesn't apply.
            let free_divider = !self.free_division_targets(atk.id, computed).is_empty();
            if blocker_ids.len() <= 1 && !free_divider {
                continue;
            }

            // CR 509.2 / 510.1c — Banding: if any blocking creature has
            // banding, the *defending* player (the blockers' controller), not
            // the attacking player, announces this attacker's damage order and
            // assignment. Otherwise the active (attacking) player decides.
            let banding_assigner = blocker_ids
                .iter()
                .find_map(|bid| {
                    computed
                        .iter()
                        .find(|c| c.id == *bid && c.keywords().has_kw(&Keyword::Banding))
                        .map(|c| c.controller)
                })
                // CR 702.22j — the "bands with other [quality]" arm.
                .or_else(|| self.quality_band_assigner(&blocker_ids, computed));
            // CR 510.1a — Defensive Formation: the defending player assigns
            // the damage of everything attacking them.
            let defending_seat = self.attacking.iter().find(|a| a.attacker == atk.id).and_then(
                |a| match a.target {
                    crate::game::types::AttackTarget::Player(d) => Some(d),
                    crate::game::types::AttackTarget::Planeswalker(pw)
                    | crate::game::types::AttackTarget::Battle(pw) => {
                        self.battlefield_find(pw).map(|c| c.controller)
                    }
                },
            );
            let defender_assigner = match defending_seat {
                Some(d) if self.battlefield.iter().any(|c| {
                    c.controller == d
                        && c.definition.static_abilities.iter().any(|sa| {
                            matches!(
                                sa.effect,
                                crate::effect::StaticEffect::ControllerAssignsAttackersCombatDamage
                            )
                        })
                }) => Some(d),
                _ => None,
            };
            let assigner = banding_assigner.or(defender_assigner).unwrap_or(active);
            let assigner_ui = self.players[assigner].wants_ui;

            // 1) Blocker order (CR 510.1c) — a free divider has no order to
            // announce, so it goes straight to the assignment.
            if !self.combat_damage_order.contains_key(&atk.id) && !free_divider {
                let decision = self.combat_damage_order_decision(atk.id, &blocker_ids);
                if assigner_ui {
                    self.pending_decision = Some(PendingDecision {
                        decision,
                        resume: ResumeContext::CombatDamage {
                            player: assigner,
                            attacker: atk.id,
                            kind: CombatDecisionKind::Order,
                        },
                    });
                    return true;
                }
                let answer = self.decider.decide(&decision);
                let order = self.resolve_damage_order(&blocker_ids, &answer);
                self.combat_damage_order.insert(atk.id, order);
            }
            let order = self.combat_damage_order.get(&atk.id).cloned().unwrap_or_default();

            // 2) Damage assignment across the ordered blockers (CR 510.1d).
            if !self.combat_damage_assignment.contains_key(&atk.id) {
                let total_power = if self.combat_damage_prevented_for_dealer(atk.id) {
                    0
                } else {
                    atk.power.max(0) as u32
                };
                let (lethals, trample) = self.combat_assignment_plan(
                    atk.id,
                    atk.has_deathtouch,
                    atk.has_trample,
                    &order,
                    computed,
                );
                // No meaningful choice with zero power — store the default.
                if total_power == 0 {
                    let split = self.default_damage_split(total_power, &lethals, trample);
                    self.combat_damage_assignment.insert(atk.id, split);
                    continue;
                }
                let decision =
                    self.assign_combat_damage_decision(atk.id, total_power, &lethals);
                if assigner_ui {
                    self.pending_decision = Some(PendingDecision {
                        decision,
                        resume: ResumeContext::CombatDamage {
                            player: assigner,
                            attacker: atk.id,
                            kind: CombatDecisionKind::Assign,
                        },
                    });
                    return true;
                }
                let answer = self.decider.decide(&decision);
                let split =
                    self.resolve_damage_assignment(total_power, &lethals, trample, &answer);
                self.combat_damage_assignment.insert(atk.id, split);
            }
        }

        // CR 509.2 / 510.1e — the mirror image for a creature blocking several
        // attackers: its controller (the defending player) orders those
        // attackers and divides the blocker's damage among them. Cached in the
        // same maps, keyed by the blocker's id — a creature is never both an
        // attacker and a blocker in one combat, so the keyspaces don't collide.
        let multi_blockers: Vec<CardId> = {
            let mut v: Vec<CardId> = self
                .block_map
                .iter()
                .filter(|(_, atks)| atks.len() > 1)
                .map(|(&b, _)| b)
                .collect();
            v.sort_by_key(|id| id.0);
            v
        };
        for bid in multi_blockers {
            let Some(bcp) = computed.iter().find(|c| c.id == bid) else { continue };
            if !blocker_filter(bcp.keywords())
                || bcp.keywords().has_kw(&Keyword::DealsNoCombatDamage)
                || self.combat_damage_prevented_creatures.contains(&bid)
                || self.assigns_no_combat_damage_this_turn.contains(&bid)
                || self.combat_damage_prevented_for_dealer(bid)
                || self.combat_damage_prevented_from(bid)
            {
                continue;
            }
            // CR 702.22k — a blocker blocking a creature with banding (or two
            // members of a "bands with other [quality]" band) has its damage
            // divided by the ACTIVE player, not by its own controller.
            let blocked = self.attackers_blocked_by(bid).to_vec();
            let banded = blocked.iter().any(|aid| {
                computed
                    .iter()
                    .any(|c| c.id == *aid && c.keywords().has_kw(&Keyword::Banding))
            }) || self.quality_band_assigner(&blocked, computed).is_some();
            let assigner = if banded { self.active_player_idx } else { bcp.controller };
            let assigner_ui = self.players[assigner].wants_ui;
            let deathtouch = bcp.keywords().has_kw(&Keyword::Deathtouch);
            let total_power = combat_damage_value(bcp).max(0) as u32;

            if !self.combat_damage_order.contains_key(&bid) {
                let default_order = blocked.clone();
                let decision = self.combat_damage_order_decision(bid, &default_order);
                if assigner_ui {
                    self.pending_decision = Some(PendingDecision {
                        decision,
                        resume: ResumeContext::CombatDamage {
                            player: assigner,
                            attacker: bid,
                            kind: CombatDecisionKind::Order,
                        },
                    });
                    return true;
                }
                let answer = self.decider.decide(&decision);
                let order = self.resolve_damage_order(&default_order, &answer);
                self.combat_damage_order.insert(bid, order);
            }
            let order = self.combat_damage_order.get(&bid).cloned().unwrap_or_default();

            if !self.combat_damage_assignment.contains_key(&bid) {
                let lethals = self.combat_lethals(deathtouch, &order, computed);
                // A blocker has no trample outlet: all its damage lands on the
                // attackers it blocks, so the split is never partial.
                if total_power == 0 {
                    let split = self.default_damage_split(0, &lethals, false);
                    self.combat_damage_assignment.insert(bid, split);
                    continue;
                }
                let decision = self.assign_combat_damage_decision(bid, total_power, &lethals);
                if assigner_ui {
                    self.pending_decision = Some(PendingDecision {
                        decision,
                        resume: ResumeContext::CombatDamage {
                            player: assigner,
                            attacker: bid,
                            kind: CombatDecisionKind::Assign,
                        },
                    });
                    return true;
                }
                let answer = self.decider.decide(&decision);
                let split = self.resolve_damage_assignment(total_power, &lethals, false, &answer);
                self.combat_damage_assignment.insert(bid, split);
            }
        }
        false
    }

    /// CR 510.1e — the combat damage `blocker` assigns to `attacker` this
    /// step. A creature blocking one attacker deals its whole power; a
    /// multi-block divides it per the cached split (defaulting to lethal in
    /// declaration order).
    fn blocker_damage_to(&self, blocker: CardId, attacker: CardId, power: u32) -> u32 {
        if self.attackers_blocked_by(blocker).len() <= 1 {
            return power;
        }
        self.combat_damage_assignment
            .get(&blocker)
            .and_then(|split| split.iter().find(|(a, _)| *a == attacker).map(|(_, n)| *n))
            .unwrap_or(0)
    }

    /// Lethal damage required for each blocker in `order` (its toughness, or 1
    /// under deathtouch per CR 702.2e). Blockers no longer on the battlefield
    /// resolve to 0.
    /// CR 702.22 — the attacking bands declared this combat, for the server
    /// view. Empty when nothing banded.
    pub fn attack_bands_view(&self) -> Vec<Vec<CardId>> {
        self.attack_bands.clone()
    }

    /// Butcher Orgg — "you may assign this creature's combat damage divided as
    /// you choose among defending player and/or any number of creatures they
    /// control". Returns the divisible creature set (every creature the
    /// defending player controls, id-ordered) or empty when the attacker has
    /// no such ability.
    fn free_division_targets(
        &self,
        attacker: CardId,
        computed: &[ComputedPermanent],
    ) -> Vec<CardId> {
        let free = computed.iter().any(|c| {
            c.id == attacker
                && c.keywords().has_kw(&Keyword::DividesCombatDamageAmongDefenders)
        });
        if !free {
            return vec![];
        }
        let Some(defender) = self
            .attacking
            .iter()
            .find(|a| a.attacker == attacker)
            .and_then(|a| self.defender_for(a.target))
        else {
            return vec![];
        };
        let mut ids: Vec<CardId> = computed
            .iter()
            .filter(|c| {
                c.controller == defender && c.card_types().contains(&crate::card::CardType::Creature)
            })
            .map(|c| c.id)
            .collect();
        ids.sort_by_key(|id| id.0);
        ids
    }

    /// The `(id, lethal)` set an attacker divides its combat damage among, and
    /// whether unassigned damage flows to the defending player. Normally the
    /// ordered blockers with trample as the outlet (CR 510.1c-d); a free
    /// divider (Butcher Orgg) uses the defending player's creatures with no
    /// lethal requirement and the player always as the outlet.
    fn combat_assignment_plan(
        &self,
        attacker: CardId,
        attacker_deathtouch: bool,
        has_trample: bool,
        order: &[CardId],
        computed: &[ComputedPermanent],
    ) -> (Vec<(CardId, u32)>, bool) {
        let free = self.free_division_targets(attacker, computed);
        if free.is_empty() {
            (self.combat_lethals(attacker_deathtouch, order, computed), has_trample)
        } else {
            (free.into_iter().map(|id| (id, 0)).collect(), true)
        }
    }

    fn combat_lethals(
        &self,
        attacker_deathtouch: bool,
        order: &[CardId],
        computed: &[ComputedPermanent],
    ) -> Vec<(CardId, u32)> {
        order
            .iter()
            .map(|&bid| {
                let tough = computed
                    .iter()
                    .find(|c| c.id == bid)
                    .map(|c| c.toughness.max(0) as u32)
                    .unwrap_or(0);
                // CR 510.1c — lethal accounts for damage already marked
                // (a double-strike trampler only needs the remainder in the
                // regular step). Deathtouch: any nonzero amount (702.2e).
                let marked = self
                    .battlefield_find(bid)
                    .map(|c| c.damage)
                    .unwrap_or(0);
                (bid, if attacker_deathtouch { 1 } else { tough.saturating_sub(marked) })
            })
            .collect()
    }

    /// Validate and cache one combat-damage decision answered via
    /// `submit_decision`, so the re-entered damage step finds it settled.
    pub(crate) fn apply_combat_decision_answer(
        &mut self,
        attacker: CardId,
        kind: crate::game::types::CombatDecisionKind,
        answer: &crate::decision::DecisionAnswer,
    ) {
        use crate::game::types::CombatDecisionKind;
        match kind {
            CombatDecisionKind::Order => {
                // `attacker` is the damage source: an attacker ordering its
                // blockers, or (CR 509.2) a multi-block blocker ordering the
                // attackers it blocks.
                let default_order = match self.attackers_blocked_by(attacker) {
                    atks if atks.len() > 1 => atks.iter().copied().collect(),
                    _ => self.blockers_of(attacker),
                };
                let order = self.resolve_damage_order(&default_order, answer);
                self.combat_damage_order.insert(attacker, order);
            }
            CombatDecisionKind::Assign => {
                let order = self
                    .combat_damage_order
                    .get(&attacker)
                    .cloned()
                    .unwrap_or_default();
                // The same computed view the resolver reads, not the whole
                // board: every id this branch looks up is a combat
                // participant, and `combat_damage_computed` already falls
                // back to the full view for the one whole-board consumer
                // (`free_division_targets`' Butcher Orgg half). Computing the
                // board here instead cost 16,939 Ir a call against ~4,900 for
                // the subset, and left the decision path reading a different
                // view from the resolver that consumes its answer.
                let computed = self.combat_damage_computed();
                let atk_cp = computed.iter().find(|c| c.id == attacker);
                let deathtouch = atk_cp
                    .is_some_and(|c| c.keywords().has_kw(&Keyword::Deathtouch));
                let power = atk_cp.map(combat_damage_value).unwrap_or(0);
                let total_power = if self.combat_damage_prevented_for_dealer(attacker) {
                    0
                } else {
                    power.max(0) as u32
                };
                // A multi-block blocker (CR 510.1e) has no trample outlet.
                let trample = self.attackers_blocked_by(attacker).len() <= 1
                    && atk_cp.is_some_and(|c| c.keywords().has_kw(&Keyword::Trample));
                let (lethals, trample) =
                    self.combat_assignment_plan(attacker, deathtouch, trample, &order, &computed);
                let split = self.resolve_damage_assignment(total_power, &lethals, trample, answer);
                self.combat_damage_assignment.insert(attacker, split);
            }
        }
    }

    /// Clear the cached combat-damage choices at the end of a combat phase.
    pub(crate) fn clear_combat_damage_plan(&mut self) {
        self.combat_damage_order.clear();
        self.combat_damage_assignment.clear();
        self.combat_damage_plan_step = None;
    }

    /// Core combat damage resolver. Each attacker has its own defending
    /// player or planeswalker (`Attack::target`); damage routing is
    /// per-attacker.
    fn resolve_combat_damage_with_filter(
        &mut self,
        computed: &[ComputedPermanent],
        attacker_filter: impl Fn(&[Keyword]) -> bool,
        blocker_filter: impl Fn(&[Keyword]) -> bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        let mut events = vec![];
        // Each call is one combat-damage batch (first-strike and regular
        // damage are separate sub-steps): reset the "one or more creatures
        // you control deal combat damage" graveyard-trigger dedupe.
        clear_cold!(self.gy_combat_trigger_fired_this_step);

        let computed_of =
            |id: CardId| -> Option<&ComputedPermanent> { computed.iter().find(|c| c.id == id) };

        let attacker_infos: Vec<AttackerInfo> = self
            .attacking
            .iter()
            .filter_map(|atk| {
                let cp = computed_of(atk.attacker)?;
                let defender_player = self.defender_for(atk.target)?;
                let kws = &cp.keywords();
                Some(AttackerInfo {
                    id: cp.id,
                    controller: cp.controller,
                    target: atk.target,
                    defender_player,
                    power: combat_damage_value(cp),
                    has_trample: kws.has_kw(&Keyword::Trample),
                    has_trample_over_pw: kws.has_kw(&Keyword::TrampleOverPlaneswalkers),
                    has_lifelink: kws.has_kw(&Keyword::Lifelink),
                    has_deathtouch: kws.has_kw(&Keyword::Deathtouch),
                    has_infect: kws.has_kw(&Keyword::Infect),
                    has_wither: kws.has_kw(&Keyword::Wither),
                    toxic: kws.iter().filter_map(|k| match k {
                        // Poisonous N (CR 702.70) folds into the same
                        // combat-damage poison rider as Toxic (CR 702.180).
                        Keyword::Toxic(n) | Keyword::Poisonous(n) => Some(*n),
                        _ => None,
                    }).sum(),
                    assigns_as_unblocked: kws
                        .has_kw(&Keyword::AssignsDamageAsThoughUnblocked),
                    // CR 510.1 — a creature with "deals no combat damage this
                    // turn" (Master of Cruelties) is skipped in both damage
                    // steps even though it's a legal attacker/blocker. CR 614.9
                    // — a Maze-of-Ith'd attacker deals no combat damage either.
                    should_deal: attacker_filter(kws)
                        && !kws.has_kw(&Keyword::DealsNoCombatDamage)
                        // CR 615.7 chosen-source prevention (Forge-Tender,
                        // Hallow, Awe Strike) is NOT short-circuited here: the
                        // damage still has to reach `apply_prevention_shields`
                        // so its life-gain / counter riders fire (CR 615.5).
                        && !self.combat_damage_prevented_creatures.contains(&cp.id)
                        // Kukemssa Pirates — "assigns no combat damage this
                        // turn" as the price of the stolen artifact.
                        && !self.assigns_no_combat_damage_this_turn.contains(&cp.id),
                })
            })
            .collect();

        // PHASE 1 — gather the active player's combat-damage ordering and
        // assignment choices for every multi-blocker attacker, before any
        // damage is dealt. For a `wants_ui` player each choice surfaces as a
        // `pending_decision` and this returns early; `submit_decision` then
        // re-enters this damage step (which re-runs the now-cached gather and
        // proceeds once every choice is settled). The choices are cached in
        // `combat_damage_order` / `combat_damage_assignment` and read in the
        // apply phase below.
        if self.gather_combat_damage_decisions(&attacker_infos, computed, &blocker_filter) {
            return Ok(vec![]);
        }
        // Past the early returns, this batch always emits at least one event
        // (`resolve_combat` appends `CombatResolved` unconditionally), and the
        // accumulator is threaded through `deal_combat_damage_to_target` and
        // `check_state_based_actions_into` after this. Those three share one
        // `Vec` and grew it 34,438 times a six-game `cube` run — 11 % of every
        // `grow_one` call in the program — climbing 0->4->8->16->32 per batch.
        events.reserve(32);

        // CR 615.1 — "Prevent all combat damage this turn" (Owlin
        // Shieldmage, Holy Day, Constant Mists). When the global flag is
        // set, every combat damage assignment yields 0; lifelink scales
        // off actual damage dealt (CR 702.15a), so prevention zeros
        // lifelink life-gain as well. Triggers that would fire off
        // "deals combat damage to a player" never see a damage event. With
        // an exception filter (Inspire Awe) the prevention is per-dealer, so
        // it's recomputed inside the per-attacker loop below.

        // CR 614.2 / 614.5 — global combat-damage doubling (Furnace of Rath)
        // and halving (Ghosts of the Innocent). Scaling applies to the amount
        // dealt (after assignment, before prevention), so a creature is still
        // assigned base lethal but takes the scaled total, and trample-over /
        // player damage scale too.


        // Creature-vs-creature combat damage recorded here and dispatched after
        // all damage in this step is dealt, so `DealsCombatDamageToCreature`
        // triggers (CR 510.2) go on the stack simultaneously (CR 603.3b).
        let mut creature_damage: SmallVec<[(CardId, CardId, u32); 8]> = SmallVec::new();

        for atk in &attacker_infos {
            // Fog (per-dealer) OR "prevent all combat damage it would deal"
            // (Azorius Ploy) both zero this attacker's outgoing damage while
            // still letting its blockers strike it back.
            //
            // `!should_deal` belongs in exactly the same bucket, and used to
            // `continue` past the whole pairing instead. CR 510.4/510.5 gate
            // each creature's damage on *its own* keywords: a first-striking
            // attacker deals in the first step and its ordinary blockers deal
            // in the regular one. Skipping the pairing whenever the attacker
            // was idle threw the blockers' half away with it, so a 3/2 first
            // striker walked through a 4/4 blocker untouched — and the mirror
            // case, a first-striking *blocker*, never got to strike before an
            // ordinary attacker hit back. The strike-back half below already
            // filters on the blocker's own keywords; it just has to be
            // reachable.
            let prevent_combat_damage = !atk.should_deal
                || self.combat_damage_prevented_for_dealer(atk.id)
                || self.combat_damage_prevented_from(atk.id);

            // CR 510.1c: the attacking player chose the order in which an
            // attacker assigns combat damage to its multiple blockers; that
            // choice was gathered in PHASE 1 and cached. Start from the
            // deterministic default (CardId = declaration-order proxy) and use
            // the cached order when present.
            let mut blocker_ids = self.blockers_of(atk.id);
            // CR 510.1a — "assigns its combat damage as though it weren't
            // blocked" (Predatory Focus): drop the blocker list so the whole
            // hit lands on the defending player. The blockers still deal
            // theirs, which the blocker loop below handles.
            // Read off `atk`, i.e. the one snapshot this step took, like
            // every other attacker keyword in this loop: CR 510.1 assignment
            // is a single turn-based action taken before any of the damage
            // below is dealt, so a life total this loop moves must not change
            // which attackers count as blocked. A fresh `computed_permanent`
            // here was a whole gather per attacker (13,601,323 Ir / 0.60 %
            // over 4,474 calls) *and* the only attacker keyword read at a
            // different game state from its siblings.
            if atk.assigns_as_unblocked {
                blocker_ids.clear();
                self.blocked_attackers.retain(|id| *id != atk.id);
            }
            if blocker_ids.len() > 1
                && let Some(order) = self.combat_damage_order.get(&atk.id)
            {
                blocker_ids = order.iter().copied().collect();
            }
            // Butcher Orgg divides over the defending player's creatures
            // instead of its blockers, blocked or not.
            let free_targets = self.free_division_targets(atk.id, computed);

            // Goblin Psychopath — the charge armed by a lost coin flip sends
            // this attacker's whole combat-damage assignment at its
            // controller instead (CR 614.9).
            // Soltari Guerrillas — the armed charge sends this attacker's
            // damage at a chosen creature instead of the defending player.
            if blocker_ids.is_empty()
                && free_targets.is_empty()
                && matches!(atk.target, AttackTarget::Player(_))
                && let Some(i) =
                    self.next_combat_damage_redirect.iter().position(|(a, _)| *a == atk.id)
            {
                let (_, victim) = self.next_combat_damage_redirect.remove(i);
                if self.battlefield_find(victim).is_some() {
                    let raw = if prevent_combat_damage { 0 } else { atk.power.max(0) as u32 };
                    self.deal_damage_to_from(
                        crate::game::effects::EntityRef::Permanent(victim),
                        raw,
                        Some(atk.id),
                        &mut events,
                    );
                    continue;
                }
            }

            if let Some(seat) = self.take_combat_damage_diversion(atk.id) {
                let raw = if prevent_combat_damage { 0 } else { atk.power.max(0) as u32 };
                self.deal_damage_to_from(
                    crate::game::effects::EntityRef::Player(seat),
                    raw,
                    Some(atk.id),
                    &mut events,
                );
                continue;
            }

            if blocker_ids.is_empty() && free_targets.is_empty() {
                // CR 510.1c — an attacker that became blocked stays blocked
                // even if all its blockers left combat (died to first-strike
                // damage or removal). Without trample it assigns no combat
                // damage; with trample everything goes to the defending
                // player (CR 702.19g).
                if self.blocked_attackers.contains(&atk.id) && !atk.has_trample {
                    continue;
                }
                let raw = if prevent_combat_damage {
                    0
                } else {
                    atk.power.max(0) as u32
                };
                // CR 615 — per-target prevention shields on the defending
                // player/planeswalker also reduce unblocked combat damage.
                // Lifelink scales off the post-prevention amount (702.15a).
                let amount = self.prevent_combat_to_target(
                    atk.target,
                    self.scale_combat_damage(Some(atk.id), atk.target, raw),
                    Some(atk.id),
                    &mut events,
                );
                if amount > 0 {
                    self.deal_combat_damage_to_target(atk, amount, &mut events);
                    if atk.has_lifelink {
                        let a = self.active_player_idx;
                        let applied = self.adjust_life_applied(a, amount as i32);
                        if applied > 0 {
                            events.push(GameEvent::LifeGained { player: a, amount: applied as u32 });
                        }
                    }
                }
            } else {
                let total_power = if prevent_combat_damage {
                    0
                } else {
                    atk.power.max(0) as u32
                };
                // CR 510.1c-d — the attacking player divides combat damage
                // among the blockers in the chosen order; that choice was
                // gathered in PHASE 1 and cached. Fall back to the default
                // lethal-to-each split when there's no cached choice (single
                // blocker, prevented, or a non-UI path that stored the
                // default). The lethal for each is 1 under deathtouch (CR
                // 702.2e).
                // CR 510.1c — lethal accounts for marked damage; deathtouch
                // needs only 1 (CR 702.2e). A free divider has no lethal
                // requirement and always spills the remainder to the player.
                let (lethals, leftover_to_player) = if free_targets.is_empty() {
                    let lethals = blocker_ids
                        .iter()
                        .map(|&bid| {
                            let tough = computed_of(bid)
                                .map(|c| c.toughness.max(0) as u32)
                                .unwrap_or(0);
                            let marked = self
                                .battlefield_find(bid)
                                .map(|c| c.damage)
                                .unwrap_or(0);
                            (bid, if atk.has_deathtouch { 1 } else { tough.saturating_sub(marked) })
                        })
                        .collect::<Vec<_>>();
                    (lethals, atk.has_trample)
                } else {
                    (free_targets.iter().map(|&id| (id, 0)).collect(), true)
                };
                let assignment = self
                    .combat_damage_assignment
                    .get(&atk.id)
                    .cloned()
                    .unwrap_or_else(|| {
                        self.default_damage_split(total_power, &lethals, leftover_to_player)
                    });
                let mut lifelink_dealt = 0i32;
                let mut assigned_to_blockers = 0u32;

                for &(blocker_id, assign) in &assignment {
                    assigned_to_blockers += assign;
                    if assign == 0 {
                        continue;
                    }
                    // One freeze scope over this pair's read-only prefix. Every
                    // check here is `&self` and the first `&mut self` call is
                    // `apply_prevention_shields` below, so no layer input can
                    // move between them — the reads already happened at one
                    // game state, this only stops each rebuilding the gather.
                    // `damage_prevented_by_protection` and `scale_damage_to`
                    // both take one, and both already freeze internally, so
                    // the scope is what merges them.
                    // The two reads after `apply_prevention_shields` — the CR
                    // 614.9 redirect and the CR 615 self-prevention pair — are
                    // `&self` and gather on their own, so they ride this scope
                    // too: three gathers per pair become one. Shields consume
                    // charges and emit events; no layer input moves between.
                    let (scaled, redirect_to, self_prevented) = self.with_frozen_layers(|g| {
                        let redirect_to = g.creature_redirects_damage_to_controller(blocker_id);
                        let self_prevented = g.combat_damage_prevented_to_self(blocker_id)
                            || g.damage_from_source_prevented_by_keyword(blocker_id, atk.id);
                        // CR 614.9 — a Maze-of-Ith'd blocker takes no combat
                        // damage. CR 615 — Emmara shields your creature tokens.
                        let scaled = if g.combat_damage_prevented_creatures.contains(&blocker_id)
                            || g.all_damage_to_creature_token_prevented(blocker_id)
                            || g.all_damage_to_your_creature_prevented(blocker_id)
                            // CR 702.16e — protection from the attacker's color
                            // prevents its combat damage to the blocker.
                            || g.damage_prevented_by_protection(atk.id, blocker_id)
                            // CR 615 — Light of Sanction: your source → your
                            // creature.
                            || g.damage_from_your_source_to_your_creature_prevented(
                                atk.id,
                                blocker_id,
                            )
                            // CR 615 — Indentured Oaf: this source prevents its
                            // own damage to creatures of a chosen color.
                            || g.source_damage_to_color_prevented(atk.id, blocker_id)
                        {
                            None
                        } else {
                            Some(g.double_creature_combat_damage(g.scale_damage_to(
                                Some(atk.id),
                                crate::game::effects::EntityRef::Permanent(blocker_id),
                                assign,
                            )))
                        };
                        (scaled, redirect_to, self_prevented)
                    });
                    let Some(scaled) = scaled else {
                        continue;
                    };
                    // CR 615 — route attacker→blocker combat damage through
                    // the blocker's prevention shields. Lifelink and the
                    // wither/infect -1/-1 counters scale off the actual
                    // (post-prevention) amount dealt (CR 702.15a).
                    let dealt = self.apply_prevention_shields(
                        crate::game::effects::EntityRef::Permanent(blocker_id),
                        scaled,
                        Some(atk.id),
                        &mut events,
                    ) as i32;
                    // Ironscale Hydra replaces the damage with a +1/+1 counter
                    // (and so the attacker's lifelink scales off 0).
                    let dealt =
                        self.ironscale_replace(blocker_id, redirect_to, dealt, &mut events);
                    // CR 615 — a blocker that prevents all damage to itself
                    // (Wall of Denial) takes none, and grants no lifelink.
                    let dealt = if self_prevented { 0 } else { dealt };
                    lifelink_dealt += dealt;

                    // Karona's Zealot — a standing turn-scoped redirect moves
                    // the whole combat-damage event onto another creature.
                    let blocker_id =
                        self.turn_damage_redirect_for(blocker_id).unwrap_or(blocker_id);
                    if dealt > 0 && let Some(b) = self.battlefield_find_mut(blocker_id) {
                        b.dealt_damage_this_turn = true;
                        b.damage_dealt_to_this_turn += dealt.max(0) as u32;
                        b.damaged_by_this_turn.push(atk.id);
                    }
                    if atk.has_infect || atk.has_wither {
                        if dealt > 0
                            && let Some(blocker) = self.battlefield_find_mut(blocker_id)
                        {
                            blocker.add_counters(
                                crate::card::CounterType::MinusOneMinusOne,
                                dealt as u32,
                            );
                            events.push(GameEvent::CounterAdded {
                                card_id: blocker_id,
                                counter_type: crate::card::CounterType::MinusOneMinusOne,
                                count: dealt as u32,
                            });
                        }
                    } else if dealt > 0
                        && let Some(blocker) = self.battlefield_find_mut(blocker_id)
                    {
                        blocker.damage += dealt as u32;
                        blocker.record_damage_from(atk.id, dealt as u32);
                        if atk.has_deathtouch {
                            blocker.dealt_deathtouch_damage = true;
                        }
                        events.push(GameEvent::DamageDealt {
                            amount: dealt as u32,
                            to_player: None,
                            to_card: Some(blocker_id),
                            combat: true,
                            from_controller: Some(atk.controller),
                            from_card: Some(atk.id),
                        });
                        creature_damage.push((atk.id, blocker_id, dealt as u32));
                    }
                }

                let trample_leftover = total_power.saturating_sub(assigned_to_blockers);
                if leftover_to_player && trample_leftover > 0 {
                    // Trample-over damage to the defending player/PW is also
                    // subject to prevention shields; lifelink follows the
                    // post-prevention amount.
                    let amount = self.prevent_combat_to_target(
                        atk.target,
                        self.scale_combat_damage(Some(atk.id), atk.target, trample_leftover),
                        Some(atk.id),
                        &mut events,
                    );
                    lifelink_dealt += amount as i32;
                    if amount > 0 {
                        self.deal_combat_damage_to_target(atk, amount, &mut events);
                    }
                }

                if atk.has_lifelink && lifelink_dealt > 0 {
                    let a = self.active_player_idx;
                    let applied = self.adjust_life_applied(a, lifelink_dealt);
                    if applied > 0 {
                        events.push(GameEvent::LifeGained { player: a, amount: applied as u32 });
                    }
                }

                // Only blockers whose own keywords say they deal damage in
                // this step strike back at the attacker. Per CR 510.4/510.5
                // the attacker's keywords don't gate the blocker's strike
                // step — a regular blocker must wait for the regular step
                // even if the attacker has first strike.
                // One freeze scope for the whole strike-back gate: every
                // predicate below is `&self`, so the per-blocker
                // `damage_prevented_by_protection` walks share one gather
                // instead of taking one each.
                let (dealing_blocker_ids, attacker_takes_strike_back) =
                    self.with_frozen_layers(|g| {
                        // A `for` loop, not seven stacked `.filter()`s: the
                        // adapters are a per-element branch each *and* seven
                        // nested `Filter` values to build per call, on a list
                        // that is one or two blockers long — so the chain's
                        // setup is most of what it costs (PERF, the
                        // eighty-seventh pass's concurrent half). The
                        // short-circuit order is the chain's, unchanged:
                        // cheapest and most selective first.
                        let mut ids: Vec<CardId> = Vec::new();
                        for &bid in blocker_ids.iter() {
                            if !computed_of(bid)
                                .is_some_and(|bc| blocker_filter(bc.keywords()))
                                // CR 614.9 — a Maze-of-Ith'd blocker deals no combat damage.
                                || g.combat_damage_prevented_creatures.contains(&bid)
                                // CR 615.1 — "prevent all combat damage it would deal"
                                // (Azorius Ploy).
                                || g.combat_damage_prevented_from(bid)
                                // CR 615.1 — fog (with Inspire Awe's per-dealer exception).
                                || g.combat_damage_prevented_for_dealer(bid)
                                // CR 702.16e — a blocker whose color the attacker has
                                // protection from deals no combat damage to it.
                                || g.damage_prevented_by_protection(bid, atk.id)
                                // CR 615 — Light of Sanction: your source → your creature.
                                || g.damage_from_your_source_to_your_creature_prevented(
                                    bid, atk.id,
                                )
                                // CR 615 — Indentured Oaf: a blocker's own damage to a
                                // chosen color is prevented.
                                || g.source_damage_to_color_prevented(bid, atk.id)
                            {
                                continue;
                            }
                            ids.push(bid);
                        }
                        let takes =
                            // CR 614.9 — a Maze-of-Ith'd attacker takes no combat damage.
                            !g.combat_damage_prevented_creatures.contains(&atk.id)
                            // CR 615 — Iroas shields attacking creatures you control.
                            && !g.damage_to_attacker_prevented(atk.id)
                            // CR 615 — Emmara shields your creature tokens.
                            && !g.all_damage_to_creature_token_prevented(atk.id)
                            // CR 615 — Rune-Tail's Essence shields all your creatures.
                            && !g.all_damage_to_your_creature_prevented(atk.id);
                        (ids, takes)
                    });

                if attacker_takes_strike_back {
                    // CR 702.90 / 615.6 — each blocker's strike-back is its
                    // own damage event: scaling (CR 614.2), prevention
                    // shields, infect/wither, deathtouch, and lifelink all
                    // apply per source, not to the summed total.
                    let mut lifelink_by_controller: crate::fxhash::HashMap<usize, i32> =
                        crate::fxhash::HashMap::default();
                    for &bid in &dealing_blocker_ids {
                        let Some(bc) = computed_of(bid) else { continue };
                        // CR 510.1e — a multi-block blocker only assigns this
                        // attacker its share of the divided damage.
                        // Same read-only prefix as the attacker side: both
                        // calls are `&self` and `apply_prevention_shields` is
                        // the first write, so one scope holds the pair.
                        let (scaled, redirect_to, self_prevented) = self.with_frozen_layers(|g| {
                            let power = g.blocker_damage_to(
                                bid,
                                atk.id,
                                combat_damage_value(bc).max(0) as u32,
                            );
                            let scaled = (power != 0).then(|| {
                                g.double_creature_combat_damage(g.scale_damage_to(
                                    Some(bid),
                                    crate::game::effects::EntityRef::Permanent(atk.id),
                                    power,
                                ))
                            });
                            // Same fold as the attacker side: the post-shield
                            // reads gather on their own outside a scope.
                            let redirect_to = g.creature_redirects_damage_to_controller(atk.id);
                            let self_prevented = g.combat_damage_prevented_to_self(atk.id)
                                || g.combat_damage_from_blockers_prevented(atk.id)
                                || g.damage_from_source_prevented_by_keyword(atk.id, bid);
                            (scaled, redirect_to, self_prevented)
                        });
                        let Some(scaled) = scaled else {
                            continue;
                        };
                        let dmg = self.apply_prevention_shields(
                            crate::game::effects::EntityRef::Permanent(atk.id),
                            scaled,
                            Some(bid),
                            &mut events,
                        );
                        // Ironscale Hydra replaces the blocker's strike-back
                        // with a +1/+1 counter (blocker's lifelink sees 0).
                        let dmg = self
                            .ironscale_replace(atk.id, redirect_to, dmg as i32, &mut events)
                            as u32;
                        // CR 615 — an attacker that prevents all damage to itself,
                        // or specifically damage from its blockers (Armored
                        // Transport), takes none from this blocker (no lifelink).
                        let dmg = if self_prevented { 0 } else { dmg };
                        if dmg == 0 {
                            continue;
                        }
                        let infect = bc.keywords().has_kw(&Keyword::Infect)
                            || bc.keywords().has_kw(&Keyword::Wither);
                        let hit = self.turn_damage_redirect_for(atk.id).unwrap_or(atk.id);
                        if let Some(attacker) = self.battlefield_find_mut(hit) {
                            attacker.dealt_damage_this_turn = true;
                            attacker.damage_dealt_to_this_turn += dmg;
                            attacker.damaged_by_this_turn.push(bid);
                            if infect {
                                attacker
                                    .add_counters(crate::card::CounterType::MinusOneMinusOne, dmg);
                                events.push(GameEvent::CounterAdded {
                                    card_id: hit,
                                    counter_type: crate::card::CounterType::MinusOneMinusOne,
                                    count: dmg,
                                });
                            } else {
                                attacker.damage += dmg;
                                attacker.record_damage_from(bid, dmg);
                                if bc.keywords().has_kw(&Keyword::Deathtouch) {
                                    attacker.dealt_deathtouch_damage = true;
                                }
                                events.push(GameEvent::DamageDealt {
                                    amount: dmg,
                                    to_player: None,
                                    to_card: Some(hit),
                                    combat: true,
                                    from_controller: Some(bc.controller),
                                    from_card: Some(bid),
                                });
                            }
                        }
                        // CR 510.2 — this blocker dealt combat damage to a
                        // creature (post-prevention amount).
                        creature_damage.push((bid, atk.id, dmg));
                        // CR 702.15a — lifelink scales off damage actually
                        // dealt; credited to the blocker's controller.
                        if bc.keywords().has_kw(&Keyword::Lifelink) {
                            let controller = self
                                .battlefield
                                .iter()
                                .find(|c| c.id == bid)
                                .map(|c| c.controller)
                                .unwrap_or(atk.defender_player);
                            *lifelink_by_controller.entry(controller).or_insert(0) += dmg as i32;
                        }
                    }
                    // Sort by seat for deterministic event ordering;
                    // life-gain math is commutative but the event log
                    // shouldn't shuffle across replays.
                    let mut lifelink_entries: Vec<(usize, i32)> =
                        lifelink_by_controller.into_iter().collect();
                    lifelink_entries.sort_by_key(|(p, _)| *p);
                    for (player, gained) in lifelink_entries {
                        if gained > 0 {
                            let applied = self.adjust_life_applied(player, gained);
                            if applied > 0 {
                                events.push(GameEvent::LifeGained {
                                    player,
                                    amount: applied as u32,
                                });
                            }
                        }
                    }
                }
            }
        }

        // CR 614 — a source with the exile-on-death damage rider (Kumano's
        // Pupils, Kumano) exiles instead of buries any creature it damaged
        // this turn that would die. Mirrors the non-combat path in
        // `deal_damage_to_from`.
        for &(source, damaged, _) in &creature_damage {
            if self.damage_exiles_victim_eot.contains(&source)
                || self
                    .battlefield_find(source)
                    .is_some_and(|c| c.definition.damage_exiles_if_dies)
            {
                self.dies_to_exile_eot.insert(damaged);
            }
            // Runesword — its damage also denies regeneration this turn.
            if self.damage_denies_regen_eot.contains(&source)
                && let Some(v) = self.battlefield_find_mut(damaged)
            {
                v.cant_regenerate_this_turn = true;
            }
        }
        // Stamp the damaging source's controller on each recipient so a
        // "whenever this is dealt combat damage" trigger can still name the
        // attacking player once `block_map` is torn down (Souls of the Faultless).
        for &(source, damaged, _) in &creature_damage {
            if let Some(ctrl) = self.battlefield_find(source).map(|c| c.controller)
                && let Some(c) = self.battlefield_find_mut(damaged)
            {
                c.combat_damager_controller = Some(ctrl);
            }
        }
        // CR 510.2 — now that all combat damage in this step has been dealt,
        // put `DealsCombatDamageToCreature` triggers on the stack.
        //
        // One `trigger_grant_sources` walk for the whole batch, not one per
        // damaged creature. That walk is a whole-board pass over every static
        // ability, it found 0.25 grants per call at the eighty-second tip, and
        // the per-event rebuild made it 12,858 calls / 25,617,351 Ir / 0.72 %
        // of `cube` between them. Reading the grants once, before any of this
        // batch's triggers reach the stack, is also the CR 510.2 order: the
        // per-event rebuild was re-evaluating each grant's gate against a
        // stack that already held the earlier triggers.
        let granted: Vec<Vec<crate::card::TriggeredAbility>> = {
            let grants = self.trigger_grant_sources();
            creature_damage
                .iter()
                .map(|&(source, ..)| {
                    self.battlefield_find(source)
                        .map(|c| self.statics_granted_triggers_with(c, &grants))
                        .unwrap_or_default()
                })
                .collect()
        };
        for ((source, damaged, amount), granted) in creature_damage.into_iter().zip(granted) {
            self.fire_source_dealt_damage_watchers(source, amount);
            self.fire_combat_damage_to_creature_triggers(source, damaged, amount, &granted);
        }

        Ok(events)
    }

    /// CR 506.2 — whether `id` carries a computed `Keyword::CantBeAttacked`,
    /// so it can't be declared as an attack target (The Aetherspark).
    pub(crate) fn permanent_cant_be_attacked(&self, id: CardId) -> bool {
        self.computed_permanent(id)
            .is_some_and(|cp| cp.keywords().has_kw(&Keyword::CantBeAttacked))
    }

    /// Apply `amount` damage from `atk` to its declared attack target. For
    /// player targets this is life loss (or poison if Infect); for
    /// planeswalker targets this is loyalty loss. Also fires
    /// `DealsCombatDamageToPlayer` triggers when a player is hit.
    /// CR 615 — apply prevention shields to combat damage headed for an
    /// attack target (player or planeswalker). Returns the unprevented
    /// remainder. Creature-vs-creature combat damage is not yet routed
    /// through shields (tracked in TODO.md).
    /// Target-aware scaling for a combat-damage event aimed at a player or
    /// planeswalker attack target (CR 614.2 / 614.5). `source` is the
    /// attacking creature (Torbran's source-scoped bonus).
    /// CR 615.1 — is `dealer`'s combat damage prevented this turn? True when
    /// the global fog flag is set, unless an exception filter (Inspire Awe)
    /// is in play and `dealer` matches it (enchanted / enchantment creatures
    /// still deal damage).
    pub(crate) fn combat_damage_prevented_for_dealer(&self, dealer: CardId) -> bool {
        if !self.prevent_combat_damage_this_turn {
            return false;
        }
        // CR 615.12 (scoped) — Questing Beast: combat damage dealt by creatures
        // its controller controls can't be prevented, so fog doesn't stop it.
        if let Some(ctrl) = self.battlefield_find(dealer).map(|c| c.controller)
            && self.battlefield.iter().any(|c| {
                c.controller == ctrl
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(
                            sa.effect,
                            crate::effect::StaticEffect::ControllerCreaturesCombatDamageCantBePrevented
                        )
                    })
            })
        {
            return false;
        }
        match &self.prevent_combat_damage_except {
            None => true,
            Some(filter) => {
                let controller =
                    self.battlefield_find(dealer).map(|c| c.controller).unwrap_or(0);
                !self.evaluate_requirement(filter, &Target::Permanent(dealer), controller)
            }
        }
    }

    /// Apply the Ironscale replacement to `recipient` taking `dealt` combat
    /// damage: if it has the static and `dealt > 0`, add one +1/+1 counter,
    /// emit the event, and return 0 (the damage is prevented); otherwise
    /// return `dealt` unchanged.
    ///
    /// `redirect_to` is [`creature_redirects_damage_to_controller`] answered by
    /// the caller: the CR 614.9 check is the only layer read this function
    /// needs, and outside a freeze scope it re-gathers every continuous effect
    /// in the game. Both damage loops take it in the read-only prefix they
    /// already freeze, so a pair pays one gather instead of two.
    ///
    /// [`creature_redirects_damage_to_controller`]: Self::creature_redirects_damage_to_controller
    fn ironscale_replace(
        &mut self,
        recipient: CardId,
        redirect_to: Option<usize>,
        dealt: i32,
        events: &mut Vec<GameEvent>,
    ) -> i32 {
        if dealt <= 0 {
            return dealt;
        }
        // CR 614.9 — Treacherous Link redirects combat damage too.
        if let Some(owner) = redirect_to {
            self.deal_damage_to_from(
                crate::game::effects::EntityRef::Player(owner),
                dealt as u32,
                None,
                events,
            );
            return 0;
        }
        // The three questions below are one walk of one card's
        // `static_abilities`, not three `battlefield_find`s and three walks:
        // Sekki's trade, Ironscale's grow, and the replace-with-counters
        // family all read the same list of the same permanent, and this runs
        // once per combat-damage assignment to a creature.
        // `creature_replaces_damage_with_counters` stays — `deal_damage_to_from`
        // asks it too — and `creature_prevents_combat_damage_grows` is gone,
        // this was its only caller.
        let (sekki, grows, replace_kind) = match self.battlefield_find(recipient) {
            Some(c) => {
                use crate::effect::StaticEffect as SE;
                let (mut sekki, mut grows, mut kind) = (false, false, None);
                for sa in &c.definition.static_abilities {
                    match sa.effect {
                        SE::PreventDamageToSelfTradingCounters { .. } => sekki = true,
                        SE::PreventCombatDamageToSelfAndGrow => grows = true,
                        SE::ReplaceDamageToSelfWithCounters { kind: k } => {
                            kind.get_or_insert(k);
                        }
                        _ => {}
                    }
                }
                (sekki, grows, kind)
            }
            None => return dealt,
        };
        // Sekki trades counters for tokens instead of growing (CR 615). The
        // gate is exactly `trade_counters_for_damage`'s own: without the
        // static it returns `false`, and `dealt > 0` is checked above.
        if sekki && self.trade_counters_for_damage(recipient, dealt as u32, events) {
            return 0;
        }
        // Ironscale Hydra grows by exactly one; Phytohydra grows by the full
        // amount. Both are replacements (CR 614), so they apply even when
        // damage can't be prevented.
        let (kind, grow) = if grows {
            (crate::card::CounterType::PlusOnePlusOne, 1)
        } else if let Some(kind) = replace_kind {
            (kind, dealt as u32)
        } else {
            return dealt;
        };
        if let Some(c) = self.battlefield_find_mut(recipient) {
            c.add_counters(kind, grow);
        }
        events.push(GameEvent::CounterAdded {
            card_id: recipient,
            counter_type: kind,
            count: grow,
        });
        0
    }

    /// Goblin Psychopath — "the next time this would deal combat damage this
    /// turn, it deals that damage to you instead". Consumes the charge and
    /// returns the creature's controller.
    pub(crate) fn take_combat_damage_diversion(&mut self, id: CardId) -> Option<usize> {
        let idx = self.next_combat_damage_to_controller.iter().position(|c| *c == id)?;
        self.next_combat_damage_to_controller.remove(idx);
        self.battlefield_find(id).map(|c| c.controller)
    }

    /// CR 614.9 — Treacherous Link: does `id` redirect damage aimed at it onto
    /// its controller? Returns that controller when it does.
    pub(crate) fn creature_redirects_damage_to_controller(&self, id: CardId) -> Option<usize> {
        let c = self.battlefield_find(id)?;
        // This is the **first** read in `resolve_combat_damage_with_filter`'s
        // per-pair freeze scope, so an ungated `computed_permanent` here is
        // what makes that scope gather — 12,786 gathers a six-game `cube` run
        // between this and its sibling below (PERF `(-81)`'s context census).
        // Same shape as `damage_prevented_by_protection_inner`: the presence
        // gate answers "no" authoritatively without gathering, and it is
        // skipped once the gather has already happened, where the memo read is
        // cheaper than the gate's own board walk.
        const KW: crate::card::Keyword = crate::card::Keyword::DamageToThisGoesToItsController;
        // `_on`, not the `CardId` form: `c` above is exactly the
        // `battlefield_find` that form opens with, and this is asked once per
        // combat-damage assignment.
        if !self.layers_memoized() && !self.card_keyword_possible_on(c, |k| *k == KW) {
            debug_assert!(
                !self
                    .computed_permanent(id)
                    .is_some_and(|cp| cp.keywords().has_kw(&KW)),
                "card_keyword_possible missed a granted DamageToThisGoesToItsController",
            );
            return None;
        }
        self.computed_permanent(id)?.keywords().has_kw(&KW).then_some(c.controller)
    }

    /// Sekki, Seasons' Guide (CR 615) — prevent `dealt` damage to `recipient`,
    /// remove that many counters, and mint that many tokens. Returns false
    /// (damage stands) when `recipient` has no such static.
    pub(crate) fn trade_counters_for_damage(
        &mut self,
        recipient: CardId,
        dealt: u32,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        if dealt == 0 || self.damage_cant_be_prevented_this_turn {
            return false;
        }
        let Some((counter, token)) = self.battlefield_find(recipient).and_then(|c| {
            c.definition.static_abilities.iter().find_map(|s| match &s.effect {
                crate::effect::StaticEffect::PreventDamageToSelfTradingCounters {
                    counter,
                    token,
                } => Some((*counter, (**token).clone())),
                _ => None,
            })
        }) else {
            return false;
        };
        let (controller, removed) = match self.battlefield_find_mut(recipient) {
            Some(c) => {
                let removed = dealt.min(c.counter_count(counter));
                c.remove_counters(counter, removed);
                (c.controller, removed)
            }
            None => return false,
        };
        if removed > 0 {
            events.push(GameEvent::CounterRemoved {
                card_id: recipient,
                counter_type: counter,
                count: removed,
            });
        }
        let def = crabomination_base::tokens::token_card_arc(&token);
        for _ in 0..dealt {
            self.mint_token_onto_battlefield(def.clone(), controller, false, events);
        }
        true
    }

    /// Phytohydra (CR 614) — does `id` replace incoming damage with that many
    /// +1/+1 counters? Reads `StaticEffect::ReplaceDamageToSelfWithCounters`.
    pub(crate) fn creature_replaces_damage_with_counters(
        &self,
        id: CardId,
    ) -> Option<crate::card::CounterType> {
        self.battlefield_find(id).and_then(|c| {
            c.definition.static_abilities.iter().find_map(|s| match s.effect {
                crate::effect::StaticEffect::ReplaceDamageToSelfWithCounters { kind } => Some(kind),
                _ => None,
            })
        })
    }

    fn scale_combat_damage(
        &self,
        source: Option<crate::card::CardId>,
        target: AttackTarget,
        amount: u32,
    ) -> u32 {
        use crate::game::effects::EntityRef;
        let ent = match target {
            AttackTarget::Player(p) => EntityRef::Player(p),
            AttackTarget::Planeswalker(pw) => EntityRef::Permanent(pw),
            AttackTarget::Battle(b) => EntityRef::Permanent(b),
        };
        self.scale_damage_to(source, ent, amount)
    }

    fn prevent_combat_to_target(
        &mut self,
        target: AttackTarget,
        amount: u32,
        source: Option<crate::card::CardId>,
        events: &mut Vec<GameEvent>,
    ) -> u32 {
        use crate::game::effects::EntityRef;
        match target {
            AttackTarget::Player(p) => {
                // CR 615 — Glacial-Chasm-style blanket prevention soaks the whole
                // combat hit before any shield is consumed.
                if self.all_damage_to_player_prevented(p) {
                    return 0;
                }
                // CR 615 — turn-scoped "prevent all combat damage that would be
                // dealt to you this turn" (Druid's Deliverance).
                if !self.damage_cant_be_prevented_this_turn
                    && self.combat_damage_prevented_to_players_this_turn.contains(&p)
                {
                    return 0;
                }
                // CR 702.16j — protection from a card type (Serra's Emissary):
                // no combat damage from an attacker of that type.
                if let Some(src) = source {
                    let types = self.player_protection_card_types(p);
                    if !types.is_empty()
                        && self
                            .computed_permanent(src)
                            .is_some_and(|c| types.iter().any(|t| c.card_types().contains(t)))
                    {
                        return 0;
                    }
                }
                self.apply_prevention_shields(EntityRef::Player(p), amount, source, events)
            }
            AttackTarget::Planeswalker(pw) => {
                self.apply_prevention_shields(EntityRef::Permanent(pw), amount, source, events)
            }
            AttackTarget::Battle(b) => {
                self.apply_prevention_shields(EntityRef::Permanent(b), amount, source, events)
            }
        }
    }

    /// CR 615 — total per-creature combat-damage reduction `p` gets from
    /// their untapped `ReduceCombatDamageToControllerWhileUntapped` permanents
    /// (Thunderstaff). Applies once per damaging creature, not once per point.
    fn combat_damage_shaved_for(&self, p: usize) -> u32 {
        self.battlefield
            .iter()
            .filter(|c| c.controller == p && !c.tapped)
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|s| match s.effect {
                crate::effect::StaticEffect::ReduceCombatDamageToControllerWhileUntapped(n) => {
                    Some(n)
                }
                _ => None,
            })
            .sum()
    }

    fn deal_combat_damage_to_target(
        &mut self,
        atk: &AttackerInfo,
        amount: u32,
        events: &mut Vec<GameEvent>,
    ) {
        match atk.target {
            AttackTarget::Player(p) => {
                // CR 615 — per-creature combat-damage shaving from an untapped
                // permanent the defender controls (Thunderstaff).
                let shave = self.combat_damage_shaved_for(p);
                let amount = amount.saturating_sub(shave);
                if amount == 0 && shave > 0 {
                    return;
                }
                // CR 614 — Szadek: this attacker's combat damage to a player
                // becomes that many +1/+1 counters on it, and the player mills
                // that many instead of losing life.
                if amount > 0
                    && self.battlefield_find(atk.id).is_some_and(|c| {
                        c.definition.static_abilities.iter().any(|s| {
                            matches!(
                                s.effect,
                                crate::effect::StaticEffect::CombatDamageToPlayerBecomesCountersAndMill
                            )
                        })
                    })
                {
                    if let Some(c) = self.battlefield_find_mut(atk.id) {
                        c.add_counters(crate::card::CounterType::PlusOnePlusOne, amount);
                    }
                    events.push(GameEvent::CounterAdded {
                        card_id: atk.id,
                        counter_type: crate::card::CounterType::PlusOnePlusOne,
                        count: amount,
                    });
                    for _ in 0..amount {
                        if self.players[p].library.is_empty() {
                            break;
                        }
                        let card = self.players[p].library.remove(0);
                        let cid = card.id;
                        if !self.route_to_graveyard(card, events) {
                            events.push(GameEvent::CardMilled { player: p, card_id: cid });
                        }
                    }
                    return;
                }
                // CR 614.9 — Palisade-Giant-style redirect: combat damage
                // aimed at the player lands on the redirector instead. Turn
                // the Tables' combat-only redirect is checked first; both
                // require the destination to still be on the battlefield.
                if let Some(redirect) = self
                    .combat_damage_redirect_this_turn
                    .iter()
                    .find(|(seat, to)| *seat == p && self.battlefield_find(*to).is_some())
                    .map(|(_, to)| *to)
                    .or_else(|| {
                        self.damage_redirect_target(crate::game::effects::EntityRef::Player(p))
                    })
                {
                    if let Some(c) = self.battlefield_find_mut(redirect) {
                        c.damage += amount;
                        c.dealt_damage_this_turn = true;
                        c.damage_dealt_to_this_turn += amount;
                        c.damaged_by_this_turn.push(atk.id);
                        c.record_damage_from(atk.id, amount);
                    }
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: None,
                        to_card: Some(redirect),
                        combat: true,
                        from_controller: Some(atk.controller),
                        from_card: Some(atk.id),
                    });
                    return;
                }
                // Phyrexian Unlife — at ≤ 0 life all damage lands as poison.
                if atk.has_infect || (self.players[p].life <= 0 && self.player_unlife_active(p)) {
                    self.add_poison(p, amount, events);
                } else {
                    // Angel's Grace / Worship — damage lands in full, but the
                    // life reduction is clamped to the floor.
                    let life_delta = self.clamp_damage_to_life_floor(p, amount);
                    let applied = self.adjust_life_applied(p, -(life_delta as i32));
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: Some(p),
                        to_card: None,
                        combat: true,
                        from_controller: Some(atk.controller),
                        from_card: Some(atk.id),
                    });
                    let amount = (-applied).max(0) as u32;
                    events.push(GameEvent::LifeLost {
                        player: p,
                        amount,
                    });
                }
                // Mark the player damaged this turn (Bloodthirst window, CR
                // 702.54) and record the attacker so "destroy target creature
                // that dealt damage to you this turn" (Spear of Heliod) can
                // filter targets.
                if amount > 0 {
                    // One `Player::deref_mut` for the run: `Player` is a CoW
                    // handle, so each write below was its own `Arc::make_mut`.
                    let atk_id = atk.id;
                    let pl = &mut *self.players[p];
                    pl.was_dealt_damage_this_turn = true;
                    pl.damage_taken_this_turn = pl.damage_taken_this_turn.saturating_add(amount);
                    pl.combat_damage_taken_this_turn =
                        pl.combat_damage_taken_this_turn.saturating_add(amount);
                    if !pl.creatures_that_damaged_me_this_turn.contains(&atk_id) {
                        pl.creatures_that_damaged_me_this_turn.push(atk_id);
                    }
                    // CR 702.76 — Prowl window: record the damaging creature's
                    // types for its controller (Changeling counts as every
                    // type, recorded via the controller-side any flag).
                    if let Some(c) = self.battlefield.find_by_id(atk.id) {
                        let ctrl = c.controller;
                        if c.definition.keywords.has_kw(&Keyword::Changeling) {
                            self.players[ctrl].prowl_any_type_this_turn = true;
                        }
                        let types = c.definition.subtypes.creature_types.clone();
                        for t in types {
                            if !self.players[ctrl].prowl_types_this_turn.contains(&t) {
                                self.players[ctrl].prowl_types_this_turn.push(t);
                            }
                        }
                    }
                }
                // CR 702.180c — Toxic N adds N poison on combat damage to a
                // player, on top of any life loss (and stacks with Infect's
                // poison). Only when damage was actually dealt.
                if atk.toxic > 0 && amount > 0 {
                    self.add_poison(p, atk.toxic, events);
                }
                // Phase M: bump the 21-commander-damage tally when the
                // attacker is a Commander. Both Infect and regular
                // damage paths credit here — CR 704.5v doesn't restrict
                // by damage type. The SBA in `check_state_based_actions`
                // reads this table and eliminates the player when any
                // single (victim, commander) entry crosses 21.
                if self.is_commander(atk.id) {
                    self.record_commander_damage(p, atk.id, amount);
                }
                // CR 725 — a creature dealing combat damage to the monarch
                // makes its controller the new monarch.
                if amount > 0 && self.monarch == Some(p) {
                    let ctrl = self.battlefield.iter()
                        .find(|c| c.id == atk.id).map(|c| c.controller);
                    if let Some(ctrl) = ctrl
                        && ctrl != p {
                            self.set_monarch(ctrl, events);
                        }
                }
                // CR 726.2 — the same handover for the initiative.
                if amount > 0 && self.initiative == Some(p) {
                    let ctrl = self.battlefield.iter()
                        .find(|c| c.id == atk.id).map(|c| c.controller);
                    if let Some(ctrl) = ctrl
                        && ctrl != p {
                            self.take_initiative(ctrl, events);
                        }
                }
                self.fire_combat_damage_to_player_triggers(atk.id, p, amount);
            }
            AttackTarget::Planeswalker(pw_id) => {
                // CR 702.19c — "trample over planeswalkers" (Thrasta): the
                // attacker assigns lethal (= remaining loyalty) to the
                // planeswalker and the excess to its controller. Plain
                // trample never spills past a planeswalker (CR 702.19f).
                let mut spill = 0u32;
                let mut spill_to: Option<usize> = None;
                if let Some(pw) = self.battlefield_find(pw_id) {
                    let current = pw.counter_count(crate::card::CounterType::Loyalty);
                    if atk.has_trample_over_pw && amount > current {
                        spill = amount - current;
                        spill_to = Some(pw.controller);
                    }
                }
                let amount = amount - spill;
                if let Some(pw) = self.battlefield_find_mut(pw_id) {
                    let current = pw.counter_count(crate::card::CounterType::Loyalty);
                    let new_loyalty = current.saturating_sub(amount);
                    pw.counters
                        .insert(crate::card::CounterType::Loyalty, new_loyalty);
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: None,
                        to_card: Some(pw_id),
                        combat: true,
                        from_controller: Some(atk.controller),
                        from_card: Some(atk.id),
                    });
                    events.push(GameEvent::LoyaltyChanged {
                        card_id: pw_id,
                        new_loyalty: new_loyalty as i32,
                    });
                    // Combat damage removes loyalty counters (Chandra, Fire
                    // Artisan's removal trigger).
                    let removed = current.saturating_sub(new_loyalty);
                    if removed > 0 {
                        events.push(GameEvent::CounterRemoved {
                            card_id: pw_id,
                            counter_type: crate::card::CounterType::Loyalty,
                            count: removed,
                        });
                    }
                }
                if amount > 0 {
                    let granted = self.static_granted_triggers_of(atk.id);
                    self.fire_combat_damage_triggers(
                        atk.id,
                        &[EventKind::DealsCombatDamageToPlaneswalker],
                        Target::Permanent(pw_id),
                        amount,
                        &granted,
                    );
                }
                if let Some(p) = spill_to
                    && spill > 0
                {
                    let mut spilled = atk.clone();
                    spilled.target = AttackTarget::Player(p);
                    self.deal_combat_damage_to_target(&spilled, spill, events);
                }
            }
            AttackTarget::Battle(b_id) => {
                // CR 310.10 — combat damage to a battle removes that many
                // defense counters. The defeat trigger fires from the SBA once
                // the last counter is gone.
                if let Some(b) = self.battlefield_find_mut(b_id) {
                    let current = b.counter_count(crate::card::CounterType::Defense);
                    let new_defense = current.saturating_sub(amount);
                    b.counters
                        .insert(crate::card::CounterType::Defense, new_defense);
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: None,
                        to_card: Some(b_id),
                        combat: true,
                        from_controller: Some(atk.controller),
                        from_card: Some(atk.id),
                    });
                }
            }
        }
    }

    /// CR 508.1g / 509.1b — pick the untapped permanent `who` taps to pay
    /// `payer`'s `AttackBlockCostTapAnother` cost. Candidates exclude the payer
    /// itself, everything declared in combat this step, and helpers already
    /// spent on an earlier payer in the same declaration.
    fn find_tap_helper(
        &self,
        who: usize,
        filter: &crate::card::SelectionRequirement,
        payer: CardId,
        declared: &[CardId],
        spent: &[CardId],
    ) -> Option<CardId> {
        self.battlefield
            .iter()
            .find(|c| {
                c.controller == who
                    && !c.tapped
                    && c.id != payer
                    && !declared.contains(&c.id)
                    && !spent.contains(&c.id)
                    && self.evaluate_requirement_static(
                        filter,
                        &crate::game::types::Target::Permanent(c.id),
                        who,
                        Some(payer),
                    )
            })
            .map(|c| c.id)
    }

    /// CR 509.1d — `(mana, life)` one declared blocker costs its controller:
    /// every active `BlockTaxToController` (Archangel of Tithes, Heat Wave,
    /// Norn's Annex) plus the turn-scoped `block_tax_this_turn`.
    ///
    /// Per blocker and additive, because a source may narrow itself to some
    /// of them and may charge life rather than mana — which is also what
    /// lets the bot's block planner trim a batch down to what it can pay.
    /// The engine pays it after every legality check has passed and rejects
    /// the declaration whole when a player cannot cover theirs, so a planner
    /// that does not price it loses the block step rather than a blocker.
    pub(crate) fn block_tax_for(&self, blocker: CardId) -> (u32, u32) {
        let (mut mana, mut life) = (self.block_tax_this_turn, 0u32);
        for c in &self.battlefield {
            for sa in &c.definition.static_abilities {
                if let crate::effect::StaticEffect::BlockTaxToController {
                    amount,
                    only_while_attacking,
                    filter,
                    life: as_life,
                } = &sa.effect
                {
                    if *only_while_attacking
                        && !self.attacking.iter().any(|a| a.attacker == c.id)
                    {
                        continue;
                    }
                    if !filter.as_ref().is_none_or(|f| {
                        self.evaluate_requirement_static(
                            f,
                            &crate::game::types::Target::Permanent(blocker),
                            c.controller,
                            Some(c.id),
                        )
                    }) {
                        continue;
                    }
                    let mut ctx =
                        crate::game::effects::EffectContext::for_spell(c.controller, None, 0, 0);
                    ctx.source = Some(c.id);
                    let n = self.evaluate_value(amount, &ctx).max(0) as u32;
                    if *as_life { life += n } else { mana += n }
                }
            }
        }
        (mana, life)
    }

    /// Does any permanent on this board charge a block tax at all? One walk,
    /// so [`block_tax_for`](Self::block_tax_for)'s per-blocker walk is only
    /// paid on the boards that have one.
    pub(crate) fn block_tax_present(&self) -> bool {
        self.block_tax_this_turn > 0
            || self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::BlockTaxToController { .. }
                    )
                })
            })
    }

    /// CR 509.1a/b — everything that bars `blocker` from blocking *at all*,
    /// as `(site, error)`, or `None` when nothing does.
    ///
    /// One walker for the four callers that each kept their own list:
    /// [`declare_blockers`](Self::declare_blockers)'s per-assignment gate,
    /// `blocker_can_block_anything` (what the bot's planner and the client's
    /// legal-block list read), CR 509.1b's "must block if able" loops, and CR
    /// 702.39's Provoke. **They had drifted in both directions and each
    /// direction is its own bug.** The gate did not enforce seven
    /// "can't block unless …" families at all — hand size, delirium, a
    /// creature died this turn, Descend N, the city's blessing, cards in
    /// exile, Hollow Warrior's helper — so those cards' restrictions did
    /// nothing on the real declaration path. And the mirror did not know
    /// about detain, a blanket "can't block this turn", or Void Winnower, so
    /// a requirement loop could oblige a block the gate then rejected and
    /// leave the defending seat with **no legal declaration in either
    /// direction** (`cr_recent100`).
    ///
    /// Keyword families are asked in one walk over the computed set rather
    /// than a `has_kw` scan apiece, the same way `attacker_self_block` does.
    pub(crate) fn blocker_self_block(
        &self,
        blocker: &crate::card::CardInstance,
        cp: Option<&ComputedPermanent>,
    ) -> Option<(u32, GameError)> {
        let no = |line: u32| Some((line, GameError::CannotBlock(blocker.id)));
        // CR 509.1a — creature-ness from the computed view, so an animated
        // land or crewed Vehicle can block and an uncrewed Vehicle can't.
        let Some(cp) = cp else { return no(line!()) };
        if !cp.card_types().contains(&crate::card::CardType::Creature) {
            return no(line!());
        }
        if blocker.tapped && !self.tapped_creatures_can_block(blocker.controller) {
            return no(line!());
        }
        // CR 701.35 — a detained permanent can't block.
        if blocker.detained_by.is_some() {
            return no(line!());
        }
        // CR 509.1b — a blanket "can't block this turn" (Concussive Bolt's
        // metalcraft rider).
        if self.cant_block_this_turn.contains(&blocker.id) {
            return no(line!());
        }
        let owner = blocker.controller;
        // Void Winnower — an even mana value can't block while an opponent
        // has the static (zero is even). Both cheap terms first: the board
        // question is a presence gate (`block_even_mv_lock_in_scope`), so
        // inside the planner's freeze scope it is one load rather than the
        // whole-battlefield walk this used to take on every even blocker.
        // Only a board that actually plays the card reaches the seat walk.
        if blocker.definition.cost.cmc().is_multiple_of(2)
            && self.block_even_mv_lock_in_scope()
            && self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::OpponentsCantBlockWithEvenMv)
                }) && !self.same_team(c.controller, owner)
            })
        {
            return no(line!());
        }
        // One walk. Everything below reads only the blocker's own computed
        // keywords and the board, never the attacker.
        let mut control_count: Option<(crate::card::SelectionRequirement, u32, bool)> = None;
        let mut tap_another: Option<crate::card::SelectionRequirement> = None;
        for k in cp.keywords().iter() {
            let barred = match k {
                Keyword::CantBlock | Keyword::Decayed => true,
                Keyword::CantAttackOrBlockUnlessEvenCounters => {
                    blocker.counters.values().sum::<u32>() % 2 != 0
                }
                Keyword::CantAttackOrBlockUnlessHandSizeAtMost(n) => {
                    self.players[owner].hand.len() as u32 > *n
                }
                Keyword::CantAttackOrBlockUnlessDelirium => !self.delirium_active(owner),
                Keyword::CantAttackOrBlockUnlessCreatureDiedThisTurn => {
                    self.players[owner].creatures_died_this_turn == 0
                }
                Keyword::CantAttackOrBlockUnlessDescend(n) => {
                    self.descend_count(owner) < *n as usize
                }
                Keyword::CantAttackOrBlockUnlessCityBlessing => !self.players[owner].city_blessing,
                Keyword::CantAttackOrBlockUnlessCardsInExile(n) => (self.exile.len() as u32) < *n,
                // Branded Brawlers — your own untapped land locks the block.
                Keyword::CantBlockIfYouHaveUntappedLand => self
                    .battlefield
                    .iter()
                    .any(|c| c.controller == owner && c.definition.is_land() && !c.tapped),
                // Deferred: both need a battlefield walk of their own, and
                // only one of each can bind.
                Keyword::CantAttackOrBlockUnlessYouControlCount {
                    filter,
                    min,
                    attack_only: false,
                    exclude_self,
                    ..
                } => {
                    control_count.get_or_insert(((**filter).clone(), *min, *exclude_self));
                    false
                }
                Keyword::AttackBlockCostTapAnother(f) => {
                    tap_another.get_or_insert((**f).clone());
                    false
                }
                _ => false,
            };
            if barred {
                return no(line!());
            }
        }
        // CR 509.1b — "can't block unless you control N+ [filter]" (Topiary
        // Stomper). An attack-only gate (Lambholt Pacifist) never restricts
        // blocking, which is why the match arm above pins `attack_only`.
        if let Some((req, min, excl)) = control_count {
            let n = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == owner
                        && !(excl && c.id == blocker.id)
                        && self.evaluate_requirement_on_card(&req, c, owner)
                })
                .count();
            if (n as u32) < min {
                return no(line!());
            }
        }
        // CR 509.1b — Hollow Warrior: a spare untapped match must exist to
        // tap, and it can be neither the blocker nor a declared attacker.
        if let Some(f) = tap_another
            && !self.battlefield.iter().any(|c| {
                c.controller == owner
                    && !c.tapped
                    && c.id != blocker.id
                    && !self.attacking.iter().any(|a| a.attacker == c.id)
                    && self.evaluate_requirement_on_card(&f, c, owner)
            })
        {
            return no(line!());
        }
        // CR 508.1g's block half — the "unless its controller pays {N}"
        // family is legal only if the seat can actually produce the tax. The
        // declaration charges the *sum* over blockers later; one blocker's
        // own tax being unpayable makes that sum unpayable too, so asking
        // here is sound and names the blocker instead of `assignments[0]`.
        let tax = self.attack_block_keyword_tax(blocker.id, cp.keywords(), false);
        if tax > 0 && !self.could_pay_generic(owner, tax) {
            return no(line!());
        }
        None
    }

    /// Is `b` **able** to block `required`, for the purpose of a CR 509.1
    /// requirement that says it must?
    ///
    /// [`declare_blockers`](Self::declare_blockers) rejects the **whole**
    /// declaration when a creature a requirement obliges is able and not
    /// assigned, so the bot's block planner has to answer this identically or
    /// it loses its entire block step. One method, called from both.
    ///
    /// **Four requirements ask this question and each used to answer it
    /// itself:** CR 702.39 Provoke (`must_block`), CR 509.1c "must be blocked
    /// if able" (`MustBeBlocked`), CR 509.1c true Lure (`AllMustBlock`), and
    /// CR 509.1c "blocks each combat if able" (`MustBlock`, asked of the
    /// blocker against every attacker). Four hand-written copies of one
    /// conjunction is the shape that generated most of ENGINE_BACKLOG P3, and
    /// three of them had already drifted from this one on the tapped term —
    /// they refused a tapped blocker outright where this respects
    /// `tapped_creatures_can_block`. The escape is the correct reading (a
    /// creature that *can* block is able to), so unifying takes this one's
    /// version; it binds requirements slightly more often, on the one static
    /// that grants it.
    ///
    /// Reads `computed_permanent` rather than the declaration's gated subset:
    /// every caller is behind a keyword or field test that is false on a
    /// board without the mechanic, so on an ordinary board it costs nothing.
    pub(crate) fn block_requirement_able(
        &self,
        b: &crate::card::CardInstance,
        required: CardId,
    ) -> bool {
        let Some(bcp) = self.computed_permanent(b.id) else { return false };
        let Some(attacker) = self.battlefield_find(required) else { return false };
        self.blocker_self_block(b, Some(&bcp)).is_none()
            && self
                .blocker_pair_block(
                    b,
                    &bcp,
                    attacker,
                    self.computed_permanent(required).as_deref(),
                    b.controller,
                )
                .is_none()
    }

    /// CR 509.1c — is `b` already **spoken for** by a different block
    /// requirement, and therefore not idle for this one?
    ///
    /// The rule the five requirement loops implement one at a time is
    /// "satisfy the *maximum number* of requirements without violating a
    /// restriction". Checked independently they demand more than that:
    /// a creature can block only one attacker, so two requirements that both
    /// name it can never both be met, and asking each in isolation makes
    /// **every** declaration illegal. A Lure attacker plus a provoker plus
    /// one able defender had exactly that shape — block nobody, block the
    /// Lure, block the provoker, all three rejected — and it is `cube` seed
    /// 15's whole residual.
    ///
    /// A creature that satisfies one binding requirement is the most any
    /// declaration can get out of it, so a declaration in which every obliged
    /// creature blocks *something that obliges it* is already maximal. That
    /// is what this tests, and it is the cheap half of the general rule: the
    /// blocker is assigned to some attacker other than `except`, and that
    /// attacker's own requirement binds this blocker.
    ///
    /// Full CR 509.1c maximization over arbitrary requirement sets is still
    /// an approximation here — see the loops' own note.
    pub(crate) fn block_spoken_for_elsewhere(
        &self,
        b: &crate::card::CardInstance,
        except: CardId,
        assignments: &[(CardId, CardId)],
    ) -> bool {
        let claims = |other: CardId| -> bool {
            if other == except {
                return false;
            }
            if b.must_block == Some(other) {
                return true;
            }
            let Some(acp) = self.computed_permanent(other) else { return false };
            (acp.keywords().has_kw(&Keyword::AllMustBlock)
                || acp.keywords().has_kw(&Keyword::MustBeBlocked))
                && self.block_requirement_able(b, other)
        };
        assignments.iter().any(|(bid, aid)| *bid == b.id && claims(*aid))
            || self.block_map.get(&b.id).is_some_and(|v| v.iter().copied().any(claims))
    }

    /// CR 509.1c — does a "must block" requirement on `attacker` **bind** at
    /// all? A restriction outranks a requirement: the defending player picks
    /// the legal declaration that satisfies the most requirements, and a
    /// declaration that breaks CR 509.1b is not one of the candidates.
    ///
    /// **Without this the engine can demand a declaration it also forbids.**
    /// An attacker with Lure *and* Menace facing one able blocker has no
    /// legal block at all: declare nobody and CR 509.1c rejects it, declare
    /// the one body and the Menace count rejects it. That contradiction is
    /// reachable in the routine pools — an aura granting `AllMustBlock`
    /// landing on a Menace creature, `cube` seed 15 — and no planner can
    /// plan around it, because there is nothing to plan.
    ///
    /// The able set is counted, not the assigned one: the question is whether
    /// a legal block *exists*, not whether this declaration made it.
    pub(crate) fn block_requirement_binds(&self, attacker: CardId) -> bool {
        let Some(acp) = self.computed_permanent(attacker) else { return true };
        let mut min_b = if acp.keywords().has_kw(&Keyword::Menace) { 2usize } else { 1 };
        for kw in acp.keywords().iter() {
            if let Keyword::CantBeBlockedExceptByN(n) = kw {
                min_b = min_b.max(*n as usize);
            }
        }
        if min_b <= 1 {
            return true;
        }
        let Some(defender_idx) = self.attacking.iter().find(|a| a.attacker == attacker).and_then(
            |a| self.defender_for(a.target),
        ) else {
            return true;
        };
        let able = self
            .battlefield
            .iter()
            .filter(|b| {
                self.same_team(b.controller, defender_idx)
                    && self.block_requirement_able(b, attacker)
            })
            .count();
        able.max(self.blocker_count_of(attacker)) >= min_b
    }

    /// CR 508.1g — the generic mana a declaration of `attacks` costs its
    /// controller: the defenders' `AttackTaxToController` statics (Ghostly
    /// Prison, Propaganda, Sphere of Safety, Elephant Grass), the two
    /// turn-scoped taxes (War Tax, Forbidding Spirit), and each attacker's
    /// own "can't attack unless you pay {N}" keyword through `keyword_tax`.
    ///
    /// **Additive per attacker, with no cross terms** — which is what makes
    /// it usable by both callers. [`declare_attackers_banded`] pays it and
    /// rejects the declaration *whole* when it can't; the bot's attack
    /// picker calls the same function to trim the batch down to what it can
    /// pay, and the monotonicity is why trimming terminates. Until the
    /// eightieth pass the picker did not model the tax at all: against a
    /// Propaganda it declared the board, could not pay, and lost its entire
    /// combat to a batch rejection that blamed `attacks[0]`. PERF (-55).
    ///
    /// `keyword_tax` is injected rather than read here because the two
    /// callers hold the computed keyword set differently — the engine has a
    /// gated `Vec<ComputedPermanent>` in scope, the picker an `Arc` per
    /// lookup — and re-deriving it inside would cost the engine its memo.
    ///
    /// [`declare_attackers_banded`]: Self::declare_attackers_banded
    pub(crate) fn attack_tax_for(
        &self,
        attacks: &[Attack],
        statics: u32,
        keyword_tax: impl Fn(CardId) -> u32,
    ) -> u32 {
        debug_assert!(
            statics & attack_static::ATTACK_TAX != 0
                || !self.battlefield.iter().any(|c| c
                    .definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(
                        sa.effect,
                        crate::effect::StaticEffect::AttackTaxToController { .. }
                    ))),
            "attack_static_scan missed an attack tax",
        );
        let mut total_tax = 0u32;
        for atk in attacks {
            // The defending player whose statics apply, and whether the attack
            // is aimed at a planeswalker (so `protect_planeswalkers` gates it).
            let (defender, at_planeswalker) = match atk.target {
                crate::game::types::AttackTarget::Player(d) => (Some(d), false),
                crate::game::types::AttackTarget::Planeswalker(pw) => {
                    (self.battlefield_find(pw).map(|c| c.controller), true)
                }
                // The protector's attack taxes apply when attacking their
                // battle; not a planeswalker for `protect_planeswalkers`.
                crate::game::types::AttackTarget::Battle(b) => {
                    (self.battlefield_find(b).and_then(|c| c.protected_by), false)
                }
            };
            let Some(d) = defender else { continue };
            // Forbidding Spirit — a temporary Propaganda tax on the defender
            // that also protects their planeswalkers.
            total_tax += self.players[d].attack_tax_until_your_turn;
            // War Tax — a symmetric per-attacker tax for the rest of the turn.
            total_tax += self.attack_tax_this_turn;
            // Evaluate each tax `amount` with the defender as "you" (and the
            // tax permanent as source) so dynamic taxes — Sphere of Safety's
            // "number of enchantments you control" — count the defender's
            // board. Fixed taxes are `Value::Const(n)`.
            for c in &self.battlefield {
                if statics & attack_static::ATTACK_TAX == 0 {
                    break; // no tax permanent anywhere — the loop body is a no-op
                }
                if c.controller != d {
                    continue;
                }
                for sa in &c.definition.static_abilities {
                    if let crate::effect::StaticEffect::AttackTaxToController {
                        amount,
                        protect_planeswalkers,
                        filter,
                    } = &sa.effect
                        && (!at_planeswalker || *protect_planeswalkers)
                        // Elephant Grass — the tax only bites on matching
                        // attackers, and is charged once per such attacker.
                        && filter.as_ref().is_none_or(|f| {
                            self.evaluate_requirement_static(
                                f,
                                &crate::game::types::Target::Permanent(atk.attacker),
                                d,
                                Some(c.id),
                            )
                        })
                    {
                        let mut ctx = crate::game::effects::EffectContext::for_spell(d, None, 0, 0);
                        ctx.source = Some(c.id);
                        total_tax += self.evaluate_value(amount, &ctx).max(0) as u32;
                    }
                }
            }
        }
        // CR 508.1g — per-attacker "can't attack unless its controller pays
        // {N}" (Oppressive Rays). Same pool as the Propaganda tax above.
        for atk in attacks {
            total_tax += keyword_tax(atk.attacker);
        }
        total_tax
    }

    /// CR 508.1a / 509.1a — the "can't attack or block unless its controller
    /// pays {N}" tax carried by `id`'s own computed keywords. Shared by the
    /// declare-attackers and declare-blockers payment loops.
    pub fn attack_block_keyword_tax(
        &self,
        id: CardId,
        keywords: &[Keyword],
        for_attack: bool,
    ) -> u32 {
        keywords
            .iter()
            .map(|k| match k {
                Keyword::CantAttackOrBlockUnlessPay(n) => *n,
                // Brainwash — attack-only, so it's excluded at the block site.
                Keyword::CantAttackUnlessPay(n) if for_attack => *n,
                // Myr Prototype — the tax is its own counter count.
                Keyword::CantAttackOrBlockUnlessPayPerCounter(kind) => {
                    self.battlefield_find(id).map(|c| c.counter_count(*kind)).unwrap_or(0)
                }
                // Cowed by Wisdom — one per card in each granting Aura's
                // controller's hand.
                Keyword::CantAttackOrBlockUnlessPayPerCardInEnchanterHand => self
                    .battlefield
                    .iter()
                    .filter(|a| a.attached_to == Some(id))
                    .filter(|a| {
                        a.definition.equipped_bonus.as_ref().is_some_and(|b| {
                            b.keywords.contains(
                                &Keyword::CantAttackOrBlockUnlessPayPerCardInEnchanterHand,
                            )
                        })
                    })
                    .map(|a| self.players[a.controller].hand.len() as u32)
                    .sum(),
                // Whipgrass Entangler — one per matching permanent anywhere.
                Keyword::CantAttackOrBlockUnlessPayPerPermanent(filter) => self
                    .battlefield
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(filter, c, c.controller))
                    .count() as u32,
                _ => 0,
            })
            .sum()
    }

    /// Push triggered abilities of `source` whose event spec is
    /// `DealsCombatDamageToPlayer` onto the stack, with `damaged_player`
    /// stored as the trigger's target so the effect can refer to "that
    /// player" via `PlayerRef::Target(0)`.
    ///
    /// Phase 1 walks the battlefield for the attacker's own
    /// `SelfSource` / `AnyPlayer` triggers (the printed
    /// "whenever this creature deals combat damage" pattern).
    ///
    /// Phase 2 walks every player's graveyard for `FromYourGraveyard`
    /// triggers whose controller (the gy owner) matches the source's
    /// controller — the "whenever your creatures deal combat damage,
    /// return this card from your graveyard" pattern used by Killian's
    /// Confidence and friends. The trigger source is bound to the
    /// graveyard card itself so a `Move(SelfSource → Hand)` body
    /// returns the right card.
    pub fn fire_combat_damage_to_player_triggers(
        &mut self,
        source: CardId,
        damaged_player: usize,
        damage_amount: u32,
    ) {
        // CR 702.179 — the dealing creature's controller has now dealt combat
        // damage to a player this turn (Freerunning's alt-cost gate).
        if let Some(c) = self.battlefield_find(source) {
            let controller = c.controller;
            self.players[controller].dealt_combat_damage_to_player_this_turn = true;
            // CR 603.4 — turn-scoped "whenever a creature you control deals
            // combat damage to a player" delayed triggers (Mistway Spy).
            let watchers: Vec<crate::game::types::DelayedTrigger> = self
                .delayed_triggers
                .iter()
                .filter(|dt| {
                    dt.controller == controller
                        && matches!(
                            dt.kind,
                            crate::game::types::DelayedKind::CreatureYouControlDealsCombatDamageThisTurn
                        )
                })
                .cloned()
                .collect();
            for dt in watchers {
                self.stack.push(
                    TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                        .trigger_source(Some(crate::game::effects::EntityRef::Permanent(source)))
                        .event_amount(damage_amount)
                        .build(),
                );
            }
        }
        self.fire_source_dealt_damage_watchers(source, damage_amount);
        self.fire_source_combat_damage_to_player_watchers(source, damage_amount);
        let granted = self.static_granted_triggers_of(source);
        // Combat damage is damage: the combat-agnostic wordings fire too, in
        // this order (CR 603.2 — one batch, grouped per kind).
        self.fire_combat_damage_triggers(
            source,
            &[
                EventKind::DealsCombatDamageToPlayer,
                EventKind::DealsDamageToPlayer,
                EventKind::DealsCombatDamage,
                EventKind::DealsDamage,
            ],
            Target::Player(damaged_player),
            damage_amount,
            &granted,
        );
        // CR 510 — "whenever combat damage is dealt to you" listeners fire off
        // the *recipient's* own permanents (SelfSource on a permanent the
        // damaged player controls). Risona sheds an indestructible counter.
        // Plain loops: a whole-board walk per damage event, and `FlatMap::next`
        // costs ~20 Ir a permanent before the filter sees a single ability
        // (PERF (-78)).
        let mut listeners: Vec<(CardId, Effect, usize)> = Vec::new();
        for c in self.battlefield.iter() {
            if c.controller != damaged_player {
                continue;
            }
            for ta in &c.definition.triggered_abilities {
                if ta.event.kind == EventKind::ControllerDealtCombatDamage
                    && ta.event.scope == crate::effect::EventScope::SelfSource
                {
                    listeners.push((c.id, ta.effect.clone(), c.controller));
                }
            }
        }
        for (listener, effect, controller) in listeners {
            let auto_target = self.auto_target_for_effect_avoiding(&effect, controller, Some(listener));
            // Bind the creature that dealt the damage as `Selector::TriggerSource`
            // so "whenever a creature deals combat damage to you, destroy it"
            // clauses can reference the dealer (Teysa, Envoy of Ghosts).
            self.stack.push(
                TriggerPush::new(listener, controller, effect)
                    .target(auto_target)
                    .trigger_source(Some(crate::game::effects::EntityRef::Permanent(source)))
                    .event_amount(damage_amount)
                    .build(),
            );
        }
    }

    /// CR 603.4 — fire any `SourceDealsDamageThisTurn` delayed triggers
    /// watching `source` (Paladin of Prahv's Forecast rider). Called from every
    /// damage-delivery path — combat to a player/creature and noncombat — so the
    /// watcher sees *any* damage the creature deals. The amount rides in via
    /// `Value::TriggerEventAmount`.
    pub(crate) fn fire_source_dealt_damage_watchers(&mut self, source: CardId, amount: u32) {
        if amount == 0 {
            return;
        }
        let watchers: Vec<crate::game::types::DelayedTrigger> = self
            .delayed_triggers
            .iter()
            .filter(|dt| {
                matches!(dt.kind, crate::game::types::DelayedKind::SourceDealsDamageThisTurn(id) if id == source)
            })
            .cloned()
            .collect();
        for dt in watchers {
            self.stack.push(
                TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                    .trigger_source(Some(crate::game::effects::EntityRef::Permanent(source)))
                    .event_amount(amount)
                    .build(),
            );
        }
    }

    /// CR 603.4 — fire any `SourceDealsCombatDamageToPlayerThisTurn` delayed
    /// triggers watching `source` (Captain Howler's pumped creature).
    pub(crate) fn fire_source_combat_damage_to_player_watchers(
        &mut self,
        source: CardId,
        amount: u32,
    ) {
        if amount == 0 {
            return;
        }
        let watchers: Vec<crate::game::types::DelayedTrigger> = self
            .delayed_triggers
            .iter()
            .filter(|dt| {
                matches!(
                    dt.kind,
                    crate::game::types::DelayedKind::SourceDealsCombatDamageToPlayerThisTurn(id)
                        if id == source
                )
            })
            .cloned()
            .collect();
        for dt in watchers {
            self.stack.push(
                TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                    .trigger_source(Some(crate::game::effects::EntityRef::Permanent(source)))
                    .event_amount(amount)
                    .build(),
            );
        }
    }

    /// CR 603.4 — fire any `YouGainLifeThisTurn` delayed triggers whose
    /// controller is `player` (Vizkopa Guildmage's "whenever you gain life,
    /// each opponent loses that much"). The gained amount rides in via
    /// `Value::TriggerEventAmount`.
    pub(crate) fn fire_life_gained_watchers(&mut self, player: usize, amount: u32) {
        if amount == 0 {
            return;
        }
        let watchers: Vec<crate::game::types::DelayedTrigger> = self
            .delayed_triggers
            .iter()
            .filter(|dt| {
                dt.controller == player
                    && matches!(dt.kind, crate::game::types::DelayedKind::YouGainLifeThisTurn)
            })
            .cloned()
            .collect();
        for dt in watchers {
            self.stack.push(
                TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                    .event_amount(amount)
                    .build(),
            );
        }
    }

    /// CR 603.4 — fire `CardEntersOpponentGraveyardThisTurn` delayed triggers
    /// (Duskmantle Guildmage) when a card is put into `owner`'s graveyard, for
    /// each watcher that treats `owner` as an opponent. The owner is bound as
    /// the body's `Target(0)` so "that player loses 1 life" is exact.
    pub(crate) fn fire_opponent_graveyard_watchers(&mut self, owner: usize) {
        let watchers: Vec<crate::game::types::DelayedTrigger> = self
            .delayed_triggers
            .iter()
            .filter(|dt| {
                matches!(
                    dt.kind,
                    crate::game::types::DelayedKind::CardEntersOpponentGraveyardThisTurn
                ) && self.opponents_of(dt.controller).contains(&owner)
            })
            .cloned()
            .collect();
        for dt in watchers {
            self.stack.push(
                TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                    .target(Some(Target::Player(owner)))
                    .build(),
            );
        }
    }

    /// Push triggered abilities of `source` whose event spec is
    /// `DealsCombatDamageToCreature` onto the stack, binding the damaged
    /// creature to the trigger's target so "destroy / exile / -1/-1 that
    /// creature" payoffs and equipment charge-triggers (Umezawa's Jitte)
    /// resolve correctly. CR 510.2. Fires once per (source, damaged-creature)
    /// pair; an equipped creature blocked by several creatures therefore
    /// charges Jitte once per blocker (a minor over-count for the rare
    /// multi-block case).
    /// `granted` is `source`'s `GrantTriggeredAbility` set, built by the
    /// caller: the CR 510.2 batch fires this once per damaged creature and
    /// deriving it here rebuilt the whole-board grant list every time.
    pub(crate) fn fire_combat_damage_to_creature_triggers(
        &mut self,
        source: CardId,
        damaged_creature: CardId,
        damage_amount: u32,
        granted: &[crate::card::TriggeredAbility],
    ) {
        // Combat damage is damage: the combat-agnostic wordings fire too.
        self.fire_combat_damage_triggers(
            source,
            &[
                EventKind::DealsCombatDamageToCreature,
                EventKind::DealsDamageToCreature,
                EventKind::DealsCombatDamage,
                EventKind::DealsDamage,
            ],
            Target::Permanent(damaged_creature),
            damage_amount,
            granted,
        );
    }

    /// The non-combat half of `EventKind::DealsDamageToCreature`: a permanent
    /// source that just dealt non-combat damage to a creature fires the same
    /// source-scoped triggers, with the damaged creature bound to slot 0.
    pub(crate) fn fire_noncombat_damage_to_creature_triggers(
        &mut self,
        source: CardId,
        damaged_creature: CardId,
        damage_amount: u32,
    ) {
        if self.battlefield_find(source).is_none() {
            return;
        }
        let granted = self.static_granted_triggers_of(source);
        self.fire_combat_damage_triggers(
            source,
            &[EventKind::DealsDamageToCreature, EventKind::DealsDamage],
            Target::Permanent(damaged_creature),
            damage_amount,
            &granted,
        );
    }

    /// The non-combat, player-side half of `EventKind::DealsDamage` /
    /// `DealsDamageToPlayer`: a permanent that just burned a player fires its
    /// own damage triggers with that player bound to slot 0.
    pub(crate) fn fire_noncombat_damage_to_player_triggers(
        &mut self,
        source: CardId,
        damaged_player: usize,
        damage_amount: u32,
    ) {
        if self.battlefield_find(source).is_none() {
            return;
        }
        let granted = self.static_granted_triggers_of(source);
        self.fire_combat_damage_triggers(
            source,
            &[EventKind::DealsDamageToPlayer, EventKind::DealsDamage],
            Target::Player(damaged_player),
            damage_amount,
            &granted,
        );
    }

    /// Shared body for the combat-damage trigger dispatch (to a player or to a
    /// creature). Walks the attacker's printed `SelfSource`/`AnyPlayer`
    /// triggers, equipment- and soulbond-granted triggers (CR 702.6e / 702.95),
    /// `YourControl`-scope listeners, and `FromYourGraveyard` triggers, pushing
    /// each onto the stack with `default_target` bound to slot 0.
    /// The `GrantTriggeredAbility` set a battlefield permanent currently
    /// carries — empty when it has left. Rebuilds the whole-board grant list,
    /// so a caller firing a *batch* hoists `trigger_grant_sources` itself and
    /// uses `statics_granted_triggers_with` (see the CR 510.2 loop).
    fn static_granted_triggers_of(&self, source: CardId) -> Vec<crate::card::TriggeredAbility> {
        self.battlefield
            .iter()
            .find(|c| c.id == source)
            .map(|c| self.statics_granted_triggers_for(c))
            .unwrap_or_default()
    }

    /// `static_granted` is the source's `GrantTriggeredAbility` set, which is
    /// board-level and *kind-independent* — the callers below fire four event
    /// kinds per damage event, so they build it once and pass it in rather
    /// than making each call rebuild `trigger_grant_sources` (a whole-board
    /// scan) for the same permanent.
    ///
    /// `kinds` is the caller's whole kind list for one damage event, for the
    /// same reason: every walk below except the graveyard one is
    /// kind-independent, so a per-kind call re-walked the battlefield four
    /// times for equipment, auras, soulbond and the `YourControl` /
    /// `AnyPlayer` dealer listeners. The pushes stay grouped per kind, in
    /// `kinds` order, so the stack sees exactly the per-kind batches it did.
    fn fire_combat_damage_triggers(
        &mut self,
        source: CardId,
        kinds: &[EventKind],
        default_target: Target,
        damage_amount: u32,
        static_granted: &[crate::card::TriggeredAbility],
    ) {
        // One [`DamageTrigger`] bucket per requested kind; drained in order at
        // the bottom.
        let slot = |k: &EventKind| kinds.iter().position(|want| want == k);
        // Inline, by (-71)'s device: `kinds` is one to three entries and the
        // outer `Vec` allocated on **every** call whether or not a trigger
        // fires — 6,480 / 20,022 / 22,134 allocations over six bench games.
        // The inner buckets stay `Vec`, and `Vec::new()` does not allocate
        // until something is pushed into it, which on most boards is never.
        let mut by_kind: SmallVec<[Vec<DamageTrigger>; 4]> =
            kinds.iter().map(|_| Vec::new()).collect();

        // One lookup of the dealer, not two: the controller Phase 1b onward
        // needs is a field of the card Phase 1 walks the battlefield to find.
        //
        // The `find` short-circuited, and the three attachment phases below
        // each walked the whole battlefield to learn there was nothing
        // attached to the dealer and no soulbond pair touching it. This walk
        // does not short-circuit and answers both, so three walks become one
        // that was already half paid for (PERF (-68)).
        let mut attacker_controller = None;
        let mut dealer: Option<&crate::card::CardInstance> = None;
        let mut any_attached = false;
        let mut soulbond_pair = false;
        for c in self.battlefield.iter() {
            if c.id == source {
                dealer = Some(c);
            }
            any_attached |= c.attached_to == Some(source);
            soulbond_pair |= c.definition.soulbond_bonus.is_some()
                && c.soulbond_partner.is_some()
                && (c.id == source || c.soulbond_partner == Some(source));
        }
        if let Some(c) = dealer {
            attacker_controller = Some(c.controller);
            // Printed + statics-granted ("Slivers you control have
            // '…combat damage…'" — Tempered/Virulent) + instance-granted
            // (`GrantTriggeredAbility` on `granted_triggers_eot` — Summon:
            // Primal Odin's Zantetsuken) fire alike.
            let instance_granted: &[crate::card::TriggeredAbility] =
                self.granted_triggers_eot.get(&c.id).map(Vec::as_slice).unwrap_or(&[]);
            for t in c
                .definition
                .triggered_abilities
                .iter()
                .chain(static_granted.iter())
                .chain(instance_granted.iter())
            {
                if !matches!(
                    t.event.scope,
                    crate::effect::EventScope::SelfSource | crate::effect::EventScope::AnyPlayer
                ) {
                    continue;
                }
                if let Some(i) = slot(&t.event.kind) {
                    by_kind[i].push((
                        c.id,
                        t.effect.clone(),
                        c.controller,
                        t.event.filter.clone(),
                        false,
                    ));
                }
            }
        }

        // Phase 1b: equipment-granted combat-damage triggers (CR 702.6e). Each
        // Equipment attached to the attacker grants its `equipped_bonus.
        // triggered_abilities` to the creature; a `DealsCombatDamageToPlayer`
        // one fires here (the Sword cycle's "create a token / mill / draw"
        // riders), bound to the attacker's controller.
        if let Some(atk_ctrl) = attacker_controller.filter(|_| any_attached) {
            for eq in &self.battlefield {
                if eq.attached_to != Some(source) {
                    continue;
                }
                let Some(bonus) = &eq.definition.equipped_bonus else { continue };
                // CR 702.6e — the granted ability fires off the creature, unless
                // the Equipment opts to fire off itself (Umezawa's Jitte puts the
                // counters on the Equipment, so `Selector::This` must read it).
                let trig_source = if bonus.triggers_on_equipment { eq.id } else { source };
                for t in &bonus.triggered_abilities {
                    if matches!(
                        t.event.scope,
                        crate::effect::EventScope::SelfSource
                            | crate::effect::EventScope::AnyPlayer
                    ) && let Some(i) = slot(&t.event.kind)
                    {
                        by_kind[i].push((
                            trig_source,
                            t.effect.clone(),
                            atk_ctrl,
                            t.event.filter.clone(),
                            false,
                        ));
                    }
                }
            }
            // Auras on the attacker with an `EnchantedBySource` combat-damage
            // trigger ("whenever enchanted creature deals combat damage to a
            // player" — Pollenbright Wings). The trigger fires off the *Aura*,
            // so `Selector::AttachedTo(This)` reaches the host.
            for aura in &self.battlefield {
                if aura.attached_to != Some(source) || !aura.definition.is_enchantment() {
                    continue;
                }
                for t in &aura.definition.triggered_abilities {
                    if t.event.scope == crate::effect::EventScope::EnchantedBySource
                        && let Some(i) = slot(&t.event.kind)
                    {
                        by_kind[i].push((
                            aura.id,
                            t.effect.clone(),
                            aura.controller,
                            t.event.filter.clone(),
                            false,
                        ));
                    }
                }
            }
        }

        // CR 702.95 — Soulbond-granted combat-damage triggers. A paired
        // creature carrying `soulbond_bonus.triggered_abilities` grants them
        // to BOTH members; a `DealsCombatDamageToPlayer` one fires off the
        // attacker (Tandem Lookout's "deals combat damage → draw"). Gated on
        // the dealer walk's `soulbond_pair`, which is the loop's own
        // `src.id != source && partner != source` test hoisted.
        if let Some(atk_ctrl) = attacker_controller.filter(|_| soulbond_pair) {
            for src in &self.battlefield {
                let Some(bonus) = &src.definition.soulbond_bonus else { continue };
                let Some(partner) = src.soulbond_partner else { continue };
                if src.id != source && partner != source {
                    continue;
                }
                if !self.battlefield.iter().any(|c| c.id == partner) {
                    continue;
                }
                for t in &bonus.triggered_abilities {
                    if let Some(i) = slot(&t.event.kind) {
                        by_kind[i].push((
                            source,
                            t.effect.clone(),
                            atk_ctrl,
                            t.event.filter.clone(),
                            false,
                        ));
                    }
                }
            }
        }

        // Phases 1.5 and 1.6, in one walk of the battlefield and one walk of
        // each permanent's printed trigger list.
        //
        // **1.5** — `YourControl`-scope listeners: "whenever a creature you
        // control deals combat damage to a player" (Quandrix Echocrasher
        // b171, Enduring Curiosity). The listener's controller must match the
        // attacker's. The source itself counts ("a creature you control"
        // includes the dealer) — its `YourControl` trigger isn't gathered in
        // Phase 1, which is `SelfSource`-only, so it would otherwise never
        // fire.
        //
        // **1.6** — `AnyPlayer`-scope listeners on *other* permanents:
        // "whenever a Goblin deals combat damage to a player" (Cabal Slaver),
        // which cares about the dealer's characteristics rather than its
        // controller. `EventSpec.dealer_filter` gates on the dealing creature;
        // the dealer's own `AnyPlayer` trigger already fired in Phase 1.
        //
        // The two used to be separate walks. Fusing them has to keep every
        // 1.5 push ahead of every 1.6 push inside a `by_kind` bucket — the
        // buckets reach the stack in order — so 1.6's hits go through
        // `any_player`, which is `Vec::new()` and therefore allocation-free
        // on every board without such a listener, and is drained after
        // (PERF (-68)).
        //
        // A plain nested loop, not the `filter`/`flat_map`/`filter`/`map`
        // chain 1.6 used to `collect()` into a `Vec`: nothing in it borrows
        // `self` mutably, so there was never anything to buffer, and the
        // adapter stack was **3,671,940 Ir / 0.30 % of a six-game `--decks
        // fixed` run** in `Map::try_fold` alone (fifty-ninth pass).
        // The per-permanent trigger walk is a word load on every permanent
        // that carries neither scope — `card::dispatch_bits::LISTENER`, the
        // same device `dispatch_board_scan` uses for its four facts. Unlike
        // the layer-4 gates, the list this replaces is *not* usually empty
        // (a cube board's creatures carry printed triggers), which is what
        // (-77)'s rule says a presence bit needs.
        //
        // And a board where *no* permanent carries either scope skips the walk
        // outright: the zone's listener lane holds that answer across every
        // combat-damage dispatch until membership or a definition moves. The
        // lane is filled from this walk — which loads the same memo word per
        // card and cannot short-circuit, so the fill is both free and exact
        // (see `zone::Battlefield::listener_lane`).
        let mut any_player: Vec<(usize, DamageTrigger)> = Vec::new();
        let lane = self.battlefield.listener_lane();
        if lane != Ok(false) {
            let mut any_listener = false;
            for c in &self.battlefield {
                if c.dispatch_scan_bits() & crate::card::dispatch_bits::LISTENER == 0 {
                    continue;
                }
                any_listener = true;
                let mine = attacker_controller == Some(c.controller);
                let other = c.id != source;
                for t in &c.definition.triggered_abilities {
                    match t.event.scope {
                        crate::effect::EventScope::YourControl if mine => {
                            if let Some(i) = slot(&t.event.kind) {
                                by_kind[i].push((
                                    c.id,
                                    t.effect.clone(),
                                    c.controller,
                                    t.event.filter.clone(),
                                    true,
                                ));
                            }
                        }
                        crate::effect::EventScope::AnyPlayer if other => {
                            if let Some(i) = slot(&t.event.kind)
                                && t.event.dealer_filter.as_ref().is_none_or(|f| {
                                    self.evaluate_requirement_static(
                                        f,
                                        &Target::Permanent(source),
                                        c.controller,
                                        None,
                                    )
                                })
                            {
                                any_player.push((
                                    i,
                                    (
                                        c.id,
                                        t.effect.clone(),
                                        c.controller,
                                        t.event.filter.clone(),
                                        true,
                                    ),
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
            if let Err(epoch) = lane {
                self.battlefield.store_listener(epoch, any_listener);
            }
        }
        for (i, trig) in any_player {
            by_kind[i].push(trig);
        }

        // Phase 2: walk every player's graveyard for `FromYourGraveyard`
        // triggers. Only fire if the attacker is controlled by the gy
        // owner (the printed "creatures you control" filter on the
        // attacker side). These cards read "whenever ONE OR MORE creatures
        // you control deal combat damage to a player" — one fire per
        // damage batch (CR 603.2), so each graveyard card fires at most
        // once per sub-step even when several attackers connect
        // (`gy_combat_trigger_fired_this_step` dedupes the per-attacker
        // walks; it's cleared at the top of each damage sub-step).
        //
        // This is the one walk that stays per-kind: the dedupe set is read and
        // written between kinds, so a merged walk would let one graveyard card
        // fire for two kinds of the same damage event.
        if let Some(atk_controller) = attacker_controller {
            for (i, kind) in kinds.iter().enumerate() {
                let mut fired: Vec<CardId> = Vec::new();
                for player in &self.players {
                    if player.id.0 != atk_controller {
                        continue;
                    }
                    for gy_card in &player.graveyard {
                        if self.gy_combat_trigger_fired_this_step.contains(&gy_card.id) {
                            continue;
                        }
                        for t in &gy_card.definition.triggered_abilities {
                            if t.event.kind == *kind
                                && matches!(
                                    t.event.scope,
                                    crate::effect::EventScope::FromYourGraveyard
                                )
                            {
                                by_kind[i].push((
                                    gy_card.id,
                                    t.effect.clone(),
                                    gy_card.owner,
                                    t.event.filter.clone(),
                                    false,
                                ));
                                fired.push(gy_card.id);
                            }
                        }
                    }
                }
                // Cold-group guard — see `clear_cold!`.
                if !fired.is_empty() {
                    self.gy_combat_trigger_fired_this_step.extend(fired);
                }
            }
        }

        // CR 702.46 — Cipher. A card exiled encoded on this creature offers its
        // controller a free copy whenever the creature deals combat damage to a
        // player. Reuses the Paradigm free-copy effect (mint a token copy of the
        // exiled card and free-cast it; the encoded original stays in exile).
        if let Some(i) = slot(&EventKind::DealsCombatDamageToPlayer)
            && let Some(atk_ctrl) = attacker_controller
        {
            for enc in &self.exile {
                if enc.encoded_on == Some(source) {
                    by_kind[i].push((enc.id, Effect::CastFreeParadigmCopy, atk_ctrl, None, false));
                }
            }
            // CR 701.54c (level 4+) — "Whenever your Ring-bearer deals combat
            // damage to a player, each opponent loses 3 life."
            if self.players[atk_ctrl].ring_temptations >= 4
                && self.effective_ring_bearer(atk_ctrl) == Some(source)
            {
                by_kind[i].push((
                    source,
                    Effect::LoseLife {
                        who: crate::effect::Selector::Player(crate::effect::PlayerRef::EachOpponent),
                        amount: crate::effect::Value::Const(3),
                    },
                    atk_ctrl,
                    None,
                    false,
                ));
            }
        }

        // "…deals combat damage to a player" bodies that act on something
        // *that player* controls (Hammer of Ruin, Mordant Dragon) read the
        // damaged seat through `ControlledByTriggerPlayer`, both here (target
        // enumeration) and again at resolution.
        if let Target::Player(p) = default_target {
            self.trigger_event_player_scratch = Some(p);
        }
        for (trig_source, effect, controller, filter, bind_dealer) in
            by_kind.into_iter().flatten()
        {
            // CR 603.4 — intervening-'if' on combat-damage triggers ("whenever
            // a creature you control *with toxic* deals combat damage…" —
            // Necrogen Rotpriest). `TriggerSource` in the filter reads the
            // dealing creature, not the listener.
            if let Some(pred) = &filter {
                let mut ctx = crate::game::effects::EffectContext::for_trigger(
                    trig_source,
                    controller,
                    Some(default_target.clone()),
                    0,
                );
                ctx.trigger_source =
                    Some(crate::game::effects::EntityRef::Permanent(source));
                if !self.evaluate_predicate(pred, &ctx) {
                    continue;
                }
            }
            // Most combat-damage triggers implicitly target the damaged player
            // (drain riders, "that player discards / loses life"). But some
            // target a *graveyard* card (Efreet Flamepainter, Venerable
            // Warsinger) or a battlefield permanent (Sword of Sinew and
            // Steel's "destroy up to one artifact") — for those, auto-pick
            // instead of mis-binding slot 0 to the damaged player. A slot-0
            // filter that can't match a player is the precise tell.
            let slot0_filter = effect.target_filter_for_slot_in_mode_kicked(0, None, false);
            let slot0_rejects_player = slot0_filter.is_some_and(|f| !f.can_match_player());
            // A slot 0 that explicitly accepts a player is the damaged player
            // ("exile the top seven of that player's library" — Lord of the
            // Void), even when a later clause moves a card (which would
            // otherwise trip `prefers_graveyard_target`).
            let slot0_accepts_player = slot0_filter.is_some_and(|f| f.can_match_player());
            let target = if !slot0_accepts_player
                && (effect.prefers_graveyard_target() || slot0_rejects_player)
            {
                // Concretize any X-from-cost gate against the damage dealt
                // (Venerable Warsinger's "mana value X or less, where X is
                // the damage this creature dealt to that player").
                self.auto_target_for_effect_avoiding_set_x(
                    &effect,
                    controller,
                    &[trig_source],
                    damage_amount,
                )
                .or(Some(default_target.clone()))
            } else {
                Some(default_target.clone())
            };
            let dealer = bind_dealer
                .then_some(crate::game::effects::EntityRef::Permanent(source));
            self.stack.push(
                TriggerPush::new(trig_source, controller, effect)
                    .target(target)
                    .trigger_source(dealer)
                    .trigger_player(match default_target {
                        Target::Player(p) => Some(p),
                        _ => None,
                    })
                    // The damage dealt doubles as the trigger's X so
                    // `…XFromCost` filters read the hit at resolution too.
                    .x_value(damage_amount)
                    // CR 119.3 — the damage dealt, so Value::TriggerEventAmount
                    // riders scale by the hit (Visions of Brutality).
                    .event_amount(damage_amount)
                    .build(),
            );
        }
    }
}

/// The combat-damage value a creature assigns: its toughness when it has
/// `Keyword::AssignsCombatDamageByToughness` (Doran, the Siege Tower; Bill the
/// Pony), otherwise its power (CR 510.1c). The substitution is unconditional —
/// a 5/1 Doran-creature assigns 1.
fn combat_damage_value(cp: &ComputedPermanent) -> i32 {
    if cp.keywords().has_kw(&Keyword::AssignsCombatDamageByToughness) {
        cp.toughness
    } else {
        cp.power
    }
}

/// Resolution-time snapshot of one attacker's combat-relevant data. Captures
/// the attacker's target so damage routes correctly even if the target moves
/// during the loop.
#[derive(Clone)]
struct AttackerInfo {
    id: CardId,
    controller: usize,
    target: AttackTarget,
    defender_player: usize,
    power: i32,
    has_trample: bool,
    /// CR 702.19c — the "trample over planeswalkers" variant (Thrasta).
    has_trample_over_pw: bool,
    has_lifelink: bool,
    has_deathtouch: bool,
    has_infect: bool,
    has_wither: bool,
    toxic: u32,
    /// CR 510.1a — "assigns its combat damage as though it weren't blocked"
    /// (Thorn Elemental, Rhox). Read from the same snapshot as the keywords
    /// above because CR 510.1 assignment is one turn-based action taken
    /// before any damage is dealt.
    assigns_as_unblocked: bool,
    should_deal: bool,
}
