//! 8-player Crabomination booster draft against bots.
//!
//! The draft runs off-engine: no `GameState`, stack or server actor. Packs,
//! bot picks and each seat's 45-card pile are plain values. The client drives
//! the loop (the user picks interactively, `bot_pick` runs for the other 7
//! seats), then `build_draft_match_state` builds a 2-player `GameState` from
//! the chosen decks.
//!
//! Pool: the cube pool (`cube::all_cube_cards()`), sampled uniformly with
//! replacement across packs and without duplicates inside one.
//!
//! Bot picks: a color-fit + curve heuristic (`score_card_for_seat`) against
//! the seat's picks so far, greedy, ties broken on mana value. No synergy or
//! archetype detection.

use crate::fxhash::HashMap;

use rand::{Rng, RngExt};
use rand::seq::SliceRandom;

use crate::card::{CreatureType, Keyword};
use crate::cube::{CardFactory, all_cube_cards};
use crate::game::GameState;
use crate::mana::{Color, ManaSymbol};
use crate::player::Player;

/// Number of seats in a draft pod. Standard MTG draft size.
pub const POD_SIZE: usize = 8;
/// Cards per pack.
pub const PACK_SIZE: usize = 15;
/// Packs each seat opens during the draft.
pub const PACKS_PER_SEAT: u32 = 3;
/// Max copies of any single card across a built deck. Matches `cube.rs`.
pub const COPY_CAP: u32 = 4;

/// Generate one 15-card pack from `pool`: no duplicates within the pack, but
/// a card can recur across packs. Returns fewer than `PACK_SIZE` cards only
/// when `pool.len() < PACK_SIZE`.
pub fn generate_pack<R: Rng>(pool: &[CardFactory], rng: &mut R) -> Vec<CardFactory> {
    if pool.is_empty() {
        return Vec::new();
    }
    let want = PACK_SIZE.min(pool.len());
    let mut pack: Vec<CardFactory> = Vec::with_capacity(want);
    let mut used: crate::fxhash::HashSet<usize> = crate::fxhash::HashSet::default();
    while pack.len() < want {
        let idx = rng.random_range(0..pool.len());
        if used.insert(idx) {
            pack.push(pool[idx]);
        }
    }
    pack
}

/// Color-identity bucket `generate_sos_pack` stratifies a Secrets of
/// Strixhaven pack by. `CardDefinition` has no rarity, so booster shape is
/// approximated by spreading cards across mono-colors, multicolor and
/// colorless/land.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SosBucket {
    Mono(Color),
    Multi,
    ColorlessOrLand,
}

/// One booster in this many carries a card off the Special Guests sheet
/// (`sos_mode::sos_special_guests`), the printed rate; most 24-pack pods see
/// none.
pub const SOS_SPECIAL_GUEST_RATE: u32 = 64;

/// Per-pack quota for each bucket, summing to 15: two mono of each color,
/// three multicolor, two colorless/land.
const SOS_PACK_RECIPE: &[(SosBucket, usize)] = &[
    (SosBucket::Mono(Color::White), 2),
    (SosBucket::Mono(Color::Blue), 2),
    (SosBucket::Mono(Color::Black), 2),
    (SosBucket::Mono(Color::Red), 2),
    (SosBucket::Mono(Color::Green), 2),
    (SosBucket::Multi, 3),
    (SosBucket::ColorlessOrLand, 2),
];

/// Classify a card into its `SosBucket` by color identity. Hybrid pips count
/// both halves, Phyrexian pips their colored half; lands and pipless cards
/// are `ColorlessOrLand`.
fn sos_bucket_of(def: &crate::card::CardDefinition) -> SosBucket {
    use crate::card::CardType;
    use crate::mana::ManaSymbol;
    if def.card_types.contains(&CardType::Land) {
        return SosBucket::ColorlessOrLand;
    }
    let mut seen = [false; 5];
    let idx = |c: Color| match c {
        Color::White => 0,
        Color::Blue => 1,
        Color::Black => 2,
        Color::Red => 3,
        Color::Green => 4,
    };
    for sym in &def.cost.symbols {
        match sym {
            ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => seen[idx(*c)] = true,
            ManaSymbol::Hybrid(a, b) => {
                seen[idx(*a)] = true;
                seen[idx(*b)] = true;
            }
            _ => {}
        }
    }
    let n_colors = seen.iter().filter(|&&b| b).count();
    match n_colors {
        0 => SosBucket::ColorlessOrLand,
        1 => SosBucket::Mono(
            Color::ALL
                .into_iter()
                .find(|c| seen[idx(*c)])
                .expect("n_colors==1 implies one color set"),
        ),
        _ => SosBucket::Multi,
    }
}

/// Stratified pack generator for the SoS pool, following `SOS_PACK_RECIPE`:
/// no duplicates within a pack, recurrence allowed across packs. A short
/// bucket is filled from `SosBucket::Multi`, then from any unused pool card,
/// so a pack stays at `PACK_SIZE` on small subset pools.
pub fn generate_sos_pack<R: Rng>(pool: &[CardFactory], rng: &mut R) -> Vec<CardFactory> {
    SosPacks::new(pool).roll(rng)
}

/// The pool-only part of [`generate_sos_pack`] (Special Guests indices and
/// each other card's colour bucket), built once and rolled from per pack
/// instead of re-derived each time. Rolls draw from `rng` in the same order
/// as the per-pack derivation did.
pub struct SosPacks<'a> {
    pool: &'a [CardFactory],
    /// Pool indices on the Special Guests sheet, kept indexable because the
    /// guest slot picks one at random.
    guests: Vec<usize>,
    by_bucket: crate::fxhash::HashMap<SosBucket, Vec<usize>>,
}

impl SosPacks<'static> {
    /// The pack roller for [`sos_draft_pool_ref`], built once per process.
    ///
    /// `new` is a whole-pool walk and the pool is a constant, so one roller
    /// serves every build (PERF (-66)).
    pub fn sos_default() -> &'static SosPacks<'static> {
        static PACKS: std::sync::OnceLock<SosPacks<'static>> = std::sync::OnceLock::new();
        PACKS.get_or_init(|| SosPacks::new(sos_draft_pool_ref()))
    }
}

impl<'a> SosPacks<'a> {
    pub fn new(pool: &'a [CardFactory]) -> Self {
        // The Special Guests sheet is collated separately (see
        // `SOS_SPECIAL_GUEST_RATE`), so keep it out of the colour buckets.
        let sheet: crate::fxhash::HashSet<usize> = crate::sos_mode::sos_special_guests()
            .into_iter()
            .map(|f| f as usize)
            .collect();
        let mut guests: Vec<usize> = Vec::new();
        let mut by_bucket: crate::fxhash::HashMap<SosBucket, Vec<usize>> =
            crate::fxhash::HashMap::default();
        for (i, factory) in pool.iter().enumerate() {
            if sheet.contains(&(*factory as usize)) {
                guests.push(i);
                continue;
            }
            let def = crate::cube::card_def(*factory);
            by_bucket.entry(sos_bucket_of(def)).or_default().push(i);
        }
        Self { pool, guests, by_bucket }
    }

    /// Roll one pack.
    pub fn roll<R: Rng>(&self, rng: &mut R) -> Vec<CardFactory> {
        let (pool, guests, by_bucket) = (self.pool, &self.guests, &self.by_bucket);
        if pool.is_empty() {
            return Vec::new();
        }
        let mut used: crate::fxhash::HashSet<usize> = crate::fxhash::HashSet::default();
        let mut pack: Vec<CardFactory> = Vec::with_capacity(PACK_SIZE);
        let pull_from = |bucket: SosBucket,
                         want: usize,
                         by_bucket: &crate::fxhash::HashMap<SosBucket, Vec<usize>>,
                         used: &mut crate::fxhash::HashSet<usize>,
                         pack: &mut Vec<CardFactory>,
                         rng: &mut R|
         -> usize {
            let Some(indices) = by_bucket.get(&bucket) else {
                return 0;
            };
            let mut taken = 0;
            let mut attempts = 0;
            // Cap attempts so a near-empty bucket can't infinite-loop.
            let max_attempts = indices.len().saturating_mul(4).max(8);
            while taken < want && attempts < max_attempts {
                attempts += 1;
                let pick = indices[rng.random_range(0..indices.len())];
                if used.insert(pick) {
                    pack.push(pool[pick]);
                    taken += 1;
                }
            }
            taken
        };
        let mut shortfall = 0usize;
        for (bucket, want) in SOS_PACK_RECIPE {
            let got = pull_from(*bucket, *want, by_bucket, &mut used, &mut pack, rng);
            shortfall += want.saturating_sub(got);
        }
        // Shortfall from Multi, then any unused pool card: the full SoS pool
        // always satisfies the recipe, a custom subset might not.
        if shortfall > 0 {
            pull_from(
                SosBucket::Multi,
                shortfall,
                by_bucket,
                &mut used,
                &mut pack,
                rng,
            );
        }
        while pack.len() < PACK_SIZE.min(pool.len()) {
            let idx = rng.random_range(0..pool.len());
            if used.insert(idx) {
                pack.push(pool[idx]);
            }
        }
        // The Special Guest, when it shows, replaces the pack's last slot.
        if !guests.is_empty()
            && !pack.is_empty()
            && rng.random_range(0..SOS_SPECIAL_GUEST_RATE) == 0
        {
            let last = pack.len() - 1;
            pack[last] = pool[guests[rng.random_range(0..guests.len())]];
        }
        pack
    }
}

impl DraftPool {
    /// Roll one 15-card pack: uniform for Cube (`generate_pack`), stratified
    /// by color for SoS (`generate_sos_pack`).
    pub fn generate_pack<R: Rng>(self, pool: &[CardFactory], rng: &mut R) -> Vec<CardFactory> {
        match self {
            DraftPool::Cube => generate_pack(pool, rng),
            DraftPool::Sos => generate_sos_pack(pool, rng),
        }
    }
}

/// All `POD_SIZE * PACKS_PER_SEAT` packs for a pod, in deal order
/// `[seat0_pack1, seat1_pack1, …, seat0_pack2, …]`.
pub fn open_all_packs<R: Rng>(pool: &[CardFactory], rng: &mut R) -> Vec<Vec<CardFactory>> {
    let mut all = Vec::with_capacity(POD_SIZE * PACKS_PER_SEAT as usize);
    for _ in 0..(POD_SIZE * PACKS_PER_SEAT as usize) {
        all.push(generate_pack(pool, rng));
    }
    all
}

/// Score `factory` for a seat with `seat_picks_so_far` (higher is better).
/// Deliberately simple, no synergy model:
/// - color fit: on-color pips earn a bonus, off-color are penalized,
///   colorless is neutral;
/// - card type: creatures and instants/sorceries edge out other types;
/// - mana value: a mild preference for 2-4; lands score low since the
///   builder adds basics later.
pub fn score_card_for_seat(factory: CardFactory, seat_picks_so_far: &[CardFactory]) -> i32 {
    score_card_with_colors(factory, &colors_of_picks(seat_picks_so_far))
}

/// What a card is worth as a card, ignoring its colors: body size,
/// evasion/combat keywords, and SOS preparation (a second spell).
///
/// [`score_card_with_colors`] has none of this (it ranks a {3}{U}{U} 5/5
/// ward flier below a vanilla two-drop), which left bombs on the bench; see
/// [`score_card_quality`]. Magnitudes are kept comparable to the color term
/// (6 per on-color pip): quality may outrank a pip of fit, not overrule
/// color discipline.
pub(crate) fn card_quality(def: &crate::card::CardDefinition) -> i32 {
    use crate::card::{CardType, Keyword};
    let mut q = 0i32;
    if def.card_types.contains(&CardType::Creature) {
        // Halved and capped: a 5/5 is worth about a pip of color fit, a
        // 7/7 not much more.
        q += ((def.power.max(0) + def.toughness.max(0)) / 2).min(5);
        let p = def.power.max(0);
        let has = |k: &Keyword| def.keywords.contains(k);
        // Evasion is what turns a body into a clock; deathtouch and
        // lifelink change how profitably it fights.
        if has(&Keyword::Flying) || has(&Keyword::Shadow) || has(&Keyword::Unblockable) {
            q += if p >= 2 { 3 } else { 2 };
        } else if has(&Keyword::Menace) || has(&Keyword::Trample) {
            q += 1;
        }
        if has(&Keyword::Deathtouch) {
            q += 2;
        }
        if has(&Keyword::Lifelink) {
            q += 1;
        }
        if has(&Keyword::Defender) {
            q -= 2;
        }
        // Ward taxes the removal that would otherwise answer the body.
        if def.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))) {
            q += 1;
        }
    }
    // A preparation card is two cards in one slot (the creature plus the
    // spell it casts off itself).
    if def.prepare_spell.is_some() {
        q += 3;
    }
    q
}

/// [`score_card_with_colors`] plus [`card_quality`] — the sealed
/// builder's scorer when `SimConfig::builder_v2` is on. The legacy
/// scorer is kept intact as the measurement control.
pub(crate) fn score_card_quality(
    factory: CardFactory,
    seat_colors: &ColorCounts,
) -> i32 {
    score_brief_quality(crate::cube::card_brief(factory), seat_colors)
}

/// [`score_card_quality`] for a caller that is already holding the brief.
pub(crate) fn score_brief_quality(
    brief: &crate::cube::CardBrief,
    seat_colors: &ColorCounts,
) -> i32 {
    score_brief_with_colors(brief, seat_colors) + brief.quality
}

/// How much an aggressive build wants a card *for its cost*, on top of
/// [`score_brief_quality`].
///
/// `base_score`'s curve weight spans three points against a `quality` term
/// reaching ~15, so the builder is curve-blind and ranks a one-drop below a
/// five-drop. This adds a spread of seven: enough to outrank a pip of colour
/// fit (6) and reorder cards of similar quality, not enough to beat a bomb.
/// Lands score zero; `assemble_lands` places them.
pub(crate) fn aggro_curve_delta(brief: &crate::cube::CardBrief) -> i32 {
    if brief.is_land {
        return 0;
    }
    match brief.cmc {
        0..=2 => 3,
        3 => 1,
        4 => 0,
        5 => -2,
        _ => -4,
    }
}

/// The sealed builder's scorer under `SimConfig::rated_cards`: a rated
/// card ([`crate::card_ratings`]) scores its measured value, an unrated one
/// its colour-independent static score (`base_score + quality`), and either
/// loses 10 a pip it cannot pay in the seat's colours
/// ([`crate::cube::CardBrief::off_pips`], which reads `{G/U}` as G or U).
///
/// On-colour pips earn nothing, unlike the static scorer's +6: the rating
/// already prices castability, and the bonus (~4.6 rating points a pip)
/// swamped the table's spread, favouring pip-heavy cards and colours.
/// Dropping it measured +1.44 / +1.77 over the old build; also dropping it
/// for unrated cards and reading hybrids as either colour, +2.99 / +3.19
/// more (`fra_card_eval --audit-builder`, two seeds each).
pub(crate) fn score_brief_rated(
    brief: &crate::cube::CardBrief,
    seat_colors: &ColorCounts,
) -> i32 {
    let seat = seat_colors.support();
    let off_cost = if seat.is_empty() { 0 } else { brief.off_pips(seat) * 10 };
    crate::card_ratings::brief_rated(brief).unwrap_or(brief.base_score + brief.quality) - off_cost
}

/// [`score_brief_quality`] plus [`aggro_curve_delta`] — the sealed builder's
/// scorer under `SimConfig::curve_aggro`.
pub(crate) fn score_brief_aggro(
    brief: &crate::cube::CardBrief,
    seat_colors: &ColorCounts,
) -> i32 {
    score_brief_quality(brief, seat_colors) + aggro_curve_delta(brief)
}

/// [`score_card_for_seat`] with the seat's pip-color totals precomputed, so a
/// caller scoring many cards against one pile hoists `colors_of_picks` out of
/// the loop instead of re-instantiating the pile per card.
pub(crate) fn score_card_with_colors(
    factory: CardFactory,
    seat_colors: &ColorCounts,
) -> i32 {
    score_brief_with_colors(crate::cube::card_brief(factory), seat_colors)
}

/// [`score_card_with_colors`] for a caller already holding the brief.
pub(crate) fn score_brief_with_colors(
    brief: &crate::cube::CardBrief,
    seat_colors: &ColorCounts,
) -> i32 {
    score_brief_in_colors(brief, seat_colors.support())
}

/// [`score_brief_with_colors`] against the seat's colours as a bitmask (the
/// form the sealed builder's shape ranker calls). The colour term depends
/// only on the set: on-colour pips +6, off-colour -4, and `off = total - on`
/// gives `10 * on - 4 * total`. The rest is precomputed as
/// [`CardBrief::base_score`](crate::cube::CardBrief::base_score).
pub(crate) fn score_brief_in_colors(
    brief: &crate::cube::CardBrief,
    seat: crate::mana::ColorSet,
) -> i32 {
    let total = brief.pips_total as i32;
    // Colorless cards carry their colour term in `base_score` (+5 unless a
    // land; a land bonus would sneak lands into spell slots, which
    // `assemble_lands` owns).
    let color_term = if total == 0 {
        0
    } else if seat.is_empty() {
        // First few picks before any colors are committed: don't penalize
        // colored cards at all — early picks define the seat's colors.
        total * 2
    } else {
        brief.pips.masked_sum(seat) as i32 * 10 - total * 4
    };
    brief.base_score + color_term
}

/// Auto-pick from `pack` for a seat with `seat_picks_so_far`; returns the
/// picked index, `None` only for an empty pack. Tie-break: score, then
/// highest CMC (a small bombs-first bias), then lowest index.
pub fn bot_pick(pack: &[CardFactory], seat_picks_so_far: &[CardFactory]) -> Option<usize> {
    if pack.is_empty() {
        return None;
    }
    let mut best_idx = 0;
    let mut best_score = i32::MIN;
    let mut best_cmc = 0u32;
    for (i, factory) in pack.iter().enumerate() {
        let score = score_card_for_seat(*factory, seat_picks_so_far);
        let cmc = crate::cube::card_brief(*factory).cmc;
        let better = score > best_score || (score == best_score && cmc > best_cmc);
        if better {
            best_idx = i;
            best_score = score;
            best_cmc = cmc;
        }
    }
    Some(best_idx)
}

/// Colored-pip totals per color, WUBRG-indexed.
///
/// An array rather than a `HashMap` (which was 12.1 % of a twelve-deck build
/// at the fifty-third tip); the fixed WUBRG order also makes
/// [`top_two_colors`]' tie-break deterministic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorCounts([u32; 5]);

impl ColorCounts {
    const fn idx(c: Color) -> usize {
        match c {
            Color::White => 0,
            Color::Blue => 1,
            Color::Black => 2,
            Color::Red => 3,
            Color::Green => 4,
        }
    }

    pub fn get(&self, c: Color) -> u32 {
        self.0[Self::idx(c)]
    }

    pub fn add(&mut self, c: Color, n: u32) {
        self.0[Self::idx(c)] += n;
    }

    /// True when no pick contributed a colored pip.
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|n| *n == 0)
    }

    /// The colors with at least one pip, as a bitmask (bit i = slot i, both
    /// WUBRG).
    pub fn support(&self) -> crate::mana::ColorSet {
        let mut bits = 0u8;
        for (i, n) in self.0.iter().enumerate() {
            bits |= u8::from(*n > 0) << i;
        }
        crate::mana::ColorSet(bits)
    }

    /// The pips in `set`, summed — the on-colour half of
    /// [`score_brief_in_colors`].
    pub fn masked_sum(&self, set: crate::mana::ColorSet) -> u32 {
        let mut n = 0;
        for (i, v) in self.0.iter().enumerate() {
            if set.0 & (1 << i) != 0 {
                n += v;
            }
        }
        n
    }

    /// Every color with its total, in WUBRG order — including zeros.
    pub fn iter(&self) -> impl Iterator<Item = (Color, u32)> + '_ {
        Color::ALL.into_iter().map(|c| (c, self.get(c)))
    }

    /// The colors with at least one pip, in WUBRG order — what
    /// [`colors_of_cost`] returns for the same cost.
    pub fn colors(&self) -> impl Iterator<Item = Color> + '_ {
        self.iter().filter(|(_, n)| *n > 0).map(|(c, _)| c)
    }

    /// Sum another distribution into this one.
    pub fn add_all(&mut self, other: &ColorCounts) {
        for (i, n) in other.0.iter().enumerate() {
            self.0[i] += n;
        }
    }
}

/// Colored pips summed across a seat's picks: which colors
/// `score_card_for_seat` treats the seat as committed to. Pipless cards
/// contribute nothing.
pub fn colors_of_picks(picks: &[CardFactory]) -> ColorCounts {
    let mut totals = ColorCounts::default();
    for factory in picks {
        totals.add_all(&crate::cube::card_brief(*factory).pips);
    }
    totals
}

/// This cost's colored pips per color, in one pass over the symbols.
///
/// Same rule as [`colored_pip_count`] (hybrid counts both halves, Phyrexian
/// its colored half) for every color at once.
pub(crate) fn pip_counts(cost: &crate::mana::ManaCost) -> ColorCounts {
    let mut totals = ColorCounts::default();
    add_pip_counts(cost, &mut totals);
    totals
}

fn add_pip_counts(cost: &crate::mana::ManaCost, totals: &mut ColorCounts) {
    for sym in &cost.symbols {
        match sym {
            ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => totals.add(*c, 1),
            ManaSymbol::Hybrid(a, b) => {
                totals.add(*a, 1);
                totals.add(*b, 1);
            }
            _ => {}
        }
    }
}

/// Distinct colors referenced by a card's printed cost (colored,
/// hybrid, or Phyrexian pips).
///
/// A [`ColorSet`], not a `Vec<Color>`: the builders call this per pool card
/// per shape, and the allocation was 12.8 % of a sealed build at `67809f9f`.
/// It iterates in WUBRG order, as the `Vec` did.
pub(crate) fn colors_of_cost(cost: &crate::mana::ManaCost) -> crate::mana::ColorSet {
    let mut seen = crate::mana::ColorSet::empty();
    for sym in &cost.symbols {
        match sym {
            ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => seen.insert(*c),
            ManaSymbol::Hybrid(a, b) => {
                seen.insert(*a);
                seen.insert(*b);
            }
            _ => {}
        }
    }
    seen
}

/// Count of colored pips of the given color on this cost. Hybrid pips
/// count for both halves; Phyrexian pips count for their colored half.
pub(crate) fn colored_pip_count(cost: &crate::mana::ManaCost, color: Color) -> u32 {
    cost.symbols
        .iter()
        .filter(|s| match s {
            ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => *c == color,
            ManaSymbol::Hybrid(a, b) => *a == color || *b == color,
            _ => false,
        })
        .count() as u32
}

/// Suggest a starting deck for `picks`: the highest-scoring spells in the
/// seat's two strongest colors, up to `target_spells` (default 23), skipping
/// cards that need a third color. Returns `(main, leftovers)`; the deck
/// builder UI starts from this and the user moves cards freely.
pub fn suggest_main_deck(picks: &[CardFactory], target_spells: usize) -> (Vec<CardFactory>, Vec<CardFactory>) {
    let colors = top_two_colors(picks);
    let mut on_color: Vec<(CardFactory, i32)> = Vec::new();
    let mut other: Vec<CardFactory> = Vec::new();
    for &factory in picks {
        let card_colors = colors_of_cost(&crate::cube::card_def(factory).cost);
        let on = card_colors.is_empty()
            || card_colors.iter().all(|c| colors.contains(&c));
        if on {
            let s = score_card_for_seat(factory, picks);
            on_color.push((factory, s));
        } else {
            other.push(factory);
        }
    }
    on_color.sort_by_key(|(_, score)| std::cmp::Reverse(*score));
    // In score order, a card joins the main deck if it fits the target size
    // and the 4-copy cap; otherwise it goes to leftovers, so over-cap copies
    // stay visible and the slot passes to the next card. (Capping after
    // slicing once dropped the extra copies entirely.)
    let mut counts: HashMap<usize, u32> = HashMap::default();
    let mut main: Vec<CardFactory> = Vec::new();
    let mut leftovers: Vec<CardFactory> = Vec::new();
    for (factory, _score) in on_color {
        let key = factory as usize;
        let count = counts.entry(key).or_insert(0);
        if main.len() < target_spells && *count < COPY_CAP {
            *count += 1;
            main.push(factory);
        } else {
            leftovers.push(factory);
        }
    }
    leftovers.extend(other);
    (main, leftovers)
}

/// Two strongest colors in `picks` by colored-pip weight, ties in WUBRG
/// order. Falls back to W / U when fewer than two colors are present.
pub fn top_two_colors(picks: &[CardFactory]) -> [Color; 2] {
    let totals = colors_of_picks(picks);
    let mut v: Vec<(Color, u32)> = totals.iter().collect();
    v.sort_by_key(|(_, weight)| std::cmp::Reverse(*weight));
    let primary = v.first().map(|(c, _)| *c).unwrap_or(Color::White);
    let secondary = v
        .iter()
        .skip(1)
        .find(|(c, _)| *c != primary)
        .map(|(c, _)| *c)
        .unwrap_or_else(|| {
            // Any color but the primary. The `find` always hits; the
            // fallback avoids a panic path in the training actor.
            Color::ALL
                .into_iter()
                .find(|c| *c != primary)
                .unwrap_or(if primary == Color::White { Color::Blue } else { Color::White })
        });
    [primary, secondary]
}

/// Basic-land split for `main_deck` in `colors`, summing to `total_lands`,
/// proportional to each color's pip weight, with at least one land of each
/// color that has a pip.
pub fn suggest_basic_split(main_deck: &[CardFactory], colors: [Color; 2], total_lands: u32) -> HashMap<Color, u32> {
    let mut weights: HashMap<Color, u32> = HashMap::default();
    for &c in &colors {
        weights.insert(c, 0);
    }
    for factory in main_deck {
        let def = crate::cube::card_def(*factory);
        for &c in &colors {
            let n = colored_pip_count(&def.cost, c);
            if n > 0 {
                *weights.entry(c).or_insert(0) += n;
            }
        }
    }
    let total_weight: u32 = weights.values().sum();
    let mut out: HashMap<Color, u32> = HashMap::default();
    if total_weight == 0 {
        // No colored pips at all (e.g. all artifacts) — split lands
        // 50/50 between the two chosen colors.
        out.insert(colors[0], total_lands / 2);
        out.insert(colors[1], total_lands - total_lands / 2);
        return out;
    }
    let mut allocated = 0u32;
    for &c in &colors {
        let w = weights.get(&c).copied().unwrap_or(0);
        let share = (w as f32 / total_weight as f32 * total_lands as f32).round() as u32;
        out.insert(c, share);
        allocated = allocated.saturating_add(share);
    }
    // Fix any rounding drift so the sum lands exactly on `total_lands`.
    if allocated < total_lands {
        let diff = total_lands - allocated;
        if let Some(v) = out.get_mut(&colors[0]) {
            *v += diff;
        }
    } else if allocated > total_lands {
        let diff = allocated - total_lands;
        if let Some(v) = out.get_mut(&colors[0]) {
            *v = v.saturating_sub(diff);
        }
    }
    // Floor each chosen color to >= 1 land if any pip was present.
    for &c in &colors {
        if weights.get(&c).copied().unwrap_or(0) > 0 && out.get(&c).copied().unwrap_or(0) == 0 {
            *out.entry(c).or_insert(0) = 1;
        }
    }
    out
}

/// [`basic_land_factory`] built straight into an `Arc`, in this frame: a
/// factory returns its 8 KB `CardDefinition` by value into the caller's
/// frame, and `run_effect` is a caller (PERF "run_effect's frame").
#[inline(never)]
pub fn basic_land_arc(color: Color) -> std::sync::Arc<crate::card::CardDefinition> {
    std::sync::Arc::new(basic_land_factory(color)())
}

/// The basic-land factory for a color; a public mirror of the private
/// `cube::basic_factory`.
pub fn basic_land_factory(color: Color) -> CardFactory {
    match color {
        Color::White => crate::catalog::plains,
        Color::Blue => crate::catalog::island,
        Color::Black => crate::catalog::swamp,
        Color::Red => crate::catalog::mountain,
        Color::Green => crate::catalog::forest,
    }
}

/// Enforce the 4-copy cap on a built deck, dropping copies past it in list
/// order.
///
/// Basic lands are exempt (CR 100.2a). The client runs every finished draft
/// deck through here after adding basics; before 2026-11-15 that cut a 23 +
/// 17 list to ~31 cards with 8 lands.
pub fn enforce_copy_cap(cards: Vec<CardFactory>) -> Vec<CardFactory> {
    let mut counts: HashMap<usize, u32> = HashMap::default();
    let mut out = Vec::with_capacity(cards.len());
    for f in cards {
        let def = crate::cube::card_def(f);
        if def.is_land() && def.supertypes.contains(&crate::card::Supertype::Basic) {
            out.push(f);
            continue;
        }
        let key = f as usize;
        let entry = counts.entry(key).or_insert(0);
        if *entry < COPY_CAP {
            *entry += 1;
            out.push(f);
        }
    }
    out
}

/// Build a 2-player `GameState` from a player and opponent deck, both seats
/// `wants_ui`. The seat names label portraits and the log ("You vs P3 (UR)").
pub fn build_draft_match_state(
    player_deck: Vec<CardFactory>,
    opponent_deck: Vec<CardFactory>,
    player_seat_name: String,
    opponent_seat_name: String,
) -> GameState {
    let mut rng = rand::rng();
    let mut state = GameState::new(vec![
        Player::new(0, &player_seat_name),
        Player::new(1, &opponent_seat_name),
    ]);
    for f in &player_deck {
        state.add_card_to_library(0, crate::cube::card_arc(*f));
    }
    state.players[0].library.shuffle(&mut rng);
    for f in &opponent_deck {
        state.add_card_to_library(1, crate::cube::card_arc(*f));
    }
    state.players[1].library.shuffle(&mut rng);
    state.players[0].wants_ui = true;
    state.players[1].wants_ui = true;
    // Seat each player's Lessons sideboard so Learn fetches resolve in
    // drafted matches, matching the cube/format build paths.
    for seat in 0..2 {
        for &f in &crate::cube::lessons_sideboard() {
            state.add_card_to_sideboard(seat, f());
        }
    }
    state
}

/// Engine-invented cards with no Scryfall art. Cube play tolerates their
/// cardback placeholder, but draft shows cards face-up, so they are filtered
/// out of draft pools.
///
/// Keep in sync with `crabomination_client::scryfall::FICTIONAL_CARDS`.
const FICTIONAL_CARD_NAMES: &[&str] = &[
    "Sundering Eruption",
    "Volcanic Fissure",
];

/// The cube pool as a draft pool, filtered to cards with real Scryfall art.
pub fn draft_pool() -> Vec<CardFactory> {
    all_cube_cards()
        .into_iter()
        .filter(|f| {
            let name = crate::cube::card_def(*f).name;
            !FICTIONAL_CARD_NAMES.iter().any(|x| x.eq_ignore_ascii_case(name))
        })
        .collect()
}

/// Secrets of Strixhaven draft pool: every ✅ SoS factory
/// (`sos_mode::all_sos_cards()`), through the same fictional-card filter.
pub fn sos_draft_pool() -> Vec<CardFactory> {
    sos_draft_pool_ref().to_vec()
}

/// [`sos_draft_pool`] without the copy, computed once per process.
///
/// The pool is a constant, but building it walks and dedups five colleges'
/// sub-pools and looks up every definition; `sealed_pool_packs` used to
/// rebuild it per pool, twice a game in a `selfplay_train` actor
/// (PERF (-66)).
pub fn sos_draft_pool_ref() -> &'static [CardFactory] {
    static POOL: std::sync::OnceLock<Vec<CardFactory>> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        crate::sos_mode::all_sos_cards()
            .into_iter()
            .filter(|f| {
                let name = crate::cube::card_def(*f).name;
                !FICTIONAL_CARD_NAMES.iter().any(|x| x.eq_ignore_ascii_case(name))
            })
            .collect()
    })
}

/// Which cards a draft draws from: the curated cube pool, or the Secrets of
/// Strixhaven cards (`sos_mode::all_sos_cards`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftPool {
    Cube,
    Sos,
}

impl DraftPool {
    pub fn factories(self) -> Vec<CardFactory> {
        match self {
            DraftPool::Cube => draft_pool(),
            DraftPool::Sos => sos_draft_pool(),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DraftPool::Cube => "Cube",
            DraftPool::Sos => "SoS",
        }
    }
}

// ── CR 905.2 draft-matters shell ─────────────────────────────────────────────

/// CR 905.2b — the values a player noted as they drafted cards with a
/// given name, keyed by that card's name. Carried out of the draft on
/// `Player.draft_notes` so Aether Searcher, Lurking Automaton, Cogwork
/// Grinder and Paliano can read them in the game that follows.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DraftNotes {
    pub numbers: HashMap<String, Vec<u32>>,
    pub names: HashMap<String, Vec<String>>,
    pub colors: HashMap<String, Vec<Color>>,
    #[serde(default)]
    pub keywords: HashMap<String, Vec<Keyword>>,
    #[serde(default)]
    pub creature_types: HashMap<String, Vec<CreatureType>>,
}

impl DraftNotes {
    pub fn note_number(&mut self, source: &str, n: u32) {
        self.numbers.entry(source.to_string()).or_default().push(n);
    }
    pub fn note_name(&mut self, source: &str, name: &str) {
        self.names.entry(source.to_string()).or_default().push(name.to_string());
    }
    pub fn note_colors(&mut self, source: &str, colors: &[Color]) {
        self.colors.entry(source.to_string()).or_default().extend_from_slice(colors);
    }
    /// Animus of Predation notes only the keywords it lists.
    pub fn note_keywords(&mut self, source: &str, keywords: &[Keyword]) {
        let noted = self.keywords.entry(source.to_string()).or_default();
        for kw in keywords.iter().filter(|k| ANIMUS_KEYWORDS.contains(k)) {
            if !noted.contains(kw) {
                noted.push(kw.clone());
            }
        }
    }
    pub fn note_creature_types(&mut self, source: &str, types: &[CreatureType]) {
        let noted = self.creature_types.entry(source.to_string()).or_default();
        for ct in types {
            if !noted.contains(ct) {
                noted.push(*ct);
            }
        }
    }
    pub fn noted_keywords(&self, source: &str) -> &[Keyword] {
        self.keywords.get(source).map(|v| v.as_slice()).unwrap_or(&[])
    }
    pub fn noted_creature_types(&self, source: &str) -> &[CreatureType] {
        self.creature_types.get(source).map(|v| v.as_slice()).unwrap_or(&[])
    }
    /// Highest number noted for `source`, or 0.
    pub fn max_number(&self, source: &str) -> u32 {
        self.numbers.get(source).and_then(|v| v.iter().copied().max()).unwrap_or(0)
    }
    /// Sum of every number noted for `source`.
    pub fn sum_numbers(&self, source: &str) -> u32 {
        self.numbers.get(source).map(|v| v.iter().sum()).unwrap_or(0)
    }
    /// One readable line per noting card, sorted by name so the projection is
    /// stable across ticks (CR 905.2b — this information is public).
    pub fn lines(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for (name, ns) in &self.numbers {
            out.push(format!("{name}: {}", ns.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")));
        }
        for (name, names) in &self.names {
            out.push(format!("{name}: {}", names.join(", ")));
        }
        for (name, colors) in &self.colors {
            let cs: Vec<String> = colors.iter().map(|c| c.short_name().to_string()).collect();
            out.push(format!("{name}: {}", cs.join(", ")));
        }
        for (name, kws) in &self.keywords {
            let ks: Vec<String> = kws.iter().map(|k| format!("{k:?}")).collect();
            out.push(format!("{name}: {}", ks.join(", ")));
        }
        for (name, cts) in &self.creature_types {
            let ts: Vec<String> = cts.iter().map(|t| format!("{t:?}")).collect();
            out.push(format!("{name}: {}", ts.join(", ")));
        }
        out.sort();
        out
    }

    pub fn has_name(&self, source: &str, name: &str) -> bool {
        self.names.get(source).is_some_and(|v| v.iter().any(|n| n == name))
    }
}

/// Card names whose draft-time text this shell implements. Kept as a
/// table so the pod, the bots and the client agree on what a card can do.
pub const AETHER_SEARCHER: &str = "Aether Searcher";
pub const AGENT_OF_ACQUISITIONS: &str = "Agent of Acquisitions";
pub const COGWORK_GRINDER: &str = "Cogwork Grinder";
pub const COGWORK_LIBRARIAN: &str = "Cogwork Librarian";
pub const LORE_SEEKER: &str = "Lore Seeker";
pub const LURKING_AUTOMATON: &str = "Lurking Automaton";
pub const PALIANO: &str = "Paliano, the High City";
pub const WHISPERGEAR_SNEAK: &str = "Whispergear Sneak";
// CN2's draft-matters cards ride the same note table.
pub const CUSTODI_PEACEKEEPER: &str = "Custodi Peacekeeper";
pub const GARBAGE_FIRE: &str = "Garbage Fire";
pub const PYRETIC_HUNTER: &str = "Pyretic Hunter";
pub const NOBLE_BANNERET: &str = "Noble Banneret";
pub const SMUGGLER_CAPTAIN: &str = "Smuggler Captain";
pub const REGICIDE: &str = "Regicide";
pub const SPIRE_PHANTASM: &str = "Spire Phantasm";
pub const ANIMUS_OF_PREDATION: &str = "Animus of Predation";
pub const PALIANO_VANGUARD: &str = "Paliano Vanguard";

/// The eleven keywords Animus of Predation lists; nothing else is noted.
pub const ANIMUS_KEYWORDS: [Keyword; 11] = [
    Keyword::Flying,
    Keyword::FirstStrike,
    Keyword::DoubleStrike,
    Keyword::Deathtouch,
    Keyword::Haste,
    Keyword::Hexproof,
    Keyword::Indestructible,
    Keyword::Lifelink,
    Keyword::Menace,
    Keyword::Reach,
    Keyword::Vigilance,
];

/// CN2's "reveal this card as you draft it and note how many cards you've
/// drafted this draft round" cycle — the same note Lurking Automaton takes.
const ROUND_PICK_NOTERS: [&str; 4] =
    [LURKING_AUTOMATON, CUSTODI_PEACEKEEPER, GARBAGE_FIRE, PYRETIC_HUNTER];

/// What a seat does with its pick this round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickAction {
    /// Draft the card at this pack index into the seat's pool.
    Take(usize),
    /// Cogwork Grinder / Animus of Predation — draft it and remove it from
    /// the draft; it never joins the pool (CR 905.2c). Every remover in the
    /// pool takes its note.
    Remove(usize),
    /// Paliano Vanguard — draft the card and reveal it, noting its creature
    /// types and turning one face-up Vanguard face down.
    TakeRevealed(usize),
    /// Cogwork Librarian — draft two cards from this pack, putting a
    /// Librarian already in the pool back into it.
    Librarian(usize, usize),
    /// Agent of Acquisitions — draft the whole pack; the seat can't draft
    /// again this round.
    TakePack,
}

/// One 8-seat booster draft, rules-side. The client owns presentation;
/// this owns pack circulation, bot picks, CR 905.2 draft-time card
/// behavior and the note table that survives into the match.
#[derive(Debug, Clone)]
pub struct DraftPod {
    pub pool: Vec<CardFactory>,
    pub pool_kind: DraftPool,
    pub pack_round: u32,
    pub pick_in_round: u32,
    /// Packs queued in front of each seat; a seat drafts from index 0.
    /// Normally one each — Lore Seeker's extra booster makes a seat
    /// briefly hold two.
    pub packs: Vec<Vec<Vec<CardFactory>>>,
    pub picks: Vec<Vec<CardFactory>>,
    pub notes: Vec<DraftNotes>,
    /// Cards removed from the draft face down, per seat.
    pub removed: Vec<Vec<CardFactory>>,
    /// Seats locked out of drafting for the rest of the round (Agent of
    /// Acquisitions).
    locked: Vec<bool>,
    /// Seats owing a name note on their next pick, one entry per noting card
    /// drafted (Aether Searcher, Smuggler Captain, Noble Banneret), tagged
    /// with which card is owed the note.
    owed_name_notes: Vec<Vec<&'static str>>,
    /// Picks a seat has made this draft round (Lurking Automaton's note).
    round_picks: Vec<u32>,
    /// Whispergear Sneaks a seat has turned face down; each one buys a peek.
    sneaks_used: Vec<u32>,
    /// Paliano Vanguards a seat has turned face down; each one buys one
    /// creature-type note.
    vanguards_used: Vec<u32>,
}

impl DraftPod {
    pub fn new(pool_kind: DraftPool) -> Self {
        let pool = pool_kind.factories();
        Self::with_pool(pool_kind, pool)
    }

    pub fn with_pool(pool_kind: DraftPool, pool: Vec<CardFactory>) -> Self {
        let mut rng = rand::rng();
        let packs = (0..POD_SIZE)
            .map(|_| vec![pool_kind.generate_pack(&pool, &mut rng)])
            .collect();
        Self {
            pool,
            pool_kind,
            pack_round: 1,
            pick_in_round: 1,
            packs,
            picks: vec![Vec::new(); POD_SIZE],
            notes: vec![DraftNotes::default(); POD_SIZE],
            removed: vec![Vec::new(); POD_SIZE],
            locked: vec![false; POD_SIZE],
            owed_name_notes: vec![Vec::new(); POD_SIZE],
            round_picks: vec![0; POD_SIZE],
            sneaks_used: vec![0; POD_SIZE],
            vanguards_used: vec![0; POD_SIZE],
        }
    }

    /// The pack `seat` is currently drafting from (empty when the seat
    /// holds none).
    pub fn pack(&self, seat: usize) -> &[CardFactory] {
        self.packs[seat].first().map(|p| p.as_slice()).unwrap_or(&[])
    }

    /// True once every pack in the pod is exhausted.
    pub fn round_over(&self) -> bool {
        self.packs.iter().flatten().all(|p| p.is_empty())
    }

    /// True while `seat` is locked out by Agent of Acquisitions.
    pub fn is_locked(&self, seat: usize) -> bool {
        self.locked[seat]
    }

    /// How many copies of `name` are in `seat`'s pool.
    pub fn owns(&self, seat: usize, name: &str) -> usize {
        self.picks[seat].iter().filter(|f| crate::cube::card_def(**f).name == name).count()
    }

    /// Draft actions `seat` may legally take beyond the plain `Take`,
    /// given the cards already in its pool. Drives the client's
    /// draft-matters buttons.
    pub fn special_actions(&self, seat: usize) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.owns(seat, AGENT_OF_ACQUISITIONS) > 0 && !self.locked[seat] {
            out.push(AGENT_OF_ACQUISITIONS);
        }
        if self.owns(seat, COGWORK_LIBRARIAN) > 0 && self.pack(seat).len() >= 2 {
            out.push(COGWORK_LIBRARIAN);
        }
        if self.owns(seat, COGWORK_GRINDER) > 0 {
            out.push(COGWORK_GRINDER);
        }
        if self.owns(seat, ANIMUS_OF_PREDATION) > 0 {
            out.push(ANIMUS_OF_PREDATION);
        }
        if self.owns(seat, WHISPERGEAR_SNEAK) as u32 > self.sneaks_used[seat] {
            out.push(WHISPERGEAR_SNEAK);
        }
        if self.owns(seat, PALIANO_VANGUARD) as u32 > self.vanguards_used[seat] {
            out.push(PALIANO_VANGUARD);
        }
        out
    }

    /// Whispergear Sneak — turn one face down to look at another seat's
    /// pack. Returns the peeked contents, or `None` when the seat has no
    /// unused Sneak.
    pub fn peek_pack(&mut self, seat: usize, at: usize) -> Option<Vec<CardFactory>> {
        if self.owns(seat, WHISPERGEAR_SNEAK) as u32 <= self.sneaks_used[seat] {
            return None;
        }
        self.sneaks_used[seat] += 1;
        Some(self.packs.get(at)?.first()?.clone())
    }

    /// Lore Seeker — add a fresh booster to the draft, queued in front of
    /// `seat` so their next pick comes from it.
    pub fn add_booster(&mut self, seat: usize) {
        let mut rng = rand::rng();
        let pack = self.pool_kind.generate_pack(&self.pool, &mut rng);
        self.packs[seat].insert(0, pack);
    }

    /// Run one full pick: `seat` takes `action`, every other seat bot-picks,
    /// then packs pass. Opens the next round (or reports the draft done)
    /// when every pack empties. Returns false once all rounds are drafted.
    pub fn advance(&mut self, seat: usize, action: PickAction) -> bool {
        self.apply(seat, action);
        for other in 0..POD_SIZE {
            if other == seat {
                continue;
            }
            if let Some(a) = self.bot_action(other) {
                self.apply(other, a);
            }
        }
        if !self.round_over() {
            self.pass_packs();
            self.pick_in_round += 1;
            return true;
        }
        self.pack_round += 1;
        self.pick_in_round = 1;
        self.locked.iter_mut().for_each(|l| *l = false);
        self.round_picks.iter_mut().for_each(|n| *n = 0);
        if self.pack_round > PACKS_PER_SEAT {
            return false;
        }
        let mut rng = rand::rng();
        for seat_packs in self.packs.iter_mut() {
            *seat_packs = vec![self.pool_kind.generate_pack(&self.pool, &mut rng)];
        }
        true
    }

    /// The action a bot seat takes. Bots draft greedily and use only the
    /// mandatory draft-time text; the optional abilities (removing cards
    /// from the draft, taking a whole pack) stay with the human seat.
    fn bot_action(&self, seat: usize) -> Option<PickAction> {
        if self.locked[seat] {
            return None;
        }
        let pack = self.pack(seat);
        // A Librarian in the pool is pure value: two cards for one pick.
        if self.owns(seat, COGWORK_LIBRARIAN) > 0 && pack.len() >= 2 {
            let a = bot_pick(pack, &self.picks[seat])?;
            let mut rest: Vec<CardFactory> = pack.to_vec();
            rest.remove(a);
            let b = bot_pick(&rest, &self.picks[seat])?;
            return Some(PickAction::Librarian(a, if b >= a { b + 1 } else { b }));
        }
        bot_pick(pack, &self.picks[seat]).map(PickAction::Take)
    }

    fn apply(&mut self, seat: usize, action: PickAction) {
        if self.locked[seat] || self.packs[seat].is_empty() {
            return;
        }
        match action {
            PickAction::Take(i) => {
                if let Some(card) = self.draw_from_pack(seat, i) {
                    self.record_pick(seat, card);
                }
            }
            PickAction::Remove(i) => {
                let grinders = self.owns(seat, COGWORK_GRINDER);
                let animi = self.owns(seat, ANIMUS_OF_PREDATION);
                if grinders == 0 && animi == 0 {
                    return self.apply(seat, PickAction::Take(i));
                }
                if let Some(card) = self.draw_from_pack(seat, i) {
                    self.round_picks[seat] += 1;
                    if grinders > 0 {
                        self.notes[seat].note_number(COGWORK_GRINDER, 1);
                    }
                    if animi > 0 {
                        let def = card();
                        self.notes[seat].note_keywords(ANIMUS_OF_PREDATION, &def.keywords);
                    }
                    self.removed[seat].push(card);
                }
            }
            PickAction::TakeRevealed(i) => {
                if self.owns(seat, PALIANO_VANGUARD) as u32 <= self.vanguards_used[seat] {
                    return self.apply(seat, PickAction::Take(i));
                }
                let Some(card) = self.draw_from_pack(seat, i) else { return };
                let def = card();
                if !def.is_creature() {
                    return self.record_pick(seat, card);
                }
                self.vanguards_used[seat] += 1;
                self.notes[seat]
                    .note_creature_types(PALIANO_VANGUARD, &def.subtypes.creature_types);
                self.record_pick(seat, card);
            }
            PickAction::Librarian(a, b) => {
                let librarian = self
                    .picks[seat]
                    .iter()
                    .position(|f| crate::cube::card_def(*f).name == COGWORK_LIBRARIAN);
                let Some(pos) = librarian else {
                    return self.apply(seat, PickAction::Take(a));
                };
                let (hi, lo) = if a > b { (a, b) } else { (b, a) };
                if hi == lo || self.pack(seat).len() <= hi {
                    return self.apply(seat, PickAction::Take(a));
                }
                let first = self.draw_from_pack(seat, hi);
                let second = self.draw_from_pack(seat, lo);
                for card in first.into_iter().chain(second) {
                    self.record_pick(seat, card);
                }
                let card = self.picks[seat].remove(pos);
                if let Some(pack) = self.packs[seat].first_mut() {
                    pack.push(card);
                }
            }
            PickAction::TakePack => {
                if self.owns(seat, AGENT_OF_ACQUISITIONS) == 0 {
                    return self.apply(seat, PickAction::Take(0));
                }
                let pack = std::mem::take(&mut self.packs[seat][0]);
                for card in pack {
                    self.record_pick(seat, card);
                }
                self.locked[seat] = true;
            }
        }
    }

    fn draw_from_pack(&mut self, seat: usize, index: usize) -> Option<CardFactory> {
        let pack = self.packs[seat].first_mut()?;
        (index < pack.len()).then(|| pack.remove(index))
    }

    /// Put a drafted card in the seat's pool, applying the "reveal this card
    /// as you draft it" notes (CR 905.2b).
    fn record_pick(&mut self, seat: usize, card: CardFactory) {
        self.round_picks[seat] += 1;
        let def = card();
        let name = def.name;
        // Noble Banneret only notes creature cards; the others take any pick.
        let owed = std::mem::take(&mut self.owed_name_notes[seat]);
        for noter in owed {
            if noter == NOBLE_BANNERET && !def.is_creature() {
                self.owed_name_notes[seat].push(noter);
                continue;
            }
            self.notes[seat].note_name(noter, name);
        }
        match name {
            AETHER_SEARCHER | SMUGGLER_CAPTAIN | NOBLE_BANNERET => {
                self.owed_name_notes[seat].push(match name {
                    SMUGGLER_CAPTAIN => SMUGGLER_CAPTAIN,
                    NOBLE_BANNERET => NOBLE_BANNERET,
                    _ => AETHER_SEARCHER,
                });
            }
            n if ROUND_PICK_NOTERS.contains(&n) => {
                let picks = self.round_picks[seat];
                self.notes[seat].note_number(n, picks);
            }
            PALIANO => {
                let colors = self.paliano_colors(seat);
                self.notes[seat].note_colors(PALIANO, &colors);
            }
            // "The player to your right chooses a color, you choose another,
            // then the player to your left chooses a third" — the same
            // three-seat walk Paliano uses.
            REGICIDE => {
                let colors = self.paliano_colors(seat);
                self.notes[seat].note_colors(REGICIDE, &colors);
            }
            // "Guess that card's name, then that player reveals it" — the
            // guess lands when only one card is left to name (CR 905.2b).
            SPIRE_PHANTASM if self.pack(seat).len() == 1 => {
                self.notes[seat].note_number(SPIRE_PHANTASM, 1);
            }
            _ => {}
        }
        self.picks[seat].push(card);
    }

    /// Paliano's three colors: the seat to the drafter's right picks first,
    /// then the drafter, then the seat to their left — each taking the
    /// strongest color still unchosen in their own pool.
    fn paliano_colors(&self, seat: usize) -> Vec<Color> {
        let order = [
            (seat + POD_SIZE - 1) % POD_SIZE,
            seat,
            (seat + 1) % POD_SIZE,
        ];
        let mut chosen: Vec<Color> = Vec::with_capacity(3);
        for who in order {
            let weights = colors_of_picks(&self.picks[who]);
            let pick = Color::ALL
                .into_iter()
                .filter(|c| !chosen.contains(c))
                .max_by_key(|c| weights.get(*c))
                .expect("five colors, at most two chosen");
            chosen.push(pick);
        }
        chosen
    }

    fn pass_packs(&mut self) {
        // Rounds 1 and 3 pass left, round 2 passes right. A seat's front
        // pack moves on; any extra booster it is holding shuffles up.
        let moving: Vec<Vec<CardFactory>> = self
            .packs
            .iter_mut()
            .map(|q| if q.is_empty() { Vec::new() } else { q.remove(0) })
            .collect();
        for (i, pack) in moving.into_iter().enumerate() {
            if pack.is_empty() {
                continue;
            }
            let to = if self.pack_round != 2 {
                (i + POD_SIZE - 1) % POD_SIZE
            } else {
                (i + 1) % POD_SIZE
            };
            self.packs[to].push(pack);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_rng() -> rand::rngs::StdRng {
        use rand::SeedableRng;
        rand::rngs::StdRng::seed_from_u64(42)
    }

    #[test]
    fn pack_has_fifteen_distinct_cards() {
        let pool = draft_pool();
        let mut rng = fixed_rng();
        let pack = generate_pack(&pool, &mut rng);
        assert_eq!(pack.len(), PACK_SIZE);
        // No duplicates within a pack.
        let mut seen: crate::fxhash::HashSet<usize> = crate::fxhash::HashSet::default();
        for f in &pack {
            assert!(seen.insert(*f as usize), "pack has duplicate factories");
        }
    }

    #[test]
    fn open_all_packs_returns_24_packs() {
        let pool = draft_pool();
        let mut rng = fixed_rng();
        let packs = open_all_packs(&pool, &mut rng);
        assert_eq!(packs.len(), POD_SIZE * PACKS_PER_SEAT as usize);
        assert!(packs.iter().all(|p| p.len() == PACK_SIZE));
    }

    #[test]
    fn bot_pick_picks_highest_scored_card() {
        // With no picks yet, color is open, but the type weight should
        // still prefer a creature over a colorless artifact.
        let pack: Vec<CardFactory> = vec![
            crate::catalog::grizzly_bears, // {1}{G} 2/2 creature
            crate::catalog::sol_ring,      // {1} colorless artifact
        ];
        let pick = bot_pick(&pack, &[]).unwrap();
        let picked = pack[pick];
        assert!(
            std::ptr::eq(
                picked as *const (),
                crate::catalog::grizzly_bears as *const (),
            ),
            "creature should outscore the colorless artifact at first pick"
        );
    }

    #[test]
    fn bot_pick_prefers_already_picked_color() {
        // Seat already has 5 Forests-worth of green pips. A {U} card
        // should score lower than a {G} card now.
        let history: Vec<CardFactory> = vec![
            crate::catalog::grizzly_bears, // {1}{G}
            crate::catalog::grizzly_bears, // {1}{G}
            crate::catalog::grizzly_bears, // {1}{G}
        ];
        let pack: Vec<CardFactory> = vec![
            crate::catalog::counterspell,   // {U}{U}
            crate::catalog::elvish_mystic,  // {G}
        ];
        let pick = bot_pick(&pack, &history).unwrap();
        assert!(
            std::ptr::eq(
                pack[pick] as *const (),
                crate::catalog::elvish_mystic as *const (),
            ),
            "bot should follow its existing green commitment"
        );
    }

    #[test]
    fn top_two_colors_picks_most_drafted() {
        let picks: Vec<CardFactory> = vec![
            crate::catalog::lightning_bolt, // {R}
            crate::catalog::lightning_bolt, // {R}
            crate::catalog::counterspell,   // {U}{U}
            crate::catalog::counterspell,   // {U}{U}
            crate::catalog::counterspell,   // {U}{U}
            crate::catalog::grizzly_bears,  // {1}{G}
        ];
        let [a, b] = top_two_colors(&picks);
        // Blue wins (6 pips), red is second (2 pips).
        assert_eq!(a, Color::Blue);
        assert_eq!(b, Color::Red);
    }

    #[test]
    fn suggest_main_deck_returns_at_most_target_size() {
        // 45 distinct picks (mix of green creatures + blue spells) so
        // the 4-copy cap doesn't drop anything.
        let picks: Vec<CardFactory> = vec![
            crate::catalog::grizzly_bears,
            crate::catalog::elvish_mystic,
            crate::catalog::llanowar_elves,
            crate::catalog::birds_of_paradise,
            crate::catalog::sylvan_caryatid,
            crate::catalog::counterspell,
            crate::catalog::lightning_bolt,
            crate::catalog::sol_ring,
        ];
        let (main, sb) = suggest_main_deck(&picks, 23);
        assert!(main.len() <= 23, "main deck capped at target size");
        // No cap drops here (each card appears once), so all picks
        // land in either main or sideboard.
        assert_eq!(main.len() + sb.len(), picks.len(), "no cards lost");
    }

    #[test]
    fn suggest_main_deck_caps_duplicates() {
        // 6 copies of the same card → 4 in main, the rest move to
        // the sideboard via the copy-cap pass.
        let picks: Vec<CardFactory> = vec![crate::catalog::lightning_bolt; 6];
        let (main, sb) = suggest_main_deck(&picks, 23);
        assert_eq!(main.len(), 4, "main capped at four copies");
        // Regression: previously the over-cap copies were silently dropped
        // because `leftovers` was built via `.skip(target_spells)` over the
        // pre-cap pile. The 5th and 6th Bolts must end up in the sideboard
        // pile so the player can still see them when deckbuilding.
        assert_eq!(main.len() + sb.len(), picks.len(),
            "no cards lost — over-cap copies must move to sideboard");
        assert_eq!(sb.len(), 2, "two over-cap copies sit in the sideboard");
        assert!(sb.iter().all(|f| *f as usize == crate::catalog::lightning_bolt as *const () as usize),
            "sideboard holds the dropped Bolts");
    }

    #[test]
    fn suggest_main_deck_fills_main_past_capped_card() {
        // 6 Bolts (over-cap) plus enough other on-color spells. With the
        // bug, the 6 Bolts consumed 6 main-deck slots before being capped
        // back to 4, leaving the next-best card stranded in the sideboard
        // even though main had room. After the fix, the cap rejection
        // frees the slot for the next on-color pick.
        let mut picks: Vec<CardFactory> = vec![crate::catalog::lightning_bolt; 6];
        picks.push(crate::catalog::counterspell);
        let (main, sb) = suggest_main_deck(&picks, 5);
        assert_eq!(main.len(), 5, "main fills to target after the cap kicks in");
        assert!(main.iter().any(|f| *f as usize == crate::catalog::counterspell as *const () as usize),
            "Counterspell takes the slot freed by the 5th Bolt being capped");
        assert_eq!(main.len() + sb.len(), picks.len(), "no cards lost");
    }

    #[test]
    fn enforce_copy_cap_caps_duplicates() {
        let cards: Vec<CardFactory> = vec![
            crate::catalog::lightning_bolt,
            crate::catalog::lightning_bolt,
            crate::catalog::lightning_bolt,
            crate::catalog::lightning_bolt,
            crate::catalog::lightning_bolt, // 5th copy — dropped
            crate::catalog::lightning_bolt, // 6th copy — dropped
            crate::catalog::counterspell,
        ];
        let capped = enforce_copy_cap(cards);
        let bolts = capped
            .iter()
            .filter(|f| {
                std::ptr::eq(
                    **f as *const (),
                    crate::catalog::lightning_bolt as *const (),
                )
            })
            .count();
        assert_eq!(bolts, 4);
        assert_eq!(capped.len(), 5); // 4 bolts + 1 counterspell
    }

    /// CR 100.2a — the four-copy cap does not apply to basic lands. The
    /// client's drafted decks (23 spells + 17 basics) went through the cap
    /// after their basics were added and reached the table at ~31 cards.
    #[test]
    fn enforce_copy_cap_keeps_every_basic_land() {
        let mut cards: Vec<CardFactory> = vec![crate::catalog::lightning_bolt; 5];
        cards.extend(std::iter::repeat_n(basic_land_factory(Color::Red), 9));
        cards.extend(std::iter::repeat_n(basic_land_factory(Color::Blue), 8));
        let capped = enforce_copy_cap(cards);
        assert_eq!(capped.len(), 4 + 17);
        assert_eq!(capped.iter().filter(|f| crate::cube::card_def(**f).is_land()).count(), 17);
    }

    #[test]
    fn draft_pool_excludes_fictional_cards() {
        let pool = draft_pool();
        // None of the fictional names should appear in the draft pool.
        for f in &pool {
            let name = crate::cube::card_def(*f).name;
            assert!(
                !FICTIONAL_CARD_NAMES.iter().any(|x| x.eq_ignore_ascii_case(name)),
                "fictional card {name:?} leaked into the draft pool"
            );
        }
        // And the pool should still be substantial (well above the
        // 360-card draft requirement).
        assert!(pool.len() > 100, "draft pool too small after filtering");
    }

    #[test]
    fn build_draft_match_state_seats_two_players() {
        let player_deck: Vec<CardFactory> = (0..40)
            .map(|_| crate::catalog::grizzly_bears as CardFactory)
            .collect();
        let opp_deck: Vec<CardFactory> = (0..40)
            .map(|_| crate::catalog::lightning_bolt as CardFactory)
            .collect();
        let state = build_draft_match_state(
            player_deck,
            opp_deck,
            "You".into(),
            "Bot".into(),
        );
        assert_eq!(state.players.len(), 2);
        assert_eq!(state.players[0].library.len(), 40);
        assert_eq!(state.players[1].library.len(), 40);
        assert!(state.players[0].wants_ui);
        assert!(state.players[1].wants_ui);
    }

    /// The Special Guests sheet is collated on its own slot: never in the
    /// colour buckets, and roughly one pack in `SOS_SPECIAL_GUEST_RATE`.
    #[test]
    fn sos_packs_collate_special_guests_on_their_own_slot() {
        let pool = sos_draft_pool();
        let sheet: crate::fxhash::HashSet<usize> = crate::sos_mode::sos_special_guests()
            .into_iter()
            .map(|f| f as usize)
            .collect();
        assert!(pool.iter().any(|f| sheet.contains(&(*f as usize))), "the sheet is in the pool");

        let mut rng = fixed_rng();
        let (packs, mut with_guest) = (4000, 0);
        for _ in 0..packs {
            let pack = generate_sos_pack(&pool, &mut rng);
            let n = pack.iter().filter(|f| sheet.contains(&(**f as usize))).count();
            assert!(n <= 1, "at most one Special Guest per pack");
            with_guest += n;
        }
        let expected = packs / SOS_SPECIAL_GUEST_RATE as usize;
        assert!(
            (expected / 2..expected * 2).contains(&with_guest),
            "{with_guest} guests in {packs} packs, expected around {expected}"
        );
    }

    #[test]
    fn suggest_basic_split_sums_to_total() {
        let picks: Vec<CardFactory> = vec![
            crate::catalog::grizzly_bears,  // {1}{G}
            crate::catalog::grizzly_bears,
            crate::catalog::counterspell,   // {U}{U}
        ];
        let split = suggest_basic_split(&picks, [Color::Green, Color::Blue], 17);
        let sum: u32 = split.values().sum();
        assert_eq!(sum, 17);
    }

    #[test]
    fn sos_draft_pool_is_substantial_and_distinct() {
        let pool = sos_draft_pool();
        assert!(
            pool.len() >= 100,
            "SoS draft pool unexpectedly small: {} cards",
            pool.len()
        );
        // All entries are unique factory pointers.
        let mut seen = crate::fxhash::HashSet::default();
        for f in &pool {
            assert!(seen.insert(*f as usize), "duplicate factory in SoS pool");
        }
    }

    #[test]
    fn draft_pool_enum_dispatches() {
        assert_eq!(DraftPool::Cube.factories().len(), draft_pool().len());
        assert_eq!(DraftPool::Sos.factories().len(), sos_draft_pool().len());
    }

    #[test]
    fn sos_pack_has_pack_size_with_no_duplicates() {
        let pool = sos_draft_pool();
        let mut rng = fixed_rng();
        let pack = generate_sos_pack(&pool, &mut rng);
        assert_eq!(pack.len(), PACK_SIZE, "SoS pack must be exactly 15 cards");
        let mut seen = crate::fxhash::HashSet::default();
        for f in &pack {
            assert!(seen.insert(*f as usize), "SoS pack has duplicate factory");
        }
    }

    #[test]
    fn sos_pack_satisfies_color_recipe_on_average() {
        // Over 100 rolls every bucket should average at least one card;
        // guards bucket assignment.
        let pool = sos_draft_pool();
        let mut rng = fixed_rng();
        let mut bucket_totals: crate::fxhash::HashMap<SosBucket, u32> =
            crate::fxhash::HashMap::default();
        let n = 100;
        for _ in 0..n {
            let pack = generate_sos_pack(&pool, &mut rng);
            for f in &pack {
                let def = crate::cube::card_def(*f);
                *bucket_totals.entry(sos_bucket_of(def)).or_insert(0) += 1;
            }
        }
        // Each mono-color quota is 2 per pack × 100 packs = 200, but
        // shortfall fills from Multi so we accept anything from 100+.
        for color in Color::ALL {
            let got = bucket_totals
                .get(&SosBucket::Mono(color))
                .copied()
                .unwrap_or(0);
            assert!(
                got >= 100,
                "mono-{color:?} undersupplied across 100 packs: got {got}"
            );
        }
        let multi = bucket_totals.get(&SosBucket::Multi).copied().unwrap_or(0);
        assert!(multi >= 100, "multi undersupplied across 100 packs: got {multi}");
    }

    #[test]
    fn cube_pool_still_uses_uniform_generator() {
        // Regression guard: the cube pool's pack rolls must keep going
        // through `generate_pack` (no stratification). A symptom of a
        // wiring slip would be cube packs suddenly clustering by color.
        let pool = draft_pool();
        let mut rng = fixed_rng();
        let pack = DraftPool::Cube.generate_pack(&pool, &mut rng);
        assert_eq!(pack.len(), PACK_SIZE);
    }

    #[test]
    fn bot_picks_drain_a_pack_to_zero() {
        let pool = draft_pool();
        let mut rng = fixed_rng();
        let mut pack = generate_pack(&pool, &mut rng);
        let mut history: Vec<CardFactory> = Vec::new();
        while !pack.is_empty() {
            let idx = bot_pick(&pack, &history).unwrap();
            history.push(pack.remove(idx));
        }
        assert_eq!(history.len(), PACK_SIZE);
    }

    #[test]
    fn draft_match_state_seats_lessons_sideboard() {
        let deck: Vec<CardFactory> = vec![crate::catalog::grizzly_bears];
        let state = build_draft_match_state(
            deck.clone(),
            deck,
            "You".to_string(),
            "Opp".to_string(),
        );
        for seat in 0..2 {
            assert_eq!(
                state.players[seat].sideboard.len(),
                crate::cube::lessons_sideboard().len(),
                "drafted match seats the standard Lessons sideboard"
            );
        }
    }
}
// touch
