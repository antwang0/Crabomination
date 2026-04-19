use std::collections::HashMap;

pub use crate::effect::{
    ActivatedAbility, Effect, EventKind, EventScope, EventSpec, LoyaltyAbility, Predicate,
    Selector, StaticAbility, StaticEffect, TriggeredAbility, Value,
};
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
    Command,
}

/// Keyword abilities supported by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
}

/// Composable filter for valid targets of a spell or ability.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenDefinition {
    pub name: &'static str,
    pub power: i32,
    pub toughness: i32,
    pub keywords: Vec<Keyword>,
    pub card_types: Vec<CardType>,
    pub colors: Vec<Color>,
    pub supertypes: Vec<Supertype>,
    pub subtypes: Subtypes,
}

// TokenDefinition's Subtypes needs PartialEq/Eq — derive it too.
impl PartialEq for Subtypes {
    fn eq(&self, other: &Self) -> bool {
        self.creature_types == other.creature_types
            && self.land_types == other.land_types
            && self.artifact_subtypes == other.artifact_subtypes
            && self.enchantment_subtypes == other.enchantment_subtypes
            && self.spell_subtypes == other.spell_subtypes
            && self.planeswalker_subtypes == other.planeswalker_subtypes
    }
}
impl Eq for Subtypes {}

// ── Card definition ───────────────────────────────────────────────────────────

/// Static blueprint for a card; cloned into `CardInstance` at game-time.
#[derive(Debug, Clone)]
pub struct CardDefinition {
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
    }
}

#[cfg(test)]
#[path = "tests/card.rs"]
mod tests;
