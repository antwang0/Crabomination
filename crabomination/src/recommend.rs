//! Sealed-deck building and bot-vs-bot match simulation.
//!
//! (The simulation-ranked `Session` recommender that used to sit on top was
//! retired 2026-09-29 with the `recommend_pool` binary.)
//!
//! * Building: enumerate candidate builds across the colour lattice (pairs,
//!   pair + splash, 3/4/5-colour) and rank them with a cheap static score,
//!   or sample a randomized build from the top shapes
//!   ([`build_random_deck_from`]) so a field looks like a room of humans,
//!   not clones. `selfplay` and the client's sealed opponents build here.
//! * Simulation: play two decks under chosen pilots
//!   ([`simulate_match_games_piloted`], the antithetic-pair loop behind
//!   `bot_ladder`), plus the shared stop diagnostics ([`stop_reason`],
//!   [`cap_diagnosis`]).

use crate::fxhash::HashMap;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, RngExt, SeedableRng};

use crate::cube::CardFactory;
use crate::draft::{COPY_CAP, colors_of_picks};
use crate::game::GameState;
use crate::mana::{Color, ColorSet};
use crate::player::Player;
use crate::server::{Bot, EvalWeights, MctsBot, MctsConfig, HeuristicBot};

/// Everything tunable about a sealed build: candidate enumeration and the
/// randomized builder. Engine callers use `SimConfig::default()` and set the
/// builder flags they opt into.
#[derive(Debug, Clone)]
pub struct SimConfig {
    /// Gauntlet build randomness; 0.0 = deterministic greedy builds.
    pub build_temperature: f64,
    /// Gauntlet spell-count range (inclusive).
    pub spell_count_range: (usize, usize),
    /// Gauntlet land-count range (inclusive).
    pub land_count_range: (u32, u32),
    /// Candidate builds: spells in the main deck.
    pub target_spells: usize,
    /// Candidate builds: total basic lands.
    pub total_lands: u32,
    /// Max cards splashed in a pair+splash candidate.
    pub splash_max_cards: usize,
    /// Minimum static score for a card to be splash-worthy.
    pub splash_min_score: i32,
    /// The repaired sealed builder: card-quality scoring
    /// ([`draft::card_quality`]), splashes restricted to single-pip cards,
    /// and basics split by squared pip demand. A flag, not a rewrite, so the
    /// old builder stays as the measurement control (the three defects were
    /// found together — a {3}{U}{U} bomb "splashed" off three Islands).
    pub builder_v2: bool,
    /// The quality- and curve-aware shape ranker
    /// ([`static_build_score_v3`]): shapes ranked by
    /// [`draft::score_card_quality`] plus a curve penalty, instead of the
    /// legacy body-blind scorer `builder_v2` still uses for shape choice.
    ///
    /// Default off: it feeds the gauntlet's shape softmax, so flipping it
    /// changes every generated field, ladder gate decks included. Stays a
    /// flag until the `--gate-builder-v3` race adopts it; the client's
    /// sealed opponent opts in explicitly.
    pub builder_v3: bool,
    /// Score main-deck picks with [`draft::aggro_curve_delta`] on top of the
    /// quality scorer: cheap proactive cards up, six-plus-drops down.
    ///
    /// Without it the builder has no working curve: `base_score`'s bucket is
    /// small against quality and ranks a one-drop below a five-drop,
    /// `card_quality` has no mana-value term, and `curve_penalty` only ranks
    /// shapes under [`builder_v3`](Self::builder_v3). Decks top out heavy
    /// ("the bot doesn't build aggressive enough"). Default off for the same
    /// reason as `builder_v3`: it changes every generated field.
    pub curve_aggro: bool,
    /// Score cards by their measured value ([`crate::card_ratings`]) instead
    /// of the static `base_score + quality`, in both the picks and the v3
    /// shape ranker. Needs `builder_v2`. Off by default: the ratings cover
    /// Reality Fracture only, and the training field and every gate deck
    /// are built without them.
    pub rated_cards: bool,
    /// Build the spells to the deck shape [`crate::card_ratings::shape_penalty`]
    /// measured flat — creatures, mana value ≤ 2 and ≥ 5 inside their ranges
    /// ([`ShapeSlots`]) — instead of strictly by score. The rated builder
    /// turns it on for half its candidates and lets the judge choose.
    pub shape_slots: bool,
    /// Restrict the lattice to one colour identity: a shape is enumerated
    /// only when its main colours and splash sit inside this set (the
    /// client's `--opponent-colors`). Empty (default) = unrestricted.
    ///
    /// Fewer than two colours leaves no shape (the lattice never builds
    /// mono-colour), so callers taking this from a user reject it up front.
    pub color_restriction: ColorSet,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            builder_v2: true,
            builder_v3: false,
            curve_aggro: false,
            rated_cards: false,
            shape_slots: false,
            build_temperature: 1.0,
            spell_count_range: (22, 24),
            land_count_range: (16, 18),
            target_spells: 23,
            total_lands: 17,
            splash_max_cards: 3,
            splash_min_score: 12,
            color_restriction: ColorSet::empty(),
        }
    }
}

/// One enumerated build from the user's pool.
#[derive(Clone)]
pub struct CandidateBuild {
    /// Main colors (2–5).
    pub colors: Vec<Color>,
    /// Splash color(s) — non-empty only for pair+splash shapes.
    pub splash: Vec<Color>,
    /// Chosen spells (no lands).
    pub main: Vec<CardFactory>,
    /// On-color nonbasic lands from the pool (school duals etc.),
    /// occupying land slots ahead of basics.
    pub duals: Vec<CardFactory>,
    /// Basic-land split for the remaining land slots.
    pub basics: HashMap<Color, u32>,
    /// Pool cards not in the main deck.
    pub leftovers: Vec<CardFactory>,
    /// Cheap heuristic rank (curve / color-fit / consistency).
    pub static_score: i32,
    /// Display label, e.g. `"R/W"`, `"R/W + b"`, `"W/U/G"`.
    pub label: String,
}

impl CandidateBuild {
    /// The full playable deck: main spells, pool duals, then expanded
    /// basics (canonical WUBRG order, so deck lists are seed-stable).
    pub fn deck(&self) -> Vec<CardFactory> {
        let mut d = self.main.clone();
        d.extend(self.duals.iter().copied());
        for c in Color::ALL {
            for _ in 0..self.basics.get(&c).copied().unwrap_or(0) {
                d.push(basic_for(c));
            }
        }
        d
    }
}

/// A deck in the opposing field, plus enough metadata to report
/// per-archetype win-rate splits.
pub struct GauntletDeck {
    pub cards: Vec<CardFactory>,
    pub label: String,
}

// ─────────────────────────────── candidates ───────────────────────────────

fn basic_for(c: Color) -> CardFactory {
    match c {
        Color::White => crate::catalog::plains,
        Color::Blue => crate::catalog::island,
        Color::Black => crate::catalog::swamp,
        Color::Red => crate::catalog::mountain,
        Color::Green => crate::catalog::forest,
    }
}

fn color_letter(c: Color) -> char {
    match c {
        Color::White => 'W',
        Color::Blue => 'U',
        Color::Black => 'B',
        Color::Red => 'R',
        Color::Green => 'G',
    }
}

/// Up to five colours, inline — a shape's colours and splash without the
/// per-shape `Vec` allocations (PERF (-63)).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ColorList {
    buf: [Color; 5],
    len: u8,
}

impl ColorList {
    const EMPTY: Self = Self { buf: Color::ALL, len: 0 };

    /// The mask's colours in WUBRG order. Not free to change: the lattice
    /// order is a tie-break in a stable sort.
    fn from_mask(mask: u8) -> Self {
        let mut out = Self::EMPTY;
        for (i, c) in Color::ALL.into_iter().enumerate() {
            if mask & (1 << i) != 0 {
                out.buf[out.len as usize] = c;
                out.len += 1;
            }
        }
        out
    }

    fn one(c: Color) -> Self {
        Self { buf: [c; 5], len: 1 }
    }

    /// A caller's `&[Color]` (always a WUBRG subset, so `take` is a bound,
    /// not a truncation).
    fn of(colors: &[Color]) -> Self {
        debug_assert!(colors.len() <= 5, "a colour list is a subset of WUBRG");
        let mut out = Self::EMPTY;
        for &c in colors.iter().take(5) {
            out.buf[out.len as usize] = c;
            out.len += 1;
        }
        out
    }

    fn as_slice(&self) -> &[Color] {
        &self.buf[..self.len as usize]
    }

    /// The list as a bitmask, for `SimConfig::color_restriction`'s subset test.
    fn set(&self) -> ColorSet {
        self.as_slice().iter().collect()
    }
}

/// The shape-independent part of each pool card: its brief and its scorer
/// output against the pool's own pip totals. The scorer's colour argument is
/// a property of the pool, so the base score is the same for every shape;
/// only the pick jitter and fixing bonus vary. Computed once instead of per
/// card per shape.
pub struct PoolScores<'a> {
    picks: &'a [CardFactory],
    /// `(brief, base score)` per entry of `picks`, in order.
    cards: Vec<(&'static crate::cube::CardBrief, i32)>,
    /// Which scorer produced `cards`' scores (`SimConfig::builder_v2`).
    /// Checked by a `debug_assert` in [`build_shape`] so mixing scorers
    /// between scoring and splash ranking fails the suite.
    quality: bool,
    /// Whether the scores carry [`draft::aggro_curve_delta`]
    /// (`SimConfig::curve_aggro`). Same `debug_assert`.
    aggro: bool,
    /// Whether the scores are the measured ones (`SimConfig::rated_cards`).
    /// Same `debug_assert`.
    rated: bool,
    /// Descending-score orders over `picks`, one per fixing bonus a shape's
    /// colour count can imply, built on first ask. See [`Self::order`].
    orders: [std::cell::OnceCell<Vec<u32>>; 3],
    /// A dense id per distinct card in `picks`, and how many there are, so
    /// the copy-cap counter is a per-shape array instead of a `HashMap`.
    dup_group: Vec<u32>,
    groups: usize,
}

impl<'a> PoolScores<'a> {
    pub fn new(picks: &'a [CardFactory], quality: bool, aggro: bool, rated: bool) -> Self {
        let pick_colors = colors_of_picks(picks);
        let cards = picks
            .iter()
            .map(|&f| {
                let brief = crate::cube::card_brief(f);
                let base = match (quality, aggro) {
                    (true, aggro) if rated => {
                        crate::draft::score_brief_rated(brief, &pick_colors)
                            + if aggro { crate::draft::aggro_curve_delta(brief) } else { 0 }
                    }
                    (true, true) => crate::draft::score_brief_aggro(brief, &pick_colors),
                    (true, false) => crate::draft::score_brief_quality(brief, &pick_colors),
                    // The legacy control scorer takes no curve term: it is
                    // kept intact precisely so it stays comparable.
                    (false, _) => crate::draft::score_brief_with_colors(brief, &pick_colors),
                };
                (brief, base)
            })
            .collect();
        let mut ids: HashMap<usize, u32> =
            HashMap::with_capacity_and_hasher(picks.len(), Default::default());
        let dup_group: Vec<u32> = picks
            .iter()
            .map(|&f| {
                let next = ids.len() as u32;
                *ids.entry(f as usize).or_insert(next)
            })
            .collect();
        let groups = ids.len();
        Self {
            picks,
            cards,
            quality,
            aggro,
            rated,
            orders: [const { std::cell::OnceCell::new() }; 3],
            dup_group,
            groups,
        }
    }

    /// The fixing bonus a build in `n` colours gives a fixing card — the
    /// only shape-dependent term in a pick's score once the jitter is zero.
    fn fixing_bonus(n: usize) -> i32 {
        match n {
            0..=1 => 0,
            2 => 1,
            _ => 3,
        }
    }

    /// `picks` indices, descending by `base + bonus * is_fixing`, stably.
    /// Three orders cover every shape, since the bonus depends only on the
    /// colour count; built on first ask.
    fn order(&self, bonus: i32) -> &[u32] {
        let slot = match bonus {
            0 => 0,
            1 => 1,
            _ => 2,
        };
        self.orders[slot].get_or_init(|| {
            let mut idx: Vec<u32> = (0..self.cards.len() as u32).collect();
            idx.sort_by_key(|&i| {
                let (brief, base) = self.cards[i as usize];
                let fix =
                    if bonus > 0 && !brief.is_land && brief.is_fixing { bonus } else { 0 };
                std::cmp::Reverse(base + fix)
            });
            idx
        })
    }

    /// The pool these scores were built from.
    pub fn picks(&self) -> &'a [CardFactory] {
        self.picks
    }
}

/// Words in the sort-free path's `allowed` bitmask, and the pool size it
/// covers. A sealed pool is ~90 cards and a whole cube ~360; anything wider
/// falls back to the per-shape sort rather than growing a stack array.
const ALLOW_WORDS: usize = 16;
const ALLOW_BITS: usize = ALLOW_WORDS * 64;

/// How much of a shape a caller needs. `Ranked` stops at the spells and the
/// score; `Full` also builds the leftovers pile and the pool-land index, which
/// exist only to feed [`assemble_lands`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Detail {
    Ranked,
    Full,
}

struct MainDeck {
    main: Vec<CardFactory>,
    /// Copies of each distinct pool card (`PoolScores::dup_group` ids) in
    /// `main` — the main deck as a multiset, which `lattice` dedups on
    /// instead of sorting a `Vec<CardFactory>` per shape (PERF (-63)).
    counts: Vec<u8>,
    leftovers: Vec<CardFactory>,
    /// `(index in leftovers, colours it taps for)` per pool land, ascending.
    /// Lands never take spell slots, so all of them reach `off` in pool order;
    /// this saves `assemble_lands` a `card_brief` per leftover.
    lands: Vec<(u32, crate::mana::ColorSet)>,
}

/// [`suggest_main_deck_in_colors_with`] keeping the land index the shape
/// builder's land assignment wants. See [`MainDeck`].
#[allow(clippy::too_many_arguments)]
fn suggest_main_deck_shape<R: Rng>(
    scores: &PoolScores<'_>,
    colors: &[Color],
    splash: &[CardFactory],
    target_spells: usize,
    noise: i32,
    rng: &mut R,
    detail: Detail,
    shape_slots: bool,
) -> MainDeck {
    let picks = scores.picks;
    // `leftovers`/`lands` only feed `assemble_lands`; a ranking-only caller
    // skips them (PERF (-63)).
    let full = detail == Detail::Full;
    // The shape's colours as a bitmask: "castable" is one subset test per
    // card. A colourless card's empty `pip_colors` is a subset of everything.
    let want = colors.iter().fold(crate::mana::ColorSet::empty(), |mut s, &c| {
        s.insert(c);
        s
    });
    let allowed = |f: CardFactory, brief: &crate::cube::CardBrief| -> bool {
        // Lands never occupy spell slots — `assemble_lands` owns the land
        // base. (High jitter used to promote off-colour duals into the main.)
        if brief.is_land {
            return false;
        }
        // The rated builder reads `{G/U}` as G or U; the others keep the
        // both-colours reading they were measured under.
        let fits = if scores.rated {
            brief.fits(want)
        } else {
            brief.pip_colors.is_subset_of(want)
        };
        fits || splash.iter().any(|s| *s as usize == f as usize)
    };
    // Fixing-aware bonus: mana rocks / land fetchers earn their keep in
    // proportion to how many colors the build stretches across.
    let fixing_bonus = PoolScores::fixing_bonus(colors.len());
    // With no jitter, walk one of `PoolScores`' cached orders and skip what
    // `allowed` (recorded in a bitmask) rejects — a stable sort commutes with
    // a filter, so this matches sorting the filtered list. Jittered builds
    // still sort.
    let ordered = (noise == 0 && picks.len() <= ALLOW_BITS).then(|| scores.order(fixing_bonus));
    // Pre-sized (`scored`/`off` partition `picks`; growth by doubling was
    // the build's top `RawVec::grow_one` caller). Pick indices, not
    // factories, so the copy cap can read the dense card id.
    let mut scored: Vec<(u32, i32)> =
        Vec::with_capacity(if ordered.is_some() { 0 } else { picks.len() });
    let mut off: Vec<CardFactory> = Vec::with_capacity(if full { picks.len() } else { 0 });
    // A sealed pool is about a fifth lands.
    let mut lands: Vec<(u32, crate::mana::ColorSet)> =
        Vec::with_capacity(if full { picks.len() / 4 } else { 0 });
    let mut allow = [0u64; ALLOW_WORDS];
    for (i, (&f, &(brief, base))) in picks.iter().zip(&scores.cards).enumerate() {
        if !allowed(f, brief) {
            if full {
                if brief.is_land && !(OLD_DUALS.get() && taps_of_colors(brief.def)) {
                    lands.push((off.len() as u32, brief.produces));
                }
                off.push(f);
            }
            continue;
        }
        if ordered.is_some() {
            allow[i >> 6] |= 1u64 << (i & 63);
            continue;
        }
        let jitter = if noise > 0 { rng.random_range(-noise..=noise) } else { 0 };
        // Lands never take spell slots — they're assigned by
        // `assemble_lands`; the fixing bonus is for rocks/fetchers.
        let fix = if fixing_bonus > 0 && !brief.is_land && brief.is_fixing {
            fixing_bonus
        } else {
            0
        };
        scored.push((i as u32, base + jitter + fix));
    }
    /// One pick down the copy-cap funnel: into `main` while it has a slot and
    /// the cap (counted per `PoolScores::dup_group` id) allows, else into the
    /// leftovers. A pick the curve or shape slots refuse goes to `deferred`
    /// for the unconstrained second pass, so a pool that cannot fill the
    /// reserve still makes a full deck.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn take(
        at: u32,
        scores: &PoolScores<'_>,
        counts: &mut [u8],
        target_spells: usize,
        main: &mut Vec<CardFactory>,
        leftovers: &mut Vec<CardFactory>,
        full: bool,
        curve: Option<&mut CurveSlots>,
        shape: Option<&mut ShapeSlots>,
        deferred: &mut Vec<u32>,
    ) {
        let f = scores.picks[at as usize];
        let count = counts[scores.dup_group[at as usize] as usize];
        // The hard stops first: claiming a curve slot for a pick the copy
        // cap is about to reject would leak the slot.
        if main.len() >= target_spells || u32::from(count) >= COPY_CAP {
            if full {
                leftovers.push(f);
            }
            return;
        }
        // The shape before the curve: the rated builder that sets it never
        // runs `curve_aggro`, and a refused shape must not leak a curve slot.
        if let Some(sh) = shape
            && !sh.claim(scores.cards[at as usize].0)
        {
            deferred.push(at);
            return;
        }
        if let Some(c) = curve
            && !c.claim(scores.cards[at as usize].0.cmc)
        {
            deferred.push(at);
            return;
        }
        counts[scores.dup_group[at as usize] as usize] += 1;
        main.push(f);
    }
    let mut counts: Vec<u8> = vec![0; scores.groups];
    let mut main = Vec::with_capacity(target_spells);
    let mut leftovers = Vec::with_capacity(if full { picks.len() } else { 0 });
    let mut curve = scores.aggro.then(|| CurveSlots::new(target_spells));
    let mut shape = shape_slots.then(|| ShapeSlots::new(target_spells));
    let mut deferred: Vec<u32> = Vec::new();
    if let Some(order) = ordered {
        for &i in order {
            if allow[i as usize >> 6] & (1u64 << (i & 63)) != 0 {
                take(i, scores, &mut counts, target_spells, &mut main, &mut leftovers, full,
                     curve.as_mut(), shape.as_mut(), &mut deferred);
            }
        }
    } else {
        scored.sort_by_key(|(_, s)| std::cmp::Reverse(*s));
        for (i, _) in scored {
            take(i, scores, &mut counts, target_spells, &mut main, &mut leftovers, full,
                 curve.as_mut(), shape.as_mut(), &mut deferred);
        }
    }
    // Second pass, unconstrained: the curve is a target, not a quota — a
    // short deck is worse than an off-curve one. Refused picks compete for
    // what is left, still in score order.
    for i in std::mem::take(&mut deferred) {
        take(i, scores, &mut counts, target_spells, &mut main, &mut leftovers, full,
             None, None, &mut Vec::new());
    }
    if full {
        // `off` is appended whole, so a land's index in `leftovers` is its
        // index in `off` shifted by whatever the scored pile left behind.
        let off_start = leftovers.len() as u32;
        for (at, _) in &mut lands {
            *at += off_start;
        }
        leftovers.extend(off);
    }
    MainDeck { main, counts, leftovers, lands }
}

/// True when a card helps cast a multicolor deck's spells: it taps for
/// mana (Page, Loose Leaf) or its effect tree searches for a land
/// (Environmental Scientist's ETB fetch). The pip scorer is otherwise
/// fixing-blind, so these never survived the cut on their own merits.
pub(crate) fn is_fixing_card(def: &crate::card::CardDefinition) -> bool {
    use crate::effect::Effect;
    fn req_mentions_land(r: &crate::card::SelectionRequirement) -> bool {
        use crate::card::SelectionRequirement as R;
        match r {
            R::Land | R::IsBasicLand => true,
            R::And(a, b) | R::Or(a, b) => req_mentions_land(a) || req_mentions_land(b),
            _ => false,
        }
    }
    fn searches_land(e: &Effect) -> bool {
        match e {
            Effect::Search { filter, .. } => req_mentions_land(filter),
            Effect::Seq(v) => v.iter().any(searches_land),
            Effect::MayDo { body, .. } => searches_land(body),
            Effect::If { then, else_, .. } => searches_land(then) || searches_land(else_),
            _ => false,
        }
    }
    let taps_for_mana = def
        .activated_abilities
        .iter()
        .any(|a| matches!(a.effect, Effect::AddMana { .. }));
    taps_for_mana
        || searches_land(&def.effect)
        || def.triggered_abilities.iter().any(|t| searches_land(&t.effect))
}

/// The five colors' pip totals, heaviest first, read off the
/// `colors_of_picks` totals the rankers already hold.
fn sorted_pip_totals(totals: &crate::draft::ColorCounts) -> [i32; 5] {
    let mut pips = [0i32; 5];
    for (i, (_, n)) in totals.iter().enumerate() {
        pips[i] = n as i32;
    }
    pips.sort_unstable_by_key(|n| std::cmp::Reverse(*n));
    pips
}

/// Cheap build rank: summed card scores, minus a shortfall penalty when
/// the pool can't fill the target spell count in these colors, minus a
/// consistency penalty on colored pips beyond the two heaviest colors
/// (what a 3+-color build pays in mana reliability).
pub(crate) fn static_build_score(main: &[CardFactory], target_spells: usize) -> i32 {
    let main_colors = colors_of_picks(main);
    // One `support()` for the whole deck rather than per card (PERF (-63)).
    let seat = main_colors.support();
    let mut score: i32 = main
        .iter()
        .map(|&f| crate::draft::score_brief_in_colors(crate::cube::card_brief(f), seat))
        .sum();
    let shortfall = target_spells.saturating_sub(main.len()) as i32;
    score -= shortfall * 8;
    let pips = sorted_pip_totals(&main_colors);
    // Basics-only fixing makes each color beyond the second expensive:
    // a flat cost per extra color plus a steep per-pip cost. Without the
    // flat term, an 82-card pool's marginal card-quality gains rank the
    // 5-color pile above every honest pair.
    let extra_colors = pips.iter().skip(2).filter(|n| **n > 0).count() as i32;
    score -= extra_colors * 12;
    score -= pips.iter().skip(2).sum::<i32>() * 6;
    score
}

/// [`static_build_score`] with the shape ranker's two v2 defects fixed:
/// per-card scores from [`draft::score_card_quality`] (v2 ranked shapes
/// body-blind, so worse colours could outrank the pair holding the bombs),
/// and a degenerate curve costs points ([`curve_penalty`]). Behind
/// `SimConfig::builder_v3` so v2 stays the control; raced by
/// `selfplay_train --gate-builder-v3`.
pub(crate) fn static_build_score_v3(main: &[CardFactory], target_spells: usize, rated: bool) -> i32 {
    let main_colors = if rated {
        hybrid_aware_colors(main)
    } else {
        colors_of_picks(main)
    };
    let mut score: i32 = main
        .iter()
        .map(|&f| {
            if rated {
                crate::draft::score_brief_rated(crate::cube::card_brief(f), &main_colors)
            } else {
                crate::draft::score_card_quality(f, &main_colors)
            }
        })
        .sum();
    let shortfall = target_spells.saturating_sub(main.len()) as i32;
    score -= shortfall * 8;
    let pips = sorted_pip_totals(&main_colors);
    let extra_colors = pips.iter().skip(2).filter(|n| **n > 0).count() as i32;
    score -= extra_colors * 12;
    score -= pips.iter().skip(2).sum::<i32>() * 6;
    score - curve_penalty(main)
}

/// [`colors_of_picks`] with each two-colour hybrid symbol counted once, for
/// whichever of its colours the deck's single-colour pips lean on more: a
/// W/G deck playing `{1}{G/U}` has no blue pip to pay for.
fn hybrid_aware_colors(main: &[CardFactory]) -> crate::draft::ColorCounts {
    use crate::mana::ManaSymbol as S;
    let mut totals = crate::draft::ColorCounts::default();
    for &f in main {
        for sym in &crate::cube::card_def(f).cost.symbols {
            if let S::Colored(c) | S::Phyrexian(c) = sym {
                totals.add(*c, 1);
            }
        }
    }
    let hard = totals;
    for &f in main {
        for sym in &crate::cube::card_def(f).cost.symbols {
            if let S::Hybrid(a, b) | S::PhyrexianHybrid(a, b) = sym {
                totals.add(if hard.get(*b) > hard.get(*a) { *b } else { *a }, 1);
            }
        }
    }
    totals
}

/// The curve a build must hit, in slots, under `SimConfig::curve_aggro`:
/// [`curve_penalty`]'s numbers (≥ 5 spells at mana value ≤ 2, ≤ 6 at ≥ 5)
/// scaled to the spell count. `curve_penalty` only scores finished shapes;
/// this constrains assembly, which otherwise fills strictly by score.
///
/// `early` is reserved for mana value ≤ 2, `top` caps mana value ≥ 5, and
/// everything else competes for `open` in score order. `early + open ==
/// target_spells`, so the deck is reshaped, not shrunk.
struct CurveSlots {
    /// Reserved for mana value ≤ 2.
    early: usize,
    /// Remaining allowance for mana value ≥ 5.
    top: usize,
    /// Claimable by anything.
    open: usize,
}

impl CurveSlots {
    fn new(target_spells: usize) -> Self {
        // `curve_penalty`'s own five-and-six, at its own 23-spell scale.
        let early = target_spells * 5 / 23;
        let top = target_spells * 6 / 23;
        Self { early, top, open: target_spells - early }
    }

    /// Claim a slot for a spell of this mana value, or refuse it.
    fn claim(&mut self, cmc: u32) -> bool {
        if cmc <= 2 {
            // Reserved first, so the cheap slots survive a run of better
            // expensive cards further down the order.
            if self.early > 0 {
                self.early -= 1;
            } else if self.open > 0 {
                self.open -= 1;
            } else {
                return false;
            }
            return true;
        }
        if self.open == 0 || (cmc >= 5 && self.top == 0) {
            return false;
        }
        if cmc >= 5 {
            self.top -= 1;
        }
        self.open -= 1;
        true
    }
}

/// The deck shape held under `SimConfig::shape_slots`: the ranges where
/// [`crate::card_ratings::shape_penalty`]'s residual was flat on FRA (16–19
/// creatures, 8–11 at mana value ≤ 2, 2–5 at ≥ 5), scaled from 23 spells.
/// A pick is refused if it would pass a maximum or make a minimum
/// unreachable; refused picks go to the unconstrained second pass, as with
/// [`CurveSlots`].
struct ShapeSlots {
    left: usize,
    have: [usize; 3],
    min: [usize; 3],
    max: [usize; 3],
}

impl ShapeSlots {
    fn new(target_spells: usize) -> Self {
        let at = |n: usize| ((n * target_spells) as f32 / 23.0).round() as usize;
        Self { left: target_spells, have: [0; 3], min: [at(16), at(8), at(2)], max: [at(19), at(11), at(5)] }
    }

    /// Take a slot for a spell with these traits, or refuse it.
    fn claim(&mut self, brief: &crate::cube::CardBrief) -> bool {
        if self.left == 0 {
            return false;
        }
        let traits = [brief.is_creature, brief.cmc <= 2, brief.cmc >= 5];
        let mut next = self.have;
        for (n, &t) in next.iter_mut().zip(&traits) {
            *n += usize::from(t);
        }
        let left = self.left - 1;
        if (0..3).any(|k| next[k] > self.max[k] || self.min[k].saturating_sub(next[k]) > left) {
            return false;
        }
        self.have = next;
        self.left = left;
        true
    }
}

/// Soft curve penalty: 4 per early spell (mana value ≤ 2) short of five,
/// 3 per five-plus drop beyond six. Deliberately below a colour-fit pip (6)
/// and a slot shortfall (8): a curve hole steers a close shape choice but
/// doesn't overrule card quality.
pub(crate) fn curve_penalty(main: &[CardFactory]) -> i32 {
    let early = main.iter().filter(|&&f| crate::cube::card_brief(f).cmc <= 2).count() as i32;
    let top = main.iter().filter(|&&f| crate::cube::card_brief(f).cmc >= 5).count() as i32;
    (5 - early).max(0) * 4 + (top - 6).max(0) * 3
}

/// Top splash-worthy cards for splashing `third` next to `pair`: colours
/// include `third`, fit within pair+third, score ≥ the bar. Gold cards
/// straddling pair and splash qualify ("U/B splashing a B/G bomb" is a real
/// build). Reuses [`PoolScores`]' per-card scores.
fn splash_cards(scores: &PoolScores<'_>, pair: &[Color], third: Color, cfg: &SimConfig) -> Vec<CardFactory> {
    // Same bitmask test as `suggest_main_deck_shape`: a whole-pool walk per
    // splash shape.
    let want = pair.iter().fold(crate::mana::ColorSet::single(third), |mut s, &c| {
        s.insert(c);
        s
    });
    let mut hits: Vec<(CardFactory, i32)> = scores
        .picks
        .iter()
        .zip(&scores.cards)
        .filter_map(|(&f, &(brief, base))| {
            let pips = &brief.pips;
            if !(brief.pip_colors.contains(third) && brief.pip_colors.is_subset_of(want)) {
                return None;
            }
            // A splash supports only one coloured pip (the builder once
            // splashed {3}{U}{U} Emeritus of Ideation off three Islands).
            if cfg.builder_v2 && pips.get(third) > 1 {
                return None;
            }
            (base >= cfg.splash_min_score).then_some((f, base))
        })
        .collect();
    hits.sort_by_key(|(_, s)| std::cmp::Reverse(*s));
    hits.into_iter().take(cfg.splash_max_cards).map(|(f, _)| f).collect()
}

/// Proportional basic split over any number of colours (≥ 1 source per
/// colour with pips), generalizing `draft::suggest_basic_split`. Pips are
/// weighted by earliness (a two-drop's pip needs its source sooner than a
/// six-drop's), so expensive splashes lean on fewer sources.
fn basic_split(
    main: &[CardFactory],
    colors: &[Color],
    total: u32,
    squared_pips: bool,
) -> HashMap<Color, u32> {
    let weights: Vec<(Color, u32)> = colors
        .iter()
        .map(|&c| {
            let w = main
                .iter()
                .map(|&f| {
                    let brief = crate::cube::card_brief(f);
                    let pips = brief.pips.get(c);
                    // {U}{U} on curve needs ~1.5x the sources {U} does;
                    // squaring stands in for the hypergeometric answer.
                    let demand = if squared_pips { pips * pips } else { pips };
                    demand * 7u32.saturating_sub(brief.cmc).max(1)
                })
                .sum();
            (c, w)
        })
        .collect();
    let total_w: u32 = weights.iter().map(|(_, w)| w).sum();
    let mut out: HashMap<Color, u32> = HashMap::default();
    if total_w == 0 {
        // No colored pips (all-artifact main): split evenly.
        let per = total / colors.len().max(1) as u32;
        for (i, &c) in colors.iter().enumerate() {
            out.insert(c, if i == 0 { total - per * (colors.len() as u32 - 1) } else { per });
        }
        return out;
    }
    let mut allocated = 0u32;
    for &(c, w) in &weights {
        let share = ((w as f64 / total_w as f64) * total as f64).round() as u32;
        // Any color with pips keeps at least one source.
        let share = if w > 0 { share.max(1) } else { share };
        out.insert(c, share);
        allocated += share;
    }
    // Fix rounding drift against the heaviest color.
    let heaviest = weights.iter().max_by_key(|(_, w)| *w).map(|(c, _)| *c).unwrap();
    let v = out.get_mut(&heaviest).unwrap();
    if allocated < total {
        *v += total - allocated;
    } else {
        *v = v.saturating_sub(allocated - total);
    }
    out
}

/// Does the rated builder give a colour holding a double-pip spell at least
/// [`DOUBLE_PIP_SOURCES`] sources? [`basic_split`]'s earliness weighting
/// barely counts a {B}{B} six-drop (the 2026-10-29 spot check found five
/// Swamps under two Apex Witchstalkers). Off: the `fra_card_eval --ab
/// --trial floor` race measured FRA +0.15..0.19, SOS +0.03 — real but under
/// the +0.3 bar set beforehand.
pub const RATED_PIP_FLOOR: bool = false;

/// The source count [`RATED_PIP_FLOOR`] guarantees: about what a strong
/// limited player runs under a colour's {C}{C} cards in a 16–17 land deck.
pub const DOUBLE_PIP_SOURCES: u32 = 7;

thread_local! {
    static PIP_FLOOR: std::cell::Cell<bool> = const { std::cell::Cell::new(RATED_PIP_FLOOR) };
}

fn pip_floor_on() -> bool {
    PIP_FLOOR.get()
}

/// Run `f` with [`RATED_PIP_FLOOR`] set to `on` — the trial arm of
/// `fra_card_eval --ab --trial`.
pub fn with_pip_floor<R>(on: bool, f: impl FnOnce() -> R) -> R {
    let was = PIP_FLOOR.replace(on);
    let r = f();
    PIP_FLOOR.set(was);
    r
}

/// Move basics toward each main colour with a double-pip spell until it has
/// [`DOUBLE_PIP_SOURCES`] sources (basics plus duals), taken from the
/// colour with the most basics that stays above its own floor (that number
/// if it has a double pip, else one).
fn double_pip_floor(main: &[CardFactory], colors: &[Color], duals: &[CardFactory], basics: &mut HashMap<Color, u32>) {
    let double = |c: Color| main.iter().any(|&f| crate::cube::card_brief(f).pips.get(c) >= 2);
    let dual_sources = |c: Color| duals.iter().filter(|&&f| crate::cube::card_brief(f).produces.contains(c)).count() as u32;
    let sources = |b: &HashMap<Color, u32>, c: Color| b.get(&c).copied().unwrap_or(0) + dual_sources(c);
    let floor = |c: Color| if double(c) { DOUBLE_PIP_SOURCES } else { 1 };
    for &c in colors {
        if !double(c) {
            continue;
        }
        while sources(basics, c) < DOUBLE_PIP_SOURCES {
            let Some(&d) = colors
                .iter()
                .filter(|&&d| d != c && basics.get(&d).copied().unwrap_or(0) > 0 && sources(basics, d) > floor(d))
                .max_by_key(|&&d| basics.get(&d).copied().unwrap_or(0))
            else {
                break;
            };
            *basics.get_mut(&d).unwrap() -= 1;
            *basics.entry(c).or_insert(0) += 1;
        }
    }
}

thread_local! {
    static OLD_DUALS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Run `f` with the builder blind to `OfColors` duals, as it was before
/// [`land_produced_colors`] read them — `fra_card_eval --ab --old-duals`'s
/// control arm.
pub fn with_old_duals<R>(f: impl FnOnce() -> R) -> R {
    OLD_DUALS.set(true);
    let r = f();
    OLD_DUALS.set(false);
    r
}

fn taps_of_colors(def: &crate::card::CardDefinition) -> bool {
    def.activated_abilities.iter().any(|ab| {
        matches!(&ab.effect, crate::effect::Effect::AddMana { pool: crate::effect::ManaPayload::OfColors(..), .. })
    })
}

/// Colors a land taps for — the union of its `AddMana` ability payloads
/// (an any-color payload counts as all five).
pub(crate) fn land_produced_colors(def: &crate::card::CardDefinition) -> crate::mana::ColorSet {
    use crate::effect::{Effect, ManaPayload};
    let mut seen = crate::mana::ColorSet::empty();
    for ab in &def.activated_abilities {
        if let Effect::AddMana { pool, .. } = &ab.effect {
            match pool {
                // "{T}: Add {W} or {U}" is `OfColors` (the FRA Annexes and
                // Commons); missing it left seven of FRA's ten duals out of
                // every deck (spot check, 2026-10-29).
                ManaPayload::Colors(cs) | ManaPayload::OfColors(cs, _) => {
                    for c in cs {
                        seen.insert(*c);
                    }
                }
                ManaPayload::OfColor(c, _) => seen.insert(*c),
                ManaPayload::AnyOneColor(_) => seen = crate::mana::ColorSet::all(),
                _ => {}
            }
        }
    }
    seen
}

/// Split `total` land slots between on-colour pool duals (tapping for ≥ 2
/// of the build's colours and nothing else — anything else is a worse
/// basic here) and proportional basics. Used duals leave `leftovers`.
fn assemble_lands(
    leftovers: &mut Vec<CardFactory>,
    lands: &[(u32, crate::mana::ColorSet)],
    main: &[CardFactory],
    colors: &[Color],
    total: u32,
    squared_pips: bool,
) -> (Vec<CardFactory>, HashMap<Color, u32>) {
    let want = colors.iter().fold(crate::mana::ColorSet::empty(), |mut s, &c| {
        s.insert(c);
        s
    });
    let mut duals: Vec<CardFactory> = Vec::new();
    // Ascending, so removing them below is one pass with no search.
    let mut taken: Vec<u32> = Vec::new();
    for &(at, produced) in lands {
        if duals.len() >= total as usize {
            break;
        }
        let on_color = crate::mana::ColorSet(produced.0 & want.0).len();
        if on_color >= 2 && produced.is_subset_of(want) {
            duals.push(leftovers[at as usize]);
            taken.push(at);
        }
    }
    if let Some(&first) = taken.first() {
        let mut next = 1usize;
        let mut write = first as usize;
        for read in write + 1..leftovers.len() {
            if next < taken.len() && taken[next] as usize == read {
                next += 1;
                continue;
            }
            leftovers[write] = leftovers[read];
            write += 1;
        }
        leftovers.truncate(write);
    }
    let basics = basic_split(main, colors, total - duals.len() as u32, squared_pips);
    (duals, basics)
}

/// Swap main-colour basics (from the colour with the most left) for basics
/// of `c` until `lands` hold `n` sources of it, so a policy-chosen splash
/// gets the sources the pip-share split would deny it.
pub fn ensure_sources(lands: &mut [CardFactory], c: Color, n: usize, main: &[Color]) {
    let makes = |f: &CardFactory| land_produced_colors(crate::cube::card_def(*f)).contains(c);
    while lands.iter().filter(|f| makes(f)).count() < n {
        let basics = |m: Color| lands.iter().filter(|f| **f as usize == basic_for(m) as usize).count();
        let Some(h) = main.iter().copied().filter(|&m| basics(m) > 0).max_by_key(|&m| basics(m)) else { break };
        let i = lands.iter().position(|f| *f as usize == basic_for(h) as usize).expect("counted above");
        lands[i] = basic_for(c);
    }
}

/// The land half of a rated build for spells chosen elsewhere
/// (`crate::deck_policy`): [`assemble_lands`] over the spells' colours
/// (hybrid-aware) and the pool's remaining duals, as `ShapeBuild::complete`
/// calls it. Returns `total` lands, fewer only when the spells have no
/// coloured pip and the pool no dual.
pub fn lands_for_spells(pool: &[CardFactory], spells: &[CardFactory], total: u32) -> Vec<CardFactory> {
    let counts = hybrid_aware_colors(spells);
    let mut colors: Vec<Color> = Color::ALL.into_iter().filter(|&c| counts.get(c) > 0).collect();
    if colors.is_empty() {
        // An all-colourless main: the pool's own heaviest colour, so the
        // basics have somewhere to go.
        let pool_counts = colors_of_picks(pool);
        colors.extend(Color::ALL.into_iter().max_by_key(|&c| pool_counts.get(c)));
    }
    let mut leftovers: Vec<CardFactory> = pool.to_vec();
    for s in spells {
        if let Some(i) = leftovers.iter().position(|f| *f as usize == *s as usize) {
            leftovers.remove(i);
        }
    }
    let slots: Vec<(u32, crate::mana::ColorSet)> = leftovers
        .iter()
        .enumerate()
        .filter(|(_, f)| crate::cube::card_def(**f).is_land())
        .map(|(i, f)| (i as u32, land_produced_colors(crate::cube::card_def(*f))))
        .collect();
    let (mut lands, basics) = assemble_lands(&mut leftovers, &slots, spells, &colors, total, true);
    for c in Color::ALL {
        for _ in 0..basics.get(&c).copied().unwrap_or(0) {
            lands.push(basic_for(c));
        }
    }
    lands
}

/// `"R/W"`, `"R/W + b"`. Pushed a char at a time into one `String`: the
/// per-colour `join` it replaced was a measurable share of a sealed build.
fn candidate_label(colors: &[Color], splash: &[Color]) -> String {
    let mut s = String::with_capacity(2 * (colors.len() + splash.len()) + 3);
    for (i, &c) in colors.iter().enumerate() {
        if i > 0 {
            s.push('/');
        }
        s.push(color_letter(c));
    }
    if !splash.is_empty() {
        s.push_str(" + ");
        for (i, &c) in splash.iter().enumerate() {
            if i > 0 {
                s.push('/');
            }
            s.push(color_letter(c).to_ascii_lowercase());
        }
    }
    s
}

/// One shape's spells, ranked but without land base or label —
/// [`ShapeBuild::complete`] finishes it into a [`CandidateBuild`]. The
/// lattice is mostly only ranked, so land assembly is deferred to callers
/// that want a deck (PERF (-63)).
pub(crate) struct ShapeBuild {
    pub colors: ColorList,
    pub splash: ColorList,
    pub main: Vec<CardFactory>,
    pub static_score: i32,
    /// Pool cards not in the main deck — `assemble_lands` takes its duals out
    /// of this and compacts it.
    leftovers: Vec<CardFactory>,
    /// `(index into leftovers, colors produced)` per land in the pool.
    land_slots: Vec<(u32, crate::mana::ColorSet)>,
    /// Land slots to fill, after the 40-card floor.
    lands: u32,
    /// Whether the two fields above were built — [`Self::complete`] needs
    /// them, and a `Ranked` shape does not carry them.
    detail: Detail,
}

/// [`build_shape`] as far as ranking needs: spells and static score.
/// `noise > 0` jitters picks for randomized builds. `None` when the shape is
/// hollow (no splash-worthy cards for a splash shape, or no playables).
fn rank_shape<R: Rng>(
    colors: &[Color],
    splash_colors: &[Color],
    // (spell slots, land count, pick jitter)
    shape: (usize, u32, i32),
    cfg: &SimConfig,
    rng: &mut R,
    // The pool and its shape-invariant per-card facts — see [`PoolScores`].
    scores: &PoolScores<'_>,
    detail: Detail,
) -> Option<(ShapeBuild, Vec<u8>)> {
    debug_assert_eq!(
        scores.quality, cfg.builder_v2,
        "PoolScores was built under the other scorer; splash_cards reads its base scores",
    );
    debug_assert_eq!(
        scores.rated, cfg.rated_cards,
        "PoolScores was built under the other card ratings; splash_cards reads its base scores",
    );
    debug_assert_eq!(
        scores.aggro, cfg.curve_aggro,
        "PoolScores was built under the other curve term; splash_cards reads its base scores",
    );
    let (spells, lands, noise) = shape;
    let splash: Vec<CardFactory> = splash_colors
        .iter()
        .flat_map(|&c| splash_cards(scores, colors, c, cfg))
        .collect();
    if !splash_colors.is_empty() && splash.is_empty() {
        return None;
    }
    let MainDeck { main, counts, leftovers, lands: land_slots } =
        suggest_main_deck_shape(scores, colors, &splash, spells, noise, rng, detail, cfg.shape_slots);
    if main.is_empty() {
        return None;
    }
    Some((ShapeBuild {
        static_score: if cfg.builder_v3 {
            static_build_score_v3(&main, spells, cfg.rated_cards)
        } else {
            static_build_score(&main, spells)
        },
        colors: ColorList::of(colors),
        splash: ColorList::of(splash_colors),
        // Legal-deck floor: when the pool can't fill the requested spell
        // count, pad lands so the deck still reaches 40 cards.
        lands: lands.max(40u32.saturating_sub(main.len() as u32)),
        main,
        leftovers,
        land_slots,
        detail,
    }, counts))
}

impl ShapeBuild {
    /// Give the shape its land base and its label.
    fn complete(self, cfg: &SimConfig) -> CandidateBuild {
        debug_assert_eq!(self.detail, Detail::Full, "a Ranked shape has no leftovers to draw duals from");
        let ShapeBuild {
            colors, splash, main, static_score, mut leftovers, land_slots, lands, detail: _,
        } = self;
        // Land colors: main colors plus any splash color actually present.
        let (colors, splash) = (colors.as_slice(), splash.as_slice());
        let mut land_colors = Vec::with_capacity(colors.len() + splash.len());
        land_colors.extend_from_slice(colors);
        for &c in splash {
            if main.iter().any(|&f| crate::cube::card_brief(f).pips.get(c) > 0) {
                land_colors.push(c);
            }
        }
        let (duals, mut basics) =
            assemble_lands(&mut leftovers, &land_slots, &main, &land_colors, lands, cfg.builder_v2);
        if cfg.rated_cards && pip_floor_on() {
            double_pip_floor(&main, colors, &duals, &mut basics);
        }
        CandidateBuild {
            static_score,
            label: candidate_label(colors, splash),
            colors: colors.to_vec(),
            splash: splash.to_vec(),
            main,
            duals,
            basics,
            leftovers,
        }
    }
}

/// Build one deck of the given shape: chosen spells, on-color duals,
/// basic split, static score. See [`rank_shape`] for the half a caller that
/// only ranks shapes needs.
fn build_shape<R: Rng>(
    colors: &[Color],
    splash_colors: &[Color],
    shape: (usize, u32, i32),
    cfg: &SimConfig,
    rng: &mut R,
    scores: &PoolScores<'_>,
) -> Option<CandidateBuild> {
    rank_shape(colors, splash_colors, shape, cfg, rng, scores, Detail::Full)
        .map(|(s, _)| s.complete(cfg))
}

/// Enumerate every candidate build for `pool`: 10 pairs, pair + splash,
/// 10 triples, 5 four-color, 1 five-color. Sorted by descending static
/// score, deduplicated by identical main-deck contents (a triple whose
/// third color contributes nothing collapses into its pair).
pub fn enumerate_candidates(pool: &[CardFactory], cfg: &SimConfig) -> Vec<CandidateBuild> {
    enumerate_candidates_with(&PoolScores::new(pool, cfg.builder_v2, cfg.curve_aggro, cfg.rated_cards), cfg)
}

/// [`enumerate_candidates`] against a prebuilt [`PoolScores`], for callers
/// building more decks from the same pool.
pub(crate) fn enumerate_candidates_with(
    scores: &PoolScores<'_>,
    cfg: &SimConfig,
) -> Vec<CandidateBuild> {
    lattice(scores, cfg, Detail::Full).into_iter().map(|s| s.complete(cfg)).collect()
}

/// [`enumerate_candidates_with`] stopping at the ranking: [`ShapeBuild`]s in
/// the same order, no land base or label. [`build_random_deck_from`] reads
/// the scores and colours and rebuilds the shape it picks.
pub(crate) fn enumerate_shapes(scores: &PoolScores<'_>, cfg: &SimConfig) -> Vec<ShapeBuild> {
    lattice(scores, cfg, Detail::Ranked)
}

/// The shape lattice itself, at whichever detail the caller can use.
fn lattice(scores: &PoolScores<'_>, cfg: &SimConfig, detail: Detail) -> Vec<ShapeBuild> {
    // (main, splash) shapes: 10 pairs each with its three splashes then
    // bare, then the 16 wide subsets. The sort below is stable, so this
    // order is a tie-break and must not change.
    const SHAPES: usize = 56;
    let mut shapes: Vec<(ColorList, ColorList)> = Vec::with_capacity(SHAPES);
    for mask in 1u8..(1 << 5) {
        if mask.count_ones() != 2 {
            continue;
        }
        let pair = ColorList::from_mask(mask);
        for (i, third) in Color::ALL.into_iter().enumerate() {
            if mask & (1 << i) == 0 {
                shapes.push((pair, ColorList::one(third)));
            }
        }
        shapes.push((pair, ColorList::EMPTY));
    }
    for mask in 1u8..(1 << 5) {
        if (3..=5).contains(&mask.count_ones()) {
            shapes.push((ColorList::from_mask(mask), ColorList::EMPTY));
        }
    }
    debug_assert_eq!(shapes.len(), SHAPES);

    // Empty means unrestricted: read it as WUBRG.
    let allowed = if cfg.color_restriction.is_empty() {
        ColorSet::all()
    } else {
        cfg.color_restriction
    };
    let mut rng = StdRng::seed_from_u64(0); // noise=0 → unused; keeps API one-shape
    let mut out: Vec<ShapeBuild> = Vec::with_capacity(SHAPES);
    // Dedup on the copy-cap counts: equal iff the main decks are the same
    // multiset, without sorting a `Vec<CardFactory>` per shape (PERF (-63)).
    let mut seen_mains: crate::fxhash::HashSet<Vec<u8>> = crate::fxhash::HashSet::default();
    for (colors, splash_colors) in shapes {
        // The restriction covers the splash too: "only Lorehold" means no
        // third colour through a splash slot.
        if !colors.set().union(splash_colors.set()).is_subset_of(allowed) {
            continue;
        }
        let Some((build, counts)) = rank_shape(
            colors.as_slice(),
            splash_colors.as_slice(),
            (cfg.target_spells, cfg.total_lands, 0),
            cfg,
            &mut rng,
            scores,
            detail,
        ) else {
            continue;
        };
        if !seen_mains.insert(counts) {
            continue;
        }
        out.push(build);
    }
    out.sort_by_key(|c| std::cmp::Reverse(c.static_score));
    out
}

// ─────────────────────────────── gauntlet ────────────────────────────────

/// Build one randomized deck from `pulls`: colours by a temperature-scaled
/// softmax over the top shapes, jittered card picks, spell/land counts
/// sampled from the config ranges.
pub(crate) fn build_random_deck<R: Rng>(pulls: &[CardFactory], cfg: &SimConfig, rng: &mut R) -> GauntletDeck {
    // Same full lattice as user candidates: a pairs-only field never bombs
    // back at splash shapes and inflates their win rates.
    let scores = PoolScores::new(pulls, cfg.builder_v2, cfg.curve_aggro, cfg.rated_cards);
    let shapes = enumerate_shapes(&scores, cfg);
    build_random_deck_from(&shapes, &scores, cfg, rng)
}

/// [`build_random_deck`] with the shape lattice (and its [`PoolScores`])
/// already built. `enumerate_shapes` is deterministic in `(pulls, cfg)` — it
/// never draws from its rng — so a caller building n decks from one pool
/// (`selfplay::build_candidates`) enumerates once; everything that varies
/// per deck (softmax roll, spell/land split, pick jitter) reads only `rng`.
pub(crate) fn build_random_deck_from<R: Rng>(
    shapes: &[ShapeBuild],
    scores: &PoolScores<'_>,
    cfg: &SimConfig,
    rng: &mut R,
) -> GauntletDeck {
    let t = cfg.build_temperature.max(0.0);
    // Softmax over the top 5 shapes' static scores: a dominant build is
    // played nearly always, close calls stay diverse. A ~12-point gap costs
    // ~e× likelihood at t = 1; t → 0 is the argmax.
    // An empty lattice (no playable card) must not panic on `shapes[0]` —
    // this is on the self-play actor's path, and a panic kills a training
    // run. Degrade to an empty deck instead.
    if shapes.is_empty() {
        return GauntletDeck { label: "empty".to_string(), cards: Vec::new() };
    }
    let k = shapes.len().min(5);
    let scale = (12.0 * t).max(1e-6);
    let best = shapes[0].static_score as f64;
    let weights: Vec<f64> =
        shapes[..k].iter().map(|s| ((s.static_score as f64 - best) / scale).exp()).collect();
    let total: f64 = weights.iter().sum();
    let mut roll = rng.random_range(0.0..total);
    let mut idx = 0;
    for (i, w) in weights.iter().enumerate() {
        if roll < *w {
            idx = i;
            break;
        }
        roll -= w;
    }
    let chosen = &shapes[idx];

    // Noisy rebuild of the chosen shape. The jitter is deliberately mild:
    // heavy jitter made the field soft and inflated measured win rates.
    let (spells, lands) = sample_deck_split(cfg, rng);
    let noise = (t * 2.0).round() as i32;
    let build = build_shape(
        chosen.colors.as_slice(),
        chosen.splash.as_slice(),
        (spells, lands, noise),
        cfg,
        rng,
        scores,
    )
    .or_else(|| {
        // Defensive: a lattice shape shouldn't be hollow; retry unjittered.
        build_shape(
            chosen.colors.as_slice(),
            chosen.splash.as_slice(),
            (cfg.target_spells, cfg.total_lands, 0),
            cfg,
            rng,
            scores,
        )
    })
    .or_else(|| {
        // Then the rest of the lattice: a hollow shape is skipped, never a
        // panic on the actor's path.
        shapes.iter().find_map(|s| {
            build_shape(
                s.colors.as_slice(),
                s.splash.as_slice(),
                (cfg.target_spells, cfg.total_lands, 0),
                cfg,
                rng,
                scores,
            )
        })
    });
    // Every shape hollow: same degradation as the empty lattice. Not a
    // `debug_assert!` — an unplayable pool is legal input, not a broken
    // invariant.
    let Some(build) = build else {
        return GauntletDeck { label: "empty".to_string(), cards: Vec::new() };
    };
    GauntletDeck { label: build.label.clone(), cards: build.deck() }
}

// ─────────────────────────────── simulation ──────────────────────────────

/// Which bot drives a seat — a value rather than a bool because the ladder
/// pits evaluation profiles against each other.
#[derive(Debug, Clone, Copy)]
pub enum Pilot {
    /// The scored bot with a specific evaluation profile.
    Scored(EvalWeights),
    /// The pre-scoring uniform-random pick. Ladder control.
    Uniform,
    /// The Monte Carlo search bot.
    Mcts(MctsConfig),
    /// The policy pilot (`server::policy_pilot`): every decision by the
    /// policy head of the net in the weights' slot, at temperature 0.
    Policy(EvalWeights),
}

impl Pilot {
    /// The evaluation profile this pilot plays, when it has one. Used to
    /// push profile settings that the *engine* reads (rather than the
    /// bot) onto the seat before the game starts.
    pub(crate) fn weights(self) -> Option<EvalWeights> {
        match self {
            Pilot::Scored(w) | Pilot::Policy(w) => Some(w),
            Pilot::Uniform | Pilot::Mcts(_) => None,
        }
    }

    pub(crate) fn build(self) -> Box<dyn Bot> {
        match self {
            Pilot::Scored(w) => Box::new(HeuristicBot::with_weights(w)),
            Pilot::Uniform => Box::new(HeuristicBot::uniform_baseline()),
            Pilot::Mcts(cfg) => Box::new(MctsBot::new(cfg)),
            Pilot::Policy(w) => Box::new(crate::server::policy_pilot::PolicyBot::new(w, 0.0, 0)),
        }
    }
}

impl Default for Pilot {
    fn default() -> Self {
        Pilot::Scored(EvalWeights::default())
    }
}

impl From<bool> for Pilot {
    /// `true` = the uniform control, matching the old `uniform_a` flags.
    fn from(uniform: bool) -> Self {
        if uniform { Pilot::Uniform } else { Pilot::default() }
    }
}

/// Simulator work done by a batch of games, for throughput measurement.
/// Win rates say whether a pilot is stronger; this says what the games
/// cost. Summed across workers by `bot_ladder --bench`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SimCost {
    /// Games played, decided or not.
    pub games: u64,
    /// Accepted `perform_action` calls — the simulator's unit of work.
    /// Bot decisions that were rejected or declined aren't counted.
    pub decisions: u64,
    /// Sum of `turn_number` at game end.
    pub turns: u64,
    /// Undecided games that ran out of action budget — the loop was still
    /// making moves.
    pub action_capped: u64,
    /// Undecided games the BOARD bound ended ([`StopReason::BoardCap`]): a
    /// token-doubling runaway, not a loop and not slow.
    ///
    /// ⚠ Counted apart from `action_capped` because a bigger
    /// `CRAB_MAX_ACTIONS` can't clear it (a Krenko board doubles past 1,024
    /// permanents in one activation); summed in, seed 1169 on `cube` read as
    /// ten defects.
    pub board_capped: u64,
    /// Undecided games where neither bot had a move accepted for eight
    /// consecutive rounds — the loop was stuck, not slow.
    pub no_legal_move: u64,
    /// Games that ended by the rules with no winner (CR 104.4 draw).
    pub draws: u64,
}

impl SimCost {
    fn record(&mut self, o: &GameOutcome) {
        self.games += 1;
        self.decisions += o.actions as u64;
        self.turns += o.turns as u64;
        self.count_stop(o.stop, o.winner.is_none());
    }

    /// Tally one game's exit into the four undecided buckets. Public so the
    /// suite can test the buckets (see `board_capped`) without the
    /// crate-private `GameOutcome`.
    pub fn count_stop(&mut self, stop: StopReason, no_winner: bool) {
        match stop {
            StopReason::ActionCap => self.action_capped += 1,
            StopReason::BoardCap => self.board_capped += 1,
            StopReason::NoLegalMove => self.no_legal_move += 1,
            StopReason::GameOver if no_winner => self.draws += 1,
            StopReason::GameOver => {}
        }
    }
}

impl std::ops::AddAssign for SimCost {
    fn add_assign(&mut self, r: Self) {
        self.games += r.games;
        self.decisions += r.decisions;
        self.turns += r.turns;
        self.action_capped += r.action_capped;
        self.board_capped += r.board_capped;
        self.no_legal_move += r.no_legal_move;
        self.draws += r.draws;
    }
}

/// Decided/undecided tally of `games` bot-vs-bot games between two decks,
/// alternating which deck sits in seat 0 (turn order). `uniform_*` selects
/// the legacy uniform-pick bot for that side (A/B ladders).
pub struct MatchTally {
    pub wins_a: u32,
    pub wins_b: u32,
    pub undecided: u32,
    /// Work done, for throughput reporting.
    pub cost: SimCost,
    /// Per-game results in play order: +1 deck A won, −1 deck B won,
    /// 0 undecided — the raw material for paired per-slot statistics.
    pub outcomes: Vec<i8>,
}

/// `seed_base`: when `Some`, game `i`'s deck shuffles come from a seeded
/// stream keyed only by `seed_base + i` — common random numbers, so
/// different deck-A candidates sharing a `seed_base` face identical
/// opponent behavior game-for-game.
pub fn simulate_match_games(
    deck_a: &[CardFactory],
    deck_b: &[CardFactory],
    games: usize,
    uniform_a: bool,
    uniform_b: bool,
    max_actions: usize,
    seed_base: Option<u64>,
) -> MatchTally {
    simulate_match_games_piloted(
        deck_a,
        deck_b,
        games,
        [uniform_a.into(), uniform_b.into()],
        max_actions,
        seed_base,
    )
}

/// [`simulate_match_games`] with an explicit pilot per side — the entry
/// point the bot ladder uses to play evaluation profile A against B.
pub fn simulate_match_games_piloted(
    deck_a: &[CardFactory],
    deck_b: &[CardFactory],
    games: usize,
    pilots: [Pilot; 2],
    max_actions: usize,
    seed_base: Option<u64>,
) -> MatchTally {
    simulate_match_games_cross(deck_a, deck_b, games, pilots, max_actions, seed_base, None, true)
}

/// [`simulate_match_games_piloted`] against the build on the far end of
/// `cross` — the cross-binary ladder's unpaired control arm. Unseeded jitter
/// is harmless here: only actions cross the link. `seed_base` must be `Some`
/// (both sides must agree on the shuffle).
#[allow(clippy::too_many_arguments)]
pub fn simulate_match_games_cross(
    deck_a: &[CardFactory],
    deck_b: &[CardFactory],
    games: usize,
    pilots: [Pilot; 2],
    max_actions: usize,
    seed_base: Option<u64>,
    mut cross: Option<&mut crate::crossplay::CrossGame>,
    i_am_a: bool,
) -> MatchTally {
    // Build both seat arrangements once (unshuffled) and clone per game —
    // far cheaper than re-invoking ~80 card factories.
    let template_a0 = build_match_template(deck_a, deck_b);
    let template_b0 = build_match_template(deck_b, deck_a);
    let mut tally = MatchTally {
        wins_a: 0,
        wins_b: 0,
        undecided: 0,
        cost: SimCost::default(),
        outcomes: Vec::with_capacity(games),
    };
    for i in 0..games {
        let a_seat0 = i % 2 == 0;
        let mut shuffle_rng = seed_base.map(|b| {
            StdRng::seed_from_u64(
                b.wrapping_add((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)),
            )
        });
        let template = if a_seat0 { &template_a0 } else { &template_b0 };
        // Swap pilots with decks so turn order (worth a few points) isn't
        // confounded with the profile under test.
        let seated =
            if a_seat0 { [pilots[0], pilots[1]] } else { [pilots[1], pilots[0]] };
        // Unpaired games keep the unseeded jitter: this is the control
        // arm, and seeding it here would quietly change what it measures.
        let outcome = match &mut cross {
            None => play_one_game(template, seated, max_actions, shuffle_rng.as_mut(), None),
            Some(cx) => {
                cx.seat_at(usize::from(a_seat0 != i_am_a));
                play_one_game_traced(
                    template,
                    seated,
                    max_actions,
                    shuffle_rng.as_mut(),
                    None,
                    GameHooks { cross: Some(&mut **cx), ..Default::default() },
                )
            }
        };
        if cross.as_ref().is_some_and(|cx| cx.fault.is_some()) {
            return tally;
        }
        tally.cost.record(&outcome);
        match outcome.winner {
            Some(seat) => {
                let a_won = (seat == 0) == a_seat0;
                if a_won {
                    tally.wins_a += 1;
                    tally.outcomes.push(1);
                } else {
                    tally.wins_b += 1;
                    tally.outcomes.push(-1);
                }
            }
            None => {
                tally.undecided += 1;
                tally.outcomes.push(0);
            }
        }
    }
    tally
}

/// A seat-swapped pair of games played on one shuffle: the unit of the
/// paired ladder. `pairs[k]` is +1 when A took both games, −1 when B did,
/// and 0 for a split (including a pair with an undecided game).
pub struct PairedTally {
    pub wins_a: u32,
    pub wins_b: u32,
    pub undecided: u32,
    /// Work done, for throughput reporting.
    pub cost: SimCost,
    /// One entry per fully decided pair, in play order. A pair with an
    /// undecided game is absent, not a split: scoring it 0 would dilute the
    /// mean toward even. Its games still count in the unpaired totals.
    pub pairs: Vec<i8>,
    /// Under `CRAB_AIVAT`, A's summed luck over each entry of `pairs`'s two
    /// games (`server::aivat`); empty otherwise.
    pub pair_luck: Vec<f64>,
}

/// Wilson score interval at `z` standard normal deviates. Not the normal
/// approximation, which has zero width at p̂ = 0 or 1 (a lucky 5-0 once
/// claimed a lower bound of 1.0; see
/// `wilson_interval_is_not_degenerate_at_extremes`).
pub fn wilson(wins: u32, n: u32, z: f64) -> (f64, f64) {
    if n == 0 {
        return (0.0, 1.0);
    }
    let (n, p) = (n as f64, wins as f64 / n as f64);
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let half = (z / denom) * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
    ((center - half).max(0.0), (center + half).min(1.0))
}

/// Paired win-rate estimate from per-pair scores in {−1, 0, +1}.
///
/// `p̂ = (E[S] + 1) / 2` and `SE(p̂) = sd(S) / (2√n)`, read off the
/// empirical distribution — no independence assumption between a pair's
/// games, which pairing breaks.
pub struct PairedStat {
    pub n: usize,
    pub p: f64,
    pub se: f64,
    /// Within-pair correlation, backed out of Var(S) = 2p(1−p)(1 + ρ).
    /// Negative is the useful direction: the deal decided both games the
    /// same way, so it cancelled.
    pub rho: f64,
}

pub fn paired_stat(pairs: &[i8]) -> Option<PairedStat> {
    let n = pairs.len();
    if n < 2 {
        return None;
    }
    let (nf, sum) = (n as f64, pairs.iter().map(|&s| s as f64).sum::<f64>());
    let mean = sum / nf;
    // S ∈ {−1,0,1} so E[S²] is just the sweep fraction.
    let var = pairs.iter().map(|&s| (s as f64) * (s as f64)).sum::<f64>() / nf - mean * mean;
    let p = (mean + 1.0) / 2.0;
    let se = (var.max(0.0) / nf).sqrt() / 2.0;
    // Independent games would give Var(S) = 2p(1−p); the shortfall is ρ.
    let indep = 2.0 * p * (1.0 - p);
    let rho = if indep > 1e-9 { var / indep - 1.0 } else { 0.0 };
    Some(PairedStat { n, p, se, rho })
}

/// [`simulate_match_games_piloted`] with antithetic seat pairing: each pair
/// plays the same shuffle twice with the pilots swapped.
///
/// In a 40-card mirror most game-to-game variance is the deal. Playing both
/// sides of one deal cancels it within the pair: a deal-decided game is won
/// by each pilot once and the pair splits, so only a real piloting edge
/// sweeps. The pair score S ∈ {−1, 0, +1} has E[S] = 2p − 1 and
/// Var(S) = 2p(1−p)(1 + ρ); a decisive-deal field drives ρ negative and the
/// estimate's variance falls by (1 + ρ). `bot_ladder` reports the measured ρ.
///
/// A variance reduction, not a new estimand: the mean is the unpaired win
/// rate, which is still reported as the control. Both games draw from one
/// `seed_base + k` stream, so seat 0 gets the same library both times.
pub fn simulate_match_pairs_piloted(
    deck_a: &[CardFactory],
    deck_b: &[CardFactory],
    pairs: usize,
    pilots: [Pilot; 2],
    max_actions: usize,
    seed_base: u64,
) -> PairedTally {
    simulate_match_pairs_cross(deck_a, deck_b, pairs, pilots, max_actions, seed_base, None, true)
}

/// [`simulate_match_pairs_piloted`] with the other build on the far end of
/// `cross`: this process is side A when `i_am_a`, and the seat it does not
/// hold is played by the peer. Both processes run this loop over the same
/// seeds, so no schedule is sent; see `crate::crossplay`.
///
/// The jitter seed is salted by seat (unlike the in-process loop, where one
/// stream serves both seats): it keeps the seats independent while a pair's
/// second game stays an exact replay with binaries swapped, so self-vs-self
/// splits every pair.
#[allow(clippy::too_many_arguments)]
pub fn simulate_match_pairs_cross(
    deck_a: &[CardFactory],
    deck_b: &[CardFactory],
    pairs: usize,
    pilots: [Pilot; 2],
    max_actions: usize,
    seed_base: u64,
    mut cross: Option<&mut crate::crossplay::CrossGame>,
    i_am_a: bool,
) -> PairedTally {
    let template_a0 = build_match_template(deck_a, deck_b);
    let template_b0 = build_match_template(deck_b, deck_a);
    let mut tally = PairedTally {
        wins_a: 0,
        wins_b: 0,
        undecided: 0,
        cost: SimCost::default(),
        pairs: Vec::with_capacity(pairs),
        pair_luck: Vec::new(),
    };
    for k in 0..pairs {
        let seed = seed_base.wrapping_add((k as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut score = 0i8;
        let mut decided_both = true;
        let mut luck_a = 0.0f64;
        for a_seat0 in [true, false] {
            // Re-seeded per game so the second game draws the same shuffle.
            let mut shuffle_rng = StdRng::seed_from_u64(seed);
            let template = if a_seat0 { &template_a0 } else { &template_b0 };
            let seated = if a_seat0 { [pilots[0], pilots[1]] } else { [pilots[1], pilots[0]] };
            // Same jitter stream for both games, so tie-breaks replay and
            // only the profiles separate the two.
            let outcome = match &mut cross {
                None => match trace_dump_dir() {
                    None => play_one_game(
                        template,
                        seated,
                        max_actions,
                        Some(&mut shuffle_rng),
                        Some(seed),
                    ),
                    Some(dir) => {
                        let mut lines = Vec::new();
                        let mut end = Vec::new();
                        let o = play_one_game_traced(
                            template,
                            seated,
                            max_actions,
                            Some(&mut shuffle_rng),
                            Some(seed),
                            GameHooks { trace: Some(&mut lines), end_zones: Some(&mut end), ..Default::default() },
                        );
                        let t = GameTrace { lines, winner: o.winner, turns: o.turns };
                        // A run-wide counter in the name: one seed base can serve
                        // several matches (`--pair-strength` plays ten decks on one).
                        static DUMPED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
                        let n = DUMPED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let path = dir.join(format!("{seed_base:016x}_{k}_{}_{n}.txt", u8::from(a_seat0)));
                        let mut text: String = zone_lines(template).into_iter().map(|l| l.replace("# p", "# start p") + "\n").collect();
                        text += &t.text();
                        text.extend(end.into_iter().map(|l| l.replace("# p", "# end p") + "\n"));
                        if let Err(e) = std::fs::write(&path, text) {
                            eprintln!("CRAB_DUMP_TRACES: {}: {e}", path.display());
                        }
                        o
                    }
                },
                Some(cx) => {
                    let my_seat = usize::from(a_seat0 != i_am_a);
                    cx.seat_at(my_seat);
                    play_one_game_traced(
                        template,
                        seated,
                        max_actions,
                        Some(&mut shuffle_rng),
                        Some(seed ^ (my_seat as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)),
                        GameHooks { cross: Some(&mut **cx), ..Default::default() },
                    )
                }
            };
            if cross.as_ref().is_some_and(|cx| cx.fault.is_some()) {
                return tally;
            }
            tally.cost.record(&outcome);
            luck_a += if a_seat0 { outcome.luck } else { -outcome.luck };
            match outcome.winner {
                Some(seat) => {
                    if (seat == 0) == a_seat0 {
                        tally.wins_a += 1;
                        score += 1;
                    } else {
                        tally.wins_b += 1;
                        score -= 1;
                    }
                }
                None => {
                    tally.undecided += 1;
                    decided_both = false;
                }
            }
        }
        if decided_both {
            tally.pairs.push(score.signum());
            if aivat_on() {
                tally.pair_luck.push(luck_a);
            }
        }
    }
    tally
}

/// Sample a spell/land split summing to 40: spells from
/// `spell_count_range`, lands = 40 − spells clamped into `land_count_range`,
/// spells re-derived from the clamp. (Independent sampling made illegal,
/// thinner 38-39-card decks that won on the bug.)
fn sample_deck_split<R: Rng>(cfg: &SimConfig, rng: &mut R) -> (usize, u32) {
    const DECK_SIZE: usize = 40;
    let spells = rng.random_range(cfg.spell_count_range.0..=cfg.spell_count_range.1);
    let lands = (DECK_SIZE.saturating_sub(spells) as u32)
        .clamp(cfg.land_count_range.0, cfg.land_count_range.1);
    let spells = DECK_SIZE.saturating_sub(lands as usize);
    (spells, lands)
}

/// Unshuffled two-seat state with both libraries loaded — the clone-me
/// template for [`play_one_game`].
pub(crate) fn build_match_template(seat0: &[CardFactory], seat1: &[CardFactory]) -> GameState {
    let mut g = GameState::new(vec![Player::new(0, "A"), Player::new(1, "B")]);
    for (seat, deck) in [seat0, seat1].into_iter().enumerate() {
        for &f in deck {
            // `card_arc`, not `f()`: a `CardDefinition` is several KB.
            g.add_card_to_library(seat, crate::cube::card_arc(f));
        }
        g.players[seat].wants_ui = true;
    }
    g
}

/// Consecutive no-move rounds before a bot play loop gives the game up;
/// deliberately small. Shared with [`crate::selfplay`]'s recording loop so
/// the two stall counts stay comparable.
pub const STALE_ROUNDS: usize = 8;

/// Why a game's play loop stopped. Undecided games split into out of
/// budget, board-capped, stuck (no bot could move) and drawn — they want
/// different fixes, so the outcome says which.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopReason {
    /// `is_game_over()` — a winner, or an in-game draw (CR 104.4).
    GameOver,
    /// `actions >= max_actions`.
    ActionCap,
    /// The battlefield passed [`MAX_BATTLEFIELD`] (or the stack
    /// [`MAX_STACK`] spells or [`MAX_STACK_ITEMS`] items): a token-doubling
    /// board or copy chain, bounded so one game cannot hold a thread for
    /// hours.
    BoardCap,
    /// [`STALE_ROUNDS`] consecutive rounds in which neither bot had an
    /// action accepted.
    NoLegalMove,
}

/// The simulator's battlefield bound. No legitimate 40-card game nears a
/// thousand permanents; a board past it is a runaway (Scute Swarm doubling
/// on every land drop — 1,377 copies at action 4,001, cube seed 43,
/// 2026-09-08), ended as undecided like an action-capped game.
pub const MAX_BATTLEFIELD: usize = 1_024;

/// The stack's bound on SPELLS: a copy chain past it copies nothing more,
/// and a stack holding that many ends the game as a `BoardCap` (Venser
/// copying a Replication Technique that copies Venser never returned; pod
/// seed 65002 game 41). Triggers aren't counted — a big attack legitimately
/// stacks hundreds.
pub const MAX_STACK: usize = 512;

/// The stack's bound on ITEMS, triggers included: four times the board
/// bound, so a whole-board fan-out fits. Catches trigger loops the action
/// budget never counts (Dina + Exquisite Blood against a seat that can't
/// lose, pod seed 4500524 game 17). Eight times was too loose: ~1,470 Scute
/// Swarms stacked 4,405 triggers at ~0.17 s a pass (pod seed 1017885).
pub const MAX_STACK_ITEMS: usize = 4 * MAX_BATTLEFIELD;

/// Where the board-bound gates stop a token effect (a held cast, a declined
/// free cast, a capped copy fan-out): an eighth short of `MAX_BATTLEFIELD`,
/// so the ordinary tokens a board keeps making (Treasure, a Clue) don't tip a
/// gated board over the bound one at a time.
pub const BOARD_GATE: usize = MAX_BATTLEFIELD - MAX_BATTLEFIELD / 8;

/// Doublings past which one more token doubler makes any later mint a
/// board-bound batch (`2^10` is `MAX_BATTLEFIELD`): the pod loop passes on an
/// activation that copies a doubler that far, and a populate's default pick
/// skips a doubler token there.
pub const TOKEN_DOUBLER_CEILING: u32 = 10;

/// [`BOARD_GATE`]'s stack sibling: an optional free cast is declined once this
/// many spells are on the stack, an eighth short of [`MAX_STACK`] (Jadzi
/// under Thousand-Year Storm; pod seed 16000293 game 20).
pub const STACK_GATE: usize = MAX_STACK - MAX_STACK / 8;

/// Why a driver loop stops, in the loop's own order — the rules first, then
/// the two simulator bounds, then staleness — or `None` to play on. Both
/// loops ([`play_one_game_traced`] and the actor's) ask this so the bounds
/// cannot drift apart.
pub fn stop_reason(
    g: &GameState,
    actions: usize,
    max_actions: usize,
    stale: usize,
) -> Option<StopReason> {
    if g.is_game_over() {
        Some(StopReason::GameOver)
    } else if actions >= max_actions {
        Some(StopReason::ActionCap)
    } else if g.battlefield.len() > MAX_BATTLEFIELD || g.stack.len() >= MAX_STACK_ITEMS || g.stack_spells_at_bound() {
        Some(StopReason::BoardCap)
    } else if stale >= STALE_ROUNDS {
        Some(StopReason::NoLegalMove)
    } else {
        None
    }
}

/// One finished game: the winning seat (`None` for a stall/draw) plus what
/// it cost to play.
pub(crate) struct GameOutcome {
    pub winner: Option<usize>,
    /// Accepted `perform_action` calls.
    pub actions: usize,
    /// `turn_number` at game end.
    pub turns: u32,
    /// Which of the loop's three exits ended this game.
    pub stop: StopReason,
    /// Seat 0's summed luck of the draw under `CRAB_AIVAT` (`server::aivat`),
    /// 0 otherwise.
    pub luck: f64,
}

/// `CRAB_AIVAT=1`: score every real draw's luck with the champion value net
/// (`server::aivat`), and play each accepted move with `perform_action`
/// instead of adopting the bot's settled state (the same state per
/// `GameState::accept_on`), since only the driver's own call is outside
/// every bot's search and may log draws. Read once.
pub fn aivat_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("CRAB_AIVAT").is_some_and(|v| v != "0"))
}

/// Play one full bot game from a prebuilt template. Mirrors the server
/// actor's fixed-point bot polling.
pub(crate) fn play_one_game(
    template: &GameState,
    pilots: [Pilot; 2],
    max_actions: usize,
    shuffle_rng: Option<&mut StdRng>,
    jitter_seed: Option<u64>,
) -> GameOutcome {
    play_one_game_traced(
        template,
        pilots,
        max_actions,
        shuffle_rng,
        jitter_seed,
        GameHooks::default(),
    )
}

/// A full game's observable history: one line per accepted action, each
/// with the state it produced. Two builds that agree on this agree on
/// everything a game can be judged by. Textual on purpose, so a moved trace's
/// diff shows which action diverged and the board there.
pub struct GameTrace {
    pub lines: Vec<String>,
    pub winner: Option<usize>,
    pub turns: u32,
}

impl GameTrace {
    /// FNV-1a over the joined lines, where committing the text would be bulk.
    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in self.lines.join("\n").bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x1000_0000_01b3);
        }
        h
    }

    pub fn text(&self) -> String {
        let mut s = self.lines.join("\n");
        s.push('\n');
        s.push_str(&format!(
            "= winner {} turns {} actions {}\n",
            match self.winner {
                Some(s) => s.to_string(),
                None => "none".to_string(),
            },
            self.turns,
            self.lines.len(),
        ));
        s
    }
}

/// `CRAB_SETTLE_CHECK=1` — the action-cap loop finder. Whenever the loop
/// adopts a bot's `settled` state it also runs the action on a clone and
/// reports: the two disagreeing, an accepted action that left the board's
/// fingerprint unchanged, and a rejected decision answer with its error (a
/// rejected answer drops the decision and rolls back the cast — the cube
/// stall of 2026-10-27, Arc Lightning offering a protected target).
/// `CRAB_SETTLE_MATCH=<substring>` also reports every action whose debug
/// text contains it. Pairs with `bot_ladder`'s `CRAB_LADDER_ONLY` and
/// `CRAB_MAX_ACTIONS`. Read once.
fn settle_check() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("CRAB_SETTLE_CHECK").is_some_and(|v| v != "0"))
}

/// `CRAB_SETTLE_MATCH=<substring>`: does `label` contain it? Read once.
fn settle_match(label: &str) -> bool {
    static PAT: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    PAT.get_or_init(|| std::env::var("CRAB_SETTLE_MATCH").ok()).as_deref().is_some_and(|m| label.contains(m))
}

fn settle_fingerprint(g: &GameState) -> String {
    let p = |i: usize| {
        let pl = &g.players[i];
        format!("life {} hand {} gy {} lib {}", pl.life, pl.hand.len(), pl.graveyard.len(), pl.library.len())
    };
    format!(
        "T{} {:?} prio {} stack {} bf {} pending {} | {} | {}",
        g.turn_number,
        g.step,
        g.priority.player_with_priority,
        g.stack.len(),
        g.battlefield.len(),
        g.pending_decision.is_some(),
        p(0),
        p(1)
    )
}

fn settle_diag(before: &GameState, settled: &GameState, label: &str, action: crate::game::GameAction) {
    let mut real = before.clone();
    let res = real.perform_action(action);
    let (b, st, r) = (settle_fingerprint(before), settle_fingerprint(settled), settle_fingerprint(&real));
    let matched = settle_match(label);
    if st != r || st == b || matched {
        eprintln!(
            "settle-check: {label}\n  before  {b}\n  settled {st}\n  real    {r} ({})",
            match &res {
                Ok(_) => "Ok".to_string(),
                Err(e) => format!("Err {e:?}"),
            }
        );
    }
}

/// `CRAB_OFFER_CHECK=1` — the census behind the settle check: at every
/// enumerable prompt (targets, modes, colours, library searches, options)
/// answer each offered option on a clone and report the ones the engine
/// rejects (the cube stall's class, which a bot only reveals by picking
/// it). One `offer-check:` line per new (source, prompt, error); `ALL` marks
/// a prompt with no accepted option. `offer-probed: N` lines count prompts
/// probed, at each power of two from 1 024. Read once.
fn offer_check() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("CRAB_OFFER_CHECK").is_some_and(|v| v != "0"))
}

fn offer_probe(g: &GameState) {
    use crate::decision::{Decision, DecisionAnswer as A};
    static SEEN: std::sync::Mutex<Option<std::collections::HashSet<String>>> = std::sync::Mutex::new(None);
    let Some(pd) = g.pending_decision.as_ref() else { return };
    let name = |id: crate::card::CardId| {
        g.find_card_anywhere(id).map(|c| c.definition.name.to_string()).unwrap_or_else(|| format!("{id:?}"))
    };
    let (src, what, answers): (String, String, Vec<A>) = match &pd.decision {
        Decision::ChooseTarget { source, legal, description, .. } => {
            (name(*source), format!("ChooseTarget {description:?}"), legal.iter().cloned().map(A::Target).collect())
        }
        Decision::ChooseMode { source, num_modes, .. } => {
            (name(*source), "ChooseMode".into(), (0..*num_modes).map(A::Mode).collect())
        }
        Decision::ChooseColor { source, legal } => {
            (name(*source), "ChooseColor".into(), legal.iter().copied().map(A::Color).collect())
        }
        Decision::ChooseOption { source, prompt, options } => {
            (name(*source), format!("ChooseOption {prompt:?}"), (0..options.len() as u32).map(A::Amount).collect())
        }
        Decision::SearchLibrary { candidates, eligible, .. } => (
            "search".into(),
            "SearchLibrary".into(),
            match eligible {
                Some(e) => e.iter().map(|id| A::Search(Some(*id))).collect(),
                None => candidates.iter().map(|(id, _)| A::Search(Some(*id))).collect(),
            },
        ),
        _ => return,
    };
    if answers.is_empty() {
        return;
    }
    // Proof of coverage, so a quiet census reads "clean", not "never ran".
    static PROBED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = PROBED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    if n.is_power_of_two() && n >= 1024 {
        eprintln!("offer-probed: {n} prompts");
    }
    let mut rejected = Vec::new();
    for a in &answers {
        let mut probe = g.clone();
        if let Err(e) = probe.perform_action(crate::game::GameAction::SubmitDecision(a.clone())) {
            rejected.push((a.clone(), e));
        }
    }
    if rejected.is_empty() {
        return;
    }
    let all = rejected.len() == answers.len();
    let mut seen = SEEN.lock().unwrap();
    let seen = seen.get_or_insert_with(Default::default);
    for (a, e) in rejected {
        let err = format!("{e:?}");
        let key = format!("{src} | {what} | {}", err.split(['(', ' ', '{']).next().unwrap_or(""));
        if seen.insert(key) {
            let shown = match &a {
                A::Target(crate::game::types::Target::Permanent(id)) => format!("Permanent({})", name(*id)),
                other => format!("{other:?}"),
            };
            eprintln!(
                "offer-check:{} {src} | {what} | answer {shown} | {err} | T{} {:?}",
                if all { " ALL" } else { "" },
                g.turn_number,
                g.step
            );
        }
    }
}

/// `CRAB_STUCK_DIAG=1` — when a game stops as stuck (no bot action accepted
/// for [`STALE_ROUNDS`] rounds), print the pending ask and what each seat
/// wanted and why the engine refused it. Read only on a stuck game.
fn stuck_diag() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("CRAB_STUCK_DIAG").is_some_and(|v| v != "0"))
}

fn report_stuck(g: &GameState, bots: &mut [Box<dyn crate::server::bot::Bot>]) {
    let ask = g.pending_decision.as_ref().map(|d| {
        let s = format!("{:?}", d.decision);
        (s.chars().take(300).collect::<String>(), d.acting_player())
    });
    eprintln!(
        "STUCK t{} {:?} priority {} stack {} ask {ask:?}",
        g.turn_number,
        g.step,
        g.player_with_priority(),
        g.stack.len(),
    );
    for (s, bot) in bots.iter_mut().enumerate() {
        let want = bot.next_action_settled(g, s).map(|b| b.action);
        let refused = want.clone().and_then(|a| g.clone().perform_action(a).err());
        eprintln!("STUCK   seat {s} wants {} -> {refused:?}", want.map_or("nothing".into(), |a| named_ids(g, &format!("{a:?}"))));
    }
}

/// `CRAB_TRACE_NAMES=1` — a traced action line also names every `CardId` it
/// mentions. Off by default: the golden traces are this format.
fn trace_names() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("CRAB_TRACE_NAMES").is_some_and(|v| v != "0"))
}

/// `line` with `[id=name, …]` for every `CardId(n)` in it.
fn named_ids(g: &GameState, line: &str) -> String {
    let names: Vec<String> = line
        .split("CardId(")
        .skip(1)
        .filter_map(|r| r.split(')').next()?.parse::<u32>().ok())
        .map(|n| format!("{n}={}", g.find_card_anywhere(crate::card::CardId(n)).map_or("?", |c| c.definition.name)))
        .collect();
    if names.is_empty() { line.to_string() } else { format!("{line} [{}]", names.join(", ")) }
}

/// `CRAB_DUMP_TRACES=<dir>` — every game the in-process paired loop plays is
/// written to `<dir>/<seed base>_<pair>_<seat order>_<n>.txt` as its
/// [`GameTrace`] text. Read once.
///
/// For "two binaries on one seed play different games": dump both runs and
/// diff the directories; the first differing line names the action and the
/// board.
fn trace_dump_dir() -> Option<&'static std::path::Path> {
    static DIR: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::path::PathBuf::from(std::env::var_os("CRAB_DUMP_TRACES")?);
        std::fs::create_dir_all(&dir).ok()?;
        Some(dir)
    })
    .as_deref()
}

/// Play one fully seeded game and record its trace. The shuffle stream and
/// the bot's tie-break jitter both key off `seed`, so the same seed on the
/// same engine must produce the same text — in this process or any other.
pub fn trace_game(
    deck_a: &[CardFactory],
    deck_b: &[CardFactory],
    seed: u64,
    max_actions: usize,
) -> GameTrace {
    let template = build_match_template(deck_a, deck_b);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut lines = Vec::new();
    let o = play_one_game_traced(
        &template,
        [Pilot::default(), Pilot::default()],
        max_actions,
        Some(&mut rng),
        Some(seed),
        GameHooks { trace: Some(&mut lines), ..Default::default() },
    );
    GameTrace { lines, winner: o.winner, turns: o.turns }
}

/// Board state after an action, compact enough to sit at the end of every
/// trace line and complete enough that a rules change moves it.
fn trace_state(g: &GameState) -> String {
    let mut bf = [0usize; 2];
    for c in g.battlefield.iter() {
        if let Some(n) = bf.get_mut(c.controller) {
            *n += 1;
        }
    }
    format!(
        "T{} {:?} | life {}/{} hand {}/{} bf {}/{} gy {}/{} lib {}/{} stack {}",
        g.turn_number,
        g.step,
        g.players[0].life,
        g.players[1].life,
        g.players[0].hand.len(),
        g.players[1].hand.len(),
        bf[0],
        bf[1],
        g.players[0].graveyard.len(),
        g.players[1].graveyard.len(),
        g.players[0].library.len(),
        g.players[1].library.len(),
        g.stack.len(),
    )
}

/// The optional observers a game can be played under, all off on the hot
/// path. Bundled so the play loop keeps one signature instead of a drifting
/// copy per observer (see filter 11).
#[derive(Default)]
pub(crate) struct GameHooks<'a> {
    /// One line per accepted action; see [`GameTrace`].
    pub trace: Option<&'a mut Vec<String>>,
    /// The other build's half of a cross-binary game: this process polls
    /// only `cross.seat` and reads the other seat's actions off the link.
    pub cross: Option<&'a mut crate::crossplay::CrossGame>,
    /// Every seat's zones when the game ends, as `id=name` lists
    /// (`CRAB_DUMP_TRACES`' footer).
    pub end_zones: Option<&'a mut Vec<String>>,
}

fn play_one_game_traced(
    template: &GameState,
    pilots: [Pilot; 2],
    max_actions: usize,
    shuffle_rng: Option<&mut StdRng>,
    jitter_seed: Option<u64>,
    hooks: GameHooks<'_>,
) -> GameOutcome {
    let GameHooks { mut trace, mut cross, end_zones } = hooks;
    let mut final_zones: Option<Vec<String>> = None;
    let mut last_snap: Option<(u32, crate::game::types::TurnStep)> = None;
    // `CRAB_MAX_ACTIONS=<n>` lowers the action cap for a diagnostic run
    // (pair with `CRAB_CAP_DIAG=1` to name the board). Read once.
    let max_actions = max_actions_override().unwrap_or(max_actions);
    // Installed for this game and cleared after, so a seeded game can't
    // leak its stream into the worker's next one. See `bot::set_jitter_seed`.
    crate::server::bot::set_jitter_seed(jitter_seed);
    let mut g = template.clone();
    // Profile settings the engine reads off the seat, not the bot.
    for (seat, pilot) in pilots.into_iter().enumerate() {
        if let Some(w) = pilot.weights() {
            g.players[seat].smart_tap = w.smart_tap;
            g.players[seat].converge_rarest = w.converge_rarest;
        }
        // `hostile_player_targets` reaches search seats too (MCTS builds its
        // candidates with the same picker, so it would self-target without
        // it). `smart_tap`/`converge_rarest` stay Scored-only: search
        // baselines were measured without them.
        g.players[seat].hostile_player_targets = match pilot {
            Pilot::Scored(w) | Pilot::Policy(w) => w.hostile_player_targets,
            Pilot::Mcts(cfg) => cfg.weights.hostile_player_targets,
            Pilot::Uniform => false,
        };
        g.players[seat].duel_sinks = match pilot {
            Pilot::Scored(w) | Pilot::Policy(w) => w.duel_sinks,
            Pilot::Mcts(cfg) => cfg.weights.duel_sinks,
            _ => false,
        };
        g.players[seat].tap_timing = match pilot {
            Pilot::Scored(w) | Pilot::Policy(w) => w.tap_timing,
            Pilot::Mcts(cfg) => cfg.weights.tap_timing,
            _ => false,
        };
        g.players[seat].color_fixing = match pilot {
            Pilot::Scored(w) | Pilot::Policy(w) => w.color_fixing,
            Pilot::Mcts(cfg) => cfg.weights.color_fixing,
            _ => false,
        };
    }
    let mut seeded = shuffle_rng;
    for seat in 0..2 {
        match &mut seeded {
            Some(r) => g.players[seat].library.shuffle(*r),
            None => g.players[seat].library.shuffle(&mut rand::rng()),
        }
    }
    // A seeded deal implies a seeded game: the state's own rng (mulligans,
    // random discards) is reseeded off the shuffle stream. Otherwise a
    // mulligan re-randomizes the deal and a pair's two games diverge.
    if let Some(r) = &mut seeded {
        g.rng.reseed(r.random());
    }
    let aivat = aivat_on();
    let mut luck = 0.0f64;
    let score_draws = |luck: &mut f64| {
        let events = crate::game::draw_log_take();
        if !events.is_empty() {
            *luck += crate::server::aivat::seat0_luck(&events, crate::server::net_eval::SLOT_BEST);
        }
    };
    if aivat {
        crate::game::draw_log_start();
    }
    g.start_mulligan_phase();
    if aivat {
        score_draws(&mut luck);
    }
    let mut bots: Vec<Box<dyn Bot>> = pilots.into_iter().map(Pilot::build).collect();
    if let Some(rate) = crate::pod::fuzz::ladder_fuzz_rate() {
        let seed = jitter_seed.unwrap_or(0) ^ g.rng.position();
        bots = bots
            .into_iter()
            .enumerate()
            .map(|(i, b)| crate::pod::fuzz::FuzzBot::wrap(b, rate, seed ^ 0xF0CC_0000 ^ i as u64))
            .collect();
    }
    let (mut actions, mut stale) = (0usize, 0usize);
    // `CRAB_PROGRESS_WATCH=<turn>` — see `progress_watch_line`. Read once a
    // game, not per action.
    let (watch_from, mut watched_turn) = (progress_watch_from(), u32::MAX);
    while stop_reason(&g, actions, max_actions, stale).is_none() {
        if let Some(from) = watch_from
            && g.turn_number >= from
            && g.turn_number != watched_turn
        {
            watched_turn = g.turn_number;
            eprintln!("{}", progress_watch_line(&g));
        }
        let mut any = false;
        for (s, bot) in bots.iter_mut().enumerate() {
            if offer_check() {
                offer_probe(&g);
            }
            // Cross-binary: poll only our seat and publish its choice; the
            // other seat's action arrives on the link with the peer's state
            // digest. It is always `plain` (settled states never cross the
            // wire), which `accept_on` documents as equivalent.
            let step = match &mut cross {
                Some(cx) => {
                    let pre = crate::crossplay::state_digest(&g);
                    let step = if cx.seat == s {
                        let step = bot.next_action_settled(&g, s);
                        cx.publish(step.as_ref().map(|b| &b.action), pre);
                        step
                    } else {
                        cx.receive(pre).map(crate::server::bot::BotStep::plain)
                    };
                    if cx.fault.is_some() {
                        break;
                    }
                    step
                }
                None => bot.next_action_settled(&g, s),
            };
            let Some(step) = step else { continue };
            let crate::server::bot::BotStep { action, settled } = step;
            let settled = if aivat { None } else { settled };
            // Formatted before `perform_action` consumes the action.
            let pending = trace.as_ref().map(|_| format!("p{s} {action:?}"));
            // Footer zones before this move: a game-ending move clears the
            // loser's zones.
            let zones_before = end_zones.as_ref().map(|_| zone_lines(&g));
            // The dump's board snapshot, once per (turn, step). Dump mode
            // only, so the golden traces keep their format.
            if end_zones.is_some()
                && let Some(t) = trace.as_mut()
                && last_snap != Some((g.turn_number, g.step))
            {
                last_snap = Some((g.turn_number, g.step));
                t.push(board_snapshot(&g));
            }
            // A `settled` state is the finalist's dry run — what
            // `perform_action(action)` would produce (`GameState::accept_on`).
            // This driver owns `g`, so adopt it rather than replay the action;
            // events are discarded anyway.
            let ok = if let Some(settled) = settled {
                if settle_check() {
                    settle_diag(&g, &settled, &format!("p{s} {action:?}"), action.clone());
                }
                g = *settled;
                true
            } else {
                // Events are discarded; hand the buffer back for its capacity
                // (`GameState::recycle_events`, PERF `(-80)`).
                let before = settle_check().then(|| (settle_fingerprint(&g), format!("p{s} {action:?}")));
                // The pre-action board, for the rejected-answer report.
                let snap = settle_check().then(|| g.clone());
                if aivat {
                    crate::game::draw_log_start();
                }
                let res = g.perform_action(action);
                if aivat {
                    score_draws(&mut luck);
                }
                match res {
                    Ok(events) => {
                        if let Some((b, label)) = before
                            && (b == settle_fingerprint(&g)
                                || settle_match(&label))
                        {
                            eprintln!(
                                "settle-check: accepted {label}\n  before {b}\n  after  {}\n  events {}",
                                settle_fingerprint(&g),
                                events.len()
                            );
                        }
                        g.recycle_events(events);
                        true
                    }
                    Err(e) => {
                        if settle_check()
                            && let Some((_, label)) = &before
                            && label.contains("SubmitDecision")
                            && let Some(state) = &snap
                        {
                            eprintln!(
                                "settle-check: REJECTED {label}: {e:?}\n  pending {:?}",
                                state.pending_decision.as_ref().map(|d| format!("{:?}", d.decision))
                            );
                        }
                        false
                    }
                }
            };
            if ok {
                // `find_card_anywhere` is only order-independent while an id
                // lives in one zone. Checked in debug traced games only: the
                // golden traces surface a zone move that forgot to remove,
                // without taxing every simulated game.
                #[cfg(debug_assertions)]
                if trace.is_some()
                    && let Some(dup) = g.duplicate_zone_id()
                {
                    panic!("card {dup:?} is in two zones after action {}", actions + 1);
                }
                if let (Some(t), Some(p)) = (trace.as_mut(), pending) {
                    let p = if trace_names() { named_ids(&g, &p) } else { p };
                    t.push(format!("{:>5}. {p}  ~~  {}", t.len() + 1, trace_state(&g)));
                }
                any = true;
                actions += 1;
                if g.is_game_over() {
                    final_zones = zones_before;
                    break;
                }
            }
        }
        // A fault voids the run: the builds disagree about an action, so
        // nothing after it is a comparison. Stop and let the caller report.
        if cross.as_ref().is_some_and(|cx| cx.fault.is_some()) {
            break;
        }
        if any { stale = 0 } else { stale += 1 }
        if stale == STALE_ROUNDS && stuck_diag() {
            report_stuck(&g, &mut bots);
        }
    }
    crate::server::bot::set_jitter_seed(None);
    // The loop's own order: `is_game_over` wins over the caps, and a game
    // that hits a cap on the same iteration it goes stale is reported as
    // capped. A cross-play fault breaks out early with no reason of its own.
    let stop = stop_reason(&g, actions, max_actions, stale).unwrap_or(StopReason::NoLegalMove);
    // `CRAB_CAP_DIAG=1` reports capped games; `CRAB_CAP_DIAG=<n>` reports
    // every game past `n` actions, capped or not — the form that finds a slow
    // game that still decided. Read once (`getenv` per game is PERF `(-169)`'s
    // class).
    if let Some(floor) = cap_diag_floor() {
        let floor = floor.unwrap_or(max_actions);
        if actions >= floor || matches!(stop, StopReason::ActionCap | StopReason::BoardCap) {
            eprintln!("{}", cap_diagnosis(&g, actions));
        }
    }
    if let Some(out) = end_zones {
        *out = final_zones.unwrap_or_else(|| zone_lines(&g));
    }
    GameOutcome { winner: g.game_over.flatten(), actions, turns: g.turn_number, stop, luck }
}

/// `# snap T{turn} {step} active pN || pK life L lands N/U hand […] crt […]`:
/// each seat's life, lands (total / untapped), hand as `name:mv` (`*` when
/// payable now), and creatures as `id=name P/T` plus flags T(apped) S(ick)
/// A(ttacking) B(locking). The `CRAB_DUMP_TRACES` board line.
fn board_snapshot(g: &GameState) -> String {
    let mut out = format!("# snap T{} {:?} active p{}", g.turn_number, g.step, g.active_player_idx);
    for (i, p) in g.players.iter().enumerate() {
        let lands: Vec<_> = g.battlefield.iter().filter(|c| c.controller == i && c.definition.is_land()).collect();
        let untapped = lands.iter().filter(|c| !c.tapped).count();
        // `*`: the mana cost is payable from what `i` has right now
        // (colours included; targets and timing are not checked).
        let hand: Vec<String> = p
            .hand
            .iter()
            .map(|c| {
                let payable = !c.definition.is_land() && g.could_pay_cost(i, &c.definition.cost);
                format!("{}:{}{}", c.definition.name, c.definition.cost.cmc(), if payable { "*" } else { "" })
            })
            .collect();
        let crt: Vec<String> = g
            .battlefield
            .iter()
            .filter(|c| c.controller == i && g.computed_is_creature(c))
            .map(|c| {
                let (pw, tg) = g.computed_permanent(c.id).map_or((0, 0), |cp| (cp.power, cp.toughness));
                let mut flags = String::new();
                if c.tapped {
                    flags.push('T');
                }
                if c.summoning_sick {
                    flags.push('S');
                }
                if g.attacking.iter().any(|a| a.attacker == c.id) {
                    flags.push('A');
                }
                if g.block_map.contains_key(&c.id) {
                    flags.push('B');
                }
                // Combat keywords as one code: F flying, R reach, D deathtouch,
                // 1 first strike, 2 double strike, M menace, P trample,
                // V vigilance, X defender, I indestructible, L lifelink,
                // U unblockable.
                let kw = g.computed_permanent(c.id).map_or(String::new(), |cp| {
                    use crate::card::{Keyword as K, KeywordSlice};
                    let k = cp.keywords();
                    [
                        (K::Flying, 'F'),
                        (K::Reach, 'R'),
                        (K::Deathtouch, 'D'),
                        (K::FirstStrike, '1'),
                        (K::DoubleStrike, '2'),
                        (K::Menace, 'M'),
                        (K::Trample, 'P'),
                        (K::Vigilance, 'V'),
                        (K::Defender, 'X'),
                        (K::Indestructible, 'I'),
                        (K::Lifelink, 'L'),
                        (K::Unblockable, 'U'),
                    ]
                    .iter()
                    .filter(|(kk, _)| k.has_kw(kk))
                    .map(|(_, ch)| *ch)
                    .collect()
                });
                format!(
                    "{}={} {pw}/{tg}{}{flags}{}",
                    c.id.0,
                    c.definition.name,
                    if flags.is_empty() { "" } else { " " },
                    if kw.is_empty() { String::new() } else { format!(" k:{kw}") }
                )
            })
            .collect();
        out += &format!(
            " || p{i} life {} lands {}/{untapped} hand [{}] crt [{}]",
            p.life,
            lands.len(),
            hand.join("; "),
            crt.join("; ")
        );
    }
    out
}

/// `# pN hand|library|graveyard: id=name, …` for every seat — the dump's
/// header (template) and footer (game end).
fn zone_lines(g: &GameState) -> Vec<String> {
    let list = |cards: &mut dyn Iterator<Item = &crate::card::CardInstance>| {
        cards.map(|c| format!("{}={}", c.id.0, c.definition.name)).collect::<Vec<_>>().join(", ")
    };
    let mut out = Vec::new();
    for (i, p) in g.players.iter().enumerate() {
        out.push(format!("# p{i} hand: {}", list(&mut p.hand.iter())));
        out.push(format!("# p{i} library: {}", list(&mut p.library.iter())));
        out.push(format!("# p{i} graveyard: {}", list(&mut p.graveyard.iter())));
    }
    out
}

/// `CRAB_MAX_ACTIONS=<n>`: a lower action cap for a diagnostic run. Read once.
fn max_actions_override() -> Option<usize> {
    static CAP: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    *CAP.get_or_init(|| {
        std::env::var_os("CRAB_MAX_ACTIONS")?.to_str()?.parse::<usize>().ok().filter(|n| *n > 0)
    })
}

/// `CRAB_CAP_DIAG=1` — print [`cap_diagnosis`] for every capped game;
/// `CRAB_CAP_DIAG=<n>` — for any game past `n` actions. Outer `Some` = set,
/// inner `Some` = the floor (when it parses above 1). Read once, so the
/// throughput path pays nothing (PERF `(-169)`). A runaway names itself in
/// one of the tallies: an unbounded stack is a trigger or activation loop,
/// an unbounded battlefield a token loop.
pub(crate) fn cap_diag_floor() -> Option<Option<usize>> {
    static FLOOR: std::sync::OnceLock<Option<Option<usize>>> = std::sync::OnceLock::new();
    *FLOOR.get_or_init(|| {
        let v = std::env::var_os("CRAB_CAP_DIAG")?;
        Some(v.to_str().and_then(|s| s.parse::<usize>().ok()).filter(|n| *n > 1))
    })
}

/// `CRAB_PROGRESS_WATCH=<turn>` — one line per turn from `<turn>` on, naming
/// every quantity CR 104.4's turn watch digests. `None` off.
fn progress_watch_from() -> Option<u32> {
    static FROM: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *FROM.get_or_init(|| std::env::var_os("CRAB_PROGRESS_WATCH")?.to_str()?.parse::<u32>().ok())
}

/// The turn watch's inputs, spelled out, for a game that will not draw.
///
/// ⚠ `cap_diagnosis` says the watch did not fire; this says why — diff two
/// consecutive lines and the drifting quantity names itself.
///
/// Printed from the driver, not the watch, because the watch also runs on
/// every bot probe's clone; here `g` is the real game. A view of
/// [`GameState::fingerprint_as`]'s field list, not a copy of its hash: the
/// two should agree on what is watched.
fn progress_watch_line(g: &GameState) -> String {
    use std::fmt::Write;
    let mut s = format!("progress: turn {}", g.turn_number);
    for (seat, p) in g.players.iter().enumerate() {
        let _ = write!(
            s,
            " | p{seat} life {} h{} l{} g{} poison{} pool{}/{} e{} x{} room{}",
            p.life,
            p.hand.len(),
            p.library.len(),
            p.graveyard.len(),
            p.poison_counters,
            p.mana_pool.total(),
            p.mana_pool.colorless_amount(),
            p.energy,
            p.experience,
            p.dungeon.as_ref().map_or(0, |(_, room)| *room as u32 + 1),
        );
    }
    // Order-sensitive digests so a huge board fits on a line: `ids` moves
    // when a permanent enters or leaves, `tapped` when tapping differs.
    let hash = |it: &mut dyn Iterator<Item = u64>| {
        it.fold(0u64, |a, v| {
            let mut z = a.wrapping_add(v).wrapping_add(0x9E37_79B9_7F4A_7C15);
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z ^ (z >> 31)
        })
    };
    let ids = hash(&mut g.battlefield.iter().map(|c| u64::from(c.id.0)));
    let tapped = hash(&mut g.battlefield.iter().map(|c| u64::from(c.tapped)));
    let _ = write!(
        s,
        " | exile {} bf {} untapped {} dmg {} ctr {} ids {ids:08x} tapped {tapped:08x}",
        g.exile.len(),
        g.battlefield.len(),
        g.battlefield.iter().filter(|c| !c.tapped).count(),
        g.battlefield.iter().map(|c| u64::from(c.damage)).sum::<u64>(),
        g.battlefield.iter().map(|c| c.counter_sum()).fold(0u32, u32::saturating_add),
    );
    s
}

/// What a capped game did, as opposed to what its board looks like now:
/// a tally of performed actions, so a loop whose card is in no zone
/// `cap_diagnosis` samples still names itself.
///
/// Off unless `CRAB_CAP_DIAG` is set; callers hoist [`Self::armed`] out of
/// the loop so a live run pays one `bool` test per action.
#[derive(Default)]
pub struct ActionCensus {
    on: bool,
    counts: HashMap<String, usize>,
    /// The same actions keyed by card alone: which cards a run never played.
    cards: HashMap<String, usize>,
    /// Printed activated (`false`) and loyalty (`true`) abilities by card and
    /// index: which abilities of a played card no seat ever activated.
    abilities: HashMap<(String, bool, usize), usize>,
    /// Cards seen as the source of a trigger on the stack: a deck card with
    /// printed triggers missing here never fired one (e.g. a dead kind/scope
    /// pair, as Tahngarth, First Mate's "whenever an opponent attacks" was).
    triggered: HashMap<String, usize>,
    /// The same by printed trigger index (matched by effect): a card with one
    /// live trigger and one dead one reads "triggered" above.
    triggered_idx: HashMap<(String, usize), usize>,
}

/// An action about to be taken, with the card it names peeled off so
/// [`ActionCensus::bump`] can count both. Built only when the census is on.
pub struct CensusKey {
    line: String,
    card: Option<String>,
    ability: Option<(bool, usize)>,
}

impl ActionCensus {
    /// One `OnceLock` read, at the top of a game loop rather than inside it.
    pub fn armed() -> Self {
        Self { on: cap_diag_floor().is_some(), ..Default::default() }
    }

    /// On regardless of `CRAB_CAP_DIAG` — the deck-coverage census, which
    /// wants every game counted and not just the capped ones.
    pub fn forced() -> Self {
        Self { on: true, ..Default::default() }
    }

    /// The `n` most frequent action lines, most frequent first (ties by
    /// name, so the listing is deterministic).
    pub fn top_actions(&self, n: usize) -> Vec<(&str, usize)> {
        let mut v: Vec<(&str, usize)> = self.counts.iter().map(|(k, c)| (k.as_str(), *c)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        v.truncate(n);
        v
    }

    /// How often each card was named by an accepted action; a deck card
    /// missing here was never cast, played or activated. Read it by key
    /// (walking a deck list), never by iteration — a report must not inherit
    /// hash order.
    pub fn card_counts(&self) -> &HashMap<String, usize> {
        &self.cards
    }

    /// How often each printed ability was activated, keyed by card name,
    /// loyalty-ness and index. Read by key, as [`Self::card_counts`].
    pub fn ability_counts(&self) -> &HashMap<(String, bool, usize), usize> {
        &self.abilities
    }

    /// How often each card was seen as a trigger's source. Read by key.
    /// [`trigger_counts`](Self::trigger_counts) by printed trigger index.
    pub fn trigger_index_counts(&self) -> &HashMap<(String, usize), usize> {
        &self.triggered_idx
    }

    pub fn trigger_counts(&self) -> &HashMap<String, usize> {
        &self.triggered
    }

    /// Note the source of every triggered ability now on `g`'s stack.
    pub fn note_triggers(&mut self, g: &GameState) {
        if !self.on {
            return;
        }
        for si in g.stack.iter() {
            if let crate::game::types::StackItem::Trigger { source, effect, .. } = si
                && let Some(c) = g.find_card_anywhere(*source)
            {
                *self.triggered.entry(c.definition.name.to_string()).or_insert(0) += 1;
                if let Some(i) = c.definition.triggered_abilities.iter().position(|t| t.effect == **effect) {
                    *self.triggered_idx.entry((c.definition.name.to_string(), i)).or_insert(0) += 1;
                }
            }
        }
    }

    /// Fold another game's census in, so a run can total across games.
    pub fn merge(&mut self, other: &Self) {
        for (k, n) in &other.counts {
            *self.counts.entry(k.clone()).or_insert(0) += n;
        }
        for (k, n) in &other.cards {
            *self.cards.entry(k.clone()).or_insert(0) += n;
        }
        for (k, n) in &other.abilities {
            *self.abilities.entry(k.clone()).or_insert(0) += n;
        }
        for (k, n) in &other.triggered {
            *self.triggered.entry(k.clone()).or_insert(0) += n;
        }
        for (k, n) in &other.triggered_idx {
            *self.triggered_idx.entry(k.clone()).or_insert(0) += n;
        }
    }

    /// The key for an action *about to be taken*, or `None` when off. Taken
    /// before the action so a cast names the card in the zone it left.
    ///
    /// `seat` is in the key so a line can't be misread against another
    /// seat's board.
    pub fn key_for(
        &self,
        g: &GameState,
        seat: usize,
        a: &crate::game::GameAction,
    ) -> Option<CensusKey> {
        self.on.then(|| {
            let (line, card) = Self::key(g, a);
            use crate::game::GameAction as A;
            let ability = match a {
                A::ActivateAbility { ability_index, .. } | A::ActivateAbilityWaterbend { ability_index, .. } => {
                    Some((false, *ability_index))
                }
                A::ActivateLoyaltyAbility { ability_index, .. } => Some((true, *ability_index)),
                _ => None,
            };
            CensusKey { line: format!("p{seat} {line}"), card, ability }
        })
    }

    /// Count a key from [`Self::key_for`], once the action is known to have
    /// been accepted.
    pub fn bump(&mut self, key: Option<CensusKey>) {
        if let Some(CensusKey { line, card, ability }) = key {
            *self.counts.entry(line).or_insert(0) += 1;
            if let Some(c) = card {
                if let Some((loyalty, i)) = ability {
                    *self.abilities.entry((c.clone(), loyalty, i)).or_insert(0) += 1;
                }
                *self.cards.entry(c).or_insert(0) += 1;
            }
        }
    }

    /// `Variant Card` — the variant name plus the card the action names, and
    /// the ability index where there is one (two abilities on one permanent
    /// are two different loops).
    fn key(g: &GameState, a: &crate::game::GameAction) -> (String, Option<String>) {
        use crate::game::GameAction as A;
        let dbg = format!("{a:?}");
        let variant = dbg.split(['{', '(', ' ']).next().unwrap_or("?");
        let (id, idx) = match a {
            A::ActivateAbility { card_id, ability_index, .. }
            | A::ActivateAbilityWaterbend { card_id, ability_index, .. }
            | A::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                (Some(*card_id), Some(*ability_index))
            }
            A::PlayLand(id) | A::PlayLandBack(id) | A::PlayLandFromGraveyard(id) => {
                (Some(*id), None)
            }
            // Suspending or foretelling a card plays it: its later cast is not
            // an action (a suspend-only Wheel of Fate read as never played).
            A::Suspend { card_id } | A::Foretell { card_id } => (Some(*card_id), None),
            _ => (a.cast_card_id(), None),
        };
        // An answer names the ask it answers and the card asking, so a loop
        // of asks is traceable to its source. Not a play: no card census.
        if let A::SubmitDecision(_) = a
            && let Some(p) = g.pending_decision.as_ref()
        {
            let d = format!("{:?}", p.decision);
            let kind = d.split(['{', '(', ' ']).next().unwrap_or("?");
            // Diagnostic only: the asking card is the Debug's `source` field.
            let src = d
                .split_once("source: CardId(")
                .and_then(|(_, r)| r.split(')').next()?.parse().ok())
                .and_then(|n| g.find_card_anywhere(crate::card::CardId(n)))
                .map_or("?", |c| c.definition.name);
            let named = d.split_once("source_name: \"").and_then(|(_, r)| r.split('"').next()).unwrap_or("");
            let desc = d.split_once("description: \"").and_then(|(_, r)| r.split('"').next()).unwrap_or("");
            // The first legal pick names the loop's other half.
            let first = d
                .split_once("legal: [Permanent(CardId(")
                .and_then(|(_, r)| r.split(')').next()?.parse().ok())
                .and_then(|n| g.find_card_anywhere(crate::card::CardId(n)))
                .map_or("", |c| c.definition.name);
            let picked = match a {
                A::SubmitDecision(crate::decision::DecisionAnswer::Target(crate::game::types::Target::Permanent(id))) => {
                    g.find_card_anywhere(*id).map_or("", |c| c.definition.name)
                }
                _ => "",
            };
            return (format!("{variant} {kind} {src} {named} {desc} {first} -> {picked}"), None);
        }
        let Some(id) = id else { return (variant.to_string(), None) };
        // A card the action names but no zone holds any more is an id, not a
        // name, and must not land in the card census as one.
        let found = g.find_card_anywhere(id).map(|c| c.definition.name.to_string());
        let name = found.clone().unwrap_or_else(|| format!("{id:?}"));
        // The ability's effect, so a granted ability (Trazyn's graveyard
        // artifacts, an anthem's grant) names what it is, not only its index.
        let effect = idx.and_then(|i| {
            let c = g.battlefield_find(id)?;
            let printed = &c.definition.activated_abilities;
            let ab = match printed.get(i) {
                Some(ab) => ab,
                None => *g.granted_abilities_of(c, &g.grant_scan()).get(i - printed.len())?,
            };
            Some(format!(
                "{}{}{}{}",
                crate::server::view::ability_effect_label(&ab.effect),
                if ab.tap_cost { ", {T}" } else { "" },
                if ab.sac_cost { ", sac self" } else { "" },
                if ab.sac_other_filter.is_some() { ", sac other" } else { "" },
            ))
        });
        let line = match (idx, effect) {
            (Some(i), Some(e)) => format!("{variant}#{i} {name} [{e}]"),
            (Some(i), None) => format!("{variant}#{i} {name}"),
            (None, _) => format!("{variant} {name}"),
        };
        (line, found)
    }

    /// The twelve commonest actions, count-descending then name-ascending so
    /// two runs of one seed print the same line.
    pub fn render(&self) -> String {
        let mut rows: Vec<(&String, &usize)> = self.counts.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        rows.truncate(12);
        rows.iter().map(|(n, c)| format!("{n} x{c}")).collect::<Vec<_>>().join(", ")
    }
}

pub(crate) fn cap_diagnosis(g: &GameState, actions: usize) -> String {
    use crate::game::StackItem;
    use std::fmt::Write;

    fn tally_all(names: impl Iterator<Item = String>) -> Vec<(String, usize)> {
        let mut out: Vec<(String, usize)> = Vec::new();
        for n in names {
            match out.iter_mut().find(|(k, _)| *k == n) {
                Some((_, c)) => *c += 1,
                None => out.push((n, 1)),
            }
        }
        out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        out
    }
    fn tally(names: impl Iterator<Item = String>) -> Vec<(String, usize)> {
        let mut out = tally_all(names);
        out.truncate(12);
        out
    }
    fn render(rows: &[(String, usize)]) -> String {
        rows.iter().map(|(n, c)| format!("{n} x{c}")).collect::<Vec<_>>().join(", ")
    }

    let mut s = String::new();
    let _ = write!(
        s,
        "cap: {actions} actions, turn {}, stack {}",
        g.turn_number,
        g.stack.len()
    );
    // ⚠ The one known shape, named so it isn't re-diagnosed: a seat near
    // `i32::MAX` life (Beacon of Immortality doubling; `cube` 1018, 1069,
    // 1076, via `CRAB_LIFE_WATCH`) can't be killed, so the cap is the
    // backstop working. ENGINE_BACKLOG's OPEN section has it (adjudication,
    // not a rules defect); `fresh_seed_sweep.sh` counts these separately.
    if g.players.iter().any(|p| p.life > crate::player::SCALE_CEILING.saturating_mul(1_000)) {
        let _ = write!(s, "  [SATURATED LIFE — the known unwinnable board, not a new defect]");
    }
    // ⚠ The watch's own state: a late cap is first a question about it.
    // CR 104.4's turn watch draws a game whose fingerprint returns to one
    // anchor `NO_PROGRESS_DRAW_REPEATS` times; `repeats` is how close it got,
    // `since` how long it has been away. `repeats 0 / since 0` on a very late
    // board means the fingerprint moves every sample (the loop is outside
    // what the watch watches), not that the budget ran out early.
    let (anchor, repeats, since) = g.no_progress_watch;
    let _ = write!(
        s,
        "\n  no-progress watch: anchor {}, repeats {repeats}/{}, since {since}/{}",
        if anchor == 0 { "unset" } else { "set" },
        GameState::NO_PROGRESS_DRAW_REPEATS,
        GameState::NO_PROGRESS_MAX_PERIOD,
    );
    for (seat, p) in g.players.iter().enumerate() {
        // Pool and untapped count show whether a repeated activation was
        // paid for (an unbounded `{cost}` loop means an unbounded mana source).
        let untapped = g
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && !c.tapped)
            .count();
        let _ = write!(
            s,
            "\n  p{seat}: life {} bf {} ({untapped} untapped) hand {} gy {} lib {} pool {}{}{}{}",
            p.life,
            g.battlefield.iter().filter(|c| c.controller == seat).count(),
            p.hand.len(),
            p.graveyard.len(),
            p.library.len(),
            p.mana_pool.total() + p.mana_pool.restricted_total(),
            // An empty library that is NOT armed for the CR 104.3c loss is the
            // interesting half: it says the seat has not attempted a draw.
            if p.pending_deck_loss { " [deck-loss armed]" } else { "" },
            if p.skip_next_draw_step > 0 { " [draw steps skipped]" } else { "" },
            // A draw is every remaining seat losing at once (CR 104.4a); the
            // causes say whether that was one event or a loop.
            p.loss_cause.map(|c| format!(" [lost: {c:?}]")).unwrap_or_default(),
        );
    }
    let name_of = |id: crate::card::CardId| {
        g.find_card_anywhere(id)
            .map(|c| c.definition.name.to_string())
            .unwrap_or_else(|| format!("{id:?}"))
    };
    // The target rides along: a trigger that keeps re-targeting the same
    // permanent names the loop (the two-Portable-Hole oscillation).
    let target_of = |t: &Option<crate::game::Target>| match t {
        Some(crate::game::Target::Permanent(id)) => format!(" -> {}", name_of(*id)),
        Some(crate::game::Target::Player(p)) => format!(" -> p{p}"),
        None => String::new(),
    };
    let on_stack = tally(g.stack.iter().map(|it| match it {
        StackItem::Spell { card, target, .. } => {
            format!("spell {}{}", card.definition.name, target_of(target))
        }
        StackItem::Trigger { source, activated, target, .. } => format!(
            "{} {}{}",
            if *activated { "ability" } else { "trigger" },
            name_of(*source),
            target_of(target),
        ),
    }));
    if !on_stack.is_empty() {
        let _ = write!(s, "\n  stack: {}", render(&on_stack));
    }
    // ⚠ Per seat and untruncated: the action census names a seat, so the
    // board must answer for that seat. A truncated whole-board tally once hid
    // the looping Eldrazi Spawn behind higher-count rows.
    for (seat, _) in g.players.iter().enumerate() {
        let board = tally_all(
            g.battlefield
                .iter()
                .filter(|c| c.controller == seat)
                .map(|c| c.definition.name.to_string()),
        );
        if !board.is_empty() {
            let _ = write!(s, "\n  p{seat} board: {}", render(&board));
        }
    }
    // Linked exiles ("until ~ leaves") name their source: a card whose link
    // points at a permanent that is itself linked back is a return loop.
    let exiled = tally(g.exile.iter().map(|c| match &c.exiled_by {
        Some(l) => format!("{} (until {} leaves)", c.definition.name, name_of(l.source)),
        None => c.definition.name.to_string(),
    }));
    if !exiled.is_empty() {
        let _ = write!(s, "\n  exile: {}", render(&exiled));
    }
    // ⚠ A loop whose card is off the battlefield at the sample point is
    // invisible above (`all` 1274 cycled one card hand -> battlefield ->
    // library every turn), so hands and libraries are listed too, with the
    // same 12-row `tally` cap.
    for (seat, p) in g.players.iter().enumerate() {
        let hand = tally(p.hand.iter().map(|c| c.definition.name.to_string()));
        if !hand.is_empty() {
            let _ = write!(s, "\n  p{seat} hand: {}", render(&hand));
        }
        let lib = tally(p.library.iter().map(|c| c.definition.name.to_string()));
        if !lib.is_empty() {
            let _ = write!(s, "\n  p{seat} lib: {}", render(&lib));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    /// A trigger stack past `MAX_STACK_ITEMS` is a `BoardCap`, whatever its
    /// play count (Dina + Exquisite Blood against a seat that can't lose
    /// passed for hours, seed 4500524).
    #[test]
    fn a_runaway_trigger_stack_is_a_board_cap() {
        let mut g = crate::game::multi_player_game(4);
        let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..MAX_STACK_ITEMS - 1 {
            g.push_stack(crate::game::types::TriggerPush::new(src, 0, crate::effect::Effect::Noop).build());
        }
        assert_eq!(stop_reason(&g, 0, 1_000, 0), None);
        g.push_stack(crate::game::types::TriggerPush::new(src, 0, crate::effect::Effect::Noop).build());
        assert_eq!(stop_reason(&g, 0, 1_000, 0), Some(StopReason::BoardCap));
    }

    /// Every pair is two games, and only fully decided pairs are scored;
    /// the unpaired totals still count every game played.
    #[test]
    fn pairs_play_two_games_each_and_score_only_decided_pairs() {
        let deck = deck_of(&[
            (catalog::mountain as CardFactory, 17),
            (catalog::lightning_bolt, 4),
            (catalog::goblin_guide, 4),
            (catalog::gray_ogre, 8),
            (catalog::hill_giant, 7),
        ]);
        const PAIRS: usize = 6;
        let t = simulate_match_pairs_piloted(
            &deck,
            &deck,
            PAIRS,
            [Pilot::default(), Pilot::default()],
            50_000,
            99,
        );
        assert_eq!(
            t.wins_a + t.wins_b + t.undecided,
            2 * PAIRS as u32,
            "each pair must play exactly two games"
        );
        assert!(t.pairs.len() <= PAIRS, "a pair cannot be scored more than once");
        assert!(t.pairs.iter().all(|&s| (-1..=1).contains(&s)));
        // Every game of a scored pair was decided, so scored pairs can
        // never claim more games than the run actually decided.
        assert!(t.pairs.len() as u32 * 2 <= t.wins_a + t.wins_b);
    }

    /// A pair's two games really are the same deal: a profile against itself
    /// splits most pairs, since the deal-favoured side wins in each
    /// direction. Not 100%: unseeded tie jitter (`main_phase_action_with`)
    /// can still diverge a pair, which is why the ladder measures rho.
    #[test]
    fn self_mirror_pairs_mostly_split() {
        let deck = deck_of(&[
            (catalog::mountain as CardFactory, 17),
            (catalog::lightning_bolt, 4),
            (catalog::goblin_guide, 4),
            (catalog::gray_ogre, 8),
            (catalog::hill_giant, 7),
        ]);
        const PAIRS: usize = 10;
        let t = simulate_match_pairs_piloted(
            &deck,
            &deck,
            PAIRS,
            [Pilot::default(), Pilot::default()],
            50_000,
            7,
        );
        let splits = t.pairs.iter().filter(|&&s| s == 0).count();
        assert!(
            splits * 2 > t.pairs.len(),
            "expected most of {} scored pairs to split, got {splits}",
            t.pairs.len()
        );
    }

    fn deck_of(spec: &[(CardFactory, usize)]) -> Vec<CardFactory> {
        let mut d = Vec::new();
        for &(f, n) in spec {
            for _ in 0..n {
                d.push(f);
            }
        }
        d
    }

    /// A W/R-heavy pool with one splash-worthy green bomb.
    fn wr_pool_with_green_bomb() -> Vec<CardFactory> {
        let mut p: Vec<CardFactory> = Vec::new();
        let mut push = |f: CardFactory, n: usize| {
            for _ in 0..n {
                p.push(f);
            }
        };
        push(catalog::lightning_bolt, 4);
        push(catalog::goblin_guide, 4);
        push(catalog::gray_ogre, 3);
        push(catalog::hill_giant, 3);
        push(catalog::white_knight, 4);
        push(catalog::benalish_hero, 3);
        push(catalog::serra_angel, 3);
        push(catalog::ambush_viper, 1); // {1}{G} — splashable off a source or two
        push(catalog::craw_wurm, 1); // {4}{G}{G} — NOT splashable, see below
        p
    }

    #[test]
    fn enumerate_ranks_the_dominant_pair_first() {
        let cfg = SimConfig::default();
        let candidates = enumerate_candidates(&wr_pool_with_green_bomb(), &cfg);
        assert!(!candidates.is_empty());
        let top = &candidates[0];
        assert!(
            top.colors.contains(&Color::White) && top.colors.contains(&Color::Red),
            "top build uses the pool's two deep colors, got {}",
            top.label,
        );
        // Dedup: no two candidates share an identical main deck.
        let mut keys: Vec<Vec<usize>> = candidates
            .iter()
            .map(|c| {
                let mut k: Vec<usize> = c.main.iter().map(|&f| f as usize).collect();
                k.sort_unstable();
                k
            })
            .collect();
        keys.sort();
        let before = keys.len();
        keys.dedup();
        assert_eq!(before, keys.len(), "candidate mains are deduplicated");
    }

    /// A pool spread evenly over three colors (too thin for any pair to
    /// fill 23 spells) surfaces a genuine 3-color candidate — wide shapes
    /// only dedup away when the extra colors contribute nothing.
    #[test]
    fn thin_tricolor_pool_surfaces_a_three_color_build() {
        let mut p: Vec<CardFactory> = Vec::new();
        let mut push = |f: CardFactory, n: usize| {
            for _ in 0..n {
                p.push(f);
            }
        };
        push(catalog::lightning_bolt, 4);
        push(catalog::goblin_guide, 4);
        push(catalog::white_knight, 4);
        push(catalog::benalish_hero, 4);
        push(catalog::craw_wurm, 4);
        push(catalog::grizzly_bears, 4);
        let cfg = SimConfig::default();
        let candidates = enumerate_candidates(&p, &cfg);
        assert!(
            candidates.iter().any(|c| c.colors.len() >= 3),
            "a 3-color candidate survives dedup in a thin tricolor pool",
        );
        // And it should be the static-score leader: every pair leaves 8
        // playables in the sideboard.
        assert_eq!(candidates[0].colors.len(), 3, "tricolor build ranks first, got {}", candidates[0].label);
    }

    #[test]
    fn splash_shape_carries_the_bomb_and_a_source() {
        let cfg = SimConfig::default();
        let candidates = enumerate_candidates(&wr_pool_with_green_bomb(), &cfg);
        let splashy = candidates
            .iter()
            .find(|c| !c.splash.is_empty() && c.splash.contains(&Color::Green))
            .expect("a green-splash candidate exists");
        assert!(
            splashy.main.iter().any(|&f| f as *const () == catalog::ambush_viper as *const ()),
            "the splash candidate actually plays the splashable green card",
        );
        assert!(
            splashy.basics.get(&Color::Green).copied().unwrap_or(0) >= 1,
            "splash color gets at least one basic source",
        );
        // …and the double-green six-drop is left out: a splash supports
        // {1}{G}, never {4}{G}{G}.
        assert!(
            !splashy.main.iter().any(|&f| f as *const () == catalog::craw_wurm as *const ()),
            "a double-pip card is not splash material",
        );
    }

    #[test]
    fn candidate_deck_is_full_size() {
        let cfg = SimConfig::default();
        let candidates = enumerate_candidates(&wr_pool_with_green_bomb(), &cfg);
        let deck = candidates[0].deck();
        // 22 spells in this pool max out below target_spells (23); deck =
        // spells + 17 basics.
        assert_eq!(
            deck.len(),
            candidates[0].main.len() + cfg.total_lands as usize,
            "deck = main + basics",
        );
    }

    /// Every randomized build is a legal 40-card deck (independent
    /// spell/land sampling used to mint 38-39-card decks that won on
    /// thinner-deck consistency).
    #[test]
    fn random_builds_are_forty_cards() {
        let cfg = SimConfig::default();
        let packs = crate::draft::SosPacks::sos_default();
        for seed in 0..4u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let pulls: Vec<CardFactory> = (0..6).flat_map(|_| packs.roll(&mut rng)).collect();
            let deck = build_random_deck(&pulls, &cfg, &mut rng);
            assert_eq!(deck.cards.len(), 40, "seed {seed}: {} is 40 cards", deck.label);
        }
    }

    /// Fixing classifier: mana-rock bodies and land fetchers qualify,
    /// vanilla creatures don't.
    #[test]
    fn fixing_cards_classified() {
        assert!(is_fixing_card(&catalog::page_loose_leaf()), "taps for {{C}}");
        assert!(
            is_fixing_card(&catalog::environmental_scientist()),
            "ETB basic-land fetch",
        );
        assert!(!is_fixing_card(&catalog::grizzly_bears()));
    }

    /// On-color pool duals occupy land slots ahead of basics; off-color
    /// duals stay in the leftovers.
    #[test]
    fn on_color_duals_replace_basics() {
        let mut p = wr_pool_with_green_bomb();
        p.push(catalog::fields_of_strife); // R/W school land — on-color
        p.push(catalog::fields_of_strife);
        p.push(catalog::forum_of_amity); // W/B — off-color for W/R
        let cfg = SimConfig::default();
        let candidates = enumerate_candidates(&p, &cfg);
        let wr = candidates
            .iter()
            .find(|c| c.splash.is_empty() && c.colors.len() == 2 && c.colors.contains(&Color::Red) && c.colors.contains(&Color::White))
            .expect("a straight W/R build exists");
        assert_eq!(wr.duals.len(), 2, "both Fields of Strife take land slots");
        let basics: u32 = wr.basics.values().sum();
        assert_eq!(basics + 2, cfg.total_lands, "basics shrink to fit");
        assert!(
            wr.leftovers.iter().any(|&f| f as usize == catalog::forum_of_amity as CardFactory as usize),
            "the off-color dual stays in the leftovers",
        );
        assert_eq!(wr.deck().len(), wr.main.len() + cfg.total_lands as usize);
    }

    #[test]
    fn simulate_real_deck_crushes_land_pile() {
        // 23 spells + 17 mountains vs 40 lands: the real deck must win
        // essentially every decided game.
        let mut deck: Vec<CardFactory> = Vec::new();
        for _ in 0..4 { deck.push(catalog::lightning_bolt); }
        for _ in 0..4 { deck.push(catalog::goblin_guide); }
        for _ in 0..4 { deck.push(catalog::gray_ogre); }
        for _ in 0..4 { deck.push(catalog::hill_giant); }
        for _ in 0..17 { deck.push(catalog::mountain); }
        let lands: Vec<CardFactory> = (0..40).map(|_| catalog::forest as CardFactory).collect();
        let tally = simulate_match_games(&deck, &lands, 6, false, false, 50_000, Some(42));
        assert!(tally.wins_a > 0, "real deck wins games");
        assert_eq!(tally.wins_b, 0, "the land pile never wins");
    }

    /// CRN: a fixed seed base reproduces the shuffle streams. Asserted
    /// against a land pile, where only A's draws decide the game, so bot
    /// jitter can't move the tally.
    #[test]
    fn crn_seeded_games_are_reproducible_vs_static_opponent() {
        let mut deck: Vec<CardFactory> = Vec::new();
        for _ in 0..8 { deck.push(catalog::lightning_bolt); }
        for _ in 0..8 { deck.push(catalog::goblin_guide); }
        for _ in 0..17 { deck.push(catalog::mountain); }
        let lands: Vec<CardFactory> = (0..33).map(|_| catalog::forest as CardFactory).collect();
        let a = simulate_match_games(&deck, &lands, 4, false, false, 50_000, Some(7));
        let b = simulate_match_games(&deck, &lands, 4, false, false, 50_000, Some(7));
        assert_eq!(
            (a.wins_a, a.wins_b, a.undecided),
            (b.wins_a, b.wins_b, b.undecided),
            "same CRN seeds → same outcomes against a static opponent",
        );
    }

    /// Curve-weighted basics: pips on cheap spells outweigh the same pip
    /// count on expensive ones — a Bolt-heavy red half wants more sources
    /// than an Angels-only white half even at fewer raw pips.
    #[test]
    fn basic_split_leans_on_cheap_pips() {
        let mut main: Vec<CardFactory> = Vec::new();
        for _ in 0..4 {
            main.push(catalog::lightning_bolt); // {R}, 4 raw R pips, all at cmc 1
        }
        for _ in 0..4 {
            main.push(catalog::serra_angel); // {3}{W}{W}, 8 raw W pips at cmc 5
        }
        let split = basic_split(&main, &[Color::Red, Color::White], 10, false);
        assert!(
            split[&Color::Red] > split[&Color::White],
            "cheap red pips outweigh late white ones, got {split:?}",
        );
        assert_eq!(split.values().sum::<u32>(), 10);
    }

    fn five_bolts_six_angels() -> Vec<CardFactory> {
        let mut main: Vec<CardFactory> = vec![catalog::lightning_bolt; 5];
        for _ in 0..6 {
            main.push(catalog::serra_angel);
        }
        main
    }

    /// The curve term in isolation: a normal spread pays nothing, a
    /// top-heavy pile pays for both its missing early plays and its
    /// excess five-plus spells.
    #[test]
    fn curve_penalty_prices_the_degenerate_shapes() {
        let bolts: Vec<CardFactory> = vec![catalog::lightning_bolt; 6];
        assert_eq!(curve_penalty(&bolts), 0, "six one-drops: early is covered, no top");
        let angels: Vec<CardFactory> = vec![catalog::serra_angel; 23];
        // No early spells (5 × 4) and seventeen five-drops past six (17 × 3).
        assert_eq!(curve_penalty(&angels), 5 * 4 + 17 * 3);
        let mixed = five_bolts_six_angels();
        assert_eq!(curve_penalty(&mixed), 0, "five early + six top is a legal curve");
    }

    /// Under `curve_aggro` the curve term reorders cards of similar quality
    /// toward the cheap end without burying a genuine bomb.
    #[test]
    fn curve_aggro_prefers_the_cheap_end() {
        use crate::draft::{aggro_curve_delta, score_brief_aggro, score_brief_quality};
        let brief = |f: CardFactory| crate::cube::card_brief(f);
        let bolt = brief(catalog::lightning_bolt); // MV 1
        let angel = brief(catalog::serra_angel); // MV 5
        assert!(
            aggro_curve_delta(bolt) > aggro_curve_delta(angel),
            "the cheap card gains where the five-drop loses",
        );
        // A land never takes a spell slot, so it is not curved.
        assert_eq!(aggro_curve_delta(brief(catalog::forest)), 0);

        let colors = colors_of_picks(&[catalog::lightning_bolt, catalog::serra_angel]);
        // The delta is exactly what separates the two scorers.
        for f in [catalog::lightning_bolt, catalog::serra_angel, catalog::forest] {
            let b = brief(f);
            assert_eq!(
                score_brief_aggro(b, &colors),
                score_brief_quality(b, &colors) + aggro_curve_delta(b),
                "aggro = quality + curve delta",
            );
        }
        // It steers without overruling: Serra Angel is still the better card.
        assert!(
            score_brief_aggro(angel, &colors) > score_brief_aggro(bolt, &colors),
            "a real bomb still outranks a one-drop — the term is a steer, not a veto",
        );
    }

    /// A pool of nothing but five-drops and one-drops: with the flag on the
    /// built main deck leans cheaper than with it off.
    #[test]
    fn curve_aggro_builds_a_cheaper_main_deck() {
        // Distinct cards, not copies: under `COPY_CAP` a pool of copies
        // builds the same deck either way.
        let white = |b: &&'static crate::cube::CardBrief| !b.is_land;
        let mut cheap: Vec<CardFactory> = Vec::new();
        let mut dear: Vec<CardFactory> = Vec::new();
        for f in crate::cube::cube_pool_all() {
            let b = crate::cube::card_brief(f);
            if !white(&b) {
                continue;
            }
            if b.cmc <= 2 && cheap.len() < 14 {
                cheap.push(f);
            } else if b.cmc >= 6 && dear.len() < 14 {
                dear.push(f);
            }
        }
        assert!(cheap.len() >= 10 && dear.len() >= 10, "pool has both ends");
        let pool: Vec<CardFactory> = dear.iter().chain(cheap.iter()).copied().collect();
        let mean_cmc = |aggro: bool| -> f64 {
            let scores = PoolScores::new(&pool, true, aggro, false);
            let colors = colors_of_picks(&pool);
            let cols: Vec<Color> = Color::ALL.iter().copied().filter(|c| colors.get(*c) > 0).collect();
            let mut rng = StdRng::seed_from_u64(7);
            let MainDeck { main, .. } =
                suggest_main_deck_shape(&scores, &cols, &[], 10, 0, &mut rng, Detail::Full, false);
            let spells: Vec<u32> =
                main.iter().map(|&f| crate::cube::card_brief(f).cmc).collect();
            spells.iter().sum::<u32>() as f64 / spells.len().max(1) as f64
        };
        let off = mean_cmc(false);
        let on = mean_cmc(true);
        assert!(on < off, "flag on builds a cheaper deck: {on} vs {off}");
    }

    /// The slot arithmetic in isolation: capacity is exactly the spell
    /// count, the early reserve is cheap-only, and the top allowance is a
    /// hard cap.
    #[test]
    fn curve_slots_reserve_early_and_cap_the_top() {
        let c = CurveSlots::new(23);
        assert_eq!((c.early, c.top, c.open), (5, 6, 18), "5 early + 18 open = 23");

        // Six five-drops: the sixth is the last the cap allows.
        let mut c = CurveSlots::new(23);
        for i in 0..6 {
            assert!(c.claim(5), "five-drop {i} fits under the cap of six");
        }
        assert!(!c.claim(5), "the seventh is refused");
        assert!(c.claim(3), "a three-drop is unaffected by the top cap");
        assert!(c.claim(2), "and the early reserve is untouched");

        // The reserve is not spendable by expensive cards: fill every open
        // slot with mid-cost spells and the five early slots must survive.
        let mut c = CurveSlots::new(23);
        for _ in 0..18 {
            assert!(c.claim(3));
        }
        assert!(!c.claim(3), "open is exhausted");
        assert!(!c.claim(6), "and so is the deck, for anything expensive");
        for _ in 0..5 {
            assert!(c.claim(1), "but the reserved cheap slots are still there");
        }
        assert!(!c.claim(1), "23 cards and no more");
    }

    /// `ShapeSlots` at 23 spells: the 20th creature is refused (cap 19),
    /// and once the remaining slots only just cover the creature minimum a
    /// noncreature is refused too.
    #[test]
    fn shape_slots_hold_the_creature_range() {
        let brief = |f: CardFactory| crate::cube::card_brief(f);
        let (bear, giant, bolt) =
            (brief(crate::catalog::grizzly_bears), brief(crate::catalog::hill_giant), brief(crate::catalog::lightning_bolt));
        let mut s = ShapeSlots::new(23);
        assert_eq!((s.min, s.max), ([16, 8, 2], [19, 11, 5]));
        // Eight cheap creatures, then eleven four-drop creatures: nineteen.
        for _ in 0..8 {
            assert!(s.claim(bear));
        }
        for i in 0..11 {
            assert!(s.claim(giant), "creature {}", 9 + i);
        }
        assert!(!s.claim(giant), "a 20th creature is over the cap");
        // Four slots left and the top-end minimum (two at mana value >= 5)
        // unmet: the bolts fit while two slots stay free for it.
        assert!(s.claim(bolt) && s.claim(bolt));
        assert!(!s.claim(bolt), "the last two slots are the top end's");

        // From empty: seven noncreatures fit, the eighth would leave fifteen
        // slots for sixteen creatures.
        let mut s = ShapeSlots::new(23);
        for _ in 0..7 {
            assert!(s.claim(bolt) || s.claim(brief(crate::catalog::divination)));
        }
        assert!(!s.claim(brief(crate::catalog::divination)), "16 creatures still have to fit");
    }

    /// End to end: a pool whose best cards are all expensive still builds to
    /// a curve under the flag.
    #[test]
    fn curve_slots_build_the_early_drops_into_the_deck() {
        let mut cheap: Vec<CardFactory> = Vec::new();
        let mut dear: Vec<CardFactory> = Vec::new();
        for f in crate::cube::cube_pool_all() {
            let b = crate::cube::card_brief(f);
            if b.is_land {
                continue;
            }
            if b.cmc <= 2 && cheap.len() < 30 {
                cheap.push(f);
            } else if b.cmc >= 5 && dear.len() < 30 {
                dear.push(f);
            }
        }
        assert!(cheap.len() >= 20 && dear.len() >= 20, "pool has both ends");
        // Expensive first, so a score-only build reaches for them.
        let pool: Vec<CardFactory> = dear.iter().chain(cheap.iter()).copied().collect();
        let buckets = |aggro: bool| -> (usize, usize) {
            let scores = PoolScores::new(&pool, true, aggro, false);
            let colors = colors_of_picks(&pool);
            let cols: Vec<Color> =
                Color::ALL.iter().copied().filter(|c| colors.get(*c) > 0).collect();
            let mut rng = StdRng::seed_from_u64(11);
            let MainDeck { main, .. } =
                suggest_main_deck_shape(&scores, &cols, &[], 23, 0, &mut rng, Detail::Full, false);
            assert_eq!(main.len(), 23, "the constraint reshapes the deck, never shrinks it");
            let cmcs: Vec<u32> = main.iter().map(|&f| crate::cube::card_brief(f).cmc).collect();
            (
                cmcs.iter().filter(|&&c| c <= 2).count(),
                cmcs.iter().filter(|&&c| c >= 5).count(),
            )
        };
        let (early_off, top_off) = buckets(false);
        let (early_on, top_on) = buckets(true);
        assert!(early_on >= 5, "flag on hits the early floor: {early_on}");
        assert!(top_on <= 6, "and respects the top cap: {top_on}");
        assert!(
            early_on > early_off || top_on < top_off,
            "and it actually changed the deck: {early_off}/{top_off} -> {early_on}/{top_on}",
        );
    }

    /// Pins the v3 ranker's composition: legacy rank + quality sum − curve
    /// penalty, so neither term can be dropped silently.
    #[test]
    fn static_score_v3_adds_quality_and_curve() {
        let main = five_bolts_six_angels();
        let colors = colors_of_picks(&main);
        let quality: i32 =
            main.iter().map(|&f| crate::draft::card_quality(crate::cube::card_def(f))).sum();
        assert!(quality > 0, "Serra Angel's flying body must count for something");
        assert_eq!(
            static_build_score_v3(&main, main.len(), false),
            static_build_score(&main, main.len()) + quality - curve_penalty(&main),
            "v3 = legacy + per-card quality − curve penalty, colors {colors:?}"
        );
    }

    /// `builder_v2`'s squared pips: four {3}{W}{W} bodies outweigh four {R}
    /// spells despite the earlier red pips — the deliberate reversal of
    /// `basic_split_leans_on_cheap_pips` (linear pips gave a {3}{U}{U} bomb
    /// three Islands).
    #[test]
    fn squared_pips_serve_double_costs() {
        let mut main: Vec<CardFactory> = Vec::new();
        for _ in 0..4 {
            main.push(catalog::lightning_bolt);
        }
        for _ in 0..4 {
            main.push(catalog::serra_angel);
        }
        let split = basic_split(&main, &[Color::Red, Color::White], 10, true);
        assert!(
            split[&Color::White] > split[&Color::Red],
            "double-white demands the sources, got {split:?}",
        );
        assert_eq!(split.values().sum::<u32>(), 10);
    }

    /// The paired estimator has to agree with the naive win rate on the
    /// mean — pairing buys precision, not a different answer. Four
    /// A-sweeps and one B-sweep out of ten is 13 of 20 games to A.
    #[test]
    fn paired_mean_matches_the_win_rate_it_estimates() {
        let pairs = [1, 1, 1, 1, -1, 0, 0, 0, 0, 0];
        let s = paired_stat(&pairs).unwrap();
        assert_eq!(s.n, 10);
        assert!((s.p - 0.65).abs() < 1e-9, "got {}", s.p);
    }

    /// All splits is a perfectly cancelled deal: dead even with zero width,
    /// which independent games can never report.
    #[test]
    fn all_splits_is_an_even_result_with_no_uncertainty() {
        let s = paired_stat(&[0; 8]).unwrap();
        assert!((s.p - 0.5).abs() < 1e-9);
        assert!(s.se < 1e-9, "splits carry no variance, got se {}", s.se);
        assert!((s.rho + 1.0).abs() < 1e-9, "expected rho -1, got {}", s.rho);
    }

    /// Independent games at p = 0.5 split half the time and sweep the
    /// other half, so that mix must read as rho = 0 — the calibration
    /// point the efficiency factor is measured against.
    #[test]
    fn half_sweeps_half_splits_reads_as_independent() {
        let pairs = [1, -1, 1, -1, 0, 0, 0, 0];
        let s = paired_stat(&pairs).unwrap();
        assert!((s.p - 0.5).abs() < 1e-9);
        assert!(s.rho.abs() < 1e-9, "expected rho 0, got {}", s.rho);
        // sd(S) = sqrt(0.5) over n = 8, halved onto the win-rate scale.
        assert!((s.se - (0.5f64 / 8.0).sqrt() / 2.0).abs() < 1e-9, "got {}", s.se);
    }

    /// The failure mode: every pair a sweep means pairing inflated variance
    /// instead of cancelling it (rho = +1, half the precision of
    /// independent games).
    #[test]
    fn all_sweeps_is_variance_inflation_not_reduction() {
        let pairs = [1, -1, 1, -1, 1, -1, 1, -1];
        let s = paired_stat(&pairs).unwrap();
        assert!((s.p - 0.5).abs() < 1e-9);
        assert!((s.rho - 1.0).abs() < 1e-9, "expected rho +1, got {}", s.rho);
    }

    /// Too few pairs for an empirical variance to mean anything.
    #[test]
    fn under_two_pairs_has_no_paired_estimate() {
        assert!(paired_stat(&[]).is_none());
        assert!(paired_stat(&[1]).is_none());
    }

    /// Wilson bounds: an undefeated small sample keeps a non-degenerate
    /// interval (the normal approximation gave ±0 at p̂ = 1, and a lucky 5-0
    /// eliminated the whole field in round 0).
    #[test]
    fn wilson_interval_is_not_degenerate_at_extremes() {
        let (lb, ub) = wilson(5, 5, 1.96);
        assert!(lb < 1.0, "5-0 keeps a lower bound below certainty, got {lb}");
        assert!(ub <= 1.0);
        assert!((ub - lb) / 2.0 > 0.05, "5-0 keeps honest width");
        let (lb, ub) = wilson(0, 5, 1.96);
        assert!(lb < 0.01);
        assert!(ub > 0.0 && ub < 1.0, "0-5 upper bound stays above zero, got {ub}");
    }

}
