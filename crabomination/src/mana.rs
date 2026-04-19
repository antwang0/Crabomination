use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

/// A single symbol in a mana cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// {S}: pay with mana from a snow source.
    Snow,
    /// {X}: variable cost determined at cast time.
    X,
}

/// The full mana cost of a card or ability (e.g. {3}{W}{W} for Serra Angel).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
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
                ManaSymbol::Snow => 1,
                ManaSymbol::X => 0, // X is 0 everywhere except on the stack
            })
            .sum()
    }

    /// True if this cost contains any X symbols.
    pub fn has_x(&self) -> bool {
        self.symbols.iter().any(|s| matches!(s, ManaSymbol::X))
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
}

/// Available mana in a player's pool during their turn.
#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

        // Pass 3: hybrid pips
        for sym in &cost.symbols {
            if let ManaSymbol::Hybrid(a, b) = sym {
                if tmp.amount(*a) > 0 {
                    *tmp.slot_mut(*a) -= 1;
                } else if tmp.amount(*b) > 0 {
                    *tmp.slot_mut(*b) -= 1;
                } else {
                    return Err(ManaError::CannotPayHybrid {
                        color_a: *a,
                        color_b: *b,
                    });
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
            for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
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
            for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
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

        *self = tmp;
        Ok(side_effects)
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
