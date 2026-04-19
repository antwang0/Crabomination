use std::collections::HashMap;

use crate::mana::{Color, ManaCost};

/// Unique runtime ID for a card instance within a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardId(pub u32);

/// A single card type. Cards may have multiple types (e.g. Enchantment + Creature).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Supertype {
    Basic,
    Legendary,
    Snow,
    World,
}

/// Creature subtypes (race/class).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    Drake, Griffin, Pegasus, Unicorn, Horse, Hound, Wolf, Fox,
    Serpent, Fish, Octopus, Squid, Jellyfish, Crab, Turtle, Frog, Crocodile,
    Dinosaur, Lizard, Snake, Scorpion, Bat, Squirrel, Ox, Boar, Goat,
    Elephant, Rhino, Hippo, Mammoth, Whale, Leviathan, Kraken,
    Lion, Kavu, Lhurgoyf, Atog, Noggle, Vedalken, Kor, Ally,
}

/// Land subtypes (basic land types + others).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LandType {
    Plains, Island, Swamp, Mountain, Forest,
    Desert, Gate, Locus, Mine, Tower, PowerPlant, Urza,
}

/// Artifact subtypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactSubtype {
    Equipment, Vehicle, Food, Treasure, Clue, Blood, Fortification, Contraption,
}

/// Enchantment subtypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnchantmentSubtype {
    Aura, Saga, Shrine, Cartouche, Curse, Room, Class, Case, Background, Role,
}

/// Spell subtypes (for instants/sorceries).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpellSubtype {
    Adventure, Lesson, Trap, Arcane,
}

/// Planeswalker subtypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaneswalkerSubtype {
    Chandra, Jace, Liliana, Ajani, Garruk, Elspeth, Gideon, Nissa, Sorin,
    Teferi, Karn, Ugin, Bolas, Ashiok, Nahiri, Vraska, Domri, Ral, Vivien,
}

/// All subtype categories collected into one struct for CardDefinition.
#[derive(Debug, Clone, Default)]
pub struct Subtypes {
    pub creature_types: Vec<CreatureType>,
    pub land_types: Vec<LandType>,
    pub artifact_subtypes: Vec<ArtifactSubtype>,
    pub enchantment_subtypes: Vec<EnchantmentSubtype>,
    pub spell_subtypes: Vec<SpellSubtype>,
    pub planeswalker_subtypes: Vec<PlaneswalkerSubtype>,
}

/// Counter types that can be placed on permanents or players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CounterType {
    /// +1/+1 counter: adds 1 to a creature's power and toughness.
    PlusOnePlusOne,
    /// -1/-1 counter: subtracts 1 from a creature's power and toughness.
    MinusOneMinusOne,
    /// Loyalty counter: tracks a planeswalker's loyalty.
    Loyalty,
    /// Generic charge counter (used by many artifacts and enchantments).
    Charge,
    /// Time counter (for vanishing, suspend, etc.).
    Time,
    /// Poison counter: player loses at 10.
    Poison,
    /// Quest/lore counter (sagas, etc.).
    Lore,
    /// Fade counter: creature is sacrificed when this runs out.
    Fade,
    /// Age counter: used by cumulative upkeep.
    Age,
    /// Level counter: used by level-up creatures.
    Level,
    /// Energy counter: pooled resource used by various abilities.
    Energy,
    /// Experience counter: used by Commander variants.
    Experience,
    /// Stun counter: prevents untapping.
    Stun,
    /// Verse counter: used by enchantments that advance each upkeep.
    Verse,
    /// Shield counter: prevents damage once.
    Shield,
    /// Wish counter: stored benefit for future use.
    Wish,
}

/// Every zone a card can occupy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Exile,
    Stack,
    /// The command zone used by Commander/other formats.
    Command,
}

/// Keyword abilities supported by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Keyword {
    // ── Evasion ───────────────────────────────────────────────────────────────
    Flying,
    Reach,
    Menace,
    /// Creature can't be blocked.
    Unblockable,
    /// Can only be blocked by other Shadow creatures.
    Shadow,
    /// Like Flying but from a different rules axis; can only block/be blocked by Horsemanship.
    Horsemanship,
    /// Can only be blocked by creatures with less power or by artifact creatures.
    Intimidate,
    /// Can't be blocked by creatures with greater power.
    Skulk,

    // ── Combat modifiers ──────────────────────────────────────────────────────
    Haste,
    Vigilance,
    FirstStrike,
    DoubleStrike,
    Trample,
    /// Attacking doesn't cause this creature to tap.
    Exert,

    // ── Damage modifiers ──────────────────────────────────────────────────────
    Lifelink,
    Deathtouch,
    /// Damage dealt to creatures is dealt as -1/-1 counters; damage to players gives poison counters.
    Infect,
    /// Damage dealt to creatures is dealt as -1/-1 counters (doesn't give poison to players).
    Wither,

    // ── Defensive ─────────────────────────────────────────────────────────────
    Defender,
    /// Protection from [Color]. Prevents all DEBT (Damage, Enchanting, Blocking, Targeting).
    Protection(Color),

    // ── Targeting restrictions ─────────────────────────────────────────────
    /// Can't be targeted by opponents' spells or abilities.
    Hexproof,
    /// Can't be targeted by any spell or ability.
    Shroud,
    /// Can't be countered by spells or abilities.
    CantBeCountered,

    // ── Survival / return ─────────────────────────────────────────────────────
    Indestructible,
    /// Pay W: regenerate this creature (tap, remove damage, skip next combat).
    Regenerate(u32),
    /// When this dies without a -1/-1 counter, return it to battlefield with one.
    Persist,
    /// When this dies without a +1/+1 counter, return it to battlefield with one.
    Undying,
    /// When this creature is put into a graveyard from anywhere, return it to hand.
    Recursion,

    // ── Speed ─────────────────────────────────────────────────────────────────
    /// Can be cast at instant speed.
    Flash,

    // ── Alternative casting ────────────────────────────────────────────────────
    /// Can be cast from the graveyard for this cost; exile after.
    Flashback(crate::mana::ManaCost),
    /// Optional additional cost; sets `kicked` flag when paid.
    Kicker(crate::mana::ManaCost),
    /// Tap any number of creatures you control to help pay this spell's cost.
    Convoke,
    /// Exile cards from your graveyard to pay generic costs ({1} per card).
    Delve,
    /// When cast, exile cards from your library until you hit one with lower CMC; cast it.
    Cascade,
    /// You may discard this and pay its cycling cost instead of drawing a card.
    Cycling(crate::mana::ManaCost),
    /// Pay echo cost on your second upkeep or sacrifice it.
    Echo(crate::mana::ManaCost),
    /// Pay additional cost when it enters; otherwise sacrifice it.
    CumulativeUpkeep(crate::mana::ManaCost),
    /// Can be cast from the graveyard by discarding a land card.
    Retrace,
    /// Phases in and out of existence each of its controller's upkeeps.
    Phasing,
    /// Replace a draw with: mill N cards, return this from graveyard to hand.
    Dredge(u32),

    // ── Attack triggers ────────────────────────────────────────────────────────
    /// Whenever this attacks, defending player sacrifices N permanents.
    Annihilator(u32),
    /// Tap N creatures to redirect an unblocked attack to this creature (complex).
    Banding,

    // ── Equipment/Aura ────────────────────────────────────────────────────────
    /// Pay this cost at sorcery speed to attach this Equipment to a creature you control.
    Equip(crate::mana::ManaCost),
    /// Pay this cost at sorcery speed to fortify a land you control.
    Fortify(crate::mana::ManaCost),

    // ── Face-down ─────────────────────────────────────────────────────────────
    /// Can be cast face-down as a 2/2 for {3}; pay this to turn face-up.
    Morph(crate::mana::ManaCost),
    /// Like Morph but reveals and grants a bonus when turned face-up.
    Megamorph(crate::mana::ManaCost),

    // ── Other ────────────────────────────────────────────────────────────────
    /// Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.
    Prowess,
    /// Ward {n}: countered unless opponent pays {n} when this is targeted.
    Ward(u32),
    /// This creature has all creature types.
    Changeling,
    /// When cast, copy this spell for each other spell cast this turn.
    Storm,
    /// This creature gains an ability when it attacks (used for Exert, Inspired variants).
    Inspired,
}

/// Composable filter for valid targets of a spell or ability.
///
/// Build complex requirements via the `.and()`, `.or()`, `.negate()` builder methods:
/// ```rust,ignore
/// // Terror: non-black, non-artifact creature
/// SelectionRequirement::Creature
///     .and(SelectionRequirement::HasColor(Color::Black).negate())
///     .and(SelectionRequirement::Artifact.negate())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionRequirement {
    /// Matches any valid game object (creature, permanent, or player).
    Any,
    /// Target must be a player.
    Player,
    /// Target must be a creature permanent.
    Creature,
    /// Target must be an artifact permanent.
    Artifact,
    /// Target must be an enchantment permanent.
    Enchantment,
    /// Target must be a planeswalker permanent.
    Planeswalker,
    /// Target must be any permanent (land, creature, artifact, enchantment, or planeswalker).
    Permanent,
    /// Target must be a land permanent.
    Land,
    /// Target must be a nonland permanent.
    Nonland,
    /// Target must be a noncreature permanent.
    Noncreature,
    /// Target must be tapped.
    Tapped,
    /// Target must be untapped.
    Untapped,
    /// Target permanent must have the given color in its mana cost.
    HasColor(Color),
    /// Target permanent must have the given keyword.
    HasKeyword(Keyword),
    /// Target creature's power must be ≤ n.
    PowerAtMost(i32),
    /// Target creature's toughness must be ≤ n.
    ToughnessAtMost(i32),
    /// Target permanent or player must have the specified counter type.
    WithCounter(CounterType),
    /// Target must be controlled by the spell's controller.
    ControlledByYou,
    /// Target must be controlled by an opponent.
    ControlledByOpponent,
    /// Target permanent must have the given supertype.
    HasSupertype(Supertype),
    /// Target permanent must have the given creature type.
    HasCreatureType(CreatureType),
    /// Target permanent must have the given land type.
    HasLandType(LandType),
    /// Target permanent must have the given artifact subtype.
    HasArtifactSubtype(ArtifactSubtype),
    /// Target permanent must have the given enchantment subtype.
    HasEnchantmentSubtype(EnchantmentSubtype),
    /// Target creature's power must be ≥ n.
    PowerAtLeast(i32),
    /// Target creature's toughness must be ≥ n.
    ToughnessAtLeast(i32),
    /// Target must be a token (created rather than cast from a card).
    IsToken,
    /// Target must not be a token.
    NotToken,
    /// Target must be a basic land.
    IsBasicLand,
    /// Target must be attacking.
    IsAttacking,
    /// Target must be blocking.
    IsBlocking,
    /// Target must be a spell on the stack.
    IsSpellOnStack,
    /// Target permanent's mana value (CMC) must be ≤ n.
    ManaValueAtMost(u32),
    /// Target permanent's mana value (CMC) must be ≥ n.
    ManaValueAtLeast(u32),
    /// Target must have the given card type in its type line.
    HasCardType(CardType),
    /// Both sub-requirements must be satisfied.
    And(Box<SelectionRequirement>, Box<SelectionRequirement>),
    /// Either sub-requirement must be satisfied.
    Or(Box<SelectionRequirement>, Box<SelectionRequirement>),
    /// The sub-requirement must NOT be satisfied.
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
#[derive(Debug, Clone)]
pub struct TokenDefinition {
    pub name: &'static str,
    pub power: i32,
    pub toughness: i32,
    pub keywords: Vec<Keyword>,
    /// The card types this token has (typically `[Creature]`).
    pub card_types: Vec<CardType>,
    /// The colors of this token (affects targeting/protection).
    pub colors: Vec<Color>,
    pub supertypes: Vec<Supertype>,
    pub subtypes: Subtypes,
}

// ── Effects ───────────────────────────────────────────────────────────────────

/// Effect produced when an instant/sorcery resolves, or an ability fires.
///
/// Effects are resolved in order. If an effect requires a `target` that isn't
/// provided, `GameError::TargetRequired` is returned.
///
/// For `ChooseOne` modal effects, the caller passes the chosen mode index in
/// `GameAction::CastSpell { mode }`. If no mode is specified, mode 0 is used.
#[derive(Debug, Clone)]
pub enum SpellEffect {
    // ── Direct damage / life ─────────────────────────────────────────────────
    DealDamage {
        amount: u32,
        target: SelectionRequirement,
    },
    /// Deal damage to ALL players and/or creatures (e.g. Earthquake, Pyroclasm).
    DealDamageToAll {
        amount: u32,
        /// Which game objects are hit (e.g. `Creature`, `Player`, `Any`).
        target_filter: SelectionRequirement,
    },
    GainLife {
        amount: u32,
    },
    LoseLife {
        amount: u32,
    },

    // ── Draw / discard ────────────────────────────────────────────────────────
    DrawCards {
        amount: u32,
    },
    /// Each player draws N cards.
    EachPlayerDraws {
        amount: u32,
    },
    /// Target player discards N cards at random.
    Discard {
        amount: u32,
    },
    /// Put the top N cards of target player's library into their graveyard.
    Mill {
        amount: u32,
    },
    /// Look at the top N cards of your library; put any number back in any order,
    /// the rest on the bottom. (Simplified: just reorders without player choice.)
    Scry {
        amount: u32,
    },

    // ── Mana ──────────────────────────────────────────────────────────────────
    /// Add the given colors of mana to the controller's pool.
    AddMana {
        colors: Vec<Color>,
    },
    /// Add X mana of any single color to the controller's pool (e.g. Gilded Lotus).
    AddManaAnyColor {
        amount: u32,
    },
    /// Add X colorless mana ({C}) to the controller's pool.
    AddColorlessMana {
        amount: u32,
    },

    // ── Destruction / removal ─────────────────────────────────────────────────
    /// Destroy target permanent matching the requirement (not indestructible).
    DestroyPermanent {
        target: SelectionRequirement,
    },
    /// Destroy all permanents matching the requirement.
    DestroyAll {
        target: SelectionRequirement,
    },
    /// Exile target permanent.
    ExilePermanent {
        target: SelectionRequirement,
    },
    /// Exile all permanents matching the requirement.
    ExileAll {
        target: SelectionRequirement,
    },

    // ── Creature stat modification ────────────────────────────────────────────
    /// Target creature gets +X/+Y until end of turn.
    PumpCreature {
        power_bonus: i32,
        toughness_bonus: i32,
    },
    /// All creatures get +X/+Y until end of turn.
    PumpAllCreatures {
        power_bonus: i32,
        toughness_bonus: i32,
        /// Only affect creatures matching this filter (use `Creature` for all).
        filter: SelectionRequirement,
    },

    // ── Bounce ────────────────────────────────────────────────────────────────
    /// Return target permanent to its owner's hand.
    ReturnToHand {
        target: SelectionRequirement,
    },
    /// Return all permanents matching the filter to their owners' hands.
    ReturnAllToHand {
        target: SelectionRequirement,
    },

    // ── Counters ──────────────────────────────────────────────────────────────
    /// Add N counters of the given type to target permanent.
    AddCounters {
        count: u32,
        counter_type: CounterType,
        target: SelectionRequirement,
    },
    /// Remove N counters of the given type from target permanent.
    RemoveCounters {
        count: u32,
        counter_type: CounterType,
        target: SelectionRequirement,
    },

    // ── Tap / untap ───────────────────────────────────────────────────────────
    TapPermanent {
        target: SelectionRequirement,
    },
    TapAll {
        target: SelectionRequirement,
    },
    UntapPermanent {
        target: SelectionRequirement,
    },
    UntapAll {
        target: SelectionRequirement,
    },

    // ── Stack interaction ─────────────────────────────────────────────────────
    /// Counter target spell on the stack.
    CounterSpell {
        target: SelectionRequirement,
    },

    // ── Token creation ────────────────────────────────────────────────────────
    CreateTokens {
        count: u32,
        definition: TokenDefinition,
    },

    // ── Library manipulation ──────────────────────────────────────────────────
    /// Search your library for a card matching the filter and put it into a zone.
    SearchLibrary {
        filter: SelectionRequirement,
        put_into: Zone,
    },
    /// Return a card matching the filter from your graveyard to a zone.
    ReturnFromGraveyard {
        filter: SelectionRequirement,
        put_into: Zone,
    },

    // ── Iterated effects ──────────────────────────────────────────────────────
    /// Apply an effect for each creature on the battlefield (e.g. Overrun).
    ForEachCreature {
        effects: Vec<SpellEffect>,
    },
    /// Apply an effect for each opponent.
    ForEachOpponent {
        effects: Vec<SpellEffect>,
    },

    // ── Modal spells (choose one) ─────────────────────────────────────────────
    /// Player chooses one of the listed option lists. The chosen mode index
    /// is passed via `GameAction::CastSpell { mode }`.
    ChooseOne {
        options: Vec<Vec<SpellEffect>>,
    },

    // ── Conditional effects ───────────────────────────────────────────────────
    /// Apply `then_effects` if the condition holds, otherwise `else_effects`.
    Conditional {
        condition: EffectCondition,
        then_effects: Vec<SpellEffect>,
        else_effects: Vec<SpellEffect>,
    },

    // ── Life gain / loss (targeted) ───────────────────────────────────────────
    /// Target player gains N life.
    TargetGainsLife {
        amount: u32,
        target: SelectionRequirement,
    },
    /// Target player loses N life.
    TargetLosesLife {
        amount: u32,
        target: SelectionRequirement,
    },
    /// Each player loses N life.
    EachPlayerLosesLife {
        amount: u32,
    },
    /// Opponent loses N life, you gain N life.
    Drain {
        amount: u32,
        target: SelectionRequirement,
    },

    // ── Card selection / filtering ─────────────────────────────────────────────
    /// Look at the top N cards, put any in graveyard, rest on top (like Scry but graveyard).
    Surveil {
        amount: u32,
    },
    /// Look at the top N cards of your library; you may put one into your hand.
    LookAtTopCards {
        amount: u32,
    },

    // ── Counters (on players) ──────────────────────────────────────────────────
    /// Give target player N poison counters.
    AddPoisonCounters {
        count: u32,
        target: SelectionRequirement,
    },

    // ── Counter proliferation ──────────────────────────────────────────────────
    /// Add one counter of each type already on each permanent or player with counters.
    Proliferate,

    // ── Control changing ──────────────────────────────────────────────────────
    /// Gain control of target permanent until end of turn (give it haste).
    GainControlUntilEndOfTurn {
        target: SelectionRequirement,
    },
    /// Gain control of target permanent indefinitely.
    GainControl {
        target: SelectionRequirement,
    },

    // ── Keyword granting ─────────────────────────────────────────────────────
    /// Target creature gains a keyword until end of turn.
    GrantKeywordUntilEndOfTurn {
        keyword: Keyword,
        target: SelectionRequirement,
    },
    /// Target creature gains keywords until end of turn (multiple at once).
    GrantKeywordsUntilEndOfTurn {
        keywords: Vec<Keyword>,
        target: SelectionRequirement,
    },
    /// All creatures you control gain a keyword until end of turn.
    GrantKeywordToYourCreaturesUntilEndOfTurn {
        keyword: Keyword,
    },

    // ── Token creation (specific) ─────────────────────────────────────────────
    /// Create N Food artifact tokens ({2}, sacrifice: gain 3 life).
    CreateFood {
        count: u32,
    },
    /// Create N Treasure artifact tokens ({T}, sacrifice: add one mana of any color).
    CreateTreasure {
        count: u32,
    },
    /// Create N Blood artifact tokens ({1}, discard, sacrifice: draw a card).
    CreateBlood {
        count: u32,
    },
    /// Create N Clue artifact tokens ({2}, sacrifice: draw a card).
    Investigate {
        count: u32,
    },

    // ── Sacrifice ────────────────────────────────────────────────────────────
    /// Controller sacrifices N permanents matching the filter.
    Sacrifice {
        count: u32,
        filter: SelectionRequirement,
    },
    /// Target opponent sacrifices N permanents matching the filter.
    OpponentSacrifices {
        count: u32,
        filter: SelectionRequirement,
    },
    /// Each player sacrifices N permanents matching the filter.
    EachPlayerSacrifices {
        count: u32,
        filter: SelectionRequirement,
    },

    // ── Discard (controller's choice) ────────────────────────────────────────
    /// Controller discards N cards of their choice.
    DiscardCards {
        amount: u32,
    },
    /// Target player discards down to N cards in hand.
    DiscardToHandSize {
        hand_size: u32,
    },

    // ── Copy effects ──────────────────────────────────────────────────────────
    /// Copy the top spell on the stack (for Storm-like effects).
    CopyTopSpell,
    /// Create N copies of this spell (used by Storm resolution).
    CreateCopies {
        count: u32,
    },

    // ── Type changing ─────────────────────────────────────────────────────────
    /// Target creature loses all abilities and becomes a 1/1 until end of turn.
    ResetCreature {
        target: SelectionRequirement,
    },
    /// Target permanent becomes a land (loses other types and abilities).
    BecomeBasicLand {
        land_type: LandType,
        target: SelectionRequirement,
    },

    // ── Library manipulation (advanced) ─────────────────────────────────────
    /// Put the top N cards of target player's library into their hand.
    DrawFromTop {
        amount: u32,
        target: SelectionRequirement,
    },
    /// Shuffle your graveyard into your library.
    ShuffleGraveyardIntoLibrary,
    /// Return target card from exile to your hand.
    ReturnFromExile {
        filter: SelectionRequirement,
    },
    /// Put target permanent from graveyard onto the battlefield (Reanimate).
    ReanimateFromGraveyard {
        filter: SelectionRequirement,
        controller: ReanimateController,
    },

    // ── Legacy effects kept for compatibility ─────────────────────────────────
    /// Destroy target creature (legacy: prefer `DestroyPermanent { target: Creature }`).
    DestroyCreature {
        target: SelectionRequirement,
    },
    /// Destroy all creature permanents (legacy: use `DestroyAll { target: Creature }`).
    DestroyAllCreatures,
    /// Goblin Guide attack trigger (opponent reveals top card; if land, draws it).
    RevealOpponentTopCard,
    /// Hypnotic Specter attack trigger: opponent discards a card at random.
    OpponentDiscardRandom,
}

/// Who controls a reanimated permanent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReanimateController {
    /// The player who cast the reanimation spell controls it.
    Caster,
    /// The card's original owner controls it.
    OriginalOwner,
}

impl SpellEffect {
    /// Returns the `SelectionRequirement` for effects that target a single entity,
    /// allowing cast-time validation of targeting restrictions.
    /// Returns `None` for effects that don't use a single targeted entity.
    pub fn targeted_requirement(&self) -> Option<&SelectionRequirement> {
        match self {
            SpellEffect::DealDamage { target, .. } => Some(target),
            SpellEffect::DestroyPermanent { target } => Some(target),
            SpellEffect::ExilePermanent { target } => Some(target),
            SpellEffect::DestroyCreature { target } => Some(target),
            SpellEffect::ReturnToHand { target } => Some(target),
            SpellEffect::AddCounters { target, .. } => Some(target),
            SpellEffect::RemoveCounters { target, .. } => Some(target),
            SpellEffect::TapPermanent { target } => Some(target),
            SpellEffect::UntapPermanent { target } => Some(target),
            SpellEffect::CounterSpell { target } => Some(target),
            _ => None,
        }
    }
}

// ── Conditions for conditional effects ───────────────────────────────────────

/// A boolean condition used by `SpellEffect::Conditional`.
#[derive(Debug, Clone)]
pub enum EffectCondition {
    /// True if the controller controls a permanent matching the filter.
    ControllerControls(SelectionRequirement),
    /// True if target permanent has at least N of the specified counter.
    TargetHasCounter(CounterType, u32),
    /// True if the controller's life total is at or below the threshold.
    ControllerLifeAtMost(i32),
    /// True if the controller's graveyard has at least N cards.
    ControllerGraveyardAtLeast(usize),
    /// True if any opponent's graveyard has at least N cards.
    OpponentGraveyardAtLeast(usize),
    /// True if the controller's hand has at least N cards.
    ControllerHandAtLeast(usize),
    /// True if the controller has no cards in hand (hellbent).
    ControllerHandEmpty,
    /// True if the controller controls a creature with the given type.
    ControllerControlsCreatureType(CreatureType),
    /// True if the controller has N or more creatures in their graveyard.
    ControllerGraveyardCreaturesAtLeast(usize),
    /// True if the controller has a land in their graveyard.
    ControllerGraveyardHasLand,
    /// True if the number of lands the controller controls equals or exceeds N (threshold variant).
    ControllerLandCountAtLeast(usize),
    /// True if it's the controller's turn.
    IsControllersTurn,
    /// True if the active player cast at least N spells this turn (for Storm checks).
    SpellsCastThisTurnAtLeast(usize),
    /// True if this permanent is attacking.
    IsAttacking,
    /// True if this permanent is blocking.
    IsBlocking,
    /// True if the target permanent is a token.
    TargetIsToken,
}

// ── Triggered abilities ───────────────────────────────────────────────────────

/// When a triggered ability fires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerCondition {
    /// Fires when this permanent enters the battlefield.
    EntersBattlefield,
    /// Fires when this creature attacks.
    Attacks,
    /// Fires when this creature becomes blocked.
    BecomesBlocked,
    /// Fires when this creature or permanent dies (is put into a graveyard from
    /// the battlefield).
    Dies,
    /// Fires when another creature dies (any creature other than this one).
    AnotherCreatureDies,
    /// Fires when any creature controlled by an opponent dies.
    OpponentCreatureDies,
    /// Fires when this creature deals combat damage to a player.
    DealsCombatDamageToPlayer,
    /// Fires when this creature deals combat damage to a creature.
    DealsCombatDamageToCreature,
    /// Fires at the beginning of combat on the active player's turn.
    BeginningOfCombat,
    /// Fires at the beginning of the active player's upkeep step.
    BeginningOfUpkeep,
    /// Fires at the beginning of the active player's end step.
    BeginningOfEndStep,
    /// Fires at the beginning of the active player's draw step (before drawing).
    BeginningOfDraw,
    /// Fires whenever the controller draws a card.
    DrawCard,
    /// Fires whenever the controller discards a card.
    DiscardCard,
    /// Fires whenever the controller gains life.
    YouGainLife,
    /// Fires whenever the controller loses life.
    YouLoseLife,
    /// Fires whenever the controller casts a spell.
    /// `noncreature_only` restricts to noncreature spells (used by Prowess).
    SpellCast { noncreature_only: bool },
    /// Fires whenever any player casts a spell matching the filter.
    AnyCastSpell { noncreature_only: bool },
    /// Fires when a land is played (by this permanent's controller).
    LandPlayed,
    /// Fires when another creature enters the battlefield under your control.
    OtherCreatureEntersBattlefield,
    /// Fires when any creature enters the battlefield.
    AnyCreatureEntersBattlefield,
    /// Fires when any permanent leaves the battlefield.
    PermanentLeavesBattlefield,
    /// Fires at the beginning of each player's upkeep (not just controller's).
    EachUpkeep,
    /// Fires at the beginning of each player's end step.
    EachEndStep,
}

/// An ability that fires automatically when its condition is met.
#[derive(Debug, Clone)]
pub struct TriggeredAbility {
    pub condition: TriggerCondition,
    pub effects: Vec<SpellEffect>,
}

/// An activated ability with a tap and/or mana cost.
#[derive(Debug, Clone)]
pub struct ActivatedAbility {
    pub tap_cost: bool,
    pub mana_cost: ManaCost,
    pub effects: Vec<SpellEffect>,
    /// If true, this ability can only be activated once per turn.
    pub once_per_turn: bool,
    /// If true, this ability can only be activated at sorcery speed.
    pub sorcery_speed: bool,
}

/// A loyalty ability on a planeswalker.  Cost is the loyalty change (positive
/// to add counters, negative to remove them).  Using a loyalty ability is
/// restricted to once per turn at sorcery speed.
#[derive(Debug, Clone)]
pub struct LoyaltyAbility {
    /// Loyalty change: +2, -2, 0, -8, etc.
    pub loyalty_cost: i32,
    pub effects: Vec<SpellEffect>,
}

/// A static ability that creates a continuous effect while the source is on the battlefield.
#[derive(Debug, Clone)]
pub struct StaticAbility {
    /// A human-readable description of the ability (e.g. "Creatures you control get +1/+1").
    pub description: &'static str,
    /// The layer/sublayer and modification this ability generates.
    /// Stored as opaque data and interpreted by the layer engine.
    pub template: StaticAbilityTemplate,
}

/// Templates for common static ability patterns.
#[derive(Debug, Clone)]
pub enum StaticAbilityTemplate {
    /// All creatures you control get +N/+M (e.g. Glorious Anthem).
    PumpYourCreatures { power: i32, toughness: i32 },
    /// All creatures get +N/+M (e.g. Elesh Norn).
    PumpAllCreatures { power: i32, toughness: i32 },
    /// Opponents' creatures get -N/-M.
    WeakenOpponentCreatures { power: i32, toughness: i32 },
    /// Creatures you control of the given type get +N/+M (lord effect, e.g. Lord of Atlantis).
    CreatureTypeGetsBonus { creature_type: CreatureType, power: i32, toughness: i32 },
    /// Creatures you control of the given type get a keyword.
    CreatureTypeGetsKeyword { creature_type: CreatureType, keyword: Keyword },
    /// Creatures you control gain a keyword.
    GrantKeywordToYourCreatures(Keyword),
    /// All creatures gain a keyword.
    GrantKeywordToAllCreatures(Keyword),
    /// This permanent has an ability (e.g. equip grants Flying to the equipped creature).
    GrantKeywordToSource(Keyword),
    /// The creature attached to this Aura/Equipment gets +N/+M.
    AttachedCreatureGetsPT { power: i32, toughness: i32 },
    /// The creature attached to this Aura/Equipment gains a keyword.
    AttachedCreatureGetsKeyword(Keyword),
    /// Creatures your opponents control enter the battlefield tapped.
    OpponentsCreaturesEnterTapped,
    /// You may play one additional land on each of your turns.
    PlayAdditionalLand,
    /// Creatures your opponents control get -N/-M.
    WeakenAllOpponentCreatures { power: i32, toughness: i32 },
    /// Spells cost {N} less to cast for each [condition] (generic reduction).
    CostReduction { amount: u32 },
}

// ── Card definition ───────────────────────────────────────────────────────────

/// Static blueprint for a card; cloned into `CardInstance` at game-time.
#[derive(Debug, Clone)]
pub struct CardDefinition {
    pub name: &'static str,
    pub cost: ManaCost,
    pub supertypes: Vec<Supertype>,
    /// All types this card has (may be more than one, e.g. `[Enchantment, Creature]`).
    pub card_types: Vec<CardType>,
    pub subtypes: Subtypes,
    /// Base power (relevant for Creature types; 0 otherwise).
    pub power: i32,
    /// Base toughness (relevant for Creature types; 0 otherwise).
    pub toughness: i32,
    /// Starting loyalty (relevant for Planeswalker types; 0 otherwise).
    pub base_loyalty: u32,
    pub keywords: Vec<Keyword>,
    /// Continuous static abilities active while this permanent is on the battlefield.
    pub static_abilities: Vec<StaticAbility>,
    /// For instants/sorceries: the effects that resolve.
    pub spell_effects: Vec<SpellEffect>,
    /// For permanents: activated abilities (e.g. "{T}: Add {G}").
    pub activated_abilities: Vec<ActivatedAbility>,
    /// Triggered abilities that fire on ETB, attack, dies, etc.
    pub triggered_abilities: Vec<TriggeredAbility>,
    /// Loyalty abilities for Planeswalkers (use via ActivateLoyaltyAbility action).
    pub loyalty_abilities: Vec<LoyaltyAbility>,
}

impl CardDefinition {
    pub fn is_land(&self) -> bool {
        self.card_types.contains(&CardType::Land)
    }
    pub fn is_creature(&self) -> bool {
        self.card_types.contains(&CardType::Creature)
    }
    pub fn is_instant(&self) -> bool {
        self.card_types.contains(&CardType::Instant)
    }
    pub fn is_sorcery(&self) -> bool {
        self.card_types.contains(&CardType::Sorcery)
    }
    pub fn is_artifact(&self) -> bool {
        self.card_types.contains(&CardType::Artifact)
    }
    pub fn is_enchantment(&self) -> bool {
        self.card_types.contains(&CardType::Enchantment)
    }
    pub fn is_planeswalker(&self) -> bool {
        self.card_types.contains(&CardType::Planeswalker)
    }
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
    /// True if this card can be cast at instant speed (Instant type or Flash keyword).
    pub fn is_instant_speed(&self) -> bool {
        self.is_instant() || self.keywords.contains(&Keyword::Flash)
    }

    pub fn is_legendary(&self) -> bool {
        self.supertypes.contains(&Supertype::Legendary)
    }
    pub fn is_basic(&self) -> bool {
        self.supertypes.contains(&Supertype::Basic)
    }
    pub fn is_snow(&self) -> bool {
        self.supertypes.contains(&Supertype::Snow)
    }
    pub fn has_creature_type(&self, ct: CreatureType) -> bool {
        self.subtypes.creature_types.contains(&ct)
    }
    pub fn has_land_type(&self, lt: LandType) -> bool {
        self.subtypes.land_types.contains(&lt)
    }

    pub fn base_power(&self) -> i32 {
        if self.is_creature() { self.power } else { 0 }
    }

    pub fn base_toughness(&self) -> i32 {
        if self.is_creature() { self.toughness } else { 0 }
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

// ── Runtime card instance ─────────────────────────────────────────────────────

/// A card in play.  Tracks mutable game state layered on top of the static definition.
#[derive(Debug, Clone)]
pub struct CardInstance {
    pub id: CardId,
    pub definition: CardDefinition,
    /// Which player (index) owns this card; determines graveyard destination on death.
    pub owner: usize,
    /// Current controller (may differ from owner due to control-changing effects).
    pub controller: usize,
    pub tapped: bool,
    /// Combat damage marked on this permanent this turn.
    pub damage: u32,
    /// True while a creature has not yet been untapped under its controller's control.
    pub summoning_sick: bool,
    /// Temporary power modifier (expires at end of turn).
    pub power_bonus: i32,
    /// Temporary toughness modifier (expires at end of turn).
    pub toughness_bonus: i32,
    /// Counters placed on this permanent (+1/+1, charge, loyalty, etc.).
    pub counters: HashMap<CounterType, u32>,
    /// For Auras and Equipment: the CardId of the permanent this is attached to.
    pub attached_to: Option<CardId>,
    /// True if this spell was cast with its Kicker cost paid.
    pub kicked: bool,
    /// True if this permanent is currently face-down (Morph).
    pub face_down: bool,
    /// True if this is a token (no card representation; can't leave battlefield to hand/library).
    pub is_token: bool,
    /// True if this creature used its loyalty ability this turn (for planeswalkers).
    pub used_loyalty_ability_this_turn: bool,
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
        }
    }

    /// Create a token instance (marks is_token = true).
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

    /// Number of counters of the given type on this permanent.
    pub fn counter_count(&self, ct: CounterType) -> u32 {
        self.counters.get(&ct).copied().unwrap_or(0)
    }

    /// Add N counters of the given type.
    pub fn add_counters(&mut self, ct: CounterType, n: u32) {
        *self.counters.entry(ct).or_insert(0) += n;
    }

    /// Remove up to N counters of the given type. Returns how many were removed.
    pub fn remove_counters(&mut self, ct: CounterType, n: u32) -> u32 {
        let entry = self.counters.entry(ct).or_insert(0);
        let removed = (*entry).min(n);
        *entry -= removed;
        removed
    }

    /// True if a creature has lethal damage (damage >= toughness) and is not indestructible.
    pub fn is_dead(&self) -> bool {
        if !self.definition.is_creature() {
            return false;
        }
        if self.has_keyword(&Keyword::Indestructible) {
            return false;
        }
        self.damage as i32 >= self.toughness()
    }

    pub fn has_keyword(&self, kw: &Keyword) -> bool {
        self.definition.keywords.contains(kw)
    }

    /// True if this creature has protection from the given color.
    pub fn has_protection_from(&self, color: Color) -> bool {
        self.definition.keywords.contains(&Keyword::Protection(color))
    }

    /// Can this creature be declared as an attacker?
    pub fn can_attack(&self) -> bool {
        self.definition.is_creature()
            && !self.tapped
            && !self.has_keyword(&Keyword::Defender)
            && (!self.summoning_sick || self.has_keyword(&Keyword::Haste))
    }

    /// Can this creature be declared as a blocker?
    pub fn can_block(&self) -> bool {
        self.definition.is_creature() && !self.tapped
    }

    /// Reset temporary pump effects and per-turn flags at end of turn.
    pub fn clear_end_of_turn_effects(&mut self) {
        self.power_bonus = 0;
        self.toughness_bonus = 0;
        self.used_loyalty_ability_this_turn = false;
    }
}

#[cfg(test)]
#[path = "tests/card.rs"]
mod tests;
