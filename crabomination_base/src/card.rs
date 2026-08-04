use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub use crate::effect::{
    ActivatedAbility, Effect, EventKind, EventScope, EventSpec, LoyaltyAbility, OpeningHandEffect,
    Predicate, Selector, StateTriggeredAbility, StaticAbility, StaticEffect, TriggeredAbility,
    Value,
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
    /// CR 314 — a Scheme. Never a permanent and never castable: it lives in
    /// the command zone, face down in the scheme deck or face up after being
    /// set in motion (CR 904.9).
    Scheme,
    /// CR 313 — a Vanguard avatar. Never a permanent: it starts in the command
    /// zone and its abilities function from there (CR 902.5).
    Vanguard,
    /// CR 315 — a Conspiracy. Never a permanent and never castable: it starts
    /// in the command zone and stays there, its static and triggered
    /// abilities functioning from there while it is face up (CR 315.5).
    Conspiracy,
    /// CR 311 — a Plane. Never a permanent and never castable: it lives in the
    /// command zone, in the planar deck or face up as the current plane
    /// (CR 901.4). Its abilities function from there (CR 901.7).
    Plane,
    /// CR 312 — a Phenomenon. Like a Plane, but its "when you encounter this"
    /// trigger leaving the stack makes its controller planeswalk (CR 704.6f).
    Phenomenon,
}

/// Supertypes that modify a card's identity and rules interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Supertype {
    Basic,
    Legendary,
    Snow,
    World,
    /// CR 904.11 — an Ongoing scheme stays face up in the command zone until
    /// something abandons it, rather than being swept by the CR 904.10 SBA.
    Ongoing,
}

/// Creature subtypes (race/class).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreatureType {
    Human, Elf, Goblin, Merfolk, Moonfolk, Zombie, Vampire, Angel, Demon, Dragon,
    Knight, Soldier, Wizard, Cleric, Rogue, Warrior, Beast, Bird, Soltari, Dauthi,
    Elemental, Djinn, Efreet, Horror, Specter, Cat, Insect, Spider, Wurm, Worm, Alien,
    Armadillo, Nautilus,
    Bear, Ape, Rat, Fungus, Snail, Treefolk, Giant, Ogre, Orgg, Shaman, Druid,
    Monk, Archer, Berserker, Barbarian, Artificer, Pirate, Scout, Mongoose, Clown, Dalek, Nomad,
    Mystic,
    Doctor,
    Advisor, Assassin, Faerie, Skeleton, Spirit, Wall, Illusion,
    Hydra, Sphinx, Phoenix, Minotaur, Centaur, Cyclops, Satyr, Nymph, Demigod,
    Kithkin, Viashino, Eldrazi, Sliver, Shapeshifter, Troll,
    Imp, Nightmare, Shade, Minion, Thrull, Carrier, Devil, Wraith, Lamia, Nightstalker,
    Drake, Griffin, Hippogriff, Pegasus, Unicorn, Horse, Hound, Wolf, Werewolf, Fox, Dog,
    Jackal, Hyena,
    Serpent, Fish, Octopus, Squid, Jellyfish, Starfish, Crab, Turtle, Frog, Crocodile, Homarid,
    Dinosaur, Lizard, Snake, Scorpion, Bat, Squirrel, Ox, Boar, Goat, Llama, Shark, Harpy, Porcupine,
    Sheep, Trilobite, Beaver, Beeble, Sponge,
    Basilisk, Cockatrice,
    Elephant, Rhino, Hippo, Mammoth, Whale, Leviathan, Kraken, Elk, Egg, Weasel,
    Lion, Kavu, Lhurgoyf, Atog, Noggle, Vedalken, Kor, Ally, Kobold, Surrakar,
    Avatar, Phyrexian, Praetor, Incarnation, Mercenary, Rebel, Monger, Archon, Aetherborn,
    Construct, Golem, Myr, Robot, Hellion, Scarecrow, Dreadnought, Sable,
    Ooze, Plant, Saproling,
    // Strixhaven-era subtypes. (Book is Codie, Vociferous Codex's
    // 2023-oracle creature type.)
    Inkling, Pest, Fractal, Book,
    // Phyrexia: All Will Be One.
    Mite,
    Orc, Warlock, Bard, Sorcerer, Pilot,
    // Misc. subtypes used by SOS body-only cards.
    Dwarf, Badger, Salamander, Giraffe, Antelope, Pangolin, Hag,
    // SOS Witherbloom Dryad subtype (Essenceknit Scholar).
    Dryad,
    // Modern supplement: Kari Zev's Ragavan token.
    Monkey,
    // Modern supplement: Brazen Scourge.
    Gremlin,
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
    // The Last Airbender hybrid beasts.
    Platypus, Bison,
    // Cube expansion (Collector Ouphe).
    Ouphe,
    // MKM / LCI expansion (Spyglass Siren, Inside Source, Slimy Dualleech).
    Siren, Citizen, Leech,
    // SPM expansion (Marvel's Spider-Man — Mary Jane Watson, Agent Venom).
    Performer, Symbiote,
    // BLB / VOW expansion (Prosperous Innkeeper).
    Halfling,
    // THB expansion (Venomous Hierophant, Pharika's Spawn).
    Gorgon,
    // Bloomburrow / MKM (Voracious Varmint).
    Varmint,
    // Bloomburrow Survival creatures (Cautious Survivor, Defiant Survivor).
    Survivor,
    // Enchantment creature subtype (Enduring Innocence).
    Glimmer,
    // Ninjutsu creature subtype (Fallen Shinobi, etc.).
    Ninja,
    // Kamigawa Samurai subtype (Bushido — Kitsune Blademaster).
    Samurai,
    // Edge of Eternities (2025) subtypes.
    Drix, Scientist,
    // Outlaws of Thunder Junction Mount subtype (Saddle, CR 702.171).
    Mount,
    // Eldraine Peasant subtype (Curious Pair, Giant Killer).
    Peasant,
    // Bloomburrow (2024) animal-folk subtypes.
    Rabbit, Raccoon, Mouse, Wolverine, Mole, Possum, Skunk, Hamster,
    // Invasion block Metathran (Living Airship, Metathran Zombie) and
    // Apocalypse Flagbearers (Standard Bearer, Coalition Honor Guard).
    Metathran, Flagbearer, Volver, Phelddagrif,
    // The Last Airbender (2026).
    Lemur, Kangaroo, Seal,
    // The Lost Caverns of Ixalan (2023).
    Capybara,
    // Modern Horizons 2 (Steel Dromedary).
    Camel,
    // +1/+1-counter "Spike" cycle (Spike Feeder).
    Spike,
    // Ravnica Weird (Turn // Burn's 0/1).
    Weird,
    // Amonkhet (Heart-Piercer Manticore).
    Manticore,
    // ONE (Argentum Masticore).
    Masticore,
    // MKM (Absolving Lammasu).
    Lammasu,
    // Artifact-creature token subtypes (Hangarback Walker's Thopters,
    // Kaladesh Fabricate Servos).
    Thopter,
    Servo,
    // Theros devotion gods (Nylea, Thassa, Erebos, ...).
    God,
    // Artifact creature subtype (Juggernaut).
    Juggernaut,
    // Ravnica Simic graft creature subtype (Cytoplast Root-Kin).
    Mutant,
    // Mirrodin Slith creature subtype (Arcbound Slith).
    Slith,
    // War of the Spark Amass Army token subtype (CR 701.43).
    Army,
    // Edge-of-Eternities Drone artifact-creature token (Pinnacle Emissary).
    Drone,
    // Battle for Zendikar Eldrazi Scion token subtype.
    Scion,
    // Battle for Zendikar / OGW Eldrazi Processor (process-matters).
    Processor,
    // Yeti (Mountain Yeti, Zhalfirin Yeti).
    Yeti,
    // Ranger (Murasa Ranger).
    Ranger,
    // Artifact creature subtype (Bottle Gnomes).
    Gnome,
    // Artifact creature subtype (Court Homunculus, Fblthp).
    Homunculus,
    // Return to Ravnica (Catacomb Slug).
    Slug,
    // Amonkhet Naga (Ramunap Excavator).
    Naga,
    // Strixhaven artifact-creature subtype (Biblioplex Assistant).
    Gargoyle,
    // Ravnica four-color rares (the Nephilim cycle) + their Sand tokens.
    Nephilim,
    Sand,
    // Innistrad Eye Horror (Concealing Curtains // Revealing Eye).
    Eye,
    // Manland animate-into bodies (Mishra's Factory, Inkmoth / Blinkmoth Nexus).
    AssemblyWorker, Blinkmoth,
    // Strixhaven (Eccentric Apprentice).
    Tiefling,
    // Theros Chimera (Daybreak Chimera, Loathsome Chimera).
    Chimera,
    // Kamigawa Zubera (the five Zubera with scaling death triggers).
    Zubera,
    // THB Nadir Kraken's Tentacle token.
    Tentacle,
    // THB Alirios, Enraptured's Reflection token.
    Reflection,
    // Ikoria Brushwagg (Almighty Brushwagg).
    Brushwagg,
    // Warhammer 40,000 Tyranids (Ravenous — CR 702.156).
    Tyranid,
    // Final Fantasy Job Select Hero token (CR 702.182).
    Hero,
    // Final Fantasy Moogle tribe.
    Moogle,
    // Final Fantasy Qu tribe (Quina).
    Qu,
    // TLA — Ember Island Production's 2/2 Coward token.
    Coward,
    // Tarkir: Dragonstorm (Cunning Coyote).
    Coyote,
    // Tarkir: Dragonstorm (Sunpearl Kirin).
    Kirin,
    // "Spellshaper" — discard-cost spellcaster creatures (Seismic Mage).
    Spellshaper,
    // Streets of New Capenna "Villain" (Kingpin's Enforcers).
    Villain,
    // Duskmourn "Toy" artifact-creature subtype (Splitskin Doll, Patched Plaything).
    Toy,
    // Modern Horizons 3 (Cursed Wombat).
    Wombat,
    // Fifth Dawn (the Bringer cycle, Summoning Station's Pincher token).
    Bringer, Pincher,
}

/// Land subtypes (basic land types + others).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LandType {
    Plains, Island, Swamp, Mountain, Forest,
    Desert, Gate, Locus, Mine, Tower, PowerPlant, Urza,
    // Tarkir: Dragonstorm planeswalking dual lands.
    Omenpath,
    // Edge of Eternities — "Land — Planet" (Station-bearing utility lands).
    Planet,
    // Lost Caverns of Ixalan — "Land — Cave" (Caves-matter payoffs).
    Cave,
    // Invasion block — "Land — Lair" (the bounce-a-land tri-lands).
    Lair,
    // Final Fantasy — "Land — Town" (Towns-matter: Affinity for Towns, etc.).
    Town,
    // Phyrexia: All Will Be One — "Land — Sphere" (the enters-tapped cycle +
    // Mirrex/Monument utility lands).
    Sphere,
}

impl LandType {
    /// CR 205.3i — the five basic land types, in WUBRG order.
    pub const BASICS: [LandType; 5] =
        [Self::Plains, Self::Island, Self::Swamp, Self::Mountain, Self::Forest];

    /// CR 205.3i — one of the five basic land types (the only ones that carry an
    /// intrinsic mana ability and can be rewritten by a CR 612 text change).
    pub fn is_basic_type(self) -> bool {
        matches!(
            self,
            Self::Plains | Self::Island | Self::Swamp | Self::Mountain | Self::Forest
        )
    }
}

/// Artifact subtypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactSubtype {
    Equipment, Vehicle, Food, Treasure, Clue, Blood, Fortification, Contraption,
    // Lost Caverns of Ixalan explore token (CR 111.10s).
    Map,
    // The Brothers' War mana token (CR 111.10q).
    Powerstone,
    // Edge of Eternities: station card (CR 721) and its Lander token.
    Spacecraft, Lander,
    // Final Fantasy: Book (grimoire equipment).
    Book,
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

/// Battle subtypes (CR 310.4). Siege is the only printed one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattleSubtype {
    Siege,
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
    // THB walker.
    Calix,
    // THS walker.
    Xenagos,
    // modern_decks: Narset, Parter of Veils.
    Narset,
    // STX: Kasmina, Enigma Sage.
    Kasmina,
    // STX Dean/Strixhaven planeswalker MDFCs.
    Rowan, Will, Lukka,
    // MH2 (Grist, the Hunger Tide).
    Grist,
    // Cube expansion: Wrenn (Wrenn and Six).
    Wrenn,
    // NEO: The Wandering Emperor.
    WanderingEmperor,
    // ONE planeswalkers.
    Koth, Kaya, Tyvar, Kaito,
    // WAR planeswalkers.
    Tibalt, Teyo, Wanderer, Nixilis, Jaya, Angrath, Huatli, Kiora, Samut, Dovin,
    Davriel, Arlinn, Sarkhan, Yanggu,
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
    #[serde(default)]
    pub battle_subtypes: Vec<BattleSubtype>,
}

/// CR 702.158b — a permanent's sector designation. Not a copiable value; it
/// is cleared once no player controls a space-sculptor permanent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sector {
    Alpha,
    Beta,
    Gamma,
}

impl Sector {
    pub const ALL: [Sector; 3] = [Sector::Alpha, Sector::Beta, Sector::Gamma];

    pub fn label(self) -> &'static str {
        match self {
            Sector::Alpha => "alpha",
            Sector::Beta => "beta",
            Sector::Gamma => "gamma",
        }
    }
}

/// Counter types that can be placed on permanents or players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CounterType {
    PlusOnePlusOne,
    MinusOneMinusOne,
    /// -0/-1 counter (Shield Sphere-adjacent block-cost counters).
    MinusZeroMinusOne,
    /// -1/-0 counter.
    MinusOneMinusZero,
    /// +1/+0 counter (Clockwork Avian's wind-up charge).
    PlusOnePlusZero,
    /// +0/+1 counter (Living Armor, Necropolis).
    PlusZeroPlusOne,
    /// +2/+0 counter (Frankenstein's Monster).
    PlusTwoPlusZero,
    /// +0/+2 counter (Frankenstein's Monster).
    PlusZeroPlusTwo,
    /// -0/-2 counter (Spirit Shackle).
    MinusZeroMinusTwo,
    /// Glyph counter (Glyph of Delusion) — the bearer doesn't untap while it
    /// carries one, and sheds one each upkeep.
    Glyph,
    /// Sleep counter (Venarian Gold) — same lock, its own kind.
    Sleep,
    /// Matrix counter (Life Matrix) — spend one to regenerate.
    Matrix,
    /// Hatchling counter (Triassic Egg).
    Hatchling,
    /// Intervention counter (Divine Intervention).
    Intervention,
    /// Pupa counter (Cocoon) — the enchanted creature is locked down while it
    /// carries one, and sheds one each upkeep.
    Pupa,
    /// Dream counter (Rasputin Dreamweaver) — spent for mana or prevention.
    Dream,
    /// Pin counter (Voodoo Doll) — one per upkeep, spent as damage.
    Pin,
    Loyalty,
    Charge,
    /// CR 122 — the manifestation counter Arbiter of the Ideal puts on the
    /// permanent it cheats in. A pure marker.
    Manifestation,
    /// Blood counter — Font of Agonies banks one per point of life paid;
    /// remove four to destroy a creature. Not the Blood artifact token.
    Blood,
    /// Plague counter — Plague Boiler's upkeep tally; at three it sacrifices
    /// itself and destroys every nonland permanent.
    Plague,
    /// Fuse counter — Goblin Bomb's upkeep coin-flip tally; remove five to
    /// deal 20. Distinct from `Charge` so the two don't share a pool.
    Fuse,
    /// Wind counter — Cyclone's escalating upkeep tally.
    Wind,
    /// Hunger counter — Fasting's upkeep tally; at five the enchantment is
    /// destroyed.
    Hunger,
    /// Mine counter — Mine Layer's land trap; a mined land that taps is
    /// destroyed.
    Mine,
    /// Shred counter — Cephalid Vandal's self-milling upkeep tally.
    Shred,
    /// Carrion counter — Osai Vultures' end-step tally of dead creatures;
    /// remove two to pump.
    Carrion,
    /// Delay counter — Delaying Shield banks damage dealt to its controller
    /// here and cashes it in at upkeep.
    Delay,
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
    /// Omen counter — Celestial Convergence's seven-turn fuse.
    Omen,
    /// Azor's Elocutors — a slow win condition (5 filibuster counters).
    Filibuster,
    Shield,
    Wish,
    /// Invitation counter — Wedding Announcement's end-step tally.
    Invitation,
    /// Petal counter — Lotus Blossom's upkeep tally; cash them in for that
    /// many mana of one colour.
    Petal,
    /// Page counter — Strixhaven Book artifacts (Diary of Dreams). Builds
    /// up on instant/sorcery cast and discounts the host's activated
    /// ability one for one. The counter-scaled cost reduction itself is
    /// not yet wired (see TODO.md), but counters tick up correctly.
    Page,
    /// Winch counter — Mercadian Lift's crank; remove X to deploy a creature
    /// with mana value X from hand.
    Winch,
    /// Scream counter — All Hallow's Eve's exile fuse; one comes off each of
    /// the owner's upkeeps and the card cashes in when the last is removed.
    Scream,
    /// Doom counter — Armageddon Clock's upkeep tally; it burns everyone for
    /// that much each draw step.
    Doom,
    /// Growth counter — used on enchantments that count tutoring or
    /// life-gain progress (Comforting Counsel, "as long as N or more
    /// growth counters …"). Distinct from `Charge` so the static-toggle
    /// variants don't collide.
    Growth,
    /// Prepared counter — the SOS "prepared" designation (count-1 flag).
    /// While a creature with a `prepare_spell` carries this counter, its
    /// controller may cast a copy of that spell from exile via
    /// `GameAction::CastPrepareSpell`; casting it removes the counter
    /// ("unprepares it"). The printed "Only creatures with prepare spells
    /// can become prepared" reminder is enforced at the *target* via
    /// `SelectionRequirement::HasPrepareSpell` on the AddCounter /
    /// RemoveCounter selectors. See `.claude/prepared.md`.
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
    /// Silver counter — a pure bookkeeping marker placed on cards in exile
    /// (Karn, Scion of Urza). His +1 exiles a card with a silver counter;
    /// his −1 returns a card you own with a silver counter from exile to
    /// hand.
    Silver,
    /// Luck counter — pure bookkeeping marker. Chance Encounter accrues one
    /// per won coin flip and wins the game at ten (CR 705.1 payoff).
    Luck,
    /// Quest counter — accrues on quest enchantments (Beastmaster Ascension:
    /// one per creature you control that attacks; at 7+ counters its team
    /// anthem switches on via a `StaticEffect::PumpTeamIf` threshold).
    Quest,
    /// Fire counter — TLA artifacts/Vehicles that grow or animate as they
    /// accumulate them (War Balloon: 3+ fire → artifact creature).
    Fire,
    /// Conqueror counter — TLA "as long as this has a conqueror counter"
    /// threshold marker (Zhao, the Moon Slayer: at 1+ conqueror counters,
    /// nonbasic lands become Mountains via `LandTypeChangerWhileCounters`).
    Conqueror,
    /// Study counter — Strixhaven bookkeeping marker on cards in exile
    /// (Kianne // Imbraham). Cards exiled with study counters are tallied by
    /// `Value::DistinctManaValuesInExileWithCounter` (Kianne's Fractal) and
    /// returned via `Effect::ReturnFromExileWithCounter` (Imbraham).
    Study,
    /// Book counters (Spell Satchel's magecraft stockpile).
    Book,
    /// Point counter — Strixhaven Stadium's combat-damage score. Ten or
    /// more on the Stadium ends the game for the damaged opponent.
    Point,
    /// Hone counter — Strixhaven cast-from-exile timer (Uvilda, Dean of
    /// Perfection). An instant/sorcery exiled with hone counters ticks one off
    /// each of the owner's upkeeps; when the last is removed it becomes
    /// castable from exile for {4} less (`GameState::process_hone`).
    Hone,
    /// Burden counter — The One Ring's draw/life tally.
    Burden,
    /// Ice counter — Thing in the Ice ticks one off per instant/sorcery you
    /// cast; when the last is removed it transforms (CR 712).
    Ice,
    /// Fate counter — Oblivion Stone's bookkeeping marker; permanents with
    /// one survive its destroy-everything activation.
    Fate,
    /// Soot counter — Smokestack's per-upkeep sacrifice tally.
    Soot,
    /// Void counter — Dauthi Voidwalker's stamp on opponents' cards its
    /// replacement exiles; its sacrifice ability frees one for a free play.
    Void,
    /// Ki counter — Kamigawa flip cards (Cunning Bandit, Faithful Squire,
    /// Hired Muscle, …) accrue these on Spirit/Arcane casts and flip at two or
    /// more; the bottom faces spend them for their activated abilities.
    Ki,
    /// Coin counter — Athreos, Shroud-Veiled marks creatures with these; when
    /// a coin-countered creature dies, its card returns under Athreos's
    /// controller's control.
    Coin,
    /// Tide counter — Ominous Seas accrues one on your first draw each turn;
    /// at four or more they're removed to mint an 8/8 Kraken.
    Tide,
    /// CR 122 — a flood counter (Quicksilver Fountain: a land with one is an
    /// Island).
    Flood,
    /// Bounty counter — Chevill, Bane of Monsters marks an opponent's
    /// creature/planeswalker; when a bounty-countered creature dies, Chevill's
    /// controller draws and gains a life.
    Bounty,
    /// Oil counter — a generic resource counter on Phyrexian permanents,
    /// spent by activated abilities (Glistener Seer, Migloz).
    Oil,
    /// Blight counter — Ultima, Origin of Oblivion. A blighted land loses
    /// all land types and abilities and has "{T}: Add {C}" while the
    /// counter remains (`StaticEffect::BlightedLandsNeutralized`).
    Blight,
    /// Valor counter — the MID/VOW Adversary cycle's "pay-any-number-of-times"
    /// counter, scaling a `PumpPTPerCounterOnSource` team anthem (Intrepid
    /// Adversary).
    Valor,
    /// Defense counter — CR 310.7. A battle enters with a number of defense
    /// counters equal to its printed defense; combat/noncombat damage removes
    /// that many, and when the last is removed the battle is defeated
    /// (`EventKind::BattleDefeated`).
    Defense,
    /// Possession counter — DSK Unwilling Vessel accrues one per Eerie
    /// trigger; on death it mints an X/X Spirit where X is the count.
    Possession,
    /// Nest counter — DSK Twitching Doll accrues one each time its mana
    /// ability is used; sacrificing it makes a Spider per counter on it.
    Nest,
    /// Muster counter — Assemble the Legion tallies one each upkeep and mints a
    /// Soldier token per counter on it.
    Muster,
    /// Acorn counter — Chitterspitter accrues one per sacrificed token and
    /// scales its Squirrel anthem by the count.
    Acorn,
    /// Incubation counter — Drake Hatcher tallies combat damage dealt; remove
    /// three to mint a 2/2 flying Drake.
    Incubation,
    /// Revival counter — Nine-Lives Familiar enters with eight; on death it
    /// returns with one fewer at the next end step while any remain.
    Revival,
    /// Stash counter — Tinybones, Bauble Burglar stamps exiled discarded cards;
    /// their owner isn't you, but you may play them from exile.
    Stash,
    /// Divinity counter — the Myojin cycle enters with one if cast from hand;
    /// it grants indestructibility and fuels a one-shot activated ability.
    Divinity,
    /// Devotion counter — Pious Kitsune's upkeep tally / Bloodthirsty Ogre's
    /// -X/-X fuel (CHK).
    Devotion,
    /// Aim counter — Hankyu banks one per activation and spends the lot as
    /// damage (CHK).
    Aim,
    /// Theft counter — Night Dealings banks one per point of damage its
    /// controller's sources deal; remove X to tutor a mana-value-X card.
    Theft,
    /// Training counter — Sensei Golden-Tail's sorcery-speed bushido grant.
    Training,
    /// Fellowship counter — Banner of Kinship enters with one per chosen-type
    /// creature; the chosen type gets +1/+1 per counter.
    Fellowship,
    /// Bait counter — Fishing Pole accrues one per activation; removing one on
    /// the equipped creature's untap mints a 1/1 Fish.
    Bait,
    /// Supply counter — Stocking the Pantry accrues one whenever you put +1/+1
    /// counters on a creature you control; remove one to draw.
    Supply,
    /// Unlock counter — MKM's Cryptex tallies one per collect-evidence
    /// activation; its sacrifice ability is gated on five or more.
    Unlock,
    /// Palliation counter — Palliation Accord accrues one whenever an opponent's
    /// creature becomes tapped; remove one to prevent 1 damage to you.
    Palliation,
    /// Eon counter — Magosi, the Waterveil banks one when you skip a turn and
    /// cashes it in for an extra turn.
    Eon,
    /// Blaze counter — Obsidian Fireheart sets a land on fire; the land keeps
    /// burning its controller for 1 each upkeep, source or no source.
    Blaze,
    /// Phylactery counter — Phylactery Lich anchors its indestructibility to
    /// an artifact; losing every phylactery-countered permanent kills it.
    Phylactery,
    /// Arrow counter — Archery Training accrues one each upkeep and fuels the
    /// enchanted creature's "deals X damage" tap ability (UDS).
    Arrow,
    /// Infection counter — Festering Wound accrues one each of your upkeeps and
    /// bites the enchanted creature's controller for the count (UDS).
    Infection,
    /// Sporogenesis — a fungus counter; a creature carrying them mints one
    /// Saproling per counter when it dies.
    Fungus,
    /// Storage counter — the MMQ storage lands bank one per tap and cash any
    /// number in for that much colored mana.
    Storage,
    /// Depletion counter — the MMQ depletion lands enter with two and are
    /// sacrificed once the last one is spent.
    Depletion,
    /// Invasion's Temporal Distortion — a tap-tracking counter that blocks
    /// the next untap.
    Hourglass,
    /// Kangee, Aerie Keeper's kicked-X counters, which pump other Birds.
    Feather,
    /// Aurification's gold counters (CR 122.1 — a marker counter).
    Gold,
    /// Trap Digger's trap counters — a marker on a land, spent by sacrificing it.
    Trap,
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
    /// CR 407 — the ante zone. Only exists while playing for ante; the winner
    /// takes everything in it (CR 407.2).
    Ante,
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
    /// Generic mana equal to the resolution's declared `{X}` — "unless its
    /// controller pays {X}" (Excise).
    GenericXFromCost,
    /// Generic mana equal to the number of `CounterType` counters on the
    /// ability's source (Primordial Ooze's escalating upkeep).
    GenericCountersOnSource(CounterType),
    Life(u32),
    /// Compound "Ward—{cost}, Pay N life" (Ovika, Enigma Goliath; Gisa, the
    /// Hellraiser). Both halves must be paid or the spell/ability is countered.
    ManaAndLife(crate::mana::ManaCost, u32),
    Discard(u32),
    /// "…unless you discard a [filter] card" — the filtered sibling of
    /// `Discard` (Body Snatcher). Unpayable when the hand holds too few
    /// matching cards.
    DiscardMatching(Box<SelectionRequirement>, u32),
    /// "...unless its controller discards their hand." (Perplex.) Payable
    /// even from an empty hand — discarding zero cards is a legal payment.
    DiscardHand,
    /// "Ward—Blight N." (CR 701.68 — Auntie Ool, Cursewretch.) The warding
    /// player must put N -1/-1 counters on a creature they control.
    Blight(u32),
    /// "Ward—Collect evidence N." (CR 701.59 — Axebane Ferox.) The warding
    /// player must exile cards with total mana value ≥ N from their graveyard.
    CollectEvidence(u32),
    /// "…unless you exile N cards from your graveyard" (Rotting Giant).
    /// Unpayable when the graveyard holds fewer than N cards; auto-pay takes
    /// the cheapest.
    ExileFromGraveyard(u32),
    /// "…unless you put N cards from your graveyard on the bottom of your
    /// library" (Anurid Scavenger). Unpayable when the graveyard is too small;
    /// auto-pay bottoms the cheapest.
    BottomFromGraveyard(u32),
    /// "…unless [player] has [this source] deal N damage to them" — the
    /// Odyssey pay-in-damage menu (Blazing Salvo, Lava Blister, Molten
    /// Influence). Always payable; the damage comes from the effect's source.
    DamageFromSource(u32),
    SacrificeCreature,
    /// "…unless you sacrifice a [filter]" — the filtered sibling of
    /// `SacrificeCreature` (Endless Wurm's enchantment, Contamination's
    /// creature). Unpayable when nothing matches.
    SacrificeMatching(Box<SelectionRequirement>),
    /// "Ward—Sacrifice N permanents." (Ulamog, the Defiler.)
    SacrificePermanents(u32),
    /// "…unless you sacrifice N [filter]" — the counted sibling of
    /// `SacrificeMatching` ("Morph—Sacrifice two Mountains", Skirk Volcanist).
    /// Unpayable when fewer than N match.
    SacrificeMatchingN(Box<SelectionRequirement>, u32),
    /// "…unless you return a [filter] you control to its owner's hand"
    /// ("Morph—Return a Bird you control to its owner's hand", Raven Guild
    /// Initiate). Unpayable when nothing matches.
    ReturnMatchingToHand(Box<SelectionRequirement>),
    /// "{X}, where X is this creature's power" — a dynamic generic cost read
    /// off the source's computed power at payment time (Esper Sentinel's
    /// rhystic tax).
    GenericSourcePower,
    /// "Pay life equal to this creature's power" — a dynamic life cost read
    /// off the source's computed power at payment time (Phyrexian Fleshgorger).
    LifeSourcePower,
    /// "...unless you remove a counter from a permanent you control" (Chisei,
    /// Heart of Oceans). Unpayable when no controlled permanent has a counter.
    RemoveCounterFromPermanent,
    /// "…unless you pay its mana cost" — the printed cost of the permanent the
    /// source is attached to (Essence Leak).
    ManaCostOfAttached,
    /// "…unless that player pays {cost} or N life" — a two-way menu (Erosion).
    /// Auto-payment prefers the mana half and falls back to the life half.
    ManaOrLife(crate::mana::ManaCost, u32),
    /// "…unless they sacrifice that [permanent]" — the permanent the source
    /// Aura is attached to, specifically (Curse Artifact). Unpayable once the
    /// Aura has come unattached.
    SacrificeAttachedHost,
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
    /// "...for as long as it remains exiled" — never sweeps; the
    /// permission dies with the card's zone change (Hostage Taker, Gonti).
    WhileExiled,
    /// CR 702.94 — the Miracle window: cast it now or lose the chance.
    /// Cleared at the next step/phase transition (`advance_step`), so the
    /// reveal-on-draw offer can't be banked for a later phase with more
    /// information. Also swept at turn end like `EndOfThisTurn`.
    EndOfThisStep,
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
    /// CR 702.94e — a Miracle cast ignores the sorcery-speed gate: the
    /// printed window is "when the miracle trigger resolves", which the
    /// engine approximates as "during the current step" (see
    /// `MayPlayDuration::EndOfThisStep`), and the rules let the sorcery
    /// be cast there. Defaults to `false` for snapshot back-compat.
    #[serde(default)]
    pub miracle: bool,
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
    /// CR 702.14c — generalized landwalk: can't be blocked while the
    /// defending player controls a land matching the filter (artifact
    /// landwalk — Vectis Gloves; nonbasic/snow landwalk variants).
    LandwalkFiltered(Box<SelectionRequirement>),
    /// Domain landwalk (CR 702.15 + 702.43) — "for each basic land type among
    /// lands you control, this creature has landwalk of that type"
    /// (Magnigoth Treefolk). Evaluated live against both players' lands.
    DomainLandwalk,
    /// Flanking (CR 702.25) — a creature without flanking that blocks this gets -1/-1 until EOT.
    Flanking,
    /// Bushido N (CR 702.45) — when this blocks or becomes blocked, it gets +N/+N until EOT.
    Bushido(u32),
    /// Melee (CR 702.121) — whenever this attacks, it gets +1/+1 until end of
    /// turn for each opponent this player attacked this combat.
    Melee,
    /// Absorb N (CR 702.64) — if a source would deal damage to this
    /// creature, prevent N of that damage (per source, per event; multiple
    /// instances each apply).
    Absorb(u32),
    /// Rampage N (CR 702.23) — when this becomes blocked, it gets +N/+N for each blocker beyond the first.
    Rampage(u32),
    /// Frenzy N (CR 702.35) — whenever this attacks and isn't blocked, it gets
    /// +N/+0 until end of turn.
    Frenzy(u32),
    /// CR 702.158 — Space sculptor. While a permanent with this keyword is on
    /// the battlefield, every creature without a sector designation gets one
    /// (CR 704.5u); the designations clear when the last one leaves.
    SpaceSculptor,
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
    /// CR 702.19c — "trample over planeswalkers" (Thrasta): excess combat
    /// damage past an attacked planeswalker's loyalty hits its controller.
    TrampleOverPlaneswalkers,
    Exert,
    Lifelink,
    Deathtouch,
    Infect,
    Wither,
    /// Toxic N (CR 702.180) — whenever this deals combat damage to a
    /// player, that player also gets N poison counters (in addition to
    /// the normal damage).
    Toxic(u32),
    /// Poisonous N (CR 702.70) — whenever this creature deals combat damage
    /// to a player, that player gets N poison counters. Mechanically the
    /// same combat-damage poison rider as Toxic (the wording differs but the
    /// effect is identical), so it folds into the same combat path.
    Poisonous(u32),
    /// Sunburst (CR 702.44) — "this enters with a +1/+1 counter on it for
    /// each color of mana spent to cast it" (a charge counter instead if it
    /// isn't entering as a creature). The count is the cast's converge value.
    Sunburst,
    /// Modular N (CR 702.43) — a marker keyword. The enters-with-N-counters
    /// half rides `enters_with_counters` and the death trigger rides
    /// `shortcut::modular_dies()`; the keyword makes "creature with modular"
    /// filters (Arcbound Overseer) exact and surfaces the ability in the UI.
    Modular(u32),
    /// Compleated (CR 702.150) — if any of this planeswalker's Phyrexian
    /// pips were paid with life, it enters with two fewer loyalty counters
    /// per 2 life paid (tracked via `CardInstance.compleated_life_paid`).
    Compleated,
    Defender,
    /// CR 702.147 — Decayed. "This creature can't block. When it attacks,
    /// sacrifice it at end of combat." Common on Zombie tokens (MID/VOW).
    Decayed,
    Protection(Color),
    /// CR 702.16h — "protection from colored spells" (Emrakul, the Aeons
    /// Torn). Enforced as a cast-time targeting gate: a spell with one or
    /// more colors can't target this permanent.
    ProtectionFromColoredSpells,
    /// CR 702.16 — "protection from spells" (Emrakul, the World Anew):
    /// can't be targeted by any spell. Cast-time targeting gate.
    ProtectionFromSpells,
    /// CR 702.16 — "protection from creatures" (Spirit Mantle): can't be
    /// blocked, and all damage from creature sources is prevented.
    ProtectionFromCreatures,
    /// CR 702.16 — protection from every source matching `filter`, the
    /// general form of the fixed protection keywords ("protection from
    /// non-Spirit creatures" — Harbinger of Spring). Checked wherever
    /// `ProtectionFromCreatureType` is.
    ProtectionFromMatching(Box<SelectionRequirement>),
    /// CR 702.16e — protection from a creature type (Kitsune Riftwalker's
    /// "protection from Spirits"): can't be blocked by, damaged by, enchanted
    /// or equipped by, or targeted by a permanent of that creature type.
    ProtectionFromCreatureType(CreatureType),
    /// CR 702.16 — protection from a spell subtype (Kitsune Riftwalker's
    /// "protection from Arcane"). Cast-time targeting gate: a spell with the
    /// subtype can't target this permanent.
    ProtectionFromSpellSubtype(SpellSubtype),
    /// CR 702.16 — "protection from each mana value other than N" (Haktos the
    /// Unscarred). Can't be blocked/damaged/targeted by a source whose mana
    /// value isn't `N`. Granted at random on ETB.
    ProtectionFromManaValueExcept(u32),
    /// CR 702.16 — "protection from each mana value of the chosen quality"
    /// (Lavabrink Venturer). `odd == true` → protection from every odd mana
    /// value; `odd == false` → every even mana value (zero is even). Granted
    /// on ETB by the controller's odd/even choice.
    ProtectionFromManaValueParity { odd: bool },
    /// CR 702.16 — "protection from multicolored" (Stonecoil Serpent, Mirran
    /// Crusader). Can't be blocked/damaged/targeted/enchanted by a source that
    /// is two or more colors.
    ProtectionFromMulticolored,
    /// CR 702.16 — "protection from monocolored" (Guardian of the Guildpact).
    /// Can't be blocked/damaged/targeted/enchanted by a source that is exactly
    /// one color (a colorless source is not monocolored, so it gets through).
    ProtectionFromMonocolored,
    /// CR 702.16j — protection from a card type (Serra's Emissary's granted
    /// "protection from the chosen card type"): can't be blocked, damaged,
    /// enchanted/equipped, or targeted by a source of that type.
    ProtectionFromCardType(CardType),
    /// CR 702.16 — "protection from instants" (Hexdrinker level 3-7). Cast-time
    /// targeting gate: an instant spell can't target this permanent. (Instants
    /// are never permanents/blockers/combat sources, so this only matters at the
    /// spell-target check.)
    ProtectionFromInstants,
    /// CR 702.16 — "protection from everything" (Hexdrinker level 8+,
    /// Progenitus): can't be blocked, targeted, enchanted/equipped, or dealt
    /// damage by anything. Returns true at every protection-check site.
    ProtectionFromEverything,
    /// CR 702.89 — Umbra armor (on an Aura): if the enchanted creature
    /// would be destroyed, instead remove all damage from it and destroy
    /// this Aura (Hyena Umbra, Spider Umbra).
    UmbraArmor,
    Hexproof,
    /// CR 702.11e — "hexproof from [color]": can't be the target of
    /// [color] spells or abilities opponents control. The targeting-only
    /// half of protection-from-color (Witchbane Orb-style, Veil of Summer's
    /// rider granted via the turn-scoped player flag).
    HexproofFromColor(Color),
    /// CR 702.11f — "hexproof from monocolored": can't be the target of
    /// exactly-one-color spells or abilities opponents control (General
    /// Ferrous Rokiric).
    HexproofFromMonocolored,
    /// "Can't be the target of nongreen spells opponents control or abilities
    /// from nongreen sources opponents control" and its siblings (Thrun,
    /// Breaker of Silence). Blocks an opponent's spell/ability whose source
    /// includes *none* of the listed colors. `HexproofExceptColors(vec![Green])`
    /// = hexproof from nongreen opponents.
    HexproofExceptColors(Vec<Color>),
    /// CR 702.11d — "hexproof from [quality]" specialized to abilities: can't
    /// be the target of activated or triggered abilities opponents control
    /// (still targetable by spells). Volatile Stormdrake.
    HexproofFromAbilities,
    Shroud,
    CantBeCountered,
    /// CR 117.x — "If X is N or more, this spell can't be countered."
    /// Banefire's printed rider. Threaded through cast time:
    /// `caster_grants_uncounterable_with_x` flips the
    /// `StackItem::Spell.uncounterable` flag when the X paid is at
    /// least the threshold.
    CantBeCounteredIfXAtLeast(u32),
    /// CR 702.16 — "has protection from its colors" (Earnest Fellowship).
    /// Resolved against the bearer's own computed colours at check time, so a
    /// colour change tracks.
    ProtectionFromOwnColors,
    Indestructible,
    Regenerate(u32),
    Persist,
    Undying,
    Recursion,
    Flash,
    Flashback(crate::mana::ManaCost),
    /// "Flashback—Tap N untapped creatures you control" — a Flashback
    /// cost paid not in mana but by tapping N creatures, optionally
    /// restricted by `filter` ("an untapped white creature" — Prismatic
    /// Strands). Recognized by `GameAction::CastFlashbackTap`. Mutually
    /// exclusive with `Keyword::Flashback(_)` in practice (a card has
    /// one Flashback variant or the other).
    FlashbackTap { count: u32, filter: Option<Box<SelectionRequirement>> },
    /// CR 702.103 — Jump-start: cast from the graveyard for the card's own
    /// mana cost plus discarding a card; exiles after resolving (rides the
    /// flashback cast path). Chemister's Insight, Radical Idea.
    JumpStart,
    Kicker(crate::mana::ManaCost),
    /// CR 702.33c — Multikicker: a kicker cost that may be paid any number
    /// of times. `CardInstance.kick_count` records how many times;
    /// `Value::TimesKicked` reads it (Everflowing Chalice).
    Multikicker(crate::mana::ManaCost),
    /// CR 702.27 — Buyback. An optional additional cost paid when casting
    /// the spell from hand; if paid, the spell returns to its owner's hand
    /// instead of the graveyard as it resolves. Cast via
    /// `GameAction::CastSpellBuyback`.
    Buyback(crate::mana::ManaCost),
    /// CR 702.41 — Entwine. An optional additional cost on a modal spell;
    /// if paid, every mode runs in order instead of one. Cast via
    /// `GameAction::CastSpellEntwine`. Tooth and Nail.
    Entwine(crate::mana::ManaCost),
    /// CR 702.176 — Bargain. An optional additional cost: as you cast the
    /// spell you may sacrifice an artifact, enchantment, or token. If you do,
    /// the spell is "bargained" (`Predicate::SpellWasBargained` gates the
    /// bonus). Cast via `GameAction::CastSpellBargain`.
    Bargain,
    /// CR 702.62 — Suspend N—[cost]. You may pay the suspend cost to exile
    /// the card from your hand with N time counters on it. At each of the
    /// owner's upkeeps a time counter is removed; when the last is removed
    /// the owner casts it without paying its mana cost (and a creature so
    /// cast has haste). First field = N, second = the suspend cost. Cast
    /// via `GameAction::Suspend`.
    Suspend(u32, crate::mana::ManaCost),
    /// Suspend accelerant (Deep-Sea Kraken): while this card is suspended,
    /// remove a time counter from it whenever an opponent casts a spell.
    SuspendAccelerant,
    Convoke,
    /// CR 702.126 — Improvise: artifacts you control can help cast this
    /// spell (each tapped artifact pays {1}). Rides the Convoke cast path
    /// (`CastSpellConvoke`), accepting untapped artifacts instead of
    /// creatures.
    Improvise,
    Delve,
    Cascade,
    /// CR 702.50 — Epic. On resolution: the controller can't cast spells
    /// for the rest of the game, and the spell is copied at each of their
    /// upkeeps (Enduring Ideal).
    Epic,
    Cycling(crate::mana::ManaCost),
    /// CR 702.47 — "Splice onto [quality] [cost]": while in hand, reveal as
    /// you cast a matching spell to add this card's rules text to it for the
    /// cost. Cast via `GameAction::CastSpellSpliced` (Glacial Ray).
    Splice(crate::mana::ManaCost, SpellSubtype),
    /// CR 702.29 — Cycling whose cost is a life payment instead of mana
    /// ("Cycling—Pay 2 life", Street Wraith).
    CyclingLife(u32),
    /// CR 702.29e — Typecycling (landcycling). A variant of Cycling: pay the
    /// cost + discard this card, then search your library for a land card with
    /// the given land type, reveal it, and put it into your hand (then
    /// shuffle). "When you cycle" triggers still fire (it is a cycling ability).
    Landcycling(crate::mana::ManaCost, LandType),
    /// CR 702.29e — general Typecycling: pay the cost + discard this card,
    /// then search your library for a card matching the requirement and put
    /// it into your hand (then shuffle). Covers "Basic landcycling"
    /// (`IsBasicLand`), Wizardcycling, Slivercycling… Plain landcycling
    /// keeps the dedicated `Landcycling` variant. The payload is boxed
    /// whole so the variant doesn't grow `Keyword` (it feeds back into
    /// `SelectionRequirement`/`Effect` sizes and deep debug-build recursion).
    Typecycling(Box<(crate::mana::ManaCost, SelectionRequirement)>),
    Echo(crate::mana::ManaCost),
    /// CR 702.29b — Echo with a non-mana cost: "Echo—Discard a card"
    /// (Rakdos Headliner). Processed by the same `process_echo` turn-based
    /// check as `Echo`.
    EchoDiscard,
    /// CR 702.24 — Cumulative upkeep: at the controller's upkeep, add an age
    /// counter, then pay the cost once per age counter or sacrifice this.
    CumulativeUpkeep(CumulativeUpkeepCost),
    /// CR 702.32 — Fading N. Enters with N fade counters. At the beginning
    /// of its controller's upkeep, remove a fade counter from it; if you
    /// can't, sacrifice it.
    Fading(u32),
    /// CR 702.62 — Vanishing N. Enters with N time counters. At the
    /// beginning of its controller's upkeep, remove a time counter from it;
    /// when the last is removed, sacrifice it.
    Vanishing(u32),
    /// CR 702.183 — Impending N—[cost]. May be cast for its impending cost;
    /// if so it enters with N time counters and isn't a creature while it has
    /// one. At the beginning of its controller's end step, remove a time
    /// counter (no sacrifice — it just turns into a creature when the last
    /// comes off). The impending mana cost rides `AlternativeCost.impending`.
    Impending(u32),
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
    /// "This doesn't untap during your untap step if it has a [kind] counter
    /// on it" (Steel Dromedary). Checked in `do_untap`.
    DoesntUntapWhileCounter(CounterType),
    /// CR 502.1 — "You may choose not to untap this during your untap step"
    /// (Hisoka's Guard). `do_untap` asks the controller; declining leaves it
    /// tapped without cancelling the untap step for anything else.
    MayChooseNotToUntap,
    /// "Prevent all damage that would be dealt to this by [filter] sources"
    /// (Argothian Treefolk's artifact sources, Argothian Pixies' artifact
    /// creatures, Artifact Ward's host). Applied in the damage funnel.
    PreventDamageFromMatching(Box<SelectionRequirement>),
    /// "This can't be the target of abilities from [filter] sources"
    /// (Artifact Ward). Checked at the activated/triggered targeting gate.
    CantBeTargetedByAbilitiesFromMatching(Box<SelectionRequirement>),
    /// "This creature can't be the target of Aura spells" (Bartel Runeaxe,
    /// Tetsuo Umezawa). Checked alongside protection in the cast-time target
    /// gate; abilities and non-Aura spells still reach it.
    CantBeTargetedByAuras,
    /// CR 702.14 — legendary landwalk: unblockable while the defending player
    /// controls a legendary land (Livonya Silone). A supertype-scoped walk, so
    /// it can't ride `Landwalk(LandType)`.
    LegendaryLandwalk,
    /// CR 614.9 — "All damage that would be dealt to this creature is dealt to
    /// its controller instead" (Treacherous Link's host). Consulted at both
    /// damage funnels before shields and marked damage.
    DamageToThisGoesToItsController,
    /// CR 702.189 — Firebending N. "Whenever this creature attacks, add N {R};
    /// you don't lose this mana as steps and phases end (until end of combat)."
    Firebending(u32),
    /// Firebending X, where X is this creature's power (CR 702.189 variant —
    /// Firebending Student). Resolves the same attack-triggered red mana, but N
    /// is the attacker's power at the time it attacks.
    FirebendingPower,
    /// Firebending X, where X is the number of creatures the attacker's
    /// controller controls at attack time (Sun Warriors). Sibling of
    /// `FirebendingPower`; both resolve the same attack-triggered red mana.
    FirebendingCreaturesYouControl,
    /// CR 702.190 — Sneak [cost]. A spell-static alt cast: during your declare
    /// blockers step you may cast this by paying [cost] and returning an
    /// unblocked creature you control to its owner's hand. Carried for display;
    /// the alt-cost itself lives in `CardDefinition.alternative_cost`.
    Sneak(crate::mana::ManaCost),
    /// CR 702.54 — Bloodthirst N. "If an opponent was dealt damage this turn,
    /// this creature enters with N +1/+1 counters on it." Carried for display;
    /// the counters ride an ETB trigger (`shortcut::bloodthirst`).
    Bloodthirst(u32),
    Banding,
    /// CR 702.22d/j — "bands with other [quality]". Creatures matching the
    /// quality may attack as a band as long as one of them has this ability,
    /// and the defending player divides the damage of a creature blocked by
    /// two or more of them (Adventurers' Guildhouse, Master of the Hunt).
    BandsWithOther(Box<SelectionRequirement>),
    Equip(crate::mana::ManaCost),
    /// CR 702.151 — Reconfigure [cost]. An Equipment-creature attaches to a
    /// creature you control (or unattaches) for this cost at sorcery speed,
    /// and isn't a creature while attached. Attach reuses the Equip path
    /// (`has_equip` returns this cost); the "isn't a creature while attached"
    /// rider is a layer-4 `RemoveCardType(Creature)` in `compute_battlefield`.
    /// Lion Sash.
    Reconfigure(crate::mana::ManaCost),
    Fortify(crate::mana::ManaCost),
    Morph(crate::mana::ManaCost),
    /// CR 702.36b — "Morph—[non-mana cost]": the turn-face-up cost is one of
    /// the [`WardCost`] menu entries rather than mana (Putrid Raptor's "discard
    /// a Zombie card", Zombie Cutthroat's "pay 5 life"). Casting the card face
    /// down is unaffected — that's always {3}.
    MorphCost(Box<WardCost>),
    Megamorph(crate::mana::ManaCost),
    /// CR 702.77 — Reinforce N—[cost]. "[cost], Discard this card: Put N +1/+1
    /// counters on target creature." An activated ability usable from the hand.
    Reinforce(u32, crate::mana::ManaCost),
    /// CR 702.166 — Disguise. Like Morph, but the face-down permanent is a 2/2
    /// with ward {2}; turn it face up for this cost. Casts face down for {3}.
    Disguise(crate::mana::ManaCost),
    /// CR 702.146 — Disturb. Cast this card from your graveyard transformed
    /// (its back face) for this cost, sorcery speed. The back face's "would
    /// be put into a graveyard → exile instead" rider is keyed off this
    /// keyword at the graveyard funnels. `GameAction::CastDisturb`.
    Disturb(crate::mana::ManaCost),
    Prowess,
    Ward(WardCost),
    /// "Whenever this creature becomes the target of a spell or ability for
    /// the first time each turn, counter that spell or ability" — the
    /// Glasskite cycle. Enforced alongside Ward at the targeting hooks, but
    /// it fires on its own controller's spells too.
    CounterFirstTargetingEachTurn,
    Changeling,
    /// CR 702.139 — Companion. Deck-construction restriction unvalidated;
    /// a sideboard copy moves to hand for {3} at sorcery speed via
    /// `GameAction::CompanionToHand`.
    Companion,
    /// CR 702.146 — Daybound. A permanent with daybound is on the battlefield
    /// only as day; on the front face of a daybound/nightbound DFC. When it
    /// becomes night, the engine transforms it to its nightbound back face.
    /// Casting/entering a daybound permanent makes it day if it's neither.
    Daybound,
    /// CR 702.146 — Nightbound (the back face). When it becomes day, the
    /// engine transforms it back to its daybound front face.
    Nightbound,
    /// CR 702.114 — Devoid is a characteristic-defining ability: the object
    /// is colorless regardless of the colored pips in its mana cost. Honored
    /// in `colors_from_card` (the layer-5 color base), so protection-from-
    /// color and "colorless permanent" payoffs treat it as colorless.
    Devoid,
    Storm,
    /// CR 702.61 — Split second. While this spell is on the stack, players
    /// can't cast other spells or activate abilities that aren't mana
    /// abilities (special actions and triggered abilities still work).
    SplitSecond,
    /// CR 702.69 — "When you cast this spell, copy it for each permanent put
    /// into a graveyard from the battlefield this turn." A self-cast copy
    /// rider mirroring Storm but counting `permanents_to_graveyard_this_turn`.
    Gravestorm,
    /// "When you cast this spell, copy it for each other instant and
    /// sorcery spell you've cast this turn." A self-cast copy rider
    /// mirroring Storm but counting the caster's
    /// `instants_or_sorceries_cast_this_turn` (Show of Confidence).
    SpellStorm,
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
    /// CR 509.1b — "This creature can't block creatures with power N or
    /// greater" (Ironclaw Orcs, Ironclaw Buzzardiers). Enforced against the
    /// attacker's *computed* power in `can_block_attacker_computed`.
    CantBlockPowerAtLeast(u32),
    /// CR 509.1b — "This creature can't block creatures with power greater
    /// than this creature's power" (Spitfire Handler). The self-relative
    /// sibling of `CantBlockPowerAtLeast`; both powers are the computed ones.
    CantBlockGreaterPowerThanSelf,
    /// CR 509.1b — "This creature can block an additional N creatures each
    /// combat" (Knight of Sorrows, Sunweb). Raises the blocker's per-combat
    /// assignment cap in `declare_blockers`; instances stack.
    CanBlockAdditional(u32),
    /// CR 509.1b — "This creature can block any number of creatures"
    /// (Guardian of the Gateless, Valor Made Real).
    CanBlockAnyNumber,
    /// "This creature can't attack." A static restriction on the bearer
    /// (or an Aura/effect grant — Pacifism, Faith's Fetters, Bound in
    /// Silence). Enforced from the *computed* keyword set in
    /// `declare_attackers`, so layer-granted variants are honored.
    CantAttack,
    /// CR 602.5c — "[This permanent]'s activated abilities can't be
    /// activated." A static restriction on the bearer (or an Aura grant —
    /// Detention Vortex, Stupor-style locks). Enforced from the *computed*
    /// keyword set in `activate_ability`; mana abilities are unaffected
    /// (they aren't "activated" through the normal permission gate).
    CantActivateAbilities,
    /// CR 702.98 — Unleash. A marker keyword; the "may enter with a +1/+1
    /// counter" half rides a `shortcut::unleash()` ETB trigger, and the
    /// "can't block while it has a +1/+1 counter" half is injected as a
    /// computed `CantBlock` in `gather_continuous_effects` when the bearer
    /// holds at least one +1/+1 counter. Rakdos Cackler, Gore-House
    /// Chainwalker, Spawn of Rix Maadi.
    Unleash,
    /// CR 508.0 — "This creature can't attack unless it's the only creature
    /// attacking" (Master of Cruelties). Enforced in `declare_attackers`:
    /// a batch that declares this creature alongside any other attacker is
    /// rejected.
    AttacksAlone,
    /// CR 508.0 — "This creature can't attack alone" (Militia Rallier,
    /// Cemetery Gatekeeper-era). Enforced in `declare_attackers`: a batch
    /// where this is the *only* attacker is rejected.
    CantAttackAlone,
    /// CR 509.1c — "This creature can't attack or block alone" (Duskmourn's
    /// Beast tokens). Enforced in `declare_attackers` / `declare_blockers`:
    /// a batch where this is the *only* attacker (resp. only blocker this
    /// combat) is rejected.
    CantAttackOrBlockAlone,
    /// CR 508.1a restriction — "This creature can't attack unless you've cast
    /// a creature spell this turn" (Goblin Cohort, Goblin War Strike-era
    /// aggro). Enforced in `declare_attackers` against the controller's
    /// `creatures_cast_this_turn` tally.
    CantAttackUnlessCastCreatureThisTurn,
    /// CR 508.1a restriction — "This creature can't attack unless there is a
    /// [land type] on the battlefield" (Glacial Crasher). Any player's land
    /// counts; enforced in `declare_attackers`.
    CantAttackUnlessLandTypeOnBattlefield(LandType),
    /// CR 508.1a restriction — "This creature can't attack unless defending
    /// player controls a [land type]" (Merchant Ship, Island Fish Jasconius).
    /// The defender-scoped sibling of `CantAttackUnlessLandTypeOnBattlefield`.
    CantAttackUnlessDefenderControlsLandType(LandType),
    /// CR 508.1a restriction — "This creature can't attack unless there are N
    /// or more [land type]s on the battlefield" (Harbor Serpent). Any player's
    /// lands count; the N=1 case is `CantAttackUnlessLandTypeOnBattlefield`.
    CantAttackUnlessLandCount(LandType, u32),
    /// CR 508.1a restriction — "This creature can't attack unless an opponent
    /// has been dealt damage this turn" (Bloodcrazed Goblin). Reads the
    /// per-turn `was_dealt_damage_this_turn` flag on each opponent.
    CantAttackUnlessOpponentDamaged,
    /// CR 508.1a restriction — "This creature can't attack if defending player
    /// controls an untapped land" (Branded Brawlers).
    CantAttackIfDefenderHasUntappedLand,
    /// CR 508.1g cost — "This creature can't attack unless its controller pays
    /// {N}" (Brainwash). The attack-only sibling of
    /// `CantAttackOrBlockUnlessPay`; paid out of the same declare-attackers
    /// pool as the Propaganda tax.
    CantAttackUnlessPay(u32),
    /// CR 502.3 — "This doesn't untap during its controller's untap step if it
    /// attacked during their last turn" (Goblin Rock Sled, Tangle Kelp).
    DoesntUntapIfAttackedLastTurn,
    /// CR 115.6 — "This creature can't be the target of spells unless it
    /// attacked or blocked this turn" (Lurker). Abilities may still target it.
    CantBeTargetedBySpellsUnlessAttackedOrBlocked,
    /// CR 508.1a restriction — "This creature can't attack if it attacked
    /// during your last turn" (Giant Turtle). Reads the bearer's
    /// `attacked_last_turn` roll-over flag.
    CantAttackIfAttackedLastTurn,
    /// CR 509.1b restriction — the blocking half of Branded Brawlers: "can't
    /// block if you control an untapped land."
    CantBlockIfYouHaveUntappedLand,
    /// CR 702.161 — Living metal: "During your turn, this permanent is an
    /// artifact creature in addition to its other types." A Vehicle that
    /// animates itself on its controller's turn, no crew needed.
    LivingMetal,
    /// CR 508.1g / 509.1b cost — "can't attack or block unless you tap an
    /// untapped [filter] you control not declared as an attacking or blocking
    /// creature this combat" (Hollow Warrior). Paid as attackers / blockers
    /// are declared; each such creature consumes its own helper.
    AttackBlockCostTapAnother(Box<SelectionRequirement>),
    /// CR 508.1a restriction — "This creature can't attack unless you control
    /// more creatures than the defending player" (Mogg Toady).
    CantAttackUnlessMoreCreaturesThanDefender,
    /// CR 509.1b restriction — the blocking half of Mogg Toady: "can't block
    /// unless you control more creatures than the attacking player."
    CantBlockUnlessMoreCreaturesThanAttacker,
    /// CR 508.1a restriction — "This creature can't attack unless a creature
    /// with greater power also attacks" (Okk). Checked against the whole
    /// declared batch, so it needs a partner big enough in the same combat.
    CantAttackUnlessGreaterPowerAttacks,
    /// CR 509.1b restriction — the blocking half of Okk: "can't block unless a
    /// creature with greater power also blocks."
    CantBlockUnlessGreaterPowerBlocks,
    /// CR 508.1g — "This creature can't attack unless you return a [filter]
    /// you control to its owner's hand." An additional cost paid as attackers
    /// are declared (Floodtide Serpent); enforced in `declare_attackers`,
    /// which picks and bounces one matching permanent per such attacker.
    AttackCostBounce(Box<SelectionRequirement>),
    /// CR 508.1g cost — "This creature can't attack unless you sacrifice N
    /// [filter]" (Leviathan). Paid as attackers are declared; the declaration
    /// is rejected when the pool is short.
    AttackCostSacrifice(Box<SelectionRequirement>, u32),
    /// CR 508.1a / 509.1a restriction — "This creature can't attack or block
    /// unless you have N or fewer cards in hand" (Hazoret the Fervent, the
    /// Amonkhet Gods). Enforced in `declare_attackers` / blocker legality
    /// against the controller's hand size.
    CantAttackOrBlockUnlessHandSizeAtMost(u32),
    /// CR 508.1a / 509.1a restriction with a cost — "This creature can't
    /// attack or block unless its controller pays {N}" (Oppressive Rays,
    /// Pacifism's taxing cousins). The controller is offered the payment as
    /// attackers/blockers are declared; declining makes the declaration
    /// illegal.
    CantAttackOrBlockUnlessPay(u32),
    /// CR 508.1a / 509.1a — the same tax, scaled by the source's own counters:
    /// "can't attack or block unless you pay {1} for each +1/+1 counter on it"
    /// (Myr Prototype).
    CantAttackOrBlockUnlessPayPerCounter(CounterType),
    /// CR 508.1a / 509.1a — the tax scaled by the hand of whoever granted it:
    /// "can't attack or block unless its controller pays {1} for each card in
    /// *your* hand" (Cowed by Wisdom). Summed per attached Aura carrying the
    /// grant, so the enchanter's hand is what's counted, not the host's.
    CantAttackOrBlockUnlessPayPerCardInEnchanterHand,
    /// CR 508.1a / 509.1a — the tax scaled by a battlefield count: "can't
    /// attack or block unless its controller pays {1} for each Cleric on the
    /// battlefield" (Whipgrass Entangler). Every matching permanent counts,
    /// whoever controls it.
    CantAttackOrBlockUnlessPayPerPermanent(Box<SelectionRequirement>),
    /// CR 508.1a / 509.1a restriction — "This creature can't attack or block
    /// unless there are four or more card types among cards in your graveyard"
    /// (Delirium; Patchwork Beastie). Enforced against `delirium_active`.
    CantAttackOrBlockUnlessDelirium,
    /// CR 508.1a / 509.1a restriction — "Descend N — This creature can't attack
    /// or block unless there are N or more permanent cards in your graveyard"
    /// (The Ancient One, descend 8). Enforced against the controller's
    /// permanent-card graveyard count (`descend_count`).
    CantAttackOrBlockUnlessDescend(u32),
    /// CR 508.1a / 509.1a restriction — "This creature can't attack or block
    /// unless you have the city's blessing" (CR 702.131 — Wayward Swordtooth).
    /// Enforced against the controller's `city_blessing` flag.
    CantAttackOrBlockUnlessCityBlessing,
    /// CR 508.1a / 509.1a restriction — "[This creature] can't attack or block
    /// unless a creature died under your control this turn" (Bontu the
    /// Glorified). Enforced against the controller's `creatures_died_this_turn`.
    CantAttackOrBlockUnlessCreatureDiedThisTurn,
    /// "This creature assigns no combat damage this turn" (Master of
    /// Cruelties' attack rider). A marker keyword — typically granted with
    /// `Duration::EndOfTurn` by a trigger — that `combat.rs` checks off the
    /// *computed* keyword set to zero out the bearer's combat damage in
    /// both the first-strike and regular damage steps.
    DealsNoCombatDamage,
    /// "You may assign this creature's combat damage divided as you choose
    /// among defending player and/or any number of creatures they control"
    /// (Butcher Orgg). Read off the *computed* keyword set in `combat.rs`:
    /// the bearer's damage bypasses the CR 510.1c blocker order entirely and
    /// is divided freely over the defending player and their creatures.
    DividesCombatDamageAmongDefenders,
    /// "Assigns combat damage equal to its toughness rather than its power"
    /// (Doran, the Siege Tower; Bill the Pony's temporary grant). A marker
    /// keyword read off the *computed* keyword set in `combat.rs`: the bearer
    /// uses its toughness as the damage value it assigns in both combat-damage
    /// steps (CR 510.1c — the substitution is unconditional, so a 5/1 bearer
    /// assigns 1).
    AssignsCombatDamageByToughness,
    /// CR 509.1c — "This creature must be blocked if able" (Lure-style
    /// block requirement, also Academic Dispute's rider). Enforced in
    /// `declare_blockers`: if an attacker carrying this keyword is left
    /// unblocked while the defending player controls an idle creature
    /// that could legally block it, the declaration is rejected. The
    /// engine models the single-requirement case; multi-Lure
    /// maximization (CR 509.1c's "as many as possible") is approximated.
    MustBeBlocked,
    /// CR 509.1c — "All creatures able to block this creature do so" (true
    /// Lure). Stronger than `MustBeBlocked` (≥1 blocker): every idle
    /// defender creature that *can* legally block this attacker must be
    /// assigned to it. Enforced in `declare_blockers`.
    AllMustBlock,
    /// CR 509.1c — "This creature blocks each combat if able." A creature
    /// carrying this keyword that can legally block at least one declared
    /// attacker must be assigned to block one of them. Enforced in
    /// `declare_blockers` (blocker side; mirror of `MustAttack`).
    MustBlock,
    /// CR 510.1a — "This creature assigns its combat damage as though it
    /// weren't blocked." A blocked attacker with this keyword assigns its
    /// damage to the defending player instead of its blockers (the blockers
    /// still deal theirs). Predatory Focus grants it for the turn.
    AssignsDamageAsThoughUnblocked,
    /// CR 508.1d — "This creature attacks each combat if able." Enforced in
    /// `declare_attackers`: an untapped, non-sick creature carrying this
    /// keyword whose controller declares attackers must be among them when
    /// it has a legal attack (any opponent in range). Models the
    /// single-creature case; the "if able" maximization across multiple
    /// requirements (Goad targeting rules, can't-attack overrides) is
    /// approximated by the per-creature gate. Juggernaut, goaded creatures.
    MustAttack,
    /// "This creature can't attack unless the defending player controls a
    /// [filter]." Enforced in `declare_attackers` against the attack's
    /// defending player (the `SelectionRequirement` is matched among that
    /// player's controlled permanents). Dandân ("can't attack unless
    /// defending player controls an Island").
    CanAttackOnlyIfDefenderControls(Box<SelectionRequirement>),
    /// "This creature can't attack unless you control a [filter]." Mirror of
    /// `CanAttackOnlyIfDefenderControls`, enforced against the attacking
    /// creature's own controller. Lovestruck Beast ("can't attack unless you
    /// control a 1/1 creature").
    CanAttackOnlyIfYouControl(Box<SelectionRequirement>),
    /// "This creature can't attack or block unless you control [N] or more
    /// [filter]." Count-based variant of `CanAttackOnlyIfYouControl`, enforced
    /// on both the attack and block side against the bearer's controller.
    /// Topiary Stomper ("…unless you control seven or more lands"). When
    /// `attack_only` is set the gate applies to attacking only, not blocking
    /// (Lambholt Pacifist — "can't attack unless you control a creature with
    /// power 4 or greater"); `block_only` is the mirror — the gate applies to
    /// blocking only (Olog-hai Crusher — "can't block unless you control a
    /// Goblin or Orc"). `exclude_self` drops the gated creature from the count
    /// so "another [filter]" reads correctly (Tiger-Dillo — "…another creature
    /// with power 4 or greater").
    CantAttackOrBlockUnlessYouControlCount {
        filter: Box<SelectionRequirement>,
        min: u32,
        #[serde(default)]
        attack_only: bool,
        #[serde(default)]
        block_only: bool,
        #[serde(default)]
        exclude_self: bool,
    },
    /// CR 702.166 — Offspring [cost]. An optional additional cast cost; if
    /// paid, the creature's ETB mints a 1/1 token copy of it. Reuses the
    /// Kicker pipeline (`has_kicker` returns this cost, `SpellWasKicked` gates
    /// the ETB token-copy). Thundertrap Trainer.
    Offspring(crate::mana::ManaCost),
    /// CR 702.157 — Squad [cost]. An optional additional cast cost payable any
    /// number of times; the creature's ETB mints one token copy of itself per
    /// payment (`Value::SquadCount` reads `CardInstance.squad_count`). Cast via
    /// `GameAction::CastSpellSquad`.
    Squad(crate::mana::ManaCost),
    /// CR 702.107 — Replicate [cost]. An optional additional cast cost on an
    /// instant/sorcery payable any number of times; when cast, the spell is
    /// copied once per payment (copies may choose new targets). Cast via
    /// `GameAction::CastSpellReplicate`.
    Replicate(crate::mana::ManaCost),
    /// Replicate whose additional cost is paid in energy rather than mana:
    /// "Replicate—Pay {E}×N" (Reiterating Bolt). Same copy-per-payment rule as
    /// `Replicate`; the cast path charges `N` energy per replication.
    ReplicateEnergy(u32),
    /// CR 702.78 — Conspire. An optional additional cast cost on an
    /// instant/sorcery: as you cast it, you may tap two untapped creatures you
    /// control that each share a color with the spell. Doing so copies the
    /// spell once (the copy may choose new targets). Cast via
    /// `GameAction::CastSpellConspire`.
    Conspire,
    /// "This creature can't attack or block unless it has an even number of
    /// counters on it." (Zero is even.) Enforced in `declare_attackers` and
    /// `declare_blockers` (and the bot/legal-attacker gates) by reading the
    /// total counter count on the creature. Sab-Sunen, Luxa Embodied.
    CantAttackOrBlockUnlessEvenCounters,
    /// CR 702.49 — Ninjutsu [cost]. A special action usable during the
    /// declare-blockers step: pay the cost and return an unblocked attacker
    /// you control to hand, then put this card from your hand onto the
    /// battlefield tapped and attacking the same defender. Handled by
    /// `GameState::ninjutsu`. Fallen Shinobi.
    Ninjutsu(crate::mana::ManaCost),
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
    /// CR 702.171 — Saddle N. "Tap any number of other untapped creatures you
    /// control with total power N or greater: This permanent becomes saddled
    /// until end of turn. Activate only as a sorcery." Activated via
    /// `GameAction::Saddle`; the value is the required total power.
    Saddle(u32),
    /// CR 702.153 — Casualty N. "As an additional cost to cast this spell, you
    /// may sacrifice a creature with power N or greater. When you cast this
    /// spell, if a casualty cost was paid, copy it." Opt-in via
    /// `GameAction::CastSpellCasualty`; the value is the minimum power.
    Casualty(u32),
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
    /// CR 702.187 — Mayhem `cost`. A static ability that works while the card
    /// is in a graveyard: "As long as you discarded this card this turn, you
    /// may cast it from your graveyard by paying [cost] rather than its mana
    /// cost." Routed through the flashback machinery (`cast_flashback`), gated
    /// on `Player.discarded_this_turn`; the spell is exiled if it would leave
    /// the stack (same exile-after tail as flashback).
    Mayhem(crate::mana::ManaCost),
    /// CR 702.180 — Harmonize `cost`. A static ability that works while the
    /// card is in a graveyard: "You may cast this card from your graveyard by
    /// paying [cost] and tapping up to one untapped creature you control rather
    /// than paying its mana cost; the total cost is reduced by generic mana
    /// equal to the tapped creature's power." Routed through the flashback
    /// machinery (`cast_harmonize`); exiled if it would leave the stack.
    Harmonize(crate::mana::ManaCost),
    /// CR 509.1b — "This creature can't be blocked except by [filter]"
    /// (Signal Pest — "except by artifact creatures and/or creatures with
    /// flying"; Silhana Ledgewalker — "except by creatures with flying").
    /// A blocker may be assigned to this attacker only if it matches the
    /// filter. Enforced in `can_block_attacker_computed` against the
    /// blocker's *computed* characteristics.
    CantBeBlockedExceptBy(Box<SelectionRequirement>),
    /// CR 509.1b — "This creature can't be blocked by [filter]"
    /// (Steel Leaf Champion — "can't be blocked by creatures with power 2 or
    /// less"). A blocker matching the filter may not be assigned to this
    /// attacker. Enforced in `can_block_attacker_computed`.
    CantBeBlockedBy(Box<SelectionRequirement>),
    /// CR 509.1g — "This creature can't be blocked by more than one creature"
    /// (Charging Rhino, Spectral Force). At most one blocker may be assigned
    /// to it. Enforced in `declare_blockers` (the inverse of Menace).
    CantBeBlockedByMoreThanOne,
    /// "This creature can't be blocked except by N or more creatures"
    /// (Pathrazer of Ulamog — N=3). Generalizes Menace (the N=2 case): the
    /// attacker must be blocked by `0` or `>= N` creatures. Enforced in
    /// `declare_blockers`.
    CantBeBlockedExceptByN(u32),
    /// "Creatures with power less than this creature's power can't block it"
    /// (Formation Breaker). The inverse of Skulk — gates on the *blocker's*
    /// computed power being below the attacker's. Enforced in
    /// `can_block_attacker_computed` (CR 509.1b).
    CantBeBlockedByPowerLess,
    /// "Can't be blocked by creatures with power N or less" (Questing Beast —
    /// N=2). Fixed-threshold cousin of `CantBeBlockedByPowerLess`; gates on the
    /// blocker's computed power being `<= N`. Enforced in
    /// `can_block_attacker_computed`.
    CantBeBlockedByPowerAtMost(u32),
    /// CR 509.1b — "can't be blocked by [creature type]" (Juggernaut, Otarian
    /// Juggernaut: Walls). Reads the blocker's *computed* creature types, so a
    /// changeling blocker still counts. Enforced in
    /// `can_block_attacker_computed`.
    CantBeBlockedByCreatureType(CreatureType),
    /// "Can't be blocked by creatures with power N or greater" (Squeak By —
    /// N=3). Fixed-threshold mirror of `CantBeBlockedByPowerAtMost`; gates on
    /// the blocker's computed power being `>= N`. Enforced in
    /// `can_block_attacker_computed`.
    CantBeBlockedByPowerAtLeast(u32),
    /// "This creature can't be blocked if you've cast N or more spells this
    /// turn" (Illvoi Infiltrator — N=2). Game-state-dependent, so it's enforced
    /// in the stateful block-declaration path (`declare_blockers`) rather than
    /// the pure two-creature `can_block_attacker_computed`. CR 509.1b.
    CantBeBlockedIfControllerCastSpells(u32),
    /// CR 509.1b — "can't be blocked as long as defending player controls a
    /// [filter]" (Neurok Spy). Enforced in `declare_blockers` against the
    /// defender's board.
    CantBeBlockedIfDefenderControls(Box<SelectionRequirement>),
    /// CR 509.1b — "can't be blocked unless defending player controls N or
    /// more creatures that share a creature type" (Graxiplon).
    CantBeBlockedUnlessDefenderSharedType(u32),
    /// CR 702.6c — "This creature can't be equipped" (Goblin Brawler). The
    /// equip ability may not target it.
    CantBeEquipped,
    /// "Creatures with power less than the number of [filter] you control
    /// can't block this creature" (Kraken of the Straits). The threshold is
    /// the attacker controller's count of matching permanents, so it's
    /// enforced in the stateful `declare_blockers` path (CR 509.1b).
    CantBeBlockedByPowerLessThanCount(Box<SelectionRequirement>),
    /// CR 509.1b — "This creature can't be blocked unless all creatures
    /// defending player controls block it" (Tromokratis). Enforced against
    /// the finished block assignment in `declare_blockers`.
    CantBeBlockedUnlessAllBlock,
    /// "This creature has hexproof unless it's attacking or blocking"
    /// (Tromokratis). Read wherever hexproof is consulted; the grant lapses
    /// while the bearer is an attacking or blocking creature.
    HexproofUnlessAttackingOrBlocking,
    /// "This creature can block only creatures with flying." A blocker-side
    /// restriction (the inverse of the others here): when set, the bearer
    /// can't be declared as a blocker for an attacker that lacks Flying.
    /// Enforced in `can_block_attacker_computed` (Wanderlight Spirit).
    CanBlockOnlyFlying,
    /// CR 702.95 — Soulbond. A marker keyword; when this or another creature
    /// enters while either is unpaired, its controller may pair them. The
    /// pairing rides `CardInstance.soulbond_partner`, and the bonus each
    /// paired creature gains is carried in `CardDefinition.soulbond_bonus`.
    /// Deadeye Navigator, Wolfir Silverheart.
    Soulbond,
    /// CR 702.179 — Start your engines! A marker keyword: when a permanent
    /// with it enters, if its controller has no speed their speed becomes 1.
    /// Speed then increases once per their turn (capped at 4) when an opponent
    /// loses life. "Max speed —" abilities (modeled as `Predicate::SpeedAtLeast
    /// { speed: 4 }`) are active at speed 4.
    StartYourEngines,
}

/// CR 702.24 — the maintenance cost paid (once per age counter) to keep a
/// permanent with cumulative upkeep. Mana (Mystic Remora), life (Glacial
/// Chasm), or sacrificing a matching permanent (Phyrexian Soulgorger).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CumulativeUpkeepCost {
    Mana(crate::mana::ManaCost),
    Life(u32),
    Sacrifice(Box<SelectionRequirement>),
    /// "Cumulative upkeep—Flip a coin." Always payable; each flip fires the
    /// controller's win/lose-a-flip triggers (Karplusan Minotaur).
    FlipCoin,
}

impl CumulativeUpkeepCost {
    /// Short human-readable cost phrase for tooltips/logs ("{2}", "Pay 1
    /// life", "Sacrifice").
    pub fn summary(&self) -> String {
        match self {
            CumulativeUpkeepCost::Mana(c) => c.summary(),
            CumulativeUpkeepCost::Life(n) => format!("Pay {n} life"),
            CumulativeUpkeepCost::Sacrifice(_) => "Sacrifice".into(),
            CumulativeUpkeepCost::FlipCoin => "Flip a coin".into(),
        }
    }
}

/// CR 702.87 — one "LEVEL N-M / P/T / abilities" band of a leveler card.
/// While the creature's level-counter count falls in `[min, max]` (`max`
/// `None` = open-ended "N+"), the band's P/T replaces the printed base
/// (layer 7a CDA) and its keywords are granted (layer 6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelBand {
    pub min: u32,
    pub max: Option<u32>,
    pub power: i32,
    pub toughness: i32,
    #[serde(default)]
    pub keywords: Vec<Keyword>,
}

/// CR 721.2 — one `{N+}[abilities][P/T]` striation of a station card.
/// While the permanent has `min` or more charge counters, it gains
/// `keywords` (layer 6); if `pt` is `Some`, it also becomes a creature with
/// that base power/toughness in addition to its other types (CR 721.2b,
/// layers 4 + 7a).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StationBand {
    pub min: u32,
    #[serde(default)]
    pub keywords: Vec<Keyword>,
    #[serde(default)]
    pub pt: Option<(i32, i32)>,
    /// CR 721.2a — static abilities active only while the threshold is met
    /// (e.g. Lumen-Class Frigate's `{2+}: other creatures you control get
    /// +1/+1`). Converted to layer effects in `gather_continuous_effects`.
    #[serde(default)]
    pub statics: Vec<crate::effect::StaticEffect>,
    /// CR 721.2a — triggered abilities active only while the threshold is met
    /// (e.g. Synthesizer Labship's `{2+}` combat-damage trigger). Surfaced
    /// off the source via `statics_granted_triggers_for` when charges ≥ `min`.
    #[serde(default)]
    pub triggers: Vec<TriggeredAbility>,
    /// CR 721.2a — activated abilities active only while the threshold is met
    /// (e.g. a Planet's `12+ | {cost}: …` band). Surfaced off the source via
    /// `granted_abilities_for` (appended after printed + instance-granted) when
    /// charges ≥ `min`, so the activation index path treats them as grants.
    #[serde(default)]
    pub activated: Vec<crate::effect::ActivatedAbility>,
}

/// Composable filter for valid targets of a spell or ability.
/// `Default` is the match-anything `Any`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionRequirement {
    #[default]
    Any,
    Player,
    /// A player who is an opponent of the effect's controller ("target
    /// opponent").
    OpponentPlayer,
    Creature,
    Artifact,
    Enchantment,
    Planeswalker,
    Permanent,
    /// A *card* whose type line includes a permanent type (creature / land /
    /// artifact / enchantment / planeswalker / battle). Unlike `Permanent`
    /// (always-true on the battlefield), this discriminates in zones that
    /// also hold instants/sorceries — graveyard/exile/milled picks.
    PermanentCard,
    Land,
    Nonland,
    Noncreature,
    Tapped,
    Untapped,
    /// CR 120.x — the permanent has been dealt damage this turn (Initiate of
    /// Blood // Goka's "target creature that was dealt damage this turn").
    DealtDamageThisTurn,
    /// The permanent was dealt damage *by the evaluating source* this turn
    /// (Bushi Tenderfoot's "a creature dealt damage by this creature this
    /// turn"). Reads the dying object's LKI snapshot.
    DamagedBySourceThisTurn,
    /// The inverse of `DamagedBySourceThisTurn` — the permanent dealt damage
    /// *to* the evaluating source this turn (Brine Hag). Reads the source's
    /// `damaged_by_this_turn`, off its death LKI when it has already left.
    DealtDamageToSourceThisTurn,
    /// The *player* target was dealt damage by the evaluating source this turn
    /// (Wicked Akuba's "target player dealt damage by this creature this turn").
    PlayerDamagedBySourceThisTurn,
    HasColor(Color),
    HasKeyword(Keyword),
    /// Has Toxic N for any N (CR 702.180). The parameter-agnostic sibling of
    /// `HasKeyword(Toxic(n))` — matches "a creature you control with toxic"
    /// regardless of the toxic value (Slaughter Singer, Skrelv's Hive).
    HasToxic,
    /// Has Modular N for any N (CR 702.43) — "each creature you control with
    /// modular" (Arcbound Overseer).
    HasModular,
    /// CR 702.140 — the card has a mutate cost (Pollywog Symbiote's
    /// "creature spell you cast … if it has mutate").
    HasMutate,
    /// The card has any cycling ability (Cycling / CyclingLife /
    /// Landcycling — CR 702.29). Zenith Flare's graveyard count.
    HasCyclingAbility,
    /// Has a Morph / Megamorph / Disguise ability (Backslide).
    HasMorphAbility,
    /// The card has Disturb (CR 702.146), regardless of its cost (Shipwreck
    /// Sifters' "Spirit card or a card with disturb" discard payoff).
    HasDisturb,
    /// The card has flashback (CR 702.34) in any of its cost shapes
    /// (Flashback / FlashbackTap). Tombfire's graveyard sweep.
    HasFlashback,
    /// The card shares a card type with a card exiled by the evaluating
    /// source (`exiled_with == source`) — Holistic Wisdom's return gate.
    SharesCardTypeWithExiledBySource,
    PowerAtMost(i32),
    /// Power no greater than the *source's* current power (Old Man of the Sea's
    /// "creature with power less than or equal to this creature's power").
    PowerAtMostSourcePower,
    ToughnessAtMost(i32),
    /// "Toughness X or less, where X is the number of [filter] you control"
    /// (Scourge of Fleets' Islands). The count is taken from the evaluating
    /// player's battlefield.
    ToughnessAtMostYourCount(Box<SelectionRequirement>),
    /// "Toughness less than or equal to the number of cards in your graveyard"
    /// (Ghastly Demise). Counted from the evaluating player's graveyard.
    ToughnessAtMostGraveyardCount,
    /// "Power less than the number of cards in your graveyard" (Temporary
    /// Insanity). Counted from the evaluating player's graveyard.
    PowerLessThanYourGraveyardCount,
    /// "Power X or less, where X is the number of [filter] you control"
    /// (Vedalken Shackles' Islands) — the power-side mirror of
    /// `ToughnessAtMostYourCount`.
    PowerAtMostYourCount(Box<SelectionRequirement>),
    WithCounter(CounterType),
    /// "…with N or more [kind] counters on it" (Bomb Squad's fuse check).
    WithCounterAtLeast(CounterType, u32),
    /// True when the candidate has at least one counter of any kind on it —
    /// "a creature with a counter on it" (Delta Bloodflies, Stalwart
    /// Successor). The any-kind complement of `WithCounter`.
    WithAnyCounter,
    /// True when the candidate has no counters of any kind on it (CR 122).
    /// Powers "target creature with no counters on it" (Heartless Act mode 0).
    HasNoCounters,
    /// True when the candidate's name differs from every card moved so far
    /// in the current resolution — "with different names" multi-search
    /// clauses (Saheeli Rai's -7).
    NameDiffersFromLastMoved,
    ControlledByYou,
    ControlledByOpponent,
    /// The object is controlled by the player bound as the firing event's
    /// subject — "target creature *that player* controls" on a triggered
    /// ability (Satyr Firedancer). Reads
    /// `GameState.trigger_event_player_scratch`; false outside a trigger.
    ControlledByTriggerPlayer,
    /// CR 108.3 — the object's owner is you (regardless of who controls it).
    /// Gruul Charm's "gain control of all permanents you own".
    OwnedByYou,
    HasSupertype(Supertype),
    HasCreatureType(CreatureType),
    /// CR 700.12 — the object is an **outlaw**: a creature that is an Assassin,
    /// Mercenary, Pirate, Rogue, or Warlock. Convenience filter for the
    /// five outlaw creature types (Vial Smasher, Rakish Crew, Hellspur Brute).
    IsOutlaw,
    HasLandType(LandType),
    HasArtifactSubtype(ArtifactSubtype),
    HasEnchantmentSubtype(EnchantmentSubtype),
    HasPlaneswalkerType(PlaneswalkerSubtype),
    /// Spell subtype on an instant/sorcery (Arcane — Kamigawa spiritcraft
    /// triggers gate on `Spirit creature OR Arcane`).
    HasSpellSubtype(SpellSubtype),
    PowerAtLeast(i32),
    ToughnessAtLeast(i32),
    /// Creature whose toughness is strictly greater than its power (Catapult
    /// Fodder's "creatures that each have toughness greater than their power").
    /// Battlefield-only; false for non-creatures.
    ToughnessGreaterThanPower,
    /// Candidate's current power exceeds its printed base power — i.e. it has
    /// been pumped (counters / bonuses). Kutzil, Malamet Exemplar; Sovereign
    /// Okinec Ahau. Battlefield-only; false for non-creatures.
    PowerGreaterThanBasePower,
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
    /// True when the candidate creature has strictly greater power than the
    /// source permanent (both read at evaluation time). Mirror of
    /// `PowerLessThanSource`; powers Training (CR 702.149 — "if it's
    /// attacking with another creature with greater power"). Battlefield-only.
    PowerGreaterThanSource,
    /// True when the candidate creature has greater power *or* greater
    /// toughness than the source permanent (both read at evaluation
    /// time). Powers Evolve (CR 702.100 — "if that creature has greater
    /// power or toughness than this creature"). Battlefield-only; false
    /// when the source is missing or either side isn't a creature.
    GreaterPowerOrToughnessThanSource,
    IsToken,
    NotToken,
    /// CR 702.185 — the permanent was cast for its warp cost (its `warped` flag
    /// is set). Backs Full Bore's "if it was cast for its warp cost" rider.
    Warped,
    /// The permanent's mana value ties the lowest among all nonland permanents
    /// on the battlefield (Culling Scales' "with the lowest mana value").
    LowestManaValueAmongNonland,
    /// Has the same name as the card exiled with the ability's source
    /// (Extraplanar Lens' imprint).
    SameNameAsExiledWithSource,
    /// CR 107.4f — the card's mana cost contains at least one Phyrexian mana
    /// symbol ("a spell with {H} in its mana cost" — Rage Extractor).
    HasPhyrexianManaInCost,
    /// The candidate shares a name with some permanent on the battlefield
    /// (Mitotic Manipulation). A permanent matches itself.
    SameNameAsAPermanent,
    /// The candidate shares a name with the effect's first target (Pack Hunt).
    SameNameAsTarget,
    /// Shares a colour with the card exiled with the ability's source
    /// (Mourner's Shield's imprint).
    SharesColorWithExiledBySource,
    /// Shares a colour with the permanent the ability's source is attached to
    /// (Konda's Banner). False when the source is unattached.
    SharesColorWithAttachedHost,
    /// Shares a creature type with the permanent the ability's source is
    /// attached to (Konda's Banner). Changeling hosts share with everything.
    SharesCreatureTypeWithAttachedHost,
    /// Shares a colour with any permanent the evaluating player controls —
    /// Empty-Shrine Kannushi's "protection from the colors of permanents you
    /// control".
    SharesColorWithPermanentYouControl,
    /// Shares a colour with the permanent sacrificed to pay this spell's
    /// additional cost (Mind Extraction). False with nothing sacrificed.
    SharesColorWithSacrificed,
    /// Shares a creature type with *every* permanent tapped to pay the
    /// resolving ability's `tap_n_filter` cost (Cryptic Gateway). Vacuously
    /// false when nothing was tapped this way.
    SharesCreatureTypeWithTapped,
    /// Shares a creature type with the permanent sacrificed to pay this
    /// spell's additional cost (Endemic Plague). Changelings match.
    SharesCreatureTypeWithSacrificed,
    /// Is of a creature type chosen by an `EachPlayerChoosesCreatureTypeThen`
    /// resolution (Harsh Mercy, Patriarch's Bidding). Changelings match.
    IsTypeChosenThisWay,
    /// Shares a colour with the mana spent on this activation's cost
    /// (Protective Sphere). Concretized to an `Or` chain of `HasColor` at
    /// resolution; colourless mana matches nothing.
    SharesColorWithManaSpent,
    /// CR 601.2c — cross-slot filter: controlled by the same player as the
    /// target already chosen for slot `.0` ("two target creatures controlled
    /// by the same player" — Barrin's Spite). Vacuously true while that slot
    /// is unchosen, so it never blocks the first pick.
    SameControllerAsTargetSlot(u8),
    /// The permanent's mana value equals the evaluating player's unspent
    /// (floating) mana — Glissa Sunseeker.
    ManaValueEqualsYourUnspentMana,
    IsBasicLand,
    /// A player who attacked this turn (Fire and Brimstone). Only meaningful
    /// against a player target.
    PlayerAttackedThisTurn,
    /// The candidate is (one of) the colour the source permanent chose as it
    /// entered (`CardInstance.chosen_color` — Story Circle's Circle-of-
    /// Protection shield). False when the source never chose one.
    HasChosenColorOfSource,
    /// CR 702.113 — the card has an Awaken alternative cost (Halimar Tidecaller).
    HasAwaken,
    /// True for a land that is **not** basic (CR 305.6) — i.e. a land card
    /// lacking the Basic supertype. Powers Thalia, Heretic Cathar's
    /// "nonbasic lands … enter the battlefield tapped" clause and any
    /// nonbasic-land targeting filter.
    IsNonbasicLand,
    /// A permanent with a mana ability that could produce {C} (an
    /// `Effect::AddMana` whose payload adds colorless). "Land that … could
    /// produce {C}" — Break the Ice, Obsidian Charmaw.
    ProducesColorless,
    /// A snow permanent/card (CR 205.4g — the Snow supertype). Break the Ice.
    IsSnow,
    IsAttacking,
    /// An attacking creature that hasn't been blocked (CR 509.1h). Reads live
    /// combat state — Sneak's "return an unblocked creature you control".
    IsUnblocked,
    /// An attacking creature that has become blocked (CR 509.1h). Reads live
    /// combat state — Smite's "destroy target blocked creature".
    IsBlocked,
    IsBlocking,
    /// CR 509 — the permanent is blocking the evaluating source, or is
    /// blocked by it (Sentinel's "target creature blocking or blocked by
    /// this creature"). Reads `block_map` both ways.
    BlockingOrBlockedBySource,
    /// True when the candidate is blocking, or is blocked by, the ability's
    /// source — the symmetric combat-partner filter (Sisters of Stone Death's
    /// "creature blocking or blocked by this creature").
    InCombatWithSource,
    /// True when the candidate creature was declared as an attacker at any
    /// point this turn (`CardInstance.attacked_this_turn`). Relentless
    /// Assault's "untap all creatures that attacked this turn".
    AttackedThisTurn,
    /// True when the candidate creature was declared as a blocker at any point
    /// this turn (`CardInstance.blocked_this_turn`) — for "creature that
    /// attacked or blocked this turn" filters (Gideon's Triumph).
    BlockedThisTurn,
    /// CR 708 — true when the candidate permanent is face down (a manifested /
    /// morphed / cloaked / disguised permanent showing a 2/2 vanilla). Powers
    /// DSK "face-down permanent you control" matters (Cryptid Inspector).
    FaceDown,
    /// True when the candidate permanent is the source of an activated or
    /// triggered ability currently on the stack. Targeting filter for
    /// `Effect::CounterAbility` (Stifle — CR 113.9).
    HasAbilityOnStack,
    /// CR 603.4 — true when the candidate permanent entered the battlefield
    /// this turn (its `CardInstance.entered_turn` equals the current
    /// `GameState.turn_number`). Battlefield-only; powers "each creature that
    /// entered the battlefield under your control this turn" (Shaile, Dean of
    /// Radiance). Combine with `ControlledByYou` for the controller clause.
    EnteredThisTurn,
    /// True when the candidate entered the battlefield from a graveyard —
    /// or was cast from one — this turn (Prized Amalgam's intervening-if).
    /// Reads `GameState.entered_from_graveyard_this_turn`.
    EnteredFromGraveyardThisTurn,
    /// True when the candidate entered the battlefield directly from exile
    /// this turn (not via a cast). Reads `GameState.entered_from_exile_this_turn`
    /// (Fire Lord Zuko's "whenever a permanent you control enters from exile").
    EnteredFromExileThisTurn,
    /// True when the candidate permanent has an Aura attached to it (CR 303
    /// "enchanted permanent"). Battlefield-only: scans for any enchantment
    /// whose `attached_to` points at the candidate. Powers Kestia's
    /// "whenever an enchanted creature … you control attacks" trigger.
    IsEnchanted,
    /// True for a graveyard card that got there from the battlefield this turn
    /// (`GameState.graveyard_from_battlefield_this_turn`) — Gleancrawler's
    /// "all creature cards in your graveyard that were put there from the
    /// battlefield this turn".
    PutIntoGraveyardFromBattlefieldThisTurn,
    /// True when the candidate permanent has an Equipment attached
    /// (CR 301.5 "equipped"). Battlefield-only. Kor Duelist.
    IsEquipped,
    /// CR 701.60 — true while the candidate creature is suspected (has the
    /// `suspected` flag). Powers "Sacrifice a suspected creature" costs
    /// (Rune-Brand Juggler) and suspected-creature payoffs.
    IsSuspected,
    /// CR 702.103 — true while the candidate is on the battlefield as a
    /// bestowed Aura rather than a creature ("if it's an Aura" —
    /// Everflame Eidolon).
    IsBestowed,
    /// True when the candidate is currently attached to *some* permanent
    /// (`attached_to.is_some()`). Source-precise "attached to this creature"
    /// filters (Faunsbane Troll's "Sacrifice an Aura attached to this
    /// creature") intersect this with the source id in the cost path, since a
    /// source-blind requirement can't know which permanent "this" is.
    AttachedToSource,
    /// The mirror of `AttachedToSource`: true when the candidate is the
    /// permanent the *source* is attached to. Source-blind evaluators answer
    /// `true`; the cost path intersects against the real host, which is what
    /// makes "{T}, Unattach this Equipment" costs tap the right creature
    /// (Leonin Bola, Surestrike Trident).
    IsHostOfSource,
    /// True when the candidate permanent has at least `n` Equipment attached
    /// (CR 301.5). Battlefield-only. Balan's "double strike as long as two or
    /// more Equipment are attached to it".
    EquippedByAtLeast(u32),
    /// CR 700.9 — "modified": the permanent has one or more counters, is
    /// equipped, or is enchanted by an Aura its own controller controls.
    /// Battlefield-only. Kodama of the West Tree.
    IsModified,
    /// True when the candidate creature dealt damage to the ability's
    /// controller this turn (combat or non-combat). Reads the controller's
    /// `Player.creatures_that_damaged_me_this_turn`. Battlefield-only;
    /// powers Spear of Heliod's "destroy target creature that dealt damage
    /// to you this turn".
    DealtDamageToControllerThisTurn,
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
    /// A stack spell at least one of whose chosen targets is the choosing
    /// player or a permanent they control (Hindering Light's "counter target
    /// spell that targets you or a permanent you control"). Only meaningful
    /// against a `StackItem::Spell`.
    SpellTargetsControllerOrControlled,
    /// A stack spell that targets at least one creature (Intervene).
    SpellTargetsCreature,
    /// A stack spell whose effect would destroy a land the evaluating player
    /// controls — either a mass destroy whose filter can catch one, or a
    /// targeted destroy pointed at one (Equinox).
    SpellWouldDestroyALandYouControl,
    /// A *player* who cast one or more sorcery spells this turn (Backdraft).
    CastSorceryThisTurn,
    /// A spell *or ability* on the stack that targets a land the evaluating
    /// player controls (Teferi's Response).
    TargetsALandYouControl,
    /// True when the candidate stack spell targets *only* the ability's
    /// source — one target slot, filled with the source (Ink-Treader
    /// Nephilim's "if that spell targets only this creature").
    SpellTargetsOnlySource,
    /// CR 115.7 — a stack spell with exactly one target ("target spell with a
    /// single target" — Ricochet Trap, Deflection).
    SpellWithSingleTarget,
    /// A stack spell that was NOT cast from its owner's hand (Wash Away's
    /// bracketed base mode — flashback/graveyard/exile casts qualify).
    SpellNotCastFromHand,
    ManaValueAtMost(u32),
    /// MV at most the number of battlefield permanents matching the inner
    /// filter that the evaluating player controls (Spellstutter Sprite's
    /// "mana value X or less, where X is the number of Faeries you control").
    ManaValueAtMostYourCount(Box<SelectionRequirement>),
    /// LCI fathomless descent — MV at most the number of permanent cards in the
    /// evaluating player's graveyard (Squirming Emergence's reanimation cap).
    ManaValueAtMostPermanentsInYourGraveyard,
    /// MV at most the evaluating player's devotion to a color (Grim Servant —
    /// "search for a card with mana value ≤ your devotion to black").
    ManaValueAtMostDevotion(crate::mana::Color),
    /// Mana value ≤ the mana spent to cast the source permanent's spell
    /// (`CardInstance.cast_mana_spent`). Read source-relative at filter time —
    /// Astelli Reclaimer's "with mana value X or less, where X is the amount of
    /// mana spent to cast this".
    ManaValueAtMostCastManaSpent,
    /// "…with mana value X or less, where X is the amount of life you
    /// gained this turn" (Moseo, Vein's New Dean's Infusion). Evaluated
    /// against the requirement CONTROLLER's `life_gained_this_turn`.
    ManaValueAtMostLifeGainedThisTurn,
    /// Mana value ≤ the source permanent's power, read last-known-information
    /// aware (battlefield power first, then the `leaves_bf_lki` snapshot of a
    /// dying source). Sandbender Scavengers' "return a creature with mana value
    /// ≤ this creature's power" reflexive death reanimation.
    ManaValueAtMostSourcePower,
    /// Mana value ≤ the X paid into the resolving spell's cost. Resolved to
    /// a concrete `ManaValueAtMost(x)` by `resolve_x` at search-resolution
    /// time (Chord of Calling); unresolved instances evaluate false.
    ManaValueAtMostXFromCost,
    /// Mana value == the X paid into the resolving spell/ability's cost.
    /// Resolved to a concrete `ManaValueExactly(x)` by `resolve_x` (Hearth
    /// Kami's "destroy target artifact with mana value X"); unresolved
    /// instances evaluate false.
    ManaValueExactlyXFromCost,
    /// Power ≤ the X paid into the resolving spell/ability's cost. Resolved to
    /// a concrete `PowerAtMost(x)` by `resolve_x` (Entrancing Lyre's "tap
    /// target creature with power X or less"); unresolved instances eval false.
    PowerAtMostXFromCost,
    /// Toughness ≤ the X paid into the resolving spell/ability's cost. Resolved
    /// to a concrete `ToughnessAtMost(x)` by `resolve_x` (Finale of Eternity's
    /// "destroy up to three target creatures with toughness X or less").
    ToughnessAtMostXFromCost,
    /// Mana value ≤ the resolving spell's converge count (distinct colors of
    /// mana spent — CR 702.86). Resolved to a concrete `ManaValueAtMost(n)`
    /// by `resolve_converge` at search-resolution time (Bring to Light);
    /// unresolved instances evaluate false.
    ManaValueAtMostConverged,
    ManaValueAtLeast(u32),
    /// True when the card's mana value (CMC) is exactly `n`. Useful for
    /// effects that want a precise CMC gate (Fix What's Broken returns
    /// "with mana value equal to X", which requires this exact-match
    /// shape rather than the `AtMost`/`AtLeast` approximations).
    /// Composes naturally with `And`/`Or` for range gates.
    ManaValueExactly(u32),
    /// Mana value equal to the firing event's amount — "destroy all permanents
    /// with that spell's mana value" (Celestial Kirin).
    ManaValueEqualsTriggerAmount,
    /// True when the card's mana value has the given parity — odd when
    /// `odd: true`, even (incl. 0) otherwise. Extinction Event's "exile each
    /// creature with mana value of the chosen parity".
    ManaValueParity { odd: bool },
    /// True when the card's mana value equals the number of counters of the
    /// given kind on the resolving ability's source (Aether Vial). Resolved
    /// to a concrete `ManaValueExactly(n)` by `resolve_source_counters` at
    /// effect resolution; unresolved instances evaluate false.
    ManaValueEqualsSourceCounters(CounterType),
    /// MV ≤ the mana value of the card discarded earlier in this resolution
    /// (Argentum Masticore's reflexive destroy).
    ManaValueAtMostDiscardedThisEffect,
    /// MV *equal to* the card discarded earlier in this resolution (Disciple
    /// of Deceit's tutor).
    ManaValueEqualsDiscardedThisEffect,
    /// True when the card's mana value equals the most-recently-sacrificed
    /// creature's mana value plus `offset`, read from the resolution scratch
    /// (`GameState.sacrificed_mana_value`). Powers Birthing Pod's "search for a
    /// creature card with mana value equal to 1 plus the sacrificed creature's
    /// mana value." Evaluates to `mv == 0 + offset` when nothing was recorded.
    ManaValueEqualsSacrificedPlus(u32),
    /// True when the card's mana value is at most the most-recently-sacrificed
    /// creature's mana value plus `offset` (`GameState.sacrificed_mana_value`).
    /// Powers Eldritch Evolution's "creature card with mana value X or less,
    /// where X is 2 plus the sacrificed creature's mana value."
    ManaValueAtMostSacrificedPlus(u32),
    /// True when the card's mana value is strictly less than the firing
    /// trigger event's amount — for died events, the dying card's mana value
    /// (`GameState.trigger_event_amount_scratch`). Powers Scrap Trawler's
    /// "return target artifact card with lesser mana value than that
    /// artifact". False outside a trigger context (scratch = 0).
    ManaValueLessThanEventAmount,
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
    /// Mana value equals the number chosen earlier in this resolution
    /// (`Effect::PlayerChoosesNumber` — Void's hand half).
    ManaValueEqualsChosenNumber,
    /// A land whose activated abilities include at least one that isn't a mana
    /// ability (CR 605.1a) — Tsabo's Web's untap lock.
    HasNonManaActivatedAbility,
    /// Another permanent on the battlefield shares this permanent's name
    /// (Winnow). Never matches when the object is the only copy out.
    SharesNameWithAnotherPermanent,
    /// Shares a colour with the most common colour among all permanents, or a
    /// colour tied for most common (Barrin's Unmaking, Tsabo's Assassin).
    /// Never matches a colourless object.
    SharesMostCommonColor,
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
    /// The candidate was NOT sacrificed during this resolution — the printed
    /// "another permanent card" in a sacrifice-then-recur line (Deadly Brew).
    /// Backed by `GameState::cards_sacrificed_this_resolution`.
    NotSacrificedThisResolution,
    /// The mirror of [`SelectionRequirement::OtherThanSource`] — the candidate
    /// *is* the ability's source. Lets a cost name the permanent itself
    /// ("Return this enchantment to its owner's hand:" — Attunement).
    IsSource,
    /// True when the candidate card is currently in some player's graveyard
    /// zone. Used to restrict zone-spanning trigger targets — e.g.
    /// Ascendant Dustspeaker / Lorehold Acolyte's "exile up to one target
    /// card from a graveyard", which must NOT be matchable by a
    /// battlefield permanent. The companion `evaluate_requirement_static`
    /// arm looks the candidate up in `players[*].graveyard`. Returns
    /// false for non-Permanent targets and for cards in any other zone.
    InGraveyard,
    /// True when the candidate card is in the evaluating player's own
    /// graveyard (CR 702.47a "your graveyard" — Soulshift). Stricter than
    /// `InGraveyard`, which matches any player's graveyard.
    InYourGraveyard,
    /// True when the candidate card is in an opponent's graveyard ("target
    /// card from an opponent's graveyard" — Nezumi Graverobber). Complement
    /// of `InYourGraveyard`.
    InOpponentGraveyard,
    /// Descend N (CR 701.x — LCI) — true when the candidate's controller has
    /// N or more permanent cards in their graveyard. Used as the `condition`
    /// of a `SelfHasKeywordWhile` for "Descend 8 — this can't be blocked as
    /// long as there are eight or more permanent cards in your graveyard"
    /// (Watertight Gondola), and as a generic gate for Descend payoffs.
    ControllerDescend(u32),
    /// CR 702.166 Corrupted, as a *target* filter — true when the candidate
    /// permanent's controller has three or more poison counters. Widens
    /// Anoint with Affliction's exile to any creature whose controller is
    /// corrupted.
    ControllerCorrupted,
    /// True when the candidate shares its name with a card in its controller's
    /// graveyard ("a spell that has the same name as a card in your graveyard"
    /// — Pyromancer Ascension). The candidate itself (typically a spell on the
    /// stack, not yet in the graveyard) is not counted.
    SharesNameWithControllerGraveyardCard,
    /// True when the candidate's controller has drawn N or more cards this turn
    /// (CR 121 — `Player.cards_drawn_this_turn`). Used as the `condition` of a
    /// `SelfHasKeywordWhile` for "as long as you've drawn two or more cards this
    /// turn" grants (Foggy Swamp Hunters' lifelink/menace, June's unblockable).
    ControllerDrewAtLeastThisTurn(u32),
    /// True when the candidate's controller has sacrificed an artifact this turn
    /// (`Player.artifacts_sacrificed_this_turn > 0`). Used as the `condition` of
    /// a `SelfHasKeywordWhile` for "can't be blocked as long as you've sacrificed
    /// an artifact this turn" (Furtive Courier).
    ControllerSacrificedArtifactThisTurn,
    /// True when it's the candidate's controller's turn (`active_player_idx ==
    /// controller`). Used as the `condition` of a `SelfHasKeywordWhile` for
    /// "during your turn, this creature has [keyword]" (Pompous Gadabout).
    ControllersTurn,
    /// True when the candidate card is in the exile zone. Mirrors
    /// `InGraveyard`; used by impulse "if you don't cast it" fallbacks
    /// (Chandra, Torch of Defiance) to detect an uncast exiled card.
    InExile,
    /// True when the candidate is an exile-zone card stamped
    /// `exiled_with == source` — "target creature card exiled with [this]"
    /// (The Darkness Crystal's recursion).
    ExiledWithSource,
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
    /// The power sibling of `HasGreatestManaValueAmongControlled`:
    /// "a creature with the greatest power among [filter] that player
    /// controls" — the candidate must (a) match `inner` and (b) have
    /// power ≥ every other matching permanent under the same
    /// controller. Ties pass permissively; battlefield-only. Professor
    /// Onyx's −3 ("each opponent sacrifices a creature with the
    /// greatest power among creatures that player controls").
    HasGreatestPowerAmongControlled(Box<SelectionRequirement>),
    /// "A creature with the greatest power among all creatures on the
    /// battlefield" — the candidate must be a creature whose power ≥ every
    /// other creature's (any controller). Ties pass. Gates Getaway Glamer's
    /// "destroy target creature if no other creature has greater power."
    HasGreatestPowerAmongAllCreatures,
    /// True when the candidate's `definition.name` exactly matches.
    /// Used by Grandeur-style activations that require discarding
    /// another card with the source's printed name (Page, Loose Leaf).
    /// Evaluated against the candidate's printed name only. Stored as
    /// `String` so the predicate round-trips through serde without a
    /// `static_str_serde` adapter — the catalog passes a one-time
    /// `.to_string()` at definition time, negligible overhead.
    HasName(String),
    /// True when the candidate's printed name was originally printed in the
    /// named expansion (CR 201.3 — Golgothian Sylex's "a name originally
    /// printed in the Antiquities expansion"). Matched against a static name
    /// table, so reprints under the same name still count.
    OriginallyPrintedIn(OriginalSet),
    /// True when the candidate's name matches the name the resolving
    /// source stamped via `Effect::NameCard` (its `named_card` field).
    /// Used by reveal-until-the-named-card effects (Spoils of the Vault):
    /// pair `NameCard { This }` with a `RevealUntilFind { find:
    /// NamedBySource, .. }`. Falls back to "no match" when the source has
    /// not named a card.
    NamedBySource,
    /// True when the candidate's card types include the type the resolving
    /// source chose via `Effect::ChooseCardTypeForSource` (World Queller's
    /// "each player sacrifices a permanent of their choice of that type").
    /// Falls back to "no match" when the source hasn't chosen one.
    IsSourceChosenCardType,
    /// True when the candidate shares the creature type the resolving source
    /// chose via `Effect::NameCreatureType` (Belbe's Portal). Changeling
    /// satisfies any type; no match when nothing was chosen.
    IsSourceChosenCreatureType,
    /// True when the candidate's mana value is ≤ the number of permanents
    /// the evaluating player controls that match the inner filter. Powers
    /// "with mana value less than or equal to the number of [X] you
    /// control" gates — Lay Down Arms ("…number of Plains you control").
    /// Battlefield/zone candidate; the count walks the battlefield for
    /// permanents matching `inner` under the evaluating controller.
    ManaValueAtMostControlledCount(Box<SelectionRequirement>),
    /// "with mana value less than or equal to the number of cards in your
    /// hand" (Rending Vines). Counts the *evaluating controller's* hand, not
    /// the candidate's owner's.
    ManaValueAtMostControllerHand,
    /// True when the candidate's mana value is ≤ the number of cards in its
    /// own controller's graveyard. Powers Drown in the Loch's "counter/
    /// destroy target with mana value ≤ cards in its controller's graveyard"
    /// gate. Works on stack spells and battlefield permanents alike (both
    /// carry a `controller`).
    ManaValueAtMostControllerGraveyard,
    /// True when the candidate has a back-face `CardDefinition` —
    /// i.e. it's a double-faced card (MDFC).
    HasBackFace,
    /// True when the candidate is a preparation card — a creature
    /// carrying an inset prepare spell (`prepare_spell.is_some()`).
    /// Enforces the SOS Prepare reminder "(Only creatures with prepare
    /// spells can become prepared.)" on Biblioplex Tomekeeper /
    /// Skycoach Waypoint's target filters.
    HasPrepareSpell,
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

    /// Rewrite every `IsSource` atom to a constant. Used when a filter is
    /// evaluated against a card that has already left the battlefield, where
    /// the "is this the granting source?" answer is known up front but the
    /// card-only evaluator can't look it up (CR 603.10a death LKI).
    pub fn resolve_is_source(&self, is_source: bool) -> Self {
        match self {
            Self::IsSource => {
                if is_source { Self::Any } else { Self::Any.negate() }
            }
            Self::And(a, b) => a.resolve_is_source(is_source).and(b.resolve_is_source(is_source)),
            Self::Or(a, b) => a.resolve_is_source(is_source).or(b.resolve_is_source(is_source)),
            Self::Not(a) => a.resolve_is_source(is_source).negate(),
            other => other.clone(),
        }
    }

    /// Concretize `NamedBySource` against the source's chosen card name
    /// (`HasName`, or "matches nothing" when nothing was named), recursing
    /// through And/Or/Not. CR 702.106 hidden agenda.
    pub fn resolve_named_by_source(&self, name: Option<&str>) -> Self {
        match self {
            Self::NamedBySource => match name {
                Some(n) => Self::HasName(n.to_string()),
                None => Self::Not(Box::new(Self::Any)),
            },
            Self::And(a, b) => Self::And(
                Box::new(a.resolve_named_by_source(name)),
                Box::new(b.resolve_named_by_source(name)),
            ),
            Self::Or(a, b) => Self::Or(
                Box::new(a.resolve_named_by_source(name)),
                Box::new(b.resolve_named_by_source(name)),
            ),
            Self::Not(inner) => Self::Not(Box::new(inner.resolve_named_by_source(name))),
            other => other.clone(),
        }
    }

    /// Concretize `SameNameAsTarget` against the effect's target name
    /// (`HasName`, or "matches nothing" with no target), recursing through
    /// And/Or/Not. Pack Hunt.
    pub fn resolve_target_name(&self, name: Option<&str>) -> Self {
        match self {
            Self::SameNameAsTarget => match name {
                Some(n) => Self::HasName(n.to_string()),
                None => Self::Not(Box::new(Self::Any)),
            },
            Self::And(a, b) => Self::And(
                Box::new(a.resolve_target_name(name)),
                Box::new(b.resolve_target_name(name)),
            ),
            Self::Or(a, b) => {
                Self::Or(Box::new(a.resolve_target_name(name)), Box::new(b.resolve_target_name(name)))
            }
            Self::Not(inner) => Self::Not(Box::new(inner.resolve_target_name(name))),
            other => other.clone(),
        }
    }

    /// Concretize `IsSourceChosenCreatureType` against the source's stamped
    /// choice (`HasCreatureType(ct)`, or "matches nothing" when unchosen),
    /// recursing through And/Or/Not. Belbe's Portal.
    pub fn resolve_chosen_creature_type(&self, chosen: Option<CreatureType>) -> Self {
        match self {
            Self::IsSourceChosenCreatureType => match chosen {
                Some(ct) => Self::HasCreatureType(ct),
                None => Self::Not(Box::new(Self::Any)),
            },
            Self::And(a, b) => Self::And(
                Box::new(a.resolve_chosen_creature_type(chosen)),
                Box::new(b.resolve_chosen_creature_type(chosen)),
            ),
            Self::Or(a, b) => Self::Or(
                Box::new(a.resolve_chosen_creature_type(chosen)),
                Box::new(b.resolve_chosen_creature_type(chosen)),
            ),
            Self::Not(inner) => Self::Not(Box::new(inner.resolve_chosen_creature_type(chosen))),
            other => other.clone(),
        }
    }

    /// Replace X-dependent atoms with concrete values
    /// (`ManaValueAtMostXFromCost` → `ManaValueAtMost(x)`), recursing
    /// through And/Or/Not. Called by `Effect::Search` with the resolving
    /// spell's paid X (Chord of Calling, CR 601.2b).
    pub fn resolve_x(&self, x: u32) -> Self {
        match self {
            Self::ManaValueAtMostXFromCost => Self::ManaValueAtMost(x),
            Self::ManaValueExactlyXFromCost => Self::ManaValueExactly(x),
            Self::PowerAtMostXFromCost => Self::PowerAtMost(x as i32),
            Self::ToughnessAtMostXFromCost => Self::ToughnessAtMost(x as i32),
            Self::And(a, b) => Self::And(Box::new(a.resolve_x(x)), Box::new(b.resolve_x(x))),
            Self::Or(a, b) => Self::Or(Box::new(a.resolve_x(x)), Box::new(b.resolve_x(x))),
            Self::Not(inner) => Self::Not(Box::new(inner.resolve_x(x))),
            other => other.clone(),
        }
    }

    /// Concretize `ManaValueAtMostSourcePower` against the source's power
    /// (Velomachus Lorehold's "with mana value less than or equal to this
    /// creature's power"), so source-less card matchers (library walks)
    /// evaluate a plain `ManaValueAtMost`. Same shape as `resolve_x`.
    pub fn resolve_source_power(&self, power: i32) -> Self {
        match self {
            Self::ManaValueAtMostSourcePower => Self::ManaValueAtMost(power.max(0) as u32),
            Self::And(a, b) => Self::And(
                Box::new(a.resolve_source_power(power)),
                Box::new(b.resolve_source_power(power)),
            ),
            Self::Or(a, b) => Self::Or(
                Box::new(a.resolve_source_power(power)),
                Box::new(b.resolve_source_power(power)),
            ),
            Self::Not(inner) => Self::Not(Box::new(inner.resolve_source_power(power))),
            other => other.clone(),
        }
    }

    /// True when the filter requires the candidate to be a *card in an
    /// off-board zone* (graveyard / exile) rather than a battlefield
    /// permanent or player. Such targets have no clickable board entity,
    /// so UIs must gather the pick through a card-list modal instead of
    /// the in-scene targeting cursor. Recurses through And/Or (a `Not`
    /// of a zone requirement doesn't *demand* the zone, so it doesn't
    /// count).
    pub fn mentions_offboard_zone(&self) -> bool {
        match self {
            Self::InGraveyard
            | Self::InYourGraveyard
            | Self::InOpponentGraveyard
            | Self::InExile => true,
            Self::And(a, b) | Self::Or(a, b) => {
                a.mentions_offboard_zone() || b.mentions_offboard_zone()
            }
            _ => false,
        }
    }

    /// Replace `ManaValueAtMostConverged` with a concrete
    /// `ManaValueAtMost(n)` for the resolving spell's converge count
    /// (Bring to Light), recursing through And/Or/Not.
    pub fn resolve_converge(&self, n: u32) -> Self {
        match self {
            Self::ManaValueAtMostConverged => Self::ManaValueAtMost(n),
            Self::And(a, b) => {
                Self::And(Box::new(a.resolve_converge(n)), Box::new(b.resolve_converge(n)))
            }
            Self::Or(a, b) => {
                Self::Or(Box::new(a.resolve_converge(n)), Box::new(b.resolve_converge(n)))
            }
            Self::Not(inner) => Self::Not(Box::new(inner.resolve_converge(n))),
            other => other.clone(),
        }
    }

    /// Replace `ManaValueEqualsSourceCounters(kind)` with a concrete
    /// `ManaValueExactly(n)` where `n` is the source's live counter count —
    /// called at effect resolution (Aether Vial's `{T}: put a creature card
    /// with mana value equal to the number of charge counters …`).
    pub fn resolve_source_counters(&self, counts: &dyn Fn(CounterType) -> u32) -> Self {
        match self {
            Self::ManaValueEqualsSourceCounters(kind) => Self::ManaValueExactly(counts(*kind)),
            Self::And(a, b) => Self::And(
                Box::new(a.resolve_source_counters(counts)),
                Box::new(b.resolve_source_counters(counts)),
            ),
            Self::Or(a, b) => Self::Or(
                Box::new(a.resolve_source_counters(counts)),
                Box::new(b.resolve_source_counters(counts)),
            ),
            Self::Not(inner) => Self::Not(Box::new(inner.resolve_source_counters(counts))),
            other => other.clone(),
        }
    }

    /// A short noun for a *target* matching this filter — "creature",
    /// "artifact", "creature you control" — for prompt / label text
    /// (`Effect::effect_short_text`, the modal-mode picker, trigger prompts).
    /// Returns `None` for filters we can't cleanly name (complex `Or`/`Not`
    /// trees, stat gates), so callers fall back to a plain "target". Kept
    /// deliberately small: it only needs to name the common targeting filters
    /// well enough that a modal like Abrade reads "destroy target artifact".
    pub fn target_noun(&self) -> Option<String> {
        use SelectionRequirement as S;
        let base = match self {
            S::Player => "player",
            S::Creature => "creature",
            S::Artifact => "artifact",
            S::Enchantment => "enchantment",
            S::Planeswalker => "planeswalker",
            S::Permanent => "permanent",
            S::Land => "land",
            S::Nonland => "nonland permanent",
            S::Noncreature => "noncreature permanent",
            S::IsBasicLand => "basic land",
            S::IsNonbasicLand => "nonbasic land",
            // A type noun plus a "you control / an opponent controls"
            // modifier — the common targeting combo (e.g. "creature you
            // control"). If neither side names a type, give up.
            S::And(a, b) => {
                let noun = a.target_noun().or_else(|| b.target_noun())?;
                let suffix = controller_suffix(a).or_else(|| controller_suffix(b));
                return Some(match suffix {
                    Some(s) => format!("{noun} {s}"),
                    None => noun,
                });
            }
            _ => return None,
        };
        Some(base.to_string())
    }

    /// Whether a *player* entity could satisfy this requirement — used by
    /// targeting heuristics that decide whether to consider player targets
    /// (e.g. "deal N damage divided among any target" allows players, but
    /// "among target creatures" does not). Conservative: an `And` requires
    /// both halves to admit a player; `Or`/`Not` are permissive.
    pub fn can_match_player(&self) -> bool {
        match self {
            Self::Any | Self::Player => true,
            Self::And(a, b) => a.can_match_player() && b.can_match_player(),
            Self::Or(a, b) => a.can_match_player() || b.can_match_player(),
            Self::Not(_) => true,
            _ => false,
        }
    }
}

/// The "you control" / "an opponent controls" clause for a controller-scoped
/// requirement, used by [`SelectionRequirement::target_noun`] to build phrases
/// like "creature you control". `None` for non-controller requirements.
fn controller_suffix(r: &SelectionRequirement) -> Option<String> {
    match r {
        SelectionRequirement::ControlledByYou => Some("you control".to_string()),
        SelectionRequirement::OpponentPlayer => Some("opponent".to_string()),
        SelectionRequirement::ControlledByOpponent => Some("an opponent controls".to_string()),
        SelectionRequirement::ControlledByTriggerPlayer => Some("that player controls".to_string()),
        _ => None,
    }
}

// ── Token definition ──────────────────────────────────────────────────────────

/// Describes a token to be created on the battlefield.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
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
    /// Static abilities the token enters with. Used for self-scaling token
    /// bodies — Karn's Construct ("+1/+1 for each artifact you control") via
    /// `StaticEffect::PumpSelfByControlledPermanents`, and any future "the
    /// token has [static]" rider. Copied into the resulting `CardDefinition`
    /// by `token_to_card_definition`.
    #[serde(default)]
    pub static_abilities: Vec<StaticAbility>,
    /// Equip bonus for Equipment tokens (CR 301 — Mabel's Cragflame,
    /// Kellan's Boots, etc.). When `Some`, the resulting token carries
    /// this `equipped_bonus`; pair it with a `Keyword::Equip(cost)` in
    /// `keywords` so the token can be equipped. `None` for non-Equipment
    /// tokens. Defaults to `None` via `#[serde(default)]`.
    #[serde(default)]
    pub equipped_bonus: Option<EquipBonus>,
    /// Mint-time dynamic P/T: evaluated against the minting effect's
    /// context and stamped over `power`/`toughness` (Shark Typhoon's
    /// "X/X … where X is that spell's mana value"). Type riders chosen at
    /// mint time survive copies per CR 707.2.
    #[serde(default)]
    pub dynamic_pt: Option<(crate::effect::Value, crate::effect::Value)>,
    /// CR 508.3a — the token enters tapped (a "tapped <token>" gift / mint).
    /// Defaults to `false` (enters untapped) via `#[serde(default)]`.
    #[serde(default)]
    pub tapped: bool,
    /// CR 111.10i / 712 — back face for a double-faced token (the Incubator
    /// token's transformable Phyrexian back). Lifted onto the resulting
    /// `CardDefinition.back_face` so `Effect::Transform` can toggle it.
    #[serde(default)]
    pub back_face: Option<Box<TokenDefinition>>,
}

/// A `CardDefinition.exile_countdown` fuse: exile the card with `count`
/// counters of `counter`, tick one off at each of the owner's upkeeps, and
/// resolve `effect` (after the card is put into its owner's graveyard) when
/// the last one comes off.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExileCountdown {
    pub counter: CounterType,
    pub count: u32,
    pub effect: crate::effect::Effect,
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
    pub name: crate::static_str_serde::StaticStr,
    pub cost: ManaCost,
    pub supertypes: Vec<Supertype>,
    pub card_types: Vec<CardType>,
    pub subtypes: Subtypes,
    /// CR 202.2 / 105.2c — color indicator. Defines the object's color when it
    /// has no mana symbols to define it (tokens, DFC/Omen back faces). The
    /// object's colors are the union of its mana cost's colors and this
    /// indicator. Empty for ordinary cards whose color comes from their cost.
    #[serde(default)]
    pub color_indicator: Vec<Color>,
    /// CR 105 — colours forced onto this object, replacing everything its cost
    /// and indicator would give (Vodalian Mystic recolouring a stack spell).
    #[serde(default)]
    pub color_override: Option<Vec<Color>>,
    pub power: i32,
    pub toughness: i32,
    /// CR 211 / 212 — a Vanguard's hand-size and starting-life modifiers,
    /// applied by `GameState::seat_vanguard`. Zero for every other card.
    #[serde(default)]
    pub hand_modifier: i32,
    #[serde(default)]
    pub life_modifier: i32,
    pub base_loyalty: u32,
    /// CR 310.7 — printed defense of a Battle. The permanent enters with this
    /// many `CounterType::Defense` counters. 0 for non-battles.
    #[serde(default)]
    pub defense: u32,
    pub keywords: Vec<Keyword>,
    pub static_abilities: Vec<StaticAbility>,
    /// For instants/sorceries: the effect that resolves. Defaults to `Effect::Noop`
    /// for permanents whose ETB behaviour lives in `triggered_abilities`.
    pub effect: Effect,
    pub activated_abilities: Vec<ActivatedAbility>,
    pub triggered_abilities: Vec<TriggeredAbility>,
    /// CR 702.87 — Level-up bands (Student of Warfare). Empty for
    /// non-leveler cards.
    #[serde(default)]
    pub level_bands: Vec<LevelBand>,
    /// CR 721 — Station-card striations (`{N+}` symbols), ordered by `min`.
    /// Empty for non-station cards. Cards with any station band always carry
    /// the Station ability (CR 721.4); add it via `shortcut::station()`.
    #[serde(default)]
    pub station: Vec<StationBand>,
    pub loyalty_abilities: Vec<LoyaltyAbility>,
    /// CR 606.3 override — "You may activate the loyalty abilities of [this]
    /// twice each turn rather than only once." (Urza, Planeswalker.)
    #[serde(default)]
    pub loyalty_twice_each_turn: bool,
    /// CR 606.3b override — "As long as [this] entered this turn, you may
    /// activate its loyalty abilities any time you could cast an instant."
    /// (The Wandering Emperor — pairs with `Keyword::Flash`.)
    #[serde(default)]
    pub flash_loyalty: bool,
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
    /// SOS preparation card — the inset "prepare spell" (Adventure/Omen-
    /// style frame, *not* an MDFC back face). A preparation card is a
    /// creature card in every zone; this spell is never castable from
    /// the hand. While the permanent is prepared (carries a
    /// `CounterType::Prepared` counter), its current controller may cast
    /// a copy of this spell from exile via `GameAction::CastPrepareSpell`,
    /// which unprepares the creature. Cards that printed "This creature
    /// enters prepared." model it with `enters_with_counters:
    /// Some((CounterType::Prepared, Value::Const(1)))`.
    #[serde(default)]
    pub prepare_spell: Option<Box<CardDefinition>>,
    /// CR 710 — flip card. When `Some`, this is the *unflipped* (top) face and
    /// the value is the flipped (bottom) characteristics. Unlike a DFC, a flip
    /// card is single-faced — it is never cast or rendered as its flip side; it
    /// only flips in place via `Effect::Flip`. Only the top stores this.
    #[serde(default)]
    pub flip_face: Option<Box<CardDefinition>>,
    /// CR 603.8 — state-triggered flip. When `Some(kw)`, this flip card flips
    /// (`Effect::Flip`) the moment its *computed* keywords include `kw`
    /// (externally granted Flying counts). Checked once per SBA pass; flipping
    /// removes the condition so it fires once. Student of Elements (Flying).
    #[serde(default)]
    pub flip_when_has_keyword: Option<Keyword>,
    /// CR 603.8 — state-triggered flip on a live board `Predicate`, evaluated
    /// from the permanent's own controller once per SBA pass (Rune-Tail,
    /// Kitsune Ascendant — "when you have 30 or more life, flip this").
    /// Flipping clears the condition, so it fires once.
    #[serde(default)]
    pub flip_when_predicate: Option<crate::effect::Predicate>,
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
    /// "If this card is in a graveyard, effects from spells named X count it
    /// as a card named X" (Pardic Firecat / Flame Burst). Read by the
    /// name-counting `Value`s alongside the card's real name.
    #[serde(with = "crate::static_str_serde::opt", default)]
    pub counts_as_named_in_graveyard: Option<crate::static_str_serde::StaticStr>,
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
    /// CR 614 — "As this enters, it becomes your choice of [modes]." An
    /// as-enters replacement that sets base P/T and grants keywords before the
    /// first SBA sweep, so a printed `*/*` (0/0) body never dies before the
    /// choice locks in. The controller picks a mode via a `ChooseMode`
    /// decision at ETB. Corrupted Shapeshifter.
    #[serde(default)]
    pub enters_as_choice: Option<Vec<EntersChoiceMode>>,
    /// CR 614 — "As this permanent enters, [effect]." A one-shot replacement
    /// resolved during the battlefield hop, *before* the first state-based
    /// action sweep and before any enters-the-battlefield trigger, so a
    /// printed `*/*` body whose CDA depends on the effect never dies as a 0/0
    /// (Ixidron's "as this creature enters, turn all other nontoken creatures
    /// face down"). Distinct from an `EntersBattlefield` trigger, which uses
    /// the stack and can be responded to.
    #[serde(default)]
    pub as_enters_effect: Option<crate::effect::Effect>,
    /// CR 603.8 state trigger — "When [condition], sacrifice this permanent."
    /// Checked once per state-based-action pass against the permanent's
    /// controller; the sacrifice happens immediately (no stack), which matches
    /// the observable outcome for the cards that use it (Phylactery Lich).
    #[serde(default)]
    pub sacrifice_when: Option<crate::effect::Predicate>,
    /// CR 603.8 state trigger — "When [condition], [effect]." Checked once per
    /// state-based-action pass; the ability goes on the stack and doesn't
    /// trigger again until the condition has been false (`CardInstance
    /// ::state_trigger_armed`). The Hidden/Veiled animations that key on a
    /// board state rather than an event (Hidden Predators, Veiled Crocodile).
    #[serde(default)]
    pub state_trigger: Option<StateTriggeredAbility>,
    /// An exile fuse: the card exiles itself with `count` counters
    /// (`Effect::ExileSelfWithCountdown`), one comes off at each of the
    /// owner's upkeeps, and when the last is removed the engine puts the card
    /// into its owner's graveyard and resolves `effect`. All Hallow's Eve.
    #[serde(default)]
    pub exile_countdown: Option<ExileCountdown>,
    /// CR 614 — "As this enters, choose [mode A] or [mode B]." A *persistent*
    /// mode choice (unlike `enters_as_choice`, which only sets P/T): the
    /// controller picks one mode as the permanent enters and that mode's
    /// triggered/static abilities and keywords are baked onto this permanent
    /// for as long as it stays on the battlefield. The Tarkir: Dragonstorm
    /// Siege cycle (Barrensteppe / Frostcliff / Glacierwood / Hollowmurk
    /// Siege). `CardInstance.chosen_mode` records the pick so a snapshot
    /// round-trip re-bakes the same mode.
    #[serde(default)]
    pub enter_modes: Option<Vec<EnterMode>>,
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
    /// CR 603.8 state trigger — "When a player other than this permanent's
    /// owner controls it, that player sacrifices it. If they do, it deals N
    /// damage to that player." (Bronze Bombshell: `Some(7)`.) Checked in the
    /// SBA pass; latched per-instance so it fires once per control change.
    #[serde(default)]
    pub sacrifice_and_burn_when_stolen: Option<u32>,
    /// CR 603.8 state trigger — "When you control no other [filter], sacrifice
    /// this permanent" (Synod Centurion). Checked in the SBA pass against the
    /// controller's other permanents; latched per-instance so the sacrifice is
    /// queued once while the condition holds.
    #[serde(default)]
    pub sacrifice_when_you_control_no_other: Option<SelectionRequirement>,
    /// CR 604.3 — characteristic-defining dynamic P/T formula (Tarmogoyf,
    /// Death's Shadow). When `Some`, `compute_battlefield` injects a
    /// layer-7a SetPT effect from the formula on every recompute.
    #[serde(default)]
    pub dynamic_pt: Option<DynamicPt>,
    /// CR 614.6 — "If this would be put into a graveyard from anywhere,
    /// shuffle it into its owner's library instead." Checked at the
    /// `route_to_graveyard` funnel (Darksteel / Blightsteel Colossus).
    #[serde(default)]
    pub shuffles_into_library_instead: bool,
    /// CR 603-style self-replacement — "If this permanent would die, exile it
    /// instead." Scoped to battlefield deaths (checked in
    /// `remove_from_battlefield_to_graveyard_raw`, alongside finality counters),
    /// so a discard/mill from another zone still lands in the graveyard.
    /// Gloomshrieker, and the escape/Titan self-exile clauses.
    #[serde(default)]
    pub dies_to_exile: bool,
    /// CR 614.6 — "If this creature would die, put it on the bottom of its
    /// owner's library instead." The library sibling of `dies_to_exile`,
    /// checked at the same battlefield-death funnel (Nissa's Chosen).
    #[serde(default)]
    pub dies_to_library_bottom: bool,
    /// CR 704.5j exception — "If there are exactly two permanents with this
    /// name on the battlefield, the legend rule doesn't apply to them."
    /// (Brothers Yamazaki.) When the same-name legend group has exactly two
    /// members and all are flagged, the SBA skips it.
    #[serde(default)]
    pub legend_pair_exempt: bool,
    /// "[This] isn't legendary if it's a token" (Aeve, Progenitor Ooze).
    /// The legend-rule SBA skips token instances of this definition.
    #[serde(default)]
    pub nonlegendary_as_token: bool,
    /// CR 202.1a / 601.3e — this card has **no printed mana cost** (distinct
    /// from `{0}`, which is payable): the normal pay-the-cost cast path
    /// rejects it; suspend / "cast without paying its mana cost" still work
    /// (Ancestral Vision, Living End, Gaea's Will, Lotus Bloom).
    #[serde(default, alias = "suspend_only")]
    pub no_mana_cost: bool,
    /// CR 604.3 CDA — "As long as [this] isn't on the battlefield, it's a
    /// 1/1 Insect creature in addition to its other types" (Grist). Zone
    /// filters treat the card as a creature everywhere but the battlefield.
    #[serde(default)]
    pub creature_off_battlefield: bool,
    /// CR 714 — Saga chapter abilities, as `(chapter_number, effect)` pairs.
    /// A combined chapter ("I, II — …") is listed once per number with the
    /// same effect. Non-empty marks the card a Saga: it enters with one lore
    /// counter (firing chapter 1), gains one more at the start of each of its
    /// controller's precombat main phases (firing that chapter), and is
    /// sacrificed by SBA once its lore counters reach the final (highest)
    /// chapter number and no chapter ability of its is still on the stack.
    /// Defaults to empty via `#[serde(default)]`.
    #[serde(default)]
    pub saga_chapters: Vec<(u32, crate::effect::Effect)>,
    /// CR 702.155 — Read Ahead. A Saga with this flag enters with a chosen
    /// number of lore counters (1..final chapter) instead of one, firing only
    /// the chosen chapter. Defaults to `false`.
    #[serde(default)]
    pub read_ahead: bool,
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
    /// CR 601.2b — "You may cast this spell as though it had flash if you pay
    /// [cost] more to cast it" (the Invasion "or Flight" cycle). The surcharge
    /// is only owed when the spell is actually cast outside sorcery timing.
    #[serde(default)]
    pub flash_surcharge: Option<crate::mana::ManaCost>,
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
    /// CR 614 — "This permanent enters under the control of an opponent of your
    /// choice." A control-setting entry replacement applied at the battlefield
    /// hop (Captive Audience). Defaults to `false`.
    #[serde(default)]
    pub enters_under_opponent_control: bool,
    /// CR 106.6b — "Spend only mana produced by creatures to cast this spell"
    /// (Myr Superion). Surfaces on `spell_kind()` as
    /// `SpellKind::creature_mana_only`, which narrows payment to the pool's
    /// creature-produced provenance and keeps auto-tap off non-creature
    /// sources. Defaults to `false`.
    #[serde(default)]
    pub spend_only_creature_mana: bool,
    /// "This spell costs {1} less to cast for each [filter] card in your
    /// graveyard" — the graveyard-counting sibling of `affinity_filter`.
    /// Generic-only, clamped by the caller. Powers Tolarian Terror /
    /// The Dawning Archaic (instant-or-sorcery), Murktide-class delve-flavor
    /// reductions, etc. Defaults to `None` for snapshot back-compat.
    #[serde(default)]
    pub affinity_graveyard_filter: Option<SelectionRequirement>,
    /// "This spell costs `{amount}` less to cast if it targets a permanent
    /// matching `filter`." A card-intrinsic, target-conditional generic
    /// reduction (Ride's End — "{3} less if it targets a tapped permanent").
    /// Generic-only, clamped by the caller; evaluated against the chosen
    /// target at cast time. Defaults to `None` for snapshot back-compat.
    #[serde(default)]
    pub self_cost_reduction_if_target: Option<(SelectionRequirement, u32)>,
    /// The colored-aware sibling of `self_cost_reduction_if_target`:
    /// "This spell costs [cost] less to cast if it targets [filter]."
    /// The reduction is a full `ManaCost` removed pip-by-pip via
    /// `ManaCost::reduce_by_cost` (CR 601.2f — a MANDATORY reduction,
    /// not an opt-in alternative cost). Powers Brush Off's "{1}{U} less
    /// if it targets an instant or sorcery spell". Defaults to `None`
    /// for snapshot back-compat.
    #[serde(default)]
    pub self_cost_reduction_cost_if_target: Option<(SelectionRequirement, ManaCost)>,
    /// "As an additional cost to cast this spell, pay X life." X is chosen
    /// at cast time (the X prompt fires for this flag even when the mana
    /// cost carries no {X} pip, capped at the caster's life total),
    /// pre-flighted against the caster's life (CR 119.4 — paying down to
    /// exactly 0 is legal), and paid as a COST on cast (CR 601.2h) — the
    /// life stays paid if the spell is countered. Resolution reads the
    /// chosen X through the normal `Value::XFromCost` plumbing. Vicious
    /// Rivalry, Fix What's Broken.
    #[serde(default)]
    pub additional_cost_pay_x_life: bool,
    /// "This spell costs `{amount}` less to cast if you control a permanent
    /// matching `filter`." Flat (non-scaling) board-state-gated generic
    /// reductions — Pearl of Wisdom ("{1} less if you control an Otter").
    /// Multiple clauses each apply independently (Geistlight Snare — "{1} less
    /// if you control a Spirit; and {1} less if you control an enchantment").
    /// Distinct from `affinity_filter`, which scales per matching permanent.
    /// Generic-only, clamped by the caller; empty by default for snapshot
    /// back-compat.
    #[serde(default)]
    pub self_cost_reduction_if_control: Vec<(SelectionRequirement, u32)>,
    /// "This spell costs `{amount}` less to cast if it's night" (Moonrager's
    /// Slash). Generic-only, clamped by the caller. `None` by default for
    /// snapshot back-compat.
    #[serde(default)]
    pub self_cost_reduction_if_night: Option<u32>,
    /// "This spell costs `{amount}` less to cast as long as there are four or
    /// more card types among cards in your graveyard" (Delirium — Drag to the
    /// Roots). Generic-only, clamped by the caller. `None` by default.
    #[serde(default)]
    pub self_cost_reduction_if_delirium: Option<u32>,
    /// "This spell costs `{amount}` less to cast if you've committed a crime
    /// this turn" (Seize the Secrets). Generic-only, clamped by the caller.
    /// Reads `Player.committed_crime_this_turn`. `None` by default.
    #[serde(default)]
    pub self_cost_reduction_if_crime: Option<u32>,
    /// "This spell costs `{amount}` less to cast if you've sacrificed an
    /// artifact this turn" (Suspicious Detonation). Generic-only, clamped by the
    /// caller. Reads `Player.artifacts_sacrificed_this_turn`. `None` by default.
    #[serde(default)]
    pub self_cost_reduction_if_sacrificed_artifact: Option<u32>,
    /// "This spell costs {1} less to cast for each card you've drawn this
    /// turn" (Deem Inferior). Generic-only, clamped by the caller. Reads
    /// `Player.cards_drawn_this_turn`. Defaults to `false`.
    #[serde(default)]
    pub self_cost_reduction_per_cards_drawn: bool,
    /// "This spell costs `{amount}` less to cast if you've cast another spell
    /// this turn" (Rally the Monastery). Generic-only, clamped by the caller.
    /// Reads `Player.spells_cast_this_turn`, which does not yet count the
    /// spell being cast, so `> 0` means a prior spell. `None` by default.
    #[serde(default)]
    pub self_cost_reduction_if_cast_spell: Option<u32>,
    /// "This spell costs `{amount}` less to cast if [condition]" — the general
    /// predicate-gated sibling of the `self_cost_reduction_if_*` family
    /// (the Prophecy Avatar cycle's board-state discounts). Generic-only,
    /// clamped by the caller; the predicate is evaluated from the caster's
    /// seat with no source or target.
    #[serde(default)]
    pub self_cost_reduction_if: Option<(crate::effect::Predicate, u32)>,
    /// "This spell costs `{amount}` less to cast if evidence was collected"
    /// (Bite Down on Crime, CR 701.59). Pairs with an
    /// `AdditionalCastCost::CollectEvidence`; generic-only, clamped by the
    /// caller. `None` by default.
    #[serde(default)]
    pub self_cost_reduction_if_collect_evidence: Option<u32>,
    /// "This spell costs `{per}` less to cast for each [`Value`]" — the
    /// scaled sibling of `self_cost_reduction_if` (Domain: Draco's `{2}` and
    /// Stratadon's `{1}` per basic land type among lands you control).
    /// Generic-only, clamped by the caller; the value is evaluated from the
    /// caster's seat with no source or target.
    #[serde(default)]
    pub self_cost_reduction_per: Option<(crate::effect::Value, u32)>,
    /// "Equipped creature gets +P/+T and has [keywords]." Read by
    /// `compute_battlefield` for any Equipment whose `attached_to` points at
    /// a creature on the battlefield — the bonus is emitted as layer-7 (P/T)
    /// and layer-6 (keyword) continuous effects on the equipped creature.
    /// `None` for non-Equipment cards (and for Equipment whose only relevant
    /// effect is an activated ability, e.g. Lightning Greaves' grant-on-
    /// activate approximation). Defaults to `None` for snapshot back-compat.
    #[serde(default)]
    pub equipped_bonus: Option<EquipBonus>,
    /// Additional {E} (energy) cost paid to equip, on top of `Keyword::Equip`'s
    /// mana. "Equip—Pay {E}{E}" (Inventor's Axe). Paid from the activator's
    /// energy pool during `equip`. Defaults to 0 via `#[serde(default)]`.
    #[serde(default)]
    pub equip_energy_cost: u32,
    /// Additional life paid to equip, on top of `Keyword::Equip`'s mana.
    /// "Equip—Pay 3 life" (Nightmare Lash). Defaults to 0.
    #[serde(default)]
    pub equip_life_cost: u32,
    /// "Equip—Sacrifice an artifact" (Piston Sledge): a sacrifice paid on top
    /// of (usually instead of) the mana equip cost. The lowest-power match is
    /// auto-picked; equipping fails cleanly when nothing matches.
    #[serde(default)]
    pub equip_sacrifice_filter: Option<SelectionRequirement>,
    /// Cheaper alternate equip cost usable when the equip target is a
    /// creature *token* — "Equip creature token {1}" (Team Pennant, the
    /// STX mascot-equipment cycle). Consulted in `equip` before the
    /// regular `Keyword::Equip` cost. Defaults to `None`.
    #[serde(default)]
    pub equip_token_cost: Option<crate::mana::ManaCost>,
    /// CR 301.5c — "This Equipment can be attached only to a [filter]"
    /// (Konda's Banner). Gates `equip` and the CR 704.5n unattach sweep, so a
    /// host that stops matching sheds the Equipment.
    #[serde(default)]
    pub attach_only_filter: Option<SelectionRequirement>,
    /// "When you cast this spell, copy it X times" where X is the cast's
    /// `x_value` — the Storm-family cast-time copy rider driven by a
    /// caster-chosen count (Plumb the Forbidden's "when you sacrifice a
    /// creature this way, copy this spell", where the sacrifice count IS
    /// the X). Copies are real, uncounterable stack objects above the
    /// original, like Storm's. Defaults to false.
    #[serde(default)]
    pub copies_on_cast_x: bool,
    /// CR 702.95 — Soulbond bonus. When `Some`, this card carries the Soulbond
    /// keyword and, while paired (`CardInstance.soulbond_partner`), confers
    /// this bonus on BOTH itself and its partner. Defaults to `None`.
    #[serde(default)]
    pub soulbond_bonus: Option<SoulbondBonus>,
    /// "If a creature dealt damage by this would die this turn, exile it
    /// instead" — a source-bound death replacement (Kumano Master Yamabushi,
    /// Frostwielder). When true, every creature this permanent damages is
    /// added to `GameState.dies_to_exile_eot` for the rest of the turn.
    /// Defaults to `false` via `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub damage_exiles_if_dies: bool,
    /// CR 601.2b/601.2f — additional cost(s) paid as the spell is cast
    /// ("As an additional cost to cast this spell, …"). Paid during
    /// casting, not folded into resolution: the spell can't be cast unless
    /// every cost is payable, and a sacrifice/discard/life payment happens
    /// immediately (death/discard triggers fire before the spell resolves).
    /// Powers Necrotic Fumes, Tend the Pests, Witherbloom Sacrosanct.
    /// Defaults to empty via `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub additional_cast_cost: Vec<AdditionalCastCost>,
    /// CR 702.34a — additional cost(s) that apply only on the **flashback**
    /// cast ("Flashback—{1}{B}, Pay 3 life"; "Flashback—Sacrifice a Mountain").
    /// Paid on top of the flashback mana cost by `cast_flashback`.
    #[serde(default)]
    pub flashback_additional_cost: Vec<AdditionalCastCost>,
    /// CR 702.41 — a non-mana Entwine cost ("Entwine—Sacrifice two lands" —
    /// Solar Tide, Betrayal of Flesh). Paid as an additional cost of an
    /// entwined cast, alongside (or instead of) `Keyword::Entwine`'s mana.
    #[serde(default)]
    pub entwine_additional_cost: Option<AdditionalCastCost>,
    /// CR 702.47 — a non-mana Splice cost ("Splice onto Arcane—Sacrifice two
    /// Mountains" — Torrent of Stone). Paid alongside `Keyword::Splice`'s mana
    /// when this card is spliced onto a spell.
    #[serde(default)]
    pub splice_extra_cost: Option<AdditionalCastCost>,
    /// "This spell costs {N} more to cast if it targets a [filter]" —
    /// Vanish into Eternity. Read by `extra_cost_for_spell` off the chosen
    /// target at cast time.
    #[serde(default)]
    pub cost_increase_if_targets: Option<(SelectionRequirement, u32)>,
    /// "This spell costs [cost] more to cast for each target beyond the first"
    /// — Fireball's `{1}`, and the JOU **Strive** cycle's colored riders
    /// ({2}{W} and friends). Charged off the chosen target-slot count at cast
    /// time by `strive_cost_for_spell`.
    #[serde(default)]
    pub cost_per_extra_target: Option<crate::mana::ManaCost>,
    /// "If you cast this spell during your main phase, you may [target one
    /// additional …]" (Return to Dust). Casts with filled extra target slots
    /// are rejected outside the caster's own main phase.
    #[serde(default)]
    pub extra_targets_main_phase_only: bool,
    /// "Cast this spell only during combat after blockers are declared"
    /// (Flash Foliage). Checked at the cast gate.
    #[serde(default)]
    pub cast_only_after_blockers: bool,
    /// "Cast this spell only during combat before blockers are declared"
    /// (Blaze of Glory). Gated in the cast-timing check.
    pub cast_only_before_blockers: bool,
    /// "Cast this spell only during combat" (Cauldron Dance, Spinal Embrace).
    /// Any combat step, either player's turn.
    #[serde(default)]
    pub cast_only_during_combat: bool,
    /// CR 407.3 — "Remove this card from your deck before playing if you're not
    /// playing for ante." Deck legality rejects the card outside an ante game.
    #[serde(default)]
    pub ante_only: bool,
    /// "Cast this spell only before attackers are declared" (Master Warcraft).
    /// Gated in the cast-timing check: legal until the Declare Attackers step
    /// has produced attackers.
    #[serde(default)]
    pub cast_only_before_attackers: bool,
    /// "You can't cast this spell unless [condition]" (Rakdos, Lord of Riots).
    /// Checked at the cast gate against the caster's game state.
    #[serde(default)]
    pub cast_condition: Option<crate::effect::Predicate>,
    /// Gate on casting via Flashback ("Corrupted — … this card has flashback"
    /// — Viral Spawning). Checked at the graveyard-cast gate.
    #[serde(default)]
    pub flashback_condition: Option<crate::effect::Predicate>,
    /// CR 702.33 / 601.2b — an *optional* non-mana additional cost:
    /// "Kicker—Sacrifice an artifact or creature" (Vayne's Treachery),
    /// "Kicker—Return a land you control" (Chocobo Kick), "you may sacrifice
    /// an artifact" (Voltage Surge). Opting in via `GameAction::CastSpellKicked`
    /// pays this cost through the additional-cast-cost machinery and stamps
    /// the cast `kicked`, which `Predicate::SpellWasKicked` reads.
    #[serde(default)]
    pub kicker_action_cost: Option<AdditionalCastCost>,
    /// CR 702.32b — "Kicker {A} and/or {B}": two independently payable kicker
    /// costs (the Apocalypse Volver cycle, Illuminate). Cast via
    /// `GameAction::CastSpellKickers`; each paid index is stamped onto the
    /// spell and read back by `Predicate::SpellWasKickedWith`.
    #[serde(default)]
    pub kicker_options: Vec<crate::mana::ManaCost>,
    /// CR 614 — "If a spell or ability an opponent controls causes you to
    /// discard this card, put it onto the battlefield with N `CounterType`
    /// counters on it instead of putting it into your graveyard" (Dodecapod).
    /// A discard-destination replacement, applied in `discard_card`.
    #[serde(default)]
    pub opponent_discard_deploys: Option<(CounterType, u32)>,
    /// CR 702.103 — Bestow alternative cost. When `Some(cost)`, the card may
    /// be cast as an Aura spell targeting a creature for this cost (via
    /// `GameAction::CastBestow`); it enters attached, grants its
    /// `equipped_bonus`, and is *not* a creature while bestowed. If the host
    /// leaves, the bestowed permanent stays as a creature. Defaults to
    /// `None` for snapshot back-compat.
    #[serde(default)]
    pub bestow: Option<crate::mana::ManaCost>,
    /// CR 702.143 — Foretell. `Some(cost)` marks the card as foretellable:
    /// pay {2} to exile it face-down (`GameAction::Foretell`); on a later
    /// turn cast it from exile for this foretell `cost`
    /// (`GameAction::CastForetell`). Defaults to `None`.
    #[serde(default)]
    pub foretell_cost: Option<crate::mana::ManaCost>,
    /// CR 715 — Adventure. When `Some`, this creature card also has an
    /// instant/sorcery "adventure" half that can be cast instead
    /// (`GameAction::CastAdventure`). On resolution the card is exiled
    /// (rather than going to the graveyard) with permission to cast the
    /// creature half from exile later (`GameAction::CastAdventureCreature`).
    /// Defaults to `None` via `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub adventure: Option<Box<Adventure>>,
    /// CR 702.183 — Omen. When `Some`, this creature card also has an
    /// instant/sorcery "Omen" half that can be cast from hand for the listed
    /// cost (`GameAction::CastOmen`). On resolution or counter the card is
    /// shuffled into its owner's library instead of going to the graveyard.
    /// Reuses the [`Adventure`] shape (name/cost/types/effect).
    #[serde(default)]
    pub omen: Option<Box<Adventure>>,
    /// CR 702.165 — Gift. When `Some`, as the spell is cast its controller may
    /// promise a gift to an opponent (`GameAction::CastGift`); if promised, the
    /// opponent receives the gift and the spell resolves its enhanced
    /// `gifted_effect` instead of the printed base `effect`.
    #[serde(default)]
    pub gift: Option<Box<Gift>>,
    /// CR 702.160 — Prototype. When `Some`, this colorless artifact creature
    /// may instead be cast for the prototype cost (`GameAction::CastPrototype`),
    /// entering with the prototype's mana cost, color, and size; it keeps its
    /// abilities and types. Defaults to `None` via `#[serde(default)]`.
    #[serde(default)]
    pub prototype: Option<Box<Prototype>>,
    /// A "[cost], Discard this card: [effect]" from-hand activated ability
    /// (`GameAction::ActivateDiscardAbility`). Magma Opus's Treasure mode.
    #[serde(default)]
    pub discard_activated: Option<Box<DiscardActivated>>,
    /// CR 702.140 — Mutate. `Some(cost)` lets this non-Human creature spell
    /// be cast for its mutate cost (`GameAction::CastMutate`), merging with a
    /// target non-Human creature you own instead of entering on its own.
    /// Defaults to `None` via `#[serde(default)]`.
    #[serde(default)]
    pub mutate: Option<crate::mana::ManaCost>,
    /// CR 701.67 — Waterbend. `Some` adds an "as an additional cost to cast
    /// this spell, waterbend {N}" rider (`GameAction::CastSpellWaterbend`).
    /// Each generic of the waterbend cost may be paid by tapping an untapped
    /// artifact or creature you control (Convoke restricted to the sub-cost,
    /// over artifacts as well as creatures), or with mana. Defaults to `None`.
    #[serde(default)]
    pub waterbend: Option<Waterbend>,
    /// CR 702.139 — Companion deck restriction. `Some` for the ten companion
    /// legends; `format::companion_restriction_met` checks a deck against it.
    #[serde(default)]
    pub companion: Option<CompanionRule>,
    /// CR 702.170 — Plot. `Some(cost)` marks the card as plottable: during
    /// your main phase with an empty stack, pay this cost to exile it
    /// face-up (`GameAction::Plot`); on a later turn cast it from exile
    /// without paying its mana cost (`GameAction::CastPlotted`). Defaults to
    /// `None` via `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub plot_cost: Option<crate::mana::ManaCost>,
    /// CR 709 — Split card. When `Some`, the main definition fields describe
    /// the **left** half (cast via the normal cast path); `right` holds the
    /// right half (cast via `GameAction::CastSplitRight`). With `fuse`, both
    /// halves can be cast as a single spell (`GameAction::CastSplitFused`).
    /// Defaults to `None` for snapshot back-compat.
    #[serde(default)]
    pub split: Option<Box<SplitCard>>,
    /// CR 702.94 — Miracle. `Some(cost)` lets the owner reveal this card as
    /// the first card they draw in a turn and cast it for the miracle `cost`
    /// (cheaper than its mana cost). Wired via a draw-time grant of the
    /// miracle alt-cost (`granted_alt_cast_cost_eot` + `may_play_until`);
    /// the window is STEP-BOUNDED (`MayPlayDuration::EndOfThisStep`) and
    /// bypasses the sorcery-speed gate (`MayPlayPermission.miracle`),
    /// matching the printed cast-it-when-revealed shape.
    /// Defaults to `None` for snapshot back-compat.
    #[serde(default)]
    pub miracle: Option<crate::mana::ManaCost>,
    /// CR 709.5 — Room enchantment doors. When `Some`, this card is a split
    /// permanent: cast either door (`GameAction::CastRoomDoor`); on the
    /// battlefield only unlocked doors' abilities are live
    /// (`CardInstance.unlocked_doors` + `room_definition_with`), and a locked
    /// door can be unlocked at sorcery speed for its cost
    /// (`GameAction::UnlockRoomDoor`). Defaults to `None` for back-compat.
    #[serde(default)]
    pub room: Option<Box<RoomDoors>>,
    /// MKM Case enchantments (Enchantment — Case). When `Some`, the card's own
    /// `triggered_abilities` are its always-on abilities; `case.solved_*` are the
    /// "Solved —" abilities, live only once solved. At the beginning of the
    /// controller's end step an unsolved Case whose `case.to_solve` predicate
    /// holds becomes solved (`CardInstance.case_solved` + `case_definition_solved`).
    /// Defaults to `None` for back-compat.
    #[serde(default)]
    pub case: Option<Box<CaseData>>,
}

/// MKM Case enchantment data. The parent [`CardDefinition`] carries the Case's
/// always-on abilities (its first line); this struct holds the "To solve"
/// condition and the "Solved —" abilities that switch on once the Case is
/// solved.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct CaseData {
    /// "To solve —" condition, evaluated (source = the Case, controller = its
    /// controller) at the beginning of the controller's end step. When it holds
    /// and the Case is unsolved, the Case is solved.
    pub to_solve: crate::effect::Predicate,
    #[serde(default)]
    pub solved_triggered: Vec<TriggeredAbility>,
    #[serde(default)]
    pub solved_activated: Vec<ActivatedAbility>,
    // Skipped on the wire — a Case round-trips by name + `case_solved`, and the
    // live definition is rebuilt from the catalog factory on load.
    #[serde(skip)]
    pub solved_static: Vec<StaticAbility>,
}

/// CR 709.5 — a Room's two door halves. The parent `CardDefinition` is the
/// combined object (full name, Enchantment — Room); each door carries its
/// own name, cost, and abilities, live only while that door is unlocked.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct RoomDoors {
    pub left: RoomDoor,
    pub right: RoomDoor,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct RoomDoor {
    /// Door name (e.g. "Ritual Chamber"). Owned: door names only feed
    /// logs/UI labels, so no `&'static str` interning is needed.
    pub name: String,
    pub cost: ManaCost,
    pub triggered_abilities: Vec<TriggeredAbility>,
    pub activated_abilities: Vec<ActivatedAbility>,
    // Skipped on the wire — a Room instance round-trips by card name +
    // `unlocked_doors`, and the live definition is rebuilt from the
    // catalog factory on load.
    #[serde(skip)]
    pub static_abilities: Vec<StaticAbility>,
}

/// CR 709 — the split layout. The left half lives on the parent
/// [`CardDefinition`]'s own `cost` / `card_types` / `effect`; this struct
/// carries the right half plus the Fuse flag.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct SplitCard {
    pub right: SplitHalf,
    /// CR 702.102 — Fuse: may cast both halves as one spell from hand.
    pub fuse: bool,
    /// CR 702.127 — Aftermath: the right half can be cast *only* from the
    /// graveyard (`GameAction::CastAftermath`), and is exiled on resolution.
    /// The left half is cast from hand normally and lands in the graveyard,
    /// where the aftermath half then becomes available.
    pub aftermath: bool,
}

/// One half of a split card (the left half's data lives directly on the
/// parent [`CardDefinition`]). The half's name isn't stored — the combined
/// card name on the parent definition is used for the log.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct SplitHalf {
    pub cost: ManaCost,
    /// Instant or Sorcery (governs cast timing).
    pub card_types: Vec<CardType>,
    pub effect: crate::effect::Effect,
}

impl SplitHalf {
    /// True if this half can be cast at instant speed.
    pub fn is_instant_speed(&self) -> bool {
        self.card_types.contains(&CardType::Instant)
    }
}

/// CR 715 — the instant/sorcery "adventure" half of an Adventurer card. The
/// creature half lives on the parent [`CardDefinition`]; this struct holds
/// only what the adventure spell needs to be cast and resolved.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Adventure {
    #[serde(with = "crate::static_str_serde")]
    pub name: crate::static_str_serde::StaticStr,
    pub cost: ManaCost,
    /// Instant or Sorcery (governs cast timing).
    pub card_types: Vec<CardType>,
    pub effect: crate::effect::Effect,
}

impl Adventure {
    /// True if the adventure half can be cast at instant speed.
    pub fn is_instant_speed(&self) -> bool {
        self.card_types.contains(&CardType::Instant)
    }
}

/// A "[cost], Discard this card: [effect]" ability usable from the hand at
/// instant speed (Magma Opus's {U/R}{U/R} Treasure mode). The effect resolves
/// as a targetless activated ability with the card as its source.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct DiscardActivated {
    pub cost: ManaCost,
    pub effect: crate::effect::Effect,
}

/// CR 702.165 — Gift. The printed spell's base resolution lives in the parent
/// [`CardDefinition::effect`]; when the gift is promised, the spell resolves
/// `gifted_effect` (which itself bestows the promised gift on the opponent
/// before its other effects). `label` is the gift's printed name, for the
/// client's "Gift a Food / a card / a tapped Fish" prompt.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Gift {
    #[serde(with = "crate::static_str_serde")]
    pub label: crate::static_str_serde::StaticStr,
    pub gifted_effect: crate::effect::Effect,
}

/// CR 701.67 — Waterbend. The "waterbend {N}" additional cast cost. `amount`
/// is the generic to pay (a [`Value`] so `waterbend {X}` reads the chosen X);
/// `optional` marks "you may waterbend {N}" riders, where paying enables an
/// "if its additional cost was paid" clause (`Predicate::SpellWasWaterbend`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Waterbend {
    pub amount: Value,
    #[serde(default)]
    pub optional: bool,
}

impl Default for Waterbend {
    fn default() -> Self {
        Waterbend { amount: Value::ZERO, optional: false }
    }
}

/// CR 702.160 — the Prototype alternative cast of a Brothers' War artifact
/// creature. The printed (full) cost/color/size live on the parent
/// [`CardDefinition`]; this holds the smaller, colored prototype face.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Prototype {
    pub cost: ManaCost,
    pub power: i32,
    pub toughness: i32,
}

/// CR 702.139 — the deck-construction restriction a Companion imposes on your
/// starting deck. If every card in the deck satisfies it, the companion may be
/// brought from the sideboard to hand once (`GameAction::CompanionToHand`,
/// {3}). Checked by `format::companion_restriction_met`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompanionRule {
    /// Lurrus — every permanent card has mana value ≤ N.
    PermanentsManaValueAtMost(u32),
    /// Keruga — every card with mana value ≥ N (lands exempt).
    NonlandManaValueAtLeast(u32),
    /// Gyruda — every card has even mana value (lands exempt).
    NonlandEvenManaValue,
    /// Obosh — every card has odd mana value (lands exempt).
    NonlandOddManaValue,
    /// Jegantha — no card's mana cost contains two or more of the same mana
    /// symbol (`{R}{R}`, `{G}{G}`, hybrid pairs, etc.).
    NoDuplicateManaSymbols,
    /// Lutri — at most one copy of each nonland card (singleton).
    Singleton,
    /// Kaheera — every creature card is among these types.
    CreatureTypesAmong(Vec<CreatureType>),
    /// Umori — every nonland card shares one card type.
    NonlandShareACardType,
    /// Yorion — the deck contains at least N cards beyond the format minimum.
    DeckSizeAtLeastOverMinimum(u32),
    /// Zirda — every permanent card has an activated ability.
    PermanentsHaveActivatedAbility,
}

/// CR 707 — "enters as a copy of [filter] permanent" spec, stored on
/// `CardDefinition.enters_as_copy`. The copier becomes a copy of the
/// chosen permanent; `extra_creature_types` are layered on top of the
/// copied subtypes (Phantasmal Image's "it's an Illusion in addition to
/// its other types").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EntersAsCopy {
    #[serde(default)]
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
    /// CR 707.2 name-retention exception ("except its name is still ~").
    /// When true the copier keeps its own printed name instead of the
    /// copied object's. Used by Mockingbird.
    #[serde(default)]
    pub keep_name: bool,
    /// Card types layered on top of the copy ("except it's an artifact in
    /// addition to its other types" — Phyrexian Metamorph). Added after the
    /// copiable characteristics are stamped, so they survive the rewrite.
    #[serde(default)]
    pub extra_card_types: Vec<CardType>,
    /// CR 707.2 — "except it's legendary in addition to its other types"
    /// (Sakashima the Impostor). Added after the copiable characteristics are
    /// stamped.
    #[serde(default)]
    pub legendary: bool,
    /// CR 707.2e — "except it's not legendary." When true the copy strips the
    /// Legendary supertype so it doesn't trigger the legend rule (Mirror Image).
    #[serde(default)]
    pub non_legendary: bool,
    /// Activated abilities layered on top of the copy (Mercurial Pretender's
    /// "except it has '{2}{U}{U}: Return this creature to its owner's hand'").
    #[serde(default)]
    pub extra_activated: Vec<crate::effect::ActivatedAbility>,
}

/// CR 614 — one mode of a `CardDefinition.enters_as_choice` as-enters
/// replacement. The chosen mode's `power`/`toughness` overwrite the printed
/// (`*/*`) base and its `keywords` are granted. Corrupted Shapeshifter's
/// "a 3/3 with flying, a 2/5 with vigilance, or a 0/12 with defender."
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EntersChoiceMode {
    pub power: i32,
    pub toughness: i32,
    #[serde(default)]
    pub keywords: Vec<Keyword>,
}

/// CR 614 — one arm of a `CardDefinition.enter_modes` persistent mode choice.
/// The chosen arm's abilities are baked onto the permanent when it enters
/// (Tarkir Siege cycle). `label` is the printed mode name ("Abzan", "Mardu").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EnterMode {
    #[serde(with = "crate::static_str_serde")]
    pub label: crate::static_str_serde::StaticStr,
    #[serde(default)]
    pub triggered_abilities: Vec<crate::effect::TriggeredAbility>,
    #[serde(default)]
    pub static_abilities: Vec<crate::effect::StaticAbility>,
    #[serde(default)]
    pub keywords: Vec<Keyword>,
}

/// An expansion a card name was *originally* printed in — the reference set
/// for `SelectionRequirement::OriginallyPrintedIn` (CR 201.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OriginalSet {
    Antiquities,
    ArabianNights,
    Homelands,
}

impl OriginalSet {
    /// The set's full name list. Reprints share the name, so a matching
    /// printed name is the whole test.
    pub fn contains(self, name: &str) -> bool {
        match self {
            OriginalSet::Antiquities => ANTIQUITIES_NAMES.binary_search(&name).is_ok(),
            OriginalSet::ArabianNights => ARABIAN_NIGHTS_NAMES.binary_search(&name).is_ok(),
            OriginalSet::Homelands => HOMELANDS_NAMES.binary_search(&name).is_ok(),
        }
    }
}

/// Antiquities' 85 distinct card names, sorted for `binary_search`.
static ANTIQUITIES_NAMES: [&str; 85] = [
    "Amulet of Kroog", "Argivian Archaeologist", "Argivian Blacksmith", "Argothian Pixies",
    "Argothian Treefolk", "Armageddon Clock", "Artifact Blast", "Artifact Possession",
    "Artifact Ward", "Ashnod's Altar", "Ashnod's Battle Gear", "Ashnod's Transmogrant", "Atog",
    "Battering Ram", "Bronze Tablet", "Candelabra of Tawnos",
    "Circle of Protection: Artifacts", "Citanul Druid", "Clay Statue", "Clockwork Avian",
    "Colossus of Sardia", "Coral Helm", "Crumble", "Cursed Rack", "Damping Field", "Detonate",
    "Drafna's Restoration", "Dragon Engine", "Dwarven Weaponsmith", "Energy Flux",
    "Feldon's Cane", "Gaea's Avenger", "Gate to Phyrexia", "Goblin Artisans",
    "Golgothian Sylex", "Grapeshot Catapult", "Haunting Wind", "Hurkyl's Recall",
    "Ivory Tower", "Jalum Tome", "Martyrs of Korlis", "Mightstone", "Millstone",
    "Mishra's Factory", "Mishra's War Machine", "Mishra's Workshop", "Obelisk of Undoing",
    "Onulet", "Orcish Mechanics", "Ornithopter", "Phyrexian Gremlins", "Power Artifact",
    "Powerleech", "Priest of Yawgmoth", "Primal Clay", "Rakalite", "Reconstruction",
    "Reverse Polarity", "Rocket Launcher", "Sage of Lat-Nam", "Shapeshifter", "Shatterstorm",
    "Staff of Zegon", "Strip Mine", "Su-Chi", "Tablet of Epityr", "Tawnos's Coffin",
    "Tawnos's Wand", "Tawnos's Weaponry", "Tetravus", "The Rack", "Titania's Song",
    "Transmute Artifact", "Triskelion", "Urza's Avenger", "Urza's Chalice", "Urza's Mine",
    "Urza's Miter", "Urza's Power Plant", "Urza's Tower", "Wall of Spears", "Weakstone",
    "Xenic Poltergeist", "Yawgmoth Demon", "Yotian Soldier",
];

/// Homelands' 115 distinct card names, sorted for `binary_search`.
static HOMELANDS_NAMES: [&str; 115] = [
    "Abbey Gargoyles", "Abbey Matron", "Aether Storm", "Aliban's Tower", "Ambush", "Ambush Party",
    "An-Havva Constable", "An-Havva Inn", "An-Havva Township", "An-Zerrin Ruins", "Anaba Ancestor",
    "Anaba Bodyguard", "Anaba Shaman", "Anaba Spirit Crafter", "Apocalypse Chime", "Autumn Willow",
    "Aysen Abbey", "Aysen Bureaucrats", "Aysen Crusader", "Aysen Highway", "Baki's Curse", "Baron Sengir",
    "Beast Walkers", "Black Carriage", "Broken Visage", "Carapace", "Castle Sengir", "Cemetery Gate",
    "Chain Stasis", "Chandler", "Clockwork Gnomes", "Clockwork Steed", "Clockwork Swarm", "Coral Reef",
    "Dark Maze", "Daughter of Autumn", "Death Speakers", "Didgeridoo", "Drudge Spell", "Dry Spell",
    "Dwarven Pony", "Dwarven Sea Clan", "Dwarven Trader", "Ebony Rhino", "Eron the Relentless",
    "Evaporate", "Faerie Noble", "Feast of the Unicorn", "Feroz's Ban", "Folk of An-Havva", "Forget",
    "Funeral March", "Ghost Hounds", "Giant Albatross", "Giant Oyster", "Grandmother Sengir",
    "Greater Werewolf", "Hazduhr the Abbot", "Headstone", "Heart Wolf", "Hungry Mist", "Ihsan's Shade",
    "Irini Sengir", "Ironclaw Curse", "Jinx", "Joven", "Joven's Ferrets", "Joven's Tools", "Koskun Falls",
    "Koskun Keep", "Labyrinth Minotaur", "Leaping Lizard", "Leeches", "Mammoth Harness", "Marjhan",
    "Memory Lapse", "Merchant Scroll", "Mesa Falcon", "Mystic Decree", "Narwhal", "Orcish Mine",
    "Primal Order", "Prophecy", "Rashka the Slayer", "Reef Pirates", "Renewal", "Retribution",
    "Reveka, Wizard Savant", "Root Spider", "Roots", "Roterothopter", "Rysorian Badger", "Samite Alchemist",
    "Sea Sprite", "Sea Troll", "Sengir Autocrat", "Sengir Bats", "Serra Aviary", "Serra Bestiary",
    "Serra Inquisitors", "Serra Paladin", "Serrated Arrows", "Shrink", "Soraya the Falconer",
    "Spectral Bears", "Timmerian Fiends", "Torture", "Trade Caravan", "Truce", "Veldrane of Sengir",
    "Wall of Kelp", "Willow Faerie", "Willow Priestess", "Winter Sky", "Wizards' School",
];

/// Arabian Nights' 78 distinct card names, sorted for `binary_search`.
static ARABIAN_NIGHTS_NAMES: [&str; 78] = [
    "Abu Ja'far", "Aladdin", "Aladdin's Lamp", "Aladdin's Ring", "Ali Baba", "Ali from Cairo",
    "Army of Allah", "Bazaar of Baghdad", "Bird Maiden", "Bottle of Suleiman", "Brass Man",
    "Camel", "City in a Bottle", "City of Brass", "Cuombajj Witches", "Cyclone",
    "Dancing Scimitar", "Dandân", "Desert", "Desert Nomads", "Desert Twister",
    "Diamond Valley", "Drop of Honey", "Ebony Horse", "El-Hajjâj", "Elephant Graveyard",
    "Erg Raiders", "Erhnam Djinn", "Eye for an Eye", "Fishliver Oil", "Flying Carpet",
    "Flying Men", "Ghazbán Ogre", "Giant Tortoise", "Guardian Beast", "Hasran Ogress",
    "Hurr Jackal", "Ifh-Bíff Efreet", "Island Fish Jasconius", "Island of Wak-Wak",
    "Jandor's Ring", "Jandor's Saddlebags", "Jeweled Bird", "Jihad", "Junún Efreet",
    "Juzám Djinn", "Khabál Ghoul", "King Suleiman", "Kird Ape", "Library of Alexandria",
    "Magnetic Mountain", "Merchant Ship", "Metamorphosis", "Mijae Djinn", "Moorish Cavalry",
    "Mountain", "Nafs Asp", "Oasis", "Old Man of the Sea", "Oubliette", "Piety", "Pyramids",
    "Repentant Blacksmith", "Ring of Ma'rûf", "Rukh Egg", "Sandals of Abdallah", "Sandstorm",
    "Serendib Djinn", "Serendib Efreet", "Shahrazad", "Sindbad", "Singing Tree",
    "Sorceress Queen", "Stone-Throwing Devils", "Unstable Mutation", "War Elephant",
    "Wyluli Wolf", "Ydwen Efreet",
];

fn one_u32() -> u32 { 1 }

/// CR 601.2b/601.2f — an additional cost paid as the spell is cast, listed
/// in `CardDefinition.additional_cast_cost`. Determined and paid during
/// casting; the spell can't be cast unless every cost is payable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]

pub enum AdditionalCastCost {
    /// "As an additional cost to cast this spell, sacrifice [count] [filter]."
    /// The first sacrificed permanent's power becomes the spell's X (read at
    /// resolution via `Value::XFromCost`) — powers Tend the Pests. `count`
    /// defaults to 1 for snapshots/cards predating the field.
    SacrificePermanent {
        filter: SelectionRequirement,
        #[serde(default = "one_u32")]
        count: u32,
    },
    /// "As an additional cost to cast this spell, you may sacrifice one or
    /// more [filter]" (Plumb the Forbidden). The caster communicates the
    /// chosen count as the cast's `x_value`; the cast pipeline rewrites
    /// this into `SacrificePermanent { count: x }` before payment, so
    /// downstream payment/trigger machinery is shared.
    SacrificeAnyNumber { filter: SelectionRequirement },
    /// "As an additional cost to cast this spell, sacrifice all [filter]"
    /// (Soulblast). Concretized to a `SacrificePermanent` of the live count at
    /// cast time.
    SacrificeAll { filter: SelectionRequirement },
    /// "As an additional cost, discard N card(s)." When `filter` is set, the
    /// discard is restricted to matching cards ("discard a land card" —
    /// Magmatic Insight); `None` allows any card (Big Score, Illuminate
    /// History). The cast is rejected if the hand holds fewer than `count`
    /// matching cards.
    Discard {
        count: u32,
        #[serde(default)]
        filter: Option<SelectionRequirement>,
    },
    /// "As an additional cost to cast this spell, discard N cards at random"
    /// (Acceptable Losses). The caster gets no pick, so the cost never
    /// suspends.
    DiscardRandom { count: u32 },
    /// "Discard X cards", where X is the cast's chosen X (Conflagrate's
    /// flashback). Concretized to a `Discard` of that count at cast time.
    DiscardXFromCost,
    /// "Discard X cards at random", where X is the cast's chosen X
    /// (Devastating Dreams). Concretized to a `DiscardRandom` at cast time.
    DiscardXRandomFromCost,
    /// "As an additional cost to cast this spell, return N permanent(s) you
    /// control matching `filter` to their owner's hand." Devour in Flames
    /// ("return a land you control"). Auto-picker bounces the lowest-impact
    /// matches (tapped first, then lowest mana value).
    ReturnToHand {
        filter: SelectionRequirement,
        #[serde(default = "one_u32")]
        count: u32,
    },
    /// "As an additional cost to cast this spell, exile [count] [filter]
    /// card(s) from your graveyard." The first (lowest-MV) exiled card's mana
    /// value becomes the spell's X (read at resolution via `Value::XFromCost`)
    /// — Draconic Intervention ("X is the exiled card's mana value"). Auto-picker
    /// exiles the lowest-MV matches. Cast is rejected if fewer than `count`
    /// matching cards are in the graveyard. `count` defaults to 1 for cards /
    /// snapshots predating the field (Abhorrent Oculus exiles six).
    ExileFromGraveyard {
        filter: SelectionRequirement,
        #[serde(default = "one_u32")]
        count: u32,
    },
    /// "As an additional cost to cast this spell, exile [count] [filter]
    /// card(s) from your graveyard or pay [pay]." SOS Soaring Stoneglider
    /// ("exile two cards from your graveyard or pay {1}{W}"). The
    /// registered cost — and therefore the card's mana value — stays at
    /// the printed base. With enough matching graveyard cards the exile
    /// half is paid (lowest-MV picks, reusing the `ExileFromGraveyard`
    /// machinery); otherwise `pay` joins the spell's cost
    /// symbol-for-symbol (colored pips included — see
    /// `or_pay_cost_symbols`). A "which half?" chooser for `wants_ui`
    /// seats is a follow-up, same as `SacrificeOrPay`.
    ExileFromGraveyardOrPay {
        filter: SelectionRequirement,
        #[serde(default = "one_u32")]
        count: u32,
        pay: crate::mana::ManaCost,
    },
    /// "As an additional cost to cast this spell, reveal a [filter] card from
    /// your hand or pay {pay}." Silvergill Adept. When the caster's hand
    /// (excluding the spell itself) holds a matching card the cost is free
    /// (the reveal is knowledge-only); otherwise `pay` generic is added to
    /// the spell's cost via `extra_cost_for_spell`.
    RevealFromHandOrPay {
        filter: SelectionRequirement,
        pay: u32,
    },
    /// "As an additional cost to cast this spell, reveal a [filter] card from
    /// your hand." The card stays in hand; its power is stamped for the body to
    /// read via `Value::RevealedForCostPower` (Titan's Presence). Announcing is
    /// gated on having a match.
    RevealFromHand { filter: SelectionRequirement },
    /// "As an additional cost to cast this spell, put a card an opponent
    /// owns from exile into that player's graveyard." (Process — Processor
    /// Assault.)
    ProcessExile,
    /// "As an additional cost to cast this spell, sacrifice a [filter] or
    /// pay {pay}." (Bayou Groff.) When the caster controls a matching
    /// permanent the sacrifice half is paid (auto-picking the cheapest
    /// match); otherwise `pay` generic joins the cost via
    /// `extra_cost_for_spell`. A "which half?" chooser for `wants_ui`
    /// seats is a follow-up.
    SacrificeOrPay {
        filter: SelectionRequirement,
        pay: u32,
    },
    /// "As an additional cost to cast this spell, tap N untapped permanents you
    /// control matching `filter`." Guardian of the Great Door ("tap four
    /// untapped artifacts, creatures, and/or lands you control"). The cast is
    /// rejected unless that many untapped matches exist; the auto-picker taps
    /// the lowest-impact ones (non-mana sources, then lowest mana value).
    TapPermanents {
        filter: SelectionRequirement,
        count: u32,
    },
    /// "As an additional cost to cast this spell, pay N life." (CR 119.4 —
    /// payable only if life ≥ N.) Deep Analysis's "Flashback—{1}{U}, Pay 3
    /// life". Paid immediately during casting, so any "loses life" watcher
    /// fires before the spell resolves.
    PayLife { amount: u32 },
    /// "As an additional cost to cast this spell, exile [count] [filter]
    /// permanent(s) you control." (Necrotic Fumes.) Auto-picker exiles the
    /// cheapest matches (tokens first, then lowest mana value); a `wants_ui`
    /// caster is prompted like a sacrifice pick.
    ExilePermanent {
        filter: SelectionRequirement,
        #[serde(default = "one_u32")]
        count: u32,
    },
    /// "As an additional cost, sacrifice a [filter] or pay N life." (Final
    /// Payment.) Auto path: sacrifice a matching token if one exists, else
    /// pay the life when affordable, else sacrifice the cheapest match.
    SacrificeOrPayLife {
        filter: SelectionRequirement,
        life: u32,
    },
    /// "As an additional cost to cast this spell, choose a creature you control
    /// or reveal a creature card from your hand." (Monstrous Emergence.) The
    /// chosen/revealed creature's power is threaded into the spell's X (read via
    /// `Value::XFromCost`); nothing is sacrificed or moved. Rejected if the
    /// caster controls no creature and reveals none. Auto-picks the highest
    /// power (battlefield first, then hand).
    ChooseOrRevealCreature,
    /// "As an additional cost to cast this spell, forage or pay {pay}." (Feed
    /// the Cycle.) When the caster can forage (three cards in the graveyard or a
    /// Food to sacrifice) the forage half is paid; otherwise `pay` generic joins
    /// the cost via `extra_cost_for_spell`. CR 701.61.
    ForageOrPay { pay: u32 },
    /// "As an additional cost, (you may) collect evidence N" (CR 701.59 — exile
    /// cards with total mana value ≥ `amount` from your graveyard). When
    /// `optional` the cost may be skipped; whether it was paid is read at
    /// resolution via `Predicate::SpellCollectedEvidence` (Behind the Mask's
    /// 1/1-instead branch) and can grant a `self_cost_reduction_if_collect_evidence`
    /// discount (Bite Down on Crime). The auto-decider collects whenever the
    /// graveyard can afford it.
    CollectEvidence {
        amount: u32,
        #[serde(default)]
        optional: bool,
    },
    /// "As an additional cost, an opponent gains N life" (Roar of Jukai's
    /// splice cost). Always payable; the caster's first opponent gains it.
    OpponentGainsLife { amount: u32 },
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
    /// Keywords REMOVED from the equipped creature (layer 6 — CR 613.1f).
    /// "Enchanted creature loses flying" (Sky Tether) and similar auras.
    #[serde(default)]
    pub remove_keywords: Vec<Keyword>,
    /// Keywords granted to the equipped creature only during the source's
    /// controller's turn (layer 6, gated on active player). Dragoon's Lance —
    /// "During your turn, equipped creature has flying."
    #[serde(default)]
    pub during_your_turn_keywords: Vec<Keyword>,
    /// "During your turn, equipped creature gets +P/+T" (layer 7c, gated on the
    /// source controller's turn — Knife's "+1/+0" half). `(0, 0)` for the
    /// common always-on case.
    #[serde(default)]
    pub during_your_turn_pt: (i32, i32),
    /// "Equipped/enchanted creature gets +P/+T while `condition` holds" (layer
    /// 7c, gated on a predicate evaluated against the source's controller —
    /// Taste for Mayhem's Hellbent `+2/+0`). `(0, 0, _)` is unused; `None` for
    /// the common unconditional case.
    #[serde(default)]
    pub conditional_pt: Option<(i32, i32, crate::effect::Predicate)>,
    /// Optional board-count scaling (CR 613 layer 7c): the attached creature
    /// gets an additional `per_power`/`per_toughness` for each permanent
    /// matching `filter` the source's controller controls, on top of the flat
    /// `power`/`toughness` (Nettlecyst — "+1/+1 for each artifact and/or
    /// enchantment you control"). `None` for the common static-bonus case.
    #[serde(default)]
    pub scale: Option<EquipScale>,
    /// CR 702.16k — "This effect doesn't remove Auras" (Spectra Ward). The
    /// protection this bonus grants doesn't shed Auras attached to the host,
    /// so the 704.5m sweep skips that host entirely.
    #[serde(default)]
    pub protection_keeps_auras: bool,
    /// CR 702.16k — "This effect doesn't remove this Aura" (Pledge of
    /// Loyalty): the narrower sibling of `protection_keeps_auras`, exempting
    /// only the granting Aura itself from the 704.5m shed.
    #[serde(default)]
    pub protection_keeps_self: bool,
    /// Triggered abilities granted to the equipped creature (CR 702.6e). Each
    /// fires as though printed on the equipped creature — `EventScope::
    /// SelfSource` reads the creature, and a `DealsCombatDamageToPlayer` body
    /// can refer to the damaged player via `PlayerRef::Target(0)`. The Sword
    /// cycle's combat-damage triggers. Empty for the common static-bonus case.
    #[serde(default)]
    pub triggered_abilities: Vec<crate::effect::TriggeredAbility>,
    /// Activated abilities granted to the equipped creature (CR 702.6e). Each
    /// resolves as though printed on the creature — its `{T}` taps the
    /// creature, `Selector::This` reads the creature. Surfaced through
    /// `granted_abilities_for` alongside the Soulbond/static grants (Wrench's
    /// "{3}, {T}: Tap target creature"). Empty for the common static case.
    #[serde(default)]
    pub activated_abilities: Vec<crate::effect::ActivatedAbility>,
    /// When true, `triggered_abilities` resolve with the **Equipment** as the
    /// trigger source (so `Selector::This` reads the Equipment, not the equipped
    /// creature). Umezawa's Jitte's "whenever equipped creature deals combat
    /// damage, put two charge counters on this Equipment". `false` keeps the
    /// default CR 702.6e behavior (the ability fires off the creature).
    #[serde(default)]
    pub triggers_on_equipment: bool,
    /// Host-conditional riders: each entry applies only while the attached
    /// creature matches its filter ("As long as enchanted creature is green,
    /// it gets +1/+1 and has indestructible" — Shield of the Oversoul,
    /// Steel of the Godhead).
    #[serde(default)]
    pub conditional: Vec<ConditionalEquipBonus>,
    /// Continuous characteristic overrides applied to the host while attached
    /// (the "becomes a 0/1 Fish with no abilities" auras — Ichthyomorphosis,
    /// One with the Stars). Each `Some`/`true` installs the matching layer
    /// modification on the host. `None`/`false` leaves that characteristic
    /// untouched (the common stat-bonus Equipment/Aura case).
    #[serde(default)]
    pub set_base_pt: Option<(i32, i32)>,
    /// When true, the host's base power and toughness are both set (layer 7b)
    /// to the Equipment controller's life total — "equipped creature has base
    /// power and toughness X/X, where X is your life total" (Aettir and Priwen).
    /// Recomputed each layer pass, so it tracks life changes live.
    #[serde(default)]
    pub set_base_pt_controller_life: bool,
    #[serde(default)]
    pub set_card_types: Option<Vec<CardType>>,
    #[serde(default)]
    pub set_creature_types: Option<Vec<CreatureType>>,
    /// Creature types the host gains *in addition* to its own (the "enchanted
    /// creature is an X in addition to its other types" auras — Avatar Destiny's
    /// "is an Avatar"). Additive layer-4, distinct from the `set_creature_types`
    /// override.
    #[serde(default)]
    pub add_creature_types: Vec<CreatureType>,
    /// Land types the host's type line becomes (the "is a Forest land" auras —
    /// Song of the Dryads). Pairs with `set_card_types: Some([Land])` so the
    /// intrinsic basic-land mana ability follows the granted type.
    #[serde(default)]
    pub set_land_types: Option<Vec<LandType>>,
    /// Artifact subtypes the host's type line becomes (the "is a Food artifact"
    /// auras — Sugar Coat). Pairs with `set_card_types: Some([Artifact])`.
    #[serde(default)]
    pub set_artifact_types: Option<Vec<ArtifactSubtype>>,
    #[serde(default)]
    pub set_colors: Option<Vec<crate::mana::Color>>,
    /// When true the host loses all abilities (layer 6 — CR 613.1f).
    #[serde(default)]
    pub remove_abilities: bool,
}

/// One "as long as enchanted creature is [filter]" rider of an
/// [`EquipBonus`]. Evaluated against the host each layer pass.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionalEquipBonus {
    pub host_filter: SelectionRequirement,
    pub power: i32,
    pub toughness: i32,
    #[serde(default)]
    pub keywords: Vec<Keyword>,
    /// Extra game-state gate ("Corrupted — enchanted creature gets an
    /// additional +1/+0 and has first strike" — Zealot's Conviction).
    /// Evaluated against the bonus source's controller each layer pass.
    #[serde(default)]
    pub condition: Option<crate::effect::Predicate>,
    /// Activated abilities the host gains only while `host_filter` matches
    /// ("as long as enchanted creature is a Wizard, it has …" — Lavamancer's
    /// Skill). Surfaced alongside `EquipBonus.activated_abilities`.
    #[serde(default)]
    pub activated_abilities: Vec<crate::effect::ActivatedAbility>,
}

/// CR 702.95 — the bonus each member of a Soulbond pair gains while paired.
/// Stored on `CardDefinition.soulbond_bonus` of the card that carries the
/// Soulbond keyword; `gather_continuous_effects` applies it to BOTH paired
/// creatures while the link is live, and `granted_abilities_for` surfaces
/// `activated_abilities` on both (Deadeye Navigator's flicker).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoulbondBonus {
    /// Power each paired creature gains (layer 7c).
    pub power: i32,
    /// Toughness each paired creature gains (layer 7c).
    pub toughness: i32,
    /// Keywords each paired creature gains (layer 6).
    pub keywords: Vec<Keyword>,
    /// Activated abilities each paired creature gains, surfaced through
    /// `granted_abilities_for` just like `StaticEffect::GrantActivatedAbility`.
    #[serde(default)]
    pub activated_abilities: Vec<crate::effect::ActivatedAbility>,
    /// Triggered abilities each paired creature gains (CR 702.6e-style grant).
    /// A `DealsCombatDamageToPlayer` one fires off either paired creature via
    /// the combat hook — Tandem Lookout's "deals combat damage → draw a card".
    #[serde(default)]
    pub triggered_abilities: Vec<crate::effect::TriggeredAbility>,
}

/// Board-count scaling for an [`EquipBonus`] (CR 613 layer 7c). See
/// `EquipBonus.scale`. `Default` filter is the match-anything `Any` (used by
/// the counter/graveyard-count variants, which ignore `filter`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipScale {
    pub filter: SelectionRequirement,
    pub per_power: i32,
    pub per_toughness: i32,
    /// When set, the count is the number of counters of this kind on the
    /// source permanent itself (Lion Sash — "+1/+1 for each +1/+1 counter on
    /// this Equipment") rather than the controlled-permanent count by `filter`.
    #[serde(default)]
    pub count_self_counters: Option<CounterType>,
    /// When set, the count is the number of cards matching this filter in the
    /// source's controller's graveyard (Avatar Destiny — "+1/+1 for each
    /// creature card in your graveyard") rather than the battlefield count.
    #[serde(default)]
    pub count_graveyard: Option<SelectionRequirement>,
    /// When set, the count is the number of cards matching this filter across
    /// *every* player's graveyard (Bonehoard — "+X/+X where X is the number of
    /// creature cards in all graveyards"), rather than just the controller's.
    #[serde(default)]
    pub count_all_graveyards: Option<SelectionRequirement>,
    /// When true, the count is the number of *other* creatures the host's
    /// controller controls that share a creature type with the host
    /// (Stoneforge Masterwork). Changelings share every type (CR 702.73a).
    #[serde(default)]
    pub count_sharing_type_with_host: bool,
    /// When true, the count is the number of colors of the attached host itself
    /// (Blessing of the Nephilim — "+1/+1 for each of its colors"), rather than
    /// any board/graveyard count.
    #[serde(default)]
    pub count_host_colors: bool,
    /// When set, the count is the number of permanents matching this filter
    /// attached to the *host* (Golem-Skin Gauntlets — "+1/+0 for each
    /// Equipment attached to it"), rather than a controlled-permanent count.
    #[serde(default)]
    pub count_host_attachments: Option<SelectionRequirement>,
    /// When true, the attached host is excluded from the `filter` count —
    /// the printed "for each **other** creature you control" (Bravado).
    #[serde(default)]
    pub exclude_host: bool,
    /// When true, the `filter` count spans *every* battlefield rather than
    /// just the source controller's — the printed "for each other enchantment
    /// on the battlefield" (Ancestral Mask).
    #[serde(default)]
    pub count_all_controllers: bool,
    /// When true, the Aura/Equipment itself is excluded from the `filter`
    /// count — the "**other**" in Ancestral Mask's "each other enchantment".
    #[serde(default)]
    pub exclude_source: bool,
    /// When true, the count is the number of cards in the *host creature's
    /// controller's* hand (Righteous Authority — "+1/+1 for each card in its
    /// controller's hand"), rather than the source-controller counts above.
    #[serde(default)]
    pub count_host_controller_hand: bool,
    /// When true, the count is the number of cards in the *source's*
    /// controller's hand — "gets -X/-X, where X is the number of cards in
    /// your hand" (Kagemaro's Clutch). The your-side sibling of
    /// `count_host_controller_hand`.
    #[serde(default)]
    pub count_source_controller_hand: bool,
    /// When set, the count is the number of cards matching this filter in the
    /// *host creature's controller's* graveyard (Death's Approach — "-X/-X
    /// where X is the number of creature cards in its controller's graveyard").
    #[serde(default)]
    pub count_host_controller_graveyard: Option<SelectionRequirement>,
}

/// Characteristic-defining dynamic P/T formula. Read by
/// `compute_battlefield` to inject a layer-7 `SetPowerToughness` for
/// the named card. Each variant encodes both the power and toughness
/// expression so the engine doesn't have to know the printed Oracle's
/// wording — just the two scalars to set.
///
/// Stored on `CardDefinition.dynamic_pt`; adding a new dynamic-P/T card
/// sets that field (no engine-side table).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicPt {
    /// The general CR 604.3 CDA: power = `base_p` + the number of permanents
    /// matching `filter` the controller controls, toughness = `base_t` + that
    /// count. Squelching Leeches (`*/*` = Swamps you control).
    PermanentsControlledMatching {
        base_p: i32,
        base_t: i32,
        filter: Box<SelectionRequirement>,
    },
    /// Power is the fixed `base_p`; toughness = `base_t` + the number of
    /// permanents matching `filter` the controller controls. The `*`-toughness
    /// sibling of `PermanentsControlledMatching` (Treefolk Seedlings, `2/*`).
    PermanentsControlledMatchingToughness {
        base_p: i32,
        base_t: i32,
        filter: Box<SelectionRequirement>,
    },
    /// Power is the fixed `base_p`; toughness = `base_t` + the number of
    /// permanents matching `filter` on the battlefield, under **any** control
    /// (An-Havva Constable, `2/1+*`). The board-wide sibling of
    /// `PermanentsControlledMatchingToughness`.
    PermanentsOnBattlefieldMatchingToughness {
        base_p: i32,
        base_t: i32,
        filter: Box<SelectionRequirement>,
    },
    /// `base_p`/`base_t` plus the number of permanents matching `filter` the
    /// controller's *opponents* control (Gaea's Avenger).
    BasePlusOpponentsMatching {
        base_p: i32,
        base_t: i32,
        filter: Box<SelectionRequirement>,
    },
    /// Power = toughness = the controller's life total (Serra Avatar).
    ControllerLife,
    /// Power = toughness = the number stamped on this permanent as it entered
    /// (`CardInstance.chosen_number`) — Nameless Race's paid life.
    ChosenNumberAsEntered,
    /// `inner` during the controller's turn, `base_p`/`base_t` on every other
    /// turn (Angry Mob).
    OnlyDuringYourTurn { inner: Box<DynamicPt>, base_p: i32, base_t: i32 },
    /// `base_p`/`base_t` plus one for each untapped permanent the controller's
    /// *opponents* control (Copperhoof Vorrac).
    BasePlusOpponentsUntappedPermanents { base_p: i32, base_t: i32 },
    /// Power = N, toughness = N+1 where N is the count of distinct card
    /// types across every player's graveyard. Tarmogoyf, Cosmogoyf.
    DistinctTypesInAllGraveyards,
    /// Power = the controller's devotion to `color` (CR 700.5), with a
    /// fixed printed toughness. Anax, Hardened in the Forge (`*/3`).
    DevotionTo { color: crate::mana::Color, base_t: i32 },
    /// Power = `base_p` + toughness = `base_t` + the amount of unspent
    /// `color` mana in the controller's pool. Omnath, Locus of Mana (1/1 +
    /// green mana). Live-recomputes as mana is added/spent (CR 604.3).
    BasePlusUnspentColorMana { base_p: i32, base_t: i32, color: crate::mana::Color },
    /// Toughness = the controller's devotion to `color`, with a fixed
    /// printed power. Daxos, Blessed by the Sun (`2/*`).
    DevotionToToughness { color: crate::mana::Color, base_p: i32 },
    /// Power = toughness = size of the controller's graveyard. Cruel
    /// Somnophage.
    ControllerGraveyardSize,
    /// Power = toughness = `base` + the number of creature cards in the
    /// controller's graveyard. Fiend Artisan (base 1/1).
    BasePlusCreaturesInControllerGraveyard { base: i32 },
    /// Fathomless descent (LCI) — power = `base_p` + the number of permanent
    /// cards in the controller's graveyard, toughness = `base_t` + that count.
    /// Souls of the Lost (*/*+1 → base_p 0, base_t 1).
    PermanentCardsInControllerGraveyard { base_p: i32, base_t: i32 },
    /// Power = the number of cards the controller owns in exile, toughness =
    /// that number + `base_t`. Cosmogoyf (*/*+1 → base_t 1).
    CardsYouOwnInExile { base_t: i32 },
    /// Power = toughness = the number of creatures the controller controls
    /// that have any of `types`. The Mycotyrant (Fungi and/or Saprolings).
    CreaturesYouControlWithTypes { types: Vec<CreatureType> },
    /// Power = the number of creatures the controller controls (including
    /// itself), with a fixed printed toughness. Snow Villiers (`*/3`).
    CreaturesYouControl { base_t: i32 },
    /// Power = toughness = `base` + the number of *other* creatures the
    /// controller controls that have flying. Skycat Sovereign (base 1/1).
    /// Reads printed flying (granted flying isn't counted).
    BasePlusOtherFlyersControlled { base: i32 },
    /// Power = toughness = the number of creatures on the battlefield, any
    /// controller. Beast of Burden.
    AllCreaturesOnBattlefield,
    /// "Power and toughness are each equal to the total mana value of other
    /// creatures you control" (Ancient Ooze).
    TotalManaValueOfOtherControlledCreatures,
    /// Power = toughness = the total number of cards in ALL players' hands.
    /// Multani, Maro-Sorcerer.
    AllPlayersHandTotal,
    /// Power = toughness = `base` + the number of *other* creatures on the
    /// battlefield with flying, any controller. Radiant, Archangel.
    BasePlusOtherFlyersOnBattlefield { base: i32 },
    /// Power = toughness = base + total land cards in all graveyards.
    /// Knight of the Reliquary (base 2/2; grows +1/+1 per land in any
    /// player's graveyard).
    BasePlusLandsInAllGraveyards { base_p: i32, base_t: i32 },
    /// Power = base_p + creature cards in all graveyards; toughness = base_t +
    /// that count. Lhurgoyf (0/1+*), Mortivore (0/0 + {B} regenerate).
    CreatureCardsInAllGraveyards { base_p: i32, base_t: i32 },
    /// Power = creature cards in all graveyards; toughness is the fixed
    /// `base_t`. Necrogoyf (`*`/4).
    CreatureCardsInAllGraveyardsPower { base_t: i32 },
    /// CR 604.3 — "power and toughness are each equal to the number of
    /// [card type] cards in all graveyards" (the Odyssey Lhurgoyf cycle:
    /// Cantivore, Cognivore, Magnivore, Terravore).
    CardTypeInAllGraveyards(CardType),
    /// Power = toughness = base + land cards in the *controller's*
    /// graveyard. Wight of the Reliquary (base 1/1, +1/+1 per land in
    /// your graveyard).
    BasePlusLandsInControllerGraveyard { base_p: i32, base_t: i32 },
    /// Power = base_p − controller's life total, toughness = base_t − life.
    /// Death's Shadow (13/13 that "gets −X/−X where X is your life total").
    BaseMinusControllerLife { base_p: i32, base_t: i32 },
    /// Power = number of colorless creatures the controller controls,
    /// toughness = `base_t`. Vile Aggregate (*/5). "Colorless" is read as
    /// Devoid-or-no-colored-pips, covering every printed colorless creature.
    ColorlessCreaturesControlled { base_t: i32 },
    /// Power = toughness = `base` + the number of creatures the controller
    /// controls (counting the source itself). Burrowguard Mentor (base 0/0).
    CreaturesControlled { base: i32 },
    /// `*`/`*` equal to the number of permanents of the source's ETB-chosen
    /// colour that its controller's opponents control (Chameleon Spirit).
    PermanentsOfChosenColorOpponentsControl,
    /// Power = `base_p` + the number of creatures the controller controls;
    /// toughness is the fixed `base_t` (`*`/N creatures — Suki, Kyoshi
    /// Warrior `*`/4). The power-only sibling of `CreaturesControlled`.
    CreaturesControlledPower { base_p: i32, base_t: i32 },
    /// Power = `base_p` + the total +1/+1 counters on lands the controller
    /// controls; toughness is the fixed `base_t` (Toph, the Blind Bandit
    /// `*`/3, tracking earthbent lands).
    PlusCountersOnLandsControlledPower { base_p: i32, base_t: i32 },
    /// Power = toughness = the number of creatures of `creature_type` the
    /// controller controls (counting the source). Pack Rat.
    CreaturesOfTypeControlled { creature_type: CreatureType },
    /// Power = toughness = the number of creatures on the battlefield (any
    /// controller) sharing the source's `chosen_creature_type`. Caller of
    /// the Hunt.
    CreaturesOfSourceChosenType,
    /// Power = toughness = the number of `creature_type` creatures on the
    /// battlefield, any controller; `also_graveyards` adds the matching
    /// creature *cards* in every graveyard. The Onslaught Avatar cycle
    /// (Doubtless One, Heedless One, Nameless One, Reckless One, Soulless One).
    CreaturesOfTypeOnBattlefield {
        creature_type: CreatureType,
        also_graveyards: bool,
    },
    /// Power = the number of lands of `land_type` on the battlefield (any
    /// controller); toughness = `base_t`. Coiling Woodworm (`*`/1).
    LandsOfTypeInPlayPower { land_type: LandType, base_t: i32 },
    /// Power = `base_p` + the controller's experience counters; toughness =
    /// `base_t` + that count. Daxos the Returned's Spirit token (0/0 base) and
    /// Kalemne, Disciple of Iroas (2/4 base, "+1/+1 for each experience").
    ControllerExperience { base_p: i32, base_t: i32 },
    /// Power = toughness = `base` + the number of lands the controller
    /// controls. Lumra, Bellow of the Woods (base 0/0).
    LandsControlled { base: i32 },
    /// Power = `base_p` + the lands the controller controls; toughness is
    /// the fixed `base_t`. `*`-power Vehicles whose power tracks lands
    /// while toughness is printed (Lumbering Worldwagon, `*/8`).
    LandsControlledPower { base_p: i32, base_t: i32 },
    /// Power = toughness = `base` + lands the controller controls + land cards
    /// in the controller's graveyard. Multani, Yavimaya's Avatar (base 0/0).
    LandsControlledPlusLandsInControllerGraveyard { base: i32 },
    /// Power = `base_p` + the number of card types among cards in the
    /// controller's *opponents'* graveyards; toughness is the fixed `base_t`.
    /// Nighthawk Scavenger (1+*/3).
    CardTypesInOpponentsGraveyards { base_p: i32, base_t: i32 },
    /// Power = `base_p` + the number of card types among cards in the
    /// *controller's own* graveyard; toughness = `base_t` + that count.
    /// Nethergoyf (*/1+* → base_p 0, base_t 1).
    CardTypesInControllerGraveyard { base_p: i32, base_t: i32 },
    /// Power = toughness = the number of cards in the controller's hand.
    /// Maro, Kagemaro.
    ControllerHandSize,
    /// Power = toughness = `factor` × the cards in the controller's hand.
    /// Masumaro, First to Live (twice your hand size).
    ControllerHandSizeTimes { factor: i32 },
    /// Base `base_p`/`base_t`, then −`per`/−`per` for each card in the
    /// controller's hand. Dread Slag (9/9, −4/−4 per card in your hand).
    BaseMinusPerCardInHand { base_p: i32, base_t: i32, per: i32 },
    /// Power = toughness = the size of the largest hand among the controller's
    /// opponents. Adamaro, First to Desire.
    MaxOpponentHandSize,
    /// Base `base_p`/`base_t`, then −1/−1 for each card in opponents' hands
    /// (Enemy of Enlightenment — "gets -1/-1 for each card in your
    /// opponents' hands").
    BaseMinusOpponentsHandTotal { base_p: i32, base_t: i32 },
    /// Power = toughness = `base` + the number of artifacts the controller
    /// controls (counting the source). Broodstar (base 0/0, CR 604.3 CDA).
    ArtifactsControlled { base: i32 },
    /// Power = `base_p` + the number of artifacts the controller controls;
    /// toughness fixed at `base_t` (Akiri, Line-Slinger — "+1/+0 for each
    /// artifact you control").
    ArtifactsControlledPower { base_p: i32, base_t: i32 },
    /// Power = `base_p` + the number of enchantments on the battlefield,
    /// toughness = `base_t` + that count. Yavimaya Enchantress (2/2 + one per
    /// enchantment in play, any controller; CR 604.3 CDA).
    EnchantmentsInPlay { base_p: i32, base_t: i32 },
    /// Power = toughness = `base` + the number of nonland permanents the
    /// controller controls. Regal Bunnicorn (`*/*`, CR 604.3 CDA).
    NonlandPermanentsControlled { base: i32 },
    /// Power = `base_p`; toughness = number of Forests on the battlefield (any
    /// controller). Traproot Kami (0/*, CR 604.3 CDA).
    ForestsInPlay { base_p: i32 },
    /// Power = number of instant and sorcery cards in the controller's
    /// graveyard and exile; toughness = `base_t`. Crackling Drake (0/4).
    InstantsSorceriesInGraveyardAndExile { base_t: i32 },
    /// Power = number of instant and sorcery cards in the controller's
    /// graveyard (only); toughness = `base_t`. Enigma Drake (*/4).
    InstantsSorceriesInControllerGraveyard { base_t: i32 },
    /// Power = number of noncreature, nonland cards in the controller's
    /// graveyard; toughness = `base_t`. Dragonfly Swarm (*/3).
    NoncreatureNonlandCardsInControllerGraveyard { base_t: i32 },
    /// P/T = `base_p`/`base_t`, each raised by the number of noncreature,
    /// nonland cards in the controller's graveyard (the +N/+N sibling of
    /// `NoncreatureNonlandCardsInControllerGraveyard`). Xande, Dark Mage.
    BasePlusNoncreatureNonlandInControllerGraveyard { base_p: i32, base_t: i32 },
    /// Power = `base_p` + the number of distinct colors among Ally creatures
    /// the controller controls; toughness = `base_t`. Earthen Ally (`*`/2).
    ColorsAmongAlliesControlledPower { base_p: i32, base_t: i32 },
    /// Imprint CDA (CR 604.3): P/T of the creature card exiled with this
    /// permanent; printed base when nothing is exiled. Duplicant.
    ExiledWithSourcePt { base_p: i32, base_t: i32 },
    /// P/T = base + `per` for each Aura attached to this permanent.
    /// Kor Spiritdancer (0/2, +2/+2 per Aura).
    BasePlusPerAttachedAura { base_p: i32, base_t: i32, per: i32 },
    /// P/T = base + `per` for each Equipment attached to this permanent.
    /// Goblin Gaveleer (1/1, +2/+0 per Equipment).
    BasePlusPerAttachedEquipment { base_p: i32, base_t: i32, per: i32 },
    /// P/T = base − the highest life total among players (Scourge of the
    /// Skyclaves: 20 − highest life).
    BaseMinusHighestLife { base_p: i32, base_t: i32 },
    /// Power = toughness = base + cards in opponents' graveyards, optionally
    /// creature cards only. Consuming Aberration (base 0, all cards), Wight
    /// of Precinct Six (base 1, creatures only).
    BasePlusOpponentGraveyards { base: i32, creatures_only: bool },
    /// P/T = base + 1/+1 for each land of `land_type` the controller controls.
    /// Outcaster Greenblade (1/2, +1/+1 per Desert).
    BasePlusLandsOfTypeControlled { land_type: LandType, base_p: i32, base_t: i32 },
    /// Base P/T plus +1/+0 per land of `land_type` you control (Tempest
    /// Djinn's "+1/+0 for each basic Island").
    PowerPlusLandsOfTypeControlled { land_type: LandType, base_p: i32, base_t: i32 },
    /// Power = `base_p` + the greatest mana value among *other* artifacts the
    /// controller controls; toughness = the fixed `base_t`. Models "gets +X/+0,
    /// where X is the greatest mana value among other artifacts you control"
    /// (Emissary Escort, base 0/4).
    BasePlusGreatestOtherArtifactMv { base_p: i32, base_t: i32 },
    /// P/T = base + `per_p`/`per_t` for each `counter_type` counter on this
    /// permanent itself. "+1/+1 per oil" (Evolving Adaptive, per 1/1) or
    /// "+1/+0 per oil" (Exuberant Fuseling, per 1/0). `per_*` default to 1.
    BasePlusCountersOnSelf {
        counter_type: CounterType,
        base_p: i32,
        base_t: i32,
        #[serde(default = "one_i32")]
        per_p: i32,
        #[serde(default = "one_i32")]
        per_t: i32,
    },
    /// Power = the number of cards the controller has drawn this turn;
    /// toughness = the fixed `base_t`. Duelist of the Mind (`*`/3, CR 604.3).
    CardsDrawnThisTurnPower { base_t: i32 },
    /// Power = toughness = `mult` × the number of instant and sorcery cards in
    /// the controller's graveyard. Melek, Reforged Researcher (`*/*`, mult 2).
    InstantSorceryCardsInControllerGraveyard { mult: i32 },
    /// Power = the total power, toughness = the total toughness, of the cards
    /// exiled with this permanent (`CardInstance.exiled_with`). Sutured Ghoul.
    ExiledWithSourceTotals,
}

fn one_i32() -> i32 { 1 }

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
    /// CR 702.113 — this alternative cost is Awaken, so the card matches
    /// `SelectionRequirement::HasAwaken` (Halimar Tidecaller).
    #[serde(default)]
    pub awaken: bool,
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
    /// True for Dash (CR 702.110) alternative costs — the resulting
    /// creature gains haste and is returned to its owner's hand at the
    /// beginning of the next end step.
    #[serde(default)]
    pub dash: bool,
    /// True for Blitz (CR 702.152) alternative costs — the resulting creature
    /// gains haste and "When this creature dies, draw a card," and is
    /// sacrificed at the beginning of the next end step.
    #[serde(default)]
    pub blitz: bool,
    /// True when casting via this alternative cost grants the spell flash
    /// timing (e.g. Rout's "cast as though it had flash if you pay {2}
    /// more"). Bypasses the sorcery-speed gate the alt-cast path otherwise
    /// enforces on noninstant spells.
    #[serde(default)]
    pub flash: bool,
    /// True when paying this alternative cost stamps the resolving spell as
    /// "kicked" (`CardInstance.kicked`), so "if its [surge/...] cost was paid"
    /// ETB riders fire via `Predicate::SpellWasKicked`. Reuses the kicker
    /// pipeline for Surge (CR 702.108) — Reckless Bushwhacker, Tyrant of
    /// Valakut.
    #[serde(default)]
    pub marks_kicked: bool,
    /// CR 702.119 — Emerge. When `Some(filter)`, this alternative cost
    /// requires sacrificing a creature you control matching `filter`, and the
    /// `mana_cost` (the emerge cost) is reduced generically by that creature's
    /// mana value. The auto-picker sacrifices the highest-MV match (max
    /// reduction). Elder Deep-Fiend, Wretched Gryff, Distended Mindbender.
    #[serde(default)]
    pub emerge: Option<SelectionRequirement>,
    /// CR 702.183 — Impending N. When non-zero, casting via this alternative
    /// cost stamps the resolving permanent with N time counters
    /// (`CardInstance.impending_counters`), so it enters as a non-creature
    /// that turns into a creature once the counters tick off.
    #[serde(default)]
    pub impending: u32,
    /// CR 702.48 — Offering. When `Some(filter)`, this alternative cost may be
    /// paid any time you could cast an instant (grants flash) by sacrificing a
    /// permanent you control matching `filter` (the offering's creature type)
    /// and reducing `mana_cost` by that permanent's whole mana cost, color
    /// included (`ManaCost::reduce_by_cost`). The auto-picker sacrifices the
    /// highest-MV match for maximum reduction. The five Kamigawa Patrons.
    #[serde(default)]
    pub offering: Option<SelectionRequirement>,
    /// Optional additional cost: tap N untapped creatures the caster controls
    /// matching the filter ("you may tap an untapped creature you control
    /// rather than pay this spell's mana cost" — the MMQ Mercadian cycle:
    /// Orim's Cure, Ramosian Rally). Rejected with
    /// `SelectionRequirementViolated` if fewer than N untapped matches exist;
    /// the auto-picker taps the lowest-power ones. Tapping as a cost doesn't
    /// trigger "becomes tapped" abilities the way `Effect::Tap` does.
    #[serde(default)]
    pub tap_creatures: Option<(SelectionRequirement, u32)>,
    /// Optional additional cost: an opponent gains N life ("rather than pay
    /// this spell's mana cost, you may have an opponent gain 3 life" —
    /// Invigorate). The auto-picker gives the life to the opponent with the
    /// lowest life total.
    #[serde(default)]
    pub opponent_gains_life: u32,
    /// Optional additional cost: discard N cards matching each filter ("you
    /// may discard a Plains card rather than pay this spell's mana cost" —
    /// Abolish; Foil's "an Island card and another card" is two entries).
    /// Rejected with `SelectionRequirementViolated` when the hand can't cover
    /// every entry; the auto-picker discards the lowest-mana-value matches.
    #[serde(default)]
    pub discard_filters: Vec<(SelectionRequirement, u32)>,
    /// EOE Warp (CR 702.x). When `true`, casting via this alternative cost
    /// stamps the resolving permanent `warped`: at the beginning of the next
    /// end step it's exiled and gains a `WhileExiled` may-play permission so
    /// its owner can recast it from exile (paying its full mana cost).
    #[serde(default)]
    pub warp: bool,
    /// CR 702.162 / 701.28 — More Than Meets the Eye: the spell is cast
    /// *converted*, so the permanent enters with its back face up.
    #[serde(default)]
    pub converted: bool,
}

impl CardDefinition {
    pub fn is_land(&self) -> bool { self.card_types.contains(&CardType::Land) }
    pub fn is_creature(&self) -> bool { self.card_types.contains(&CardType::Creature) }
    pub fn is_instant(&self) -> bool { self.card_types.contains(&CardType::Instant) }
    pub fn is_sorcery(&self) -> bool { self.card_types.contains(&CardType::Sorcery) }
    pub fn is_artifact(&self) -> bool { self.card_types.contains(&CardType::Artifact) }
    pub fn is_enchantment(&self) -> bool { self.card_types.contains(&CardType::Enchantment) }
    pub fn is_planeswalker(&self) -> bool { self.card_types.contains(&CardType::Planeswalker) }
    pub fn is_battle(&self) -> bool { self.card_types.contains(&CardType::Battle) }
    pub fn is_vanguard(&self) -> bool { self.card_types.contains(&CardType::Vanguard) }
    pub fn is_conspiracy(&self) -> bool { self.card_types.contains(&CardType::Conspiracy) }
    /// CR 314.1 — a scheme card.
    pub fn is_scheme(&self) -> bool { self.card_types.contains(&CardType::Scheme) }
    /// CR 311.1 — a plane card.
    pub fn is_plane(&self) -> bool { self.card_types.contains(&CardType::Plane) }
    /// CR 312.1 — a phenomenon card.
    pub fn is_phenomenon(&self) -> bool { self.card_types.contains(&CardType::Phenomenon) }
    /// CR 904.11 — an "Ongoing Scheme", exempt from the 904.10 sweep.
    pub fn is_ongoing_scheme(&self) -> bool {
        self.is_scheme() && self.supertypes.contains(&Supertype::Ongoing)
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
    pub fn is_instant_speed(&self) -> bool {
        self.is_instant() || self.keywords.contains(&Keyword::Flash)
    }

    /// True if any activated ability could add {C} ("could produce {C}" —
    /// Break the Ice, Obsidian Charmaw). Walks `AddMana` payloads, seeing
    /// through `Restricted` wrappers and `Seq` bodies.
    pub fn produces_colorless(&self) -> bool {
        fn payload_colorless(p: &crate::effect::ManaPayload) -> bool {
            use crate::effect::ManaPayload as MP;
            match p {
                MP::Colorless(_) => true,
                MP::Restricted(inner, _) | MP::RestrictedToChosenType(inner) => {
                    payload_colorless(inner)
                }
                _ => false,
            }
        }
        fn effect_colorless(e: &crate::effect::Effect) -> bool {
            use crate::effect::Effect;
            match e {
                Effect::AddMana { pool, .. } => payload_colorless(pool),
                Effect::Seq(v) => v.iter().any(effect_colorless),
                _ => false,
            }
        }
        self.activated_abilities.iter().any(|a| effect_colorless(&a.effect))
    }

    /// Printed colors from the mana cost's colored pips (CR 105.2), with
    /// the Devoid CDA (CR 702.114) yielding colorless.
    pub fn printed_colors(&self) -> Vec<crate::mana::Color> {
        use crate::mana::ManaSymbol;
        if let Some(forced) = &self.color_override {
            return forced.clone();
        }
        if self.keywords.contains(&Keyword::Devoid) {
            return Vec::new();
        }
        let mut colors = Vec::new();
        let mut push = |c: crate::mana::Color| {
            if !colors.contains(&c) {
                colors.push(c);
            }
        };
        // CR 105.2c — a color indicator defines color for objects (tokens,
        // back faces) whose mana cost can't.
        for c in &self.color_indicator {
            push(*c);
        }
        for sym in &self.cost.symbols {
            match sym {
                ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) | ManaSymbol::MonoHybrid(_, c) => {
                    push(*c)
                }
                ManaSymbol::Hybrid(a, b) => {
                    push(*a);
                    push(*b);
                }
                _ => {}
            }
        }
        colors
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

    /// Spend-restriction context for casting this card as a spell — gates
    /// which restricted mana [`crate::mana::ManaPool::pay_for_spell`] may drain.
    pub fn spell_kind(&self) -> crate::mana::SpellKind {
        crate::mana::SpellKind {
            instant_or_sorcery: self.is_instant() || self.is_sorcery(),
            artifact: self.is_artifact(),
            creature_types: if self.is_creature() {
                self.subtypes.creature_types.clone()
            } else {
                Vec::new()
            },
            changeling: self.is_creature() && self.keywords.contains(&Keyword::Changeling),
            land_ability: false,
            creature: self.is_creature(),
            creature_ability: false,
            casting_nonartifact_spell: !self.is_artifact(),
            activating_ability: false,
            lesson: self.subtypes.spell_subtypes.contains(&crate::card::SpellSubtype::Lesson),
            devoid: self.keywords.contains(&Keyword::Devoid),
            equipment: self.is_equipment(),
            colorless: self.printed_colors().is_empty(),
            mana_value: self.cost.cmc(),
            has_x: self.cost.has_x(),
            omen: false,
            enchantment: self.is_enchantment(),
            multicolored: self.printed_colors().len() >= 2,
            planeswalker: self.is_planeswalker(),
            legendary: self.supertypes.contains(&Supertype::Legendary),
            creature_mana_only: self.spend_only_creature_mana,
        }
    }

    /// Spend-restriction context for activating an ability of this card
    /// ("… or activate abilities of artifacts" — Power Depot).
    pub fn ability_spend_kind(&self) -> crate::mana::SpellKind {
        crate::mana::SpellKind {
            artifact: self.is_artifact(),
            land_ability: self.is_land(),
            creature_ability: self.is_creature(),
            activating_ability: true,
            equipment: self.is_equipment(),
            ..Default::default()
        }
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
    /// Returns the Saddle cost (required total power) if this card has
    /// `Keyword::Saddle(N)` (CR 702.171).
    pub fn saddle_cost(&self) -> Option<u32> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Saddle(n) = kw { Some(*n) } else { None }
        })
    }
    /// Returns the Casualty number (minimum power of the creature to
    /// sacrifice) if this card has `Keyword::Casualty(N)` (CR 702.153).
    pub fn casualty_cost(&self) -> Option<u32> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Casualty(n) = kw { Some(*n) } else { None }
        })
    }
    pub fn is_aura(&self) -> bool {
        self.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Aura)
    }
    /// CR 716 — a Class enchantment (enters at level 1, gains levels at
    /// sorcery speed via its `Level N` activated abilities).
    pub fn is_class(&self) -> bool {
        self.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Class)
    }

    /// The "enchant ___" restriction an Aura attaches under, recovered from
    /// its resolution `Effect::Attach { to: TargetFiltered { filter, .. } }`
    /// (CR 303.4a). Returns `None` when the attach target can't be read as a
    /// single filtered selector (bestow, scripted attaches), so callers that
    /// re-check legality (the 704.5n SBA) can conservatively skip it.
    pub fn aura_enchant_filter(&self) -> Option<&SelectionRequirement> {
        fn from_effect(e: &crate::effect::Effect) -> Option<&SelectionRequirement> {
            use crate::effect::{Effect, Selector};
            match e {
                Effect::Attach { to: Selector::TargetFiltered { filter, .. }, .. } => Some(filter),
                Effect::Seq(inner) => inner.iter().find_map(from_effect),
                _ => None,
            }
        }
        from_effect(&self.effect)
    }

    pub fn has_flashback(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Flashback(cost) = kw { Some(cost) } else { None }
        })
    }
    /// Returns the number of creatures that must be tapped to flashback
    /// this card if it has `Keyword::FlashbackTap`. None otherwise.
    pub fn has_flashback_tap(&self) -> Option<u32> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::FlashbackTap { count, .. } = kw { Some(*count) } else { None }
        })
    }

    /// The per-creature filter on this card's `Keyword::FlashbackTap`, if any.
    pub fn flashback_tap_filter(&self) -> Option<&SelectionRequirement> {
        self.keywords.iter().find_map(|kw| match kw {
            Keyword::FlashbackTap { filter, .. } => filter.as_deref(),
            _ => None,
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
    /// CR 702.103 — the Bestow mana cost, if this card has Bestow.
    pub fn has_bestow(&self) -> Option<&ManaCost> {
        self.bestow.as_ref()
    }
    /// CR 715 — the adventure half, if this card has Adventure.
    pub fn has_adventure(&self) -> Option<&Adventure> {
        self.adventure.as_deref()
    }
    /// CR 702.183 — the Omen half, if this card has Omen.
    pub fn has_omen(&self) -> Option<&Adventure> {
        self.omen.as_deref()
    }
    /// CR 702.160 — the Prototype face, if this card has Prototype.
    pub fn has_prototype(&self) -> Option<&Prototype> {
        self.prototype.as_deref()
    }
    /// CR 702.160c — a clone of this definition with the prototype cost,
    /// color, and size applied (color follows the cost; the `prototype`
    /// field is retained so a snapshot round-trip can re-apply it).
    pub fn with_prototype_applied(&self) -> Option<CardDefinition> {
        let proto = self.has_prototype()?;
        let mut def = self.clone();
        def.cost = proto.cost.clone();
        def.power = proto.power;
        def.toughness = proto.toughness;
        Some(def)
    }
    /// CR 614 — this definition with mode `idx` of `enter_modes` baked in: its
    /// triggered/static abilities and keywords are appended. Returns `None`
    /// when the card has no such mode. Used both at ETB and on snapshot load
    /// (re-baking the recorded `chosen_mode`).
    pub fn with_mode_applied(&self, idx: usize) -> Option<CardDefinition> {
        let mode = self.enter_modes.as_ref()?.get(idx)?;
        let mut def = self.clone();
        def.triggered_abilities.extend(mode.triggered_abilities.iter().cloned());
        def.static_abilities.extend(mode.static_abilities.iter().cloned());
        for kw in &mode.keywords {
            if !def.keywords.contains(kw) {
                def.keywords.push(kw.clone());
            }
        }
        Some(def)
    }
    /// CR 709 — the split definition, if this is a split card.
    pub fn has_split(&self) -> Option<&SplitCard> {
        self.split.as_deref()
    }
    /// CR 702.27 — the Buyback mana cost, if this card has Buyback.
    pub fn has_buyback(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Buyback(cost) = kw { Some(cost) } else { None }
        })
    }
    pub fn has_entwine(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Entwine(cost) = kw { Some(cost) } else { None }
        })
    }
    /// CR 709.5 — this Room definition with only the unlocked doors' (bit 0 =
    /// left, bit 1 = right) abilities live. Locked halves contribute nothing.
    pub fn room_definition_with(&self, unlocked: u8) -> CardDefinition {
        let Some(room) = self.room.as_deref() else {
            return self.clone();
        };
        let mut d = self.clone();
        d.triggered_abilities.clear();
        d.activated_abilities.clear();
        d.static_abilities.clear();
        for (bit, door) in [(1u8, &room.left), (2u8, &room.right)] {
            if unlocked & bit != 0 {
                d.triggered_abilities.extend(door.triggered_abilities.iter().cloned());
                d.activated_abilities.extend(door.activated_abilities.iter().cloned());
                d.static_abilities.extend(door.static_abilities.iter().cloned());
            }
        }
        d
    }
    /// The live definition of a solved Case: the always-on abilities plus the
    /// Case's "Solved —" abilities. A no-op clone for a non-Case.
    pub fn case_definition_solved(&self) -> CardDefinition {
        let Some(case) = self.case.as_deref() else {
            return self.clone();
        };
        let mut d = self.clone();
        d.triggered_abilities.extend(case.solved_triggered.iter().cloned());
        d.activated_abilities.extend(case.solved_activated.iter().cloned());
        d.static_abilities.extend(case.solved_static.iter().cloned());
        d
    }
    pub fn has_kicker(&self) -> Option<&ManaCost> {
        // Offspring (CR 702.166) is an optional additional cast cost that
        // reuses the Kicker pipeline (pay it → `SpellWasKicked` → ETB mints a
        // 1/1 token copy). A card carries one or the other, not both.
        self.keywords.iter().find_map(|kw| match kw {
            Keyword::Kicker(cost) | Keyword::Offspring(cost) => Some(cost),
            _ => None,
        })
    }

    /// CR 702.33c — the Multikicker cost, if this card has the keyword.
    pub fn has_multikicker(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Multikicker(cost) = kw { Some(cost) } else { None }
        })
    }

    /// The Offspring cost (CR 702.166), if this card has the keyword.
    pub fn has_offspring(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Offspring(cost) = kw { Some(cost) } else { None }
        })
    }
    /// CR 702.157 — the Squad cost if this card has `Keyword::Squad`.
    pub fn squad_cost(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Squad(cost) = kw { Some(cost) } else { None }
        })
    }
    /// CR 702.107 — the Replicate cost if this card has `Keyword::Replicate`.
    pub fn replicate_cost(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Replicate(cost) = kw { Some(cost) } else { None }
        })
    }
    /// Energy paid per replication for a `Keyword::ReplicateEnergy` card.
    pub fn replicate_energy_cost(&self) -> Option<u32> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::ReplicateEnergy(n) = kw { Some(*n) } else { None }
        })
    }
    /// CR 702.35 — the Madness cost if this card has `Keyword::Madness`.
    pub fn madness_cost(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Madness(cost) = kw { Some(cost) } else { None }
        })
    }
    /// CR 702.187 — the Mayhem cost if this card has `Keyword::Mayhem`.
    pub fn mayhem_cost(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Mayhem(cost) = kw { Some(cost) } else { None }
        })
    }
    /// CR 702.180 — the Harmonize cost if this card has `Keyword::Harmonize`.
    pub fn harmonize_cost(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Harmonize(cost) = kw { Some(cost) } else { None }
        })
    }
    pub fn has_equip(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| match kw {
            Keyword::Equip(cost) | Keyword::Reconfigure(cost) => Some(cost),
            _ => None,
        })
    }

    /// The Reconfigure cost (CR 702.151), if this card has the keyword.
    pub fn has_reconfigure(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Reconfigure(cost) = kw { Some(cost) } else { None }
        })
    }

    /// The Fortify cost (CR 702.71), if this card has the keyword.
    pub fn has_fortify(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Fortify(cost) = kw { Some(cost) } else { None }
        })
    }
}

/// CR 708.2 — the characteristics of a face-down permanent: a 2/2 colorless
/// creature with no name, types, subtypes, abilities, or mana cost. Used as
/// the active `definition` while a permanent is face down (morph / manifest);
/// the real card is stashed in `CardInstance.face_up_def`.
pub fn facedown_creature_definition() -> CardDefinition {
    CardDefinition {
        name: "",
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// CR 702.166c — a face-down permanent cast/manifested via Disguise (or Cloak)
/// is a 2/2 colorless creature with ward {2}.
pub fn facedown_disguise_definition() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(crate::mana::ManaCost {
            symbols: vec![crate::mana::ManaSymbol::Generic(2)],
        }))],
        ..facedown_creature_definition()
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
    /// Return to the battlefield tapped under its owner's control
    /// (Parallax Tide — "return all cards exiled with it to the
    /// battlefield tapped").
    BattlefieldTapped,
    /// Return to its owner's hand (Brain Maggot, Tidehollow Sculler,
    /// Kitesail Freebooter).
    Hand,
    /// The card stays in exile; its owner instead creates an X/X blue
    /// Illusion token, X = the exiled card's mana value (Skyclave
    /// Apparition).
    IllusionToken,
    /// Return to its owner's graveyard (Gravegouger).
    Graveyard,
}

/// Records that a card sits in exile because of another permanent's
/// "exile until ~ leaves the battlefield" ability. When the linking
/// `source` permanent leaves play, the engine returns this card to
/// `return_to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExileLink {
    pub source: CardId,
    pub return_to: ExileReturnZone,
    /// CR 725 — Palace Jailer's "until an opponent becomes the monarch". When
    /// `Some(p)`, the card returns as soon as the monarchy leaves player `p`
    /// (not when `source` leaves the battlefield), so `return_linked_exiles`
    /// skips it and `set_monarch` drives the return instead.
    #[serde(default)]
    pub monarch_guard: Option<usize>,
}

// ── Runtime card instance ─────────────────────────────────────────────────────

/// A card in play.  Tracks mutable game state layered on top of the static definition.
///
/// CR 508.1a — a pending or active "can't attack during its controller's
/// next turn" ban (Wall of Dust). Set to `Pending` when the ban lands; the
/// bearer's untap step promotes it to `Active`, and that turn's cleanup
/// clears it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AttackBan {
    #[default]
    None,
    Pending,
    Active,
}

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
    /// Permanent-duration P/T deltas (`Effect::PumpPT { duration: Permanent }`,
    /// e.g. Wall of Roots's -0/-1). Unlike `power_bonus`/`toughness_bonus`,
    /// these survive Cleanup and clear only when the permanent leaves the
    /// battlefield (new object, CR 400.7).
    pub perm_power_bonus: i32,
    pub perm_toughness_bonus: i32,
    pub counters: HashMap<CounterType, u32>,
    pub attached_to: Option<CardId>,
    /// CR 303.4a — the seat this Aura enchants, for "enchant player" Auras
    /// (the Curse cycle, Psychic Possession). Mutually exclusive with
    /// `attached_to`.
    pub attached_to_player: Option<usize>,
    /// CR 702.95 — the creature this one is Soulbond-paired with, if any.
    /// Either member of a pair points at the other. Cleared when either
    /// creature leaves the battlefield (SBA in `stack.rs`).
    pub soulbond_partner: Option<CardId>,
    pub kicked: bool,
    /// Activated abilities granted to this specific permanent by resolved
    /// effects (`Effect::GainActivatedAbility` — Urza's Saga chapters I/II).
    /// Cleared when the permanent leaves the battlefield (a new object,
    /// CR 400.7). Surfaced after the printed abilities in both
    /// `activate_ability` index space and the client view.
    pub granted_activated_abilities: Vec<ActivatedAbility>,
    /// "Until end of turn" sibling of `granted_activated_abilities` — cleared
    /// by `clear_end_of_turn_effects` (Lightning Volley, Retraction Helix).
    pub granted_activated_eot: Vec<ActivatedAbility>,
    /// CR 702.33c — how many times this spell's Multikicker cost was paid.
    /// Persists onto the permanent so `Value::TimesKicked` can read it
    /// (Everflowing Chalice's enters-with-counters). Defaults to 0.
    pub kick_count: u32,
    /// CR 702.157 — how many times this spell's optional Squad cost was paid.
    /// Persists onto the permanent so its ETB mints one token copy per payment
    /// (`Value::SquadCount`). Defaults to 0 (no Squad / not paid).
    pub squad_count: u32,
    /// Mana spent to cast the spell that became this permanent, stamped at
    /// resolution so ETB riders can read it after the spell left the stack
    /// (`Value::CastSpellManaSpent`, `ManaValueAtMostCastManaSpent` — Astelli
    /// Reclaimer). Defaults to 0.
    pub cast_mana_spent: u32,
    /// X paid into the spell that became this permanent, stamped at resolution
    /// so ETB *triggered abilities* can read the cast's X after the spell left
    /// the stack (`Value::XFromCost`, `ManaValueAtMostXFromCost` — Dune
    /// Drifter's "return an artifact/creature card with MV ≤ X"). Defaults to 0.
    pub cast_x_value: u32,
    /// CR 601 — per-color breakdown of the mana spent paying this spell's
    /// cost, stamped at cast time. Read by `Predicate::ManaSpentOfColorAtLeast`
    /// (Adamant, CR 702.137) and `Predicate::CastSpellNoColoredManaSpent`
    /// (Void Mirror). Empty for free/uncast objects.
    pub cast_mana_spent_by_color: Vec<(crate::mana::Color, u32)>,
    /// CR 702.176 — true if this spell was cast paying its optional Bargain
    /// cost (sacrifice an artifact, enchantment, or token). Read at resolution
    /// by `Predicate::SpellWasBargained`.
    pub bargained: bool,
    /// One-shot cast-time rider: when this spell resolves as a permanent it
    /// enters with these counters (Noctis's graveyard-cast finality counter).
    /// Drained at ETB by the stack resolver; empty for ordinary casts.
    pub pending_etb_counters: Vec<(CounterType, u32)>,
    /// CR 702.172 — Spree mode indices chosen at cast time (empty for
    /// non-Spree spells). Stamped by `GameAction::CastSpellSpree` and read at
    /// resolution by `Effect::Spree`.
    pub spree_modes: Vec<u8>,
    /// CR 702.47 — rules text gained by splicing cards onto this spell,
    /// resolved after the main effect (each entry reads its target from the
    /// matching `additional_targets` slot). Cleared when the spell leaves
    /// the stack (702.47e).
    pub spliced_effects: Vec<Effect>,
    /// Names of the cards spliced onto this spell, parallel to
    /// `spliced_effects` — Minamo's Meddling discards by name. Cleared with
    /// them when the spell leaves the stack.
    pub spliced_names: Vec<String>,
    /// CR 702.27 — true if this spell was cast paying its optional Buyback
    /// cost. On resolution the resolver returns the card to its owner's
    /// hand instead of the graveyard.
    pub bought_back: bool,
    /// CR 702.41 — true if this spell was cast paying its optional Entwine
    /// cost: its `ChooseMode` runs every mode in order.
    pub entwined: bool,
    /// CR 702.165 — true if this spell was cast with its Gift promised: on
    /// resolution it runs `definition.gift.gifted_effect` instead of `effect`.
    pub gift_promised: bool,
    /// CR 702.103 — true while this permanent is on the battlefield as a
    /// bestowed Aura. It's an Aura (not a creature) for as long as this is
    /// set; cleared by the SBA when the enchanted creature leaves, at which
    /// point it reverts to a creature.
    pub bestowed: bool,
    pub face_down: bool,
    /// CR 712 — true while this double-faced permanent is showing its back
    /// face. `definition` is swapped to the active face; `front_face` stashes
    /// the front so `Effect::Transform` can toggle back. Reconstructed on
    /// snapshot load from the front name + this flag.
    pub transformed: bool,
    /// CR 712 — the front-face definition, kept while `transformed` so the
    /// permanent can flip back. In-memory only (not serialized): the serde
    /// wire stores the front name and rebuilds this on load.
    pub front_face: Option<Arc<CardDefinition>>,
    /// CR 702.160 — the printed (full, colorless) definition, kept while
    /// `cast_as_prototype` so the permanent reverts to its full mana cost,
    /// color, and size when it leaves the battlefield. In-memory only: the
    /// wire keeps the name + `cast_as_prototype` and rebuilds this on load.
    pub prototype_printed: Option<Arc<CardDefinition>>,
    /// CR 710 — true while this flip card is showing its flipped (bottom) face.
    /// `definition` is swapped to the flip face; `unflipped_def` stashes the
    /// top so it can be restored on zone change. Reconstructed on snapshot load
    /// from the top name + this flag.
    pub flipped: bool,
    /// CR 710 — the unflipped (top) definition, kept while `flipped`. In-memory
    /// only: the serde wire stores the top name + the `flipped` flag.
    pub unflipped_def: Option<Arc<CardDefinition>>,
    /// CR 708 — while this permanent is on the battlefield face down (morph /
    /// manifest), `definition` is swapped to the vanilla 2/2 face-down
    /// definition and the real card is stashed here so it can be turned face
    /// up (and is restored as the card leaves the battlefield). In-memory
    /// only: the serde wire stores the real name + a `face_down_permanent`
    /// flag and rebuilds this on load.
    pub face_up_def: Option<Arc<CardDefinition>>,
    /// CR 702.182 — true while this permanent is face down because it was
    /// Cloaked (so its 2/2 face-down body has ward {2}, like Disguise). The
    /// real card carries no keyword to re-derive this, so it's tracked here and
    /// serialized.
    pub cloaked: bool,
    /// CR 709.5c — Room unlocked-door designations (bit 0 = left, bit 1 =
    /// right). The live `definition` is rebuilt to the union of unlocked
    /// doors' abilities (`room_definition_with`).
    pub unlocked_doors: u8,
    /// MKM Case — true once this Case has been solved. The live `definition` is
    /// rebuilt to include the "Solved —" abilities (`case_definition_solved`).
    pub case_solved: bool,
    /// CR 716.2 — a Class enchantment's current level (0 for non-Class cards;
    /// a Class enters at level 1). Its level-2/3 abilities are gated on this
    /// via `Predicate::SourceClassLevelIs`/`SourceClassLevelAtLeast` and
    /// `StaticEffect::WhileClassLevelAtLeast`, so no definition rebuild is
    /// needed.
    pub class_level: u8,
    pub is_token: bool,
    /// CR 606.3 — loyalty activations so far this turn (normally capped at
    /// one; two with `CardDefinition.loyalty_twice_each_turn`).
    pub loyalty_uses_this_turn: u8,
    /// One-turn "activate loyalty abilities twice" grant (Kaito, Dancing
    /// Shadow's combat-damage rider). Cleared at end-of-turn cleanup.
    pub loyalty_twice_this_turn: bool,
    /// True if this card was cast via an evoke alternative cost — it will
    /// be sacrificed on ETB after its ETB triggers fire.
    pub evoked: bool,
    /// True if this card was cast via a Dash alternative cost (CR 702.110)
    /// — on ETB it gains haste and is scheduled to return to its owner's
    /// hand at the beginning of the next end step.
    pub dashed: bool,
    /// True if this card was free-cast off the last Suspend time counter
    /// (CR 702.62f) — on ETB a creature so cast gains haste.
    pub cast_from_suspend: bool,
    /// True if this card was cast via Escape (CR 702.139) from the
    /// graveyard. Read by the "sacrifice it unless it escaped" ETB rider
    /// on the Theros-Beyond-Death titans (Kroxa, Uro).
    pub cast_from_escape: bool,
    /// True if this card was cast via a Blitz alternative cost (CR 702.152)
    /// — on ETB it gains haste and a death-draw rider, and is sacrificed at
    /// the next end step.
    pub blitzed: bool,
    /// True if this card was cast via a Warp alternative cost (EOE) — on ETB
    /// it arms a delayed exile-at-next-end-step that grants a `WhileExiled`
    /// may-play so it can be recast from exile.
    pub warped: bool,
    /// CR 701.28 — this permanent spell was cast *converted* (its More Than
    /// Meets the Eye alternative cost), so it enters with its back face up.
    /// Consumed at ETB resolution, then irrelevant.
    pub cast_converted: bool,
    /// CR 702.183 — number of time counters this permanent should enter with
    /// because it was cast for its Impending cost. Consumed at ETB (the
    /// counters are added to the permanent), then irrelevant. 0 = cast
    /// normally / not impending.
    pub impending_counters: u32,
    /// True if this card was cast from its owner's hand on its current
    /// trip through the stack. Used by the rebound resolution path to
    /// distinguish hand-casts (rebound triggers) from re-casts from exile
    /// (rebound does **not** chain).
    pub cast_from_hand: bool,
    /// True if this spell's primary target was a battlefield permanent at
    /// cast time. Drives the CR 608.2b resolution-time legality re-check
    /// (a zone-loose filter targeting a graveyard card never fizzles).
    pub cast_target_was_battlefield: bool,
    /// True if this permanent's current battlefield entry was caused by
    /// resolving the spell you cast (not a token, reanimation, or blink).
    /// Set when a permanent spell resolves; cleared at the CR 400.7 new-object
    /// reset on any other entry. Read by `Predicate::EnteredByCast` for
    /// "whenever a creature you control enters, if you cast it, …" (The Sibsig
    /// Ceremony). Not serialized — a fresh object reconstructs it as `false`.
    pub entered_by_cast: bool,
    /// True if this card was cast from a graveyard via its Flashback
    /// cost. On resolution the resolver routes the card to exile instead
    /// of the owner's graveyard. Replaces an earlier overload of the
    /// `kicked` flag so that a card with both Kicker and Flashback can
    /// be disambiguated cleanly.
    pub cast_via_flashback: bool,
    /// Feather, the Redeemed — when set on a stacked instant/sorcery, the
    /// resolver exiles it instead of routing to the graveyard, then schedules
    /// its return to the caster's hand at the next end step. Not serialized;
    /// cleared off the stack.
    pub feather_exile_return: bool,
    /// CR 702.187 — true if this card was cast from a graveyard for its Mayhem
    /// cost. Read by `Predicate::SpellWasMayhem` so "if this spell's mayhem cost
    /// was paid" riders (Sandman's Quicksand) can branch. Cleared off the stack.
    pub cast_via_mayhem: bool,
    /// CR 701.67 — true if this spell's optional "you may waterbend {N}"
    /// additional cost was paid. Read by `Predicate::SpellWasWaterbend` for
    /// "if its additional cost was paid" riders. Cleared off the stack.
    pub cast_via_waterbend: bool,
    /// CR 701.59 — true if this spell's "collect evidence N" additional cost was
    /// paid at cast time. Read by `Predicate::SpellCollectedEvidence` so
    /// "if evidence was collected" branches (Behind the Mask, Analyze the
    /// Pollen) can fork at resolution. Cleared off the stack.
    pub cast_collected_evidence: bool,
    /// True if this card was cast from exile on its current trip through the
    /// stack (suspend/foretell/plot/impulse free or alt-cost casts). Powers
    /// "whenever you cast a spell from exile" payoffs (Nassari, Dean of
    /// Expression). Cleared when the card leaves the stack.
    pub cast_from_exile: bool,
    /// True if this card was cast from its owner's library on its current trip
    /// through the stack (a play-from-top-of-library grant). Powers
    /// "whenever you cast a spell from your library" payoffs (Melek, Izzet
    /// Paragon). Cleared when the card leaves the stack.
    pub cast_from_library: bool,
    /// Modes this permanent has already picked for a "choose one that hasn't
    /// been chosen" ability (Captive Audience). Persists across turns.
    pub modes_chosen: Vec<u8>,
    /// One-shot permission to cast this MDFC's **back face from the
    /// graveyard** (Pestilent Cauldron's "sacrifice, then cast Restorative
    /// Burst transformed"). Set by `Effect::GrantCastBackFromGraveyard` once
    /// the card is in the graveyard; consumed (cleared) by
    /// `cast_spell_back_face` when it hops the card to hand to cast the back.
    /// (serde handled via the wire-mirror struct, like `cast_via_flashback`.)
    pub may_cast_back_from_graveyard: bool,
    /// "As [this] enters, choose a creature type." Cavern of Souls. The
    /// chosen type narrows which creature spells the controller can cast as
    /// uncounterable through this permanent. `None` until the ETB choice
    /// resolves — `caster_grants_uncounterable` treats `None` as
    /// "unrestricted" (legacy behaviour, used by tests that hand-craft a
    /// Cavern via `add_card_to_battlefield` without firing its ETB).
    pub chosen_creature_type: Option<CreatureType>,
    /// "As [this] enters, choose a basic land type." Realmwright. Read by
    /// `StaticEffect::LandsYouControlAreChosenType` to add that type to the
    /// controller's lands. `None` until the ETB choice resolves.
    pub chosen_land_type: Option<LandType>,
    /// A number chosen as this permanent entered (Sanctum Prelate — "choose a
    /// number"). Read by `noncreature_spell_cast_locked` for the chosen-MV lock.
    pub chosen_number: Option<u32>,
    /// CR 614 — the `enter_modes` index chosen as this permanent entered (the
    /// Tarkir Siege cycle). Recorded so a snapshot round-trip re-bakes the same
    /// mode's abilities onto the freshly-resolved definition. `None` for cards
    /// without a persistent mode choice.
    pub chosen_mode: Option<u8>,
    /// A card type chosen as this permanent entered (Serra's Emissary).
    /// Read by the chosen-card-type protection static.
    pub chosen_card_type: Option<CardType>,
    /// CR 702.29 — set once this permanent's echo cost has been paid (or on
    /// a permanent without echo). `false` on battlefield entry; `process_echo`
    /// checks unpaid echoes at the controller's upkeep.
    pub echo_paid: bool,
    /// CR 702.62e — this exiled card gained suspend from an effect (the card
    /// "Suspend"): `process_suspend` ticks it even though its definition
    /// carries no `Keyword::Suspend`.
    pub granted_suspend: bool,
    /// This permanent is phased out "until [source] leaves the battlefield"
    /// (Out of Time): `do_phasing` skips it, and it phases in when the
    /// source leaves.
    pub phased_out_by: Option<CardId>,
    /// Bitmask of name choices already taken from this permanent's
    /// choose-a-name ability (Garth One-Eye — each name once per game).
    pub name_choices_used: u32,
    /// Another permanent chosen and remembered as this one entered (Dauntless
    /// Bodyguard — "As this enters, choose another creature you control").
    /// `Selector::ChosenPermanentOfSource` resolves to it. `None` until the
    /// ETB choice resolves (or if no legal choice existed).
    pub chosen_permanent: Option<CardId>,
    /// A player chosen and remembered by this permanent
    /// (`Effect::RememberPlayerOnSource`). `PlayerRef::ChosenPlayerOfSource`
    /// resolves to it — "that player" on a later trigger (Soul Scourge).
    pub chosen_player: Option<usize>,
    /// A number this permanent remembered (`Effect::LoseAllButLifeRemembered`
    /// stamps the life lost). `Value::RememberedAmountOfSource` reads it —
    /// "gain life equal to the life you lost" (Soulgorger Orgg).
    pub remembered_amount: Option<i32>,
    /// Indices of activated abilities flagged `once_per_turn` that have
    /// already been used this turn. Cleared at the start of each turn by
    /// `clean_per_turn_state`. Empty for the common case (most abilities
    /// don't have the flag set).
    pub once_per_turn_used: Vec<usize>,
    /// CR 702.177 — indices of `exhaust` activated abilities already used.
    /// Unlike `once_per_turn_used`, this is **never** cleared (once per game).
    pub exhausted_abilities: Vec<usize>,
    /// Keywords granted with `Duration::EndOfTurn` via `Effect::GrantKeyword`.
    /// Cleared at the Cleanup step alongside `power_bonus`/`toughness_bonus`.
    /// Stored separately from `definition.keywords` so the printed-Oracle
    /// keywords aren't permanently mutated by an EOT pump (engine fix —
    /// push modern_decks batch 24). `has_keyword` checks both vectors.
    pub granted_keywords_eot: Vec<Keyword>,
    /// Per-grant layer timestamps for `granted_keywords_eot` (same indices;
    /// CR 613.7). Lets the layer walk order an EOT grant against
    /// RemoveKeyword / RemoveAllAbilities effects instead of always
    /// pre-merging it. Grants pushed without a timestamp (tests, legacy
    /// snapshots) default to 0 — ordered before every tracked effect,
    /// matching the old pre-merge behavior.
    pub granted_keywords_eot_ts: Vec<u64>,
    /// Keywords removed until end of turn via `Effect::LoseKeyword`
    /// (Shadowspear's "creatures your opponents control lose hexproof and
    /// indestructible until end of turn"). Removal beats printed/granted/
    /// counter sources for the turn; cleared at Cleanup.
    pub removed_keywords_eot: Vec<Keyword>,
    /// Keywords removed indefinitely (`Duration::Permanent`) — Ageless
    /// Sentinels' "it loses defender". Survives cleanup.
    pub removed_keywords: Vec<Keyword>,
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
    /// CR 120.x — true if this permanent has been dealt damage this turn
    /// (any source, combat or not, including wither/-1/-1 damage). Reset at
    /// cleanup. Powers "target creature that was dealt damage this turn"
    /// (Initiate of Blood // Goka). In-memory only.
    pub dealt_damage_this_turn: bool,
    /// CR 120.x — total damage dealt to this permanent this turn (any source,
    /// combat or not). Unlike `damage`, this survives regeneration and damage
    /// wipes, so "if 4 or more damage was dealt to it this turn" (Rushing-Tide
    /// Zubera) reads correctly. Reset at cleanup. In-memory only.
    pub damage_dealt_to_this_turn: u32,
    /// Sources that have dealt damage to this creature this turn. Powers
    /// "When a creature dealt damage by this creature this turn dies"
    /// (Bushi Tenderfoot). Read off the LKI snapshot at death-trigger time.
    /// Reset at cleanup; in-memory only.
    pub damaged_by_this_turn: Vec<CardId>,
    /// Controller of the source that most recently dealt *combat* damage to
    /// this creature. Stamped at the combat-damage chokepoint so a
    /// "whenever this is dealt combat damage" trigger can name the attacking
    /// player (`PlayerRef::CombatDamagerController` — Souls of the Faultless)
    /// after `block_map` has already been torn down. Transient; in-memory only.
    pub combat_damager_controller: Option<usize>,
    /// CR 701.15 — Regeneration shields. Each is a one-shot replacement:
    /// "the next time this permanent would be destroyed this turn, instead
    /// remove a regeneration shield, tap it, remove it from combat, and
    /// heal all marked damage." Transient (cleared every cleanup, like
    /// `granted_keywords_eot`), so it's intentionally **not** serialized —
    /// a mid-turn snapshot reload defaults shields back to 0.
    pub regeneration_shields: u32,
    /// CR 701.15g — "it can't be regenerated this turn": shields already on
    /// the permanent stop applying and new ones do nothing. Transient, cleared
    /// at cleanup like `regeneration_shields`, so it isn't serialized.
    pub cant_regenerate_this_turn: bool,
    /// CR 702.83 — Exert. When this creature attacks and is exerted, it
    /// won't untap during its controller's next untap step. Set at attack
    /// time; consumed (and the untap skipped) by `do_untap`. Transient —
    /// not serialized (defaults to false on snapshot reload).
    pub skip_next_untap: bool,
    /// Entrancing Lyre — this permanent doesn't untap during its controller's
    /// untap step for as long as the linked source permanent stays tapped (and
    /// on the battlefield). Cleared once the source untaps or leaves.
    pub untap_locked_by: Option<CardId>,
    /// Shipbreaker Kraken — this permanent doesn't untap for as long as the
    /// linked source permanent remains on the battlefield (regardless of the
    /// source's tapped state). Released once the source leaves.
    pub untap_locked_while_present: Option<CardId>,
    /// CR 702.142 — set when this creature is declared as an attacker;
    /// gates Boast activated abilities ("activate only if this attacked
    /// this turn"). Cleared in per-turn cleanup. Transient — not
    /// serialized (defaults to false on snapshot reload).
    pub attacked_this_turn: bool,
    /// Whether this creature attacked during its controller's *current or
    /// most recent* turn. Unlike `attacked_this_turn` it survives the
    /// intervening turns, and rolls into `attacked_last_turn` at its
    /// controller's untap step.
    pub attacked_own_turn: bool,
    /// Seats this permanent has dealt damage to this *game*, and the
    /// planeswalkers likewise (The Fallen). Never reset per turn.
    pub damaged_players_this_game: Vec<usize>,
    pub damaged_permanents_this_game: Vec<CardId>,
    /// Whether this creature attacked during its controller's *previous*
    /// turn, so `Keyword::CantAttackIfAttackedLastTurn` (Giant Turtle) can
    /// read it.
    pub attacked_last_turn: bool,
    /// Wall of Dust — "that creature can't attack during its controller's
    /// next turn". `Pending` is set when the ban lands; it promotes to
    /// `Active` at the bearer's controller's untap step and clears at that
    /// turn's cleanup.
    pub attack_ban: AttackBan,
    /// Damage dealt to this permanent this turn, tallied per damaging
    /// source's *name* (Blazing Effigy's "other sources named Blazing
    /// Effigy"). Reset at cleanup; in-memory only.
    pub damage_by_source_name_this_turn: Vec<(&'static str, u32)>,
    /// Set when this creature is declared as a blocker; powers "creature that
    /// attacked or blocked this turn" filters (Gideon's Triumph). Cleared in
    /// per-turn cleanup. Transient — not serialized (defaults false on reload).
    pub blocked_this_turn: bool,
    /// Attackers this creature blocked this turn, in declaration order.
    /// Outlives combat teardown (`block_map` is cleared at end of combat) so
    /// delayed "each creature that was blocked by one of those creatures this
    /// turn" clauses resolve (Triton Tactics). Cleared in per-turn cleanup.
    pub blocked_attackers_this_turn: Vec<CardId>,
    /// CR 702.39 — Provoke: the attacker this creature must block this
    /// combat if able. Set when an attacker provokes it (untap + force
    /// block); cleared at end of combat. Transient — not serialized.
    pub must_block: Option<CardId>,
    /// CR 603.6e — set on a card in exile that a permanent's "exile until
    /// ~ leaves" ability put there. When that source leaves the
    /// battlefield the engine returns this card to `ExileLink::return_to`.
    /// `None` for ordinary (permanent) exile.
    pub exiled_by: Option<ExileLink>,
    /// The permanent whose ability created this token — "tokens created with
    /// this enchantment" (Saproling Burst). `None` for cards and for tokens
    /// minted outside a permanent's ability.
    pub created_by: Option<CardId>,
    /// "Until end of turn, this loses 'Prevent all damage that would be dealt
    /// to this'" (the Glittering Lion / Lynx escape hatch). Consulted by
    /// `permanent_prevents_all_damage_to_self`; cleared at cleanup.
    pub damage_prevention_off_eot: bool,
    /// CR 720-style "exiled with [a permanent]": a permanent tag stamped on a
    /// card in exile that some source put there and keeps caring about (e.g.
    /// Keen-Eyed Curator's "card types among cards exiled with this
    /// creature"). Unlike `exiled_by`, the card never returns — this is a
    /// pure association used by counting effects. `None` for ordinary exile.
    pub exiled_with: Option<CardId>,
    /// CR 706.8a — die results "stored" on this permanent (Centaur of
    /// Attention). Every stored result on a card in the wild is a d6, so only
    /// the value is noted; `Effect::RerollStoredResults` rolls the same kind
    /// again.
    pub stored_die_results: Vec<u8>,
    /// CR 702.46 — Cipher. While this card is exiled "encoded on" a creature,
    /// `encoded_on` holds that creature's id. Whenever the encoded creature
    /// deals combat damage to a player, the controller may cast a free copy of
    /// this card. `None` for ordinary exile.
    pub encoded_on: Option<CardId>,
    /// Until-end-of-turn flashback granted to this card while it sits in a
    /// graveyard — "target instant/sorcery card in your graveyard gains
    /// flashback until end of turn; the flashback cost equals its mana
    /// cost" (the SOS "Flashback" instant). Read by `cast_flashback` via
    /// [`effective_flashback`]; cleared at cleanup (including graveyard
    /// cards). Transient grant, so it shares `granted_keywords_eot`'s
    /// lifetime; serialized for mid-turn snapshot consistency.
    pub granted_flashback_eot: Option<crate::mana::ManaCost>,
    /// Until-end-of-turn Harmonize (CR 702.180) granted to this card while it
    /// sits in a graveyard — "target instant/sorcery card in your graveyard
    /// gains harmonize until end of turn; its harmonize cost equals its mana
    /// cost" (Songcrafter Mage). Read by `cast_harmonize` via
    /// [`effective_harmonize`]; cleared at cleanup.
    pub granted_harmonize_eot: Option<crate::mana::ManaCost>,
    /// Alternative cost the controller may pay to cast this card via its
    /// `may_play_until` permission instead of casting it for free — the
    /// "miracle {N}" cost granted by Lorehold, the Historian. Read by
    /// `cast_from_zone_without_paying`; its lifetime tracks `may_play_until`
    /// (cleared together by the expiry sweep and when a cast consumes the
    /// permission). `None` for ordinary free may-play grants.
    pub granted_alt_cast_cost_eot: Option<crate::mana::ManaCost>,
    /// Conditional surcharge on a granted may-play cast: "it costs [cost]
    /// more to cast this way unless the spell targets a permanent matching
    /// [filter]" (Mavinda, Students' Advocate's {8}-unless-your-creature).
    /// Shares `may_play_until`'s lifetime.
    pub granted_cast_surcharge_eot: Option<(crate::mana::ManaCost, SelectionRequirement)>,
    /// CR 201.3 — a card name chosen as this permanent entered (Pithing
    /// Needle, Phyrexian Revoker "as this enters, choose a card name").
    /// Persistent state read by `activate_ability` to suppress non-mana
    /// activated abilities of sources with the chosen name. `None` for the
    /// vast majority of permanents that never name a card.
    pub named_card: Option<String>,
    /// A color chosen as this permanent entered (CR 614/107.4 — Coldsteel
    /// Heart, choose-a-color mana rocks). Read by `ManaPayload::
    /// ChosenColorOfSource` so a `{T}: Add the chosen color` ability taps for
    /// it. `None` until an `Effect::ChooseColorForSelf` stamps it.
    pub chosen_color: Option<crate::mana::Color>,
    /// CR 702.32b — which of the definition's `kicker_options` were paid for
    /// this cast (Anavolver kicked with {1}{U} only). Empty for every other
    /// spell.
    pub kicked_options: Vec<u8>,
    /// Two colors chosen as this permanent entered (Tablet of the Guilds).
    /// Empty until an `Effect::ChooseTwoColorsForSource` stamps them.
    pub chosen_colors: Vec<crate::mana::Color>,
    /// CR 701.38 — players who have goaded this creature. A goaded creature
    /// attacks each combat if able and attacks a player other than a goader
    /// if able, until that goader's next turn. Each goader's entry is
    /// cleared when their turn begins (`do_untap`). Empty for the vast
    /// majority of creatures. Round-trips through `CardInstanceWire` with a
    /// `#[serde(default)]` for snapshot back-compat.
    pub goaded_by: Vec<usize>,
    /// CR 701.31 — true once this permanent has become monstrous. Persistent
    /// (never cleared); gates the once-only `Effect::Monstrosity` counter add
    /// and any "as long as ~ is monstrous" state. Round-trips through
    /// `CardInstanceWire` with `#[serde(default)]`.
    pub monstrous: bool,
    /// CR 702.158b — this permanent's sector designation, assigned by the
    /// space-sculptor state-based action (CR 704.5u).
    pub sector: Option<Sector>,
    /// CR 702.93 — true once this creature has become renowned. Persistent;
    /// gates the once-only Renown counter add.
    pub renowned: bool,
    /// CR 701.60 — true while this creature is suspected. A suspected
    /// creature has menace and can't block (injected as computed keywords).
    /// Persistent until it leaves the battlefield. Round-trips through
    /// `CardInstanceWire` with a `#[serde(default)]`.
    pub suspected: bool,
    /// CR 715 — true while this card is on the stack as its adventure
    /// (instant/sorcery) half. The resolver runs `definition.adventure`'s
    /// effect instead of the creature body and exiles the card (setting
    /// `on_adventure`) instead of sending it to the graveyard.
    pub adventuring: bool,
    /// CR 715 — true while this card sits in exile after going on an
    /// adventure, available to be cast as its creature half from exile
    /// (`GameAction::CastAdventureCreature`). Cleared once the creature is
    /// cast (or the card otherwise changes zones).
    pub on_adventure: bool,
    /// CR 702.183 — true while this card is on the stack as its Omen
    /// (instant/sorcery) half. The resolver runs `definition.omen`'s effect
    /// instead of the creature body and, on resolution or counter, shuffles the
    /// card into its owner's library instead of the graveyard.
    pub omen_casting: bool,
    /// CR 702.160 — true when this card was cast for its Prototype cost, so it
    /// entered with the prototype's mana cost, color, and size. The live
    /// `definition` already carries the applied values; this flag lets a
    /// snapshot round-trip re-apply them (`with_prototype_applied`).
    pub cast_as_prototype: bool,
    /// CR 702.171 — true while this permanent is saddled (a marker set by a
    /// Saddle activation, until end of turn). Read by `Predicate::SourceSaddled`
    /// to gate "whenever this attacks while saddled" triggers. Cleared by
    /// `clear_end_of_turn_effects`.
    pub saddled: bool,
    /// CR 702.171 — the creatures that have saddled this permanent this turn
    /// (the riders tapped by a Saddle activation). Read by
    /// `Effect::ExileAndReturnSelfWithSaddler` for "exile it and up to one
    /// creature that saddled it this turn" (Fortune, Loyal Steed). Cleared with
    /// `saddled`.
    pub saddled_by: Vec<CardId>,
    /// CR 702.9 — the creatures that have crewed this Vehicle this turn. Read by
    /// `Value::SourceCrewerCount` for "for each creature that crewed it this
    /// turn" (Luxurious Locomotive). Cleared at end of turn / on leaving play.
    pub crewed_by: Vec<CardId>,
    /// CR 709 — which half of a split card this spell is being cast as while
    /// on the stack: `None`/`0` = left (default cast path), `1` = right
    /// (`CastSplitRight`), `2` = fused (`CastSplitFused`). Drives effect
    /// selection in `continue_spell_resolution`. Cleared off the stack.
    pub split_cast: Option<u8>,
    /// CR 603.4 — the turn number on which this permanent last entered the
    /// battlefield. `Some(t)` lets "creatures that entered this turn" filters
    /// (`SelectionRequirement::EnteredThisTurn` — Shaile, Dean of Radiance)
    /// compare against `GameState.turn_number`. `None` for objects that never
    /// entered through the battlefield-entry path (hand/library/stack cards).
    pub entered_turn: Option<u32>,
    /// CR 613.7a/d — this object's timestamp, drawn from the same counter as
    /// resolved-effect timestamps when it entered the battlefield, and
    /// re-stamped on attach (613.7e), face up/down (613.7f), and transform
    /// (613.7g). Static-ability continuous effects order by it. 0 = never
    /// stamped (the layer walk falls back to `id` order).
    pub battlefield_timestamp: u64,
    /// CR 701.35 — Detain. `Some(detainer)` while this permanent is detained:
    /// it can't attack or block and its activated abilities can't be activated
    /// until the detaining player's next turn (cleared at the start of that
    /// player's turn). `None` for the common undetained case. Round-trips with
    /// `#[serde(default)]`.
    pub detained_by: Option<usize>,
    /// CR 702.150 — life paid to Phyrexian pips while casting a Compleated
    /// planeswalker. Consumed (and cleared) when it enters, dropping the
    /// starting loyalty by this amount. 0 otherwise.
    pub compleated_life_paid: u32,
    /// CR 712.16 — the two component cards of a melded permanent. Non-empty
    /// only on a melded object; when it leaves the battlefield, the parts go
    /// to that zone instead and the melded shell ceases to exist.
    pub meld_parts: Vec<CardInstance>,
    /// CR 702.140 — the component cards of a merged (mutated) permanent,
    /// top-to-bottom. Non-empty only on a mutate pile; `definition` is the
    /// synthesized union (top card's characteristics + every card's
    /// abilities). When the pile leaves the battlefield each component goes
    /// to that zone as its own card.
    pub mutate_stack: Vec<CardInstance>,
    /// Set on a mutate spell on the stack: `(host id, on_top)`. On resolution
    /// the spell merges onto the host instead of entering as a new creature.
    pub mutate_onto: Option<(CardId, bool)>,
    /// CR 310.6 — the protector of a Battle: the opponent its controller chose
    /// as it entered. That player is the defending player for attacks aimed at
    /// the battle (CR 508.4) and its creatures block them. `None` for
    /// non-battles. Round-trips via `CardInstanceWire` with `#[serde(default)]`.
    pub protected_by: Option<usize>,
    /// Riders stamped on a permanent-spell copy by
    /// `Effect::CopySpellWithRiders` (Choreographed Sparks — "the copy
    /// gains haste and 'At the beginning of the end step, sacrifice this
    /// token.'"). Applied when the spell resolves onto the battlefield:
    /// `.0` grants haste until end of turn, `.1` schedules a
    /// next-end-step sacrifice. `None` for ordinary cards/copies.
    /// Round-trips via `CardInstanceWire` with `#[serde(default)]`.
    pub resolve_riders: Option<(bool, bool)>,
}

impl CardInstance {
    /// The instant/sorcery alternate half (Adventure or Omen) this card is
    /// currently on the stack as, if any. While set, the card resolves down the
    /// spell path with the half's card types and effect (CR 715 / CR 702.183).
    pub fn alt_spell_half(&self) -> Option<&Adventure> {
        if self.adventuring {
            self.definition.adventure.as_deref()
        } else if self.omen_casting {
            self.definition.omen.as_deref()
        } else {
            None
        }
    }

    /// True while this card is on the stack as an instant/sorcery alternate
    /// half (Adventure or Omen) — i.e. it resolves as a noncreature spell.
    pub fn casting_alt_half(&self) -> bool {
        self.adventuring || self.omen_casting
    }

    pub fn new(id: CardId, definition: impl Into<Arc<CardDefinition>>, owner: usize) -> Self {
        let definition = definition.into();
        let summoning_sick = definition.is_creature();
        let base_loyalty = definition.base_loyalty;
        let is_planeswalker = definition.is_planeswalker();
        let mut counters = HashMap::new();
        if is_planeswalker && base_loyalty > 0 {
            counters.insert(CounterType::Loyalty, base_loyalty);
        }
        // CR 310.7 — a Battle enters with defense counters equal to its printed
        // defense (re-seeded on reanimate/blink in `move_card_to`).
        if definition.is_battle() && definition.defense > 0 {
            counters.insert(CounterType::Defense, definition.defense);
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
            perm_power_bonus: 0,
            perm_toughness_bonus: 0,
            counters,
            attached_to: None,
            attached_to_player: None,
            soulbond_partner: None,
            kicked: false,
            granted_activated_abilities: Vec::new(),
            granted_activated_eot: Vec::new(),
            kick_count: 0,
            squad_count: 0,
            cast_mana_spent: 0,
            cast_x_value: 0,
            cast_mana_spent_by_color: Vec::new(),
            bargained: false,
            pending_etb_counters: Vec::new(),
            spree_modes: Vec::new(),
            spliced_effects: Vec::new(),
            spliced_names: Vec::new(),
            bought_back: false,
            entwined: false,
            gift_promised: false,
            bestowed: false,
            face_down: false,
            transformed: false,
            front_face: None,
            prototype_printed: None,
            flipped: false,
            unflipped_def: None,
            unlocked_doors: 0,
            case_solved: false,
            class_level: 0,
            face_up_def: None,
            cloaked: false,
            is_token: false,
            loyalty_uses_this_turn: 0,
            loyalty_twice_this_turn: false,
            evoked: false,
            dashed: false,
            cast_from_suspend: false,
            cast_from_escape: false,
            blitzed: false,
            warped: false,
            cast_converted: false,
            impending_counters: 0,
            cast_from_hand: false,
            cast_target_was_battlefield: false,
            entered_by_cast: false,
            cast_via_flashback: false,
            feather_exile_return: false,
            cast_via_mayhem: false,
            cast_via_waterbend: false,
            cast_collected_evidence: false,
            cast_from_exile: false,
            cast_from_library: false,
            modes_chosen: Vec::new(),
            may_cast_back_from_graveyard: false,
            chosen_creature_type: None,
            chosen_land_type: None,
            chosen_number: None,
            chosen_mode: None,
            chosen_card_type: None,
            echo_paid: false,
            granted_suspend: false,
            phased_out_by: None,
            name_choices_used: 0,
            chosen_permanent: None,
            chosen_player: None,
            remembered_amount: None,
            once_per_turn_used: Vec::new(),
            exhausted_abilities: Vec::new(),
            granted_keywords_eot: Vec::new(),
            granted_keywords_eot_ts: Vec::new(),
            removed_keywords_eot: Vec::new(),
            removed_keywords: Vec::new(),
            keyword_counters: std::collections::HashMap::new(),
            may_play_until: None,
            dealt_deathtouch_damage: false,
            dealt_damage_this_turn: false,
            damage_dealt_to_this_turn: 0,
            damaged_by_this_turn: Vec::new(),
            combat_damager_controller: None,
            regeneration_shields: 0,
            cant_regenerate_this_turn: false,
            skip_next_untap: false,
            untap_locked_by: None,
            created_by: None,
            damage_prevention_off_eot: false,
            untap_locked_while_present: None,
            attacked_this_turn: false,
            attacked_own_turn: false,
            damaged_players_this_game: Vec::new(),
            damaged_permanents_this_game: Vec::new(),
            attacked_last_turn: false,
            attack_ban: AttackBan::None,
            damage_by_source_name_this_turn: Vec::new(),
            blocked_this_turn: false,
            blocked_attackers_this_turn: Vec::new(),
            must_block: None,
            exiled_by: None,
            exiled_with: None,
            stored_die_results: Vec::new(),
            encoded_on: None,
            granted_flashback_eot: None,
            granted_harmonize_eot: None,
            granted_alt_cast_cost_eot: None,
            granted_cast_surcharge_eot: None,
            named_card: None,
            chosen_color: None,
            kicked_options: Vec::new(),
            chosen_colors: Vec::new(),
            goaded_by: Vec::new(),
            monstrous: false,
            sector: None,
            renowned: false,
            suspected: false,
            adventuring: false,
            on_adventure: false,
            omen_casting: false,
            cast_as_prototype: false,
            saddled: false,
            saddled_by: Vec::new(),
            crewed_by: Vec::new(),
            split_cast: None,
            entered_turn: None,
            battlefield_timestamp: 0,
            detained_by: None,
            compleated_life_paid: 0,
            meld_parts: Vec::new(),
            mutate_stack: Vec::new(),
            mutate_onto: None,
            protected_by: None,
            resolve_riders: None,
        }
    }

    pub fn new_token(id: CardId, definition: impl Into<Arc<CardDefinition>>, owner: usize) -> Self {
        let mut instance = Self::new(id, definition, owner);
        instance.is_token = true;
        instance
    }

    /// CR 613.7a — the timestamp this object's static-ability effects order
    /// by. Id-order fallback for objects never stamped (direct test pushes,
    /// pre-stamp snapshots).
    pub fn object_timestamp(&self) -> u64 {
        if self.battlefield_timestamp != 0 {
            self.battlefield_timestamp
        } else {
            self.id.0 as u64
        }
    }

    pub fn power(&self) -> i32 {
        let plus = self.counter_count(CounterType::PlusOnePlusOne) as i32;
        let minus = self.counter_count(CounterType::MinusOneMinusOne) as i32;
        let minus_one_zero = self.counter_count(CounterType::MinusOneMinusZero) as i32;
        let plus_one_zero = self.counter_count(CounterType::PlusOnePlusZero) as i32;
        let plus_two_zero = self.counter_count(CounterType::PlusTwoPlusZero) as i32;
        self.definition.base_power() + self.power_bonus + self.perm_power_bonus + plus
            - minus
            - minus_one_zero
            + plus_one_zero
            + 2 * plus_two_zero
    }

    pub fn toughness(&self) -> i32 {
        let plus = self.counter_count(CounterType::PlusOnePlusOne) as i32;
        let minus = self.counter_count(CounterType::MinusOneMinusOne) as i32;
        let minus_zero_one = self.counter_count(CounterType::MinusZeroMinusOne) as i32;
        let minus_zero_two = self.counter_count(CounterType::MinusZeroMinusTwo) as i32;
        let plus_zero_one = self.counter_count(CounterType::PlusZeroPlusOne) as i32;
        let plus_zero_two = self.counter_count(CounterType::PlusZeroPlusTwo) as i32;
        self.definition.base_toughness() + self.toughness_bonus + self.perm_toughness_bonus + plus
            - minus
            - minus_zero_one
            - 2 * minus_zero_two
            + plus_zero_one
            + 2 * plus_zero_two
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
        if self.removed_keywords_eot.contains(kw) || self.removed_keywords.contains(kw) {
            return false;
        }
        self.definition.keywords.contains(kw)
            || self.granted_keywords_eot.contains(kw)
            || self.keyword_counters.get(kw).copied().unwrap_or(0) > 0
    }

    /// True if this permanent has Toxic N for any N (printed or EOT-granted).
    /// The value-agnostic sibling of `has_keyword(&Toxic(n))`.
    pub fn has_toxic(&self) -> bool {
        self.definition.keywords.iter().chain(self.granted_keywords_eot.iter())
            .any(|k| matches!(k, Keyword::Toxic(_)))
    }

    /// True if this permanent has Modular N for any N (CR 702.43).
    pub fn has_modular(&self) -> bool {
        self.definition.keywords.iter().chain(self.granted_keywords_eot.iter())
            .any(|k| matches!(k, Keyword::Modular(_)))
    }

    /// True if this permanent can't be destroyed — either the
    /// Indestructible keyword (printed, granted, or via a keyword counter)
    /// or one or more `CounterType::Indestructible` counters (CR 122.1).
    pub fn is_indestructible(&self) -> bool {
        self.has_keyword(&Keyword::Indestructible)
            || self.counter_count(CounterType::Indestructible) > 0
    }

    pub fn has_protection_from(&self, color: Color) -> bool {
        // Granted/stripped EOT keywords count, same as `has_keyword`.
        self.has_keyword(&Keyword::Protection(color))
    }

    /// CR 708 — flip this permanent face down: stash the real definition in
    /// CR 902.5 / 315.5 — this command-zone card's abilities function from
    /// there. A Vanguard avatar always does; a Conspiracy only while it is
    /// face up (CR 315.5b — a face-down conspiracy has no characteristics).
    pub fn command_zone_abilities_active(&self) -> bool {
        self.definition.is_vanguard()
            || self.definition.is_plane()
            || (self.definition.is_conspiracy() && !self.face_down)
    }

    /// `face_up_def` and swap `definition` to the vanilla 2/2 face-down
    /// creature. No-op if already face down.
    pub fn turn_face_down(&mut self) {
        if self.face_up_def.is_some() {
            return;
        }
        // CR 702.166c / 702.182 — a Disguise or Cloak permanent is face down
        // with ward {2}.
        let warded = self.cloaked
            || self
                .definition
                .keywords
                .iter()
                .any(|k| matches!(k, Keyword::Disguise(_)));
        self.face_up_def = Some(self.definition.clone());
        self.definition = Arc::new(if warded {
            facedown_disguise_definition()
        } else {
            facedown_creature_definition()
        });
        self.face_down = true;
    }

    /// CR 702.182 — Cloak this card: turn it face down with a ward-{2} body. It
    /// can later be turned face up for its mana cost if it's a creature card.
    pub fn cloak(&mut self) {
        self.cloaked = true;
        self.turn_face_down();
    }

    /// CR 708.5/708.10 — flip this permanent face up (or restore the real
    /// card as it leaves the battlefield), restoring its real definition.
    /// Returns the real definition's name when a flip actually happened.
    /// CR 709.5c — unlocked-door designations are battlefield-only; clear
    /// them (and restore the printed no-abilities definition) as a Room
    /// permanent leaves the battlefield.
    pub fn reset_room_doors(&mut self) {
        if self.unlocked_doors != 0 && self.definition.room.is_some() {
            self.unlocked_doors = 0;
            self.definition = Arc::new(self.definition.room_definition_with(0));
        }
    }

    /// CR 716.2 — the Class level is battlefield-only; clear it as a Class
    /// leaves the battlefield so it re-enters at level 1.
    pub fn reset_class_level(&mut self) {
        self.class_level = 0;
    }

    /// MKM — the solved designation is battlefield-only; clear it (and restore
    /// the printed always-on definition) as a Case leaves the battlefield.
    pub fn reset_case(&mut self) {
        if self.case_solved && self.definition.case.is_some() {
            self.case_solved = false;
            // Rebuild from the case data on the current (solved) def by dropping
            // the appended solved abilities: re-derive from the un-appended base.
            // The base always-on abilities are those minus the solved set.
            if let Some(case) = self.definition.case.clone() {
                let mut d = (*self.definition).clone();
                let (t, a, s) = (&mut d.triggered_abilities, &mut d.activated_abilities, &mut d.static_abilities);
                t.truncate(t.len().saturating_sub(case.solved_triggered.len()));
                a.truncate(a.len().saturating_sub(case.solved_activated.len()));
                s.truncate(s.len().saturating_sub(case.solved_static.len()));
                self.definition = Arc::new(d);
            }
        }
    }

    /// MKM — mark this Case solved and rebuild its live definition to switch on
    /// the "Solved —" abilities. Returns false if not a Case or already solved.
    pub fn solve_case(&mut self) -> bool {
        if self.case_solved || self.definition.case.is_none() {
            return false;
        }
        self.case_solved = true;
        self.definition = Arc::new(self.definition.case_definition_solved());
        true
    }

    /// CR 709.5c/f — give this Room permanent an unlocked-door designation
    /// (bit 0 = left, bit 1 = right) and rebuild the live definition to the
    /// union of unlocked doors. Returns false if not a Room or already
    /// unlocked.
    pub fn unlock_room_door(&mut self, right: bool) -> bool {
        if self.definition.room.is_none() {
            return false;
        }
        let bit = if right { 2u8 } else { 1u8 };
        if self.unlocked_doors & bit != 0 {
            return false;
        }
        self.unlocked_doors |= bit;
        self.definition = Arc::new(self.definition.room_definition_with(self.unlocked_doors));
        true
    }

    pub fn turn_face_up(&mut self) -> Option<&'static str> {
        let real = self.face_up_def.take()?;
        let name = real.name;
        self.definition = real;
        self.face_down = false;
        self.cloaked = false;
        Some(name)
    }

    /// CR 710.2 — flip this permanent to its flipped (bottom) face in place.
    /// No-op if already flipped or it has no flip face. Returns the flip
    /// face's name when a flip happened.
    pub fn flip(&mut self) -> Option<&'static str> {
        if self.flipped {
            return None;
        }
        let mut flip = self.definition.flip_face.as_ref().map(|f| (**f).clone())?;
        let name = flip.name;
        // CR 710.1c — a flip card's colour and mana cost don't change when it
        // flips, so the bottom half inherits the top's cost (catalog flip
        // faces are written costless).
        if flip.cost.cmc() == 0 && !self.definition.cost.symbols.is_empty() {
            flip.cost = self.definition.cost.clone();
            flip.color_indicator = self.definition.printed_colors();
        }
        self.unflipped_def = Some(self.definition.clone());
        self.definition = Arc::new(flip);
        self.flipped = true;
        Some(name)
    }

    /// CR 710.2 — outside the battlefield only the unflipped characteristics
    /// exist; restore the top face as the card changes zones.
    pub fn revert_flip(&mut self) {
        if let Some(top) = self.unflipped_def.take() {
            self.definition = top;
            self.flipped = false;
        }
    }

    /// CR 712.4 — a transforming double-faced permanent that leaves the
    /// battlefield has only its front-face characteristics in the new zone.
    /// Restore the front face (no-op for a permanent showing its front).
    pub fn revert_transform(&mut self) {
        if let Some(front) = self.front_face.take() {
            self.definition = front;
            self.transformed = false;
        }
    }

    /// CR 702.160c — a permanent cast for its prototype cost has only its
    /// printed (full, colorless) characteristics once it leaves the
    /// battlefield. Restore the printed definition as the card changes zones.
    pub fn revert_prototype(&mut self) {
        if let Some(printed) = self.prototype_printed.take() {
            self.definition = printed;
            self.cast_as_prototype = false;
        }
    }

    /// CR 702.140e — merge `incoming` onto this host permanent. `on_top`
    /// puts the incoming card's characteristics on top of the merged pile.
    /// The host keeps its id, counters, damage, and summoning sickness; only
    /// its printed characteristics + abilities change.
    pub fn apply_mutate(&mut self, incoming: CardInstance, on_top: bool) {
        if self.mutate_stack.is_empty() {
            // Seed the pile with a bare component carrying the host's printed
            // card, so leaving the battlefield can scatter every component.
            let mut host = CardInstance::new(self.id, self.definition.clone(), self.owner);
            host.is_token = self.is_token;
            self.mutate_stack.push(host);
        }
        let mut incoming = incoming;
        incoming.mutate_onto = None;
        if on_top {
            self.mutate_stack.insert(0, incoming);
        } else {
            self.mutate_stack.push(incoming);
        }
        self.rebuild_mutate_definition();
    }

    /// Recompute the merged permanent's live `definition`: the top card's
    /// characteristics with every component card's abilities unioned in.
    pub fn rebuild_mutate_definition(&mut self) {
        let Some(top) = self.mutate_stack.first() else { return };
        let mut def = (*top.definition).clone();
        for comp in self.mutate_stack.iter().skip(1) {
            def.triggered_abilities
                .extend(comp.definition.triggered_abilities.iter().cloned());
            def.activated_abilities
                .extend(comp.definition.activated_abilities.iter().cloned());
            def.static_abilities
                .extend(comp.definition.static_abilities.iter().cloned());
            for kw in &comp.definition.keywords {
                if !def.keywords.contains(kw) {
                    def.keywords.push(kw.clone());
                }
            }
        }
        self.definition = Arc::new(def);
        // CR 730.2d — the merged permanent is a token only if its *topmost*
        // component is one.
        self.is_token = top.is_token;
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

    /// The full Ward cost, if any (CR 702.21). Engine ward enforcement
    /// matches on the `WardCost` variants directly; this is the inspection
    /// helper.
    pub fn ward(&self) -> Option<&WardCost> {
        self.definition.keywords.iter().find_map(|kw| {
            if let Keyword::Ward(cost) = kw {
                Some(cost)
            } else {
                None
            }
        })
    }

    /// Mana-Ward mana value, `None` for no ward or a non-mana ward. Colored
    /// pips count via cmc — use [`Self::ward`] when the exact cost matters.
    pub fn ward_cost(&self) -> Option<u32> {
        match self.ward() {
            Some(WardCost::Mana(cost)) => Some(cost.cmc()),
            _ => None,
        }
    }

    /// CR 702.146e — true for a transformed Disturb card (a back face whose
    /// front prints Disturb): "if it would be put into a graveyard from
    /// anywhere, exile it instead." Consulted at the graveyard funnels.
    pub fn disturb_back_exiles(&self) -> bool {
        self.transformed
            && self
                .front_face
                .as_ref()
                .is_some_and(|f| f.keywords.iter().any(|k| matches!(k, Keyword::Disturb(_))))
    }

    /// Fold `amount` damage from a source named `name` into this turn's
    /// per-name tally (Blazing Effigy).
    pub fn record_damage_from_named(&mut self, name: &'static str, amount: u32) {
        match self.damage_by_source_name_this_turn.iter_mut().find(|(n, _)| *n == name) {
            Some((_, n)) => *n += amount,
            None => self.damage_by_source_name_this_turn.push((name, amount)),
        }
    }

    pub fn clear_end_of_turn_effects(&mut self) {
        self.power_bonus = 0;
        self.toughness_bonus = 0;
        self.loyalty_uses_this_turn = 0;
        self.loyalty_twice_this_turn = false;
        self.once_per_turn_used.clear();
        self.granted_keywords_eot.clear();
        self.granted_keywords_eot_ts.clear();
        self.granted_activated_eot.clear();
        self.removed_keywords_eot.clear();
        self.granted_flashback_eot = None;
        self.granted_harmonize_eot = None;
        self.granted_alt_cast_cost_eot = None;
        self.granted_cast_surcharge_eot = None;
        self.dealt_deathtouch_damage = false;
        self.dealt_damage_this_turn = false;
        self.damage_dealt_to_this_turn = 0;
        self.damaged_by_this_turn.clear();
        self.damage_by_source_name_this_turn.clear();
        // CR 701.15g — unused regeneration shields expire at end of turn,
        // and so does the "can't be regenerated this turn" lock.
        self.regeneration_shields = 0;
        self.cant_regenerate_this_turn = false;
        self.damage_prevention_off_eot = false;
        // CR 702.171 — "saddled until end of turn" ends here.
        self.saddled = false;
        self.saddled_by.clear();
        self.crewed_by.clear();
    }

    /// The flashback cost this card can currently be cast with from a
    /// graveyard — its printed `Keyword::Flashback`, or an until-end-of-turn
    /// grant (the SOS "Flashback" instant). `None` if neither applies.
    pub fn effective_flashback(&self) -> Option<&ManaCost> {
        self.definition
            .has_flashback()
            .or(self.granted_flashback_eot.as_ref())
    }

    /// The Harmonize cost this card can currently be cast with from a
    /// graveyard — its printed `Keyword::Harmonize`, or an until-end-of-turn
    /// grant (Songcrafter Mage). `None` if neither applies.
    pub fn effective_harmonize(&self) -> Option<&ManaCost> {
        self.definition
            .harmonize_cost()
            .or(self.granted_harmonize_eot.as_ref())
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
    /// Permanent-duration pump deltas. `#[serde(default)]` for back-compat.
    #[serde(default)]
    perm_power_bonus: i32,
    #[serde(default)]
    perm_toughness_bonus: i32,
    counters: Vec<(CounterType, u32)>,
    attached_to: Option<CardId>,
    /// CR 303.4a — enchanted seat for "enchant player" Auras.
    #[serde(default)]
    attached_to_player: Option<usize>,
    /// CR 702.95 Soulbond partner. `#[serde(default)]` so older snapshots load
    /// as `None`.
    #[serde(default)]
    soulbond_partner: Option<CardId>,
    kicked: bool,
    /// CR 702.33c multikicker count (Everflowing Chalice's ETB counters).
    /// `#[serde(default)]` for back-compat.
    #[serde(default)]
    kick_count: u32,
    /// CR 702.157 squad-payment count. `#[serde(default)]` for back-compat.
    #[serde(default)]
    squad_count: u32,
    /// Mana spent to cast this permanent's spell. `#[serde(default)]`.
    #[serde(default)]
    cast_mana_spent: u32,
    /// X paid into this permanent's spell. `#[serde(default)]`.
    #[serde(default)]
    cast_x_value: u32,
    #[serde(default)]
    cast_mana_spent_by_color: Vec<(crate::mana::Color, u32)>,
    /// CR 702.176 bargain flag. `#[serde(default)]` for back-compat.
    #[serde(default)]
    bargained: bool,
    /// Noctis cast-time ETB-counter rider. `#[serde(default)]` for back-compat.
    #[serde(default)]
    pending_etb_counters: Vec<(CounterType, u32)>,
    /// CR 702.47 spliced rules text. `#[serde(default)]` for back-compat.
    #[serde(default)]
    spliced_effects: Vec<Effect>,
    #[serde(default)]
    spliced_names: Vec<String>,
    /// CR 702.46 — creature this card is ciphered onto. `#[serde(default)]`
    /// for back-compat.
    #[serde(default)]
    encoded_on: Option<CardId>,
    /// CR 608.2b fizzle bookkeeping — primary target was a battlefield
    /// permanent at cast time. `#[serde(default)]` for back-compat.
    #[serde(default)]
    cast_target_was_battlefield: bool,
    /// `Effect::GainActivatedAbility` grants (Urza's Saga). `#[serde(default)]`
    /// for back-compat.
    #[serde(default)]
    granted_activated_abilities: Vec<ActivatedAbility>,
    /// The EOT-duration half of the same grant. `#[serde(default)]` for
    /// back-compat.
    #[serde(default)]
    granted_activated_eot: Vec<ActivatedAbility>,
    /// CR 702.27 buyback flag. `#[serde(default)]` so older snapshots load
    /// as `false`.
    #[serde(default)]
    bought_back: bool,
    /// CR 702.41 entwine flag. `#[serde(default)]` for back-compat.
    #[serde(default)]
    entwined: bool,
    /// CR 702.165 gift-promised flag. `#[serde(default)]` for back-compat.
    #[serde(default)]
    gift_promised: bool,
    /// CR 702.103 bestowed flag. `#[serde(default)]` so older snapshots load
    /// as `false`.
    #[serde(default)]
    bestowed: bool,
    face_down: bool,
    /// CR 709.5c — Room unlocked-door bitmask. `#[serde(default)]` for
    /// back-compat; the live definition is rebuilt on load.
    #[serde(default)]
    unlocked_doors: u8,
    /// MKM — Case solved flag. `#[serde(default)]` for back-compat; the live
    /// definition is rebuilt on load.
    #[serde(default)]
    case_solved: bool,
    /// CR 716.2 — Class level. `#[serde(default)]` (0) for back-compat.
    #[serde(default)]
    class_level: u8,
    /// CR 708 — on the battlefield face down (morph / manifest). `name` stores
    /// the real card's name so the registry resolves it; on load the real
    /// definition is stashed and `definition` swapped to the vanilla 2/2.
    /// `#[serde(default)]` for back-compat.
    #[serde(default)]
    face_down_permanent: bool,
    /// CR 702.182 — face down because Cloaked (ward-{2} body). `#[serde(default)]`
    /// for back-compat.
    #[serde(default)]
    cloaked: bool,
    /// CR 712 — showing the back face. `name` always stores the FRONT face's
    /// name so the registry resolves it; the back is recovered as
    /// `front.back_face` on load. `#[serde(default)]` for back-compat.
    #[serde(default)]
    transformed: bool,
    /// CR 710 — showing the flipped face. `name` stores the unflipped (top)
    /// name; the flip face is recovered as `top.flip_face` on load.
    #[serde(default)]
    flipped: bool,
    is_token: bool,
    #[serde(default)]
    loyalty_uses_this_turn: u8,
    #[serde(default)]
    loyalty_twice_this_turn: bool,
    evoked: bool,
    #[serde(default)]
    dashed: bool,
    #[serde(default)]
    cast_from_suspend: bool,
    #[serde(default)]
    cast_from_escape: bool,
    #[serde(default)]
    blitzed: bool,
    #[serde(default)]
    warped: bool,
    #[serde(default)]
    impending_counters: u32,
    cast_from_hand: bool,
    /// `#[serde(default)]` so snapshots predating the field deserialize
    /// as `false` (matching the old "not cast via flashback" path).
    #[serde(default)]
    cast_via_flashback: bool,
    #[serde(default)]
    cast_via_mayhem: bool,
    #[serde(default)]
    cast_via_waterbend: bool,
    #[serde(default)]
    cast_collected_evidence: bool,
    #[serde(default)]
    cast_from_exile: bool,
    #[serde(default)]
    cast_from_library: bool,
    #[serde(default)]
    modes_chosen: Vec<u8>,
    #[serde(default)]
    may_cast_back_from_graveyard: bool,
    chosen_creature_type: Option<CreatureType>,
    #[serde(default)]
    chosen_land_type: Option<LandType>,
    #[serde(default)]
    chosen_number: Option<u32>,
    #[serde(default)]
    chosen_mode: Option<u8>,
    #[serde(default)]
    chosen_card_type: Option<CardType>,
    #[serde(default)]
    echo_paid: bool,
    #[serde(default)]
    granted_suspend: bool,
    #[serde(default)]
    phased_out_by: Option<CardId>,
    #[serde(default)]
    name_choices_used: u32,
    #[serde(default)]
    chosen_permanent: Option<CardId>,
    #[serde(default)]
    chosen_player: Option<usize>,
    #[serde(default)]
    remembered_amount: Option<i32>,
    #[serde(default)]
    once_per_turn_used: Vec<usize>,
    #[serde(default)]
    exhausted_abilities: Vec<usize>,
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
    /// Grant timestamps for `granted_keywords_eot` (CR 613.7). Same
    /// indices; `#[serde(default)]` for back-compat (missing = 0).
    #[serde(default)]
    granted_keywords_eot_ts: Vec<u64>,
    /// Until-end-of-turn keyword removals (Shadowspear). Shares
    /// `granted_keywords_eot`'s lifetime. `#[serde(default)]` for back-compat.
    #[serde(default)]
    removed_keywords_eot: Vec<Keyword>,
    /// Indefinite keyword removals (Ageless Sentinels).
    #[serde(default)]
    removed_keywords: Vec<Keyword>,
    /// Until-end-of-turn flashback grant (SOS "Flashback"). Shares the
    /// transient lifetime of `granted_keywords_eot`; serialized so a
    /// mid-turn snapshot restores it. `#[serde(default)]` for back-compat.
    #[serde(default)]
    granted_flashback_eot: Option<crate::mana::ManaCost>,
    #[serde(default)]
    granted_harmonize_eot: Option<crate::mana::ManaCost>,
    /// Until-end-of-turn alternative cast cost (Lorehold's miracle {N}).
    /// Shares `may_play_until`'s lifetime. `#[serde(default)]` for
    /// back-compat.
    #[serde(default)]
    granted_alt_cast_cost_eot: Option<crate::mana::ManaCost>,
    /// Conditional may-play surcharge (Mavinda). `#[serde(default)]`.
    #[serde(default)]
    granted_cast_surcharge_eot: Option<(crate::mana::ManaCost, SelectionRequirement)>,
    /// CR 201.3 named card (Pithing Needle / Phyrexian Revoker). Persistent
    /// state; `#[serde(default)]` so older snapshots load as `None`.
    #[serde(default)]
    named_card: Option<String>,
    /// Chosen color (Coldsteel Heart-style mana rocks). `#[serde(default)]`
    /// so older snapshots load as `None`.
    #[serde(default)]
    chosen_color: Option<crate::mana::Color>,
    #[serde(default)]
    chosen_colors: Vec<crate::mana::Color>,
    /// CR 702.32b — the paid "and/or" kicker indices.
    #[serde(default)]
    kicked_options: Vec<u8>,
    /// CR 701.38 goad — players who have goaded this creature.
    /// `#[serde(default)]` so older snapshots load as empty.
    #[serde(default)]
    goaded_by: Vec<usize>,
    /// CR 701.31 monstrous flag. `#[serde(default)]` so older snapshots load
    /// as `false`.
    #[serde(default)]
    monstrous: bool,
    /// CR 702.93 renowned flag. `#[serde(default)]` so older snapshots load
    /// as `false`.
    #[serde(default)]
    renowned: bool,
    /// CR 701.60 suspected flag. `#[serde(default)]` so older snapshots load
    /// as `false`.
    #[serde(default)]
    suspected: bool,
    /// CR 715 Adventure flags. `#[serde(default)]` so older snapshots load
    /// as `false`.
    #[serde(default)]
    adventuring: bool,
    #[serde(default)]
    on_adventure: bool,
    /// CR 702.183 Omen marker. `#[serde(default)]` so older snapshots load
    /// as `false`.
    #[serde(default)]
    omen_casting: bool,
    /// CR 702.160 prototype marker. `#[serde(default)]` so older snapshots
    /// load as `false`.
    #[serde(default)]
    cast_as_prototype: bool,
    /// CR 702.171 saddled marker. `#[serde(default)]` so older snapshots
    /// load as `false`.
    #[serde(default)]
    saddled: bool,
    /// CR 702.171 — riders that saddled this permanent this turn.
    #[serde(default)]
    saddled_by: Vec<CardId>,
    #[serde(default)]
    crewed_by: Vec<CardId>,
    /// CR 709 split-half marker. `#[serde(default)]` so older snapshots load
    /// as `None`.
    #[serde(default)]
    split_cast: Option<u8>,
    /// CR 603.6e linked-exile state. `exiled_by` powers "return when source
    /// leaves" (Banisher Priest); `exiled_with` powers imprint / linked-exile
    /// reads (Chrome Mox, Isochron Scepter, Keen-Eyed Curator). Persisted so
    /// mid-game name→factory snapshots keep the links. `#[serde(default)]` for
    /// back-compat.
    #[serde(default)]
    exiled_by: Option<ExileLink>,
    #[serde(default)]
    exiled_with: Option<CardId>,
    /// CR 603.4 — turn this permanent last entered. `#[serde(default)]` so
    /// older snapshots load as `None`.
    #[serde(default)]
    entered_turn: Option<u32>,
    /// CR 613.7 object timestamp. `#[serde(default)]` so older snapshots
    /// load as 0 (id-order fallback).
    #[serde(default)]
    battlefield_timestamp: u64,
    /// CR 701.35 Detain marker. `#[serde(default)]` so older snapshots load as
    /// `None`.
    #[serde(default)]
    detained_by: Option<usize>,
    /// CR 702.150 Compleated life payment. `#[serde(default)]`.
    #[serde(default)]
    compleated_life_paid: u32,
    /// CR 712.16 melded-component cards. `#[serde(default)]` so older
    /// snapshots load as empty.
    #[serde(default)]
    meld_parts: Vec<CardInstance>,
    /// CR 702.140 merged-component cards. `#[serde(default)]` so older
    /// snapshots load as empty; the union `definition` is rebuilt on load.
    #[serde(default)]
    mutate_stack: Vec<CardInstance>,
    /// Pending mutate-onto target for a mutate spell on the stack.
    #[serde(default)]
    mutate_onto: Option<(CardId, bool)>,
    /// Entrancing Lyre tap-lock: this permanent doesn't untap while the
    /// linked source stays tapped. `#[serde(default)]` for back-compat.
    #[serde(default)]
    untap_locked_by: Option<CardId>,
    /// CR 310.6 Battle protector. `#[serde(default)]` for back-compat.
    #[serde(default)]
    protected_by: Option<usize>,
    /// The permanent whose ability minted this token. `#[serde(default)]`
    /// for back-compat.
    #[serde(default)]
    created_by: Option<CardId>,
    /// Spell-copy riders (haste, sac-at-end-step). `#[serde(default)]`
    /// for back-compat.
    #[serde(default)]
    resolve_riders: Option<(bool, bool)>,
}

impl serde::Serialize for CardInstance {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let wire = CardInstanceWire {
            id: self.id,
            // Always serialize the FRONT name so the registry can resolve it;
            // a transformed permanent's active definition is the (unregistered)
            // back face.
            name: self
                .face_up_def
                .as_ref()
                .or(self.front_face.as_ref())
                .or(self.unflipped_def.as_ref())
                .map(|f| f.name.to_string())
                .unwrap_or_else(|| self.definition.name.to_string()),
            owner: self.owner,
            controller: self.controller,
            tapped: self.tapped,
            damage: self.damage,
            summoning_sick: self.summoning_sick,
            power_bonus: self.power_bonus,
            toughness_bonus: self.toughness_bonus,
            perm_power_bonus: self.perm_power_bonus,
            perm_toughness_bonus: self.perm_toughness_bonus,
            counters: self.counters.iter().map(|(k, v)| (*k, *v)).collect(),
            attached_to: self.attached_to,
            attached_to_player: self.attached_to_player,
            soulbond_partner: self.soulbond_partner,
            kicked: self.kicked,
            kicked_options: self.kicked_options.clone(),
            kick_count: self.kick_count,
            squad_count: self.squad_count,
            cast_mana_spent: self.cast_mana_spent,
            cast_x_value: self.cast_x_value,
            cast_mana_spent_by_color: self.cast_mana_spent_by_color.clone(),
            bargained: self.bargained,
            pending_etb_counters: self.pending_etb_counters.clone(),
            spliced_effects: self.spliced_effects.clone(),
            spliced_names: self.spliced_names.clone(),
            encoded_on: self.encoded_on,
            cast_target_was_battlefield: self.cast_target_was_battlefield,
            granted_activated_abilities: self.granted_activated_abilities.clone(),
            granted_activated_eot: self.granted_activated_eot.clone(),
            bought_back: self.bought_back,
            entwined: self.entwined,
            gift_promised: self.gift_promised,
            bestowed: self.bestowed,
            face_down: self.face_down,
            unlocked_doors: self.unlocked_doors,
            case_solved: self.case_solved,
            class_level: self.class_level,
            face_down_permanent: self.face_up_def.is_some(),
            cloaked: self.cloaked,
            transformed: self.transformed,
            flipped: self.flipped,
            is_token: self.is_token,
            loyalty_uses_this_turn: self.loyalty_uses_this_turn,
            loyalty_twice_this_turn: self.loyalty_twice_this_turn,
            evoked: self.evoked,
            dashed: self.dashed,
            cast_from_suspend: self.cast_from_suspend,
            cast_from_escape: self.cast_from_escape,
            blitzed: self.blitzed,
            warped: self.warped,
            impending_counters: self.impending_counters,
            cast_from_hand: self.cast_from_hand,
            cast_via_flashback: self.cast_via_flashback,
            cast_via_mayhem: self.cast_via_mayhem,
            cast_via_waterbend: self.cast_via_waterbend,
            cast_collected_evidence: self.cast_collected_evidence,
            cast_from_exile: self.cast_from_exile,
            cast_from_library: self.cast_from_library,
            modes_chosen: self.modes_chosen.clone(),
            may_cast_back_from_graveyard: self.may_cast_back_from_graveyard,
            chosen_creature_type: self.chosen_creature_type,
            chosen_land_type: self.chosen_land_type,
            chosen_number: self.chosen_number,
            chosen_mode: self.chosen_mode,
            chosen_card_type: self.chosen_card_type.clone(),
            echo_paid: self.echo_paid,
            granted_suspend: self.granted_suspend,
            phased_out_by: self.phased_out_by,
            name_choices_used: self.name_choices_used,
            chosen_permanent: self.chosen_permanent,
            chosen_player: self.chosen_player,
            remembered_amount: self.remembered_amount,
            once_per_turn_used: self.once_per_turn_used.clone(),
            exhausted_abilities: self.exhausted_abilities.clone(),
            may_play_until: self.may_play_until,
            keyword_counters: self
                .keyword_counters
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
            granted_keywords_eot: self.granted_keywords_eot.clone(),
            granted_keywords_eot_ts: self.granted_keywords_eot_ts.clone(),
            removed_keywords_eot: self.removed_keywords_eot.clone(),
            removed_keywords: self.removed_keywords.clone(),
            granted_flashback_eot: self.granted_flashback_eot.clone(),
            granted_harmonize_eot: self.granted_harmonize_eot.clone(),
            granted_alt_cast_cost_eot: self.granted_alt_cast_cost_eot.clone(),
            granted_cast_surcharge_eot: self.granted_cast_surcharge_eot.clone(),
            named_card: self.named_card.clone(),
            chosen_color: self.chosen_color,
            chosen_colors: self.chosen_colors.clone(),
            goaded_by: self.goaded_by.clone(),
            monstrous: self.monstrous,
            renowned: self.renowned,
            suspected: self.suspected,
            adventuring: self.adventuring,
            on_adventure: self.on_adventure,
            omen_casting: self.omen_casting,
            cast_as_prototype: self.cast_as_prototype,
            saddled: self.saddled,
            saddled_by: self.saddled_by.clone(),
            crewed_by: self.crewed_by.clone(),
            split_cast: self.split_cast,
            exiled_by: self.exiled_by,
            exiled_with: self.exiled_with,
            entered_turn: self.entered_turn,
            battlefield_timestamp: self.battlefield_timestamp,
            detained_by: self.detained_by,
            compleated_life_paid: self.compleated_life_paid,
            meld_parts: self.meld_parts.clone(),
            mutate_stack: self.mutate_stack.clone(),
            mutate_onto: self.mutate_onto,
            untap_locked_by: self.untap_locked_by,
            created_by: self.created_by,
            protected_by: self.protected_by,
            resolve_riders: self.resolve_riders,
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
        c.perm_power_bonus = wire.perm_power_bonus;
        c.perm_toughness_bonus = wire.perm_toughness_bonus;
        c.counters = wire.counters.into_iter().collect();
        c.attached_to = wire.attached_to;
        c.attached_to_player = wire.attached_to_player;
        c.soulbond_partner = wire.soulbond_partner;
        c.kicked = wire.kicked;
        c.kicked_options = wire.kicked_options;
        c.kick_count = wire.kick_count;
        c.squad_count = wire.squad_count;
        c.cast_mana_spent = wire.cast_mana_spent;
        c.cast_x_value = wire.cast_x_value;
        c.cast_mana_spent_by_color = wire.cast_mana_spent_by_color;
        c.bargained = wire.bargained;
        c.pending_etb_counters = wire.pending_etb_counters.clone();
        c.spliced_effects = wire.spliced_effects.clone();
        c.spliced_names = wire.spliced_names.clone();
        c.encoded_on = wire.encoded_on;
        c.cast_target_was_battlefield = wire.cast_target_was_battlefield;
        c.granted_activated_abilities = wire.granted_activated_abilities;
        c.granted_activated_eot = wire.granted_activated_eot;
        c.bought_back = wire.bought_back;
        c.entwined = wire.entwined;
        c.gift_promised = wire.gift_promised;
        c.bestowed = wire.bestowed;
        c.face_down = wire.face_down;
        // CR 708 — restore a face-down permanent: stash the real definition
        // (resolved from `name`) and swap to the vanilla 2/2 face-down body.
        if wire.face_down_permanent {
            // Set `cloaked` first so the warded face-down body is restored.
            c.cloaked = wire.cloaked;
            c.turn_face_down();
        }
        // CR 712 — restore a transformed permanent: stash the front, flip the
        // active definition to the back face.
        if wire.transformed
            && let Some(back) = c.definition.back_face.as_ref().map(|b| (**b).clone())
        {
            c.front_face = Some(c.definition.clone());
            c.definition = Arc::new(back);
            c.transformed = true;
        }
        // CR 710 — restore a flipped permanent: stash the top, swap the active
        // definition to the flip face.
        if wire.flipped
            && let Some(flip) = c.definition.flip_face.as_ref().map(|f| (**f).clone())
        {
            c.unflipped_def = Some(c.definition.clone());
            c.definition = Arc::new(flip);
            c.flipped = true;
        }
        // CR 709.5 — restore a Room permanent's unlocked doors (the live
        // definition is the union of unlocked doors' abilities).
        if wire.unlocked_doors != 0 && c.definition.room.is_some() {
            c.unlocked_doors = wire.unlocked_doors;
            c.definition = Arc::new(c.definition.room_definition_with(wire.unlocked_doors));
        }
        // MKM — restore a solved Case (switch on its "Solved —" abilities).
        if wire.case_solved && c.definition.case.is_some() {
            c.case_solved = true;
            c.definition = Arc::new(c.definition.case_definition_solved());
        }
        // CR 716.2 — Class level round-trips as a plain field (abilities are
        // predicate-gated, so no definition rebuild is needed).
        c.class_level = wire.class_level;
        c.is_token = wire.is_token;
        c.loyalty_uses_this_turn = wire.loyalty_uses_this_turn;
        c.loyalty_twice_this_turn = wire.loyalty_twice_this_turn;
        c.evoked = wire.evoked;
        c.dashed = wire.dashed;
        c.cast_from_suspend = wire.cast_from_suspend;
        c.cast_from_escape = wire.cast_from_escape;
        c.blitzed = wire.blitzed;
        c.warped = wire.warped;
        c.impending_counters = wire.impending_counters;
        c.cast_from_hand = wire.cast_from_hand;
        c.cast_via_flashback = wire.cast_via_flashback;
        c.cast_via_mayhem = wire.cast_via_mayhem;
        c.cast_via_waterbend = wire.cast_via_waterbend;
        c.cast_collected_evidence = wire.cast_collected_evidence;
        c.cast_from_exile = wire.cast_from_exile;
        c.cast_from_library = wire.cast_from_library;
        c.modes_chosen = wire.modes_chosen;
        c.chosen_creature_type = wire.chosen_creature_type;
        c.chosen_land_type = wire.chosen_land_type;
        c.chosen_number = wire.chosen_number;
        c.chosen_mode = wire.chosen_mode;
        // CR 614 — re-bake the recorded Siege mode onto the freshly-resolved
        // definition (the name→factory lookup returns the printed card).
        if let Some(idx) = wire.chosen_mode
            && let Some(def) = c.definition.with_mode_applied(idx as usize)
        {
            c.definition = Arc::new(def);
        }
        c.chosen_card_type = wire.chosen_card_type;
        c.echo_paid = wire.echo_paid;
        c.granted_suspend = wire.granted_suspend;
        c.phased_out_by = wire.phased_out_by;
        c.name_choices_used = wire.name_choices_used;
        c.chosen_permanent = wire.chosen_permanent;
        c.chosen_player = wire.chosen_player;
        c.once_per_turn_used = wire.once_per_turn_used;
        c.exhausted_abilities = wire.exhausted_abilities;
        c.may_play_until = wire.may_play_until;
        c.keyword_counters = wire.keyword_counters.into_iter().collect();
        c.granted_keywords_eot = wire.granted_keywords_eot;
        c.granted_keywords_eot_ts = wire.granted_keywords_eot_ts;
        c.removed_keywords_eot = wire.removed_keywords_eot;
        c.removed_keywords = wire.removed_keywords;
        c.granted_flashback_eot = wire.granted_flashback_eot;
        c.granted_harmonize_eot = wire.granted_harmonize_eot;
        c.granted_alt_cast_cost_eot = wire.granted_alt_cast_cost_eot;
        c.granted_cast_surcharge_eot = wire.granted_cast_surcharge_eot;
        c.named_card = wire.named_card;
        c.chosen_color = wire.chosen_color;
        c.chosen_colors = wire.chosen_colors;
        c.goaded_by = wire.goaded_by;
        c.monstrous = wire.monstrous;
        c.renowned = wire.renowned;
        c.suspected = wire.suspected;
        c.adventuring = wire.adventuring;
        c.on_adventure = wire.on_adventure;
        c.omen_casting = wire.omen_casting;
        // CR 702.160 — restore a prototype-cast permanent: the name→factory
        // definition is the full (colorless, big) card, so re-apply the
        // prototype cost/color/size that it entered with.
        if wire.cast_as_prototype
            && let Some(proto_def) = c.definition.with_prototype_applied()
        {
            c.prototype_printed = Some(c.definition.clone());
            c.definition = Arc::new(proto_def);
            c.cast_as_prototype = true;
        }
        c.saddled = wire.saddled;
        c.saddled_by = wire.saddled_by;
        c.crewed_by = wire.crewed_by;
        c.split_cast = wire.split_cast;
        c.exiled_by = wire.exiled_by;
        c.exiled_with = wire.exiled_with;
        c.remembered_amount = wire.remembered_amount;
        c.entered_turn = wire.entered_turn;
        c.battlefield_timestamp = wire.battlefield_timestamp;
        c.detained_by = wire.detained_by;
        c.compleated_life_paid = wire.compleated_life_paid;
        c.meld_parts = wire.meld_parts;
        c.mutate_stack = wire.mutate_stack;
        c.mutate_onto = wire.mutate_onto;
        if !c.mutate_stack.is_empty() {
            c.rebuild_mutate_definition();
        }
        c.untap_locked_by = wire.untap_locked_by;
        c.created_by = wire.created_by;
        c.protected_by = wire.protected_by;
        c.resolve_riders = wire.resolve_riders;
        Ok(c)
    }
}
