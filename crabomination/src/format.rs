//! Format rules and deck-construction validation.
//!
//! Each [`Format`] has a corresponding [`FormatRules`] that controls deck size,
//! copy limits, starting life, and other per-format rules.  Use
//! [`validate_deck`] to check that a list of card definitions is legal in a
//! given format before starting a game.

use std::collections::HashMap;

use crate::card::{CardDefinition, Supertype};

// ── Format enum ───────────────────────────────────────────────────────────────

/// A Magic: The Gathering constructed or limited format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    // ── Constructed ───────────────────────────────────────────────────────────
    /// The most recent sets (roughly the last two years).
    Standard,
    /// Pioneer (Return to Ravnica forward).
    Pioneer,
    /// Modern (Eighth Edition / Mirrodin forward).
    Modern,
    /// Legacy (all sets, with a ban list).
    Legacy,
    /// Vintage (all sets, restricted list instead of bans).
    Vintage,
    /// Pauper (commons only, various legality pools).
    Pauper,

    // ── Multiplayer / casual ──────────────────────────────────────────────────
    /// Commander / EDH: 100-card singleton, 40 life, one legendary commander.
    Commander,
    /// Brawl: Standard-legal Commander variant, 60 cards.
    Brawl,
    /// Two-Headed Giant (team format; not enforced here, just tracked).
    TwoHeadedGiant,

    // ── Limited ───────────────────────────────────────────────────────────────
    /// Booster draft (40-card minimum deck).
    Draft,
    /// Sealed deck (40-card minimum deck).
    Sealed,

    // ── Custom ────────────────────────────────────────────────────────────────
    /// No deck-construction restrictions applied.
    Freeform,
}

// ── Format rules ──────────────────────────────────────────────────────────────

/// Rules that govern deck construction and game setup for a given format.
#[derive(Debug, Clone)]
pub struct FormatRules {
    /// Minimum number of cards in the main deck.
    pub min_deck_size: u32,
    /// Maximum number of cards in the main deck (`None` = unlimited).
    pub max_deck_size: Option<u32>,
    /// Maximum number of copies of any single non-basic-land card.
    pub max_copies: u32,
    /// Starting life total.
    pub starting_life: i32,
    /// Number of cards drawn in the opening hand.
    pub opening_hand_size: u32,
    /// Whether each player may have a sideboard and how large it can be.
    pub sideboard_size: Option<u32>,
    /// Whether the format is singleton (at most 1 copy of each non-basic card).
    pub singleton: bool,
    /// Whether a commander/companion is required.
    pub requires_commander: bool,
    /// Starting life total in multiplayer (overrides `starting_life` when > 2 players).
    pub multiplayer_starting_life: Option<i32>,
}

impl Format {
    /// Return the rules for this format.
    pub fn rules(self) -> FormatRules {
        match self {
            Format::Standard | Format::Pioneer | Format::Modern | Format::Legacy => FormatRules {
                min_deck_size: 60,
                max_deck_size: None,
                max_copies: 4,
                starting_life: 20,
                opening_hand_size: 7,
                sideboard_size: Some(15),
                singleton: false,
                requires_commander: false,
                multiplayer_starting_life: None,
            },
            Format::Vintage => FormatRules {
                min_deck_size: 60,
                max_deck_size: None,
                // Restricted cards are limited to 1; unrestricted cards allow 4.
                // Enforcing the restricted list requires a per-card lookup not
                // included here; use max_copies=4 and handle restrictions externally.
                max_copies: 4,
                starting_life: 20,
                opening_hand_size: 7,
                sideboard_size: Some(15),
                singleton: false,
                requires_commander: false,
                multiplayer_starting_life: None,
            },
            Format::Pauper => FormatRules {
                min_deck_size: 60,
                max_deck_size: None,
                max_copies: 4,
                starting_life: 20,
                opening_hand_size: 7,
                sideboard_size: Some(15),
                singleton: false,
                requires_commander: false,
                multiplayer_starting_life: None,
            },
            Format::Commander => FormatRules {
                min_deck_size: 100,
                max_deck_size: Some(100),
                max_copies: 1,
                starting_life: 40,
                opening_hand_size: 7,
                sideboard_size: None,
                singleton: true,
                requires_commander: true,
                multiplayer_starting_life: Some(40),
            },
            Format::Brawl => FormatRules {
                min_deck_size: 60,
                max_deck_size: Some(60),
                max_copies: 1,
                starting_life: 25,
                opening_hand_size: 7,
                sideboard_size: None,
                singleton: true,
                requires_commander: true,
                multiplayer_starting_life: None,
            },
            Format::TwoHeadedGiant => FormatRules {
                min_deck_size: 60,
                max_deck_size: None,
                max_copies: 4,
                // Teams share 30 life in 2HG (sometimes house-ruled to 40).
                starting_life: 30,
                opening_hand_size: 7,
                sideboard_size: Some(15),
                singleton: false,
                requires_commander: false,
                multiplayer_starting_life: None,
            },
            Format::Draft | Format::Sealed => FormatRules {
                min_deck_size: 40,
                max_deck_size: None,
                max_copies: u32::MAX,
                starting_life: 20,
                opening_hand_size: 7,
                sideboard_size: None,
                singleton: false,
                requires_commander: false,
                multiplayer_starting_life: None,
            },
            Format::Freeform => FormatRules {
                min_deck_size: 1,
                max_deck_size: None,
                max_copies: u32::MAX,
                starting_life: 20,
                opening_hand_size: 7,
                sideboard_size: None,
                singleton: false,
                requires_commander: false,
                multiplayer_starting_life: None,
            },
        }
    }
}

// ── Deck validation ───────────────────────────────────────────────────────────

/// The ways a deck can be invalid for a given format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeckError {
    /// The deck has fewer cards than the format minimum.
    TooFewCards { found: u32, minimum: u32 },
    /// The deck has more cards than the format maximum.
    TooManyCards { found: u32, maximum: u32 },
    /// A non-basic-land card appears more times than allowed.
    TooManyCopies { card_name: &'static str, found: u32, maximum: u32 },
}

impl std::fmt::Display for DeckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeckError::TooFewCards { found, minimum } => {
                write!(f, "Deck has {found} cards but needs at least {minimum}")
            }
            DeckError::TooManyCards { found, maximum } => {
                write!(f, "Deck has {found} cards but the maximum is {maximum}")
            }
            DeckError::TooManyCopies { card_name, found, maximum } => {
                write!(f, "{card_name}: {found} copies, maximum is {maximum}")
            }
        }
    }
}

impl std::error::Error for DeckError {}

/// Basic land names that are exempt from the copies-per-deck limit.
const BASIC_LANDS: &[&str] = &["Plains", "Island", "Swamp", "Mountain", "Forest",
                                "Wastes", "Snow-Covered Plains", "Snow-Covered Island",
                                "Snow-Covered Swamp", "Snow-Covered Mountain", "Snow-Covered Forest"];

fn is_basic_land(def: &CardDefinition) -> bool {
    // Prefer the supertype check; fall back to the name list for legacy definitions.
    if def.supertypes.contains(&Supertype::Basic) && def.is_land() {
        return true;
    }
    def.is_land() && BASIC_LANDS.contains(&def.name)
}

/// Validate a deck against the given format's construction rules.
///
/// Returns `Ok(())` if the deck is legal or a list of errors otherwise.
pub fn validate_deck(deck: &[CardDefinition], format: Format) -> Result<(), Vec<DeckError>> {
    let rules = format.rules();
    let mut errors = Vec::new();

    let count = deck.len() as u32;

    if count < rules.min_deck_size {
        errors.push(DeckError::TooFewCards { found: count, minimum: rules.min_deck_size });
    }
    if let Some(max) = rules.max_deck_size
        && count > max {
            errors.push(DeckError::TooManyCards { found: count, maximum: max });
        }

    // Count copies of each non-basic card.
    let mut copy_counts: HashMap<&'static str, u32> = HashMap::new();
    for card in deck {
        if !is_basic_land(card) {
            *copy_counts.entry(card.name).or_insert(0) += 1;
        }
    }

    for (name, count) in &copy_counts {
        if *count > rules.max_copies {
            errors.push(DeckError::TooManyCopies {
                card_name: name,
                found: *count,
                maximum: rules.max_copies,
            });
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

#[cfg(test)]
#[path = "tests/format.rs"]
mod tests;
