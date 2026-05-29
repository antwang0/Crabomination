use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

impl Color {
    /// The five colors in WUBRG order. Use this when iterating over all
    /// colors, rather than rebuilding the array at the call site.
    pub const ALL: [Color; 5] = [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ];

    /// Single-letter MTG abbreviation (W/U/B/R/G). Used by the cost
    /// label formatter so `{R}` renders as `{R}` rather than `{Red}`,
    /// and by the cube format's color-pair helper.
    pub const fn short_name(self) -> char {
        match self {
            Color::White => 'W',
            Color::Blue => 'U',
            Color::Black => 'B',
            Color::Red => 'R',
            Color::Green => 'G',
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_name())
    }
}

/// A set of colors (subset of WUBRG). Bit-packed five-bit field with
/// `contains` / `insert` / `is_subset_of` helpers. Used by Phase K's
/// color-identity validator for Commander deckbuilding.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorSet(pub u8);

impl ColorSet {
    /// The empty set (colorless).
    pub const fn empty() -> Self {
        Self(0)
    }

    /// All five colors set ("WUBRG").
    pub const fn all() -> Self {
        Self(0b11111)
    }

    pub const fn single(c: Color) -> Self {
        Self(Self::bit_for(c))
    }

    const fn bit_for(c: Color) -> u8 {
        1 << match c {
            Color::White => 0,
            Color::Blue => 1,
            Color::Black => 2,
            Color::Red => 3,
            Color::Green => 4,
        }
    }

    pub fn insert(&mut self, c: Color) {
        self.0 |= Self::bit_for(c);
    }

    pub fn contains(self, c: Color) -> bool {
        self.0 & Self::bit_for(c) != 0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// True iff every color in `self` is also in `other`. Used by
    /// Commander color-identity: a non-commander card is legal iff
    /// `card_identity.is_subset_of(commander_identity)`.
    pub fn is_subset_of(self, other: ColorSet) -> bool {
        self.0 & !other.0 == 0
    }

    /// Union of two sets.
    pub fn union(self, other: ColorSet) -> ColorSet {
        ColorSet(self.0 | other.0)
    }

    /// Number of colors in the set.
    pub fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// CR 105.2a — true iff this set is exactly one color. Used by
    /// effects that key on "monocolored objects" (e.g. Painter's
    /// Servant, monochrome charms).
    pub fn is_monocolored(self) -> bool {
        self.len() == 1
    }

    /// CR 105.2b — true iff this set is two or more colors. Used by
    /// "multicolored" matters effects (Naya Charm, Brokers Charm,
    /// Maelstrom Wanderer variants).
    pub fn is_multicolored(self) -> bool {
        self.len() >= 2
    }

    /// CR 105.2c — true iff this set has no colors. Note: "colorless"
    /// is not a color (CR 105.4) — this is just a tag for objects that
    /// are colored by nothing. Mirrors `is_empty()` with an explicit
    /// rules-anchored name; both are kept for readability at the call
    /// site.
    pub fn is_colorless(self) -> bool {
        self.0 == 0
    }
}

/// A single symbol in a mana cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManaSymbol {
    /// One mana of a specific color.
    Colored(Color),
    /// {n} mana of any type.
    Generic(u32),
    /// {C}: must be paid with colorless mana specifically.
    Colorless(u32),
    /// {A/B}: pay with either color.
    Hybrid(Color, Color),
    /// {C/P}: pay the colored cost or 2 life.
    Phyrexian(Color),
    /// {n/C}: monocolored hybrid — pay either {n} generic mana or one
    /// mana of the given color (e.g. {2/R}). Mana value is `n`
    /// (CR 202.3f). Used by the SOS "Archaic" Avatar cycle.
    MonoHybrid(u32, Color),
    /// {S}: pay with mana from a snow source.
    Snow,
    /// {X}: variable cost determined at cast time.
    X,
}

/// The full mana cost of a card or ability (e.g. {3}{W}{W} for Serra Angel).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ManaCost {
    pub symbols: Vec<ManaSymbol>,
}

impl ManaCost {
    pub fn new(symbols: Vec<ManaSymbol>) -> Self {
        Self { symbols }
    }

    /// Mana value (formerly converted mana cost): sum of all pip values.
    pub fn cmc(&self) -> u32 {
        self.symbols
            .iter()
            .map(|s| match s {
                ManaSymbol::Colored(_) => 1,
                ManaSymbol::Generic(n) => *n,
                ManaSymbol::Colorless(n) => *n,
                ManaSymbol::Hybrid(_, _) => 1,
                ManaSymbol::Phyrexian(_) => 1,
                // CR 202.3f: monocolored hybrid MV is the generic amount.
                ManaSymbol::MonoHybrid(n, _) => *n,
                ManaSymbol::Snow => 1,
                ManaSymbol::X => 0, // X is 0 everywhere except on the stack
            })
            .sum()
    }

    /// True if this cost contains any X symbols.
    pub fn has_x(&self) -> bool {
        self.symbols.iter().any(|s| matches!(s, ManaSymbol::X))
    }

    /// Number of *distinct* colors referenced by this cost. Hybrid pips
    /// (`{W/B}`) contribute both halves; Phyrexian pips (`{B/P}`) contribute
    /// their colored half. Generic / Colorless / Snow / X return 0.
    /// Used by `SelectionRequirement::Multicolored` / `Colorless` and any
    /// Converge-style payoff that wants to peek at the printed pip set.
    pub fn distinct_colors(&self) -> u32 {
        let mut seen = [false; 5];
        let idx = |c: Color| match c {
            Color::White => 0,
            Color::Blue => 1,
            Color::Black => 2,
            Color::Red => 3,
            Color::Green => 4,
        };
        for s in &self.symbols {
            match s {
                ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => {
                    seen[idx(*c)] = true;
                }
                ManaSymbol::Hybrid(a, b) => {
                    seen[idx(*a)] = true;
                    seen[idx(*b)] = true;
                }
                ManaSymbol::MonoHybrid(_, c) => {
                    seen[idx(*c)] = true;
                }
                _ => {}
            }
        }
        seen.iter().filter(|b| **b).count() as u32
    }

    /// Render this cost as a printed-Oracle-style string like
    /// `{2}{W}{B}` or `{X}{R}{R}`. Used by client tooltips
    /// (Cycling / Flashback / activated-ability cost labels) and the
    /// server view's `format_mana_cost_for_label`. Free-cost {0} costs
    /// render as `{0}` rather than the empty string so the renderer
    /// always has something to display.
    pub fn summary(&self) -> String {
        if self.symbols.is_empty() {
            return "{0}".into();
        }
        let mut s = String::new();
        for sym in &self.symbols {
            match sym {
                ManaSymbol::Generic(n) => s.push_str(&format!("{{{n}}}")),
                ManaSymbol::Colorless(n) => {
                    for _ in 0..*n {
                        s.push_str("{C}");
                    }
                }
                ManaSymbol::Colored(col) => {
                    s.push_str(&format!("{{{}}}", color_pip_letter(*col)));
                }
                ManaSymbol::Hybrid(a, b) => {
                    s.push_str(&format!(
                        "{{{}/{}}}",
                        color_pip_letter(*a),
                        color_pip_letter(*b),
                    ));
                }
                ManaSymbol::Phyrexian(c) => {
                    s.push_str(&format!("{{{}/P}}", color_pip_letter(*c)));
                }
                ManaSymbol::MonoHybrid(n, c) => {
                    s.push_str(&format!("{{{}/{}}}", n, color_pip_letter(*c)));
                }
                ManaSymbol::Snow => s.push_str("{S}"),
                ManaSymbol::X => s.push_str("{X}"),
            }
        }
        s
    }

    /// Returns the set of colors present in this mana cost.
    pub fn colors(&self) -> Vec<Color> {
        let mut result = Vec::new();
        for s in &self.symbols {
            match s {
                ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => {
                    if !result.contains(c) {
                        result.push(*c);
                    }
                }
                ManaSymbol::Hybrid(a, b) => {
                    if !result.contains(a) {
                        result.push(*a);
                    }
                    if !result.contains(b) {
                        result.push(*b);
                    }
                }
                ManaSymbol::MonoHybrid(_, c) => {
                    if !result.contains(c) {
                        result.push(*c);
                    }
                }
                _ => {}
            }
        }
        result
    }

    /// Return a copy of this cost with X symbols replaced by Generic(x_value).
    pub fn with_x_value(&self, x_value: u32) -> ManaCost {
        ManaCost {
            symbols: self
                .symbols
                .iter()
                .map(|s| match s {
                    ManaSymbol::X => ManaSymbol::Generic(x_value),
                    other => *other,
                })
                .collect(),
        }
    }

    /// Subtract `amount` from this cost's total Generic pips, clamping at
    /// zero. Colored / colorless / hybrid / Phyrexian / snow / X pips are
    /// untouched — CR 601.2f and CR 117.7c forbid cost reductions from
    /// reducing a colored or X pip. Returns the actually-applied
    /// reduction (so callers can short-circuit when the cost is already
    /// floored). If multiple Generic pips exist, drain them in order.
    pub fn reduce_generic(&mut self, amount: u32) -> u32 {
        let mut remaining = amount;
        let mut applied = 0u32;
        let mut new_syms = Vec::with_capacity(self.symbols.len());
        for sym in &self.symbols {
            match sym {
                ManaSymbol::Generic(n) if remaining > 0 => {
                    let drained = remaining.min(*n);
                    remaining -= drained;
                    applied += drained;
                    let kept = n - drained;
                    if kept > 0 {
                        new_syms.push(ManaSymbol::Generic(kept));
                    }
                }
                other => new_syms.push(*other),
            }
        }
        self.symbols = new_syms;
        applied
    }
}

/// Available mana in a player's pool during their turn.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManaPool {
    white: u32,
    blue: u32,
    black: u32,
    red: u32,
    green: u32,
    /// True colorless mana (from Wastes, Eldrazi sources, Sol Ring, etc.).
    colorless: u32,
    /// How many mana in the pool came from snow sources.
    snow: u32,
}

/// Side effects produced by paying a mana cost (e.g. Phyrexian mana life loss).
#[derive(Debug, Clone, Default)]
pub struct PaymentSideEffects {
    pub life_lost: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManaError {
    InsufficientColored { color: Color, needed: u32, have: u32 },
    InsufficientGeneric { needed: u32, have: u32 },
    InsufficientColorless { needed: u32, have: u32 },
    InsufficientSnow { needed: u32, have: u32 },
    CannotPayHybrid { color_a: Color, color_b: Color },
}

impl fmt::Display for ManaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManaError::InsufficientColored { color, needed, have } => {
                write!(f, "Need {needed} {:?} mana but only have {have}", color)
            }
            ManaError::InsufficientGeneric { needed, have } => {
                write!(f, "Need {needed} generic mana but only have {have} total")
            }
            ManaError::InsufficientColorless { needed, have } => {
                write!(f, "Need {needed} colorless mana but only have {have}")
            }
            ManaError::InsufficientSnow { needed, have } => {
                write!(f, "Need {needed} snow mana but only have {have}")
            }
            ManaError::CannotPayHybrid { color_a, color_b } => {
                write!(f, "Cannot pay hybrid {:?}/{:?} cost", color_a, color_b)
            }
        }
    }
}

impl std::error::Error for ManaError {}

impl ManaPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, color: Color, amount: u32) {
        *self.slot_mut(color) += amount;
    }

    pub fn add_colorless(&mut self, amount: u32) {
        self.colorless += amount;
    }

    /// Add mana from a snow source. The mana is both colored and snow.
    pub fn add_snow(&mut self, color: Color, amount: u32) {
        *self.slot_mut(color) += amount;
        self.snow += amount;
    }

    /// Add colorless mana from a snow source.
    pub fn add_snow_colorless(&mut self, amount: u32) {
        self.colorless += amount;
        self.snow += amount;
    }

    pub fn amount(&self, color: Color) -> u32 {
        *self.slot(color)
    }

    pub fn colorless_amount(&self) -> u32 {
        self.colorless
    }

    pub fn snow_amount(&self) -> u32 {
        self.snow
    }

    pub fn total(&self) -> u32 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless
    }

    /// Pay a ManaCost atomically. On failure the pool is left unchanged.
    ///
    /// Returns side effects (e.g. life loss from Phyrexian mana) on success.
    ///
    /// Algorithm:
    /// 1. Pay each `Colored` pip from the matching bucket.
    /// 2. Pay `Colorless` pips from the colorless bucket only.
    /// 3. Pay `Hybrid` pips from whichever color is available (tries A first).
    /// 4. Pay `Phyrexian` pips from color if available, else 2 life.
    /// 5. Pay `Snow` pips from the snow counter (any color).
    /// 6. Pay all `Generic` pips from whatever remains (any color or colorless).
    ///
    /// X symbols should be replaced before calling this (via `ManaCost::with_x_value`).
    pub fn pay(&mut self, cost: &ManaCost) -> Result<PaymentSideEffects, ManaError> {
        let mut tmp = self.clone();
        let mut side_effects = PaymentSideEffects::default();

        // Pass 1: colored pips
        for sym in &cost.symbols {
            if let ManaSymbol::Colored(c) = sym {
                let have = tmp.amount(*c);
                if have == 0 {
                    return Err(ManaError::InsufficientColored {
                        color: *c,
                        needed: 1,
                        have: 0,
                    });
                }
                *tmp.slot_mut(*c) -= 1;
            }
        }

        // Pass 2: colorless-specific pips ({C})
        let colorless_needed: u32 = cost
            .symbols
            .iter()
            .filter_map(|s| if let ManaSymbol::Colorless(n) = s { Some(*n) } else { None })
            .sum();
        if colorless_needed > 0 {
            if colorless_needed > tmp.colorless {
                return Err(ManaError::InsufficientColorless {
                    needed: colorless_needed,
                    have: tmp.colorless,
                });
            }
            tmp.colorless -= colorless_needed;
        }

        // Pass 3: hybrid pips ({a/b}). Resolve with constraint
        // propagation — pay any pip that currently has only one
        // affordable color *first*, so a limited pool isn't spent on the
        // wrong half. Example: paying {W/R}{W/G} from a {W, R} pool must
        // take W for the {W/G} pip and R for the {W/R} pip; a naive
        // "always try A first" pays W for {W/R} then can't cover {W/G}.
        let mut hybrids: Vec<(Color, Color)> = cost
            .symbols
            .iter()
            .filter_map(|s| match s {
                ManaSymbol::Hybrid(a, b) => Some((*a, *b)),
                _ => None,
            })
            .collect();
        while !hybrids.is_empty() {
            // Prefer a "forced" pip (exactly one color affordable);
            // otherwise the first pip with any affordable color.
            let forced = hybrids
                .iter()
                .position(|(a, b)| (tmp.amount(*a) > 0) ^ (tmp.amount(*b) > 0));
            let idx = match forced.or_else(|| {
                hybrids
                    .iter()
                    .position(|(a, b)| tmp.amount(*a) > 0 || tmp.amount(*b) > 0)
            }) {
                Some(i) => i,
                None => {
                    let (a, b) = hybrids[0];
                    return Err(ManaError::CannotPayHybrid { color_a: a, color_b: b });
                }
            };
            let (a, b) = hybrids.remove(idx);
            if tmp.amount(a) > 0 {
                *tmp.slot_mut(a) -= 1;
            } else {
                *tmp.slot_mut(b) -= 1;
            }
        }

        // Pass 3.5: monocolored hybrid pips ({n/C}). Prefer paying the
        // colored side (1 mana) when a matching colored mana is on hand;
        // otherwise pay {n} generic from any remaining bucket. This is
        // greedy but optimal for the single-pip "Archaic" costs.
        for sym in &cost.symbols {
            if let ManaSymbol::MonoHybrid(n, c) = sym {
                if tmp.amount(*c) > 0 {
                    *tmp.slot_mut(*c) -= 1;
                } else {
                    // Pay {n} generic from any bucket (colors then colorless).
                    let mut rem = *n;
                    for color in Color::ALL {
                        if rem == 0 { break; }
                        let drain = rem.min(tmp.amount(color));
                        *tmp.slot_mut(color) -= drain;
                        rem -= drain;
                    }
                    if rem > 0 {
                        let drain = rem.min(tmp.colorless);
                        tmp.colorless -= drain;
                        rem -= drain;
                    }
                    if rem > 0 {
                        return Err(ManaError::InsufficientGeneric {
                            needed: *n,
                            have: *n - rem,
                        });
                    }
                }
            }
        }

        // Pass 4: phyrexian pips (pay color if available, else 2 life)
        for sym in &cost.symbols {
            if let ManaSymbol::Phyrexian(c) = sym {
                if tmp.amount(*c) > 0 {
                    *tmp.slot_mut(*c) -= 1;
                } else {
                    side_effects.life_lost += 2;
                }
            }
        }

        // Pass 5: snow pips
        let snow_needed: u32 = cost
            .symbols
            .iter()
            .filter(|s| matches!(s, ManaSymbol::Snow))
            .count() as u32;
        if snow_needed > 0 {
            if snow_needed > tmp.snow {
                return Err(ManaError::InsufficientSnow {
                    needed: snow_needed,
                    have: tmp.snow,
                });
            }
            tmp.snow -= snow_needed;
            // Also drain one mana from any bucket per snow pip
            let mut rem = snow_needed;
            for color in Color::ALL {
                if rem == 0 { break; }
                let drain = rem.min(tmp.amount(color));
                *tmp.slot_mut(color) -= drain;
                rem -= drain;
            }
            if rem > 0 {
                let drain = rem.min(tmp.colorless);
                tmp.colorless -= drain;
                rem -= drain;
            }
            if rem > 0 {
                return Err(ManaError::InsufficientSnow {
                    needed: snow_needed,
                    have: self.snow,
                });
            }
        }

        // Pass 6: generic pips (drain from any bucket)
        let generic: u32 = cost
            .symbols
            .iter()
            .filter_map(|s| {
                if let ManaSymbol::Generic(n) = s {
                    Some(*n)
                } else {
                    None
                }
            })
            .sum();

        if generic > 0 {
            let have = tmp.total();
            if generic > have {
                return Err(ManaError::InsufficientGeneric {
                    needed: generic,
                    have,
                });
            }
            let mut rem = generic;
            for color in Color::ALL {
                if rem == 0 { break; }
                let drain = rem.min(tmp.amount(color));
                *tmp.slot_mut(color) -= drain;
                rem -= drain;
            }
            // Drain colorless last for generic
            if rem > 0 {
                tmp.colorless -= rem;
            }
        }

        // The snow counter tracks "of the mana in the pool, how many
        // came from snow sources." Passes 1/2/3/4/6 drain colored or
        // colorless buckets without knowing which underlying mana was
        // snow, so the counter can drift above the remaining total.
        // Clamp it down — we can't distinguish which mana was snow, so
        // conservatively cap snow at the actual remaining pool size.
        tmp.snow = tmp.snow.min(tmp.total());

        *self = tmp;
        Ok(side_effects)
    }

    /// Spend `n` generic mana from the pool, draining colorless first, then
    /// colors in WUBRG order. Panics if `total() < n`.
    pub fn spend_generic(&mut self, mut n: u32) {
        let drain = n.min(self.colorless);
        self.colorless -= drain;
        n -= drain;
        for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            if n == 0 { break; }
            let have = self.slot_mut(color);
            let drain = n.min(*have);
            *have -= drain;
            n -= drain;
        }
        // See `pay()`: snow counter doesn't track which mana was snow.
        // Clamp conservatively so we never claim more snow than total.
        self.snow = self.snow.min(self.total());
    }

    pub fn empty(&mut self) {
        *self = Self::default();
    }

    fn slot(&self, color: Color) -> &u32 {
        match color {
            Color::White => &self.white,
            Color::Blue => &self.blue,
            Color::Black => &self.black,
            Color::Red => &self.red,
            Color::Green => &self.green,
        }
    }

    fn slot_mut(&mut self, color: Color) -> &mut u32 {
        match color {
            Color::White => &mut self.white,
            Color::Blue => &mut self.blue,
            Color::Black => &mut self.black,
            Color::Red => &mut self.red,
            Color::Green => &mut self.green,
        }
    }
}

/// Single-letter MTG color identifier — used by every mana-cost
/// label renderer (server view, client tooltip, debug dumps). Pulled
/// out so the four-or-five duplicated `match c { White → 'W', … }`
/// inlines in `view.rs` and `mana.rs` resolve to one definition.
pub fn color_pip_letter(c: Color) -> char {
    match c {
        Color::White => 'W',
        Color::Blue => 'U',
        Color::Black => 'B',
        Color::Red => 'R',
        Color::Green => 'G',
    }
}

// ── Convenience constructors ──────────────────────────────────────────────────

pub fn colored(color: Color) -> ManaSymbol {
    ManaSymbol::Colored(color)
}

pub fn generic(n: u32) -> ManaSymbol {
    ManaSymbol::Generic(n)
}

pub fn colorless(n: u32) -> ManaSymbol {
    ManaSymbol::Colorless(n)
}

pub fn hybrid(a: Color, b: Color) -> ManaSymbol {
    ManaSymbol::Hybrid(a, b)
}

pub fn phyrexian(color: Color) -> ManaSymbol {
    ManaSymbol::Phyrexian(color)
}

pub fn mono_hybrid(n: u32, color: Color) -> ManaSymbol {
    ManaSymbol::MonoHybrid(n, color)
}

pub fn snow_mana() -> ManaSymbol {
    ManaSymbol::Snow
}

pub fn x() -> ManaSymbol {
    ManaSymbol::X
}

pub fn w() -> ManaSymbol {
    ManaSymbol::Colored(Color::White)
}
pub fn u() -> ManaSymbol {
    ManaSymbol::Colored(Color::Blue)
}
pub fn b() -> ManaSymbol {
    ManaSymbol::Colored(Color::Black)
}
pub fn r() -> ManaSymbol {
    ManaSymbol::Colored(Color::Red)
}
pub fn g() -> ManaSymbol {
    ManaSymbol::Colored(Color::Green)
}

pub fn cost(symbols: &[ManaSymbol]) -> ManaCost {
    ManaCost::new(symbols.to_vec())
}

#[cfg(test)]
#[path = "tests/mana.rs"]
mod tests;
