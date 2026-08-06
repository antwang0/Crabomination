//! Format rules and deck-construction validation.
//!
//! Each [`Format`] has a corresponding [`FormatRules`] that controls deck size,
//! copy limits, starting life, and other per-format rules.  Use
//! [`validate_deck`] to check that a list of card definitions is legal in a
//! given format before starting a game.

use std::collections::HashMap;

use crate::card::{CardDefinition, CompanionRule, Supertype};
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
    /// CR 904 — Archenemy: one player with a scheme deck faces a team.
    Archenemy,

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

/// Representative point-in-time ban lists, filtered to cards that could
/// plausibly appear in the catalog. Not exhaustive — extend as the catalog
/// grows.
const MODERN_BANNED: &[&str] = &[
    "Uro, Titan of Nature's Wrath", "Oko, Thief of Crowns", "Once Upon a Time",
    "Hogaak, Arisen Necropolis", "Nadu, Winged Wisdom", "Birthing Pod",
    "Blazing Shoal", "Bridge from Below", "Chrome Mox", "Dark Depths",
    "Deathrite Shaman", "Dig Through Time", "Dread Return", "Glimpse of Nature",
    "Hypergenesis", "Krark-Clan Ironworks", "Mental Misstep", "Mox Opal",
    "Mystic Sanctuary", "Treasure Cruise", "Seething Song", "Second Sunrise",
    "Sensei's Divining Top", "Simian Spirit Guide", "Skullclamp", "Summer Bloom",
    "Tibalt's Trickery", "Umezawa's Jitte", "Up the Beanstalk", "Eye of Ugin",
    "Arcum's Astrolabe", "Gitaxian Probe", "Golgari Grave-Troll",
];
const LEGACY_BANNED: &[&str] = &[
    "Black Lotus", "Ancestral Recall", "Time Walk", "Timetwister",
    "Mox Pearl", "Mox Sapphire", "Mox Jet", "Mox Ruby", "Mox Emerald",
    "Treasure Cruise", "Dig Through Time", "Deathrite Shaman",
    "Sensei's Divining Top", "Wrenn and Six", "Oko, Thief of Crowns",
    "Expressive Iteration", "Mental Misstep", "Gitaxian Probe", "Skullclamp",
    "Demonic Tutor", "Necropotence", "Frantic Search", "Windfall", "Channel",
];
const VINTAGE_RESTRICTED: &[&str] = &[
    "Black Lotus", "Ancestral Recall", "Time Walk", "Timetwister",
    "Mox Pearl", "Mox Sapphire", "Mox Jet", "Mox Ruby", "Mox Emerald",
    "Demonic Tutor", "Brainstorm", "Ponder", "Treasure Cruise",
    "Dig Through Time", "Chalice of the Void", "Channel", "Windfall",
    "Necropotence", "Sol Ring", "Time Vault", "Trinisphere",
    "Mystic Forge", "Karn, the Great Creator",
];
const COMMANDER_BANNED: &[&str] = &[
    "Black Lotus", "Ancestral Recall", "Time Walk", "Timetwister",
    "Mox Pearl", "Mox Sapphire", "Mox Jet", "Mox Ruby", "Mox Emerald",
    "Emrakul, the Aeons Torn", "Griselbrand", "Flash", "Hullbreacher",
    "Paradox Engine", "Prophet of Kruphix", "Channel", "Upheaval",
    "Biorhythm", "Limited Resources", "Sundering Titan", "Karakas",
];

impl Format {
    /// Cards banned outright in this format (representative list).
    pub fn banned_cards(self) -> &'static [&'static str] {
        match self {
            Format::Modern | Format::Pioneer | Format::Standard | Format::Pauper => MODERN_BANNED,
            Format::Legacy => LEGACY_BANNED,
            Format::Commander | Format::Brawl => COMMANDER_BANNED,
            _ => &[],
        }
    }

    /// Cards restricted to one copy in this format (Vintage).
    pub fn restricted_cards(self) -> &'static [&'static str] {
        match self {
            Format::Vintage => VINTAGE_RESTRICTED,
            _ => &[],
        }
    }

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
            // CR 904.5 — the archenemy starts at 40, everyone else at 20.
            Format::Archenemy => FormatRules {
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
    /// The card is on the format's ban list.
    BannedCard { card_name: &'static str },
    /// The card is restricted (Vintage) and appears more than once.
    RestrictedCard { card_name: &'static str, found: u32 },
    /// A card violates the chosen companion's deck-construction restriction
    /// (CR 702.139c). `card_name` is the first offending card.
    CompanionRestriction { companion: &'static str, card_name: &'static str },
    /// CR 100.4a — the sideboard is over the format's limit.
    SideboardTooLarge { found: u32, maximum: u32 },
    /// CR 100.4 — the format doesn't use sideboards at all.
    SideboardNotAllowed { found: u32 },
    /// CR 407.3 — an ante-only card in a deck that isn't playing for ante.
    AnteCardOutsideAnteGame { card_name: &'static str },
    /// Sovereign's Realm — "your starting deck can't have basic land cards".
    BasicLandsForbidden { card_name: &'static str },
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
            DeckError::BannedCard { card_name } => {
                write!(f, "{card_name} is banned in this format")
            }
            DeckError::RestrictedCard { card_name, found } => {
                write!(f, "{card_name} is restricted ({found} copies, maximum is 1)")
            }
            DeckError::CompanionRestriction { companion, card_name } => {
                write!(f, "{card_name} violates {companion}'s companion restriction")
            }
            DeckError::SideboardTooLarge { found, maximum } => {
                write!(f, "Sideboard has {found} cards but the maximum is {maximum}")
            }
            DeckError::SideboardNotAllowed { found } => {
                write!(f, "This format has no sideboard, but {found} cards were listed")
            }
            DeckError::AnteCardOutsideAnteGame { card_name } => {
                write!(f, "{card_name} may only be played in a game played for ante")
            }
            DeckError::BasicLandsForbidden { card_name } => {
                write!(f, "{card_name}: your starting deck can't have basic land cards")
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

    // CR 407.3 — "Remove this card from your deck before playing if you're not
    // playing for ante." No format here is an ante game.
    for card in deck.iter().filter(|c| c.ante_only) {
        errors.push(DeckError::AnteCardOutsideAnteGame { card_name: card.name });
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

    // Ban / restricted lists.
    let banned = format.banned_cards();
    let restricted = format.restricted_cards();
    for (name, count) in &copy_counts {
        if banned.contains(name) {
            errors.push(DeckError::BannedCard { card_name: name });
        } else if *count > 1 && restricted.contains(name) {
            errors.push(DeckError::RestrictedCard { card_name: name, found: *count });
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// CR 100.2a / 100.4a — validate a whole decklist: the main deck against
/// [`validate_deck`], the sideboard against the format's size limit, and the
/// copy limit against **main + sideboard combined** (the printed rule counts
/// both). Basic lands stay exempt.
/// Total "your minimum deck size is reduced by N" across a deck's cards
/// (Advantageous Proclamation lives in the sideboard / command zone).
pub fn min_deck_size_reduction(deck: &Deck) -> u32 {
    deck.main
        .iter()
        .chain(deck.sideboard.iter())
        .flat_map(|c| c.static_abilities.iter())
        .filter_map(|sa| match sa.effect {
            crate::effect::StaticEffect::ReduceMinimumDeckSize(n) => Some(n),
            _ => None,
        })
        .sum()
}

pub fn validate_full_deck(deck: &Deck, format: Format) -> Result<(), Vec<DeckError>> {
    let rules = format.rules();
    let mut errors = match validate_deck(&deck.main, format) {
        Ok(()) => Vec::new(),
        Err(e) => e,
    };
    // CR 100.2 — Advantageous Proclamation and friends shrink the minimum.
    // The reduction lives on cards outside the deck proper, so it can only be
    // applied after `validate_deck` has counted.
    let reduction = min_deck_size_reduction(deck);
    if reduction > 0 {
        let floor = rules.min_deck_size.saturating_sub(reduction);
        errors.retain(|e| !matches!(e, DeckError::TooFewCards { found, .. } if *found >= floor));
    }

    // Sovereign's Realm — a command-zone conspiracy can forbid basics outright.
    if deck.sideboard.iter().flat_map(|c| c.static_abilities.iter()).any(|sa| {
        matches!(sa.effect, crate::effect::StaticEffect::StartingDeckCantHaveBasicLands)
    }) {
        for card in deck.main.iter().filter(|c| is_basic_land(c)) {
            errors.push(DeckError::BasicLandsForbidden { card_name: card.name });
        }
    }

    let side = deck.sideboard.len() as u32;
    match rules.sideboard_size {
        None if side > 0 => errors.push(DeckError::SideboardNotAllowed { found: side }),
        Some(max) if side > max => {
            errors.push(DeckError::SideboardTooLarge { found: side, maximum: max })
        }
        _ => {}
    }

    // CR 100.2a — the four-of limit counts the sideboard too. `validate_deck`
    // already flagged main-only overruns, so only report names the combined
    // count pushes over that the main deck alone did not.
    let mut main_counts: HashMap<&'static str, u32> = HashMap::new();
    let mut total_counts: HashMap<&'static str, u32> = HashMap::new();
    for card in deck.main.iter().chain(deck.sideboard.iter()) {
        if is_basic_land(card) {
            continue;
        }
        *total_counts.entry(card.name).or_insert(0) += 1;
    }
    for card in &deck.main {
        if !is_basic_land(card) {
            *main_counts.entry(card.name).or_insert(0) += 1;
        }
    }
    for (name, total) in &total_counts {
        let main = main_counts.get(name).copied().unwrap_or(0);
        if *total > rules.max_copies && main <= rules.max_copies {
            errors.push(DeckError::TooManyCopies {
                card_name: name,
                found: *total,
                maximum: rules.max_copies,
            });
        }
    }

    // CR 702.139c — a sideboard companion must legalise the main deck.
    for c in deck.sideboard.iter().filter(|c| c.companion.is_some()) {
        if companion_restriction_met(c, &deck.main, rules.min_deck_size).is_err()
            && let Some(bad) = deck
                .main
                .iter()
                .find(|m| !card_meets_companion(&c.companion.clone().unwrap(), m))
        {
            errors.push(DeckError::CompanionRestriction {
                companion: c.name,
                card_name: bad.name,
            });
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

// ── Companion deck restriction (CR 702.139c) ──────────────────────────────

/// Does a single card satisfy a companion's per-card restriction? Returns
/// `false` for the *offending* card so the caller can name it. Lands are
/// exempt from the mana-value clauses (which speak of "each card" but the
/// printed restrictions only ever constrain nonland / permanent cards).
fn card_meets_companion(rule: &CompanionRule, c: &CardDefinition) -> bool {
    let mv = c.cost.cmc();
    match rule {
        CompanionRule::PermanentsManaValueAtMost(n) => !c.is_permanent() || mv <= *n,
        CompanionRule::NonlandManaValueAtLeast(n) => c.is_land() || mv >= *n,
        CompanionRule::NonlandEvenManaValue => c.is_land() || mv.is_multiple_of(2),
        CompanionRule::NonlandOddManaValue => c.is_land() || !mv.is_multiple_of(2),
        CompanionRule::NoDuplicateManaSymbols => !cost_has_duplicate_symbol(c),
        CompanionRule::Singleton => true, // handled deck-wide below
        CompanionRule::CreatureTypesAmong(types) => {
            !c.is_creature()
                || c.subtypes.creature_types.iter().any(|t| types.contains(t))
                // Changelings are every creature type (CR 702.73a).
                || c.keywords.contains(&crate::card::Keyword::Changeling)
        }
        CompanionRule::NonlandShareACardType => true, // handled deck-wide below
        CompanionRule::DeckSizeAtLeastOverMinimum(_) => true, // deck-wide
        CompanionRule::PermanentsHaveActivatedAbility => {
            !c.is_permanent() || !c.activated_abilities.is_empty()
        }
    }
}

/// True if any single mana symbol appears two or more times in the cost
/// (Jegantha). Hybrid/Phyrexian pips count by their printed symbol.
fn cost_has_duplicate_symbol(c: &CardDefinition) -> bool {
    use std::collections::HashMap;
    let mut seen: HashMap<String, u32> = HashMap::new();
    for s in &c.cost.symbols {
        // Generic/colorless/X amounts are a single symbol regardless of size.
        let key = match s {
            ManaSymbol::Colored(col) => format!("{col:?}"),
            ManaSymbol::Hybrid(a, b) => format!("H{a:?}{b:?}"),
            ManaSymbol::MonoHybrid(n, col) => format!("M{n}{col:?}"),
            ManaSymbol::Phyrexian(col) => format!("P{col:?}"),
            ManaSymbol::PhyrexianHybrid(a, b) => format!("PH{a:?}{b:?}"),
            ManaSymbol::Generic(_) => "generic".into(),
            ManaSymbol::Colorless(_) => "colorless".into(),
            ManaSymbol::Snow => "snow".into(),
            ManaSymbol::X => "x".into(),
        };
        let e = seen.entry(key).or_insert(0);
        *e += 1;
        if *e >= 2 {
            return true;
        }
    }
    false
}

/// CR 702.139c — validate a deck (main + any lands) against the companion's
/// deck-construction restriction. `min_deck_size` feeds Yorion's "≥N over the
/// minimum" clause. Returns the first offending card, if any.
pub fn companion_restriction_met(
    companion: &CardDefinition,
    deck: &[CardDefinition],
    min_deck_size: u32,
) -> Result<(), DeckError> {
    let Some(rule) = &companion.companion else { return Ok(()) };

    // Deck-wide clauses first.
    match rule {
        CompanionRule::DeckSizeAtLeastOverMinimum(extra)
            if (deck.len() as u32) < min_deck_size + extra => {
                return Err(DeckError::CompanionRestriction {
                    companion: companion.name,
                    card_name: "deck size",
                });
            }
        CompanionRule::Singleton => {
            let mut seen: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
            for c in deck {
                if is_basic_land(c) {
                    continue;
                }
                let e = seen.entry(c.name).or_insert(0);
                *e += 1;
                if *e > 1 {
                    return Err(DeckError::CompanionRestriction {
                        companion: companion.name,
                        card_name: c.name,
                    });
                }
            }
        }
        CompanionRule::NonlandShareACardType => {
            // Umori: there must exist a card type held by every nonland card.
            use crate::card::CardType;
            let candidates = [
                CardType::Creature, CardType::Artifact, CardType::Enchantment,
                CardType::Instant, CardType::Sorcery, CardType::Planeswalker,
            ];
            let nonland: Vec<&CardDefinition> = deck.iter().filter(|c| !c.is_land()).collect();
            let shared = candidates
                .iter()
                .any(|t| nonland.iter().all(|c| c.card_types.contains(t)));
            if !shared && let Some(c) = nonland.last() {
                return Err(DeckError::CompanionRestriction {
                    companion: companion.name,
                    card_name: c.name,
                });
            }
        }
        _ => {}
    }

    // Per-card clauses.
    for c in deck {
        if !card_meets_companion(rule, c) {
            return Err(DeckError::CompanionRestriction {
                companion: companion.name,
                card_name: c.name,
            });
        }
    }
    Ok(())
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
    union_face_identity(&mut out, def);
    if let Some(back) = &def.back_face {
        union_face_identity(&mut out, back.as_ref());
    }
    out
}

/// CR 903.4 — a face's identity is its mana cost, its color indicator
/// (105.2c — costless DFC back faces like werewolves), its activated-ability
/// mana costs, and the costs of its alternate halves (adventure / split).
fn union_face_identity(out: &mut ColorSet, def: &CardDefinition) {
    union_cost_identity(out, &def.cost);
    for c in &def.color_indicator {
        out.insert(*c);
    }
    for ab in &def.activated_abilities {
        union_cost_identity(out, &ab.mana_cost);
    }
    if let Some(adv) = &def.adventure {
        union_cost_identity(out, &adv.cost);
    }
    if let Some(split) = &def.split {
        union_cost_identity(out, &split.right.cost);
    }
}

fn union_cost_identity(out: &mut ColorSet, cost: &crate::mana::ManaCost) {
    for s in &cost.symbols {
        match s {
            ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => out.insert(*c),
            ManaSymbol::Hybrid(a, b) | ManaSymbol::PhyrexianHybrid(a, b) => {
                out.insert(*a);
                out.insert(*b);
            }
            ManaSymbol::MonoHybrid(_, c) => out.insert(*c),
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

