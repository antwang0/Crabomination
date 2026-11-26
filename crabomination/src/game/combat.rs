use super::*;
use crate::card::Keyword;
use crate::effect::{Effect, EventKind, Selector, Value};
use crate::game::layers::ComputedPermanent;
use smallvec::SmallVec;

/// `(blocker, lethal)` for one attacker's damage assignment, one entry per
/// blocker in damage order. Built fresh per attacker per step and read as a
/// slice, so four inline slots avoid the allocation. PERF `(-365)`.
type Lethals = SmallVec<[(CardId, u32); 4]>;

/// Census pricing a `battlefield.writes()`-keyed gate in front of
/// `fire_combat_damage_triggers`' opening fold (PERF's candidates head).
///
/// The fold walks the whole battlefield for `any_attached` and
/// `soulbond_pair`. Both read instance fields, so `(-315)`'s definition-keyed
/// lane can't hold them; the sound key is `(-306)`'s write counter. The gate
/// only pays on a board with nothing attached and nothing paired (one Aura
/// defeats it, the shape `(-312)` refuted), so the hit rate is the question.
///
/// Gated on the `trig-census` feature (shipped binary unchanged);
/// `CRAB_COMBAT_CENSUS=1` at run time, printed by `bot_ladder`.
#[cfg(feature = "trig-census")]
pub mod combat_census {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    /// `[calls, permanents walked, calls whose board has NO attachment at all,
    /// calls whose board has no attachment AND no soulbond pairing, calls that
    /// found `any_attached` for this dealer, calls that found `soulbond_pair`,
    /// calls a `battlefield.writes()`-keyed fold would have HIT, calls
    /// `(-318)`'s fold actually SKIPPED the walk on]`.
    ///
    /// The last slot decides the key: combat damage is itself a write, so a
    /// `writes`-keyed fold is re-taken between calls. If it reads low, the key
    /// must be a definition-only "can this be attached" lane — an inference
    /// over the 26 sites that assign `attached_to`, needing its own
    /// `debug_assert!` in the ABSENT arm.
    pub static N: [AtomicU64; 8] = [const { AtomicU64::new(0) }; 8];

    /// The `writes()` this census last saw, for the hit-rate column. Single
    /// thread by the census's contract (`--threads 1`), and wrong rather than
    /// unsound if that is broken.
    pub static LAST_WRITES: AtomicU64 = AtomicU64::new(u64::MAX);

    pub fn on() -> bool {
        static LEVEL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *LEVEL.get_or_init(|| match std::env::var("CRAB_COMBAT_CENSUS") {
            Ok(v) => !v.is_empty() && v != "0",
            _ => false,
        })
    }

    #[inline]
    pub(super) fn add(i: usize, n: u64) {
        if n > 0 {
            N[i].fetch_add(n, Relaxed);
        }
    }

    pub fn snapshot() -> [u64; 8] {
        std::array::from_fn(|i| N[i].load(Relaxed))
    }
}

/// One damage trigger gathered by `fire_combat_damage_triggers` before its
/// intervening-'if' runs: `(source, effect, controller, intervening-if,
/// bind_dealer, once)`. `bind_dealer` binds the dealer as the body's
/// `TriggerSource` (Kaito's "return one of them to hand"); `once` is a
/// `once_per_turn` trigger's printed index, the CR 603.3d key
/// (`triggered_once_per_turn_used`), so "one or more creatures you control
/// deal combat damage" fires once for the batch, not once per dealer.
type DamageTrigger = (CardId, Effect, usize, Option<crate::card::Predicate>, bool, Option<(usize, bool)>);

/// CR 603.2c — what a "whenever one or more … deal combat damage to X"
/// trigger batches over. Combat damage is simultaneous (CR 510.4) but each
/// damaged object is its own event: an alpha strike across three pod seats
/// is three events. The dedupe key collapses the per-attacker walk to one
/// fire per damaged subject.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum BatchSubject {
    Player(usize),
    Permanent(CardId),
    /// Every damaged player at once — `EventSpec::batch_across_players`.
    AnyPlayer,
    /// Every recipient of one dealer — the recipient-agnostic kinds
    /// ("whenever ~ deals combat damage"), which trigger once for a dealer's
    /// simultaneous damage however many objects it hits (Sword of Hours).
    Dealer(CardId),
    /// Noncombat twin of `Dealer`: every recipient of one dealer within one
    /// damage effect is one event (a Pestilence sweep). Lives only inside one
    /// `effects::damage_batch` scope.
    NoncombatDealer(CardId),
    /// The open-batch marker those keys sit above.
    NoncombatBatch,
}

impl BatchSubject {
    /// The subject a combat-damage dispatch's `default_target` names.
    fn of(target: &Target) -> Self {
        match *target {
            Target::Player(p) => Self::Player(p),
            Target::Permanent(id) => Self::Permanent(id),
        }
    }
}

/// One SelfSource Attacks trigger `declare_attackers_banded` pushes itself,
/// before its filter runs post-batch: `(source, effect, controller, filter,
/// once)` — `once` as in `DamageTrigger`, the CR 603.3d key of an "attacks
/// for the first time each turn" trigger (Aurelia, Godo).
type AttackTrigger = (CardId, Effect, usize, Option<crate::effect::Predicate>, Option<(usize, bool)>);

/// The `once` key of a printed trigger at index `idx`: `Some` only when the
/// ability says "only once each turn" (a granted trigger passes `None`, as
/// the dispatcher's `trig_idx < n_printed` rule does).
#[inline]
/// The slot a printed trigger fires under: `(index, true)` once a turn (CR
/// 603.3d), `(index, false)` once a batch ("one or more", CR 603.2c); `None`
/// when uncapped or granted.
fn once_key(idx: Option<usize>, t: &crate::card::TriggeredAbility) -> Option<(usize, bool)> {
    idx.filter(|_| t.event.once_per_turn || t.event.once_per_batch)
        .map(|i| (i, t.event.once_per_turn))
}

/// One attacker's granted Attacks triggers: the statics' (fired off the
/// attacker) and the attachments' (`(source, ability)` — the attachment's own
/// under `triggers_on_equipment`).
type AttackerGrants = (
    Vec<crate::card::TriggeredAbility>,
    Vec<(CardId, crate::card::TriggeredAbility)>,
);

/// Static prohibitions `declare_attackers_banded` checks against the whole
/// battlefield: one walk up front, a bit per family (same device as
/// [`crate::game::actions::cast_static`]). Each gated block still runs its
/// own tests, so a clear bit only skips work that was a no-op.
///
/// Sound only for these four, which `matches!` one raw `StaticEffect`
/// variant. Magnetic Web's `AttackTogether` and Arboria's
/// `PlayersCantBeAttackedUnlessTheyActedLastTurn` go through `active_static`,
/// which peels `WhileYourTurn`-style wrappers, so they are deliberately left
/// ungated: gating them would need a second copy of that peeling list
/// (walker drift).
pub(crate) mod attack_static {
    /// `AttackerCapAgainstController` (Silent Arbiter-style per-defender cap).
    pub const ATTACKER_CAP: u32 = 1 << 0;
    /// `AttackPowerCapByControllerHand` (Ghostly Prison-adjacent power caps).
    pub const POWER_CAP: u32 = 1 << 1;
    /// `CreaturesCantAttackController` (Propaganda's prohibition half).
    pub const CANT_ATTACK_CONTROLLER: u32 = 1 << 2;
    /// `AttackTaxToController` (Propaganda, Elephant Grass, Norn's Annex).
    pub const ATTACK_TAX: u32 = 1 << 3;
    /// `OpponentsMustAttackWithAtLeastOne` (Seeker of Slaanesh).
    pub const MUST_ATTACK_WITH_ONE: u32 = 1 << 4;
}

/// Tag which of `declare_attackers_banded`'s rejections fired, under
/// `CRAB_SIM_REJECTS=names`. Twenty-eight returns share three error kinds,
/// so PERF (-55)'s census could name the card but not the rule; `line!()` at
/// the return site is a tag that can't drift. Off by default: one atomic load
/// and a branch, value returned unchanged.
#[inline]
fn attack_reject(line: u32, e: GameError) -> GameError {
    if crate::game::reject_trace_level() >= 2 {
        eprintln!("attack_reject combat.rs:{line} {e:?}");
    }
    e
}

/// [`attack_reject`] for `declare_blockers`: `CannotBlock(id)` names the
/// blocker but not which of ~twenty prohibitions barred it, and batch-level
/// rejections (`MustBeBlockedIfAble`, block cost) name a card not at fault.
#[inline]
fn block_reject(line: u32, e: GameError) -> GameError {
    if crate::game::reject_trace_level() >= 2 {
        eprintln!("block_reject combat.rs:{line} {e:?}");
    }
    e
}

/// CR 702.189a — which Firebending keyword an attacker carries, without the
/// amount: the trigger's `Effect::Firebend` reads the amount as it resolves
/// (after a Melee pump, CR 702.121). See [`AttackerDeclFacts`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FirebendKind {
    /// `Firebending(n)`.
    N(u32),
    /// Firebending X = this creature's computed power, clamped at 0.
    Power,
    /// Firebending = creatures you control.
    CreaturesYouControl,
    /// Firebending = your experience counters.
    Experience,
}

/// What [`GameState::declare_attackers_banded`]'s commit loop asks of one
/// attacker's computed keyword set, in one pass (each field was its own
/// `computed_kw(id)` lookup). PERF `(-338)`.
///
/// ⚠ The payload fields keep `find_map`'s first-match rule, not a max: with
/// two Annihilator keywords the first in the computed set is the one that
/// fired. (`AttackerBlockReqs` takes a max because its question is "any
/// bound unmet".)
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct AttackerDeclFacts {
    /// CR 702.147 — Decayed.
    decayed: bool,
    /// CR 702.20 — Vigilance.
    vigilance: bool,
    /// CR 702.83 — Exert.
    exert: bool,
    /// Exert only if not exerted this turn (`Keyword::ExertOncePerTurn`).
    exert_once: bool,
    /// CR 702.121 — Melee.
    melee: bool,
    /// CR 702.85a — the first `Annihilator(n)`.
    annihilator: Option<u32>,
    /// CR 702.189a — the first Firebending arm.
    firebending: Option<FirebendKind>,
}

/// [`AttackerDeclFacts`] for one computed keyword set.
#[inline]
fn attacker_decl_facts(kws: &[Keyword]) -> AttackerDeclFacts {
    let mut f = AttackerDeclFacts::default();
    for k in kws {
        match k {
            Keyword::Decayed => f.decayed = true,
            Keyword::Vigilance => f.vigilance = true,
            Keyword::Exert => f.exert = true,
            Keyword::ExertOncePerTurn => {
                f.exert = true;
                f.exert_once = true;
            }
            Keyword::Melee => f.melee = true,
            Keyword::Annihilator(n) if f.annihilator.is_none() => f.annihilator = Some(*n),
            Keyword::Firebending(n) if f.firebending.is_none() => {
                f.firebending = Some(FirebendKind::N(*n))
            }
            Keyword::FirebendingPower if f.firebending.is_none() => {
                f.firebending = Some(FirebendKind::Power)
            }
            Keyword::FirebendingCreaturesYouControl if f.firebending.is_none() => {
                f.firebending = Some(FirebendKind::CreaturesYouControl)
            }
            Keyword::FirebendingExperience if f.firebending.is_none() => {
                f.firebending = Some(FirebendKind::Experience)
            }
            _ => {}
        }
    }
    f
}

/// See [`attack_static`]. One battlefield walk; `u32::MAX` is the ungated
/// reading every gated site `debug_assert!`s against.
pub(crate) fn attack_static_scan(state: &GameState) -> u32 {
    use crate::effect::StaticEffect as SE;
    let mut m = 0u32;
    for card in state.battlefield.iter() {
        for sa in &card.definition.static_abilities {
            // Classify the innermost effect of a gated static (Queen Mother
            // Ramonda's "as long as you're the monarch"); the gate itself is
            // read by the consumer through `active_static`.
            let mut eff = &sa.effect;
            while let SE::WhileClassLevelAtLeast { inner, .. }
            | SE::WhileYourTurn { inner }
            | SE::WhileNotYourTurn { inner }
            | SE::WhileCondition { inner, .. } = eff
            {
                eff = inner;
            }
            m |= match *eff {
                SE::AttackerCapAgainstController { .. }
                | SE::AttackerCapAgainstControllerWhileTapped { .. }
                | SE::YourPlaneswalkersAttackerCap { .. } => attack_static::ATTACKER_CAP,
                SE::AttackPowerCapByControllerHand => attack_static::POWER_CAP,
                SE::CreaturesCantAttackController { .. } => {
                    attack_static::CANT_ATTACK_CONTROLLER
                }
                SE::AttackTaxToController { .. } | SE::AttackTaxOnYourPlaneswalkers { .. } => {
                    attack_static::ATTACK_TAX
                }
                SE::OpponentsMustAttackWithAtLeastOne | SE::OpponentsMustAttackYouWithAtLeastOne => {
                    attack_static::MUST_ATTACK_WITH_ONE
                }
                _ => 0,
            };
        }
    }
    m
}

/// The `AttackBlockCostTapAnother` filters carried by a computed keyword list
/// (Hollow Warrior) — one helper must be tapped per entry.
fn tap_another_filters(kws: &[Keyword]) -> Vec<crate::card::SelectionRequirement> {
    // Asked per declared attacker and blocker and almost always empty; an
    // empty `collect()` still calls `Vec::from_iter`, so scan first.
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

/// CR 702.180 / 702.70 — the poison a creature's combat damage adds: Toxic N
/// and Poisonous N fold into one rider.
///
/// ⚠ A sum, not a `has_kw`: layer 6 keeps every granted instance (CR
/// 702.180b / 702.70b — Plague Nurse's granted toxic 1 stacks with a printed
/// one), so only the computed list carries the right total. Shared by the
/// damage step and the bot's block planner, which used to read
/// `definition.keywords` and miss granted Toxic.
pub(crate) fn toxic_poison_value(kws: &[Keyword]) -> u32 {
    kws.iter()
        .filter_map(|k| match k {
            Keyword::Toxic(n) | Keyword::Poisonous(n) => Some(*n),
            _ => None,
        })
        .sum()
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

    /// [`bands_with_other_qualities`](Self::bands_with_other_qualities) off a
    /// computed set the caller already holds (`combat_damage_computed` covers
    /// attackers and blockers), so it costs a battlefield walk, not a gather.
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

    /// CR 506.2 / 509.1b — the tightest "no more than N creatures can
    /// attack/block each combat" cap in play (Silent Arbiter), or `None`.
    ///
    /// Behind the battlefield's `has_combat_cap_static` lane (PERF `(-324)`);
    /// a lane rather than a presence slot because all four callers are
    /// outside a freeze scope.
    pub(crate) fn combat_participation_cap(&self, blocking: bool) -> Option<u32> {
        use crate::effect::StaticEffect;
        if !self.battlefield.has_combat_cap_static() {
            return None;
        }
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

    /// `max_blocks_for` plus the board-dependent riders keyed on the blocker
    /// itself ("…for each Equipment attached to this creature").
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
        self.ignores_defender_against_anyone(card) || self.defender_waived_only_vs_last_turn_attackers(card)
    }

    /// Weathered Sentinels — Defender is waived only against a player who
    /// attacked its controller during their last turn (that player's
    /// `attacked_players_this_turn` lasts until their next turn begins).
    pub(crate) fn defender_waived_only_vs_last_turn_attackers(&self, card: &CardInstance) -> bool {
        card.definition
            .static_abilities
            .iter()
            .any(|sa| matches!(sa.effect, crate::effect::StaticEffect::CanAttackPlayersWhoAttackedYouLastTurn))
            && !self.ignores_defender_against_anyone(card)
            && self.players.iter().any(|pl| pl.attacked_players_this_turn.contains(&card.controller))
    }

    /// An attacker whose Defender is waived only by
    /// `CanAttackPlayersWhoAttackedYouLastTurn`: it may attack only such a
    /// player (no planeswalker or battle).
    pub(crate) fn sentinel_bound(&self, attacker: CardId) -> bool {
        let Some(card) = self.battlefield_find(attacker) else { return false };
        card.definition
            .static_abilities
            .iter()
            .any(|sa| matches!(sa.effect, crate::effect::StaticEffect::CanAttackPlayersWhoAttackedYouLastTurn))
            && self
                .computed_permanent(attacker)
                .is_some_and(|c| c.keywords().contains(&Keyword::Defender))
            && !self.ignores_defender_against_anyone(card)
    }

    fn ignores_defender_against_anyone(&self, card: &CardInstance) -> bool {
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
        // Asked per tapped candidate blocker; the exact answer is a whole-board
        // scan, so the freeze scope's seat-independent presence slot goes
        // first. `false` there is authoritative for every seat.
        self.tapped_block_static_in_scope()
            && self.battlefield.iter().any(|c| {
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
        if let Some(chooser) = self.turn.block_chooser_this_combat.or(self.turn.block_chooser_this_turn) {
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
            // CR 509.1a — blocks are declared by the defending players: in a
            // duel "not the active player", at three or more seats only the
            // players being attacked (or their teammates). With nothing
            // attacking there is nothing to declare either way.
            None => {
                seat != self.active_player_idx
                    && (self.attacking.is_empty()
                        || self.attacking.iter().any(|a| {
                            self.defender_for(a.target)
                                .is_some_and(|d| self.same_team(seat, d))
                        }))
            }
        }
    }

    // ── Declare attackers ─────────────────────────────────────────────────────

    /// CR 508.1a — the restrictions on one attacker that depend only on the
    /// creature and the board, not on the defender
    /// ([`Self::attacker_target_block`]) or the rest of the batch (Okk, the
    /// participation cap).
    ///
    /// One walker shared by [`Self::declare_attackers_banded`]'s gate and its
    /// CR 508.1d "attacks each combat if able" loops, so they can't disagree
    /// about who is able — a hand-written copy once made a creature required
    /// to attack and then rejected for attacking.
    ///
    /// Returns the rejecting line for `CRAB_SIM_REJECTS=names`.
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
            .unwrap_or_else(|| self.computed_is_creature(card));
        // CR 701.35 — detain; CR 508.1a — Wall of Dust's one-turn ban. Both
        // report `CannotAttack`, as does a permanent that isn't a creature
        // right now (bestowed Aura, de-animated Vehicle) — not "summoning sick".
        if !is_creature_now
            || card.detained_by.is_some()
            || card.attack_ban == crate::card::AttackBan::Active
            || self.turn.attack_locks.iter().any(|(f, ctrl, except)| {
                !except.contains(&id) && self.evaluate_requirement_static_on(f, card, *ctrl, None)
            })
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
        // One walk over the computed keywords: this runs per attack candidate
        // in the bot's search, and asking each family separately read `fixed`
        // +0.17 %.
        let mut hasted = false;
        // A granted "can attack as though it didn't have defender" (Assault
        // Formation's {G}, Walking Bulwark, the magecraft Walls) waives it too.
        let waives_defender = kws.iter().any(|k| matches!(k, Keyword::AttacksAsThoughNoDefender));
        for k in kws {
            match k {
                Keyword::Haste => hasted = true,
                Keyword::CantAttack => {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                Keyword::Defender if !waives_defender && !self.ignores_defender_for_attack(card) => {
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
                Keyword::CantAttackOrBlockUnlessAHandIsEmpty if !self.some_hand_is_empty() => {
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
                        .any(|c| self.permanent_has_land_type(c, *lt)) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Harbor Serpent: five or more Islands.
                Keyword::CantAttackUnlessLandCount(lt, n)
                    if (self
                        .battlefield
                        .iter()
                        .filter(|c| self.permanent_has_land_type(c, *lt))
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
                    if !card.counter_sum().is_multiple_of(2) =>
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
        if card.summoning_sick && !hasted && !self.attack_as_though_haste(p, None) {
            return Some((line!(), GameError::SummoningSickness(id)));
        }
        None
    }

    /// CR 302.6 — Frenzied Saddlebrute: may a summoning-sick creature `p`
    /// controls attack `defender` as though it had haste? `None` asks whether
    /// any live defender is covered.
    pub(crate) fn attack_as_though_haste(&self, p: usize, defender: Option<usize>) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|src| {
            let q = src.controller;
            src.definition.static_abilities.iter().any(|sa| {
                self.static_active_is(&sa.effect, src, |e| matches!(e, StaticEffect::CreaturesAttackYourOpponentsAsThoughHaste))
            }) && match defender {
                Some(d) => d != p && !self.same_team(q, d),
                None => (0..self.players.len()).any(|d| {
                    d != p && self.players[d].is_alive() && !self.same_team(p, d) && !self.same_team(q, d)
                }),
            }
        })
    }

    /// CR 508.1a — the restrictions on one attacker that depend on which
    /// defender it is aimed at. Split out so [`Self::attacker_is_able`] can
    /// ask whether any defender would do.
    pub(crate) fn attacker_target_block(
        &self,
        p: usize,
        id: CardId,
        kws: &[Keyword],
        defender: Option<usize>,
    ) -> Option<(u32, GameError)> {
        if let Some(d) = defender
            && self.battlefield_find(id).is_some_and(|c| self.obligated_to(c, d))
        {
            return Some((line!(), GameError::CannotAttack(id)));
        }
        // CR 302.6 — a summoning-sick attacker past `attacker_self_block` is
        // there on a Frenzied Saddlebrute waiver, which covers only that
        // source's controller's opponents.
        if defender.is_some()
            && !kws.contains(&Keyword::Haste)
            && self.battlefield_find(id).is_some_and(|c| c.summoning_sick)
            && !self.attack_as_though_haste(p, defender)
        {
            return Some((line!(), GameError::SummoningSickness(id)));
        }
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
                // CR 726 — Crown-Hunter Hireling: only the monarch.
                Keyword::CantAttackUnlessDefenderIsMonarch if defender != self.monarch => {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Merchant Ship: the defender controls a land of
                // the named type.
                Keyword::CantAttackUnlessDefenderControlsLandType(lt)
                    if defender.is_some_and(|d| {
                        !self.battlefield.iter().any(|c| {
                            c.controller == d && self.permanent_has_land_type(c, *lt)
                        })
                    }) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Port Razer: not a player it attacked this turn.
                Keyword::CantAttackPlayerAttackedThisTurn
                    if defender.is_some_and(|d| {
                        self.battlefield_find(id).is_some_and(|c| {
                            c.cold_any(|k| k.combat_defenders.as_ref().is_some_and(|v| v.contains(&d)))
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
                // CR 508.1a — Vow of Duty: not the player (or a planeswalker
                // of the player) who controls an attached Aura granting it.
                Keyword::CantAttackAuraController
                    if defender.is_some_and(|d| {
                        self.battlefield.iter().any(|c| {
                            c.attached_to == Some(id)
                                && c.controller == d
                                && c.definition.equipped_bonus.as_ref().is_some_and(|b| {
                                    b.keywords.contains(&Keyword::CantAttackAuraController)
                                })
                        })
                    }) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                Keyword::CantAttackPlayer(s) if defender == Some(*s) => {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
                // CR 508.1a — Xantcha: not its owner, nor the owner's
                // planeswalkers (`defender` is the planeswalker's controller).
                Keyword::CantAttackOwner
                    if defender.is_some() && defender == self.battlefield_find(id).map(|c| c.owner) =>
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
                // CR 508.1a — Vantress Gargoyle: a full enough graveyard.
                Keyword::CantAttackUnlessDefenderGraveyardAtLeast(n)
                    if defender.is_some_and(|d| (self.players[d].graveyard.len() as u32) < *n) =>
                {
                    return Some((line!(), GameError::CannotAttack(id)));
                }
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
                    | Keyword::CantAttackAuraController
                    | Keyword::CantAttackPlayer(_)
                    | Keyword::CantAttackOwner
                    | Keyword::CantAttackUnlessMoreCreaturesThanDefender
                    | Keyword::CantAttackUnlessMoreLandsThanDefender
                    | Keyword::CantAttackUnlessDefenderGraveyardAtLeast(_)
                    // CR 508.1d — Port Razer (or a copy) that has attacked
                    // every open seat this turn is able to attack nobody; the
                    // fast path required it (6-seat pod, seed 4506217).
                    | Keyword::CantAttackPlayerAttackedThisTurn
            )
        })
    }

    /// The Ensnaring Bridge-style caps in scope, one per static (CR 613 — each
    /// reads its own controller's hand). `statics` is [`attack_static_scan`]'s
    /// bitmask, so a board without the family pays one bit test.
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

    /// May `card` be declared as an attacker against `defender`,
    /// batch-independent? The bot's attack picker calls this so its filter
    /// can't drift from [`Self::declare_attackers_banded`] (the PERF (-55)
    /// bug class). Batch-level rules (Okk, participation cap, attacks-alone)
    /// are the caller's.
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

    /// CR 508.1 — the tightest "no more than N creatures can attack you"
    /// cap seat `d` controls (Crawlspace).
    pub(crate) fn attacker_cap_against(&self, d: usize) -> Option<usize> {
        self.battlefield
            .iter()
            .filter(|c| c.controller == d)
            .filter_map(|c| {
                c.definition.static_abilities.iter().find_map(|sa| match sa.effect {
                    crate::effect::StaticEffect::AttackerCapAgainstController { n } => Some(n),
                    crate::effect::StaticEffect::AttackerCapAgainstControllerWhileTapped { n } if c.tapped => {
                        Some(n)
                    }
                    _ => None,
                })
            })
            .min()
    }

    /// Tomik — the smallest `YourPlaneswalkersAttackerCap` among `d`'s
    /// permanents: how many creatures may attack each of `d`'s planeswalkers.
    pub(crate) fn planeswalker_attacker_cap(&self, d: usize) -> Option<usize> {
        self.battlefield
            .iter()
            .filter(|c| c.controller == d)
            .flat_map(|c| &c.definition.static_abilities)
            .filter_map(|sa| match sa.effect {
                crate::effect::StaticEffect::YourPlaneswalkersAttackerCap { n } => Some(n),
                _ => None,
            })
            .min()
    }

    /// CR 508.1d — a bound creature may stay home when a cap leaves no room:
    /// the global cap (Silent Arbiter) is full, or every opponent it could
    /// attack is at its own cap (Crawlspace). Only maximal when every declared
    /// attacker is itself bound — the caller's half.
    fn attack_cap_leaves_no_room(&self, p: usize, card: &crate::card::CardInstance, kws: &[Keyword], attacks: &[Attack], statics: u32) -> bool {
        if let Some(cap) = self.combat_participation_cap(false)
            && self.attacking.len() + attacks.len() >= cap as usize
        {
            return true;
        }
        if statics & attack_static::ATTACKER_CAP == 0 {
            return false;
        }
        (0..self.players.len())
            .filter(|&d| {
                !self.same_team(p, d)
                    && self.players[d].is_alive()
                    && self.attacker_target_block(p, card.id, kws, Some(d)).is_none()
            })
            .all(|d| {
                self.attacker_cap_against(d).is_some_and(|cap| {
                    attacks.iter().filter(|a| a.target == AttackTarget::Player(d)).count() >= cap
                }) && !self.battlefield.iter().any(|c| {
                    c.controller == d
                        && (self.computed_has_card_type(c, crate::card::CardType::Planeswalker)
                            || self.computed_has_card_type(c, crate::card::CardType::Battle))
                })
            })
    }

    /// CR 508.1d — is `card` able to attack at all? Shared by the three
    /// "attacks each combat if able" loops: nothing about the creature or
    /// board blocks it, and some legal defender survives the
    /// defender-dependent restrictions. Same walkers as the declaration gate,
    /// because a requirement the gate rejects has no legal answer.
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
        // Fast path: nothing about which defender matters. A goad, a "can't
        // attack you" lock or an Island Sanctuary shield can bar every seat,
        // so each takes the walk below (6-seat pod, seed 793022).
        if !Self::has_defender_dependent_restriction(kws)
            && !taxed
            && statics & attack_static::CANT_ATTACK_CONTROLLER == 0
            && !card.cold_any(|k| !k.goad_holds.is_empty() || !k.goaded_by.is_empty())
            && !self.players.iter().any(|pl| pl.attackable_only_by_until_next_turn.is_some())
        {
            return true;
        }
        // CR 508.1d — able means some defender takes the attack unrestricted
        // (Blazing Archon, a Second Doctor ban, Island Sanctuary bar seats).
        // Same seats `attack_target_options` scores.
        let restriction = self.attack_left_right_defender();
        (0..self.players.len()).any(|d| {
            self.player_open_to_attack(p, d, restriction)
                && self.attacker_target_block(p, card.id, kws, Some(d)).is_none()
                && (!taxed || self.attack_is_untaxed(card.id, kws, d, statics))
                && self
                    .attack_target_prohibited(&Attack { attacker: card.id, target: AttackTarget::Player(d) }, statics)
                    .is_none()
        })
    }

    /// CR 508.1g — can attacking cost anything on this board? Three sources:
    /// `AttackTaxToController` statics (Propaganda; the only one the
    /// `attack_static_scan` bitmask covers), War Tax, and Forbidding Spirit.
    pub(crate) fn attack_tax_possible(&self, statics: u32) -> bool {
        statics & attack_static::ATTACK_TAX != 0
            || self.attack_tax_this_turn > 0
            || self.players.iter().any(|pl| pl.attack_tax_until_your_turn > 0)
    }

    /// CR 508.1d — does `id` attack `d` for free? A player isn't required to
    /// pay a cost to attack, so a requirement binds only free attacks. Asking
    /// "could the tax be paid" instead forced a goaded creature through
    /// Ghostly Prison (5-seat pod, seed 777042) and a Juggernaut behind
    /// Propaganda (`build_cube_state_seeded(3637)`). Prices this attacker
    /// alone; the batch total is the declaration gate's.
    fn attack_is_untaxed(
        &self,
        id: CardId,
        kws: &[Keyword],
        d: usize,
        statics: u32,
    ) -> bool {
        let atk = Attack { attacker: id, target: crate::game::types::AttackTarget::Player(d) };
        let tax = self.attack_tax_for(std::slice::from_ref(&atk), statics, |_| {
            self.attack_block_keyword_tax(id, kws, true)
        });
        tax == 0
    }


    pub fn declare_attackers(
        &mut self,
        attacks: Vec<Attack>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.declare_attackers_banded(attacks, vec![])
    }

    /// CR 701.43d — is exerting `id` worth its cost to `controller`? The bonus
    /// is a linked `Exerted` trigger (CR 607.2h): one needing a target with
    /// none available is a wasted untap (Glory-bringer on an empty board);
    /// anything else is pure upside and taken.
    fn exert_pays_off(&self, id: CardId, controller: usize) -> bool {
        let Some(card) = self.battlefield_find(id) else { return false };
        let mut linked = card
            .definition
            .triggered_abilities
            .iter()
            .filter(|t| t.event.kind == EventKind::Exerted)
            .peekable();
        // No linked bonus: the exert still pays when a permanent you control
        // triggers on "whenever you exert a creature" (Trueheart Twins,
        // Battlefield Scavenger); with neither it is a pure downside.
        if linked.peek().is_none() {
            return self.battlefield.iter().any(|c| {
                c.controller == controller
                    && c.definition.triggered_abilities.iter().any(|t| {
                        t.event.kind == EventKind::Exerted && t.event.scope == crate::effect::EventScope::YourControl
                    })
            });
        }
        linked.any(|t| {
            !t.effect.requires_target()
                || !self.enumerate_legal_targets_xc(&t.effect, controller, Some(id), 0, 0).is_empty()
        })
    }

    /// CR 508.1g — [`declare_attackers_banded`] with the optional costs to
    /// attack announced in the same step. `exert` is the set of chosen
    /// attackers that pay exert; a creature outside it does not, whatever its
    /// keyword says.
    ///
    /// [`declare_attackers_banded`]: Self::declare_attackers_banded
    pub fn declare_attackers_exerting(
        &mut self,
        attacks: Vec<Attack>,
        exert: Vec<CardId>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.declare_attackers_full(attacks, vec![], Some(exert))
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
        self.declare_attackers_full(attacks, bands, None)
    }

    /// The body of the three declare-attackers entries. `exert` is `None`
    /// when no optional cost was announced, which leaves the choice to
    /// [`exert_pays_off`](Self::exert_pays_off).
    pub fn declare_attackers_full(
        &mut self,
        attacks: Vec<Attack>,
        bands: Vec<Vec<CardId>>,
        exert: Option<Vec<CardId>>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let exert = exert.as_deref();
        if self.step != TurnStep::DeclareAttackers {
            return Err(GameError::WrongStep { actual: self.step });
        }
        // A combat whose end-of-combat step was never left (a caller that
        // jumped the step) is over by now: CR 511.3's removal, late.
        if self.combat_damage_dealt() {
            self.remove_all_from_combat();
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
        // One battlefield walk for the four static prohibitions; see
        // [`attack_static`]. Two are asked per attacker.
        let statics = attack_static_scan(self);

        // CR 803.1a/b — under the attack-left / attack-right option, the only
        // legal defending player is the nearest living opponent in that
        // direction (and "more than one seat away" means you can't attack at
        // all, which falls out of the same walk).
        let seat_restriction = self.attack_left_right_defender();

        // Validate every target up front: the defender must be an opponent.
        // `same_team(a, a)` is true, so this one check rejects self and
        // teammates (2HG).
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
                        // CR 508.1 — The Second Doctor's next-turn ban.
                        || self.next_turn_attack_banned(self.active_player_idx, target_player)
                        // CR 508.1a — Weathered Sentinels attacks only a player
                        // who attacked you during their last turn.
                        || (self.sentinel_bound(atk.attacker)
                            && !self.players[target_player].attacked_players_this_turn.contains(&p))
                    {
                        return Err(GameError::InvalidAttackTarget(target_player));
                    }
                }
                AttackTarget::Planeswalker(pw_id) => {
                    if !self.planeswalker_attackable(p, pw_id, atk.attacker, seat_restriction) {
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
                        || self.sentinel_bound(atk.attacker)
                        || protector == Some(self.active_player_idx)
                        || protector.is_none_or(|pr| !self.players[pr].is_alive())
                    {
                        return Err(GameError::InvalidPlaneswalkerAttackTarget(b_id));
                    }
                }
            }
        }

        // CR 508.1d — "attacks each combat if able" (Juggernaut, goaded
        // creatures): every able MustAttack creature must be declared while
        // an opponent is in range; reject an incomplete declaration.
        let has_legal_target = self
            .players
            .iter()
            .enumerate()
            .any(|(i, pl)| !self.same_team(p, i) && pl.is_alive());
        // CR 508.1d — Magnetic Web: once one of the group attacks, every able
        // member must join. Built before the layer pass so an empty list can
        // keep that pass small. Two `for` loops, not `flat_map`, because
        // `FlatMap::next` costs per permanent even when empty (PERF (-78)).
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

        // One layer pass (one freeze scope) for the whole declaration;
        // everything up to trigger collection is validation. Only the CR
        // 508.1d requirement loop and the `AttackTogether` loop read the whole
        // board, and both are no-ops unless some permanent can carry an attack
        // requirement — otherwise the pass covers only the attackers, band
        // members and creatures already attacking.
        let (computed, attack_requirement) = self.with_frozen_layers(|g| {
            let requirement = has_legal_target
                && (g.any_goad_present()
                    || g.attack_lure_of(p).is_some()
                    || g.side_attacks_if_able(p)
                    || g.any_creature_lure(p)
                    || !g.attack_player_requirements.is_empty()
                    || statics & attack_static::MUST_ATTACK_WITH_ONE != 0
                    || g.board_keyword_in_scope(&[
                        Keyword::MustAttack,
                        Keyword::MustAttackOrBlock,
                        Keyword::MustAttackIfAnotherAttacks,
                        Keyword::MustAttackChosenPlayer,
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

        // CR 702.22c-d — band legality: all members attacking the same
        // defender, at least one with banding, at most one without. Computed
        // keywords, so granted banding counts.
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
                            | crate::effect::StaticEffect::AttackerCapAgainstControllerWhileTapped { .. }
                    ))),
            "attack_static_scan missed an attacker cap",
        );
        for p in 0..self.players.len() {
            if statics & attack_static::ATTACKER_CAP == 0 {
                break; // no cap on the board — the per-seat walks below are all `None`
            }
            let Some(cap) = self.attacker_cap_against(p) else {
                continue;
            };
            let against =
                attacks.iter().filter(|a| a.target == AttackTarget::Player(p)).count();
            if against > cap {
                return Err(GameError::InvalidAttackTarget(p));
            }
        }
        // Tomik — each planeswalker its controller's cap covers takes no more
        // than `n` attackers.
        if statics & attack_static::ATTACKER_CAP != 0 {
            for atk in &attacks {
                let AttackTarget::Planeswalker(pw) = atk.target else { continue };
                let Some(owner_seat) = self.battlefield_find(pw).map(|c| c.controller) else { continue };
                let Some(cap) = self.planeswalker_attacker_cap(owner_seat) else { continue };
                if attacks.iter().filter(|a| a.target == AttackTarget::Planeswalker(pw)).count() > cap {
                    return Err(GameError::InvalidAttackTarget(owner_seat));
                }
            }
        }

        // CR 508.1d / 701.15b — defender-naming requirements (goad's "a player
        // other than the goader", a lure's planeswalker) are scored together:
        // each attack must obey as many as its best unrestricted, free
        // defender would.
        for atk in &attacks {
            if !self.attack_target_obeys_most(p, atk.attacker, atk.target) {
                let defender = match atk.target {
                    AttackTarget::Player(q) => q,
                    AttackTarget::Planeswalker(cid) | AttackTarget::Battle(cid) => {
                        self.battlefield_find(cid).map_or(usize::MAX, |c| c.controller)
                    }
                };
                return Err(attack_reject(line!(), GameError::InvalidAttackTarget(defender)));
            }
        }

        // Angelic Arbiter — an opponent who cast a spell this turn can't
        // attack with creatures at all.
        if !attacks.is_empty() && self.cast_bars_attacks(self.active_player_idx) {
            return Err(attack_reject(line!(), GameError::CannotAttack(attacks[0].attacker)));
        }

        // CR 508.1c — "attacks only alone" (Master of Cruelties), computed
        // keywords so granted variants count.
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

        // CR 508.1c — "can't attack alone" (Militia Rallier). A lone attacker
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

        // Two events per attacker (the tap and the declaration), sized once
        // (PERF (-273)).
        let mut events: Vec<GameEvent> = Vec::with_capacity(2 * attacks.len());
        // Per CR 506.5, the Attacks trigger filter must be evaluated
        // post-batch, so we carry the optional filter alongside each
        // queued trigger.
        let mut triggers: Vec<AttackTrigger> = vec![];
        // Attackers that paid exert this declaration (CR 701.43a).
        let mut exerted: Vec<CardId> = Vec::new();
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

        // Hoisted above the CR 508.1d loops: an Ensnaring Bridge cap makes a
        // must-attack creature unable, so they need it too.
        let attack_power_caps = self.attack_power_caps(statics);
        // CR 508.1d — one able predicate for the three requirement loops and
        // the gate below (see `attacker_is_able`). No creature is able when no
        // defender is open (every opponent banned by The Second Doctor, no
        // walker or battle) — 5-seat pod, seed 780004. Asked lazily, once.
        let no_defender = std::cell::OnceCell::new();
        let no_defender = || {
            *no_defender.get_or_init(|| {
                // A banned seat's permanents are barred with it ("you or
                // permanents you control"); Angelic Arbiter bars them all.
                self.cast_bars_attacks(p)
                    || (self.attackable_players_for(p).is_empty()
                    && !self.battlefield.iter().any(|c| {
                        let d = if self.computed_has_card_type(c, crate::card::CardType::Planeswalker) {
                            Some(c.controller)
                        } else if self.computed_has_card_type(c, crate::card::CardType::Battle) {
                            c.protected_by
                        } else {
                            None
                        };
                        d.is_some_and(|d| d != p && !self.next_turn_attack_banned(p, d))
                    }))
            })
        };
        let able_to_attack = |c: &crate::card::CardInstance| {
            !no_defender()
                && self.attacker_is_able(
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

        // CR 508.1d — the declared attackers a requirement bound ("if that
        // creature had to attack this combat", Firkraag), kept once the
        // declaration stands.
        let mut had_to: SmallVec<[CardId; 4]> = SmallVec::new();
        // CR 508.1d — a lure (Gideon Jura's +2): every creature able to attack
        // must, and at the lure. Can't be satisfied → no requirement, so this
        // runs only while the lured permanent is still an attackable walker.
        if self.attack_lure_of(p).is_some() {
            had_to.extend(attacks.iter().map(|a| a.attacker));
            for c in &self.battlefield {
                if c.controller == p
                    && self.computed_is_creature(c)
                    && able_to_attack(c)
                    && !attacks.iter().any(|a| a.attacker == c.id)
                {
                    return Err(attack_reject(line!(), GameError::CannotAttack(c.id)));
                }
            }
        }

        // CR 508.1d — Rowan Kenrith's +2: every creature able to attack must.
        if self.side_attacks_if_able(p) {
            had_to.extend(attacks.iter().map(|a| a.attacker));
        }
        if self.side_attacks_if_able(p)
            && let Some(c) = self.battlefield.iter().find(|c| {
                c.controller == p
                    && self.computed_is_creature(c)
                    && able_to_attack(c)
                    && !attacks.iter().any(|a| a.attacker == c.id)
            })
        {
            return Err(attack_reject(line!(), GameError::CannotAttack(c.id)));
        }

        // CR 508.1d — a one-creature lure (Gideon, Battle-Forged's +2): that
        // creature attacks the lured walker if able.
        if self.any_creature_lure(p) {
            for c in &self.battlefield {
                if c.controller != p {
                    continue;
                }
                if self.creature_lure_of(p, c.id).is_none() {
                    continue;
                }
                had_to.push(c.id);
                if able_to_attack(c) && !attacks.iter().any(|a| a.attacker == c.id) {
                    return Err(attack_reject(line!(), GameError::CannotAttack(c.id)));
                }
            }
        }

        if attack_requirement {
            let mut missing: SmallVec<[&crate::card::CardInstance; 4]> = SmallVec::new();
            for c in &self.battlefield {
                // Controller test first: a field compare, where `must` costs
                // three subset finds per permanent (PERF `(-338)`).
                if c.controller != p {
                    continue;
                }
                // MustAttack (Juggernaut) or goaded (CR 701.38); one pass over
                // the computed set for the three keywords.
                let ckw = computed_kw(c.id);
                let (mut must_attack, mut must_either, mut must_if_another) = (false, false, false);
                for k in ckw {
                    match k {
                        Keyword::MustAttack => must_attack = true,
                        Keyword::MustAttackOrBlock => must_either = true,
                        // CR 508.1d — Ekundu Cyclops only has to join an
                        // attack someone else already started.
                        Keyword::MustAttackIfAnotherAttacks => must_if_another = true,
                        // CR 508.1d — Raving Dead attacks the chosen
                        // opponent if able; which defender is scored with the
                        // other defender-naming requirements above.
                        Keyword::MustAttackChosenPlayer => {
                            if let Some(q) = c.chosen_player
                                && self.players.get(q).is_some_and(|pl| pl.is_alive())
                                && !self.same_team(p, q)
                            {
                                must_attack = true;
                            }
                        }
                        _ => {}
                    }
                }
                let must = must_attack
                    || must_either
                    || (must_if_another && attacks.iter().any(|a| a.attacker != c.id))
                    || self.is_goaded(c)
                    || self.must_attack_a_player_this_combat(p, c.id);
                if !must {
                    continue;
                }
                had_to.push(c.id);
                if able_to_attack(c) && !attacks.iter().any(|atk| atk.attacker == c.id) {
                    missing.push(c);
                }
            }
            // CR 508.1d — most requirements obeyed without breaking a
            // restriction: a bound creature a full cap shuts out, or one that
            // can't attack alone with no partner, is excused — provided no
            // declared attacker is unbound (a swap would obey more).
            let all_bound = attacks.iter().all(|a| had_to.contains(&a.attacker));
            let alone_excused = |c: &crate::card::CardInstance| {
                let kws = computed_kw(c.id);
                (kws.has_kw(&Keyword::CantAttackAlone) || kws.has_kw(&Keyword::CantAttackOrBlockAlone))
                    && attacks.is_empty()
                    && self.attacking.is_empty()
                    && !self.battlefield.iter().any(|o| {
                        o.controller == p && o.id != c.id && self.computed_is_creature(o) && able_to_attack(o)
                    })
            };
            // …and Okk with no bigger creature able to attack beside it.
            let power = |id: CardId| computed.iter().find(|x| x.id == id).map_or(0, |x| x.power);
            let okk_excused = |c: &crate::card::CardInstance| {
                computed_kw(c.id).has_kw(&Keyword::CantAttackUnlessGreaterPowerAttacks)
                    && !self.battlefield.iter().any(|o| {
                        o.controller == p
                            && o.id != c.id
                            && power(o.id) > power(c.id)
                            && self.computed_is_creature(o)
                            && able_to_attack(o)
                    })
                    && !self.attacking.iter().any(|a| power(a.attacker) > power(c.id))
            };
            if let Some(c) = missing.iter().find(|c| {
                let capped = all_bound && self.attack_cap_leaves_no_room(p, c, computed_kw(c.id), &attacks, statics);
                !(alone_excused(c) || okk_excused(c) || capped)
            }) {
                return Err(attack_reject(line!(), GameError::CannotAttack(c.id)));
            }
        }
        // CR 508.1d — Seeker of Slaanesh: at least one attacker, if able.
        if attack_requirement
            && attacks.is_empty()
            && statics & attack_static::MUST_ATTACK_WITH_ONE != 0
            && self.opponent_forces_an_attack(p)
            && let Some(c) = self.battlefield.iter().find(|c| c.controller == p && able_to_attack(c))
        {
            return Err(attack_reject(line!(), GameError::CannotAttack(c.id)));
        }
        // CR 508.1d — Trove of Temptation: each demanding opponent must be
        // attacked while some able creature is idle or aimed at a seat no
        // Trove names.
        if attack_requirement && statics & attack_static::MUST_ATTACK_WITH_ONE != 0 {
            let demanding = self.opponents_demanding_attack(p);
            if !demanding.is_empty() {
                let aims_at = |a: &Attack| match a.target {
                    AttackTarget::Player(q) => Some(q),
                    AttackTarget::Planeswalker(pw) => self.battlefield.find_by_id(pw).map(|c| c.controller),
                    _ => None,
                };
                let unmet = demanding.iter().any(|&q| !attacks.iter().any(|a| aims_at(a) == Some(q)));
                let spare = self.battlefield.iter().find(|c| {
                    c.controller == p
                        && able_to_attack(c)
                        && attacks
                            .iter()
                            .find(|a| a.attacker == c.id)
                            .is_none_or(|a| aims_at(a).is_none_or(|q| !demanding.contains(&q)))
                });
                if unmet && let Some(c) = spare {
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
            // A full cap excuses a member it shuts out while every declared
            // attacker is a member or otherwise bound (CR 508.1d).
            let all_bound = attacks.iter().all(|a| matches(a.attacker) || had_to.contains(&a.attacker));
            for c in &self.battlefield {
                if c.controller != p || !matches(c.id) {
                    continue;
                }
                if able_to_attack(c)
                    && !attacks.iter().any(|atk| atk.attacker == c.id)
                    && !(all_bound && self.attack_cap_leaves_no_room(p, c, computed_kw(c.id), &attacks, statics))
                {
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
            let mut seen: crate::game::types::IdSet<CardId> = Default::default();
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
                // CR 508.1a — every creature-and-board restriction, in the
                // walker the CR 508.1d loops share (two copies drifted).
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
                // CR 508.1a — Okk: needs a strictly bigger partner in the batch
                // (already-declared attackers count). Batch-dependent, so it
                // stays out of the shared walker.
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

        // CR 508.1 — the prohibitions that read the attacked player, in the
        // walker the bot's picker shares (`attack_target_prohibited`).
        for atk in &attacks {
            if let Some(line) = self.attack_target_prohibited(atk, statics) {
                return Err(attack_reject(line, GameError::CannotAttack(atk.attacker)));
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
            if self.try_pay_with_auto_tap_queued(p, &tax_cost).is_err() {
                return Err(attack_reject(line!(), GameError::CannotAttack(attacks[0].attacker)));
            }
        }
        // CR 508.1g / 119.4 — a life tax (Sivitri): payable only with that
        // much life.
        let life_tax = self.attack_life_tax_for(&attacks);
        if life_tax > 0 {
            if !super::cast_cost::life_payable(self.players[p].life, life_tax) {
                return Err(attack_reject(line!(), GameError::CannotAttack(attacks[0].attacker)));
            }
            self.pay_life_cost(p, life_tax);
        }

        // CR 508.1g — Floodtide Serpent: each such attacker returns one
        // matching permanent to hand; checked and consumed together so two
        // Serpents need two.
        {
            let mut spent: Vec<CardId> = Vec::new();
            for atk in &attacks {
                // The `collect` only ends the keyword borrow before `&mut
                // self`; almost no attacker has the keyword and an empty
                // `collect()` still calls `Vec::from_iter`, so ask the slice
                // first (same for the `AttackCostSacrifice` twin).
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

        // CR 508.1g — Hollow Warrior: tap an untapped matching non-attacking
        // permanent, one per such attacker. Same presence-first shape.
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
            for &id in &tapped {
                if let Some(c) = self.battlefield_find_mut(id) {
                    c.tapped = true;
                }
            }
        }

        // CR 508.1h-j / 506.4 — a creature the attack costs removed (Blood Pet
        // sacrificed for Ghostly Prison's tax) was declared but is not in
        // combat; the rest attack.
        let mut attacks = attacks;
        attacks.retain(|a| self.battlefield.find_by_id(a.attacker).is_some_and(|c| c.controller == p));
        let any_attackers = !attacks.is_empty();
        // CR 702.121a — Melee counts distinct opponents attacked; a walker or
        // battle doesn't count its controller (Adriana ruling, 2016-08-23).
        let melee_opponents: i32 = {
            let mut seats: crate::game::types::IdSet<usize> = Default::default();
            for atk in &attacks {
                let seat = match atk.target {
                    AttackTarget::Player(s) => Some(s),
                    AttackTarget::Planeswalker(_) | AttackTarget::Battle(_) => None,
                };
                if let Some(s) = seat.filter(|s| !self.same_team(*s, p)) {
                    seats.insert(s);
                }
            }
            seats.len() as i32
        };
        // Statics-granted (Thorncaster) and equipment-granted (CR 702.6e)
        // attack triggers come off whole-board scans whose answer is the same
        // for every attacker, so walk once here. Consumed by value in the main
        // loop to free its `battlefield.iter_mut()` borrow.
        // CR 113.10b — an attacker stripped of all abilities has no printed
        // attack trigger.
        let stripped = self.stripped_permanents();
        let attacker_grants: Vec<AttackerGrants> = {
            let trigger_grants = self.trigger_grant_sources();
            let equip_grants = self.equip_granted_trigger_sources();
            attacks
                .iter()
                .map(|atk| {
                    self.battlefield
                        .find_by_id(atk.attacker)
                        .map(|c| {
                            let mut equip = self.equip_granted_triggers_with(c, &equip_grants);
                            // CR 702.95 — a paired creature's `soulbond_bonus`
                            // grants its "whenever this creature attacks" to
                            // both members (Imperious Mindbreaker); it fires
                            // off the attacker, like an attachment's grant.
                            if let Some(partner) = c.soulbond_partner
                                && self.battlefield.find_by_id(partner).is_some()
                            {
                                for holder in [c.id, partner] {
                                    let Some(bonus) = self
                                        .battlefield
                                        .find_by_id(holder)
                                        .and_then(|h| h.definition.soulbond_bonus.as_ref())
                                    else {
                                        continue;
                                    };
                                    equip.extend(
                                        bonus
                                            .triggered_abilities
                                            .iter()
                                            .filter(|t| t.event.kind == EventKind::Attacks)
                                            .map(|t| (c.id, t.clone())),
                                    );
                                }
                            }
                            (self.statics_granted_triggers_with(c, &trigger_grants), equip)
                        })
                        .unwrap_or_default()
                })
                .collect()
        };
        // Instance grants (`Effect::GrantTriggeredAbility`) of the two kinds
        // the whole-board listener walks below read: none in the catalog
        // today, read so neither walk can drop one silently.
        let own_attacks_grant = self.any_granted_trigger_of_kind(&EventKind::Attacks);
        let own_you_attack_grant = self.any_granted_trigger_of_kind(&EventKind::YouAttack);
        // CR 603.2c on the defender's side: "whenever an opponent attacks you
        // and/or one or more planeswalkers you control" fires once per
        // declaration, not per attacker (Cunning Rhetoric). Declared outside
        // the per-attacker loop because the batch is the whole declaration.
        let mut defender_batch_fired: Vec<(CardId, usize)> = Vec::new();
        // CR 508.1m / 603.3d — a defender-side trigger that picks its own
        // target ("target creature attacking you", Lulu) waits for the whole
        // declaration, so its choice sees every attacker.
        let mut own_target_fires: Vec<(CardId, usize, Effect, CardId)> = Vec::new();
        if !had_to.is_empty() {
            self.had_to_attack.extend(had_to.iter().copied().filter(|id| attacks.iter().any(|a| a.attacker == *id)));
        }
        // Pick Up the Pace's "as long as you attacked this turn" and Boros
        // Strike-Captain's "attacked with three or more creatures" grants wake
        // for this seat (CR 508.1: declared attackers).
        if !attacks.is_empty() {
            self.wake_may_play(crate::card::MayPlayDuration::TurnsHolderAttacks { holder: p }, p);
            if attacks.len() >= 3 {
                self.wake_may_play(crate::card::MayPlayDuration::TurnsHolderAttacksWithThreeOrMore { holder: p }, p);
            }
        }
        for (atk, (static_granted, equip_granted)) in attacks.into_iter().zip(attacker_grants) {
            let id = atk.attacker;
            // Validated above — commit only. Filter by controller, not owner:
            // a stolen creature attacks for its current controller. Both reads
            // borrow the cold group, so take them before the battlefield `&mut`.
            // One pass over the computed set for the six keywords this loop
            // reads (PERF `(-338)`).
            let dfacts = attacker_decl_facts(computed_kw(id));
            debug_assert!(
                dfacts.decayed == computed_kw(id).has_kw(&Keyword::Decayed)
                    && dfacts.vigilance == computed_kw(id).has_kw(&Keyword::Vigilance)
                    && dfacts.exert
                        == (computed_kw(id).has_kw(&Keyword::Exert)
                            || computed_kw(id).has_kw(&Keyword::ExertOncePerTurn))
                    && dfacts.melee == computed_kw(id).has_kw(&Keyword::Melee)
                    && dfacts.annihilator
                        == computed_kw(id).iter().find_map(|kw| match kw {
                            Keyword::Annihilator(n) => Some(*n),
                            _ => None,
                        }),
                "attacker_decl_facts disagrees with the per-keyword asks it replaced",
            );
            let decayed = dfacts.decayed;
            let granted: Vec<crate::card::TriggeredAbility> = self.granted_triggers(id).cloned().collect();
            // CR 701.43 / 508.1g — Exert is an optional cost to attack. `exert`
            // carries the active player's announcement; with none, the engine
            // takes it only when `exert_pays_off`. Read before the `&mut` below.
            let exerted_now = dfacts.exert
                && !(dfacts.exert_once && self.exerted_this_turn.contains(&id))
                && match exert {
                    Some(chosen) => chosen.contains(&id),
                    None => self.exert_pays_off(id, p),
                };
            // Neyali's "during any turn you attacked with a token" and Neriv's
            // "… with a commander" (CR 903.3 — any player's) grants wake for
            // this seat (CR 508.1: a declared attacker).
            if self.battlefield.find_by_id(id).is_some_and(|c| c.is_token) {
                self.wake_may_play(crate::card::MayPlayDuration::TurnsHolderAttacksWithAToken { holder: p }, p);
            }
            // Robber of the Rich's "during any turn you attacked with a Rogue".
            self.wake_may_play_for_attacker_type(id, p);
            if !self.players[p].attacked_with_commander_this_turn && self.is_commander(id) {
                self.players[p].attacked_with_commander_this_turn = true;
                self.wake_may_play(crate::card::MayPlayDuration::TurnsHolderAttacksWithACommander { holder: p }, p);
            }
            self.note_attack_defender(atk.target);
            if exerted_now && !self.exerted_this_turn.contains(&id) {
                self.exerted_this_turn.push(id);
            }
            // Ramses, Assassin Lord — "attacked this turn by an Assassin you
            // controlled".
            if let AttackTarget::Player(d) = atk.target
                && d < self.players.len()
                && self.permanent_has_creature_type(id, crate::card::CreatureType::Assassin)
            {
                self.players[d].attacked_by_assassins_of |= 1u64 << p;
            }
            if let AttackTarget::Planeswalker(pw) = atk.target
                && let Some(d) = self.battlefield_find(pw).map(|c| c.controller)
                && !self.players[p].attacked_walkers_of_this_turn.contains(&d)
            {
                self.players[p].attacked_walkers_of_this_turn.push(d);
            }
            // CR 603.3a — an attachment's own Attacks trigger
            // (`triggers_on_equipment`: Mirror Mockery on an opponent's
            // creature) is its controller's, not the attacker's.
            let equip_ctrl: Vec<usize> = equip_granted
                .iter()
                .map(|(s, _)| if *s == id { p } else { self.battlefield_find(*s).map_or(p, |c| c.controller) })
                .collect();
            let card = self
                .battlefield
                .iter_mut()
                .find(|c| c.id == id && c.controller == p)
                .ok_or(GameError::CardNotOnBattlefield(id))?;
            if !dfacts.vigilance {
                card.tapped = true;
                // CR 508.1f — attacking taps the creature; surface a
                // "becomes tapped" event so Tapped triggers fire (Magda).
                events.push(GameEvent::PermanentTapped { card_id: id, actor: None, as_attacker: true });
            }
            // The creature then skips its next untap step.
            if exerted_now {
                card.skip_next_untap = true;
                exerted.push(id);
            }
            // CR 702.121 — Melee: +1/+1 until end of turn per opponent attacked.
            if melee_opponents > 0 && dfacts.melee {
                card.pump(melee_opponents, melee_opponents);
            }
            // CR 702.142 — record that this creature attacked (gates Boast).
            card.attacked_this_turn = true;
            card.attacked_own_turn = true;
            card.times_attacked_this_turn = card.times_attacked_this_turn.saturating_add(1);
            if let AttackTarget::Player(d) = atk.target {
                card.attacked_players |= 1u16.checked_shl(d as u32).unwrap_or(0);
            }
            // Territorial Hellkite's record of whom it attacked this combat.
            if let AttackTarget::Player(d) = atk.target
                && card.cold_any(|k| k.combat_defenders.as_ref().is_some_and(|v| !v.contains(&d)))
                && let Some(v) = card.combat_defenders.as_mut()
            {
                v.push(d);
            }
            if let AttackTarget::Player(d) = atk.target
                && card.definition.keywords.has_kw(&Keyword::CantAttackPlayerAttackedThisTurn)
                && !card.cold_any(|k| k.combat_defenders.as_ref().is_some_and(|v| v.contains(&d)))
            {
                card.combat_defenders.get_or_insert_with(Vec::new).push(d);
            }
            self.attacking.push(atk);
            // Raid (CR 702.108 ability word): the controller attacked this turn.
            self.players[p].attacked_this_turn = true;
            self.players[p].creatures_attacked_this_turn += 1;
            if let AttackTarget::Player(d) = atk.target
                && !self.players[p].attacked_players_this_turn.contains(&d)
            {
                self.players[p].attacked_players_this_turn.push(d);
            }
            events.push(GameEvent::AttackerDeclared(id));
            // Printed plus transient granted Attacks triggers (Root
            // Manipulation, via `granted_triggers_timed`). An attachment's
            // grant fires off the attachment when `triggers_on_equipment`
            // says so; everything else off the attacker.
            let printed: &[crate::card::TriggeredAbility] =
                if stripped.contains(&id) { &[] } else { &card.definition.triggered_abilities };
            let own = printed
                .iter()
                .enumerate()
                .map(|(i, t)| (id, Some(i), t, p))
                .chain(granted.iter().chain(static_granted.iter()).map(|t| (id, None, t, p)));
            let equipped = equip_granted.iter().zip(&equip_ctrl).map(|((s, t), c)| (*s, None, t, *c));
            for (src, idx, t, ctrl) in own.chain(equipped) {
                // Only SelfSource Attacks triggers are pushed here; YourControl
                // ones (Exalted, Battle Banner) go through
                // `dispatch_triggers_for_events` off `AttackerDeclared`, and
                // pushing them here too would double-fire.
                // CR 701.43d — the linked "when you do" rides `Exerted`, so it
                // fires only for a creature that actually paid the cost.
                if t.event.kind == EventKind::Exerted
                    && t.event.scope == crate::effect::EventScope::SelfSource
                    && exerted_now
                {
                    triggers.push((src, t.effect.clone(), ctrl, t.event.filter.clone(), once_key(idx, t)));
                }
                if t.event.kind == EventKind::Attacks
                    && t.event.scope == crate::effect::EventScope::SelfSource
                {
                    // Keep the filter to re-evaluate after the whole batch is
                    // declared (CR 506.5 "attacking alone" needs that view).
                    triggers.push((src, t.effect.clone(), ctrl, t.event.filter.clone(), once_key(idx, t)));
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
            // CR 702.85a — Annihilator N as an Attacks trigger:
            // `Effect::Sacrifice { who: defender, count: N, filter: Any }`. For
            // a planeswalker attack the defender is its controller (CR 506.4a).
            let annihilator_n = dfacts.annihilator;
            if let Some(n) = annihilator_n
                && let Some(defender) = self.defender_for(atk.target)
            {
                let sac_effect = Effect::Sacrifice {
                    who: Selector::Player(crate::effect::PlayerRef::Seat(defender)),
                    count: Value::Const(n as i32),
                    filter: crate::card::SelectionRequirement::Permanent,
                };
                triggers.push((id, sac_effect, p, None, None));
            }
            // CR 702.189a — Firebending N: "whenever this creature attacks, add
            // N {R}", a stack trigger (CR 605.1b, 2025 ruling). The amount is
            // read on resolution; the mana lasts until end of combat
            // (`firebending_kept_red`).
            if let Some(k) = dfacts.firebending {
                let amount = match k {
                    FirebendKind::N(n) => Value::Const(n as i32),
                    FirebendKind::Power => Value::PowerOf(Box::new(Selector::This)),
                    FirebendKind::CreaturesYouControl => Value::CountOf(Box::new(Selector::EachPermanent(
                        crate::card::SelectionRequirement::Creature
                            .and(crate::card::SelectionRequirement::ControlledByYou),
                    ))),
                    FirebendKind::Experience => Value::ControllerExperience,
                };
                triggers.push((id, Effect::Firebend { amount }, p, None, None));
            }
            // ControllerAttackedByOpponent (CR 508.1g listeners): the defender's
            // "when a creature an opponent controls attacks me" permanents
            // (Coveted Jewel). The attacker's controller binds the target slot.
            if let Some(defender) = self.defender_for(atk.target)
                && defender != p
            {
                // The planeswalker-only sibling scope fires only when the
                // attack is aimed at a planeswalker (Mila) — not the player.
                let is_pw_attack =
                    matches!(atk.target, crate::game::types::AttackTarget::Planeswalker(_));
                // Plain loops over the trigger member list, not the board
                // (PERF `(-222)`, and `flat_map` costs per permanent), unless
                // an instance grant of the kind is live — a grant can land on
                // a permanent with no printed trigger.
                /// One defender-side listener: its source, its effect, its
                /// `EventSpec` filter, and the CR 603.3d/603.2c key it fires
                /// under (`None` when uncapped or granted).
                ///
                /// ⚠ The filter must be carried here like every other walk in
                /// this file: it was once dropped, so `.with_filter` on a
                /// defender-side trigger was silently ignored (Reveille Squad's
                /// "if this creature is untapped").
                type Listener =
                    (CardId, Effect, Option<crate::card::Predicate>, Option<(usize, bool)>);
                let mut listeners: Vec<Listener> = Vec::new();
                let listens = |t: &crate::card::TriggeredAbility| {
                    t.event.kind == EventKind::Attacks
                        && (t.event.scope == crate::effect::EventScope::ControllerAttackedByOpponent
                            || (is_pw_attack
                                && t.event.scope
                                    == crate::effect::EventScope::ControllerPlaneswalkerAttackedByOpponent)
                            || (!is_pw_attack
                                && t.event.scope
                                    == crate::effect::EventScope::ControllerAttackedDirectlyByOpponent))
                };
                // By value into whichever walk runs, so the body inlines
                // (a `&mut` closure is an out-of-line call per permanent).
                let visit = |c: &crate::card::CardInstance| {
                    if c.controller != defender || stripped.contains(&c.id) {
                        return;
                    }
                    for (i, t) in c.definition.triggered_abilities.iter().enumerate() {
                        if listens(t) {
                            listeners.push((
                                c.id,
                                t.effect.clone(),
                                t.event.filter.clone(),
                                once_key(Some(i), t),
                            ));
                        }
                    }
                    if own_attacks_grant {
                        // A granted trigger has no printed index, so it passes
                        // `None` — the dispatcher's `trig_idx < n_printed` rule.
                        self.for_each_granted_trigger_matching(c.id, listens, |t| {
                            listeners.push((c.id, t.effect.clone(), t.event.filter.clone(), None))
                        });
                    }
                };
                self.battlefield.for_each_triggerer_or_all(own_attacks_grant, visit);
                for (src, effect, filter, once) in listeners {
                    // CR 603.2 — the trigger's own condition, before the
                    // once-key, so a rejected declaration doesn't spend the slot.
                    //
                    // ⚠ Evaluated per attacker while `self.attacking` is still
                    // being filled, so a predicate that counts the declaration
                    // (`AttackedWithCountAtLeast` and its defender-side
                    // sibling) would see a partial batch; put it in the effect
                    // as an `Effect::If` instead. Filtering after the batch
                    // would mean moving the pushes, which reorders the stack.
                    if let Some(predicate) = filter {
                        let ctx = crate::game::effects::EffectContext {
                            controller: defender,
                            source: Some(src),
                            trigger_source: Some(crate::game::effects::EntityRef::Permanent(id)),
                            ..crate::game::effects::EffectContext::default()
                        };
                        if !self.evaluate_predicate(&predicate, &ctx) {
                            continue;
                        }
                    }
                    // CR 603.3d / 603.2c — one fire a turn, or one a
                    // declaration; `once_per_turn` keys the shared per-turn
                    // set, `once_per_batch` this declaration's own.
                    if let Some((i, per_turn)) = once {
                        let key = (src, i);
                        let spent = if per_turn {
                            !self.triggered_once_per_turn_used.insert(key)
                        } else if defender_batch_fired.contains(&key) {
                            true
                        } else {
                            defender_batch_fired.push(key);
                            false
                        };
                        if spent {
                            continue;
                        }
                    }
                    if effect.requires_target()
                        && effect.target_filter_for_slot(0).is_some_and(|f| !f.can_match_player())
                    {
                        own_target_fires.push((src, defender, effect, id));
                        continue;
                    }
                    self.push_stack(
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
        for (src, defender, effect, id) in own_target_fires {
            // CR 603.3d — a prompting defender names the target.
            if self.trigger_asks_targets(defender, &effect) {
                self.queue_trigger_asking(crate::game::types::PendingTriggerPush {
                    source: src,
                    controller: defender,
                    effect,
                    subject: Some(crate::game::effects::EntityRef::Permanent(id)),
                    event_amount: 0,
                    mode: None,
                    intervening_if: None,
                    actor: None,
                    from_mana_ability: false,
                    x_value: 0,
                    converged_value: 0,
                    mana_spent: 0,
                });
                continue;
            }
            let target = self.auto_target_for_effect_avoiding(&effect, defender, Some(src));
            self.push_stack(
                TriggerPush::new(src, defender, effect)
                    .target(target)
                    .trigger_source(Some(crate::game::effects::EntityRef::Permanent(id)))
                    .build(),
            );
        }
        // CR 702.22e — a declared band lasts for the rest of combat; drop
        // members that never made it into combat. ⚠ `attack_bands` is a
        // `ColdState` field: compare (a `&self` read) before writing, or every
        // declare-attackers deep-copies the cold group.
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

        // YourControl-scoped Attacks triggers (Battle Banner, Sparring
        // Regimen) are not walked here: `dispatch_triggers_for_events` in
        // `mod.rs` picks them up off `AttackerDeclared`, and walking them here
        // too would double-fire. `is_event_hardcoded` marks only SelfSource
        // Attacks as handled.

        // CR 508.1 — "whenever a player attacks one of your opponents" (Breena,
        // Combat Calligrapher): once per attacked player per listener, for
        // every listener that player is an opponent of. The attacking player
        // rides the target slot, the attacked player the trigger source.
        if any_attackers {
            let mut defended: Vec<usize> = Vec::new();
            for a in self.attacking.iter() {
                if let AttackTarget::Player(d) = a.target
                    && !defended.contains(&d)
                {
                    defended.push(d);
                }
            }
            defended.sort_unstable();
            let listens = |t: &crate::card::TriggeredAbility| {
                t.event.kind == EventKind::Attacks
                    && t.event.scope == crate::effect::EventScope::OpponentOfYoursAttacked
            };
            // CR 603.2c — `once_per_batch` is "one or more of your opponents"
            // (Jolene, the Plunder Queen): one fire for the declaration, not
            // one per opponent attacked.
            let mut listeners: Vec<(CardId, usize, Effect, Option<crate::card::Predicate>, bool)> = Vec::new();
            for c in self.battlefield.iter().filter(|c| !stripped.contains(&c.id)) {
                for t in c.definition.triggered_abilities.iter().filter(|t| listens(t)) {
                    listeners.push((c.id, c.controller, t.effect.clone(), t.event.filter.clone(), t.event.once_per_batch));
                }
            }
            let mut batch_fired: Vec<usize> = Vec::new();
            for d in defended {
                for (i, (src, ctrl, effect, filter, once)) in listeners.iter().enumerate() {
                    if *ctrl == d || self.same_team(*ctrl, d) || !self.players[*ctrl].is_alive() {
                        continue;
                    }
                    if *once && batch_fired.contains(&i) {
                        continue;
                    }
                    let ctx = crate::game::effects::EffectContext {
                        controller: *ctrl,
                        source: Some(*src),
                        targets: vec![Target::Player(p)],
                        trigger_source: Some(crate::game::effects::EntityRef::Player(d)),
                        ..crate::game::effects::EffectContext::default()
                    };
                    if filter.as_ref().is_some_and(|f| !self.evaluate_predicate(f, &ctx)) {
                        continue;
                    }
                    if *once {
                        batch_fired.push(i);
                    }
                    self.push_stack(
                        TriggerPush::new(*src, *ctrl, effect.clone())
                            .target(Some(Target::Player(p)))
                            .trigger_source(Some(crate::game::effects::EntityRef::Player(d)))
                            .build(),
                    );
                }
            }
            // CR 508.1 — "whenever a player attacks" (Mirkwood Trapper): once
            // per declaration, the attacking player in the target slot and as
            // the trigger source.
            let any_attack: Vec<(CardId, usize, Effect, Option<crate::card::Predicate>)> = self
                .battlefield
                .iter()
                .filter(|c| self.players[c.controller].is_alive() && !stripped.contains(&c.id))
                .flat_map(|c| {
                    c.definition
                        .triggered_abilities
                        .iter()
                        .filter(|t| {
                            t.event.kind == EventKind::Attacks
                                && t.event.scope == crate::effect::EventScope::AnyPlayerAttacks
                        })
                        .map(move |t| (c.id, c.controller, t.effect.clone(), t.event.filter.clone()))
                })
                .collect();
            for (src, ctrl, effect, filter) in any_attack {
                let subject = Some(crate::game::effects::EntityRef::Player(p));
                let ctx = crate::game::effects::EffectContext {
                    controller: ctrl,
                    source: Some(src),
                    targets: vec![Target::Player(p)],
                    trigger_source: subject,
                    ..crate::game::effects::EffectContext::default()
                };
                if filter.as_ref().is_some_and(|f| !self.evaluate_predicate(f, &ctx)) {
                    continue;
                }
                self.push_stack(
                    TriggerPush::new(src, ctrl, effect)
                        .target(Some(Target::Player(p)))
                        .trigger_source(subject)
                        .build(),
                );
            }
            // CR 508.1 — "whenever you attack a player" (Karazikar, Firkraag):
            // once per player the attacking player attacks, that player bound
            // as the trigger source and subject, so the queue can target
            // "a creature that player controls" (CR 603.3d).
            let mine: Vec<(CardId, Effect, Option<crate::card::Predicate>)> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && !stripped.contains(&c.id))
                .flat_map(|c| {
                    c.definition
                        .triggered_abilities
                        .iter()
                        .filter(|t| {
                            t.event.kind == EventKind::Attacks
                                && t.event.scope == crate::effect::EventScope::YouAttackedPlayer
                        })
                        .map(move |t| (c.id, t.effect.clone(), t.event.filter.clone()))
                })
                .collect();
            if !mine.is_empty() {
                let mut queue = Vec::new();
                for a_def in self.attacking.iter().filter_map(|a| match a.target {
                    AttackTarget::Player(d) => Some(d),
                    _ => None,
                }).collect::<std::collections::BTreeSet<_>>() {
                    for (src, effect, filter) in &mine {
                        let subject = Some(crate::game::effects::EntityRef::Player(a_def));
                        if let Some(f) = filter {
                            let mut ctx = crate::game::effects::EffectContext::for_trigger(*src, p, None, 0);
                            ctx.trigger_source = subject;
                            if !self.evaluate_predicate(f, &ctx) {
                                continue;
                            }
                        }
                        queue.push(crate::game::types::PendingTriggerPush {
                            source: *src,
                            controller: p,
                            effect: effect.clone(),
                            subject,
                            event_amount: 0,
                            mode: None,
                            intervening_if: None,
                            // No actor: it would stamp the attacker as the
                            // trigger player over the attacked subject.
                            actor: None,
                            from_mana_ability: false,
                            x_value: 0,
                            converged_value: 0,
                            mana_spent: 0,
                        });
                    }
                }
                self.drain_trigger_queue(queue);
            }
        }

        // CR 701.43 — "whenever you exert a creature" (Trueheart Twins,
        // Battlefield Scavenger): once per exerted attacker, off each
        // permanent of the attacking player's that listens.
        for _ in &exerted {
            let p = self.active_player_idx;
            for c in self.battlefield.iter().filter(|c| c.controller == p) {
                for (i, t) in c.definition.triggered_abilities.iter().enumerate() {
                    if t.event.kind == EventKind::Exerted && t.event.scope == crate::effect::EventScope::YourControl {
                        triggers.push((c.id, t.effect.clone(), p, t.event.filter.clone(), once_key(Some(i), t)));
                    }
                }
            }
        }

        // "Whenever one or more …" slots spent by this declaration.
        let mut batch_fired: Vec<(CardId, usize)> = Vec::new();
        // Firebender Ascension's listeners, `(id, controller)`.
        let questers: Vec<(CardId, usize)> = if triggers.is_empty() {
            Vec::new()
        } else {
            self.battlefield
                .iter()
                .filter(|c| c.definition.keywords.contains(&Keyword::AttackTriggerQuestCopy))
                .map(|c| (c.id, c.controller))
                .collect()
        };
        for (source, effect, controller, filter, once) in triggers {
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
                    cast_from_graveyard: false,
                    event_amount: 0,
                    kicked: false,
                    trigger_source_left_gy: false,
                    kicked_options: Vec::new(),
                    kick_count: 0,
                    bargained: false,
                    cast_via_mayhem: false,
                    cast_via_madness: false,
                    spell_cast_from_exile: false,
                    cast_via_waterbend: false,
                    cast_collected_evidence: false,
                    entwined: false,
                    cast_condition_held: false,
                    event_player: None,
                    spree_modes: Vec::new(),
                };
                if !self.evaluate_predicate(&predicate, &ctx) {
                    continue;
                }
            }
            // CR 603.3d / 603.2c — after the filter, so a declaration the
            // filter rejects does not spend the slot (the dispatcher's order).
            if let Some((i, per_turn)) = once {
                let key = (source, i);
                let spent = if per_turn {
                    !self.triggered_once_per_turn_used.insert(key)
                } else if batch_fired.contains(&key) {
                    true
                } else {
                    batch_fired.push(key);
                    false
                };
                if spent {
                    continue;
                }
            }
            // Isshin / Windcrag Siege (Mardu): a self-source attack trigger of a
            // permanent you control fires an additional time per doubler.
            let fires = 1
                + self.attack_trigger_extra_fires(controller)
                + crate::game::actions::ally_trigger_extra_fires(self, controller, source)
                + self.source_trigger_extra_fires(source);
            // CR 603.3d — a prompting attacker names the targets, each fire
            // on its own.
            if self.trigger_asks_targets(controller, &effect) {
                let mode = self.pick_trigger_mode(&effect, source, controller);
                for _ in 0..fires {
                    self.queue_trigger_asking(crate::game::types::PendingTriggerPush {
                        source,
                        controller,
                        effect: effect.clone(),
                        subject: None,
                        event_amount: 0,
                        mode,
                        intervening_if: None,
                        actor: None,
                        from_mana_ability: false,
                        x_value: 0,
                        converged_value: 0,
                        mana_spent: 0,
                    });
                    self.push_quest_copy_triggers(&questers, source, controller, true);
                }
                continue;
            }
            let (mode, auto_target) =
                self.trigger_mode_and_target(&effect, controller, Some(source));
            // CR 115.1c — fill any additional "up to N target" slots (Lagorin's
            // "put a +1/+1 counter on each of up to two target Mounts and/or
            // Vehicles"), same as the self-source ETB path.
            let additional = self.auto_extra_targets_for(
                effect.targeting_view(mode),
                source,
                controller,
                auto_target.clone(),
            );
            // `repeat_n` clones `fires - 1` times and yields the original
            // last, so the ordinary board — no doubler, `fires == 1` — pays
            // no clone at all (PERF `(-361)`).
            for (effect, auto_target, additional) in
                std::iter::repeat_n((effect, auto_target, additional), fires)
            {
                self.push_stack(
                    TriggerPush::new(source, controller, effect)
                        .target(auto_target)
                        .additional_targets(additional)
                        .mode(mode)
                        .build(),
                );
                self.push_quest_copy_triggers(&questers, source, controller, false);
            }
        }

        // CR 508 — "Whenever you attack" fires once for the attacking player.
        // Active-player permanents fire for SelfSource/YourControl, an
        // `OpponentControl` one for each opponent of the attacker, and
        // `AnyPlayer` observers (Argent Dais) from every controller, once per
        // combat, fired for the ability's controller.
        if any_attackers {
            let ap = self.active_player_idx;
            #[allow(clippy::type_complexity)]
            // Plain loops, same reason as `groups` and `listeners` above, and
            // over the trigger member list for the same reason as `listeners`.
            let mut you_attack: Vec<(CardId, usize, Effect, Option<crate::effect::Predicate>)> =
                Vec::new();
            // A graveyard-scoped "whenever you attack" (Persistent
            // Marshstalker) functions only from the graveyard (CR 603.3d);
            // its battlefield copy is walked below, not here.
            // "Whenever an opponent attacks" (`OpponentControl`, Tahngarth,
            // First Mate) fires for every seat not on the attacker's team.
            let listens = |t: &crate::card::TriggeredAbility, ctrl: usize| {
                t.event.kind == EventKind::YouAttack
                    && !t.event.scope.from_graveyard()
                    && (ctrl == ap
                        || t.event.scope == crate::effect::EventScope::AnyPlayer
                        || (t.event.scope == crate::effect::EventScope::OpponentControl && !self.same_team(ctrl, ap)))
            };
            let visit = |c: &crate::card::CardInstance| {
                if stripped.contains(&c.id) {
                    return;
                }
                let ctrl = c.controller;
                for t in &c.definition.triggered_abilities {
                    if listens(t, ctrl) {
                        you_attack.push((c.id, ctrl, t.effect.clone(), t.event.filter.clone()));
                    }
                }
                // CR 722.2a — a station card's `{N+}` band triggers, live at
                // `min` charge counters (Dawnsire's "whenever you attack").
                if !c.definition.station.is_empty() {
                    let charges = c.counter_count(crate::card::CounterType::Charge);
                    for band in c.definition.station.iter().filter(|b| charges >= b.min) {
                        for t in band.triggers.iter().filter(|t| listens(t, ctrl)) {
                            you_attack.push((c.id, ctrl, t.effect.clone(), t.event.filter.clone()));
                        }
                    }
                }
                if own_you_attack_grant {
                    self.for_each_granted_trigger_matching(c.id, |t| listens(t, ctrl), |t| {
                        you_attack.push((c.id, ctrl, t.effect.clone(), t.event.filter.clone()))
                    });
                }
            };
            self.battlefield.for_each_triggerer_or_all(own_you_attack_grant, visit);
            // "Whenever you attack" from the attacking player's graveyard
            // (Persistent Marshstalker's threshold return), behind the
            // zone's lane like the step and combat-damage walks.
            if self.players[ap].graveyard.has_graveyard_trigger() {
                for c in &self.players[ap].graveyard {
                    for t in &c.definition.triggered_abilities {
                        if t.event.kind == EventKind::YouAttack && t.event.scope.from_graveyard() {
                            you_attack.push((c.id, c.owner, t.effect.clone(), t.event.filter.clone()));
                        }
                    }
                }
            }
            // CR 113.6b — Eminence: a "whenever you attack" that functions
            // from the command zone (Sidar Jabari of Zhalfir).
            if !self.command_zones_are_empty() {
                for c in self.players[ap].command.iter() {
                    let all = c.command_zone_abilities_active();
                    for t in &c.definition.triggered_abilities {
                        if (all || t.event.zone.in_command_zone()) && listens(t, ap) {
                            you_attack.push((c.id, ap, t.effect.clone(), t.event.filter.clone()));
                        }
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
                // Isshin / Fractured Realm: an attack-caused trigger of a
                // permanent you control fires an additional time per doubler.
                let fires = 1
                    + self.attack_trigger_extra_fires(ctrl)
                    + crate::game::actions::ally_trigger_extra_fires(self, ctrl, src)
                    + self.source_trigger_extra_fires(src);
                // CR 603.3d — a prompting controller names the targets.
                if self.trigger_asks_targets(ctrl, &effect) {
                    let mode = self.pick_trigger_mode(&effect, src, ctrl);
                    for _ in 0..fires {
                        self.queue_trigger_asking(crate::game::types::PendingTriggerPush {
                            source: src,
                            controller: ctrl,
                            effect: effect.clone(),
                            subject: None,
                            event_amount: 0,
                            mode,
                            intervening_if: None,
                            actor: None,
                            from_mana_ability: false,
                            x_value: 0,
                            converged_value: 0,
                            mana_spent: 0,
                        });
                    }
                    continue;
                }
                let (mode, auto_target) = self.trigger_mode_and_target(&effect, ctrl, Some(src));
                // CR 115.1c — "up to N target" slots past the first (Master's
                // Guidance's "each of up to two target attacking creatures").
                let additional =
                    self.auto_extra_targets_for(effect.targeting_view(mode), src, ctrl, auto_target.clone());
                for (effect, auto_target, additional) in
                    std::iter::repeat_n((effect, auto_target, additional), fires)
                {
                    self.push_stack(
                        TriggerPush::new(src, ctrl, effect)
                            .target(auto_target)
                            .additional_targets(additional)
                            .mode(mode)
                            .build(),
                    );
                }
            }
        }

        self.fire_declare_attackers_delayed();
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

        // CR 509.1a / 509.1c — each defending player declares their own blocks
        // (the batches merge), so requirements are checked against the
        // declarer's creatures only (6-seat pod, seed 780059). `None` (a block
        // chooser, or no defender priority) means every defending creature.
        let declarer: Option<SmallVec<[usize; 4]>> = match self.block_chooser() {
            Some(_) => None,
            None => {
                let q = self.priority.player_with_priority;
                (q != self.active_player_idx && self.may_declare_blocks(q))
                    .then(|| (0..self.players.len()).filter(|&t| self.same_team(t, q)).collect())
            }
        };
        let for_declarer = |ctl: usize| declarer.as_ref().is_none_or(|team| team.contains(&ctl));

        // One layer pass for the whole declaration. Every consumer below looks
        // up participants by id except the five CR 509.1 requirement loops,
        // which ask the whole board; four are keyword-keyed and one reads
        // `must_block`, so when no permanent can carry one the pass covers only
        // the participants. See `board_keyword_in_scope` for why `false` is
        // authoritative.
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
            // `block_map` is an `IdMap`, so its key order is declaration
            // order; it only decides which permanents get computed anyway,
            // and every reader looks up by id.
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

        // CR 509.1b/c — the six per-attacker block requirements, folded once
        // per attacker (PERF `(-337)`). Sound across the payment/tap mutations
        // below: `self.attacking` is written only by `declare_attackers` and
        // `computed` is an owned snapshot.
        let atk_reqs: SmallVec<[crate::game::AttackerBlockReqs; 8]> =
            self.attacking.iter().map(|a| crate::game::attacker_block_reqs(kws_of(a.attacker))).collect();
        debug_assert!(
            self.attacking.iter().zip(atk_reqs.iter()).all(|(a, r)| {
                let k = kws_of(a.attacker);
                r.menace == k.has_kw(&Keyword::Menace)
                    && r.no_more_than_one == k.has_kw(&Keyword::CantBeBlockedByMoreThanOne)
                    && r.must_be_blocked == k.has_kw(&Keyword::MustBeBlocked)
                    && r.all_must_block == k.has_kw(&Keyword::AllMustBlock)
                    && r.unless_all_block == k.has_kw(&Keyword::CantBeBlockedUnlessAllBlock)
                    && r.except_by_n
                        == k.iter()
                            .filter_map(|kw| match kw {
                                Keyword::CantBeBlockedExceptByN(n) => Some(*n),
                                _ => None,
                            })
                            .max()
                            .unwrap_or(0)
            }),
            "attacker_block_reqs disagrees with the per-keyword asks it replaced",
        );

        // Validate all assignments before mutating anything; each blocker's
        // controller must be the defender of its attacker. CR 509.1b — one
        // attacker per blocker unless `CanBlockAdditional` / `CanBlockAnyNumber`;
        // count the merged set (earlier blocks plus this batch) and reject a
        // repeated pair.
        let mut batch_blocks: crate::game::types::SmallIdMap<CardId, SmallVec<[CardId; 4]>> =
            Default::default();
        // CR 509.1b — Silent Arbiter: "No more than N creatures can block each
        // combat." Count distinct blockers across already-declared blocks and
        // this batch.
        if let Some(cap) = self.combat_participation_cap(true)
            && let Some(&(first, _)) = assignments.first()
        {
            let mut distinct: crate::game::types::SmallIdSet<CardId> =
                self.block_map.keys().copied().collect();
            distinct.extend(assignments.iter().map(|(b, _)| *b));
            if distinct.len() > cap as usize {
                return Err(block_reject(line!(), GameError::CannotBlock(first)));
            }
        }
        // Mirri's "can't block with more than one creature this combat": the
        // blocking player's own distinct blockers.
        if let Some(&(first, _)) = assignments.first()
            && let Some(owner) = self.battlefield_find(first).map(|c| c.controller)
            && let Some(cap) = self.players[owner].block_cap_this_combat
        {
            let mut distinct: crate::game::types::SmallIdSet<CardId> = self
                .block_map
                .keys()
                .copied()
                .filter(|b| self.battlefield_find(*b).is_some_and(|c| c.controller == owner))
                .collect();
            distinct.extend(assignments.iter().map(|(b, _)| *b));
            if distinct.len() > cap as usize {
                return Err(block_reject(line!(), GameError::CannotBlock(first)));
            }
        }
        for &(blocker_id, attacker_id) in &assignments {
            let taken = batch_blocks.entry_or_default(blocker_id);
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
                .find_by_id(blocker_id)
                .ok_or(GameError::CardNotOnBattlefield(blocker_id))?;

            // CR 509.1a: any creature the defending player (or, in team
            // formats, a teammate) controls may block.
            if !self.same_team(blocker.controller, defender_idx) {
                return Err(block_reject(line!(), GameError::BlockerWrongDefender { blocker: blocker_id }));
            }

            // CR 509.1a/b — everything about the blocker alone, in the one
            // walker the bot's planner and the requirement loops also read.
            // See `blocker_self_block` for what each of them used to miss.
            if let Some((line, e)) = self.blocker_self_block(blocker, cp_of(blocker_id)) {
                return Err(block_reject(line, e));
            }

            // CR 509.1b — everything about this pair, in the walker the
            // planner and requirement loops share; see `blocker_pair_block`.
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
            let mut all_blockers: crate::game::types::SmallIdSet<CardId> =
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

        // CR 509.1d — block tax (Archangel of Tithes): sum every active
        // `BlockTaxToController` per blocker (an `only_while_attacking` source
        // counts only while attacking; Heat Wave taxes only nonblue blockers;
        // some charge life) and pay from each controller's pool, auto-tapping
        // for any shortfall. Reject the whole declaration if a player can't
        // pay. The spend waits until every legality check passes, so a
        // rejected declaration never costs mana.
        let mut block_tax_by_controller: crate::game::types::SmallIdMap<usize, (u32, u32)> =
            Default::default();
        for &(blocker_id, attacker_id) in &assignments {
            let (mana, life) = self.block_tax_for(blocker_id);
            let mana = mana + self.enchanted_attacker_block_tax(attacker_id);
            if mana == 0 && life == 0 {
                continue;
            }
            if let Some(b) = self.battlefield_find(blocker_id) {
                let e = block_tax_by_controller.entry_or_default(b.controller);
                e.0 += mana;
                e.1 += life;
            }
        }

        // CR 702.110b — Menace: blocked by 2+ creatures or not at all. Counts
        // the merged block set so multi-defender submissions compose.
        for (atk, reqs) in self.attacking.iter().zip(atk_reqs.iter()) {
            if reqs.menace {
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
            let mut owed: crate::game::types::SmallIdMap<usize, u32> = Default::default();
            for &(blocker_id, _) in &assignments {
                let Some(seat) = self.battlefield_find(blocker_id).map(|c| c.controller) else {
                    continue;
                };
                *owed.entry_or_default(seat) +=
                    self.attack_block_keyword_tax(blocker_id, kws_of(blocker_id), false);
            }
            for (seat, amount) in owed {
                // Nothing owed, nothing to pay: a `{0}` auto-tap payment is a
                // board snapshot for nothing (PERF `(-223)`).
                if amount == 0 {
                    continue;
                }
                let tax = crate::mana::cost(&[crate::mana::generic(amount)]);
                if self.try_pay_with_auto_tap_queued(seat, &tax).is_err() {
                    return Err(block_reject(line!(), GameError::CannotBlock(assignments[0].0)));
                }
            }
        }

        // CR 509.1b — the blocking half of Hollow Warrior's tap cost; helpers
        // may be neither an attacker nor a declared blocker. Presence-gated
        // like the attack side so `declared` isn't built on every declaration.
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
            for &id in &tapped {
                if let Some(c) = self.battlefield_find_mut(id) {
                    c.tapped = true;
                }
            }
        }

        // "Can't be blocked except by N or more creatures" (Pathrazer of
        // Ulamog). Generalized Menace: 0 or >= N blockers, never 1..N-1.
        for (atk, reqs) in self.attacking.iter().zip(atk_reqs.iter()) {
            if reqs.except_by_n > 0 {
                let blocker_count = assignments
                    .iter()
                    .filter(|(_, aid)| *aid == atk.attacker)
                    .count()
                    + self.blocker_count_of(atk.attacker);
                if blocker_count > 0 && (blocker_count as u32) < reqs.except_by_n {
                    return Err(block_reject(line!(), GameError::MenaceRequiresTwoBlockers(atk.attacker)));
                }
            }
        }

        // CR 509.1g — "can't be blocked by more than one creature" (Charging
        // Rhino). At most one blocker may be assigned (the inverse of Menace).
        for (atk, reqs) in self.attacking.iter().zip(atk_reqs.iter()) {
            if reqs.no_more_than_one {
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

        // CR 509.1c — requirements are obeyed only as far as restrictions
        // allow: an attacker that can't be blocked by more than one creature
        // and already has one takes no more, whatever a Lure asks (granted on
        // Nessian Boar, 8-seat pod, seed 791012).
        let full = |a: CardId| {
            self.attacking.iter().zip(atk_reqs.iter()).any(|(x, r)| x.attacker == a && r.no_more_than_one)
                && self.blocker_count_of(a) + assignments.iter().filter(|(_, x)| *x == a).count() >= 1
        };

        // CR 509.1c — "must be blocked if able" (Lure / Academic Dispute):
        // reject if such an attacker is unblocked while the defender has an
        // idle creature that could block it. Uses the merged block set so
        // multiplayer submissions compose. Full maximization across several
        // simultaneous requirements is approximated.
        for (atk, reqs) in self.attacking.iter().zip(atk_reqs.iter()) {
            if !reqs.must_be_blocked || !self.block_requirement_binds(atk.attacker) {
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
                    && for_declarer(b.controller)
                    && !self.is_blocking(b.id)
                    && !assignments.iter().any(|(bid, _)| *bid == b.id)
                    && self.block_requirement_able(b, atk.attacker)
                    && !self.block_spoken_for_elsewhere(b, atk.attacker, &assignments)
            });
            if idle_able_blocker && !self.block_cap_shuts_out(&assignments) {
                return Err(block_reject(line!(), GameError::MustBeBlockedIfAble(atk.attacker)));
            }
        }

        // CR 509.1c — "must be blocked by N or more creatures if able" (Gorm
        // the Great): short of N while enough idle able bodies remain to reach
        // N is rejected; when N can't be reached the requirement can't be met.
        if self.board_keyword_matching(|k| matches!(k, Keyword::MustBeBlockedByAtLeast(_))) {
            for atk in self.attacking.iter() {
                let Some(n) = self.must_be_blocked_by_at_least(atk.attacker) else { continue };
                if !self.block_requirement_binds(atk.attacker) || full(atk.attacker) {
                    continue;
                }
                let Some(defender_idx) = self.defender_for(atk.target) else { continue };
                let on = self.blocker_count_of(atk.attacker)
                    + assignments.iter().filter(|(_, aid)| *aid == atk.attacker).count();
                if on >= n {
                    continue;
                }
                let idle = self
                    .battlefield
                    .iter()
                    .filter(|b| {
                        self.same_team(b.controller, defender_idx)
                    && for_declarer(b.controller)
                            && !self.is_blocking(b.id)
                            && !assignments.iter().any(|(bid, _)| *bid == b.id)
                            && self.block_requirement_able(b, atk.attacker)
                            && !self.block_spoken_for_elsewhere(b, atk.attacker, &assignments)
                    })
                    .count();
                if on + idle >= n && !self.block_cap_shuts_out(&assignments) {
                    return Err(block_reject(line!(), GameError::MustBeBlockedIfAble(atk.attacker)));
                }
            }
        }

        // CR 509.1c — "must be blocked by a [filter] if able" (Ace's Baseball
        // Bat). Met by one matching blocker on it; rejected while it has none
        // and an idle matching creature could block it.
        for atk in self.attacking.iter() {
            let Some(filter) = self.must_be_blocked_by(atk.attacker) else { continue };
            if !self.block_requirement_binds(atk.attacker) {
                continue;
            }
            let Some(defender_idx) = self.defender_for(atk.target) else { continue };
            let matches = |b: CardId| {
                self.evaluate_requirement_static(&filter, &Target::Permanent(b), defender_idx, None)
            };
            let met = self.battlefield.iter().any(|b| self.blocks(b.id, atk.attacker) && matches(b.id))
                || assignments.iter().any(|(bid, aid)| *aid == atk.attacker && matches(*bid));
            if met {
                continue;
            }
            let idle_able = self.battlefield.iter().any(|b| {
                self.same_team(b.controller, defender_idx)
                    && for_declarer(b.controller)
                    && !self.is_blocking(b.id)
                    && !assignments.iter().any(|(bid, _)| *bid == b.id)
                    && matches(b.id)
                    && self.block_requirement_able(b, atk.attacker)
                    && !self.block_spoken_for_elsewhere(b, atk.attacker, &assignments)
            });
            if idle_able && !self.block_cap_shuts_out(&assignments) {
                return Err(block_reject(line!(), GameError::MustBeBlockedIfAble(atk.attacker)));
            }
        }

        // CR 509.1c — true Lure ("all creatures able to block this do so").
        // Every idle defender creature that *can* legally block such an
        // attacker must be assigned to it in the merged block set.
        for (atk, reqs) in self.attacking.iter().zip(atk_reqs.iter()) {
            if !reqs.all_must_block || !self.block_requirement_binds(atk.attacker) || full(atk.attacker) {
                continue;
            }
            let Some(defender_idx) = self.defender_for(atk.target) else { continue };
            if self.battlefield_find(atk.attacker).is_none() {
                continue;
            }
            let unmet = self.battlefield.iter().any(|b| {
                self.same_team(b.controller, defender_idx)
                    && for_declarer(b.controller)
                    && self.block_requirement_able(b, atk.attacker)
                    // Able to block it but not assigned to it (here or earlier).
                    && !self.blocks(b.id, atk.attacker)
                    && !assignments.iter().any(|(bid, aid)| *bid == b.id && *aid == atk.attacker)
                    && !self.block_spoken_for_elsewhere(b, atk.attacker, &assignments)
            });
            if unmet && !self.block_cap_shuts_out(&assignments) {
                return Err(block_reject(line!(), GameError::MustBeBlockedIfAble(atk.attacker)));
            }
        }

        // CR 702.39 — Provoke: a creature provoked this combat must block
        // that attacker if able. Provoke untapped it, so "able" is the normal
        // can-block check.
        for b in &self.battlefield {
            let Some(required) = b.must_block else { continue };
            if !for_declarer(b.controller) {
                continue;
            }
            // The provoker must still be attacking for the requirement to
            // bind, and a CR 509.1b count it cannot reach un-binds it too.
            if !self.attacking.iter().any(|a| a.attacker == required)
                || !self.block_requirement_binds(required)
                || (full(required) && !assignments.iter().any(|(bid, aid)| *bid == b.id && *aid == required))
            {
                continue;
            }
            if !self.block_requirement_able(b, required) {
                continue;
            }
            let assigned = self.blocks(b.id, required)
                || assignments.iter().any(|(bid, aid)| *bid == b.id && *aid == required);
            if !assigned && !self.block_spoken_for_elsewhere(b, required, &assignments) && !self.block_cap_shuts_out(&assignments) {
                return Err(block_reject(line!(), GameError::MustBeBlockedIfAble(required)));
            }
        }

        // CR 509.1b — Tromokratis: once such an attacker is blocked at all,
        // *every* untapped defending creature able to block it must also be
        // assigned to it. Checked against the merged block set.
        for (atk, reqs) in self.attacking.iter().zip(atk_reqs.iter()) {
            if !reqs.unless_all_block {
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
                    && for_declarer(b.controller)
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

        // CR 509.1c — "blocks each combat if able" (`MustBlock`): a creature
        // with it that can block a declared attacker must block one. Unlike the
        // loops above it asks the computed view before the deciding keyword,
        // so it takes the gate directly.
        let must_block_scan: &[crate::card::CardInstance] =
            if block_requirement { &self.battlefield } else { &[] };
        for b in must_block_scan {
            // One `kws_of` for the pair: it is a linear `find` over the
            // subset, so asking twice was two finds (PERF `(-337)`).
            if !for_declarer(b.controller) {
                continue;
            }
            let bkw = kws_of(b.id);
            if !(bkw.has_kw(&Keyword::MustBlock) || bkw.has_kw(&Keyword::MustAttackOrBlock)) {
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
                    && !full(atk.attacker)
            });
            if could_block && !self.block_cap_shuts_out(&assignments) {
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
                        self.try_pay_with_auto_tap_queued(player, &cost).is_ok()
                    });
                if !ok {
                    for (p, s) in snapshots {
                        self.restore_payment_state(p, s);
                    }
                    for &(p, n) in &life_paid {
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

        // Flanking (CR 702.25), Bushido (CR 702.45), Rampage (CR 702.23) P/T
        // deltas on block declaration, snapshotted before mutating. Only the
        // assignments' participants are read, so compute just those.
        let mut ids: SmallVec<[CardId; 8]> = SmallVec::new();
        for &(b, a) in &assignments {
            if !ids.contains(&b) {
                ids.push(b);
            }
            if !ids.contains(&a) {
                ids.push(a);
            }
        }
        // Only when some permanent can carry one of the three (PERF
        // `(-245)` context); `false` is authoritative for every computed
        // keyword set (see `board_keyword_in_scope`), so an empty list reads
        // exactly as the views would have.
        let pt_keyword_in_scope = self
            .board_keyword_matching(|k| matches!(k, Keyword::Flanking | Keyword::Bushido(_) | Keyword::Rampage(_)));
        let computed: SmallVec<[(CardId, Vec<Keyword>); 8]> = if pt_keyword_in_scope {
            self.with_frozen_layers(|g| {
                ids.iter()
                    .map(|&id| {
                        let kws = g
                            .computed_permanent(id)
                            .map(|c| c.keywords().to_vec())
                            .unwrap_or_default();
                        (id, kws)
                    })
                    .collect()
            })
        } else {
            SmallVec::new()
        };
        let kws_for = |id: CardId| -> &[Keyword] {
            computed.iter().find(|(i, _)| *i == id).map_or(&[][..], |(_, k)| k)
        };
        let sum_n = |kws: &[Keyword], pick: fn(&Keyword) -> Option<i32>| -> i32 {
            kws.iter().filter_map(pick).sum()
        };
        let mut pt_deltas: SmallVec<[(CardId, i32); 8]> = SmallVec::new();
        let mut blocked: crate::game::types::SmallIdMap<CardId, usize> = Default::default();
        for &(b, a) in &assignments {
            *blocked.entry_or_default(a) += 1;
            let bk = kws_for(b);
            let ak = kws_for(a);
            // Flanking: nonflanking blocker shrinks once per flanking instance.
            let mut flank = ak.iter().filter(|k| **k == Keyword::Flanking).count() as i32;
            if flank > 0 {
                flank += self.extra_flanking_instances(a);
            }
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
            // The flag is a hot `CardInstance` bool; the pair goes to the
            // game-level log below, not on the card — a per-blocker list there
            // cost a `CardData` copy per block on a simulation clone (PERF `(-285)`).
            if let Some(b) = self.battlefield_find_mut(blocker_id) {
                b.blocked_this_turn = true;
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
                    let n = u32::from(self.block_poison_this_turn);
                    self.add_poison(ctrl, n, &mut events);
                }
            }
        }
        for &(id, d) in &pt_deltas {
            if let Some(c) = self.battlefield_find_mut(id) {
                c.pump(d, d);
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
                    // CR 702.68 — Frenzy N: an unblocked attacker gets +N/+0.
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
        for &(id, d) in &frenzy_deltas {
            if let Some(c) = self.battlefield_find_mut(id) {
                c.pump(d, 0);
            }
        }
        // CR 506.4 / 509.1h — a blocker its own block cost killed (Wall of
        // Roots paying Archangel of Tithes' tax at 0/1) was declared, so its
        // attacker is blocked, but it leaves combat.
        let gone: SmallVec<[CardId; 2]> =
            self.block_map.keys().copied().filter(|&b| self.battlefield_find(b).is_none()).collect();
        for b in gone {
            self.remove_from_combat(b);
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

    /// Can any combat participant's computed keywords carry first or double
    /// strike, answered without gathering? `false` is authoritative; `true`
    /// only means [`has_first_strikers`](Self::has_first_strikers) must gather.
    ///
    /// [`card_keyword_possible`] per participant, with the board-wide grant
    /// scan hoisted out of the loop. Lets a board with no possible first
    /// striker skip the first-strike step without a gather.
    ///
    /// [`card_keyword_possible`]: crate::game::GameState::card_keyword_possible
    fn first_strike_possible(&self) -> bool {
        let strikes =
            |k: &Keyword| matches!(k, Keyword::FirstStrike | Keyword::DoubleStrike);
        let printed = |id: CardId| {
            self.battlefield_find(id).is_some_and(|c| {
                c.definition.keywords.iter().any(strikes)
                    || c.cold_any(|k| {
                        k.granted_keywords_eot.iter().any(strikes)
                            || k.keyword_counters.iter().any(|(k, n)| *n > 0 && strikes(k))
                    })
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
    /// Every reader under that resolver looks up by id. The one whole-board
    /// consumer is [`free_division_targets`] (Butcher Orgg divides among any
    /// of the defender's creatures), so the full board is computed only when
    /// such an attacker is in combat. Tested:
    /// `butcher_orgg_divides_damage_among_defenders` fails if this falls back
    /// to the subset.
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
        // `block_map` key order is declaration order; it only picks which
        // permanents are computed, readers look up by id, and the one that
        // collects (`free_division_targets`) sorts.
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
        let mut events = Vec::new();
        self.resolve_first_strike_damage_into(&mut events)?;
        Ok(events)
    }

    /// [`resolve_first_strike_damage`](Self::resolve_first_strike_damage)
    /// into the caller's buffer — `advance_step`'s recycled one (PERF `(-225)`).
    pub fn resolve_first_strike_damage_into(
        &mut self,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let computed = self.combat_damage_computed();
        // CR 510.4: in the first-strike combat damage step, only creatures
        // with first strike or double strike deal combat damage. The same
        // gate applies to attackers (who deals?) and blockers (who strikes
        // back at the attacker?).
        let fs_or_ds = |kws: &[Keyword]| {
            kws.has_kw(&Keyword::FirstStrike) || kws.has_kw(&Keyword::DoubleStrike)
        };
        self.resolve_combat_damage_with_filter(&computed, fs_or_ds, fs_or_ds, events)?;
        // Suspended on a `wants_ui` player's combat-damage choice — no damage
        // has been dealt yet; `submit_decision` re-enters this step.
        if self.pending_decision.is_some() {
            return Ok(());
        }
        self.check_state_based_actions_into(events);
        events.push(GameEvent::FirstStrikeDamageResolved);
        Ok(())
    }

    pub fn resolve_combat(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let mut events = Vec::new();
        self.resolve_combat_into(&mut events)?;
        Ok(events)
    }

    /// [`resolve_combat`](Self::resolve_combat) into the caller's buffer —
    /// `advance_step`'s recycled one (PERF `(-225)`), so the damage step's
    /// `reserve(32)` lands on capacity the scratch already has.
    pub fn resolve_combat_into(&mut self, events: &mut Vec<GameEvent>) -> Result<(), GameError> {
        if self.combat_damage_dealt() {
            return Ok(());
        }
        let computed = self.combat_damage_computed();
        // CR 510.5: in the regular step, everyone who didn't deal first-strike
        // damage deals now — no first strike, plus double strikers.
        let regular_or_ds = |kws: &[Keyword]| {
            !kws.has_kw(&Keyword::FirstStrike) || kws.has_kw(&Keyword::DoubleStrike)
        };
        self.resolve_combat_damage_with_filter(&computed, regular_or_ds, regular_or_ds, events)?;

        // Suspended on a `wants_ui` player's combat-damage choice — no damage
        // dealt yet; combat is not torn down. `submit_decision` re-enters.
        if self.pending_decision.is_some() {
            return Ok(());
        }

        self.check_state_based_actions_into(events);
        self.clear_combat_damage_plan();
        self.set_combat_damage_dealt(true);
        // CR 511.3 — attackers and blockers stay in combat through the end of
        // combat step; `remove_all_from_combat` runs as that step ends.
        events.push(GameEvent::CombatResolved);
        Ok(())
    }

    /// CR 511.3 — "As the end of combat step ends, all creatures, battles,
    /// and planeswalkers are removed from combat." Also CR 725.1b's
    /// end-the-turn teardown.
    pub(crate) fn remove_all_from_combat(&mut self) {
        if !self.attack_player_requirements.is_empty() {
            self.attack_player_requirements.clear();
        }
        if !self.attacking.is_empty() {
            self.attacking.clear();
        }
        if !self.sneak_defenders.is_empty() {
            self.sneak_defenders.clear();
        }
        // CR 508.5 — the record lasts only to end of combat, even if all its
        // attackers died earlier (a stolen Saheeli kept the old defender,
        // 4-seat dflt pod seed 4400396).
        if !self.attacked_permanent_defenders.is_empty() {
            self.attacked_permanent_defenders.clear();
        }
        // Dropped, not cleared — a cleared `HashMap` keeps its table and
        // every later `GameState::clone` re-allocates it (see `resolve_effect`'s
        // per-resolution reset).
        if !self.block_map.is_empty() {
            self.block_map = Default::default();
        }
        // ⚠ Both are `ColdState` fields: an unguarded `clear` on a combat
        // boundary deep-copies the whole cold group (PERF, twenty-eighth pass).
        if !self.blocked_attackers.is_empty() {
            self.blocked_attackers.clear();
        }
        if !self.left_while_attacking.is_empty() {
            self.left_while_attacking.clear();
        }
        if !self.had_to_attack.is_empty() {
            self.had_to_attack.clear();
        }
        if !self.left_while_blocking.is_empty() {
            self.left_while_blocking.clear();
        }
        if !self.attack_bands.is_empty() {
            clear_cold!(self.attack_bands);
        }
        self.blockers_declared = false;
        self.set_combat_damage_dealt(false);
        for p in 0..self.players.len() {
            if self.players[p].block_cap_this_combat.is_some() {
                self.players[p].block_cap_this_combat = None;
            }
        }
        // CR 702.39 — provoke's "block this combat" requirement ends here.
        // ⚠ Guarded: the write is a `DerefMut` on CoW `CardData`, so clearing
        // an already-`None` field deep-copies the battlefield.
        for c in self.battlefield.iter_mut() {
            if c.must_block.is_some() {
                c.must_block = None;
            }
        }
    }

    /// CR 508.1 — the prohibitions on `atk` that read the attacked player
    /// ("creatures can't attack you" — Blazing Archon, filtered Moat-style
    /// locks; Island Sanctuary; Arboria). `Some(line)` names the first that
    /// bars it. Shared by `declare_attackers` and the bot's picker.
    pub(crate) fn attack_target_prohibited(&self, atk: &Attack, statics: u32) -> Option<u32> {
        let (defender, at_planeswalker) = match atk.target {
            crate::game::types::AttackTarget::Player(d) => (Some(d), false),
            crate::game::types::AttackTarget::Planeswalker(pw) => {
                (self.battlefield_find(pw).map(|c| c.controller), true)
            }
            crate::game::types::AttackTarget::Battle(b) => {
                (self.battlefield_find(b).and_then(|c| c.protected_by), false)
            }
        };
        let d = defender?;
        // CR 508.1 — Island Sanctuary's shield bars attacks on the player.
        if matches!(atk.target, crate::game::types::AttackTarget::Player(_))
            && self.attacker_barred_by_shield(atk.attacker, d)
        {
            return Some(line!());
        }
        // CR 508.1 — Arboria: a player who did nothing on their own last
        // turn can't be attacked at all.
        if !self.acted_on_their_last_turn(d)
            && self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    self.static_active_is(&sa.effect, c, |e| matches!(e, crate::effect::StaticEffect::
                            PlayersCantBeAttackedUnlessTheyActedLastTurn))
                })
            })
        {
            return Some(line!());
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
        // Each live prohibition asked in one short-circuiting pass, with no
        // collected `(id, filter)` list or filter clone.
        let barred = statics & attack_static::CANT_ATTACK_CONTROLLER != 0
            && self
            .battlefield
            .iter()
            .filter(|c| c.controller == d)
            .flat_map(|c| c.definition.static_abilities.iter().map(move |sa| (c, sa)))
            .filter_map(|(c, sa)| {
                self.active_static_where(&sa.effect, c, |e| {
                    matches!(e, crate::effect::StaticEffect::CreaturesCantAttackController { .. })
                })
                .map(|e| (c.id, e))
            })
            .any(|(src, e)| match e {
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
            return Some(line!());
        }
        None
    }

    /// CR 802 / 803 / 801.3 — the seats `seat` may legally declare attacks
    /// against right now. Empty when it isn't `seat`'s combat, and narrowed to
    /// one entry (or none) by the attack-left / attack-right option; under a
    /// limited range of influence, only opponents inside `seat`'s range.
    pub fn attackable_players_for(&self, seat: usize) -> Vec<usize> {
        if self.active_player_idx != seat || self.cast_bars_attacks(seat) {
            return Vec::new();
        }
        let restriction = self.attack_left_right_defender();
        (0..self.players.len()).filter(|&p| self.player_open_to_attack(seat, p, restriction)).collect()
    }

    /// CR 506.2 / 508.1b — `attacker` may be declared at planeswalker
    /// `pw_id` by `p`. One walker for the declaration and the pod bot's
    /// settle pass. `restriction` is [`Self::attack_left_right_defender`].
    pub(crate) fn planeswalker_attackable(
        &self,
        p: usize,
        pw_id: CardId,
        attacker: CardId,
        restriction: Option<Option<usize>>,
    ) -> bool {
        let Some(pw) = self.battlefield_find(pw_id) else { return false };
        !self.sentinel_bound(attacker) && self.walker_open_to(p, pw, restriction)
    }

    /// [`Self::planeswalker_attackable`] short of the attacker's own bar: the
    /// walkers the bot's pickers may aim at. Reads the *computed* type, so
    /// Luxior's "isn't a planeswalker" closes it (6-seat pod, seed 808174).
    pub(crate) fn walker_open_to(&self, p: usize, pw: &CardInstance, restriction: Option<Option<usize>>) -> bool {
        self.computed_has_card_type(pw, crate::card::CardType::Planeswalker)
            // CR 506.2 — "can't be attacked" (The Aetherspark while attached
            // to a creature).
            && !self.permanent_cant_be_attacked(pw.id)
            && !self.same_team(self.active_player_idx, pw.controller)
            && self.players.get(pw.controller).is_some_and(|pl| pl.is_alive())
            && restriction.is_none_or(|only| only == Some(pw.controller))
            && self.player_in_range_of(p, pw.controller)
            && self.seat_attackable_from(p, pw.controller)
    }

    /// CR 508.1 — Angelic Arbiter: `seat` cast a spell this turn and an
    /// opponent's static bars its creatures from attacking at all.
    pub(crate) fn cast_bars_attacks(&self, seat: usize) -> bool {
        self.players.get(seat).is_some_and(|pl| pl.spells_cast_this_game_turn > 0)
            && self.opponent_has_static(seat, |e| {
                matches!(e, crate::effect::StaticEffect::OpponentsWhoCastCantAttack)
            })
    }

    /// One seat of [`Self::attackable_players_for`]: `defender` is a live
    /// opponent `seat` may declare attacks against at all. `restriction` is
    /// [`Self::attack_left_right_defender`], taken once by the caller.
    pub(crate) fn player_open_to_attack(
        &self,
        seat: usize,
        defender: usize,
        restriction: Option<Option<usize>>,
    ) -> bool {
        self.players.get(defender).is_some_and(|pl| pl.is_alive())
            && !self.same_team(seat, defender)
            && restriction.is_none_or(|only| only == Some(defender))
            && self.player_in_range_of(seat, defender)
            && self.seat_attackable_from(seat, defender)
            && !self.player_cant_be_attacked_at_all(seat, defender)
    }

    /// [`Self::attack_left_right_defender`] for callers outside this module.
    pub(crate) fn attack_restriction(&self) -> Option<Option<usize>> {
        self.attack_left_right_defender()
    }

    /// CR 508.1 — `defender` can't be attacked by `seat` whatever creature is
    /// declared: an unfiltered "creatures can't attack you" (Blazing Archon),
    /// Arboria, or a turn-scoped Web of Inertia ban. Attacker-filtered
    /// prohibitions are left to `declare_attackers`.
    pub(crate) fn player_cant_be_attacked_at_all(&self, seat: usize, defender: usize) -> bool {
        use crate::effect::StaticEffect;
        if self.cant_attack_player_this_turn.contains(&(seat, defender))
            || self.next_turn_attack_banned(seat, defender)
        {
            return true;
        }
        self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| {
                match self.active_static_where(&sa.effect, c, |e| matches!(e, StaticEffect::CreaturesCantAttackController { .. } | StaticEffect::PlayersCantBeAttackedUnlessTheyActedLastTurn)) {
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
            // Mystic Barrier imposes the same shape on a free-for-all.
            crate::game::AttackOption::MultiplePlayers => return self.barrier_defender(),
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

    /// CR 702.15 — does `defender` control a land with the given land type?
    /// Reads printed land subtypes, so dual lands and nonbasics count.
    pub(crate) fn defender_controls_land_type(
        &self,
        defender: usize,
        lt: &crate::card::LandType,
    ) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller == defender && self.permanent_has_land_type(c, *lt)
        })
    }

    /// CR 614 — is an extra turn `seat` would begin replaced with a skip?
    /// `OpponentsSkipExtraTurns` (Trouble in Pairs) on a permanent another
    /// seat controls ("if an opponent would begin an extra turn"), or
    /// `PlayersSkipExtraTurns` (Gerrard's Hourglass Pendant), which binds
    /// every seat. Printed statics only.
    pub fn extra_turn_denied_for(&self, seat: usize) -> bool {
        use crate::effect::StaticEffect as SE;
        self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| match sa.effect {
                SE::OpponentsSkipExtraTurns => c.controller != seat,
                SE::PlayersSkipExtraTurns => true,
                _ => false,
            })
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

    /// Validate a `DamageOrder` answer into a blocker order. Omitted ids keep
    /// their default position and unknown ids are ignored, so any answer is
    /// legal.
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

    /// CR 510.1c-d — divide an attacker's `total_power` among its blockers
    /// (in assignment order, with their `lethal`). Returns `(blocker_id,
    /// amount)` pairs; unassigned power is the trample-over leftover.
    ///
    /// The default (and `AutoDecider`) split assigns lethal in order. A
    /// `wants_ui` / scripted decider may answer `CombatDamageAssignment` to
    /// over-assign (e.g. deny trample) under CR 510.1c: a blocker gets damage
    /// only once every earlier one has lethal, and the total can't exceed
    /// `total_power`. A malformed answer falls back to the default.
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
        // CR 510.1d — all damage must be assigned: without trample the excess
        // goes to the blockers (default: the last one); with trample it's left
        // for the trample-over path (CR 702.19g).
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

    /// CR 510.1c-d — gather (and cache) the active player's damage ordering
    /// and assignment for every multi-blocker attacker, before any damage.
    /// Returns `true` if it suspended on a `wants_ui` decision (caller returns
    /// and re-enters the step after the answer). Writes only the decision
    /// caches and `pending_decision`.
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
            clear_cold!(self.combat_assigns_unblocked);
            self.combat_damage_plan_step = Some(self.step);
        }
        let active = self.active_player_idx;
        for atk in attacker_infos.iter().filter(|a| a.should_deal) {
            let blocker_ids = self.blockers_of(atk.id);
            // Outmaneuver: nothing to order or divide over.
            if atk.must_assign_as_unblocked && !blocker_ids.is_empty() {
                continue;
            }
            let asks_unblocked = atk.may_assign_as_unblocked && !blocker_ids.is_empty();
            // A free divider (Butcher Orgg) always gets an assignment choice —
            // it divides over the defending player's creatures, not its
            // blockers, so the multi-blocker gate doesn't apply.
            let free_divider =
                !self.free_division_targets(atk.id, atk.free_divider, computed).is_empty();
            if blocker_ids.len() <= 1 && !free_divider && !asks_unblocked {
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
            let assigner_ui = self.seat_prompts(assigner);

            // Thorn Elemental's "as though it weren't blocked" (CR 510.1c) is
            // chosen at assignment by whoever assigns (the defender against
            // banding, per the ruling); yes leaves nothing to order or divide.
            if asks_unblocked {
                let answered =
                    self.combat_assigns_unblocked.iter().find(|(id, _)| *id == atk.id).map(|(_, b)| *b);
                let yes = match answered {
                    Some(b) => b,
                    None => {
                        let decision = crate::decision::Decision::OptionalTrigger {
                            source: atk.id,
                            description: "Assign its combat damage as though it weren't blocked?".into(),
                            kind: crate::decision::OptionalKind::Neutral,
                        };
                        if assigner_ui {
                            self.pending_decision = Some(Box::new(PendingDecision {
                                decision,
                                resume: ResumeContext::CombatDamage {
                                    player: assigner,
                                    attacker: atk.id,
                                    kind: CombatDecisionKind::AsThoughUnblocked,
                                },
                            }));
                            return true;
                        }
                        let b = matches!(
                            self.decider.decide(&decision),
                            crate::decision::DecisionAnswer::Bool(true)
                        );
                        self.combat_assigns_unblocked.push((atk.id, b));
                        b
                    }
                };
                if yes || (blocker_ids.len() <= 1 && !free_divider) {
                    continue;
                }
            }

            // 1) Blocker order (CR 510.1c) — a free divider has no order to
            // announce, so it goes straight to the assignment.
            if !self.combat_damage_order.contains_key(&atk.id) && !free_divider {
                let decision = self.combat_damage_order_decision(atk.id, &blocker_ids);
                if assigner_ui {
                    self.pending_decision = Some(Box::new(PendingDecision {
                        decision,
                        resume: ResumeContext::CombatDamage {
                            player: assigner,
                            attacker: atk.id,
                            kind: CombatDecisionKind::Order,
                        },
                    }));
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
                    atk.free_divider,
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
                    self.pending_decision = Some(Box::new(PendingDecision {
                        decision,
                        resume: ResumeContext::CombatDamage {
                            player: assigner,
                            attacker: atk.id,
                            kind: CombatDecisionKind::Assign,
                        },
                    }));
                    return true;
                }
                let answer = self.decider.decide(&decision);
                let split =
                    self.resolve_damage_assignment(total_power, &lethals, trample, &answer);
                self.combat_damage_assignment.insert(atk.id, split);
            }
        }

        // CR 509.2 / 510.1e — the mirror for a creature blocking several
        // attackers: the defending player orders and divides its damage.
        // Cached in the same maps keyed by the blocker's id; a creature is
        // never both attacker and blocker, so keys don't collide.
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
            let assigner_ui = self.seat_prompts(assigner);
            let deathtouch = bcp.keywords().has_kw(&Keyword::Deathtouch);
            let total_power = combat_damage_value(bcp).max(0) as u32;

            if !self.combat_damage_order.contains_key(&bid) {
                let default_order = blocked.clone();
                let decision = self.combat_damage_order_decision(bid, &default_order);
                if assigner_ui {
                    self.pending_decision = Some(Box::new(PendingDecision {
                        decision,
                        resume: ResumeContext::CombatDamage {
                            player: assigner,
                            attacker: bid,
                            kind: CombatDecisionKind::Order,
                        },
                    }));
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
                    self.pending_decision = Some(Box::new(PendingDecision {
                        decision,
                        resume: ResumeContext::CombatDamage {
                            player: assigner,
                            attacker: bid,
                            kind: CombatDecisionKind::Assign,
                        },
                    }));
                    return true;
                }
                let answer = self.decider.decide(&decision);
                let split = self.resolve_damage_assignment(total_power, &lethals, false, &answer);
                self.combat_damage_assignment.insert(bid, split);
            }
        }
        false
    }

    /// CR 510.1e — the combat damage `blocker` assigns to `attacker`: its
    /// whole power for a single block, otherwise the cached split (default
    /// lethal in declaration order).
    fn blocker_damage_to(&self, blocker: CardId, attacker: CardId, power: u32) -> u32 {
        if self.attackers_blocked_by(blocker).len() <= 1 {
            return power;
        }
        self.combat_damage_assignment
            .get(&blocker)
            .and_then(|split| split.iter().find(|(a, _)| *a == attacker).map(|(_, n)| *n))
            .unwrap_or(0)
    }

    /// CR 702.22 — the attacking bands declared this combat, for the server
    /// view. Empty when nothing banded.
    pub fn attack_bands_view(&self) -> Vec<Vec<CardId>> {
        self.attack_bands.clone()
    }

    /// Butcher Orgg — "you may assign this creature's combat damage divided as
    /// you choose among defending player and/or any number of creatures they
    /// control". Returns every creature the defending player controls
    /// (id-ordered), or empty when the attacker can't divide.
    ///
    /// `free_divider` is the attacker's own `DividesCombatDamageAmongDefenders`,
    /// read by the caller off its `ComputedPermanent` rather than a
    /// whole-board `any()` here (PERF `(-113)`).
    fn free_division_targets(
        &self,
        attacker: CardId,
        free_divider: bool,
        computed: &[ComputedPermanent],
    ) -> Vec<CardId> {
        if !free_divider {
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
        free_divider: bool,
        order: &[CardId],
        computed: &[ComputedPermanent],
    ) -> (Lethals, bool) {
        let free = self.free_division_targets(attacker, free_divider, computed);
        if free.is_empty() {
            (self.combat_lethals(attacker_deathtouch, order, computed), has_trample)
        } else {
            (free.into_iter().map(|id| (id, 0)).collect(), true)
        }
    }

    /// Lethal damage required for each blocker in `order` (its toughness, or 1
    /// under deathtouch per CR 702.2e). Blockers no longer on the battlefield
    /// resolve to 0.
    fn combat_lethals(
        &self,
        attacker_deathtouch: bool,
        order: &[CardId],
        computed: &[ComputedPermanent],
    ) -> Lethals {
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
            CombatDecisionKind::AsThoughUnblocked => {
                let yes = matches!(answer, crate::decision::DecisionAnswer::Bool(true));
                self.combat_assigns_unblocked.push((attacker, yes));
            }
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
                // The resolver's own computed view, not the whole board: every
                // id here is a participant, and `combat_damage_computed`
                // already widens for Butcher Orgg. Also keeps the decision
                // reading the same view as the resolver that consumes it.
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
                let free_divider = atk_cp.is_some_and(|c| {
                    c.keywords().has_kw(&Keyword::DividesCombatDamageAmongDefenders)
                });
                let (lethals, trample) = self.combat_assignment_plan(
                    attacker,
                    deathtouch,
                    trample,
                    free_divider,
                    &order,
                    &computed,
                );
                let split = self.resolve_damage_assignment(total_power, &lethals, trample, answer);
                self.combat_damage_assignment.insert(attacker, split);
            }
        }
    }

    /// Clear the cached combat-damage choices at the end of a combat phase.
    pub(crate) fn clear_combat_damage_plan(&mut self) {
        self.combat_damage_order.clear();
        self.combat_damage_assignment.clear();
        clear_cold!(self.combat_assigns_unblocked);
        self.combat_damage_plan_step = None;
    }

    /// Core combat damage resolver. Each attacker has its own defending
    /// player or planeswalker (`Attack::target`); routing is per-attacker.
    /// Writes into `events`, the caller's recycled buffer (PERF `(-225)`).
    fn resolve_combat_damage_with_filter(
        &mut self,
        computed: &[ComputedPermanent],
        attacker_filter: impl Fn(&[Keyword]) -> bool,
        blocker_filter: impl Fn(&[Keyword]) -> bool,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        // Each call is one combat-damage batch (first-strike and regular
        // damage are separate sub-steps): reset the "one or more creatures
        // you control deal combat damage" graveyard-trigger dedupe.
        clear_cold!(self.combat_trigger_fired_this_step);
        // Weeping Angel's prevented-and-shuffled recipients, moved once every
        // assignment is dealt.
        let mut shuffled: Vec<(CardId, CardId)> = Vec::new();

        let computed_of =
            |id: CardId| -> Option<&ComputedPermanent> { computed.iter().find(|c| c.id == id) };

        // Inline storage (`(-71)`'s device): never leaves the frame, and a
        // `filter_map` collect can't size-hint. Eight covers an alpha strike;
        // wider spills to the heap. Not a `reserve`: it removes the
        // allocation, so `(-80)`'s floor doesn't apply.
        let all_wither = self.all_damage_is_wither_now();
        let attacker_infos: SmallVec<[AttackerInfo; 8]> = self
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
                    has_wither: all_wither || kws.has_kw(&Keyword::Wither),
                    toxic: toxic_poison_value(kws),
                    may_assign_as_unblocked: kws
                        .has_kw(&Keyword::AssignsDamageAsThoughUnblocked),
                    must_assign_as_unblocked: kws
                        .has_kw(&Keyword::MustAssignDamageAsThoughUnblocked),
                    free_divider: kws
                        .has_kw(&Keyword::DividesCombatDamageAmongDefenders),
                    // CR 510.1 — "deals no combat damage this turn" (Master of
                    // Cruelties) skips both steps; CR 614.9 — so does a
                    // Maze-of-Ith'd attacker.
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

        // PHASE 1 — gather every multi-blocker attacker's ordering and
        // assignment before any damage. For a `wants_ui` player a choice
        // surfaces as `pending_decision` and this returns; `submit_decision`
        // re-enters the step. Choices are cached in `combat_damage_order` /
        // `combat_damage_assignment` for the apply phase.
        if self.gather_combat_damage_decisions(&attacker_infos, computed, &blocker_filter) {
            return Ok(());
        }
        // Past the early returns this batch always emits an event, and the
        // buffer is shared with `deal_combat_damage_to_target` and
        // `check_state_based_actions_into`; reserving avoids the 0->4->...->32
        // growth ladder per batch.
        events.reserve(32);

        // CR 615.1 — "Prevent all combat damage this turn" (Owlin Shieldmage,
        // Holy Day): every assignment yields 0, which zeroes lifelink too (CR
        // 702.15a) and fires no "deals combat damage to a player" triggers.
        // With an exception filter (Inspire Awe) it's per-dealer, recomputed in
        // the loop below.

        // CR 614.2 / 614.5 — global doubling (Furnace of Rath) and halving
        // (Ghosts of the Innocent) scale the amount dealt, after assignment
        // and before prevention: lethal is assigned at base, trample-over and
        // player damage scale too.


        // Creature-vs-creature combat damage recorded here and dispatched after
        // all damage in this step is dealt, so `DealsCombatDamageToCreature`
        // triggers (CR 510.2) go on the stack simultaneously (CR 603.3b).
        let mut creature_damage: SmallVec<[(CardId, CardId, u32); 8]> = SmallVec::new();

        for atk in &attacker_infos {
            // Fog (per-dealer) or "prevent all combat damage it would deal"
            // (Azorius Ploy) zero this attacker's outgoing damage while its
            // blockers still strike back.
            //
            // `!should_deal` goes in the same bucket — it must not `continue`
            // past the pairing. CR 510.4/510.5 gate each creature on its own
            // keywords, so skipping the pairing for an idle attacker dropped
            // its blockers' damage too (a first-striking attacker or blocker
            // walked through untouched).
            let prevent_combat_damage = !atk.should_deal
                || self.combat_damage_prevented_for_dealer(atk.id)
                || self.combat_damage_prevented_from(atk.id);

            // CR 510.1c: the attacking player's damage order for this attacker,
            // cached in PHASE 1; default is CardId order (declaration proxy).
            let mut blocker_ids = self.blockers_of(atk.id);
            // CR 510.1a — "assigns its combat damage as though it weren't
            // blocked" (Predatory Focus): drop the blockers so the whole hit
            // lands on the player; the blockers still deal theirs below.
            // Read off `atk`, this step's one snapshot: CR 510.1 assignment is
            // a single action before any damage, so a life total this loop
            // moves must not change which attackers count as blocked (and a
            // fresh `computed_permanent` here was a gather per attacker).
            if atk.must_assign_as_unblocked
                || (atk.may_assign_as_unblocked
                    && self.combat_assigns_unblocked.contains(&(atk.id, true)))
            {
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
            let free_targets = self.free_division_targets(atk.id, atk.free_divider, computed);

            // Goblin Psychopath — a lost coin flip sends this attacker's whole
            // assignment at its controller (CR 614.9). Soltari Guerrillas — the
            // armed charge sends it at a chosen creature instead.
            if blocker_ids.is_empty()
                && free_targets.is_empty()
                && matches!(atk.target, AttackTarget::Player(_))
                && let Some(i) =
                    self.turn.next_combat_damage_redirect.iter().position(|(a, _)| *a == atk.id)
            {
                let (_, victim) = self.turn.next_combat_damage_redirect.remove(i);
                if self.battlefield_find(victim).is_some() {
                    let raw = if prevent_combat_damage { 0 } else { atk.power.max(0) as u32 };
                    self.deal_damage_to_from(
                        crate::game::effects::EntityRef::Permanent(victim),
                        raw,
                        Some(atk.id),
                        events,
                    );
                    continue;
                }
            }

            if let Some(seat) = self.take_combat_damage_diversion(atk.id) {
                let raw = if prevent_combat_damage { 0 } else { atk.power.max(0) as u32 };
                self.deal_redirected_combat_damage_to_player(seat, raw, atk.id, events);
                continue;
            }

            if blocker_ids.is_empty() && free_targets.is_empty() {
                // CR 510.1c — a blocked attacker stays blocked even if all its
                // blockers left combat: no damage without trample, all to the
                // player with it (CR 702.19g).
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
                    events,
                );
                if amount > 0 {
                    // CR 702.15b — lifelink gains for the damage actually
                    // dealt: a replacement (Crumbling Sanctuary, The
                    // Mindskinner) or Thunderstaff's shave leaves less.
                    let dealt = self.deal_combat_damage_to_target(atk, amount, events);
                    if atk.has_lifelink && dealt > 0 {
                        let a = self.active_player_idx;
                        self.adjust_life_emit(a, crate::game::life_amount(dealt), events);
                    }
                }
            } else {
                let total_power = if prevent_combat_damage {
                    0
                } else {
                    atk.power.max(0) as u32
                };
                // CR 510.1c-d — the PHASE 1 cached division among blockers, or
                // the default lethal-to-each split (single blocker, prevented,
                // or a non-UI path). Lethal accounts for marked damage and is 1
                // under deathtouch (CR 702.2e). A free divider has no lethal
                // requirement and always spills the rest to the player.
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
                        .collect::<Lethals>();
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
                    // One freeze scope over this pair's read-only prefix: every
                    // check is `&self` until `apply_prevention_shields`, so no
                    // layer input moves. It merges the scopes
                    // `damage_prevented_by_protection` and `scale_damage_to`
                    // each open, and the two `&self` reads after the shields
                    // (CR 614.9 redirect, CR 615 self-prevention) ride it too —
                    // shields consume charges and emit events but move no layer
                    // input.
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
                            // prevents its damage to the blocker. Over this
                            // frame's views, not a nested scope (PERF `(-202)`).
                            || computed_of(blocker_id).is_some_and(|bc| {
                                g.protection_prevents_views(atk.id, computed_of(atk.id), bc)
                            })
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
                            Some(g.double_creature_combat_damage(g.attached_combat_damage_doubling(
                                Some(atk.id),
                                Some(blocker_id),
                                g.scale_damage_to(
                                    Some(atk.id),
                                    crate::game::effects::EntityRef::Permanent(blocker_id),
                                    assign,
                                ),
                            )))
                        };
                        (scaled, redirect_to, self_prevented)
                    });
                    let Some(scaled) = scaled else {
                        continue;
                    };
                    // CR 615 — attacker→blocker damage through the blocker's
                    // shields. Lifelink and wither/infect counters scale off
                    // the post-prevention amount (CR 702.15a).
                    let dealt = self.apply_combat_prevention_shields(
                        crate::game::effects::EntityRef::Permanent(blocker_id),
                        scaled,
                        Some(atk.id),
                        events,
                    ) as i32;
                    // CR 122.1c — a shield counter prevents it (and goes).
                    let dealt = if dealt > 0 && self.consume_shield_counter(blocker_id) { 0 } else { dealt };
                    // Ironscale Hydra replaces the damage with a +1/+1 counter
                    // (and so the attacker's lifelink scales off 0).
                    let dealt =
                        self.ironscale_replace(blocker_id, atk.id, redirect_to, dealt, events);
                    let dealt = if dealt > 0 && self.combat_damage_shuffles_creatures(atk.id) {
                        shuffled.push((atk.id, blocker_id));
                        0
                    } else {
                        dealt
                    };
                    // CR 615 — a blocker that prevents all damage to itself
                    // (Wall of Denial) takes none, and grants no lifelink.
                    let dealt = if self_prevented { 0 } else { dealt };
                    lifelink_dealt += dealt;

                    // Karona's Zealot — a standing turn-scoped redirect moves
                    // the whole combat-damage event onto another creature.
                    let blocker_id =
                        self.turn_damage_redirect_for(blocker_id).unwrap_or(blocker_id);
                    let excess = self.note_excess_damage(blocker_id, dealt.max(0) as u32, atk.has_deathtouch, Some(atk.id));
                    self.push_excess_damage_event(blocker_id, excess, true, events);
                    if dealt > 0 && let Some(b) = self.battlefield_find_mut(blocker_id) {
                        b.dealt_damage_this_turn = true;
                        b.damage_dealt_to_this_turn += dealt.max(0) as u32;
                        b.damaged_by_this_turn.push(atk.id);
                    }
                    if atk.has_infect || atk.has_wither {
                        // CR 614.16 / 122.1 — the counters are a placement.
                        let n = if dealt > 0 {
                            self.place_counters(blocker_id, crate::card::CounterType::MinusOneMinusOne, dealt as u32)
                        } else {
                            0
                        };
                        if n > 0 {
                            events.push(GameEvent::CounterAdded {
                                card_id: blocker_id,
                                counter_type: crate::card::CounterType::MinusOneMinusOne,
                                count: n, placer: Some(atk.controller),
                            });
                        }
                        // CR 120.3d / 702.90b — the damage was still dealt (as
                        // counters): damage triggers, the per-source record and
                        // deathtouch (CR 702.2b) see it.
                        if dealt > 0 {
                            if let Some(blocker) = self.battlefield_find_mut(blocker_id) {
                                blocker.record_damage_from(atk.id, dealt as u32);
                                if atk.has_deathtouch {
                                    blocker.dealt_deathtouch_damage = true;
                                }
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
                    } else if dealt > 0
                        && let Some(blocker) = self.battlefield_find_mut(blocker_id)
                    {
                        blocker.damage = blocker.damage.saturating_add(dealt as u32);
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
                        events,
                    );
                    if amount > 0 {
                        lifelink_dealt += self.deal_combat_damage_to_target(atk, amount, events) as i32;
                    }
                }

                if atk.has_lifelink && lifelink_dealt > 0 {
                    let a = self.active_player_idx;
                    self.adjust_life_emit(a, lifelink_dealt, events);
                }

                // Only blockers whose own keywords deal in this step strike
                // back (CR 510.4/510.5): a regular blocker waits for the
                // regular step even against a first striker. One freeze scope
                // so the per-blocker protection checks share a gather.
                let (dealing_blocker_ids, attacker_takes_strike_back) =
                    self.with_frozen_layers(|g| {
                        // A `for` loop, not seven stacked `.filter()`s: on a
                        // one- or two-blocker list the adapter setup is most of
                        // the cost (PERF, eighty-seventh pass). Short-circuit
                        // order unchanged: cheapest and most selective first.
                        // Inline storage: multi-blocks of more than four don't
                        // happen and this never leaves the frame.
                        let mut ids: SmallVec<[CardId; 4]> = SmallVec::new();
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
                                // protection from deals no combat damage to it. Same
                                // held views as the attacker's side (PERF `(-202)`).
                                || computed_of(atk.id).is_some_and(|ac| {
                                    g.protection_prevents_views(bid, computed_of(bid), ac)
                                })
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
                    // CR 702.90 / 615.6 — each blocker's strike-back is its own
                    // damage event: scaling, prevention, infect/wither,
                    // deathtouch and lifelink apply per source.
                    let mut lifelink_by_controller: crate::fxhash::HashMap<usize, i32> =
                        crate::fxhash::HashMap::default();
                    for &bid in &dealing_blocker_ids {
                        let Some(bc) = computed_of(bid) else { continue };
                        // CR 510.1e — a multi-blocker assigns this attacker only
                        // its share. Same read-only prefix as the attacker side,
                        // so one scope holds both calls.
                        let (scaled, redirect_to, self_prevented) = self.with_frozen_layers(|g| {
                            let power = g.blocker_damage_to(
                                bid,
                                atk.id,
                                combat_damage_value(bc).max(0) as u32,
                            );
                            let scaled = (power != 0).then(|| {
                                g.double_creature_combat_damage(g.attached_combat_damage_doubling(
                                    Some(bid),
                                    Some(atk.id),
                                    g.scale_damage_to(
                                        Some(bid),
                                        crate::game::effects::EntityRef::Permanent(atk.id),
                                        power,
                                    ),
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
                        let dmg = self.apply_combat_prevention_shields(
                            crate::game::effects::EntityRef::Permanent(atk.id),
                            scaled,
                            Some(bid),
                            events,
                        );
                        // CR 122.1c — a shield counter prevents it (and goes).
                        let dmg = if dmg > 0 && self.consume_shield_counter(atk.id) { 0 } else { dmg };
                        // Ironscale Hydra replaces the blocker's strike-back
                        // with a +1/+1 counter (blocker's lifelink sees 0).
                        let dmg = self
                            .ironscale_replace(atk.id, bid, redirect_to, dmg as i32, events)
                            as u32;
                        let dmg = if dmg > 0 && self.combat_damage_shuffles_creatures(bid) {
                            shuffled.push((bid, atk.id));
                            0
                        } else {
                            dmg
                        };
                        // CR 615 — an attacker that prevents all damage to itself,
                        // or specifically damage from its blockers (Armored
                        // Transport), takes none from this blocker (no lifelink).
                        let dmg = if self_prevented { 0 } else { dmg };
                        if dmg == 0 {
                            continue;
                        }
                        let infect = all_wither
                            || bc.keywords().has_kw(&Keyword::Infect)
                            || bc.keywords().has_kw(&Keyword::Wither);
                        let hit = self.turn_damage_redirect_for(atk.id).unwrap_or(atk.id);
                        let excess =
                            self.note_excess_damage(hit, dmg, bc.keywords().has_kw(&Keyword::Deathtouch), Some(bid));
                        self.push_excess_damage_event(hit, excess, true, events);
                        // CR 614.16 / 122.1 — infect/wither counters are a
                        // placement, scaled before the mutable borrow.
                        let infect_n = if infect && !self.counters_locked() {
                            self.scaled_counter_count_on(hit, crate::card::CounterType::MinusOneMinusOne, dmg)
                        } else {
                            0
                        };
                        if let Some(attacker) = self.battlefield_find_mut(hit) {
                            attacker.dealt_damage_this_turn = true;
                            attacker.damage_dealt_to_this_turn += dmg;
                            attacker.damaged_by_this_turn.push(bid);
                            if infect {
                                if infect_n > 0 {
                                    attacker.add_counters(crate::card::CounterType::MinusOneMinusOne, infect_n);
                                    events.push(GameEvent::CounterAdded {
                                        card_id: hit,
                                        counter_type: crate::card::CounterType::MinusOneMinusOne,
                                        count: infect_n, placer: Some(bc.controller),
                                    });
                                }
                            } else {
                                attacker.damage = attacker.damage.saturating_add(dmg);
                            }
                            // CR 120.3d — dealt either way (as counters or as
                            // marked damage): the record, deathtouch and the
                            // event follow the damage, not its form.
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
                        // CR 510.2 — this blocker dealt combat damage to a
                        // creature (post-prevention amount).
                        creature_damage.push((bid, atk.id, dmg));
                        // CR 702.15a — lifelink scales off damage actually
                        // dealt; credited to the blocker's controller.
                        if bc.keywords().has_kw(&Keyword::Lifelink) {
                            let controller = self
                                .battlefield
                                .find_by_id(bid)
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
                            self.adjust_life_emit(player, gained, events);
                        }
                    }
                }
            }
        }

        // CR 614 — an exile-on-death damage rider (Kumano's Pupils) exiles
        // any creature it damaged this turn that would die. Mirrors the
        // non-combat path in `deal_damage_to_from`.
        for &(source, damaged, _) in &creature_damage {
            if self.damage_exiles_victim_eot.contains(&source)
                || self
                    .battlefield_find(source)
                    .is_some_and(|c| c.definition.damage_exiles_if_dies)
            {
                self.turn.dies_to_exile_eot.insert(damaged);
            }
            // Runesword — its damage also denies regeneration this turn.
            if self.damage_denies_regen_eot.contains(&source)
                && let Some(v) = self.battlefield_find_mut(damaged)
            {
                v.cant_regenerate_this_turn = true;
            }
        }
        for (source, victim) in shuffled {
            let Some(owner) = self.find_card_anywhere(victim).filter(|_| self.battlefield_find(victim).is_some()).map(|c| c.owner)
            else {
                continue;
            };
            let ctrl = self.battlefield_find(source).map_or(owner, |c| c.controller);
            let ctx = crate::game::effects::EffectContext::for_ability(source, ctrl, None);
            let dest = crate::effect::ZoneDest::Library {
                who: crate::effect::PlayerRef::Seat(owner),
                pos: crate::effect::LibraryPosition::Shuffled,
            };
            self.move_card_to(victim, &dest, &ctx, events);
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
        // CR 510.2 — all combat damage in this step is dealt; put
        // `DealsCombatDamageToCreature` triggers on the stack.
        //
        // One `trigger_grant_sources` walk for the batch, not per damaged
        // creature. Reading grants once, before any of this batch's triggers
        // are on the stack, is also the CR 510.2 order — a per-event rebuild
        // re-checked each grant's gate against a stack holding earlier ones.
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
            self.fire_combat_damage_watchers(source, amount, false);
            self.fire_combat_damage_to_creature_triggers(source, damaged, amount, &granted);
        }

        Ok(())
    }

    /// CR 506.2 — whether `id` carries a computed `Keyword::CantBeAttacked`,
    /// so it can't be declared as an attack target (The Aetherspark).
    pub(crate) fn permanent_cant_be_attacked(&self, id: CardId) -> bool {
        if self.players.get(self.active_player_idx).is_some_and(|p| !p.next_turn_attack_bans.is_empty())
            && let Some(c) = self.battlefield_find(id)
            && self.next_turn_attack_banned(self.active_player_idx, c.controller)
        {
            return true;
        }
        if !self.cant_attack_walkers_this_turn.is_empty()
            && let Some(pw) = self.battlefield_find(id)
            && self.cant_attack_walkers_this_turn.contains(&(self.active_player_idx, pw.controller))
        {
            return true;
        }
        if !self.cant_attack_pw_type_this_turn.is_empty()
            && let Some(pw) = self.battlefield_find(id)
            && self.cant_attack_pw_type_this_turn.iter().any(|(a, ctrl, t)| {
                *a == self.active_player_idx
                    && *ctrl == pw.controller
                    && pw.definition.subtypes.planeswalker_subtypes.contains(t)
            })
        {
            return true;
        }
        self.computed_permanent(id)
            .is_some_and(|cp| cp.keywords().has_kw(&Keyword::CantBeAttacked))
    }

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

    /// Combat damage a replacement sends at a player instead of its first
    /// recipient (CR 614.9 Treacherous Link, Goblin Psychopath's diversion):
    /// dealt by the attacker, through the player's prevention, and still
    /// combat damage from that commander (CR 903.10a).
    fn deal_redirected_combat_damage_to_player(
        &mut self,
        seat: usize,
        amount: u32,
        source: CardId,
        events: &mut Vec<GameEvent>,
    ) {
        let mark = events.len();
        self.deal_damage_to_from(crate::game::effects::EntityRef::Player(seat), amount, Some(source), events);
        if let Some(cmdr) = self.commander_card_of(source) {
            let landed: u32 = events[mark..]
                .iter()
                .filter_map(|e| match e {
                    GameEvent::DamageDealt { to_player: Some(q), amount, from_card: Some(f), .. }
                        if *q == seat && *f == source =>
                    {
                        Some(*amount)
                    }
                    _ => None,
                })
                .sum();
            self.record_commander_damage(seat, cmdr, landed);
        }
    }

    /// Ironscale: if `recipient` has the static and `dealt > 0`, add one
    /// +1/+1 counter, emit the event and return 0 (damage prevented);
    /// otherwise return `dealt`.
    ///
    /// `redirect_to` is [`creature_redirects_damage_to_controller`], answered
    /// by the caller inside its freeze scope — the CR 614.9 check is this
    /// function's only layer read, and it would gather on its own.
    ///
    /// [`creature_redirects_damage_to_controller`]: Self::creature_redirects_damage_to_controller
    fn ironscale_replace(
        &mut self,
        recipient: CardId,
        source: CardId,
        redirect_to: Option<usize>,
        dealt: i32,
        events: &mut Vec<GameEvent>,
    ) -> i32 {
        if dealt <= 0 {
            return dealt;
        }
        // CR 614.9 — Treacherous Link redirects combat damage too.
        if let Some(owner) = redirect_to {
            self.deal_redirected_combat_damage_to_player(owner, dealt as u32, source, events);
            return 0;
        }
        // One walk of one card's `static_abilities` answers Sekki's trade,
        // Ironscale's grow and the replace-with-counters family; this runs per
        // damage assignment to a creature.
        let (sekki, grows, replace_kind, own) = match self.battlefield_find(recipient) {
            Some(c) => {
                use crate::effect::StaticEffect as SE;
                let (mut sekki, mut grows, mut kind) = (false, false, None);
                for sa in &c.definition.static_abilities {
                    // A gated replacement (Jared Carthalion's "while you're
                    // the monarch") counts while its gate is open.
                    let Some(effect) = self.active_static(&sa.effect, c) else { continue };
                    match *effect {
                        SE::PreventDamageToSelfTradingCounters { .. }
                        | SE::PreventDamageToSelfWhileCountersForRad { .. }
                        | SE::PreventDamageToSelfOpponentGainsControl
                        | SE::PreventDamageToSelfSourceControllerDraws
                        | SE::ReplaceDamageToSelfWithSacrifices => sekki = true,
                        SE::PreventCombatDamageToSelfAndGrow => grows = true,
                        SE::ReplaceDamageToSelfWithCounters { kind: k, how } => {
                            kind.get_or_insert((k, how, c.controller));
                        }
                        _ => {}
                    }
                }
                (
                    sekki,
                    grows,
                    kind.or_else(|| self.attached_replaces_damage_with_counters(recipient))
                        .or_else(|| self.teammate_replaces_damage_with_counters(recipient)),
                    c.controller,
                )
            }
            None => return dealt,
        };
        // Sekki trades counters for tokens instead of growing (CR 615). The
        // gate is exactly `trade_counters_for_damage`'s own: without the
        // static it returns `false`, and `dealt > 0` is checked above.
        if sekki && self.trade_counters_for_damage(recipient, dealt as u32, Some(source), events) {
            return 0;
        }
        // Ironscale Hydra ("prevent that damage and put a +1/+1 counter")
        // grows by exactly one; the replace-with-counters family by the
        // amount, as its wording reads unpreventable damage (CR 615.12).
        use crate::effect::DamageToCounters as How;
        // CR 109.5 / 614.1a — the replacement's controller puts the counters.
        let (kind, how, per, placer) = if grows {
            (crate::card::CounterType::PlusOnePlusOne, How::PreventAndPut, false, own)
        } else if let Some((kind, how, placer)) = replace_kind {
            (kind, how, true, placer)
        } else {
            return dealt;
        };
        let unpreventable = how != How::Instead
            && self.damage_is_unpreventable_now(Some(source));
        let (grow, left) = damage_to_counters_split(how, unpreventable, dealt as u32);
        let grow = if per { grow } else { grow.min(1) };
        let grow = self.place_counters(recipient, kind, grow);
        if grow > 0 {
            events.push(GameEvent::CounterAdded {
                card_id: recipient,
                counter_type: kind,
                count: grow, placer: Some(placer),
            });
        }
        left as i32
    }

    /// Weeping Angel — `src` prevents its combat damage to creatures and
    /// shuffles them away instead.
    fn combat_damage_shuffles_creatures(&self, src: CardId) -> bool {
        self.battlefield_find(src).is_some_and(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, crate::effect::StaticEffect::CombatDamageToCreatureShufflesIt))
        })
    }

    /// Goblin Psychopath — "the next time this would deal combat damage this
    /// turn, it deals that damage to you instead". Consumes the charge and
    /// returns the creature's controller.
    pub(crate) fn take_combat_damage_diversion(&mut self, id: CardId) -> Option<usize> {
        let idx = self.turn.next_combat_damage_to_controller.iter().position(|c| *c == id)?;
        self.turn.next_combat_damage_to_controller.remove(idx);
        self.battlefield_find(id).map(|c| c.controller)
    }

    /// CR 614.9 — Treacherous Link: does `id` redirect damage aimed at it onto
    /// its controller? Returns that controller when it does.
    pub(crate) fn creature_redirects_damage_to_controller(&self, id: CardId) -> Option<usize> {
        let c = self.battlefield_find(id)?;
        // The first read in the resolver's per-pair freeze scope, so an
        // ungated `computed_permanent` here makes that scope gather (PERF
        // `(-81)` context). As in `damage_prevented_by_protection_inner`, the
        // presence gate answers "no" without gathering, and is skipped once
        // the gather has happened (the memo read is then cheaper).
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

    /// CR 508.1d — does an opponent of `p` control a "each opponent must
    /// attack with at least one creature" static (Seeker of Slaanesh)?
    pub(crate) fn opponent_forces_an_attack(&self, p: usize) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller != p
                && !self.same_team(p, c.controller)
                && c.definition.static_abilities.iter().any(|s| {
                    matches!(
                        s.effect,
                        crate::effect::StaticEffect::OpponentsMustAttackWithAtLeastOne
                            | crate::effect::StaticEffect::OpponentsMustAttackYouWithAtLeastOne
                    )
                })
        })
    }

    /// CR 508.1d — the opponents of `p` whose Trove of Temptation demands an
    /// attack aimed at them or their planeswalkers, in seat order.
    pub(crate) fn opponents_demanding_attack(&self, p: usize) -> smallvec::SmallVec<[usize; 4]> {
        let mut out = smallvec::SmallVec::new();
        for c in self.battlefield.iter() {
            if c.controller != p
                && !self.same_team(p, c.controller)
                && self.players.get(c.controller).is_some_and(|q| q.is_alive())
                && !out.contains(&c.controller)
                && c.definition.static_abilities.iter().any(|s| {
                    matches!(s.effect, crate::effect::StaticEffect::OpponentsMustAttackYouWithAtLeastOne)
                })
            {
                out.push(c.controller);
            }
        }
        out.sort_unstable();
        out
    }

    /// Sekki, Seasons' Guide (CR 615) — prevent `dealt` damage to `recipient`,
    /// remove that many counters, and mint that many tokens. Returns false
    /// (damage stands) when `recipient` has no such static.
    pub(crate) fn trade_counters_for_damage(
        &mut self,
        recipient: CardId,
        dealt: u32,
        source: Option<CardId>,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        if dealt == 0 {
            return false;
        }
        // Dralnu, Lich Lord — a replacement (CR 614), so it bites even when
        // damage can't be prevented: its controller sacrifices that many
        // permanents, others first, instead.
        if let Some(controller) = self.battlefield_find(recipient).and_then(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|s| matches!(s.effect, crate::effect::StaticEffect::ReplaceDamageToSelfWithSacrifices))
                .then_some(c.controller)
        }) {
            let cands = self.sacrifice_candidates(controller, &crate::card::SelectionRequirement::Any, Some(recipient));
            for id in self.auto_pick_sacrifices(&cands, dealt as usize, Some(recipient), false, false) {
                self.sacrifice_one(id, controller, events);
            }
            return true;
        }
        if self.prevention_off() {
            return false;
        }
        // Swans of Bryn Argoll — prevent it; the source's controller draws
        // that many (CR 615.5: a sourceless or vanished source draws none).
        if self.battlefield_find(recipient).is_some_and(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|s| matches!(s.effect, crate::effect::StaticEffect::PreventDamageToSelfSourceControllerDraws))
        }) {
            if let Some(p) = source.and_then(|s| self.damage_source_controller(s)) {
                for _ in 0..dealt {
                    if !self.draw_one_or_deck(p, events) {
                        break;
                    }
                }
            }
            return true;
        }
        // Khârn the Betrayer — prevent it, and it changes hands (CR 800.4:
        // never to a player who has left the game).
        let betrayed = self.battlefield_find(recipient).and_then(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|s| matches!(s.effect, crate::effect::StaticEffect::PreventDamageToSelfOpponentGainsControl))
                .then_some(c.controller)
        });
        if let Some(controller) = betrayed {
            // "An opponent of your choice" — the controller's ballot.
            if let Some(to) = self.choose_opponent_at_once(controller, recipient, "Choose the opponent who gains control") {
                self.change_control(recipient, to);
            }
            return true;
        }
        // Bloatfly Swarm — only while a counter remains; each removed counter
        // is a rad counter for every player.
        let rad = self.battlefield_find(recipient).and_then(|c| {
            c.definition.static_abilities.iter().find_map(|s| match s.effect {
                crate::effect::StaticEffect::PreventDamageToSelfWhileCountersForRad { counter }
                    if c.counter_count(counter) > 0 =>
                {
                    Some(counter)
                }
                _ => None,
            })
        });
        if let Some(counter) = rad {
            let Some(c) = self.battlefield_find_mut(recipient) else { return false };
            let removed = dealt.min(c.counter_count(counter));
            c.remove_counters(counter, removed);
            events.push(GameEvent::CounterRemoved { card_id: recipient, counter_type: counter, count: removed });
            for p in 0..self.players.len() {
                if !self.players[p].eliminated {
                    self.players[p].rad_counters = self.players[p].rad_counters.saturating_add(removed);
                }
            }
            return true;
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
        self.mint_tokens_scaled(def, controller, dealt, false, events);
        true
    }

    /// Phytohydra (CR 614) — does `id` replace incoming damage with that many
    /// +1/+1 counters? Reads `StaticEffect::ReplaceDamageToSelfWithCounters`.
    ///
    /// The third field is the seat that puts the counters: the replacement
    /// source's controller (CR 109.5 / 614.1a), not the damage source's — so
    /// an opponent's "whenever you put counters" never sees them and All Will
    /// Be One can't loop on an opponent's Anti-Venom.
    pub(crate) fn creature_replaces_damage_with_counters(
        &self,
        id: CardId,
    ) -> Option<(crate::card::CounterType, crate::effect::DamageToCounters, usize)> {
        self.battlefield_find(id)
            .and_then(|c| {
                c.definition.static_abilities.iter().find_map(|s| match self.active_static_where(&s.effect, c, |e| matches!(e, crate::effect::StaticEffect::ReplaceDamageToSelfWithCounters { .. })) {
                    Some(crate::effect::StaticEffect::ReplaceDamageToSelfWithCounters { kind, how }) => {
                        Some((*kind, *how, c.controller))
                    }
                    _ => None,
                })
            })
            .or_else(|| self.attached_replaces_damage_with_counters(id))
            .or_else(|| self.teammate_replaces_damage_with_counters(id))
    }

    /// Vigor — another permanent of the creature's controller turning damage
    /// to it into counters.
    fn teammate_replaces_damage_with_counters(
        &self,
        id: CardId,
    ) -> Option<(crate::card::CounterType, crate::effect::DamageToCounters, usize)> {
        let c = self.battlefield_find(id)?;
        if !self.computed_is_creature(c) {
            return None;
        }
        let ctrl = c.controller;
        self.battlefield.iter().filter(|s| s.controller == ctrl && s.id != id).find_map(|s| {
            s.definition.static_abilities.iter().find_map(|sa| match sa.effect {
                crate::effect::StaticEffect::ReplaceDamageToOtherCreaturesYouControlWithCounters { kind, how } => {
                    Some((kind, how, s.controller))
                }
                _ => None,
            })
        })
    }

    /// Panther Habit — an attachment turning damage to `id` into counters.
    fn attached_replaces_damage_with_counters(
        &self,
        id: CardId,
    ) -> Option<(crate::card::CounterType, crate::effect::DamageToCounters, usize)> {
        self.battlefield.iter().filter(|a| a.attached_to == Some(id)).find_map(|a| {
            a.definition.static_abilities.iter().find_map(|s| match s.effect {
                crate::effect::StaticEffect::ReplaceDamageToAttachedWithCounters { kind, how } => {
                    Some((kind, how, a.controller))
                }
                _ => None,
            })
        })
    }

    /// Target-aware scaling for a combat-damage event aimed at a player or
    /// planeswalker attack target (CR 614.2 / 614.5). `source` is the
    /// attacking creature (Torbran's source-scoped bonus).
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
        let to = match target {
            AttackTarget::Player(_) => None,
            AttackTarget::Planeswalker(id) | AttackTarget::Battle(id) => Some(id),
        };
        let scaled = self.attached_combat_damage_doubling(source, to, self.scale_damage_to(source, ent, amount));
        // Charging Tuskodon — its own combat damage to a player, doubled.
        let scaled = match (source, target) {
            (Some(src), AttackTarget::Player(_))
                if scaled > 0 && self.battlefield.has_damage_scaler(crate::game::card_can_scale_damage) =>
            {
                let n = self.battlefield_find(src).map_or(0, |c| {
                    c.definition
                        .static_abilities
                        .iter()
                        .filter(|sa| {
                            self.static_active_is(&sa.effect, c, |e| matches!(e, crate::effect::StaticEffect::SelfDealsDoubleCombatDamageToPlayers))
                        })
                        .count()
                });
                scaled.saturating_mul(1u32 << n.min(8))
            }
            _ => scaled,
        };
        // Jeska, Thrice Reborn — triple to one of the registrant's opponents.
        match (source, target) {
            (Some(src), AttackTarget::Player(p)) if !self.tripled_combat_damage_to_opponents.is_empty() => {
                let n = self
                    .tripled_combat_damage_to_opponents
                    .iter()
                    .filter(|(c, reg)| *c == src && !self.same_team(*reg, p))
                    .count() as u32;
                scaled.saturating_mul(3u32.saturating_pow(n))
            }
            _ => scaled,
        }
    }

    /// CR 615 — apply prevention shields to combat damage headed for an
    /// attack target (player or planeswalker). Returns the unprevented
    /// remainder.
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
                // Both report the prevention (CR 615.5): "whenever damage is
                // prevented" and "for each damage prevented this way" read it.
                if self.all_damage_to_player_prevented(p)
                    || !self.prevention_off()
                        && self.combat_damage_prevented_to_players_this_turn.contains(&p)
                {
                    if amount > 0 {
                        events.push(GameEvent::DamagePrevented { amount, to_player: Some(p), to_card: None });
                    }
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
                self.apply_combat_prevention_shields(EntityRef::Player(p), amount, source, events)
            }
            AttackTarget::Planeswalker(pw) => {
                if !self.prevention_off()
                    && !self.combat_damage_prevented_to_walkers_this_turn.is_empty()
                    && self
                        .battlefield_find(pw)
                        .is_some_and(|c| self.combat_damage_prevented_to_walkers_this_turn.contains(&c.controller))
                {
                    return 0;
                }
                let dealt = self.apply_combat_prevention_shields(EntityRef::Permanent(pw), amount, source, events);
                // CR 122.1c — a shield counter prevents it (and goes).
                if dealt > 0 && self.consume_shield_counter(pw) { 0 } else { dealt }
            }
            AttackTarget::Battle(b) => {
                let dealt = self.apply_combat_prevention_shields(EntityRef::Permanent(b), amount, source, events);
                if dealt > 0 && self.consume_shield_counter(b) { 0 } else { dealt }
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

    /// CR 614.9 — the untapped creature `p` controls that absorbs an unblocked
    /// attacker's combat damage to them (Weathered Bodyguards), if any.
    fn unblocked_combat_damage_absorber(&self, p: usize, attacker: CardId) -> Option<CardId> {
        if self.attacker_is_blocked(attacker) {
            return None;
        }
        self.battlefield
            .iter()
            .filter(|c| c.controller == p && !c.tapped && !c.face_down)
            .find(|c| {
                c.definition.static_abilities.iter().any(|s| {
                    matches!(s.effect, crate::effect::StaticEffect::AbsorbUnblockedCombatDamageWhileUntapped)
                })
            })
            .map(|c| c.id)
    }

    /// The combat-damage-becomes-mill replacements' shared half: `p` mills
    /// `amount` (Szadek, Undead Alchemist).
    fn mill_instead_of_combat_damage(&mut self, p: usize, amount: u32, events: &mut Vec<GameEvent>) {
        for _ in 0..amount {
            if self.players[p].library.is_empty() {
                break;
            }
            let card = self.players[p].library.remove(0);
            let cid = card.id;
            if !self.route_to_graveyard(card, events) {
                self.note_milled(p, cid);
                events.push(GameEvent::CardMilled { player: p, card_id: cid });
            }
        }
    }

    /// Does a `CombatDamageToPlayersBecomesMill` static `controller` controls
    /// cover `attacker`?
    fn combat_damage_becomes_mill(&self, attacker: CardId, controller: usize) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller == controller
                && c.definition.static_abilities.iter().any(|sa| {
                    self.static_active_is(&sa.effect, c, |e| matches!(e, crate::effect::StaticEffect::CombatDamageToPlayersBecomesMill { filter } if self.evaluate_requirement_static(filter, &Target::Permanent(attacker), controller, Some(c.id))))
                })
        })
    }

    /// Apply `amount` damage from `atk` to its declared attack target: life
    /// loss (or poison under Infect) for a player, loyalty loss for a
    /// planeswalker. Fires `DealsCombatDamageToPlayer` triggers on a player hit.
    fn deal_combat_damage_to_target(
        &mut self,
        atk: &AttackerInfo,
        amount: u32,
        events: &mut Vec<GameEvent>,
    ) -> u32 {
        match atk.target {
            AttackTarget::Player(p) => {
                // CR 615 — per-creature combat-damage shaving from an untapped
                // permanent the defender controls (Thunderstaff).
                let shave = self.combat_damage_shaved_for(p);
                let amount = amount.saturating_sub(shave);
                if amount == 0 && shave > 0 {
                    return 0;
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
                    let n = self.place_counters(atk.id, crate::card::CounterType::PlusOnePlusOne, amount);
                    if n > 0 {
                        events.push(GameEvent::CounterAdded {
                            card_id: atk.id,
                            counter_type: crate::card::CounterType::PlusOnePlusOne,
                            count: n, placer: self.resolution_causer,
                        });
                    }
                    self.mill_instead_of_combat_damage(p, amount, events);
                    return 0;
                }
                // CR 615 — Sokrates's Sokratic Dialogue: prevented, and the
                // attacker's controller and the player each draw half.
                if amount > 0
                    && !self.prevention_off()
                    && self.turn.combat_damage_becomes_draws.contains(&atk.id)
                {
                    events.push(GameEvent::DamagePrevented { amount, to_player: Some(p), to_card: None });
                    let half = (amount / 2) as usize;
                    self.draw_n(atk.controller, half, events);
                    self.draw_n(p, half, events);
                    return 0;
                }
                // CR 614.1 — combat damage is damage too: The Mindskinner,
                // Crumbling Sanctuary, Delaying Shield and Nefarious Lich
                // replace it exactly as they replace a spell's.
                if amount > 0 && self.replace_damage_to_player(p, amount, Some(atk.controller), Some(atk.id), events) {
                    return 0;
                }
                // CR 614 — Undead Alchemist: a matching attacker's combat
                // damage to a player is a mill of that many instead.
                if amount > 0 && self.combat_damage_becomes_mill(atk.id, atk.controller) {
                    self.mill_instead_of_combat_damage(p, amount, events);
                    return 0;
                }
                // CR 614.9 — Palisade-Giant-style redirect to the redirector.
                // Turn the Tables' combat-only redirect is checked first; both
                // need the destination still on the battlefield.
                if let Some(redirect) = self
                    .combat_damage_redirect_this_turn
                    .iter()
                    .find(|(seat, to)| *seat == p && self.battlefield_find(*to).is_some())
                    .map(|(_, to)| *to)
                    .or_else(|| {
                        self.damage_redirect_target(crate::game::effects::EntityRef::Player(p))
                    })
                    .or_else(|| self.unblocked_combat_damage_absorber(p, atk.id))
                {
                    let excess = self.note_excess_damage(redirect, amount, atk.has_deathtouch, Some(atk.id));
                    self.push_excess_damage_event(redirect, excess, true, events);
                    if let Some(c) = self.battlefield_find_mut(redirect) {
                        c.damage = c.damage.saturating_add(amount);
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
                    return amount;
                }
                // Phyrexian Unlife — at ≤ 0 life all damage lands as poison.
                if atk.has_infect || (self.players[p].life <= 0 && self.player_unlife_active(p)) {
                    self.add_poison(p, amount, events);
                    // Infect damage is still damage dealt to the player (CR
                    // 702.90b): "whenever you're dealt damage" sees it, as it
                    // does on the noncombat path.
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: Some(p),
                        to_card: None,
                        combat: true,
                        from_controller: Some(atk.controller),
                        from_card: Some(atk.id),
                    });
                } else {
                    // Angel's Grace / Worship — damage lands in full, but the
                    // life reduction is clamped to the floor.
                    let life_delta = self.clamp_damage_to_life_floor(p, amount);
                    let applied = self.adjust_life_applied(p, -crate::game::life_amount(life_delta));
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
                // Mark the player damaged this turn (Bloodthirst, CR 702.54)
                // and record the attacker (Spear of Heliod's targeting).
                if amount > 0 {
                    // One `Player::deref_mut` for the run: `Player` is a CoW
                    // handle, so each write below was its own `Arc::make_mut`.
                    let atk_id = atk.id;
                    self.combat_damaged_since_your_turn |= crate::game::seat_bit(p) as u32;
                    let pl = &mut *self.players[p];
                    pl.was_dealt_damage_this_turn = true;
                    pl.damage_taken_this_turn = pl.damage_taken_this_turn.saturating_add(amount);
                    pl.record_damage_from(Some(atk_id), amount);
                    pl.combat_damage_taken_this_turn =
                        pl.combat_damage_taken_this_turn.saturating_add(amount);
                    if !pl.creatures_that_damaged_me_this_turn.contains(&atk_id) {
                        pl.creatures_that_damaged_me_this_turn.push(atk_id);
                    }
                    if !pl.creatures_that_combat_damaged_me_this_turn.contains(&atk_id) {
                        pl.creatures_that_combat_damaged_me_this_turn.push(atk_id);
                    }
                    // CR 702.76 — Prowl: record the damaging creature's types
                    // for its controller (Changeling via the any flag).
                    // ⚠ `remembers` rides this scan; a second `find_by_id`
                    // cost more than the gate saved (PERF `(-154)`).
                    // ⚠ `self.battlefield` is borrowed while `self.players` is
                    // written — disjoint fields, named separately for the
                    // borrow checker; the `.clone()` this replaced was a top
                    // allocation site (PERF `(-355)`).
                    let mut remembers = false;
                    let mut by_name: Option<String> = None;
                    let battlefield = &self.battlefield;
                    if let Some(c) = battlefield.find_by_id(atk.id) {
                        let ctrl = c.controller;
                        remembers = c.definition.remembers_damage_victims();
                        by_name = c.definition.remembers_combat_victims_by_name().then(|| c.definition.name.to_string());
                        let players = &mut self.players;
                        if c.definition.keywords.has_kw(&Keyword::Changeling) {
                            players[ctrl].prowl_any_type_this_turn = true;
                        }
                        for t in &c.definition.subtypes.creature_types {
                            if !players[ctrl].prowl_types_this_turn.contains(t) {
                                players[ctrl].prowl_types_this_turn.push(*t);
                            }
                        }
                    }
                    // The Fallen — "each player this creature has dealt damage
                    // to this game". ⚠ Combat damage to a player takes this
                    // path, not `deal_damage_to_from`'s recorder, so it must
                    // record here. Gated on the definition asking (PERF
                    // `(-154)`): the mutable find unshares the battlefield.
                    if remembers
                        && let Some(c) = self.battlefield_find_mut(atk_id)
                        && !c.damaged_players_this_game.contains(&p)
                    {
                        c.damaged_players_this_game.push(p);
                    }
                    // Gollum — "dealt combat damage this game by a creature
                    // named ...", keyed by name so another Gollum's hits count.
                    if let Some(name) = by_name
                        && !self.combat_victims_by_name.iter().any(|(n, q)| *n == name && *q == p)
                    {
                        self.combat_victims_by_name.push((name, p));
                    }
                }
                // CR 702.164c — Toxic N adds N poison on combat damage to a
                // player, on top of any life loss (and stacks with Infect's
                // poison). Only when damage was actually dealt.
                if atk.toxic > 0 && amount > 0 {
                    self.add_poison(p, atk.toxic, events);
                }
                // Commander damage tally (CR 704.6c), credited for infect and
                // regular damage alike; the SBA eliminates a player when any
                // (victim, commander) entry reaches 21.
                if let Some(cmdr) = self.commander_card_of(atk.id) {
                    self.record_commander_damage(p, cmdr, amount);
                }
                // CR 726 — a creature dealing combat damage to the monarch
                // makes its controller the new monarch.
                if amount > 0 && self.monarch == Some(p) {
                    let ctrl = self.battlefield.find_by_id(atk.id).map(|c| c.controller);
                    if let Some(ctrl) = ctrl
                        && ctrl != p {
                            self.set_monarch(ctrl, events);
                        }
                }
                // CR 727.2 — the same handover for the initiative.
                if amount > 0 && self.initiative == Some(p) {
                    let ctrl = self.battlefield.find_by_id(atk.id).map(|c| c.controller);
                    if let Some(ctrl) = ctrl
                        && ctrl != p {
                            self.take_initiative(ctrl, events);
                        }
                }
                self.fire_combat_damage_to_player_triggers(atk.id, p, amount);
                amount
            }
            AttackTarget::Planeswalker(pw_id) => {
                // CR 702.19c — "trample over planeswalkers" (Thrasta): lethal
                // (remaining loyalty) to the walker, excess to its controller.
                // Plain trample never spills past a walker (CR 702.19f).
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
                let excess = self.note_planeswalker_excess(pw_id, amount);
                self.push_excess_damage_event(pw_id, excess, true, events);
                let current =
                    self.battlefield_find(pw_id).map_or(0, |c| c.counter_count(crate::card::CounterType::Loyalty));
                let floored = self.loyalty_after_damage(pw_id, current, amount);
                if let Some(pw) = self.battlefield_find_mut(pw_id) {
                    let new_loyalty = floored;
                    pw.counters
                        .insert(crate::card::CounterType::Loyalty, new_loyalty);
                    // The same per-victim record the creature paths keep, so
                    // "destroy the creature that dealt damage to her"
                    // (`LastDamagerOf`, Vraska the Unseen's +1) has a damager.
                    if amount > 0 {
                        pw.dealt_damage_this_turn = true;
                        pw.damage_dealt_to_this_turn += amount;
                        pw.damaged_by_this_turn.push(atk.id);
                        pw.record_damage_from(atk.id, amount);
                    }
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
                    // Combat damage is damage: the recipient-agnostic
                    // wordings fire too (CR 510.2).
                    self.fire_combat_damage_triggers(
                        atk.id,
                        &[
                            EventKind::DealsCombatDamageToPlaneswalker,
                            EventKind::DealsCombatDamage,
                            EventKind::DealsDamage,
                        ],
                        Target::Permanent(pw_id),
                        amount,
                        &granted,
                        true,
                    );
                }
                let mut spill_dealt = 0;
                if let Some(p) = spill_to
                    && spill > 0
                {
                    let mut spilled = atk.clone();
                    spilled.target = AttackTarget::Player(p);
                    // CR 615 — the player's own prevention (Fog-style shields,
                    // protection) applies to the spilled part as it does to an
                    // unblocked hit; prevented damage is never commander damage.
                    let spill = self.prevent_combat_to_target(spilled.target, spill, Some(atk.id), events);
                    if spill > 0 {
                        spill_dealt = self.deal_combat_damage_to_target(&spilled, spill, events);
                    }
                }
                amount + spill_dealt
            }
            AttackTarget::Battle(b_id) => {
                // CR 310.6 — combat damage to a battle removes that many
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
                // "…to a player or battle" (`EventSpec::or_battle`) and the
                // recipient-agnostic wordings, the battle in slot 0.
                if amount > 0 {
                    let granted = self.static_granted_triggers_of(atk.id);
                    self.fire_combat_damage_triggers(
                        atk.id,
                        &[
                            EventKind::DealsCombatDamageToPlayer,
                            EventKind::DealsCombatDamage,
                            EventKind::DealsDamage,
                        ],
                        Target::Permanent(b_id),
                        amount,
                        &granted,
                        true,
                    );
                }
                amount
            }
        }
    }

    /// CR 508.1g / 509.1b — pick the untapped permanent `who` taps for
    /// `payer`'s `AttackBlockCostTapAnother` cost, excluding the payer,
    /// everything declared this step, and helpers already spent.
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
    /// Per blocker and additive (a source may narrow itself or charge life),
    /// so the bot's planner can trim a batch to what it can pay. The engine
    /// rejects the whole declaration when a player can't pay, so a planner
    /// that ignores it loses the block step.
    pub(crate) fn block_tax_for(&self, blocker: CardId) -> (u32, u32) {
        let (mut mana, mut life) = (self.block_tax_this_turn, 0u32);
        // The walk matches one static, and the lane holds whether any
        // permanent prints it (PERF `(-241)`): asked per declared blocker.
        if !self.battlefield.has_block_tax_static() {
            return (mana, life);
        }
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

    /// CR 702.25b — flanking instances `attacker` gets beyond its own: one
    /// per `OtherFlankersHaveFlanking` its controller controls on another
    /// permanent (Cavalry Master).
    fn extra_flanking_instances(&self, attacker: CardId) -> i32 {
        let Some(ctrl) = self.battlefield_find(attacker).map(|c| c.controller) else { return 0 };
        self.battlefield
            .iter()
            .filter(|c| c.controller == ctrl && c.id != attacker)
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter(|sa| matches!(sa.effect, crate::effect::StaticEffect::OtherFlankersHaveFlanking))
            .count() as i32
    }

    /// CR 509.1d — the mana one blocker assigned to `attacker` costs: each
    /// Aura on it with `BlockTaxForEnchantedAttacker` (Awesome Presence).
    pub(crate) fn enchanted_attacker_block_tax(&self, attacker: CardId) -> u32 {
        if !self.battlefield.has_block_tax_static() {
            return 0;
        }
        self.battlefield
            .iter()
            .filter(|a| a.attached_to == Some(attacker))
            .flat_map(|a| a.definition.static_abilities.iter())
            .map(|sa| match sa.effect {
                crate::effect::StaticEffect::BlockTaxForEnchantedAttacker { amount } => amount,
                _ => 0,
            })
            .sum()
    }

    /// Does any permanent on this board charge a block tax at all? One walk,
    /// so [`block_tax_for`](Self::block_tax_for)'s per-blocker walk is only
    /// paid on the boards that have one.
    pub(crate) fn block_tax_present(&self) -> bool {
        // The lane's predicate is the walk this used to make (PERF `(-241)`).
        self.block_tax_this_turn > 0 || self.battlefield.has_block_tax_static()
    }

    /// CR 509.1a/b — everything that bars `blocker` from blocking at all, as
    /// `(site, error)`, or `None`.
    ///
    /// One walker for [`declare_blockers`](Self::declare_blockers)'s gate,
    /// `blocker_can_block_anything` (bot planner, client legal-block list),
    /// the CR 509.1b "must block if able" loops and CR 702.39 Provoke. Their
    /// separate copies had drifted both ways — the gate skipped seven "can't
    /// block unless …" families, and the loops missed detain and Void
    /// Winnower, obliging blocks the gate rejected (`cr_recent100`).
    ///
    /// Keyword families are asked in one walk over the computed set, as in
    /// `attacker_self_block`.
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
        if self.turn.cant_block_this_turn.contains(&blocker.id) {
            return no(line!());
        }
        let owner = blocker.controller;
        // Void Winnower — an even mana value (zero included) can't block while
        // an opponent has the static. Cheap terms first; the board question is
        // a presence gate (`block_even_mv_lock_in_scope`), so only a board
        // playing the card reaches the seat walk.
        if blocker.printed_cmc().is_multiple_of(2)
            && self.block_even_mv_lock_in_scope()
            && self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::OpponentsCantBlockWithEvenMv)
                }) && !self.same_team(c.controller, owner)
            })
        {
            return no(line!());
        }
        // Bothersome Quasit — an opponent's goaded creature can't block. Same
        // presence gate; the goad walk runs only on a board that plays it.
        if self.block_even_mv_lock_in_scope()
            && self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::OpponentsGoadedCreaturesCantBlock)
                }) && !self.same_team(c.controller, owner)
            })
            && self.is_goaded(blocker)
        {
            return no(line!());
        }
        // One walk. Everything below reads only the blocker's own computed
        // keywords and the board, never the attacker.
        let mut control_count: Option<(crate::card::SelectionRequirement, u32, bool)> = None;
        let mut tap_another: Option<crate::card::SelectionRequirement> = None;
        // CR 508.1g's block half: note whether one of the four families
        // `attack_block_keyword_tax(.., for_attack = false)` prices is
        // present, so the tax isn't re-walked per candidate on boards where
        // it's 0.
        let mut taxed = false;
        for k in cp.keywords().iter() {
            let barred = match k {
                Keyword::CantBlock | Keyword::Decayed => true,
                Keyword::CantAttackOrBlockUnlessPay(_)
                | Keyword::CantAttackOrBlockUnlessPayPerCounter(_)
                | Keyword::CantAttackOrBlockUnlessPayPerCardInEnchanterHand
                | Keyword::CantAttackOrBlockUnlessPayPerPermanent(_) => {
                    taxed = true;
                    false
                }
                Keyword::CantAttackOrBlockUnlessEvenCounters => {
                    !blocker.counter_sum().is_multiple_of(2)
                }
                Keyword::CantAttackOrBlockUnlessHandSizeAtMost(n) => {
                    self.players[owner].hand.len() as u32 > *n
                }
                Keyword::CantAttackOrBlockUnlessAHandIsEmpty => !self.some_hand_is_empty(),
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
                // Vantress Gargoyle — a thin hand locks the block.
                Keyword::CantBlockUnlessHandSizeAtLeast(n) => (self.players[owner].hand.len() as u32) < *n,
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
        // CR 508.1g's block half — "unless its controller pays {N}" is legal
        // only if the seat can produce the tax. The declaration charges the
        // sum later; one unpayable share makes the sum unpayable, so asking
        // here is sound and names the right blocker. `taxed` makes the gate
        // exact.
        if taxed {
            let tax = self.attack_block_keyword_tax(blocker.id, cp.keywords(), false);
            if tax > 0 && !self.could_pay_generic(owner, tax) {
                return no(line!());
            }
        }
        debug_assert!(
            taxed || self.attack_block_keyword_tax(blocker.id, cp.keywords(), false) == 0,
            "the block-tax gate skipped a blocker whose keyword tax is non-zero",
        );
        None
    }

    /// Is `b` able to block `required`, for a CR 509.1 requirement that says
    /// it must?
    ///
    /// [`declare_blockers`](Self::declare_blockers) rejects the whole
    /// declaration when an obliged, able creature isn't assigned, so the bot's
    /// planner must answer identically; both call this. Shared by CR 702.39
    /// Provoke (`must_block`), CR 509.1c `MustBeBlocked`, `AllMustBlock` (true
    /// Lure) and `MustBlock`, whose separate copies had drifted on the tapped
    /// term; this one respects `tapped_creatures_can_block`.
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

    /// CR 509.1c — a full blocker cap (Silent Arbiter) whose every blocker a
    /// requirement binds: no declaration obeys more, so an unmet requirement
    /// is excused (a Lure attacker and two able blockers under a cap of one).
    pub(crate) fn block_cap_shuts_out(&self, assignments: &[(CardId, CardId)]) -> bool {
        let Some(cap) = self.combat_participation_cap(true) else { return false };
        let mut distinct: crate::game::types::SmallIdSet<CardId> = self.block_map.keys().copied().collect();
        distinct.extend(assignments.iter().map(|(b, _)| *b));
        distinct.len() >= cap as usize
            && distinct.iter().all(|&bid| {
                self.battlefield_find(bid).is_none_or(|b| {
                    self.computed_permanent(bid).is_some_and(|cp| {
                        cp.keywords().has_kw(&Keyword::MustBlock) || cp.keywords().has_kw(&Keyword::MustAttackOrBlock)
                    }) || self.block_spoken_for_elsewhere(b, CardId(u32::MAX), assignments)
                })
            })
    }

    /// CR 509.1c — is `b` already spoken for by a different block
    /// requirement, and so not idle for this one?
    ///
    /// The rule is "satisfy the maximum number of requirements without
    /// violating a restriction". Checked independently, two requirements
    /// naming one creature can never both be met, making every declaration
    /// illegal (a Lure attacker plus a provoker plus one able defender —
    /// `cube` seed 15). A declaration where every obliged creature blocks
    /// something that obliges it is already maximal; that is what this tests
    /// (assigned to an attacker other than `except` whose requirement binds
    /// this blocker).
    ///
    /// Full CR 509.1c maximization is still approximated — see the loops' note.
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

    /// CR 509.1c — the creature an Equipment on `attacker` says must block
    /// it if able (`EquippedMustBeBlockedByIfAble`: Ace's Baseball Bat's
    /// Dalek), if any. Several such Equipment still want only one blocker.
    pub(crate) fn must_be_blocked_by(&self, attacker: CardId) -> Option<crate::card::SelectionRequirement> {
        self.battlefield.iter().filter(|c| c.attached_to == Some(attacker)).find_map(|c| {
            c.definition.static_abilities.iter().find_map(|sa| match self.active_static_where(&sa.effect, c, |e| matches!(e, crate::effect::StaticEffect::EquippedMustBeBlockedByIfAble { .. })) {
                Some(crate::effect::StaticEffect::EquippedMustBeBlockedByIfAble { filter }) => Some(filter.clone()),
                _ => None,
            })
        })
    }

    /// CR 509.1c — the largest N of `attacker`'s computed
    /// `MustBeBlockedByAtLeast(N)` (Gorm the Great), if any.
    pub(crate) fn must_be_blocked_by_at_least(&self, attacker: CardId) -> Option<usize> {
        let cp = self.computed_permanent(attacker)?;
        cp.keywords()
            .iter()
            .filter_map(|k| match k {
                Keyword::MustBeBlockedByAtLeast(n) => Some(*n as usize),
                _ => None,
            })
            .max()
    }

    /// CR 509.1c — does a "must block" requirement on `attacker` bind at all?
    /// A restriction outranks a requirement (CR 509.1b): an attacker with Lure
    /// and Menace facing one able blocker has no legal block, so the
    /// requirement must not bind (`cube` seed 15). Counts the able set, not
    /// the assigned one: the question is whether a legal block exists.
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
        // CR 506.2 / 509.1c — a block cap (Silent Arbiter) with fewer free
        // slots than the count needs leaves the requirement no legal form
        // (must-be-blocked menace Jon Irenicus, 7-seat pod, seed 806097).
        if let Some(cap) = self.combat_participation_cap(true)
            && (cap as usize).saturating_sub(self.block_map.len()) + self.blocker_count_of(attacker) < min_b
        {
            return false;
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

    /// CR 508.1g — the life `attacks` cost under a defender's "pays N life for
    /// each" window (Sivitri): per attacker at that player or a planeswalker
    /// they control.
    pub(crate) fn attack_life_tax_for(&self, attacks: &[Attack]) -> u32 {
        if self.players.iter().all(|pl| pl.attack_life_tax_until_your_turn == 0) {
            return 0;
        }
        attacks
            .iter()
            .filter_map(|a| match a.target {
                crate::game::types::AttackTarget::Player(d) => Some(d),
                crate::game::types::AttackTarget::Planeswalker(pw) => self.battlefield_find(pw).map(|c| c.controller),
                crate::game::types::AttackTarget::Battle(_) => None,
            })
            .map(|d| self.players[d].attack_life_tax_until_your_turn)
            .sum()
    }

    /// CR 508.1g — the generic mana a declaration of `attacks` costs:
    /// defenders' `AttackTaxToController` statics (Ghostly Prison, Propaganda,
    /// Sphere of Safety, Elephant Grass), War Tax, Forbidding Spirit, and each
    /// attacker's own "can't attack unless you pay {N}" via `keyword_tax`.
    ///
    /// Additive per attacker with no cross terms, so both callers can use it:
    /// [`declare_attackers_banded`] pays it or rejects the whole declaration,
    /// and the bot's picker trims its batch to what it can pay (monotonic, so
    /// trimming terminates). PERF (-55).
    ///
    /// `keyword_tax` is injected because the callers hold computed keywords
    /// differently (engine: gated `Vec<ComputedPermanent>`; picker: an `Arc`
    /// per lookup), and deriving it here would cost the engine its memo.
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
                            | crate::effect::StaticEffect::AttackTaxOnYourPlaneswalkers { .. }
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
            // Evaluate each `amount` with the defender as "you" and the tax
            // permanent as source, so dynamic taxes (Sphere of Safety) count
            // the defender's board.
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
                        // The attacker, for a per-attacker amount (Nils's
                        // "X is the number of counters on that creature").
                        ctx.trigger_source = Some(crate::game::effects::EntityRef::Permanent(atk.attacker));
                        total_tax += self.evaluate_value(amount, &ctx).max(0) as u32;
                    }
                    if let crate::effect::StaticEffect::AttackTaxOnYourPlaneswalkers { amount } = &sa.effect
                        && at_planeswalker
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
                Keyword::CantAttackUnlessPayPerCounter(kind) if for_attack => {
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

    /// Push `source`'s `DealsCombatDamageToPlayer` triggers, with
    /// `damaged_player` as target so the effect can say "that player"
    /// (`PlayerRef::Target(0)`).
    ///
    /// Phase 1: the attacker's own `SelfSource` / `AnyPlayer` triggers.
    /// Phase 2: `FromYourGraveyard` triggers in the graveyard of the
    /// source's controller ("return this from your graveyard" — Killian's
    /// Confidence), with the graveyard card bound as trigger source so
    /// `Move(SelfSource → Hand)` moves the right card.
    pub fn fire_combat_damage_to_player_triggers(
        &mut self,
        source: CardId,
        damaged_player: usize,
        damage_amount: u32,
    ) {
        // The dealing creature's controller has now dealt combat damage to a
        // player this turn; an Assassin or a commander also opens their
        // freerunning costs (CR 702.173a — its types as it dealt the damage).
        if let Some(c) = self.battlefield_find(source) {
            let controller = c.controller;
            self.players[controller].dealt_combat_damage_to_player_this_turn = true;
            if !self.players[controller].freerunning_unlocked_this_turn
                && (self.permanent_has_creature_type(source, crate::card::CreatureType::Assassin)
                    || self.is_commander(source))
            {
                self.players[controller].freerunning_unlocked_this_turn = true;
            }
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
            // CR 603.2c — the batched, filtered kind (Aphelia): one fire per
            // damaged seat per step, keyed in the step's batch set under the
            // watcher's source and an index no printed trigger uses.
            let batched: Vec<crate::game::types::DelayedTrigger> = self
                .delayed_triggers
                .iter()
                .filter(|dt| {
                    dt.controller == controller
                        && matches!(
                            &dt.kind,
                            crate::game::types::DelayedKind::MatchingCreaturesDealCombatDamageBatchThisTurn(f)
                                if self.battlefield_find(source).is_some_and(|c| {
                                    self.evaluate_requirement_on_card(f, c, controller)
                                })
                        )
                })
                .cloned()
                .collect();
            for dt in batched {
                let key = (dt.source, usize::MAX, BatchSubject::Player(damaged_player));
                if self.combat_trigger_fired_this_step.contains(&key) {
                    continue;
                }
                self.combat_trigger_fired_this_step.push(key);
                self.push_stack(
                    TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                        .trigger_source(Some(crate::game::effects::EntityRef::Permanent(source)))
                        .trigger_player(Some(damaged_player))
                        .event_amount(damage_amount)
                        .build(),
                );
            }
            for dt in watchers {
                // The damaged seat is `PlayerRef::TriggerEventPlayer`
                // (Aphelia's "that player loses half their life").
                self.push_stack(
                    TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                        .trigger_source(Some(crate::game::effects::EntityRef::Permanent(source)))
                        .trigger_player(Some(damaged_player))
                        .event_amount(damage_amount)
                        .build(),
                );
            }
        }
        self.fire_named_creature_combat_damage_watchers(source, damaged_player, damage_amount);
        self.fire_source_dealt_damage_watchers(source, damage_amount);
        self.fire_source_combat_damage_to_player_watchers(source, damage_amount);
        self.fire_combat_damage_watchers(source, damage_amount, true);
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
            true,
        );
        // CR 510 — "whenever combat damage is dealt to you" listeners on the
        // damaged player's own permanents (Risona). Plain loops over the
        // trigger member list (PERF (-78), `(-224)`), unless an instance grant
        // of the kind is live — none in the catalog today, but read so the
        // walk can't drop one silently.
        let own_grant = self.any_granted_trigger_of_kind(&EventKind::ControllerDealtCombatDamage);
        // `(listener, effect, controller, filter, batch index)`: the filter
        // and "one or more" batch slot are honoured as the dispatcher would
        // (Norn's Decree: one poison for a whole swing, opponents' creatures
        // only).
        type Listener = (CardId, Effect, usize, Option<crate::effect::Predicate>, Option<usize>);
        let mut listeners: Vec<Listener> = Vec::new();
        let listens = |ta: &crate::card::TriggeredAbility| {
            ta.event.kind == EventKind::ControllerDealtCombatDamage
                && ta.event.scope == crate::effect::EventScope::SelfSource
        };
        // CR 113.10b — a stripped listener hears nothing.
        let stripped = self.stripped_permanents();
        let visit = |c: &crate::card::CardInstance| {
            if c.controller != damaged_player || stripped.contains(&c.id) {
                return;
            }
            for (i, ta) in c.definition.triggered_abilities.iter().enumerate() {
                if listens(ta) {
                    listeners.push((
                        c.id,
                        ta.effect.clone(),
                        c.controller,
                        ta.event.filter.clone(),
                        ta.event.once_per_batch.then_some(i),
                    ));
                }
            }
            if own_grant {
                self.for_each_granted_trigger_matching(c.id, listens, |ta| {
                    listeners.push((c.id, ta.effect.clone(), c.controller, ta.event.filter.clone(), None))
                });
            }
        };
        self.battlefield.for_each_triggerer_or_all(own_grant, visit);
        // "Whenever combat damage is dealt to you …, if this card is in your
        // graveyard" (Vengeful Pharaoh): the damaged player's graveyard
        // listens too, behind its own trigger lane (PERF `(-210)`'s word).
        if self.players[damaged_player].graveyard.has_graveyard_trigger() {
            for c in &self.players[damaged_player].graveyard {
                for (i, ta) in c.definition.triggered_abilities.iter().enumerate() {
                    if ta.event.kind == EventKind::ControllerDealtCombatDamage && ta.event.scope.from_graveyard() {
                        listeners.push((
                            c.id,
                            ta.effect.clone(),
                            damaged_player,
                            ta.event.filter.clone(),
                            ta.event.once_per_batch.then_some(i),
                        ));
                    }
                }
            }
        }
        for (listener, effect, controller, filter, batch) in listeners {
            let dealer = Some(crate::game::effects::EntityRef::Permanent(source));
            if let Some(f) = &filter {
                let ctx = crate::game::effects::EffectContext::for_intervening_filter(
                    controller,
                    listener,
                    dealer,
                    damage_amount,
                );
                if !self.evaluate_predicate(f, &ctx) {
                    continue;
                }
            }
            if let Some(i) = batch {
                let key = (listener, i, BatchSubject::Player(damaged_player));
                if self.combat_trigger_fired_this_step.contains(&key) {
                    continue;
                }
                self.combat_trigger_fired_this_step.push(key);
            }
            // CR 603.3d — a prompting damaged player names the target.
            if self.trigger_asks_targets(controller, &effect) {
                let mode = self.pick_trigger_mode(&effect, listener, controller);
                self.queue_trigger_asking(crate::game::types::PendingTriggerPush {
                    source: listener,
                    controller,
                    effect,
                    subject: dealer,
                    event_amount: damage_amount,
                    mode,
                    intervening_if: None,
                    actor: None,
                    from_mana_ability: false,
                    x_value: 0,
                    converged_value: 0,
                    mana_spent: 0,
                });
                continue;
            }
            let (mode, auto_target) =
                self.trigger_mode_and_target(&effect, controller, Some(listener));
            // Bind the creature that dealt the damage as `Selector::TriggerSource`
            // so "whenever a creature deals combat damage to you, destroy it"
            // clauses can reference the dealer (Teysa, Envoy of Ghosts).
            self.push_stack(
                TriggerPush::new(listener, controller, effect)
                    .target(auto_target)
                    .mode(mode)
                    .trigger_source(Some(crate::game::effects::EntityRef::Permanent(source)))
                    .event_amount(damage_amount)
                    .build(),
            );
        }
    }

    /// CR 603.4 — fire `SourceDealsDamageThisTurn` delayed triggers watching
    /// `source` (Paladin of Prahv's Forecast). Called from every damage path
    /// so any damage counts; the amount rides `Value::TriggerEventAmount`.
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
            self.push_stack(
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
            self.push_stack(
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
            self.push_stack(
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
            self.push_stack(
                TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                    .target(Some(Target::Player(owner)))
                    .build(),
            );
        }
    }

    /// Push `source`'s `DealsCombatDamageToCreature` triggers (CR 510.2),
    /// binding the damaged creature as target (removal payoffs, Umezawa's
    /// Jitte charges). Fires once per (source, damaged creature), so Jitte
    /// over-counts on a multi-block (minor, rare).
    /// `granted` is `source`'s `GrantTriggeredAbility` set, built once by the
    /// caller for the whole CR 510.2 batch.
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
            true,
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
            false,
        );
    }

    /// CR 120.3 — the non-combat half for a noncreature permanent (a
    /// planeswalker or battle): only the recipient-agnostic `DealsDamage`
    /// hears it, with the damaged permanent bound to slot 0.
    pub(crate) fn fire_noncombat_damage_to_noncreature_triggers(
        &mut self,
        source: CardId,
        damaged: CardId,
        damage_amount: u32,
    ) {
        if self.battlefield_find(source).is_none() {
            return;
        }
        let granted = self.static_granted_triggers_of(source);
        self.fire_combat_damage_triggers(
            source,
            &[EventKind::DealsDamage],
            Target::Permanent(damaged),
            damage_amount,
            &granted,
            false,
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
            false,
        );
    }

    /// The `GrantTriggeredAbility` set a battlefield permanent currently
    /// carries — empty when it has left. Rebuilds the whole-board grant list,
    /// so a caller firing a *batch* hoists `trigger_grant_sources` itself and
    /// uses `statics_granted_triggers_with` (see the CR 510.2 loop).
    fn static_granted_triggers_of(&self, source: CardId) -> Vec<crate::card::TriggeredAbility> {
        self.battlefield
            .find_by_id(source)
            .map(|c| self.statics_granted_triggers_for(c))
            .unwrap_or_default()
    }

    /// Shared body for combat-damage trigger dispatch (to a player or a
    /// creature): the attacker's printed `SelfSource`/`AnyPlayer` triggers,
    /// equipment- and soulbond-granted ones (CR 702.6e / 702.95), `YourControl`
    /// listeners and `FromYourGraveyard` triggers, each pushed with
    /// `default_target` in slot 0.
    ///
    /// `static_granted` (the source's board-level, kind-independent grant
    /// set) and `kinds` (every kind for one damage event) are passed in so
    /// the board walks run once per event, not once per kind. Pushes stay
    /// grouped per kind, in `kinds` order.
    fn fire_combat_damage_triggers(
        &mut self,
        source: CardId,
        kinds: &[EventKind],
        default_target: Target,
        damage_amount: u32,
        static_granted: &[crate::card::TriggeredAbility],
        combat_batch: bool,
    ) {
        // CR 603.2c / 510.4 — the batch a `once_per_batch` trigger collapses
        // over: one per damaged subject in a combat sub-step. A noncombat
        // delivery is its own batch (`None`) and doesn't touch the
        // step-scoped set.
        let batch_subject = combat_batch.then(|| BatchSubject::of(&default_target));
        // CR 603.2c / 120.3 — a noncombat effect damaging several recipients
        // is one event per source: "whenever ~ deals damage" fires once for the
        // total. Only inside an effect that opened the batch.
        let noncombat_batch = !combat_batch
            && kinds.contains(&EventKind::DealsDamage)
            && self.noncombat_damage_batch_open();
        // One [`DamageTrigger`] bucket per requested kind; drained in order at
        // the bottom.
        // CR 310.6 — a battle delivery passes `DealsCombatDamageToPlayer`
        // with the battle as the target: only an `or_battle` trigger hears it.
        let to_battle = matches!(default_target, Target::Permanent(_))
            && kinds.first() == Some(&EventKind::DealsCombatDamageToPlayer);
        let slot = |e: &crate::effect::EventSpec| {
            kinds
                .iter()
                .position(|want| *want == e.kind)
                .filter(|_| !to_battle || e.or_battle || e.kind != EventKind::DealsCombatDamageToPlayer)
        };
        // Inline (`(-71)`'s device): `kinds` is one to three entries; inner
        // buckets stay `Vec`, which doesn't allocate until pushed. A loop,
        // not a `collect`: `SmallVec`'s `Extend` is external iteration
        // (`(-211)`).
        let mut by_kind: SmallVec<[Vec<DamageTrigger>; 4]> = SmallVec::new();
        for _ in kinds {
            by_kind.push(Vec::new());
        }

        // The dealer via the hint-cached lookup, not folded into the walk
        // below: the fold would pay a compare and store per permanent for a
        // load-and-compare answer (PERF `(-307)`; `(-304)`'s rule).
        let dealer = self.battlefield.find_by_id(source);
        let mut attacker_controller = None;
        // The two attachment facts the three phases below would each walk for.
        // Still one walk, and it still does not short-circuit.
        let mut any_attached = false;
        let mut soulbond_pair = false;
        #[cfg(feature = "trig-census")]
        let census = combat_census::on();
        // The walk below usually finds nothing (PERF `(-318)`). Its two facts
        // are instance fields (`attached_to`, `soulbond_partner`), so no
        // definition lane can hold them; `battlefield.writes()` keys a
        // source-independent "is anything attached or paired" fold that
        // serves every dealer. With it clear the walk can't fire.
        //
        // ⚠ The presence walk is its own short-circuiting `any`, not two more
        // `|=` in the loop below: the fused form taxed every miss and read
        // +0.018 % on `fixed`.
        let present = self.attachment_in_scope();
        if present {
            for c in self.battlefield.iter() {
                any_attached |= c.attached_to == Some(source);
                // Instance fields first, the definition deref last (PERF
                // `(-211)`).
                soulbond_pair |= c.soulbond_partner.is_some()
                    && (c.id == source || c.soulbond_partner == Some(source))
                    && c.definition.soulbond_bonus.is_some();
            }
        }
        #[cfg(feature = "trig-census")]
        if census {
            let (mut a, mut pr) = (false, false);
            for c in self.battlefield.iter() {
                a |= c.attached_to.is_some();
                pr |= c.soulbond_partner.is_some();
            }
            combat_census::add(0, 1);
            combat_census::add(1, self.battlefield.len() as u64);
            combat_census::add(2, u64::from(!a));
            combat_census::add(3, u64::from(!a && !pr));
            combat_census::add(4, u64::from(any_attached));
            combat_census::add(5, u64::from(soulbond_pair));
            let w = u64::from(self.battlefield.writes());
            let prev = combat_census::LAST_WRITES
                .swap(w, std::sync::atomic::Ordering::Relaxed);
            combat_census::add(6, u64::from(prev == w));
            combat_census::add(7, u64::from(!present));
        }
        if let Some(c) = dealer {
            attacker_controller = Some(c.controller);
            // Printed, statics-granted (Tempered/Virulent Sliver) and
            // instance-granted (`granted_triggers_timed` — Primal Odin's
            // Zantetsuken) fire alike.
            let instance_granted = self.granted_triggers(c.id);
            // CR 113.10b — a dealer that has lost all its abilities (Darksteel
            // Mutation, Turn to Frog) has no printed trigger to fire, as the
            // main dispatcher reads it; grants from other sources still apply.
            let stripped = self.ability_strip_possible()
                && self.computed_permanent(source).is_some_and(|cp| cp.lost_all_abilities);
            let printed_list: &[crate::card::TriggeredAbility] =
                if stripped { &[] } else { &c.definition.triggered_abilities };
            let printed = printed_list.iter().enumerate().map(|(i, t)| (Some(i), t));
            for (idx, t) in
                printed.chain(static_granted.iter().chain(instance_granted).map(|t| (None, t)))
            {
                if !matches!(
                    t.event.scope,
                    crate::effect::EventScope::SelfSource | crate::effect::EventScope::AnyPlayer
                ) {
                    continue;
                }
                // The dealer's own `AnyPlayer` trigger ("whenever a Goblin
                // deals combat damage") still checks `dealer_filter` against
                // itself, as Phase 1.6 does for a bystander (Cabal Slaver).
                if let Some(i) = slot(&t.event)
                    && t.event.dealer_filter.as_ref().is_none_or(|f| {
                        self.evaluate_requirement_static(
                            f,
                            &Target::Permanent(source),
                            c.controller,
                            Some(c.id),
                        )
                    })
                {
                    by_kind[i].push((
                        c.id,
                        t.effect.clone(),
                        c.controller,
                        t.event.filter.clone(),
                        false,
                        once_key(idx, t),
                    ));
                }
            }
        }

        // Phase 1b: equipment-granted combat-damage triggers (CR 702.6e) — the
        // Sword cycle's riders — bound to the attacker's controller.
        if let Some(atk_ctrl) = attacker_controller.filter(|_| any_attached) {
            for eq in &self.battlefield {
                if eq.attached_to != Some(source) {
                    continue;
                }
                let Some(bonus) = &eq.definition.equipped_bonus else { continue };
                // CR 702.6e — the granted ability fires off the creature, unless
                // the Equipment opts to fire off itself (Umezawa's Jitte puts the
                // counters on the Equipment, so `Selector::This` must read it).
                let (trig_source, trig_ctrl) = bonus.trigger_binding((eq.id, eq.controller), (source, atk_ctrl));
                for t in &bonus.triggered_abilities {
                    if matches!(
                        t.event.scope,
                        crate::effect::EventScope::SelfSource
                            | crate::effect::EventScope::AnyPlayer
                    ) && let Some(i) = slot(&t.event)
                    {
                        by_kind[i].push((
                            trig_source,
                            t.effect.clone(),
                            trig_ctrl,
                            t.event.filter.clone(),
                            false,
                            None,
                        ));
                    }
                }
            }
            // Attachments with an `EnchantedBySource` combat-damage trigger
            // (Pollenbright Wings; Vorpal Sword's temporary one) fire off the
            // attachment, so `Selector::AttachedTo(This)` reaches the host.
            for aura in &self.battlefield {
                if aura.attached_to != Some(source) {
                    continue;
                }
                let printed = aura.definition.triggered_abilities.iter().enumerate().map(|(i, t)| (Some(i), t));
                for (idx, t) in printed.chain(self.granted_triggers(aura.id).map(|t| (None, t))) {
                    if t.event.scope == crate::effect::EventScope::EnchantedBySource
                        && let Some(i) = slot(&t.event)
                    {
                        by_kind[i].push((
                            aura.id,
                            t.effect.clone(),
                            aura.controller,
                            t.event.filter.clone(),
                            false,
                            once_key(idx, t),
                        ));
                    }
                }
            }
        }

        // CR 702.95 — soulbond-granted combat-damage triggers fire off the
        // attacker for both pair members (Tandem Lookout). Gated on the
        // dealer walk's `soulbond_pair`.
        if let Some(atk_ctrl) = attacker_controller.filter(|_| soulbond_pair) {
            for src in &self.battlefield {
                let Some(bonus) = &src.definition.soulbond_bonus else { continue };
                let Some(partner) = src.soulbond_partner else { continue };
                if src.id != source && partner != source {
                    continue;
                }
                if self.battlefield.find_by_id(partner).is_none() {
                    continue;
                }
                for t in &bonus.triggered_abilities {
                    if let Some(i) = slot(&t.event) {
                        by_kind[i].push((
                            source,
                            t.effect.clone(),
                            atk_ctrl,
                            t.event.filter.clone(),
                            false,
                            None,
                        ));
                    }
                }
            }
        }

        // Phases 1.5 and 1.6, fused into one battlefield walk:
        //
        // 1.5 — `YourControl` listeners ("whenever a creature you control
        // deals combat damage to a player" — Enduring Curiosity), controller
        // must match the attacker's. Includes the dealer itself, whose
        // `YourControl` trigger Phase 1 (`SelfSource`-only) skips.
        //
        // 1.6 — `AnyPlayer` listeners on other permanents ("whenever a Goblin
        // deals combat damage" — Cabal Slaver), gated by
        // `EventSpec.dealer_filter`; the dealer's own fired in Phase 1.
        //
        // ⚠ Every 1.5 push must precede every 1.6 push within a `by_kind`
        // bucket (buckets reach the stack in order), so 1.6 hits buffer in
        // `any_player` and drain after (PERF (-68)).
        //
        // A plain nested loop (nothing borrows `self` mutably, so nothing to
        // buffer). Each permanent's trigger walk is gated by the
        // `dispatch_bits::LISTENER` memo bit, and a board with no listener
        // skips the walk via the zone's listener lane, which this walk fills
        // exactly (see `zone::Battlefield::listener_lane`).
        let mut any_player: Vec<(usize, DamageTrigger)> = Vec::new();
        let lane = self.battlefield.listener_lane();
        if lane != Ok(false) {
            let mut any_listener = false;
            // CR 113.10b — a stripped listener hears nothing.
            let strip_set = self.stripped_permanents();
            for c in &self.battlefield {
                if c.dispatch_scan_bits() & crate::card::dispatch_bits::LISTENER == 0 {
                    continue;
                }
                // The lane records that a listener exists, stripped or not.
                any_listener = true;
                if strip_set.contains(&c.id) {
                    continue;
                }
                let mine = attacker_controller == Some(c.controller);
                let other = c.id != source;
                for (idx, t) in c.definition.triggered_abilities.iter().enumerate() {
                    match t.event.scope {
                        crate::effect::EventScope::YourControl if mine => {
                            // "Whenever a Mutant, Ninja or Turtle you control
                            // deals combat damage" gates on the dealer too.
                            if let Some(i) = slot(&t.event)
                                && t.event.dealer_filter.as_ref().is_none_or(|f| {
                                    self.evaluate_requirement_static(
                                        f,
                                        &Target::Permanent(source),
                                        c.controller,
                                        Some(c.id),
                                    )
                                })
                            {
                                by_kind[i].push((
                                    c.id,
                                    t.effect.clone(),
                                    c.controller,
                                    t.event.filter.clone(),
                                    true,
                                    once_key(Some(idx), t),
                                ));
                            }
                        }
                        crate::effect::EventScope::AnyPlayer if other => {
                            if let Some(i) = slot(&t.event)
                                && t.event.dealer_filter.as_ref().is_none_or(|f| {
                                    self.evaluate_requirement_static(
                                        f,
                                        &Target::Permanent(source),
                                        c.controller,
                                        Some(c.id),
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
                                        once_key(Some(idx), t),
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
        // The same two scopes granted by a static (Feywild Visitor's
        // commander grant); the walk above reads printed triggers only.
        let grants: Vec<_> = self
            .trigger_grant_sources()
            .into_iter()
            .filter(|g| {
                matches!(
                    g.ability.event.scope,
                    crate::effect::EventScope::YourControl | crate::effect::EventScope::AnyPlayer
                ) && slot(&g.ability.event).is_some()
            })
            .collect();
        if !grants.is_empty() {
            let strip_set = self.stripped_permanents();
            for c in &self.battlefield {
                if strip_set.contains(&c.id) {
                    continue;
                }
                let mine = attacker_controller == Some(c.controller);
                // A granted ability has no printed index; its position in this
                // permanent's (stable) grant list keys the once-per-batch dedupe.
                for (j, t) in self.statics_granted_triggers_with(c, &grants).into_iter().enumerate() {
                    let listens = match t.event.scope {
                        crate::effect::EventScope::YourControl => mine,
                        crate::effect::EventScope::AnyPlayer => c.id != source,
                        _ => false,
                    };
                    let Some(i) = slot(&t.event) else { continue };
                    if !listens
                        || !t.event.dealer_filter.as_ref().is_none_or(|f| {
                            self.evaluate_requirement_static(f, &Target::Permanent(source), c.controller, Some(c.id))
                        })
                    {
                        continue;
                    }
                    let key = once_key(Some(usize::MAX / 2 + j), &t);
                    let trig = (c.id, t.effect.clone(), c.controller, t.event.filter.clone(), true, key);
                    if t.event.scope == crate::effect::EventScope::AnyPlayer {
                        any_player.push((i, trig));
                    } else {
                        by_kind[i].push(trig);
                    }
                }
            }
        }
        for (i, trig) in any_player {
            by_kind[i].push(trig);
        }

        // Phase 2: `FromYourGraveyard` triggers in the attacker controller's
        // graveyard ("whenever one or more creatures you control deal combat
        // damage to a player"). One fire per damage batch (CR 603.2):
        // `combat_trigger_fired_this_step` dedupes the per-attacker walks and
        // is cleared at each damage sub-step.
        //
        // The one walk that stays per kind: the dedupe set is read and written
        // between kinds, so merging would fire one card for two kinds. Behind
        // the pile's own lane, so a graveyard with no such card costs a word
        // load (PERF `(-210)`).
        if let Some(atk_controller) = attacker_controller
            && self.players[atk_controller].graveyard.has_graveyard_trigger()
        {
            // Cross-kind dedupe within one dispatch: a noncombat delivery has
            // no `batch_subject` but still fires a graveyard card once.
            let mut fired_here: Vec<CardId> = Vec::new();
            for (i, kind) in kinds.iter().enumerate() {
                let mut fired: Vec<(CardId, usize, BatchSubject)> = Vec::new();
                for player in &self.players {
                    if player.id.0 != atk_controller {
                        continue;
                    }
                    for gy_card in &player.graveyard {
                        // Per damaged subject (CR 603.2c): the same graveyard
                        // card fires again for the next defending seat.
                        if fired_here.contains(&gy_card.id)
                            || self
                                .combat_trigger_fired_this_step
                                .iter()
                                .any(|&(c, _, s)| c == gy_card.id && Some(s) == batch_subject)
                        {
                            continue;
                        }
                        for t in &gy_card.definition.triggered_abilities {
                            if t.event.kind == *kind && t.event.scope.from_graveyard() {
                                by_kind[i].push((
                                    gy_card.id,
                                    t.effect.clone(),
                                    gy_card.owner,
                                    t.event.filter.clone(),
                                    false,
                                    None,
                                ));
                                fired_here.push(gy_card.id);
                                if let Some(subject) = batch_subject {
                                    fired.push((gy_card.id, usize::MAX, subject));
                                }
                            }
                        }
                    }
                }
                // Cold-group guard — see `clear_cold!`.
                if !fired.is_empty() {
                    self.combat_trigger_fired_this_step.extend(fired);
                }
            }
        }

        // CR 702.99 — Cipher: an encoded card offers a free copy whenever the
        // creature deals combat damage to a player, via the Paradigm
        // free-copy effect (the encoded original stays in exile).
        if let Some(i) = kinds.iter().position(|k| *k == EventKind::DealsCombatDamageToPlayer).filter(|_| !to_battle)
            && let Some(atk_ctrl) = attacker_controller
        {
            // Behind the pile's lane: `encoded_on` sits behind each exiled
            // card's `Arc`, so the walk was a pointer chase per exiled card
            // per damage event on a zone that only grows (PERF `(-218)`).
            if self.exile.has_encoded() {
                for enc in &self.exile {
                    if enc.encoded_on == Some(source) {
                        by_kind[i].push((enc.id, Effect::CastFreeParadigmCopy, atk_ctrl, None, false, None));
                    }
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
                    None,
                ));
            }
        }

        // Bodies acting on something the damaged player controls (Hammer of
        // Ruin, Mordant Dragon) read the seat via `ControlledByTriggerPlayer`,
        // here and at resolution.
        if let Target::Player(p) = default_target {
            self.trigger_event_player_scratch = Some(p);
        }
        for (kind_i, (trig_source, effect, controller, filter, bind_dealer, once)) in by_kind
            .into_iter()
            .enumerate()
            .flat_map(|(i, bucket)| bucket.into_iter().map(move |e| (i, e)))
        {
            // CR 510.2 — "whenever ~ deals combat damage" names no recipient:
            // one event per dealer per step, amount the total. A later
            // recipient adds to the fire already on the stack.
            let agnostic = combat_batch
                && matches!(kinds[kind_i], EventKind::DealsCombatDamage | EventKind::DealsDamage);
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
                // "Deals exactly 1 damage" (Ghyrson Starn) reads the amount.
                ctx.event_amount = damage_amount;
                if !self.evaluate_predicate(pred, &ctx) {
                    continue;
                }
            }
            // CR 603.3d / 603.2c — one fire a turn or per damage batch,
            // checked after the filter; the batch slot lives on the
            // per-sub-step set since this hook runs per dealer. Hordewing
            // Skaab's "number of opponents dealt damage this way" counts the
            // distinct players hit.
            let mut counts_players = false;
            let noncombat_key = (noncombat_batch && matches!(kinds[kind_i], EventKind::DealsDamage))
                .then_some((trig_source, usize::MAX - 1, BatchSubject::NoncombatDealer(source)));
            if let Some(key) = noncombat_key
                && self.combat_trigger_fired_this_step.contains(&key)
            {
                self.add_to_dealer_fire(trig_source, source, damage_amount);
                continue;
            }
            if agnostic {
                let key = (trig_source, once.map_or(usize::MAX - 1, |(i, _)| i), BatchSubject::Dealer(source));
                if self.combat_trigger_fired_this_step.contains(&key) {
                    self.add_to_dealer_fire(trig_source, source, damage_amount);
                    continue;
                }
                self.combat_trigger_fired_this_step.push(key);
            } else if let Some((i, per_turn)) = once {
                if per_turn {
                    if !self.triggered_once_per_turn_used.insert((trig_source, i)) {
                        continue;
                    }
                } else if let Some(mut subject) = batch_subject {
                    // CR 603.2c — one fire per damaged subject: two defending
                    // seats in one alpha strike are two events. "…to one or
                    // more players" batches over the seats too.
                    if matches!(subject, BatchSubject::Player(_))
                        && self.battlefield_find(trig_source).is_some_and(|c| {
                            c.definition
                                .triggered_abilities
                                .get(i)
                                .is_some_and(|t| t.event.batch_across_players)
                        })
                    {
                        if matches!(subject, BatchSubject::Player(_))
                            && self.battlefield_find(trig_source).is_some_and(|c| {
                                c.definition.triggered_abilities.get(i).is_some_and(|t| t.event.batch_counts_subjects)
                            })
                        {
                            counts_players = true;
                        }
                        subject = BatchSubject::AnyPlayer;
                    }
                    // "…deals damage to one or more creatures" (Briarbridge
                    // Patrol): one fire per dealer, however many blockers.
                    if matches!(subject, BatchSubject::Permanent(_))
                        && matches!(
                            kinds[kind_i],
                            EventKind::DealsDamageToCreature | EventKind::DealsCombatDamageToCreature
                        )
                    {
                        subject = BatchSubject::Dealer(source);
                    }
                    let key = (trig_source, i, subject);
                    if self.combat_trigger_fired_this_step.contains(&key) {
                        self.add_to_batched_damage(trig_source, i, &default_target, damage_amount);
                        if matches!(effect, Effect::OverTriggerBatch { .. }) {
                            self.add_dealer_to_trigger_batch(trig_source, &default_target, source);
                        }
                        continue;
                    }
                    self.combat_trigger_fired_this_step.push(key);
                    // The first player the batch counts.
                    if counts_players && let Target::Player(p) = default_target {
                        self.combat_trigger_fired_this_step.push((trig_source, i, BatchSubject::Player(p)));
                    }
                }
            }
            // Keyed only once the fire survived its once-per-turn gate, so a
            // later recipient never merges into an older fire.
            if let Some(key) = noncombat_key {
                self.combat_trigger_fired_this_step.push(key);
            }
            let agnostic = agnostic || noncombat_key.is_some();
            // CR 603.2c — "each of those creatures": the batch's dealers,
            // this one first; later dealers join through
            // `add_dealer_to_trigger_batch` (Heroes in a Half Shell).
            let effect = match effect {
                Effect::OverTriggerBatch { body, .. } => Effect::OverTriggerBatch { body, ids: vec![source] },
                e => e,
            };
            // Most combat-damage triggers target the damaged player, but some
            // target a graveyard card (Efreet Flamepainter) or a permanent
            // (Sword of Sinew and Steel); a slot-0 filter that can't match a
            // player is the tell, and those auto-pick instead.
            // CR 700.2a — a modal trigger's mode first; the slot is its mode's.
            let mode = self.pick_trigger_mode(&effect, trig_source, controller);
            let view = effect.targeting_view(mode);
            let slot0_filter = view.target_filter_for_slot_in_mode_kicked(0, None, false);
            let slot0_rejects_player = slot0_filter.is_some_and(|f| !f.can_match_player());
            // A slot 0 that accepts a player is the damaged player (Lord of
            // the Void), even if a later clause moves a card.
            let slot0_accepts_player = slot0_filter.is_some_and(|f| f.can_match_player());
            // CR 115.1 / 608.2b — a default the slot can't take (a damaged
            // *creature* on a "target player mills" trigger — Towering-Wave
            // Mystic) would only fizzle: pick a legal target instead.
            let default_misfits = slot0_filter.is_some_and(|f| {
                !self.evaluate_requirement_static(f, &default_target, controller, Some(trig_source))
            });
            // CR 115.4 / 603.3d — a printed "any target" (a player *or* a
            // permanent: Sword of Fire and Ice) is the controller's choice,
            // not "that player"; only a bare or player-only slot is bound.
            let slot0_is_choice = slot0_filter.is_some_and(|f| {
                f.can_match_player()
                    && !f.is_player_only()
                    && !matches!(f, crate::card::SelectionRequirement::Any | crate::card::SelectionRequirement::Not(_))
            });
            let picked = default_misfits
                || slot0_is_choice
                || (!slot0_accepts_player && (view.prefers_graveyard_target() || slot0_rejects_player));
            let target = if picked {
                // Concretize any X-from-cost gate against the damage dealt
                // (Venerable Warsinger's "mana value X or less, where X is
                // the damage this creature dealt to that player").
                self.auto_target_for_effect_avoiding_set_x(
                    view,
                    controller,
                    &[trig_source],
                    damage_amount,
                )
                .or(Some(default_target.clone()))
            } else {
                Some(default_target.clone())
            };
            // The trigger's subject: the dealer under `bind_dealer` (Phase 1.5
            // "whenever a creature deals combat damage to you" — "it" is the
            // dealer), otherwise the damaged player ("that player"). This
            // must match `damage_event_subject` in the event dispatcher; it
            // once bound `None`, so `PlayerRef::Triggerer` bodies did nothing
            // (Phage the Untouchable).
            let dealer = if bind_dealer || agnostic {
                Some(crate::game::effects::EntityRef::Permanent(source))
            } else {
                match default_target {
                    Target::Player(p) => Some(crate::game::effects::EntityRef::Player(p)),
                    _ => None,
                }
            };
            // Felix Five-Boots — a trigger of a permanent you control caused
            // by a creature you control hitting a player fires once more per
            // such static.
            let extra = if combat_batch
                && matches!(default_target, Target::Player(_))
                && attacker_controller == Some(controller)
                && self.battlefield_find(trig_source).is_some()
            {
                self.battlefield
                    .iter()
                    .filter(|c| c.controller == controller)
                    .flat_map(|c| &c.definition.static_abilities)
                    .filter(|sa| {
                        matches!(
                            sa.effect,
                            crate::effect::StaticEffect::DoubleControllerCombatDamageToPlayerTriggers
                        )
                    })
                    .count()
            } else {
                0
            } + self.source_trigger_extra_fires(trig_source);
            // CR 603.3d — a slot the engine picks (not the bound damaged
            // player) is a prompting controller's to name: Sword of Sinew and
            // Steel, Efreet Flamepainter. A batched or merged fire stays
            // direct, since later dealers find it on the stack.
            if picked && !agnostic && once.is_none() && extra == 0 && self.trigger_asks_targets(controller, view) {
                self.queue_trigger_asking(crate::game::types::PendingTriggerPush {
                    source: trig_source,
                    controller,
                    effect,
                    subject: dealer,
                    event_amount: if counts_players { 1 } else { damage_amount },
                    mode,
                    intervening_if: None,
                    actor: match default_target {
                        Target::Player(p) => Some(p),
                        _ => None,
                    },
                    from_mana_ability: false,
                    x_value: damage_amount,
                    converged_value: 0,
                    mana_spent: 0,
                });
                continue;
            }
            // CR 115.1c — slots past the first ("… and target player draws X
            // cards", Niv-Mizzet, Guildpact), as the death funnels fill them.
            let additional = self.auto_extra_targets_for(&effect, trig_source, controller, target.clone());
            for _ in 0..extra {
                self.push_stack(
                    TriggerPush::new(trig_source, controller, effect.clone())
                        .target(target.clone())
                        .additional_targets(additional.clone())
                        .mode(mode)
                        .trigger_source(dealer)
                        .trigger_player(match default_target {
                            Target::Player(p) => Some(p),
                            _ => None,
                        })
                        .x_value(damage_amount)
                        .event_amount(damage_amount)
                        .build(),
                );
            }
            self.push_stack(
                TriggerPush::new(trig_source, controller, effect)
                    .target(target)
                    .additional_targets(additional)
                    .mode(mode)
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
                    .event_amount(if counts_players { 1 } else { damage_amount })
                    .build(),
            );
        }
    }
}

impl GameState {
    /// A recipient-agnostic damage fire already on the stack for `dealer`
    /// (CR 510.2 combat, CR 603.2c noncombat) takes a later recipient's
    /// damage instead of firing again.
    pub(crate) fn add_to_dealer_fire(&mut self, trig_source: CardId, dealer: CardId, amount: u32) {
        let dealer = Some(crate::game::effects::EntityRef::Permanent(dealer));
        if let Some(StackItem::Trigger { event_amount, x_value, .. }) = self.stack.iter_mut().rev().find(|si| {
            matches!(si, StackItem::Trigger { source: s, trigger_source, .. }
                if *s == trig_source && *trigger_source == dealer)
        }) {
            *event_amount += amount;
            *x_value += amount;
        }
    }

    /// A later dealer of a damage batch whose trigger is already on the stack
    /// joins its `Effect::OverTriggerBatch` ("each of those creatures").
    fn add_dealer_to_trigger_batch(&mut self, src: CardId, to: &Target, dealer: CardId) {
        let Target::Player(p) = *to else { return };
        if let Some(StackItem::Trigger { effect, .. }) = self.stack.iter_mut().rev().find(|si| {
            matches!(si, StackItem::Trigger { source, trigger_player, .. } if *source == src && *trigger_player == Some(p))
        }) && let Effect::OverTriggerBatch { ids, .. } = effect.as_mut()
            && !ids.contains(&dealer)
        {
            ids.push(dealer);
        }
    }

    /// `EventSpec::batch_sums_damage` — a later dealer in an already-fired
    /// batch adds its damage to the fire on the stack, so the one trigger
    /// reads the batch's total. A no-op for every other batched trigger.
    fn add_to_batched_damage(&mut self, src: CardId, idx: usize, to: &Target, amount: u32) {
        let Some(event) = self
            .battlefield_find(src)
            .and_then(|c| c.definition.triggered_abilities.get(idx).map(|t| (t.event.batch_sums_damage, t.event.batch_counts_subjects && t.event.batch_across_players)))
        else {
            return;
        };
        let (sums, counts_players) = event;
        let Target::Player(p) = *to else { return };
        // A player the across-players batch hasn't counted yet adds one.
        if counts_players {
            let seen = (src, idx, BatchSubject::Player(p));
            if !self.combat_trigger_fired_this_step.contains(&seen) {
                self.combat_trigger_fired_this_step.push(seen);
                if let Some(StackItem::Trigger { event_amount, .. }) = self
                    .stack
                    .iter_mut()
                    .rev()
                    .find(|si| matches!(si, StackItem::Trigger { source, .. } if *source == src))
                {
                    *event_amount += 1;
                }
            }
            return;
        }
        if !sums {
            return;
        }
        if let Some(StackItem::Trigger { event_amount, x_value, .. }) =
            self.stack.iter_mut().rev().find(|si| {
                matches!(si, StackItem::Trigger { source, trigger_player, .. }
                    if *source == src && *trigger_player == Some(p))
            })
        {
            *event_amount += amount;
            *x_value += amount;
        }
    }
}

/// The combat-damage value a creature assigns (CR 510.1c): its toughness
/// under `AssignsCombatDamageByToughness` (Doran; unconditional, so a 5/1
/// assigns 1), |power| for a negative-power
/// `AssignsCombatDamageAsThoughPowerPositive` bearer (Loot), else its power.
fn combat_damage_value(cp: &ComputedPermanent) -> i32 {
    if cp.keywords().has_kw(&Keyword::AssignsCombatDamageByToughness) {
        cp.toughness
    } else if cp.power < 0 && cp.keywords().has_kw(&Keyword::AssignsCombatDamageAsThoughPowerPositive) {
        -cp.power
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
    may_assign_as_unblocked: bool,
    must_assign_as_unblocked: bool,
    /// Butcher Orgg (CR 510.1a variant) — read here so the three consumers
    /// don't each linear-scan `computed` for this attacker. See
    /// [`GameState::free_division_targets`].
    free_divider: bool,
    should_deal: bool,
}

impl GameState {
    /// Wake `p`'s dormant `TurnsHolderAttacksWithType` grants whose type the
    /// declared attacker `id` has. One exile read when none is parked.
    fn wake_may_play_for_attacker_type(&mut self, id: CardId, p: usize) {
        let mut types: Vec<crate::card::CreatureType> = Vec::new();
        for c in self.exile.iter() {
            if let Some(perm) = c.may_play_until
                && perm.player == crate::card::MAY_PLAY_DORMANT
                && let crate::card::MayPlayDuration::TurnsHolderAttacksWithType { holder, creature_type } = perm.duration
                && holder == p
                && !types.contains(&creature_type)
            {
                types.push(creature_type);
            }
        }
        for creature_type in types {
            if self.permanent_has_creature_type(id, creature_type) {
                self.wake_may_play(crate::card::MayPlayDuration::TurnsHolderAttacksWithType { holder: p, creature_type }, p);
            }
        }
    }

    /// Re-arm every dormant exile permission of exactly `duration` for `p`
    /// (Neyali's token attacks, Neriv's commander attacks).
    fn wake_may_play(&mut self, duration: crate::card::MayPlayDuration, p: usize) {
        let wakes = |c: &crate::card::CardInstance| {
            c.cold_any(|k| {
                k.may_play_until.is_some_and(|perm| {
                    perm.player == crate::card::MAY_PLAY_DORMANT && perm.duration == duration
                })
            })
        };
        if self.exile.iter().any(wakes) {
            for c in self.exile.iter_mut() {
                if wakes(c)
                    && let Some(perm) = c.may_play_until
                {
                    c.may_play_until = Some(crate::card::MayPlayPermission { cast_only: false, locks_further_casts: false, player: p, ..perm });
                }
            }
        }
    }
}

/// CR 614 / 615.5 / 615.12 — a damage-to-counters static meeting `amount`
/// damage: `(counters put, damage still dealt)`.
pub(crate) fn damage_to_counters_split(
    how: crate::effect::DamageToCounters,
    unpreventable: bool,
    amount: u32,
) -> (u32, u32) {
    use crate::effect::DamageToCounters as How;
    match (how, unpreventable) {
        (How::Instead, _) | (_, false) => (amount, 0),
        (How::PreventAndPut, true) => (amount, amount),
        (How::PreventPerPoint, true) => (0, amount),
    }
}

#[cfg(test)]
mod tests {
    use crate::game::types::{Attack, AttackTarget};

    /// CR 508.5 / 511.3 — the attacked planeswalker's recorded defender ends
    /// with combat even when every attacker left combat first; a stale record
    /// named the old controller as a later attack's defender (seed 4400396).
    #[test]
    fn cr_508_5_the_defender_record_ends_with_combat_after_its_attackers() {
        let mut g = crate::game::multi_player_game(3);
        let pw = g.add_card_to_battlefield(1, crate::catalog::jace_the_mind_sculptor());
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.attacking.push(Attack { attacker: bear, target: AttackTarget::Planeswalker(pw) });
        g.note_attack_defender(AttackTarget::Planeswalker(pw));
        g.attacking.clear();
        g.remove_all_from_combat();
        if let Some(c) = g.battlefield_find_mut(pw) {
            c.controller = 2;
        }
        g.note_attack_defender(AttackTarget::Planeswalker(pw));
        assert_eq!(g.recorded_attack_defender(pw), Some(2), "the new attack's defender is the new controller");
    }
}
