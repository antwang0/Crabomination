use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub use crate::effect::{
    ActivatedAbility, Effect, EventKind, EventScope, EventSpec, LoyaltyAbility, OpeningHandEffect,
    Predicate, Selector, StaticAbility, StaticEffect, TriggeredAbility, Value,
};
use crate::mana::{Color, ManaCost};

/// Unique runtime ID for a card instance within a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CardId(pub u32);

/// A single card type. Cards may have multiple types (e.g. Enchantment + Creature).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardType {
    Land,
    Creature,
    Artifact,
    Enchantment,
    Planeswalker,
    Battle,
    Instant,
    Sorcery,
    Kindred,
}

/// Supertypes that modify a card's identity and rules interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Supertype {
    Basic,
    Legendary,
    Snow,
    World,
}

/// Creature subtypes (race/class).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreatureType {
    Human, Elf, Goblin, Merfolk, Zombie, Vampire, Angel, Demon, Dragon,
    Knight, Soldier, Wizard, Cleric, Rogue, Warrior, Beast, Bird,
    Elemental, Djinn, Horror, Specter, Cat, Insect, Spider, Wurm,
    Bear, Ape, Rat, Fungus, Treefolk, Giant, Ogre, Shaman, Druid,
    Monk, Archer, Berserker, Barbarian, Artificer, Pirate, Scout,
    Advisor, Assassin, Faerie, Skeleton, Spirit, Wall, Illusion,
    Hydra, Sphinx, Phoenix, Minotaur, Centaur, Cyclops, Satyr, Nymph,
    Kithkin, Viashino, Eldrazi, Sliver, Shapeshifter, Troll,
    Imp, Nightmare, Shade, Minion, Thrull, Carrier,
    Drake, Griffin, Pegasus, Unicorn, Horse, Hound, Wolf, Fox, Dog,
    Serpent, Fish, Octopus, Squid, Jellyfish, Crab, Turtle, Frog, Crocodile,
    Dinosaur, Lizard, Snake, Scorpion, Bat, Squirrel, Ox, Boar, Goat,
    Elephant, Rhino, Hippo, Mammoth, Whale, Leviathan, Kraken, Elk,
    Lion, Kavu, Lhurgoyf, Atog, Noggle, Vedalken, Kor, Ally,
    Avatar, Phyrexian, Praetor, Incarnation, Mercenary,
    Construct, Golem,
    Ooze, Plant,
    // Strixhaven-era subtypes.
    Inkling, Pest, Fractal,
    Orc, Warlock, Bard, Sorcerer, Pilot,
    // Misc. subtypes used by SOS body-only cards.
    Dwarf, Badger, Salamander, Giraffe,
    // SOS Witherbloom Dryad subtype (Essenceknit Scholar).
    Dryad,
    // Modern supplement: Kari Zev's Ragavan token.
    Monkey,
    // Strixhaven Elder Dragon legendary creatures (Lorehold, Prismari,
    // Quandrix, Silverquill, Witherbloom, the Balancer).
    Elder,
    // Lorehold Sloth subtype (Pestbrood Sloth, Startled Relic Sloth).
    Sloth,
    // Multicolor creature subtypes added with the modern_decks cube
    // expansion (Lord Xander, Korvold, etc.).
    Noble, Fae,
    // modern_decks batch 103 cube expansion (Lonis Genetics Expert,
    // Loot the Pathfinder).
    Otter, Detective,
    // Cube expansion (Collector Ouphe).
    Ouphe,
    // Enchantment creature subtype (Enduring Innocence).
    Glimmer,
    // Ninjutsu creature subtype (Fallen Shinobi, etc.).
    Ninja,
    // +1/+1-counter "Spike" cycle (Spike Feeder).
    Spike,
    // Artifact-creature token subtype (Hangarback Walker's Thopters).
    Thopter,
    // Theros devotion gods (Nylea, Thassa, Erebos, ...).
    God,
}

/// Land subtypes (basic land types + others).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LandType {
    Plains, Island, Swamp, Mountain, Forest,
    Desert, Gate, Locus, Mine, Tower, PowerPlant, Urza,
}

/// Artifact subtypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactSubtype {
    Equipment, Vehicle, Food, Treasure, Clue, Blood, Fortification, Contraption,
}

/// Enchantment subtypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnchantmentSubtype {
    Aura, Saga, Shrine, Cartouche, Curse, Room, Class, Case, Background, Role,
}

/// Spell subtypes (for instants/sorceries).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpellSubtype {
    Adventure, Lesson, Trap, Arcane,
}

/// Planeswalker subtypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaneswalkerSubtype {
    Chandra, Jace, Liliana, Ajani, Garruk, Elspeth, Gideon, Nissa, Sorin,
    Teferi, Karn, Ugin, Bolas, Ashiok, Nahiri, Vraska, Domri, Ral, Vivien,
    Tezzeret, Oko,
    // SOS Witherbloom Dellian planeswalker subtype (Professor Dellian Fel).
    Dellian,
    // Modern_decks cube expansion (Saheeli Rai, Tamiyo Collector of Tales,
    // Geyadrone Dihada, Urza Chief Artificer).
    Saheeli, Tamiyo, Dihada, Urza,
}

/// All subtype categories collected into one struct for CardDefinition.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subtypes {
    pub creature_types: Vec<CreatureType>,
    pub land_types: Vec<LandType>,
    pub artifact_subtypes: Vec<ArtifactSubtype>,
    pub enchantment_subtypes: Vec<EnchantmentSubtype>,
    pub spell_subtypes: Vec<SpellSubtype>,
    pub planeswalker_subtypes: Vec<PlaneswalkerSubtype>,
}

/// Counter types that can be placed on permanents or players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CounterType {
    PlusOnePlusOne,
    MinusOneMinusOne,
    Loyalty,
    Charge,
    Time,
    Poison,
    Lore,
    Fade,
    Age,
    Level,
    Energy,
    Experience,
    Stun,
    Verse,
    Shield,
    Wish,
    /// Page counter — Strixhaven Book artifacts (Diary of Dreams). Builds
    /// up on instant/sorcery cast and discounts the host's activated
    /// ability one for one. The counter-scaled cost reduction itself is
    /// not yet wired (see TODO.md), but counters tick up correctly.
    Page,
    /// Growth counter — used on enchantments that count tutoring or
    /// life-gain progress (Comforting Counsel, "as long as N or more
    /// growth counters …"). Distinct from `Charge` so the static-toggle
    /// variants don't collide.
    Growth,
    /// Prepared counter — SOS Prepare mechanic (Biblioplex Tomekeeper,
    /// Skycoach Waypoint). A boolean state flag on a creature; in the
    /// shipped set the flag itself has no payoff yet (Half 2 of the
    /// mechanic is still pending — see `.claude/prepared.md`). The
    /// printed "Only creatures with prepare spells can become prepared"
    /// reminder is enforced at the *target* via
    /// `SelectionRequirement::HasBackFace` on the AddCounter / Remove-
    /// Counter selectors — a creature must have a back-face spell (a
    /// "prepare spell") to be a legal target.
    Prepared,
    /// Finality counter — CR 122.1h. One or more finality counters on a
    /// permanent create a replacement effect: "If this permanent would
    /// be put into a graveyard from the battlefield, exile it instead."
    /// Implemented in `resolve_zone_change` by redirecting Battlefield
    /// → Graveyard to Battlefield → Exile when the moving card has at
    /// least one finality counter.
    Finality,
    /// Indestructible counter — CR 122.1 / 702.12. A permanent with one or
    /// more indestructible counters can't be destroyed (lethal damage and
    /// "destroy" effects don't kill it), exactly like the Indestructible
    /// keyword. Used by Zopandrel, Hunger Dominus's activated ability.
    Indestructible,
}

/// Every zone a card can occupy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Exile,
    Stack,
    Command,
}

/// The "unless [...]" payment menu for `Keyword::Ward`. Each printed
/// variant maps to a cost the spell/ability controller may pay to
/// dodge the Ward trigger.
///
/// `Mana` carries a full `ManaCost` so colored Ward (e.g. "Ward—{U}")
/// works alongside the common generic `{N}` shape. The convenience
/// constructor `WardCost::generic(n)` covers the generic case.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WardCost {
    Mana(crate::mana::ManaCost),
    Life(u32),
    Discard(u32),
    SacrificeCreature,
}

impl WardCost {
    /// `Ward {N}` — pay {N} generic mana. The common printed shape.
    pub fn generic(n: u32) -> Self {
        Self::Mana(crate::mana::ManaCost::new(vec![
            crate::mana::ManaSymbol::Generic(n),
        ]))
    }
}

/// How long a "you may play/cast that card without paying its mana cost"
/// permission stays valid after the granting effect resolves. The window
/// closes during the controller's cleanup at the named boundary; once
/// expired the `CardInstance.may_play_until` slot clears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MayPlayDuration {
    /// "...until end of turn" — clears at the end of the granting turn.
    EndOfThisTurn,
    /// "...until the end of your next turn" — clears at the end of the
    /// controller's next turn (i.e. one full untap-to-cleanup cycle later).
    EndOfControllersNextTurn,
}

/// Per-instance permission for "you may cast that card without paying its
/// mana cost" — granted by Practiced Scrollsmith, Suspend Aggression,
/// Elemental Mascot, Tablet of Discovery, Ark of Hunger, Archaic's Agony,
/// Nita (Forum Conciliator), and similar. Lives on the *card* (not the
/// player) so it survives zone changes and so the engine can drop it
/// cleanly when the card moves to a zone where casting it would no longer
/// make sense (e.g. battlefield, hand).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MayPlayPermission {
    /// Who has permission to cast. Most cards grant to the granter's
    /// controller; Suspend Aggression's "its owner may play it" instead
    /// grants to the card's owner — the caller sets this accordingly.
    pub player: usize,
    /// Turn number when the permission was granted. Used together with
    /// `duration` to compute expiry.
    pub granted_turn: u32,
    pub duration: MayPlayDuration,
    /// If `true`, the cast resolves with an instance-level
    /// "exile instead of graveyard" replacement (Nita Forum Conciliator,
    /// The Dawning Archaic). Sets `cast_via_flashback`-equivalent routing
    /// at finalize-cast time.
    #[serde(default)]
    pub exile_after: bool,
}

/// Keyword abilities supported by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Keyword {
    Flying,
    Reach,
    Menace,
    Unblockable,
    Shadow,
    Horsemanship,
    /// Landwalk (CR 702.15) — this creature can't be blocked as long as the
    /// defending player controls a land of the named type (Forestwalk,
    /// Islandwalk, Swampwalk, Mountainwalk, Plainswalk, …).
    Landwalk(LandType),
    /// Flanking (CR 702.25) — a creature without flanking that blocks this gets -1/-1 until EOT.
    Flanking,
    /// Bushido N (CR 702.45) — when this blocks or becomes blocked, it gets +N/+N until EOT.
    Bushido(u32),
    /// Rampage N (CR 702.23) — when this becomes blocked, it gets +N/+N for each blocker beyond the first.
    Rampage(u32),
    Intimidate,
    Skulk,
    /// CR 702.36 — Fear. "This creature can't be blocked except by
    /// artifact creatures and/or black creatures."
    Fear,
    Haste,
    Vigilance,
    FirstStrike,
    DoubleStrike,
    Trample,
    Exert,
    Lifelink,
    Deathtouch,
    Infect,
    Wither,
    /// Toxic N (CR 702.180) — whenever this deals combat damage to a
    /// player, that player also gets N poison counters (in addition to
    /// the normal damage).
    Toxic(u32),
    Defender,
    Protection(Color),
    Hexproof,
    Shroud,
    CantBeCountered,
    /// CR 117.x — "If X is N or more, this spell can't be countered."
    /// Banefire's printed rider. Threaded through cast time:
    /// `caster_grants_uncounterable_with_x` flips the
    /// `StackItem::Spell.uncounterable` flag when the X paid is at
    /// least the threshold.
    CantBeCounteredIfXAtLeast(u32),
    Indestructible,
    Regenerate(u32),
    Persist,
    Undying,
    Recursion,
    Flash,
    Flashback(crate::mana::ManaCost),
    /// "Flashback—Tap N untapped creatures you control" — a Flashback
    /// cost paid not in mana but by tapping N creatures. Used by Group
    /// Project (Tap three) and any future tap-creatures-as-flashback
    /// card. Recognized by `GameAction::CastFlashbackTap`. Mutually
    /// exclusive with `Keyword::Flashback(_)` in practice (a card has
    /// one Flashback variant or the other).
    FlashbackTap(u32),
    Kicker(crate::mana::ManaCost),
    Convoke,
    Delve,
    Cascade,
    Cycling(crate::mana::ManaCost),
    Echo(crate::mana::ManaCost),
    CumulativeUpkeep(crate::mana::ManaCost),
    Retrace,
    /// CR 702.139 — Escape. Cast this card from your graveyard by paying
    /// its escape mana cost plus exiling N other cards from your
    /// graveyard. `Escape(cost, n)`. Instants/sorceries resolve to the
    /// graveyard normally (re-escapable); permanents enter the
    /// battlefield as usual.
    Escape(crate::mana::ManaCost, u32),
    Phasing,
    Dredge(u32),
    Annihilator(u32),
    Banding,
    Equip(crate::mana::ManaCost),
    Fortify(crate::mana::ManaCost),
    Morph(crate::mana::ManaCost),
    Megamorph(crate::mana::ManaCost),
    Prowess,
    Ward(WardCost),
    Changeling,
    Storm,
    Inspired,
    /// CR 707 — "This spell can't be copied." Carried by the spell card
    /// (Choreographed Sparks); `Effect::CopySpell` skips a stack spell whose
    /// definition lists this keyword.
    CantBeCopied,
    /// "This creature can't block." A static restriction on the creature
    /// that holds the keyword (Postmortem Professor, Goblin Goon, etc.)
    /// or a transient grant from a pump spell (Duel Tactics, Volley
    /// Veteran). Enforced inside `declare_blockers` — any blocker
    /// declaration involving a creature with this keyword is rejected.
    CantBlock,
    /// "This creature can't attack." A static restriction on the bearer
    /// (or an Aura/effect grant — Pacifism, Faith's Fetters, Bound in
    /// Silence). Enforced from the *computed* keyword set in
    /// `declare_attackers`, so layer-granted variants are honored.
    CantAttack,
    /// CR 508.0 — "This creature can't attack unless it's the only creature
    /// attacking" (Master of Cruelties). Enforced in `declare_attackers`:
    /// a batch that declares this creature alongside any other attacker is
    /// rejected.
    AttacksAlone,
    /// CR 509.1c — "This creature must be blocked if able" (Lure-style
    /// block requirement, also Academic Dispute's rider). Enforced in
    /// `declare_blockers`: if an attacker carrying this keyword is left
    /// unblocked while the defending player controls an idle creature
    /// that could legally block it, the declaration is rejected. The
    /// engine models the single-requirement case; multi-Lure
    /// maximization (CR 509.1c's "as many as possible") is approximated.
    MustBeBlocked,
    /// "When you cast this spell from your hand, exile it as it resolves.
    /// At the beginning of your next upkeep, you may cast this card from
    /// exile without paying its mana cost." Wired in
    /// `continue_spell_resolution`: cast-from-hand spells with Rebound go
    /// to exile (instead of graveyard) and schedule a `YourNextUpkeep`
    /// delayed trigger that re-runs the spell's effect with a fresh
    /// auto-target.
    Rebound,
    /// CR 702.122 — Crew N. "Tap any number of other untapped creatures you
    /// control with total power N or greater: This Vehicle becomes an
    /// artifact creature until end of turn." Activated via
    /// `GameAction::Crew`; the value is the required total power.
    Crew(u32),
    /// CR 702.35 — Madness `cost`. Two linked abilities: (1) a static
    /// ability in the hand — "If a player would discard this card, they
    /// discard it, but exile it instead of putting it into their
    /// graveyard"; (2) a triggered ability in exile — "its owner may cast
    /// it by paying its madness cost rather than putting it into their
    /// graveyard." Wired centrally in `GameState::discard_card`: a
    /// discarded madness card is exiled (still a discard — `CardDiscarded`
    /// fires and discard-matters counters bump), then its owner is offered
    /// a yes/no cast for the madness cost from their floated pool. Declining
    /// or being unable to pay sends it on to the graveyard. The
    /// `AutoDecider` declines by default (so ordinary bot games are
    /// unaffected); tests pre-float the cost and feed `Bool(true)`.
    Madness(crate::mana::ManaCost),
}

/// Composable filter for valid targets of a spell or ability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionRequirement {
    Any,
    Player,
    Creature,
    Artifact,
    Enchantment,
    Planeswalker,
    Permanent,
    Land,
    Nonland,
    Noncreature,
    Tapped,
    Untapped,
    HasColor(Color),
    HasKeyword(Keyword),
    PowerAtMost(i32),
    ToughnessAtMost(i32),
    WithCounter(CounterType),
    ControlledByYou,
    ControlledByOpponent,
    HasSupertype(Supertype),
    HasCreatureType(CreatureType),
    HasLandType(LandType),
    HasArtifactSubtype(ArtifactSubtype),
    HasEnchantmentSubtype(EnchantmentSubtype),
    PowerAtLeast(i32),
    ToughnessAtLeast(i32),
    /// Candidate's power + toughness (layer-computed) is at most `n`. Used
    /// by Cut Down ("destroy target creature with total power and toughness
    /// 5 or less"). Battlefield-only; false for non-creatures.
    PowerPlusToughnessAtMost(i32),
    /// True when the candidate's power is strictly less than the source
    /// permanent's power (both read at evaluation time). Powers Mentor
    /// (CR 702.114 — "target attacking creature with lesser power") so the
    /// "lesser power" check re-evaluates against the source's current power
    /// instead of a hard-coded threshold. Battlefield-only; false when the
    /// source is missing or either side isn't a creature.
    PowerLessThanSource,
    /// True when the candidate creature has greater power *or* greater
    /// toughness than the source permanent (both read at evaluation
    /// time). Powers Evolve (CR 702.100 — "if that creature has greater
    /// power or toughness than this creature"). Battlefield-only; false
    /// when the source is missing or either side isn't a creature.
    GreaterPowerOrToughnessThanSource,
    IsToken,
    NotToken,
    IsBasicLand,
    IsAttacking,
    IsBlocking,
    /// CR 506.5: "A creature attacks alone if it's the only creature
    /// declared as an attacker during the declare attackers step. A
    /// creature is attacking alone if it's attacking but no other
    /// creatures are." Used by cards like Marauding Raptor's "whenever
    /// a Dinosaur enters …" rider, Battlemastery / "Whenever this
    /// creature attacks alone" payoffs, and any future single-attacker
    /// combat trick. Reads the `GameState.attacking` Vec: returns true
    /// when the card is in `attacking` AND `attacking.len() == 1`.
    IsAttackingAlone,
    /// CR 506.5: "A creature blocks alone if it's the only creature
    /// declared as a blocker during the declare blockers step. A
    /// creature is blocking alone if it's blocking but no other
    /// creatures are." Returns true when the card is in
    /// `block_map.keys()` AND `block_map.len() == 1`.
    IsBlockingAlone,
    IsSpellOnStack,
    ManaValueAtMost(u32),
    ManaValueAtLeast(u32),
    /// True when the card's mana value (CMC) is exactly `n`. Useful for
    /// effects that want a precise CMC gate (Fix What's Broken returns
    /// "with mana value equal to X", which requires this exact-match
    /// shape rather than the `AtMost`/`AtLeast` approximations).
    /// Composes naturally with `And`/`Or` for range gates.
    ManaValueExactly(u32),
    HasCardType(CardType),
    /// True when the card's printed mana cost contains at least one
    /// `{X}` symbol. Used by SOS Paradox Surveyor's reveal filter
    /// ("you may reveal a land card *or a card with {X} in its mana
    /// cost*"). Library / hand / graveyard searches consult this on
    /// the candidate cards' definitions; for `evaluate_requirement_*`
    /// targeting, the on-battlefield permanent's printed cost is read
    /// the same way.
    HasXInCost,
    /// True when the card's mana cost contains two or more *distinct*
    /// colored pips. Hybrid pips (`{W/B}`) count both halves; Phyrexian
    /// pips count their colored half only. Used by Mage Tower Referee
    /// ("Whenever you cast a multicolored spell, …") and similar
    /// "multicolored spell"/permanent payoffs.
    Multicolored,
    /// True when the card's mana cost contains *no* colored pips
    /// (generic + colorless + Snow only). Used by colorless-spell
    /// payoffs and the "colorless permanent" variant of various
    /// Eldrazi/devoid hooks.
    Colorless,
    /// True when the card has **exactly one** distinct color in its
    /// mana cost (multiple pips of the same color still count as
    /// monocolored). Used by Strixhaven's Vanishing Verse "target
    /// nonland, monocolored permanent" — the printed exact-shape
    /// targeting filter. Hybrid pips (`{W/B}`) count as both halves;
    /// Phyrexian pips count their colored half. Colorless cards fail
    /// this check (use `Colorless` instead).
    Monocolored,
    /// True when the candidate is **not** the source permanent that owns
    /// the surrounding ability. Used by printed "Other [type]" wording
    /// — Hofri Ghostforge's "Other creatures you control get +1/+0",
    /// "another creature" targeting filters for triggers, and any
    /// future "another permanent" filter. When this predicate is part
    /// of a static `applies_to` selector, `affected_from_requirement`
    /// detects it and sets `exclude_source: true` on the resulting
    /// `AffectedPermanents` variant; for selection-requirement-style
    /// targeting filters it routes through `evaluate_requirement_*`
    /// which read the source id from the resolution context.
    OtherThanSource,
    /// True when the candidate card is currently in some player's graveyard
    /// zone. Used to restrict zone-spanning trigger targets — e.g.
    /// Ascendant Dustspeaker / Lorehold Acolyte's "exile up to one target
    /// card from a graveyard", which must NOT be matchable by a
    /// battlefield permanent. The companion `evaluate_requirement_static`
    /// arm looks the candidate up in `players[*].graveyard`. Returns
    /// false for non-Permanent targets and for cards in any other zone.
    InGraveyard,
    /// True when the candidate has the greatest mana value among all
    /// permanents that match `inner` and are controlled by the same
    /// player as the candidate. Used by SOS End of the Hunt's
    /// "Target opponent exiles a creature or planeswalker they control
    /// with the greatest mana value among creatures and planeswalkers
    /// they control" — `inner` is `Creature ∨ Planeswalker` and the
    /// candidate must (a) match `inner` and (b) have an MV ≥ every
    /// other permanent matching `inner` under the same controller.
    ///
    /// Ties are broken permissively: every candidate with the maximum
    /// MV satisfies the predicate (the engine picks among them via the
    /// auto-target heuristic). Battlefield-only — the predicate
    /// returns false for entities outside the battlefield.
    HasGreatestManaValueAmongControlled(Box<SelectionRequirement>),
    /// True when the candidate's `definition.name` exactly matches.
    /// Used by Grandeur-style activations that require discarding
    /// another card with the source's printed name (Page, Loose Leaf).
    /// Evaluated against the candidate's printed name only. Stored as
    /// `String` so the predicate round-trips through serde without a
    /// `static_str_serde` adapter — the catalog passes a one-time
    /// `.to_string()` at definition time, negligible overhead.
    HasName(String),
    /// True when the candidate has a back-face `CardDefinition` —
    /// i.e. it's a double-faced card (MDFC). Used by the SOS Prepare
    /// mechanic, whose printed reminder text reads "(Only creatures
    /// with prepare spells can become prepared.)" A "prepare spell"
    /// is a back-face spell on a creature, so the legal-target rule
    /// reduces to `back_face.is_some()`. Without this predicate,
    /// Biblioplex Tomekeeper / Skycoach Waypoint could illegally
    /// prepare a vanilla creature with no back face.
    HasBackFace,
    And(Box<SelectionRequirement>, Box<SelectionRequirement>),
    Or(Box<SelectionRequirement>, Box<SelectionRequirement>),
    Not(Box<SelectionRequirement>),
}

impl SelectionRequirement {
    pub fn and(self, other: Self) -> Self {
        Self::And(Box::new(self), Box::new(other))
    }
    pub fn or(self, other: Self) -> Self {
        Self::Or(Box::new(self), Box::new(other))
    }
    pub fn negate(self) -> Self {
        Self::Not(Box::new(self))
    }
}

// ── Token definition ──────────────────────────────────────────────────────────

/// Describes a token to be created on the battlefield.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenDefinition {
    /// Token name. Stored as `String` (rather than `&'static str` like
    /// `CardDefinition.name`) so `Effect::CreateToken { definition: ... }`
    /// can round-trip through serde without a static-lifetime borrow
    /// constraint. The catalog still constructs token names from string
    /// literals; they're cloned into a `String` once at construction time.
    pub name: String,
    pub power: i32,
    pub toughness: i32,
    pub keywords: Vec<Keyword>,
    pub card_types: Vec<CardType>,
    pub colors: Vec<Color>,
    pub supertypes: Vec<Supertype>,
    pub subtypes: Subtypes,
    /// Activated abilities the token enters with. Used for Treasures
    /// (`{T}, Sac: Add one mana of any color`), Food (`{2}, {T}, Sac:
    /// Gain 3 life`), Clues (`{2}, Sac: Draw a card`), etc. Copied into
    /// the resulting `CardDefinition` by `token_to_card_definition`.
    pub activated_abilities: Vec<ActivatedAbility>,
    /// Triggered abilities the token enters with. Used for SOS Pest
    /// tokens (`Whenever this token attacks, you gain 1 life`),
    /// Strixhaven Spirit tokens (combat triggers), Inkling tokens, and
    /// any future "the token has X" rider on a `CreateToken` effect.
    /// Copied into the resulting `CardDefinition` by
    /// `token_to_card_definition`.
    #[serde(default)]
    pub triggered_abilities: Vec<TriggeredAbility>,
}

// ── Card definition ───────────────────────────────────────────────────────────

/// Static blueprint for a card; cloned into `CardInstance` at game-time.
///
/// `Default` is derived so card constructors can use
/// `..Default::default()` to skip boilerplate fields. Default values:
/// `name = ""`, `cost = ManaCost::default()` (free), all `Vec`s empty,
/// `effect = Effect::Noop`, and all `Option`s = `None`. Numeric fields
/// (`power`, `toughness`, `base_loyalty`) default to 0. New cards
/// usually only need to set `name`, `cost`, `card_types`, P/T (for
/// creatures), and the relevant ability/effect fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct CardDefinition {
    #[serde(with = "crate::static_str_serde")]
    pub name: &'static str,
    pub cost: ManaCost,
    pub supertypes: Vec<Supertype>,
    pub card_types: Vec<CardType>,
    pub subtypes: Subtypes,
    pub power: i32,
    pub toughness: i32,
    pub base_loyalty: u32,
    pub keywords: Vec<Keyword>,
    pub static_abilities: Vec<StaticAbility>,
    /// For instants/sorceries: the effect that resolves. Defaults to `Effect::Noop`
    /// for permanents whose ETB behaviour lives in `triggered_abilities`.
    pub effect: Effect,
    pub activated_abilities: Vec<ActivatedAbility>,
    pub triggered_abilities: Vec<TriggeredAbility>,
    pub loyalty_abilities: Vec<LoyaltyAbility>,
    /// Optional alternative ("pitch") cost. When `Some`, the player can cast
    /// this spell by paying the alternative cost instead of `cost` — typically
    /// some life and exiling a card from hand matching `exile_filter`.
    /// Used for Force of Will, Force of Negation, and similar.
    pub alternative_cost: Option<AlternativeCost>,
    /// Modal-double-faced-card back face. When `Some`, the player can play
    /// the card via its back face (e.g. `GameAction::PlayLandBack`); the
    /// resulting `CardInstance` adopts this definition wholesale, so all
    /// downstream abilities, types, and costs are the back face's. Only the
    /// front face stores `back_face` — the back's `back_face` is `None`.
    pub back_face: Option<Box<CardDefinition>>,
    /// Opening-hand effect ("If this card is in your opening hand…"): start
    /// in play (Leyline of Sanctity, Gemstone Caverns), reveal for a delayed
    /// effect (Chancellor of the Tangle, Chancellor of the Annex), or mark
    /// as a mulligan helper (Serum Powder). Resolved post-mulligan by
    /// `GameState::apply_opening_hand_effects`.
    pub opening_hand: Option<OpeningHandEffect>,
    /// CR 614.12 "enters with N counters" replacement. When `Some((kind,
    /// value))`, the engine drops that many counters of `kind` onto the
    /// permanent **before** the first state-based-action sweep on the new
    /// permanent. That timing lets cards with printed lines like
    /// "Pterafractyl enters with X +1/+1 counters on it" and "Symmathematics
    /// enters with two +1/+1 counters on it" survive ETB even when their
    /// base toughness is 0 — the printed Oracle was previously approximated
    /// by an `EntersBattlefield` trigger which fires *after* SBA, forcing
    /// a base-toughness bump. Resolved at ETB time in `place_card_in_dest`
    /// with the controller's `EffectContext`; the value is evaluated against
    /// the source's just-known `x_value` (so `Value::XFromCost` reads the
    /// X paid at cast time correctly).
    ///
    /// Defaults to `None` via `#[serde(default)]` so all existing literal
    /// CardDefinition initialisations pick up the new field automatically.
    #[serde(default)]
    pub enters_with_counters: Option<(CounterType, crate::effect::Value)>,
    /// CR 707 — "You may have this enter as a copy of [filter] permanent."
    /// Applied during ETB placement (in `continue_spell_resolution`), before
    /// the first state-based-action sweep, so a printed 0/0 body (Clone,
    /// Phantasmal Image) never dies as a 0/0 before the copy locks in. The
    /// engine auto-picks the highest-power matching permanent (best for the
    /// controller); if none matches, the permanent stays itself and a 0/0
    /// dies to SBA — matching the printed "you may" decline. The copy reuses
    /// `Effect::BecomeCopyOf`, so the copied card's own ETB triggers do not
    /// re-fire (printed/static characteristics only).
    ///
    /// Defaults to `None` via `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub enters_as_copy: Option<EntersAsCopy>,
    /// CR 122.4 — "This permanent can't have more than N counters of
    /// `kind` on it." When this card has more than `max` counters of
    /// `kind` on it, the state-based-action sweep removes the excess
    /// (down to exactly `max`). Used by cards that bake a counter cap
    /// into their printed text (e.g. Helix Pinnacle's "can't have more
    /// than 100 storage counters" check, custom storage permanents).
    ///
    /// Defaults to `None` via `#[serde(default)]` for snapshot back-
    /// compat. Cards without an explicit cap aren't subject to the
    /// SBA pruning step.
    #[serde(default)]
    pub max_counters_of_kind: Option<(CounterType, u32)>,
    /// CR 701.x — "Exile this spell" rider for instants and sorceries that
    /// route to exile instead of their owner's graveyard after resolution.
    /// Used by Strixhaven's "Then exile this spell" wording (Awaken the
    /// Ages MDFC back-face, Divergent Equation, Settle the Score's
    /// printed rider) and various Mystical Archive cards. When `true`,
    /// `continue_spell_resolution` places the resolved spell card into
    /// exile rather than its owner's graveyard. Has no effect on
    /// permanent spells (those go to the battlefield, not graveyard).
    ///
    /// Defaults to `false` via `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub exile_on_resolve: bool,
    /// "This spell costs {1} less to cast for each [filter] (on the
    /// battlefield)" — Affinity-class generic cost reduction whose discount
    /// scales off the caster's permanent count matching `filter`.
    ///
    /// Affects only the generic-pip side of the cost (CR 601.2f / 117.7c),
    /// clamped at the spell's printed generic total. Read by
    /// `cost_reduction_for_spell` at cast time. Powers:
    /// - **Vanquish the Horde** (`SelectionRequirement::Creature`)
    /// - **Witherbloom, the Balancer** (`SelectionRequirement::Creature
    ///   .and(ControlledByYou)`) — its second Affinity-for-creatures static
    ///   the engine's separate per-cast static still won't model, only the
    ///   self-cast discount.
    /// - Future Affinity-for-X (Artifacts, Lands, Pests) cards plug in
    ///   against this same primitive without any new engine code.
    ///
    /// Defaults to `None` via `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub affinity_filter: Option<SelectionRequirement>,
    /// "Equipped creature gets +P/+T and has [keywords]." Read by
    /// `compute_battlefield` for any Equipment whose `attached_to` points at
    /// a creature on the battlefield — the bonus is emitted as layer-7 (P/T)
    /// and layer-6 (keyword) continuous effects on the equipped creature.
    /// `None` for non-Equipment cards (and for Equipment whose only relevant
    /// effect is an activated ability, e.g. Lightning Greaves' grant-on-
    /// activate approximation). Defaults to `None` for snapshot back-compat.
    #[serde(default)]
    pub equipped_bonus: Option<EquipBonus>,
    /// CR 601.2b/601.2f — additional cost(s) paid as the spell is cast
    /// ("As an additional cost to cast this spell, …"). Paid during
    /// casting, not folded into resolution: the spell can't be cast unless
    /// every cost is payable, and a sacrifice/discard/life payment happens
    /// immediately (death/discard triggers fire before the spell resolves).
    /// Powers Necrotic Fumes, Tend the Pests, Witherbloom Sacrosanct.
    /// Defaults to empty via `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub additional_cast_cost: Vec<AdditionalCastCost>,
}

/// CR 707 — "enters as a copy of [filter] permanent" spec, stored on
/// `CardDefinition.enters_as_copy`. The copier becomes a copy of the
/// chosen permanent; `extra_creature_types` are layered on top of the
/// copied subtypes (Phantasmal Image's "it's an Illusion in addition to
/// its other types").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntersAsCopy {
    pub filter: SelectionRequirement,
    #[serde(default)]
    pub extra_creature_types: Vec<CreatureType>,
    /// Triggered abilities layered on top of the copy (Phantasmal Image's
    /// "it gains 'When this becomes the target of a spell or ability,
    /// sacrifice it'"). Appended after the copiable characteristics are
    /// stamped, so they survive the definition rewrite.
    #[serde(default)]
    pub extra_triggered: Vec<crate::effect::TriggeredAbility>,
    /// Keywords layered on top of the copy (Stunt Double's "except it has
    /// flash" — though flash matters at cast time; Sakashima-style riders).
    #[serde(default)]
    pub extra_keywords: Vec<Keyword>,
}

/// CR 601.2b/601.2f — an additional cost paid as the spell is cast, listed
/// in `CardDefinition.additional_cast_cost`. Determined and paid during
/// casting; the spell can't be cast unless every cost is payable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdditionalCastCost {
    /// "As an additional cost to cast this spell, sacrifice a [filter]."
    /// The sacrificed permanent's power becomes the spell's X (read at
    /// resolution via `Value::XFromCost`) — powers Tend the Pests.
    SacrificePermanent { filter: SelectionRequirement },
    /// "As an additional cost, discard N card(s)."
    Discard { count: u32 },
}

/// The static bonus an Equipment confers on the creature it's attached to.
/// Mirrors the printed "Equipped creature gets +P/+T and has [keywords]"
/// clause. Stored on `CardDefinition.equipped_bonus`; applied by
/// `compute_battlefield` only while the Equipment is attached to a creature
/// that's currently on the battlefield.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipBonus {
    /// Power modification granted to the equipped creature (layer 7c).
    pub power: i32,
    /// Toughness modification granted to the equipped creature (layer 7c).
    pub toughness: i32,
    /// Keywords granted to the equipped creature (layer 6).
    pub keywords: Vec<Keyword>,
}

/// Characteristic-defining dynamic P/T formula. Read by
/// `compute_battlefield` to inject a layer-7 `SetPowerToughness` for
/// the named card. Each variant encodes both the power and toughness
/// expression so the engine doesn't have to know the printed Oracle's
/// wording — just the two scalars to set.
///
/// Mapping from card name to formula lives in
/// `game::mod::dynamic_pt_for_name` — matches the lookup-table pattern
/// used by `lifegain_selfpump_for_name`, `graveyard_anthem_for_name`,
/// etc. Adding a new dynamic-P/T card is one row in that table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicPt {
    /// Power = N, toughness = N+1 where N is the count of distinct card
    /// types across every player's graveyard. Tarmogoyf, Cosmogoyf.
    DistinctTypesInAllGraveyards,
    /// Power = toughness = size of the controller's graveyard. Cruel
    /// Somnophage.
    ControllerGraveyardSize,
    /// Power = toughness = base + total land cards in all graveyards.
    /// Knight of the Reliquary (base 2/2; grows +1/+1 per land in any
    /// player's graveyard).
    BasePlusLandsInAllGraveyards { base_p: i32, base_t: i32 },
    /// Power = toughness = base + land cards in the *controller's*
    /// graveyard. Wight of the Reliquary (base 1/1, +1/+1 per land in
    /// your graveyard).
    BasePlusLandsInControllerGraveyard { base_p: i32, base_t: i32 },
}

/// An alternative (pitch) cost. Replaces the normal mana cost when the
/// player chooses to cast via this path. Models pitch (Force of Will,
/// Force of Negation) and evoke (Solitude) — the latter additionally
/// sacrifices the resulting permanent on ETB via `evoke_sacrifice`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlternativeCost {
    /// Mana paid for the alternative cast (often empty / `{0}` for pitch
    /// spells, but non-empty for evoke or kicker-style alternatives).
    pub mana_cost: ManaCost,
    /// Life paid as additional cost.
    pub life_cost: u32,
    /// If `Some`, the player must exile a card from their hand matching this
    /// filter as part of casting via the alternative.
    pub exile_filter: Option<SelectionRequirement>,
    /// True for evoke costs — the resulting permanent is sacrificed on ETB
    /// (after its ETB triggers fire).
    pub evoke_sacrifice: bool,
    /// True if this alt cost is only legal on a turn that isn't the caster's
    /// (Force of Negation, Foundation Breaker, Force of Vigor, etc.). The
    /// engine rejects the alt cast when the caster *is* the active player.
    pub not_your_turn_only: bool,
    /// Optional extra target filter applied **only** on the alt-cast path.
    /// Lets a spell expose a cheaper alt cost that's restricted to a
    /// narrower set of targets (e.g. Mystical Dispute's "{U} less if blue":
    /// regular target is any spell, alt-cost target must be a blue spell).
    /// When `Some`, `cast_spell_alternative` validates the chosen target
    /// against this filter on top of the spell's normal target filter.
    pub target_filter: Option<SelectionRequirement>,
    /// Optional cast-time game-state predicate. When `Some`, the alt cast
    /// is rejected unless `condition` evaluates to true against the
    /// caster's resolution context (no source, no target, no mode). Used
    /// by SOS Wilt in the Heat's "{2} less if one or more cards left your
    /// graveyard this turn" rider — the alt cost is gated on
    /// `Predicate::CardsLeftGraveyardThisTurnAtLeast(You, 1)`. The
    /// existing `target_filter` slot covers per-target gating; this slot
    /// covers per-game-state gating that's independent of any chosen
    /// target.
    #[serde(default)]
    pub condition: Option<crate::effect::Predicate>,
    /// Optional additional cost: exile N cards from the caster's
    /// graveyard. Mirrors `exile_filter` (which exiles **one** card from
    /// **hand**) but for "exile N cards from your graveyard" additional
    /// costs. The auto-picker takes the lowest-CMC matching cards so the
    /// caster keeps higher-value cards in their graveyard. Activation is
    /// rejected with `GameError::SelectionRequirementViolated` if fewer
    /// than N cards are in the graveyard. Currently powers SOS Soaring
    /// Stoneglider's "exile two cards from your graveyard or pay {1}{W}"
    /// alt cost.
    #[serde(default)]
    pub exile_from_graveyard_count: u32,
    /// Optional additional cost: return N permanents the caster controls
    /// matching the filter to their owner's hand. Powers "free" spells
    /// whose alt cost is a bounce-your-own-lands tempo hit — Gush
    /// ("return two Islands"), Daze ("return an Island"), Foil, etc. The
    /// auto-picker returns the lowest-impact matching permanents (untapped
    /// basics first). Rejected with `SelectionRequirementViolated` if the
    /// caster controls fewer than N matches.
    #[serde(default)]
    pub return_to_hand: Option<(SelectionRequirement, u32)>,
    /// Optional additional cost: sacrifice N permanents the caster controls
    /// matching the filter. Powers free spells whose alt cost is a sacrifice
    /// (Fireblast "sacrifice two Mountains", Snuff Out, Disappear). Rejected
    /// with `SelectionRequirementViolated` if the caster controls fewer than
    /// N matches; the auto-picker sacrifices the lowest-impact matches.
    #[serde(default)]
    pub sacrifice_permanents: Option<(SelectionRequirement, u32)>,
    /// Optional effect override when casting via the alternative cost.
    /// When `Some`, the spell uses this effect instead of its normal
    /// `definition.effect` on resolution. Powers Overload ("change each
    /// instance of 'target' to 'each'") and similar alt-cost modes that
    /// change the spell's resolution behavior.
    #[serde(default)]
    pub effect_override: Option<crate::effect::Effect>,
}

impl CardDefinition {
    pub fn is_land(&self) -> bool { self.card_types.contains(&CardType::Land) }
    pub fn is_creature(&self) -> bool { self.card_types.contains(&CardType::Creature) }
    pub fn is_instant(&self) -> bool { self.card_types.contains(&CardType::Instant) }
    pub fn is_sorcery(&self) -> bool { self.card_types.contains(&CardType::Sorcery) }
    pub fn is_artifact(&self) -> bool { self.card_types.contains(&CardType::Artifact) }
    pub fn is_enchantment(&self) -> bool { self.card_types.contains(&CardType::Enchantment) }
    pub fn is_planeswalker(&self) -> bool { self.card_types.contains(&CardType::Planeswalker) }
    pub fn is_permanent(&self) -> bool {
        self.card_types.iter().any(|t| {
            matches!(
                t,
                CardType::Land
                    | CardType::Creature
                    | CardType::Enchantment
                    | CardType::Artifact
                    | CardType::Planeswalker
                    | CardType::Battle
            )
        })
    }
    pub fn is_instant_speed(&self) -> bool {
        self.is_instant() || self.keywords.contains(&Keyword::Flash)
    }

    pub fn is_legendary(&self) -> bool { self.supertypes.contains(&Supertype::Legendary) }
    pub fn is_basic(&self) -> bool { self.supertypes.contains(&Supertype::Basic) }
    pub fn is_snow(&self) -> bool { self.supertypes.contains(&Supertype::Snow) }
    pub fn has_creature_type(&self, ct: CreatureType) -> bool {
        self.subtypes.creature_types.contains(&ct)
    }
    pub fn has_land_type(&self, lt: LandType) -> bool {
        self.subtypes.land_types.contains(&lt)
    }

    // Vehicles (CR 301.7) carry printed P/T even though they aren't
    // creatures until crewed, so their base P/T must survive into the layer
    // system — when a Crew activation animates them via a layer-4
    // AddCardType(Creature), the printed power/toughness is what the new
    // creature uses. A non-crewed Vehicle is still not a creature, so the
    // base P/T is inert for combat / "creatures you control" purposes.
    pub fn base_power(&self) -> i32 {
        if self.is_creature() || self.is_vehicle() { self.power } else { 0 }
    }
    pub fn base_toughness(&self) -> i32 {
        if self.is_creature() || self.is_vehicle() { self.toughness } else { 0 }
    }

    pub fn is_equipment(&self) -> bool {
        self.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Equipment)
    }
    pub fn is_vehicle(&self) -> bool {
        self.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Vehicle)
    }
    /// Returns the Crew cost (required total power) if this card has
    /// `Keyword::Crew(N)`.
    pub fn crew_cost(&self) -> Option<u32> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Crew(n) = kw { Some(*n) } else { None }
        })
    }
    pub fn is_aura(&self) -> bool {
        self.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Aura)
    }

    pub fn has_flashback(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Flashback(cost) = kw { Some(cost) } else { None }
        })
    }
    /// Returns the number of creatures that must be tapped to flashback
    /// this card if it has `Keyword::FlashbackTap(N)`. None otherwise.
    pub fn has_flashback_tap(&self) -> Option<u32> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::FlashbackTap(n) = kw { Some(*n) } else { None }
        })
    }
    /// True if this card has Retrace (CR 702.81) — castable from the
    /// graveyard for its mana cost plus discarding a land card.
    pub fn has_retrace(&self) -> bool {
        self.keywords.contains(&Keyword::Retrace)
    }
    /// CR 702.139 — the escape mana cost and the number of other
    /// graveyard cards that must be exiled, if this card has Escape.
    pub fn has_escape(&self) -> Option<(&ManaCost, u32)> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Escape(cost, n) = kw { Some((cost, *n)) } else { None }
        })
    }
    pub fn has_kicker(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Kicker(cost) = kw { Some(cost) } else { None }
        })
    }
    /// CR 702.35 — the Madness cost if this card has `Keyword::Madness`.
    pub fn madness_cost(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Madness(cost) = kw { Some(cost) } else { None }
        })
    }
    pub fn has_equip(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Equip(cost) = kw { Some(cost) } else { None }
        })
    }
}

/// CR 603.6e linked exile — where a card returns when the permanent that
/// exiled it (Banisher Priest, Brain Maggot, Oblivion Ring, …) leaves the
/// battlefield.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExileReturnZone {
    /// Return to the battlefield under its owner's control (Banisher
    /// Priest, Fiend Hunter, Oblivion Ring).
    Battlefield,
    /// Return to its owner's hand (Brain Maggot, Tidehollow Sculler,
    /// Kitesail Freebooter).
    Hand,
}

/// Records that a card sits in exile because of another permanent's
/// "exile until ~ leaves the battlefield" ability. When the linking
/// `source` permanent leaves play, the engine returns this card to
/// `return_to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExileLink {
    pub source: CardId,
    pub return_to: ExileReturnZone,
}

// ── Runtime card instance ─────────────────────────────────────────────────────

/// A card in play.  Tracks mutable game state layered on top of the static definition.
///
/// `Serialize`/`Deserialize` are implemented manually below — `definition`
/// is round-tripped by *card name* (via `catalog::lookup_by_name`) rather
/// than by serializing the full `CardDefinition` tree, which would force
/// the parent's `Deserialize<'de>` impl to bound `'de: 'static` because
/// of the `&'static str` name field. Manual impls let `CardInstance` be
/// `Deserialize<'de>` for any `'de`, which is a hard requirement for
/// containers like `Box<CardInstance>` inside `StackItem`.
#[derive(Debug, Clone)]
pub struct CardInstance {
    pub id: CardId,
    /// Static blueprint, shared behind an `Arc` so cloning a `CardInstance`
    /// (and therefore a whole `GameState` — the bot dry-runs every candidate
    /// action against a clone) is a refcount bump rather than a deep copy of
    /// the definition's ~two dozen `Vec` fields. The definition is immutable
    /// for the common case; the handful of effects that rewrite it
    /// (MDFC face-swap, "loses all abilities", overload effect override,
    /// keyword grants) go through `Arc::make_mut`, which clones lazily only
    /// when the `Arc` is actually shared.
    pub definition: Arc<CardDefinition>,
    pub owner: usize,
    pub controller: usize,
    pub tapped: bool,
    pub damage: u32,
    pub summoning_sick: bool,
    pub power_bonus: i32,
    pub toughness_bonus: i32,
    pub counters: HashMap<CounterType, u32>,
    pub attached_to: Option<CardId>,
    pub kicked: bool,
    pub face_down: bool,
    pub is_token: bool,
    pub used_loyalty_ability_this_turn: bool,
    /// True if this card was cast via an evoke alternative cost — it will
    /// be sacrificed on ETB after its ETB triggers fire.
    pub evoked: bool,
    /// True if this card was cast from its owner's hand on its current
    /// trip through the stack. Used by the rebound resolution path to
    /// distinguish hand-casts (rebound triggers) from re-casts from exile
    /// (rebound does **not** chain).
    pub cast_from_hand: bool,
    /// True if this card was cast from a graveyard via its Flashback
    /// cost. On resolution the resolver routes the card to exile instead
    /// of the owner's graveyard. Replaces an earlier overload of the
    /// `kicked` flag so that a card with both Kicker and Flashback can
    /// be disambiguated cleanly.
    pub cast_via_flashback: bool,
    /// "As [this] enters, choose a creature type." Cavern of Souls. The
    /// chosen type narrows which creature spells the controller can cast as
    /// uncounterable through this permanent. `None` until the ETB choice
    /// resolves — `caster_grants_uncounterable` treats `None` as
    /// "unrestricted" (legacy behaviour, used by tests that hand-craft a
    /// Cavern via `add_card_to_battlefield` without firing its ETB).
    pub chosen_creature_type: Option<CreatureType>,
    /// Indices of activated abilities flagged `once_per_turn` that have
    /// already been used this turn. Cleared at the start of each turn by
    /// `clean_per_turn_state`. Empty for the common case (most abilities
    /// don't have the flag set).
    pub once_per_turn_used: Vec<usize>,
    /// Keywords granted with `Duration::EndOfTurn` via `Effect::GrantKeyword`.
    /// Cleared at the Cleanup step alongside `power_bonus`/`toughness_bonus`.
    /// Stored separately from `definition.keywords` so the printed-Oracle
    /// keywords aren't permanently mutated by an EOT pump (engine fix —
    /// push modern_decks batch 24). `has_keyword` checks both vectors.
    pub granted_keywords_eot: Vec<Keyword>,
    /// CR 122.1b — Keyword counters. Each entry maps a keyword to its
    /// count; the host gets the keyword while one or more such counters
    /// are on it. Applied as a layer-6 keyword addition during
    /// `compute_battlefield`. Distinct from `definition.keywords`
    /// (printed) and `granted_keywords_eot` (transient EOT grants) so
    /// the printed/granted/counter sources can be inspected separately
    /// (e.g., for "remove all abilities" effects). Defaults to empty.
    /// Push (modern_decks batch 183): added per CR 122.1b.
    pub keyword_counters: std::collections::HashMap<Keyword, u32>,
    /// "You may cast/play this card without paying its mana cost" permission
    /// granted by Practiced Scrollsmith, Suspend Aggression, Nita, …
    /// Set by `Effect::GrantMayPlay`; consumed by
    /// `GameAction::CastFromZoneWithoutPaying`; cleared on expiry by
    /// `clean_per_turn_state` / next-turn cleanup, and also cleared
    /// whenever the card changes zones (the grantor's "that card" stops
    /// referring to it once it moves).
    pub may_play_until: Option<MayPlayPermission>,
    /// CR 704.5h: true if this creature has been dealt damage by a source
    /// with deathtouch since the last time SBAs were checked. Causes
    /// destruction regardless of damage amount vs toughness.
    pub dealt_deathtouch_damage: bool,
    /// CR 701.15 — Regeneration shields. Each is a one-shot replacement:
    /// "the next time this permanent would be destroyed this turn, instead
    /// remove a regeneration shield, tap it, remove it from combat, and
    /// heal all marked damage." Transient (cleared every cleanup, like
    /// `granted_keywords_eot`), so it's intentionally **not** serialized —
    /// a mid-turn snapshot reload defaults shields back to 0.
    pub regeneration_shields: u32,
    /// CR 702.83 — Exert. When this creature attacks and is exerted, it
    /// won't untap during its controller's next untap step. Set at attack
    /// time; consumed (and the untap skipped) by `do_untap`. Transient —
    /// not serialized (defaults to false on snapshot reload).
    pub skip_next_untap: bool,
    /// CR 603.6e — set on a card in exile that a permanent's "exile until
    /// ~ leaves" ability put there. When that source leaves the
    /// battlefield the engine returns this card to `ExileLink::return_to`.
    /// `None` for ordinary (permanent) exile.
    pub exiled_by: Option<ExileLink>,
    /// Until-end-of-turn flashback granted to this card while it sits in a
    /// graveyard — "target instant/sorcery card in your graveyard gains
    /// flashback until end of turn; the flashback cost equals its mana
    /// cost" (the SOS "Flashback" instant). Read by `cast_flashback` via
    /// [`effective_flashback`]; cleared at cleanup (including graveyard
    /// cards). Transient grant, so it shares `granted_keywords_eot`'s
    /// lifetime; serialized for mid-turn snapshot consistency.
    pub granted_flashback_eot: Option<crate::mana::ManaCost>,
    /// Alternative cost the controller may pay to cast this card via its
    /// `may_play_until` permission instead of casting it for free — the
    /// "miracle {N}" cost granted by Lorehold, the Historian. Read by
    /// `cast_from_zone_without_paying`; its lifetime tracks `may_play_until`
    /// (cleared together by the expiry sweep and when a cast consumes the
    /// permission). `None` for ordinary free may-play grants.
    pub granted_alt_cast_cost_eot: Option<crate::mana::ManaCost>,
}

impl CardInstance {
    pub fn new(id: CardId, definition: impl Into<Arc<CardDefinition>>, owner: usize) -> Self {
        let definition = definition.into();
        let summoning_sick = definition.is_creature();
        let base_loyalty = definition.base_loyalty;
        let is_planeswalker = definition.is_planeswalker();
        let mut counters = HashMap::new();
        if is_planeswalker && base_loyalty > 0 {
            counters.insert(CounterType::Loyalty, base_loyalty);
        }
        Self {
            id,
            definition,
            owner,
            controller: owner,
            tapped: false,
            damage: 0,
            summoning_sick,
            power_bonus: 0,
            toughness_bonus: 0,
            counters,
            attached_to: None,
            kicked: false,
            face_down: false,
            is_token: false,
            used_loyalty_ability_this_turn: false,
            evoked: false,
            cast_from_hand: false,
            cast_via_flashback: false,
            chosen_creature_type: None,
            once_per_turn_used: Vec::new(),
            granted_keywords_eot: Vec::new(),
            keyword_counters: std::collections::HashMap::new(),
            may_play_until: None,
            dealt_deathtouch_damage: false,
            regeneration_shields: 0,
            skip_next_untap: false,
            exiled_by: None,
            granted_flashback_eot: None,
            granted_alt_cast_cost_eot: None,
        }
    }

    pub fn new_token(id: CardId, definition: impl Into<Arc<CardDefinition>>, owner: usize) -> Self {
        let mut instance = Self::new(id, definition, owner);
        instance.is_token = true;
        instance
    }

    pub fn power(&self) -> i32 {
        let plus = self.counter_count(CounterType::PlusOnePlusOne) as i32;
        let minus = self.counter_count(CounterType::MinusOneMinusOne) as i32;
        self.definition.base_power() + self.power_bonus + plus - minus
    }

    pub fn toughness(&self) -> i32 {
        let plus = self.counter_count(CounterType::PlusOnePlusOne) as i32;
        let minus = self.counter_count(CounterType::MinusOneMinusOne) as i32;
        self.definition.base_toughness() + self.toughness_bonus + plus - minus
    }

    pub fn counter_count(&self, ct: CounterType) -> u32 {
        self.counters.get(&ct).copied().unwrap_or(0)
    }

    pub fn add_counters(&mut self, ct: CounterType, n: u32) {
        *self.counters.entry(ct).or_insert(0) += n;
    }

    pub fn remove_counters(&mut self, ct: CounterType, n: u32) -> u32 {
        let entry = self.counters.entry(ct).or_insert(0);
        let removed = (*entry).min(n);
        *entry -= removed;
        removed
    }

    pub fn is_dead(&self) -> bool {
        if !self.definition.is_creature() { return false; }
        if self.has_keyword(&Keyword::Indestructible) { return false; }
        self.damage as i32 >= self.toughness()
    }

    pub fn has_keyword(&self, kw: &Keyword) -> bool {
        // Printed keyword, EOT-granted, or keyword counter (CR 122.1b)
        // all qualify. The keyword-counter check requires at least one
        // counter of the matching type to be present.
        self.definition.keywords.contains(kw)
            || self.granted_keywords_eot.contains(kw)
            || self.keyword_counters.get(kw).copied().unwrap_or(0) > 0
    }

    /// True if this permanent can't be destroyed — either the
    /// Indestructible keyword (printed, granted, or via a keyword counter)
    /// or one or more `CounterType::Indestructible` counters (CR 122.1).
    pub fn is_indestructible(&self) -> bool {
        self.has_keyword(&Keyword::Indestructible)
            || self.counter_count(CounterType::Indestructible) > 0
    }

    pub fn has_protection_from(&self, color: Color) -> bool {
        self.definition.keywords.contains(&Keyword::Protection(color))
    }

    pub fn can_attack(&self) -> bool {
        self.definition.is_creature()
            && !self.tapped
            && !self.has_keyword(&Keyword::Defender)
            && !self.has_keyword(&Keyword::CantAttack)
            && (!self.summoning_sick || self.has_keyword(&Keyword::Haste))
    }

    pub fn can_block(&self) -> bool {
        self.definition.is_creature() && !self.tapped
    }

    pub fn ward_cost(&self) -> Option<u32> {
        self.definition.keywords.iter().find_map(|kw| {
            if let Keyword::Ward(WardCost::Mana(cost)) = kw {
                Some(cost.cmc())
            } else {
                None
            }
        })
    }

    pub fn clear_end_of_turn_effects(&mut self) {
        self.power_bonus = 0;
        self.toughness_bonus = 0;
        self.used_loyalty_ability_this_turn = false;
        self.once_per_turn_used.clear();
        self.granted_keywords_eot.clear();
        self.granted_flashback_eot = None;
        self.granted_alt_cast_cost_eot = None;
        self.dealt_deathtouch_damage = false;
        // CR 701.15g — unused regeneration shields expire at end of turn.
        self.regeneration_shields = 0;
    }

    /// The flashback cost this card can currently be cast with from a
    /// graveyard — its printed `Keyword::Flashback`, or an until-end-of-turn
    /// grant (the SOS "Flashback" instant). `None` if neither applies.
    pub fn effective_flashback(&self) -> Option<&ManaCost> {
        self.definition
            .has_flashback()
            .or(self.granted_flashback_eot.as_ref())
    }
}

// ── CardInstance serde: round-trip the definition by name ────────────────────
//
// Manual impls so `CardInstance: Deserialize<'de>` for *any* `'de` —
// derived `Deserialize` would inherit `'de: 'static` from the
// `CardDefinition`'s `&'static str name`. We side-step by serializing
// the card's name and re-resolving through `catalog::lookup_by_name`
// at deserialize time. Tokens whose definitions aren't in the standard
// catalog (Clue/Treasure/Food/Blood are; ad-hoc tokens are not) will
// fail to round-trip with `unknown card name: ...`.

#[derive(Serialize, Deserialize)]
struct CardInstanceWire {
    id: CardId,
    name: String,
    owner: usize,
    controller: usize,
    tapped: bool,
    damage: u32,
    summoning_sick: bool,
    power_bonus: i32,
    toughness_bonus: i32,
    counters: Vec<(CounterType, u32)>,
    attached_to: Option<CardId>,
    kicked: bool,
    face_down: bool,
    is_token: bool,
    used_loyalty_ability_this_turn: bool,
    evoked: bool,
    cast_from_hand: bool,
    /// `#[serde(default)]` so snapshots predating the field deserialize
    /// as `false` (matching the old "not cast via flashback" path).
    #[serde(default)]
    cast_via_flashback: bool,
    chosen_creature_type: Option<CreatureType>,
    #[serde(default)]
    once_per_turn_used: Vec<usize>,
    #[serde(default)]
    may_play_until: Option<MayPlayPermission>,
    /// CR 122.1b keyword counters — permanent state (never cleared at
    /// cleanup), so it must survive a snapshot round-trip just like
    /// `counters`. Stored as a `Vec` because `Keyword` can't be a JSON
    /// map key. `#[serde(default)]` so snapshots predating the field
    /// load as empty.
    #[serde(default)]
    keyword_counters: Vec<(Keyword, u32)>,
    /// Until-end-of-turn keyword grants. Cleared at cleanup, but
    /// `power_bonus`/`toughness_bonus` share that lifetime and are
    /// serialized, so a mid-turn snapshot must preserve these too for a
    /// consistent restore. `#[serde(default)]` for back-compat.
    #[serde(default)]
    granted_keywords_eot: Vec<Keyword>,
    /// Until-end-of-turn flashback grant (SOS "Flashback"). Shares the
    /// transient lifetime of `granted_keywords_eot`; serialized so a
    /// mid-turn snapshot restores it. `#[serde(default)]` for back-compat.
    #[serde(default)]
    granted_flashback_eot: Option<crate::mana::ManaCost>,
    /// Until-end-of-turn alternative cast cost (Lorehold's miracle {N}).
    /// Shares `may_play_until`'s lifetime. `#[serde(default)]` for
    /// back-compat.
    #[serde(default)]
    granted_alt_cast_cost_eot: Option<crate::mana::ManaCost>,
}

impl serde::Serialize for CardInstance {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let wire = CardInstanceWire {
            id: self.id,
            name: self.definition.name.to_string(),
            owner: self.owner,
            controller: self.controller,
            tapped: self.tapped,
            damage: self.damage,
            summoning_sick: self.summoning_sick,
            power_bonus: self.power_bonus,
            toughness_bonus: self.toughness_bonus,
            counters: self.counters.iter().map(|(k, v)| (*k, *v)).collect(),
            attached_to: self.attached_to,
            kicked: self.kicked,
            face_down: self.face_down,
            is_token: self.is_token,
            used_loyalty_ability_this_turn: self.used_loyalty_ability_this_turn,
            evoked: self.evoked,
            cast_from_hand: self.cast_from_hand,
            cast_via_flashback: self.cast_via_flashback,
            chosen_creature_type: self.chosen_creature_type,
            once_per_turn_used: self.once_per_turn_used.clone(),
            may_play_until: self.may_play_until,
            keyword_counters: self
                .keyword_counters
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
            granted_keywords_eot: self.granted_keywords_eot.clone(),
            granted_flashback_eot: self.granted_flashback_eot.clone(),
            granted_alt_cast_cost_eot: self.granted_alt_cast_cost_eot.clone(),
        };
        wire.serialize(ser)
    }
}

impl<'de> serde::Deserialize<'de> for CardInstance {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let wire = CardInstanceWire::deserialize(de)?;
        let def = crate::registry::resolve_card(&wire.name).ok_or_else(|| {
            serde::de::Error::custom(format!("unknown card name: {:?}", wire.name))
        })?;
        let mut c = CardInstance::new(wire.id, Arc::new(def), wire.owner);
        c.controller = wire.controller;
        c.tapped = wire.tapped;
        c.damage = wire.damage;
        c.summoning_sick = wire.summoning_sick;
        c.power_bonus = wire.power_bonus;
        c.toughness_bonus = wire.toughness_bonus;
        c.counters = wire.counters.into_iter().collect();
        c.attached_to = wire.attached_to;
        c.kicked = wire.kicked;
        c.face_down = wire.face_down;
        c.is_token = wire.is_token;
        c.used_loyalty_ability_this_turn = wire.used_loyalty_ability_this_turn;
        c.evoked = wire.evoked;
        c.cast_from_hand = wire.cast_from_hand;
        c.cast_via_flashback = wire.cast_via_flashback;
        c.chosen_creature_type = wire.chosen_creature_type;
        c.once_per_turn_used = wire.once_per_turn_used;
        c.may_play_until = wire.may_play_until;
        c.keyword_counters = wire.keyword_counters.into_iter().collect();
        c.granted_keywords_eot = wire.granted_keywords_eot;
        c.granted_flashback_eot = wire.granted_flashback_eot;
        c.granted_alt_cast_cost_eot = wire.granted_alt_cast_cost_eot;
        Ok(c)
    }
}
