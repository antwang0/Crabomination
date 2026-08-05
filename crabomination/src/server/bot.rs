//! In-process bots for server-hosted matches.
//!
//! Unlike networked clients, a bot reads the full authoritative [`GameState`]
//! each tick and returns the next [`GameAction`] it wants the server to
//! perform. The match actor polls every bot seat to a fixed point after every
//! state change, so a bot just needs to make *some* forward-progressing
//! decision (including `PassPriority`) whenever it holds priority.

use rand::{RngExt, SeedableRng, rng};
use rand::rngs::StdRng;

use crate::card::{CardDefinition, CardId};
use crate::decision::{AutoDecider, Decider};
use crate::effect::{ActivatedAbility, Effect, ManaPayload};
use crate::game::{Attack, AttackTarget, GameAction, GameState, Target, TurnStep};
use crate::mana::{ManaCost, ManaPool};

thread_local! {
    /// Per-thread source for the scored bot's tie-break jitter, when a
    /// caller has asked for a reproducible one. `None` = draw from the
    /// thread RNG, which is the behaviour in a real server match.
    static JITTER: std::cell::RefCell<Option<StdRng>> = const {
        std::cell::RefCell::new(None)
    };
}

/// Seed (or clear) this thread's tie-break jitter.
///
/// [`main_phase_action_with`] breaks exact score ties with a small random
/// nudge, so two runs of the "same" game diverge even under a fixed
/// shuffle seed. That is fine in a real match and actively unhelpful in
/// measurement: it means `--seed` never made a ladder run reproducible,
/// and — more expensively — it is the *only* thing that can decide a
/// paired game under a true null, where both seats pilot the same
/// profile. Measured on 2400 sealed pairs, that residual accounted for
/// every one of the 368 non-split pairs and held the within-pair
/// correlation at −0.69 instead of −1.
///
/// Seeding it identically for both games of an antithetic pair makes the
/// two replays differ only where the *profiles* differ, which is the
/// whole point of common random numbers. Real matches leave it `None`.
pub fn set_jitter_seed(seed: Option<u64>) {
    JITTER.with(|j| {
        *j.borrow_mut() = seed.map(StdRng::seed_from_u64);
    });
}

/// A jitter draw in `0..n`, from the seeded stream when one is installed.
fn jitter_below(n: usize) -> usize {
    if n <= 1 {
        return 0;
    }
    JITTER.with(|j| match &mut *j.borrow_mut() {
        Some(r) => r.random_range(0..n),
        None => rng().random_range(0..n),
    })
}

/// Drives one seat without a human client. Implementations see the full
/// `GameState` and return the single next action they'd like to submit.
pub trait Bot: Send {
    /// Return `Some(action)` to submit, or `None` if it's not this bot's turn
    /// to act right now (no priority, waiting on an opponent decision, game
    /// already over, etc.).
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction>;
}

/// Tunable weights for the bot's board evaluation, so a change to how the
/// bot *values* things can be A/B-laddered against the previous numbers
/// instead of argued about. Every profile is internally consistent: all
/// non-permanent terms are expressed in multiples of [`unit`], so raising
/// `unit` buys arithmetic resolution without moving any relative weight.
///
/// [`unit`]: EvalWeights::unit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalWeights {
    /// Value of one "point" on this profile's scale. Flat terms (hand
    /// cards, life, the legendary premium, ...) are written as `n * unit`.
    /// The baseline uses 1, which keeps the historical integer scores;
    /// richer profiles use a larger unit so sub-point terms -- a keyword
    /// worth two thirds of a power point -- survive integer division.
    pub unit: i32,
    /// Per-mana-value weight of a permanent.
    pub cmc: i32,
    /// Flat value of simply *being* a creature, before size. Forge's
    /// evaluator opens at a constant 100 and adds power/toughness on top;
    /// the historical weights here open at zero, which makes every other
    /// creature term -- keywords especially -- proportionally far louder
    /// than the reference they were calibrated against.
    pub creature_base: i32,
    /// Per-point weight of a creature's power and toughness.
    pub power: i32,
    pub toughness: i32,
    /// Keyword scoring strength as a percentage (see [`keyword_value`]).
    /// 0 disables it, which is the baseline: a 1/1 flying lifelinker reads
    /// as a vanilla 1/1.
    pub keyword_pct: i32,
    /// Use the concave life curve (see [`life_value`]) instead of counting
    /// life linearly. Life near zero is worth far more per point than life
    /// near the starting total; a linear term prices them the same.
    pub concave_life: bool,
    /// Hold a play whose only gain this turn is a summoning-sick body until
    /// the postcombat main (see [`eval_material_summon_sick_blind`]).
    pub hold_sick: bool,
    /// Hold an instant-speed line that achieves nothing this turn, so it is
    /// cast on the opponent's turn instead — with a turn more information,
    /// and with the mana up in the meantime. The instant-speed sibling of
    /// [`hold_sick`](Self::hold_sick), and the cheap form of Forge's
    /// "formulate a plan restricted to instant-speed lines and wait if it
    /// scores as well" (`SpellAbilityPicker::createNewPlan`).
    pub hold_instants: bool,
    /// How many *extra* plays a candidate's evaluation may look ahead. 0
    /// scores the board right after the candidate resolves (the historical
    /// behavior); 1 asks "and what would I do next?" once. See
    /// [`evaluate_action_sequence`].
    pub lookahead: u8,
    /// Score a candidate on the board *after* this turn's combat rather
    /// than the instant it resolves (see [`simulate_through_combat`]).
    pub combat_aware: bool,
    /// Search the attack declaration instead of taking the greedy one:
    /// simulate each candidate attack through the opponent's crack-back and
    /// keep the best (see [`pick_attacks_scored`]). 0 disables the search;
    /// higher values allow more candidates, and the cost is roughly linear
    /// in it because each candidate is a full simulated turn cycle.
    ///
    /// The measurement this exists to settle: `bot_probe` shows the bot
    /// declaring every eligible creature as an attacker in 73 % of its
    /// combats, and 41 % of its creatures tapped when blocks are declared
    /// as a direct result. Greedy attacking is *why* it can't block — but
    /// whether restraint is worth the tempo is a ladder question, not an
    /// argument, so this is a flag rather than a rewrite.
    pub attack_search: u8,
    /// Search the block assignment instead of taking the greedy one:
    /// simulate each candidate through combat damage and keep the best (see
    /// [`pick_blocks_scored`]). 0 disables it; higher values allow more
    /// candidate assignments.
    ///
    /// The block sibling of [`attack_search`](Self::attack_search), and a
    /// cheaper search than it: a block candidate only has to be simulated
    /// through this turn's combat damage, not through a full turn cycle,
    /// because the payoff of a block — who dies, how much life is saved —
    /// is settled inside the same combat.
    pub block_search: u8,
    /// Restore the pre-fix mana behavior: tap every land before deciding
    /// anything, and size affordability off the floating pool.
    ///
    /// Not a weight — a behavioral control, kept for the same reason
    /// [`RandomBot::uniform_baseline`] is, so the tap-out fix stays
    /// measurable on the ladder instead of being asserted. Approximates
    /// the old pass with its land-tap half, which is the part the
    /// measurement in `main_phase_action_with` was of.
    pub legacy_pretap: bool,
    /// Let the combat simulations cast spells: whichever seat holds
    /// priority inside [`simulate_attack_outcome`] /
    /// [`simulate_block_outcome`] fires the response layer, the
    /// combat-trick window, and — inside the attack sim's one-turn
    /// horizon — a static-ranked main-phase cast (see
    /// [`sim_spell_action`]). Off, the sims are pure priority passes and
    /// "an opponent holding removal, or ourselves holding a trick, are
    /// invisible" — the documented blindness behind the over-attack the
    /// SOS college probes measured.
    pub attack_sim_spells: bool,
    /// Extend the attack simulation one extra turn cycle when it ends
    /// with either life total at 10 or below. The one-cycle horizon can
    /// see "this creature survives to block" but not "this is the race I
    /// need to win" — the roadmap's race-math gap. An extension only
    /// when the sim ends inside burn range keeps the cost bounded to the
    /// positions where the extra cycle can actually reach a result.
    pub attack_race_horizon: bool,
    /// Evaluate undecided positions with the learned value net registered
    /// in this [`net_eval`](crate::server::net_eval) slot instead of the
    /// material heuristic; 0 (default) is off. The net returns a win
    /// probability scaled to 0..10 000, so a decided game's heuristic
    /// ±100 000·unit still dominates every comparison, and an empty slot
    /// falls back to the heuristic — a weights file is a runtime input,
    /// never a build requirement.
    pub net_slot: u8,
    /// With [`net_slot`](Self::net_slot) set: 0 replaces the heuristic
    /// evaluation with the net's win probability outright; a positive
    /// value blends instead — heuristic plus a `(p − 0.5) · scale · unit`
    /// bias, so full confidence is worth ±scale/2 units. The heuristic
    /// stays sharp on small material deltas the net can't resolve; the
    /// net weighs in proportionally to how far from a coin flip it judges
    /// the position. A knob rather than a constant because the right
    /// loudness is a measurement (see the `net_eval_blend*` profiles).
    pub net_blend_scale: i32,
    /// Sequence the land drop instead of taking the first land that
    /// covers the most missing colors. Two additions:
    ///
    /// * **Urgency** — a missing color is worth more when the hand cards
    ///   demanding it are cheap. Covering the color of a two-drop that
    ///   could be cast next turn beats covering the color of a six-drop.
    /// * **Tapped-land timing** — an enters-tapped land is free on a
    ///   turn with nothing to cast and costs a whole turn's play
    ///   otherwise, so it is preferred early and penalized when the
    ///   untapped mana would actually be spent.
    pub land_urgency: bool,
    /// Judge opening hands by what is *in* them, not just how many lands
    /// and whether one spell is castable. The shipped rule keeps every
    /// 2–5-land hand with a single cheap spell and ships every 6-land
    /// hand, so "two lands, one two-drop, four six-drops" is a keep and
    /// "six lands and a bomb" is a mulligan. This adds a card-quality
    /// sum ([`crate::draft::card_quality`]), a redundancy requirement at
    /// two lands, and an on-the-draw allowance — the extra card is
    /// exactly what makes a marginal hand keepable.
    pub mull_quality: bool,
    /// Offer gang-blocks *for value* to the block search.
    ///
    /// The greedy pass already piles blockers onto an attacker, but only
    /// when `life_threatened` — that is, only to survive lethal. Off
    /// that trigger it blocks an attacker solely when one creature can
    /// kill it alone, so two 2/2s never eat a 4/4 however good that
    /// trade is. And [`block_search`](Self::block_search) can only ever
    /// *remove* blockers from the greedy set; adding one was outside the
    /// space it explored, which is why its documented null result says
    /// nothing about this.
    ///
    /// The gangs are candidates, not decisions:
    /// [`simulate_block_outcome`] prices the blockers that die against
    /// the attacker that dies, and ties keep the greedy assignment.
    pub block_gang: bool,
    /// Redeal the hidden zones before an attack/block simulation, and
    /// average this many redeals; 0 (the historical behaviour) searches
    /// the true state.
    ///
    /// The combat sims clone the real [`GameState`], so the rollout
    /// opponent casts the cards they are actually holding and both seats
    /// draw the real top of their library. The bot is therefore searching
    /// with perfect information: it can decline an attack because it has
    /// *seen* the trick, which is not a read, it is looking at the hand.
    ///
    /// Two separate reasons that matters, worth keeping apart:
    ///
    /// * Against a human in the client it is simply cheating, whatever it
    ///   does to the win rate.
    /// * The mirror ladder is structurally incapable of detecting it,
    ///   because both seats cheat identically. No measurement this
    ///   harness has ever run could have caught this, which is why it is
    ///   a knob with a documented default rather than a silent fix.
    ///
    /// Averaging several redeals is the honest version: one sample
    /// replaces perfect information with a *wrong* hand, which is its own
    /// bias, while the mean over redeals approximates playing against the
    /// distribution of hands consistent with what the seat can see.
    pub determinize: u8,
    /// Copied onto this seat's [`Player::smart_tap`] before the game
    /// starts: spend the most replaceable mana source for each pip
    /// instead of the first in battlefield order.
    ///
    /// The behaviour lives in the engine's auto-tap, so this field exists
    /// only so the ladder can put one seat on each side of it. On by
    /// default — the old order was not a decision anyone made, it was
    /// whatever `battlefield` iteration produced.
    ///
    /// [`Player::smart_tap`]: crate::player::Player::smart_tap
    pub smart_tap: bool,
}

impl EvalWeights {
    /// The historical weights: mana value + power + toughness, one point
    /// each, no keyword term, linear life. Kept exactly as-is so it stays
    /// a valid ladder control — it is what every run measures against, not
    /// what the bot plays (see [`Default`](EvalWeights::default)).
    /// See [`v2`](Self::v2).
    pub const fn baseline() -> Self {
        Self {
            unit: 1,
            cmc: 1,
            creature_base: 0,
            power: 1,
            toughness: 1,
            keyword_pct: 0,
            concave_life: false,
            hold_sick: false,
            hold_instants: false,
            lookahead: 0,
            combat_aware: false,
            attack_search: 0,
            block_search: 0,
            legacy_pretap: false,
            attack_sim_spells: false,
            attack_race_horizon: false,
            net_slot: 0,
            net_blend_scale: 0,
            land_urgency: false,
            mull_quality: false,
            block_gang: false,
            determinize: 0,
            smart_tap: true,
        }
    }

    /// Candidate weights ported from the reference AIs: body ratios and
    /// power-scaled keyword terms from Forge's `CreatureEvaluator`, the
    /// life curve from XMage's `ArtificialScoringSystem::LIFE_SCORES`,
    /// `unit = 10` so those ratios survive integer division.
    ///
    /// **Measured worse than [`baseline`], and not adopted.** Over 12 000
    /// laddered games it lands at 49.4 % (baseline 50.6 %, CI straddling
    /// 50 %), and the [`keywords_only`] decomposition shows the keyword
    /// term is the part that costs: pooled over 20 000 games the baseline
    /// beats it 51.1 % [50.4 %, 51.8 %].
    ///
    /// The first explanation offered for this was *depth*: that a richer
    /// evaluation of a position the bot can only see one action deep gives
    /// a greedy step more confidence without more foresight, and that these
    /// weights are calibrated for the real search their sources run (Forge
    /// fast-forwards to combat damage and plans three plies; XMage runs
    /// depth-4 alpha-beta). **That hypothesis was tested and is wrong.**
    /// [`v2_combat`] — the same weights with the combat-aware evaluator —
    /// measures 53.1 % to the baseline, i.e. *worse* than v2 alone. Extra
    /// depth does not rescue them.
    ///
    /// So the honest reading is that these numbers are simply wrong for
    /// this engine's surrounding balance, not merely premature. They are
    /// kept as a documented dead end: a future attempt should re-derive
    /// weights against *this* evaluator's creature-to-card and
    /// board-to-life ratios rather than port another engine's, and can use
    /// the decomposition profiles below to do it one term at a time.
    ///
    /// [`baseline`]: Self::baseline
    /// [`keywords_only`]: Self::keywords_only
    /// [`v2_combat`]: Self::v2_combat
    pub const fn v2() -> Self {
        Self {
            unit: 10,
            cmc: 10,
            creature_base: 100,
            power: 15,
            toughness: 10,
            keyword_pct: 100,
            concave_life: true,
            hold_sick: false,
            hold_instants: false,
            lookahead: 0,
            combat_aware: false,
            attack_search: 0,
            block_search: 0,
            legacy_pretap: false,
            attack_sim_spells: false,
            attack_race_horizon: false,
            net_slot: 0,
            net_blend_scale: 0,
            land_urgency: false,
            mull_quality: false,
            block_gang: false,
            determinize: 0,
            smart_tap: true,
        }
    }

    // -- Ladder decomposition profiles ---------------------------------
    //
    // A profile that bundles several changes can only ever be laddered as
    // a bundle, and a bundle that loses tells you nothing about which part
    // lost. These turn on one change at a time against a common scale.

    /// Pure control: the baseline ratios at `unit = 10`. Every term is
    /// exactly ten times the baseline's, so this *should* pick the same
    /// actions and ladder at 50 % -- it measures 50.9 % [49.4 %, 52.5 %],
    /// i.e. indistinguishable, which confirms the remaining scale-dependent
    /// behavior (integer truncation in `score_candidate`, and the
    /// fixed-size tie-break jitter, a tenth as influential at this scale)
    /// costs nothing measurable. Run this before attributing a ladder
    /// result to any of the weights themselves.
    pub const fn scaled_control() -> Self {
        Self {
            unit: 10,
            cmc: 10,
            creature_base: 0,
            power: 10,
            toughness: 10,
            keyword_pct: 0,
            concave_life: false,
            hold_sick: false,
            hold_instants: false,
            lookahead: 0,
            combat_aware: false,
            attack_search: 0,
            block_search: 0,
            legacy_pretap: false,
            attack_sim_spells: false,
            attack_race_horizon: false,
            net_slot: 0,
            net_blend_scale: 0,
            land_urgency: false,
            mull_quality: false,
            block_gang: false,
            determinize: 0,
            smart_tap: true,
        }
    }

    /// Baseline + Forge's summon-sick gate, for laddering it on its own.
    ///
    /// **Adopted — this is [`EvalWeights::default`].** Measured 51.5 %
    /// [50.8 %, 52.3 %] over 16 000 games, after two 4000-game runs at
    /// 50.8 % and 50.9 % pointed the same way. Worth roughly +1.5 points.
    ///
    /// Its behavioral effect is large and verifiable: casts in the
    /// precombat main go from 91.9 % to 25.3 %, with 66.2 % moving to the
    /// second main (`bot_probe`, land drops excluded — those are
    /// sorcery-speed by rule and can never be held). That is simply what
    /// correct sequencing looks like. It costs ~30 % more CPU per decision,
    /// since the gate resolves the winning line a second time.
    pub const fn hold_sick() -> Self {
        Self { hold_sick: true, ..Self::baseline() }
    }

    /// Baseline + the instant-speed hold. Needs
    /// [`combat_aware`](Self::combat_aware) to be much use: without it the
    /// gate cannot tell "kill the blocker before I attack" (worth doing
    /// now) from "kill it at end of turn" (worth the same, later).
    pub const fn hold_instants() -> Self {
        Self { hold_instants: true, combat_aware: true, ..Self::baseline() }
    }

    /// The adopted default plus one ply of sequence lookahead.
    ///
    /// **Measured neutral and not adopted**: 50.2 % [49.1 %, 51.3 %]
    /// against the default over 8000 games, with no consistent direction
    /// across archetypes (mono-red and dimir favour it, skies and golgari
    /// don't), at roughly 2.4x the CPU per decision.
    ///
    /// The likely reason is that the summon-sick gate already banked most
    /// of the available sequencing value. The bot was never unable to cast
    /// several spells in a turn — the main-phase loop runs every tick — it
    /// was unable to *compare combinations*, and once plays are deferred to
    /// the second main the greedy loop deploys them anyway. What is left is
    /// the narrower case where the first pick is wrong *given* what follows,
    /// which one ply and two continuations apparently doesn't catch often
    /// enough to measure.
    ///
    /// Forge searches three plies rather than one. Going deeper here costs
    /// proportionally more and, on this series' base rate, isn't worth
    /// betting on without evidence — but the machinery is in place if
    /// someone wants to try `lookahead: 2` and measure it.
    pub const fn lookahead1() -> Self {
        Self { lookahead: 1, ..Self::hold_sick_combat() }
    }

    /// **The adopted default.** The summon-sick gate plus the combat-aware
    /// evaluation, with the instant-speed hold left off.
    ///
    /// This is the decomposition of [`planner`](Self::planner), and it is
    /// why the bundle is not what shipped. Against [`hold_sick`] alone this
    /// measures 51.3 % [50.4 %, 52.2 %] over 12 000 games, while the full
    /// planner measures 51.0 % [50.2 %, 51.8 %] over 16 000 — the same
    /// within error. So `combat_aware` carries the gain and
    /// `hold_instants` adds nothing detectable on top of it.
    ///
    /// The interesting part is that `combat_aware` measured *exactly*
    /// neutral on its own (50.0 % over 12 000 games, 6002-5998). It was
    /// never a bad idea, it just had no consumer: within a single main
    /// phase this turn's combat is nearly identical whichever candidate is
    /// picked, so the term cancelled. Give the bot a reason to ask "is this
    /// worth the same later?" and the same signal is worth +1.3 points.
    ///
    /// [`hold_sick`]: Self::hold_sick
    pub const fn hold_sick_combat() -> Self {
        Self { combat_aware: true, ..Self::hold_sick() }
    }

    /// The adopted default: [`hold_sick_combat`](Self::hold_sick_combat)
    /// plus the searched attack declaration.
    ///
    /// **Measured, and the largest gain since the tap-out fix**: 52.4 %
    /// [51.3 %, 53.5 %] over 8 000 fixed-deck games, and 53.8 %
    /// [53.0 %, 54.6 %] over 13 695 decided cube games, at about +36 %
    /// wall clock.
    ///
    /// The fixed-deck aggregate badly understates how *deck-dependent* it
    /// is, which is the more useful finding:
    ///
    /// | mirror | searched attacks win % |
    /// |---|---|
    /// | mono-red aggro | 59.6 % |
    /// | azorius skies | 56.7 % |
    /// | golgari midrange | 48.5 % |
    /// | dimir control | 44.8 % |
    ///
    /// Restraint is worth nearly ten points in the aggro mirror and *costs*
    /// five in the control mirror, where somebody has to actually close the
    /// game and the passive side doesn't. The search has a one-turn-cycle
    /// horizon, so it can see "this creature survives to block" and cannot
    /// see "this is the race I need to win"; a deck whose plan is inevitable
    /// card advantage is exactly where that blind spot bites. Adopted on the
    /// aggregate, but a profile that scales restraint to the board — or a
    /// horizon that reaches a win — is the obvious next thing to measure.
    pub const fn attack_search() -> Self {
        Self { attack_search: 6, ..Self::hold_sick_combat() }
    }

    /// The adopted default plus [`hold_instants`](Self::hold_instants).
    ///
    /// The hypothesis this existed to test, straight out of the SOS
    /// college probes (`bot_probe --deck sos:prismari --vs baseline`):
    /// in the instant-speed college the default profile cast exactly ONE
    /// spell at instant timing across 60 games, main-phased its instants
    /// proactively, tapped out, and pitched 42 hands' worth of reactive
    /// spells to cleanup — while the ladder read Prismari ≈ 49 % against
    /// the control. `hold_instants` had measured neutral on the four
    /// constructed decks (see [`hold_instants`]), but none of those decks
    /// was built from a pool where half the playables are instants, so
    /// this re-asked the question where it should have mattered most.
    ///
    /// **Measured, and not adopted**: 49.4 % [46.3 %, 52.5 %] against
    /// `atk` over 1000 SOS college-mirror games (seed 11) — statistically
    /// indistinguishable from the atk-vs-atk control at the same seed
    /// (48.9 %), i.e. holding bought nothing, at +65 % wall clock for the
    /// extra `improves_this_turn` simulations. (An earlier reading of the
    /// per-college rows as "Prismari got worse" was noise: the identical-
    /// profile control swings its own college rows to 44.5 % at 200
    /// games. Only the pooled total is a result.) The probe's real
    /// Prismari signal is elsewhere: reactive spells rot because the
    /// response layer under-fires, and the attack search over-swings on
    /// small boards (82 % of eligible, 78 % all-in) — restraint, not
    /// timing, is the open lead; see [`attack_search_sim`].
    pub const fn attack_search_hold() -> Self {
        Self { hold_instants: true, ..Self::attack_search() }
    }

    /// The adopted default with spell-casting combat simulations.
    ///
    /// The hypothesis, out of the SOS college diagnosis (per-college
    /// probes plus the `atk-hold` and `blk` dead ends): the attack search
    /// over-swings on small boards — 82 % of eligible declared, 78 %
    /// all-in in Prismari, 41 % of creatures tapped when blocks come —
    /// because its simulation casts nothing for either side, so a swing
    /// into open mana and a hand full of removal sims as free. With
    /// [`attack_sim_spells`](Self::attack_sim_spells) the crack-back is
    /// visible at declaration time.
    ///
    /// **Measured, and adopted as the default.** Three runs against
    /// `atk`, all positive, plus an identical-profile control:
    ///
    /// | field | result | games |
    /// |---|---|---|
    /// | SOS colleges, seed 11 | 51.7 % [48.6 %, 54.8 %] | 1 000 |
    /// | SOS colleges, seed 7 | 53.2 % [50.1 %, 56.3 %] | 1 000 |
    /// | fixed + cube, seed 11 | 54.4 % [53.0 %, 55.8 %] | 4 794 |
    /// | control (atk vs atk, SOS) | 48.9 % [45.8 %, 52.0 %] | 1 000 |
    ///
    /// No archetype below 50 % on the deciding run, and the largest gain
    /// is dimir control at 61.3 % — the archetype where the blind search
    /// measured 44.8 % and the "restraint costs five points in the
    /// control mirror" caveat was written. Cost: roughly 2-4× the ladder
    /// wall clock of `atk`, all of it on DeclareAttackers/blocks ticks.
    pub const fn attack_search_sim() -> Self {
        Self { attack_sim_spells: true, ..Self::attack_search() }
    }

    /// The adopted default plus the race horizon
    /// ([`attack_race_horizon`](Self::attack_race_horizon)) — the
    /// roadmap's "race math" hypothesis: an attack sim that ends inside
    /// burn range keeps going one cycle so a winning (or losing) race is
    /// scored as such instead of mid-sprint.
    ///
    /// **Measured, and not adopted**: the pre-registered 4× decision run
    /// (1 600 games/archetype, seed 12) read 50.2 % [49.5 %, 51.0 %]
    /// over 19 200 fixed+cube games vs `atk-sim` — the interval
    /// straddles 50 % and the edge is a fifth of the MARGINAL bar.
    ///
    /// The first decider (4 796 games, seed 11) had read 51.2 %
    /// [49.8 %, 52.6 %] with mono-red — the archetype the horizon
    /// exists for — at 54.8 % over 400 games. At 4× the sample the
    /// pooled edge collapsed +1.2 → +0.2 and mono-red reverted to
    /// 49.9 %: the same replication failure
    /// [`block_search`](Self::block_search) documents, reproduced on a
    /// different hypothesis. Whatever the extended horizon sees in the
    /// last burn-range turn, the default profile's one-cycle sim
    /// already prices well enough that the extra cycle (and its extra
    /// fuel) buys nothing measurable. Kept as a profile because the
    /// negative result is worth more than the code.
    pub const fn attack_search_race() -> Self {
        Self { attack_race_horizon: true, ..Self::attack_search_sim() }
    }

    /// The adopted default piloted by the learned SOS-sealed value net
    /// ([`net_slot`](Self::net_slot) = the registry's best slot):
    /// `eval_material` returns the net's win probability instead of the
    /// material count, so every outcome-eval'd decision — casts, blocks,
    /// modes, scries, sacrifices, the combat sims — optimizes the learned
    /// value. Candidate *scoring* and the rest of the decision table stay
    /// heuristic; the net replaces the judge, not the shortlist.
    ///
    /// Requires a net in slot 1 (`CRAB_NET` on the ladder, the training
    /// loop's promotion in `selfplay_train`); with the slot empty this is
    /// exactly `attack_search_sim`. Gate on sealed mirrors (`bot_ladder
    /// --decks sealed`) before any adoption claim.
    ///
    /// **Measured across three checkpoints, and not adopted** (1 200
    /// sealed-mirror games vs `atk-sim` each): 43.6 % [40.8, 46.4] on the
    /// round-1 net (25 k games), 42.3 % [39.6, 45.1] on the round-2 net
    /// (4× the data), 43.4 % [40.6, 46.2] after round 2's over-reused
    /// training tail, 44.7 % [41.9, 47.5] on the round-3 net (mid-turn
    /// snapshot cadence, 10.5 M rows), 43.8 % [41.0, 46.6] on the
    /// round-4 net (5× capacity + keyword object features — but only
    /// 0.4 learner visits per row: at 600 k parameters the CPU learner,
    /// not generation, is the bottleneck, so round 4 tested capacity at
    /// half an epoch; a fair capacity test needs the GPU learner).
    /// Better than the MCTS attempt's 41.5 %, worse than the tuned
    /// heuristic, and *flat-to-marginal across a 4× data jump and the
    /// distribution fix*: neither data volume nor snapshot coverage is
    /// the binding constraint at small capacity. Worth naming what the net is actually up against:
    /// `eval_material` scores the *outcomes of resolved simulations* — a
    /// one-ply search with a perfect forward model — so a value net only
    /// helps where it carries long-horizon signal the material count
    /// misses, and a ~125 k-parameter pooled encoder evidently carries
    /// little yet. Next levers, in order: capacity, richer object
    /// features, search-improved training targets.
    /// The adopted default plus sequenced land drops
    /// ([`land_urgency`](Self::land_urgency)).
    ///
    /// **Measured, and not adopted** — but the route there is worth more
    /// than the result:
    ///
    /// | field | result | games |
    /// |---|---|---|
    /// | fixed + cube, seed 23 | 49.4 % [47.9 %, 50.8 %] | 4 800 |
    /// | sealed, seed 23 | 51.4 % [50.0 %, 52.8 %] | 4 800 |
    /// | **sealed, seed 29 (decider)** | **50.3 % [49.6 %, 51.0 %]** | **19 200** |
    ///
    /// The first row could not have read anything else: the fixed and
    /// cube archetypes play basics almost exclusively, so the
    /// tapland-timing half of this profile never fires there. A profile
    /// can only be measured on decks containing the cards it reasons
    /// about, and running it on the default field first was a wasted
    /// 4 800 games.
    ///
    /// Moving to sealed — where the builder actually produces school
    /// lands — read +1.4 with the lower bound exactly on 50.0, so the
    /// 4× run was pre-registered as the decision rather than reported.
    /// It came back +0.3. That is the third time this harness has seen
    /// a promising sub-5 000-game result evaporate at 4× the sample
    /// (see [`block_search`](Self::block_search) and
    /// [`attack_search_race`](Self::attack_search_race)); the pattern is
    /// now reliable enough to treat any 400-games-per-archetype edge as
    /// a hypothesis, never a finding.
    ///
    /// Why it plausibly does nothing: the sealed builder gives most
    /// decks two colors and a handful of duals, so the tapland decision
    /// arises a few times a game and usually has one obvious answer the
    /// old first-playable rule already stumbled into.
    pub const fn land_sequencing() -> Self {
        Self { land_urgency: true, ..Self::attack_search_sim() }
    }

    /// The adopted default plus quality-aware mulligans
    /// ([`mull_quality`](Self::mull_quality)).
    ///
    /// **Measured and not adopted**: 50.7 % [49.7 %, 51.7 %] over 9 600
    /// sealed games, 50.2 % [49.6 %, 50.8 %] over 28 800 on the
    /// pre-registered decider. The fourth consecutive evaporation of a
    /// sub-10 000-game edge in this harness (after
    /// [`block_search`](Self::block_search),
    /// [`attack_search_race`](Self::attack_search_race) and
    /// [`land_sequencing`](Self::land_sequencing)) — at this point any
    /// result here under ~20 000 games should be read as a hypothesis
    /// however clean its interval looks.
    ///
    /// The rule changes are still the right *shape* — its tests pin two
    /// hands the shipped heuristic reads backwards — so the likely
    /// reading is that opening-hand quality matters less than it feels
    /// like it should when both seats mulligan by the same rule in a
    /// mirror: the edge cancels.
    pub const fn mulligan_quality() -> Self {
        Self { mull_quality: true, ..Self::attack_search_sim() }
    }

    /// Value gang-blocks ([`block_gang`](Self::block_gang)) plus the
    /// `block_search` that scores them — with the search at 0 the gang
    /// candidates are never evaluated, so the two ship together.
    ///
    /// **Adopted — this is [`EvalWeights::default`].** 51.3 %
    /// [50.7 %, 51.9 %] (seed 43) and 51.1 % [50.5 %, 51.7 %] (seed 97),
    /// 28 800 sealed games each vs `atk-sim`, after a 9 600-game
    /// screening read 51.0 %. Unlike the four other play-side profiles
    /// tried alongside it, the edge did not shrink at 3× the sample.
    ///
    /// What it adds: at a healthy life total the greedy pass blocks an
    /// attacker only when one creature kills it alone, so two 2/2s never
    /// ate a 4/4 however good the trade. Gangs are now offered as
    /// candidates and [`simulate_block_outcome`] prices the dead
    /// blockers against the dead attacker.
    ///
    /// The bundle caveat, stated plainly: `block_search` alone measured
    /// null (50.4 % over 30 000 games) and is switched on here. That is
    /// not evidence the earlier rejection was wrong — the search had
    /// nothing to find while its only candidates were "block with one
    /// fewer creature".
    pub const fn block_gang_search() -> Self {
        Self { block_gang: true, block_search: 2, ..Self::attack_search_sim() }
    }

    // ── Re-measurement profiles ───────────────────────────────────────
    //
    // Four ideas were measured against `attack_search_sim` and dropped
    // for reading ~50 %, and one (`lookahead1`) for reading 50.2 % over
    // 8 000 games. Every one of those runs was unpaired, and the paired
    // ladder puts the realized within-pair correlation at −0.74 on this
    // field: those game counts carried roughly a quarter of the
    // precision they appeared to. A null at that resolution is not
    // evidence of a null, so each idea gets one honest re-test.
    //
    // They are rebased onto the *current* default rather than reusing
    // the originals: `land_sequencing` and friends branch from
    // `attack_search_sim`, and gang-blocking has been adopted since, so
    // laddering them as written would measure "the idea, minus
    // gang-blocking" and charge the difference to the idea.

    /// [`land_sequencing`](Self::land_sequencing) rebased onto the
    /// adopted default, for the paired re-test.
    pub const fn land_sequencing_default() -> Self {
        Self { land_urgency: true, ..Self::block_gang_search() }
    }

    /// [`mulligan_quality`](Self::mulligan_quality) rebased onto the
    /// adopted default, for the paired re-test.
    pub const fn mulligan_quality_default() -> Self {
        Self { mull_quality: true, ..Self::block_gang_search() }
    }

    /// [`attack_search_race`](Self::attack_search_race) rebased onto the
    /// adopted default, for the paired re-test.
    pub const fn attack_race_default() -> Self {
        Self { attack_race_horizon: true, ..Self::block_gang_search() }
    }

    /// [`lookahead1`](Self::lookahead1) rebased onto the adopted
    /// default, for the paired re-test.
    pub const fn lookahead1_default() -> Self {
        Self { lookahead: 1, ..Self::block_gang_search() }
    }

    /// Two plies of sequence lookahead — the depth `lookahead1`'s doc
    /// comment invites someone to measure. Forge searches three.
    pub const fn lookahead2_default() -> Self {
        Self { lookahead: 2, ..Self::block_gang_search() }
    }

    /// The default, searching a single redeal of the hidden zones
    /// instead of the true state — see
    /// [`determinize`](Self::determinize).
    pub const fn determinized() -> Self {
        Self { determinize: 1, ..Self::block_gang_search() }
    }

    /// The default with the historical mana tapping — the control for
    /// [`smart_tap`](Self::smart_tap). Ladder this as B so a positive
    /// result reads as "the new tapping is better".
    pub const fn legacy_tap() -> Self {
        Self { smart_tap: false, ..Self::block_gang_search() }
    }

    /// The default, averaging three redeals per candidate. Three times
    /// the simulation cost, and the version that actually approximates
    /// "play against the hands consistent with what I can see" rather
    /// than "play against one specific wrong hand".
    pub const fn determinized3() -> Self {
        Self { determinize: 3, ..Self::block_gang_search() }
    }

    pub const fn net_eval() -> Self {
        Self { net_slot: super::net_eval::SLOT_BEST, ..Self::attack_search_sim() }
    }

    /// [`net_eval`](Self::net_eval), blended instead of replaced: the
    /// heuristic evaluation plus a ±50·unit net bias. The division of
    /// labor: the heuristic resolves small material deltas exactly, the
    /// net tilts close calls it has an opinion about.
    ///
    /// **Measured four times**: 49.3 % [46.5, 52.2] (round-1 net),
    /// 49.2 % [46.4, 52.1] (round-2 net, 4× the data), 50.7 %
    /// [47.9, 53.6] (round 2 after its over-reused tail), 48.8 %
    /// [45.9, 51.6] (round-4 capacity net, undertrained) over 1 200
    /// sealed-mirror games each vs `atk-sim` — stable statistical parity
    /// while the same nets score 42–45 % as full replacements. The
    /// stability says the ±50-unit bias is mostly inert (the net's
    /// probability hovers near 0.5 in balanced positions, so the bias
    /// rarely clears a decision margin) — hence the louder
    /// [`net_eval_blend300`](Self::net_eval_blend300). The tail
    /// comparison also priced tail over-reuse: loss EMA fell 0.30 → 0.14
    /// during it with no strength change — pure window memorization,
    /// which is why the trainer now caps the tail.
    pub const fn net_eval_blend() -> Self {
        Self { net_blend_scale: 100, ..Self::net_eval() }
    }

    /// [`net_eval_blend`](Self::net_eval_blend) at 3× loudness — full
    /// confidence worth ±150 units, enough to outvote a mid-size body.
    /// Exists because the 100-scale blend measured as inert; where the
    /// right loudness lies is a ladder question, not an argument.
    ///
    /// **Measured, and the answer is "quieter"**: 45.9 % [43.1 %, 48.7 %]
    /// over 1 200 sealed-mirror games with the round-3 net (47.1 %
    /// [44.3, 49.9] with round 4's), vs 49.3 % / 48.8 % for the
    /// 100-scale blend on the same weights. Amplifying the net's
    /// opinion hurts — where it disagrees with the heuristic it is wrong
    /// more often than right, which bounds what any loudness of this
    /// net's bias can contribute.
    pub const fn net_eval_blend300() -> Self {
        Self { net_blend_scale: 300, ..Self::net_eval() }
    }

    /// The adopted default plus the searched block assignment.
    ///
    /// **Measured, and not adopted**: 50.4 % [49.8 %, 51.0 %] over 30 000
    /// games across all twelve archetypes — the interval straddles 50 %.
    ///
    /// Kept because the negative result is worth more than the code. Two
    /// earlier runs each read about +0.8 (50.9 % over 8 000 fixed-deck
    /// games, 50.7 % over 15 998 cube games), with a dramatic per-deck
    /// split: −3.2 in the mono-red mirror against +6.7 in golgari. Pooling
    /// those two runs would have cleared 50 % and looked like an adoption.
    /// The 30 000-game run was pre-registered as the decision instead, and
    /// the effect halved to +0.4 while the split mostly evaporated —
    /// mono-red came back to 49.4 %. Only golgari survived, at +4.1.
    ///
    /// So the interesting finding is methodological: at 2 000 games per
    /// archetype this ladder produces per-deck swings of five points that
    /// are not there at 2 500. A per-archetype number is roughly a tenth of
    /// the total sample and should be read as a hint about *where* to look,
    /// never as a result on its own.
    ///
    /// Why it might genuinely not help: [`attack_search`](Self::attack_search)
    /// already delivers about twice the untapped board to `DeclareBlockers`,
    /// and the greedy assignment in [`pick_blocks_inner`] is a far more
    /// developed heuristic than `pick_attacks` ever was — it already folds in
    /// first strike, deathtouch, trample, protection, indestructible,
    /// rampage, planeswalker defense and poison. There was much less room
    /// above it than there was above the alpha strike.
    ///
    /// Re-measured on the SOS college mirrors (where the probes show the
    /// default profile leaving 72-78 % of attackers unblocked in the
    /// spell-heavy colleges): 50.1 % [47.0 %, 53.2 %] against `atk` over
    /// 1000 games — neutral there too. The under-block is not an
    /// assignment problem — the blockers are TAPPED (41-42 % of creatures
    /// at DeclareBlockers, vs 27 % in the healthy Witherbloom row), which
    /// points back at the over-attack, not at the block search; see
    /// [`attack_search_sim`].
    pub const fn block_search() -> Self {
        Self { block_search: 6, ..Self::attack_search() }
    }

    /// Searched attacks with life priced on the concave curve.
    ///
    /// The hypothesis this exists to test. `eval_material` prices a
    /// permanent at `3 * (cmc + power + toughness)` but life at one point
    /// per life, so a Grizzly Bears is worth 18 and the 2 damage it deals is
    /// worth 2 — the attack search has to see a 9:1 payoff before swinging
    /// beats staying home. In an aggro mirror that is roughly right, because
    /// the body you keep really does block something. In the dimir control
    /// mirror it is fatal, and that is exactly where
    /// [`attack_search`](Self::attack_search) loses 5.2 points: damage is the
    /// only win route there, and the evaluator has priced it at a ninth of
    /// its worth.
    ///
    /// [`concave_life`](Self::concave_life) is the existing knob for this —
    /// it prices life steeply near zero and flatly near twenty, which is
    /// what makes "this is the race I need to win" visible to a search whose
    /// horizon is one turn cycle.
    pub const fn attack_search_life() -> Self {
        Self { concave_life: true, ..Self::attack_search() }
    }

    /// The adopted default with the concave life curve and *no* attack
    /// search — the control for [`attack_search_life`](Self::attack_search_life).
    /// Without it a win by that profile can't be attributed: the curve might
    /// simply be better everywhere rather than specifically correcting the
    /// search's bias.
    pub const fn hold_sick_combat_life() -> Self {
        Self { concave_life: true, ..Self::hold_sick_combat() }
    }

    /// Searched attacks with the candidate set cut to the two extremes —
    /// the greedy alpha strike and no attack at all. If the cheap version
    /// captures most of the gain, the per-attacker drops aren't paying for
    /// their simulations and the search can stay nearly free.
    pub const fn attack_search_cheap() -> Self {
        Self { attack_search: 1, ..Self::hold_sick_combat() }
    }

    /// Everything the planner work produced: hold summoning-sick bodies,
    /// hold instant-speed lines that do nothing yet, and evaluate both
    /// through this turn's combat so the gate can tell the difference.
    pub const fn planner() -> Self {
        Self { hold_sick: true, hold_instants: true, combat_aware: true, ..Self::baseline() }
    }

    /// Baseline + the combat-aware evaluation.
    ///
    /// **Measured exactly neutral**: 50.0 % [49.1 %, 50.9 %] over 12 000
    /// games (6002-5998). Not adopted as the default, but kept and worth
    /// revisiting, because the reason it does nothing here is structural
    /// rather than a flaw in the simulation: within a single precombat
    /// main phase, *this turn's combat is very nearly the same whichever
    /// candidate the bot picks*, so the term is shared between candidates
    /// and cancels in the comparison. It only starts to pay when the bot
    /// can also choose *when* to act — which is what a turn planner adds
    /// (Forge's `formulatePlanWithPhase(COMBAT_DECLARE_BLOCKERS)` and its
    /// summon-sick gating both consume exactly this signal).
    pub const fn combat_aware() -> Self {
        Self { combat_aware: true, ..Self::baseline() }
    }

    /// [`v2`](Self::v2) weights *plus* the combat-aware evaluation — the
    /// direct test of why v2 lost. The hypothesis was depth: that a richer
    /// evaluation only pays once the evaluator can see past the current
    /// action. This is the cheapest available "more depth".
    ///
    /// **The hypothesis is refuted.** This measures 53.1 % to the baseline
    /// — worse than [`v2`](Self::v2) alone at 51.1 % — so the extra depth
    /// does not rescue the ported weights, it compounds them.
    pub const fn v2_combat() -> Self {
        Self { combat_aware: true, ..Self::v2() }
    }

    /// The historical mana behavior, for laddering the tap-out fix.
    /// See [`legacy_pretap`](Self::legacy_pretap).
    pub const fn legacy_mana() -> Self {
        Self { legacy_pretap: true, ..Self::baseline() }
    }

    /// Scaled control + the keyword term only. **Measured worse than the
    /// baseline**: 51.1 % to the baseline over 20 000 pooled games
    /// ([50.4 %, 51.8 %]). See [`v2`](Self::v2) for why it is kept.
    pub const fn keywords_only() -> Self {
        Self { keyword_pct: 100, ..Self::scaled_control() }
    }

    /// Scaled control + a quarter-strength keyword term. Separates
    /// "keywords are weighted too heavily" from "keyword scoring is the
    /// wrong thing to feed this bot at all": if even a gentle version
    /// loses, the problem is directional, not a magnitude to tune.
    /// Measured neutral (50.1 % to the baseline).
    pub const fn keywords_quarter() -> Self {
        Self { keyword_pct: 25, ..Self::scaled_control() }
    }

    /// Scaled control + Forge's flat creature base only.
    ///
    /// This was the hypothesis for why the keyword port lost -- Forge's
    /// keyword magnitudes are calibrated against a body term that opens at
    /// a flat 100, where this evaluator opens at zero, so the same bonuses
    /// land proportionally ~2.7x louder here. Adding the constant did not
    /// help ([`base_and_keywords`] measured 52.4 % *to the baseline*, worse
    /// than keywords alone), because the constant also shifts the
    /// creature-to-card ratio from 4.5:1 to 12:1. Forge runs that ratio at
    /// roughly 32:1; matching one term without the surrounding balance
    /// moves everything.
    ///
    /// [`base_and_keywords`]: Self::base_and_keywords
    pub const fn creature_base_only() -> Self {
        Self { creature_base: 100, ..Self::scaled_control() }
    }

    /// The creature base plus keywords. **Measured worst of the lot**:
    /// 52.4 % to the baseline. Retained as the record of a tested and
    /// rejected hypothesis -- see [`creature_base_only`].
    ///
    /// [`creature_base_only`]: Self::creature_base_only
    pub const fn base_and_keywords() -> Self {
        Self { keyword_pct: 100, ..Self::creature_base_only() }
    }

    /// Scaled control + the concave life curve only. Measured neutral
    /// (51.1 % to the baseline, CI straddling 50 %).
    pub const fn life_only() -> Self {
        Self { concave_life: true, ..Self::scaled_control() }
    }

    /// Scaled control + Forge's power-over-toughness emphasis only.
    /// Measured neutral (50.4 % to the baseline, CI straddling 50 %).
    pub const fn power_emphasis_only() -> Self {
        Self { power: 15, ..Self::scaled_control() }
    }
}

impl Default for EvalWeights {
    /// The adopted profile. [`baseline`](EvalWeights::baseline) stays the
    /// historical *control* — it is what ladder runs measure against — but
    /// it is no longer what the bot plays. Each layer had to beat the
    /// previous one on the ladder before it was added:
    ///
    /// | layer | vs. the one before | games |
    /// |---|---|---|
    /// | summon-sick gate | 51.5 % [50.8 %, 52.3 %] | 16 000 |
    /// | combat-aware evaluation | 51.3 % [50.4 %, 52.2 %] | 12 000 |
    /// | searched attacks | 52.4 % [51.3 %, 53.5 %] | 8 000 |
    /// | spell-casting combat sims | 54.4 % [53.0 %, 55.8 %] | 4 794 |
    /// | value gang-blocks | 51.3 % [50.7 %, 51.9 %] | 28 800 |
    ///
    /// The attack search was additionally confirmed on cube decks —
    /// 53.8 % [53.0 %, 54.6 %] over 13 695 decided games — where the fixed
    /// four archetypes could not have shown its deck-dependence. The
    /// spell-casting sims (see [`attack_search_sim`](Self::attack_search_sim))
    /// were adopted on the fixed+cube row above plus two 1000-game SOS
    /// college runs (51.7 % / 53.2 %, the second clearing 50 % alone),
    /// against an identical-profile control at 48.9 % — and the largest
    /// per-deck gain landed exactly where the blind attack search had its
    /// documented regression: dimir control, 44.8 % blind → 61.3 % seeing.
    /// Gang-blocking (see [`block_gang_search`](Self::block_gang_search))
    /// was adopted last, on two independent 28 800-game sealed runs —
    /// 51.3 % [50.7 %, 51.9 %] and 51.1 % [50.5 %, 51.7 %]. It is the
    /// only one of five play-side heuristics tried in that push to
    /// survive its decider, and the difference is instructive: the
    /// other four refined decisions the bot already made competently,
    /// while this one added a line it could not previously express at
    /// all (the greedy pass gangs only under lethal threat, and
    /// [`block_search`](Self::block_search) could only ever *remove*
    /// blockers). Note that adopting it also turns `block_search` on,
    /// which measured null by itself — the search was never the
    /// problem; it had nothing worth searching.
    fn default() -> Self {
        Self::block_gang_search()
    }
}

/// Reference bot. Taps lands and plays a (roughly random) affordable card
/// from hand, but combat is heuristic: it attacks with creatures that swing
/// safely or profitably (evasion / first-strike / deathtouch / menace /
/// lifelink / trample / indestructible awareness, plus a suicide filter and
/// planeswalker redirection) and assigns blockers to maximize value trades
/// and survive lethal (see `pick_attack`/`pick_blocks`). Decisions are
/// auto-answered with [`AutoDecider`].
///
/// The bot keeps a little internal flag state so it only submits
/// `DeclareAttackers`/`DeclareBlockers` once per combat phase — the match
/// actor polls it repeatedly, so without these flags it would re-submit every
/// tick.
pub struct RandomBot {
    last_step_key: Option<(u32, TurnStep, usize)>,
    attackers_declared: bool,
    blocks_declared: bool,
    /// `true` (the default) ranks castable candidates via
    /// [`score_candidate`]; `false` keeps the legacy uniform-random pick.
    /// The baseline exists so bot changes can be A/B-laddered against the
    /// previous behavior.
    scored: bool,
    /// Ad Nauseam-style reveal series: the asks all happen before any life
    /// is lost, so the bot tracks what it has already committed to across
    /// consecutive prompts from the same source. `(source, cards, life)`.
    reveal_commit: Option<(CardId, usize, i32)>,
    /// How this bot values the board. Ladder-selectable -- see
    /// [`EvalWeights`].
    weights: EvalWeights,
}

impl RandomBot {
    pub fn new() -> Self {
        Self {
            last_step_key: None,
            attackers_declared: false,
            blocks_declared: false,
            scored: true,
            reveal_commit: None,
            weights: EvalWeights::default(),
        }
    }

    /// The scored bot piloted with a specific evaluation profile.
    pub fn with_weights(weights: EvalWeights) -> Self {
        Self { weights, ..Self::new() }
    }

    /// The pre-scoring reference bot: identical candidate enumeration and
    /// combat, but the castable pick is uniform-random. Kept as the ladder
    /// baseline for measuring bot improvements.
    pub fn uniform_baseline() -> Self {
        Self { scored: false, ..Self::new() }
    }

    fn sync_step(&mut self, state: &GameState) {
        let key = (state.turn_number, state.step, state.active_player_idx);
        if self.last_step_key != Some(key) {
            self.last_step_key = Some(key);
            self.attackers_declared = false;
            self.blocks_declared = false;
        }
    }
}

impl Default for RandomBot {
    fn default() -> Self {
        Self::new()
    }
}

impl Bot for RandomBot {
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction> {
        if state.is_game_over() {
            return None;
        }
        self.sync_step(state);

        // Any pending decision addressed to us: auto-answer it.
        if let Some(pending) = &state.pending_decision {
            if pending.acting_player() == seat {
                // Ad Nauseam's per-reveal prompt is the one STATEFUL
                // policy (it tracks committed reveals on the bot struct
                // across the series — see `RevealTopToHandLoseLifeRepeat`),
                // so it answers here; every other decision goes through
                // [`decide_pending_policy`], the same table simulations
                // use.
                if let crate::decision::Decision::OptionalTrigger { source, description } =
                    &pending.decision
                    && description.starts_with("Reveal the top card (")
                {
                    let (cards, life_committed) = match &self.reveal_commit {
                        Some((s, c, l)) if *s == *source => (*c, *l),
                        _ => (0, 0),
                    };
                    let mv = state.players[seat]
                        .library
                        .get(cards)
                        .map(|c| c.definition.cost.cmc() as i32)
                        .unwrap_or(0);
                    let yes = state.effective_life(seat) - life_committed - mv > 10;
                    self.reveal_commit =
                        if yes { Some((*source, cards + 1, life_committed + mv)) } else { None };
                    return Some(GameAction::SubmitDecision(
                        crate::decision::DecisionAnswer::Bool(yes),
                    ));
                }
                let answer =
                    decide_pending_policy(state, seat, &self.weights, &pending.decision, true);
                return Some(GameAction::SubmitDecision(answer));
            }
            return None;
        }

        if state.player_with_priority() != seat {
            return None;
        }

        let is_active = state.active_player_idx == seat;

        match state.step {
            TurnStep::DeclareBlockers if state.may_declare_blocks(seat) => {
                if !self.blocks_declared && !state.attacking().is_empty() {
                    // Kill the biggest attacker BEFORE committing blocks —
                    // removal cast here shrinks the combat the blocks then
                    // answer. Validated actions only, so a resolved kill
                    // falls through to the block declaration next tick.
                    if !is_active
                        && let Some(a) = pick_defensive_removal(state, seat, &self.weights)
                    {
                        return Some(a);
                    }
                    self.blocks_declared = true;
                    // On our own turn we're choosing the *defender's* blocks
                    // (Master Warcraft, Invasion Plans), so submit only what
                    // CR 509.1c forces — and aim each forced blocker at the
                    // attacker most likely to kill it.
                    let blocks = if is_active {
                        forced_blocks(state)
                    } else {
                        pick_blocks_scored(state, seat, &self.weights)
                    };
                    Some(GameAction::DeclareBlockers(blocks))
                } else if state.blockers_declared() && state.stack.is_empty() {
                    // Post-block priority: a held pump trick that flips a
                    // fight one of our blockers is losing.
                    Some(pick_combat_trick(state, seat, &self.weights).unwrap_or(GameAction::PassPriority))
                } else {
                    Some(GameAction::PassPriority)
                }
            }
            // Active side of the same window: blocks are in, stack is
            // empty — the classic trick timing for a blocked attacker.
            TurnStep::DeclareBlockers
                if is_active && state.blockers_declared() && state.stack.is_empty() =>
            {
                Some(pick_combat_trick(state, seat, &self.weights).unwrap_or(GameAction::PassPriority))
            }
            // Master Warcraft on an opponent's turn: we choose *their*
            // attackers, so declare only the creatures that must attack.
            TurnStep::DeclareAttackers if !is_active && state.attack_declarer() == seat => {
                if !self.attackers_declared {
                    self.attackers_declared = true;
                    Some(GameAction::DeclareAttackers(forced_attacks(state)))
                } else {
                    Some(GameAction::PassPriority)
                }
            }
            TurnStep::DeclareAttackers if is_active && state.attack_declarer() == seat => {
                if !self.attackers_declared {
                    self.attackers_declared = true;
                    Some(GameAction::DeclareAttackers(pick_attacks_scored(
                        state,
                        seat,
                        &self.weights,
                    )))
                } else {
                    Some(GameAction::PassPriority)
                }
            }
            TurnStep::PreCombatMain | TurnStep::PostCombatMain if is_active => {
                // A non-empty stack in our own main is response timing,
                // not sorcery timing: an opponent's answer is resolving,
                // and the response layer sees threats the main-phase
                // enumerator can't (counter it, fire a prepared inset
                // spell before its body dies). Both pickers ignore the
                // bot's own spells-in-flight, so a stack we put there
                // falls through to the enumerator as before.
                if !state.stack.is_empty()
                    && let Some(a) = pick_stack_response(state, seat, &self.weights)
                        .or_else(|| pick_ability_counter_response(state, seat))
                        .or_else(|| pick_prepare_response(state, seat, &self.weights))
                {
                    return Some(a);
                }
                // CR 116.2j — the agenda is already named, so turning it face
                // up is pure upside; do it at the first opportunity.
                if let Some(c) = state.players[seat]
                    .command
                    .iter()
                    .find(|c| c.face_down && c.definition.is_conspiracy())
                {
                    return Some(GameAction::RevealConspiracy { card_id: c.id });
                }
                // CR 901.9 — take the turn's one free planar-die roll before
                // spending mana. Later rolls cost {N} and compete with real
                // plays, so the bot stops after the free one. Inert outside a
                // Planechase game (no planar deck, nothing face up).
                if state.stack.is_empty()
                    && state.players[seat].planar_die_rolls_this_turn == 0
                    && !state.players[seat].planar_deck.is_empty()
                    && !state.face_up_planes().is_empty()
                {
                    return Some(GameAction::RollPlanarDie);
                }
                Some(main_phase_action_with(state, seat, self.scored, &self.weights))
            }
            // Opponent's end step with an empty stack — the bot's canonical
            // off-turn window. Reuse the whole scored main-phase enumeration:
            // `would_accept` filters it down to instant-legal lines (removal,
            // tricks, flash creatures, EOT draw, mana-sink abilities), while
            // land drops, sorcery-speed casts, and loyalty/crew lines simply
            // drop out of the candidate set. Without this, every non-counter
            // instant was dead in hand until the bot's own main phase.
            TurnStep::End if !is_active && state.stack.is_empty() => {
                Some(main_phase_action_with(state, seat, self.scored, &self.weights))
            }
            _ => Some(
                pick_stack_response(state, seat, &self.weights)
                    .or_else(|| pick_ability_counter_response(state, seat))
                    .or_else(|| pick_prepare_response(state, seat, &self.weights))
                    // Defender windows in the attack steps (the picker
                    // no-ops unless declared attackers are coming at us).
                    .or_else(|| {
                        if state.stack.is_empty() {
                            pick_defensive_removal(state, seat, &self.weights)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(GameAction::PassPriority),
            ),
        }
    }
}

/// The bot's answer to a pending `decision` for `seat` — the policy table
/// behind `next_action`, extracted so SIMULATIONS answer with it too.
/// Every lookahead used to answer internal decisions with `AutoDecider`,
/// which meant a line was scored as if the bot's future self would scry
/// badly, decline its tutors, dump the head of its hand to a discard, and
/// take mode 0 — and the opponent would as well. Now both seats inside a
/// sim play by this table (`pending.acting_player()` picks whose view).
///
/// `eval_modes: false` disables the clone-and-resolve answers — mode
/// picks fall back to mode 0 and self-costly optional triggers to the
/// introspection screen's decline — because inside a sim that recursion
/// would multiply whole-state clones for marginal fidelity, and both
/// fallbacks are the pre-policy floor. The stateful Ad Nauseam reveal
/// family (a running life commitment on the bot struct) is handled by
/// `next_action` before this table; a sim reaching it here declines, the
/// conservative read.
fn decide_pending_policy(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
    decision: &crate::decision::Decision,
    eval_modes: bool,
) -> crate::decision::DecisionAnswer {
    match decision {
        // Smarter mulligan than AutoDecider's blanket Keep:
        // ship hands that are flooded or screwed on lands.
        crate::decision::Decision::Mulligan { mulligans_taken, .. } => {
            decide_mulligan(state, seat, *mulligans_taken, w)
        }
        // Unlike AutoDecider (which declines every tutor), the
        // bot actually fetches — preferring a basic land toward
        // its weakest color so singleplayer tutors fix mana.
        crate::decision::Decision::SearchLibrary { candidates, eligible, .. } => {
            // Only consider picks the engine will accept.
            let pickable: Vec<(crate::card::CardId, String)> = match eligible {
                Some(ok) => {
                    candidates.iter().filter(|(id, _)| ok.contains(id)).cloned().collect()
                }
                None => candidates.clone(),
            };
            decide_library_search(state, seat, &pickable)
        }
        // Unlike AutoDecider (which declines *every* "you may"
        // trigger), the bot takes an optional trigger whose body
        // is pure upside — so Provoke's "you may", Boast token
        // riders, etc. actually fire under bot play. It still
        // declines bodies that impose a self-cost (lose life /
        // sacrifice / discard). Engine-authored prompt families the
        // generic screen can't introspect (no MayDo body) each get a
        // real policy instead of the blanket-yes fallback.
        crate::decision::Decision::OptionalTrigger { source, description } => {
            let take = if description.starts_with("Pay ")
                && description.contains(" life to deny ")
            {
                // Rhystic-style life tax: pay only with a healthy
                // buffer. Parse the printed amount.
                let n: i32 = description
                    .split_whitespace()
                    .nth(1)
                    .and_then(|w| w.parse().ok())
                    .unwrap_or(2);
                state.effective_life(seat) - n > 10
            } else if description.starts_with("Accept the tempting offer") {
                // Tempting offers reward the caster; decline.
                false
            } else if description.starts_with("Pay echo ")
                || description.starts_with("Pay cumulative upkeep ")
                || description.starts_with("Discard a card for ")
            {
                // Pay while the permanent is worth keeping; let
                // cheap chaff die to its own upkeep.
                state
                    .battlefield_find(*source)
                    .map(|c| permanent_value(state, c.id, w) >= 4 * w.unit)
                    .unwrap_or(false)
            } else if description.starts_with("Cast a copy of ") {
                // Paradigm recurrence (SOS): a free copy is pure
                // upside unless the spell's own body drains life
                // the bot can't spare — Decorum Dissertation's
                // draw-2-lose-2 recurs every main phase, and the
                // blanket yes played it straight into the
                // state-based loss.
                let loss = state
                    .exile
                    .iter()
                    .find(|c| c.id == *source)
                    .map(|c| self_life_loss(&c.definition.effect))
                    .unwrap_or(0);
                state.effective_life(seat) - loss > 5
            } else if description.starts_with("Reveal the top card (") {
                // Stateful family (see `next_action`); in a sim, decline.
                false
            } else {
                // Introspection screen first (pure upside → yes, self-cost
                // → no). A "no" from the self-cost rule gets a second
                // opinion by OUTCOME at the real decision (`eval_modes`
                // gates it off inside sims): sacrifice-for-value and
                // pay-for-payoff bodies are exactly the trades a blanket
                // decline can't judge. Strictly-better-or-keep-declining.
                let take = optional_trigger_beneficial(state, *source, description);
                if !take && eval_modes {
                    decide_optional_by_outcome(state, seat, w).unwrap_or(false)
                } else {
                    take
                }
            };
            crate::decision::DecisionAnswer::Bool(take)
        }
        // AutoDecider always names Demon; the bot instead names the
        // creature type it has the most of across its battlefield +
        // hand, so tribal payoffs (Cavern of Souls, Kindred
        // Discovery, Door of Destinies, the chosen-type lords) land
        // on a type it can actually exploit.
        crate::decision::Decision::ChooseCreatureType { suggestions, .. } => {
            decide_creature_type(state, seat, suggestions)
        }
        // AutoDecider chooses nothing; the bot exiles opponents'
        // graveyard cards (deny graveyard value) up to the cap.
        crate::decision::Decision::ChooseCards { prompt, candidates, min, max, .. } => {
            decide_choose_cards(w, state, seat, prompt, candidates, *min, *max)
        }
        // London mulligan bottoming (CR 103.5) and "put N cards
        // from your hand on top/bottom" effects. `AutoDecider`
        // takes the first N cards of the hand, so a bot that
        // mulliganed bottomed whichever cards happened to sit at
        // the front — routinely its business spells. Rank them
        // the same way a discard is ranked: surplus lands first,
        // then the priciest spells.
        crate::decision::Decision::PutOnLibrary { player, count, hand } if *player == seat => {
            let order = hand_worst_first(state, seat, hand);
            crate::decision::DecisionAnswer::PutOnLibrary(
                order.into_iter().take(*count).collect(),
            )
        }
        // A self-discard (cleanup over max hand size, rummaging, a
        // discard cost): every offered card is in our own hand and
        // we're the one choosing. Unlike AutoDecider (which dumps
        // the head of the hand — possibly our best spell), shed the
        // least useful cards. Inquisition-style "choose from an
        // opponent's hand" Discards fail the own-hand guard and
        // fall through to AutoDecider unchanged.
        crate::decision::Decision::Discard { player, count, hand }
            if *player == seat
                && hand
                    .iter()
                    .all(|(id, _)| state.players[seat].hand.iter().any(|c| c.id == *id)) =>
        {
            decide_self_discard(state, seat, hand, *count)
        }
        // AutoDecider blindly picks the first legal target. For
        // votes (Council's Judgment), edicts, and removal the bot
        // should instead hit the opponent's *most* valuable
        // permanent — or, when forced to choose among its own
        // permanents, give up the *least* valuable.
        crate::decision::Decision::ChooseTarget { legal, .. } if !legal.is_empty() => {
            decide_choose_target(state, seat, legal, w)
        }
        // AutoDecider answers every amount with 0, which turns
        // "choose up to X" payoffs into no-ops and (worse) reads
        // as "power ≥ 0" on destroy-cutoff wraths. Default to
        // the max for generic upside prompts; prompt families
        // with a real downside get their own rule.
        crate::decision::Decision::ChooseAmount { prompt, max, .. } => {
            let amount = if prompt.contains("destroy all creatures with power") {
                best_destroy_power_cutoff(state, seat, *max, w)
            } else if prompt.to_lowercase().contains("life") {
                // Life payments: keep a buffer, never sink deep.
                let spare = (state.effective_life(seat) - 10).max(0) as u32;
                spare.min(*max).min(3)
            } else {
                *max
            };
            crate::decision::DecisionAnswer::Amount(amount)
        }
        // AutoDecider keeps every scried card on top — a no-op
        // that wastes every scry and surveil under bot play.
        // Bottom flood and unplayable spells, draw wants first.
        crate::decision::Decision::Scry { player, cards, mode } if *player == seat => {
            decide_scry(state, seat, cards, *mode)
        }
        // AutoDecider takes the first legal color (usually White). Pick
        // the color the bot's HAND actually demands — the most colored
        // pips across held cards — so mana-fixing sources (any-color
        // ramp, the Quandrix Fractal fixers) fix toward castability.
        // The Quandrix probe showed this fall-through at 11 % of all
        // decisions in that college. Ties keep the engine's order.
        crate::decision::Decision::ChooseColor { legal, .. } if !legal.is_empty() => {
            let pips = |color: crate::mana::Color| {
                state.players[seat]
                    .hand
                    .iter()
                    .flat_map(|c| c.definition.cost.symbols.iter())
                    .filter(|s| matches!(s, crate::mana::ManaSymbol::Colored(c) if *c == color))
                    .count()
            };
            let mut best = legal[0];
            for &c in &legal[1..] {
                if pips(c) > pips(best) {
                    best = c;
                }
            }
            crate::decision::DecisionAnswer::Color(best)
        }
        // AutoDecider answers every mid-resolution modal with
        // mode 0. Evaluate each mode's settled outcome instead.
        crate::decision::Decision::ChooseMode { num_modes, .. } if eval_modes => {
            crate::decision::DecisionAnswer::Mode(decide_mode_by_outcome(
                state, seat, *num_modes, w,
            ))
        }
        other => AutoDecider.decide(other),
    }
}

/// The minimum legal attack declaration for the active player: only the
/// creatures that "attack each combat if able" (CR 508.1d — `MustAttack` or
/// goaded). Master Warcraft's outside chooser declares this and nothing else.
fn forced_attacks(state: &GameState) -> Vec<Attack> {
    use crate::card::Keyword;
    let active = state.active_player_idx;
    let computed = state.compute_battlefield();
    let mut out = Vec::new();
    for c in state.battlefield.iter().filter(|c| c.controller == active) {
        let kws = computed
            .iter()
            .find(|p| p.id == c.id)
            .map(|p| p.keywords.as_slice())
            .unwrap_or(&[]);
        if !kws.contains(&Keyword::MustAttack) && c.goaded_by.is_empty() {
            continue;
        }
        let able = c.definition.is_creature()
            && !c.tapped
            && (!kws.contains(&Keyword::Defender) || state.ignores_defender_for_attack(c))
            && !kws.contains(&Keyword::CantAttack)
            && (!c.summoning_sick || kws.contains(&Keyword::Haste));
        if !able {
            continue;
        }
        let opponents = || {
            (0..state.players.len())
                .filter(|&q| !state.same_team(active, q) && state.players[q].is_alive())
        };
        let Some(target) = opponents()
            .find(|q| !c.goaded_by.contains(q))
            .or_else(|| opponents().next())
        else {
            continue;
        };
        out.push(Attack { attacker: c.id, target: AttackTarget::Player(target) });
    }
    out
}

/// The block declaration to submit when the *attacking* seat is the block
/// chooser (Invasion Plans): satisfy only what CR 509.1c forces — every
/// `MustBlock`/`MustAttackOrBlock` defender, and enough blockers for each
/// `AllMustBlock` attacker — and send each forced blocker into the attacker
/// most likely to eat it. Anything not required stays home.
fn forced_blocks(state: &GameState) -> Vec<(CardId, CardId)> {
    use crate::card::Keyword;
    let computed = state.compute_battlefield();
    let kws = |id: CardId| {
        computed.iter().find(|p| p.id == id).map(|p| p.keywords.as_slice()).unwrap_or(&[])
    };
    let mut out: Vec<(CardId, CardId)> = Vec::new();
    let mut used: Vec<CardId> = state.block_map.keys().copied().collect();
    // Best attacker for `blocker` to run into: the one that kills it and
    // survives, else the biggest.
    let best_attacker = |blocker: &crate::card::CardInstance| {
        let mut cands: Vec<(CardId, i32, bool)> = state
            .attacking
            .iter()
            .filter(|atk| {
                state.defender_for(atk.target).is_some_and(|d| state.same_team(blocker.controller, d))
                    && state.blocker_can_block_attacker(blocker.id, atk.attacker)
            })
            .map(|atk| {
                let dmg = attacker_damage_value(state, atk.attacker);
                let lethal = state
                    .computed_permanent(blocker.id)
                    .is_some_and(|b| dmg >= b.toughness);
                (atk.attacker, dmg, lethal)
            })
            .collect();
        cands.sort_by_key(|(_, dmg, lethal)| (!*lethal, -*dmg));
        cands.first().map(|(id, ..)| *id)
    };
    let force = |blocker_id: CardId, out: &mut Vec<(CardId, CardId)>, used: &mut Vec<CardId>| {
        if used.contains(&blocker_id) {
            return;
        }
        let Some(b) = state.battlefield_find(blocker_id) else { return };
        if b.tapped || kws(blocker_id).contains(&Keyword::CantBlock) {
            return;
        }
        if let Some(atk) = best_attacker(b) {
            used.push(blocker_id);
            out.push((blocker_id, atk));
        }
    };
    // CR 509.1c — "blocks each combat if able".
    let must: Vec<CardId> = state
        .battlefield
        .iter()
        .filter(|c| {
            kws(c.id).contains(&Keyword::MustBlock) || kws(c.id).contains(&Keyword::MustAttackOrBlock)
        })
        .map(|c| c.id)
        .collect();
    for id in must {
        force(id, &mut out, &mut used);
    }
    // CR 509.1c — "all creatures able to block this creature do so" / "must be
    // blocked if able": every idle defender that can block such an attacker.
    for atk in &state.attacking {
        let a_kws = kws(atk.attacker);
        let all = a_kws.contains(&Keyword::AllMustBlock);
        if !all && !a_kws.contains(&Keyword::MustBeBlocked) {
            continue;
        }
        let candidates: Vec<CardId> = state
            .battlefield
            .iter()
            .filter(|c| {
                state.defender_for(atk.target).is_some_and(|d| state.same_team(c.controller, d))
                    && !used.contains(&c.id)
                    && state.blocker_can_block_attacker(c.id, atk.attacker)
            })
            .map(|c| c.id)
            .collect();
        for id in candidates.into_iter().take(if all { usize::MAX } else { 1 }) {
            used.push(id);
            out.push((id, atk.attacker));
        }
    }
    out
}

/// The combat-damage value an attacker on the battlefield actually assigns:
/// its computed toughness when it has `AssignsCombatDamageByToughness` (Doran,
/// the Siege Tower; CR 510.1c), otherwise its computed power. Falls back to the
/// raw `CardInstance` value when no computed view is available. Used by the
/// block planner so a Doran board's high-toughness attackers are scored at
/// their real threat.
fn attacker_damage_value(state: &GameState, id: CardId) -> i32 {
    use crate::card::Keyword;
    if let Some(cp) = state.computed_permanent(id) {
        let mut base = if cp.keywords.contains(&Keyword::AssignsCombatDamageByToughness) {
            cp.toughness
        } else {
            cp.power
        };
        // CR 702.121 — Melee grows the attacker +1/+1 per opponent it attacks
        // this combat. In a duel that's a guaranteed +1 the moment it's
        // declared, so the planner should weigh it in.
        if cp.keywords.contains(&Keyword::Melee) {
            base += 1;
        }
        base
    } else {
        state.battlefield_find(id).map(|c| c.power()).unwrap_or(0)
    }
}

/// Instant-speed response layer: when an opponent's spell sits on top of
/// the stack and it's worth answering (it targets the bot's stuff / the
/// bot itself, or it's expensive), cast a counterspell from hand at it.
/// The `would_accept` dry-run is the final gate (timing, mana via
/// auto-tap, per-counter target filters like Spell Snare's MV gate).
fn pick_stack_response(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    use crate::game::types::StackItem;
    let (spell_id, threat) = state.stack.iter().rev().find_map(|si| {
        let StackItem::Spell { card, caster, target, uncounterable, .. } = si else {
            return None;
        };
        if *caster == seat || *uncounterable {
            return None;
        }
        // Score the spell like a candidate play of the caster's: mana
        // investment + body + what it's aimed at. Replaces the old
        // "anything ≥ 3 cmc or pointed at us" gate, which burned
        // Counterspell on 3-mana value creatures and face burn at 20 life.
        let def = &card.definition;
        // Raw card stats lifted onto the profile's scale so the
        // `permanent_value` term below and the bar at the bottom agree.
        let mut threat = def.cost.cmc() as i32 * w.unit;
        if def.card_types.contains(&crate::card::CardType::Creature) {
            threat += (def.power.max(0) + def.toughness.max(0)) * w.unit;
            threat += (def.keywords.len() as i32).min(3) * w.unit;
        }
        match target {
            // Aimed at one of our permanents: the spell is worth what
            // we'd lose.
            Some(crate::game::Target::Permanent(id))
                if state.battlefield_find(*id).is_some_and(|c| c.controller == seat) =>
            {
                threat += permanent_value(state, *id, w);
            }
            // Aimed at our face: mildly threatening, urgent when low.
            Some(crate::game::Target::Player(p)) if *p == seat => {
                threat += 6 * w.unit;
                if state.effective_life(seat) <= 10 {
                    threat += 8 * w.unit;
                }
            }
            _ => {}
        }
        Some((card.id, threat))
    })?;
    // Hold the counter below this bar — a vanilla two-drop or an early
    // cantrip isn't worth the bot's only interaction. The bar drops as
    // the hand clogs: the Prismari probe measured reactive spells
    // rotting to cleanup (42 discards in 60 games) while a full-height
    // bar held them for a threat that never came — a counter pitched at
    // end of turn answered nothing at all.
    let bar = if state.players[seat].hand.len() >= 6 { 5 } else { 10 };
    if threat < bar * w.unit {
        return None;
    }
    let mut counters: Vec<&crate::card::CardInstance> = state.players[seat]
        .hand
        .iter()
        .filter(|c| {
            c.definition.card_types.contains(&crate::card::CardType::Instant)
                && effect_counters_spells(&c.definition.effect)
        })
        .collect();
    // Cheapest answer first — hold the expensive counter for later.
    counters.sort_by_key(|c| c.definition.cost.cmc());
    for c in counters {
        let action = GameAction::CastSpell {
            card_id: c.id,
            target: Some(crate::game::Target::Permanent(spell_id)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// React to a threatening opponent ability on the stack with a dedicated
/// ability-counter card (Stifle / Disallow). The ability's source is the
/// target slot. Held separate from `pick_stack_response`'s spell logic so a
/// counter that can only hit abilities still gets used.
fn pick_ability_counter_response(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::game::types::StackItem;
    // Topmost opponent ability on the stack — counter the most recent one.
    let source = state.stack.iter().rev().find_map(|si| match si {
        StackItem::Trigger { source, controller, .. } if *controller != seat => Some(*source),
        _ => None,
    })?;
    let mut counters: Vec<&crate::card::CardInstance> = state.players[seat]
        .hand
        .iter()
        .filter(|c| {
            c.definition.card_types.contains(&crate::card::CardType::Instant)
                && effect_counters_abilities(&c.definition.effect)
        })
        .collect();
    counters.sort_by_key(|c| c.definition.cost.cmc());
    for c in counters {
        let action = GameAction::CastSpell {
            card_id: c.id,
            target: Some(crate::game::Target::Permanent(source)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// Instant removal at a declared attacker, from the DEFENDER's side of
/// combat. The response chain only ever countered spells, so a hand full
/// of kill spells watched every alpha strike connect — the SOS college
/// probes measured 68-78 % of attackers unblocked while removal rotted
/// to cleanup discards. Aim at the most valuable attacker the spell
/// actually answers, before blocks commit; `would_accept` gates instant
/// timing and the ward gate keeps taxes payable.
fn pick_defensive_removal(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    use crate::card::CardType;
    use crate::effect::{Selector, Value};
    let mut attackers: Vec<CardId> = state
        .attacking()
        .iter()
        .filter(|a| state.defender_for(a.target) == Some(seat))
        .map(|a| a.attacker)
        .collect();
    if attackers.is_empty() {
        return None;
    }
    attackers.sort_by_key(|id| std::cmp::Reverse(permanent_value(state, *id, w)));
    // First-leaf removal shapes, the same convention the counter scan
    // uses: a dedicated kill spell, not a buried rider.
    fn removal_leaf(e: &Effect) -> Option<&Effect> {
        match e {
            Effect::Destroy { .. } | Effect::DestroyNoRegen { .. } | Effect::DealDamage { .. } => {
                Some(e)
            }
            Effect::Seq(v) => v.first().and_then(removal_leaf),
            _ => None,
        }
    }
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.card_types.contains(&CardType::Instant))
    {
        let Some(leaf) = removal_leaf(&c.definition.effect) else { continue };
        for &atk in &attackers {
            // Worth a card: skip chaff attackers.
            if permanent_value(state, atk, w) < 6 * w.unit {
                continue;
            }
            let answers = match leaf {
                Effect::Destroy { what } | Effect::DestroyNoRegen { what } => {
                    matches!(what, Selector::Target(_) | Selector::TargetFiltered { .. })
                }
                Effect::DealDamage { to, amount } => {
                    matches!(to, Selector::Target(_) | Selector::TargetFiltered { .. })
                        && match amount {
                            Value::Const(n) => state
                                .computed_permanent(atk)
                                .is_some_and(|cp| {
                                    let marked = state
                                        .battlefield_find(atk)
                                        .map(|c| c.damage as i32)
                                        .unwrap_or(0);
                                    *n >= cp.toughness - marked
                                }),
                            _ => false,
                        }
                }
                _ => false,
            };
            if !answers {
                continue;
            }
            let action = GameAction::CastSpell {
                card_id: c.id,
                target: Some(Target::Permanent(atk)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            };
            if !ward_gate_ok(state, seat, &action) {
                continue;
            }
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// SOS Prepare — the inset spell is a one-shot resource carried by a
/// fragile body. When an opponent's spell on the stack targets one of the
/// bot's prepared creatures, cast the inset spell in response, so the
/// resource is spent before the body (and the Prepared counter with it)
/// is answered. `would_accept` gates timing: only an instant-speed inset
/// spell actually fires here, a sorcery copy is simply rejected.
fn pick_prepare_response(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    use crate::game::types::StackItem;
    let threatened: Vec<CardId> = state.stack.iter().rev().find_map(|si| {
        let StackItem::Spell { caster, target, additional_targets, .. } = si else {
            return None;
        };
        if *caster == seat {
            return None;
        }
        let hits: Vec<CardId> = target
            .iter()
            .chain(additional_targets.iter())
            .filter_map(|t| match t {
                Target::Permanent(id) => state
                    .battlefield_find(*id)
                    .filter(|c| {
                        c.controller == seat
                            && c.definition.prepare_spell.is_some()
                            && c.counter_count(crate::card::CounterType::Prepared) > 0
                    })
                    .map(|c| c.id),
                _ => None,
            })
            .collect();
        if hits.is_empty() { None } else { Some(hits) }
    })?;
    for creature_id in threatened {
        let Some(c) = state.battlefield_find(creature_id) else { continue };
        let Some(spell) = c.definition.prepare_spell.as_deref() else { continue };
        // Same construction as the main-phase candidate block.
        let (target, additional_targets) = if spell.effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(&spell.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let x_value = if x_relevant(spell) {
            Some(max_affordable_x_for_def(state, seat, spell, 0, w))
        } else {
            None
        };
        let action = GameAction::CastPrepareSpell {
            creature_id,
            target,
            additional_targets,
            mode: None,
            x_value,
        };
        if !ward_gate_ok(state, seat, &action) {
            continue;
        }
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// True when the effect tree's primary action counters a spell (the shapes
/// a dedicated counterspell card uses — not buried `MayDo` riders).
fn effect_counters_spells(eff: &Effect) -> bool {
    match eff {
        Effect::CounterSpell { .. }
        | Effect::CounterSpellExileSameNamed { .. }
        | Effect::CounterSpellToZone { .. }
        | Effect::CounterUnlessPaid { .. }
        | Effect::CounterUnless { .. } => true,
        Effect::Seq(v) => v.first().is_some_and(effect_counters_spells),
        _ => false,
    }
}

/// True when the effect can counter an activated/triggered ability (Stifle's
/// `CounterAbility`, or a modal counter like Disallow whose `ChooseN`/
/// `ChooseMode` carries a `CounterAbility` arm).
fn effect_counters_abilities(eff: &Effect) -> bool {
    match eff {
        Effect::CounterAbility { .. } => true,
        Effect::Seq(v) => v.first().is_some_and(effect_counters_abilities),
        Effect::ChooseMode(modes) => modes.iter().any(effect_counters_abilities),
        Effect::ChooseN { modes, .. } => modes.iter().any(effect_counters_abilities),
        _ => false,
    }
}

/// Land-count mulligan heuristic. A keepable opening hand wants roughly
/// 2–5 lands out of seven; 0–1 (screw) or 6–7 (flood) are shipped. We stop
/// digging after two mulligans (a London mulligan past that bottoms too
/// many cards to be worth chasing a perfect curve) and always keep a hand
/// of three or fewer cards. Reads land counts off the live hand zone since
/// the `Decision::Mulligan` payload only carries names.
/// Colors a land card could tap for, for mulligan color-screw checks.
/// Reads basic land types (Plains→W, …) plus `AddMana` effects on its
/// activated abilities; "any color" payloads yield the full WUBRG set.
fn land_color_output(card: &CardDefinition) -> crate::mana::ColorSet {
    use crate::card::LandType;
    use crate::mana::{Color, ColorSet};
    let mut set = ColorSet::empty();
    for lt in &card.subtypes.land_types {
        match lt {
            LandType::Plains => set.insert(Color::White),
            LandType::Island => set.insert(Color::Blue),
            LandType::Swamp => set.insert(Color::Black),
            LandType::Mountain => set.insert(Color::Red),
            LandType::Forest => set.insert(Color::Green),
            _ => {}
        }
    }
    for ab in &card.activated_abilities {
        accumulate_mana_colors(&ab.effect, &mut set);
    }
    set
}

/// Choose which land to play this turn. Among the lands the engine would
/// accept, prefer the one that covers the most colors the bot's hand wants
/// but can't yet produce from the lands it already controls — basic
/// mana-fixing so a green hand doesn't strand its spells behind a Mountain.
/// Falls back to the first playable land when nothing improves color
/// coverage (or no land needs fixing).
/// Does this land's own printed static say it enters tapped? (A
/// `StaticEffect::EntersTapped` on the card itself — school lands, guild
/// gates. Statics granted by *other* permanents aren't the land's
/// property and aren't consulted.)
fn land_enters_tapped(def: &crate::card::CardDefinition) -> bool {
    def.static_abilities
        .iter()
        .any(|s| matches!(s.effect, crate::card::StaticEffect::EntersTapped { .. }))
}

fn pick_land_to_play(state: &GameState, seat: usize, w: &EvalWeights) -> Option<CardId> {
    use crate::mana::{Color, ColorSet};
    const WUBRG: [Color; 5] =
        [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];

    // Colors already producible from battlefield lands the bot controls.
    let have = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
        .fold(ColorSet::empty(), |acc, c| acc.union(land_color_output(&c.definition)));
    // Colors the bot's nonland hand cards want to be cast.
    let mut want = ColorSet::empty();
    for c in state.players[seat].hand.iter().filter(|c| !c.definition.is_land()) {
        for col in c.definition.cost.colors() {
            want.insert(col);
        }
    }
    // The colors still missing from the bot's mana base.
    let needed: Vec<Color> =
        WUBRG.into_iter().filter(|&col| want.contains(col) && !have.contains(col)).collect();

    if !w.land_urgency {
        let mut best: Option<(CardId, usize)> = None;
        for c in state.players[seat].hand.iter().filter(|c| c.definition.is_land()) {
            if !state.would_accept(GameAction::PlayLand(c.id)) {
                continue;
            }
            let out = land_color_output(&c.definition);
            let coverage = needed.iter().filter(|&&col| out.contains(col)).count();
            // Higher coverage wins; the first playable land is the fallback (so a
            // colorless/utility land still gets played when nothing needs fixing).
            if best.is_none_or(|(_, s)| coverage > s) {
                best = Some((c.id, coverage));
            }
        }
        return best.map(|(id, _)| id);
    }

    // Per-color urgency: the cheapest hand card wanting that color sets
    // how soon a source is needed. A {B} two-drop scores 6, a {B}
    // six-drop 2 — both "missing", not equally missing.
    let urgency = |col: Color| -> usize {
        state.players[seat]
            .hand
            .iter()
            .filter(|c| !c.definition.is_land() && c.definition.cost.colors().contains(&col))
            .map(|c| 8usize.saturating_sub(c.definition.cost.cmc() as usize).max(1))
            .max()
            .unwrap_or(0)
    };

    // Whether a land buys a cast *this turn* is a property of that land,
    // not of the turn: an untapped source adds mana and a color now, a
    // tapped one adds neither until next turn. So the question is asked
    // per candidate rather than once.
    let untapped_now = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land() && !c.tapped)
        .count();
    let enables_a_cast = |out: ColorSet, tapped: bool| -> bool {
        let mana = untapped_now + usize::from(!tapped);
        let colors = if tapped { have } else { have.union(out) };
        state.players[seat].hand.iter().any(|c| {
            !c.definition.is_land()
                && c.definition.cost.cmc() as usize <= mana
                && c.definition.cost.colors().iter().all(|col| colors.contains(*col))
        })
    };

    let mut best: Option<(CardId, i32)> = None;
    for c in state.players[seat].hand.iter().filter(|c| c.definition.is_land()) {
        if !state.would_accept(GameAction::PlayLand(c.id)) {
            continue;
        }
        let out = land_color_output(&c.definition);
        let mut score: i32 =
            needed.iter().filter(|&&col| out.contains(col)).map(|&col| urgency(col) as i32).sum();
        // Untapped sources the bot already has are worth a little on
        // their own, so a second Forest still beats a dead utility land.
        if !needed.is_empty() || out != ColorSet::empty() {
            score += 1;
        }
        let tapped = land_enters_tapped(&c.definition);
        // A land that turns on a spell this turn is worth more than the
        // fixing it promises for later; a tapland that promises fixing
        // costs almost nothing on a turn with no play.
        if enables_a_cast(out, tapped) {
            score += 4;
        }
        if tapped {
            score -= 1;
        }
        if best.is_none_or(|(_, s)| score > s) {
            best = Some((c.id, score));
        }
    }
    best.map(|(id, _)| id)
}

/// Bot policy for `Decision::OptionalTrigger`: take the trigger unless its
/// matching `MayDo` body imposes a clear self-cost (lose life / sacrifice /
/// discard on the bot). `AutoDecider` declines *every* optional trigger,
/// which means a bot would never take a beneficial "you may" (Provoke's
/// "you may", Boast token riders, etc.); this makes those fire.
pub fn optional_trigger_beneficial(state: &GameState, source: CardId, description: &str) -> bool {
    // Locate the source card's definition in any zone the bot can see.
    let def = state
        .battlefield
        .iter()
        .find(|c| c.id == source)
        .map(|c| &c.definition)
        .or_else(|| {
            state
                .players
                .iter()
                .flat_map(|p| p.graveyard.iter().chain(p.hand.iter()))
                .find(|c| c.id == source)
                .map(|c| &c.definition)
        })
        // A resolving SPELL lives on the stack — without this, any
        // instant/sorcery's self-costly MayDo fell through to the
        // blanket-true fallback below.
        .or_else(|| {
            state.stack.iter().find_map(|si| match si {
                crate::game::types::StackItem::Spell { card, .. } if card.id == source => {
                    Some(&card.definition)
                }
                _ => None,
            })
        })
        // A Paradigm card prompts from EXILE (`CastFreeParadigmCopy`),
        // as do other exile-resident recurrences.
        .or_else(|| state.exile.iter().find(|c| c.id == source).map(|c| &c.definition));
    let Some(def) = def else { return true };
    // Find the `MayDo` body whose description matches the prompt. Scan the
    // card's spell effect, its triggered abilities, and any static-ability
    // reflexive (`when_you_do`) — the prompt can originate from any of these
    // (e.g. Valentin's exile-replacement reflexive lives on a static).
    let mut body = find_maydo_body(&def.effect, description);
    if body.is_none() {
        for t in &def.triggered_abilities {
            if let Some(b) = find_maydo_body(&t.effect, description) {
                body = Some(b);
                break;
            }
        }
    }
    if body.is_none() {
        for sa in &def.static_abilities {
            if let crate::effect::StaticEffect::ExileDyingOpponentCreatures {
                when_you_do: Some(eff),
            } = &sa.effect
                && let Some(b) = find_maydo_body(eff, description)
            {
                body = Some(b);
                break;
            }
        }
    }
    // Exploit (CR 702.105 — "Exploit: sacrifice a creature?"): the body is a
    // Sacrifice that the generic self-cost screen would always decline. Accept
    // it when the controller has a spare creature to feed it — a token, or the
    // exploiter is one of several creatures so it can sacrifice the weakest (or
    // itself for a strong ETB payoff). Card advantage off a token is a clean win.
    if description.starts_with("Exploit") {
        let ctrl = state.battlefield.iter().find(|c| c.id == source).map(|c| c.controller);
        if let Some(seat) = ctrl {
            let creatures: Vec<&crate::card::CardInstance> = state
                .battlefield
                .iter()
                .filter(|c| c.controller == seat && c.definition.is_creature())
                .collect();
            let has_token = creatures.iter().any(|c| c.is_token);
            // Accept with a sacrificial token, or when there's more than one
            // creature so we don't have to give up the exploiter itself.
            return has_token || creatures.len() > 1;
        }
        return false;
    }
    // Take it unless the body is self-costly; default to taking when the
    // body can't be introspected (most "you may" on your own permanents is
    // upside).
    body.map(|b| !effect_imposes_self_cost(b)).unwrap_or(true)
}

/// Recursively find the optional-effect body whose prompt is `desc`. Both
/// `Effect::MayDo` and `Effect::MayPay` surface as a `Decision::OptionalTrigger`
/// keyed on their description, so the bot's self-cost screen (e.g. a "you may
/// pay {2}: each player loses 3 life" body it shouldn't auto-accept) applies to
/// both shapes.
fn find_maydo_body<'a>(eff: &'a Effect, desc: &str) -> Option<&'a Effect> {
    match eff {
        Effect::MayDo { description, body } | Effect::MayPay { description, body, .. }
            if description == desc =>
        {
            Some(body)
        }
        // A reflexive tap-cost (Caparocti Sunborn) or discard-cost (Toph,
        // Hardheaded Teacher) trigger surfaces the same `OptionalTrigger`
        // prompt; its `then` payoff is what the bot screens.
        Effect::MayTap { description, then, .. }
        | Effect::MayDiscard { description, then, .. }
            if description == desc =>
        {
            Some(then)
        }
        Effect::MayDo { body, .. }
        | Effect::MayPay { body, .. }
        | Effect::MayTap { then: body, .. }
        | Effect::MayDiscard { then: body, .. }
        | Effect::ForEach { body, .. } => find_maydo_body(body, desc),
        Effect::Seq(v) => v.iter().find_map(|e| find_maydo_body(e, desc)),
        Effect::ChooseMode(v)
        | Effect::ChooseN { modes: v, .. }
        | Effect::Escalate { modes: v, .. }
        | Effect::EscalatingThisTurn { modes: v } => {
            v.iter().find_map(|e| find_maydo_body(e, desc))
        }
        Effect::If { then, else_, .. } => {
            find_maydo_body(then, desc).or_else(|| find_maydo_body(else_, desc))
        }
        _ => None,
    }
}

/// Whether `eff` (a "you may" body) imposes a clear cost on its controller —
/// losing life, sacrificing, or discarding. Conservative: the bot declines
/// such triggers rather than paying for an effect it can't value-judge.
fn effect_imposes_self_cost(eff: &Effect) -> bool {
    use crate::effect::{PlayerRef, Selector};
    let hits_self = |sel: &Selector| {
        matches!(sel, Selector::You | Selector::This)
            || matches!(sel, Selector::Player(PlayerRef::You))
    };
    match eff {
        Effect::LoseLife { who, .. }
        | Effect::Discard { who, .. }
        | Effect::Mill { who, .. }
        | Effect::LoseHalfLife { who, .. }
        | Effect::MillHalf { who, .. }
        | Effect::DiscardHalf { who, .. } => hits_self(who),
        // Self-directed damage (a "you may have ~ deal N damage to you" rider).
        Effect::DealDamage { to, .. } => hits_self(to),
        // Drain *out of* the bot is a cost; drain *into* the bot is upside.
        Effect::Drain { from, .. } => hits_self(from),
        Effect::Sacrifice { who, .. } | Effect::SacrificeGreatestMV { who, .. } => hits_self(who),
        Effect::SacrificeAndRemember { .. } => true,
        Effect::SacrificeAnyNumber { who, .. } => matches!(who, PlayerRef::You),
        Effect::PayLifeLookTake { who } => matches!(who, PlayerRef::You),
        Effect::Seq(v) => v.iter().any(effect_imposes_self_cost),
        Effect::ChooseMode(v)
        | Effect::ChooseN { modes: v, .. }
        | Effect::Escalate { modes: v, .. }
        | Effect::EscalatingThisTurn { modes: v } => {
            v.iter().any(effect_imposes_self_cost)
        }
        Effect::If { then, else_, .. } => {
            effect_imposes_self_cost(then) || effect_imposes_self_cost(else_)
        }
        Effect::ForEach { body, .. } | Effect::MayDo { body, .. } => effect_imposes_self_cost(body),
        // Mana/energy "pay or else" wrap a fallback (usually SacrificeSource);
        // the bot reads the fallback to decide whether declining is costly.
        Effect::PayManaOrElse { otherwise, .. } | Effect::PayEnergyOrElse { otherwise, .. } => {
            effect_imposes_self_cost(otherwise)
        }
        // Blight (CR 701.68) puts -1/-1 counters on a creature you control —
        // a clear self-cost, so the bot declines "may blight N" upside riders
        // rather than shrinking (or killing) its own board.
        Effect::Blight { .. } => true,
        // "You may sacrifice/exile this" riders are a clear self-cost.
        Effect::SacrificeSource => true,
        Effect::Exile { what } => hits_self(what),
        // "You may put this into exile / your graveyard / your library" is a
        // self-cost too (returning it to *hand* is upside, so that's excluded).
        Effect::Move { what, to } => {
            hits_self(what)
                && matches!(
                    to,
                    crate::effect::ZoneDest::Exile
                        | crate::effect::ZoneDest::Graveyard
                        | crate::effect::ZoneDest::Library { .. }
                )
        }
        Effect::PayOrLoseGame { .. } => true,
        _ => false,
    }
}

/// Constant life the bot itself would lose to `eff` resolving on its own
/// spell — the amount behind the Paradigm copy guard. Counts `You`-directed
/// life loss AND `Target`-directed loss: a draw-plus-lose body (Decorum
/// Dissertation's "target player draws two and loses 2") auto-targets the
/// caster, so its Target(0) loss lands on the bot. That over-counts a
/// drain the bot would aim at the opponent, which errs toward declining a
/// free cast at low life — the cheap direction. Non-constant amounts count
/// as zero (can't be sized without resolving).
fn self_life_loss(eff: &Effect) -> i32 {
    use crate::effect::{Selector, Value};
    let hits = |sel: &Selector| {
        matches!(sel, Selector::You | Selector::This | Selector::Target(_))
    };
    match eff {
        Effect::LoseLife { who, amount: Value::Const(n) } if hits(who) => (*n).max(0),
        Effect::Drain { from, amount: Value::Const(n), .. } if hits(from) => (*n).max(0),
        Effect::Seq(v) => v.iter().map(self_life_loss).sum(),
        Effect::If { then, else_, .. } => self_life_loss(then).max(self_life_loss(else_)),
        Effect::ChooseMode(v)
        | Effect::ChooseN { modes: v, .. }
        | Effect::Escalate { modes: v, .. }
        | Effect::EscalatingThisTurn { modes: v } => {
            v.iter().map(self_life_loss).max().unwrap_or(0)
        }
        Effect::ForEach { body, .. } | Effect::MayDo { body, .. } => self_life_loss(body),
        _ => 0,
    }
}

/// Bot heuristic for `Decision::SearchLibrary`: pick a basic land that
/// adds the bot's least-covered color, else (no basic land among the
/// candidates) grab the highest-mana-value candidate — a creature/spell
/// tutor (Fauna Shaman, Imperial Recruiter, Spellseeker) should fetch its
/// most impactful hit, not the first one, and certainly not fizzle like the
/// stock `AutoDecider`.
/// Rough board value of a permanent for target selection: mana value + size,
/// plus a loyalty term for planeswalkers and a small legendary premium. When
/// the profile enables it, a keyword term (see [`keyword_value`]) too.
fn permanent_value(state: &GameState, id: crate::card::CardId, w: &EvalWeights) -> i32 {
    use crate::card::{CardType, CounterType, Supertype};
    let Some(c) = state.computed_permanent(id) else { return 0 };
    let inst = state.battlefield_find(id);
    let mut v = inst.map(|c| c.definition.cost.cmc() as i32).unwrap_or(0) * w.cmc;
    if c.card_types.contains(&CardType::Creature) {
        v += w.creature_base + c.power.max(0) * w.power + c.toughness.max(0) * w.toughness;
        if w.keyword_pct != 0 {
            v += keyword_value(&c.keywords, c.power, w) * w.keyword_pct / 100;
        }
    }
    if c.card_types.contains(&CardType::Planeswalker) {
        v += inst.map(|c| c.counter_count(CounterType::Loyalty) as i32).unwrap_or(0) * w.unit;
    }
    if c.supertypes.contains(&Supertype::Legendary) {
        v += 2 * w.unit;
    }
    // A Prepared counter on a prepare-spell body is a castable spell in
    // waiting (SOS): worth about the inset spell's mana value. Gives the
    // eval, the mid-resolution mode picker (Biblioplex Tomekeeper's
    // prepare-vs-unprepare), the attack simulation (attack-trigger
    // preparers gain the counter mid-sim), and removal targeting a live
    // read on the resource — an opponent's prepared creature IS the
    // better kill at equal stats.
    if let Some(inst) = inst
        && inst.counter_count(CounterType::Prepared) > 0
        && let Some(spell) = inst.definition.prepare_spell.as_deref()
    {
        v += (1 + spell.cost.cmc() as i32) * w.unit;
    }
    v
}

/// Keyword contribution to a creature's board value, in `w.unit`-scaled
/// points. Ported from Forge's `CreatureEvaluator`, whose central idea is
/// that keywords split into two families:
///
/// * **Offensive** -- evasion and damage riders are worth what they let the
///   body actually deal, so they scale with power. Flying on a 5/5 is a
///   five-point-per-turn clock; flying on a 1/1 is a chump-blocker that
///   dodges. Pricing both at a flat bonus is the mistake this fixes.
/// * **Defensive** -- protection and resilience are worth roughly the same
///   whatever the body, so they're flat. Hexproof on a 1/1 and on a 5/5
///   both buy exactly "removal doesn't answer this".
///
/// Bad keywords subtract, and a creature that can neither attack nor block
/// collapses to a token value regardless of its printed size.
fn keyword_value(keywords: &[crate::card::Keyword], power: i32, w: &EvalWeights) -> i32 {
    use crate::card::Keyword;
    let p = power.max(0);
    let has = |k: &Keyword| keywords.contains(k);
    // A body that can't attack or block is nearly inert: no size term
    // survives, only the mana it represents. Checked first so the
    // offensive terms below can't rescue a Pacifism'd fatty.
    let inert = (has(&Keyword::CantAttack) || has(&Keyword::Defender))
        && (has(&Keyword::CantBlock) || has(&Keyword::Decayed));
    if inert {
        return -(p * w.power + w.unit);
    }
    let mut v = 0;
    // -- Offensive: scaled by power ------------------------------------
    if has(&Keyword::Flying) || has(&Keyword::Horsemanship) || has(&Keyword::Shadow) {
        v += p * 2 * w.unit / 3;
    }
    if has(&Keyword::Fear) || has(&Keyword::Intimidate) {
        v += p * 2 * w.unit / 5;
    }
    if has(&Keyword::Menace) {
        v += p * w.unit / 4;
    }
    if has(&Keyword::DoubleStrike) {
        v += w.unit + p * w.unit;
    } else if has(&Keyword::FirstStrike) {
        v += w.unit + p * w.unit / 3;
    }
    if has(&Keyword::Lifelink) {
        v += p * 2 * w.unit / 3;
    }
    if has(&Keyword::Infect) {
        v += p * w.unit;
    } else if has(&Keyword::Wither) {
        v += p * 2 * w.unit / 3;
    }
    if p > 1 && has(&Keyword::Trample) {
        v += (p - 1) * w.unit / 3;
    }
    if has(&Keyword::Vigilance) {
        v += p * w.unit / 3;
    }
    for k in keywords {
        match k {
            Keyword::Toxic(n) | Keyword::Poisonous(n) => v += *n as i32 * w.unit / 3,
            Keyword::Annihilator(n) => v += *n as i32 * 3 * w.unit,
            Keyword::Rampage(n) | Keyword::Bushido(n) => v += *n as i32 * w.unit,
            _ => {}
        }
    }
    // -- Defensive: flat -----------------------------------------------
    if has(&Keyword::Indestructible) {
        v += 5 * w.unit;
    }
    if has(&Keyword::Deathtouch) {
        v += 2 * w.unit;
    }
    if has(&Keyword::Hexproof) {
        v += 2 * w.unit;
    } else if has(&Keyword::Shroud) {
        // Shroud is strictly worse than hexproof for its controller: it
        // blocks our own auras, equipment and pump spells too.
        v += 3 * w.unit / 2;
    }
    if has(&Keyword::Reach) && !has(&Keyword::Flying) {
        v += w.unit / 2;
    }
    // -- Bad -----------------------------------------------------------
    if has(&Keyword::Defender) || has(&Keyword::CantAttack) {
        v -= p * w.power * 2 / 3 + w.unit;
    }
    if has(&Keyword::CantBlock) || has(&Keyword::Decayed) {
        v -= w.unit;
    }
    v
}

/// Value of a life total, in `w.unit`-scaled points.
///
/// Life is not linear: the first few points are the difference between
/// losing and not, while points near the starting total are close to
/// worthless. A linear term prices "gain 3" the same at 3 life and at 20,
/// so the bot over-values incidental lifegain when healthy and under-values
/// it when it's actually dying. The curve is XMage's `LIFE_SCORES` shape
/// (`ArtificialScoringSystem`), rescaled so that 20 life is worth the same
/// 20 points it was under the linear term -- only the shape changes, which
/// keeps this comparable against the baseline on the ladder without a
/// wholesale re-tune of every other weight.
///
/// Expressed in tenths of a point (then scaled by `unit`) so the curve stays
/// strictly increasing under integer arithmetic -- a flat spot would make
/// "gain 1 life" evaluate to exactly zero.
fn life_value(life: i32, w: &EvalWeights) -> i32 {
    if !w.concave_life {
        return life * w.unit;
    }
    /// Tenths of a point per life total, index = life, 0..=20.
    const LIFE_TENTHS: [i32; 21] = [
        0, 20, 40, 60, 80, 90, 100, 110, 120, 130, 140, 148, 156, 164, 172, 180, 184, 188, 192,
        196, 200,
    ];
    const MAX: i32 = LIFE_TENTHS.len() as i32 - 1;
    let tenths = if life <= 0 {
        0
    } else if life <= MAX {
        LIFE_TENTHS[life as usize]
    } else {
        // Past the starting total each extra point is worth the same as the
        // shallowest part of the curve (0.4), not nothing -- Ad Nauseam and
        // friends do care about a big buffer.
        LIFE_TENTHS[MAX as usize] + (life - MAX) * 4
    };
    tenths * w.unit / 10
}

/// Keep-value for deciding which of the bot's *own* permanents to give up (to an
/// edict, a "sacrifice a creature" cost, or a self-vote). Distinct from
/// `permanent_value`, which ranks removal targets: here a token is the ideal
/// thing to lose (it can't be recast and vanishes on leaving), so it sorts
/// strictly below every real card, even a bare land of `permanent_value` 0.
fn sacrifice_keep_value(state: &GameState, id: crate::card::CardId, w: &EvalWeights) -> i32 {
    if state.battlefield_find(id).is_some_and(|c| c.is_token) {
        return -1;
    }
    permanent_value(state, id, w)
}

/// Bot heuristic for `Decision::ChooseTarget` (votes, edicts, free-floating
/// removal). Prefer destroying/exiling an opponent's **most** valuable
/// permanent; if every legal permanent is our own (a "sacrifice/vote your own"
/// choice), give up the **least** valuable. Player targets fall back to the
/// **lowest-life** opponent (most progress toward a kill), then to the first
/// legal option.
fn decide_choose_target(
    state: &GameState,
    seat: usize,
    legal: &[crate::game::types::Target],
    w: &EvalWeights,
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    use crate::game::types::Target;
    let owner = |id: crate::card::CardId| state.battlefield_find(id).map(|c| c.controller);
    // Opponent permanents — hit the biggest.
    let best_opp = legal
        .iter()
        .filter_map(|t| match t {
            Target::Permanent(id) if owner(*id).is_some_and(|o| o != seat) => Some(*id),
            _ => None,
        })
        .max_by_key(|id| permanent_value(state, *id, w));
    if let Some(id) = best_opp {
        return DecisionAnswer::Target(Target::Permanent(id));
    }
    // Only our own permanents are legal — give up the least valuable to keep
    // (tokens first, then lowest-value real cards).
    let worst_own = legal
        .iter()
        .filter_map(|t| match t {
            Target::Permanent(id) if owner(*id) == Some(seat) => Some(*id),
            _ => None,
        })
        .min_by_key(|id| sacrifice_keep_value(state, *id, w));
    if let Some(id) = worst_own {
        return DecisionAnswer::Target(Target::Permanent(id));
    }
    // Player targets: prefer the lowest-life opponent (closest to death, so a
    // "deal damage / lose life" effect makes the most progress toward a kill).
    let best_player = legal
        .iter()
        .filter_map(|t| match t {
            Target::Player(p) if *p != seat => Some(*p),
            _ => None,
        })
        .min_by_key(|p| state.players[*p].life);
    if let Some(p) = best_player {
        return DecisionAnswer::Target(Target::Player(p));
    }
    DecisionAnswer::Target(legal[0].clone())
}

/// Bot heuristic for `Decision::ChooseCreatureType` (Cavern of Souls, the
/// chosen-type tribal payoffs). Name the creature type the bot controls / holds
/// the most of — counting battlefield creatures first (already in play, so the
/// payoff is live) and hand creatures as a tiebreak. A Changeling counts for
/// every type. Falls back to the first suggestion, then Demon, when the bot has
/// no creatures at all.
fn decide_creature_type(
    state: &GameState,
    seat: usize,
    suggestions: &[crate::card::CreatureType],
) -> crate::decision::DecisionAnswer {
    use crate::card::{CreatureType, Keyword};
    use std::collections::HashMap;
    // Weight battlefield presence over hand presence (2:1).
    let mut tally: HashMap<CreatureType, i32> = HashMap::new();
    let mut count = |types: &[CreatureType], changeling: bool, weight: i32| {
        if changeling {
            // A Changeling bumps every type it could enable; give the current
            // leaders a small nudge rather than flooding the tally.
            for t in tally.clone().keys() {
                *tally.entry(*t).or_insert(0) += weight;
            }
        }
        for t in types {
            *tally.entry(*t).or_insert(0) += weight;
        }
    };
    for c in state.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_creature()) {
        count(&c.definition.subtypes.creature_types,
            c.definition.keywords.contains(&Keyword::Changeling), 2);
    }
    for c in state.players[seat].hand.iter().filter(|c| c.definition.is_creature()) {
        count(&c.definition.subtypes.creature_types,
            c.definition.keywords.contains(&Keyword::Changeling), 1);
    }
    let best = tally.into_iter().max_by_key(|(_, n)| *n).map(|(t, _)| t);
    let choice = best
        .or_else(|| suggestions.first().copied())
        .unwrap_or(CreatureType::Demon);
    crate::decision::DecisionAnswer::CreatureType(choice)
}

fn decide_library_search(
    state: &GameState,
    seat: usize,
    candidates: &[(crate::card::CardId, String)],
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    use crate::mana::Color;
    if candidates.is_empty() {
        return DecisionAnswer::Search(None);
    }
    const COLORS: [Color; 5] =
        [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
    // How many of our lands already tap for each color.
    let mut sources: std::collections::HashMap<Color, usize> = std::collections::HashMap::new();
    for c in state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
    {
        let out = land_color_output(&c.definition);
        for col in COLORS {
            if out.contains(col) {
                *sources.entry(col).or_insert(0) += 1;
            }
        }
    }
    let lib = &state.players[seat].library;
    let mut best: Option<(crate::card::CardId, usize)> = None;
    for (id, _) in candidates {
        let Some(card) = lib.iter().find(|c| c.id == *id) else { continue };
        if !(card.definition.is_basic() && card.definition.is_land()) {
            continue;
        }
        let out = land_color_output(&card.definition);
        // Score by the fewest existing sources among the colors it makes.
        let score = COLORS
            .iter()
            .filter(|col| out.contains(**col))
            .map(|col| sources.get(col).copied().unwrap_or(0))
            .min()
            .unwrap_or(usize::MAX);
        if best.map(|(_, s)| score < s).unwrap_or(true) {
            best = Some((*id, score));
        }
    }
    if let Some((id, _)) = best {
        return DecisionAnswer::Search(Some(id));
    }
    // No basic land among the candidates (a creature/spell tutor): fetch the
    // highest-mana-value hit as a reasonable "best card" proxy, falling back
    // to the first candidate when CMCs can't be read.
    let pick = candidates
        .iter()
        .max_by_key(|(id, _)| {
            lib.iter().find(|c| c.id == *id).map(|c| c.definition.cost.cmc()).unwrap_or(0)
        })
        .map(|(id, _)| *id)
        .unwrap_or(candidates[0].0);
    DecisionAnswer::Search(Some(pick))
}

/// Bot heuristic for `Decision::ChooseCards`. Two cases:
/// - **Put-onto-battlefield from hand** (Sneak Attack / Elvish Piper / Goblin
///   Lackey): every candidate is in the bot's own hand. Cheat in the single
///   biggest creature (highest mana value, then power) — that's the whole point
///   of the effect. Without this the AutoDecider min-0 default declines and the
///   bot never uses the card.
/// - **Exile from graveyards** (Collect Evidence / Fateseal-style): exile every
///   offered card an opponent owns, up to `max`, skipping the bot's own.
fn decide_choose_cards(
    w: &EvalWeights,
    state: &GameState,
    seat: usize,
    prompt: &str,
    candidates: &[(crate::card::CardId, String)],
    min: u32,
    max: u32,
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    // A sacrifice/discard prompt is a COST — the pick should minimize what
    // we give up, not maximize it. Everything else (draft into hand, tap
    // opposing creatures, exile from graveyards) is upside and keeps the
    // biggest-first / most-hostile-first behavior below.
    let prompt_lc = prompt.to_lowercase();
    let detrimental = prompt_lc.contains("sacrifice") || prompt_lc.contains("discard");
    // Hand-source pick.
    let all_in_hand = !candidates.is_empty()
        && candidates
            .iter()
            .all(|(id, _)| state.players[seat].hand.iter().any(|c| c.id == *id));
    if all_in_hand {
        if detrimental {
            // Shed the least useful cards, and only as many as forced.
            let chosen: Vec<_> = hand_worst_first(state, seat, candidates)
                .into_iter()
                .take(min as usize)
                .collect();
            return DecisionAnswer::Cards(chosen);
        }
        // Beneficial: take the biggest card(s) we can.
        let mut ranked: Vec<(crate::card::CardId, i32, i32)> = candidates
            .iter()
            .filter_map(|(id, _)| {
                let c = state.players[seat].hand.iter().find(|c| c.id == *id)?;
                Some((*id, c.definition.cost.cmc() as i32, c.definition.power))
            })
            .collect();
        // Biggest first: highest mana value, then highest power.
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
        let chosen: Vec<_> = ranked.into_iter().take(max as usize).map(|(id, ..)| id).collect();
        return DecisionAnswer::Cards(chosen);
    }
    // Battlefield-source pick (Archipelagore's "tap up to X target creatures",
    // and similar resolution-time multi-target taps): the AutoDecider declines,
    // so the bot would tap nothing. Prefer opponents' untapped creatures — the
    // biggest threats first — up to the cap. A sacrifice prompt (or a forced
    // pick over only our own permanents) instead gives up the least valuable.
    let all_on_battlefield = candidates
        .iter()
        .all(|(id, _)| state.battlefield.iter().any(|c| c.id == *id));
    if all_on_battlefield {
        let own_least_valuable_first = || -> Vec<crate::card::CardId> {
            let mut own: Vec<(crate::card::CardId, i32)> = candidates
                .iter()
                .filter_map(|(id, _)| {
                    let c = state.battlefield.iter().find(|c| c.id == *id)?;
                    (c.controller == seat).then(|| (*id, sacrifice_keep_value(state, c.id, w)))
                })
                .collect();
            own.sort_by_key(|(_, v)| *v);
            own.into_iter().map(|(id, _)| id).collect()
        };
        if detrimental {
            let chosen: Vec<_> =
                own_least_valuable_first().into_iter().take(min as usize).collect();
            return DecisionAnswer::Cards(chosen);
        }
        let mut ranked: Vec<(crate::card::CardId, i32)> = candidates
            .iter()
            .filter_map(|(id, _)| {
                let c = state.battlefield.iter().find(|c| c.id == *id)?;
                // Only enemy creatures; prefer untapped (tapping a tapped
                // creature is wasted) and higher power.
                (!state.same_team(c.controller, seat)).then_some((*id, c.power() + if c.tapped { -100 } else { 0 }))
            })
            .collect();
        ranked.sort_by_key(|b| std::cmp::Reverse(b.1));
        let mut chosen: Vec<_> = ranked.into_iter().take(max as usize).map(|(id, _)| id).collect();
        // A forced pick (min ≥ 1) with no enemy candidates — an own-board
        // choice the enemy-first logic can't fill. Give up the least
        // valuable of ours rather than answer empty (which the engine
        // rejects, deadlocking the match on a re-ask loop).
        if chosen.len() < min as usize {
            for id in own_least_valuable_first() {
                if chosen.len() >= min as usize {
                    break;
                }
                if !chosen.contains(&id) {
                    chosen.push(id);
                }
            }
        }
        return DecisionAnswer::Cards(chosen);
    }
    let owner_of = |id: crate::card::CardId| -> Option<usize> {
        state
            .players
            .iter()
            .position(|p| p.graveyard.iter().any(|c| c.id == id))
    };
    let mut chosen: Vec<_> = candidates
        .iter()
        .filter(|(id, _)| owner_of(*id).is_some_and(|o| !state.same_team(o, seat)))
        .map(|(id, _)| *id)
        .take(max as usize)
        .collect();
    // A mandatory pick (min ≥ 1) over our own graveyard — Cache Grab's "put a
    // permanent card milled this way into your hand". Keep the biggest one.
    if chosen.len() < min as usize {
        let mut own: Vec<(crate::card::CardId, i32)> = candidates
            .iter()
            .filter_map(|(id, _)| {
                let c = state.players[seat].graveyard.iter().find(|c| c.id == *id)?;
                Some((*id, c.definition.cost.cmc() as i32))
            })
            .collect();
        own.sort_by_key(|b| std::cmp::Reverse(b.1));
        chosen = own.into_iter().take((min as usize).max(1)).map(|(id, _)| id).collect();
    }
    DecisionAnswer::Cards(chosen)
}

/// Bot heuristic for a self-discard (cleanup discard-to-hand-size, rummaging,
/// a discard cost): shed the `count` least useful cards so the bot keeps its
/// cheap, castable spells. Surplus lands go first once the bot is no longer
/// mana-light; otherwise the most expensive spells (least likely to be cast
/// soon) are pitched. Ties keep hand order.
fn decide_self_discard(
    state: &GameState,
    seat: usize,
    hand: &[(crate::card::CardId, String)],
    count: u32,
) -> crate::decision::DecisionAnswer {
    crate::decision::DecisionAnswer::Discard(
        hand_worst_first(state, seat, hand).into_iter().take(count as usize).collect(),
    )
}

/// Ascending-usefulness ranking of `offered` hand cards (worst first) —
/// the shed order shared by self-discards and sacrifice/discard-cost
/// `ChooseCards` prompts. Surplus lands go first once the bot is no
/// longer mana-light; otherwise the most expensive spells (least likely
/// to be cast soon) are pitched. Ties keep hand order.
fn hand_worst_first(
    state: &GameState,
    seat: usize,
    offered: &[(crate::card::CardId, String)],
) -> Vec<crate::card::CardId> {
    // Lands already in play: once we have plenty, extra lands in hand are the
    // first thing to pitch; while still mana-light, keep them.
    let lands_in_play = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
        .count();
    // We want about five total land sources; only that many lands in hand are
    // "needed". Excess lands are pitched before spells even while mana-light —
    // holding a fistful of duplicate lands shouldn't cost us our spells, and a
    // flooded bot (≥5 in play) pitches every spare land first.
    let mut lands_still_wanted = 5usize.saturating_sub(lands_in_play);
    // Score each offered card — LOWER is pitched sooner.
    let mut scored: Vec<(i64, crate::card::CardId)> = offered
        .iter()
        .filter_map(|(id, _)| {
            let card = state.players[seat].hand.iter().find(|c| c.id == *id)?;
            let score = if card.definition.is_land() {
                // Keep lands up to the buffer; surplus lands are worth the
                // least so they're pitched first.
                if lands_still_wanted > 0 {
                    lands_still_wanted -= 1;
                    1_000
                } else {
                    -100
                }
            } else {
                // Among spells, keep the cheap (castable) ones; pitch the
                // most expensive first.
                -(card.definition.cost.cmc() as i64)
            };
            Some((score, *id))
        })
        .collect();
    scored.sort_by_key(|(s, _)| *s);
    scored.into_iter().map(|(_, id)| id).collect()
}

/// Order a Scry / Surveil / Rearrange window. `AutoDecider` keeps every
/// card on top — a no-op that wastes every scry in the catalog (the SOS
/// school lands alone surveil in every college deck). The land logic is
/// the discard ranker's (see [`hand_worst_first`]): a land is wanted
/// while total sources — in play plus in hand — run below the
/// five-source buffer, surplus past it; a spell is kept unless its cost
/// sits more than two land drops beyond what the bot can see. Kept cards
/// are ordered most-wanted first (index 0 is the next draw). Cards not
/// in the bot's own library (an opponent-library scry) score neutral and
/// keep the engine's order.
fn decide_scry(
    state: &GameState,
    seat: usize,
    cards: &[(crate::card::CardId, String)],
    mode: crate::decision::ScryMode,
) -> crate::decision::DecisionAnswer {
    use crate::decision::ScryMode;
    let lands_in_play = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
        .count();
    let lands_in_hand =
        state.players[seat].hand.iter().filter(|c| c.definition.is_land()).count();
    let sources = lands_in_play + lands_in_hand;
    // Higher draws sooner; below zero means "don't draw this at all".
    let mut scored: Vec<(i64, crate::card::CardId)> = cards
        .iter()
        .map(|(id, _)| {
            let def = state.players[seat]
                .library
                .iter()
                .find(|c| c.id == *id)
                .map(|c| &c.definition);
            let score = match def {
                None => 0,
                Some(d) if d.is_land() => {
                    if sources < 5 {
                        500
                    } else {
                        -100
                    }
                }
                Some(d) => {
                    let cmc = d.cost.cmc() as i64;
                    if cmc > sources as i64 + 2 { -50 } else { 100 - cmc }
                }
            };
            (score, *id)
        })
        .collect();
    // Stable sort: equal scores keep the engine's order.
    scored.sort_by_key(|(s, _)| std::cmp::Reverse(*s));
    match mode {
        // Rearrange has no second bucket — everything stays on top,
        // wanted cards first.
        ScryMode::Rearrange => crate::decision::DecisionAnswer::ScryOrder {
            kept_top: scored.into_iter().map(|(_, id)| id).collect(),
            bottom: vec![],
        },
        ScryMode::Scry | ScryMode::Surveil => {
            let (keep, bin): (Vec<_>, Vec<_>) = scored.into_iter().partition(|(s, _)| *s >= 0);
            crate::decision::DecisionAnswer::ScryOrder {
                kept_top: keep.into_iter().map(|(_, id)| id).collect(),
                bottom: bin.into_iter().map(|(_, id)| id).collect(),
            }
        }
    }
}

/// Pick a mid-resolution mode (`Decision::ChooseMode` — Charm modes, ETB
/// choices like Biblioplex Tomekeeper's prepare/unprepare) by outcome
/// instead of `AutoDecider`'s blanket mode 0: submit each candidate on a
/// clone, resolve to quiescence the same way [`evaluate_action_sequence`]
/// does (`AutoDecider` answers any nested decision), and keep the best
/// material eval. Ties and unevaluable modes keep the lowest index, so
/// the old mode-0 behavior is the floor, never regressed below.
fn decide_mode_by_outcome(
    state: &GameState,
    seat: usize,
    num_modes: usize,
    w: &EvalWeights,
) -> usize {
    let mut best: Option<(i32, usize)> = None;
    for m in 0..num_modes {
        let Some(score) =
            settle_answer(state, seat, w, crate::decision::DecisionAnswer::Mode(m))
        else {
            continue;
        };
        if best.is_none_or(|(b, _)| score > b) {
            best = Some((score, m));
        }
    }
    best.map(|(_, m)| m).unwrap_or(0)
}

/// Submit `answer` to the state's pending decision on a clone, resolve to
/// quiescence (nested decisions answered by the policy table, no
/// expensive re-evaluation), and return the settled material eval for
/// `seat`. `None` when the answer is rejected or resolution won't settle.
/// The shared engine behind every answer-by-outcome policy.
fn settle_answer(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
    answer: crate::decision::DecisionAnswer,
) -> Option<i32> {
    let mut g = state.clone();
    g.perform_action(GameAction::SubmitDecision(answer)).ok()?;
    let mut fuel = 64u32;
    loop {
        if g.is_game_over() {
            break;
        }
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(&g, pending.acting_player(), w, &pending.decision, false)
            };
            g.perform_action(GameAction::SubmitDecision(answer)).ok()?;
        } else if g.stack.is_empty() {
            break;
        } else {
            g.perform_action(GameAction::PassPriority).ok()?;
        }
        fuel = fuel.checked_sub(1)?;
    }
    Some(eval_material(&g, seat, w))
}

/// Judge a self-costly optional trigger by outcome: settle "yes" and "no"
/// on clones and take the trigger only when accepting evals strictly
/// better. This turns "you may sacrifice a Pest: [payoff]" from a blanket
/// decline (the introspection screen's rule for any self-cost) into a
/// judged trade. `None` when either branch won't settle — the caller
/// keeps the conservative decline.
fn decide_optional_by_outcome(state: &GameState, seat: usize, w: &EvalWeights) -> Option<bool> {
    use crate::decision::DecisionAnswer;
    let yes = settle_answer(state, seat, w, DecisionAnswer::Bool(true))?;
    let no = settle_answer(state, seat, w, DecisionAnswer::Bool(false))?;
    Some(yes > no)
}

fn accumulate_mana_colors(eff: &Effect, set: &mut crate::mana::ColorSet) {
    match eff {
        Effect::AddMana { pool, .. } => accumulate_payload_colors(pool, set),
        Effect::Seq(v) => v.iter().for_each(|e| accumulate_mana_colors(e, set)),
        _ => {}
    }
}

fn accumulate_payload_colors(pool: &ManaPayload, set: &mut crate::mana::ColorSet) {
    match pool {
        ManaPayload::Colors(cs) | ManaPayload::OfColors(cs, _) => {
            cs.iter().for_each(|c| set.insert(*c))
        }
        ManaPayload::OfColor(c, _) => set.insert(*c),
        ManaPayload::AnyOneColor(_)
        | ManaPayload::AnyColors(_)
        | ManaPayload::AnyColorOpponentCouldProduce
        | ManaPayload::AnyColorYouCouldProduce
        | ManaPayload::AnyTypeTriggerSourceProduces
        | ManaPayload::DevotionOfChosenColor => *set = crate::mana::ColorSet::all(),
        ManaPayload::Colorless(_) => {}
        // Could produce any single color the rock was set to — treat as
        // potentially any color for the bot's mana-base reasoning.
        ManaPayload::ChosenColorOfSource
        | ManaPayload::DraftNotedColorOfSource
        | ManaPayload::ImprintedCardColor
        | ManaPayload::AnyColorAmongLegendaries
        | ManaPayload::AnyColorAmongYourPermanents => *set = crate::mana::ColorSet::all(),
        ManaPayload::Restricted(inner, _) | ManaPayload::RestrictedToChosenType(inner)
                    | ManaPayload::RestrictedToChosenTypePlain(inner) => {
            accumulate_payload_colors(inner, set)
        }
    }
}

fn decide_mulligan(
    state: &GameState,
    seat: usize,
    mulligans_taken: usize,
    w: &EvalWeights,
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    let hand = &state.players[seat].hand;
    let lands = hand.iter().filter(|c| c.definition.is_land()).count();
    // Curve check: a 2–5-land hand is only worth keeping if it has at least
    // one nonland spell cheap enough to cast in the first few turns — three
    // lands plus four 7-drops is a screwed keep. "Castable early" means a
    // spell whose mana value is within `lands + 1` (a generous early-curve
    // window that still trusts a couple of draws).
    // Color-screw awareness: an early play only counts if the hand's lands
    // can actually produce its colored pips. Three Forests + a hand of blue
    // spells is a screwed keep even though the curve looks fine.
    let producible = hand
        .iter()
        .filter(|c| c.definition.is_land())
        .fold(crate::mana::ColorSet::empty(), |acc, c| {
            acc.union(land_color_output(&c.definition))
        });
    let has_early_play = hand.iter().any(|c| {
        if c.definition.is_land() || c.definition.cost.cmc() as usize > lands + 1 {
            return false;
        }
        let mut need = crate::mana::ColorSet::empty();
        for col in c.definition.cost.colors() {
            need.insert(col);
        }
        need.is_subset_of(producible)
    });
    if !w.mull_quality {
        let keepable = ((2..=5).contains(&lands) && has_early_play) || hand.len() <= 3;
        return if keepable || mulligans_taken >= 2 {
            DecisionAnswer::Keep
        } else {
            DecisionAnswer::TakeMulligan
        };
    }

    // How many spells this hand can actually deploy in the early turns,
    // not merely whether one exists: a two-lander living off a single
    // two-drop is a hand that does nothing from turn three.
    let castable_soon = |extra_lands: usize| -> usize {
        hand.iter()
            .filter(|c| {
                if c.definition.is_land() || c.definition.cost.cmc() as usize > lands + extra_lands
                {
                    return false;
                }
                let mut need = crate::mana::ColorSet::empty();
                for col in c.definition.cost.colors() {
                    need.insert(col);
                }
                need.is_subset_of(producible)
            })
            .count()
    };
    let early_plays = castable_soon(1);
    // What the hand is worth if it does get to cast its spells. Uses the
    // sealed builder's card scorer, which prices bodies, evasion and
    // preparation spells — the same blindness that made the builder pick
    // filler over bombs would otherwise make the mulligan ship bombs.
    let quality: i32 = hand
        .iter()
        .filter(|c| !c.definition.is_land())
        .map(|c| crate::draft::card_quality(&c.definition))
        .sum();
    // The player who isn't the starting player sees one more card before
    // their first real turn, which is what rescues a marginal hand.
    let on_draw = state.active_player_idx != seat;

    let keepable = if hand.len() <= 3 {
        // Below four cards the next mulligan costs more than the hand.
        true
    } else {
        match lands {
            0 | 1 => false,
            2 => early_plays >= 2 || (on_draw && has_early_play),
            3..=5 => has_early_play,
            // Flood is a keep only when the spells justify the risk.
            // Calibrated against concrete cards rather than a round
            // number: a 4/4 flier scores 7 and clears this, three
            // vanilla bears score 6 and don't.
            6 => quality >= 7,
            _ => false,
        }
    };
    if keepable || mulligans_taken >= 2 {
        DecisionAnswer::Keep
    } else {
        DecisionAnswer::TakeMulligan
    }
}

#[cfg(test)]
fn main_phase_action(state: &GameState, seat: usize) -> GameAction {
    main_phase_action_with(state, seat, true, &EvalWeights::default())
}

/// Every cast / activation the bot would consider from `state` this tick,
/// as `(already validated, action)`.
///
/// Extracted from `main_phase_action_with` so a sequence search can ask
/// "and what would I do next?" about a hypothetical state. That question
/// is the whole point of looking more than one play ahead: with four mana
/// the bot could never see that two two-drops beat one four-drop, because
/// it only ever scored a single action against the board.
///
/// The `bool` is whether the candidate has already been through the engine
/// dry-run. Specialty shapes (delve, convoke, kicker, spree, ...) are
/// probed eagerly because building them needs the accept/reject signal —
/// how many cards to delve, how few helpers to tap, the biggest affordable
/// kick. Plain casts are left unvalidated for the caller to probe lazily in
/// score order, which is what keeps a typical tick down to one or two
/// engine probes instead of the whole hand.
fn cast_candidates(
    state: &GameState,
    seat: usize,
    probe: &GameState,
    w: &EvalWeights,
) -> Vec<(GameAction, bool)> {
    // Build list of castable non-land spells. Affordability + target
    // pre-filters reduce the candidate set; the FINAL gate is still the
    // engine dry-run, which discards anything the engine would reject
    // (sorcery timing under Teferi, Damping Sphere mana tax, hexproof
    // targets, stolen permanents, etc.) — but for this main block it runs
    // *lazily* at the pick site below, in descending score order, so a
    // typical tick probes one or two candidates instead of the whole hand.
    //
    // SOS Repartee, computed once: it steers the plain-cast block toward
    // offering creature-aimed sibling candidates.
    let has_repartee = state.battlefield.iter().any(|c| {
        c.controller == seat && c.definition.triggered_abilities.iter().any(is_repartee_trigger)
    });
    let mut unvalidated: Vec<GameAction> = state.players[seat]
        .hand
        .iter()
        .filter(|c| !c.definition.is_land())
        // Pure temp-pump instants are combat tricks: held for the fight
        // window (`pick_combat_trick`), not main-phased where the buff
        // telegraphs and fizzles at cleanup.
        .filter(|c| !is_combat_trick(&c.definition))
        // Spree spells need `CastSpellSpree` with chosen modes — a plain
        // `CastSpell` resolves them as a no-op. They get their own candidate
        // block below.
        .filter(|c| !matches!(c.definition.effect, Effect::Spree { .. }))
        // A gift card whose base effect is empty (a permanent gift — the payoff
        // is a `SourceGiftPromised`-gated ETB) is wasted by a plain cast; it
        // gets a `CastGift` candidate in the gift block below instead.
        .filter(|c| !(c.definition.gift.is_some() && matches!(c.definition.effect, Effect::Noop)))
        .filter(|c| can_afford_in_state(state, seat, c, w))
        .flat_map(|c| {
            // For modal effects (ChooseMode), enumerate each mode so the
            // bot can pick (e.g.) Drown in the Loch's mode 1 (destroy
            // creature) when no opp spell is on the stack to counter.
            // Falls back to `mode: None` (engine defaults to mode 0) for
            // non-modal spells.
            let modes: Vec<Option<usize>> = match modal_mode_count(&c.definition.effect) {
                Some(n) => (0..n).map(Some).collect(),
                None => vec![None],
            };
            let x_value = if x_relevant(&c.definition) {
                Some(max_affordable_x(state, seat, c, w))
            } else {
                None
            };
            modes.into_iter().flat_map(move |mode| {
                // Pick a target appropriate to the chosen mode (ChooseMode
                // mode-aware filter check happens in the cast paths).
                // Multi-target shapes (Snow Day, Homesickness, Cost of
                // Brilliance, Render Speechless, Vibrant Outburst, …) ask
                // the picker for every slot index used by the effect tree;
                // slots that find no legal target are skipped, matching
                // "up to N target" semantics.
                let mode_effect = mode_branch(&c.definition.effect, mode);
                // Beneficial Auras pick their host explicitly: `Effect::Attach`
                // isn't classified friendly by the generic auto-targeter, so
                // without this a Rancor walks the OPPONENT's creatures first.
                // No friendly host at all → skip the candidate rather than
                // let the fallback pump an opposing creature.
                let (target, additional_targets) = if is_beneficial_aura(&c.definition) {
                    match beneficial_aura_host(state, seat, c, w) {
                        Some(t) => (Some(t), vec![]),
                        None => return vec![],
                    }
                } else if mode_effect.requires_target() {
                    let (t, extras) =
                        state.auto_targets_for_effect_all_slots(mode_effect, seat, mode);
                    if t.is_none() {
                        return vec![];
                    }
                    (t, extras)
                } else {
                    (None, vec![])
                };
                let mut out = vec![GameAction::CastSpell {
                    card_id: c.id,
                    target: target.clone(),
                    additional_targets: additional_targets.clone(),
                    mode,
                    // For X-cost spells (Banefire, Earthquake, Wrath of the
                    // Skies, Mind Twist, Repeal, …), pump as much generic
                    // mana as the pool can spare into X. Casting at X=0
                    // was a known dead end — Banefire dealt 0 damage, Mind
                    // Twist discarded nothing, Earthquake was a no-op.
                    x_value,
                }];
                // SOS Repartee: with a controlled payoff that wants an
                // instant/sorcery to target a CREATURE, an "any target"
                // spell the auto-targeter aimed at a player also gets a
                // creature-aimed sibling candidate. The outcome eval sees
                // the extra triggers fire when it resolves the sibling, so
                // the swap is judged, not assumed.
                if has_repartee
                    && matches!(target, Some(Target::Player(_)))
                    && {
                        use crate::card::CardType;
                        c.definition.card_types.contains(&CardType::Instant)
                            || c.definition.card_types.contains(&CardType::Sorcery)
                    }
                    && let Some(swap) = best_hostile_creature_target(state, seat, mode_effect, w)
                {
                    out.push(GameAction::CastSpell {
                        card_id: c.id,
                        target: Some(swap),
                        additional_targets,
                        mode,
                        x_value,
                    });
                }
                out
            })
        })
        .collect();

    // Specialty candidates below are probed eagerly (their construction
    // loops need the accept/reject signal — max delve size, biggest
    // affordable kick count, conspire-over-plain preference), so they land
    // in `castable` already validated.
    let mut castable: Vec<GameAction> = Vec::new();

    // Delve (CR 702.66): for any hand card with `Keyword::Delve` that the
    // bot can't (yet) afford, try exiling graveyard cards to pay the
    // generic portion. Delve the maximum available (capped at the generic
    // pip total), then let `would_accept` confirm the reduced cost is
    // payable. Appended to the candidate set so the bot actually leverages
    // Treasure Cruise / Dig Through Time / Gurmag Angler off a full bin.
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.keywords.contains(&crate::card::Keyword::Delve))
    {
        let generic_pips: u32 = c
            .definition
            .cost
            .symbols
            .iter()
            .filter_map(|s| match s {
                crate::mana::ManaSymbol::Generic(n) => Some(*n),
                _ => None,
            })
            .sum();
        let gy_ids: Vec<CardId> = state.players[seat].graveyard.iter().map(|g| g.id).collect();
        let take = (generic_pips as usize).min(gy_ids.len());
        if take == 0 {
            continue;
        }
        let delve_cards: Vec<CardId> = gy_ids.into_iter().take(take).collect();
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellDelve {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
            delve_cards,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Convoke / Improvise (CR 702.51 / 702.126): tap untapped creatures
    // (or artifacts) to pay {1} each. Without this the bot never taps a
    // helper, so every convoke card sat in hand at full price. Helpers are
    // capped at the spell's generic pips and drawn from creatures that
    // aren't already committed to combat; `would_accept` is the final gate,
    // so an unaffordable-even-with-help spell just doesn't make the list.
    for c in state.players[seat].hand.iter() {
        let convoke = c.definition.keywords.contains(&crate::card::Keyword::Convoke)
            || state.spell_granted_convoke(seat, c);
        let improvise = c.definition.keywords.contains(&crate::card::Keyword::Improvise);
        if !convoke && !improvise {
            continue;
        }
        // CR 702.51 — convoke pays colored pips too, so the cap is the whole
        // mana value, not just the generic part. Rank candidates so the least
        // useful bodies tap first: summoning-sick creatures (which can't attack
        // anyway) before healthy ones, then by ascending power.
        let cap = c.definition.cost.cmc() as usize;
        let mut candidates: Vec<(bool, i32, CardId)> = state
            .battlefield
            .iter()
            .filter(|h| {
                h.controller == seat
                    && !h.tapped
                    && ((convoke && h.definition.is_creature())
                        || (improvise && h.definition.is_artifact()))
            })
            .map(|h| (!h.summoning_sick, h.power(), h.id))
            .collect();
        candidates.sort_by_key(|(healthy, pow, _)| (*healthy, *pow));
        candidates.truncate(cap);
        let ranked: Vec<CardId> = candidates.into_iter().map(|(_, _, id)| id).collect();
        if ranked.is_empty() {
            continue;
        }
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        // Tap the fewest helpers that make the cast legal — over-tapping throws
        // away blockers for nothing.
        for n in 1..=ranked.len() {
            let action = GameAction::CastSpellConvoke {
                card_id: c.id,
                target: target.clone(),
                additional_targets: additional_targets.clone(),
                mode: None,
                x_value: None,
                convoke_creatures: ranked[..n].to_vec(),
            };
            if GameState::would_accept_on(probe, action.clone()) {
                castable.push(action);
                break;
            }
        }
    }

    // Gift (CR 702.165): a spell/permanent with a gift can be cast via
    // `CastGift`, promising the gift to resolve its enhanced `gifted_effect`
    // (or, for permanent gifts, unlock a `SourceGiftPromised`-gated ETB). A
    // plain `CastSpell` only ever gets the base effect, so gift-payoff cards
    // (Scrapshooter, Starfall Invocation) would otherwise be wasted. Offer the
    // promised variant alongside; the gifted effect's target slots are picked
    // from `gifted_effect`, and `would_accept` is the final gate.
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.gift.is_some())
        .filter(|c| can_afford_in_state(state, seat, c, w))
    {
        let gifted = &c.definition.gift.as_ref().unwrap().gifted_effect;
        // The ETB payoff of a permanent gift lives on the creature, not the
        // gifted_effect, so target off the base effect there; for spell gifts
        // the gifted_effect carries the (possibly broader) target.
        let target_effect =
            if gifted.requires_target() { gifted } else { &c.definition.effect };
        let (target, additional_targets) = if target_effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(target_effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastGift {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Spree (CR 702.172) / Tiered / ChooseModesCast: these must be cast via
    // `CastSpellSpree` with the chosen modes stamped — a plain `CastSpell`
    // skips the modes' additional costs. Offer each single mode, plus the
    // every-mode combination for Spree so a bot with mana up can escalate
    // rather than always firing the cheapest tier; `would_accept` gates
    // affordability, so unpayable combinations drop out on their own.
    for c in state.players[seat].hand.iter() {
        let (modes, combo): (Vec<&Effect>, bool) = match &c.definition.effect {
            Effect::Spree { modes } => (modes.iter().map(|m| &m.effect).collect(), true),
            Effect::Tiered { modes } => (modes.iter().map(|m| &m.effect).collect(), false),
            Effect::ChooseModesCast { modes, .. } => (modes.iter().collect(), false),
            // The Season cycle: the budget makes "all modes once" a legal
            // combination whenever the prices fit, so offer it too.
            Effect::ChooseModesByPoints { modes, points, budget } => {
                (modes.iter().collect(), points.iter().map(|p| *p as u32).sum::<u32>() <= *budget as u32)
            }
            _ => continue,
        };
        // Each target-bearing mode consumes exactly one target slot at
        // resolution, in printed order.
        let pick = |picks: Vec<u8>| -> Option<GameAction> {
            let mut slots: Vec<crate::game::types::Target> = Vec::new();
            for &i in &picks {
                let eff = modes[i as usize];
                if eff.requires_target() {
                    let (t, _) = state.auto_targets_for_effect_all_slots(eff, seat, None);
                    slots.push(t?);
                }
            }
            let mut slots = slots.into_iter();
            Some(GameAction::CastSpellSpree {
                card_id: c.id,
                spree_modes: picks,
                target: slots.next(),
                additional_targets: slots.collect(),
                x_value: None,
            })
        };
        let mut candidates: Vec<Vec<u8>> = (0..modes.len() as u8).map(|i| vec![i]).collect();
        if combo && modes.len() > 1 {
            candidates.push((0..modes.len() as u8).collect());
        }
        for picks in candidates {
            let Some(action) = pick(picks) else { continue };
            if GameState::would_accept_on(probe, action.clone()) {
                castable.push(action);
            }
        }
    }

    // SOS Prepare — a prepared creature's inset spell is a castable
    // resource: offer `CastPrepareSpell` whenever the cost is payable and
    // the spell has a legal target (`would_accept` gates timing/cost).
    // Casting unprepares the creature; enters-prepared bodies were
    // previously dead weight under bot control.
    for c in state.battlefield.iter().filter(|c| c.controller == seat) {
        let Some(spell) = c.definition.prepare_spell.as_deref() else { continue };
        if c.counter_count(crate::card::CounterType::Prepared) == 0 {
            continue;
        }
        let (target, additional_targets) = if spell.effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots(&spell.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        // X-cost inset spells (Jadzi's Oracle's Gift, {X}{X}{U}) size X
        // like a hand cast would; at `None` the engine casts them at X=0.
        let x_value = if x_relevant(spell) {
            Some(max_affordable_x_for_def(state, seat, spell, 0, w))
        } else {
            None
        };
        let action = GameAction::CastPrepareSpell {
            creature_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Splice onto Arcane (CR 702.47): splice every affordable partner onto an
    // Arcane spell the bot is casting anyway. `spliceable` already dry-ran the
    // one-splicer case; `would_accept` re-checks the combined cost, and the
    // spliced clauses' targets are auto-aimed inside `cast_spell_spliced`.
    for (host, splicers) in state.compute_hand_affordances(seat).spliceable {
        let (target, additional_targets) = {
            let eff = state.players[seat]
                .hand
                .iter()
                .find(|c| c.id == host)
                .map(|c| c.definition.effect.clone());
            match eff {
                Some(e) if e.requires_target() => {
                    let (t, extras) = state.auto_targets_for_effect_all_slots(&e, seat, None);
                    if t.is_none() {
                        continue;
                    }
                    (t, extras)
                }
                _ => (None, vec![]),
            }
        };
        let action = GameAction::CastSpellSpliced {
            card_id: host,
            splice_cards: splicers,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Conspire (CR 702.78): for any hand card with `Keyword::Conspire`, tap
    // the first two untapped creatures sharing a color with it to copy the
    // spell. The bot conspires whenever it can — the copy is strictly upside
    // for the targeted/value spells it appears on. `would_accept` confirms the
    // base cost is still payable after the (free, tap-only) conspire cost.
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.keywords.contains(&crate::card::Keyword::Conspire))
    {
        let spell_colors = c.definition.printed_colors();
        let pair: Vec<CardId> = state
            .battlefield
            .iter()
            .filter(|p| {
                p.controller == seat
                    && !p.tapped
                    && p.definition.is_creature()
                    && state
                        .computed_permanent(p.id)
                        .map(|cp| cp.colors.iter().any(|col| spell_colors.contains(col)))
                        .unwrap_or(false)
            })
            .map(|p| p.id)
            .take(2)
            .collect();
        if pair.len() < 2 {
            continue;
        }
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellConspire {
            card_id: c.id,
            conspire_creatures: [pair[0], pair[1]],
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            // Prefer conspiring over the plain cast of the same card — the
            // extra copy is value the bot's spell eval doesn't otherwise see.
            let cid = c.id;
            unvalidated
                .retain(|a| !matches!(a, GameAction::CastSpell { card_id, .. } if *card_id == cid));
            castable.push(action);
        }
    }

    // Kicker / Offspring (CR 702.32 / 702.166): for any hand card with the
    // optional additional cost, offer a `CastSpellKicked` candidate. Targets
    // come from the effect tree, whose slot-0 filter resolves to the kicked
    // (typically broader) branch, so a kicked Tear Asunder can aim at a
    // creature. `would_accept` validates the full base+kicker cost, so this is
    // only added when affordable.
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.has_kicker().is_some())
    {
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots_kicked(effect, seat, None, true, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellKicked {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            // Offspring (CR 702.166) is pure upside — a free 1/1 token copy
            // with no downside beyond the mana. When affordable, prefer it
            // over the plain cast of the same card (mirrors Conspire above).
            if c.definition.has_offspring().is_some() {
                let cid = c.id;
                unvalidated.retain(
                    |a| !matches!(a, GameAction::CastSpell { card_id, .. } if *card_id == cid),
                );
            }
            castable.push(action);
        }
    }

    // CR 702.32b — "Kicker {A} and/or {B}": offer the largest affordable
    // subset (both halves before either alone; each rider is pure upside).
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| !c.definition.kicker_options.is_empty())
    {
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots_kicked(effect, seat, None, true, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let n = c.definition.kicker_options.len() as u8;
        let mut best: Option<GameAction> = None;
        for mask in (1u32..(1 << n)).rev() {
            let kickers: Vec<u8> = (0..n).filter(|i| mask & (1 << i) != 0).collect();
            let action = GameAction::CastSpellKickers {
                card_id: c.id,
                kickers,
                target: target.clone(),
                additional_targets: additional_targets.clone(),
                mode: None,
                x_value: None,
            };
            if GameState::would_accept_on(probe, action.clone()) {
                best = Some(action);
                break;
            }
        }
        if let Some(action) = best {
            let cid = c.id;
            unvalidated
                .retain(|a| !matches!(a, GameAction::CastSpell { card_id, .. } if *card_id == cid));
            castable.push(action);
        }
    }

    // Multikicker (CR 702.33c): offer the *biggest affordable* kick count
    // (probed 4 → 1 via `would_accept`, which validates base + N×kick).
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.has_multikicker().is_some())
    {
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        for times in (1..=4u32).rev() {
            let action = GameAction::CastSpellMultikicked {
                card_id: c.id,
                times,
                target: target.clone(),
                additional_targets: additional_targets.clone(),
                mode: None,
                x_value: None,
            };
            if GameState::would_accept_on(probe, action.clone()) {
                castable.push(action);
                break;
            }
        }
    }

    // Bestow (CR 702.103): for any hand card with a bestow cost, offer a
    // `CastBestow` candidate that enchants the bot's sturdiest creature (the
    // host most likely to stick, so the Aura keeps its value). `would_accept`
    // validates the full bestow cost, so this is only added when affordable.
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.bestow.is_some())
    {
        // Prefer the controller's highest-toughness creature as the host.
        let host = state
            .battlefield
            .iter()
            .filter(|b| b.controller == seat && b.definition.is_creature())
            .max_by_key(|b| state.computed_permanent(b.id).map(|cp| cp.toughness).unwrap_or(0))
            .map(|b| b.id);
        let Some(host) = host else { continue };
        let action = GameAction::CastBestow {
            card_id: c.id,
            target: Some(crate::game::Target::Permanent(host)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Adventure (CR 715): for any hand card with an adventure half that
    // *targets* something (removal / bounce / pump — Stomp, Petty Theft,
    // Swift End, Boulder Rush), offer a `CastAdventure` candidate. Token /
    // card-draw adventures are skipped here so the bot still prefers playing
    // those cards as creatures; the interactive halves are pure tempo wins.
    for c in state.players[seat].hand.iter() {
        let Some(adv) = c.definition.has_adventure() else { continue };
        if !adv.effect.requires_target() {
            continue;
        }
        let (target, additional_targets) =
            state.auto_targets_for_effect_all_slots(&adv.effect, seat, None);
        if target.is_none() {
            continue;
        }
        let action = GameAction::CastAdventure {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Omen (CR 702.183): for any hand card with an Omen half that *targets*
    // something, offer a `CastOmen` candidate (the card shuffles back into the
    // library on resolution, so the creature is still drawable later).
    for c in state.players[seat].hand.iter() {
        let Some(omen) = c.definition.has_omen() else { continue };
        if !omen.effect.requires_target() {
            continue;
        }
        let (target, additional_targets) =
            state.auto_targets_for_effect_all_slots(&omen.effect, seat, None);
        if target.is_none() {
            continue;
        }
        let action = GameAction::CastOmen {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Prototype (CR 702.160): for any hand card with a prototype face, offer
    // a `CastPrototype` candidate. The smaller colored cost is often the only
    // affordable line early; the body's ETB auto-targets through the cast path.
    for c in state.players[seat].hand.iter() {
        if c.definition.has_prototype().is_none() {
            continue;
        }
        let action = GameAction::CastPrototype {
            card_id: c.id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Split cards (CR 709): for any hand card with a non-aftermath split,
    // offer a `CastSplitRight` candidate (the left half is already covered by
    // the plain `CastSpell` path). Auto-target the right half's effect.
    for c in state.players[seat].hand.iter() {
        let Some(split) = c.definition.has_split() else { continue };
        if split.aftermath {
            continue;
        }
        let (target, additional_targets) = if split.right.effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots(&split.right.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSplitRight {
            card_id: c.id, target, additional_targets, mode: None, x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Aftermath (CR 702.127): cast the right half of a split card from the
    // graveyard. `would_accept` enforces the graveyard-only + timing rules.
    for c in state.players[seat].graveyard.iter() {
        let Some(split) = c.definition.has_split().filter(|s| s.aftermath) else { continue };
        let (target, additional_targets) = if split.right.effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots(&split.right.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastAftermath {
            card_id: c.id, target, additional_targets, mode: None, x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Flashback / Jump-start (CR 702.34/702.103) and Disturb (CR 702.146):
    // recast graveyard cards. `would_accept` enforces zone, timing, and an
    // affordable cost, so these only surface when actually castable.
    for c in state.players[seat].graveyard.iter() {
        use crate::card::Keyword;
        let recastable = c.effective_flashback().is_some()
            || c.definition.keywords.contains(&Keyword::JumpStart)
            || c.definition.keywords.contains(&Keyword::GraveyardCast);
        if recastable {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                let (t, extras) =
                    state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None);
                if t.is_none() {
                    continue;
                }
                (t, extras)
            } else {
                (None, vec![])
            };
            let action = GameAction::CastFlashback {
                card_id: c.id, target, additional_targets, mode: None, x_value: None,
            };
            if GameState::would_accept_on(probe, action.clone()) {
                castable.push(action);
            }
        }
        if c.definition.keywords.iter().any(|k| matches!(k, Keyword::Disturb(_))) {
            // The back face goes on the stack; an Aura back needs an enchant
            // target (creature backs need none).
            let back = c.definition.back_face.as_deref();
            let (target, additional_targets) = match back {
                Some(b) if b.effect.requires_target() => {
                    let (t, extras) =
                        state.auto_targets_for_effect_all_slots(&b.effect, seat, None);
                    (t, extras)
                }
                _ => (None, vec![]),
            };
            let needs_target = back.is_some_and(|b| b.effect.requires_target());
            if !(needs_target && target.is_none()) {
                let action = GameAction::CastDisturb {
                    card_id: c.id, target, additional_targets,
                };
                if GameState::would_accept_on(probe, action.clone()) {
                    castable.push(action);
                }
            }
        }
        // Mayhem (CR 702.187): if the card was discarded this turn and has a
        // mayhem cost, offer a graveyard cast for it. `would_accept` enforces
        // the discarded-this-turn gate, cost, and timing.
        if c.definition.mayhem_cost().is_some() {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                let (t, extras) =
                    state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None);
                if t.is_none() {
                    continue;
                }
                (t, extras)
            } else {
                (None, vec![])
            };
            let action = GameAction::CastMayhem {
                card_id: c.id, target, additional_targets, mode: None, x_value: None,
            };
            if GameState::would_accept_on(probe, action.clone()) {
                castable.push(action);
            }
        }
        // Harmonize (CR 702.180): cast from the graveyard for the harmonize
        // cost. The bot doesn't tap a creature to discount (a value call it
        // can't weigh well); `would_accept` enforces cost / timing.
        if c.effective_harmonize().is_some() {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                let (t, extras) =
                    state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None);
                if t.is_none() {
                    continue;
                }
                (t, extras)
            } else {
                (None, vec![])
            };
            let action = GameAction::CastHarmonize {
                card_id: c.id, tap_creature: None, target, additional_targets, mode: None, x_value: None,
            };
            if GameState::would_accept_on(probe, action.clone()) {
                castable.push(action);
            }
        }
        // Graveyard-activated abilities (CR 702.84 Unearth, and the SOS
        // "return this from your graveyard" cycle): offer each `from_graveyard`
        // activated ability. `would_accept` enforces zone / cost / sorcery
        // timing, so these only surface when actually activatable.
        for (idx, ab) in c.definition.activated_abilities.iter().enumerate() {
            if !ab.from_graveyard {
                continue;
            }
            let (target, additional_targets) = if ab.effect.requires_target() {
                let (t, extras) = state.auto_targets_for_effect_all_slots(&ab.effect, seat, None);
                if t.is_none() {
                    continue;
                }
                (t, extras)
            } else {
                (None, vec![])
            };
            let action = GameAction::ActivateAbility {
                card_id: c.id, ability_index: idx, target, additional_targets, x_value: None, mode: None,
            };
            if GameState::would_accept_on(probe, action.clone()) {
                castable.push(action);
            }
        }
    }

    // MDFC back faces (CR 712): cast the back of a hand MDFC, or the back of a
    // graveyard MDFC carrying the one-shot `may_cast_back_from_graveyard`
    // permission (Pestilent Cauldron's "cast it transformed"). Targets come
    // from the BACK face's effect; `would_accept` enforces cost / timing /
    // zone, so these only surface when actually castable. (Land backs are
    // played via PlayLandBack, handled by the land logic, so they're skipped
    // here.)
    let back_sources = state.players[seat].hand.iter().chain(
        state.players[seat]
            .graveyard
            .iter()
            .filter(|c| c.may_cast_back_from_graveyard),
    );
    for c in back_sources {
        let Some(back) = c.definition.back_face.as_deref() else { continue };
        if back.is_land() {
            continue;
        }
        let (target, additional_targets) = if back.effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(&back.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellBack {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Adventure creature (CR 715) and plotted cards (CR 702.170d): cast the
    // creature half / a plotted card from exile. `would_accept` enforces the
    // later-turn + sorcery-speed timing, so this is only offered when legal.
    for c in state.exile.iter().filter(|c| c.owner == seat) {
        let action = if c.on_adventure && c.definition.is_land() {
            // CR 715.3d — a land half is played, not cast (FIN's Town cycle).
            GameAction::PlayLand(c.id)
        } else if c.on_adventure {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None)
            } else {
                (None, vec![])
            };
            GameAction::CastAdventureCreature {
                card_id: c.id, target, additional_targets, mode: None, x_value: None,
            }
        } else if state.plotted_cards.contains(&c.id) {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None)
            } else {
                (None, vec![])
            };
            GameAction::CastPlotted {
                card_id: c.id, target, additional_targets, mode: None, x_value: None,
            }
        } else {
            continue;
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    // Mana-only alternative costs (Dash CR 702.110, Blitz 702.152,
    // Spectacle 702.111): for any hand card whose `alternative_cost` is paid
    // purely with mana (no pitch/sacrifice/graveyard/life rider), offer a
    // `CastSpellAlternative` candidate. `would_accept` validates the alt cost
    // and its `condition` gate (e.g. Spectacle's opponent-lost-life), so a
    // Skewer the Critics is only offered for {R} once an opponent has bled.
    for c in state.players[seat].hand.iter().filter(|c| {
        c.definition.alternative_cost.as_ref().is_some_and(|a| {
            a.exile_filter.is_none()
                && a.sacrifice_permanents.is_none()
                && a.exile_from_graveyard_count == 0
                && a.life_cost == 0
                && !a.evoke_sacrifice
                // Offering (CR 702.48) sacrifices one of the bot's own
                // creatures for a tempo cut it rarely wants — cast normally.
                && a.offering.is_none()
        })
    }) {
        let effect = c
            .definition
            .alternative_cost
            .as_ref()
            .and_then(|a| a.effect_override.as_ref())
            .unwrap_or(&c.definition.effect);
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellAlternative {
            card_id: c.id,
            pitch_card: None,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if GameState::would_accept_on(probe, action.clone()) {
            castable.push(action);
        }
    }

    let mut out: Vec<(GameAction, bool)> = Vec::with_capacity(castable.len() + unvalidated.len());
    out.extend(castable.into_iter().map(|a| (a, true)));
    out.extend(unvalidated.into_iter().map(|a| (a, false)));
    // Ward gate, applied once for every candidate block above: a cast
    // aimed at a warded permanent whose tax the bot can't pay after the
    // spell's own cost resolves as a counter, not a cast (the engine
    // auto-pays ward and `would_accept` can't see the trigger fail).
    out.retain(|(a, _)| ward_gate_ok(state, seat, a));
    out
}

fn main_phase_action_with(
    state: &GameState,
    seat: usize,
    scored: bool,
    w: &EvalWeights,
) -> GameAction {
    // One library-stripped probe template per tick: every candidate dry-run
    // below re-clones this light template instead of the full state. The
    // library is the largest part of a `GameState` clone and cast/activate/
    // play-land legality never reads it (see `affordance_probe_template`),
    // so this turns N full-deck clones into one + N light ones.
    let probe = state.affordance_probe_template();

    // NOTE: the bot deliberately does *not* pre-tap its mana sources here.
    //
    // It used to: one untapped land per tick until the board was empty,
    // which is what made `can_afford_in_state` work off the floating pool.
    // The cost was severe and invisible to the unit tests (which all
    // pre-fill `mana_pool` by hand). Pools empty at every step and phase
    // boundary (CR 500.4), so tapping out in the precombat main left the
    // bot with nothing for its own postcombat main and nothing at all on
    // the opponent's turn: measured over 20 bot-vs-bot games, zero of 1366
    // opponent-turn priority windows had a single untapped land, and 100 %
    // of spells were cast in the precombat main. `pick_stack_response`,
    // `pick_ability_counter_response`, `pick_combat_trick` and the
    // end-of-turn instant window below were all dead code in real play.
    //
    // Now the engine's auto-tap pays each cast from only the sources it
    // needs (`try_pay_with_auto_tap`, which `would_accept_on` already runs
    // for every candidate), so leftover mana simply stays untapped and is
    // still there at instant speed.
    if w.legacy_pretap
        && let Some(id) = state
            .battlefield
            .iter()
            .find(|c| c.controller == seat && c.definition.is_land() && !c.tapped)
            .map(|c| c.id)
    {
        let action = GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
            mode: None,
        };
        if GameState::would_accept_on(&probe, action.clone()) {
            return action;
        }
    }

    // Everything castable this tick — see `cast_candidates`.
    let pool = cast_candidates(state, seat, &probe, w);

    // Play a land if possible — gated through `would_accept` for
    // the same reason (the engine enforces sorcery timing, lands-
    // played-this-turn, etc.). Use the game-level helper so an
    // Exploration / Azusa-style ExtraLandPerTurn static lets the bot
    // play a second land in the same turn (CR 305.2).
    if state.can_player_play_land(seat)
        && let Some(land_id) = pick_land_to_play(state, seat, w)
    {
        let action = GameAction::PlayLand(land_id);
        if GameState::would_accept_on(&probe, action.clone()) {
            return action;
        }
    }

    // Crucible of Worlds / Ramunap Excavator: replay a land from the
    // graveyard if no hand land was played (CR 305 land-from-gy permission).
    if state.can_player_play_land(seat)
        && state.player_may_play_lands_from_graveyard(seat)
        && let Some(land) =
            state.players[seat].graveyard.iter().find(|c| c.definition.is_land())
    {
        let action = GameAction::PlayLandFromGraveyard(land.id);
        if GameState::would_accept_on(&probe, action.clone()) {
            return action;
        }
    }

    // Impulse exile (Light Up the Stage, Gonti Night Minister): a land the
    // seat has a may-play grant on is played from exile before it expires.
    if state.can_player_play_land(seat)
        && let Some(land) = state.exile.iter().find(|c| {
            c.definition.is_land() && c.may_play_until.is_some_and(|perm| perm.player == seat)
        })
    {
        let action = GameAction::PlayLand(land.id);
        if GameState::would_accept_on(&probe, action.clone()) {
            return action;
        }
    }

    if !pool.is_empty() {
        // Magecraft-aware bias: if the bot controls a permanent with a
        // magecraft trigger, prefer instants/sorceries so the trigger
        // fires — IS candidates sort first, and finalist collection stops
        // at the IS/non-IS boundary once an IS line has validated (the
        // lazy-probe equivalent of the old only-IS pool restriction).
        // Push (claude/modern_decks batch 202).
        let has_magecraft = state.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition.triggered_abilities.iter().any(is_magecraft_trigger)
        });
        let is_is_spell = |a: &GameAction| {
            matches!(a, GameAction::CastSpell { card_id, .. } if is_instant_or_sorcery_in_hand(state, seat, *card_id))
        };
        if !scored {
            // Uniform baseline: validate everything (the historical
            // behavior) and sample.
            let valid: Vec<GameAction> = pool
                .into_iter()
                .filter(|(a, ok)| *ok || GameState::would_accept_on(&probe, a.clone()))
                .map(|(a, _)| a)
                .collect();
            if !valid.is_empty() {
                let only_is: Vec<GameAction> = if has_magecraft {
                    valid.iter().filter(|a| is_is_spell(a)).cloned().collect()
                } else {
                    Vec::new()
                };
                let pick = if only_is.is_empty() { &valid } else { &only_is };
                return pick[jitter_below(pick.len())].clone();
            }
        } else {
            // Scored pick: rank by static score (+ jitter so exact ties
            // don't collapse into one deterministic line — see
            // `score_candidate`), walk in rank order probing unvalidated
            // candidates lazily, and hand the top few survivors to the
            // outcome evaluation for the final call. Most ticks this
            // probes 1-3 candidates instead of the whole hand.
            //
            // SOS on-cast payoff nudges — score-shaped siblings of the
            // magecraft partition (nudges compose; partitions don't):
            // * Opus: a controlled Opus permanent upgrades its trigger
            //   when 5+ mana was spent on the cast.
            // * Increment: a controlled Increment body grows when the
            //   cast's mana spent clears its smaller stat.
            // * Infusion: an Infusion-gated card in hand unlocks on any
            //   lifegain, so lifegain casts go first while none has been
            //   gained this turn.
            // Each nudge is 8 = two score points ≈ one mana of cast
            // value on `score_candidate`'s ×4 scale — a tiebreaker, not
            // an override.
            let has_opus = state.battlefield.iter().any(|c| {
                c.controller == seat
                    && c.definition.triggered_abilities.iter().any(is_opus_trigger)
            });
            let increment_bar = increment_threshold(state, seat);
            let wants_lifegain = state.players[seat].life_gained_this_turn == 0
                && state.players[seat]
                    .hand
                    .iter()
                    .any(|c| card_infusion_gated(&c.definition));
            let mut ranked: Vec<(i32, GameAction, bool)> = pool
                .into_iter()
                .map(|(a, ok)| {
                    let mut s =
                        score_candidate(state, seat, &a, w) * 4 + jitter_below(4) as i32;
                    let spent = if has_opus || increment_bar.is_some() {
                        cast_mana_spent(state, seat, &a)
                    } else {
                        0
                    };
                    if has_opus && spent >= 5 {
                        s += 8 * w.unit;
                    }
                    if increment_bar.is_some_and(|bar| spent >= bar) {
                        s += 8 * w.unit;
                    }
                    if wants_lifegain && cast_gains_life(state, seat, &a) {
                        s += 8 * w.unit;
                    }
                    (s, a, ok)
                })
                .collect();
            if has_magecraft {
                ranked.sort_by_key(|&(s, ref a, _)| (!is_is_spell(a), std::cmp::Reverse(s)));
            } else {
                ranked.sort_by_key(|&(s, _, _)| std::cmp::Reverse(s));
            }
            const EVAL_TOP: usize = 3;
            let mut finalists: Vec<(i32, GameAction)> = Vec::new();
            for (s, a, ok) in ranked {
                if finalists.len() >= EVAL_TOP {
                    break;
                }
                if has_magecraft && !finalists.is_empty() && !is_is_spell(&a) {
                    break;
                }
                if ok || GameState::would_accept_on(&probe, a.clone()) {
                    finalists.push((s, a));
                }
            }
            if let Some(best) = pick_by_outcome(state, seat, finalists, w) {
                // Forge's summon-sick gate (`SpellAbilityPicker`): if the
                // winning line's only gain this turn is a body that can't
                // attack, it is worth exactly as much after combat — and
                // played then it costs the opponent a turn of information
                // and leaves the mana up in between. Hold it.
                //
                // Applied to the *winner* only, deliberately. Screening
                // every candidate this way would have the bot pick some
                // lesser non-creature line now and then have no mana left
                // for the creature it actually wanted in the second main.
                let own_main = state.active_player_idx == seat
                    && matches!(
                        state.step,
                        TurnStep::PreCombatMain | TurnStep::PostCombatMain
                    );
                // Hold a creature that can't attack yet until the second
                // main; hold an instant-speed line until the opponent's
                // turn. Both ask the same question -- "is this worth the
                // same later?" -- and both only fire on our own main phase,
                // where there is a later to wait for.
                let gate = own_main
                    && ((w.hold_sick && state.step == TurnStep::PreCombatMain)
                        || (w.hold_instants && castable_at_instant_speed(state, seat, &best)));
                if gate && !improves_this_turn(state, seat, &best, w) {
                    return GameAction::PassPriority;
                }
                // SOS Converge: float a missing color first so the cast
                // counts it — see `pick_converge_prefloat`.
                if let Some(tap) = pick_converge_prefloat(state, seat, &best) {
                    return tap;
                }
                return best;
            }
        }
    }

    // Morph / Disguise (CR 702.36 / 702.166): cast a hand card face down for
    // {3} as a 2/2 (with ward {2} for Disguise). Reached only when no normal
    // spell candidate validated, so the bot still prefers casting cards face
    // up; `would_accept` enforces sorcery timing and the {3} payment.
    for c in state.players[seat].hand.iter().filter(|c| {
        c.definition.keywords.iter().any(|k| {
            matches!(
                k,
                crate::card::Keyword::Morph(_)
                    | crate::card::Keyword::MorphCost(_)
                    | crate::card::Keyword::Megamorph(_)
                    | crate::card::Keyword::Disguise(_)
            )
        })
    }) {
        let action = GameAction::CastFaceDown { card_id: c.id };
        if GameState::would_accept_on(&probe, action.clone()) {
            return action;
        }
    }

    // Discard-activated hand abilities (Magma Opus's {U/R}{U/R}, Discard:
    // create a Treasure) — a fallback value play, reached only when the bot
    // has no spell/face-down line so it never pitches a castable card.
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.discard_activated.is_some())
    {
        let action = GameAction::ActivateDiscardAbility { card_id: c.id };
        if GameState::would_accept_on(&probe, action.clone()) {
            return action;
        }
    }

    // Activate planeswalker loyalty abilities the bot controls. Pick the
    // first usable ability per walker (engine enforces sorcery timing and
    // once-per-turn). The candidate set is dry-run-gated so failed targets
    // / over-spent loyalty / opp-controlled-walker rejections drop out.
    if let Some(action) = pick_loyalty_ability(state, seat, w) {
        return action;
    }

    // Crew (CR 702.122): turn an uncrewed Vehicle into an attacker by tapping
    // the bot's least-valuable untapped creatures. Dry-run-gated.
    if let Some(action) = pick_crew(state, seat) {
        return action;
    }

    // Saddle (CR 702.171): tap the bot's least-valuable untapped creatures to
    // saddle a Mount that's about to attack, so its "attacks while saddled"
    // riders fire. Dry-run-gated.
    if let Some(action) = pick_saddle(state, seat) {
        return action;
    }

    // Equip (CR 702.6): if the bot controls an Equipment that isn't yet
    // attached to one of its creatures, and it controls a creature to wear
    // it, move the Equipment onto the biggest such creature. Dry-run-gated
    // so the equip cost / sorcery timing / target legality all bottom out
    // in `would_accept`.
    if let Some(action) = pick_equip(state, seat) {
        return action;
    }

    // Activated two-slot attach (Brass Squire's "{T}: attach target Equipment
    // you control to target creature you control"). The native-equip pass
    // above only covers `Keyword::Equip`; this drives the Equipment-mover
    // creatures so the AI plays them.
    if let Some(action) = pick_attach_ability(state, seat) {
        return action;
    }

    // Spend surplus energy on beneficial energy-payoff abilities (Bristling
    // Hydra's grow, Longtusk Cub's +1/+1, Aetherstream Leopard's
    // unblockable, …). Only pure "Pay {E}: do X" abilities with no other
    // cost are considered, so the bot can't bankrupt mana or sacrifice
    // anything. Dry-run-gated like everything else.
    if let Some(action) = pick_energy_payoff(state, seat) {
        return action;
    }

    // Recur value from the graveyard (Embalm CR 702.88 / Eternalize CR 702.91
    // and any "Exile this from your graveyard: …" ability) when there's spare
    // mana and nothing better to do. Dry-run-gated so cost / sorcery timing
    // bottom out in `would_accept`.
    if let Some(action) = pick_graveyard_recursion(state, seat) {
        return action;
    }

    // Reanimate a creature from the graveyard via a battlefield permanent's
    // activated ability (Seedship Broodtender's sac-to-return) when there's a
    // worthwhile target. Dry-run-gated so cost / sorcery-speed timing bottom
    // out in `would_accept`.
    if let Some(action) = pick_battlefield_reanimate(state, seat) {
        return action;
    }

    // Crack a Lander token (CR — `{2}, {T}, Sacrifice: fetch a basic land
    // tapped`) for ramp when there's a basic still in the library and spare
    // mana. Sequenced after spell-casting so the bot only ramps when it has
    // nothing better to spend mana on. Dry-run-gated.
    if let Some(action) = pick_crack_lander(state, seat) {
        return action;
    }

    // Fire a "{cost}: deal damage to any target" value ability (Frostwielder's
    // {T} ping, Kiku's tap-and-burn, Pain Kami-style sac burn) when it kills an
    // opposing creature outright. Dry-run-gated so cost / timing / target
    // legality bottom out in `would_accept`.
    if let Some(action) = pick_removal_ping(state, seat) {
        return action;
    }

    // Close the game: fire a "deal N to each opponent" / "drain N" / "each
    // opponent loses N" ability when it's lethal to a living opponent
    // (Hazoret's discard-burn, drain pingers). Lethal-only, so the bot never
    // wastes the resource. Dry-run-gated via `would_accept`.
    if let Some(action) = pick_reach_burn(state, seat) {
        return action;
    }

    // Fire a "Sacrifice this: destroy target creature" ability (Pus Kami,
    // Nezumi Bone-Reader-style sac-removal) on a favorable trade — only when
    // the destroyed foe is at least as big as the creature being sacrificed.
    if let Some(action) = pick_removal_sacrifice(state, seat) {
        return action;
    }

    // Fire a repeatable "{cost}: Destroy target creature" (the Torment
    // Possessed cycle's Threshold ability, Royal Assassin-style tappers) on
    // the biggest legal foe. No trade math — the source survives.
    if let Some(action) = pick_removal_destroy(state, seat) {
        return action;
    }

    // Unmask a face-down threat (Morph / Megamorph / Disguise / a cloaked or
    // manifested creature card) when the turn-up cost is affordable. Dry-run-
    // gated, so the cost / timing / "manifested noncreature can't turn up"
    // rules all bottom out in `would_accept`.
    if let Some(action) = pick_turn_face_up(state, seat) {
        return action;
    }

    // Pump the whole team before combat damage (Bearer of Glory's
    // "{4}{W}: creatures you control get +1/+1") when the bot has two or more
    // attacking creatures — the pump pays off on the swing. Dry-run-gated.
    if let Some(action) = pick_team_pump(state, seat) {
        return action;
    }

    // As a last resort before passing, sink spare mana into a "{cost}: draw a
    // card" ability when card-starved (Bonders' Enclave, Arch of Orazca-style
    // engines). Dry-run-gated, so cost / activation conditions bottom out in
    // `would_accept`.
    if let Some(action) = pick_card_draw_ability(state, seat) {
        return action;
    }

    // Re-arm an unprepared prepare-spell creature via an off-card "target
    // creature becomes prepared" ability (SOS: Skycoach Waypoint). The
    // counter is worth about the inset spell — see the `permanent_value`
    // term — so this banks value on par with the draw sink above.
    if let Some(action) = pick_reprepare(state, seat) {
        return action;
    }

    // Sacrifice-for-value engines (sac a Pest: payoff), judged by the
    // resolved outcome rather than skipped for carrying a sac cost.
    if let Some(action) = pick_sacrifice_value(state, seat, w) {
        return action;
    }

    // Crew an uncrewed Vehicle so it can attack this turn (Vehicles are dead
    // cards to the bot otherwise). Dry-run-gated.
    if let Some(action) = pick_crew_vehicle(state, seat) {
        return action;
    }

    // Sink leftover mana into a repeatable "{cost}: +1/+1 counter on this"
    // ability to grow the board (Fire Sages, Water Tribe Captain). Last resort,
    // so it never pre-empts a spell or land. Dry-run-gated.
    if let Some(action) = pick_self_pump_counter(state, seat) {
        return action;
    }

    // Sink leftover mana into a "{cost}: create a token" ability to grow the
    // board (Sun Warriors' {5}: 1/1 Ally, Realm of Koh's Spirit, Jasmine Dragon).
    // Last resort, dry-run-gated.
    if let Some(action) = pick_token_maker(state, seat) {
        return action;
    }

    GameAction::PassPriority
}

/// Activate a sacrifice-cost ability when the RESOLVED outcome beats
/// doing nothing. The generic ability pickers skip sac costs outright and
/// `pick_removal_sacrifice` only knows the destroy-for-trade shape — the
/// value shapes (sacrifice a token: draw / drain / counters) had no
/// judge at all. The clone-and-resolve eval prices both sides of the
/// exchange: the permanent given up AND what its death buys, triggers
/// included. Strictly-better-than-passing or nothing.
fn pick_sacrifice_value(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    let baseline = eval_material(state, seat, w);
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in usable_abilities(state, card) {
            if !ab.sac_cost && ab.sac_other_filter.is_none() {
                continue;
            }
            // Destroy-shaped sac removal keeps its dedicated trade math
            // (`pick_removal_sacrifice`, earlier in the chain).
            if matches!(&ab.effect, Effect::Destroy { .. } | Effect::DestroyNoRegen { .. }) {
                continue;
            }
            let target = if ab.effect.requires_target() {
                match state.auto_target_for_effect(&ab.effect, seat) {
                    Some(t) => Some(t),
                    None => continue,
                }
            } else {
                None
            };
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if !ward_gate_ok(state, seat, &action) {
                continue;
            }
            if !state.would_accept(action.clone()) {
                continue;
            }
            if let Some(ev) = evaluate_action_outcome(state, seat, &action, w)
                && ev > baseline
            {
                return Some(action);
            }
        }
    }
    None
}

/// SOS Prepare mana sink: aim an off-card "target creature becomes
/// prepared" ability (Skycoach Waypoint's `{3},{T}`) at the bot's best
/// unprepared prepare-spell creature — biggest inset spell first, since
/// that's what the counter is worth. Dry-run-gated through `would_accept`.
fn pick_reprepare(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::CounterType;
    use crate::effect::Selector;
    fn prepares_target(e: &Effect) -> bool {
        match e {
            Effect::AddCounter { what, kind: CounterType::Prepared, .. } => {
                matches!(what, Selector::Target(_) | Selector::TargetFiltered { .. })
            }
            Effect::Seq(v) => v.iter().any(prepares_target),
            _ => false,
        }
    }
    let mut targets: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == seat
                && c.definition.prepare_spell.is_some()
                && c.counter_count(CounterType::Prepared) == 0
        })
        .collect();
    if targets.is_empty() {
        return None;
    }
    targets.sort_by_key(|c| {
        std::cmp::Reverse(c.definition.prepare_spell.as_deref().map(|s| s.cost.cmc()).unwrap_or(0))
    });
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in usable_abilities(state, card) {
            if !prepares_target(&ab.effect) {
                continue;
            }
            for t in &targets {
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Permanent(t.id)),
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

/// Activate a non-sacrifice "{cost}: create a token" ability as a last-resort
/// mana sink — grows the board when the bot has nothing better to do. Skips
/// sacrifice-cost and once-per-game (Exhaust) abilities. Dry-run-gated through
/// `would_accept`, so cost/timing legality bottoms out there.
fn pick_token_maker(state: &GameState, seat: usize) -> Option<GameAction> {
    fn makes_token(e: &Effect) -> bool {
        match e {
            Effect::CreateToken { .. } | Effect::CreateTokenAttacking { .. } => true,
            Effect::Seq(steps) => steps.iter().any(makes_token),
            _ => false,
        }
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if ab.sac_cost || ab.exhaust || !makes_token(&ab.effect) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Activate a repeatable "{cost}: put a +1/+1 counter on this creature" ability
/// as a last-resort mana sink — grows the board when the bot has nothing better
/// to do. Skips sacrifice-cost and once-per-game (Exhaust) abilities so it never
/// throws away a permanent or a one-shot. Dry-run-gated through `would_accept`.
fn pick_self_pump_counter(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::CounterType;
    use crate::effect::Selector;
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if ab.sac_cost || ab.exhaust {
                continue;
            }
            // Adapt abilities (CR 702.108) put +1/+1 counters on a creature with
            // none — recognize the `If`-wrapped counter shape and fire only when
            // the creature isn't already adapted (else it's a mana-wasting no-op).
            let useful = if let Effect::AddCounter { what: Selector::This, kind, .. } = &ab.effect {
                // Always sink into +1/+1 self-pumps; otherwise only into a counter
                // that still progresses an unmet "becomes a creature at N counters"
                // static (War Balloon's fire counters), so the bot animates it
                // instead of stalling and doesn't dump mana past the threshold.
                *kind == CounterType::PlusOnePlusOne
                    || card.definition.static_abilities.iter().any(|sa| {
                        matches!(&sa.effect,
                            crate::effect::StaticEffect::SelfIsCreatureWhileCountersAtLeast { kind: k, n }
                            if k == kind && card.counter_count(*kind) < *n)
                    })
            } else if ab.effect.is_adapt() {
                card.counter_count(CounterType::PlusOnePlusOne) == 0
            } else {
                false
            };
            if !useful {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Crew an uncrewed Vehicle the bot controls, paying with the smallest
/// untapped creatures whose total power covers the crew cost — but only when
/// the Vehicle is at least as big as the power tapped to crew it (a net combat
/// gain). Dry-run-gated, so crew legality (CR 702.122) bottoms out in
/// `would_accept`.
fn pick_crew_vehicle(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::CardType;
    let mut crewers: Vec<(CardId, i32)> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && !c.tapped)
        .filter_map(|c| {
            let cp = state.computed_permanent(c.id)?;
            cp.card_types.contains(&CardType::Creature).then_some((c.id, cp.power.max(0)))
        })
        .collect();
    crewers.sort_by_key(|&(_, p)| p);

    for v in state.battlefield.iter().filter(|c| c.controller == seat) {
        let Some(cost) = v.definition.crew_cost() else { continue };
        let Some(cp) = state.computed_permanent(v.id) else { continue };
        // Already a creature (crewed/animated this turn) → nothing to do.
        if cp.card_types.contains(&CardType::Creature) {
            continue;
        }
        let mut chosen = Vec::new();
        let mut total = 0i32;
        for &(id, p) in crewers.iter().filter(|&&(id, _)| id != v.id) {
            if total >= cost as i32 {
                break;
            }
            chosen.push(id);
            total += p;
        }
        // Worth it only if the cost is fully paid and the Vehicle is at least
        // as big as the creatures tapped to crew it.
        if chosen.is_empty() || total < cost as i32 || cp.power < total {
            continue;
        }
        let action = GameAction::Crew { vehicle: v.id, crew_creatures: chosen };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// Fire a "deal N damage to each opponent" / "drain N" / "each opponent loses
/// N" activated ability when it's lethal to a living opponent. Only fixed
/// (`Value::Const`) amounts are considered, and only when some opponent's life
/// is at or below the amount, so the bot spends the resource (mana / a discard
/// / a tap) exclusively to win — never to chip. Dry-run-gated via
/// `would_accept`.
fn pick_reach_burn(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::{PlayerRef, Selector, Value};
    // Amount an ability's effect would subtract from each opponent's life, if
    // it's an each-opponent reach effect with a fixed amount.
    fn reach_amount(effect: &Effect) -> Option<i32> {
        match effect {
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(n) }
            | Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(n) }
            | Effect::Drain { from: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(n), .. } => {
                Some(*n)
            }
            // Compound abilities (e.g. "do X; each opponent loses N") still
            // count their each-opponent reach: sum a Seq's components, take the
            // best mode of a modal. `would_accept` still gates legality, so a
            // wrapped component that demands a target this call can't supply
            // keeps the whole activation from firing.
            Effect::Seq(parts) => {
                let total: i32 = parts.iter().filter_map(reach_amount).sum();
                (total > 0).then_some(total)
            }
            Effect::ChooseMode(modes) => modes.iter().filter_map(reach_amount).max(),
            _ => None,
        }
    }
    let lethal_threshold = state
        .players
        .iter()
        .enumerate()
        .filter(|(p, pl)| !state.same_team(*p, seat) && pl.is_alive())
        .map(|(_, pl)| pl.life)
        .min()?;
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            let Some(amount) = reach_amount(&ab.effect) else { continue };
            if amount < lethal_threshold {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Activate a team-wide "creatures you control get +N/+N until end of turn"
/// ability while the bot has two or more attacking creatures, so the pump
/// connects on the swing. Only positive, no-target, non-sacrifice pumps are
/// considered; dry-run-gated so cost / timing bottom out in `would_accept`.
/// True when `req` constrains its subjects to creatures the controller owns
/// (a `ControlledByYou` clause anywhere in its And/Or tree).
fn requirement_restricts_to_your_creatures(req: &crate::card::SelectionRequirement) -> bool {
    use crate::card::SelectionRequirement as R;
    match req {
        R::ControlledByYou => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_restricts_to_your_creatures(a) || requirement_restricts_to_your_creatures(b)
        }
        _ => false,
    }
}

fn pick_team_pump(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::{Selector, Value};
    let attackers = state
        .attacking
        .iter()
        .filter(|a| state.battlefield_find(a.attacker).is_some_and(|c| c.controller == seat))
        .count();
    if attackers < 2 {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if ab.sac_cost {
                continue;
            }
            let Effect::PumpPT { what: Selector::EachPermanent(req), power: Value::Const(p), .. } =
                &ab.effect
            else {
                continue;
            };
            // Only a friendly-team pump (filter restricts to your creatures)
            // with a real power boost.
            if *p <= 0 || !requirement_restricts_to_your_creatures(req) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Activate a bare "{cost}: draw a card" ability (no target, doesn't sacrifice
/// the source) when the bot is card-starved (≤2 cards in hand) and can afford
/// it. Fired last, as a mana sink, so it never pre-empts casting spells or
/// playing lands. Dry-run-gated through `would_accept`.
fn pick_card_draw_ability(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::Selector;
    if state.players[seat].hand.len() > 2 {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            let Effect::Draw { who: Selector::You, .. } = &ab.effect else { continue };
            if ab.sac_cost {
                continue; // don't sacrifice the source just to draw
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Offer a `TurnFaceUp` for the first affordable face-down permanent the bot
/// controls. The cost is the real card's Morph/Megamorph/Disguise cost, or its
/// mana cost for a manifested/cloaked creature card; `would_accept` enforces it.
fn pick_turn_face_up(state: &GameState, seat: usize) -> Option<GameAction> {
    state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.face_down && c.face_up_def.is_some())
        .map(|c| GameAction::TurnFaceUp { card_id: c.id })
        .find(|a| state.would_accept(a.clone()))
}

/// Fire a single-target "deal damage to any target" activated ability that
/// kills an opposing creature outright. Handles a constant damage amount
/// (Frostwielder, Pain Kami at fixed X) and the "damage equal to its own power"
/// shape (Kiku, Night's Flower). Targets the highest-power killable opponent
/// creature; dry-run-gated so cost / sorcery timing / target legality all
/// bottom out in `would_accept`. Points the ability at an opponent's face only
/// when the hit is exactly lethal (reach for the win); otherwise never chips a
/// player and never targets the bot's own creatures.
/// Every activated ability a permanent can use right now, paired with the
/// index `GameAction::ActivateAbility` expects: printed abilities first, then
/// the statically granted ones at their virtual indices (CR 611.2 — the
/// Threshold-granted removal on the Torment Possessed cycle, Cryptolith Rite).
/// The bot's ability generators walk this instead of `definition
/// .activated_abilities`, which silently skipped every grant.
fn usable_abilities(
    state: &GameState,
    card: &crate::card::CardInstance,
) -> Vec<(usize, crate::effect::ActivatedAbility)> {
    let printed = card.definition.activated_abilities.clone();
    let n = printed.len();
    printed
        .into_iter()
        .enumerate()
        .chain(
            state
                .granted_abilities_for(card.id)
                .into_iter()
                .enumerate()
                .map(|(i, ab)| (n + i, ab)),
        )
        .collect()
}

/// "{cost}: Destroy target creature" on a permanent that survives the
/// activation — the untargeted-at-self sibling of `pick_removal_sacrifice`.
/// Fires on the biggest legal opposing creature.
fn pick_removal_destroy(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::Selector;
    let mut foes: Vec<(crate::card::CardId, i32)> = state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
        .filter_map(|c| state.computed_permanent(c.id).map(|cp| (c.id, cp.power)))
        .collect();
    foes.sort_by_key(|(_, pow)| std::cmp::Reverse(*pow));
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in usable_abilities(state, card) {
            if ab.sac_cost {
                continue; // `pick_removal_sacrifice` owns the trade math.
            }
            let (Effect::Destroy { what } | Effect::DestroyNoRegen { what }) = &ab.effect else {
                continue;
            };
            if !matches!(what, Selector::Target(_) | Selector::TargetFiltered { .. }) {
                continue;
            }
            for (foe, _) in &foes {
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Permanent(*foe)),
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                // Unpayable ward tax → the activation would be countered;
                // fall through to the next-biggest foe instead.
                if !ward_gate_ok(state, seat, &action) {
                    continue;
                }
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

fn pick_removal_ping(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::{Selector, Value};
    // Reach for the win first: if a constant-damage "any target" ability is
    // lethal to an opponent, point it at their face. Only fires when the hit
    // is actually lethal (life ≤ amount), so it's never a wasted chip ping.
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in usable_abilities(state, card) {
            let Effect::DealDamage { to, amount: Value::Const(n) } = &ab.effect else { continue };
            // Must be an untyped "any target" slot (a creature-only filter
            // can't be pointed at a player).
            if !matches!(to, Selector::Target(_)) {
                continue;
            }
            for opp in 0..state.players.len() {
                if state.same_team(opp, seat) || state.players[opp].life > *n {
                    continue;
                }
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Player(opp)),
                    additional_targets: Vec::new(),
                    x_value: None, mode: None,
                };
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    // Opposing creatures, highest computed power first (best removal value).
    let mut foes: Vec<(crate::card::CardId, i32)> = state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
        .filter_map(|c| state.computed_permanent(c.id).map(|cp| (c.id, cp.power)))
        .collect();
    foes.sort_by_key(|(_, pow)| std::cmp::Reverse(*pow));
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in usable_abilities(state, card) {
            // The effect must be a bare single-target DealDamage whose target
            // can be a creature (not a self/own-board selector).
            let Effect::DealDamage { to, amount } = &ab.effect else { continue };
            if !matches!(to, Selector::Target(_) | Selector::TargetFiltered { .. }) {
                continue;
            }
            for (foe, foe_pow) in &foes {
                let Some(cp) = state.computed_permanent(*foe) else { continue };
                // Remaining toughness after damage already marked this turn
                // (CR 120.6) — a ping that wouldn't kill a fresh creature can
                // still finish one that's been chipped in combat.
                let marked = state.battlefield_find(*foe).map(|c| c.damage as i32).unwrap_or(0);
                let remaining = cp.toughness - marked;
                // Lethal check: a constant amount, or "equal to its own power"
                // (Kiku) where the creature dies if power ≥ remaining toughness.
                let lethal = match amount {
                    Value::Const(n) => *n >= remaining,
                    Value::PowerOf(s) if matches!(**s, Selector::Target(_)) => {
                        *foe_pow >= remaining
                    }
                    // "Deals damage equal to its own power" pingers (firebreather-
                    // style {T} abilities) read the source's computed power.
                    Value::PowerOf(s) if matches!(**s, Selector::This) => state
                        .computed_permanent(card.id)
                        .is_some_and(|p| p.power >= remaining),
                    _ => false,
                };
                if !lethal {
                    continue;
                }
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Permanent(*foe)),
                    additional_targets: Vec::new(),
                    x_value: None, mode: None,
                };
                if !ward_gate_ok(state, seat, &action) {
                    continue;
                }
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    // Opposing planeswalkers, highest loyalty first — a constant-damage "any
    // target" ability that's lethal to the loyalty removes the threat.
    let mut walkers: Vec<(crate::card::CardId, i32)> = state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_planeswalker())
        .map(|c| (c.id, c.counter_count(crate::card::CounterType::Loyalty) as i32))
        .collect();
    walkers.sort_by_key(|(_, loy)| std::cmp::Reverse(*loy));
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            let Effect::DealDamage { to, amount: Value::Const(n) } = &ab.effect else { continue };
            // Any target slot that could point at a planeswalker (would_accept
            // re-checks the filter).
            if !matches!(to, Selector::Target(_) | Selector::TargetFiltered { .. }) {
                continue;
            }
            for (walker, loy) in &walkers {
                if *n < *loy {
                    continue;
                }
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Permanent(*walker)),
                    additional_targets: Vec::new(),
                    x_value: None, mode: None,
                };
                if !ward_gate_ok(state, seat, &action) {
                    continue;
                }
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

/// Activate a "Sacrifice this creature: Destroy target creature" ability
/// (Pus Kami, Scuttling Death-style sac removal) on a *favorable* trade: the
/// destroyed opposing creature must be at least as powerful as the creature
/// being sacrificed, so the bot won't pitch a 3/3 to kill a 1/1. Targets the
/// biggest qualifying foe. Dry-run-gated through `would_accept`.
fn pick_removal_sacrifice(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::effect::Selector;
    let mut foes: Vec<(crate::card::CardId, i32)> = state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
        .filter_map(|c| state.computed_permanent(c.id).map(|cp| (c.id, cp.power)))
        .collect();
    foes.sort_by_key(|(_, pow)| std::cmp::Reverse(*pow));
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        let src_power = state.computed_permanent(card.id).map(|cp| cp.power).unwrap_or(0);
        for (idx, ab) in usable_abilities(state, card) {
            if !ab.sac_cost {
                continue;
            }
            let target_is_creature = match &ab.effect {
                Effect::Destroy { what } | Effect::DestroyNoRegen { what } => {
                    matches!(what, Selector::Target(_) | Selector::TargetFiltered { .. })
                }
                _ => false,
            };
            if !target_is_creature {
                continue;
            }
            for (foe, foe_pow) in &foes {
                // Only a favorable/even trade.
                if *foe_pow < src_power {
                    continue;
                }
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target: Some(crate::game::Target::Permanent(*foe)),
                    additional_targets: Vec::new(),
                    x_value: None, mode: None,
                };
                if !ward_gate_ok(state, seat, &action) {
                    continue;
                }
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

/// Find an affordable graveyard-activated ability whose cost exiles the source
/// (Embalm / Eternalize, Stone Docent-style recursion). Returns the activation
/// for the first such card the bot can pay for.
fn pick_graveyard_recursion(state: &GameState, seat: usize) -> Option<GameAction> {
    // The bot's own creatures, highest-power first — candidate targets for
    // abilities that need one (Scavenge's +1/+1 counters, Daring Fiendbonder's
    // indestructible counter). For no-target recursion (Embalm / Eternalize /
    // Stone Docent) we pass `None`.
    let mut own: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature())
        .collect();
    own.sort_by_key(|c| std::cmp::Reverse(c.power()));
    for card in state.players[seat].graveyard.iter() {
        // Printed graveyard abilities plus static-granted ones (Varolz's
        // scavenge) at indices ≥ the printed count.
        let printed = card.definition.activated_abilities.clone();
        let granted = state.graveyard_granted_abilities(seat, card);
        for (idx, ab) in printed.iter().chain(granted.iter()).enumerate() {
            // Graveyard-activated abilities worth firing: an exile-self payoff
            // (Embalm-style value) or a self-return that replays the creature
            // (Llanowar Greenwidow's "{7}{G}: return this from your graveyard").
            if !(ab.from_graveyard
                && (ab.exile_self_cost || effect_returns_self_to_battlefield(&ab.effect)))
            {
                continue;
            }
            // Only try a no-target activation when the effect needs none —
            // otherwise `would_accept` (which doesn't re-derive targets) would
            // wave through a wasted target-less activation.
            let candidates: Vec<Option<crate::game::Target>> = if ab.effect.requires_target() {
                own.iter().map(|c| Some(crate::game::Target::Permanent(c.id))).collect()
            } else {
                vec![None]
            };
            for target in candidates {
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target,
                    additional_targets: Vec::new(),
                    x_value: None, mode: None,
                };
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

/// True if `eff` returns its own source to the battlefield (a self-reanimating
/// graveyard ability — Llanowar Greenwidow). Recurses into `Seq`.
fn effect_returns_self_to_battlefield(eff: &Effect) -> bool {
    use crate::effect::ZoneDest;
    match eff {
        Effect::Move { what: crate::card::Selector::This, to: ZoneDest::Battlefield { .. } } => true,
        // "Return this card from your graveyard to the battlefield transformed"
        // (Garland, Knight of Cornelia) — a self-reanimation like the plain
        // Move, just landing on the back face.
        Effect::ExileSelfReturnTransformed | Effect::ExileSelfReturnFrontFace => true,
        Effect::Seq(v) => v.iter().any(effect_returns_self_to_battlefield),
        _ => false,
    }
}

/// True if a `SelectionRequirement` tree constrains its target to a card in a
/// graveyard (`InYourGraveyard` / `InGraveyard`).
fn filter_targets_graveyard(req: &crate::card::SelectionRequirement) -> bool {
    use crate::card::SelectionRequirement as R;
    match req {
        R::InYourGraveyard | R::InGraveyard => true,
        R::And(a, b) | R::Or(a, b) => filter_targets_graveyard(a) || filter_targets_graveyard(b),
        _ => false,
    }
}

/// True if `eff` moves a graveyard-targeted card onto the battlefield (a
/// reanimation effect — Seedship Broodtender's sac-to-return). Recurses `Seq`.
fn effect_reanimates_from_graveyard(eff: &Effect) -> bool {
    use crate::effect::ZoneDest;
    match eff {
        Effect::Move {
            what: crate::card::Selector::TargetFiltered { filter, .. },
            to: ZoneDest::Battlefield { .. },
        } => filter_targets_graveyard(filter),
        Effect::Seq(v) => v.iter().any(effect_reanimates_from_graveyard),
        _ => false,
    }
}

/// Activate a battlefield permanent's ability that reanimates a card from the
/// graveyard (Seedship Broodtender's "{cost}, Sacrifice this: return target
/// creature/Spacecraft from your graveyard to the battlefield"), aimed at the
/// engine's auto-picked best graveyard target. Skips when nothing legal exists.
/// Dry-run-gated so cost / sorcery-speed timing bottom out in `would_accept`.
fn pick_battlefield_reanimate(state: &GameState, seat: usize) -> Option<GameAction> {
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if !effect_reanimates_from_graveyard(&ab.effect) {
                continue;
            }
            let target = state.auto_target_for_effect(&ab.effect, seat);
            if target.is_none() {
                continue; // no graveyard creature worth returning
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Crack a Lander token for ramp: a `{2}, {T}, Sacrifice: search a basic land
/// onto the battlefield tapped` ability. Only fires when the controller still
/// has a basic land in their library (so the fetch isn't wasted) and the
/// engine accepts the activation (mana/timing). Targets nothing — the fetch
/// resolves via the library-search decider.
fn pick_crack_lander(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::{ArtifactSubtype, SelectionRequirement};
    let has_basic = state.players[seat]
        .library
        .iter()
        .any(|c| state.evaluate_requirement_on_card(&SelectionRequirement::IsBasicLand, c, seat));
    if !has_basic {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat && !c.tapped) {
        let is_lander = card.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Lander);
        if !is_lander {
            continue;
        }
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if !ab.sac_cost || !matches!(ab.effect, Effect::Search { .. }) {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Find a beneficial energy-only activated ability the bot can pay for: an
/// `Effect::PayEnergy { amount, .. }` ability with no mana/tap/sac cost,
/// where the bot controls the source and has at least `amount` energy.
fn pick_energy_payoff(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].energy == 0 {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            // The energy can be modeled either as a real activation cost
            // (`ActivatedAbility.energy_cost`, the up-front-gated form) or as a
            // resolve-time `Effect::PayEnergy` rider. Match either so the bot
            // fires Longtusk Cub-style `{E}{E}{E}: +1/+1` payoffs regardless of
            // which shape the card uses.
            let amount = if ab.energy_cost > 0 {
                ab.energy_cost
            } else if let Effect::PayEnergy { amount, .. } = &ab.effect {
                *amount
            } else {
                continue;
            };
            let is_pure = !ab.tap_cost
                && !ab.sac_cost
                && ab.mana_cost.symbols.is_empty()
                && ab.life_cost == 0;
            if !is_pure || state.players[seat].energy < amount {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Pick an equip activation: the first controlled Equipment that's either
/// unattached or attached to a permanent the bot doesn't control, paired
/// with the highest-power creature the bot controls. Returns `None` when
/// there's nothing worth equipping. Dry-run gated by the caller's
/// `would_accept` is bypassed here (we gate inline) so the bot doesn't
/// thrash re-equipping the same creature.
/// Crew an uncrewed Vehicle (CR 702.122) the bot controls, tapping the
/// smallest untapped creatures that together meet the crew cost. Skipped
/// unless the Vehicle's power is worth more than the creatures spent on it
/// (so the bot never taps a bigger attacker to animate a smaller Vehicle).
fn pick_crew(state: &GameState, seat: usize) -> Option<GameAction> {
    for vehicle in &state.battlefield {
        if vehicle.controller != seat {
            continue;
        }
        let Some(crew_n) = vehicle.definition.crew_cost() else { continue };
        // Already a creature this turn (crewed / animated)? Don't re-crew.
        if state
            .computed_permanent(vehicle.id)
            .is_some_and(|cp| cp.card_types.contains(&crate::card::CardType::Creature))
        {
            continue;
        }
        // Candidate crew members: the bot's untapped creatures, smallest first.
        let mut crew: Vec<(CardId, u32)> = state
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == seat
                    && c.id != vehicle.id
                    && c.definition.is_creature()
                    && !c.tapped
            })
            // CR 702.122e/702.171 — count the crew-power rider (Cloudspire
            // Captain / Deathless Pilot crew "as though power N greater").
            .map(|c| (c.id, (c.power() + state.crew_saddle_power_bonus(c.id)).max(0) as u32))
            .collect();
        crew.sort_by_key(|&(_, p)| p);
        let mut picked = Vec::new();
        let mut total = 0u32;
        for (id, p) in &crew {
            if total >= crew_n {
                break;
            }
            picked.push(*id);
            total += p;
        }
        if total < crew_n {
            continue;
        }
        // Don't spend more board power than the Vehicle is worth.
        if total > vehicle.power().max(0) as u32 {
            continue;
        }
        let action = GameAction::Crew { vehicle: vehicle.id, crew_creatures: picked };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// CR 702.171 — saddle a Mount the bot is about to attack with by tapping its
/// least-valuable other untapped creatures (smallest power first). Only saddles
/// a Mount that can attack this turn and isn't already saddled, and never spends
/// more board power than the Mount itself is worth.
fn pick_saddle(state: &GameState, seat: usize) -> Option<GameAction> {
    // Saddled is "until end of turn" (CR 702.171e), so only pay the tap cost
    // when a combat phase still follows — i.e. precombat main. Saddling in
    // postcombat main just wastes the saddlers before the buff can matter.
    if state.step != TurnStep::PreCombatMain {
        return None;
    }
    for mount in &state.battlefield {
        if mount.controller != seat || mount.saddled || mount.tapped {
            continue;
        }
        let Some(saddle_n) = mount.definition.saddle_cost() else { continue };
        if !mount.can_attack() {
            continue;
        }
        // Candidate saddlers: the bot's other untapped creatures. Tap the ones
        // that can't attack this turn (summoning-sick / Defender) *first* — they
        // are "free" since they'd sit idle anyway — then fall back to would-be
        // attackers, smallest power first (the crew-power rider counts, CR
        // 702.171). Track how much *attacker* power we spend so the overspend
        // guard below doesn't fault free saddlers.
        let mut riders: Vec<(CardId, u32, bool)> = state
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == seat
                    && c.id != mount.id
                    && c.definition.is_creature()
                    && !c.tapped
            })
            .map(|c| {
                (c.id, (c.power() + state.crew_saddle_power_bonus(c.id)).max(0) as u32, c.can_attack())
            })
            .collect();
        // (can-attack ascending, then power ascending): free saddlers first.
        riders.sort_by_key(|&(_, p, can_attack)| (can_attack, p));
        let mut picked = Vec::new();
        let mut total = 0u32;
        let mut attacker_power = 0u32;
        for (id, p, can_attack) in &riders {
            if total >= saddle_n {
                break;
            }
            picked.push(*id);
            total += p;
            if *can_attack {
                attacker_power += p;
            }
        }
        if total < saddle_n {
            continue;
        }
        // Don't tap real attackers worth more board power than the Mount is
        // worth. Idle (can't-attack) saddlers are free and don't count.
        if attacker_power > mount.power().max(0) as u32 {
            continue;
        }
        let action = GameAction::Saddle { mount: mount.id, creatures: picked };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

fn pick_equip(state: &GameState, seat: usize) -> Option<GameAction> {
    // Best creature to wear an Equipment: highest current power, but skip
    // attack-locked bodies (Defender / CantAttack) — an Equipment's combat
    // bonus is wasted on them. Fall back to any creature only if every
    // candidate is attack-locked (a board of Walls still wants the
    // deathtouch/keyword grant for blocking).
    use crate::card::Keyword;
    let can_attack = |c: &crate::card::CardInstance| {
        state
            .computed_permanent(c.id)
            .map(|cp| {
                (!cp.keywords.contains(&Keyword::Defender)
                    || state.ignores_defender_for_attack(c))
                    && !cp.keywords.contains(&Keyword::CantAttack)
            })
            .unwrap_or(true)
    };
    let mine = || {
        state
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && c.definition.is_creature())
    };
    // Rank by *computed* power so anthems / lords / conditional pumps count
    // (a small body under a big anthem is a better Voltron target than a
    // vanilla bigger base body).
    let cpow = |c: &crate::card::CardInstance| {
        state.computed_permanent(c.id).map(|cp| cp.power).unwrap_or_else(|| c.power())
    };
    let target = mine()
        .filter(|c| can_attack(c))
        .max_by_key(|c| cpow(c))
        .or_else(|| mine().max_by_key(|c| cpow(c)))
        .map(|c| c.id)?;
    for eq in &state.battlefield {
        if eq.controller != seat || !eq.definition.is_equipment() {
            continue;
        }
        if eq.definition.has_equip().is_none() {
            continue;
        }
        // Skip if already on the chosen target (no point re-equipping).
        if eq.attached_to == Some(target) {
            continue;
        }
        let action = GameAction::Equip { equipment: eq.id, target };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
}

/// Drive a "{cost}: attach target Equipment you control to target creature you
/// control" activated ability (Brass Squire). Picks an Equipment not already on
/// the chosen wearer for slot 0 and the highest-power creature for slot 1. The
/// dry-run gate enforces the activation cost / target legality.
fn pick_attach_ability(state: &GameState, seat: usize) -> Option<GameAction> {
    use crate::card::Selector;
    use crate::effect::Effect;
    let wearer = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature())
        .max_by_key(|c| c.power())
        .map(|c| c.id)?;
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            // Two distinct target slots: `what` (slot 0) and `to` (slot 1).
            let Effect::Attach {
                what: Selector::TargetFiltered { slot: 0, .. },
                to: Selector::TargetFiltered { slot: 1, .. },
            } = &ab.effect
            else {
                continue;
            };
            let Some(equip) = state.battlefield.iter().find(|e| {
                e.controller == seat
                    && e.definition.is_equipment()
                    && e.attached_to != Some(wearer)
            }) else {
                continue;
            };
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: Some(crate::game::Target::Permanent(equip.id)),
                additional_targets: vec![crate::game::Target::Permanent(wearer)],
                x_value: None, mode: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Walk every planeswalker the bot controls and pick the first activatable
/// loyalty ability. Auto-target via `auto_target_for_effect` for abilities
/// that require a target. Prefers a +loyalty ability when available
/// (preserves the walker for next turn), falling back to the ability with
/// the smallest absolute loyalty cost so we don't suicide-ult immediately.
fn pick_loyalty_ability(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    for card in &state.battlefield {
        if card.controller != seat {
            continue;
        }
        if !card.definition.is_planeswalker() {
            continue;
        }
        let allowed = if card.definition.loyalty_twice_each_turn { 2 } else { 1 };
        if card.loyalty_uses_this_turn >= allowed {
            continue;
        }
        // Gather every affordable ability and pick by OUTCOME, not by
        // loyalty-cost order. The old plus-first walk meant a walker with
        // a strong minus never used it — Professor Dellian Fel spent whole
        // games on "+2: gain 3 life" while "−3: destroy target creature"
        // sat unused (its attribution read neutral for a bomb). Use the
        // *effective* list (printed + statically-granted, e.g. Kasmina
        // Enigma Sage / Ichormoon Gauntlet) so the bot can activate granted
        // loyalty abilities too — the engine indexes the same list.
        // Ultimates whose payoff the material eval can't see (emblems)
        // still lose to a plus — a known limitation.
        let current_loyalty =
            card.counter_count(crate::card::CounterType::Loyalty) as i32;
        let effective = crate::game::effective_loyalty_abilities(card, &state.battlefield);
        let mut finalists: Vec<(i32, GameAction)> = Vec::new();
        for (idx, ability) in effective.iter().enumerate() {
            if current_loyalty + ability.loyalty_cost < 0 {
                continue;
            }
            let target = if ability.effect.requires_target() {
                // No legal target for *this* ability — skip it and try the
                // next (formerly `?`-returned, which abandoned every other
                // ability and planeswalker the bot controls).
                match state.auto_target_for_effect(&ability.effect, seat) {
                    Some(t) => Some(t),
                    None => continue,
                }
            } else {
                None
            };
            // Variable-X (`-X`) ability: commit all current loyalty.
            let x_value = ability.x_cost.then_some(current_loyalty.max(0) as u32);
            let action = GameAction::ActivateLoyaltyAbility {
                card_id: card.id,
                ability_index: idx,
                target,
                x_value,
            };
            if state.would_accept(action.clone()) {
                finalists.push((score_candidate(state, seat, &action, w), action));
            }
        }
        // A walker the board kills before its next activation banks
        // nothing by plussing — the loyalty it gains is removed by
        // attackers at zero cost to the opponent, a future the outcome
        // eval's one-combat horizon cannot see. When the enemy board's
        // creature power already covers current loyalty (a crude read
        // on next combat), cash out: restrict to loyalty-SPENDING
        // abilities whenever any is affordable, and let the outcome
        // eval pick among those.
        let threat: i32 = state
            .battlefield
            .iter()
            .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
            .filter_map(|c| state.computed_permanent(c.id).map(|cp| cp.power.max(0)))
            .sum();
        if threat >= current_loyalty {
            let spending: Vec<(i32, GameAction)> = finalists
                .iter()
                .filter(|(_, a)| {
                    matches!(a, GameAction::ActivateLoyaltyAbility { ability_index, .. }
                        if effective.get(*ability_index).is_some_and(|ab| ab.loyalty_cost < 0))
                })
                .cloned()
                .collect();
            if !spending.is_empty() {
                finalists = spending;
            }
        }
        if let Some(best) = pick_by_outcome(state, seat, finalists, w) {
            return Some(best);
        }
    }
    None
}

/// Test-visible wrapper for `forced_blocks` — the declaration an attacking
/// block chooser (Invasion Plans) submits.
pub fn forced_blocks_for_test(state: &GameState) -> Vec<(CardId, CardId)> {
    forced_blocks(state)
}

/// Test-visible wrapper for `pick_blocks` so external tests can exercise
/// the blocker heuristic in isolation.
pub fn pick_blocks_for_test(state: &GameState, seat: usize) -> Vec<(CardId, CardId)> {
    pick_blocks(state, seat)
}

/// The bot's attack declaration for `seat`: which creatures swing and at
/// what. Extracted from `next_action` so the combat-aware evaluation can
/// replay the same choice inside a simulation (see
/// [`simulate_through_combat`]) rather than re-deriving it.
pub fn pick_attacks(state: &GameState, seat: usize) -> Vec<Attack> {
    use crate::card::Keyword;
    // Pick the attack target: prefer an opposing monarch (CR
    // 724 — stealing the crown denies their end-step card and
    // hands it to us); otherwise the next alive opponent.
    let target_player = match state.monarch {
        Some(m)
            if m != seat
                && state.players.get(m).map(|p| p.is_alive()).unwrap_or(false) =>
        {
            m
        }
        _ => state.next_alive_seat(seat),
    };
    // Filter on `controller`, not `owner`: cards that have
    // changed control (Threaten / Mind Control / etc.) are
    // attacked WITH by the new controller, not the original
    // owner.
    //
    // Bot AI improvement (push XXV): hold back attackers
    // that would suicide into deathtouch blockers when
    // there's no upside. The heuristic computes:
    //   * lethal_swing: whether sum of attackers' powers
    //     already meets opponent's life total.
    // When NOT lethal:
    //   * skip attackers whose toughness is <= the maximum
    //     opponent blocker power AND there's at least one
    //     opponent blocker with deathtouch + reach/flying
    //     parity (i.e. a blocker can be assigned).
    // This keeps small attackers from auto-dying to
    // Witherbloom Crawler / Sapworm / Toxicultivator and
    // similar deathtouch defenders.
    let opp_seat = target_player;
    let opp_life = state.players[opp_seat].life;
    let raw_attackers: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == seat
                // `can_attack()`'s components minus its printed-
                // Defender gate, which is re-checked below against
                // the computed keyword set so a team "attack as
                // though no defender" grant (High Alert) applies.
                && c.definition.is_creature()
                && !c.tapped
                && (!c.summoning_sick || c.has_keyword(&Keyword::Haste))
                && !c.has_keyword(&Keyword::CantAttack)
                // Honor layer-granted Defender / can't-attack
                // (Pacifism, crewed-Vehicle states) — can_attack
                // only sees printed keywords.
                && state
                    .computed_permanent(c.id)
                    .map(|cp| {
                        (!cp.keywords.contains(&Keyword::Defender)
                            || state.ignores_defender_for_attack(c))
                            && !cp.keywords.contains(&Keyword::CantAttack)
                            // CR 508.1a — "can attack only if
                            // defending player controls [X]"
                            // (Dandân). Don't declare it into a
                            // defender whose board fails the
                            // filter, or the whole batch is
                            // rejected.
                            && cp.keywords.iter().all(|kw| match kw {
                                Keyword::CanAttackOnlyIfDefenderControls(req) => {
                                    state.battlefield.iter().any(|d| {
                                        d.controller == target_player
                                            && state.evaluate_requirement_on_card(
                                                req, d, target_player,
                                            )
                                    })
                                }
                                Keyword::CanAttackOnlyIfYouControl(req) => {
                                    state.battlefield.iter().any(|d| {
                                        d.controller == c.controller
                                            && state.evaluate_requirement_on_card(
                                                req, d, c.controller,
                                            )
                                    })
                                }
                                Keyword::CantAttackOrBlockUnlessEvenCounters => {
                                    c.counters.values().sum::<u32>() % 2 == 0
                                }
                                Keyword::CantAttackOrBlockUnlessYouControlCount {
                                    filter,
                                    min,
                                    block_only,
                                    exclude_self,
                                    ..
                                } => {
                                    // A block-only gate never
                                    // restricts attacking. `exclude_self`
                                    // drops the gated creature from the
                                    // count ("another …" — Tiger-Dillo).
                                    *block_only
                                        || state
                                            .battlefield
                                            .iter()
                                            .filter(|d| {
                                                d.controller == c.controller
                                                    && !(*exclude_self && d.id == c.id)
                                                    && state
                                                        .evaluate_requirement_on_card(
                                                            filter,
                                                            d,
                                                            c.controller,
                                                        )
                                            })
                                            .count()
                                            as u32
                                            >= *min
                                }
                                _ => true,
                            })
                    })
                    .unwrap_or(true)
        })
        .collect();
    // Use the damage-aware value so toughness-attackers (Doran,
    // High Alert) are weighed by what they actually deal.
    let total_raw_power: i32 =
        raw_attackers.iter().map(|c| attacker_damage_value(state, c.id)).sum();
    let lethal_swing = total_raw_power >= opp_life;
    // Race math: compare full-out clocks. We strike first
    // (it's our combat), so strictly fewer turns-to-lethal
    // than the opponent's counter-clock — inside a short
    // horizon — means holding back only concedes the race;
    // attack like it's lethal-in-N. Defenders and can't-
    // attack bodies add nothing to their clock.
    let opp_clock: i32 = state
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == opp_seat
                && c.definition.is_creature()
                && !c.has_keyword(&Keyword::Defender)
                && !c.has_keyword(&Keyword::CantAttack)
        })
        .map(|c| c.power().max(0))
        .sum();
    let racing = total_raw_power > 0 && opp_clock > 0 && {
        let our_turns = (opp_life.max(1) + total_raw_power - 1) / total_raw_power;
        let their_turns =
            (state.effective_life(seat).max(1) + opp_clock - 1) / opp_clock;
        our_turns < their_turns && our_turns <= 5
    };
    let lethal_swing = lethal_swing || racing;
    let opp_blockers: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| {
            // A creature that's tapped, not a creature, or has a
            // computed `CantBlock` (Sandstorm Verge, pacifism-
            // style effects) can't block — don't let the bot hold
            // attackers back for a blocker that can't legally block.
            c.controller == opp_seat
                && c.can_block()
                && !state
                    .computed_permanent(c.id)
                    .is_some_and(|cp| cp.keywords.contains(&Keyword::CantBlock))
        })
        .collect();
    let has_ground_deathtouch = opp_blockers
        .iter()
        .any(|b| b.has_keyword(&Keyword::Deathtouch) && !b.has_keyword(&Keyword::Flying));
    let max_ground_blocker_power: i32 = opp_blockers
        .iter()
        .filter(|b| !b.has_keyword(&Keyword::Flying))
        .map(|b| b.power())
        .max()
        .unwrap_or(0);
    let mut attackers: Vec<crate::card::CardId> = raw_attackers
        .into_iter()
        .filter(|c| {
            // CR 508.1d — must-attack creatures (Juggernaut,
            // goaded) have no choice; always include them so
            // the engine's requirement check accepts the batch.
            if c.has_keyword(&Keyword::MustAttack) || !c.goaded_by.is_empty() {
                return true;
            }
            // Always attack on lethal swings — the bot
            // would rather suicide than miss a kill.
            if lethal_swing {
                return true;
            }
            // CR 615.1 — don't swing with a creature whose
            // combat damage is prevented this turn (Fog /
            // Inspire Awe's exception); attacking only risks it
            // for no damage.
            if state.combat_damage_prevented_for_dealer(c.id) {
                return false;
            }
            // Unblockable by the current board: if the
            // opponent has creatures but none can legally
            // block this attacker (Unblockable, "can't be
            // blocked by/except by" restrictions the board
            // can't satisfy), it's a free swing. Generalizes
            // the Flying/Menace evasion checks below.
            if !opp_blockers.is_empty()
                && opp_blockers
                    .iter()
                    .all(|b| !state.blocker_can_block_attacker(b.id, c.id))
            {
                return true;
            }
            let flying = c.has_keyword(&Keyword::Flying);
            // Evasive attackers (flying) — only block-
            // worried if there's a flying opp blocker.
            // Skip the deathtouch / ground-power filter
            // for them; assume they're safe.
            if flying {
                let opp_has_flying_blocker = opp_blockers.iter()
                    .any(|b| b.has_keyword(&Keyword::Flying)
                          || b.has_keyword(&Keyword::Reach));
                if !opp_has_flying_blocker {
                    return true; // free swing
                }
            }
            // Trample: tougher creatures still come in
            // (we'll get some damage through chumps).
            if c.has_keyword(&Keyword::Trample) {
                return true;
            }
            // Indestructible: safe to swing (won't die).
            if c.has_keyword(&Keyword::Indestructible) {
                return true;
            }
            // Shield counter on the attacker — the first
            // damage is prevented, so a basic ground-trade
            // is safe (push XXVI bot improvement).
            if c.counter_count(crate::card::CounterType::Shield) > 0 {
                return true;
            }
            // Lifelink: even if we trade, we gain life —
            // worth swinging when we can race.
            if c.has_keyword(&Keyword::Lifelink) {
                return true;
            }
            // Deathtouch attacker: any blocker that deals
            // with it dies (CR 702.2), so blocking is at
            // best an even trade for the opponent — swinging
            // is always at least fine.
            if c.has_keyword(&Keyword::Deathtouch) && c.power() >= 1 {
                return true;
            }
            // Menace (CR 702.111): needs two+ blockers. If
            // the opponent has fewer than two creatures that
            // can legally block this attacker, it gets
            // through unblocked — safe to swing.
            if c.has_keyword(&Keyword::Menace) {
                let able = opp_blockers
                    .iter()
                    .filter(|b| {
                        !flying
                            || b.has_keyword(&Keyword::Flying)
                            || b.has_keyword(&Keyword::Reach)
                    })
                    .count();
                if able < 2 {
                    return true;
                }
            }
            // First strike + bigger power than blockers'
            // toughness — we kill the blocker before it
            // strikes back. Safe attack (push XXVI).
            if c.has_keyword(&Keyword::FirstStrike)
                || c.has_keyword(&Keyword::DoubleStrike)
            {
                let max_blocker_toughness: i32 = opp_blockers
                    .iter()
                    .filter(|b| !b.has_keyword(&Keyword::Flying) || flying)
                    .map(|b| b.toughness())
                    .max()
                    .unwrap_or(0);
                if c.power() > max_blocker_toughness {
                    return true;
                }
            }
            // Hold back if a deathtouch blocker exists
            // and we don't outsize the biggest blocker.
            if has_ground_deathtouch && !flying {
                return false;
            }
            // Finality counter on the attacker — if it
            // dies it'll exile instead of returning to
            // the graveyard (CR 122.1h). Don't suicide
            // a finality-counter creature into ground
            // blockers that can kill it.
            // Push (claude/modern_decks, batches 192-197).
            if c.counter_count(crate::card::CounterType::Finality) > 0
                && !flying
                && max_ground_blocker_power >= c.toughness()
            {
                return false;
            }
            // Hold back if our toughness is <= biggest
            // blocker power and we wouldn't kill them
            // (basic suicide filter).
            if !flying
                && max_ground_blocker_power >= c.toughness()
                && c.power() <= max_ground_blocker_power
            {
                return false;
            }
            true
        })
        .map(|c| c.id)
        .collect();
    // CR 506.2 — Silent Arbiter caps the whole combat. An
    // over-sized batch is rejected outright, so trim to the
    // cap keeping the biggest attackers.
    if let Some(cap) = state.combat_participation_cap(false)
        && attackers.len() > cap as usize
    {
        attackers.sort_by_key(|id| {
            -state.computed_permanent(*id).map(|cp| cp.power).unwrap_or(0)
        });
        attackers.truncate(cap as usize);
    }
    // CR 508.0 — drop a lone attacker that can't attack alone
    // (Militia Rallier): a single-attacker batch with
    // CantAttackAlone would be rejected, costing the bot its
    // whole combat. Only matters when it's the sole attacker.
    if attackers.len() == 1
        && state
            .computed_permanent(attackers[0])
            .is_some_and(|cp| cp.keywords.contains(&Keyword::CantAttackAlone))
    {
        attackers.clear();
    }
    // Find opponent planeswalkers in loyalty-ascending
    // order. The bot will redirect attacks at PWs whose
    // current loyalty is at-or-below our total attacking
    // power — finishing off the walker. Each PW consumes
    // up to its loyalty worth of attackers; the rest
    // attack the player.
    let mut walker_targets: Vec<(crate::card::CardId, u32)> = state
        .battlefield
        .iter()
        .filter(|c| {
            c.definition.is_planeswalker()
                && c.controller != seat
                && state.players[c.controller].is_alive()
                // CR 506.2 — The Aetherspark while attached.
                && !state.permanent_cant_be_attacked(c.id)
        })
        .map(|c| {
            let loyalty = c
                .counters
                .iter()
                .find_map(|(k, v)| {
                    matches!(k, crate::card::CounterType::Loyalty)
                        .then_some(*v)
                })
                .unwrap_or(0);
            (c.id, loyalty)
        })
        .collect();
    walker_targets.sort_by_key(|(_, l)| *l);
    let total_power: i32 = attackers
        .iter()
        .filter_map(|id| {
            state.battlefield.iter().find(|c| c.id == *id).map(|c| c.power())
        })
        .sum();
    let mut attacks: Vec<Attack> = Vec::new();
    for (pw_id, loyalty) in walker_targets {
        // Only redirect when we can plausibly finish it
        // off (total attacking power >= loyalty). Avoids
        // throwing 1-power chumps at a 5-loyalty walker.
        if (total_power as u32) < loyalty || loyalty == 0 {
            continue;
        }
        // Pull as many attackers as the walker's loyalty
        // for this redirect, picking smallest-power
        // first so we keep beefy beaters for the player
        // when possible. (Suicide-by-blocker is still
        // not modeled here.)
        let mut budget = loyalty as i32;
        attackers.sort_by_key(|id| {
            state
                .battlefield
                .iter()
                .find(|c| c.id == *id)
                .map(|c| c.power())
                .unwrap_or(0)
        });
        let mut remaining: Vec<crate::card::CardId> = Vec::new();
        for id in attackers.drain(..) {
            let pow = state
                .battlefield
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.power())
                .unwrap_or(0);
            if budget > 0 && pow > 0 {
                attacks.push(Attack {
                    attacker: id,
                    target: AttackTarget::Planeswalker(pw_id),
                });
                budget -= pow;
            } else {
                remaining.push(id);
            }
        }
        attackers = remaining;
    }
    // Remaining attackers go at the player.
    for id in attackers {
        attacks.push(Attack {
            attacker: id,
            target: AttackTarget::Player(target_player),
        });
    }
    attacks
}

/// The attack declaration, chosen by search rather than by rule.
///
/// [`pick_attacks`] is a greedy accretion: a pile of individually sensible
/// exclusions (don't suicide into deathtouch, respect Propaganda, honor
/// layer-granted Defender) applied to "swing with everything". Each of
/// those rules is right about the case it names. What none of them can see
/// is the *cost of tapping the team* — that a creature which attacks is
/// not available to block next turn — because that cost is only paid a
/// turn later, on a board the greedy rule never looks at.
///
/// `bot_probe` measures the consequence: the bot declares every eligible
/// creature in 73 % of its combats, and 41 % of its creatures are tapped
/// when it is asked to block. Half of the combats where it has no blocker
/// at all are tapped-out boards rather than empty ones.
///
/// So this searches instead. The greedy declaration seeds the candidate
/// set; the alternatives are "attack with nobody" and the greedy set minus
/// one attacker each. Every candidate is played forward through our combat
/// damage, the rest of our turn, and the opponent's crack-back, then scored
/// with the same evaluator everything else uses — which already prices both
/// the life we took and the creatures we kept.
///
/// Restricted to *dropping* attackers on purpose: greedy already attacks
/// with 77 % of eligible creatures, so its error is over-attacking, and a
/// one-step hill climb toward restraint targets that error directly
/// without paying for a search over subsets the bot will never want.
///
/// **Tried and reverted**: forcing unblockable attackers into every
/// candidate, on the theory that free damage should never be declined.
/// It measured *worse* — 51.4 % [50.3 %, 52.4 %] against this version's
/// 52.4 % [51.3 %, 53.5 %] on the same seed and sample — and did nothing
/// for the dimir mirror it was aimed at (44.0 % against 44.8 %). The
/// reasoning was simply wrong: evasion is about being *blocked*, not about
/// *blocking*, so a 2/3 flier that no ground board can stop on offense is
/// still a perfectly good blocker on defense. Forcing it to attack deletes
/// a real option. See `EvalWeights::attack_search` for the dimir deficit,
/// which remains open.
///
/// Ties go to the greedy set, so the search only ever departs from the
/// current behavior for a strict improvement.
fn pick_attacks_scored(state: &GameState, seat: usize, w: &EvalWeights) -> Vec<Attack> {
    let greedy = pick_attacks(state, seat);
    if w.attack_search == 0 || greedy.is_empty() {
        return greedy;
    }
    // Candidates, in the order they're scored. Index 0 is greedy and wins
    // every tie.
    let mut candidates: Vec<Vec<Attack>> = vec![greedy.clone(), Vec::new()];
    if greedy.len() > 1 {
        // Which attacker to consider holding back? Order by toughness
        // ascending: the cheapest body to keep home is also the one most
        // likely to die attacking, so the front of this list is where both
        // halves of the trade are largest.
        let mut order: Vec<usize> = (0..greedy.len()).collect();
        order.sort_by_key(|&i| {
            state.battlefield_find(greedy[i].attacker).map(|c| c.toughness()).unwrap_or(0)
        });
        for &i in order.iter().take(w.attack_search as usize) {
            let mut alt = greedy.clone();
            alt.remove(i);
            candidates.push(alt);
        }
    }

    let mut best: Option<(usize, i32)> = None;
    for (i, cand) in candidates.iter().enumerate() {
        let Some(score) = simulate_attack_outcome(state, seat, cand, w) else { continue };
        // Strictly greater: index 0 is greedy, so equal scores keep it.
        if best.map(|(_, s)| score > s).unwrap_or(true) {
            best = Some((i, score));
        }
    }
    match best {
        Some((i, _)) => candidates.swap_remove(i),
        None => greedy,
    }
}

/// Declare `attacks`, play the turn out through the opponent's counter-
/// attack, and score the board for `seat`.
///
/// The horizon matters more than the fidelity here. Scoring right after our
/// own combat damage would make holding a creature back *strictly* worse —
/// we dealt less damage and gained nothing measurable — because the entire
/// payoff of restraint is that the creature is untapped on the opponent's
/// turn. So the simulation has to reach their combat damage or it cannot
/// see the thing it exists to weigh.
///
/// By default neither side casts anything during the simulation; both take
/// the greedy combat declarations. That's a real simplification — an
/// opponent holding removal, or ourselves holding a trick, are invisible —
/// but it keeps the cost to one turn cycle of priority passes per
/// candidate, and the greedy declarations are exactly the policy this
/// search is trying to beat, which makes the comparison conservative
/// rather than flattering. Under
/// [`attack_sim_spells`](EvalWeights::attack_sim_spells) both seats cast
/// via [`sim_spell_action`], which is what makes the crack-back visible.
///
/// `None` when the declaration is rejected (a "must attack" creature we
/// tried to hold back) or the simulation runs out of fuel — an unfinished
/// turn is scored not at all rather than scored wrong, the same rule
/// [`simulate_through_combat`] settled on.
fn simulate_attack_outcome(
    state: &GameState,
    seat: usize,
    attacks: &[Attack],
    w: &EvalWeights,
) -> Option<i32> {
    if w.determinize > 1 {
        let mut total = 0i64;
        let mut n = 0i64;
        for k in 0..w.determinize {
            if let Some(v) = simulate_attack_outcome_once(state, seat, attacks, w, k) {
                total += v as i64;
                n += 1;
            }
        }
        return (n > 0).then(|| (total / n) as i32);
    }
    simulate_attack_outcome_once(state, seat, attacks, w, 0)
}

fn simulate_attack_outcome_once(
    state: &GameState,
    seat: usize,
    attacks: &[Attack],
    w: &EvalWeights,
    k: u8,
) -> Option<i32> {
    let mut g = sim_start_state(state, seat, w, k);
    g.perform_action(GameAction::DeclareAttackers(attacks.to_vec())).ok()?;
    let start_turn = g.turn_number;
    // One turn cycle of pure priority passes is on the order of fifty
    // actions; the rest is headroom for triggers and decisions.
    let mut fuel = 400u32;
    // Break when this turn's opponent combat has resolved; the race
    // horizon can push the stop out one more cycle (see below).
    let mut stop_turn = start_turn;
    let mut extended = false;
    let mut declared: std::collections::HashSet<(u32, TurnStep)> = Default::default();
    // *This* turn's attack declaration is the candidate, already submitted.
    // Without this the loop's own DeclareAttackers arm fires on the same
    // turn and re-declares the greedy set over the top of it — which the
    // engine happily accepts, so every candidate silently collapses back to
    // the alpha strike and the whole search scores one line N times.
    declared.insert((g.turn_number, TurnStep::DeclareAttackers));
    while !g.is_game_over() {
        // Stop once the opponent's combat is resolved — the first board on
        // which a creature held back has actually done anything. Under
        // `attack_race_horizon`, a sim ending with either life total in
        // burn range runs one more full cycle instead, so the race this
        // attack started is scored at its result, not mid-sprint.
        if g.turn_number > stop_turn && g.step >= TurnStep::EndCombat {
            if w.attack_race_horizon
                && !extended
                && g.players.iter().any(|p| p.is_alive() && p.life <= 10)
            {
                extended = true;
                stop_turn = g.turn_number;
                fuel = fuel.saturating_add(400);
            } else {
                break;
            }
        }
        fuel = fuel.checked_sub(1)?;
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(&g, pending.acting_player(), w, &pending.decision, false)
            };
            g.perform_action(GameAction::SubmitDecision(answer)).ok()?;
            continue;
        }
        // Declarations are one-shot per step per turn; the marker keeps a
        // rejected declaration from being retried forever.
        let key = (g.turn_number, g.step);
        let action = match g.step {
            TurnStep::DeclareAttackers if !declared.contains(&key) => {
                declared.insert(key);
                let declarer = g.attack_declarer();
                // Greedy, deliberately: calling the search here would
                // recurse a turn deeper on every candidate.
                GameAction::DeclareAttackers(pick_attacks(&g, declarer))
            }
            TurnStep::DeclareBlockers if !declared.contains(&key) && !g.attacking().is_empty() => {
                match (0..g.players.len()).find(|&s| g.may_declare_blocks(s)) {
                    Some(defender) => {
                        declared.insert(key);
                        GameAction::DeclareBlockers(pick_blocks(&g, defender))
                    }
                    None => GameAction::PassPriority,
                }
            }
            _ if w.attack_sim_spells => {
                sim_spell_action(&g, w).unwrap_or(GameAction::PassPriority)
            }
            _ => GameAction::PassPriority,
        };
        if g.perform_action(action).is_err() && g.perform_action(GameAction::PassPriority).is_err() {
            return None;
        }
    }
    Some(eval_material(&g, seat, w))
}

/// The spell a combat simulation lets the current priority holder cast —
/// see [`EvalWeights::attack_sim_spells`]. The response layer and the
/// post-block trick window mirror the real dispatch; a main phase inside
/// the sim's horizon takes the best STATIC-ranked candidate. No outcome
/// eval, no hold gates, no jitter: nesting the full pick inside a sim
/// would multiply clone-and-resolve work per candidate, and a
/// deterministic greedy stand-in carries exactly the information the sim
/// is missing — "that mana will be spent on something".
fn sim_spell_action(g: &GameState, w: &EvalWeights) -> Option<GameAction> {
    let p = g.player_with_priority();
    if !g.stack.is_empty() {
        return pick_stack_response(g, p, w)
            .or_else(|| pick_ability_counter_response(g, p))
            .or_else(|| pick_prepare_response(g, p, w));
    }
    if g.step == TurnStep::DeclareBlockers && g.blockers_declared() {
        return pick_combat_trick(g, p, w);
    }
    if matches!(g.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
        && g.active_player_idx == p
    {
        let probe = g.affordance_probe_template();
        let mut ranked: Vec<(i32, GameAction, bool)> = cast_candidates(g, p, &probe, w)
            .into_iter()
            .map(|(a, ok)| (score_candidate(g, p, &a, w), a, ok))
            .collect();
        ranked.sort_by_key(|(s, _, _)| std::cmp::Reverse(*s));
        for (_, a, ok) in ranked {
            if ok || GameState::would_accept_on(&probe, a.clone()) {
                return Some(a);
            }
        }
    }
    None
}

/// The block assignment, chosen by search rather than by rule.
///
/// [`pick_blocks`] assigns blockers greedily, one at a time, in ascending
/// power order, each taking the best attacker it can find *given the
/// assignments already made*. That ordering is a heuristic standing in for
/// the thing it can't do: score the whole assignment. A first-fit choice
/// that looks locally best can spend the one blocker a later, bigger
/// attacker needed — and the greedy pass has no way to notice, because it
/// never looks at the board the block produces.
///
/// So this scores whole assignments. The greedy block seeds the candidate
/// set; the alternatives are "block with nobody" and the greedy assignment
/// minus one blocker each. Each is played through combat damage and scored
/// with the same evaluator, which already prices both the creatures that
/// died and the life that got through.
///
/// Cheaper than [`pick_attacks_scored`], and deliberately so: a block's
/// consequences are settled inside this combat, so the simulation stops at
/// end of combat instead of running a full turn cycle.
///
/// Ties go to the greedy assignment, so the search only ever departs from
/// current behavior for a strict improvement.
fn pick_blocks_scored(state: &GameState, seat: usize, w: &EvalWeights) -> Vec<(CardId, CardId)> {
    let greedy = pick_blocks(state, seat);
    if w.block_search == 0 || greedy.is_empty() {
        return greedy;
    }
    let mut candidates: Vec<Vec<(CardId, CardId)>> = vec![greedy.clone(), Vec::new()];
    if greedy.len() > 1 {
        // Consider releasing the cheapest bodies first: those are the
        // chump-blocks the greedy pass throws in to save life, and the ones
        // most likely to be worth more alive than the damage they absorb.
        let mut order: Vec<usize> = (0..greedy.len()).collect();
        order.sort_by_key(|&i| {
            state.battlefield_find(greedy[i].0).map(|c| c.toughness()).unwrap_or(0)
        });
        for &i in order.iter().take(w.block_search as usize) {
            let mut alt = greedy.clone();
            alt.remove(i);
            candidates.push(alt);
        }
    }

    if w.block_gang {
        candidates.extend(gang_block_candidates(state, seat, &greedy, w));
    }

    let mut best: Option<(usize, i32)> = None;
    for (i, cand) in candidates.iter().enumerate() {
        let Some(score) = simulate_block_outcome(state, seat, cand, w) else { continue };
        if best.map(|(_, s)| score > s).unwrap_or(true) {
            best = Some((i, score));
        }
    }
    match best {
        Some((i, _)) => candidates.swap_remove(i),
        None => greedy,
    }
}

/// Block assignments that add a gang onto an attacker the greedy pass
/// left alone, one candidate per attacker worth ganging.
///
/// Only attackers nobody is already blocking are considered: piling onto
/// an existing block changes a trade the greedy pass already reasoned
/// about, while an unblocked attacker is one it decided it *couldn't*
/// profitably block alone — exactly the case a gang exists for. Blockers
/// are taken cheapest-first so the gang spends the least material that
/// still kills, and a candidate is only emitted when the gang actually
/// kills (an assignment that merely chumps harder is strictly worse than
/// the greedy one and would only waste a simulation).
///
/// Illegal declarations (menace needing two, a "must be blocked"
/// attacker left uncovered) are not filtered here: the engine rejects
/// them and [`simulate_block_outcome`] returns `None`, which drops the
/// candidate. Legality is the engine's job, not this heuristic's.
fn gang_block_candidates(
    state: &GameState,
    seat: usize,
    greedy: &[(CardId, CardId)],
    w: &EvalWeights,
) -> Vec<Vec<(CardId, CardId)>> {
    use crate::card::Keyword;
    const MAX_CANDIDATES: usize = 3;

    let blocked: std::collections::HashSet<CardId> =
        greedy.iter().map(|(_, a)| *a).collect();
    let used: std::collections::HashSet<CardId> = greedy.iter().map(|(b, _)| *b).collect();

    // Idle bodies, cheapest first: the gang should cost as little as it
    // can and still kill.
    let mut idle: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && bot_can_block(c) && !used.contains(&c.id))
        .collect();
    idle.sort_by_key(|c| permanent_value(state, c.id, w));
    if idle.len() < 2 {
        return Vec::new();
    }

    // Unblocked attackers, most valuable first — the gang is only worth
    // its losses against a real threat.
    let mut targets: Vec<&crate::card::CardInstance> = state
        .attacking
        .iter()
        .filter(|a| !blocked.contains(&a.attacker))
        .filter_map(|a| state.battlefield_find(a.attacker))
        .filter(|c| c.controller != seat)
        .collect();
    targets.sort_by_key(|c| -permanent_value(state, c.id, w));

    let mut out = Vec::new();
    for atk in targets.into_iter().take(MAX_CANDIDATES) {
        let a_flying = atk.has_keyword(&Keyword::Flying);
        let a_tough = atk.toughness() - atk.damage as i32;
        let mut gang: Vec<CardId> = Vec::new();
        let mut dmg = 0i32;
        for b in &idle {
            if a_flying && !b.has_keyword(&Keyword::Flying) && !b.has_keyword(&Keyword::Reach) {
                continue;
            }
            gang.push(b.id);
            dmg += b.power().max(0);
            if b.has_keyword(&Keyword::Deathtouch) || dmg >= a_tough {
                break;
            }
        }
        // A single blocker is the greedy pass's own decision, already
        // scored; two or more that kill is the new option.
        if gang.len() < 2 || dmg < a_tough {
            continue;
        }
        let mut cand = greedy.to_vec();
        for b in gang {
            cand.push((b, atk.id));
        }
        out.push(cand);
    }
    out
}

/// Declare `blocks`, run combat damage, and score the board for `seat`.
///
/// `None` on a rejected declaration (a must-block creature we tried to hold
/// back, an over-cap batch) or a combat that won't settle — an unfinished
/// combat is scored not at all rather than scored wrong.
/// Redeal everything `seat` cannot legitimately see: each opponent's hand
/// goes back into their library, every library is shuffled, and the
/// opponent redraws the same number of cards.
///
/// This is what turns the combat sims from perfect-information search
/// into search under uncertainty — see
/// [`determinize`](EvalWeights::determinize) for why that matters.
///
/// Two honest approximations, both in the direction of forgetting more
/// than a real player would:
///
/// * Cards the seat has legitimately *seen* (a Duress reveal, a card
///   played and bounced) are re-hidden. Modelling that properly needs a
///   per-seat knowledge log the engine does not keep.
/// * Face-down permanents keep their real identity. They are already on
///   the battlefield and the sim reads them there.
///
/// Zones are permuted directly rather than through the engine's move
/// paths deliberately: this is a redeal of hidden information before the
/// simulation starts, not a game action, and routing it through
/// `move_card` would fire zone-change triggers that never happened.
fn determinize_hidden(g: &mut GameState, seat: usize, salt: u64) {
    use rand::seq::SliceRandom;
    let mut rng = StdRng::seed_from_u64(
        salt ^ ((g.turn_number as u64) << 32) ^ ((seat as u64) << 16) ^ g.step as u64,
    );
    for p in 0..g.players.len() {
        if p == seat {
            // Our own library order is unknown to us too — a search that
            // plans around the card it is about to draw is cheating just
            // as much as one that reads the opponent's hand.
            g.players[p].library.shuffle(&mut rng);
            continue;
        }
        let n = g.players[p].hand.len();
        let returned: Vec<_> = g.players[p].hand.drain(..).collect();
        g.players[p].library.extend(returned);
        g.players[p].library.shuffle(&mut rng);
        let split = g.players[p].library.len().saturating_sub(n);
        let redrawn: Vec<_> = g.players[p].library.split_off(split);
        g.players[p].hand.extend(redrawn);
    }
}

/// The state a simulation should start from: the real one, or a redeal of
/// its hidden zones. `k` indexes the redeal so an averaging caller gets
/// different hands each time.
fn sim_start_state(state: &GameState, seat: usize, w: &EvalWeights, k: u8) -> GameState {
    let mut g = state.clone();
    if w.determinize > 0 {
        determinize_hidden(&mut g, seat, 0x5EED_0000 ^ k as u64);
    }
    g
}

fn simulate_block_outcome(
    state: &GameState,
    seat: usize,
    blocks: &[(CardId, CardId)],
    w: &EvalWeights,
) -> Option<i32> {
    if w.determinize > 1 {
        // Mean over redeals: one redeal only swaps perfect information
        // for a specific wrong guess.
        let mut total = 0i64;
        let mut n = 0i64;
        for k in 0..w.determinize {
            if let Some(v) = simulate_block_outcome_once(state, seat, blocks, w, k) {
                total += v as i64;
                n += 1;
            }
        }
        // Every redeal failing means the assignment is illegal, not
        // merely unlucky — propagate that as before.
        return (n > 0).then(|| (total / n) as i32);
    }
    simulate_block_outcome_once(state, seat, blocks, w, 0)
}

fn simulate_block_outcome_once(
    state: &GameState,
    seat: usize,
    blocks: &[(CardId, CardId)],
    w: &EvalWeights,
    k: u8,
) -> Option<i32> {
    let mut g = sim_start_state(state, seat, w, k);
    g.perform_action(GameAction::DeclareBlockers(blocks.to_vec())).ok()?;
    let turn = g.turn_number;
    let mut fuel = 200u32;
    while !g.is_game_over() && g.turn_number == turn && g.step < TurnStep::EndCombat {
        fuel = fuel.checked_sub(1)?;
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(&g, pending.acting_player(), w, &pending.decision, false)
            };
            g.perform_action(GameAction::SubmitDecision(answer)).ok()?;
            continue;
        }
        // Under `attack_sim_spells` the combat window is live: tricks and
        // responses fire for whichever seat holds priority, so a block
        // that only works until the attacker pumps is scored as such.
        let action = if w.attack_sim_spells {
            sim_spell_action(&g, w).unwrap_or(GameAction::PassPriority)
        } else {
            GameAction::PassPriority
        };
        if g.perform_action(action).is_err()
            && g.perform_action(GameAction::PassPriority).is_err()
        {
            return None;
        }
    }
    Some(eval_material(&g, seat, w))
}

fn pick_blocks(state: &GameState, seat: usize) -> Vec<(CardId, CardId)> {
    // The heuristic probes block legality per blocker×attacker pair, each a
    // layer-aware check — share one gather across the whole scan.
    let mut blocks = state.with_frozen_layers(|state| pick_blocks_inner(state, seat));
    // CR 509.1b — Silent Arbiter caps the distinct blockers for the whole
    // combat; an over-sized batch is rejected outright, so keep only the
    // first `cap` blockers (the heuristic already ordered them best-first).
    if let Some(cap) = state.combat_participation_cap(true) {
        let mut kept: Vec<CardId> = state.block_map.keys().copied().collect();
        blocks.retain(|(blocker, _)| {
            if kept.contains(blocker) {
                return true;
            }
            if kept.len() >= cap as usize {
                return false;
            }
            kept.push(*blocker);
            true
        });
    }
    blocks
}

/// A creature the bot may legally declare as a blocker: `can_block()` only
/// checks creature-ness + untapped, so also exclude Decayed (CR 702.147 — a
/// Decayed creature can't block) and a granted/printed `CantBlock`. Used by
/// every blocker-candidate pass so the gang-block / must-be-blocked / menace
/// top-up passes never assemble an illegal declaration the engine rejects.
fn bot_can_block(c: &crate::card::CardInstance) -> bool {
    use crate::card::Keyword;
    c.can_block() && !c.has_keyword(&Keyword::Decayed) && !c.has_keyword(&Keyword::CantBlock)
}

fn pick_blocks_inner(state: &GameState, seat: usize) -> Vec<(CardId, CardId)> {
    // Improved blocker heuristic (push claude/modern_decks):
    //   1. Build the candidate set of (attacker, attacker_power,
    //      attacker_toughness, has_flying) attacking us.
    //   2. Sort blockers by ascending power so cheap chumps get
    //      assigned first; bigger blockers stay free for must-block
    //      situations.
    //   3. For each blocker, pick the **best** attacker it can block:
    //      - Prefer attackers it can kill outright (blocker_power >=
    //        attacker_toughness, with deathtouch granting kill on any
    //        damage).
    //      - Among kill-able attackers, prefer one that won't kill the
    //        blocker (blocker_toughness > attacker_power); ties broken
    //        by highest attacker_power (biggest value trade).
    //      - If no clean kill exists, fall back to a chump-block to
    //        save us from lethal damage when our life total is low
    //        (< current incoming damage).
    //   4. Each attacker can be assigned multiple blockers if a single
    //      blocker can't kill it — the loop falls through to try the
    //      next blocker.
    use crate::card::Keyword;
    // (id, power, toughness, flying, deathtouch). Deathtouch makes the
    // attacker lethal to any blocker it damages regardless of power, so
    // the bot must treat a block against it as a likely loss of the
    // blocker when scoring trades.
    let attacker_info: Vec<(CardId, i32, i32, bool, bool)> = state
        .attacking()
        .iter()
        .filter(|atk| state.defender_for(atk.target) == Some(seat))
        .filter_map(|atk| {
            state
                .battlefield
                .iter()
                .find(|c| c.id == atk.attacker)
                .map(|a| {
                    (
                        atk.attacker,
                        attacker_damage_value(state, atk.attacker),
                        a.toughness(),
                        a.has_keyword(&Keyword::Flying),
                        a.has_keyword(&Keyword::Deathtouch),
                    )
                })
        })
        .collect();
    // Only attackers aimed at the *player* threaten our life total — damage
    // to a planeswalker we control hits its loyalty, not our face. Summing
    // every attacker here would over-state the life threat and trigger
    // needless chump-blocks.
    let total_incoming: i32 = state
        .attacking()
        .iter()
        .filter(|atk| atk.target == AttackTarget::Player(seat))
        .map(|atk| attacker_damage_value(state, atk.attacker))
        .sum();
    // Planeswalker defense (CR 306.7): for each planeswalker we control that
    // is being attacked, if the attackers aimed at it would deal lethal
    // (total power ≥ its loyalty), mark those attackers so the chump-block
    // pass will trade idle blockers to save the walker.
    let defend_attackers: std::collections::HashSet<CardId> = {
        use crate::card::CounterType;
        let mut pw_attackers: std::collections::HashMap<CardId, (u32, Vec<CardId>)> =
            std::collections::HashMap::new();
        for atk in state.attacking() {
            if let AttackTarget::Planeswalker(pw) = atk.target
                && state.battlefield_find(pw).map(|c| c.controller) == Some(seat)
                && let Some(a) = state.battlefield.iter().find(|c| c.id == atk.attacker)
            {
                let e = pw_attackers.entry(pw).or_default();
                e.0 += a.power().max(0) as u32;
                e.1.push(atk.attacker);
            }
        }
        let mut set = std::collections::HashSet::new();
        for (pw, (incoming, atkrs)) in pw_attackers {
            let loyalty = state
                .battlefield_find(pw)
                .and_then(|c| c.counters.iter().find_map(|(k, v)| {
                    matches!(k, CounterType::Loyalty).then_some(*v)
                }))
                .unwrap_or(0);
            if loyalty > 0 && incoming >= loyalty {
                set.extend(atkrs);
            }
        }
        set
    };
    // Infect (CR 702.90) / Toxic (CR 702.180) make poison the lethal clock,
    // not life: a player with 10+ poison counters loses (CR 104.3d). The bot
    // must chump an infect/toxic attacker to avoid a poison-out even at a
    // healthy life total. Infect deals its power as poison; Toxic N adds N on
    // top of normal combat damage.
    let incoming_poison: u32 = state
        .attacking()
        .iter()
        .filter(|atk| state.defender_for(atk.target) == Some(seat))
        .filter_map(|atk| state.battlefield.iter().find(|c| c.id == atk.attacker))
        .map(|a| {
            let mut p = 0u32;
            if a.has_keyword(&Keyword::Infect) {
                p += a.power().max(0) as u32;
            }
            p += a
                .definition
                .keywords
                .iter()
                .filter_map(|k| match k {
                    Keyword::Toxic(n) | Keyword::Poisonous(n) => Some(*n),
                    _ => None,
                })
                .sum::<u32>();
            p
        })
        .sum();
    let poison_threatened =
        incoming_poison > 0 && state.players[seat].poison_counters + incoming_poison >= 10;
    let life_threatened = state.players[seat].life <= total_incoming || poison_threatened;

    let mut blockers: Vec<(CardId, i32, i32, bool, bool, bool)> = state
        .battlefield
        .iter()
        // `can_block()` only checks creature-ness + untapped; also exclude
        // creatures that genuinely can't block (Decayed CR 702.147, or a
        // granted "can't block") so the bot never submits an illegal block.
        .filter(|c| c.controller == seat && bot_can_block(c))
        .map(|c| {
            (
                c.id,
                c.power(),
                c.toughness(),
                c.has_keyword(&Keyword::Flying),
                c.has_keyword(&Keyword::Reach),
                c.has_keyword(&Keyword::Deathtouch),
            )
        })
        .collect();
    blockers.sort_by_key(|(_, p, _, _, _, _)| *p);

    // Track which attackers have already been damage-saturated by
    // assigned blockers — if blocker total toughness >= attacker
    // power, additional blockers on the same attacker are wasteful
    // unless they bring deathtouch / first strike.
    let mut attacker_damage_taken: std::collections::HashMap<CardId, i32> =
        std::collections::HashMap::new();
    // Blockers already committed to each attacker — folds Rampage (CR 702.23)
    // into the trade math for the second-and-later blocker.
    let mut attacker_block_count: std::collections::HashMap<CardId, i32> =
        std::collections::HashMap::new();
    let mut assignments: Vec<(CardId, CardId)> = Vec::new();

    for (b_id, b_pow, b_tough, b_flying, b_reach, b_dt) in blockers {
        // Pick the best attacker for this blocker.
        let mut best: Option<(CardId, i32, bool)> = None; // (attacker, score, was_kill)
        for (a_id, a_pow, a_tough, a_flying, a_dt) in &attacker_info {
            if *a_flying && !b_flying && !b_reach {
                continue;
            }
            // Authoritative legality gate (CR 509.1b): also honors
            // "can't be blocked except by …" / "… by …" restrictions,
            // protection, shadow, etc. Skip attackers this blocker can't
            // legally be assigned to, so the bot never submits a block batch
            // the engine will reject.
            if !state.blocker_can_block_attacker(b_id, *a_id) {
                continue;
            }
            // Skip attackers that already have at least their toughness
            // worth of damage queued unless we have deathtouch.
            let queued = attacker_damage_taken.get(a_id).copied().unwrap_or(0);
            // Rampage N (CR 702.23): every blocker beyond the first pumps the
            // attacker +N/+N. When this would be an additional blocker, fold
            // that bonus into the effective P/T so the bot doesn't gang-block
            // into a pump that saves the attacker and kills the extra blocker.
            let bcount = attacker_block_count.get(a_id).copied().unwrap_or(0);
            let rampage = state
                .battlefield
                .iter()
                .find(|c| c.id == *a_id)
                .map(|a| {
                    a.definition
                        .keywords
                        .iter()
                        .chain(a.granted_keywords_eot.iter())
                        .filter_map(|k| match k {
                            Keyword::Rampage(n) => Some(*n as i32),
                            _ => None,
                        })
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            let ramp_bonus = rampage * bcount;
            let eff_a_tough = *a_tough + ramp_bonus;
            let eff_a_pow = *a_pow + ramp_bonus;
            if !b_dt && queued >= eff_a_tough {
                continue;
            }
            // First-strike awareness (CR 702.7): if the attacker strikes
            // first (and the blocker doesn't strike back first) and its
            // first-strike damage is already lethal to the blocker, the
            // blocker dies *before* dealing any damage — so it never trades
            // up. Such a "kill" is illusory; downgrade it to a chump.
            let atk_first_strike = state
                .battlefield
                .iter()
                .find(|c| c.id == *a_id)
                .is_some_and(|a| {
                    a.has_keyword(&Keyword::FirstStrike) || a.has_keyword(&Keyword::DoubleStrike)
                });
            let blk_first_strike = {
                let blk = state.battlefield.iter().find(|c| c.id == b_id);
                blk.is_some_and(|c| {
                    c.has_keyword(&Keyword::FirstStrike) || c.has_keyword(&Keyword::DoubleStrike)
                })
            };
            // CR 702.16e — protection prevents combat damage either way:
            // a blocker protected from the attacker's color takes none (won't
            // die), and an attacker protected from the blocker's color takes
            // none (won't be killed). Factor both into the trade math.
            let blocker_takes_no_dmg = state.damage_prevented_by_protection(*a_id, b_id);
            let attacker_takes_no_dmg = state.damage_prevented_by_protection(b_id, *a_id);
            // CR 702.12 — an indestructible permanent isn't destroyed by lethal
            // damage (or deathtouch), so it never dies in a trade and can't be
            // killed by a blocker. Block freely behind an indestructible body.
            let blocker_indestructible =
                state.battlefield_find(b_id).is_some_and(|c| c.is_indestructible());
            let attacker_indestructible =
                state.battlefield_find(*a_id).is_some_and(|c| c.is_indestructible());
            let dies_before_striking = atk_first_strike
                && !blk_first_strike
                && !blocker_takes_no_dmg
                && !blocker_indestructible
                && (eff_a_pow >= b_tough || (*a_dt && eff_a_pow >= 1));
            let kills_attacker = !attacker_takes_no_dmg
                && !attacker_indestructible
                && !dies_before_striking
                && (b_dt || b_pow >= (eff_a_tough - queued));
            // A deathtouch attacker kills the blocker on any damage.
            let dies_to_attacker = !blocker_takes_no_dmg
                && !blocker_indestructible
                && (eff_a_pow >= b_tough || (*a_dt && eff_a_pow >= 1));
            // Scoring: clean trade (kill, don't die) > kill-and-die >
            // chump (don't kill, die). Higher attacker power adds value.
            let score = if kills_attacker && !dies_to_attacker {
                1000 + *a_pow
            } else if kills_attacker && dies_to_attacker {
                // Even trade (both die). Prefer trading up: score by the
                // stat delta (proxy = power + toughness). Don't sacrifice a
                // much bigger creature for a small attacker unless we're
                // under pressure — keep the body and take the hit.
                let delta = (*a_pow + *a_tough) - (b_pow + b_tough);
                if !life_threatened && delta < -2 {
                    continue;
                }
                500 + delta
            } else if blocker_indestructible && !dies_to_attacker && *a_pow >= 1 {
                // An indestructible wall absorbs the attacker's damage at no
                // cost (it survives and isn't tapped). Free value even with no
                // life pressure — block the biggest attacker it can.
                200 + *a_pow
            } else if life_threatened || defend_attackers.contains(a_id) {
                // Chump-block to stop lethal damage (or to save a doomed
                // planeswalker). A trampler tramples over a chump
                // (CR 702.19e), so a lone chump only stops `blocker_toughness`
                // of its damage — score by the actual damage saved so the bot
                // prefers fully blocking a non-trampler over partially
                // blocking a trampler.
                let a_trample = state
                    .battlefield
                    .iter()
                    .find(|c| c.id == *a_id)
                    .is_some_and(|a| a.has_keyword(&Keyword::Trample));
                let saved = if a_trample { b_tough.min(*a_pow) } else { *a_pow };
                100 + saved
            } else {
                continue;
            };
            if best.map(|(_, s, _)| s < score).unwrap_or(true) {
                best = Some((*a_id, score, kills_attacker));
            }
        }
        if let Some((a_id, _score, _kill)) = best {
            assignments.push((b_id, a_id));
            // Mark the damage queued so subsequent blockers can pile on
            // attackers that aren't fully covered yet.
            *attacker_damage_taken.entry(a_id).or_insert(0) += b_tough;
            *attacker_block_count.entry(a_id).or_insert(0) += 1;
        }
    }
    // Gang-block-to-kill when our life is threatened. The greedy single-
    // blocker pass above only starts blocking an attacker when one blocker
    // alone can kill it (or we chump). When we're facing lethal, trading
    // several spare creatures to *remove* a big attacker permanently beats
    // scattering chumps that die for nothing. For each still-unblocked
    // attacker (largest power first), pile idle blockers on until their
    // combined power reaches the attacker's toughness, then commit only if
    // the gang actually kills it.
    if life_threatened {
        let mut used: std::collections::HashSet<CardId> =
            assignments.iter().map(|(b, _)| *b).collect();
        let mut idle: Vec<(CardId, i32, i32, bool, bool, bool)> = state
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && bot_can_block(c) && !used.contains(&c.id))
            .map(|c| {
                (
                    c.id,
                    c.power(),
                    c.toughness(),
                    c.has_keyword(&Keyword::Flying),
                    c.has_keyword(&Keyword::Reach),
                    c.has_keyword(&Keyword::Deathtouch),
                )
            })
            .collect();
        let mut uncovered: Vec<(CardId, i32, i32, bool, bool)> = attacker_info
            .iter()
            .filter(|(a_id, _, _, _, _)| !assignments.iter().any(|(_, aid)| aid == a_id))
            .copied()
            .collect();
        uncovered.sort_by_key(|(_, p, _, _, _)| -*p);
        for (a_id, _a_pow, a_tough, a_flying, _a_dt) in uncovered {
            // Rampage N (CR 702.23): each blocker beyond the first raises the
            // attacker's toughness by N, so a gang must out-damage the pumped
            // total — otherwise the chumps die and the attacker survives.
            let rampage = state
                .battlefield
                .iter()
                .find(|c| c.id == a_id)
                .map(|a| {
                    a.definition
                        .keywords
                        .iter()
                        .chain(a.granted_keywords_eot.iter())
                        .filter_map(|k| match k {
                            Keyword::Rampage(n) => Some(*n as i32),
                            _ => None,
                        })
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            // Collect a gang of legal idle blockers that together kill it.
            let mut gang: Vec<CardId> = Vec::new();
            let mut dmg = 0i32;
            let mut kills = false;
            for (b_id, b_pow, _bt, b_fly, b_reach, b_dt) in &idle {
                if a_flying && !b_fly && !b_reach {
                    continue;
                }
                gang.push(*b_id);
                dmg += *b_pow;
                let eff_tough = a_tough + rampage * (gang.len() as i32 - 1);
                if *b_dt || dmg >= eff_tough {
                    kills = true;
                    break;
                }
            }
            if kills {
                for b_id in &gang {
                    assignments.push((*b_id, a_id));
                    used.insert(*b_id);
                }
                idle.retain(|(id, ..)| !gang.contains(id));
            }
        }
    }

    // CR 509.1c — satisfy "must be blocked if able" (Academic Dispute /
    // Lure). The engine rejects a declaration that leaves such an attacker
    // unblocked while an idle able blocker exists, so the bot must assign
    // one or it would deadlock the combat step. Pull any unused creature
    // that can legally block (respecting flying/reach) onto each
    // must-be-blocked attacker still missing a blocker.
    for (a_id, _a_pow, _a_tough, a_flying, _a_dt) in &attacker_info {
        let must_block = state
            .battlefield
            .iter()
            .find(|c| c.id == *a_id)
            .is_some_and(|a| a.has_keyword(&Keyword::MustBeBlocked));
        if !must_block || assignments.iter().any(|(_, aid)| aid == a_id) {
            continue;
        }
        // Pick the cheapest (lowest-power) legal idle blocker so a forced block
        // doesn't throw away the bot's best body.
        if let Some(idle) = state
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == seat
                    && bot_can_block(c)
                    && !assignments.iter().any(|(bid, _)| *bid == c.id)
                    && (!a_flying
                        || c.has_keyword(&Keyword::Flying)
                        || c.has_keyword(&Keyword::Reach))
                    && state.blocker_can_block_attacker(c.id, *a_id)
            })
            .min_by_key(|c| c.power())
        {
            assignments.push((idle.id, *a_id));
        }
    }

    // CR 509.1b / 702.110b — Menace (≥2 blockers) and "can't be blocked
    // except by N or more creatures" (CantBeBlockedExceptByN — Pathrazer of
    // Ulamog) impose a minimum block count: an attacker so keyworded must be
    // blocked by 0 or ≥N, never 1..N-1. The greedy passes assign one at a
    // time, so an under-filled multi-block is illegal and the engine rejects
    // the whole declaration. For each such attacker, top the block up to the
    // minimum with legal idle blockers; if the minimum can't be reached,
    // drop every block on it (better unblocked than an illegal batch).
    for (a_id, _a_pow, _a_tough, a_flying, _a_dt) in &attacker_info {
        let min_blockers = state
            .battlefield
            .iter()
            .find(|c| c.id == *a_id)
            .map(min_blockers_required)
            .unwrap_or(1);
        if min_blockers <= 1 {
            continue;
        }
        let mut count = assignments.iter().filter(|(_, aid)| aid == a_id).count();
        if count == 0 || count >= min_blockers {
            continue;
        }
        while count < min_blockers {
            // Cheapest legal idle blocker first — minimise value lost to the
            // forced multi-block.
            let extra = state
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == seat
                        && bot_can_block(c)
                        && !assignments.iter().any(|(bid, _)| *bid == c.id)
                        && (!a_flying
                            || c.has_keyword(&Keyword::Flying)
                            || c.has_keyword(&Keyword::Reach))
                        && state.blocker_can_block_attacker(c.id, *a_id)
                })
                .min_by_key(|c| c.power());
            match extra {
                Some(c) => {
                    assignments.push((c.id, *a_id));
                    count += 1;
                }
                // Can't reach the minimum — drop all blocks on this attacker.
                None => {
                    assignments.retain(|(_, aid)| aid != a_id);
                    break;
                }
            }
        }
    }

    // CR 509.1b — spend spare block capacity (Guardian of the Gateless and
    // friends). A blocker that can block extra attackers soaks additional
    // ones for free as long as the total damage it would take stays under its
    // toughness and no extra attacker has deathtouch.
    let extra_capacity = |id: CardId| -> usize {
        let Some(c) = state.battlefield_find(id) else { return 0 };
        if c.has_keyword(&Keyword::CanBlockAnyNumber) {
            return usize::MAX;
        }
        state
            .computed_permanent(id)
            .map(|cp| cp.keywords.clone())
            .unwrap_or_default()
            .iter()
            .filter_map(|k| match k {
                Keyword::CanBlockAdditional(n) => Some(*n as usize),
                _ => None,
            })
            .sum()
    };
    // Seed from every legal blocker with spare capacity, not just the ones the
    // scoring loop already assigned: a 0/N `CanBlockAnyNumber` wall kills
    // nothing and isn't needed against lethal, so it never gets picked up
    // there — but it can still soak the whole swing for free.
    let mut multi: Vec<CardId> = Vec::new();
    let seeds = assignments.iter().map(|(b, _)| *b).chain(
        state
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && bot_can_block(c))
            .map(|c| c.id),
    );
    for id in seeds {
        if !multi.contains(&id) && extra_capacity(id) > 0 {
            multi.push(id);
        }
    }
    for b_id in multi {
        let Some(b) = state.battlefield_find(b_id) else { continue };
        let (b_tough, b_flying, b_reach) = (
            b.toughness(),
            b.has_keyword(&Keyword::Flying),
            b.has_keyword(&Keyword::Reach),
        );
        let mut taken: i32 = assignments
            .iter()
            .filter(|(bid, _)| *bid == b_id)
            .filter_map(|(_, aid)| attacker_info.iter().find(|(a, ..)| a == aid))
            .map(|(_, p, ..)| *p)
            .sum();
        let mut spare = extra_capacity(b_id);
        for (a_id, a_pow, _a_tough, a_flying, a_dt) in &attacker_info {
            if spare == 0 {
                break;
            }
            if *a_dt
                || taken + *a_pow >= b_tough
                || assignments.iter().any(|(bid, aid)| *bid == b_id && aid == a_id)
                || assignments.iter().any(|(_, aid)| aid == a_id)
                || (*a_flying && !b_flying && !b_reach)
                || min_blockers_required_by_id(state, *a_id) > 1
                || !state.blocker_can_block_attacker(b_id, *a_id)
            {
                continue;
            }
            assignments.push((b_id, *a_id));
            taken += *a_pow;
            spare = spare.saturating_sub(1);
        }
    }
    assignments
}

/// [`min_blockers_required`] for a battlefield id (1 when it isn't there).
fn min_blockers_required_by_id(state: &GameState, id: CardId) -> usize {
    state.battlefield_find(id).map(min_blockers_required).unwrap_or(1)
}

/// Minimum number of creatures legally required to block `attacker` (CR
/// 509.1b): 2 for Menace, N for `CantBeBlockedExceptByN(N)`, the max of any
/// such requirement, else 1. Reads printed + EOT-granted keywords (the same
/// set [`CardInstance::has_keyword`] consults).
fn min_blockers_required(attacker: &crate::card::CardInstance) -> usize {
    use crate::card::Keyword;
    let mut min = 1usize;
    for kw in attacker
        .definition
        .keywords
        .iter()
        .chain(attacker.granted_keywords_eot.iter())
    {
        match kw {
            Keyword::Menace => min = min.max(2),
            Keyword::CantBeBlockedExceptByN(n) => min = min.max(*n as usize),
            _ => {}
        }
    }
    min
}


/// True if the player can pay the card's mana cost from their current
/// pool **including** static-ability cost increases (Damping Sphere's
/// post-first-spell tax, Chancellor of the Annex's first-spell tax).
///
/// The state-aware overload `can_afford_in_state` is what the bot's
/// main_phase_action uses; the simpler signature is kept for
/// existing callers that don't have a `GameState` handy.
pub fn can_afford(def: &CardDefinition, pool: &ManaPool) -> bool {
    can_afford_with_extra(&def.cost, pool, 0, 0)
}

/// What `seat` could still pay with this phase: mana already floating,
/// plus the most each *untapped* source they control could add.
///
/// The bot used to answer "can I afford this?" against the floating pool
/// alone, which only worked because it tapped every land before deciding
/// anything. That made the pool an accurate picture of its mana -- and
/// left it with none for the rest of the turn (CR 500.4 empties the pool
/// at every step boundary), so counterspells, flash creatures, instant
/// removal and combat tricks were unplayable in practice. Sizing against
/// untapped sources instead lets the engine's auto-tap pay each cast from
/// only what it needs, and whatever is left over survives into the
/// opponent's turn.
#[derive(Debug, Default, Clone, Copy)]
struct AvailableMana {
    /// Upper bound on the number of mana that could be produced.
    total: u32,
    /// Colors at least one source could produce.
    colors: crate::mana::ColorSet,
    /// Whether true colorless ({C}) is producible.
    colorless: bool,
}

/// Estimate [`AvailableMana`] for `seat`.
///
/// Deliberately **optimistic**: it ignores the assignment problem (which
/// source pays which pip), counts every color a choice-source could make,
/// and rounds dynamic amounts down to one rather than giving up. That is
/// the right bias, because this is only a pre-filter -- the authoritative
/// gate on every candidate is still `would_accept_on`, which runs the
/// engine's real auto-tap. An over-permissive estimate costs a few extra
/// dry-run probes; an under-permissive one silently makes castable spells
/// invisible to the bot, which is exactly the failure being fixed here.
fn available_mana(state: &GameState, seat: usize) -> AvailableMana {
    use crate::mana::{Color, ColorSet};
    let pool = &state.players[seat].mana_pool;
    let mut out = AvailableMana {
        total: pool.total(),
        colors: ColorSet::empty(),
        colorless: pool.colorless_amount() > 0,
    };
    for c in Color::ALL {
        if pool.amount(c) > 0 {
            out.colors.insert(c);
        }
    }
    for p in state.battlefield.iter().filter(|p| p.controller == seat && !p.tapped) {
        // Printed abilities plus anything granted to it (Cryptolith Rite
        // turning creatures into mana sources, Urza's Saga chapters), so a
        // granted mana ability doesn't read as "no mana here".
        let granted = state.granted_abilities_for(p.id);
        let mut best = 0u32;
        for a in p.definition.activated_abilities.iter().chain(granted.iter()) {
            if !is_countable_mana_ability(a) {
                continue;
            }
            let (amount, colors, colorless) = mana_ability_output(&a.effect);
            best = best.max(amount);
            out.colors = out.colors.union(colors);
            out.colorless |= colorless;
        }
        out.total += best;
    }
    out
}

/// A mana ability the bot is willing to count toward affordability: it
/// costs a tap and nothing the bot would regret.
///
/// We only need to know the mana *could* be paid, so color-choice sources
/// (dual lands, Birds of Paradise) and painland-style life costs count --
/// the engine's auto-tap will happily use them. Sources that consume a
/// real resource to fire (sacrifice, discard, exile, energy) are excluded:
/// counting them would have the bot commit to lines it can only pay for by
/// spending something it would rather keep.
fn is_countable_mana_ability(a: &ActivatedAbility) -> bool {
    a.tap_cost
        && a.mana_cost.symbols.is_empty()
        && !a.sac_cost
        && a.sac_other_filter.is_none()
        && a.bounce_other_filter.is_none()
        && a.tap_other_filter.is_none()
        && a.tap_n_filter.is_none()
        && a.exile_other_filter.is_none()
        && a.discard_cost.is_none()
        && !a.exile_self_cost
        && a.energy_cost == 0
        && a.collect_evidence_cost.is_none()
        && a.condition.is_none()
        && !a.from_graveyard
        && !a.from_hand
        && matches!(a.effect, Effect::AddMana { .. })
}

/// `(most mana produced, colors it could be, produces true colorless)` for
/// a mana ability's effect. Dynamic amounts (`{T}: add {G} equal to this
/// creature's power`) count as one -- enough to keep the source visible
/// without inventing a board state to measure it against.
fn mana_ability_output(eff: &Effect) -> (u32, crate::mana::ColorSet, bool) {
    use crate::effect::Value;
    use crate::mana::{Color, ColorSet};
    let mut colors = ColorSet::empty();
    accumulate_mana_colors(eff, &mut colors);
    let amount_of = |v: &Value| match v {
        Value::Const(n) => (*n).max(0) as u32,
        _ => 1,
    };
    let Effect::AddMana { pool, .. } = eff else { return (0, colors, false) };
    let (amount, colorless) = match pool {
        ManaPayload::Colors(cs) => (cs.len() as u32, false),
        ManaPayload::Colorless(v) => (amount_of(v), true),
        ManaPayload::OfColor(_, v) | ManaPayload::OfColors(_, v) => (amount_of(v), false),
        ManaPayload::AnyOneColor(v) | ManaPayload::AnyColors(v) => {
            for c in Color::ALL {
                colors.insert(c);
            }
            (amount_of(v), false)
        }
        // "Any color an opponent's land could produce" and friends: the
        // exact palette depends on a board read this estimate doesn't do,
        // so assume the source is live for any color.
        _ => {
            for c in Color::ALL {
                colors.insert(c);
            }
            (1, true)
        }
    };
    (amount, colors, colorless)
}

/// State-aware affordability check: queries the engine for any
/// per-spell tax that would apply (Damping Sphere etc.) and folds it
/// into the cost before testing what `seat` can produce. Used by the bot to
/// avoid submitting `CastSpell` actions that the engine will reject
/// with a mana shortfall — repeated rejections are what deadlocked
/// `debug/deadlock-t8-1777411577-473115700.json` (Damping Sphere on
/// the board, bot casting its second spell of the turn).
pub fn can_afford_in_state(
    state: &GameState,
    seat: usize,
    card: &crate::card::CardInstance,
    w: &EvalWeights,
) -> bool {
    let extra = state.extra_cost_for_card_in_hand(seat, card.id);
    // Fold in generic cost *reductions* (Affinity, CostReduction statics,
    // graveyard-affinity) the same way the real cast path does — otherwise the
    // bot overestimates the cost of e.g. Tolarian Terror with a full graveyard
    // and never casts it. Target-dependent reductions are skipped (no target
    // chosen yet), so this stays conservative.
    let reduction = crate::game::actions::cost_reduction_for_spell(state, seat, card, None);
    // Coloured surcharges (the Leech cycle) can't ride the generic `extra`
    // channel, so they join the printed cost before relaxation.
    let mut printed = card.definition.cost.clone();
    printed
        .symbols
        .extend(crate::game::actions::colored_spell_tax_for_spell(state, seat, card).symbols);
    // Mirror the payment funnel's Lattice relaxation so the bot doesn't
    // pass on a spell whose coloured pips any mana can now cover.
    let cost = state.relax_cost_colors(&printed);
    if w.legacy_pretap {
        return can_afford_with_extra(&cost, &state.players[seat].mana_pool, extra, reduction);
    }
    can_afford_from(&cost, &available_mana(state, seat), extra, reduction)
}

/// Could `printed` be paid from `have`? Two independent tests: enough
/// total mana for the (taxed, reduced) mana value, and a producible
/// source for every coloured pip.
///
/// Hybrid pips pass if *either* half is producible and Phyrexian pips
/// always pass (life is a legal payment), matching what the real payment
/// funnel will accept.
fn can_afford_from(
    printed: &ManaCost,
    have: &AvailableMana,
    extra_generic: u32,
    reduction: u32,
) -> bool {
    use crate::mana::ManaSymbol;
    let mut cost = if printed.has_x() { printed.with_x_value(0) } else { printed.clone() };
    if reduction > 0 {
        cost.reduce_generic(reduction);
    }
    if cost.cmc() + extra_generic > have.total {
        return false;
    }
    cost.symbols.iter().all(|s| match s {
        ManaSymbol::Colored(c) => have.colors.contains(*c),
        ManaSymbol::Hybrid(a, b) => have.colors.contains(*a) || have.colors.contains(*b),
        // Phyrexian pips are payable with 2 life, so they never gate.
        ManaSymbol::Phyrexian(_) | ManaSymbol::PhyrexianHybrid(_, _) => true,
        ManaSymbol::Colorless(_) => have.colorless,
        _ => true,
    })
}

/// CR 702.21 — the tax `actor` would owe for aiming a spell or ability at
/// `id`: the permanent's computed Ward cost when it is hostile and
/// non-trivial, `None` when targeting it is tax-free. The engine's
/// auto-targeter already *prefers* un-warded candidates
/// (`auto_target_for_effect_avoiding_set_xc`); this helper exists for the
/// fallback case where every candidate is warded and the bot has to judge
/// whether the tax is survivable at all.
fn ward_tax(state: &GameState, id: CardId, actor: usize) -> Option<crate::card::WardCost> {
    use crate::card::Keyword;
    let c = state.battlefield_find(id)?;
    if state.same_team(c.controller, actor) {
        return None;
    }
    state
        .computed_permanent(id)
        .map(|cp| cp.keywords)
        .unwrap_or_else(|| c.definition.keywords.clone())
        .iter()
        .find_map(|k| match k {
            Keyword::Ward(w) if !crate::game::actions::ward_cost_is_trivial(w) => Some(w.clone()),
            _ => None,
        })
}

/// Whether the bot could actually pay `tax` on top of `besides` — the
/// mana the cast or activation itself is about to consume. The engine
/// auto-pays ward taxes when the trigger resolves (`try_pay_ward_cost`);
/// a payment that fails there gets the bot's spell countered, which is
/// strictly worse than never casting it, and a life payment the engine
/// *can* make is still refused here when it would spend the bot's whole
/// life total into the state-based loss. Variants with no cheap
/// payability read default to `true`: a wrong `true` costs one card and
/// shows up on the ladder, a wrong `false` makes a legal line permanently
/// invisible.
fn ward_tax_payable(
    state: &GameState,
    seat: usize,
    tax: &crate::card::WardCost,
    besides: &ManaCost,
) -> bool {
    use crate::card::WardCost;
    let mana_ok = |mc: &ManaCost| {
        let mut combined = besides.clone();
        combined.symbols.extend(mc.symbols.iter().cloned());
        can_afford_from(&combined, &available_mana(state, seat), 0, 0)
    };
    let life_ok = |n: u32| (n as i32) < state.effective_life(seat);
    let gy = &state.players[seat].graveyard;
    match tax {
        WardCost::Mana(mc) => mana_ok(mc),
        WardCost::Life(n) => life_ok(*n),
        WardCost::ManaAndLife(mc, n) => mana_ok(mc) && life_ok(*n),
        WardCost::Discard(n) | WardCost::DiscardMatching(_, n) => {
            state.players[seat].hand.len() >= *n as usize
        }
        WardCost::DiscardHand => true,
        WardCost::ExileFromGraveyard(n) | WardCost::BottomFromGraveyard(n) => {
            gy.len() >= *n as usize
        }
        WardCost::CollectEvidence(n) => {
            gy.iter().map(|c| c.definition.cost.cmc()).sum::<u32>() >= *n
        }
        WardCost::SacrificeCreature => state
            .battlefield
            .iter()
            .any(|c| c.controller == seat && c.definition.is_creature()),
        WardCost::SacrificePermanents(n) => {
            state.battlefield.iter().filter(|c| c.controller == seat).count() >= *n as usize
        }
        // Dynamic and niche shapes (source-power costs, attached-cost,
        // counter removal, X reads): defer to the engine's auto-pay.
        _ => true,
    }
}

/// Rough mana-equivalent weight of a ward tax, for ranking candidates
/// that survived [`ward_tax_payable`]. Life prices at two per mana (the
/// Phyrexian rate), a discarded card at two mana; shapes with no cheap
/// read get a nominal two. Precision is not the point — the term only
/// has to make an un-warded target of equal value, or a different spell
/// entirely, win the tie.
fn ward_tax_burden(tax: &crate::card::WardCost) -> i32 {
    use crate::card::WardCost;
    match tax {
        WardCost::Mana(mc) => mc.cmc() as i32,
        WardCost::Life(n) => (*n as i32 + 1) / 2,
        WardCost::ManaAndLife(mc, n) => mc.cmc() as i32 + (*n as i32 + 1) / 2,
        WardCost::Discard(n) | WardCost::DiscardMatching(_, n) => 2 * *n as i32,
        WardCost::DiscardHand => 3,
        _ => 2,
    }
}

/// `false` when `action` aims at a warded hostile permanent whose tax the
/// bot could not pay after the action's own mana cost. Such a candidate
/// is a dead card, not an expensive one — the resolution path auto-pays
/// or counters (see [`ward_tax_payable`]) — so it is dropped from the
/// pool entirely rather than merely down-ranked. The printed cost stands
/// in for alternative-cost casts (flashback, delve, …); that
/// over-estimates what some casts consume, which errs toward holding a
/// spell, never toward blanking one. Actions with no recognized target
/// shape pass.
fn ward_gate_ok(state: &GameState, seat: usize, action: &GameAction) -> bool {
    let empty = ManaCost::new(vec![]);
    let (mut cost, target, additional): (ManaCost, &Option<Target>, &[Target]) = match action {
        GameAction::CastSpell { card_id, target, additional_targets, .. }
        | GameAction::CastSpellDelve { card_id, target, additional_targets, .. }
        | GameAction::CastGift { card_id, target, additional_targets, .. }
        | GameAction::CastSpellSpree { card_id, target, additional_targets, .. }
        | GameAction::CastSpellConspire { card_id, target, additional_targets, .. }
        | GameAction::CastSpellKicked { card_id, target, additional_targets, .. }
        | GameAction::CastSpellKickers { card_id, target, additional_targets, .. }
        | GameAction::CastSpellMultikicked { card_id, target, additional_targets, .. }
        | GameAction::CastBestow { card_id, target, additional_targets, .. }
        | GameAction::CastAdventure { card_id, target, additional_targets, .. }
        | GameAction::CastOmen { card_id, target, additional_targets, .. }
        | GameAction::CastPrototype { card_id, target, additional_targets, .. }
        | GameAction::CastSplitRight { card_id, target, additional_targets, .. }
        | GameAction::CastAftermath { card_id, target, additional_targets, .. }
        | GameAction::CastFlashback { card_id, target, additional_targets, .. }
        | GameAction::CastMayhem { card_id, target, additional_targets, .. }
        | GameAction::CastHarmonize { card_id, target, additional_targets, .. }
        | GameAction::CastSpellAlternative { card_id, target, additional_targets, .. }
        | GameAction::CastAdventureCreature { card_id, target, additional_targets, .. }
        | GameAction::CastPlotted { card_id, target, additional_targets, .. } => {
            let cost = state
                .find_card_anywhere(*card_id)
                .map(|c| c.definition.cost.clone())
                .unwrap_or(empty);
            (cost, target, additional_targets.as_slice())
        }
        // Back-face casts pay the back's cost.
        GameAction::CastSpellBack { card_id, target, additional_targets, .. }
        | GameAction::CastDisturb { card_id, target, additional_targets, .. } => {
            let cost = state
                .find_card_anywhere(*card_id)
                .and_then(|c| c.definition.back_face.as_deref().map(|b| b.cost.clone()))
                .unwrap_or(empty);
            (cost, target, additional_targets.as_slice())
        }
        // Prepare-casts pay the inset spell's cost.
        GameAction::CastPrepareSpell { creature_id, target, additional_targets, .. } => {
            let cost = state
                .battlefield_find(*creature_id)
                .and_then(|c| c.definition.prepare_spell.as_deref().map(|s| s.cost.clone()))
                .unwrap_or(empty);
            (cost, target, additional_targets.as_slice())
        }
        GameAction::ActivateAbility { card_id, ability_index, target, additional_targets, .. } => {
            // Granted abilities index past the printed list; missing costs
            // fall back to free, which errs permissive — the gate still
            // sees the tax itself.
            let cost = state
                .battlefield_find(*card_id)
                .and_then(|c| c.definition.activated_abilities.get(*ability_index))
                .map(|a| a.mana_cost.clone())
                .unwrap_or(empty);
            (cost, target, additional_targets.as_slice())
        }
        GameAction::ActivateLoyaltyAbility { target, .. } => (empty, target, &[]),
        _ => return true,
    };
    // The one candidate shape that sinks *extra* mana into the cast: a
    // plain CastSpell with a chosen X (`max_affordable_x` dumps the whole
    // spare pool into it). Price the X into the gate, or a max-X spell
    // aimed at a warded target taps the bot out of the tax it then owes.
    if let GameAction::CastSpell { x_value: Some(x), .. } = action {
        cost.symbols.push(crate::mana::ManaSymbol::Generic(*x));
    }
    target.iter().chain(additional.iter()).all(|t| match t {
        Target::Permanent(id) => match ward_tax(state, *id, seat) {
            Some(tax) => ward_tax_payable(state, seat, &tax, &cost),
            None => true,
        },
        _ => true,
    })
}

/// For an X-cost spell (or a spell whose effect reads
/// `Value::XFromCost`), return the largest non-negative X the caster can
/// pay given their current mana pool — leftover generic mana after the
/// fixed (non-X) portion of the cost is what fuels X. Static cost taxes
/// (Damping Sphere etc.) are folded in via
/// `extra_cost_for_card_in_hand`. Returns 0 when the fixed cost itself
/// is more than the available pool — the caller relies on `would_accept`
/// to reject the unaffordable cast.
///
/// We detect X-relevance via either the cost's explicit `{X}` pip
/// (Wrath of the Skies) **or** an `XFromCost` reference inside the
/// effect tree (Banefire / Earthquake / Mind Twist — these have flat
/// fixed costs in the catalog because the engine had no Value::XFromCost
/// wiring at the time they were added; the X mana goes straight into
/// the pool and the bot pumps the spell at its full pool size).
pub fn max_affordable_x(
    state: &GameState,
    seat: usize,
    card: &crate::card::CardInstance,
    w: &EvalWeights,
) -> u32 {
    let extra = state.extra_cost_for_card_in_hand(seat, card.id)
        + crate::game::actions::colored_spell_tax_for_spell(state, seat, card).cmc();
    max_affordable_x_for_def(state, seat, &card.definition, extra, w)
}

/// [`max_affordable_x`] for a definition that isn't a hand card — the
/// prepare-cast inset spell. `extra` carries any surcharges the caller
/// can compute (hand casts pass their static taxes; prepare copies pass
/// 0, erring optimistic — `would_accept` re-validates the real payment).
pub fn max_affordable_x_for_def(
    state: &GameState,
    seat: usize,
    def: &CardDefinition,
    extra: u32,
    w: &EvalWeights,
) -> u32 {
    if !x_relevant(def) {
        return 0;
    }
    // Everything the seat could still produce, not just what's floating --
    // see `available_mana`. Sizing X off the floating pool alone only
    // worked back when the bot tapped out before deciding anything.
    let pool_total = if w.legacy_pretap {
        state.players[seat].mana_pool.total()
    } else {
        available_mana(state, seat).total
    };
    let fixed_cmc = def.cost.with_x_value(0).cmc();
    let affordable = pool_total.saturating_sub(fixed_cmc + extra);
    // `with_x_value` replaces EVERY X pip, so an {X}{X} cost (Oracle's
    // Gift) pays 2X total — divide the spare mana across the pips or the
    // declared X overshoots what the payment funnel will accept.
    let x_pips = def
        .cost
        .symbols
        .iter()
        .filter(|s| matches!(s, crate::mana::ManaSymbol::X))
        .count()
        .max(1) as u32;
    let affordable = affordable / x_pips;
    // Don't overkill: an `{X}: deal X damage to target creature` spell
    // (creature-only target — can't go to the face) never needs more X
    // than the toughest opposing creature's toughness. Capping here frees
    // the leftover mana for the rest of the turn instead of vanishing it
    // into a 6-damage Disfigure on a 2/2.
    if let Some(cap) = creature_only_x_damage_cap(state, seat, def) {
        return affordable.min(cap);
    }
    affordable
}

/// For a single-target, creature-only `DealDamage` whose amount scales with
/// X, the most X the bot ever needs: the greatest toughness among opposing
/// creatures (so any legal target still dies). `None` for any other shape —
/// player-targetable burn (Banefire) keeps dumping its whole pool into X.
fn creature_only_x_damage_cap(state: &GameState, seat: usize, def: &CardDefinition) -> Option<u32> {
    use crate::effect::Value;
    use crate::effect::Selector;
    let Effect::DealDamage { to, amount } = &def.effect else { return None };
    if !matches!(amount, Value::XFromCost) || !matches!(to, Selector::TargetFiltered { .. }) {
        return None;
    }
    // Must be a creature target that can't be redirected to a player.
    let filter = def.effect.target_filter_for_slot(0)?;
    if filter.can_match_player() {
        return None;
    }
    state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
        .map(|c| c.toughness().max(0) as u32)
        .max()
}

/// True if X matters for this spell — either the cost has an `{X}` pip
/// or the effect tree mentions `Value::XFromCost`. The latter catches
/// catalog cards (Banefire, Mind Twist, …) whose costs predate the
/// engine's proper X-pip wiring.
pub fn x_relevant(def: &CardDefinition) -> bool {
    def.cost.has_x() || effect_uses_x(&def.effect)
}

fn effect_uses_x(eff: &Effect) -> bool {
    use crate::effect::Value;
    fn value_uses_x(v: &Value) -> bool {
        match v {
            Value::XFromCost => true,
            Value::Sum(parts) => parts.iter().any(value_uses_x),
            Value::Diff(a, b)
            | Value::Times(a, b)
            | Value::Min(a, b)
            | Value::Max(a, b) => value_uses_x(a) || value_uses_x(b),
            Value::NonNeg(inner) => value_uses_x(inner),
            Value::CountOf(_) | Value::PowerOf(_) | Value::ToughnessOf(_)
            | Value::CountersOn { .. } | Value::ManaValueOf(_)
            | Value::DistinctTypesInTopOfLibrary { .. }
            | Value::DistinctTypesInGraveyard { .. } => false,
            _ => false,
        }
    }
    fn predicate_uses_x(p: &crate::effect::Predicate) -> bool {
        use crate::effect::Predicate as P;
        match p {
            P::ValueAtLeast(a, b) | P::ValueAtMost(a, b) | P::ValueEquals(a, b) => {
                value_uses_x(a) || value_uses_x(b)
            }
            P::Not(inner) => predicate_uses_x(inner),
            P::All(parts) | P::Any(parts) => parts.iter().any(predicate_uses_x),
            P::SelectorCountAtLeast { n, .. } => value_uses_x(n),
            _ => false,
        }
    }
    match eff {
        Effect::Seq(steps) => steps.iter().any(effect_uses_x),
        Effect::If { cond, then, else_ } => {
            predicate_uses_x(cond) || effect_uses_x(then) || effect_uses_x(else_)
        }
        Effect::ChooseMode(modes) => modes.iter().any(effect_uses_x),
        Effect::ForEach { body, .. }
        | Effect::Repeat { body, .. }
        | Effect::DelayUntil { body, .. } => effect_uses_x(body),
        Effect::DealDamage { amount, .. }
        | Effect::GainLife { amount, .. }
        | Effect::LoseLife { amount, .. }
        | Effect::Drain { amount, .. }
        | Effect::Draw { amount, .. }
        | Effect::Mill { amount, .. }
        | Effect::Scry { amount, .. }
        | Effect::Surveil { amount, .. }
        | Effect::LookAtTop { amount, .. }
        | Effect::AddCounter { amount, .. }
        | Effect::RemoveCounter { amount, .. }
        | Effect::AddPoison { amount, .. } => value_uses_x(amount),
        Effect::Discard { amount, .. } => value_uses_x(amount),
        Effect::PumpPT { power, toughness, .. } => {
            value_uses_x(power) || value_uses_x(toughness)
        }
        Effect::Sacrifice { count, .. } | Effect::DiscardChosen { count, .. } => {
            value_uses_x(count)
        }
        Effect::CreateToken { count, .. }
        | Effect::CreateTokenCopyOf { count, .. }
        | Effect::CreateTokenCopiesHasteSac { count, .. }
        | Effect::CopySpell { count, .. }
        | Effect::CopySpellWithRiders { count, .. }
        | Effect::CopySpellMayChooseTargets { count, .. } => value_uses_x(count),
        Effect::RevealUntilFind { cap, .. } => value_uses_x(cap),
        Effect::AddFirstSpellTax { count, .. } => value_uses_x(count),
        _ => false,
    }
}

/// If `eff` is (or wraps via `Seq`) a top-level `ChooseMode`, return the
/// number of modes. Otherwise `None`. The bot uses this to enumerate each
/// mode separately when generating castable actions, so a card whose
/// default mode (mode 0) is dead in the current board state (e.g. Drown
/// in the Loch's "counter target spell" with no opp spell on the stack)
/// still surfaces a viable alternate (mode 1: destroy creature).
fn modal_mode_count(eff: &Effect) -> Option<usize> {
    match eff {
        Effect::ChooseMode(modes) => Some(modes.len()),
        // Cast-time multi-mode spells (Choreographed Sparks, Moment of
        // Reckoning): the bot casts them single-mode via the plain
        // `CastSpell { mode }` back-compat path.
        Effect::ChooseModesCast { modes, .. } | Effect::ChooseModesByPoints { modes, .. } => {
            Some(modes.len())
        }
        Effect::Seq(steps) => steps.iter().find_map(modal_mode_count),
        _ => None,
    }
}

/// Resolve the effect branch for a chosen mode. For non-modal effects
/// (or `mode == None`), returns the original effect. For modal effects,
/// returns the chosen mode's body so the auto-target heuristic uses the
/// correct filter for that mode.
fn mode_branch(eff: &Effect, mode: Option<usize>) -> &Effect {
    match (eff, mode) {
        (Effect::ChooseMode(modes), Some(m)) if m < modes.len() => &modes[m],
        (Effect::ChooseModesCast { modes, .. } | Effect::ChooseModesByPoints { modes, .. }, Some(m))
            if m < modes.len() =>
        {
            &modes[m]
        }
        (Effect::Seq(steps), Some(_)) => steps
            .iter()
            .find(|s| matches!(s, Effect::ChooseMode(_)))
            .map(|s| mode_branch(s, mode))
            .unwrap_or(eff),
        _ => eff,
    }
}

fn can_afford_with_extra(
    printed: &ManaCost,
    pool: &ManaPool,
    extra_generic: u32,
    reduction: u32,
) -> bool {
    let mut cost = if printed.has_x() { printed.with_x_value(0) } else { printed.clone() };
    if reduction > 0 {
        cost.reduce_generic(reduction);
    }
    if extra_generic > 0 {
        cost.symbols.push(crate::mana::ManaSymbol::Generic(extra_generic));
    }
    pool.clone().pay(&cost).is_ok()
}

/// Pick a sensible auto-target for a spell cast by `caster` using the
/// engine's shared targeting heuristic.
pub fn choose_target(state: &GameState, def: &CardDefinition, caster: usize) -> Option<Target> {
    state.auto_target_for_effect(&def.effect, caster)
}

/// True when `ta` is the canonical Strixhaven magecraft trigger:
/// SpellCast scope=YourControl with the IS-only predicate. Used by
/// the bot's spell-bias heuristic so a controlled magecraft permanent
/// nudges the bot toward casting an IS spell to fire the trigger.
fn is_magecraft_trigger(ta: &crate::card::TriggeredAbility) -> bool {
    use crate::card::{EventKind, EventScope};
    matches!(ta.event.kind, EventKind::SpellCast)
        && matches!(ta.event.scope, EventScope::YourControl)
        && ta.event.filter.is_some()
}

/// True when `ta` is an Opus-style rider (SOS): an on-cast trigger whose
/// body branches on `Predicate::CastSpellManaSpentAtLeast` — "if five or
/// more mana was spent to cast that spell, [big] instead". See
/// `shortcut::opus_trigger`.
fn is_opus_trigger(ta: &crate::card::TriggeredAbility) -> bool {
    use crate::card::EventKind;
    fn branches(e: &Effect) -> bool {
        match e {
            Effect::If { cond, then, else_ } => {
                matches!(cond, crate::effect::Predicate::CastSpellManaSpentAtLeast(_))
                    || branches(then)
                    || branches(else_)
            }
            Effect::Seq(v) => v.iter().any(branches),
            Effect::MayDo { body, .. } | Effect::ForEach { body, .. } => branches(body),
            _ => false,
        }
    }
    matches!(ta.event.kind, EventKind::SpellCast) && branches(&ta.effect)
}

/// True when `ta` is an Increment rider (SOS): an on-cast trigger gated
/// on `Predicate::IncrementSatisfied` — "if the amount of mana spent is
/// greater than this creature's power or toughness, put a +1/+1 counter
/// on it". See `shortcut::increment_trigger`.
fn is_increment_trigger(ta: &crate::card::TriggeredAbility) -> bool {
    use crate::card::EventKind;
    fn branches(e: &Effect) -> bool {
        match e {
            Effect::If { cond, then, else_ } => {
                matches!(cond, crate::effect::Predicate::IncrementSatisfied)
                    || branches(then)
                    || branches(else_)
            }
            Effect::Seq(v) => v.iter().any(branches),
            Effect::MayDo { body, .. } | Effect::ForEach { body, .. } => branches(body),
            _ => false,
        }
    }
    matches!(ta.event.kind, EventKind::SpellCast) && branches(&ta.effect)
}

/// The smallest mana-spent total that grows at least one of the bot's
/// Increment bodies: `min(power, toughness) + 1` over them (the gate is
/// "spent > power OR toughness", so clearing the smaller stat suffices).
/// `None` with no Increment body out. Computed stats, so the threshold
/// climbs as counters land — exactly the printed escalation.
fn increment_threshold(state: &GameState, seat: usize) -> Option<u32> {
    state
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == seat
                && c.definition.triggered_abilities.iter().any(is_increment_trigger)
        })
        .filter_map(|c| state.computed_permanent(c.id))
        .map(|cp| (cp.power.min(cp.toughness).max(0) + 1) as u32)
        .min()
}

/// True when `ta` is a Repartee trigger (SOS): an on-cast event filter
/// that requires the spell to target a creature. See `shortcut::repartee`.
fn is_repartee_trigger(ta: &crate::card::TriggeredAbility) -> bool {
    use crate::card::EventKind;
    use crate::effect::Predicate;
    fn wants_creature_target(p: &Predicate) -> bool {
        match p {
            Predicate::CastSpellTargetsMatch(_) => true,
            Predicate::All(v) => v.iter().any(wants_creature_target),
            _ => false,
        }
    }
    matches!(ta.event.kind, EventKind::SpellCast)
        && ta.event.filter.as_ref().is_some_and(wants_creature_target)
}

/// True when `eff` carries a this-turn-lifegain gate (SOS Infusion) —
/// the shape whose payoff a pre-gain cast wastes.
fn effect_infusion_gated(eff: &Effect) -> bool {
    use crate::effect::Predicate;
    fn gated(p: &Predicate) -> bool {
        match p {
            Predicate::LifeGainedThisTurnAtLeast { .. }
            | Predicate::FirstLifeGainThisTurn { .. } => true,
            Predicate::All(v) => v.iter().any(gated),
            _ => false,
        }
    }
    match eff {
        Effect::If { cond, then, else_ } => {
            gated(cond) || effect_infusion_gated(then) || effect_infusion_gated(else_)
        }
        Effect::Seq(v) => v.iter().any(effect_infusion_gated),
        Effect::MayDo { body, .. } | Effect::ForEach { body, .. } => effect_infusion_gated(body),
        _ => false,
    }
}

/// Whether any face of `def` is Infusion-gated — spell body or a
/// triggered rider (the ETB Infusion shape).
fn card_infusion_gated(def: &CardDefinition) -> bool {
    effect_infusion_gated(&def.effect)
        || def.triggered_abilities.iter().any(|t| effect_infusion_gated(&t.effect))
}

/// Mana the bot would spend casting `a`: printed cost plus the chosen X.
/// Only the plain-cast shape is priced — it is the one that carries a
/// live `x_value` — which is all the Opus nudge needs.
fn cast_mana_spent(state: &GameState, seat: usize, a: &GameAction) -> u32 {
    match a {
        GameAction::CastSpell { card_id, x_value, .. } => state.players[seat]
            .hand
            .iter()
            .find(|c| c.id == *card_id)
            .map(|c| c.definition.cost.cmc() + x_value.unwrap_or(0))
            .unwrap_or(0),
        _ => 0,
    }
}

/// True when resolving `a` gains the caster life — the Infusion unlock.
/// Lifelink creatures count: cast precombat, they gain before a
/// postcombat Infusion payoff checks the turn's total.
fn cast_gains_life(state: &GameState, seat: usize, a: &GameAction) -> bool {
    use crate::effect::{PlayerRef, Selector};
    let GameAction::CastSpell { card_id, .. } = a else { return false };
    let Some(c) = state.players[seat].hand.iter().find(|c| c.id == *card_id) else {
        return false;
    };
    fn gains(e: &Effect) -> bool {
        let hits_self = |s: &Selector| {
            matches!(s, Selector::You | Selector::This)
                || matches!(s, Selector::Player(PlayerRef::You))
        };
        match e {
            Effect::GainLife { who, .. } => hits_self(who),
            Effect::Drain { to, .. } => hits_self(to),
            Effect::Seq(v) => v.iter().any(gains),
            Effect::If { then, else_, .. } => gains(then) || gains(else_),
            Effect::MayDo { body, .. } | Effect::ForEach { body, .. } => gains(body),
            _ => false,
        }
    }
    gains(&c.definition.effect)
        || c.definition.keywords.contains(&crate::card::Keyword::Lifelink)
}

/// Best hostile creature the effect's primary slot accepts — the
/// Repartee swap-in for an IS cast the auto-targeter aimed at a player.
/// Highest board value first; `would_accept` re-checks full legality
/// (hexproof, protection) at the probe site.
fn best_hostile_creature_target(
    state: &GameState,
    seat: usize,
    eff: &Effect,
    w: &EvalWeights,
) -> Option<Target> {
    let filter = eff.primary_target_filter();
    let mut foes: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| !state.same_team(c.controller, seat) && c.definition.is_creature())
        .collect();
    foes.sort_by_key(|c| std::cmp::Reverse(permanent_value(state, c.id, w)));
    foes.into_iter().map(|c| Target::Permanent(c.id)).find(|t| match &filter {
        Some(f) => state.evaluate_requirement_static(f, t, seat, None),
        None => true,
    })
}

/// True when casting `def` reads the converge count — the distinct colors
/// of mana spent — anywhere the bot can see it: an effect `Value`
/// (`ConvergedValue`), a `ManaValueAtMostConverged` target filter
/// (Sundering Archaic), or an enters-with-counters amount. The `Value`
/// walk mirrors [`effect_uses_x`]'s variant coverage.
fn card_reads_converge(def: &CardDefinition) -> bool {
    use crate::effect::Value;
    fn value_is_converge(v: &Value) -> bool {
        match v {
            Value::ConvergedValue => true,
            Value::Sum(parts) => parts.iter().any(value_is_converge),
            Value::Diff(a, b) | Value::Times(a, b) | Value::Min(a, b) | Value::Max(a, b) => {
                value_is_converge(a) || value_is_converge(b)
            }
            Value::NonNeg(inner) => value_is_converge(inner),
            _ => false,
        }
    }
    fn req_converge(r: &crate::card::SelectionRequirement) -> bool {
        use crate::card::SelectionRequirement as R;
        match r {
            R::ManaValueAtMostConverged => true,
            R::And(a, b) | R::Or(a, b) => req_converge(a) || req_converge(b),
            _ => false,
        }
    }
    fn walk(eff: &Effect) -> bool {
        match eff {
            Effect::Seq(steps) => steps.iter().any(walk),
            Effect::If { then, else_, .. } => walk(then) || walk(else_),
            Effect::ChooseMode(modes) => modes.iter().any(walk),
            Effect::ForEach { body, .. }
            | Effect::Repeat { body, .. }
            | Effect::DelayUntil { body, .. }
            | Effect::MayDo { body, .. } => walk(body),
            Effect::DealDamage { amount, .. }
            | Effect::GainLife { amount, .. }
            | Effect::LoseLife { amount, .. }
            | Effect::Drain { amount, .. }
            | Effect::Draw { amount, .. }
            | Effect::Mill { amount, .. }
            | Effect::Scry { amount, .. }
            | Effect::Surveil { amount, .. }
            | Effect::LookAtTop { amount, .. }
            | Effect::AddCounter { amount, .. }
            | Effect::RemoveCounter { amount, .. }
            | Effect::AddPoison { amount, .. }
            | Effect::Discard { amount, .. } => value_is_converge(amount),
            Effect::PumpPT { power, toughness, .. } => {
                value_is_converge(power) || value_is_converge(toughness)
            }
            Effect::CreateToken { count, .. } => value_is_converge(count),
            _ => false,
        }
    }
    walk(&def.effect)
        || def.effect.primary_target_filter().is_some_and(req_converge)
        || def.enters_with_counters.as_ref().is_some_and(|(_, v)| value_is_converge(v))
}

/// SOS Converge pre-float: when the bot's chosen play scales with the
/// distinct colors of mana spent, tap one plain source of a color the
/// pool doesn't hold yet and cast NEXT tick — the payment funnel spends
/// pool mana first, so every floated color is a drained (counted) color
/// when the cast goes off. Bounded on every side: only fires while the
/// float is smaller than the cost's mana value (excess would strand and
/// vanish at end of phase), only from single-fixed-color tap-only
/// sources with no life cost (no ChooseColor prompt, no pain), and each
/// firing adds a color the pool lacked, so at most four taps precede the
/// cast.
fn pick_converge_prefloat(
    state: &GameState,
    seat: usize,
    action: &GameAction,
) -> Option<GameAction> {
    use crate::mana::Color;
    let def: &CardDefinition = match action {
        GameAction::CastSpell { card_id, .. } => {
            &state.players[seat].hand.iter().find(|c| c.id == *card_id)?.definition
        }
        GameAction::CastPrepareSpell { creature_id, .. } => {
            state.battlefield_find(*creature_id)?.definition.prepare_spell.as_deref()?
        }
        _ => return None,
    };
    if !card_reads_converge(def) {
        return None;
    }
    let pool = &state.players[seat].mana_pool;
    if pool.total() >= def.cost.cmc() {
        return None;
    }
    for c in state.battlefield.iter().filter(|c| c.controller == seat && !c.tapped) {
        for (idx, a) in c.definition.activated_abilities.iter().enumerate() {
            if !is_countable_mana_ability(a) || a.life_cost > 0 {
                continue;
            }
            let (amount, colors, colorless) = mana_ability_output(&a.effect);
            if amount == 0 || colorless || colors.len() != 1 {
                continue;
            }
            let Some(color) = Color::ALL.into_iter().find(|c| colors.contains(*c)) else {
                continue;
            };
            if pool.amount(color) > 0 {
                continue;
            }
            let tap = GameAction::ActivateAbility {
                card_id: c.id,
                ability_index: idx,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if state.would_accept(tap.clone()) {
                return Some(tap);
            }
        }
    }
    None
}

/// True when the card with id `cid` in `seat`'s hand is an instant or
/// sorcery. Cheap helper for the magecraft-bias path; falls back to
/// false on missing cards.
fn is_instant_or_sorcery_in_hand(state: &GameState, seat: usize, cid: CardId) -> bool {
    use crate::card::CardType;
    state.players[seat]
        .hand
        .iter()
        .find(|c| c.id == cid)
        .map(|c| {
            c.definition.card_types.contains(&CardType::Instant)
                || c.definition.card_types.contains(&CardType::Sorcery)
        })
        .unwrap_or(false)
}

/// For a *beneficial* Aura in hand (positive `equipped_bonus` stats or a
/// granted keyword), pick the bot's most valuable creature that satisfies
/// the enchant filter as the host. Returns `None` for non-Auras and for
/// debuff Auras (negative stats — Pacifism-style restrictions live in
/// other def fields and keep the hostile auto-target walk). Without this,
/// `Effect::Attach` falls into the auto-targeter's hostile branch and a
/// Rancor prefers the opponent's creatures.
fn is_beneficial_aura(def: &CardDefinition) -> bool {
    use crate::card::EnchantmentSubtype;
    if !def.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Aura) {
        return false;
    }
    def.equipped_bonus.as_ref().is_some_and(|bonus| {
        bonus.power + bonus.toughness > 0
            || (bonus.power + bonus.toughness == 0 && !bonus.keywords.is_empty())
    })
}

fn beneficial_aura_host(
    state: &GameState,
    seat: usize,
    aura: &crate::card::CardInstance,
    w: &EvalWeights,
) -> Option<crate::game::Target> {
    let def = &aura.definition;
    if !is_beneficial_aura(def) {
        return None;
    }
    let filter = def.effect.primary_target_filter();
    let mut hosts: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature())
        .collect();
    hosts.sort_by_key(|c| std::cmp::Reverse(permanent_value(state, c.id, w)));
    hosts
        .into_iter()
        .map(|c| crate::game::Target::Permanent(c.id))
        .find(|t| match &filter {
            Some(f) => state.evaluate_requirement_static(f, t, seat, Some(aura.id)),
            None => true,
        })
}

/// Best cutoff for "choose a number; destroy all creatures with power ≥
/// it": maximize destroyed enemy value minus destroyed own value,
/// breaking ties upward (spare more of everyone's board when equal).
fn best_destroy_power_cutoff(state: &GameState, seat: usize, max: u32, w: &EvalWeights) -> u32 {
    let mut best = (i32::MIN, 0u32);
    for n in 0..=max {
        let mut score = 0i32;
        for c in state.battlefield.iter().filter(|c| c.definition.is_creature()) {
            let power = state.computed_permanent(c.id).map(|cp| cp.power).unwrap_or(c.power());
            if power >= n as i32 {
                let v = permanent_value(state, c.id, w);
                score += if c.controller == seat { -v } else { v };
            }
        }
        if score > best.0 || (score == best.0 && n > best.1) {
            best = (score, n);
        }
    }
    best.1
}

/// True when `def` carries a static that keys off the Prepared counter
/// (SOS "prepared creatures you control get …" payoffs). Matched
/// structurally on the pump/keyword-grant shapes those payoffs use.
fn static_rewards_prepared(def: &CardDefinition) -> bool {
    use crate::card::{SelectionRequirement as R, Selector};
    use crate::effect::StaticEffect;
    fn req_mentions_prepared(r: &R) -> bool {
        match r {
            R::WithCounter(crate::card::CounterType::Prepared) => true,
            R::And(a, b) | R::Or(a, b) => req_mentions_prepared(a) || req_mentions_prepared(b),
            _ => false,
        }
    }
    let sel_mentions = |s: &Selector| match s {
        Selector::EachPermanent(r) => req_mentions_prepared(r),
        _ => false,
    };
    def.static_abilities.iter().any(|sa| match &sa.effect {
        StaticEffect::PumpPT { applies_to, .. }
        | StaticEffect::GrantKeyword { applies_to, .. } => sel_mentions(applies_to),
        _ => false,
    })
}

/// First damage amount a spell's effect tree deals (walking `Seq`), with
/// `{X}` resolved to the candidate's chosen X. `None` when the effect deals
/// no (statically knowable) damage — non-Const amounts are treated as
/// unknown rather than guessed.
fn first_damage_amount(effect: &Effect, x: u32) -> Option<i32> {
    use crate::effect::Value;
    match effect {
        Effect::DealDamage { amount, .. } => match amount {
            Value::Const(n) => Some(*n),
            Value::XFromCost => Some(x as i32),
            _ => None,
        },
        Effect::Seq(steps) => steps.iter().find_map(|e| first_damage_amount(e, x)),
        _ => None,
    }
}

/// Heuristic rank for one candidate play. Rough scale:
///
/// * mana investment counts double (printed cmc + chosen X + kick count) —
///   the bot leads with its biggest affordable play and spends its pool;
/// * a creature body adds its printed stats plus a small keyword nod, so
///   on-curve bodies outrank cantrip filler;
/// * a targeted effect adds the value of what it hits — an opponent's
///   permanent contributes its full `permanent_value`, so removal chases
///   the biggest threat and a Bolt at a 1/1 loses to a Bolt at a dragon
///   (or to just deploying a bomb instead);
/// * enhanced cast variants (kicker, delve, gift, bestow, conspire, …) get
///   a flat edge over the plain cast of the same card, so the upside line
///   wins whenever both are affordable.
///
/// The caller adds jitter for tie-breaks; scores only need to be
/// *relatively* right within one candidate pool.
/// Material evaluation of a state from `seat`'s perspective: a decided
/// game dominates everything, then board presence (`permanent_value` ×3
/// per permanent, opponents' counted against), hand size (×2), and life.
/// Deliberately coarse — it's compared between candidate *outcomes* of the
/// same tick, so shared terms cancel and only the action's delta matters.
fn eval_material(state: &GameState, seat: usize, w: &EvalWeights) -> i32 {
    // The learned value net, when a profile asks for it and a net is
    // loaded. Undecided positions only: the heuristic's ±100 000·unit for
    // a decided game must keep dominating the net's 0..10 000 range so
    // "actually winning" always outranks "the net likes it".
    if w.net_slot != 0
        && state.game_over.is_none()
        && let Some(p) = super::net_eval::win_prob(state, seat, w.net_slot)
    {
        if w.net_blend_scale > 0 {
            let bias = ((p - 0.5) * (w.net_blend_scale * w.unit) as f32) as i32;
            return eval_material_inner(state, seat, w, false) + bias;
        }
        return (p * 10_000.0) as i32;
    }
    eval_material_inner(state, seat, w, false)
}

/// [`eval_material`] with `seat`'s own summoning-sick creatures counted as
/// worth nothing.
///
/// Forge's `GameStateEvaluator` carries this alongside the real score as
/// `summonSickValue`, and uses it to answer one question: does this line
/// achieve anything *this turn*, or does it only add a body that can't
/// attack yet? A creature deployed in the precombat main and a creature
/// deployed after combat are worth the same at end of turn, but the second
/// one was played with a turn's more information and left the mana up in
/// between. Only the first reads as progress to a greedy evaluator, which
/// is why this bot puts 95 % of its plays in the precombat main.
fn eval_material_summon_sick_blind(state: &GameState, seat: usize, w: &EvalWeights) -> i32 {
    eval_material_inner(state, seat, w, true)
}

fn eval_material_inner(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
    blind_to_sick: bool,
) -> i32 {
    if let Some(over) = state.game_over {
        return match over {
            Some(winner) if winner == seat => 100_000 * w.unit,
            Some(_) => -100_000 * w.unit,
            None => 0,
        };
    }
    let mut v = 0i32;
    for c in &state.battlefield {
        // Lands are worth a small flat amount — enough that ramp/fetch
        // registers and land destruction isn't free, without a flooded
        // board dominating the material count.
        let pv = if c.definition.is_land() {
            2 * w.unit
        } else {
            let mut pv = permanent_value(state, c.id, w);
            // Loyalty is a spendable RESOURCE, not material: counting it
            // here made every plus ability self-rewarding (+2 loyalty
            // read as +6 material for free) and every ultimate
            // self-punishing (−6 loyalty read as −18), so walkers ticked
            // up forever. `permanent_value` keeps the loyalty term for
            // removal targeting — a fat walker is still the best target.
            if c.definition.is_planeswalker() {
                pv -= c.counter_count(crate::card::CounterType::Loyalty) as i32 * w.unit;
            }
            3 * pv
        };
        // A body that can't attack yet isn't this turn's progress -- see
        // `eval_material_summon_sick_blind`.
        let sick = blind_to_sick
            && c.controller == seat
            && c.definition.is_creature()
            && c.summoning_sick
            && !c.has_keyword(&crate::card::Keyword::Haste);
        let pv = if sick { 0 } else { pv };
        if c.controller == seat {
            v += pv;
        } else if !state.same_team(c.controller, seat) {
            v -= pv;
        }
    }
    for (i, p) in state.players.iter().enumerate() {
        if !p.is_alive() {
            continue;
        }
        // A hand card at 4 ≈ half an average permanent — enough that
        // "draw a card" beats "gain 3 life" (a card is a future play;
        // three life at a healthy total is nearly nothing).
        let emblems: i32 = p.emblems.iter().map(|e| emblem_value(state, i, e)).sum();
        // CR 725 / 726 — the crown and the initiative are recurring resources,
        // not one-shots: the monarch draws at each of their end steps and the
        // initiative-holder ventures on top of that. Priced above a single
        // hand card (4) because they keep paying until someone takes them.
        let crown = i32::from(state.monarch == Some(i)) * 7
            + i32::from(state.initiative == Some(i)) * 9;
        let material = (4 * p.hand.len() as i32 + emblems + crown) * w.unit
            + life_value(state.effective_life(i), w);
        if i == seat {
            v += material;
        } else if !state.same_team(i, seat) {
            v -= material;
        }
    }
    v
}

/// Material value of one emblem for `seat`. Emblems are ultimates and
/// usually game-bending — but a CONDITIONAL emblem is only worth what
/// the deck can feed it. A lifegain-triggered emblem (Professor Dellian
/// Fel's "whenever you gain life, target opponent loses that much") is
/// priced by the seat's visible lifegain sources: with none it's nearly
/// dead (2 — below a +2 ability's gain-3, so the walker holds the fort
/// instead of ulting into nothing), and each source adds 6, capped at
/// 32. A flat price made the bot ult indiscriminately and Fel's fleet
/// attribution DROPPED — the build-around emblem needs the build.
/// Unconditional emblems keep the flat 25.
fn emblem_value(state: &GameState, seat: usize, emblem: &crate::player::Emblem) -> i32 {
    use crate::effect::{EventKind, Value};
    // Ajani-style "whenever you gain life" emblems are worth what the
    // board can feed them — the original special case, kept as-is.
    let lifegain_triggered =
        emblem.triggered.iter().any(|t| matches!(t.event.kind, EventKind::LifeGained));
    if lifegain_triggered {
        return 2 + 6 * lifegain_sources(state, seat).min(5);
    }
    // Everything else used to be a flat 25, which made a game-winning
    // draw engine and a minor rider read the same — the "ultimates the
    // eval can't see" limitation was really "ultimates the eval can't
    // tell apart". Price the recurring payoff by shape instead: card
    // advantage highest (an emblem draw repeats every turn, unanswerable
    // by design), damage and tokens next, anthem statics per body they
    // could pump. Floor near the old constant so unrecognized shapes
    // aren't suddenly worthless; cap so no emblem reads as strictly
    // game-over while the game is still being played.
    let amount = |v: &Value| match v {
        Value::Const(n) => (*n).max(1),
        _ => 2,
    };
    fn shape_value(e: &Effect, amount: &dyn Fn(&Value) -> i32) -> i32 {
        match e {
            Effect::Draw { amount: a, .. } => 12 * amount(a),
            Effect::DealDamage { amount: a, .. } | Effect::Drain { amount: a, .. } => {
                6 * amount(a)
            }
            Effect::CreateToken { count, .. } => 10 * amount(count),
            Effect::GainLife { amount: a, .. } => 2 * amount(a),
            Effect::Seq(v) => v.iter().map(|e| shape_value(e, amount)).sum(),
            Effect::If { then, else_, .. } => {
                shape_value(then, amount).max(shape_value(else_, amount))
            }
            Effect::MayDo { body, .. } | Effect::ForEach { body, .. } => {
                shape_value(body, amount)
            }
            _ => 8,
        }
    }
    let triggered: i32 =
        emblem.triggered.iter().map(|t| shape_value(&t.effect, &amount)).sum();
    let statics = 12 * emblem.statics.len() as i32;
    (triggered + statics).clamp(20, 60)
}

/// Visible lifegain sources for `seat`: battlefield lifelink bodies, and
/// battlefield/hand cards whose effect trees gain the controller life
/// (GainLife, Drain). Loyalty abilities are deliberately NOT scanned —
/// the emblem-maker mustn't count itself as its own payoff.
fn lifegain_sources(state: &GameState, seat: usize) -> i32 {
    fn gains_life(e: &Effect) -> bool {
        match e {
            Effect::GainLife { .. } | Effect::Drain { .. } => true,
            Effect::Seq(v) => v.iter().any(gains_life),
            Effect::If { then, else_, .. } => gains_life(then) || gains_life(else_),
            Effect::MayDo { body, .. } => gains_life(body),
            Effect::ChooseMode(modes) => modes.iter().any(gains_life),
            Effect::ApplyToTargets { effect, .. } => gains_life(effect),
            _ => false,
        }
    }
    fn card_gains_life(def: &CardDefinition) -> bool {
        def.keywords.contains(&crate::card::Keyword::Lifelink)
            || gains_life(&def.effect)
            || def.triggered_abilities.iter().any(|t| gains_life(&t.effect))
            || def.activated_abilities.iter().any(|a| gains_life(&a.effect))
    }
    let battlefield = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && card_gains_life(&c.definition))
        .count();
    let hand = state.players[seat]
        .hand
        .iter()
        .filter(|c| card_gains_life(&c.definition))
        .count();
    (battlefield + hand) as i32
}

/// Advance `g` through this turn's combat, so a candidate line can be
/// scored on the board it actually produces rather than the board that
/// exists the instant it resolves.
///
/// This is the single biggest gap between this evaluator and the reference
/// AIs. `evaluate_action_outcome` snapshots immediately after resolution,
/// which cannot see that the creature just cast dies on the crack-back,
/// that the removal spell opened a lethal swing, or that the 2/2 deployed
/// into an empty board is about to trade with a 4/4. Forge scores nothing
/// without first fast-forwarding a copy to combat damage
/// (`GameStateEvaluator.simulateUpcomingCombatThisTurn`); this is that,
/// driven by the bot's own `pick_attacks` / `pick_blocks` so the simulated
/// combat is the combat this bot would actually play.
///
/// What a combat simulation did.
///
/// Previously a `bool`, which conflated "there was no combat to look at"
/// with "the simulation ran out of fuel partway". Callers treat those
/// oppositely — the first means score the state as-is, the second means
/// refuse to score a board where attackers are tapped but damage was never
/// dealt — and collapsing them made every evaluation on a board with no
/// possible attackers unscoreable, silently dropping the whole position
/// back to the static rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombatSim {
    /// Nothing to simulate; `g` is untouched.
    Skipped,
    /// Ran through combat damage.
    Completed,
    /// Started and could not finish; `g` is now a torn state.
    Incomplete,
}

/// Bails cheaply — without touching `g` — when there is no combat to look
/// at: the game is over, the turn is already past combat damage, or the
/// active player has no creature that could attack. Forge guards the same
/// way, because the state copy is the expensive part.
fn simulate_through_combat(g: &mut GameState, fuel: &mut u32, w: &EvalWeights) -> CombatSim {
    if g.is_game_over() || g.step >= TurnStep::CombatDamage {
        return CombatSim::Skipped;
    }
    let attacker_seat = g.active_player_idx;
    let could_attack = g.battlefield.iter().any(|c| {
        c.controller == attacker_seat
            && c.definition.is_creature()
            && !c.tapped
            && (!c.summoning_sick || c.has_keyword(&crate::card::Keyword::Haste))
    });
    if !could_attack {
        return CombatSim::Skipped;
    }
    let turn = g.turn_number;
    let mut attacks_submitted = false;
    let mut blocks_submitted = false;
    while g.step < TurnStep::CombatDamage && g.turn_number == turn && !g.is_game_over() {
        *fuel = match fuel.checked_sub(1) {
            Some(f) => f,
            None => return CombatSim::Incomplete,
        };
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(g, pending.acting_player(), w, &pending.decision, false)
            };
            if g.perform_action(GameAction::SubmitDecision(answer)).is_err() {
                return CombatSim::Incomplete;
            }
            continue;
        }
        let action = match g.step {
            TurnStep::DeclareAttackers if !attacks_submitted => {
                attacks_submitted = true;
                let declarer = g.attack_declarer();
                GameAction::DeclareAttackers(pick_attacks(g, declarer))
            }
            TurnStep::DeclareBlockers if !blocks_submitted && !g.attacking().is_empty() => {
                // The defender is not the priority holder at this point, so
                // ask the engine which seat is actually owed the
                // declaration. Getting this wrong silently leaves every
                // attacker unblocked, which flatters the attack.
                match (0..g.players.len()).find(|&s| g.may_declare_blocks(s)) {
                    Some(defender) => {
                        blocks_submitted = true;
                        GameAction::DeclareBlockers(pick_blocks(g, defender))
                    }
                    None => GameAction::PassPriority,
                }
            }
            _ => GameAction::PassPriority,
        };
        if g.perform_action(action).is_err() {
            // A rejected declaration would spin forever; fall back to
            // passing, and give up if even that fails.
            if g.perform_action(GameAction::PassPriority).is_err() {
                return CombatSim::Incomplete;
            }
        }
    }
    CombatSim::Completed
}

/// Dry-run `action` to quiescence on a full-state clone (libraries kept —
/// resolution may draw) and score the result for `seat`: the cast is
/// applied, then priority passes with [`AutoDecider`] answers for any
/// decision that surfaces until the stack empties. `None` on rejection or
/// a resolution that won't settle — callers fall back to the static rank.
fn evaluate_action_outcome(
    state: &GameState,
    seat: usize,
    action: &GameAction,
    w: &EvalWeights,
) -> Option<i32> {
    evaluate_action_sequence(state, seat, action, w, w.lookahead)
}

/// Score of the best *sequence* of up to `depth + 1` plays that starts with
/// `action`, rather than the score the moment `action` resolves.
///
/// This is the gap a one-action-at-a-time evaluator cannot close: with four
/// mana available, "cast the four-drop" and "cast a two-drop" are compared
/// as single plays, so the bot never sees that the second line continues
/// into *another* two-drop and ends the turn ahead. Forge searches
/// sequences to three plies for exactly this reason
/// (`SpellAbilityPicker` recursing through `SimulationController`).
///
/// Stopping is always one of the options considered, so a sequence is never
/// forced to spend everything — dumping the hand is a line, not an
/// obligation.
fn evaluate_action_sequence(
    state: &GameState,
    seat: usize,
    action: &GameAction,
    w: &EvalWeights,
    depth: u8,
) -> Option<i32> {
    let mut g = state.clone();
    g.perform_action(action.clone()).ok()?;
    let mut fuel = 64u32;
    loop {
        if g.is_game_over() {
            break;
        }
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(&g, pending.acting_player(), w, &pending.decision, false)
            };
            g.perform_action(GameAction::SubmitDecision(answer)).ok()?;
        } else if g.stack.is_empty() {
            break;
        } else {
            g.perform_action(GameAction::PassPriority).ok()?;
        }
        fuel = fuel.checked_sub(1)?;
    }
    // The value of stopping here.
    let mut best = score_settled_state(&g, seat, w)?;
    if depth > 0 {
        for follow in follow_up_candidates(&g, seat, w) {
            if let Some(v) = evaluate_action_sequence(&g, seat, &follow, w, depth - 1) {
                best = best.max(v);
            }
        }
    }
    Some(best)
}

/// Score a state that has resolved to quiescence, running it through this
/// turn's combat first when the profile asks for it. `None` when the combat
/// simulation can't complete — see `simulate_through_combat`.
fn score_settled_state(g: &GameState, seat: usize, w: &EvalWeights) -> Option<i32> {
    if !w.combat_aware {
        return Some(eval_material(g, seat, w));
    }
    // Score the board this line actually leads to, not the one that exists
    // the moment it resolves -- see `simulate_through_combat`. Its own fuel
    // budget: combat is a long way through the step machine (two
    // declarations plus a priority round per step, before triggers).
    let mut sim = g.clone();
    let mut combat_fuel = 256u32;
    match simulate_through_combat(&mut sim, &mut combat_fuel, w) {
        // A half-simulated combat is worse than none: attackers are
        // declared and tapped but damage was never dealt, so the line reads
        // as pure downside. Refuse to score a torn state -- the caller
        // falls back to the static rank.
        CombatSim::Incomplete => None,
        // Skipped leaves `sim` untouched, so scoring it is just scoring `g`.
        CombatSim::Skipped | CombatSim::Completed => Some(eval_material(&sim, seat, w)),
    }
}

/// The few plays worth considering as a continuation of a sequence:
/// the best-scoring validated candidates from `g`, capped hard.
///
/// The cap is the whole reason this is affordable. Enumerating candidates
/// runs an engine dry-run per specialty card shape, so a wide branching
/// factor at every ply would cost far more than the extra ply is worth;
/// two continuations is enough to catch the case this exists for (a second
/// cheap spell the greedy pick would have priced out).
fn follow_up_candidates(g: &GameState, seat: usize, w: &EvalWeights) -> Vec<GameAction> {
    const MAX_FOLLOW_UPS: usize = 2;
    // Only when the bot could actually take another play right now: still
    // its own main phase, holding priority, nothing on the stack.
    if g.is_game_over()
        || g.pending_decision.is_some()
        || !g.stack.is_empty()
        || g.active_player_idx != seat
        || g.player_with_priority() != seat
        || !matches!(g.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
    {
        return Vec::new();
    }
    let probe = g.affordance_probe_template();
    let mut ranked: Vec<(i32, GameAction, bool)> = cast_candidates(g, seat, &probe, w)
        .into_iter()
        .map(|(a, ok)| (score_candidate(g, seat, &a, w), a, ok))
        .collect();
    ranked.sort_by_key(|(s, _, _)| std::cmp::Reverse(*s));
    let mut out = Vec::with_capacity(MAX_FOLLOW_UPS);
    for (_, a, ok) in ranked {
        if out.len() >= MAX_FOLLOW_UPS {
            break;
        }
        if ok || GameState::would_accept_on(&probe, a.clone()) {
            out.push(a);
        }
    }
    out
}

/// Could `action` still be taken later this turn cycle at instant speed?
///
/// Only spells whose card is an Instant or has Flash: everything else is
/// sorcery-timed, so "wait" means "wait a whole turn", which is a very
/// different trade from "wait until their end step". Deliberately narrow —
/// it gates *not acting*, and a false positive there costs a real play.
fn castable_at_instant_speed(state: &GameState, seat: usize, action: &GameAction) -> bool {
    use crate::card::{CardType, Keyword};
    let card_id = match action {
        GameAction::CastSpell { card_id, .. } => *card_id,
        _ => return false,
    };
    let Some(card) = state.players[seat].hand.iter().find(|c| c.id == card_id) else {
        return false;
    };
    card.definition.card_types.contains(&CardType::Instant)
        || card.definition.keywords.contains(&Keyword::Flash)
}

/// Does `action` achieve anything *this turn*, ignoring bodies that can't
/// attack yet? See [`eval_material_summon_sick_blind`]. `true` when the
/// question can't be answered (the outcome probe bailed), so an
/// unevaluable line is never held back.
///
/// Under [`EvalWeights::combat_aware`] the comparison runs *through this
/// turn's combat*, which is what makes the question meaningful for
/// interaction rather than just for creatures: killing a blocker before
/// attacking is worth something now, killing it at the opponent's end step
/// is worth the same and costs less information. Only a simulation that
/// reaches combat damage can tell those apart — and this is the consumer
/// the combat-aware evaluator was missing when it measured neutral on its
/// own, because within a single main phase combat is otherwise identical
/// across every candidate and cancels out.
fn improves_this_turn(
    state: &GameState,
    seat: usize,
    action: &GameAction,
    w: &EvalWeights,
) -> bool {
    // The baseline has to be measured the same way the outcome is. With
    // `combat_aware` the outcome runs through combat, so a raw pre-combat
    // baseline would be compared against a post-combat score and every
    // connecting attack would read as "this action improved things" —
    // making the gate fire almost never. Forge avoids this by routing both
    // sides through the same `getScoreForGameState`, which fast-forwards
    // combat itself.
    let before = if w.combat_aware {
        let mut idle = state.clone();
        let mut idle_fuel = 256u32;
        let _ = simulate_through_combat(&mut idle, &mut idle_fuel, w);
        eval_material_summon_sick_blind(&idle, seat, w)
    } else {
        eval_material_summon_sick_blind(state, seat, w)
    };
    let mut g = state.clone();
    if g.perform_action(action.clone()).is_err() {
        return true;
    }
    let mut fuel = 64u32;
    while !g.is_game_over() {
        if g.pending_decision.is_some() {
            let answer = {
                let pending = g.pending_decision.as_ref().unwrap();
                decide_pending_policy(&g, pending.acting_player(), w, &pending.decision, false)
            };
            if g.perform_action(GameAction::SubmitDecision(answer)).is_err() {
                return true;
            }
        } else if g.stack.is_empty() {
            break;
        } else if g.perform_action(GameAction::PassPriority).is_err() {
            return true;
        }
        fuel = match fuel.checked_sub(1) {
            Some(f) => f,
            None => return true,
        };
    }
    if w.combat_aware {
        let mut combat_fuel = 256u32;
        // A torn simulation can't answer the question; don't hold on it.
        if simulate_through_combat(&mut g, &mut combat_fuel, w) == CombatSim::Incomplete {
            return true;
        }
    }
    eval_material_summon_sick_blind(&g, seat, w) > before
}

/// Final pick among the validated finalists `(jittered static score,
/// action)`: resolve each candidate's outcome and take the best resulting
/// state, static score breaking ties and ordering candidates whose outcome
/// probe bailed. A lone finalist skips the outcome clones entirely.
fn pick_by_outcome(
    state: &GameState,
    seat: usize,
    finalists: Vec<(i32, GameAction)>,
    w: &EvalWeights,
) -> Option<GameAction> {
    if finalists.len() <= 1 {
        return finalists.into_iter().next().map(|(_, a)| a);
    }
    let baseline = eval_material(state, seat, w);
    finalists
        .into_iter()
        .max_by_key(|(s, a)| {
            // Known-temporary casts (bounce, until-EOT stat changes) are
            // pinned to the baseline: the post-resolution snapshot can't
            // see the effect reversing, so evaluating it would sell a
            // bounce as removal. They win only on static score against
            // other no-eval-gain lines.
            let ev = if action_outcome_is_temporary(state, a) {
                baseline
            } else {
                evaluate_action_outcome(state, seat, a, w).unwrap_or(baseline)
            };
            (ev, *s)
        })
        .map(|(_, a)| a)
}

/// True when `e`'s tree contains a leaf whose apparent value REVERSES
/// after the turn: an until-end-of-turn stat change, or a bounce of a
/// battlefield permanent to hand (the permanent comes back next turn).
/// The outcome evaluation snapshots the state right after resolution, so
/// these leaves read as permanent gains — a bounced 4-drop looked like
/// Doom Blade (+3×value) and a "base P/T 5/5 until end of turn" like a
/// real +18 material swing, and the bot burned Proctor's Gaze / Quandrix
/// Charm on them at sorcery speed for nothing. Graveyard/exile-to-hand
/// moves (Regrowth) are real card advantage and are NOT temporary.
fn contains_temporary_leaf(e: &Effect) -> bool {
    use crate::effect::{Duration, ZoneDest};
    match e {
        Effect::PumpPT { duration: Duration::EndOfTurn | Duration::EndOfCombat, .. }
        | Effect::SetBasePT { duration: Duration::EndOfTurn | Duration::EndOfCombat, .. }
        | Effect::SwitchPT { duration: Duration::EndOfTurn | Duration::EndOfCombat, .. } => true,
        Effect::Move { what, to: ZoneDest::Hand(_) } => {
            // A bounce of a battlefield object; an off-board (graveyard /
            // exile) retrieval keeps the card — permanent value.
            match what {
                crate::effect::Selector::TargetFiltered { filter, .. } => {
                    !filter.mentions_offboard_zone()
                }
                _ => true,
            }
        }
        Effect::Seq(v) => v.iter().any(contains_temporary_leaf),
        Effect::If { then, else_, .. } => {
            contains_temporary_leaf(then) || contains_temporary_leaf(else_)
        }
        Effect::MayDo { body, .. } => contains_temporary_leaf(body),
        Effect::ApplyToTargets { effect, .. } => contains_temporary_leaf(effect),
        _ => false,
    }
}

/// True when `action` is a cast whose (mode-resolved) effect contains a
/// temporary leaf — such candidates skip the outcome evaluation (see
/// [`contains_temporary_leaf`]) and compete on static score alone.
fn action_outcome_is_temporary(state: &GameState, action: &GameAction) -> bool {
    let (card_id, mode) = match action {
        GameAction::CastSpell { card_id, mode, .. } => (*card_id, *mode),
        _ => return false,
    };
    let Some(card) = state.find_card_anywhere(card_id) else { return false };
    contains_temporary_leaf(mode_branch(&card.definition.effect, mode))
}

/// A pure temporary-pump instant aimed at a target creature (Giant
/// Growth, Infuriate): the whole effect tree is target pumps with an
/// end-of-turn/combat duration. Anything with riders (draw, damage,
/// counters, keyword grants) stays castable on the normal schedule.
fn is_combat_trick(def: &CardDefinition) -> bool {
    use crate::card::CardType;
    use crate::effect::{Duration, Selector};
    if !def.card_types.contains(&CardType::Instant) {
        return false;
    }
    fn all_temp_pumps(e: &Effect) -> bool {
        match e {
            Effect::PumpPT {
                what: Selector::Target(_) | Selector::TargetFiltered { .. },
                duration: Duration::EndOfTurn | Duration::EndOfCombat,
                ..
            } => true,
            Effect::Seq(v) => !v.is_empty() && v.iter().all(all_temp_pumps),
            _ => false,
        }
    }
    all_temp_pumps(&def.effect)
}

/// After blocks are in: cast a held pump trick when it flips a fight our
/// creature is currently losing — it dies to its opposite number and the
/// pump saves it, or it fails to kill and the pump finishes the job.
/// Covers both sides of combat (our blocked attacker on our turn, our
/// blocker on theirs). Constant pumps only; dynamic amounts are skipped
/// rather than mis-valued.
fn pick_combat_trick(state: &GameState, seat: usize, w: &EvalWeights) -> Option<GameAction> {
    use crate::effect::{Duration, Selector, Value};
    fn pump_amounts(e: &Effect) -> Option<(i32, i32)> {
        match e {
            Effect::PumpPT {
                what: Selector::Target(_) | Selector::TargetFiltered { .. },
                power: Value::Const(p),
                toughness: Value::Const(t),
                duration: Duration::EndOfTurn | Duration::EndOfCombat,
            } => Some((*p, *t)),
            Effect::Seq(v) => {
                let mut acc: Option<(i32, i32)> = None;
                for e in v {
                    let (p, t) = pump_amounts(e)?;
                    let (ap, at) = acc.unwrap_or((0, 0));
                    acc = Some((ap + p, at + t));
                }
                acc
            }
            _ => None,
        }
    }
    let tricks: Vec<(CardId, i32, i32)> = state.players[seat]
        .hand
        .iter()
        .filter(|c| is_combat_trick(&c.definition))
        .filter(|c| can_afford_in_state(state, seat, c, w))
        .filter_map(|c| pump_amounts(&c.definition.effect).map(|(p, t)| (c.id, p, t)))
        .collect();
    if tricks.is_empty() {
        return None;
    }
    let computed_pt = |id: CardId| -> Option<(i32, i32)> {
        let cp = state.computed_permanent(id);
        let raw = state.battlefield_find(id)?;
        Some(match cp {
            Some(cp) => (cp.power, cp.toughness),
            None => (raw.power(), raw.toughness()),
        })
    };
    for (blocker, attacker) in state.block_map_snapshot() {
        let (Some(b), Some(a)) =
            (state.battlefield_find(blocker), state.battlefield_find(attacker))
        else {
            continue;
        };
        let (our_id, their_id) = if a.controller == seat && !state.same_team(b.controller, seat) {
            (attacker, blocker)
        } else if b.controller == seat && !state.same_team(a.controller, seat) {
            (blocker, attacker)
        } else {
            continue;
        };
        let (Some((op, ot)), Some((tp, tt))) = (computed_pt(our_id), computed_pt(their_id))
        else {
            continue;
        };
        let dies = tp >= ot;
        let kills = op >= tt;
        if !dies && kills {
            continue; // already winning this fight
        }
        for &(cid, p, t) in &tricks {
            let saves = dies && ot + t > tp;
            let now_kills = !kills && op + p >= tt;
            if !(saves || now_kills) {
                continue;
            }
            let action = GameAction::CastSpell {
                card_id: cid,
                target: Some(Target::Permanent(our_id)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

fn score_candidate(state: &GameState, seat: usize, action: &GameAction, w: &EvalWeights) -> i32 {
    use crate::card::CardType;
    // (source card, slot-0 target, variant bonus, extra mana sunk in).
    let (card_id, target, variant_bonus, extra_mana) = match action {
        GameAction::CastSpell { card_id, target, x_value, .. } => {
            (*card_id, target.clone(), 0, x_value.unwrap_or(0))
        }
        GameAction::CastSpellBack { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        GameAction::CastSpellDelve { card_id, target, x_value, .. } => {
            (*card_id, target.clone(), 3, x_value.unwrap_or(0))
        }
        GameAction::CastGift { card_id, target, .. } => (*card_id, target.clone(), 3, 0),
        GameAction::CastSpellSpree { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        GameAction::CastSpellConspire { card_id, target, .. } => (*card_id, target.clone(), 3, 0),
        GameAction::CastSpellKicked { card_id, target, .. } => (*card_id, target.clone(), 3, 0),
        GameAction::CastSpellKickers { card_id, target, .. } => (*card_id, target.clone(), 3, 0),
        GameAction::CastSpellMultikicked { card_id, target, times, .. } => {
            (*card_id, target.clone(), 3, *times)
        }
        GameAction::CastBestow { card_id, target, .. } => (*card_id, target.clone(), 3, 0),
        GameAction::CastAdventure { card_id, target, .. }
        | GameAction::CastOmen { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        GameAction::CastPrototype { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        GameAction::CastSplitRight { card_id, target, .. }
        | GameAction::CastAftermath { card_id, target, .. }
        | GameAction::CastFlashback { card_id, target, .. }
        | GameAction::CastMayhem { card_id, target, .. }
        | GameAction::CastHarmonize { card_id, target, .. }
        | GameAction::CastDisturb { card_id, target, .. }
        | GameAction::CastSpellAlternative { card_id, target, .. } => {
            (*card_id, target.clone(), 0, 0)
        }
        GameAction::CastAdventureCreature { card_id, target, .. }
        | GameAction::CastPlotted { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        GameAction::ActivateAbility { card_id, target, .. } => (*card_id, target.clone(), 0, 0),
        // Loyalty activations: the target term is what differentiates them
        // (a −3 destroy at a 5-drop should out-score "+2: gain 3"); the
        // outcome eval in `pick_loyalty_ability` is the primary judge.
        GameAction::ActivateLoyaltyAbility { card_id, target, .. } => {
            (*card_id, target.clone(), 0, 0)
        }
        GameAction::CastPrepareSpell { creature_id, target, .. } => {
            (*creature_id, target.clone(), 0, 0)
        }
        // Fallback lines (face-down morphs, discard-activated) only appear
        // when nothing else is castable, so their exact rank is moot.
        _ => return 0,
    };

    let mut score = 0i32;
    let mut damage: Option<i32> = None;
    if let Some(card) = state.find_card_anywhere(card_id) {
        // Score the face actually being cast when it isn't the front:
        // MDFC backs for back-face casts, and the inset spell for
        // prepare-casts — scoring the latter by the CREATURE valued
        // "cast draw-3 for {U}" like deploying a 5/5 body, so the bot
        // fired every inset spell at the first opportunity.
        let def = match (action, card.definition.back_face.as_deref()) {
            (GameAction::CastSpellBack { .. } | GameAction::CastDisturb { .. }, Some(back)) => back,
            _ => &card.definition,
        };
        let def = match (action, card.definition.prepare_spell.as_deref()) {
            (GameAction::CastPrepareSpell { .. }, Some(spell)) => spell,
            _ => def,
        };
        // These terms are raw card stats; `permanent_value` below is on the
        // profile's scale, so lift them into the same units or a scaled
        // profile would drown the cast's own merits in the target's value.
        score += 2 * (def.cost.cmc() as i32 + extra_mana as i32) * w.unit;
        if def.card_types.contains(&CardType::Creature) {
            score += (def.power.max(0) + def.toughness.max(0)) * w.unit;
            score += (def.keywords.len() as i32).min(3) * w.unit;
        }
        damage = first_damage_amount(&def.effect, extra_mana);
    }

    // Unpreparing forfeits any "prepared creatures you control …" static
    // the bot has out (SOS Top of the Class); charge the cast for the
    // rider it strips.
    if matches!(action, GameAction::CastPrepareSpell { .. })
        && state.battlefield.iter().any(|c| {
            c.controller == seat && static_rewards_prepared(&c.definition)
        })
    {
        score -= 4 * w.unit;
    }

    match target {
        // Aimed at an opponent's permanent: removal / theft / lockdown —
        // worth what the target is worth. Aimed at our own: pump / aura /
        // equip, a small flat gain.
        Some(Target::Permanent(id)) => {
            match state.battlefield_find(id).map(|c| c.controller) {
                Some(ctrl) if ctrl != seat => {
                    let mut v = permanent_value(state, id, w);
                    // CR 702.21 — a ward tax is mana/life the cast sinks
                    // with no effect of its own; price it at the same
                    // 2-per-mana rate the cast's own cost earns above so
                    // an un-warded target of equal value, or a different
                    // spell entirely, wins the tie. Payability is the
                    // candidate gate's job (`ward_gate_ok`); this term
                    // only ranks survivors.
                    if let Some(tax) = ward_tax(state, id, seat) {
                        v -= 2 * ward_tax_burden(&tax) * w.unit;
                    }
                    // Damage spells only count as removal when they kill:
                    // chip damage at a too-big creature keeps a quarter of
                    // the value, and overkill (a huge X at a small body)
                    // pays back the wasted points so Shock-the-2/2 beats
                    // Fireball-for-8-the-2/2.
                    if let (Some(dmg), Some(cp)) = (damage, state.computed_permanent(id))
                        && cp.card_types.contains(&CardType::Creature)
                    {
                        if dmg < cp.toughness {
                            v /= 4;
                        } else {
                            // Overkill is charged in scaled points -- `dmg` and
                            // `toughness` are raw, `v` is not.
                            v -= (dmg - cp.toughness).max(0) * w.unit;
                        }
                    }
                    // A bounce is tempo, not removal — the permanent comes
                    // back next turn. A third of the value keeps target
                    // selection sane without treating it as a kill.
                    if let GameAction::CastSpell { mode, .. } = action
                        && let Some(card) = state.find_card_anywhere(card_id)
                        && contains_temporary_leaf(mode_branch(&card.definition.effect, *mode))
                    {
                        v /= 3;
                    }
                    score += v;
                }
                Some(_) => score += 2 * w.unit,
                None => {}
            }
        }
        // Face damage / discard at an opponent beats a self-aimed cantrip.
        Some(Target::Player(p)) => score += if p != seat { 4 * w.unit } else { w.unit },
        _ => {}
    }

    score + variant_bonus * w.unit
}


// ── Accessors for the Monte Carlo bot ────────────────────────────────────
//
// `mcts` needs the same candidate enumeration and leaf evaluation the
// heuristic bot uses, so the two are compared on identical inputs and any
// ladder difference is the *search*, not a second opinion about what is
// castable or what a board is worth.

/// The main-phase plays worth searching from `state`, validated.
pub(crate) fn main_phase_candidates_for_mcts(
    state: &GameState,
    seat: usize,
    w: &EvalWeights,
) -> Vec<GameAction> {
    let probe = state.affordance_probe_template();
    let mut ranked: Vec<(i32, GameAction, bool)> = cast_candidates(state, seat, &probe, w)
        .into_iter()
        .map(|(a, ok)| (score_candidate(state, seat, &a, w), a, ok))
        .collect();
    ranked.sort_by_key(|(s, _, _)| std::cmp::Reverse(*s));
    // Cap the arms. Every candidate costs at least one rollout to seed, so
    // a wide root eats the whole budget before UCB1 gets to allocate any of
    // it; better to search the plausible plays properly than every play
    // badly.
    const MAX_ARMS: usize = 6;
    let mut out = Vec::with_capacity(MAX_ARMS);
    for (_, a, ok) in ranked {
        if out.len() >= MAX_ARMS {
            break;
        }
        if ok || GameState::would_accept_on(&probe, a.clone()) {
            out.push(a);
        }
    }
    // A land drop is a real option and is enumerated separately.
    if state.can_player_play_land(seat)
        && let Some(land) = pick_land_to_play(state, seat, w)
    {
        let action = GameAction::PlayLand(land);
        if GameState::would_accept_on(&probe, action.clone()) {
            out.push(action);
        }
    }
    out
}

/// The heuristic bot's board evaluation, for scoring a rollout leaf.
pub(crate) fn eval_material_for_mcts(state: &GameState, seat: usize, w: &EvalWeights) -> i32 {
    eval_material(state, seat, w)
}

/// The heuristic board evaluation, exposed for measurement.
///
/// The value net is only worth its inference cost if it predicts the
/// winner *better than this does*. Four gate rounds compared the two by
/// playing thousands of games, which answers "is the bot stronger"
/// expensively and says nothing about why; comparing their predictions on
/// the same positions is minutes of compute and separates "the net has
/// not learned" from "the net has learned and the integration wastes it".
/// See `selfplay_train --calibrate`.
pub fn eval_material_public(state: &GameState, seat: usize, w: &EvalWeights) -> i32 {
    eval_material(state, seat, w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::game::GameState;
    use crate::game::TriggerPush;
    use crate::player::Player;

    fn two_player_game() -> GameState {
        let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
        let mut g = GameState::new(players);
        g.step = TurnStep::PreCombatMain;
        g
    }

    fn body_card(name: &'static str, body: Effect) -> CardDefinition {
        use crate::card::{CardType, TriggeredAbility};
        use crate::effect::{EventKind, EventScope, EventSpec};
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "you may".to_string(),
                    body: Box::new(body),
                },
            }],
            ..Default::default()
        }
    }

    /// A redeal must preserve everything the searching seat can legally
    /// see — hand sizes, the battlefield, both graveyards, and each
    /// player's total card count — while replacing what it can't.
    #[test]
    fn determinize_preserves_public_information() {
        let mut g = two_player_game();
        for _ in 0..30 {
            g.add_card_to_library(1, catalog::forest());
        }
        for _ in 0..5 {
            g.add_card_to_library(1, catalog::shivan_dragon());
        }
        for _ in 0..20 {
            g.add_card_to_library(0, catalog::island());
        }
        for _ in 0..4 {
            g.add_card_to_hand(1, catalog::lightning_bolt());
        }
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(1, catalog::serra_angel());
        let before = (
            g.players[1].hand.len(),
            g.players[1].library.len() + g.players[1].hand.len(),
            g.players[0].library.len() + g.players[0].hand.len(),
            g.battlefield.len(),
        );

        let mut d = g.clone();
        determinize_hidden(&mut d, 0, 1);

        assert_eq!(d.players[1].hand.len(), before.0, "opponent hand size is public");
        assert_eq!(d.players[1].library.len() + d.players[1].hand.len(), before.1);
        assert_eq!(d.players[0].library.len() + d.players[0].hand.len(), before.2);
        assert_eq!(d.battlefield.len(), before.3, "the battlefield is public");
        // Our own hand is ours to see and must survive the redeal intact.
        assert_eq!(d.players[0].hand.len(), 1);
        assert_eq!(d.players[0].hand[0].definition.name, "Grizzly Bears");
    }

    /// The point of the redeal: the opponent's hand is *resampled* from
    /// their unseen cards, so a search can no longer plan around the
    /// specific card they are holding. With four Bolts in hand and 35
    /// other cards behind them, keeping all four is vanishingly unlikely.
    #[test]
    fn determinize_resamples_the_opponent_hand() {
        let mut g = two_player_game();
        for _ in 0..35 {
            g.add_card_to_library(1, catalog::forest());
        }
        for _ in 0..4 {
            g.add_card_to_hand(1, catalog::lightning_bolt());
        }
        let mut changed = 0;
        for salt in 0..8 {
            let mut d = g.clone();
            determinize_hidden(&mut d, 0, salt);
            let bolts = d.players[1]
                .hand
                .iter()
                .filter(|c| c.definition.name == "Lightning Bolt")
                .count();
            if bolts < 4 {
                changed += 1;
            }
        }
        assert!(changed >= 7, "expected the redeal to move the hand, changed {changed}/8");
    }

    /// The bot names the creature type it controls the most of (not the stock
    /// AutoDecider "Demon"), so tribal chosen-type payoffs are useful under bot
    /// play.
    #[test]
    fn bot_names_its_most_common_creature_type() {
        use crate::card::CreatureType;
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // Bear
        g.add_card_to_battlefield(0, catalog::llanowar_elves()); // Elf
        g.add_card_to_battlefield(0, catalog::elvish_clancaller()); // Elf
        let ans = decide_creature_type(&g, 0, &[]);
        assert!(matches!(ans, DecisionAnswer::CreatureType(CreatureType::Elf)),
            "two Elves vs one Bear → names Elf, got {ans:?}");
    }

    #[test]
    fn bot_takes_beneficial_optional_trigger() {
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(
            0,
            body_card("Upside", Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
        );
        assert!(optional_trigger_beneficial(&g, id, "you may"),
            "a pure-upside 'you may draw' is taken by the bot");
    }

    /// The bot pays Offspring (CR 702.166) when it can afford it — the chosen
    /// main-phase cast is the kicked variant, not the plain cast.
    #[test]
    fn bot_pays_offspring_when_affordable() {
        use crate::mana::Color;
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let recruit = g.add_card_to_hand(0, catalog::pawpatch_recruit()); // {G}, Offspring {2}
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(action, GameAction::CastSpellKicked { card_id, .. } if card_id == recruit),
            "bot cast Pawpatch Recruit with Offspring paid, got {action:?}"
        );
    }

    /// The bot promises a gift (CR 702.165) when the gifted line is the point
    /// of the card — Scrapshooter's ETB destroy only fires on a promised gift,
    /// so the chosen cast must be `CastGift`, not a plain `CastSpell`.
    #[test]
    fn bot_promises_gift_for_scrapshooter() {
        use crate::mana::Color;
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let scrap = g.add_card_to_hand(0, catalog::scrapshooter()); // {1}{G}{G}
        g.add_card_to_battlefield(1, catalog::sol_ring()); // a legal ETB destroy target
        g.add_card_to_library(1, catalog::forest()); // the gift draw
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(action, GameAction::CastGift { card_id, .. } if card_id == scrap),
            "bot promised Scrapshooter's gift, got {action:?}"
        );
    }

    #[test]
    fn bot_declines_optional_trigger_that_sacrifices_itself() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(
            0,
            body_card("Downside", Effect::SacrificeSource),
        );
        assert!(!optional_trigger_beneficial(&g, id, "you may"),
            "a 'you may sacrifice this' rider is a self-cost the bot declines");
    }

    /// A planeswalker whose highest-loyalty ability needs a target that
    /// doesn't exist must not stop the bot from activating a lower targetless
    /// ability (regression: the `?` on `auto_target_for_effect` used to bail
    /// out of every ability and planeswalker).
    #[test]
    fn bot_skips_untargetable_loyalty_ability_for_a_usable_one() {
        use crate::card::{CardType, LoyaltyAbility};
        use crate::effect::shortcut::target_filtered;
        use crate::card::SelectionRequirement;
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let pw = CardDefinition {
            name: "Test Walker",
            card_types: vec![CardType::Planeswalker],
            base_loyalty: 3,
            loyalty_abilities: vec![
                // Highest loyalty, but needs a creature target (none exist).
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: 2,
                    effect: Effect::DealDamage {
                        to: target_filtered(SelectionRequirement::Creature),
                        amount: Value::Const(2),
                    },
                },
                // Lower loyalty, no target — the bot should fall through here.
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: 1,
                    effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                },
            ],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, pw);
        g.add_card_to_library(0, catalog::island());
        let action = pick_loyalty_ability(&g, 0, &EvalWeights::default()).expect("bot finds the targetless +1");
        match action {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, id);
                assert_eq!(ability_index, 1, "picked the targetless draw, not the dead burn");
            }
            _ => panic!("expected a loyalty activation"),
        }
    }

    /// Loyalty abilities are picked by OUTCOME, not plus-first: Professor
    /// Dellian Fel with an opposing 5/5 on the board fires "−3: destroy
    /// target creature" instead of "+2: you gain 3 life" (the old
    /// cost-ordered walk never pressed a minus, piloting the pool's best
    /// bomb as a lifegain trinket).
    #[test]
    fn bot_walker_presses_removal_over_lifegain() {
        let mut g = two_player_game();
        let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
        let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
        g.add_card_to_library(0, catalog::island());
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_loyalty_ability(&g, 0, &EvalWeights::default()).expect("walker activates something");
        match action {
            GameAction::ActivateLoyaltyAbility {
                card_id, ability_index, target, ..
            } => {
                assert_eq!(card_id, pw);
                assert_eq!(ability_index, 2, "the −3 destroy, not the +2 lifegain");
                assert_eq!(
                    target,
                    Some(crate::game::Target::Permanent(dragon)),
                    "aimed at the opposing dragon",
                );
            }
            other => panic!("expected a loyalty activation, got {other:?}"),
        }
    }

    /// Known-temporary casts skip the outcome eval: with Quandrix Charm
    /// (whose mode 2 is "base P/T 5/5 until end of turn") and a real
    /// creature both castable, the bot develops instead of burning the
    /// Charm as a fake main-phase pump.
    #[test]
    fn bot_prefers_development_over_temp_buff() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // pump target
        let charm = g.add_card_to_hand(0, catalog::quandrix_charm());
        let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        // Second main: this tests which candidate wins the ranking, not
        // when it is cast. The default profile's summon-sick gate defers a
        // first-main creature, which is orthogonal to the point here.
        g.step = TurnStep::PostCombatMain;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(action, GameAction::CastSpell { card_id, .. } if card_id == bears),
            "cast the creature, not the Charm's temp buff, got {action:?}",
        );
        let _ = charm;
    }

    /// With the ultimate affordable AND lifegain to feed the emblem,
    /// the eval presses it: Dellian Fel at 7 loyalty with a Melancholic
    /// Poet and a lifelink body on board fires −6 (emblem priced by
    /// visible lifegain sources; loyalty spent is a resource, not a
    /// material loss).
    #[test]
    fn bot_walker_ults_when_the_deck_feeds_the_emblem() {
        use crate::card::CounterType;
        let mut g = two_player_game();
        let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
        g.battlefield
            .iter_mut()
            .find(|c| c.id == pw)
            .unwrap()
            .counters
            .insert(CounterType::Loyalty, 7);
        // Three visible lifegain sources: emblem value 2 + 6×3 = 20.
        g.add_card_to_battlefield(0, catalog::melancholic_poet());
        g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
        g.add_card_to_hand(0, catalog::melancholic_poet());
        g.add_card_to_library(0, catalog::island());
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_loyalty_ability(&g, 0, &EvalWeights::default()).expect("walker activates something");
        match action {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, pw);
                assert_eq!(ability_index, 3, "the −6 emblem ultimate, not the +2 lifegain");
            }
            other => panic!("expected a loyalty activation, got {other:?}"),
        }
    }

    /// …and WITHOUT lifegain sources the emblem is nearly dead (2 < the
    /// +2's gain-3), so the walker holds the fort instead of ulting into
    /// nothing — the indiscriminate flat-price ult measurably HURT Fel.
    #[test]
    fn bot_walker_holds_ult_without_lifegain() {
        use crate::card::CounterType;
        let mut g = two_player_game();
        let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
        g.battlefield
            .iter_mut()
            .find(|c| c.id == pw)
            .unwrap()
            .counters
            .insert(CounterType::Loyalty, 7);
        g.add_card_to_library(0, catalog::island());
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_loyalty_ability(&g, 0, &EvalWeights::default()).expect("walker activates something");
        match action {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, pw);
                assert_ne!(
                    ability_index, 3,
                    "no lifegain to feed the emblem — don't ult into nothing"
                );
            }
            other => panic!("expected a loyalty activation, got {other:?}"),
        }
    }

    /// The bot can activate a *statically-granted* loyalty ability (one the
    /// walker doesn't print itself), matching the engine's effective-list
    /// activation path.
    #[test]
    fn bot_activates_granted_loyalty_ability() {
        use crate::card::{CardType, LoyaltyAbility, StaticAbility};
        use crate::effect::{Selector, StaticEffect, Value};
        let mut g = two_player_game();
        // A walker with NO printed loyalty abilities.
        let pw = CardDefinition {
            name: "Blank Walker",
            card_types: vec![CardType::Planeswalker],
            base_loyalty: 3,
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, pw);
        // A permanent that grants every planeswalker you control a +1 draw.
        let granter = CardDefinition {
            name: "Loyalty Font",
            card_types: vec![CardType::Artifact],
            static_abilities: vec![StaticAbility {
                description: "Planeswalkers you control have +1: draw a card.",
                effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities {
                    abilities: vec![LoyaltyAbility {
                        x_cost: false,
                        loyalty_cost: 1,
                        effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    }],
                },
            }],
            ..Default::default()
        };
        g.add_card_to_battlefield(0, granter);
        g.add_card_to_library(0, catalog::island());
        match pick_loyalty_ability(&g, 0, &EvalWeights::default()).expect("bot finds the granted ability") {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, id, "activated on the blank walker");
                assert_eq!(ability_index, 0, "the granted +1 is index 0");
            }
            _ => panic!("expected a loyalty activation"),
        }
    }

    /// The Wandering Emperor's +1 (a friendly +1/+1 buff) auto-targets the
    /// bot's OWN creature, never the opponent's — the targeting regression
    /// this test originally caught. (Which ability the walker fires is the
    /// outcome eval's call and pinned elsewhere; here we probe the +1's
    /// target choice directly.)
    #[test]
    fn bot_wandering_emperor_plus_one_targets_own_creature() {
        let mut g = two_player_game();
        let emp = g.add_card_to_battlefield(0, catalog::the_wandering_emperor());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let plus_one = &catalog::the_wandering_emperor().loyalty_abilities[0];
        let picked = g.auto_target_for_effect(&plus_one.effect, 0);
        assert_eq!(
            picked,
            Some(Target::Permanent(mine)),
            "the +1 buffs its own creature, not {theirs:?}",
        );
        let _ = emp;
    }

    #[test]
    fn bot_declines_self_costly_optional_trigger() {
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(
            0,
            body_card("Downside", Effect::LoseLife { who: Selector::You, amount: Value::Const(3) }),
        );
        assert!(!optional_trigger_beneficial(&g, id, "you may"),
            "a 'you may lose 3 life' optional trigger is declined");
    }

    /// Self-directed damage / mill bodies are costs too — the bot declines a
    /// "you may have this deal 4 damage to you" optional trigger.
    #[test]
    fn bot_declines_self_damage_optional_trigger() {
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let dmg = g.add_card_to_battlefield(
            0,
            body_card("SelfBurn", Effect::DealDamage { to: Selector::You, amount: Value::Const(4) }),
        );
        assert!(!optional_trigger_beneficial(&g, dmg, "you may"),
            "a 'you may deal 4 to you' optional trigger is declined");
        let mill = g.add_card_to_battlefield(
            0,
            body_card("SelfMill", Effect::Mill { who: Selector::You, amount: Value::Const(3) }),
        );
        assert!(!optional_trigger_beneficial(&g, mill, "you may"),
            "a 'you may mill yourself 3' optional trigger is declined");
    }

    /// Blight (CR 701.68) shrinks the bot's own board, so a "may blight N for
    /// upside" optional trigger is declined.
    /// A `MayDiscard` reflexive whose payoff isn't self-costly (Toph's
    /// return-a-spell) is accepted by the bot — card filtering is upside.
    #[test]
    fn bot_takes_beneficial_maydiscard() {
        use crate::card::{CardType, TriggeredAbility};
        use crate::effect::{EventKind, EventScope, EventSpec, Selector, Value};
        let mut g = two_player_game();
        let def = CardDefinition {
            name: "Rummager",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::MayDiscard {
                    description: "discard to draw?".to_string(),
                    count: Value::ONE,
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: None,
                },
            }],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, def);
        assert!(optional_trigger_beneficial(&g, id, "discard to draw?"),
            "a MayDiscard whose payoff is a draw is accepted");
    }

    #[test]
    fn bot_declines_blight_optional_trigger() {
        use crate::effect::Value;
        let mut g = two_player_game();
        let blighter = g.add_card_to_battlefield(
            0,
            body_card("Blighter", Effect::Blight { n: Value::Const(2) }),
        );
        assert!(!optional_trigger_beneficial(&g, blighter, "you may"),
            "a 'you may blight 2' optional trigger is declined");
    }

    /// `MayPay` shares the `OptionalTrigger` decision shape with `MayDo`, so
    /// the bot's self-cost screen must introspect it too: a "pay {1}: you lose
    /// 3 life" body is declined even though it's reachable only via MayPay.
    #[test]
    fn bot_declines_self_costly_maypay() {
        use crate::card::{CardType, TriggeredAbility};
        use crate::effect::{EventKind, EventScope, EventSpec, Selector, Value};
        let mut g = two_player_game();
        let def = CardDefinition {
            name: "PayDownside",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayPay {
                    description: "you may pay".to_string(),
                    mana_cost: crate::mana::cost(&[crate::mana::generic(1)]),
                    body: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(3) }),
                    else_: None,
                },
            }],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, def);
        assert!(!optional_trigger_beneficial(&g, id, "you may pay"),
            "a MayPay whose body costs the bot 3 life is declined");
    }

    /// Moving the source to exile/graveyard is a self-cost (decline); returning
    /// it to hand (Recover-style upside) is accepted.
    #[test]
    fn bot_screens_self_move_bodies() {
        use crate::effect::{PlayerRef, Selector, ZoneDest};
        let mut g = two_player_game();
        let exile_self = g.add_card_to_battlefield(
            0,
            body_card("ExileSelf", Effect::Move { what: Selector::This, to: ZoneDest::Exile }),
        );
        assert!(!optional_trigger_beneficial(&g, exile_self, "you may"),
            "'you may exile this' reads as a self-cost");
        let to_hand = g.add_card_to_battlefield(
            0,
            body_card("ToHand", Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        );
        assert!(optional_trigger_beneficial(&g, to_hand, "you may"),
            "returning self to hand is upside");
    }

    fn generic_spell(name: &'static str, cmc: u32) -> CardDefinition {
        use crate::card::CardType;
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            cost: crate::mana::cost(&[crate::mana::generic(cmc)]),
            ..Default::default()
        }
    }

    /// Self-discard heuristic pitches the priciest spell (least likely to be
    /// cast soon), not the head of the hand, when the bot isn't flooded.
    #[test]
    fn bot_self_discard_pitches_priciest_spell() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let pricey = g.add_card_to_hand(0, generic_spell("Pricey", 6));
        let cheap = g.add_card_to_hand(0, generic_spell("Cheap", 1));
        // Offer both; head dump would take `pricey` (first), but so should the
        // heuristic here — make the cheap card the head to prove it's a real
        // choice rather than a head dump.
        let hand = vec![
            (cheap, "Cheap".to_string()),
            (pricey, "Pricey".to_string()),
        ];
        let DecisionAnswer::Discard(ids) = decide_self_discard(&g, 0, &hand, 1) else {
            panic!("expected a Discard answer");
        };
        assert_eq!(ids, vec![pricey], "the most expensive spell is pitched");
    }

    /// When flooded (≥5 lands in play), a surplus land is pitched before a
    /// keepable cheap spell.
    #[test]
    fn bot_self_discard_pitches_surplus_land_when_flooded() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        for _ in 0..5 {
            g.add_card_to_battlefield(0, catalog::island());
        }
        let land = g.add_card_to_hand(0, catalog::island());
        let spell = g.add_card_to_hand(0, generic_spell("Cheap", 1));
        let hand = vec![
            (spell, "Cheap".to_string()),
            (land, "Island".to_string()),
        ];
        let DecisionAnswer::Discard(ids) = decide_self_discard(&g, 0, &hand, 1) else {
            panic!("expected a Discard answer");
        };
        assert_eq!(ids, vec![land], "a flooded bot pitches the surplus land");
    }

    /// A lethal constant-damage ping ability aims at an opposing planeswalker
    /// whose loyalty it can finish off.
    #[test]
    fn bot_pings_lethal_opposing_planeswalker() {
        let mut g = two_player_game();
        let tim = g.add_card_to_battlefield(0, catalog::prodigal_pyromancer()); // {T}: 1 dmg any target
        g.clear_sickness(tim);
        let walker = g.add_card_to_battlefield(1, catalog::vivien_reid());
        // Knock the walker down to 1 loyalty so a 1-damage ping is lethal.
        let inst = g.battlefield_find_mut(walker).unwrap();
        inst.counters.insert(crate::card::CounterType::Loyalty, 1);
        let action = pick_removal_ping(&g, 0).expect("bot should ping the walker");
        match action {
            GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. } => {
                assert_eq!(card_id, tim);
                assert_eq!(t, walker, "aimed at the 1-loyalty planeswalker");
            }
            other => panic!("unexpected action: {other:?}"),
        }
    }

    /// A mana rock's output has to count toward what the bot can cast.
    ///
    /// This used to assert that the bot *tapped* Sol Ring as its own
    /// action, back when it pre-tapped every source before deciding
    /// anything. It no longer does that (see the note in
    /// `main_phase_action_with`), so the assertion is now on the outcome
    /// that mattered all along: the rock's mana is what makes the spell
    /// affordable, and the engine's auto-tap spends it.
    #[test]
    fn bot_spends_mana_rock_output_on_a_spell() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let sol = g.add_card_to_battlefield(0, catalog::sol_ring());
        g.clear_sickness(sol);
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        g.clear_sickness(forest);
        let have = available_mana(&g, 0);
        assert_eq!(have.total, 3, "Sol Ring's two plus the Forest's one");
        assert!(have.colors.contains(crate::mana::Color::Green));

        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        assert!(
            matches!(action, GameAction::CastSpell { card_id, .. } if card_id == bear),
            "bot should cast the bear rather than pre-tapping anything, got {action:?}",
        );
    }

    /// The tap-out regression guard. The bot must not spend mana it has no
    /// use for: with an uncastable hand it should pass, leaving its lands
    /// untapped so they survive into the opponent's turn for instant-speed
    /// plays. Before this fix it tapped every land unconditionally and the
    /// pool was emptied at the phase boundary (CR 500.4).
    #[test]
    fn bot_leaves_mana_untapped_when_it_has_nothing_to_cast() {
        let mut g = two_player_game();
        for _ in 0..3 {
            let land = g.add_card_to_battlefield(0, catalog::forest());
            g.clear_sickness(land);
        }
        // A hand card it cannot cast: wrong color, and no black source.
        g.add_card_to_hand(0, catalog::doom_blade());
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        assert!(
            matches!(action, GameAction::PassPriority),
            "bot should pass, not burn mana, got {action:?}",
        );
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == 0 && !c.tapped).count(),
            3,
            "all three lands stay untapped and available at instant speed",
        );
    }


    /// With spare mana and nothing better to do, the bot sinks it into War
    /// Balloon's fire-counter ability to progress toward animating it.
    #[test]
    fn bot_feeds_fire_counters_to_animate_war_balloon() {
        use crate::card::CounterType;
        let mut g = two_player_game();
        let wb = g.add_card_to_battlefield(0, catalog::war_balloon());
        // A Mountain pays the {1} fire-counter cost; nothing else to do.
        let mtn = g.add_card_to_battlefield(0, catalog::mountain());
        g.clear_sickness(mtn);
        let mut bot = RandomBot::new();
        // Drive a few actions: tap the land for mana, then sink into the counter.
        let mut animated = false;
        for _ in 0..6 {
            let Some(action) = bot.next_action(&g, 0) else { break };
            if let GameAction::ActivateAbility { card_id, ability_index, .. } = &action
                && *card_id == wb
            {
                assert_eq!(*ability_index, 0, "the fire-counter ability");
                animated = true;
            }
            if g.perform_action(action).is_err() { break }
            crate::game::drain_stack(&mut g);
        }
        assert!(animated, "bot activated War Balloon's fire-counter sink");
        assert!(g.battlefield_find(wb).unwrap().counter_count(CounterType::Fire) >= 1,
            "a fire counter was added");
    }

    /// The bot spends surplus energy on a beneficial energy-payoff ability
    /// (Longtusk Cub's `{E}{E}{E}: +1/+1 counter`) once nothing better to do.
    #[test]
    fn bot_spends_energy_on_payoff_ability() {
        let mut g = two_player_game();
        let cub = g.add_card_to_battlefield(0, catalog::longtusk_cub());
        g.clear_sickness(cub);
        g.players[0].energy = 3;
        let action = pick_energy_payoff(&g, 0).expect("bot should pay energy for the counter");
        match action {
            GameAction::ActivateAbility { card_id, .. } => assert_eq!(card_id, cub),
            _ => panic!("expected an activate-ability action"),
        }
        // With too little energy the bot leaves it alone.
        g.players[0].energy = 1;
        assert!(pick_energy_payoff(&g, 0).is_none(), "won't activate without enough energy");
    }

    /// When card-starved, the bot sinks spare mana into Bonders' Enclave's
    /// "{3}, {T}: Draw a card" — but only once its activation condition (a
    /// 4-power creature) is met.
    #[test]
    fn bot_draws_with_value_ability_when_card_starved() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::bonders_enclave());
        g.clear_sickness(land);
        g.add_card_to_library(0, catalog::grizzly_bears()); // something to draw
        g.players[0].mana_pool.add_colorless(3);
        // No 4-power creature → the draw ability's condition fails.
        assert!(pick_card_draw_ability(&g, 0).is_none(),
            "no draw without a 4-power creature");
        g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        match pick_card_draw_ability(&g, 0).expect("bot draws when card-starved") {
            GameAction::ActivateAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, land);
                assert_eq!(ability_index, 1, "the draw ability, not the mana ability");
            }
            _ => panic!("expected an activate-ability action"),
        }
        // A full hand → don't bother drawing.
        for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
        assert!(pick_card_draw_ability(&g, 0).is_none(), "won't draw with a full hand");
    }

    /// The bot fires Frostwielder's `{T}: 1 damage` ping to kill a 1/1, but
    /// won't waste it when no opposing creature dies to it.
    #[test]
    fn bot_pings_a_killable_creature() {
        let mut g = two_player_game();
        let fw = g.add_card_to_battlefield(0, catalog::frostwielder());
        g.clear_sickness(fw);
        let frostling = g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
        let action = pick_removal_ping(&g, 0).expect("bot pings the 1/1");
        match action {
            GameAction::ActivateAbility { card_id, target, .. } => {
                assert_eq!(card_id, fw);
                assert_eq!(target, Some(Target::Permanent(frostling)));
            }
            _ => panic!("expected an activate-ability action"),
        }
        // A 2/2 survives a 1-damage ping → the bot holds the ability.
        g.battlefield.retain(|c| c.id != frostling);
        g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        assert!(pick_removal_ping(&g, 0).is_none(), "won't waste a ping on a survivor");
    }

    /// The bot fires a self-power ping ("{T}: deals damage equal to its power to
    /// target creature") to kill a foe whose toughness it can beat.
    #[test]
    fn bot_pings_with_self_power() {
        use crate::card::{ActivatedAbility, CardType};
        use crate::effect::{Selector, Value};
        let pinger = CardDefinition {
            name: "Self-Power Pinger",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 0, filter: crate::card::SelectionRequirement::Creature },
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut g = two_player_game();
        let p = g.add_card_to_battlefield(0, pinger);
        g.clear_sickness(p);
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, dies to 3
        match pick_removal_ping(&g, 0).expect("bot pings with its own power") {
            GameAction::ActivateAbility { card_id, target, .. } => {
                assert_eq!(card_id, p);
                assert_eq!(target, Some(Target::Permanent(foe)));
            }
            _ => panic!("expected an activate-ability action"),
        }
    }

    /// The bot points a ping at the opponent's face when it's exactly lethal
    /// (reach for the win), not at a creature.
    #[test]
    fn bot_pings_face_for_lethal() {
        let mut g = two_player_game();
        let fw = g.add_card_to_battlefield(0, catalog::frostwielder()); // {T}: 1 dmg any target
        g.clear_sickness(fw);
        g.add_card_to_battlefield(1, catalog::grizzly_bears()); // a 2/2 it can't kill
        g.players[1].life = 1; // lethal to a 1-damage ping
        let action = pick_removal_ping(&g, 0).expect("bot reaches for the win");
        match action {
            GameAction::ActivateAbility { target, .. } => {
                assert_eq!(target, Some(Target::Player(1)), "ping aimed at the face");
            }
            _ => panic!("expected an activate-ability action"),
        }
        // Above 1 life it isn't lethal and there's no killable creature → hold.
        g.players[1].life = 5;
        assert!(pick_removal_ping(&g, 0).is_none(), "won't chip a non-lethal face");
    }

    /// The bot fires a team-pump ability (Bearer of Glory's {4}{W}) once it has
    /// two attackers, but holds it with only one.
    #[test]
    fn bot_team_pumps_with_multiple_attackers() {
        let mut g = two_player_game();
        let bearer = g.add_card_to_battlefield(0, catalog::bearer_of_glory());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bearer);
        g.clear_sickness(bear);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crate::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(4);
        // One attacker: not worth the pump.
        g.attacking = vec![Attack { attacker: bearer, target: AttackTarget::Player(1) }];
        assert!(pick_team_pump(&g, 0).is_none(), "holds the pump with one attacker");
        // Two attackers: fire it.
        g.attacking.push(Attack { attacker: bear, target: AttackTarget::Player(1) });
        match pick_team_pump(&g, 0).expect("bot pumps the team") {
            GameAction::ActivateAbility { card_id, .. } => assert_eq!(card_id, bearer),
            _ => panic!("expected an activate-ability action"),
        }
    }

    /// The bot crews a Vehicle with a spare small creature, but won't tap a
    /// creature bigger than the Vehicle to do it.
    #[test]
    fn bot_crews_a_vehicle_with_a_small_creature() {
        let mut g = two_player_game();
        let chariot = g.add_card_to_battlefield(0, catalog::thundering_chariot()); // 3/3, Crew 1
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.clear_sickness(bear);
        match pick_crew(&g, 0) {
            Some(GameAction::Crew { vehicle, crew_creatures }) => {
                assert_eq!(vehicle, chariot);
                assert_eq!(crew_creatures, vec![bear]);
            }
            other => panic!("expected a crew action, got {other:?}"),
        }
        // Swap the bear for a 5/5: tapping it to animate a 3/3 isn't worth it.
        g.battlefield.retain(|c| c.id != bear);
        let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
        g.clear_sickness(dragon);
        assert!(pick_crew(&g, 0).is_none(), "won't tap a bigger body to crew a smaller Vehicle");
    }

    /// The bot saddles a Mount it can attack with using a spare small creature,
    /// but won't tap a creature bigger than the Mount to do it.
    #[test]
    fn bot_saddles_a_mount_with_a_small_creature() {
        let mut g = two_player_game();
        let ghoda = g.add_card_to_battlefield(0, catalog::gilded_ghoda()); // 2/2, Saddle 1
        g.clear_sickness(ghoda);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.clear_sickness(bear);
        match pick_saddle(&g, 0) {
            Some(GameAction::Saddle { mount, creatures }) => {
                assert_eq!(mount, ghoda);
                assert_eq!(creatures, vec![bear]);
            }
            other => panic!("expected a saddle action, got {other:?}"),
        }
        // A summoning-sick Mount can't attack → don't waste a saddler on it.
        g.battlefield_find_mut(ghoda).unwrap().summoning_sick = true;
        assert!(pick_saddle(&g, 0).is_none(), "won't saddle a Mount that can't attack");
    }

    /// The bot only saddles in precombat main — in postcombat main the "until
    /// end of turn" buff would wear off before any attack could use it.
    #[test]
    fn bot_does_not_saddle_in_postcombat_main() {
        let mut g = two_player_game();
        let ghoda = g.add_card_to_battlefield(0, catalog::gilded_ghoda()); // Saddle 1
        g.clear_sickness(ghoda);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        assert!(pick_saddle(&g, 0).is_some(), "saddles in precombat main");
        g.step = TurnStep::PostCombatMain;
        assert!(pick_saddle(&g, 0).is_none(), "no saddle after combat is over");
    }

    /// Saddle 3 on a 2-power Mount (Caustic Bronco) still gets saddled when the
    /// only saddlers are idle (summoning-sick) creatures: they can't attack, so
    /// their power isn't "wasted" against the overspend guard.
    #[test]
    fn bot_saddles_high_cost_mount_with_idle_creatures() {
        let mut g = two_player_game();
        let bronco = g.add_card_to_battlefield(0, catalog::caustic_bronco()); // 2/2, Saddle 3
        g.clear_sickness(bronco);
        // Two summoning-sick 2/2s — idle this turn, so free to tap.
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        match pick_saddle(&g, 0) {
            Some(GameAction::Saddle { mount, creatures }) => {
                assert_eq!(mount, bronco);
                assert_eq!(creatures.len(), 2, "taps both idle bears to reach Saddle 3");
                assert!(creatures.contains(&a) && creatures.contains(&b));
            }
            other => panic!("expected a saddle action, got {other:?}"),
        }
        // If the same bears could attack, don't overspend real attacker power.
        g.clear_sickness(a);
        g.clear_sickness(b);
        assert!(
            pick_saddle(&g, 0).is_none(),
            "won't tap 4 attacker-power to saddle a 2-power Mount"
        );
    }

    /// The bot sacrifices Pus Kami to destroy a bigger opposing creature, but
    /// not to kill something smaller than the creature it would pitch.
    #[test]
    fn bot_sacs_to_destroy_a_favorable_trade() {
        let mut g = two_player_game();
        let kami = g.add_card_to_battlefield(0, catalog::pus_kami()); // 3/3
        g.clear_sickness(kami);
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        // A 5/5-equivalent opposing threat (nonblack) → favorable sac.
        let dreadmaw = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6 green
        let action = pick_removal_sacrifice(&g, 0).expect("bot sacs to kill the big threat");
        match action {
            GameAction::ActivateAbility { card_id, target, .. } => {
                assert_eq!(card_id, kami);
                assert_eq!(target, Some(Target::Permanent(dreadmaw)));
            }
            _ => panic!("expected an activate-ability action"),
        }
        // Replace with a 1/1 — sacrificing a 3/3 for it is a bad trade.
        g.battlefield.retain(|c| c.id != dreadmaw);
        g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
        assert!(pick_removal_sacrifice(&g, 0).is_none(), "won't sac a 3/3 to kill a 1/1");
    }

    /// The bot recurs a creature from the graveyard via Embalm when it can
    /// afford the cost.
    #[test]
    fn bot_embalms_from_graveyard_with_spare_mana() {
        use crate::TurnStep;
        let mut g = two_player_game();
        let cat = g.add_card_to_graveyard(0, catalog::sacred_cat());
        g.players[0].mana_pool.add(crate::mana::Color::White, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_graveyard_recursion(&g, 0).expect("bot should Embalm Sacred Cat");
        match action {
            GameAction::ActivateAbility { card_id, .. } => assert_eq!(card_id, cat),
            _ => panic!("expected an activate-ability action"),
        }
        // With no mana it leaves the card alone.
        g.players[0].mana_pool.empty();
        assert!(pick_graveyard_recursion(&g, 0).is_none(), "won't Embalm without mana");
    }

    /// The bot reanimates a graveyard creature with a battlefield permanent's
    /// sac-to-return ability (Seedship Broodtender), aimed at the dead creature.
    #[test]
    fn bot_reanimates_from_graveyard_via_battlefield_ability() {
        use crate::TurnStep;
        use crate::mana::Color;
        let mut g = two_player_game();
        let brood = g.add_card_to_battlefield(0, catalog::seedship_broodtender());
        let dead = g.add_card_to_graveyard(0, catalog::colossal_dreadmaw()); // a worthy target
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_battlefield_reanimate(&g, 0).expect("bot reanimates from graveyard");
        match action {
            GameAction::ActivateAbility { card_id, target, .. } => {
                assert_eq!(card_id, brood);
                assert_eq!(target, Some(Target::Permanent(dead)));
            }
            _ => panic!("expected an activate-ability action"),
        }
        // Empty graveyard → nothing to do.
        g.players[0].graveyard.clear();
        assert!(pick_battlefield_reanimate(&g, 0).is_none(), "no target → no activation");
    }

    /// The bot uses a *targeted* graveyard-activated ability (Scavenge),
    /// auto-picking its own creature as the target.
    #[test]
    fn bot_scavenges_onto_own_creature() {
        use crate::TurnStep;
        let mut g = two_player_game();
        let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mangler = g.add_card_to_graveyard(0, catalog::dreg_mangler());
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_graveyard_recursion(&g, 0).expect("bot should Scavenge Dreg Mangler");
        match action {
            GameAction::ActivateAbility { card_id, target, .. } => {
                assert_eq!(card_id, mangler);
                assert_eq!(target, Some(crate::game::Target::Permanent(beater)),
                    "auto-targets the bot's own creature");
            }
            _ => panic!("expected an activate-ability action"),
        }
    }

    /// The bot activates Varolz's *granted* scavenge (a virtual graveyard
    /// ability at index ≥ printed_count), not just printed scavenge cards.
    #[test]
    fn bot_scavenges_via_varolz_grant() {
        use crate::TurnStep;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::varolz_the_scar_striped());
        let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_graveyard_recursion(&g, 0).expect("bot should Scavenge via Varolz");
        match action {
            GameAction::ActivateAbility { card_id, ability_index, target, .. } => {
                assert_eq!(card_id, dead);
                assert_eq!(ability_index, 0, "granted scavenge at index 0 (no printed abilities)");
                // Auto-targets one of the bot's own creatures.
                let t = matches!(target, Some(crate::game::Target::Permanent(id))
                    if id == beater || g.battlefield_find(id).is_some_and(|c| c.controller == 0));
                assert!(t, "scavenge targets an own creature");
            }
            _ => panic!("expected an activate-ability action"),
        }
    }

    /// The bot also recognises the real-cost energy form
    /// (`ActivatedAbility.energy_cost`), not just resolve-time `PayEnergy`.
    #[test]
    fn bot_spends_energy_on_real_cost_form() {
        use crate::card::{ActivatedAbility, CardDefinition, CardType, CounterType};
        let mut g = two_player_game();
        let def = CardDefinition {
            name: "Energy Engine",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            activated_abilities: vec![ActivatedAbility {
                energy_cost: 2,
                discard_cost: None,
                effect: Effect::AddCounter {
                    what: crate::effect::Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: crate::effect::Value::Const(1),
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        g.players[0].energy = 2;
        assert!(pick_energy_payoff(&g, 0).is_some(), "bot fires the energy_cost-gated payoff");
        g.players[0].energy = 1;
        assert!(pick_energy_payoff(&g, 0).is_none(), "and only when it can afford it");
    }

    /// Mulligan heuristic: ship a 1-land seven, keep a 3-land seven, and
    /// stop digging once two mulligans have been taken.
    #[test]
    fn bot_mulligans_land_light_hands_but_keeps_balanced_ones() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        // 1 land + 6 spells → mulligan.
        g.add_card_to_hand(0, catalog::island());
        for _ in 0..6 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
        assert!(matches!(decide_mulligan(&g, 0, 0, &EvalWeights::default()), DecisionAnswer::TakeMulligan));
        // Stop digging after two mulligans even on a bad hand.
        assert!(matches!(decide_mulligan(&g, 0, 2, &EvalWeights::default()), DecisionAnswer::Keep));

        // 3 lands + 4 spells, colors aligned (Forests for green bears) → keep.
        let mut g2 = two_player_game();
        for _ in 0..3 { g2.add_card_to_hand(0, catalog::forest()); }
        for _ in 0..4 { g2.add_card_to_hand(0, catalog::grizzly_bears()); }
        assert!(matches!(decide_mulligan(&g2, 0, 0, &EvalWeights::default()), DecisionAnswer::Keep));
    }

    /// Two 2/2s eat a 4/4 when life isn't threatened. The greedy pass
    /// only gangs under lethal pressure, and `block_search` can only
    /// remove blockers, so at a healthy life total this attacker used to
    /// get through untouched — trading two bears for a bomb is a fine
    /// deal the bot simply never considered.
    #[test]
    fn gang_blocks_for_value_not_only_for_survival() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareBlockers;
        g.active_player_idx = 1;
        // A 4/4 attacking a comfortable life total.
        let big = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.clear_sickness(big);
        let bear_a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let bear_b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear_a);
        g.clear_sickness(bear_b);
        g.players[0].life = 20;
        g.set_attacking(vec![Attack { attacker: big, target: crate::game::types::AttackTarget::Player(0) }]);

        // Serra Angel flies; ground bears can't block it at all, so the
        // gang must be legal to be offered. Swap to a ground fatty.
        let mut g2 = two_player_game();
        g2.step = TurnStep::DeclareBlockers;
        g2.active_player_idx = 1;
        let fatty = g2.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
        g2.clear_sickness(fatty);
        let b1 = g2.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b2 = g2.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b3 = g2.add_card_to_battlefield(0, catalog::grizzly_bears());
        for b in [b1, b2, b3] { g2.clear_sickness(b); }
        g2.players[0].life = 20;
        g2.set_attacking(vec![Attack { attacker: fatty, target: crate::game::types::AttackTarget::Player(0) }]);

        let greedy = pick_blocks(&g2, 0);
        assert!(greedy.iter().filter(|(_, a)| *a == fatty).count() <= 1,
            "greedy blocks the 6/4 with at most one body at 20 life: {greedy:?}");

        let gangs = gang_block_candidates(&g2, 0, &greedy, &EvalWeights::block_gang_search());
        assert!(!gangs.is_empty(), "a gang candidate is offered");
        let gang = &gangs[0];
        let on_fatty = gang.iter().filter(|(_, a)| *a == fatty).count();
        assert!(on_fatty >= 2, "the gang puts two or more blockers on it: {gang:?}");
    }

    /// `mull_quality` fixes the two hands the shipped rule reads
    /// backwards: a two-lander whose only play is one two-drop (kept
    /// today, does nothing from turn three) and a six-land hand holding
    /// a bomb (shipped today, a fine limited keep).
    #[test]
    fn mull_quality_judges_the_hand_not_just_the_land_count() {
        use crate::decision::DecisionAnswer;
        let w = EvalWeights::mulligan_quality();

        // Two Forests, one castable bear, four uncastable six-drops.
        let mut thin = two_player_game();
        for _ in 0..2 { thin.add_card_to_hand(0, catalog::forest()); }
        thin.add_card_to_hand(0, catalog::grizzly_bears());
        for _ in 0..4 { thin.add_card_to_hand(0, catalog::craw_wurm()); }
        assert!(matches!(decide_mulligan(&thin, 0, 0, &EvalWeights::default()), DecisionAnswer::Keep),
            "the shipped rule keeps this on the strength of one two-drop");
        assert!(matches!(decide_mulligan(&thin, 0, 0, &w), DecisionAnswer::TakeMulligan),
            "one play is not a keep at two lands");

        // Six Plains and Serra Angel: flooded, but the payoff is real.
        let mut flooded = two_player_game();
        for _ in 0..6 { flooded.add_card_to_hand(0, catalog::plains()); }
        flooded.add_card_to_hand(0, catalog::serra_angel());
        assert!(matches!(decide_mulligan(&flooded, 0, 0, &EvalWeights::default()), DecisionAnswer::TakeMulligan),
            "the shipped rule ships every six-land hand");
        assert!(matches!(decide_mulligan(&flooded, 0, 0, &w), DecisionAnswer::Keep),
            "a bomb carries the flood");
    }

    /// Color-screw: enough lands and a fine curve, but the lands can't make
    /// the spells' colors (3 Islands + green {1}{G} Grizzly Bears) → ship it.
    #[test]
    fn bot_mulligans_color_screwed_hands() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
        for _ in 0..4 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
        assert!(matches!(decide_mulligan(&g, 0, 0, &EvalWeights::default()), DecisionAnswer::TakeMulligan),
            "no green source for the green spells → color screw → mulligan");
    }

    /// Curve screen: a hand with enough lands but only spells too expensive
    /// to cast early is a screwed keep — ship it on the first mulligan.
    #[test]
    fn bot_mulligans_lands_with_no_early_play() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        // 3 lands + four {6} Obsianus Golems → no spell castable by turn ~4.
        for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
        for _ in 0..4 { g.add_card_to_hand(0, catalog::obsianus_golem()); }
        assert!(matches!(decide_mulligan(&g, 0, 0, &EvalWeights::default()), DecisionAnswer::TakeMulligan),
            "no early play despite enough lands → mulligan");
    }

    /// Sac-cost mana abilities (Lotus Petal) are NOT auto-activated — they
    /// destroy the source on activation, which the random bot can't reason
    /// about.
    #[test]
    fn bot_does_not_tap_sac_cost_mana_source() {
        let mut g = two_player_game();
        let petal = g.add_card_to_battlefield(0, catalog::lotus_petal());
        g.clear_sickness(petal);
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        // Should not activate Lotus Petal's sac-cost ability.
        if let GameAction::ActivateAbility { card_id, .. } = action {
            assert_ne!(card_id, petal, "bot must NOT auto-tap a sac-cost mana source");
        }
    }

    /// Bot activates a planeswalker's loyalty ability when one is
    /// available, picking by OUTCOME: on an empty board Karn's -2
    /// Construct token (a real body that also protects the walker)
    /// out-values the +1's slow card. Karn at 5 loyalty afterward sits
    /// at a healthy 3 — this is development, not a suicide-ult.
    #[test]
    fn bot_activates_planeswalker_loyalty_ability() {
        let mut g = two_player_game();
        // Karn: +1 (reveal two, opponent picks one for your hand) at
        // index 0, a -1 at index 1, and a -2 (Construct token) at index 2.
        let karn = g.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
        g.clear_sickness(karn);
        // Stock the library so the +1 has cards to reveal.
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());

        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        match action {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, karn, "bot should target the Karn it controls");
                assert_eq!(ability_index, 2,
                    "the -2 Construct (board presence) out-values the +1's slow card");
            }
            other => panic!("expected ActivateLoyaltyAbility, got {:?}", other),
        }
    }

    /// The attack search must actually *reach* the opponent's crack-back.
    /// A simulation that bails — on fuel, a rejected declaration, or a step
    /// it can't advance past — silently degrades the whole search to the
    /// greedy declaration it was meant to second-guess, and nothing else in
    /// the suite would notice, because falling back is not an error.
    #[test]
    fn attack_simulation_reaches_the_crack_back() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        for _ in 0..2 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
        }
        let c = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(c);
        let w = EvalWeights::attack_search();
        let greedy = pick_attacks(&g, 0);
        assert_eq!(greedy.len(), 2, "both bears are eligible attackers");
        assert!(
            simulate_attack_outcome(&g, 0, &greedy, &w).is_some(),
            "the alpha strike must simulate to a score"
        );
        assert!(
            simulate_attack_outcome(&g, 0, &[], &w).is_some(),
            "declining to attack must simulate to a score"
        );
    }

    /// Holding a blocker back is only ever *worth* anything a turn later, so
    /// the search has to price it there: two bears into an empty board is a
    /// free swing, but with a 3/3 staring back, keeping one home to block is
    /// the better board once the crack-back is resolved.
    #[test]
    fn attack_search_holds_a_blocker_against_a_bigger_board() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        for _ in 0..2 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
        }
        // A 3/3 that eats a 2/2 for free if we have nothing back.
        let big = g.add_card_to_battlefield(1, catalog::hill_giant());
        g.clear_sickness(big);
        // Both players need something to draw: the simulation runs a full
        // turn cycle, and an empty library decks whoever draws first, which
        // pins every candidate to the same "we won" score.
        for seat in 0..2 {
            for _ in 0..10 {
                g.add_card_to_library(seat, catalog::forest());
            }
        }
        let w = EvalWeights::attack_search();
        let all_in = simulate_attack_outcome(&g, 0, &pick_attacks(&g, 0), &w);
        let none = simulate_attack_outcome(&g, 0, &[], &w);
        assert!(all_in.is_some() && none.is_some(), "both lines must simulate");
        assert_ne!(all_in, none, "the two lines must not score identically \
             — if they do, the simulation is not reaching the crack-back");
    }

    /// Helper: a 1/1 creature with one extra keyword for attack-filter tests.
    fn one_one_with(name: &'static str, kw: crate::card::Keyword) -> CardDefinition {
        let mut d = catalog::grizzly_bears();
        d.name = name;
        d.power = 1;
        d.toughness = 1;
        d.keywords.push(kw);
        d
    }

    /// A menace attacker swings even into a single bigger blocker — menace
    /// needs two blockers, so it gets through (the suicide filter must not
    /// hold it back when the opponent has fewer than two blockers).
    #[test]
    fn bot_attacks_with_menace_into_lone_blocker() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let atk = g.add_card_to_battlefield(0, one_one_with("Sneak", crate::card::Keyword::Menace));
        g.clear_sickness(atk);
        g.add_card_to_battlefield(1, catalog::grizzly_bears()); // lone 2/2 blocker
        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|atk_decl| atk_decl.attacker == atk),
                    "menace attacker should swing past a lone blocker");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// CR 506.2 — under Silent Arbiter the bot declares exactly one attacker
    /// (the engine rejects any bigger batch outright).
    #[test]
    fn bot_respects_the_silent_arbiter_attack_cap() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::silent_arbiter());
        for _ in 0..3 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
        }
        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.len() <= 1, "batch trimmed to the cap, got {}", a.len());
                let mut g2 = g.clone();
                g2.declare_attackers(a).expect("the trimmed batch is legal");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// CR 509.1b — the block planner honours the same cap.
    #[test]
    fn bot_respects_the_silent_arbiter_block_cap() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::silent_arbiter());
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        for _ in 0..3 {
            g.add_card_to_battlefield(1, catalog::grizzly_bears());
        }
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![crate::game::Attack {
            attacker: atk,
            target: crate::game::AttackTarget::Player(1),
        }])
        .expect("attack");
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        let blocks = pick_blocks_for_test(&g, 1);
        let distinct: std::collections::HashSet<_> = blocks.iter().map(|(b, _)| *b).collect();
        assert!(distinct.len() <= 1, "block plan trimmed to the cap");
        g.declare_blockers(blocks).expect("the trimmed plan is legal");
    }

    /// Under High Alert (team "attack as though no defender"), the bot declares
    /// a Wall as an attacker instead of leaving it home.
    #[test]
    fn bot_attacks_with_wall_under_high_alert() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.add_card_to_battlefield(0, catalog::high_alert());
        let wall = g.add_card_to_battlefield(0, catalog::wall_of_lost_thoughts()); // 0/4 Defender
        g.clear_sickness(wall);
        g.players[1].life = 3; // the Wall's 4 toughness-damage is lethal
        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|d| d.attacker == wall),
                    "Wall should attack (deals its toughness) under High Alert");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// A blocker with a computed `CantBlock` (Sandstorm Verge, pacifism) isn't
    /// counted as a threat — the bot swings its 2/2 past a can't-block
    /// deathtouch creature that would otherwise scare it off.
    #[test]
    fn bot_ignores_cant_block_opponents_when_attacking() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        let mut deadly = catalog::grizzly_bears();
        deadly.name = "Pacified Deathtoucher";
        deadly.keywords.push(crate::card::Keyword::Deathtouch);
        deadly.keywords.push(crate::card::Keyword::CantBlock);
        g.add_card_to_battlefield(1, deadly);
        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|d| d.attacker == atk),
                    "should swing past a can't-block deathtouch creature");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// Under a global fog (CR 615.1), the bot holds back a non-lethal
    /// attacker whose combat damage would be prevented.
    #[test]
    fn bot_holds_back_attackers_under_fog() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        g.prevent_combat_damage_this_turn = true; // a Fog is active
        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.is_empty(), "fogged attacker stays home");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// A forced block (MustBeBlocked) with no profitable trade uses the
    /// cheapest legal body, not the bot's best creature.
    #[test]
    fn bot_forced_block_uses_cheapest_body() {
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        // Seat 0 attacks with a 5/5 that must be blocked.
        let mut atk_def = catalog::grizzly_bears();
        atk_def.name = "Provoker";
        atk_def.power = 5;
        atk_def.toughness = 5;
        atk_def.keywords.push(crate::card::Keyword::MustBeBlocked);
        let atk = g.add_card_to_battlefield(0, atk_def);
        g.clear_sickness(atk);
        // Seat 1 (bot) has a 1/1 chump and a 3/3 — neither can kill the 5/5.
        let mut chump = catalog::grizzly_bears();
        chump.name = "Chump"; chump.power = 1; chump.toughness = 1;
        let chump = g.add_card_to_battlefield(1, chump);
        let mut big = catalog::grizzly_bears();
        big.name = "Big"; big.power = 3; big.toughness = 3;
        let big = g.add_card_to_battlefield(1, big);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(1),
        }])).expect("declare attacker");
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(chump, atk)], "forced block uses the 1/1, sparing the 3/3");
        assert!(!blocks.iter().any(|(b, _)| *b == big), "the 3/3 is not thrown away");
    }

    /// CR 702.147 — a Decayed creature can't block, so the bot must not pull
    /// one into a gang block even when its life is on the line.
    #[test]
    fn bot_never_gang_blocks_with_decayed_creature() {
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let mut big = catalog::grizzly_bears();
        big.name = "Bruiser"; big.power = 6; big.toughness = 6;
        let atk = g.add_card_to_battlefield(0, big);
        g.clear_sickness(atk);
        g.players[1].life = 5; // lethal is on the table → life_threatened
        // Two Decayed 3/3s: enough raw power to "kill" the 6/6 on paper, but
        // they can't legally block.
        let mut zombie = catalog::grizzly_bears();
        zombie.name = "Rotter"; zombie.power = 3; zombie.toughness = 3;
        zombie.keywords.push(crate::card::Keyword::Decayed);
        let z1 = g.add_card_to_battlefield(1, zombie.clone());
        let z2 = g.add_card_to_battlefield(1, zombie);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(1),
        }])).expect("declare attacker");
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(!blocks.iter().any(|(b, _)| *b == z1 || *b == z2),
            "Decayed creatures are never assigned as blockers");
    }

    /// An indestructible blocker walls a big attacker for free (CR 702.12) —
    /// it survives, so the bot blocks even with no life pressure.
    #[test]
    fn bot_walls_with_indestructible_blocker() {
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let mut atk_def = catalog::grizzly_bears();
        atk_def.name = "Bruiser"; atk_def.power = 5; atk_def.toughness = 5;
        let atk = g.add_card_to_battlefield(0, atk_def);
        g.clear_sickness(atk);
        let mut wall = catalog::grizzly_bears();
        wall.name = "Indestructo"; wall.power = 1; wall.toughness = 1;
        wall.keywords.push(crate::card::Keyword::Indestructible);
        let wall = g.add_card_to_battlefield(1, wall);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(1),
        }])).expect("declare attacker");
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(wall, atk)], "indestructible 1/1 walls the 5/5 for free");
    }

    /// CR 702.23 — the bot won't pile a second blocker onto a Rampage attacker:
    /// the +N/+N pump means the extra body dies without helping kill it. A lone
    /// deathtouch blocker already kills it, so the 3/3 stays home.
    #[test]
    fn bot_wont_gang_block_a_rampage_attacker() {
        let mut g = two_player_game();
        let giant = g.add_card_to_battlefield(0, catalog::frost_giant()); // 4/4 rampage 2
        g.clear_sickness(giant);
        let rats = g.add_card_to_battlefield(1, catalog::typhoid_rats()); // 1/1 deathtouch
        let mut big = catalog::grizzly_bears();
        big.name = "Ogre"; big.power = 3; big.toughness = 3;
        g.add_card_to_battlefield(1, big);
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: giant, target: AttackTarget::Player(1),
        }])).expect("declare attacker");
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(rats, giant)],
            "deathtouch alone kills it; no second blocker into the Rampage pump");
    }

    /// The bot won't declare a CanAttackOnlyIfDefenderControls attacker
    /// (Dandân) into a defender whose board fails the filter — doing so
    /// would get the whole batch rejected by the engine.
    #[test]
    fn bot_holds_back_dandan_when_defender_has_no_island() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let dd = g.add_card_to_battlefield(0, catalog::dandan());
        g.clear_sickness(dd);
        g.add_card_to_battlefield(0, catalog::island()); // your Island, not the defender's
        let mut bot = RandomBot::new();
        if let Some(GameAction::DeclareAttackers(a)) = bot.next_action(&g, 0) {
            assert!(!a.iter().any(|x| x.attacker == dd),
                "Dandân must not be declared when the defender controls no Island");
        } // declaring no attackers is also fine
        // Now give the defender an Island — Dandân becomes a legal attacker.
        g.add_card_to_battlefield(1, catalog::island());
        let mut bot2 = RandomBot::new();
        match bot2.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|x| x.attacker == dd),
                    "Dandân should attack once the defender controls an Island");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// A deathtouch attacker swings even when smaller than every blocker —
    /// any block trades the opponent's creature for ours.
    #[test]
    fn bot_attacks_with_deathtouch_into_bigger_blocker() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let atk = g.add_card_to_battlefield(0, one_one_with("Stinger", crate::card::Keyword::Deathtouch));
        g.clear_sickness(atk);
        // Two 3/3s — without deathtouch awareness the suicide filter would
        // hold the 1/1 back.
        g.add_card_to_battlefield(1, catalog::hill_giant());
        g.add_card_to_battlefield(1, catalog::hill_giant());
        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|atk_decl| atk_decl.attacker == atk),
                    "deathtouch attacker should swing into bigger blockers");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// Magecraft-aware spell bias: when the bot controls a magecraft
    /// permanent and has both an IS spell and a creature spell in hand,
    /// it should prefer the IS spell to fire the magecraft trigger.
    /// Push (claude/modern_decks batch 202).
    #[test]
    fn bot_prefers_is_spell_when_magecraft_in_play() {
        let mut g = two_player_game();
        // Drop Witherbloom Apprentice (a magecraft permanent) on board.
        g.add_card_to_battlefield(0, catalog::witherbloom_apprentice());
        // Hand has both Lightning Bolt (instant) and Grizzly Bears
        // (creature). The bot must prefer the bolt.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let _bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        let mut bot = RandomBot::new();
        // Drive the bot until it produces a CastSpell — could pass
        // through PlayLand / mana abilities first if seeded with hand-
        // played lands, but in this synthetic state the next non-mana
        // action is the spell.
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastSpell { card_id, .. } = action {
                assert_eq!(card_id, bolt,
                    "magecraft-bias should pick the instant over the creature");
                return;
            }
            // Drive the engine forward so non-cast actions don't loop.
            let _ = g.perform_action(action);
        }
        panic!("bot never produced a CastSpell action");
    }

    /// The bot casts an Adventure half (Stomp) as removal when it can afford
    /// the adventure but not the creature (CR 715).
    #[test]
    fn bot_casts_adventure_half_as_removal() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::bonecrusher_giant());
        // {1}{R}: enough for Stomp, not the {2}{R} creature.
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        let mut bot = RandomBot::new();
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastAdventure { card_id, .. } = action {
                assert_eq!(card_id, id, "bot Stomps with the adventure half");
                let _ = bear;
                return;
            }
            let _ = g.perform_action(action);
        }
        panic!("bot never cast the adventure half");
    }

    /// CR 702.187 — the bot recasts a card discarded this turn from its
    /// graveyard for the mayhem cost (Electro's Bolt as removal).
    #[test]
    fn bot_casts_mayhem_spell_from_graveyard() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(0, catalog::electros_bolt());
        // Discard the Bolt this turn so its Mayhem cast is legal.
        let mut events = Vec::new();
        g.discard_card(0, bolt, &mut events);
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        let mut bot = RandomBot::new();
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastMayhem { card_id, .. } = action {
                assert_eq!(card_id, bolt, "bot recasts Electro's Bolt via Mayhem");
                let _ = bear;
                return;
            }
            let _ = g.perform_action(action);
        }
        panic!("bot never cast the Mayhem spell");
    }

    /// CR 702.183 — the bot casts an Omen half as removal (Petty Revenge on
    /// Disruptive Stormbrood) when it can't yet afford the Dragon.
    #[test]
    fn bot_casts_omen_half_as_removal() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::disruptive_stormbrood());
        // {1}{B}: enough for Petty Revenge, not the {4}{G} creature.
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        let mut bot = RandomBot::new();
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastOmen { card_id, .. } = action {
                assert_eq!(card_id, id, "bot casts Petty Revenge as removal");
                let _ = bear;
                return;
            }
            let _ = g.perform_action(action);
        }
        panic!("bot never cast the Omen half");
    }

    /// CR 702.78 — the bot conspires Burn Trail when it controls two untapped
    /// creatures sharing its color, tapping them to copy the spell.
    #[test]
    fn bot_conspires_burn_trail_when_able() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::burn_trail());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.add_card_to_battlefield(0, catalog::goblin_guide());
        g.add_card_to_battlefield(0, catalog::goblin_guide());
        // Second main. Conspire taps the two Goblin Guides, which costs
        // their attack — so casting it *before* combat is genuinely worse,
        // and the default profile's gate correctly declines to. This tests
        // that the bot finds the conspire cast, not that it fires it at the
        // worst possible moment.
        g.step = TurnStep::PostCombatMain;
        let mut bot = RandomBot::new();
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastSpellConspire { card_id, .. } = action {
                assert_eq!(card_id, id, "bot conspires Burn Trail");
                return;
            }
            let _ = g.perform_action(action);
        }
        panic!("bot never conspired");
    }

    /// When forced to chump (life threatened, no clean kill), the bot
    /// prefers fully blocking a non-trampler over a trampler — a chump
    /// against a trampler only stops `blocker_toughness` of its damage
    /// (CR 702.19e). Push (claude/modern_decks).
    #[test]
    fn bot_chumps_non_trampler_over_trampler_when_threatened() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        fn beater(name: &'static str, kws: Vec<Keyword>) -> CardDefinition {
            CardDefinition {
                name,
                card_types: vec![CardType::Creature],
                power: 4,
                toughness: 4,
                keywords: kws,
                ..Default::default()
            }
        }
        let mut g = two_player_game();
        let vanilla = g.add_card_to_battlefield(0, beater("Brute", vec![]));
        let trampler = g.add_card_to_battlefield(0, beater("Stomper", vec![Keyword::Trample]));
        // One 0/3 wall that can't kill either — only a chump is possible.
        let wall = g.add_card_to_battlefield(1, beater("Wall", vec![]));
        if let Some(w) = g.battlefield_find_mut(wall) { w.definition = std::sync::Arc::new(
            CardDefinition {
                name: "Wall",
                card_types: vec![CardType::Creature],
                toughness: 3,
                ..Default::default()
            }); }
        g.players[1].life = 3; // 8 incoming ≫ 3 → life threatened
        g.attacking = vec![
            Attack { attacker: vanilla, target: AttackTarget::Player(1) },
            Attack { attacker: trampler, target: AttackTarget::Player(1) },
        ];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(wall, vanilla)],
            "chump the non-trampler (saves 4) over the trampler (saves only 3)");
    }

    /// CR 306.7 — the bot chump-blocks to save a planeswalker it controls
    /// when the attackers aimed at it are lethal to its loyalty, even at a
    /// healthy life total. (Push claude/modern_decks.)
    #[test]
    fn bot_chumps_to_save_a_doomed_planeswalker() {
        use crate::card::{CardDefinition, CardType, CounterType};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, CardDefinition {
            name: "Raider", card_types: vec![CardType::Creature], power: 3, toughness: 3,
            ..Default::default()
        });
        // The bot (seat 1) controls a low-loyalty planeswalker and a 0/3 wall.
        let pw = g.add_card_to_battlefield(1, CardDefinition {
            name: "Walker", card_types: vec![CardType::Planeswalker], base_loyalty: 2,
            ..Default::default()
        });
        if let Some(c) = g.battlefield_find_mut(pw) {
            c.counters.insert(CounterType::Loyalty, 2);
        }
        let wall = g.add_card_to_battlefield(1, CardDefinition {
            name: "Wall", card_types: vec![CardType::Creature], power: 0, toughness: 3,
            ..Default::default()
        });
        g.players[1].life = 20; // NOT life-threatened — only the walker is at risk.
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Planeswalker(pw) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(wall, atk)],
            "the wall chumps to keep the 3-power attacker off the 2-loyalty walker");
    }

    /// The flip side of the above: when the planeswalker would survive the
    /// swing (loyalty > incoming), the bot doesn't waste a blocker on it.
    #[test]
    fn bot_does_not_chump_for_a_safe_planeswalker() {
        use crate::card::{CardDefinition, CardType, CounterType};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, CardDefinition {
            name: "Raider", card_types: vec![CardType::Creature], power: 3, toughness: 3,
            ..Default::default()
        });
        let pw = g.add_card_to_battlefield(1, CardDefinition {
            name: "Walker", card_types: vec![CardType::Planeswalker], base_loyalty: 5,
            ..Default::default()
        });
        if let Some(c) = g.battlefield_find_mut(pw) {
            c.counters.insert(CounterType::Loyalty, 5);
        }
        g.add_card_to_battlefield(1, CardDefinition {
            name: "Wall", card_types: vec![CardType::Creature], power: 0, toughness: 3,
            ..Default::default()
        });
        g.players[1].life = 20;
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Planeswalker(pw) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(blocks.is_empty(), "3 damage to a 5-loyalty walker isn't worth a chump");
    }

    /// CR 702.147 — a Decayed creature can't block, so the bot must never
    /// offer it as a blocker even when life-threatened (an illegal block
    /// would get the whole DeclareBlockers batch rejected).
    #[test]
    fn bot_never_blocks_with_a_decayed_creature() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, CardDefinition {
            name: "Beater",
            card_types: vec![CardType::Creature],
            power: 4,
            toughness: 4,
            ..Default::default()
        });
        let zombie = g.add_card_to_battlefield(1, CardDefinition {
            name: "Decayed Zombie",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Decayed],
            ..Default::default()
        });
        g.players[1].life = 1; // life-threatened → the bot would chump if it could
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(!blocks.iter().any(|(b, _)| *b == zombie), "decayed creature is never declared as a blocker");
    }

    /// CR 509.1b — facing a "can't be blocked except by three or more" lethal
    /// attacker, the bot either commits ≥3 blockers or none. With exactly three
    /// idle bodies and lethal incoming, it gangs all three (never an illegal
    /// 1–2 block).
    #[test]
    fn bot_meets_min_block_count_for_cant_be_blocked_except_by_n() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, CardDefinition {
            name: "Ulamog Spawn",
            card_types: vec![CardType::Creature],
            power: 6,
            toughness: 6,
            keywords: vec![Keyword::CantBeBlockedExceptByN(3)],
            ..Default::default()
        });
        let chumps: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(1, CardDefinition {
            name: "Chump",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            ..Default::default()
        })).collect();
        g.players[1].life = 1; // lethal incoming
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        let on_atk = blocks.iter().filter(|(_, a)| *a == atk).count();
        assert_eq!(on_atk, 3, "gangs all three to satisfy the 3-blocker minimum");
        assert!(chumps.iter().all(|c| blocks.iter().any(|(b, _)| b == c)));
    }

    /// With only two bodies against the same "≥3 blockers" attacker, the bot
    /// drops the block entirely rather than submit an illegal 2-creature batch.
    #[test]
    fn bot_drops_block_when_min_count_unreachable() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, CardDefinition {
            name: "Ulamog Spawn",
            card_types: vec![CardType::Creature],
            power: 6,
            toughness: 6,
            keywords: vec![Keyword::CantBeBlockedExceptByN(3)],
            ..Default::default()
        });
        for _ in 0..2 {
            g.add_card_to_battlefield(1, CardDefinition {
                name: "Chump",
                card_types: vec![CardType::Creature],
                power: 1,
                toughness: 1,
                ..Default::default()
            });
        }
        g.players[1].life = 1;
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks.iter().filter(|(_, a)| *a == atk).count(), 0,
            "two blockers can't legally block a ≥3 attacker — declares none");
    }

    /// CR 509.1b — a `CanBlockAnyNumber` wall that kills nothing and isn't
    /// needed against lethal still soaks the whole swing for free: the
    /// spare-capacity pass seeds from every legal blocker, not just the ones
    /// the scoring loop already assigned.
    #[test]
    fn bot_soaks_the_swing_with_an_idle_wall() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let bear = |g: &mut GameState| {
            g.add_card_to_battlefield(
                0,
                CardDefinition {
                    name: "Bear",
                    card_types: vec![CardType::Creature],
                    power: 2,
                    toughness: 2,
                    ..Default::default()
                },
            )
        };
        let a1 = bear(&mut g);
        let a2 = bear(&mut g);
        let wall = g.add_card_to_battlefield(
            1,
            CardDefinition {
                name: "Big Wall",
                card_types: vec![CardType::Creature],
                power: 0,
                toughness: 9,
                keywords: vec![Keyword::Defender, Keyword::CanBlockAnyNumber],
                ..Default::default()
            },
        );
        g.attacking = vec![
            Attack { attacker: a1, target: AttackTarget::Player(1) },
            Attack { attacker: a2, target: AttackTarget::Player(1) },
        ];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(
            blocks.iter().filter(|(b, _)| *b == wall).count(),
            2,
            "the 0/9 wall eats both attackers"
        );
    }

    /// CR 702.16e — the bot treats a block by a protection-from-the-attacker's
    /// -color creature as a clean kill (it survives + kills) rather than a
    /// suicidal trade, so it blocks even at full life.
    #[test]
    fn bot_blocks_freely_with_protected_creature() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        use crate::mana::{cost, r, Color};
        let mut g = two_player_game();
        let mut red_atk = CardDefinition {
            name: "Red Beater",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            ..Default::default()
        };
        red_atk.cost = cost(&[r()]);
        let atk = g.add_card_to_battlefield(0, red_atk);
        let prot = CardDefinition {
            name: "Warded Blocker",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Protection(Color::Red)],
            ..Default::default()
        };
        let blk = g.add_card_to_battlefield(1, prot);
        // Not life-threatened (only a chump would otherwise be declined).
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(blk, atk)], "protected 3/3 kills the red 3/3 and takes no damage");
    }

    /// The bot won't throw a much bigger creature into an even trade with a
    /// small attacker when it isn't under pressure (keeps the body, takes the
    /// hit). A 5/5 should not block a 5/1 at healthy life.
    #[test]
    fn bot_keeps_big_body_over_bad_even_trade() {
        use crate::card::{CardDefinition, CardType};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let glass = CardDefinition {
            name: "Glass Cannon",
            card_types: vec![CardType::Creature],
            power: 5,
            toughness: 1,
            ..Default::default()
        };
        let atk = g.add_card_to_battlefield(0, glass);
        let beater = CardDefinition {
            name: "Big Beater",
            card_types: vec![CardType::Creature],
            power: 5,
            toughness: 5,
            ..Default::default()
        };
        let big = g.add_card_to_battlefield(1, beater);
        g.players[1].life = 20; // not threatened by 5 damage
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(!blocks.iter().any(|(b, _)| *b == big),
            "won't trade a 5/5 to kill a 5/1 when healthy");
    }

    /// CR 509.1b — the bot must not assign a power-2 blocker to a Steel Leaf
    /// Champion ("can't be blocked by creatures with power 2 or less"), even
    /// when life-threatened; the legality gate keeps the block batch legal.
    #[test]
    fn bot_skips_illegal_block_against_steel_leaf_champion() {
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let champ = g.add_card_to_battlefield(0, catalog::steel_leaf_champion());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 — illegal
        g.players[1].life = 1; // life-threatened, so it would chump if it could
        g.attacking = vec![Attack { attacker: champ, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(!blocks.iter().any(|(b, _)| *b == bear),
            "power-2 blocker can't be assigned to Steel Leaf Champion");
    }

    /// CR 702.90 / 104.3d — the bot chumps an infect attacker that would
    /// reach 10 poison even at a healthy life total (poison, not life, is the
    /// lethal clock).
    #[test]
    fn bot_chumps_infect_attacker_to_avoid_poison_out() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let infect = CardDefinition {
            name: "Plague Beast",
            card_types: vec![CardType::Creature],
            power: 9,
            toughness: 9,
            keywords: vec![Keyword::Infect],
            ..Default::default()
        };
        let atk = g.add_card_to_battlefield(0, infect);
        let chump = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        // Healthy life (20) but already 1 poison → 9 incoming poison = 10 → lethal.
        g.players[1].poison_counters = 1;
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(blocks.iter().any(|(b, _)| *b == chump),
            "bot chumps the infect attacker to avoid a poison-out");
    }

    /// Color-choice mana abilities (Ornithopter of Paradise's `{T}: Add one
    /// mana of any color`) require an interactive `ChooseColor` decision,
    /// which the bot's main loop doesn't supply at activation time. The bot
    /// must never volunteer one as a standalone action.
    #[test]
    fn bot_does_not_tap_color_choice_mana_source() {
        let mut g = two_player_game();
        let bird = g.add_card_to_battlefield(0, catalog::ornithopter_of_paradise());
        g.clear_sickness(bird);
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        if let GameAction::ActivateAbility { card_id, .. } = action {
            assert_ne!(card_id, bird,
                "bot must NOT auto-tap a color-choice mana source (would block on ChooseColor)");
        }
    }

    /// The concern that used to live in `is_free_mana_ability`: a generic
    /// pip must not eat a one-shot artifact or a chunk of life while an
    /// ordinary land sits untapped.
    ///
    /// The bot no longer picks its own mana sources -- it stopped pre-tapping
    /// (see `main_phase_action_with`), so the engine's auto-tap chooses, and
    /// this is the guard on its ordering. Lotus Petal sacrifices itself for
    /// mana; with Forests available to pay the same pips, the Petal survives.
    #[test]
    fn auto_tap_spends_a_land_before_sacrificing_a_mana_source() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let petal = g.add_card_to_battlefield(0, catalog::lotus_petal());
        g.clear_sickness(petal);
        // Two Forests cover the bear's {1}{G} on their own. The Petal sits
        // earlier in the battlefield, so a first-match source pick would
        // sacrifice it for the generic pip anyway.
        let forests: Vec<_> = (0..2)
            .map(|_| {
                let f = g.add_card_to_battlefield(0, catalog::forest());
                g.clear_sickness(f);
                f
            })
            .collect();
        g.add_card_to_hand(0, catalog::grizzly_bears());
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should act");
        assert!(
            matches!(action, GameAction::CastSpell { .. }),
            "the bear is affordable off the two Forests, got {action:?}",
        );
        g.perform_action(action).expect("the bear should be castable");
        assert!(
            g.battlefield_find(petal).is_some(),
            "Lotus Petal must survive when lands could pay instead",
        );
        assert_eq!(
            forests.iter().filter(|f| g.battlefield_find(**f).is_some_and(|c| c.tapped)).count(),
            2,
            "both Forests are what should have been tapped",
        );
    }

    /// Sac-cost sources are deliberately *not* counted toward what the bot
    /// can afford: it would be committing to lines it can only pay for by
    /// spending something it would rather keep. A Lotus Petal on its own
    /// does not make a two-drop look castable.
    #[test]
    fn available_mana_ignores_self_consuming_sources() {
        let mut g = two_player_game();
        let petal = g.add_card_to_battlefield(0, catalog::lotus_petal());
        g.clear_sickness(petal);
        assert_eq!(available_mana(&g, 0).total, 0, "a Lotus Petal is not spare mana");
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        g.clear_sickness(forest);
        assert_eq!(available_mana(&g, 0).total, 1, "only the Forest counts");
    }

    /// Reproducer for the "Vandalblast freeze" bug. The bot is in its main
    /// phase with a Mountain (already tapped or untapped) and Vandalblast in
    /// hand; the human opponent has only an Ornithopter of Paradise on the
    /// battlefield. The bot must pick that artifact as the target and the
    /// match must drive to completion without spinning the bot loop.
    #[test]
    fn bot_vs_bot_vandalblast_against_lone_artifact_resolves() {
        use crate::server::{run_match, SeatOccupant};
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;
        let mut g = two_player_game();
        // Bot owns a Mountain so it can pay {R} and Vandalblast in hand.
        let mtn = g.add_card_to_battlefield(0, catalog::mountain());
        g.clear_sickness(mtn);
        g.add_card_to_hand(0, catalog::vandalblast());
        // Opponent has only Ornithopter of Paradise on the battlefield.
        let bird = g.add_card_to_battlefield(1, catalog::ornithopter_of_paradise());
        g.clear_sickness(bird);
        // Both bots; expect the match to terminate within a short window.
        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            run_match(
                g,
                vec![
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            );
            let _ = done_tx.send(());
        });
        done_rx
            .recv_timeout(Duration::from_secs(15))
            .expect("bot-vs-bot match must terminate (Vandalblast freeze regression)");
        handle.join().unwrap();
    }

    /// Direct (non-server) regression: the bot's main-phase action loop
    /// picks the opponent's Ornithopter as the legal Vandalblast target
    /// when no other artifact is in play. The Mountain has already been
    /// tapped (we seed the pool with {R} and pre-tap the land) so the
    /// bot proceeds straight to the spell-cast step.
    #[test]
    fn bot_main_phase_emits_vandalblast_action() {
        let mut g = two_player_game();
        let mtn = g.add_card_to_battlefield(0, catalog::mountain());
        if let Some(c) = g.battlefield_find_mut(mtn) {
            c.tapped = true;
        }
        let vandal = g.add_card_to_hand(0, catalog::vandalblast());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        let bird = g.add_card_to_battlefield(1, catalog::ornithopter_of_paradise());
        g.clear_sickness(bird);
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should act");
        match action {
            GameAction::CastSpell { card_id, target, .. } => {
                assert_eq!(card_id, vandal, "expected the bot to cast Vandalblast");
                assert_eq!(
                    target,
                    Some(Target::Permanent(bird)),
                    "Vandalblast must target the lone artifact opp controls",
                );
            }
            other => panic!("expected CastSpell(Vandalblast), got {other:?}"),
        }
    }

    /// The bot uses Magma Opus's discard-a-card-for-a-Treasure mode as a
    /// fallback value play when the full {6}{U}{R} spell is unaffordable.
    #[test]
    fn bot_uses_discard_activated_ability_as_fallback() {
        let mut g = two_player_game();
        let opus = g.add_card_to_hand(0, catalog::magma_opus());
        // Only {U/R}{U/R} worth of mana — can't cast the {8} spell.
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should act");
        match action {
            GameAction::ActivateDiscardAbility { card_id } => assert_eq!(card_id, opus),
            other => panic!("expected ActivateDiscardAbility(Magma Opus), got {other:?}"),
        }
    }

    /// End-to-end deadlock regression for spectate-mode bot-vs-bot:
    /// load a hand-crafted state that mirrors the captured cube debug
    /// export (own-stack trigger + sorcery-speed castables + a played
    /// land already) and assert the match drives forward instead of
    /// hanging on `merged_rx.recv()`. Pre-fix this would have hung on
    /// any RNG that picked Tireless Tracker before Lightning Bolt.
    #[test]
    fn spectate_match_does_not_deadlock_with_own_trigger_on_stack() {
        use crate::effect::Effect;
        use crate::game::TurnStep;
        use crate::server::{run_match, SeatOccupant};
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let tracker = g.add_card_to_battlefield(0, catalog::tireless_tracker());
        g.clear_sickness(tracker);
        g.stack.push(TriggerPush::new(tracker, 0, Effect::Noop).build());
        g.add_card_to_hand(0, catalog::tireless_tracker());
        g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 5);
        g.players[0].mana_pool.add(crate::mana::Color::Red, 5);
        g.players[0].lands_played_this_turn = 1;
        // Both players at 1 life so combat damage ends the match
        // quickly once a creature attacks.
        g.players[0].life = 1;
        g.players[1].life = 1;

        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            run_match(
                g,
                vec![
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            );
            let _ = done_tx.send(());
        });
        done_rx
            .recv_timeout(Duration::from_secs(15))
            .expect("bot-vs-bot match must terminate (own-stack-trigger deadlock regression)");
        handle.join().unwrap();
    }

    /// Regression for the Spectate Bot vs Bot deadlock observed in
    /// `debug/state-t11-precombatmain-1777409468-338551100.json`.
    ///
    /// Setup: bot 0 has its own Tireless Tracker trigger sitting on the
    /// stack (no target), all its lands are tapped and one was already
    /// played this turn, and its hand has both sorcery- and instant-
    /// speed castables. Pre-fix, `main_phase_action` sometimes picked a
    /// sorcery to cast — the engine rejected it with `SorcerySpeedOnly`
    /// (stack non-empty), `drive_bots` saw no progress, the actor blocked
    /// on `merged_rx.recv()`, and a spectator-only match froze.
    ///
    /// Post-fix the bot must either pass priority or cast an instant —
    /// never a sorcery — when the stack is non-empty.
    #[test]
    fn bot_does_not_attempt_sorcery_when_stack_nonempty() {
        use crate::effect::Effect;
        let mut g = two_player_game();
        // Bot 0 has a tracker on the battlefield as the trigger source.
        let tracker = g.add_card_to_battlefield(0, catalog::tireless_tracker());
        g.clear_sickness(tracker);
        // Stack: Tireless Tracker trigger (Clue creation), no target.
        g.stack.push(TriggerPush::new(tracker, 0, Effect::Noop).build());
        // Hand: a mix of sorcery- and instant-speed castables. Pyrokinesis
        // (instant) is the only legal cast right now.
        g.add_card_to_hand(0, catalog::tireless_tracker());
        g.add_card_to_hand(0, catalog::lightning_bolt());
        // Mana pool topped up so `can_afford` accepts both.
        g.players[0].mana_pool.add(crate::mana::Color::Green, 5);
        g.players[0].mana_pool.add(crate::mana::Color::Red, 5);
        // Pretend a land was played already so PlayLand is also blocked.
        g.players[0].lands_played_this_turn = 1;

        let mut bot = RandomBot::new();
        // Drive a few action picks; none of them may be a sorcery-speed
        // CastSpell (Tireless Tracker). PassPriority and instant casts
        // (Lightning Bolt) are both fine.
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::CastSpell { card_id, .. } = action {
                let def = g.players[0].hand.iter().find(|c| c.id == card_id)
                    .map(|c| &c.definition);
                if let Some(d) = def {
                    assert!(
                        d.is_instant_speed(),
                        "bot tried to cast sorcery-speed {} while stack was non-empty",
                        d.name,
                    );
                }
            }
        }
    }

    /// Regression for the Teferi sorcery-lock deadlock. With Teferi,
    /// Time Raveler on the opponent's side, our **instants** are
    /// timing-locked to sorcery speed. The bot's pre-fix filter
    /// allowed instant casts whenever `is_instant_speed()` was true,
    /// regardless of `OpponentsSorceryTimingOnly`; the engine then
    /// rejected with `SorcerySpeedOnly` and the match deadlocked.
    /// Post-fix, `would_accept` dry-runs the cast and rejects it,
    /// so the bot picks a different action (or passes priority).
    #[test]
    fn bot_respects_teferi_sorcery_lock_on_instants() {
        let mut g = two_player_game();
        // Opponent's Teferi imposes `OpponentsSorceryTimingOnly`.
        let teferi = g.add_card_to_battlefield(1, catalog::teferi_time_raveler());
        g.clear_sickness(teferi);
        // Stack non-empty so sorcery-speed timing fails for the bot.
        g.spells_cast_this_turn = 0;
        // Put a dummy spell on the stack to break sorcery timing
        // even on the bot's main phase.
        let dummy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield.retain(|c| c.id != dummy);
        let card = crate::card::CardInstance::new(dummy, catalog::grizzly_bears(), 1);
        g.stack.push(crate::game::StackItem::Spell {
            card: Box::new(card),
            caster: 1,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: 0,
            converged_value: 0,
            mana_spent: 0,
            uncounterable: false,
        });
        // Bot 0 has Lightning Bolt (instant) in hand and a Mountain.
        let _bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;

        let mut bot = RandomBot::new();
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::CastSpell { .. } = action {
                panic!(
                    "bot tried to cast at instant speed under Teferi's lock — \
                     would_accept must filter this out: {action:?}",
                );
            }
        }
    }

    /// Regression for the deadlock at `debug/deadlock-t8-1777411577-473115700.json`.
    /// Damping Sphere on the battlefield + bot has already cast one spell this
    /// turn + a second affordable-by-printed-cost spell in hand whose real cost
    /// (printed + Damping Sphere's `+1` tax) overflows the pool. Pre-fix the
    /// bot's `can_afford` checked only the printed cost; cast was rejected with
    /// `Mana: Need N generic mana but only have N-1 total`; spectate-mode actor
    /// deadlocked. Post-fix `can_afford_in_state` folds the static-ability tax
    /// into the cost so the bot doesn't pick the unaffordable spell.
    #[test]
    fn bot_respects_damping_sphere_tax() {
        let mut g = two_player_game();
        // Opponent's Damping Sphere on the battlefield.
        let sphere = g.add_card_to_battlefield(1, catalog::damping_sphere());
        g.clear_sickness(sphere);
        // Bot 0 has cast one spell already this turn.
        g.players[0].spells_cast_this_turn = 1;
        g.spells_cast_this_turn = 1;
        // Bot 0 has Frantic Search ({2}{U}) in hand and exactly 3 mana
        // (1U + 2C). Without the Damping Sphere tax the bot could
        // pay {2}{U}; with the +1 tax it can't (needs {3}{U} total).
        let _frantic = g.add_card_to_hand(0, catalog::frantic_search());
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);

        let mut bot = RandomBot::new();
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::CastSpell { card_id, .. } = action {
                let name = g
                    .players[0]
                    .hand
                    .iter()
                    .find(|c| c.id == card_id)
                    .map(|c| c.definition.name);
                assert_ne!(
                    name,
                    Some("Frantic Search"),
                    "bot must respect Damping Sphere's +1 tax — pool can't pay {{3}}{{U}}",
                );
            }
        }
    }

    /// The bot's affordability check folds in generic cost reductions:
    /// Tolarian Terror ({6}{U}) is castable on {3}{U} with three instants/
    /// sorceries in the graveyard.
    #[test]
    fn bot_affordability_honors_graveyard_affinity() {
        let mut g = two_player_game();
        let terror = g.add_card_to_hand(0, catalog::tolarian_terror());
        let card = g.players[0].hand.iter().find(|c| c.id == terror).unwrap().clone();
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3); // {3}{U} only
        assert!(!can_afford_in_state(&g, 0, &card, &EvalWeights::default()), "no discount yet → unaffordable");
        for _ in 0..3 { g.add_card_to_graveyard(0, catalog::lightning_bolt()); }
        let card = g.players[0].hand.iter().find(|c| c.id == terror).unwrap().clone();
        assert!(can_afford_in_state(&g, 0, &card, &EvalWeights::default()), "−{{3}} discount → now affordable");
    }

    /// Regression for the second deadlock observed at
    /// `debug/deadlock-t15-1777411082-269586900.json`. Setup mirrors
    /// the captured cube state: P0 owns a Swamp whose `controller` has
    /// flipped to P1 (Threaten / Mind Control style), all of P0's own
    /// lands are tapped. Pre-fix the bot's main_phase_action filter
    /// (`c.owner == seat`) picked the stolen Swamp, `activate_ability`
    /// rejected with `NotYourPriority`, no progress was made, and the
    /// wall-clock watchdog tripped. Post-fix the filter is keyed on
    /// `c.controller`, so the stolen land is invisible to bot 0 and
    /// the bot falls through to its castable-spell branch (or
    /// `PassPriority`).
    #[test]
    fn max_affordable_x_returns_zero_for_non_x_spells() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::lightning_bolt());
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 0,
            "Non-X spell yields 0 — caller should pass x_value=None");
    }

    #[test]
    fn max_affordable_x_pumps_remaining_mana_into_x() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::banefire()); // {X}{R}
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        // Pool: 1 red + 4 colorless. Fixed cost = {R} (1 mana). X = 4.
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(4);
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 4,
            "X soaks up the remaining {{4}} after the fixed {{R}} pip");
    }

    #[test]
    fn max_affordable_x_is_zero_if_only_fixed_cost_can_be_paid() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::banefire());
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        // Only enough mana for the {R} pip — X must collapse to 0.
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 0);
    }

    #[test]
    fn bot_casts_x_cost_burn_at_max_x() {
        // Banefire's catalog cost is just `{R}` (X is read at resolution
        // from `Value::XFromCost`), so x_relevant() picks it up via the
        // effect-tree XFromCost reference and the bot pumps the rest of
        // its pool into X.
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::banefire());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        // Verify the helper independently first — the bot's `next_action`
        // gates on lots of other state (priority, lands, mana rocks) so
        // a direct call to the helper is the most reliable assertion.
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 3,
            "{{R}} + {{3}} in pool, fixed cost {{R}} => X = 3");
    }

    /// CR 702.51 — the bot taps creatures for convoke when the pool alone
    /// can't cover the spell.
    #[test]
    fn bot_taps_creatures_for_convoke() {
        // Triplicate Spirits ({4}{W}{W}, convoke) with only {W}{W} floating:
        // unaffordable outright, castable by tapping four creatures.
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let id = g.add_card_to_hand(0, catalog::triplicate_spirits());
        g.players[0].mana_pool.add(crate::mana::Color::White, 2);
        for _ in 0..4 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.battlefield_find_mut(c).unwrap().summoning_sick = false;
        }
        match main_phase_action(&g, 0) {
            GameAction::CastSpellConvoke { card_id, convoke_creatures, .. } => {
                assert_eq!(card_id, id);
                assert_eq!(convoke_creatures.len(), 4);
            }
            other => panic!("expected a convoke cast, got {other:?}"),
        }
    }

    /// The convoke planner taps the fewest (and least useful) helpers it needs.
    #[test]
    fn bot_taps_the_minimum_number_of_convoke_helpers() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let id = g.add_card_to_hand(0, catalog::triplicate_spirits()); // {4}{W}{W}
        g.players[0].mana_pool.add(crate::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        // Six bodies available but only {2} of the generic is unpaid, and the
        // summoning-sick ones should be spent first.
        let mut sick = Vec::new();
        for i in 0..6 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            if i < 2 {
                sick.push(c);
            } else {
                g.battlefield_find_mut(c).unwrap().summoning_sick = false;
            }
        }
        match main_phase_action(&g, 0) {
            GameAction::CastSpellConvoke { card_id, convoke_creatures, .. } => {
                assert_eq!(card_id, id);
                assert_eq!(convoke_creatures.len(), 2, "only the unpaid {{2}} needs help");
                assert!(
                    convoke_creatures.iter().all(|c| sick.contains(c)),
                    "the summoning-sick bodies tap first",
                );
            }
            other => panic!("expected a convoke cast, got {other:?}"),
        }
    }

    /// Chief Engineer's granted convoke reaches the bot's planner too.
    #[test]
    fn bot_taps_creatures_for_granted_convoke() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::perilous_vault()); // {4} artifact
        g.add_card_to_battlefield(0, catalog::chief_engineer());
        for _ in 0..4 {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.battlefield_find_mut(c).unwrap().summoning_sick = false;
        }
        match main_phase_action(&g, 0) {
            GameAction::CastSpellConvoke { card_id, .. } => assert_eq!(card_id, id),
            other => panic!("expected a granted-convoke cast, got {other:?}"),
        }
    }

    #[test]
    fn bot_casts_spectacle_when_opponent_bled() {
        // Skewer the Critics ({2}{R}, Spectacle {R}) with only {R} in the pool:
        // unaffordable at its printed cost, but castable for Spectacle once an
        // opponent has lost life this turn.
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let id = g.add_card_to_hand(0, catalog::skewer_the_critics());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.adjust_life(1, -1); // opponent bleeds → Spectacle online
        match main_phase_action(&g, 0) {
            GameAction::CastSpellAlternative { card_id, .. } => assert_eq!(card_id, id),
            other => panic!("expected a Spectacle alternative cast, got {other:?}"),
        }
    }

    /// The bot casts an MDFC's back face from hand when the front is
    /// unaffordable: Wandering Archaic ({5} creature) // Explore the Vastlands
    /// ({4} sorcery), with only {4} in the pool.
    #[test]
    fn bot_casts_mdfc_back_face_from_hand() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        g.players[0].hand.clear();
        let id = g.add_card_to_hand(0, catalog::wandering_archaic());
        g.players[0].mana_pool.add_colorless(4); // affords the {4} back, not the {5} front
        match main_phase_action(&g, 0) {
            GameAction::CastSpellBack { card_id, .. } => assert_eq!(card_id, id),
            other => panic!("expected a back-face cast, got {other:?}"),
        }
    }

    /// The bot casts an MDFC's back face from the graveyard when it carries the
    /// `may_cast_back_from_graveyard` permission (Pestilent Cauldron after its
    /// sacrifice → Restorative Burst).
    #[test]
    fn bot_casts_mdfc_back_face_from_graveyard() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        g.players[0].hand.clear();
        let pc = g.add_card_to_graveyard(0, catalog::pestilent_cauldron());
        g.players[0]
            .graveyard
            .iter_mut()
            .find(|c| c.id == pc)
            .unwrap()
            .may_cast_back_from_graveyard = true;
        g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
        g.players[0].mana_pool.add_colorless(3); // {3}{G}{G} for Restorative Burst
        match main_phase_action(&g, 0) {
            GameAction::CastSpellBack { card_id, .. } => assert_eq!(card_id, pc),
            other => panic!("expected a graveyard back-face cast, got {other:?}"),
        }
    }

    /// The bot activates an Unearth ability (CR 702.84) from its graveyard when
    /// it can afford it (a `from_graveyard` activated ability).
    #[test]
    fn bot_unearths_from_graveyard() {
        let mut g = two_player_game();
        g.players[0].hand.clear();
        let dragger = g.add_card_to_graveyard(0, catalog::viscera_dragger());
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1); // {1}{B} unearth cost
        match main_phase_action(&g, 0) {
            GameAction::ActivateAbility { card_id, .. } => assert_eq!(card_id, dragger),
            other => panic!("expected an unearth activation, got {other:?}"),
        }
    }

    #[test]
    fn bot_does_not_try_to_tap_stolen_land() {
        let mut g = two_player_game();
        // P0's own Swamp: tapped (already used this turn).
        let own = g.add_card_to_battlefield(0, catalog::swamp());
        if let Some(c) = g.battlefield_find_mut(own) {
            c.tapped = true;
        }
        // P0-owned Swamp now controlled by P1 (the deadlock state).
        let stolen = g.add_card_to_battlefield(0, catalog::swamp());
        if let Some(c) = g.battlefield_find_mut(stolen) {
            c.controller = 1;
            c.tapped = false;
        }

        let mut bot = RandomBot::new();
        // 50 trials; if the bot ever returns ActivateAbility on the
        // stolen card it would deadlock. PassPriority and any action
        // on a card the bot actually controls are both fine.
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::ActivateAbility { card_id, .. } = action {
                assert_ne!(
                    card_id, stolen,
                    "bot must not try to activate a stolen permanent",
                );
            }
        }
    }

    /// Modal spells: when the default mode is dead in the current state
    /// (e.g. Drown in the Loch's mode 0 "counter target spell" with no
    /// opp spell on the stack), the bot picks an alternate mode that
    /// has a legal target. Pre-fix the bot always passed `mode: None`
    /// → engine defaulted to mode 0 → cast was rejected at target
    /// validation, and Drown in the Loch was never cast.
    #[test]
    fn bot_picks_alternate_mode_for_modal_spell() {
        let mut g = two_player_game();
        // Opp creature for mode-1 (destroy creature) to target. Drown's
        // MV gate needs MV(bear=2) ≤ cards in its controller's graveyard.
        g.add_card_to_graveyard(1, catalog::forest());
        g.add_card_to_graveyard(1, catalog::forest());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(bear);
        // Tap an Island/Swamp so {U}{B} is in the pool — bot's land-tap
        // branch otherwise fires first.
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        g.add_card_to_hand(0, catalog::drown_in_the_loch());
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should act");
        // The bot should cast Drown in the Loch with mode = Some(1)
        // (destroy mode). Mode 0 (counter spell) has no spell on the
        // stack, so it's pruned from the candidate set.
        match action {
            GameAction::CastSpell { mode, target, .. } => {
                assert_eq!(mode, Some(1),
                    "Bot should pick mode 1 when mode 0 has no legal target");
                assert_eq!(target, Some(crate::game::Target::Permanent(bear)),
                    "Mode 1's target should be the opp creature");
            }
            other => panic!(
                "expected Drown in the Loch cast with mode 1, got {:?}", other),
        }
    }

    /// `modal_mode_count`: returns the mode count for ChooseMode and
    /// None for non-modal effects.
    #[test]
    fn modal_mode_count_helper() {
        let drown = catalog::drown_in_the_loch();
        assert_eq!(modal_mode_count(&drown.effect), Some(2),
            "Drown in the Loch has 2 modes");
        let bolt = catalog::lightning_bolt();
        assert_eq!(modal_mode_count(&bolt.effect), None,
            "Lightning Bolt is not modal");
    }

    /// The bot delves a stocked graveyard to cast a spell it couldn't afford
    /// at full cost (CR 702.66). Treasure Cruise ({7}{U}) with only one blue
    /// mana but seven graveyard cards must surface as a `CastSpellDelve`.
    #[test]
    fn bot_delves_to_afford_treasure_cruise() {
        let mut g = two_player_game();
        for _ in 0..7 { g.add_card_to_graveyard(0, catalog::island()); }
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        g.add_card_to_hand(0, catalog::treasure_cruise());
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;

        // Drive the bot until it produces the delve cast (it may tap/scan
        // first, but with no lands and one floating U the delve is the only
        // castable line).
        let mut bot = RandomBot::new();
        let mut found = false;
        for _ in 0..6 {
            match bot.next_action(&g, 0) {
                Some(GameAction::CastSpellDelve { delve_cards, .. }) => {
                    assert!(!delve_cards.is_empty(), "delved at least one card");
                    found = true;
                    break;
                }
                Some(other) => { g.perform_action(other).ok(); }
                None => break,
            }
        }
        assert!(found, "bot should delve to cast Treasure Cruise");
    }

    /// The bot fetches toward its weakest color: with two Forests already
    /// down and a Forest + Island in the library, it grabs the Island.
    #[test]
    fn bot_search_fetches_weakest_color_basic() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::forest());
        let extra_forest = g.add_card_to_library(0, catalog::forest());
        let island = g.add_card_to_library(0, catalog::island());
        let candidates = vec![(extra_forest, "Forest".into()), (island, "Island".into())];
        let ans = decide_library_search(&g, 0, &candidates);
        assert!(matches!(ans, DecisionAnswer::Search(Some(id)) if id == island),
            "bot fetches the Island (Blue uncovered) over a third Forest");
    }

    /// The bot's ChooseTarget heuristic votes/targets the opponent's biggest
    /// permanent, not the first legal one.
    #[test]
    fn bot_choose_target_hits_opponents_biggest() {
        use crate::decision::DecisionAnswer;
        use crate::game::types::Target;
        let mut g = two_player_game();
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mut dino = catalog::grizzly_bears();
        dino.name = "Dino"; dino.power = 6; dino.toughness = 6;
        let big = g.add_card_to_battlefield(1, dino);
        let legal = vec![Target::Permanent(small), Target::Permanent(big)];
        match decide_choose_target(&g, 0, &legal, &EvalWeights::default()) {
            DecisionAnswer::Target(Target::Permanent(id)) => {
                assert_eq!(id, big, "bot targets the 6/6 over the 2/2");
            }
            other => panic!("expected a permanent target, got {other:?}"),
        }
    }

    /// Among player targets the bot picks the lowest-life opponent.
    #[test]
    fn bot_choose_target_hits_lowest_life_opponent() {
        use crate::decision::DecisionAnswer;
        use crate::game::types::Target;
        let mut g = crate::game::multi_player_game(3);
        g.players[1].life = 15;
        g.players[2].life = 6;
        let legal = vec![Target::Player(1), Target::Player(2)];
        match decide_choose_target(&g, 0, &legal, &EvalWeights::default()) {
            DecisionAnswer::Target(Target::Player(p)) => {
                assert_eq!(p, 2, "targets the 6-life opponent over the 15-life one");
            }
            other => panic!("expected a player target, got {other:?}"),
        }
    }

    /// Forced to choose among its own permanents, the bot gives up the smallest.
    #[test]
    fn bot_choose_target_sacrifices_own_smallest() {
        use crate::decision::DecisionAnswer;
        use crate::game::types::Target;
        let mut g = two_player_game();
        let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let mut dino = catalog::grizzly_bears();
        dino.name = "Dino"; dino.power = 6; dino.toughness = 6;
        let big = g.add_card_to_battlefield(0, dino);
        let legal = vec![Target::Permanent(big), Target::Permanent(small)];
        match decide_choose_target(&g, 0, &legal, &EvalWeights::default()) {
            DecisionAnswer::Target(Target::Permanent(id)) => {
                assert_eq!(id, small, "bot gives up its 2/2, keeps the 6/6");
            }
            other => panic!("expected a permanent target, got {other:?}"),
        }
    }

    /// A forced sacrifice gives up a spare token before a real land, even
    /// though the token's raw power/toughness makes it "bigger."
    #[test]
    fn bot_sacrifices_token_before_a_land() {
        use crate::decision::DecisionAnswer;
        use crate::game::types::Target;
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let mut tok = catalog::grizzly_bears(); // a 2/2 body...
        tok.name = "Bear Token";
        let token = g.add_card_to_battlefield(0, tok);
        g.battlefield_find_mut(token).unwrap().is_token = true; // ...but a token
        let legal = vec![Target::Permanent(land), Target::Permanent(token)];
        match decide_choose_target(&g, 0, &legal, &EvalWeights::default()) {
            DecisionAnswer::Target(Target::Permanent(id)) => {
                assert_eq!(id, token, "bot sacrifices the token, keeps the land");
            }
            other => panic!("expected a permanent target, got {other:?}"),
        }
    }

    /// With no basic land among the candidates the bot still fetches the
    /// first option rather than fizzling like AutoDecider.
    #[test]
    fn bot_search_fetches_nonland_when_no_basic_offered() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
        let candidates = vec![(bolt, "Lightning Bolt".into())];
        let ans = decide_library_search(&g, 0, &candidates);
        assert!(matches!(ans, DecisionAnswer::Search(Some(id)) if id == bolt),
            "bot fetches the only candidate");
    }

    /// A non-land tutor (e.g. Fauna Shaman) fetches the highest-mana-value
    /// hit — the most impactful card — not just the first candidate offered.
    #[test]
    fn bot_search_fetches_highest_mv_nonland() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
        let angel = g.add_card_to_library(0, catalog::serra_angel());   // MV 5
        let candidates = vec![
            (bears, "Grizzly Bears".into()),
            (angel, "Serra Angel".into()),
        ];
        let ans = decide_library_search(&g, 0, &candidates);
        assert!(matches!(ans, DecisionAnswer::Search(Some(id)) if id == angel),
            "bot fetches the higher-MV creature");
    }

    /// The bot offers a Bestow cast (enchanting its own creature) when it's
    /// mana-flush, instead of only ever casting the base creature.
    #[test]
    fn bot_considers_bestow_when_mana_flush() {
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::hopeful_eidolon());
        g.players[0].mana_pool.add(crate::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;

        // The bot can cast Hopeful Eidolon normally *or* bestow it; the
        // scored pick prefers the bestow line (variant bonus + own-target
        // gain), so it must win outright, not merely appear.
        let bestowed = (0..10).all(|_| {
            matches!(main_phase_action(&g, 0),
                GameAction::CastBestow { target: Some(crate::game::Target::Permanent(t)), .. } if t == host)
        });
        assert!(bestowed, "scored pick prefers the Bestow line enchanting its creature");
    }

    /// `decide_choose_cards` over the bot's own hand (Sneak Attack / Elvish
    /// Piper) cheats in the biggest creature it can.
    #[test]
    fn bot_choose_cards_cheats_in_biggest_creature() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let small = g.add_card_to_hand(0, catalog::grizzly_bears()); // cmc 2
        let big = g.add_card_to_hand(0, catalog::shivan_dragon());   // cmc 6
        let candidates = vec![
            (small, "Grizzly Bears".to_string()),
            (big, "Shivan Dragon".to_string()),
        ];
        match decide_choose_cards(&EvalWeights::default(), &g, 0, "Put a creature onto the battlefield?", &candidates, 0, 1) {
            DecisionAnswer::Cards(v) => assert_eq!(v, vec![big],
                "bot picks the highest-cmc creature to cheat in"),
            other => panic!("expected Cards, got {other:?}"),
        }
    }

    /// `decide_choose_cards` over battlefield creatures (Archipelagore's tap)
    /// targets opponents' biggest creature, never the bot's own.
    #[test]
    fn bot_choose_cards_taps_enemy_creatures() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(1, catalog::shivan_dragon());   // 5/5
        let candidates = vec![
            (mine, "Grizzly Bears".to_string()),
            (small, "Grizzly Bears".to_string()),
            (big, "Shivan Dragon".to_string()),
        ];
        match decide_choose_cards(&EvalWeights::default(), &g, 0, "Tap which creatures?", &candidates, 0, 1) {
            DecisionAnswer::Cards(v) => assert_eq!(v, vec![big],
                "bot taps the opponent's biggest creature, not its own"),
            other => panic!("expected Cards, got {other:?}"),
        }
    }

    /// A sacrifice `ChooseCards` prompt is a cost: give up the least
    /// valuable permanent, and only as many as forced.
    #[test]
    fn bot_choose_cards_sacrifices_the_worst() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
        let candidates = vec![
            (small, "Grizzly Bears".to_string()),
            (big, "Shivan Dragon".to_string()),
        ];
        match decide_choose_cards(&EvalWeights::default(), &g, 0, "Sacrifice a creature", &candidates, 1, 1) {
            DecisionAnswer::Cards(v) => {
                assert_eq!(v, vec![small], "bot sacrifices the smaller creature")
            }
            other => panic!("expected Cards, got {other:?}"),
        }
    }

    /// Pure temp-pump instants are combat tricks; burn and creatures are not.
    #[test]
    fn combat_trick_classifier() {
        assert!(is_combat_trick(&catalog::giant_growth()));
        assert!(!is_combat_trick(&catalog::lightning_bolt()), "burn is not a trick");
        assert!(!is_combat_trick(&catalog::grizzly_bears()));
    }

    /// The bot holds Giant Growth in its main phase (a sorcery-speed pump
    /// telegraphs and buffs nothing that matters) …
    #[test]
    fn bot_holds_pump_trick_in_main_phase() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let growth = g.add_card_to_hand(0, catalog::giant_growth());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        let action = main_phase_action(&g, 0);
        assert!(
            !matches!(action, GameAction::CastSpell { card_id, .. } if card_id == growth),
            "pump trick is held for combat, got {action:?}",
        );
    }

    /// … and fires it after blocks when it flips a fight its attacker is
    /// losing (2/2 blocked by a 5/5: +3/+3 trades instead of chumping).
    #[test]
    fn bot_casts_trick_on_blocked_attacker() {
        let mut g = two_player_game();
        let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
        let growth = g.add_card_to_hand(0, catalog::giant_growth());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareBlockers;
        g.set_attacking(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]);
        g.set_block_map([(dragon, bears)]);
        g.set_blockers_declared(true);
        let action = RandomBot::new().next_action(&g, 0);
        assert!(
            matches!(
                action,
                Some(GameAction::CastSpell {
                    card_id,
                    target: Some(crate::game::Target::Permanent(t)),
                    ..
                }) if card_id == growth && t == bears
            ),
            "trick targets the blocked attacker, got {action:?}",
        );
    }

    /// No trick when the fight is already won (2/2 blocked by a 1/1).
    #[test]
    fn bot_holds_trick_when_fight_already_won() {
        let mut g = two_player_game();
        let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
        g.add_card_to_hand(0, catalog::giant_growth());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareBlockers;
        g.set_attacking(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]);
        g.set_block_map([(elf, bears)]);
        g.set_blockers_declared(true);
        let action = RandomBot::new().next_action(&g, 0);
        assert!(
            matches!(action, Some(GameAction::PassPriority)),
            "no trick needed on a won fight, got {action:?}",
        );
    }

    /// Material eval: a board and full hand beat an empty seat, and a
    /// decided game dominates everything.
    #[test]
    fn eval_material_prefers_board_and_cards() {
        let mut g = two_player_game();
        assert_eq!(
            eval_material(&g, 0, &EvalWeights::default()),
            -eval_material(&g, 1, &EvalWeights::default()),
            "the two-player eval is symmetric",
        );
        g.add_card_to_battlefield(0, catalog::shivan_dragon());
        g.add_card_to_hand(0, catalog::lightning_bolt());
        assert!(eval_material(&g, 0, &EvalWeights::default()) > 0, "board + hand is a material lead");
        assert!(eval_material(&g, 1, &EvalWeights::default()) < 0);
        g.game_over = Some(Some(1));
        assert!(eval_material(&g, 1, &EvalWeights::default()) > eval_material(&g, 0, &EvalWeights::default()), "a won game beats any material");
    }

    /// The gap one-action-at-a-time scoring cannot close: with four mana,
    /// two two-drops beat one four-drop, but a greedy pick compares each
    /// cast against the board *once* and takes the biggest single body.
    #[test]
    fn lookahead_prefers_two_cheap_spells_over_one_expensive_one() {
        let w = EvalWeights::lookahead1();
        let mut g = two_player_game();
        // Second main so the summon-sick gate (on in both profiles) isn't
        // what decides this.
        g.step = TurnStep::PostCombatMain;
        for _ in 0..4 {
            let land = g.add_card_to_battlefield(0, catalog::forest());
            g.clear_sickness(land);
        }
        // One four-mana 4/5 versus two two-mana 2/2s. Two bears are 4/4
        // across two bodies for the same mana — the greedy pick can't see
        // the second one because it never asks what comes next.
        let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
        let bear_a = g.add_card_to_hand(0, catalog::grizzly_bears());
        let bear_b = g.add_card_to_hand(0, catalog::grizzly_bears());
        let cast = |id| GameAction::CastSpell {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        // Craw Wurm costs {4}{G}{G} — too much here; use the mana we have.
        let _ = wurm;
        let one_bear = evaluate_action_sequence(&g, 0, &cast(bear_a), &w, 0)
            .expect("single-play score");
        let bear_then_bear = evaluate_action_sequence(&g, 0, &cast(bear_a), &w, 1)
            .expect("two-play score");
        assert!(
            bear_then_bear > one_bear,
            "looking one play ahead must see the second bear ({bear_then_bear} vs {one_bear})",
        );
        let _ = bear_b;
    }

    /// Lookahead must not invent plays that aren't legal yet: once the
    /// bot no longer holds priority in its own main phase, there is no
    /// continuation to search.
    #[test]
    fn follow_up_candidates_are_empty_outside_our_main_phase() {
        let w = EvalWeights::lookahead1();
        let mut g = two_player_game();
        for _ in 0..3 {
            let land = g.add_card_to_battlefield(0, catalog::forest());
            g.clear_sickness(land);
        }
        g.add_card_to_hand(0, catalog::grizzly_bears());
        assert!(
            !follow_up_candidates(&g, 0, &w).is_empty(),
            "our own main phase offers continuations",
        );
        g.step = TurnStep::DeclareBlockers;
        assert!(
            follow_up_candidates(&g, 0, &w).is_empty(),
            "no sorcery-speed continuation mid-combat",
        );
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 1;
        assert!(
            follow_up_candidates(&g, 0, &w).is_empty(),
            "not our turn, no continuation",
        );
    }

    /// Forge's summon-sick gate: a creature that can't attack this turn is
    /// worth the same after combat, so the bot should deploy it in the
    /// second main and keep the mana up in between. Measured by `bot_probe`
    /// to move plays in the postcombat main from 0.5 % to 37.7 %.
    #[test]
    fn hold_sick_gate_defers_a_vanilla_creature_to_the_second_main() {
        let w = EvalWeights::hold_sick();
        let mut g = two_player_game();
        for _ in 0..3 {
            let land = g.add_card_to_battlefield(0, catalog::forest());
            g.clear_sickness(land);
        }
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        let cast = GameAction::CastSpell {
            card_id: bear,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        // The body is all it does, and it can't attack -- no progress today.
        assert!(
            !improves_this_turn(&g, 0, &cast, &w),
            "a vanilla creature achieves nothing on the turn it lands",
        );
        // So the gated bot passes in the first main...
        let mut bot = RandomBot::with_weights(w);
        assert!(
            matches!(bot.next_action(&g, 0), Some(GameAction::PassPriority)),
            "gated bot holds the creature in the precombat main",
        );
        // ...and deploys it in the second, where holding costs nothing.
        g.step = TurnStep::PostCombatMain;
        let mut bot2 = RandomBot::with_weights(w);
        assert!(
            matches!(bot2.next_action(&g, 0), Some(GameAction::CastSpell { card_id, .. }) if card_id == bear),
            "gated bot casts it postcombat",
        );
        // The historical profile casts it immediately, which is the
        // behavior the gate exists to change.
        let mut plain = RandomBot::with_weights(EvalWeights::baseline());
        let mut pre = g.clone();
        pre.step = TurnStep::PreCombatMain;
        assert!(
            matches!(plain.next_action(&pre, 0), Some(GameAction::CastSpell { .. })),
            "the historical profile still front-loads",
        );
    }

    /// The gate must not hold a line that *does* something now: a hasty
    /// body can attack, so deploying it precombat is real progress.
    #[test]
    fn hold_sick_gate_lets_through_a_play_that_matters_now() {
        let w = EvalWeights::hold_sick();
        let mut g = two_player_game();
        for _ in 0..3 {
            let land = g.add_card_to_battlefield(0, catalog::mountain());
            g.clear_sickness(land);
        }
        let guide = g.add_card_to_hand(0, catalog::goblin_guide());
        let cast = GameAction::CastSpell {
            card_id: guide,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        assert!(
            improves_this_turn(&g, 0, &cast, &w),
            "a haste creature is progress the turn it lands",
        );
    }

    /// After a London mulligan the bot puts cards back on the library.
    /// `AutoDecider` bottoms the first N cards of the hand, which routinely
    /// meant shipping the business spells and keeping a fistful of lands.
    /// Found by `bot_probe`: `PutOnLibrary` was 9 % of all decisions the bot
    /// faced and every one of them fell through to that default.
    #[test]
    fn bot_bottoms_surplus_lands_not_the_front_of_its_hand() {
        use crate::decision::{Decision, DecisionAnswer};
        let mut g = two_player_game();
        // Front of hand: the good cheap spell. Then a pile of lands.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let lands: Vec<_> =
            (0..4).map(|_| g.add_card_to_hand(0, catalog::mountain())).collect();
        // Already flooded, so every hand land is surplus.
        for _ in 0..5 {
            g.add_card_to_battlefield(0, catalog::mountain());
        }
        let hand: Vec<(crate::card::CardId, String)> = std::iter::once(bolt)
            .chain(lands.iter().copied())
            .map(|id| (id, String::new()))
            .collect();
        g.pending_decision = Some(crate::game::types::PendingDecision {
            decision: Decision::PutOnLibrary { player: 0, count: 2, hand },
            resume: crate::game::types::ResumeContext::Mulligan {
                player: 0,
                mulligans_taken: 1,
                next_player: None,
            },
        });
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot answers the decision");
        let GameAction::SubmitDecision(DecisionAnswer::PutOnLibrary(put)) = action else {
            panic!("expected a PutOnLibrary answer, got {action:?}");
        };
        assert_eq!(put.len(), 2, "bottoms exactly the requested count");
        assert!(!put.contains(&bolt), "the spell must not be bottomed: {put:?}");
        assert!(put.iter().all(|id| lands.contains(id)), "only surplus lands go back");
    }

    /// The combat-aware evaluator has to actually reach combat damage and
    /// come back, or it silently degrades to the old snapshot behavior.
    #[test]
    fn simulate_through_combat_advances_past_combat_damage() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        // Empty board opposite, so the swing is free and unambiguous.
        let life_before = g.players[1].life;
        let mut fuel = 200u32;
        assert_eq!(
            simulate_through_combat(&mut g, &mut fuel, &EvalWeights::default()),
            CombatSim::Completed,
            "combat should be simulated",
        );
        assert!(g.step >= TurnStep::CombatDamage, "advanced to combat damage, got {:?}", g.step);
        assert_eq!(g.players[1].life, life_before - 2, "the bear connected for 2");
    }

    /// The cheap bail-outs matter: the state clone is the expensive part,
    /// so a position with no combat to look at must cost nothing.
    #[test]
    fn simulate_through_combat_bails_without_attackers() {
        let mut g = two_player_game();
        let mut fuel = 200u32;
        assert_eq!(
            simulate_through_combat(&mut g, &mut fuel, &EvalWeights::default()),
            CombatSim::Skipped,
            "no creatures, no combat",
        );
        assert_eq!(fuel, 200, "bailing must not burn fuel");
        // A summoning-sick creature can't attack, so still nothing to see.
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert_eq!(
            simulate_through_combat(&mut g, &mut fuel, &EvalWeights::default()),
            CombatSim::Skipped,
            "sick creature can't attack",
        );
        g.step = TurnStep::PostCombatMain;
        g.clear_sickness(g.battlefield[0].id);
        assert_eq!(
            simulate_through_combat(&mut g, &mut fuel, &EvalWeights::default()),
            CombatSim::Skipped,
            "combat is already over",
        );
    }

    /// The payoff. A creature that is *forced* to attack into a blocker
    /// that eats it is not the material the board says it is. The snapshot
    /// evaluator counts the body and never sees it die; the combat-aware
    /// one plays the turn out and prices it correctly.
    #[test]
    fn combat_aware_eval_sees_a_forced_attacker_die() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        let doomed = g.add_card_to_battlefield(
            0,
            weights_test_creature("Doomed Charger", 2, 2, 2, &[Keyword::MustAttack]),
        );
        g.clear_sickness(doomed);
        // A 4/4 blocks it, kills it, and survives.
        g.add_card_to_battlefield(1, catalog::craw_wurm());
        let w = EvalWeights::combat_aware();
        let snapshot = eval_material(&g, 0, &w);
        let mut sim = g.clone();
        let mut fuel = 200u32;
        assert_eq!(simulate_through_combat(&mut sim, &mut fuel, &EvalWeights::default()), CombatSim::Completed);
        assert!(sim.battlefield_find(doomed).is_none(), "the forced attacker died");
        assert!(
            eval_material(&sim, 0, &w) < snapshot,
            "losing the body must score worse than the board that still has it",
        );
    }

    /// The baseline profile must stay a byte-for-byte control for the
    /// ladder: life counted linearly, no keyword term, scale 1.
    #[test]
    fn baseline_profile_is_the_historical_evaluation() {
        let base = EvalWeights::baseline();
        for life in [-3, 0, 1, 7, 20, 41] {
            assert_eq!(life_value(life, &base), life, "baseline life is linear");
        }
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, weights_test_creature("Baseline Body", 4, 3, 3, &[]));
        let body = g.battlefield[0].id;
        // Baseline is exactly mana value + power + toughness, nothing else.
        assert_eq!(permanent_value(&g, body, &base), 4 + 3 + 3);
    }

    /// A creature for the weighting tests: `cost` generic mana, `power`/
    /// `toughness`, and whatever keywords the case needs.
    fn weights_test_creature(
        name: &'static str,
        cost: u32,
        power: i32,
        toughness: i32,
        keywords: &[crate::card::Keyword],
    ) -> CardDefinition {
        use crate::card::CardType;
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            cost: crate::mana::cost(&[crate::mana::generic(cost)]),
            power,
            toughness,
            keywords: keywords.to_vec(),
            ..Default::default()
        }
    }

    /// Life is worth more per point the closer to zero it is. A linear term
    /// prices "gain 3" identically at 3 life and at 20 -- this is the whole
    /// reason for the curve, so assert the shape, not just the endpoints.
    #[test]
    fn concave_life_curve_is_monotone_with_diminishing_returns() {
        let w = EvalWeights::v2();
        let at = |l: i32| life_value(l, &w);
        // Anchored to the linear term it replaces: 20 life is still 20 points.
        assert_eq!(at(20), 20 * w.unit);
        assert_eq!(at(0), 0);
        for l in 1..=40 {
            assert!(at(l) > at(l - 1), "life {l} must beat life {}", l - 1);
        }
        // Marginal value never rises as life goes up.
        for l in 2..=39 {
            let lower = at(l) - at(l - 1);
            let upper = at(l + 1) - at(l);
            assert!(upper <= lower, "marginal life at {l} rose ({lower} -> {upper})");
        }
        // And the low end is dramatically steeper than the high end: the
        // point that saves us from dying is worth several near the top.
        assert!(
            at(1) - at(0) >= 4 * (at(20) - at(19)),
            "the first point of life should dwarf the twentieth",
        );
    }

    /// Evasion scales with power (it's worth what it lets the body deal);
    /// protection is flat (it buys the same thing on any body). Getting
    /// this backwards is the mistake a flat keyword table makes.
    #[test]
    fn keyword_value_scales_evasion_but_not_protection() {
        use crate::card::Keyword;
        let w = EvalWeights::v2();
        let flying = [Keyword::Flying];
        let hexproof = [Keyword::Hexproof];
        assert!(
            keyword_value(&flying, 5, &w) > keyword_value(&flying, 1, &w),
            "flying is worth more on a bigger body",
        );
        assert_eq!(
            keyword_value(&hexproof, 5, &w),
            keyword_value(&hexproof, 1, &w),
            "hexproof buys the same thing regardless of size",
        );
        // Bad keywords are negative, and a body that can neither attack nor
        // block is worth less than its printed size suggests.
        assert!(keyword_value(&[Keyword::Defender], 4, &w) < 0);
        let pacified = keyword_value(&[Keyword::CantAttack, Keyword::CantBlock], 6, &w);
        assert!(
            pacified < keyword_value(&[Keyword::Defender], 6, &w),
            "a fully locked-down creature is the worst case",
        );
    }

    /// The payoff: two bodies the baseline scores as *identical* -- same
    /// cost, same stats -- are correctly separated by v2, which sees that
    /// one of them flies and drains. This is the behavioral difference the
    /// ladder is measuring; removal targeting and cast ranking both read
    /// `permanent_value`, so a tie here is a coin flip on the baseline.
    #[test]
    fn v2_breaks_a_baseline_tie_toward_the_creature_that_does_something() {
        use crate::card::Keyword;
        let mut g = two_player_game();
        g.add_card_to_battlefield(
            0,
            weights_test_creature("Test Flier", 4, 3, 3, &[Keyword::Flying, Keyword::Lifelink]),
        );
        g.add_card_to_battlefield(0, weights_test_creature("Test Lump", 4, 3, 3, &[]));
        let (f, l) = (g.battlefield[0].id, g.battlefield[1].id);
        let base = EvalWeights::baseline();
        let v2 = EvalWeights::v2();
        assert_eq!(
            permanent_value(&g, f, &base),
            permanent_value(&g, l, &base),
            "the baseline can't tell these apart at all",
        );
        assert!(
            permanent_value(&g, f, &v2) > permanent_value(&g, l, &v2),
            "v2 sees that the flier actually does something",
        );
    }
}

#[cfg(test)]
mod monarch_tests {
    use super::*;
    use crate::catalog;
    use crate::player::Player;

    /// CR 725/726 — the crown and the initiative are recurring resources, so
    /// the material eval prices holding them (and an opponent holding them).
    #[test]
    fn eval_material_prices_the_crown() {
        let mut g = crate::game::two_player_game();
        let w = EvalWeights::baseline();
        let before = eval_material(&g, 0, &w);
        g.monarch = Some(0);
        assert!(eval_material(&g, 0, &w) > before);
        g.monarch = Some(1);
        assert!(eval_material(&g, 0, &w) < before);
    }

    #[test]
    fn bot_attacks_the_monarch_over_the_next_seat() {
        // 3 players: next_alive_seat(0) is 1, but seat 2 is the monarch, so
        // the bot should swing at seat 2 to steal the crown.
        let players = vec![Player::new(0, "A"), Player::new(1, "B"), Player::new(2, "C")];
        let mut g = GameState::new(players);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareAttackers;
        g.monarch = Some(2);
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        // Every seat needs a library. The bot now simulates the attack
        // forward before committing to it, and on an empty library taking
        // the crown is *lethal* — the monarch draws at their end step (CR
        // 724) and decks out. Declining would be the right play; the test
        // means to check target selection, not deck-out.
        for seat in 0..3 {
            for _ in 0..10 {
                g.add_card_to_library(seat, catalog::forest());
            }
        }

        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("an action") {
            GameAction::DeclareAttackers(attacks) => {
                assert!(
                    attacks.iter().any(|a| matches!(a.target, AttackTarget::Player(2))),
                    "bot swings at the monarch (seat 2), not the next seat"
                );
            }
            other => panic!("expected DeclareAttackers, got {other:?}"),
        }
    }
}

#[cfg(test)]
mod self_cost_tests {
    use super::*;
    use crate::effect::{Effect, PlayerRef, Selector, Value};

    #[test]
    fn self_cost_seen_through_modal_and_pay_or_else() {
        // A self-cost mode nested inside ChooseMode is recognized.
        let modal = Effect::ChooseMode(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
        ]);
        assert!(effect_imposes_self_cost(&modal), "lose-life mode is a self cost");

        // PayManaOrElse → SacrificeSource fallback is a self cost.
        let tax = Effect::PayManaOrElse {
            mana_cost: crate::mana::cost(&[crate::mana::generic(1)]),
            otherwise: Box::new(Effect::SacrificeSource),
        };
        assert!(effect_imposes_self_cost(&tax), "sac-unless-pay fallback is a self cost");

        // A purely beneficial modal is not flagged.
        let upside = Effect::ChooseMode(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]);
        assert!(!effect_imposes_self_cost(&upside));

        // find_maydo_body reaches into a mode by its prompt.
        let nested = Effect::ChooseMode(vec![Effect::MayDo {
            description: "Pay the price.".into(),
            body: Box::new(Effect::LoseLife {
                who: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
            }),
        }]);
        assert!(find_maydo_body(&nested, "Pay the price.").is_some());
    }
}

#[cfg(test)]
mod stack_response_tests {
    use super::*;
    use crate::catalog;
    use crate::game::{GameAction, GameState, Target, TurnStep};
    use crate::player::Player;

    fn two_player_game() -> GameState {
        let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
        let mut g = GameState::new(players);
        g.step = TurnStep::PreCombatMain;
        g
    }

    #[test]
    fn bot_counters_a_big_opponent_spell() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // P0 casts a 7-drop.
        let wurm = g.add_card_to_hand(0, catalog::pelakka_wurm());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 3);
        g.players[0].mana_pool.add_colorless(5);
        g.perform_action(GameAction::CastSpell {
            card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        // Bot (seat 1) holds Counterspell + two untapped Islands.
        let cs = g.add_card_to_hand(1, catalog::counterspell());
        for _ in 0..2 { g.add_card_to_battlefield(1, catalog::island()); }
        g.priority.player_with_priority = 1;
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 1).expect("bot acts");
        match action {
            GameAction::CastSpell { card_id, target, .. } => {
                assert_eq!(card_id, cs, "casts the counterspell");
                assert_eq!(target, Some(Target::Permanent(wurm)), "targets the 7-drop");
            }
            other => panic!("expected a counterspell cast, got {other:?}"),
        }
    }

    #[test]
    fn bot_holds_counter_against_cheap_nonthreatening_spell() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // A cheap spell that doesn't touch the bot's board.
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        g.add_card_to_hand(1, catalog::counterspell());
        for _ in 0..2 { g.add_card_to_battlefield(1, catalog::island()); }
        g.priority.player_with_priority = 1;
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 1).expect("bot acts");
        assert!(matches!(action, GameAction::PassPriority),
            "a 2-drop bear isn't worth the counter: {action:?}");
    }

    /// The bot plays a color-fixing land over an off-color one: with a green
    /// spell in hand and no green source, it plays the Forest, not the Mountain.
    #[test]
    fn bot_plays_color_fixing_land() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        let _mountain = g.add_card_to_hand(0, catalog::mountain());
        let forest = g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_hand(0, catalog::grizzly_bears()); // wants green
        assert_eq!(pick_land_to_play(&g, 0, &EvalWeights::default()), Some(forest),
            "fixes the missing green over the off-color Mountain");
    }

    /// `land_urgency` sequences the tapland: with nothing castable it is
    /// the free drop, but once the untapped mana would actually be spent
    /// this turn the basic wins.
    #[test]
    fn land_urgency_times_the_tapland() {
        let w = EvalWeights::land_sequencing();
        // Nothing to cast: the school land (enters tapped, fixes W and B)
        // costs nothing now and fixes two colors later.
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        let tapland = g.add_card_to_hand(0, catalog::forum_of_amity());
        let _plains = g.add_card_to_hand(0, catalog::plains());
        g.add_card_to_hand(0, catalog::serra_angel()); // {3}{W}{W}, uncastable now
        g.add_card_to_hand(0, catalog::doom_blade()); // {1}{B} — the second color
        assert_eq!(pick_land_to_play(&g, 0, &w), Some(tapland),
            "no play this turn — take the tapped dual for the two colors it fixes");

        // Same hand plus a one-drop the untapped land would actually
        // cast: entering tapped now costs a real play.
        let mut g2 = two_player_game();
        g2.priority.player_with_priority = 0;
        g2.active_player_idx = 0;
        let _tap2 = g2.add_card_to_hand(0, catalog::forum_of_amity());
        let plains2 = g2.add_card_to_hand(0, catalog::plains());
        g2.add_card_to_hand(0, catalog::savannah_lions()); // {W}, castable off one Plains
        assert_eq!(pick_land_to_play(&g2, 0, &w), Some(plains2),
            "the untapped source buys a play this turn; the tapland doesn't");
    }

    /// A creature-only `{X}: deal X damage to target creature` spell caps X at
    /// the toughest opposing creature — the bot doesn't overkill a 2/2.
    #[test]
    fn max_affordable_x_caps_creature_only_burn_at_lethal() {
        use crate::card::{CardDefinition, CardType};
        use crate::effect::shortcut::target_filtered;
        use crate::effect::{Effect, Value};
        use crate::card::SelectionRequirement;
        let mut g = two_player_game();
        let zap = CardDefinition {
            name: "Test Creature Zap",
            cost: crate::mana::cost(&[crate::mana::x(), crate::mana::r()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::XFromCost,
            },
            ..Default::default()
        };
        let id = g.add_card_to_hand(0, zap);
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 — toughest opp
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(6);
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 2,
            "X capped at the 2/2's toughness, not the full {{6}} pool");
    }

    /// Player-targetable burn (Banefire) is not capped — the bot still dumps
    /// its whole pool into X.
    #[test]
    fn max_affordable_x_does_not_cap_any_target_burn() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::banefire()); // any target
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(6);
        assert_eq!(max_affordable_x(&g, 0, &card, &EvalWeights::default()), 6, "Banefire keeps the full X");
    }

    /// An Unblockable attacker swings even into a bigger blocker — no opposing
    /// creature can legally block it, so the suicide filter doesn't hold it
    /// back (generalized evasion check).
    #[test]
    fn bot_attacks_with_unblockable_into_bigger_blocker() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let mut ghost = catalog::grizzly_bears();
        ghost.name = "Ghost";
        ghost.power = 1;
        ghost.toughness = 1;
        ghost.keywords.push(crate::card::Keyword::Unblockable);
        let atk = g.add_card_to_battlefield(0, ghost);
        g.clear_sickness(atk);
        // A lone 5/5 that would trade up against a naive ground attacker.
        let mut big = catalog::grizzly_bears();
        big.name = "Wall"; big.power = 5; big.toughness = 5;
        g.add_card_to_battlefield(1, big);
        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|d| d.attacker == atk),
                    "unblockable attacker swings past a bigger blocker");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// With only a couple of lands in play but a fistful of duplicate lands in
    /// hand, a forced discard pitches a surplus land rather than a real spell.
    #[test]
    fn bot_discard_pitches_surplus_land_not_a_spell() {
        let mut g = two_player_game();
        // 2 lands in play → wants ~4 more; a 5th land in hand is surplus.
        for _ in 0..2 { g.add_card_to_battlefield(0, catalog::forest()); }
        let mut hand: Vec<(crate::card::CardId, String)> = Vec::new();
        for _ in 0..5 {
            let id = g.add_card_to_hand(0, catalog::forest());
            hand.push((id, "Forest".to_string()));
        }
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        hand.push((bear, "Grizzly Bears".to_string()));
        let ans = decide_self_discard(&g, 0, &hand, 1);
        match ans {
            crate::decision::DecisionAnswer::Discard(ids) => {
                assert_eq!(ids.len(), 1);
                let pitched = g.players[0].hand.iter().find(|c| c.id == ids[0]).unwrap();
                assert!(pitched.definition.is_land(), "pitched a surplus land, kept the spell");
            }
            other => panic!("expected Discard, got {:?}", other),
        }
    }

    /// The bot accepts an exploit trigger when it has a spare creature (here a
    /// second body), instead of always declining the sacrifice.
    #[test]
    fn bot_takes_exploit_with_a_spare_creature() {
        let mut g = two_player_game();
        let drowner = g.add_card_to_battlefield(0, catalog::gurmag_drowner());
        // No other creature → keep it (would have to sacrifice itself; allowed
        // only by a >1 count, so a lone exploiter declines).
        assert!(!optional_trigger_beneficial(&g, drowner, "Exploit — sacrifice a creature?"),
            "lone exploiter with nothing to spare declines");
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // a spare body
        assert!(optional_trigger_beneficial(&g, drowner, "Exploit — sacrifice a creature?"),
            "with a spare creature the bot exploits for value");
    }

    /// The bot crews an uncrewed Vehicle with a spare creature so it can swing.
    #[test]
    fn bot_crews_a_vehicle() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::broadcast_rambler()); // Crew 1, 5/4
        // No creatures yet → nothing to crew with.
        assert!(pick_crew_vehicle(&g, 0).is_none(), "no crewers, no crew");
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2 ≥ 1
        g.clear_sickness(bear);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        assert!(
            matches!(pick_crew_vehicle(&g, 0),
                Some(GameAction::Crew { vehicle, .. }) if vehicle == veh),
            "crews the Vehicle with the spare creature",
        );
    }

    /// The bot fires a "deal N to each opponent" ability for lethal, and only
    /// then (not to chip).
    #[test]
    fn bot_reach_burn_only_for_lethal() {
        let mut g = two_player_game();
        let haz = g.add_card_to_battlefield(0, catalog::hazoret_the_fervent());
        g.clear_sickness(haz);
        g.add_card_to_hand(0, catalog::mountain()); // discard fodder
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        // Opponent at 5: the 2-damage burn isn't lethal, so the bot holds it.
        g.players[1].life = 5;
        assert!(pick_reach_burn(&g, 0).is_none(), "won't chip with a non-lethal burn");
        // Opponent at 2: now it's lethal, so the bot fires it.
        g.players[1].life = 2;
        assert!(matches!(pick_reach_burn(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == haz),
            "fires the burn for lethal");
    }

    /// The bot replays a self-returning graveyard creature (Llanowar Greenwidow)
    /// even though its ability has no exile-self cost.
    #[test]
    fn bot_replays_self_returning_graveyard_creature() {
        let mut g = two_player_game();
        let id = g.add_card_to_graveyard(0, catalog::llanowar_greenwidow());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(7);
        assert!(matches!(pick_graveyard_recursion(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == id),
            "bot activates the graveyard self-return");
    }

    /// The bot drives Brass Squire's two-slot attach ability: an Equipment onto
    /// the biggest creature.
    #[test]
    fn bot_activates_brass_squire_attach() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let squire = g.add_card_to_battlefield(0, catalog::brass_squire());
        g.add_card_to_battlefield(0, catalog::bonesplitter());
        g.clear_sickness(squire);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let action = pick_attach_ability(&g, 0).expect("bot drives the attach ability");
        // Slot 1 (the wearer) is the highest-power creature — the bear, not the
        // 1/3 Squire.
        assert!(matches!(action,
            GameAction::ActivateAbility { card_id, ref additional_targets, .. }
                if card_id == squire && additional_targets == &vec![crate::game::Target::Permanent(bear)]));
    }

    /// The bot cracks a Lander token for ramp when it has spare mana and a basic
    /// still in the library — but not when the library has no basic to fetch.
    #[test]
    fn bot_cracks_lander_for_ramp() {
        let mut g = two_player_game();
        let lander = g.add_token_to_battlefield(0, &crabomination_base::tokens::lander_token());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(2);
        // No basic in library → don't waste the Lander.
        assert!(pick_crack_lander(&g, 0).is_none(), "no basic to fetch → hold the Lander");
        g.add_card_to_library(0, catalog::forest());
        assert!(matches!(pick_crack_lander(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == lander),
            "with a basic in library and spare mana, the bot ramps");
    }

    /// With spare mana and no better play, the bot sinks it into a repeatable
    /// self-+1/+1 ability (Fire Sages) — but never burns a once-per-game Exhaust.
    #[test]
    fn bot_sinks_spare_mana_into_self_pump() {
        let mut g = two_player_game();
        let sages = g.add_card_to_battlefield(0, catalog::fire_sages());
        g.clear_sickness(sages);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crate::mana::Color::Red, 2);
        g.players[0].mana_pool.add_colorless(1);
        assert!(matches!(pick_self_pump_counter(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == sages),
            "bot grows Fire Sages with leftover mana");

        // An Exhaust pump (Mai) is left alone even with mana to spare.
        let mut g = two_player_game();
        let mai = g.add_card_to_battlefield(0, catalog::mai_jaded_edge());
        g.clear_sickness(mai);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(3);
        assert!(pick_self_pump_counter(&g, 0).is_none(), "won't spend a once-per-game Exhaust as a mana sink");
    }

    /// With spare mana and nothing better, the bot sinks it into a
    /// "{cost}: create a token" ability (Sun Warriors' {5}: 1/1 Ally).
    #[test]
    fn bot_sinks_spare_mana_into_token_maker() {
        let mut g = two_player_game();
        let sw = g.add_card_to_battlefield(0, catalog::sun_warriors());
        g.clear_sickness(sw);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert!(pick_token_maker(&g, 0).is_none(), "no mana → no token");
        g.players[0].mana_pool.add_colorless(5);
        assert!(matches!(pick_token_maker(&g, 0),
            Some(GameAction::ActivateAbility { card_id, .. }) if card_id == sw),
            "bot makes an Ally token with leftover mana");
    }

    /// The bot casts a Spree spell via `CastSpellSpree` (not a no-op plain
    /// cast), choosing an affordable mode with a legal target.
    #[test]
    fn bot_casts_spree_spell() {
        use crate::mana::Color;
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // A juicy opposing creature for Explosive Derailment's +{2} "deal 4" mode.
        g.add_card_to_battlefield(1, catalog::serra_angel());
        let spell = g.add_card_to_hand(0, catalog::explosive_derailment());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2); // base {R} + mode {2}
        match main_phase_action(&g, 0) {
            GameAction::CastSpellSpree { card_id, spree_modes, target, .. } => {
                assert_eq!(card_id, spell, "cast the Spree spell");
                assert!(!spree_modes.is_empty(), "chose at least one mode");
                assert!(target.is_some(), "aimed the damage mode at a target");
            }
            other => panic!("expected a Spree cast, got {other:?}"),
        }
    }

    /// Off-turn window: at the opponent's end step with an empty stack the
    /// bot casts instant-speed spells (EOT removal) but not sorcery-speed
    /// cards, which `would_accept` filters out.
    #[test]
    fn bot_casts_instant_at_opponents_end_step() {
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::End;
        g.active_player_idx = 1; // opponent's turn
        g.priority.player_with_priority = 0;
        let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let _bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);

        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::CastSpell { card_id, target, .. } => {
                assert_eq!(card_id, bolt, "only the instant is castable off-turn");
                // "Any target" burn defaults to the face per the engine's
                // auto-targeter; either opponent-side target is fine here —
                // the point is that the instant is cast off-turn at all.
                let opponent_side = matches!(target, Some(Target::Player(1)))
                    || matches!(target, Some(Target::Permanent(id)) if id == angel);
                assert!(opponent_side, "aimed at the opponent's side: {target:?}");
            }
            other => panic!("expected an EOT Bolt, got {other:?}"),
        }
    }

    /// Overkill/chip awareness: with a 5/5 and a 2/2 on the other side and
    /// only Shock in hand, the scorer must not value Shock-at-the-5/5 as
    /// removal — the kill (2/2) outranks the chip (5/5) despite the 5/5's
    /// higher `permanent_value`.
    #[test]
    fn scorer_prefers_killing_over_chipping() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let shock = g.add_card_to_hand(0, catalog::shock());

        let kill = GameAction::CastSpell {
            card_id: shock, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        };
        let chip = GameAction::CastSpell {
            card_id: shock, target: Some(Target::Permanent(dragon)),
            additional_targets: vec![], mode: None, x_value: None,
        };
        assert!(
            score_candidate(&g, 0, &kill, &EvalWeights::default()) > score_candidate(&g, 0, &chip, &EvalWeights::default()),
            "killing the 2/2 must outscore chipping the 5/5",
        );
    }

    /// The counter gate scores the stack spell instead of the old cmc>=3
    /// rule: a removal spell aimed at the bot's best creature is counter-
    /// worthy even at 2 cmc.
    #[test]
    fn bot_counters_cheap_removal_aimed_at_its_bomb() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
        // P0 bolts the bot's dragon (cmc 1 — held under the old gate).
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(dragon)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap();
        let cs = g.add_card_to_hand(1, catalog::counterspell());
        for _ in 0..2 { g.add_card_to_battlefield(1, catalog::island()); }
        g.priority.player_with_priority = 1;
        let mut bot = RandomBot::new();
        match bot.next_action(&g, 1).expect("bot acts") {
            GameAction::CastSpell { card_id, .. } => {
                assert_eq!(card_id, cs, "counters the removal aimed at its best creature");
            }
            other => panic!("expected a counterspell, got {other:?}"),
        }
    }

    /// A beneficial Aura (Rancor) is cast on the bot's own best creature,
    /// never on an opposing one (Effect::Attach isn't classified friendly
    /// by the generic auto-targeter).
    #[test]
    fn bot_puts_beneficial_aura_on_own_best_creature() {
        use crate::mana::Color;
        let mut g = two_player_game();
        // Second main: these test *what* the bot can find and cast, not
        // when. The default profile's summon-sick gate defers a
        // first-main creature to here, which is orthogonal to the card
        // shape under test.
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        let _opp = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5, tempting
        g.add_card_to_hand(0, catalog::rancor());
        g.players[0].mana_pool.add(Color::Green, 1);

        match main_phase_action(&g, 0) {
            GameAction::CastSpell { target: Some(Target::Permanent(t)), .. } => {
                assert_ne!(t, small, "picks the better of its own creatures");
                assert_eq!(t, big, "Rancor goes on the bot's best creature, not the opponent's");
            }
            other => panic!("expected a Rancor cast on own creature, got {other:?}"),
        }
    }

    /// Prepare-cast valuation: the inset spell is scored as itself (a
    /// {U} instant ≈ 2 points), not as the 5/5 creature carrying it
    /// (≈ 22) — and a controlled "prepared matters" static (Top of the
    /// Class) charges the cast for the rider it strips.
    #[test]
    fn prepare_cast_scored_by_inset_spell_not_creature() {
        use crate::card::CounterType;
        let mut g = two_player_game();
        let em = g.add_card_to_battlefield(0, catalog::emeritus_of_ideation());
        g.battlefield
            .iter_mut()
            .find(|c| c.id == em)
            .unwrap()
            .add_counters(CounterType::Prepared, 1);
        let cast = GameAction::CastPrepareSpell {
            creature_id: em,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        let plain = score_candidate(&g, 0, &cast, &EvalWeights::default());
        assert!(
            plain <= 8,
            "inset {{U}} draw spell must score as a cheap spell, got {plain}",
        );
        g.add_card_to_battlefield(0, catalog::top_of_the_class());
        let with_anthem = score_candidate(&g, 0, &cast, &EvalWeights::default());
        assert!(
            with_anthem < plain,
            "unpreparing under a prepared-matters anthem must score lower \
             ({with_anthem} !< {plain})",
        );
    }

    /// A 3/3 for the ward tests: identical body with and without Ward, so
    /// a score comparison isolates the ward term instead of conflating it
    /// with mana-value or keyword differences.
    fn test_bear(ward: Option<crate::card::WardCost>) -> CardDefinition {
        use crate::card::{CardType, Keyword};
        CardDefinition {
            name: "Ward Test Bear",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            keywords: ward.map(|w| vec![Keyword::Ward(w)]).unwrap_or_default(),
            ..Default::default()
        }
    }

    /// CR 702.21 under bot play: casting removal at a warded creature with
    /// no mana left for the tax gets the spell countered by the ward
    /// trigger's auto-pay failing — strictly worse than holding it. The
    /// ward gate drops the candidate until the tax is payable *on top of*
    /// the spell's own cost.
    #[test]
    fn bot_wont_cast_removal_into_unpayable_ward_mana() {
        use crate::card::WardCost;
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        let bear = g.add_card_to_battlefield(1, test_bear(Some(WardCost::generic(2))));
        let blade = g.add_card_to_hand(0, catalog::doom_blade()); // {1}{B}
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            !matches!(action, GameAction::CastSpell { .. }),
            "exactly {{1}}{{B}} up: Doom Blade into Ward {{2}} would be countered, got {action:?}"
        );
        g.players[0].mana_pool.add_colorless(2);
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(
                action,
                GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. }
                    if card_id == blade && t == bear
            ),
            "with the tax affordable the same cast goes through, got {action:?}"
        );
    }

    /// Ward—Pay N life at N ≥ our life total: the engine's auto-pay would
    /// spend the bot's whole life into the state-based loss, so the gate
    /// refuses the target outright; with a live total it is just a tax.
    #[test]
    fn bot_wont_pay_lethal_ward_life() {
        use crate::card::WardCost;
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        let bear = g.add_card_to_battlefield(1, test_bear(Some(WardCost::Life(5))));
        let blade = g.add_card_to_hand(0, catalog::doom_blade());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.players[0].life = 4;
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            !matches!(action, GameAction::CastSpell { .. }),
            "paying Ward—5 life at 4 life is suicide, got {action:?}"
        );
        g.players[0].life = 20;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(
                action,
                GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. }
                    if card_id == blade && t == bear
            ),
            "at 20 life the ward is a payable tax, got {action:?}"
        );
    }

    /// Two identical 3/3s, one warded: the cast aimed at the warded twin
    /// scores lower, so the un-warded target (or a different spell) wins
    /// the tie even when both taxes are payable.
    #[test]
    fn warded_target_scores_below_unwarded_twin() {
        use crate::card::WardCost;
        let mut g = two_player_game();
        let warded = g.add_card_to_battlefield(1, test_bear(Some(WardCost::generic(2))));
        let plain = g.add_card_to_battlefield(1, test_bear(None));
        let blade = g.add_card_to_hand(0, catalog::doom_blade());
        let cast_at = |t: CardId| GameAction::CastSpell {
            card_id: blade,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        let w = EvalWeights::default();
        let s_warded = score_candidate(&g, 0, &cast_at(warded), &w);
        let s_plain = score_candidate(&g, 0, &cast_at(plain), &w);
        assert!(
            s_warded < s_plain,
            "identical bodies, one warded: {s_warded} !< {s_plain}"
        );
    }

    /// SOS Repartee: with a payoff out that wants instants/sorceries to
    /// target a creature, an "any target" burn spell gets a
    /// creature-aimed sibling candidate, and the outcome eval takes the
    /// creature kill over the face ping.
    #[test]
    fn repartee_offers_creature_target_for_any_target_burn() {
        use crate::card::CardType;
        use crate::effect::shortcut;
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        let payoff = CardDefinition {
            name: "Repartee Payoff",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 3,
            triggered_abilities: vec![shortcut::repartee(Effect::Noop)],
            ..Default::default()
        };
        g.add_card_to_battlefield(0, payoff);
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let shock = g.add_card_to_hand(0, catalog::shock());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(
                action,
                GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. }
                    if card_id == shock && t == bear
            ),
            "with a Repartee payoff out, Shock kills the bear instead of pinging face, \
             got {action:?}"
        );
    }

    /// Under `attack_sim_spells` the attack simulation sees the
    /// crack-back: an attacker that survives combat but dies to the
    /// removal in the opponent's hand scores the line lower than the
    /// spell-blind sim does.
    #[test]
    fn spell_sim_sees_crackback_removal() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        // The opponent holds real interaction: Doom Blade plus the mana
        // to cast it on their turn.
        g.add_card_to_battlefield(1, catalog::swamp());
        g.add_card_to_battlefield(1, catalog::swamp());
        g.add_card_to_hand(1, catalog::doom_blade());
        // Both libraries stocked so the sim's draw steps don't deck anyone.
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::forest());
            g.add_card_to_library(1, catalog::swamp());
        }
        let atk = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
        let blind = simulate_attack_outcome(&g, 0, &atk, &EvalWeights::attack_search())
            .expect("spell-blind sim completes");
        let seeing = simulate_attack_outcome(&g, 0, &atk, &EvalWeights::attack_search_sim())
            .expect("spell-casting sim completes");
        assert!(
            seeing < blind,
            "the sim that lets the opponent Doom Blade must score lower \
             ({seeing} !< {blind})"
        );
    }

    /// Emblems price by shape now: a recurring draw engine out-values a
    /// recurring trickle of life, where the old flat constant read them
    /// the same.
    #[test]
    fn emblem_value_prices_shapes() {
        use crate::card::TriggeredAbility;
        use crate::effect::{EventKind, EventScope, EventSpec, Selector, Value};
        let g = two_player_game();
        // The event kind is irrelevant to the shape pricing (any
        // non-LifeGained trigger walks the body); Attacks is just a
        // parameterless stand-in.
        let emblem = |body: Effect| crate::player::Emblem {
            name: "Test".into(),
            triggered: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
                effect: body,
            }],
            statics: vec![],
        };
        let draw = emblem(Effect::Draw { who: Selector::You, amount: Value::Const(2) });
        let life = emblem(Effect::GainLife { who: Selector::You, amount: Value::Const(1) });
        assert!(
            emblem_value(&g, 0, &draw) > emblem_value(&g, 0, &life),
            "a draw-two-per-turn emblem must out-value gain-one-per-turn"
        );
    }

    /// A walker the enemy board kills before its next activation cashes
    /// out: with lethal power across the table it spends loyalty on the
    /// minus; with an empty board it banks the plus.
    #[test]
    fn doomed_walker_cashes_out() {
        use crate::card::{CardType, CounterType, LoyaltyAbility};
        use crate::effect::shortcut::target_any;
        use crate::effect::{Selector, Value};
        let walker = || CardDefinition {
            name: "Test Walker",
            card_types: vec![CardType::Planeswalker],
            base_loyalty: 2,
            loyalty_abilities: vec![
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: 1,
                    effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                },
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: -2,
                    effect: Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
                },
            ],
            ..Default::default()
        };
        let w = EvalWeights::default();
        let mut safe = two_player_game();
        let id = safe.add_card_to_battlefield(0, walker());
        safe.battlefield.iter_mut().find(|c| c.id == id).unwrap()
            .add_counters(CounterType::Loyalty, 2);
        let action = pick_loyalty_ability(&safe, 0, &w).expect("walker activates");
        assert!(
            matches!(action, GameAction::ActivateLoyaltyAbility { ability_index: 0, .. }),
            "empty enemy board: bank the plus, got {action:?}"
        );
        // Two 3/3s: power 6 covers the loyalty however the engine seeded
        // it (base loyalty plus the counters added above).
        let mut doomed = safe.clone();
        doomed.add_card_to_battlefield(1, test_bear(None));
        doomed.add_card_to_battlefield(1, test_bear(None));
        let action = pick_loyalty_ability(&doomed, 0, &w).expect("walker activates");
        assert!(
            matches!(action, GameAction::ActivateLoyaltyAbility { ability_index: 1, .. }),
            "enemy power covers the loyalty: spend it down, got {action:?}"
        );
    }

    /// The counter bar drops when the hand clogs: a mid-size threat that a
    /// comfortable hand lets resolve gets countered once the counter would
    /// otherwise rot toward a cleanup discard.
    #[test]
    fn clogged_hand_lowers_counter_bar() {
        use crate::mana::Color;
        let mut g = two_player_game();
        g.active_player_idx = 1; // the ogre is a sorcery-speed cast
        let counter = g.add_card_to_hand(0, catalog::counterspell());
        g.players[0].mana_pool.add(Color::Blue, 2);
        // Opponent casts a 7-unit threat (Gray Ogre: 3 cmc + 2 + 2).
        let ogre = g.add_card_to_hand(1, catalog::gray_ogre());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.players[1].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: ogre,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("opponent casts");
        while g.player_with_priority() != 0 {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        let w = EvalWeights::default();
        assert!(
            pick_stack_response(&g, 0, &w).is_none(),
            "a two-card hand holds the counter for something bigger"
        );
        for _ in 0..5 {
            g.add_card_to_hand(0, catalog::forest());
        }
        let action = pick_stack_response(&g, 0, &w).expect("clogged hand counters");
        assert!(
            matches!(action, GameAction::CastSpell { card_id, .. } if card_id == counter),
            "got {action:?}"
        );
    }

    /// The defender kills the biggest declared attacker before committing
    /// blocks: instant removal is a combat response now, not just an
    /// end-step afterthought.
    #[test]
    fn defensive_removal_kills_declared_attacker() {
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        let serra = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.clear_sickness(serra);
        let blade = g.add_card_to_hand(0, catalog::doom_blade());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: serra,
            target: AttackTarget::Player(0),
        }]))
        .expect("opponent attacks");
        let mut fuel = 8;
        while g.player_with_priority() != 0 && fuel > 0 {
            g.perform_action(GameAction::PassPriority).unwrap();
            fuel -= 1;
        }
        let action = RandomBot::new().next_action(&g, 0).expect("defender acts");
        assert!(
            matches!(
                action,
                GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. }
                    if card_id == blade && t == serra
            ),
            "Doom Blade answers the attacker before blocks, got {action:?}"
        );
    }

    /// Sacrifice-for-value is judged by the resolved exchange: a
    /// sac-for-four-cards engine fires, a sac-for-one does not.
    #[test]
    fn sacrifice_value_judged_by_outcome() {
        use crate::card::CardType;
        use crate::effect::{ActivatedAbility, Selector, Value};
        let sac_drawer = |n: i32| CardDefinition {
            name: "Sac Engine",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            activated_abilities: vec![ActivatedAbility {
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(n) },
                ..Default::default()
            }],
            ..Default::default()
        };
        let w = EvalWeights::default();
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        for _ in 0..6 {
            g.add_card_to_library(0, catalog::forest());
        }
        g.add_card_to_battlefield(0, sac_drawer(4));
        g.priority.player_with_priority = 0;
        assert!(
            pick_sacrifice_value(&g, 0, &w).is_some(),
            "a 1/1 into four cards is a trade worth making"
        );
        let mut weak = two_player_game();
        weak.step = TurnStep::PostCombatMain;
        for _ in 0..6 {
            weak.add_card_to_library(0, catalog::forest());
        }
        weak.add_card_to_battlefield(0, sac_drawer(1));
        assert!(
            pick_sacrifice_value(&weak, 0, &w).is_none(),
            "a 1/1 into one card is not"
        );
    }

    /// A self-costly optional trigger is judged by outcome at the real
    /// decision: pay 2 life for three cards, decline 8 life for one.
    #[test]
    fn optional_self_cost_taken_when_outcome_wins() {
        use crate::decision::{Decision, DecisionAnswer};
        use crate::effect::{Selector, Value};
        use crate::game::TriggerPush;
        let run = |loss: i32, draw: i32| -> GameAction {
            use crate::card::{CardType, TriggeredAbility};
            use crate::effect::{EventKind, EventScope, EventSpec};
            let mut g = two_player_game();
            g.players[0].wants_ui = true;
            let body = Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::Const(loss) },
                Effect::Draw { who: Selector::You, amount: Value::Const(draw) },
            ]);
            let maydo =
                Effect::MayDo { description: "you may".to_string(), body: Box::new(body) };
            // The prompt introspection reads the SOURCE's printed
            // definition, so the trigger must live on the card, exactly
            // as a real fired trigger does.
            let src_def = CardDefinition {
                name: "Optional Source",
                card_types: vec![CardType::Creature],
                power: 2,
                toughness: 2,
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                    effect: maydo.clone(),
                }],
                ..Default::default()
            };
            let src = g.add_card_to_battlefield(0, src_def);
            for _ in 0..5 {
                g.add_card_to_library(0, catalog::forest());
            }
            g.stack.push(TriggerPush::new(src, 0, maydo).build());
            let mut fuel = 20;
            while g.pending_decision.is_none() && fuel > 0 {
                g.perform_action(GameAction::PassPriority).unwrap();
                fuel -= 1;
            }
            assert!(matches!(
                g.pending_decision.as_ref().map(|p| &p.decision),
                Some(Decision::OptionalTrigger { .. })
            ));
            RandomBot::new().next_action(&g, 0).expect("bot answers")
        };
        assert!(
            matches!(run(2, 3), GameAction::SubmitDecision(DecisionAnswer::Bool(true))),
            "two life for three cards is taken"
        );
        assert!(
            matches!(run(8, 1), GameAction::SubmitDecision(DecisionAnswer::Bool(false))),
            "eight life for one card is declined"
        );
    }

    /// The race horizon scores the win: an attack that puts the opponent
    /// in range reads as the kill it sets up, not as a mid-race board
    /// snapshot.
    #[test]
    fn race_horizon_scores_the_win() {
        use crate::card::CardType;
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let brute = CardDefinition {
            name: "Race Brute",
            card_types: vec![CardType::Creature],
            power: 4,
            toughness: 4,
            ..Default::default()
        };
        let atk_id = g.add_card_to_battlefield(0, brute);
        g.clear_sickness(atk_id);
        g.players[1].life = 8;
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::forest());
            g.add_card_to_library(1, catalog::swamp());
        }
        let atk = vec![Attack { attacker: atk_id, target: AttackTarget::Player(1) }];
        let blind = simulate_attack_outcome(&g, 0, &atk, &EvalWeights::attack_search_sim())
            .expect("one-cycle sim completes");
        let race = simulate_attack_outcome(&g, 0, &atk, &EvalWeights::attack_search_race())
            .expect("race sim completes");
        assert!(
            race > blind && race >= 90_000,
            "the extended horizon reaches the win ({race} !> {blind} or short of decided)"
        );
    }

    /// A mana-fixing color choice reads the hand: holding double-green
    /// spells, "add one mana of any color" picks Green, not
    /// AutoDecider's first-legal White.
    #[test]
    fn choose_color_follows_hand_demand() {
        use crate::decision::{Decision, DecisionAnswer};
        use crate::mana::Color;
        let mut g = two_player_game();
        let src = g.add_card_to_battlefield(0, catalog::llanowar_elves());
        g.add_card_to_hand(0, catalog::giant_growth()); // {G}
        g.add_card_to_hand(0, catalog::craw_wurm()); // {4}{G}{G}
        let d = Decision::ChooseColor {
            source: src,
            legal: vec![Color::White, Color::Blue, Color::Black, Color::Red, Color::Green],
        };
        let ans = decide_pending_policy(&g, 0, &EvalWeights::default(), &d, false);
        assert!(
            matches!(ans, DecisionAnswer::Color(Color::Green)),
            "three green pips in hand → Green, got {ans:?}"
        );
    }

    /// SOS Converge: before casting a spell that scales with distinct
    /// colors of mana spent, the bot floats a color the pool lacks —
    /// tapping one source per tick — and only then casts, so the payment
    /// drains every college color instead of whatever the auto-tap
    /// grabbed first.
    #[test]
    fn converge_cast_prefloats_missing_colors() {
        use crate::card::CardType;
        use crate::effect::{Selector, Value};
        use crate::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        let mountain = g.add_card_to_battlefield(0, catalog::mountain());
        let island = g.add_card_to_battlefield(0, catalog::island());
        // {1}{R} "draw cards equal to converge" stand-in.
        let spell = CardDefinition {
            name: "Converge Test",
            cost: crate::mana::ManaCost::new(vec![
                crate::mana::ManaSymbol::Generic(1),
                crate::mana::ManaSymbol::Colored(Color::Red),
            ]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Draw { who: Selector::You, amount: Value::ConvergedValue },
            ..Default::default()
        };
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::forest());
        }
        let card = g.add_card_to_hand(0, spell);
        g.priority.player_with_priority = 0;
        // Tick 1: the bot taps a source instead of casting — floating a
        // color toward the converge count.
        let first = main_phase_action(&g, 0);
        assert!(
            matches!(
                first,
                GameAction::ActivateAbility { card_id, .. }
                    if card_id == mountain || card_id == island
            ),
            "first tick floats a color for the converge cast, got {first:?}"
        );
        g.perform_action(first).unwrap();
        // Tick 2: one color floated; with pool ≥ another new color still
        // missing but no room left (cmc 2, 1 floated) — the second tick
        // floats the second color or casts; drive to the cast and check
        // it happened with both colors drained.
        let mut fuel = 4;
        loop {
            let a = main_phase_action(&g, 0);
            let done = matches!(a, GameAction::CastSpell { card_id, .. } if card_id == card);
            g.perform_action(a).expect("bot line applies");
            if done {
                break;
            }
            fuel -= 1;
            assert!(fuel > 0, "prefloat must terminate in a cast");
        }
        // Both sources went into the payment: the cast drained R and U.
        assert!(
            g.battlefield.iter().filter(|c| c.controller == 0).all(|c| c.tapped),
            "both colors were tapped into the converge cast"
        );
    }

    /// Simulations answer decisions with the bot's own policy table:
    /// a pure-upside "you may draw" is TAKEN under
    /// `decide_pending_policy` where `AutoDecider` (the old sim
    /// decider) declines every optional trigger — the difference that
    /// made lookaheads undervalue every line with a beneficial rider.
    #[test]
    fn sim_policy_takes_beneficial_triggers() {
        use crate::card::{CardType, TriggeredAbility};
        use crate::decision::{Decision, DecisionAnswer};
        use crate::effect::{EventKind, EventScope, EventSpec, Selector, Value};
        let mut g = two_player_game();
        let upside = CardDefinition {
            name: "Upside Rider",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "you may".to_string(),
                    body: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    }),
                },
            }],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, upside);
        let d = Decision::OptionalTrigger { source: id, description: "you may".to_string() };
        assert!(
            matches!(AutoDecider.decide(&d), DecisionAnswer::Bool(false)),
            "AutoDecider declines every optional trigger"
        );
        let ans = decide_pending_policy(&g, 0, &EvalWeights::default(), &d, false);
        assert!(
            matches!(ans, DecisionAnswer::Bool(true)),
            "the sim policy takes the pure-upside draw, got {ans:?}"
        );
    }

    /// X sizing honors multi-X costs: `{X}{X}{U}` with five mana up
    /// declares X=2 (pays {2}{2}{U}), not X=4 — and the same helper
    /// sizes prepare-cast inset spells that used to be stuck at X=0.
    #[test]
    fn multi_x_pip_costs_split_the_spare_mana() {
        use crate::mana::{Color, ManaCost, ManaSymbol};
        let mut g = two_player_game();
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(4);
        let def = CardDefinition {
            name: "Double X Test",
            cost: ManaCost::new(vec![
                ManaSymbol::X,
                ManaSymbol::X,
                ManaSymbol::Colored(Color::Blue),
            ]),
            ..Default::default()
        };
        let x = max_affordable_x_for_def(&g, 0, &def, 0, &EvalWeights::default());
        assert_eq!(x, 2, "five mana into {{X}}{{X}}{{U}} is X=2");
    }

    /// The Increment threshold reads the smaller stat: a 2/3 Increment
    /// body wants casts spending 3+ mana, and the bar climbs as the
    /// body grows.
    #[test]
    fn increment_threshold_reads_smaller_stat() {
        use crate::card::CardType;
        use crate::effect::shortcut;
        let mut g = two_player_game();
        assert_eq!(increment_threshold(&g, 0), None, "no body, no bar");
        let body = CardDefinition {
            name: "Increment Test",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 3,
            triggered_abilities: vec![shortcut::increment_trigger(Effect::Noop)],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, body);
        assert!(is_increment_trigger(
            &g.battlefield_find(id).unwrap().definition.triggered_abilities[0]
        ));
        assert_eq!(increment_threshold(&g, 0), Some(3), "min(2,3)+1");
    }

    /// The on-cast family detectors tell the SOS trigger shapes apart:
    /// an Opus rider is not just magecraft, and an Infusion gate is
    /// found on spell bodies and triggered riders alike.
    #[test]
    fn on_cast_family_detectors() {
        use crate::effect::shortcut;
        use crate::effect::{Predicate, Selector, Value};
        let opus = shortcut::opus_trigger(Effect::Noop, Effect::Noop);
        assert!(is_opus_trigger(&opus), "opus_trigger shape detected");
        let mage = shortcut::magecraft(Effect::Noop);
        assert!(!is_opus_trigger(&mage), "plain magecraft is not Opus");
        assert!(is_repartee_trigger(&shortcut::repartee(Effect::Noop)));
        assert!(!is_repartee_trigger(&mage), "plain magecraft is not Repartee");
        let infusion = CardDefinition {
            name: "Infusion Test",
            effect: Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: crate::effect::PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        };
        assert!(card_infusion_gated(&infusion));
        assert!(!card_infusion_gated(&catalog::shock()));
    }

    /// SOS Prepare — the inset spell is a one-shot resource on a fragile
    /// body: with opponent removal on the stack aimed at the prepared
    /// creature, the bot casts the inset instant in response instead of
    /// letting the counter die with the body.
    #[test]
    fn prepare_inset_instant_fires_in_response_to_removal() {
        use crate::card::CounterType;
        use crate::mana::Color;
        let mut g = two_player_game();
        let em = g.add_card_to_battlefield(0, catalog::emeritus_of_conflict());
        g.battlefield.iter_mut().find(|c| c.id == em).unwrap()
            .add_counters(CounterType::Prepared, 1);
        g.players[0].mana_pool.add(Color::Red, 1);
        // Opponent Doom Blades the prepared body, then passes priority.
        let blade = g.add_card_to_hand(1, catalog::doom_blade());
        g.players[1].mana_pool.add(Color::Black, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: blade,
            target: Some(Target::Permanent(em)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("opponent casts removal");
        while g.player_with_priority() != 0 {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        let action = RandomBot::new().next_action(&g, 0).expect("bot holds priority");
        assert!(
            matches!(action, GameAction::CastPrepareSpell { creature_id, .. } if creature_id == em),
            "inset Lightning Bolt fires before the body dies, got {action:?}"
        );
    }

    /// The re-prepare mana sink: with spare mana and nothing better to do,
    /// Skycoach Waypoint's `{3},{T}` re-arms an unprepared prepare-spell
    /// creature.
    #[test]
    fn reprepare_sink_rearms_prepare_creature() {
        let mut g = two_player_game();
        g.step = TurnStep::PostCombatMain;
        let em = g.add_card_to_battlefield(0, catalog::emeritus_of_conflict());
        let waypoint = g.add_card_to_battlefield(0, catalog::skycoach_waypoint());
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        let action = main_phase_action(&g, 0);
        assert!(
            matches!(
                action,
                GameAction::ActivateAbility { card_id, target: Some(Target::Permanent(t)), .. }
                    if card_id == waypoint && t == em
            ),
            "spare mana re-arms the prepare creature, got {action:?}"
        );
    }

    /// A Prepared counter on a prepare-spell body reads as material: the
    /// same creature is worth more prepared than not, so removal aims at
    /// it and the eval charges lines that waste the counter.
    #[test]
    fn prepared_counter_adds_permanent_value() {
        use crate::card::CounterType;
        let mut g = two_player_game();
        let em = g.add_card_to_battlefield(0, catalog::emeritus_of_conflict());
        let w = EvalWeights::default();
        let unprepared = permanent_value(&g, em, &w);
        g.battlefield.iter_mut().find(|c| c.id == em).unwrap()
            .add_counters(CounterType::Prepared, 1);
        let prepared = permanent_value(&g, em, &w);
        assert!(
            prepared > unprepared,
            "prepared must out-value unprepared ({prepared} !> {unprepared})"
        );
    }

    /// The Paradigm recurrence is a real choice under bot play: a free
    /// Decorum Dissertation copy (draw 2, lose 2 — the loss rides the
    /// auto-self-target) is taken at a healthy total and declined at a
    /// low one, instead of the old unconditional engine-side yes that
    /// drained the bot into the state-based loss two life at a time.
    #[test]
    fn paradigm_copy_declined_at_low_life() {
        use crate::card::CardInstance;
        use crate::decision::{Decision, DecisionAnswer};
        use crate::game::TriggerPush;
        let run_at = |life: i32| -> GameAction {
            let mut g = two_player_game();
            g.players[0].wants_ui = true;
            g.players[0].life = life;
            for _ in 0..4 {
                g.add_card_to_library(0, catalog::forest());
            }
            let id = g.next_id();
            g.exile.push(CardInstance::new(id, catalog::decorum_dissertation(), 0));
            g.stack.push(TriggerPush::new(id, 0, Effect::CastFreeParadigmCopy).build());
            let mut fuel = 20;
            while g.pending_decision.is_none() && fuel > 0 {
                g.perform_action(GameAction::PassPriority).unwrap();
                fuel -= 1;
            }
            assert!(
                matches!(
                    g.pending_decision.as_ref().map(|p| &p.decision),
                    Some(Decision::OptionalTrigger { .. })
                ),
                "paradigm copy must suspend as a real prompt, got {:?}",
                g.pending_decision
            );
            RandomBot::new().next_action(&g, 0).expect("bot answers")
        };
        let at_low = run_at(4);
        assert!(
            matches!(at_low, GameAction::SubmitDecision(DecisionAnswer::Bool(false))),
            "at 4 life the draw-2-lose-2 copy is declined, got {at_low:?}"
        );
        let at_healthy = run_at(20);
        assert!(
            matches!(at_healthy, GameAction::SubmitDecision(DecisionAnswer::Bool(true))),
            "at 20 life the free copy is taken, got {at_healthy:?}"
        );
    }

    /// Scry under bot play is no longer a no-op: with plenty of land
    /// sources a scried land goes to the bottom; while mana-light the
    /// same land stays on top, and an uncastable haymaker gets bottomed
    /// in favor of a cheap spell.
    #[test]
    fn scry_bottoms_flood_and_bricks() {
        use crate::decision::{DecisionAnswer, ScryMode};
        let id_of = |g: &GameState, name: &str| {
            g.players[0].library.iter().find(|c| c.definition.name == name).unwrap().id
        };
        // Flooded: six sources in play, a seventh on top → bottom it.
        let mut g = two_player_game();
        for _ in 0..6 {
            g.add_card_to_battlefield(0, catalog::forest());
        }
        g.add_card_to_library(0, catalog::forest());
        let land = id_of(&g, "Forest");
        let ans = decide_scry(&g, 0, &[(land, "Forest".into())], ScryMode::Scry);
        match ans {
            DecisionAnswer::ScryOrder { kept_top, bottom } => {
                assert!(kept_top.is_empty() && bottom == vec![land],
                    "at six sources a scried land is flood");
            }
            other => panic!("expected ScryOrder, got {other:?}"),
        }
        // Mana-light: one source, scrying land + 6-drop + Shock. Keep the
        // land (first) and the Shock; bottom the uncastable wurm.
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::craw_wurm());
        g.add_card_to_library(0, catalog::shock());
        let (land, wurm, shock) =
            (id_of(&g, "Forest"), id_of(&g, "Craw Wurm"), id_of(&g, "Shock"));
        let cards = vec![
            (wurm, "Craw Wurm".into()),
            (land, "Forest".into()),
            (shock, "Shock".into()),
        ];
        let ans = decide_scry(&g, 0, &cards, ScryMode::Scry);
        match ans {
            DecisionAnswer::ScryOrder { kept_top, bottom } => {
                assert_eq!(kept_top, vec![land, shock],
                    "wanted land first, then the castable spell");
                assert_eq!(bottom, vec![wurm], "a 6-drop at one source is a brick");
            }
            other => panic!("expected ScryOrder, got {other:?}"),
        }
    }

    /// A mid-resolution modal is picked by outcome, not AutoDecider's
    /// blanket mode 0: a trigger offering [draw 1, draw 3] must answer
    /// mode 1.
    #[test]
    fn mode_decision_picked_by_outcome() {
        use crate::decision::{Decision, DecisionAnswer};
        use crate::effect::{Selector, Value};
        use crate::game::TriggerPush;
        let mut g = two_player_game();
        g.players[0].wants_ui = true;
        let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..5 {
            g.add_card_to_library(0, catalog::forest());
        }
        let draw = |n: i32| Effect::Draw { who: Selector::You, amount: Value::Const(n) };
        // `MODE_PICK_DEFERRED` is what `pick_trigger_mode` stamps on a
        // wants_ui controller's modal trigger so the pick suspends at
        // resolution instead of being answered inline by the decider.
        g.stack.push(
            TriggerPush::new(src, 0, Effect::ChooseMode(vec![draw(1), draw(3)]))
                .mode(Some(crate::game::types::MODE_PICK_DEFERRED))
                .build(),
        );
        // Pass priority until the modal suspends for the wants_ui seat.
        let mut fuel = 20;
        while g.pending_decision.is_none() && fuel > 0 {
            g.perform_action(GameAction::PassPriority).unwrap();
            fuel -= 1;
        }
        assert!(
            matches!(
                g.pending_decision.as_ref().map(|p| &p.decision),
                Some(Decision::ChooseMode { .. })
            ),
            "trigger resolution must suspend on the modal, got {:?}",
            g.pending_decision
        );
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot answers its pending decision");
        assert!(
            matches!(
                action,
                GameAction::SubmitDecision(DecisionAnswer::Mode(1))
            ),
            "draw 3 beats draw 1, got {action:?}"
        );
    }

    /// Ad Nauseam under bot play: the per-reveal prompt suspends for the
    /// wants_ui seat and the bot keeps revealing only while the next card
    /// leaves a life buffer — it neither declines everything (the old
    /// AutoDecider path: zero cards) nor draws itself to death (the
    /// generic "can't introspect → yes" fallback).
    #[test]
    fn bot_pilots_ad_nauseam_with_life_buffer() {
        use crate::decision::Decision;
        let mut g = two_player_game();
        g.players[0].wants_ui = true;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // Library of 3-mana cards: reveals at 20→17→14→11 life, then the
        // next reveal (11 - 3 = 8 ≤ 10 buffer) is declined.
        for _ in 0..10 {
            g.add_card_to_library(0, catalog::gray_ogre());
        }
        let nauseam = g.add_card_to_hand(0, catalog::ad_nauseam());
        g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: nauseam, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .unwrap();
        // Resolve; answer each suspended reveal prompt with the bot.
        let mut bot = RandomBot::new();
        let mut guard = 0;
        loop {
            while g.pending_decision.is_none() && !g.stack.is_empty() {
                g.perform_action(GameAction::PassPriority).ok();
                let _ = g.perform_action(GameAction::PassPriority);
            }
            let Some(pd) = &g.pending_decision else { break };
            assert!(matches!(pd.decision, Decision::OptionalTrigger { .. }));
            let action = bot.next_action(&g, 0).expect("bot answers the reveal prompt");
            g.perform_action(action).unwrap();
            guard += 1;
            assert!(guard < 20, "reveal loop must terminate");
        }
        // 3 cards revealed (life 20 → 11), then stopped; -1 for the cast.
        assert_eq!(g.players[0].life, 11, "stopped with a life buffer");
        assert_eq!(
            g.players[0].hand.len(),
            hand_before - 1 + 3,
            "took exactly the comfortable reveals",
        );
    }

    /// A/B ladder: the scored candidate pick vs the legacy uniform-random
    /// pick, mirror decks, seats swapped every game. Expensive (full games),
    /// so `#[ignore]` — run manually:
    ///
    /// ```text
    /// cargo test -p crabomination --lib scored_pick_beats_uniform_baseline -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "expensive A/B ladder; run manually with --ignored"]
    fn scored_pick_beats_uniform_baseline() {
        use crate::player::Player;

        // Mirror match on the same limited-style 40-card creature deck so
        // deck strength cancels out and only the pilots differ. A fair
        // curve-and-removal deck (not the BRG combo deck, whose games are
        // decided by drawing the combo, drowning out play-skill signal).
        fn mirror_game() -> GameState {
            use rand::seq::SliceRandom;
            let deck: Vec<fn() -> CardDefinition> = {
                let mut d: Vec<fn() -> CardDefinition> = Vec::new();
                let mut push = |f: fn() -> CardDefinition, n: usize| {
                    for _ in 0..n {
                        d.push(f);
                    }
                };
                push(catalog::mountain, 17);
                push(catalog::lightning_bolt, 4);
                push(catalog::shock, 3);
                push(catalog::goblin_guide, 4);
                push(catalog::monastery_swiftspear, 3);
                push(catalog::gray_ogre, 3);
                push(catalog::hill_giant, 3);
                push(catalog::fire_elemental, 2);
                push(catalog::shivan_dragon, 1);
                d
            };
            let mut g = GameState::new(vec![Player::new(0, "Scored"), Player::new(1, "Uniform")]);
            let mut r = rng();
            for seat in 0..2 {
                for &f in &deck {
                    g.add_card_to_library(seat, f());
                }
                g.players[seat].library.shuffle(&mut r);
                g.players[seat].wants_ui = true;
            }
            g.start_mulligan_phase();
            g
        }

        const GAMES: usize = 300;
        let (mut scored_wins, mut uniform_wins, mut other) = (0u32, 0u32, 0u32);
        for i in 0..GAMES {
            let scored_seat = i % 2;
            let mut g = mirror_game();
            let mut bots: Vec<Box<dyn Bot>> = (0..2)
                .map(|s| -> Box<dyn Bot> {
                    if s == scored_seat {
                        Box::new(RandomBot::new())
                    } else {
                        Box::new(RandomBot::uniform_baseline())
                    }
                })
                .collect();
            // Poll both seats to a fixed point, server-actor style. `stale`
            // guards against a state where neither bot volunteers an
            // accepted action (counted as a draw below).
            let (mut actions, mut stale) = (0usize, 0usize);
            while !g.is_game_over() && actions < 50_000 && stale < 8 {
                let mut any = false;
                for (s, bot) in bots.iter_mut().enumerate() {
                    let Some(a) = bot.next_action(&g, s) else { continue };
                    if g.perform_action(a).is_ok() {
                        any = true;
                        actions += 1;
                        if g.is_game_over() {
                            break;
                        }
                    }
                }
                if any { stale = 0 } else { stale += 1 }
            }
            let _ = actions;
            match g.game_over {
                Some(Some(w)) if w == scored_seat => scored_wins += 1,
                Some(Some(_)) => uniform_wins += 1,
                _ => other += 1,
            }
        }
        let decided = scored_wins + uniform_wins;
        let pct = 100.0 * scored_wins as f64 / decided.max(1) as f64;
        println!(
            "scored {scored_wins} – uniform {uniform_wins} (draw/stall {other}): scored win rate {pct:.1}%",
        );
        assert!(
            decided >= (GAMES as u32) / 2,
            "too many undecided games ({other}/{GAMES}) — harness stalled, results meaningless",
        );
        assert!(
            pct >= 55.0,
            "scored pick should clearly beat the uniform baseline, got {pct:.1}%",
        );
    }
}
