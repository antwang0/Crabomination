//! Format rules and deck-construction validation.
//!
//! Each [`Format`] has a corresponding [`FormatRules`] that controls deck size,
//! copy limits, starting life, and other per-format rules.  Use
//! [`validate_deck`] to check that a list of card definitions is legal in a
//! given format before starting a game.

use std::collections::HashMap;

use crate::card::{CardDefinition, Supertype};
use crate::mana::{ColorSet, ManaSymbol};

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

/// A complete deck list. Used by the loader and by formats that
/// distinguish main deck from sideboard / commander zone (Phase J).
///
/// `commanders` is plural to accommodate Partner / Background — the
/// Commander format permits zero, one, or two commander cards. Other
/// formats leave it empty.
#[derive(Debug, Clone, Default)]
pub struct Deck {
    pub main: Vec<CardDefinition>,
    pub commanders: Vec<CardDefinition>,
    pub sideboard: Vec<CardDefinition>,
}

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

// ── Commander color identity (Phase K) ────────────────────────────────────

/// Compute a card's *color identity* — the union of all colored mana
/// symbols in its mana cost (CR 903.4). Hybrid pips contribute both
/// halves; Phyrexian pips contribute their colored half. Generic /
/// Colorless / Snow / X contribute nothing.
///
/// Phase K limitation: rules-text mana symbols and printed color
/// indicators are not modeled. The format doesn't track rules text
/// as parseable mana tokens, and no cards in scope rely on the
/// distinction (cards like Cao Cao that grant identity via reminder
/// text aren't in the catalog). When such a card is added, extend
/// `CardDefinition` with a `printed_color_identity: Option<ColorSet>`
/// override field that this helper unions in.
///
/// CR 903.4d: "The back face of a double-faced card is included when
/// determining a card's color identity." We recursively union the
/// back-face cost into the front-face identity so an MDFC's combined
/// identity is correct for Commander deck-validation.
pub fn color_identity(def: &CardDefinition) -> ColorSet {
    let mut out = ColorSet::empty();
    union_cost_identity(&mut out, def);
    if let Some(back) = &def.back_face {
        union_cost_identity(&mut out, back.as_ref());
    }
    out
}

fn union_cost_identity(out: &mut ColorSet, def: &CardDefinition) {
    for s in &def.cost.symbols {
        match s {
            ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => out.insert(*c),
            ManaSymbol::Hybrid(a, b) => {
                out.insert(*a);
                out.insert(*b);
            }
            ManaSymbol::Generic(_)
            | ManaSymbol::Colorless(_)
            | ManaSymbol::Snow
            | ManaSymbol::X => {}
        }
    }
}

/// Errors specific to Commander deck validation (on top of the
/// generic [`DeckError`] checks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommanderDeckError {
    /// No commander was supplied (Commander requires at least one).
    MissingCommander,
    /// More than two commanders were supplied (Partner / Background
    /// caps at two — anything beyond is illegal).
    TooManyCommanders { found: u32 },
    /// A commander card is not a legendary creature (CR 903.3a).
    /// Phase K accepts Planeswalkers that printed text grants
    /// commander-eligibility via a different path; that nuance can
    /// be added later by extending `CardDefinition` with a
    /// `can_be_commander: bool` override.
    NotLegendaryCreature { card_name: &'static str },
    /// A main-deck card's color identity is not a subset of the
    /// commander's combined color identity.
    OffColorIdentity {
        card_name: &'static str,
        card_identity: ColorSet,
        commander_identity: ColorSet,
    },
}

impl std::fmt::Display for CommanderDeckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommanderDeckError::MissingCommander => write!(f, "Commander deck requires a commander"),
            CommanderDeckError::TooManyCommanders { found } => {
                write!(f, "Too many commanders ({found}); maximum is 2 (Partner / Background)")
            }
            CommanderDeckError::NotLegendaryCreature { card_name } => {
                write!(f, "{card_name} is not a legendary creature and cannot be a commander")
            }
            CommanderDeckError::OffColorIdentity {
                card_name,
                card_identity,
                commander_identity,
            } => write!(
                f,
                "{card_name} (identity {card_identity:?}) is outside the commander's identity ({commander_identity:?})",
            ),
        }
    }
}

impl std::error::Error for CommanderDeckError {}

/// Validate a Commander-format deck. Runs the generic
/// [`validate_deck`] checks first (100-card singleton main, etc.),
/// then layers on Commander-specific rules: at least one commander,
/// at most two, each must be a legendary creature, every main-deck
/// card's color identity ⊆ commander's combined identity.
///
/// Errors from the two layers are returned as a single combined
/// `Vec` (generic deck errors wrapped, commander errors plain).
pub fn validate_commander_deck(
    deck: &Deck,
) -> Result<(), (Vec<DeckError>, Vec<CommanderDeckError>)> {
    let mut generic = Vec::new();
    if let Err(es) = validate_deck(&deck.main, Format::Commander) {
        generic = es;
    }

    let mut cmd_errors = Vec::new();
    if deck.commanders.is_empty() {
        cmd_errors.push(CommanderDeckError::MissingCommander);
    } else if deck.commanders.len() > 2 {
        cmd_errors.push(CommanderDeckError::TooManyCommanders {
            found: deck.commanders.len() as u32,
        });
    }

    // Each commander must be a legendary creature.
    for cmd in &deck.commanders {
        if !(cmd.is_legendary() && cmd.is_creature()) {
            cmd_errors.push(CommanderDeckError::NotLegendaryCreature { card_name: cmd.name });
        }
    }

    // Combined color identity is the union of every commander's.
    let mut combined = ColorSet::empty();
    for cmd in &deck.commanders {
        combined = combined.union(color_identity(cmd));
    }

    // Every main-deck card must fit inside the commander identity.
    for card in &deck.main {
        let id = color_identity(card);
        if !id.is_subset_of(combined) {
            cmd_errors.push(CommanderDeckError::OffColorIdentity {
                card_name: card.name,
                card_identity: id,
                commander_identity: combined,
            });
        }
    }

    if generic.is_empty() && cmd_errors.is_empty() {
        Ok(())
    } else {
        Err((generic, cmd_errors))
    }
}

#[cfg(test)]
#[path = "tests/format.rs"]
mod tests;
