//! Sealed-deck recommender.
//!
//! Given a sealed pool, enumerate candidate builds across the full color
//! lattice (pairs, pair + splash, 3/4/5-color), rank them with a cheap
//! static score, then evaluate the top K by playing bot-vs-bot matches
//! against a *field gauntlet*: decks built (with per-pool seeded
//! randomness) from independently generated sealed pools of the same
//! set. Candidates never play each other — the question answered is
//! "how does this build do against the format", not "who wins the
//! mirror".
//!
//! Evaluation is embarrassingly parallel (every game is an independent
//! `GameState`); a job-queue over `std::thread::scope` workers spreads
//! games across cores, and an optional racing mode (successive halving)
//! concentrates the game budget on statistically close candidates.
//!
//! A [`Session`] owns the gauntlet plus a per-deck outcome cache shared
//! across the staged pipeline (rank → refine → local search): the
//! gauntlet is generated once, and a deck re-raced in a later stage
//! (the stage-1 winner returning as refine's v0, the incumbent entering
//! every search generation) replays its recorded outcomes instead of
//! re-simulating them. Per-game outcomes are kept per (opponent, game
//! slot), so candidate comparisons — racing elimination, local-search
//! acceptance — use paired differences over shared slots, which the CRN
//! shuffles make far tighter than comparing independent intervals.
//!
//! Determinism: `SimConfig::seed` fully determines the gauntlet
//! (pool contents and randomized builds). Match *outcomes* still use
//! the global thread-local RNG (deck shuffles, bot tie-jitter), so win
//! rates carry sampling noise — the confidence intervals reported per
//! candidate are the honest error bars for that.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, RngExt, SeedableRng};

use crate::cube::CardFactory;
use crate::draft::{
    COPY_CAP, colored_pip_count, colors_of_cost, colors_of_picks, generate_sos_pack,
    score_card_with_colors, sos_draft_pool,
};
use crate::game::GameState;
use crate::mana::Color;
use crate::player::Player;
use crate::server::{Bot, EvalWeights, MctsBot, MctsConfig, RandomBot};

/// Everything tunable about a recommender run. Maps 1:1 onto the client's
/// simulation-settings panel; engine callers use `SimConfig::default()`.
#[derive(Debug, Clone)]
pub struct SimConfig {
    /// Synthetic sealed pools in the opposing field.
    pub gauntlet_size: usize,
    /// Games per racing round per candidate *per sampled opponent*
    /// (racing off: per gauntlet opponent, full round-robin).
    pub games_per_pairing: usize,
    /// Candidates advancing from static ranking to simulation.
    pub candidate_cap: usize,
    /// Successive-halving elimination between rounds.
    pub racing: bool,
    /// z-score for the racing elimination bound (1.96 ≈ 95%).
    pub racing_confidence_z: f64,
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
    /// ([`draft::card_quality`]), splash candidates restricted to single
    /// colored pips, and basic lands split by squared pip demand so a
    /// double-pip card gets the sources it actually needs.
    ///
    /// A flag rather than a rewrite so the previous builder stays as the
    /// measurement control — all three defects were found together (a
    /// {3}{U}{U} bomb "splashed" off three Islands) and are fixed
    /// together, so they are measured together.
    pub builder_v2: bool,
    /// Gauntlet seed — a run with the same (set, seed) faces the same field.
    pub seed: u64,
    /// Worker threads; 0 = available cores minus one.
    pub threads: usize,
    /// Pilot the gauntlet decks with the uniform-baseline bot instead of
    /// the scored bot (candidates always use the scored bot).
    pub uniform_opponent_bot: bool,
    /// Safety cap on turns-worth of actions per game before calling it
    /// undecided (mirrors the ladder-test guard).
    pub max_actions_per_game: usize,
    /// Common random numbers: seed each game's deck shuffles from
    /// (opponent, game-index) — *not* the candidate — so every candidate
    /// faces the same opponent draws. Sharply reduces the variance of
    /// candidate-vs-candidate *differences*. Partial by design: bot
    /// tie-jitter and in-game randomness still use the global RNG.
    pub crn: bool,
    /// Refinement stage: how many top-ranked shapes get variant builds.
    pub refine_top: usize,
    /// Refinement stage: builds per shape (the greedy build + jittered
    /// variants with sampled spell/land counts).
    pub variants_per_shape: usize,
    /// Racing rounds. More rounds with a smaller `games_per_pairing`
    /// prunes a big fleet cheaply and spends the saved games on the
    /// finalists (each round widens the opponent sample and adds
    /// `games_per_pairing` games per pairing).
    pub racing_rounds: u32,
    /// Stage-3 local search: generations of attribution-guided card swaps
    /// around the incumbent winner. 0 disables (default).
    pub search_generations: usize,
    /// Stage-3 local search: swap children raced per generation.
    pub search_children: usize,
    /// Stage-3 local search: one-sided z for adopting a child over the
    /// incumbent, applied to the paired win-rate difference over their
    /// shared game slots. 1.0 ≈ 84% one-sided confidence — mild on
    /// purpose: children are near-neighbors of the incumbent, so true
    /// edges are small and a strict bar stalls the search.
    pub search_accept_z: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            gauntlet_size: 20,
            games_per_pairing: 20,
            candidate_cap: 8,
            racing: true,
            racing_confidence_z: 1.96,
            builder_v2: true,
            build_temperature: 1.0,
            spell_count_range: (22, 24),
            land_count_range: (16, 18),
            target_spells: 23,
            total_lands: 17,
            splash_max_cards: 3,
            splash_min_score: 12,
            seed: 0,
            threads: 0,
            uniform_opponent_bot: false,
            max_actions_per_game: 50_000,
            crn: true,
            refine_top: 3,
            variants_per_shape: 6,
            racing_rounds: 3,
            search_generations: 0,
            search_children: 8,
            search_accept_z: 1.0,
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

/// Rolling evaluation state for one simulated candidate. `wins`/`losses`
/// count decided games from the candidate's perspective; `undecided`
/// counts stalls/draws (excluded from the win rate).
#[derive(Debug, Clone)]
pub struct CandidateEval {
    /// Index into the simulated-candidates slice.
    pub candidate: usize,
    pub wins: u32,
    pub losses: u32,
    pub undecided: u32,
    /// Racing round this candidate was eliminated in (`None` = survived).
    pub eliminated_round: Option<u32>,
}

impl CandidateEval {
    fn new(candidate: usize) -> Self {
        Self { candidate, wins: 0, losses: 0, undecided: 0, eliminated_round: None }
    }

    pub fn decided(&self) -> u32 {
        self.wins + self.losses
    }

    pub fn win_rate(&self) -> f64 {
        let n = self.decided();
        if n == 0 { 0.5 } else { self.wins as f64 / n as f64 }
    }

    /// Wilson score interval for the win rate at `z`. Well-behaved at
    /// small n and at p̂ = 0 or 1, where the normal approximation
    /// collapses to zero width — an undefeated candidate would claim a
    /// lower bound of 1.0 and racing would eliminate the entire field.
    pub fn ci_bounds(&self, z: f64) -> (f64, f64) {
        let n = self.decided() as f64;
        if n < 1.0 {
            return (0.0, 1.0);
        }
        let p = self.win_rate();
        let z2 = z * z;
        let denom = 1.0 + z2 / n;
        let center = (p + z2 / (2.0 * n)) / denom;
        let half = (z / denom) * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
        ((center - half).max(0.0), (center + half).min(1.0))
    }

    /// Half the Wilson interval width — the "± x%" for display. (The
    /// interval is not centered on the raw win rate; comparisons should
    /// use [`Self::ci_bounds`] directly.)
    pub fn ci_halfwidth(&self, z: f64) -> f64 {
        let (lo, hi) = self.ci_bounds(z);
        (hi - lo) / 2.0
    }
}

/// Full output of a recommender run.
pub struct Recommendation {
    /// Every enumerated candidate, sorted by descending static score.
    pub candidates: Vec<CandidateBuild>,
    /// Evaluations for the first `evals.len()` candidates (the top K).
    pub evals: Vec<CandidateEval>,
    /// Indices into `candidates` sorted by simulated win rate (best first).
    pub ranking: Vec<usize>,
    /// The seed the gauntlet was built from (echoed for reproduction).
    pub seed: u64,
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

/// All k-subsets of the 5 colors, k in `sizes`.
fn color_subsets(sizes: &[usize]) -> Vec<Vec<Color>> {
    let mut out = Vec::new();
    for mask in 1u8..(1 << 5) {
        let k = mask.count_ones() as usize;
        if !sizes.contains(&k) {
            continue;
        }
        let set: Vec<Color> = Color::ALL
            .into_iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, c)| c)
            .collect();
        out.push(set);
    }
    out
}

/// Build a main deck restricted to `colors` (plus the explicitly allowed
/// `splash` cards), greedy by static card score with optional `noise`
/// jitter for randomized gauntlet builds. Same copy-cap/overflow behavior
/// as `draft::suggest_main_deck`.
pub fn suggest_main_deck_in_colors<R: Rng>(
    picks: &[CardFactory],
    colors: &[Color],
    splash: &[CardFactory],
    target_spells: usize,
    noise: i32,
    rng: &mut R,
    quality: bool,
) -> (Vec<CardFactory>, Vec<CardFactory>) {
    let allowed = |f: CardFactory| -> bool {
        let def = f();
        // Lands never occupy spell slots in the sealed builder —
        // `assemble_lands` owns the land base. (High jitter used to
        // promote off-color duals into the 22-24 spell main.)
        if def.card_types.contains(&crate::card::CardType::Land) {
            return false;
        }
        let card_colors = colors_of_cost(&def.cost);
        card_colors.is_empty()
            || card_colors.iter().all(|c| colors.contains(c))
            || splash.iter().any(|s| *s as usize == f as usize)
    };
    // Hoisted: pip totals over the pile are invariant while scoring it.
    let pick_colors = colors_of_picks(picks);
    // Fixing-aware bonus: mana rocks / land fetchers earn their keep in
    // proportion to how many colors the build stretches across.
    let fixing_bonus = match colors.len() {
        0..=1 => 0,
        2 => 1,
        _ => 3,
    };
    let mut scored: Vec<(CardFactory, i32)> = Vec::new();
    let mut off: Vec<CardFactory> = Vec::new();
    for &f in picks {
        if allowed(f) {
            let jitter = if noise > 0 { rng.random_range(-noise..=noise) } else { 0 };
            // Lands never take spell slots — they're assigned by
            // `assemble_lands`; the fixing bonus is for rocks/fetchers.
            let def = f();
            let fix = if fixing_bonus > 0
                && !def.card_types.contains(&crate::card::CardType::Land)
                && is_fixing_card(&def)
            {
                fixing_bonus
            } else {
                0
            };
            let base = if quality {
                crate::draft::score_card_quality(f, &pick_colors)
            } else {
                score_card_with_colors(f, &pick_colors)
            };
            scored.push((f, base + jitter + fix));
        } else {
            off.push(f);
        }
    }
    scored.sort_by_key(|(_, s)| std::cmp::Reverse(*s));
    let mut counts: HashMap<usize, u32> = HashMap::new();
    let mut main = Vec::new();
    let mut leftovers = Vec::new();
    for (f, _) in scored {
        let count = counts.entry(f as usize).or_insert(0);
        if main.len() < target_spells && *count < COPY_CAP {
            *count += 1;
            main.push(f);
        } else {
            leftovers.push(f);
        }
    }
    leftovers.extend(off);
    (main, leftovers)
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

/// Cheap build rank: summed card scores, minus a shortfall penalty when
/// the pool can't fill the target spell count in these colors, minus a
/// consistency penalty on colored pips beyond the two heaviest colors
/// (what a 3+-color build pays in mana reliability).
pub(crate) fn static_build_score(main: &[CardFactory], target_spells: usize) -> i32 {
    let main_colors = colors_of_picks(main);
    let mut score: i32 = main.iter().map(|&f| score_card_with_colors(f, &main_colors)).sum();
    let shortfall = target_spells.saturating_sub(main.len()) as i32;
    score -= shortfall * 8;
    let mut pips: Vec<i32> = Color::ALL
        .into_iter()
        .map(|c| main.iter().map(|&f| colored_pip_count(&f().cost, c) as i32).sum())
        .collect();
    pips.sort_unstable_by_key(|n| std::cmp::Reverse(*n));
    // Basics-only fixing makes each color beyond the second expensive:
    // a flat cost per extra color plus a steep per-pip cost. Without the
    // flat term, an 82-card pool's marginal card-quality gains rank the
    // 5-color pile above every honest pair.
    let extra_colors = pips.iter().skip(2).filter(|n| **n > 0).count() as i32;
    score -= extra_colors * 12;
    score -= pips.iter().skip(2).sum::<i32>() * 6;
    score
}

/// Top splash-worthy cards for splashing `third` next to `pair`: any card
/// whose colors include `third` and fit within pair+third (score ≥ the
/// bar). Gold cards straddling the pair and the splash qualify — "U/B
/// splashing a B/G bomb" is a real sealed build, and a mono-only rule
/// silently excluded exactly the cards most worth splashing.
fn splash_cards(pool: &[CardFactory], pair: &[Color], third: Color, cfg: &SimConfig) -> Vec<CardFactory> {
    let pool_colors = colors_of_picks(pool);
    let mut hits: Vec<(CardFactory, i32)> = pool
        .iter()
        .filter(|&&f| {
            let def = f();
            let cs = colors_of_cost(&def.cost);
            if !(cs.contains(&third) && cs.iter().all(|c| *c == third || pair.contains(c))) {
                return false;
            }
            // A splash is a handful of off-color sources, so it can only
            // support ONE colored pip. Without this the builder splashed
            // {3}{U}{U} Emeritus of Ideation off three Islands — a card
            // it then measurably failed to cast.
            !cfg.builder_v2 || colored_pip_count(&def.cost, third) <= 1
        })
        .map(|&f| {
            let s = if cfg.builder_v2 {
                crate::draft::score_card_quality(f, &pool_colors)
            } else {
                score_card_with_colors(f, &pool_colors)
            };
            (f, s)
        })
        .filter(|(_, s)| *s >= cfg.splash_min_score)
        .collect();
    hits.sort_by_key(|(_, s)| std::cmp::Reverse(*s));
    hits.into_iter().take(cfg.splash_max_cards).map(|(f, _)| f).collect()
}

/// Proportional basic split over arbitrarily many colors (≥1 source per
/// color with any pips when possible). Generalizes
/// `draft::suggest_basic_split` beyond two colors. Pips are weighted by
/// earliness — a {B} pip on a two-drop needs its source in the opening
/// hand, a {B} pip on a six-drop can wait — so a splash of expensive
/// cards leans on fewer sources than the raw pip count suggests.
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
                    let def = f();
                    let pips = colored_pip_count(&def.cost, c);
                    // Casting {U}{U} on curve needs roughly half again
                    // the sources {U} does, not the same — a linear pip
                    // count under-serves every double-pip card in the
                    // deck. Squaring is the cheap standing-in for the
                    // hypergeometric answer.
                    let demand = if squared_pips { pips * pips } else { pips };
                    demand * 7u32.saturating_sub(def.cost.cmc()).max(1)
                })
                .sum();
            (c, w)
        })
        .collect();
    let total_w: u32 = weights.iter().map(|(_, w)| w).sum();
    let mut out: HashMap<Color, u32> = HashMap::new();
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

/// Colors a land taps for — the union of its `AddMana` ability payloads
/// (an any-color payload counts as all five).
fn land_produced_colors(def: &crate::card::CardDefinition) -> Vec<Color> {
    use crate::effect::{Effect, ManaPayload};
    let mut seen = [false; 5];
    for ab in &def.activated_abilities {
        if let Effect::AddMana { pool, .. } = &ab.effect {
            match pool {
                ManaPayload::Colors(cs) => {
                    for c in cs {
                        seen[*c as usize] = true;
                    }
                }
                ManaPayload::AnyOneColor(_) => seen = [true; 5],
                _ => {}
            }
        }
    }
    Color::ALL.into_iter().filter(|c| seen[*c as usize]).collect()
}

/// Split `total` land slots between on-color duals from the pool and
/// proportional basics. A pool land qualifies when it taps for at least
/// two of the build's colors and nothing outside them — an off-color or
/// mono-color utility land is just a worse basic to this builder. Used
/// duals are removed from `leftovers`.
fn assemble_lands(
    leftovers: &mut Vec<CardFactory>,
    main: &[CardFactory],
    colors: &[Color],
    total: u32,
    squared_pips: bool,
) -> (Vec<CardFactory>, HashMap<Color, u32>) {
    use crate::card::CardType;
    let mut duals: Vec<CardFactory> = Vec::new();
    leftovers.retain(|&f| {
        if duals.len() >= total as usize {
            return true;
        }
        let def = f();
        if !def.card_types.contains(&CardType::Land) {
            return true;
        }
        let produced = land_produced_colors(&def);
        let on_color = produced.iter().filter(|c| colors.contains(c)).count();
        if on_color >= 2 && on_color == produced.len() {
            duals.push(f);
            false
        } else {
            true
        }
    });
    let basics = basic_split(main, colors, total - duals.len() as u32, squared_pips);
    (duals, basics)
}

fn candidate_label(colors: &[Color], splash: &[Color]) -> String {
    let mut s: String = colors
        .iter()
        .map(|&c| color_letter(c).to_string())
        .collect::<Vec<_>>()
        .join("/");
    if !splash.is_empty() {
        s.push_str(" + ");
        s.push_str(
            &splash
                .iter()
                .map(|&c| color_letter(c).to_ascii_lowercase().to_string())
                .collect::<Vec<_>>()
                .join("/"),
        );
    }
    s
}

/// Build one deck of the given shape: chosen spells, on-color duals,
/// basic split, static score. `noise > 0` jitters the card picks for
/// randomized (gauntlet / variant) builds. `None` when the shape is
/// hollow — no splash-worthy cards for a splash shape, or no playables
/// at all.
fn build_shape<R: Rng>(
    pool: &[CardFactory],
    colors: &[Color],
    splash_colors: &[Color],
    // (spell slots, land count, pick jitter)
    shape: (usize, u32, i32),
    cfg: &SimConfig,
    rng: &mut R,
) -> Option<CandidateBuild> {
    let (spells, lands, noise) = shape;
    let splash: Vec<CardFactory> = splash_colors
        .iter()
        .flat_map(|&c| splash_cards(pool, colors, c, cfg))
        .collect();
    if !splash_colors.is_empty() && splash.is_empty() {
        return None;
    }
    let (main, mut leftovers) =
        suggest_main_deck_in_colors(pool, colors, &splash, spells, noise, rng, cfg.builder_v2);
    if main.is_empty() {
        return None;
    }
    // Land colors: main colors plus any splash color actually present.
    let mut land_colors = colors.to_vec();
    for &c in splash_colors {
        if main.iter().any(|&f| colors_of_cost(&f().cost).contains(&c)) {
            land_colors.push(c);
        }
    }
    // Legal-deck floor: when the pool can't fill the requested spell
    // count, pad lands so the deck still reaches 40 cards.
    let lands = lands.max(40u32.saturating_sub(main.len() as u32));
    let (duals, basics) =
        assemble_lands(&mut leftovers, &main, &land_colors, lands, cfg.builder_v2);
    Some(CandidateBuild {
        static_score: static_build_score(&main, spells),
        label: candidate_label(colors, splash_colors),
        colors: colors.to_vec(),
        splash: splash_colors.to_vec(),
        main,
        duals,
        basics,
        leftovers,
    })
}

/// Enumerate every candidate build for `pool`: 10 pairs, pair + splash,
/// 10 triples, 5 four-color, 1 five-color. Sorted by descending static
/// score, deduplicated by identical main-deck contents (a triple whose
/// third color contributes nothing collapses into its pair).
pub fn enumerate_candidates(pool: &[CardFactory], cfg: &SimConfig) -> Vec<CandidateBuild> {
    // (main colors, splash colors) shapes.
    let mut shapes: Vec<(Vec<Color>, Vec<Color>)> = Vec::new();
    for pair in color_subsets(&[2]) {
        for third in Color::ALL {
            if !pair.contains(&third) {
                shapes.push((pair.clone(), vec![third]));
            }
        }
        shapes.push((pair, vec![]));
    }
    for wide in color_subsets(&[3, 4, 5]) {
        shapes.push((wide, vec![]));
    }

    let mut rng = StdRng::seed_from_u64(cfg.seed); // noise=0 → unused; keeps API one-shape
    let mut out: Vec<CandidateBuild> = Vec::new();
    let mut seen_mains: std::collections::HashSet<Vec<usize>> = std::collections::HashSet::new();
    for (colors, splash_colors) in shapes {
        let Some(build) = build_shape(
            pool,
            &colors,
            &splash_colors,
            (cfg.target_spells, cfg.total_lands, 0),
            cfg,
            &mut rng,
        ) else {
            continue;
        };
        let mut key: Vec<usize> = build.main.iter().map(|&f| f as usize).collect();
        key.sort_unstable();
        if !seen_mains.insert(key) {
            continue;
        }
        out.push(build);
    }
    out.sort_by_key(|c| std::cmp::Reverse(c.static_score));
    out
}

// ─────────────────────────────── gauntlet ────────────────────────────────

/// Build one randomized deck from `pulls`. Color identity is sampled by a
/// softmax over the top shapes' static scores (temperature-scaled), card
/// choice gets score jitter, and spell/land counts are sampled from the
/// config ranges — so a field of these looks like a room of humans, not
/// clones.
pub(crate) fn build_random_deck<R: Rng>(pulls: &[CardFactory], cfg: &SimConfig, rng: &mut R) -> GauntletDeck {
    // Same shape lattice as user candidates (pairs, splashes, 3/4/5-color).
    // A field that only ever builds pairs never bombs back at splash-shaped
    // candidates — inflating their measured win rates.
    let shapes = enumerate_candidates(pulls, cfg);
    let t = cfg.build_temperature.max(0.0);
    // Honest field: weight shape choice by the actual static-score gaps
    // (softmax over the top 5) instead of flat rank weights — a pool whose
    // best build dominates plays it nearly always, while close calls stay
    // diverse. A ~12-point score gap costs ~e× likelihood at t = 1;
    // t → 0 collapses to the argmax.
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

    // Noisy rebuild of the chosen shape: jittered card picks, sampled
    // spell/land counts. The jitter is deliberately mild — a field of
    // near-greedy builds is the honest opposition; heavy jitter made the
    // field soft and inflated every candidate's measured win rate.
    let (spells, lands) = sample_deck_split(cfg, rng);
    let noise = (t * 2.0).round() as i32;
    let build =
        build_shape(pulls, &chosen.colors, &chosen.splash, (spells, lands, noise), cfg, rng)
            .unwrap_or_else(|| {
                // The noisy rebuild can only fail if the shape was hollow —
                // and it came from enumerate, so it isn't. Defensive: fall
                // back to the enumerated build itself.
                build_shape(
                    pulls,
                    &chosen.colors,
                    &chosen.splash,
                    (cfg.target_spells, cfg.total_lands, 0),
                    cfg,
                    rng,
                )
                .expect("enumerated shape rebuilds")
            });
    GauntletDeck { label: build.label.clone(), cards: build.deck() }
}

/// Generate the opposing field: `gauntlet_size` independent sealed pools
/// (6 SOS packs each), one randomized build per pool. Fully determined
/// by `cfg.seed`.
pub fn generate_gauntlet(cfg: &SimConfig) -> Vec<GauntletDeck> {
    let pool = sos_draft_pool();
    let n = cfg.gauntlet_size;
    // Pools are independent and per-index seeded, so building them in
    // parallel changes nothing about determinism — only wall clock.
    let out: Mutex<Vec<Option<GauntletDeck>>> = Mutex::new((0..n).map(|_| None).collect());
    let cursor = AtomicUsize::new(0);
    let threads = worker_threads(cfg).min(n.max(1));
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let i = cursor.fetch_add(1, Ordering::Relaxed);
                    if i >= n {
                        break;
                    }
                    // Distinct stream per pool; the odd multiplier
                    // decorrelates adjacent seeds.
                    let mut rng = StdRng::seed_from_u64(
                        cfg.seed
                            ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1),
                    );
                    let pulls: Vec<CardFactory> =
                        (0..6).flat_map(|_| generate_sos_pack(&pool, &mut rng)).collect();
                    let deck = build_random_deck(&pulls, cfg, &mut rng);
                    out.lock().unwrap()[i] = Some(deck);
                }
            });
        }
    });
    out.into_inner().unwrap().into_iter().map(|d| d.expect("all pools built")).collect()
}

// ─────────────────────────────── simulation ──────────────────────────────

/// Which bot drives a seat. The recommender only ever needs the scored
/// bot and the uniform control, but the bot ladder pits two *evaluation
/// profiles* against each other, so the seat pilot is a value rather than
/// a bool.
#[derive(Debug, Clone, Copy)]
pub enum Pilot {
    /// The scored bot with a specific evaluation profile.
    Scored(EvalWeights),
    /// The pre-scoring uniform-random pick. Ladder control.
    Uniform,
    /// The Monte Carlo search bot.
    Mcts(MctsConfig),
}

impl Pilot {
    /// The evaluation profile this pilot plays, when it has one. Used to
    /// push profile settings that the *engine* reads (rather than the
    /// bot) onto the seat before the game starts.
    fn weights(self) -> Option<EvalWeights> {
        match self {
            Pilot::Scored(w) => Some(w),
            Pilot::Uniform | Pilot::Mcts(_) => None,
        }
    }

    fn build(self) -> Box<dyn Bot> {
        match self {
            Pilot::Scored(w) => Box::new(RandomBot::with_weights(w)),
            Pilot::Uniform => Box::new(RandomBot::uniform_baseline()),
            Pilot::Mcts(cfg) => Box::new(MctsBot::new(cfg)),
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
        match o.stop {
            StopReason::ActionCap => self.action_capped += 1,
            StopReason::NoLegalMove => self.no_legal_move += 1,
            StopReason::GameOver if o.winner.is_none() => self.draws += 1,
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
    // Build the two seat arrangements ONCE (libraries loaded, unshuffled)
    // and clone per game — definitions are Arc'd, so a state clone is a
    // fraction of re-invoking ~80 card factories per game.
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
        // Swap the pilots along with the decks so side A plays seat 0 and
        // seat 1 equally often — turn order is worth a few points of win
        // rate on its own and would otherwise be confounded with the
        // profile under test.
        let seated =
            if a_seat0 { [pilots[0], pilots[1]] } else { [pilots[1], pilots[0]] };
        // Unpaired games keep the unseeded jitter: this is the control
        // arm, and seeding it here would quietly change what it measures.
        let outcome = play_one_game(template, seated, max_actions, shuffle_rng.as_mut(), None);
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
    /// One entry per *fully decided* pair, in play order. A pair with an
    /// undecided game is absent rather than recorded as a split: its games
    /// still count in the unpaired totals above, but it carries no paired
    /// evidence, and scoring it 0 would dilute the mean toward "even" with
    /// games that never finished.
    pub pairs: Vec<i8>,
}

/// Wilson score interval at `z` standard normal deviates.
///
/// Not the normal approximation: at p̂ = 0 or 1 that has zero width, which
/// once let a lucky 5-0 candidate claim a lower bound of 1.0 and eliminate
/// the entire rest of the field (see
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
/// `p̂ = (E[S] + 1) / 2` and `SE(p̂) = sd(S) / (2√n)`, both read off the
/// empirical pair-score distribution — no independence assumption between
/// the two games of a pair, which is exactly the assumption pairing
/// breaks.
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

/// [`simulate_match_games_piloted`] with **antithetic seat pairing**: each
/// pair plays the same shuffle twice with the pilots swapped between the
/// seats.
///
/// The ordinary ladder draws a fresh shuffle for every game and swaps
/// seats on alternate games, so seat luck and deal luck are averaged away
/// rather than cancelled. In a 40-card mirror most of the game-to-game
/// variance *is* the deal — one pilot flooding while the other curves out
/// says nothing about either. Playing both sides of one deal makes that
/// noise cancel within the pair instead of across thousands of games:
/// whoever was going to win on the strength of the cards wins their copy
/// of it in each direction, and the pair splits. Only a genuine piloting
/// edge produces sweeps.
///
/// Formally the pair score S ∈ {−1, 0, +1} has E[S] = 2p − 1 and
/// Var(S) = 2p(1−p)(1 + ρ), where ρ is the within-pair correlation. A
/// decisive-deal field drives ρ negative, and the variance of the win-rate
/// estimate falls by exactly that (1 + ρ) factor versus the same number of
/// unpaired games. `bot_ladder` measures ρ and reports the realized
/// efficiency rather than asserting one.
///
/// This is only a variance reduction, not a change of estimand: the mean
/// is the same win rate the unpaired ladder estimates, which is why the
/// unpaired total is still reported alongside it as the control.
///
/// Both games of a pair take their shuffle from one `seed_base + k` stream,
/// so a mirror deals seat 0 the identical library in both directions.
pub fn simulate_match_pairs_piloted(
    deck_a: &[CardFactory],
    deck_b: &[CardFactory],
    pairs: usize,
    pilots: [Pilot; 2],
    max_actions: usize,
    seed_base: u64,
) -> PairedTally {
    let template_a0 = build_match_template(deck_a, deck_b);
    let template_b0 = build_match_template(deck_b, deck_a);
    let mut tally = PairedTally {
        wins_a: 0,
        wins_b: 0,
        undecided: 0,
        cost: SimCost::default(),
        pairs: Vec::with_capacity(pairs),
    };
    for k in 0..pairs {
        let seed = seed_base.wrapping_add((k as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut score = 0i8;
        let mut decided_both = true;
        for a_seat0 in [true, false] {
            // Re-seeded per game, *not* carried across the pair: the second
            // game has to draw the same shuffle as the first, which a shared
            // generator would not do.
            let mut shuffle_rng = StdRng::seed_from_u64(seed);
            let template = if a_seat0 { &template_a0 } else { &template_b0 };
            let seated = if a_seat0 { [pilots[0], pilots[1]] } else { [pilots[1], pilots[0]] };
            // Both games of the pair get the *same* jitter stream, so the
            // bots' tie-breaks replay identically and the only thing left
            // that can separate the two games is the profiles themselves.
            let outcome =
                play_one_game(template, seated, max_actions, Some(&mut shuffle_rng), Some(seed));
            tally.cost.record(&outcome);
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
        }
    }
    tally
}

/// Sample a variant's spell/land split that always sums to the legal
/// 40-card sealed deck: spells from `spell_count_range`, lands = 40 −
/// spells (clamped into `land_count_range`, re-deriving spells from the
/// clamp if the config ranges don't line up). Independent sampling used
/// to produce 38-39-card decks, which are illegal AND thinner-deck
/// advantaged — they won tournaments on the bug.
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
            g.add_card_to_library(seat, f());
        }
        g.players[seat].wants_ui = true;
    }
    g
}

/// Consecutive no-move rounds before a bot play loop gives the game up.
/// Two bots that both pass priority forever produce one such round per
/// pass, so this is deliberately small. Shared with
/// [`crate::selfplay`]'s recording loop, which is otherwise a separate
/// hand-written copy of the same fixed point — the two stall counts are
/// only comparable if this number is.
pub const STALE_ROUNDS: usize = 8;

/// Why a game's play loop stopped. An undecided game is not one failure
/// mode but three, and the bench's `stalls` count merged them: a capped
/// game was doing work and ran out of budget, a stuck one had no bot able
/// to move at all, and a draw is a legitimate rules outcome. They want
/// different fixes, so the outcome says which.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopReason {
    /// `is_game_over()` — a winner, or an in-game draw (CR 104.4).
    GameOver,
    /// `actions >= max_actions`.
    ActionCap,
    /// [`STALE_ROUNDS`] consecutive rounds in which neither bot had an
    /// action accepted.
    NoLegalMove,
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
    play_one_game_traced(template, pilots, max_actions, shuffle_rng, jitter_seed, None)
}

/// A full game's observable history: one line per accepted action, each
/// pairing the action with the state it produced. Two engine builds that
/// agree on this agree on everything a game can be judged by.
///
/// Deliberately textual. A struct would be cheaper to compare and useless
/// to read: when a trace moves, the question is always *which* action
/// diverged and what the board looked like there, and a committed text
/// snapshot answers that from the diff alone.
pub struct GameTrace {
    pub lines: Vec<String>,
    pub winner: Option<usize>,
    pub turns: u32,
}

impl GameTrace {
    /// FNV-1a over the joined lines. Compact stand-in for the whole trace
    /// where committing the text would be all bulk and no signal.
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
        Some(&mut lines),
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

fn play_one_game_traced(
    template: &GameState,
    pilots: [Pilot; 2],
    max_actions: usize,
    shuffle_rng: Option<&mut StdRng>,
    jitter_seed: Option<u64>,
    mut trace: Option<&mut Vec<String>>,
) -> GameOutcome {
    // Installed for the duration of this game and cleared after, so a
    // seeded game can't leak its stream into whatever the worker plays
    // next. See `bot::set_jitter_seed`.
    crate::server::bot::set_jitter_seed(jitter_seed);
    let mut g = template.clone();
    // Profile settings the engine reads off the seat, not the bot.
    for (seat, pilot) in pilots.into_iter().enumerate() {
        if let Some(w) = pilot.weights() {
            g.players[seat].smart_tap = w.smart_tap;
        }
    }
    let mut seeded = shuffle_rng;
    for seat in 0..2 {
        match &mut seeded {
            Some(r) => g.players[seat].library.shuffle(*r),
            None => g.players[seat].library.shuffle(&mut rand::rng()),
        }
    }
    // A seeded deal implies a seeded *game*: mulligan reshuffles, random
    // discards and every other in-game roll come off the state's own
    // stream, drawn from the shuffle stream so it stays a function of the
    // caller's seed. Without this a mulligan re-randomized the deal and
    // the pair's two games diverged — which is also what broke the
    // antithetic pairing's variance reduction on exactly those games.
    if let Some(r) = &mut seeded {
        g.rng.reseed(r.random());
    }
    g.start_mulligan_phase();
    let mut bots: Vec<Box<dyn Bot>> = pilots.into_iter().map(Pilot::build).collect();
    let (mut actions, mut stale) = (0usize, 0usize);
    while !g.is_game_over() && actions < max_actions && stale < STALE_ROUNDS {
        let mut any = false;
        for (s, bot) in bots.iter_mut().enumerate() {
            let Some(a) = bot.next_action(&g, s) else { continue };
            // Formatted before the move, because `perform_action` takes it
            // by value; only kept when the move is accepted.
            let pending = trace.as_ref().map(|_| format!("p{s} {a:?}"));
            if g.perform_action(a).is_ok() {
                if let (Some(t), Some(p)) = (trace.as_mut(), pending) {
                    t.push(format!("{:>5}. {p}  ~~  {}", t.len() + 1, trace_state(&g)));
                }
                any = true;
                actions += 1;
                if g.is_game_over() {
                    break;
                }
            }
        }
        if any { stale = 0 } else { stale += 1 }
    }
    crate::server::bot::set_jitter_seed(None);
    // Tested in the loop's own order: `is_game_over` wins over both caps,
    // and a game that hits the action cap on the same iteration it goes
    // stale is reported as capped.
    let stop = if g.is_game_over() {
        StopReason::GameOver
    } else if actions >= max_actions {
        StopReason::ActionCap
    } else {
        StopReason::NoLegalMove
    };
    GameOutcome { winner: g.game_over.flatten(), actions, turns: g.turn_number, stop }
}

// ─────────────────────────────── evaluation ──────────────────────────────

/// Per-game outcomes keyed by (gauntlet opponent, game slot): +1 win,
/// −1 loss, 0 undecided. Slot indices are shared across candidates (CRN
/// seeds every candidate's slot identically), which is what makes two
/// candidates' outcomes pairable game-for-game.
pub type SlotOutcomes = HashMap<(u32, u32), i8>;

/// Paired win-rate difference a − b over the slots both sides decided.
pub struct PairedDiff {
    /// Shared decided slots.
    pub n: usize,
    /// Mean per-slot difference on the win-rate scale (−1 ..= 1).
    pub mean: f64,
    /// Standard error of `mean`, from the empirical per-slot variance.
    pub se: f64,
}

/// Compare two candidates game-for-game. On CRN slots this is the
/// variance-reduced comparison the shared shuffles pay for: concordant
/// slots (both win, or both lose) contribute zero variance, so the
/// error shrinks with the discordant count rather than the game count.
/// Still valid without CRN — it just degrades to an ordinary difference
/// test. `None` when no slot is decided for both.
pub fn paired_diff(a: &SlotOutcomes, b: &SlotOutcomes) -> Option<PairedDiff> {
    let (mut n, mut pos, mut neg) = (0usize, 0usize, 0usize);
    for (slot, &xa) in a {
        if xa == 0 {
            continue;
        }
        let Some(&xb) = b.get(slot) else { continue };
        if xb == 0 {
            continue;
        }
        n += 1;
        if xa > xb {
            pos += 1;
        } else if xa < xb {
            neg += 1;
        }
    }
    if n == 0 {
        return None;
    }
    let mean = (pos as f64 - neg as f64) / n as f64;
    let var = (pos + neg) as f64 / n as f64 - mean * mean;
    Some(PairedDiff { n, mean, se: (var.max(0.0) / n as f64).sqrt() })
}

/// Below this many shared decided slots the paired test yields to the
/// Wilson-bound comparison — too few slots for the empirical per-slot
/// variance to mean anything (smoke-test-sized configs live here).
const MIN_PAIRED_SLOTS: usize = 20;

struct EvalState {
    evals: Vec<CandidateEval>,
    slots: Vec<SlotOutcomes>,
}

struct Job {
    candidate: usize,
    opponent: usize,
    games: usize,
    /// Index of this chunk's first game within the (candidate, opponent)
    /// pairing's full schedule — the CRN seed component, so re-visiting a
    /// pairing in a later racing round plays *new* seeded games rather
    /// than replaying earlier ones.
    game_offset: usize,
}

fn worker_threads(cfg: &SimConfig) -> usize {
    if cfg.threads > 0 {
        return cfg.threads;
    }
    std::thread::available_parallelism().map(|n| n.get().saturating_sub(1)).unwrap_or(1).max(1)
}

/// Evaluate candidate decks against the gauntlet field in parallel.
///
/// Racing on: round r samples `min(gauntlet, 5·2^r)` opponents per active
/// candidate at `games_per_pairing` games each, then eliminates every
/// candidate sitting significantly below the current leader — by paired
/// per-slot comparison when enough shared decided slots exist (CRN makes
/// the slots genuinely paired), by Wilson-bound overlap otherwise.
/// Racing off: one full round-robin (every candidate × every gauntlet
/// deck).
///
/// `on_progress` is invoked (from worker threads) with the full eval
/// snapshot after every finished job — wire it to an `mpsc` sender for
/// live UI updates.
pub fn evaluate_candidates<F>(
    candidate_decks: &[Vec<CardFactory>],
    gauntlet: &[GauntletDeck],
    cfg: &SimConfig,
    on_progress: F,
) -> Vec<CandidateEval>
where
    F: Fn(&[CandidateEval]) + Sync,
{
    let prefill = vec![SlotOutcomes::new(); candidate_decks.len()];
    evaluate_candidates_slots(candidate_decks, gauntlet, cfg, &prefill, &on_progress).0
}

/// [`evaluate_candidates`] plus per-candidate slot outcomes, seeded from
/// `prefill`: any chunk of the schedule whose slots are all present in a
/// candidate's prefill map (this exact deck already played those seeded
/// games earlier in the session) is credited instantly instead of
/// simulated.
fn evaluate_candidates_slots<F>(
    candidate_decks: &[Vec<CardFactory>],
    gauntlet: &[GauntletDeck],
    cfg: &SimConfig,
    prefill: &[SlotOutcomes],
    on_progress: &F,
) -> (Vec<CandidateEval>, Vec<SlotOutcomes>)
where
    F: Fn(&[CandidateEval]) + Sync,
{
    let state = Mutex::new(EvalState {
        evals: (0..candidate_decks.len()).map(CandidateEval::new).collect(),
        slots: vec![SlotOutcomes::new(); candidate_decks.len()],
    });
    let mut active: Vec<usize> = (0..candidate_decks.len()).collect();
    let rounds: u32 = if cfg.racing { cfg.racing_rounds.max(1) } else { 1 };
    let threads = worker_threads(cfg);

    for round in 0..rounds {
        if active.len() <= 1 {
            break;
        }
        // Opponent sample for this round (racing widens it each round;
        // non-racing plays the whole field).
        let opps_this_round = if cfg.racing {
            (5usize << round).min(gauntlet.len())
        } else {
            gauntlet.len()
        };
        // Chunked jobs (~5 games each) keep tail latency low: workers
        // stay busy instead of waiting on the slowest big pairing.
        //
        // Every active candidate faces the SAME opponent subset in the
        // same game slots — the paired-comparison half of CRN. (The old
        // per-candidate rotation traded that away for subset diversity;
        // diversity comes from later rounds widening the subset instead.)
        // A pairing sampled since round `entry(opp)` has already played
        // `games_per_pairing` games per elapsed round — that's its offset.
        let entry = |opp: usize| -> u32 {
            if !cfg.racing {
                return 0;
            }
            let mut r = 0u32;
            while opp >= (5usize << r) {
                r += 1;
            }
            r
        };
        // 10-game chunks amortize the per-job match-template build while
        // keeping tail latency acceptable.
        const CHUNK: usize = 10;
        let mut jobs: Vec<Job> = Vec::new();
        let mut credited = false;
        {
            let mut st = state.lock().unwrap();
            for &cand in &active {
                for opp in 0..opps_this_round {
                    let base_offset =
                        cfg.games_per_pairing * (round.saturating_sub(entry(opp))) as usize;
                    let mut done = 0;
                    while done < cfg.games_per_pairing {
                        let n = (cfg.games_per_pairing - done).min(CHUNK);
                        let offset = base_offset + done;
                        // Fully cached chunk → replay the recorded outcomes.
                        // (Chunks are played atomically, so a partial hit
                        // only happens on a config change — resimulate.)
                        let cached = (0..n).all(|i| {
                            prefill[cand].contains_key(&(opp as u32, (offset + i) as u32))
                        });
                        if cached {
                            for i in 0..n {
                                let key = (opp as u32, (offset + i) as u32);
                                let o = prefill[cand][&key];
                                let e = &mut st.evals[cand];
                                match o {
                                    1 => e.wins += 1,
                                    -1 => e.losses += 1,
                                    _ => e.undecided += 1,
                                }
                                st.slots[cand].insert(key, o);
                            }
                            credited = true;
                        } else {
                            jobs.push(Job {
                                candidate: cand,
                                opponent: opp,
                                games: n,
                                game_offset: offset,
                            });
                        }
                        done += n;
                    }
                }
            }
        }
        if credited {
            let snapshot = state.lock().unwrap().evals.clone();
            on_progress(&snapshot);
        }
        let cursor = AtomicUsize::new(0);
        std::thread::scope(|s| {
            for _ in 0..threads {
                // Deep SOS effect trees overflow the 2MB spawn default under
                // the bot's dry-run recursion — give workers a roomy stack.
                let builder = std::thread::Builder::new().stack_size(32 * 1024 * 1024);
                builder
                    .spawn_scoped(s, || {
                    loop {
                        let i = cursor.fetch_add(1, Ordering::Relaxed);
                        let Some(job) = jobs.get(i) else { break };
                        // CRN seed: opponent + game slot, NEVER the
                        // candidate — identical across candidates by
                        // construction.
                        let seed_base = cfg.crn.then(|| {
                            cfg.seed
                                ^ (job.opponent as u64).wrapping_mul(0xA24B_AED4_963E_E407)
                                ^ (job.game_offset as u64)
                                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                        });
                        let tally = simulate_match_games(
                            &candidate_decks[job.candidate],
                            &gauntlet[job.opponent].cards,
                            job.games,
                            false,
                            cfg.uniform_opponent_bot,
                            cfg.max_actions_per_game,
                            seed_base,
                        );
                        let snapshot = {
                            let mut st = state.lock().unwrap();
                            {
                                let e = &mut st.evals[job.candidate];
                                e.wins += tally.wins_a;
                                e.losses += tally.wins_b;
                                e.undecided += tally.undecided;
                            }
                            for (i, &o) in tally.outcomes.iter().enumerate() {
                                st.slots[job.candidate].insert(
                                    (job.opponent as u32, (job.game_offset + i) as u32),
                                    o,
                                );
                            }
                            st.evals.clone()
                        };
                        on_progress(&snapshot);
                    }
                })
                    .expect("spawn recommender worker");
            }
        });

        // Successive halving: drop candidates significantly behind the
        // leader. Every active candidate has played the same slots, so
        // the paired test applies whenever the shared decided count is
        // meaningful; the Wilson-bound overlap is the fallback.
        if cfg.racing && round + 1 < rounds {
            let mut st = state.lock().unwrap();
            let EvalState { evals, slots } = &mut *st;
            let z = cfg.racing_confidence_z;
            let &leader = active
                .iter()
                .max_by(|&&a, &&b| {
                    evals[a]
                        .win_rate()
                        .partial_cmp(&evals[b].win_rate())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .expect("racing round has active candidates");
            let leader_lb =
                active.iter().map(|&i| evals[i].ci_bounds(z).0).fold(f64::MIN, f64::max);
            active.retain(|&i| {
                if i == leader {
                    return true;
                }
                let behind = match paired_diff(&slots[i], &slots[leader]) {
                    Some(pd) if pd.n >= MIN_PAIRED_SLOTS => pd.mean + z * pd.se < 0.0,
                    _ => evals[i].ci_bounds(z).1 < leader_lb,
                };
                if behind {
                    evals[i].eliminated_round = Some(round);
                }
                !behind
            });
        }
    }
    let st = state.into_inner().unwrap();
    (st.evals, st.slots)
}

/// One-shot [`Session::recommend`] over a throwaway session. Staged
/// callers (refine / local search) should hold a [`Session`] instead, so
/// the gauntlet and the outcome cache carry across stages.
pub fn recommend<F>(pool: &[CardFactory], cfg: &SimConfig, on_progress: F) -> Recommendation
where
    F: Fn(&[CandidateEval]) + Sync,
{
    Session::new(cfg.clone()).recommend(pool, on_progress)
}

/// One card's attribution across an evaluated fleet: mean win rate of the
/// builds playing it vs the builds benching it. Deck-level results can't
/// credit single cards; this can (noisily — mind the sample counts).
pub struct CardAttribution {
    pub name: &'static str,
    pub mean_in: f64,
    pub n_in: usize,
    pub mean_out: f64,
    pub n_out: usize,
    /// How many colour strata played *and* benched the card. Zero means
    /// there is no within-archetype comparison at all and
    /// [`delta`](Self::delta) is purely cross-archetype — see
    /// [`stratified_delta`](Self::stratified_delta).
    pub strata: usize,
    /// Attribution with the archetype confound removed: the per-stratum
    /// in-minus-out delta, pooled by inverse variance. `None` when no
    /// stratum has the card on both sides.
    pub stratified_delta: Option<f64>,
}

impl CardAttribution {
    /// The raw in-minus-out delta.
    ///
    /// **This is a cross-archetype marginal, not a card grade**, and it
    /// is the number that made Professor Dellian Fel read −2.4. A black
    /// card is played by the black builds and benched by every white,
    /// blue and red one, so its "out" group is a different deck rather
    /// than the same deck without it — the delta measures the archetype.
    /// Prefer [`stratified_delta`](Self::stratified_delta), which only
    /// compares builds of the same colours.
    pub fn delta(&self) -> f64 {
        self.mean_in - self.mean_out
    }

    /// The best available attribution: within-archetype where the samples
    /// support one, falling back to the raw marginal.
    pub fn best_delta(&self) -> f64 {
        self.stratified_delta.unwrap_or_else(|| self.delta())
    }
}

/// Per-card attribution over `(build, win rate)` samples. Only names
/// appearing in AND missing from at least `min_side` samples are
/// comparable (a card in every build has no counterfactual). Sorted by
/// descending delta.
pub fn per_card_attribution(
    samples: &[(&CandidateBuild, f64)],
    min_side: usize,
) -> Vec<CardAttribution> {
    let all_names: std::collections::HashSet<&'static str> = samples
        .iter()
        .flat_map(|(c, _)| c.main.iter().chain(c.duals.iter()).map(|&f| f().name))
        .collect();
    let mut per: HashMap<&'static str, (Vec<f64>, Vec<f64>)> = HashMap::new();
    for (c, wr) in samples {
        let in_deck: std::collections::HashSet<&'static str> =
            c.main.iter().chain(c.duals.iter()).map(|&f| f().name).collect();
        for name in &all_names {
            let e = per.entry(name).or_default();
            if in_deck.contains(name) { e.0.push(*wr) } else { e.1.push(*wr) }
        }
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;

    // Within-archetype comparison, keyed by the build's colour identity.
    // Splash is part of the key: "B/G" and "B/G + u" are different mana
    // bases and a card can be right in one and wrong in the other.
    let mut strata: HashMap<String, Vec<(&CandidateBuild, f64)>> = HashMap::new();
    for (c, wr) in samples {
        let mut key: Vec<String> = c.colors.iter().map(|x| format!("{x:?}")).collect();
        key.sort();
        let mut sp: Vec<String> = c.splash.iter().map(|x| format!("{x:?}")).collect();
        sp.sort();
        key.push(format!("+{}", sp.join("")));
        strata.entry(key.join("")).or_default().push((*c, *wr));
    }

    let mut rows: Vec<CardAttribution> = per
        .into_iter()
        .filter(|(_, (i, o))| i.len() >= min_side && o.len() >= min_side)
        .map(|(name, (i, o))| {
            // Pool the per-stratum deltas by inverse variance: a
            // difference of means over n_in and n_out samples has
            // variance proportional to 1/n_in + 1/n_out, so the weight is
            // the harmonic term below.
            let (mut num, mut den, mut used) = (0.0f64, 0.0f64, 0usize);
            for group in strata.values() {
                let (mut si, mut so) = (Vec::new(), Vec::new());
                for (c, wr) in group {
                    let plays =
                        c.main.iter().chain(c.duals.iter()).any(|&f| f().name == name);
                    if plays { si.push(*wr) } else { so.push(*wr) }
                }
                if si.len() < min_side || so.len() < min_side {
                    continue;
                }
                let w = (si.len() * so.len()) as f64 / (si.len() + so.len()) as f64;
                num += w * (mean(&si) - mean(&so));
                den += w;
                used += 1;
            }
            CardAttribution {
                name,
                mean_in: mean(&i),
                n_in: i.len(),
                mean_out: mean(&o),
                n_out: o.len(),
                strata: used,
                stratified_delta: (den > 0.0).then(|| num / den),
            }
        })
        .collect();
    // Ranked by the confound-free number where there is one, so a card
    // can no longer top (or bottom) the list purely for being the colour
    // the winning archetype happens to be.
    rows.sort_by(|a, b| {
        b.best_delta().partial_cmp(&a.best_delta()).unwrap_or(std::cmp::Ordering::Equal)
    });
    rows
}

/// [`per_card_attribution`] restricted to the samples whose build plays
/// `anchor` — the build-around lens. A card whose delta INSIDE anchor
/// decks beats its global delta is a synergy partner (Professor Dellian
/// Fel's emblem turning every lifegain trigger into a drain). Returns
/// the subset size alongside the rows so callers can report the sample.
pub fn per_card_attribution_within(
    samples: &[(&CandidateBuild, f64)],
    anchor: &str,
    min_side: usize,
) -> (usize, Vec<CardAttribution>) {
    // Case-insensitive: the anchor arrives from the CLI, and an exact
    // compare silently returned an empty subset for "professor dellian
    // fel" vs the printed "Professor Dellian Fel" — which then let a
    // cross-archetype marginal attribution stand in for the conditional
    // one. An empty-looking anchor should be loud, never a quiet no-op.
    let subset: Vec<(&CandidateBuild, f64)> = samples
        .iter()
        .filter(|(c, _)| {
            c.main
                .iter()
                .chain(c.duals.iter())
                .any(|&f| f().name.eq_ignore_ascii_case(anchor))
        })
        .map(|(c, w)| (*c, *w))
        .collect();
    let n = subset.len();
    (n, per_card_attribution(&subset, min_side))
}

/// A single-swap child of `parent`: `main[out_idx]` goes to the bench,
/// `in_card` comes off it, and the basics re-split for the new pips. The
/// spell count (and so the 40-card total) is unchanged by construction.
fn swap_child(parent: &CandidateBuild, out_idx: usize, in_card: CardFactory, label: String) -> CandidateBuild {
    let mut child = parent.clone();
    let removed = child.main[out_idx];
    child.main[out_idx] = in_card;
    if let Some(pos) = child.leftovers.iter().position(|&f| f as usize == in_card as usize) {
        child.leftovers.remove(pos);
    }
    child.leftovers.push(removed);
    // Land colors track the new main (a splash card swapped out may free
    // its basics; one swapped in needs a source).
    let mut land_colors = child.colors.clone();
    for &c in &child.splash {
        if child.main.iter().any(|&f| colors_of_cost(&f().cost).contains(&c)) {
            land_colors.push(c);
        }
    }
    let basic_total: u32 = parent.basics.values().sum();
    // Swap children re-split the basics of a build that already exists;
    // they inherit that build's mana model rather than re-deciding it.
    child.basics = basic_split(&child.main, &land_colors, basic_total, true);
    child.static_score = static_build_score(&child.main, child.main.len());
    child.label = label;
    child
}

// ─────────────────────────────── session ─────────────────────────────────

/// Canonical cache key for a full deck: sorted factory addresses.
fn deck_key(deck: &[CardFactory]) -> Vec<usize> {
    let mut k: Vec<usize> = deck.iter().map(|&f| f as usize).collect();
    k.sort_unstable();
    k
}

/// A recommender session: the gauntlet plus a per-deck outcome cache,
/// shared across pipeline stages. The staged flow re-races the same
/// decks repeatedly — the stage-1 winner returns as refine's v0, the
/// incumbent enters every search generation — and without the cache each
/// re-race would replay its exact seeded schedule from scratch. Instead,
/// outcomes are recorded per (deck, opponent, game slot) and replayed
/// when the same deck meets the same slot again.
pub struct Session {
    cfg: SimConfig,
    gauntlet: Vec<GauntletDeck>,
    cache: HashMap<Vec<usize>, SlotOutcomes>,
}

impl Session {
    /// Generate the gauntlet (fully determined by `cfg.seed`) once, up
    /// front — every stage of this session faces the same field.
    pub fn new(cfg: SimConfig) -> Self {
        let gauntlet = generate_gauntlet(&cfg);
        Self { cfg, gauntlet, cache: HashMap::new() }
    }

    pub fn cfg(&self) -> &SimConfig {
        &self.cfg
    }

    pub fn gauntlet(&self) -> &[GauntletDeck] {
        &self.gauntlet
    }

    /// Evaluate the first `cap` candidates against the session gauntlet,
    /// crediting cached outcomes and folding fresh ones back into the
    /// cache. Returns the recommendation plus the per-candidate slot
    /// maps (parallel to `evals`).
    fn eval_prepared(
        &mut self,
        candidates: Vec<CandidateBuild>,
        cap: usize,
        on_progress: &(impl Fn(&[CandidateEval]) + Sync),
    ) -> (Recommendation, Vec<SlotOutcomes>) {
        let top_k = candidates.len().min(cap);
        let decks: Vec<Vec<CardFactory>> = candidates[..top_k].iter().map(|c| c.deck()).collect();
        let keys: Vec<Vec<usize>> = decks.iter().map(|d| deck_key(d)).collect();
        let prefill: Vec<SlotOutcomes> =
            keys.iter().map(|k| self.cache.get(k).cloned().unwrap_or_default()).collect();
        let (evals, slots) =
            evaluate_candidates_slots(&decks, &self.gauntlet, &self.cfg, &prefill, on_progress);
        for (key, s) in keys.into_iter().zip(&slots) {
            self.cache.entry(key).or_default().extend(s.iter());
        }
        let mut ranking: Vec<usize> = (0..top_k).collect();
        ranking.sort_by(|&a, &b| {
            evals[b]
                .win_rate()
                .partial_cmp(&evals[a].win_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        (Recommendation { candidates, evals, ranking, seed: self.cfg.seed }, slots)
    }

    /// End-to-end stage 1: enumerate → static-rank → simulate the top K.
    pub fn recommend<F>(&mut self, pool: &[CardFactory], on_progress: F) -> Recommendation
    where
        F: Fn(&[CandidateEval]) + Sync,
    {
        let candidates = enumerate_candidates(pool, &self.cfg);
        self.recommend_prepared(candidates, on_progress)
    }

    /// Like [`Session::recommend`] but with a caller-supplied candidate
    /// list — the first `candidate_cap` entries are the ones simulated,
    /// so callers can reorder to pin builds the static rank would cut.
    pub fn recommend_prepared<F>(
        &mut self,
        candidates: Vec<CandidateBuild>,
        on_progress: F,
    ) -> Recommendation
    where
        F: Fn(&[CandidateEval]) + Sync,
    {
        let cap = self.cfg.candidate_cap;
        self.eval_prepared(candidates, cap, &on_progress).0
    }

    /// Stage-2 refinement: take the top `refine_top` shapes from a
    /// completed [`Recommendation`], generate `variants_per_shape` builds
    /// of each (the greedy build plus jittered rebuilds with sampled
    /// spell/land counts, deduplicated by contents), and race them
    /// against the session gauntlet — the stage-1 winner's v0 replays
    /// from cache. Variant labels carry a suffix ("U/B/G v3, 24+16").
    ///
    /// Coarse-to-fine on purpose: stage 1 answers "which colors", this
    /// answers "which 40 cards" — expanding variants for *every* shape
    /// would blow up the candidate list while still under-sampling the
    /// shapes that matter. Pair with `crn: true`; within-shape variant
    /// differences are small, and paired shuffles are what make them
    /// resolvable.
    pub fn refine<F>(
        &mut self,
        pool: &[CardFactory],
        base: &Recommendation,
        on_progress: F,
    ) -> Recommendation
    where
        F: Fn(&[CandidateEval]) + Sync,
    {
        let cfg = self.cfg.clone();
        let noise = (cfg.build_temperature.max(0.0) * 4.0).round() as i32;
        let mut variants: Vec<CandidateBuild> = Vec::new();
        let mut seen: std::collections::HashSet<Vec<usize>> = std::collections::HashSet::new();
        for &ci in base.ranking.iter().take(cfg.refine_top) {
            let shape = &base.candidates[ci];
            for v in 0..cfg.variants_per_shape.max(1) {
                let mut rng = StdRng::seed_from_u64(
                    cfg.seed
                        ^ (ci as u64).wrapping_mul(0xA24B_AED4_963E_E407)
                        ^ (v as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
                );
                // v0 = the shape's greedy build, verbatim; later variants
                // jitter picks and sample counts, with the jitter widening
                // as the fleet grows so big sweeps explore past the
                // near-greedy neighborhood instead of colliding into dedup.
                let (spells, lands, n) = if v == 0 {
                    (cfg.target_spells, cfg.total_lands, 0)
                } else {
                    let (s, l) = sample_deck_split(&cfg, &mut rng);
                    (s, l, noise + (v as i32 / 16) * 2)
                };
                let Some(mut build) = build_shape(
                    pool,
                    &shape.colors,
                    &shape.splash,
                    (spells, lands, n),
                    &cfg,
                    &mut rng,
                ) else {
                    continue;
                };
                // Dedup on the full 40 (main + lands), not just spells — a
                // variant differing only in land count is still a variant.
                if !seen.insert(deck_key(&build.deck())) {
                    continue;
                }
                if v > 0 {
                    build.label =
                        format!("{} v{v}, {}+{}", build.label, build.main.len(), lands);
                }
                variants.push(build);
            }
        }
        let cap = variants.len().max(1);
        self.eval_prepared(variants, cap, &on_progress).0
    }

    /// Stage-3 refinement: attribution-guided local search around the
    /// winning build. Each generation computes per-card attribution over
    /// *every* build measured so far, proposes children that swap the
    /// weakest in-deck cards for the strongest bench cards (plus seeded
    /// random swaps for exploration), and races children + incumbent
    /// against the session gauntlet — the incumbent's games replay from
    /// cache, and the comparison is paired game-for-game. A child is
    /// adopted only when it beats the incumbent on their shared slots at
    /// `search_accept_z` (a raw win-rate edge at these sample sizes is
    /// mostly noise, and chasing it walks the search randomly). Stops at
    /// `search_generations`, on a generation with no adopted child, or
    /// when no legal swap remains.
    ///
    /// This replaces "read the attribution table by hand and re-run with
    /// a pin": the gradient the table exposes is followed automatically.
    pub fn local_search<F>(&mut self, base: &Recommendation, on_progress: F) -> Recommendation
    where
        F: Fn(&[CandidateEval]) + Sync,
    {
        let cfg = self.cfg.clone();
        let mut incumbent: CandidateBuild = base.candidates[base.ranking[0]].clone();
        incumbent.label = format!("{} (incumbent)", incumbent.label);
        let mut samples: Vec<(CandidateBuild, f64)> = base.candidates[..base.evals.len()]
            .iter()
            .zip(&base.evals)
            .map(|(c, e)| (c.clone(), e.win_rate()))
            .collect();
        let mut last: Option<Recommendation> = None;
        for generation in 0..cfg.search_generations {
            // Which cards may come in: bench nonlands whose colors fit the
            // build, honoring the copy cap.
            let legal_in = |f: CardFactory, main: &[CardFactory]| -> bool {
                let def = f();
                if def.card_types.contains(&crate::card::CardType::Land) {
                    return false;
                }
                let cs = colors_of_cost(&def.cost);
                let fits = cs.is_empty()
                    || cs
                        .iter()
                        .all(|c| incumbent.colors.contains(c) || incumbent.splash.contains(c));
                fits && (main.iter().filter(|&&m| m as usize == f as usize).count() as u32)
                    < COPY_CAP
            };
            let refs: Vec<(&CandidateBuild, f64)> =
                samples.iter().map(|(c, w)| (c, *w)).collect();
            let attribution = per_card_attribution(&refs, 3);
            let delta_of = |f: CardFactory| -> f64 {
                attribution.iter().find(|a| a.name == f().name).map(|a| a.delta()).unwrap_or(0.0)
            };
            // Candidate swaps: every (weak in-deck, strong bench) pair ranked
            // by expected gain, then seeded random swaps to keep exploring
            // when the gradient runs dry.
            let mut proposals: Vec<(usize, CardFactory, f64)> = Vec::new();
            for (i, &out_card) in incumbent.main.iter().enumerate() {
                for &in_card in &incumbent.leftovers {
                    if in_card as usize == out_card as usize
                        || !legal_in(in_card, &incumbent.main)
                    {
                        continue;
                    }
                    let gain = delta_of(in_card) - delta_of(out_card);
                    proposals.push((i, in_card, gain));
                }
            }
            proposals
                .sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            let mut rng = StdRng::seed_from_u64(
                cfg.seed ^ (generation as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93),
            );
            let explore = cfg.search_children / 4;
            let mut children: Vec<CandidateBuild> = Vec::new();
            let mut seen: std::collections::HashSet<Vec<usize>> =
                std::collections::HashSet::new();
            seen.insert(deck_key(&incumbent.main));
            // Gradient children first (positive expected gain only), then
            // random exploration swaps.
            for &(i, in_card, gain) in &proposals {
                if children.len() >= cfg.search_children.saturating_sub(explore) || gain <= 0.0 {
                    break;
                }
                let child = swap_child(
                    &incumbent,
                    i,
                    in_card,
                    format!("g{generation} s{}", children.len()),
                );
                if seen.insert(deck_key(&child.main)) {
                    children.push(child);
                }
            }
            for _ in 0..cfg.search_children * 4 {
                if children.len() >= cfg.search_children || proposals.is_empty() {
                    break;
                }
                let &(i, in_card, _) = &proposals[rng.random_range(0..proposals.len())];
                let child = swap_child(
                    &incumbent,
                    i,
                    in_card,
                    format!("g{generation} x{}", children.len()),
                );
                if seen.insert(deck_key(&child.main)) {
                    children.push(child);
                }
            }
            if children.is_empty() {
                break;
            }
            let mut cands = vec![incumbent.clone()];
            cands.extend(children);
            let cap = cands.len();
            let (rec, slot_maps) = self.eval_prepared(cands, cap, &on_progress);
            samples.extend(
                rec.candidates[..rec.evals.len()]
                    .iter()
                    .zip(&rec.evals)
                    .map(|(c, e)| (c.clone(), e.win_rate())),
            );
            let best = rec.ranking[0];
            // Paired acceptance: the top child must beat the incumbent on
            // their shared game slots at `search_accept_z`, not merely post
            // a higher raw win rate — an unpaired nominal edge at these
            // sample sizes is mostly noise, and chasing it walks the search
            // randomly. Tiny overlaps (smoke-test configs) fall back to the
            // raw comparison.
            let improved = best != 0
                && match paired_diff(&slot_maps[best], &slot_maps[0]) {
                    Some(pd) if pd.n >= MIN_PAIRED_SLOTS => {
                        pd.mean - cfg.search_accept_z * pd.se > 0.0
                    }
                    _ => rec.evals[best].win_rate() > rec.evals[0].win_rate(),
                };
            if improved {
                incumbent = rec.candidates[best].clone();
                incumbent.label = format!("{} (incumbent)", incumbent.label);
            }
            last = Some(rec);
            if !improved {
                break;
            }
        }
        last.unwrap_or_else(|| Recommendation {
            candidates: vec![incumbent],
            evals: vec![base.evals[base.ranking[0]].clone()],
            ranking: vec![0],
            seed: cfg.seed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    /// Every pair is two games, and only fully decided pairs are scored.
    /// The unpaired totals stay the ground truth for "how many games did
    /// we actually play", which is what the ladder's control column reads.
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

    /// Antithetic pairing is only worth its complexity if the two games of
    /// a pair really are the same deal. They are unobservable from the
    /// tally directly, so this asserts the consequence: a profile played
    /// against *itself* splits its pairs far more often than independent
    /// games would, because whichever side the deal favoured wins its copy
    /// in each direction.
    ///
    /// Not asserted at 100%: the scored bot jitters ties from an unseeded
    /// RNG (`main_phase_action_with`), so a pair can still diverge. That
    /// residual is exactly why the ladder measures the realized rho
    /// instead of assuming the ideal one.
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

    /// The Dellian Fel defect, reproduced in miniature.
    ///
    /// Two archetypes: a strong one (65 % base) and a weak one (45 %).
    /// The card under test is genuinely *bad* — it costs 4 points
    /// wherever it appears — but it only appears in the strong
    /// archetype's builds. The raw marginal therefore reports it as a
    /// star, because its "out" group is mostly the weak archetype. The
    /// stratified delta, comparing only same-colour builds, sees the
    /// truth.
    #[test]
    fn stratified_attribution_removes_the_archetype_confound() {
        let mut samples_owned: Vec<(CandidateBuild, f64)> = Vec::new();
        for i in 0..8 {
            // Strong archetype; half of them play the card.
            let plays = i % 2 == 0;
            let mut main = vec![catalog::grizzly_bears as CardFactory; 20];
            if plays {
                main.push(catalog::craw_wurm);
            }
            let wr = 0.65 - if plays { 0.04 } else { 0.0 };
            samples_owned.push((build_with(vec![Color::Red, Color::White], main), wr));
        }
        for _ in 0..8 {
            // Weak archetype; never plays the card.
            let main = vec![catalog::grizzly_bears as CardFactory; 20];
            samples_owned.push((build_with(vec![Color::Blue, Color::Black], main), 0.45));
        }
        let samples: Vec<(&CandidateBuild, f64)> =
            samples_owned.iter().map(|(c, w)| (c, *w)).collect();

        let rows = per_card_attribution(&samples, 3);
        let wurm = rows
            .iter()
            .find(|r| r.name == "Craw Wurm")
            .expect("the card under test should be comparable");

        assert!(
            wurm.delta() > 0.0,
            "the raw marginal should be fooled into liking it, got {:+.3}",
            wurm.delta()
        );
        let within = wurm.stratified_delta.expect("one stratum plays and benches it");
        assert!(
            (within + 0.04).abs() < 1e-9,
            "within-archetype delta should recover the true −4 points, got {within:+.3}"
        );
        assert_eq!(wurm.strata, 1, "only the R/W stratum has it on both sides");
    }

    /// A card no single archetype both plays and benches has no
    /// within-archetype comparison at all, and must say so rather than
    /// quietly reporting the marginal as if it were one.
    #[test]
    fn a_card_confined_to_one_archetype_reports_no_strata() {
        let mut samples_owned: Vec<(CandidateBuild, f64)> = Vec::new();
        for _ in 0..6 {
            let mut main = vec![catalog::grizzly_bears as CardFactory; 20];
            main.push(catalog::craw_wurm);
            samples_owned.push((build_with(vec![Color::Green], main), 0.6));
        }
        for _ in 0..6 {
            let main = vec![catalog::grizzly_bears as CardFactory; 20];
            samples_owned.push((build_with(vec![Color::Blue], main), 0.4));
        }
        let samples: Vec<(&CandidateBuild, f64)> =
            samples_owned.iter().map(|(c, w)| (c, *w)).collect();
        let rows = per_card_attribution(&samples, 3);
        let wurm = rows.iter().find(|r| r.name == "Craw Wurm").unwrap();
        assert_eq!(wurm.strata, 0);
        assert!(wurm.stratified_delta.is_none());
        // ...and `best_delta` falls back rather than pretending.
        assert!((wurm.best_delta() - wurm.delta()).abs() < 1e-12);
    }

    fn build_with(colors: Vec<Color>, main: Vec<CardFactory>) -> CandidateBuild {
        CandidateBuild {
            colors,
            splash: Vec::new(),
            main,
            duals: Vec::new(),
            basics: HashMap::new(),
            leftovers: Vec::new(),
            static_score: 0,
            label: String::new(),
        }
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
        // …and the double-green six-drop in the same pool is left out of
        // it. A splash is a couple of off-color sources: it can support
        // {1}{G}, never {4}{G}{G}. Leaving that distinction out is what
        // put a {3}{U}{U} bomb in a deck with three Islands.
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

    /// Every variant and gauntlet deck is a legal 40-card sealed deck —
    /// independent spell/land sampling used to mint 38-39-card decks,
    /// which won tournaments on thinner-deck consistency.
    #[test]
    fn variants_and_gauntlet_decks_are_forty_cards()  {
        let cfg = SimConfig {
            gauntlet_size: 4,
            games_per_pairing: 1,
            candidate_cap: 1,
            racing: false,
            threads: 2,
            refine_top: 1,
            variants_per_shape: 12,
            ..Default::default()
        };
        for deck in generate_gauntlet(&cfg) {
            assert_eq!(deck.cards.len(), 40, "gauntlet deck {} is 40 cards", deck.label);
        }
        let pool = wr_pool_with_green_bomb();
        let mut session = Session::new(cfg);
        let base = session.recommend(&pool, |_| {});
        let refined = session.refine(&pool, &base, |_| {});
        for c in &refined.candidates {
            assert_eq!(c.deck().len(), 40, "variant {} is 40 cards", c.label);
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
    fn gauntlet_is_seed_deterministic() {
        let cfg = SimConfig { gauntlet_size: 3, ..Default::default() };
        let names = |g: &[GauntletDeck]| -> Vec<Vec<&'static str>> {
            g.iter().map(|d| d.cards.iter().map(|&f| f().name).collect()).collect()
        };
        let a = generate_gauntlet(&cfg);
        let b = generate_gauntlet(&cfg);
        assert_eq!(names(&a), names(&b), "same seed → identical gauntlet");
        let c = generate_gauntlet(&SimConfig { seed: 1234, ..cfg });
        assert_ne!(names(&a), names(&c), "different seed → different gauntlet");
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

    /// CRN: with a fixed seed base, the shuffle streams are reproducible —
    /// two identical simulate calls give identical tallies when both bots
    /// are deterministic-ish enough to be draw-dominated. We assert the
    /// weaker structural property that holds regardless of bot jitter:
    /// deck-A-vs-land-pile is decided purely by A's draws, so the same
    /// seeds give the same tally.
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

    /// Refinement: variants of the top shapes are generated, deduped, and
    /// ranked; v0 of the top shape reproduces the stage-1 build.
    #[test]
    fn refine_generates_ranked_variants_of_top_shapes() {
        let cfg = SimConfig {
            gauntlet_size: 2,
            games_per_pairing: 1,
            candidate_cap: 2,
            racing: false,
            threads: 2,
            refine_top: 2,
            variants_per_shape: 3,
            ..Default::default()
        };
        let pool = wr_pool_with_green_bomb();
        let mut session = Session::new(cfg);
        let base = session.recommend(&pool, |_| {});
        let refined = session.refine(&pool, &base, |_| {});
        assert!(
            refined.candidates.len() >= 2,
            "at least the two shapes' greedy builds survive dedup",
        );
        assert_eq!(refined.ranking.len(), refined.evals.len());
        // v0 of the stage-1 winner is reproduced verbatim.
        let winner = &base.candidates[base.ranking[0]];
        let v0 = &refined.candidates[0];
        assert_eq!(v0.label, winner.label, "variant 0 keeps the plain shape label");
        let names = |b: &CandidateBuild| -> Vec<&'static str> {
            let mut n: Vec<&'static str> = b.deck().iter().map(|&f| f().name).collect();
            n.sort_unstable();
            n
        };
        assert_eq!(names(v0), names(winner), "variant 0 is the stage-1 build");
        // Jittered variants carry the suffix label.
        assert!(
            refined.candidates.iter().any(|c| c.label.contains(" v")),
            "at least one jittered variant exists",
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

    /// `builder_v2`'s mana model: a double pip needs more than twice one
    /// pip's sources, so four {3}{W}{W} bodies outweigh four one-mana
    /// {R} spells even though the red pips are earlier. This is the
    /// deliberate reversal of `basic_split_leans_on_cheap_pips` above —
    /// the builder that produced three Islands for a {3}{U}{U} bomb was
    /// counting pips linearly.
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

    /// Local search: terminates, every candidate stays a legal 40-card
    /// deck, and the incumbent is always in the raced set.
    #[test]
    fn local_search_produces_legal_swaps() {
        let cfg = SimConfig {
            gauntlet_size: 2,
            games_per_pairing: 1,
            candidate_cap: 4,
            racing: false,
            threads: 2,
            refine_top: 2,
            variants_per_shape: 4,
            search_generations: 1,
            search_children: 3,
            ..Default::default()
        };
        let pool = wr_pool_with_green_bomb();
        let mut session = Session::new(cfg);
        let base = session.recommend(&pool, |_| {});
        let refined = session.refine(&pool, &base, |_| {});
        let searched = session.local_search(&refined, |_| {});
        assert!(!searched.candidates.is_empty());
        assert!(searched.candidates[0].label.contains("(incumbent)"));
        for c in &searched.candidates {
            assert_eq!(c.deck().len(), 40, "swap child {} stays 40 cards", c.label);
        }
    }

    /// End-to-end smoke: tiny config, racing off, two threads — must
    /// terminate with a populated ranking and progress callbacks fired.
    #[test]
    fn recommend_end_to_end_smoke() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        // Kept intentionally tiny — this is a wiring smoke test, not a
        // ranking-quality test; real runs use SimConfig::default().
        let cfg = SimConfig {
            gauntlet_size: 2,
            games_per_pairing: 1,
            candidate_cap: 2,
            racing: false,
            threads: 2,
            ..Default::default()
        };
        let progress_calls = AtomicUsize::new(0);
        let rec = recommend(&wr_pool_with_green_bomb(), &cfg, |_evals| {
            progress_calls.fetch_add(1, Ordering::Relaxed);
        });
        assert_eq!(rec.evals.len(), 2, "top-K candidates evaluated");
        assert_eq!(rec.ranking.len(), 2);
        assert!(progress_calls.load(Ordering::Relaxed) > 0, "progress streamed");
        let games: u32 = rec.evals.iter().map(|e| e.decided() + e.undecided).sum();
        assert_eq!(games as usize, 2 * 2 * cfg.games_per_pairing, "full round-robin game count");
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

    /// All splits is the signature of a perfectly cancelled deal: the
    /// estimate is dead even and, crucially, has *zero* width. Independent
    /// games can never report that, which is the whole point.
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

    /// The failure mode worth being able to see: every pair a sweep means
    /// the two games of a pair agreed *more* than chance, so the pairing
    /// inflated variance instead of cancelling it (rho = +1, half the
    /// precision of independent games). If a real run ever printed this,
    /// pairing would be costing us games rather than saving them.
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
    /// interval. The normal approximation gave ±0 at p̂ = 1, so a lucky
    /// 5-0 candidate claimed lower bound 1.0 and racing eliminated the
    /// entire rest of the field in round 0.
    #[test]
    fn wilson_interval_is_not_degenerate_at_extremes() {
        let mut e = CandidateEval::new(0);
        e.wins = 5;
        let (lb, ub) = e.ci_bounds(1.96);
        assert!(lb < 1.0, "5-0 keeps a lower bound below certainty, got {lb}");
        assert!(ub <= 1.0);
        assert!(e.ci_halfwidth(1.96) > 0.05, "5-0 keeps honest width");
        e.wins = 0;
        e.losses = 5;
        let (lb, ub) = e.ci_bounds(1.96);
        assert!(lb < 0.01);
        assert!(ub > 0.0 && ub < 1.0, "0-5 upper bound stays above zero, got {ub}");
    }

    /// Paired comparison: concordant slots contribute no variance, so a
    /// candidate losing every discordant slot is significantly behind
    /// even when the records look close unpaired; unshared and undecided
    /// slots are excluded.
    #[test]
    fn paired_diff_uses_shared_decided_slots_only() {
        let mut a = SlotOutcomes::new();
        let mut b = SlotOutcomes::new();
        for i in 0..30u32 {
            a.insert((0, i), 1);
            b.insert((0, i), 1); // concordant wins: no variance
        }
        for i in 30..40u32 {
            a.insert((0, i), -1);
            b.insert((0, i), 1); // b wins every discordant slot
        }
        a.insert((1, 0), 1); // unshared → ignored
        a.insert((0, 40), 1);
        b.insert((0, 40), 0); // undecided on one side → ignored
        let pd = paired_diff(&a, &b).unwrap();
        assert_eq!(pd.n, 40);
        assert!((pd.mean + 0.25).abs() < 1e-9, "mean −10/40, got {}", pd.mean);
        assert!(pd.mean + 1.96 * pd.se < 0.0, "a is significantly behind b");
        assert!(paired_diff(&SlotOutcomes::new(), &b).is_none(), "no shared slots → None");
    }

    /// Session cache: the same deck re-raced in a later stage replays its
    /// recorded outcomes instead of re-simulating — identical evals (bot
    /// jitter is unseeded, so fresh games would drift) and no per-job
    /// progress callbacks, just the one credit snapshot per round.
    #[test]
    fn session_cache_replays_previous_outcomes() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let cfg = SimConfig {
            gauntlet_size: 2,
            games_per_pairing: 2,
            candidate_cap: 2,
            racing: false,
            threads: 2,
            ..Default::default()
        };
        let pool = wr_pool_with_green_bomb();
        let candidates = enumerate_candidates(&pool, &cfg);
        let mut session = Session::new(cfg);
        let first_calls = AtomicUsize::new(0);
        let a = session.recommend_prepared(candidates.clone(), |_| {
            first_calls.fetch_add(1, Ordering::Relaxed);
        });
        let second_calls = AtomicUsize::new(0);
        let b = session.recommend_prepared(candidates, |_| {
            second_calls.fetch_add(1, Ordering::Relaxed);
        });
        let stats = |r: &Recommendation| -> Vec<(u32, u32, u32)> {
            r.evals.iter().map(|e| (e.wins, e.losses, e.undecided)).collect()
        };
        assert_eq!(stats(&a), stats(&b), "second run replays cached outcomes verbatim");
        assert_eq!(
            second_calls.load(Ordering::Relaxed),
            1,
            "fully cached run credits in one snapshot, no simulation jobs",
        );
        assert!(first_calls.load(Ordering::Relaxed) > 1, "first run actually simulated");
    }
}
