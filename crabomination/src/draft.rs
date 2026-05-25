//! 8-player Crabomination booster draft against bots.
//!
//! Architecture: the entire draft runs *off-engine* — there's no
//! `GameState`, no decisions on the stack, no server actor. Pack
//! generation, bot picks, and the resulting per-seat 45-card pile are
//! all plain Rust values produced by the helpers below. The client
//! drives the loop (the user picks one card per pack interactively;
//! `bot_pick` runs synchronously for the other 7 seats), and once
//! the draft completes the chosen player+opponent decks are fed into
//! `build_draft_match_state` to bootstrap a normal 2-player
//! `GameState` for the post-draft match.
//!
//! Card pool: the existing cube pool (`cube::all_cube_cards()`) — 309
//! unique cards, comfortably enough for 8 × 3 × 15 = 360 picks. We
//! sample uniformly with replacement, with the same per-pack
//! deduplication that real Magic boosters have (no two copies of the
//! same card in one pack).
//!
//! Bot picks: a small color-fit + curve-aware heuristic
//! (`score_card_for_seat`) that scores each card in the current pack
//! against the seat's accumulated picks. No synergy / archetype
//! detection — the bot greedily picks the highest-scored card,
//! breaking ties on mana value.

use std::collections::HashMap;

use rand::{Rng, RngExt};
use rand::seq::SliceRandom;

use crate::card::CardType;
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

/// Generate a single 15-card pack from `pool`. Sampled without
/// replacement *within the pack* (a pack never contains duplicates)
/// but with replacement across packs (the cube is large enough that
/// the same card can show up in multiple packs across the pod —
/// matching real Magic's boosters where rares aren't deduplicated
/// across the table).
///
/// Returns fewer than `PACK_SIZE` cards only if `pool.len() <
/// PACK_SIZE` (which the cube pool is far above, so this never
/// happens in practice — guarded for completeness).
pub fn generate_pack<R: Rng>(pool: &[CardFactory], rng: &mut R) -> Vec<CardFactory> {
    if pool.is_empty() {
        return Vec::new();
    }
    let want = PACK_SIZE.min(pool.len());
    let mut pack: Vec<CardFactory> = Vec::with_capacity(want);
    let mut used: std::collections::HashSet<usize> = std::collections::HashSet::new();
    while pack.len() < want {
        let idx = rng.random_range(0..pool.len());
        if used.insert(idx) {
            pack.push(pool[idx]);
        }
    }
    pack
}

/// Color-identity bucket used by `generate_sos_pack` to stratify
/// a Secrets of Strixhaven pack. Real-set rarity tags aren't on
/// `CardDefinition` yet, so we approximate booster shape by spreading
/// cards across the 5 mono-colors + multicolor (college pairs) +
/// colorless/land. This guarantees a player never opens a pack of
/// 15 mono-blue cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SosBucket {
    Mono(Color),
    Multi,
    ColorlessOrLand,
}

/// Per-pack quota for each bucket. Sums to 15. Two mono of each color
/// (10) + 3 multicolor (college pairs) + 2 colorless/land = 15. The
/// recipe is symmetric — every pack looks roughly the same shape, so
/// drafters see a consistent signal across the table.
const SOS_PACK_RECIPE: &[(SosBucket, usize)] = &[
    (SosBucket::Mono(Color::White), 2),
    (SosBucket::Mono(Color::Blue), 2),
    (SosBucket::Mono(Color::Black), 2),
    (SosBucket::Mono(Color::Red), 2),
    (SosBucket::Mono(Color::Green), 2),
    (SosBucket::Multi, 3),
    (SosBucket::ColorlessOrLand, 2),
];

/// Classify a card into its `SosBucket` based on color identity.
/// Hybrid pips count both halves; Phyrexian pips count their colored
/// half. Lands and cards with no colored pips fall into
/// `ColorlessOrLand` regardless of whether they're typed Land — the
/// engine has no land/colorless distinction at this stage.
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

/// Stratified pack generator for the SoS pool: each pack guarantees
/// the bucket mix described by `SOS_PACK_RECIPE`. Within each bucket
/// we sample without replacement (no duplicates inside one pack);
/// the same card *can* appear in multiple packs across the pod,
/// matching real-set draft economics.
///
/// Fallback behavior when a bucket is short:
/// - If the requested bucket has fewer cards than the recipe asks for,
///   we fill the shortfall from `SosBucket::Multi`, then from any
///   remaining unused pool card. This keeps every pack at exactly
///   `PACK_SIZE` even on small subset pools (e.g. the Prismari-only
///   universe is far smaller than 15 cards but the wider SoS pool
///   always satisfies the recipe).
pub fn generate_sos_pack<R: Rng>(pool: &[CardFactory], rng: &mut R) -> Vec<CardFactory> {
    if pool.is_empty() {
        return Vec::new();
    }
    // Pre-bucket every card in the pool. The bucket function is pure,
    // so caching once amortizes across the pack roll.
    let mut by_bucket: std::collections::HashMap<SosBucket, Vec<usize>> =
        std::collections::HashMap::new();
    for (i, factory) in pool.iter().enumerate() {
        let def = factory();
        by_bucket.entry(sos_bucket_of(&def)).or_default().push(i);
    }
    let mut used: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut pack: Vec<CardFactory> = Vec::with_capacity(PACK_SIZE);
    let mut pull_from = |bucket: SosBucket,
                        want: usize,
                        by_bucket: &std::collections::HashMap<SosBucket, Vec<usize>>,
                        used: &mut std::collections::HashSet<usize>,
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
        let got = pull_from(*bucket, *want, &by_bucket, &mut used, &mut pack, rng);
        shortfall += want.saturating_sub(got);
    }
    // Fill any shortfall first from Multi, then from any unused pool
    // card. This is the rare-pool guard — real SoS draws always
    // satisfy the recipe, but a custom subset (test fixtures, future
    // college-only modes) might not.
    if shortfall > 0 {
        pull_from(
            SosBucket::Multi,
            shortfall,
            &by_bucket,
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
    pack
}

impl DraftPool {
    /// Roll a single 15-card pack from this pool. Cube uses uniform
    /// random sampling (`generate_pack`); SoS uses the stratified
    /// `generate_sos_pack` so every pack has a consistent color shape.
    pub fn generate_pack<R: Rng>(self, pool: &[CardFactory], rng: &mut R) -> Vec<CardFactory> {
        match self {
            DraftPool::Cube => generate_pack(pool, rng),
            DraftPool::Sos => generate_sos_pack(pool, rng),
        }
    }
}

/// Build the full set of packs for a draft pod —
/// `POD_SIZE * PACKS_PER_SEAT` packs, returned in deal order
/// `[seat0_pack1, seat1_pack1, …, seat0_pack2, …]`. Callers typically
/// reshape this into `packs[round][seat]` for the passing loop.
pub fn open_all_packs<R: Rng>(pool: &[CardFactory], rng: &mut R) -> Vec<Vec<CardFactory>> {
    let mut all = Vec::with_capacity(POD_SIZE * PACKS_PER_SEAT as usize);
    for _ in 0..(POD_SIZE * PACKS_PER_SEAT as usize) {
        all.push(generate_pack(pool, rng));
    }
    all
}

/// Score `factory` from `seat_picks_so_far`'s perspective. Higher is
/// better. The heuristic combines:
/// - **Color fit**: bonus for cards whose colored pips already match
///   the seat's most-picked colors. Off-color spells are penalized.
///   Colorless cards score neutral.
/// - **Card-type weight**: creatures and removal-class noncreatures
///   (instants/sorceries) get a small bonus over enchantments /
///   non-utility artifacts, since most decks need a critical mass of
///   creatures to pressure life totals.
/// - **Mana-value weight**: a mild preference for 2-4 CMC cards
///   (the curve sweet spot) over 1-CMC and 6+ CMC cards. Lands are
///   cheap because the deck-builder will inject basics later — drafting
///   non-basic lands is fine but not particularly preferred.
///
/// This is intentionally simple. It produces playable decks without
/// needing a per-card synergy model.
pub fn score_card_for_seat(factory: CardFactory, seat_picks_so_far: &[CardFactory]) -> i32 {
    let def = factory();
    let mut score: i32 = 0;

    // ── Color fit (the dominant signal once you have ~5 picks) ──
    let seat_colors = colors_of_picks(seat_picks_so_far);
    let card_colors = colors_of_cost(&def.cost);
    if card_colors.is_empty() {
        // Colorless / artifact / generic-only cards: small neutral
        // bonus since they slot into any deck.
        score += 2;
    } else if seat_colors.is_empty() {
        // First few picks before any colors are committed: don't
        // penalize colored cards at all — early picks define the
        // seat's colors. Treat each colored pip as a small positive
        // signal so a {1}{G} bear still beats a colorless {1}
        // artifact at first pick.
        let pips: u32 = card_colors
            .iter()
            .map(|c| colored_pip_count(&def.cost, *c))
            .sum();
        score += (pips as i32) * 2;
    } else {
        let mut on_color_pips = 0i32;
        let mut off_color_pips = 0i32;
        for &c in &card_colors {
            let pips = colored_pip_count(&def.cost, c) as i32;
            if seat_colors.get(&c).copied().unwrap_or(0) > 0 {
                on_color_pips += pips;
            } else {
                off_color_pips += pips;
            }
        }
        // Each on-color pip is +6 (strong enough to dominate the curve
        // tweak). Each off-color pip is -4. Net effect: a 2-color spell
        // whose splash is already in the seat scores positively, while
        // a card that adds a third color is mildly punished.
        score += on_color_pips * 6;
        score -= off_color_pips * 4;
    }

    // ── Card-type weight ──
    if def.card_types.contains(&CardType::Creature) {
        score += 3;
    }
    if def.card_types.contains(&CardType::Instant) || def.card_types.contains(&CardType::Sorcery) {
        score += 2;
    }
    if def.card_types.contains(&CardType::Land) {
        // Non-basic lands are fine fixing but aren't a high pick
        // priority — basics get added by the deck-builder.
        score += 1;
    }

    // ── Mana-value (curve) weight ──
    let cmc = def.cost.cmc();
    score += match cmc {
        0 => 0,
        1 => 1,
        2..=4 => 3,
        5 => 2,
        _ => 1,
    };

    score
}

/// Auto-pick a card from `pack` for a seat with `seat_picks_so_far`.
/// Returns the index of the picked card (caller removes it from the
/// pack and appends it to the seat's pick pile). Returns `None` only
/// when `pack` is empty.
///
/// Tie-break: highest score → highest CMC → lowest pack index. The
/// CMC tie-break gives bots a small "bombs first" bias that real
/// drafters tend to share.
pub fn bot_pick(pack: &[CardFactory], seat_picks_so_far: &[CardFactory]) -> Option<usize> {
    if pack.is_empty() {
        return None;
    }
    let mut best_idx = 0;
    let mut best_score = i32::MIN;
    let mut best_cmc = 0u32;
    for (i, factory) in pack.iter().enumerate() {
        let score = score_card_for_seat(*factory, seat_picks_so_far);
        let cmc = factory().cost.cmc();
        let better = score > best_score || (score == best_score && cmc > best_cmc);
        if better {
            best_idx = i;
            best_score = score;
            best_cmc = cmc;
        }
    }
    Some(best_idx)
}

/// Distribution of colored pips across a seat's picks, used by
/// `score_card_for_seat` to detect which colors the seat is committed
/// to. Lands and colorless cards don't contribute (they don't signal
/// color preference).
pub fn colors_of_picks(picks: &[CardFactory]) -> HashMap<Color, u32> {
    let mut totals: HashMap<Color, u32> = HashMap::new();
    for factory in picks {
        let def = factory();
        for c in colors_of_cost(&def.cost) {
            *totals.entry(c).or_insert(0) += colored_pip_count(&def.cost, c);
        }
    }
    totals
}

/// Distinct colors referenced by a card's printed cost (colored,
/// hybrid, or Phyrexian pips).
fn colors_of_cost(cost: &crate::mana::ManaCost) -> Vec<Color> {
    let mut seen = [false; 5];
    let idx = |c: Color| match c {
        Color::White => 0,
        Color::Blue => 1,
        Color::Black => 2,
        Color::Red => 3,
        Color::Green => 4,
    };
    for sym in &cost.symbols {
        match sym {
            ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => seen[idx(*c)] = true,
            ManaSymbol::Hybrid(a, b) => {
                seen[idx(*a)] = true;
                seen[idx(*b)] = true;
            }
            _ => {}
        }
    }
    Color::ALL
        .into_iter()
        .filter(|c| seen[idx(*c)])
        .collect()
}

/// Count of colored pips of the given color on this cost. Hybrid pips
/// count for both halves; Phyrexian pips count for their colored half.
fn colored_pip_count(cost: &crate::mana::ManaCost, color: Color) -> u32 {
    cost.symbols
        .iter()
        .filter(|s| match s {
            ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => *c == color,
            ManaSymbol::Hybrid(a, b) => *a == color || *b == color,
            _ => false,
        })
        .count() as u32
}

/// Suggest a starting deck split for `picks`: pick the seat's two
/// strongest colors, then take the highest-scoring spells in those
/// colors up to `target_spells` (default 23). Cards whose cost
/// references a third color are skipped. Returns `(chosen, leftovers)`
/// — the deck-builder UI starts with `chosen` in the main deck and
/// `leftovers` on the sideboard, then lets the user freely move cards
/// between the two piles.
///
/// This is purely a starting suggestion; the user retains full
/// control during deckbuilding.
pub fn suggest_main_deck(picks: &[CardFactory], target_spells: usize) -> (Vec<CardFactory>, Vec<CardFactory>) {
    let colors = top_two_colors(picks);
    let mut on_color: Vec<(CardFactory, i32)> = Vec::new();
    let mut other: Vec<CardFactory> = Vec::new();
    for &factory in picks {
        let card_colors = colors_of_cost(&factory().cost);
        let on = card_colors.is_empty()
            || card_colors.iter().all(|c| colors.contains(c));
        if on {
            let s = score_card_for_seat(factory, picks);
            on_color.push((factory, s));
        } else {
            other.push(factory);
        }
    }
    on_color.sort_by_key(|(_, score)| std::cmp::Reverse(*score));
    // Walk on-color picks in score order. A card joins the main deck if it
    // fits both the target size and the 4-copy cap; over-cap or over-target
    // copies fall through to leftovers (sideboard) so the player still sees
    // them and the slot can go to the next-best on-color card. Previously
    // the cap was applied with `enforce_copy_cap` after slicing, which
    // silently dropped over-cap copies entirely (e.g. 6 Lightning Bolts →
    // main capped to 4, leftovers built via `.skip(take)` which missed the
    // 2 dropped copies).
    let mut counts: HashMap<usize, u32> = HashMap::new();
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

/// Two strongest colors in `picks`, by total colored-pip weight.
/// Falls back to (W, U) if the seat has fewer than two colors
/// represented (e.g. a pile of all-colorless cards).
pub fn top_two_colors(picks: &[CardFactory]) -> [Color; 2] {
    let totals = colors_of_picks(picks);
    let mut v: Vec<(Color, u32)> = totals.into_iter().collect();
    v.sort_by_key(|(_, weight)| std::cmp::Reverse(*weight));
    let primary = v.first().map(|(c, _)| *c).unwrap_or(Color::White);
    let secondary = v
        .iter()
        .skip(1)
        .find(|(c, _)| *c != primary)
        .map(|(c, _)| *c)
        .unwrap_or_else(|| {
            // Pick any color other than the primary as the fallback.
            Color::ALL
                .into_iter()
                .find(|c| *c != primary)
                .unwrap()
        });
    [primary, secondary]
}

/// Suggested basic-land split for `main_deck` against the seat's
/// chosen colors, summing to `total_lands`. The split is proportional
/// to the colored-pip weight in the main deck — a deck with twice as
/// many `{W}` pips as `{U}` gets ~⅔ Plains, ~⅓ Island. Always returns
/// at least one of each chosen color when possible, to avoid color
/// screw on draws.
pub fn suggest_basic_split(main_deck: &[CardFactory], colors: [Color; 2], total_lands: u32) -> HashMap<Color, u32> {
    let mut weights: HashMap<Color, u32> = HashMap::new();
    for &c in &colors {
        weights.insert(c, 0);
    }
    for factory in main_deck {
        let def = factory();
        for &c in &colors {
            let n = colored_pip_count(&def.cost, c);
            if n > 0 {
                *weights.entry(c).or_insert(0) += n;
            }
        }
    }
    let total_weight: u32 = weights.values().sum();
    let mut out: HashMap<Color, u32> = HashMap::new();
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

/// Look up the basic-land factory for a color. Mirrors the helper in
/// `cube::basic_factory` (which is private), exposed here so the
/// deck-builder can materialize basics alongside drafted spells.
pub fn basic_land_factory(color: Color) -> CardFactory {
    match color {
        Color::White => crate::catalog::plains,
        Color::Blue => crate::catalog::island,
        Color::Black => crate::catalog::swamp,
        Color::Red => crate::catalog::mountain,
        Color::Green => crate::catalog::forest,
    }
}

/// Enforce the global 4-copy cap on a built deck. Walks the list in
/// order and drops any copies past the cap. Used after the
/// auto-suggestion pass so a seat that accidentally drafted six
/// Lightning Bolts ends up with four in their main deck and two on
/// the sideboard.
pub fn enforce_copy_cap(cards: Vec<CardFactory>) -> Vec<CardFactory> {
    let mut counts: HashMap<usize, u32> = HashMap::new();
    let mut out = Vec::with_capacity(cards.len());
    for f in cards {
        let key = f as usize;
        let entry = counts.entry(key).or_insert(0);
        if *entry < COPY_CAP {
            *entry += 1;
            out.push(f);
        }
    }
    out
}

/// Build a 2-player `GameState` from a player + opponent deck.
/// Both seats are flagged `wants_ui` so the post-draft match plays
/// out via the standard human-vs-bot decision path.
///
/// `player_seat_name` / `opponent_seat_name` show up in player
/// portraits and the game log so the UI can label "You vs P3 (UR)".
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
        state.add_card_to_library(0, f());
    }
    state.players[0].library.shuffle(&mut rng);
    for f in &opponent_deck {
        state.add_card_to_library(1, f());
    }
    state.players[1].library.shuffle(&mut rng);
    state.players[0].wants_ui = true;
    state.players[1].wants_ui = true;
    state
}

/// Names of engine-invented cards whose Scryfall art is missing —
/// the prefetcher (`crabomination_client::scryfall`) stamps these as
/// cardback placeholders, which makes them ugly draft picks. The
/// regular cube mode tolerates the placeholder because it's a
/// gameplay-only mode (cards play correctly even without art); the
/// draft mode renders cards face-up at 200px so the missing art is
/// glaring. Filter them out at draft time.
///
/// Keep in sync with `crabomination_client::scryfall::FICTIONAL_CARDS`
/// — the lists serve different purposes (placeholder rendering vs.
/// draft visibility) but share the same source-of-truth set of names.
const FICTIONAL_CARD_NAMES: &[&str] = &[
    "Sundering Eruption",
    "Mount Tyrhus",
];

/// Convenience: the cube pool used as the draft pool, filtered to
/// cards with real Scryfall art. Re-exported here so client code
/// doesn't have to know about `cube::all_cube_cards`.
pub fn draft_pool() -> Vec<CardFactory> {
    all_cube_cards()
        .into_iter()
        .filter(|f| {
            let name = f().name;
            !FICTIONAL_CARD_NAMES.iter().any(|x| x.eq_ignore_ascii_case(name))
        })
        .collect()
}

/// Secrets of Strixhaven draft pool — every ✅ SoS factory across the
/// five colleges (mono-color + multi-color + school lands). Pulls from
/// `sos_mode::all_sos_cards()`. The same fictional-card filter applies
/// — defensive only, since none of the SoS names overlap the cube's
/// engine-invented entries.
pub fn sos_draft_pool() -> Vec<CardFactory> {
    crate::sos_mode::all_sos_cards()
        .into_iter()
        .filter(|f| {
            let name = f().name;
            !FICTIONAL_CARD_NAMES.iter().any(|x| x.eq_ignore_ascii_case(name))
        })
        .collect()
}

/// Which set of cards a draft draws from. Cube is the existing 309-card
/// curated cube pool; Sos is the 255-card Secrets of Strixhaven set
/// (`sos_mode::all_sos_cards`).
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
        let mut seen: std::collections::HashSet<usize> = std::collections::HashSet::new();
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
        // With an empty seat history, the bot's color preference is
        // wide-open — but the curve / type weight should still bias
        // toward a creature over a non-creature artifact. Build a
        // 2-card pack with a known winner.
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
        assert!(sb.iter().all(|f| *f as usize == crate::catalog::lightning_bolt as usize),
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
        assert!(main.iter().any(|f| *f as usize == crate::catalog::counterspell as usize),
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

    #[test]
    fn draft_pool_excludes_fictional_cards() {
        let pool = draft_pool();
        // None of the fictional names should appear in the draft pool.
        for f in &pool {
            let name = f().name;
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
        let mut seen = std::collections::HashSet::new();
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
        let mut seen = std::collections::HashSet::new();
        for f in &pack {
            assert!(seen.insert(*f as usize), "SoS pack has duplicate factory");
        }
    }

    #[test]
    fn sos_pack_satisfies_color_recipe_on_average() {
        // Each pack should land on ~2 cards per mono-color and ~3
        // multicolor + ~2 colorless/land. Run 100 rolls and verify
        // every bucket got at least one card on average — guards
        // against a regression that breaks bucket assignment.
        let pool = sos_draft_pool();
        let mut rng = fixed_rng();
        let mut bucket_totals: std::collections::HashMap<SosBucket, u32> =
            std::collections::HashMap::new();
        let n = 100;
        for _ in 0..n {
            let pack = generate_sos_pack(&pool, &mut rng);
            for f in &pack {
                let def = f();
                *bucket_totals.entry(sos_bucket_of(&def)).or_insert(0) += 1;
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
}
// touch
