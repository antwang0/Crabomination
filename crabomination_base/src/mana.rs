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
                ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c)
                    if !result.contains(c) => {
                        result.push(*c);
                    }
                ManaSymbol::Hybrid(a, b) => {
                    if !result.contains(a) {
                        result.push(*a);
                    }
                    if !result.contains(b) {
                        result.push(*b);
                    }
                }
                ManaSymbol::MonoHybrid(_, c)
                    if !result.contains(c) => {
                        result.push(*c);
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

/// A "spend this mana only to …" restriction carried by certain mana
/// sources (the Strixhaven school mana dorks/artifacts/lands: Abstract
/// Paintmage, Tablet of Discovery, Hydro-Channeler, Great Hall of the
/// Biblioplex, Resonating Lute). Mana tagged with a restriction lives in
/// a separate pool bucket and can only pay for things the restriction
/// permits — see [`ManaPool::pay_for_spell`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpendRestriction {
    /// "Spend this mana only to cast instant and sorcery spells."
    InstantSorceryOnly,
    /// "Spend this mana only to cast artifact spells or activate
    /// abilities of artifacts." (Power Depot, Mishra's Workshop kin.)
    ArtifactOnly,
    /// "Spend this mana only to cast a creature spell of the chosen
    /// type, and that spell can't be countered." (Cavern of Souls.)
    /// Spending it stamps the cast uncounterable via
    /// [`PaymentSideEffects::spent_restrictions`].
    CreatureOfTypeUncounterable(crate::card::CreatureType),
    /// "Spend this mana only to cast [type] creature spells" — the plain
    /// per-type restriction without Cavern's uncounterable rider
    /// (Eldrazi Temple).
    CreatureOfType(crate::card::CreatureType),
    /// "Spend this mana only to activate abilities of land sources."
    /// (Sunken Citadel.)
    LandAbilitiesOnly,
    /// "Spend this mana only to cast a creature spell." (Ancient Ziggurat.)
    CreatureOnly,
    /// "Spend this mana only to cast creature spells or activate abilities
    /// of creatures." (Castle Garenbrig.)
    CreatureSpellsOrAbilities,
}

impl SpendRestriction {
    /// True iff mana under this restriction may fund a payment of `kind`.
    pub fn allows(self, kind: &SpellKind) -> bool {
        match self {
            SpendRestriction::InstantSorceryOnly => kind.instant_or_sorcery,
            SpendRestriction::ArtifactOnly => kind.artifact,
            SpendRestriction::LandAbilitiesOnly => kind.land_ability,
            SpendRestriction::CreatureOfTypeUncounterable(t)
            | SpendRestriction::CreatureOfType(t) => {
                kind.changeling || kind.creature_types.contains(&t)
            }
            SpendRestriction::CreatureOnly => kind.creature,
            SpendRestriction::CreatureSpellsOrAbilities => {
                kind.creature || kind.creature_ability
            }
        }
    }
}

/// What a payment is funding, so spend-restricted mana can decide whether
/// it is allowed to contribute. Spell casts build this from the card being
/// cast ([`crate::card::CardDefinition::spell_kind`]); ability activations
/// describe their source ([`crate::card::CardDefinition::ability_spend_kind`]);
/// everything else pays with the empty [`SpellKind::default`], which no
/// restricted mana may fund.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpellKind {
    /// Casting an instant or sorcery spell.
    pub instant_or_sorcery: bool,
    /// Casting an artifact spell, or activating an ability of an artifact.
    pub artifact: bool,
    /// The creature types of a creature spell being cast (empty otherwise).
    pub creature_types: Vec<crate::card::CreatureType>,
    /// The spell is a Changeling (every creature type, CR 702.73).
    pub changeling: bool,
    /// Activating an ability of a land source (Sunken Citadel).
    pub land_ability: bool,
    /// Casting a creature spell (Ancient Ziggurat).
    pub creature: bool,
    /// Activating an ability of a creature (Castle Garenbrig).
    pub creature_ability: bool,
}

/// WUBRG index for a color — used to bucket restricted mana per color.
const fn color_index(c: Color) -> usize {
    match c {
        Color::White => 0,
        Color::Blue => 1,
        Color::Black => 2,
        Color::Red => 3,
        Color::Green => 4,
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
    /// Colored mana carrying a "spend only on …" restriction. Kept out of
    /// the flat color buckets so the unrestricted [`pay`]/[`total`] paths
    /// never see it; [`pay_for_spell`] folds in the entries whose
    /// restriction permits the spell being cast (draining them ahead of
    /// unrestricted mana of the same color). Empty for all but a handful
    /// of Strixhaven sources, so the common path is unaffected.
    #[serde(default)]
    restricted: Vec<(Color, u32, SpendRestriction)>,
}

/// Side effects produced by paying a mana cost (e.g. Phyrexian mana life loss).
#[derive(Debug, Clone, Default)]
pub struct PaymentSideEffects {
    pub life_lost: u32,
    /// Restrictions of the spend-restricted mana that actually funded the
    /// payment — lets the cast path apply spend-triggered riders (Cavern
    /// of Souls' "and that spell can't be countered").
    pub spent_restrictions: Vec<SpendRestriction>,
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

    /// Add `amount` mana of `color` carrying a spend `restriction`. The
    /// mana is held apart from the flat color buckets and only becomes
    /// spendable through [`pay_for_spell`] when the restriction permits
    /// the spell being cast. Entries with the same (color, restriction)
    /// coalesce so the vec stays short.
    pub fn add_restricted(&mut self, color: Color, amount: u32, restriction: SpendRestriction) {
        if amount == 0 {
            return;
        }
        if let Some(entry) = self
            .restricted
            .iter_mut()
            .find(|(c, _, r)| *c == color && *r == restriction)
        {
            entry.1 += amount;
        } else {
            self.restricted.push((color, amount, restriction));
        }
    }

    /// Total spend-restricted mana floating in the pool (any color/
    /// restriction). Exposed for UI/debug surfaces that show floated mana;
    /// `total()` deliberately excludes it since it isn't freely spendable.
    pub fn restricted_total(&self) -> u32 {
        self.restricted.iter().map(|(_, n, _)| *n).sum()
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

    /// Render the floating mana as Oracle-style pips (`{R}{R}{U}{C}`) for UI
    /// prompts (e.g. the "spend floating mana?" confirmation). Empty pool →
    /// `{0}`. Spend-restricted mana isn't freely spendable, so it's omitted.
    pub fn summary(&self) -> String {
        let mut s = String::new();
        for (color, n) in [
            (Color::White, self.white),
            (Color::Blue, self.blue),
            (Color::Black, self.black),
            (Color::Red, self.red),
            (Color::Green, self.green),
        ] {
            for _ in 0..n {
                s.push('{');
                s.push(color_pip_letter(color));
                s.push('}');
            }
        }
        for _ in 0..self.colorless {
            s.push_str("{C}");
        }
        if s.is_empty() {
            "{0}".into()
        } else {
            s
        }
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

        // Pass 3: hybrid ({a/b}) and mono-hybrid ({n/C}) pips, assigned
        // jointly by backtracking so a tight pool routes each pip to the
        // half that keeps the rest payable (CR 601.2g) — `{W/U}{W/G}{W/G}`
        // from {W,U,G} and `{W/U}{2/W}` from {W,U} both resolve. The pip
        // count is tiny, so the worst case is a handful of branches. The
        // search also leaves enough total mana behind for the cost's
        // later generic / snow drains.
        enum HyPip {
            Hybrid(Color, Color),
            Mono(u32, Color),
        }
        let pips: Vec<HyPip> = cost
            .symbols
            .iter()
            .filter_map(|s| match s {
                ManaSymbol::Hybrid(a, b) => Some(HyPip::Hybrid(*a, *b)),
                ManaSymbol::MonoHybrid(n, c) => Some(HyPip::Mono(*n, *c)),
                _ => None,
            })
            .collect();
        if !pips.is_empty() {
            let downstream: u32 = cost
                .symbols
                .iter()
                .map(|s| match s {
                    ManaSymbol::Generic(n) => *n,
                    ManaSymbol::Snow => 1,
                    _ => 0,
                })
                .sum();
            fn assign(
                pool: &ManaPool,
                pips: &[HyPip],
                deferred_generic: u32,
                downstream: u32,
            ) -> Option<ManaPool> {
                let Some((pip, rest)) = pips.split_first() else {
                    if pool.total() < deferred_generic + downstream {
                        return None;
                    }
                    // Deduct the mono-hybrid generic halves now (colors
                    // first, colorless last — same bucket order as the
                    // generic pass).
                    let mut done = pool.clone();
                    let mut rem = deferred_generic;
                    for color in Color::ALL {
                        if rem == 0 {
                            break;
                        }
                        let drain = rem.min(done.amount(color));
                        *done.slot_mut(color) -= drain;
                        rem -= drain;
                    }
                    let drain = rem.min(done.colorless);
                    done.colorless -= drain;
                    rem -= drain;
                    return (rem == 0).then_some(done);
                };
                let options: Vec<(Option<Color>, u32)> = match pip {
                    HyPip::Hybrid(a, b) => vec![(Some(*a), 0), (Some(*b), 0)],
                    // Colored half first — it's 1 mana vs {n}.
                    HyPip::Mono(n, c) => vec![(Some(*c), 0), (None, *n)],
                };
                for (color, generic) in options {
                    let mut next = pool.clone();
                    if let Some(c) = color {
                        if next.amount(c) == 0 {
                            continue;
                        }
                        *next.slot_mut(c) -= 1;
                    }
                    if let Some(done) =
                        assign(&next, rest, deferred_generic + generic, downstream)
                    {
                        return Some(done);
                    }
                }
                None
            }
            match assign(&tmp, &pips, 0, downstream) {
                Some(done) => tmp = done,
                None => {
                    // Surface the most informative error for the pip mix.
                    if let Some(HyPip::Hybrid(a, b)) =
                        pips.iter().find(|p| matches!(p, HyPip::Hybrid(..)))
                    {
                        return Err(ManaError::CannotPayHybrid { color_a: *a, color_b: *b });
                    }
                    let needed =
                        pips.iter().map(|p| if let HyPip::Mono(n, _) = p { *n } else { 0 }).sum();
                    return Err(ManaError::InsufficientGeneric { needed, have: tmp.total() });
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
            // Drain colorless first — generic pips shouldn't eat colored
            // mana a follow-up payment (splice cost, kicker, a second
            // spell) might need. Any {C} pips in THIS cost were already
            // taken in pass 2.
            let mut rem = generic;
            let drain = rem.min(tmp.colorless);
            tmp.colorless -= drain;
            rem -= drain;
            for color in Color::ALL {
                if rem == 0 { break; }
                let drain = rem.min(tmp.amount(color));
                *tmp.slot_mut(color) -= drain;
                rem -= drain;
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

    /// Pay `cost` to cast a spell of `kind`, allowing spend-restricted
    /// mana whose restriction permits `kind` to contribute. Restricted
    /// mana of a color is drained ahead of unrestricted mana of that color
    /// so it isn't wasted when the pool empties (CR 605 lets restricted
    /// mana be applied freely; spending it first is always legal once the
    /// full cost is covered). When the pool holds no usable restricted
    /// mana — the overwhelmingly common case — this is exactly [`pay`].
    ///
    /// On failure the pool is left unchanged.
    pub fn pay_for_spell(
        &mut self,
        cost: &ManaCost,
        kind: &SpellKind,
    ) -> Result<PaymentSideEffects, ManaError> {
        // How much restricted mana, per color, may fund a spell of `kind`?
        let mut spendable = [0u32; 5];
        for (c, n, r) in &self.restricted {
            if r.allows(kind) {
                spendable[color_index(*c)] += *n;
            }
        }
        if spendable.iter().all(|n| *n == 0) {
            // No usable restricted mana — identical to the plain path,
            // which leaves the restricted bucket untouched.
            return self.pay(cost);
        }

        // Fold the spendable restricted mana into a working clone's flat
        // buckets and pay there. Within a color, restricted and
        // unrestricted mana are fungible for payment, so the total spent
        // per color is fixed by `pay`; we only have to split the drain
        // afterward (restricted first).
        let mut work = self.clone();
        work.restricted.clear();
        for c in Color::ALL {
            work.add(c, spendable[color_index(c)]);
        }
        let mut side_effects = work.pay(cost)?;

        // Settle each color: restricted drains before unrestricted.
        let mut result = work.clone();
        result.restricted = self.restricted.clone();
        for c in Color::ALL {
            let idx = color_index(c);
            let before = self.amount(c) + spendable[idx];
            let after = work.amount(c);
            let spent = before - after;
            let from_restricted = spent.min(spendable[idx]);
            let from_unrestricted = spent - from_restricted;
            // Flat bucket keeps the unspent unrestricted mana.
            *result.slot_mut(c) = self.amount(c) - from_unrestricted;
            // Drain `from_restricted` from this color's permitting entries.
            let mut rem = from_restricted;
            for entry in result.restricted.iter_mut() {
                if rem == 0 {
                    break;
                }
                if entry.0 == c && entry.2.allows(kind) {
                    let d = rem.min(entry.1);
                    entry.1 -= d;
                    rem -= d;
                    if d > 0 && !side_effects.spent_restrictions.contains(&entry.2) {
                        side_effects.spent_restrictions.push(entry.2);
                    }
                }
            }
        }
        result.restricted.retain(|(_, n, _)| *n > 0);
        // Folding restricted mana into the flat buckets inflated `total()`
        // for the snow clamp in `pay`; re-clamp against the real total now
        // that the restricted remainder is back out of the buckets.
        result.snow = result.snow.min(result.total());

        *self = result;
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

    /// Fold every bucket of `other` into this pool (colors, colorless, snow,
    /// and spend-restricted entries). Used to restore protected floating mana
    /// after paying a cost from freshly-tapped sources (the "keep my floating
    /// mana" branch of the float-spend confirmation).
    pub fn absorb(&mut self, other: &ManaPool) {
        self.white += other.white;
        self.blue += other.blue;
        self.black += other.black;
        self.red += other.red;
        self.green += other.green;
        self.colorless += other.colorless;
        self.snow += other.snow;
        for (c, n, r) in &other.restricted {
            self.add_restricted(*c, *n, *r);
        }
    }

    /// Saturating inverse of [`absorb`](Self::absorb) over the plain colour /
    /// colorless buckets — used to lift out "protected" floating mana (the
    /// off-colour excess the player chose to keep) before paying a cost from
    /// freshly-tapped sources, then `absorb` it back afterward. Snow / restricted
    /// buckets are left untouched (protected pools only carry plain mana).
    pub fn remove_pool(&mut self, other: &ManaPool) {
        self.white = self.white.saturating_sub(other.white);
        self.blue = self.blue.saturating_sub(other.blue);
        self.black = self.black.saturating_sub(other.black);
        self.red = self.red.saturating_sub(other.red);
        self.green = self.green.saturating_sub(other.green);
        self.colorless = self.colorless.saturating_sub(other.colorless);
        self.snow = self.snow.min(self.total());
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

#[cfg(test)]
mod hybrid_solver_tests {
    use super::*;

    fn pool(colors: &[Color]) -> ManaPool {
        let mut p = ManaPool::default();
        for &c in colors {
            p.add(c, 1);
        }
        p
    }

    #[test]
    fn hybrid_assignment_backtracks_across_pips() {
        // {W/U}{W/G}{W/G} from {W,U,G}: only U→{W/U}, W+G→{W/G}s works.
        let mut p = pool(&[Color::White, Color::Blue, Color::Green]);
        let c = ManaCost {
            symbols: vec![
                ManaSymbol::Hybrid(Color::White, Color::Blue),
                ManaSymbol::Hybrid(Color::White, Color::Green),
                ManaSymbol::Hybrid(Color::White, Color::Green),
            ],
        };
        p.pay(&c).expect("payable with the right routing");
        assert_eq!(p.total(), 0);
    }

    #[test]
    fn hybrid_and_mono_hybrid_assigned_jointly() {
        // {W/U}{2/W} from {W,U}: {W/U}→U and {2/W}→W.
        let mut p = pool(&[Color::White, Color::Blue]);
        let c = ManaCost {
            symbols: vec![
                ManaSymbol::Hybrid(Color::White, Color::Blue),
                ManaSymbol::MonoHybrid(2, Color::White),
            ],
        };
        p.pay(&c).expect("payable with the right routing");
        assert_eq!(p.total(), 0);
    }

    #[test]
    fn hybrid_choice_leaves_mana_for_generic() {
        // {1}{W/U} from {W,U}: the hybrid must not strand the generic.
        let mut p = pool(&[Color::White, Color::Blue]);
        let c = ManaCost {
            symbols: vec![ManaSymbol::Generic(1), ManaSymbol::Hybrid(Color::White, Color::Blue)],
        };
        p.pay(&c).expect("payable");
        assert_eq!(p.total(), 0);
    }

    #[test]
    fn unpayable_hybrid_still_errors() {
        let mut p = pool(&[Color::Red]);
        let c = ManaCost {
            symbols: vec![ManaSymbol::Hybrid(Color::White, Color::Blue)],
        };
        assert!(p.pay(&c).is_err());
        assert_eq!(p.total(), 1, "failed payment must not drain the pool");
    }
}
