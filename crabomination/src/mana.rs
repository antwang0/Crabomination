use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

/// A single symbol in a mana cost. `Generic(n)` means "n mana of any type".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManaSymbol {
    Colored(Color),
    Generic(u32),
}

/// The full mana cost of a card or ability (e.g. {3}{W}{W} for Serra Angel).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManaCost {
    pub symbols: Vec<ManaSymbol>,
}

impl ManaCost {
    pub fn new(symbols: Vec<ManaSymbol>) -> Self {
        Self { symbols }
    }

    /// Converted mana value: sum of all pip values.
    pub fn cmc(&self) -> u32 {
        self.symbols
            .iter()
            .map(|s| match s {
                ManaSymbol::Colored(_) => 1,
                ManaSymbol::Generic(n) => *n,
            })
            .sum()
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManaError {
    InsufficientColored { color: Color, needed: u32, have: u32 },
    InsufficientGeneric { needed: u32, have: u32 },
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

    pub fn amount(&self, color: Color) -> u32 {
        *self.slot(color)
    }

    pub fn total(&self) -> u32 {
        self.white + self.blue + self.black + self.red + self.green
    }

    /// Pay a ManaCost atomically. On failure the pool is left unchanged.
    ///
    /// Algorithm:
    /// 1. Pay each `Colored` pip from the matching bucket (exact color).
    /// 2. Pay all `Generic` pips from whatever remains (any color).
    pub fn pay(&mut self, cost: &ManaCost) -> Result<(), ManaError> {
        // Work on a clone so we never partially drain on failure.
        let mut tmp = self.clone();

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

        // Pass 2: generic pips (drain from any bucket)
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
                if rem == 0 {
                    break;
                }
                let drain = rem.min(tmp.amount(color));
                *tmp.slot_mut(color) -= drain;
                rem -= drain;
            }
        }

        *self = tmp;
        Ok(())
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmc_of_lightning_bolt_is_1() {
        let c = cost(&[r()]);
        assert_eq!(c.cmc(), 1);
    }

    #[test]
    fn cmc_of_serra_angel_is_5() {
        let c = cost(&[generic(3), w(), w()]);
        assert_eq!(c.cmc(), 5);
    }

    #[test]
    fn cmc_of_free_card_is_0() {
        assert_eq!(ManaCost::default().cmc(), 0);
    }

    #[test]
    fn pay_exact_colored_cost() {
        let mut pool = ManaPool::new();
        pool.add(Color::Red, 1);
        assert!(pool.pay(&cost(&[r()])).is_ok());
        assert_eq!(pool.amount(Color::Red), 0);
    }

    #[test]
    fn pay_fails_wrong_color() {
        let mut pool = ManaPool::new();
        pool.add(Color::Green, 1);
        let err = pool.pay(&cost(&[r()]));
        assert!(err.is_err());
        // Pool must be unchanged
        assert_eq!(pool.amount(Color::Green), 1);
    }

    #[test]
    fn pay_generic_with_any_mana() {
        let mut pool = ManaPool::new();
        pool.add(Color::Green, 2);
        assert!(pool.pay(&cost(&[generic(2)])).is_ok());
        assert_eq!(pool.total(), 0);
    }

    #[test]
    fn pay_mixed_cost_grizzly_bears() {
        // {1}{G} with GG in pool
        let mut pool = ManaPool::new();
        pool.add(Color::Green, 2);
        assert!(pool.pay(&cost(&[generic(1), g()])).is_ok());
        assert_eq!(pool.total(), 0);
    }

    #[test]
    fn pay_fails_not_enough_generic() {
        let mut pool = ManaPool::new();
        pool.add(Color::Red, 1);
        let err = pool.pay(&cost(&[generic(3)]));
        assert!(err.is_err());
        assert_eq!(pool.amount(Color::Red), 1); // unchanged
    }

    #[test]
    fn pay_dark_ritual_cost() {
        // Pay {B}, then manually add BBB to pool
        let mut pool = ManaPool::new();
        pool.add(Color::Black, 1);
        pool.pay(&cost(&[b()])).unwrap();
        pool.add(Color::Black, 3); // Dark Ritual's effect
        assert_eq!(pool.amount(Color::Black), 3);
    }

    #[test]
    fn empty_drains_all_mana() {
        let mut pool = ManaPool::new();
        pool.add(Color::Red, 2);
        pool.add(Color::Green, 3);
        pool.empty();
        assert_eq!(pool.total(), 0);
    }

    #[test]
    fn pay_does_not_partially_drain_on_failure() {
        // {W}{U} cost, pool has WW but no U — colored check must fail atomically
        let mut pool = ManaPool::new();
        pool.add(Color::White, 2);
        let err = pool.pay(&cost(&[w(), u()]));
        assert!(err.is_err());
        assert_eq!(pool.amount(Color::White), 2); // still intact
    }
}
