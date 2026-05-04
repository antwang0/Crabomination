use std::collections::HashMap;

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
    Construct, Golem, Thopter,
    Ooze, Plant,
    // Strixhaven-era subtypes.
    Inkling, Pest, Fractal,
    Orc, Warlock, Bard, Sorcerer, Pilot,
    // Misc. subtypes used by SOS body-only cards.
    Dwarf, Badger, Salamander, Giraffe,
    // SOS Witherbloom Dryad subtype (Essenceknit Scholar).
    Dryad,
    // Strixhaven Elder Dragon legendary creatures (Lorehold, Prismari,
    // Quandrix, Silverquill, Witherbloom, the Balancer).
    Elder,
    // Lorehold Sloth subtype (Pestbrood Sloth, Startled Relic Sloth).
    Sloth,
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
    Tezzeret,
    // SOS Witherbloom Dellian planeswalker subtype (Professor Dellian Fel).
    Dellian,
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

/// Keyword abilities supported by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Keyword {
    Flying,
    Reach,
    Menace,
    Unblockable,
    Shadow,
    Horsemanship,
    Intimidate,
    Skulk,
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
    Defender,
    Protection(Color),
    Hexproof,
    Shroud,
    CantBeCountered,
    Indestructible,
    Regenerate(u32),
    Persist,
    Undying,
    Recursion,
    Flash,
    Flashback(crate::mana::ManaCost),
    Kicker(crate::mana::ManaCost),
    Convoke,
    Delve,
    Cascade,
    Cycling(crate::mana::ManaCost),
    Echo(crate::mana::ManaCost),
    CumulativeUpkeep(crate::mana::ManaCost),
    Retrace,
    Phasing,
    Dredge(u32),
    Annihilator(u32),
    Banding,
    Equip(crate::mana::ManaCost),
    Fortify(crate::mana::ManaCost),
    Morph(crate::mana::ManaCost),
    Megamorph(crate::mana::ManaCost),
    Prowess,
    Ward(u32),
    Changeling,
    Storm,
    Inspired,
    /// "This creature can't block." A static restriction on the creature
    /// that holds the keyword (Postmortem Professor, Goblin Goon, etc.)
    /// or a transient grant from a pump spell (Duel Tactics, Volley
    /// Veteran). Enforced inside `declare_blockers` — any blocker
    /// declaration involving a creature with this keyword is rejected.
    CantBlock,
    /// "When you cast this spell from your hand, exile it as it resolves.
    /// At the beginning of your next upkeep, you may cast this card from
    /// exile without paying its mana cost." Wired in
    /// `continue_spell_resolution`: cast-from-hand spells with Rebound go
    /// to exile (instead of graveyard) and schedule a `YourNextUpkeep`
    /// delayed trigger that re-runs the spell's effect with a fresh
    /// auto-target.
    Rebound,
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
    IsToken,
    NotToken,
    IsBasicLand,
    IsAttacking,
    IsBlocking,
    IsSpellOnStack,
    ManaValueAtMost(u32),
    ManaValueAtLeast(u32),
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
    /// True when the card's mana cost contains exactly *one* distinct
    /// colored pip (hybrid pips count both halves toward the count;
    /// Phyrexian counts the colored side; generic/colorless/Snow/X
    /// pips don't count). Used by STX Vanishing Verse ("exile target
    /// nonland, monocolored permanent") and similar mono-color-matters
    /// hooks. Strictly stronger than `!Colorless && !Multicolored`
    /// when both halves are evaluated together.
    Monocolored,
    /// True when the card's printed name matches this exact string.
    /// Used by name-matters payoffs that look at a specific card name —
    /// Dragon's Approach ("if you have four or more cards named
    /// Dragon's Approach in your graveyard"), Persistent Petitioners,
    /// Rat Colony, Slime Against Humanity, etc. Stored as
    /// `Cow<'static, str>` so card-side construction (`HasName(name)`
    /// on a `&'static str`) works without allocating, and snapshot
    /// restore (which builds owned strings from JSON) avoids leaking.
    HasName(std::borrow::Cow<'static, str>),
    /// SOS Prepare mechanic: matches creatures whose printed back face is
    /// a prepare spell (an instant or sorcery sitting in `back_face`).
    /// Used by Biblioplex Tomekeeper / Skycoach Waypoint to gate their
    /// "becomes prepared / unprepared" toggles to the legal target set —
    /// i.e. the printed reminder text "Only creatures with prepare
    /// spells can become prepared." Evaluated via
    /// `crate::card::is_prepare_spell` on the candidate's
    /// `CardDefinition`.
    HasPrepareSpell,
    /// SOS Prepare mechanic: matches permanents currently flagged
    /// `prepared` (see `CardInstance.prepared`). Battlefield-only —
    /// non-permanent zones return false. Reserved for prepare-payoff
    /// cards that condition on the flag (none in the catalog yet, but
    /// the targeting/predicate primitive is wired so future "Prepare
    /// {cost}: …" abilities can gate on it).
    IsPrepared,
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
    /// "As an additional cost to cast this spell, sacrifice a [filter]."
    /// When `Some(filter)`, casting this spell requires the controller to
    /// sacrifice exactly one of their permanents matching `filter` *as
    /// part of the cast*, in addition to mana. The cast is illegal if the
    /// controller has no matching permanent. The pre-cast sacrifice
    /// happens before the spell goes on the stack — so the sacrificed
    /// permanent is in the graveyard before the spell resolves and any
    /// ETB triggers see the post-sacrifice board.
    ///
    /// Used by Strixhaven's "as an additional cost: sacrifice a creature"
    /// design (Daemogoth Woe-Eater, Eyeblight Cullers, Tend the Pests'
    /// pre-cast sacrifice clause), and any future cast-time-sacrifice
    /// alternative-cost shape.
    ///
    /// Distinct from a regular `Effect::Sacrifice` rider on an ETB
    /// trigger: the cast-time path makes the spell *illegal* without a
    /// matching permanent, while ETB-sacrifice silently no-ops (the
    /// spell still resolves). It also fires *before* the spell goes on
    /// the stack, so the sacrificed permanent's death triggers run
    /// before the cast spell does.
    #[serde(default)]
    pub additional_sac_cost: Option<crate::card::SelectionRequirement>,
    /// "As an additional cost to cast this spell, discard N cards."
    /// When `Some(n)`, casting this spell requires the controller to
    /// discard exactly `n` cards from hand *as part of the cast*, in
    /// addition to mana. The cast is illegal if the controller has fewer
    /// than `n` cards in hand (excluding the spell card itself, which is
    /// already in the stack-pending state at this point). The discard
    /// happens before the spell goes on the stack — so madness / discard-
    /// trigger interactions resolve before the cast spell does.
    ///
    /// Used by Strixhaven's "as an additional cost: discard a card"
    /// design (Thrilling Discovery's discard-1 rider, Cathartic Reunion's
    /// discard-2 rider) and the broader rummager / madness family.
    ///
    /// Distinct from a regular `Effect::Discard` rider in resolution: the
    /// cast-time path makes the spell *illegal* without sufficient cards
    /// in hand, while in-resolution discard silently no-ops or partially
    /// resolves when hand is short. Sister field to `additional_sac_cost`.
    #[serde(default)]
    pub additional_discard_cost: Option<u32>,
    /// "As an additional cost to cast this spell, pay X life." When
    /// `Some(value)`, casting this spell deducts the evaluated `Value`
    /// life from the controller as part of the cast, in addition to
    /// mana. The deduction happens before the spell goes on the stack
    /// — so it's reflected in mana-spent introspection effects, and
    /// "whenever you lose life" triggers fire pre-resolution.
    ///
    /// Used by Strixhaven's "as an additional cost: pay X life" design
    /// (Vicious Rivalry's `pay X life`). Sister field to
    /// `additional_sac_cost` (push XXXIX) and `additional_discard_cost`
    /// (push XLIII). Distinguished from `Effect::LoseLife` in resolution
    /// because the cast-time deduction lands before the spell goes on
    /// the stack, not after.
    ///
    /// Note: unlike `additional_sac_cost`, this *does not* make the
    /// spell illegal at zero life — paying life to negative life
    /// triggers loss-of-game on the next SBA pass per CR 119.4
    /// (a rule the engine already enforces during resolve_stack).
    #[serde(default)]
    pub additional_life_cost: Option<crate::effect::Value>,
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
    /// "This permanent enters with N {counter} counters on it" replacement.
    /// When `Some((kind, value))`, the engine adds `value`-many `kind`
    /// counters at the moment the card enters the battlefield — *before*
    /// any ETB trigger fires and before SBAs run. The `Value` is evaluated
    /// against the cast-time `EffectContext` (the spell's `x_value`,
    /// `converged_value`, and `targets[]` are in scope), so X-cost
    /// permanents like Pterafractyl ({X}{G}{U}, "enters with X +1/+1
    /// counters") and converge-scaled bodies (Body of Research, Rancorous
    /// Archaic) read the actual paid X / color count.
    ///
    /// Distinct from an ETB trigger that adds counters via
    /// `Effect::AddCounter`: ETB triggers fire *after* the permanent is on
    /// the battlefield, so a 1/0 body (Pterafractyl) would die to the
    /// 0-toughness SBA before its trigger gets a chance to resolve. The
    /// replacement form wires the counters in atomically with the bf
    /// entry, surviving the post-entry SBA pass.
    ///
    /// Only honored on the spell-resolution path in `stack.rs`; tokens
    /// and `Move → Battlefield` paths skip this hook (tokens have no X
    /// or paid mana to reference, and reanimate-style returns shouldn't
    /// re-add the original counter count).
    ///
    /// Implements **CR 122.6 / 122.6a**: "Some spells and abilities
    /// refer to counters being put on an object. This refers to putting
    /// counters on that object while it's on the battlefield and also
    /// to an object that's given counters as it enters the battlefield.
    /// / If an object enters the battlefield with counters on it, the
    /// effect causing the object to be given counters may specify which
    /// player puts those counters on it. If the effect doesn't specify
    /// a player, the object's controller puts those counters on it." The
    /// field doesn't expose a player parameter (today no card needs
    /// the "your opponent puts X counters" variant), so the controller
    /// (`caster` in `stack.rs`) is the implicit placer per the
    /// unspecified-player default. Also referenced by **CR 707.5**:
    /// copy effects honor "enters with" replacements on copied
    /// objects.
    #[serde(default)]
    pub enters_with_counters: Option<(CounterType, crate::effect::Value)>,
}

/// An alternative (pitch) cost. Replaces the normal mana cost when the
/// player chooses to cast via this path. Models pitch (Force of Will,
/// Force of Negation) and evoke (Solitude) — the latter additionally
/// sacrifices the resulting permanent on ETB via `evoke_sacrifice`.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// When `Some(idx)`, casting via this alt cost auto-selects mode
    /// `idx` of a modal spell, overriding any caller-supplied mode.
    /// Used by spells whose alternative cost *is* the mode selector —
    /// e.g. Devastating Mastery's mastery alt cost ({7}{W}{W}) implies
    /// mode 1 (Wrath + reanimate), while the regular cast ({4}{W}{W})
    /// resolves mode 0 (Wrath only). Defaults to None for back-compat.
    #[serde(default)]
    pub mode_on_alt: Option<usize>,
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

    pub fn base_power(&self) -> i32 { if self.is_creature() { self.power } else { 0 } }
    pub fn base_toughness(&self) -> i32 { if self.is_creature() { self.toughness } else { 0 } }

    /// SOS Prepare mechanic: true when this definition is a creature whose
    /// back face is a prepare spell (an instant or sorcery glued to a
    /// vanilla creature front). Used by `SelectionRequirement::
    /// HasPrepareSpell` to gate prepare/unprepare toggles to the legal
    /// target set — matches the reminder text "Only creatures with
    /// prepare spells can become prepared."
    pub fn has_prepare_spell(&self) -> bool {
        is_prepare_spell(self)
    }

    pub fn is_equipment(&self) -> bool {
        self.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Equipment)
    }
    pub fn is_aura(&self) -> bool {
        self.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Aura)
    }

    pub fn has_flashback(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Flashback(cost) = kw { Some(cost) } else { None }
        })
    }
    pub fn has_kicker(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Kicker(cost) = kw { Some(cost) } else { None }
        })
    }
    pub fn has_equip(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Equip(cost) = kw { Some(cost) } else { None }
        })
    }
}

/// SOS Prepare mechanic: detect a "prepared card" — a creature whose
/// printed back face is a prepare spell (an instant or sorcery glued to
/// the creature front). Mirrors the convention in
/// `crabomination/src/catalog/sets/sos/mdfcs.rs` where
/// `vanilla_front(...)` always pairs a creature front with an
/// instant/sorcery `spell_back(...)`. Returns false for non-creature
/// fronts and for back faces that are themselves creatures or lands
/// (Plargg/Augusta-style flips, Sundering Eruption-style sorcery↔land
/// MDFCs).
pub fn is_prepare_spell(def: &CardDefinition) -> bool {
    if !def.is_creature() {
        return false;
    }
    let Some(back) = def.back_face.as_ref() else {
        return false;
    };
    back.card_types
        .iter()
        .any(|t| matches!(t, CardType::Instant | CardType::Sorcery))
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
    pub definition: CardDefinition,
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
    /// SOS Prepare mechanic: per-permanent boolean toggled by
    /// "becomes prepared" / "becomes unprepared" effects. Sticky across
    /// turns; cleared only when the permanent leaves the battlefield
    /// (handled implicitly because `CardInstance::new` defaults to
    /// `false` on every fresh instantiation).
    pub prepared: bool,
}

impl CardInstance {
    pub fn new(id: CardId, definition: CardDefinition, owner: usize) -> Self {
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
            chosen_creature_type: None,
            once_per_turn_used: Vec::new(),
            prepared: false,
        }
    }

    pub fn new_token(id: CardId, definition: CardDefinition, owner: usize) -> Self {
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
        self.definition.keywords.contains(kw)
    }

    pub fn has_protection_from(&self, color: Color) -> bool {
        self.definition.keywords.contains(&Keyword::Protection(color))
    }

    pub fn can_attack(&self) -> bool {
        self.definition.is_creature()
            && !self.tapped
            && !self.has_keyword(&Keyword::Defender)
            && (!self.summoning_sick || self.has_keyword(&Keyword::Haste))
    }

    pub fn can_block(&self) -> bool {
        self.definition.is_creature() && !self.tapped
    }

    pub fn clear_end_of_turn_effects(&mut self) {
        self.power_bonus = 0;
        self.toughness_bonus = 0;
        self.used_loyalty_ability_this_turn = false;
        self.once_per_turn_used.clear();
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
    chosen_creature_type: Option<CreatureType>,
    #[serde(default)]
    once_per_turn_used: Vec<usize>,
    #[serde(default)]
    prepared: bool,
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
            chosen_creature_type: self.chosen_creature_type,
            once_per_turn_used: self.once_per_turn_used.clone(),
            prepared: self.prepared,
        };
        wire.serialize(ser)
    }
}

impl<'de> serde::Deserialize<'de> for CardInstance {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let wire = CardInstanceWire::deserialize(de)?;
        let def = crate::catalog::lookup_by_name(&wire.name).ok_or_else(|| {
            serde::de::Error::custom(format!("unknown card name: {:?}", wire.name))
        })?;
        let mut c = CardInstance::new(wire.id, def, wire.owner);
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
        c.chosen_creature_type = wire.chosen_creature_type;
        c.once_per_turn_used = wire.once_per_turn_used;
        c.prepared = wire.prepared;
        Ok(c)
    }
}

#[cfg(test)]
#[path = "tests/card.rs"]
mod tests;
