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
    Knight, Soldier, Wizard, Cleric, Rogue, Warrior, Beast, Bird, Soltari, Dauthi,
    Elemental, Djinn, Efreet, Horror, Specter, Cat, Insect, Spider, Wurm,
    Bear, Ape, Rat, Fungus, Treefolk, Giant, Ogre, Shaman, Druid,
    Monk, Archer, Berserker, Barbarian, Artificer, Pirate, Scout, Mongoose,
    Advisor, Assassin, Faerie, Skeleton, Spirit, Wall, Illusion,
    Hydra, Sphinx, Phoenix, Minotaur, Centaur, Cyclops, Satyr, Nymph,
    Kithkin, Viashino, Eldrazi, Sliver, Shapeshifter, Troll,
    Imp, Nightmare, Shade, Minion, Thrull, Carrier, Devil, Wraith,
    Drake, Griffin, Hippogriff, Pegasus, Unicorn, Horse, Hound, Wolf, Werewolf, Fox, Dog,
    Jackal,
    Serpent, Fish, Octopus, Squid, Jellyfish, Crab, Turtle, Frog, Crocodile,
    Dinosaur, Lizard, Snake, Scorpion, Bat, Squirrel, Ox, Boar, Goat, Llama, Shark,
    Elephant, Rhino, Hippo, Mammoth, Whale, Leviathan, Kraken, Elk,
    Lion, Kavu, Lhurgoyf, Atog, Noggle, Vedalken, Kor, Ally,
    Avatar, Phyrexian, Praetor, Incarnation, Mercenary, Rebel, Archon, Aetherborn,
    Construct, Golem, Myr, Robot, Hellion, Scarecrow,
    Ooze, Plant, Saproling,
    // Strixhaven-era subtypes.
    Inkling, Pest, Fractal,
    Orc, Warlock, Bard, Sorcerer, Pilot,
    // Misc. subtypes used by SOS body-only cards.
    Dwarf, Badger, Salamander, Giraffe,
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
    // Cube expansion (Collector Ouphe).
    Ouphe,
    // MKM / LCI expansion (Spyglass Siren, Inside Source, Slimy Dualleech).
    Siren, Citizen, Leech,
    // Bloomburrow / MKM (Voracious Varmint).
    Varmint,
    // Enchantment creature subtype (Enduring Innocence).
    Glimmer,
    // Ninjutsu creature subtype (Fallen Shinobi, etc.).
    Ninja,
    // Outlaws of Thunder Junction Mount subtype (Saddle, CR 702.171).
    Mount,
    // Eldraine Peasant subtype (Curious Pair, Giant Killer).
    Peasant,
    // Bloomburrow (2024) animal-folk subtypes.
    Rabbit, Raccoon, Mouse, Wolverine, Mole,
    // +1/+1-counter "Spike" cycle (Spike Feeder).
    Spike,
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
    // Amonkhet Naga (Ramunap Excavator).
    Naga,
    // Strixhaven artifact-creature subtype (Biblioplex Assistant).
    Gargoyle,
    // Innistrad Eye Horror (Concealing Curtains // Revealing Eye).
    Eye,
    // Manland animate-into bodies (Mishra's Factory, Inkmoth / Blinkmoth Nexus).
    AssemblyWorker, Blinkmoth,
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
    // Lost Caverns of Ixalan explore token (CR 111.10s).
    Map,
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
    // modern_decks: Narset, Parter of Veils.
    Narset,
    // STX: Kasmina, Enigma Sage.
    Kasmina,
    // STX Dean/Strixhaven planeswalker MDFCs.
    Rowan, Will, Lukka,
    // Cube expansion: Wrenn (Wrenn and Six).
    Wrenn,
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
    /// -0/-1 counter (Shield Sphere-adjacent block-cost counters).
    MinusZeroMinusOne,
    /// -1/-0 counter.
    MinusOneMinusZero,
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
    /// Study counter — Strixhaven bookkeeping marker on cards in exile
    /// (Kianne // Imbraham). Cards exiled with study counters are tallied by
    /// `Value::DistinctManaValuesInExileWithCounter` (Kianne's Fractal) and
    /// returned via `Effect::ReturnFromExileWithCounter` (Imbraham).
    Study,
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
    /// "...for as long as it remains exiled" — never sweeps; the
    /// permission dies with the card's zone change (Hostage Taker, Gonti).
    WhileExiled,
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
    /// CR 702.147 — Decayed. "This creature can't block. When it attacks,
    /// sacrifice it at end of combat." Common on Zombie tokens (MID/VOW).
    Decayed,
    Protection(Color),
    /// CR 702.16h — "protection from colored spells" (Emrakul, the Aeons
    /// Torn). Enforced as a cast-time targeting gate: a spell with one or
    /// more colors can't target this permanent.
    ProtectionFromColoredSpells,
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
    /// CR 702.103 — Jump-start: cast from the graveyard for the card's own
    /// mana cost plus discarding a card; exiles after resolving (rides the
    /// flashback cast path). Chemister's Insight, Radical Idea.
    JumpStart,
    Kicker(crate::mana::ManaCost),
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
    Cycling(crate::mana::ManaCost),
    /// CR 702.29 — Cycling whose cost is a life payment instead of mana
    /// ("Cycling—Pay 2 life", Street Wraith).
    CyclingLife(u32),
    /// CR 702.29e — Typecycling (landcycling). A variant of Cycling: pay the
    /// cost + discard this card, then search your library for a land card with
    /// the given land type, reveal it, and put it into your hand (then
    /// shuffle). "When you cycle" triggers still fire (it is a cycling ability).
    Landcycling(crate::mana::ManaCost, LandType),
    Echo(crate::mana::ManaCost),
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
    Banding,
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
    /// CR 702.69 — "When you cast this spell, copy it for each permanent put
    /// into a graveyard from the battlefield this turn." A self-cast copy
    /// rider mirroring Storm but counting `permanents_to_graveyard_this_turn`.
    Gravestorm,
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
    /// "This creature assigns no combat damage this turn" (Master of
    /// Cruelties' attack rider). A marker keyword — typically granted with
    /// `Duration::EndOfTurn` by a trigger — that `combat.rs` checks off the
    /// *computed* keyword set to zero out the bearer's combat damage in
    /// both the first-strike and regular damage steps.
    DealsNoCombatDamage,
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
    /// CR 702.95 — Soulbond. A marker keyword; when this or another creature
    /// enters while either is unpaired, its controller may pair them. The
    /// pairing rides `CardInstance.soulbond_partner`, and the bonus each
    /// paired creature gains is carried in `CardDefinition.soulbond_bonus`.
    /// Deadeye Navigator, Wolfir Silverheart.
    Soulbond,
}

/// CR 702.24 — the maintenance cost paid (once per age counter) to keep a
/// permanent with cumulative upkeep. Mana (Mystic Remora), life (Glacial
/// Chasm), or sacrificing a matching permanent (Phyrexian Soulgorger).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CumulativeUpkeepCost {
    Mana(crate::mana::ManaCost),
    Life(u32),
    Sacrifice(Box<SelectionRequirement>),
}

impl CumulativeUpkeepCost {
    /// Short human-readable cost phrase for tooltips/logs ("{2}", "Pay 1
    /// life", "Sacrifice").
    pub fn summary(&self) -> String {
        match self {
            CumulativeUpkeepCost::Mana(c) => c.summary(),
            CumulativeUpkeepCost::Life(n) => format!("Pay {n} life"),
            CumulativeUpkeepCost::Sacrifice(_) => "Sacrifice".into(),
        }
    }
}

/// Composable filter for valid targets of a spell or ability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// The card has any cycling ability (Cycling / CyclingLife /
    /// Landcycling — CR 702.29). Zenith Flare's graveyard count.
    HasCyclingAbility,
    PowerAtMost(i32),
    ToughnessAtMost(i32),
    WithCounter(CounterType),
    /// True when the candidate has no counters of any kind on it (CR 122).
    /// Powers "target creature with no counters on it" (Heartless Act mode 0).
    HasNoCounters,
    /// True when the candidate's name differs from every card moved so far
    /// in the current resolution — "with different names" multi-search
    /// clauses (Saheeli Rai's -7).
    NameDiffersFromLastMoved,
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
    IsBasicLand,
    /// True for a land that is **not** basic (CR 305.6) — i.e. a land card
    /// lacking the Basic supertype. Powers Thalia, Heretic Cathar's
    /// "nonbasic lands … enter the battlefield tapped" clause and any
    /// nonbasic-land targeting filter.
    IsNonbasicLand,
    IsAttacking,
    IsBlocking,
    /// True when the candidate creature was declared as an attacker at any
    /// point this turn (`CardInstance.attacked_this_turn`). Relentless
    /// Assault's "untap all creatures that attacked this turn".
    AttackedThisTurn,
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
    /// True when the candidate permanent has an Aura attached to it (CR 303
    /// "enchanted permanent"). Battlefield-only: scans for any enchantment
    /// whose `attached_to` points at the candidate. Powers Kestia's
    /// "whenever an enchanted creature … you control attacks" trigger.
    IsEnchanted,
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
    ManaValueAtMost(u32),
    /// Mana value ≤ the X paid into the resolving spell's cost. Resolved to
    /// a concrete `ManaValueAtMost(x)` by `resolve_x` at search-resolution
    /// time (Chord of Calling); unresolved instances evaluate false.
    ManaValueAtMostXFromCost,
    ManaValueAtLeast(u32),
    /// True when the card's mana value (CMC) is exactly `n`. Useful for
    /// effects that want a precise CMC gate (Fix What's Broken returns
    /// "with mana value equal to X", which requires this exact-match
    /// shape rather than the `AtMost`/`AtLeast` approximations).
    /// Composes naturally with `And`/`Or` for range gates.
    ManaValueExactly(u32),
    /// True when the card's mana value equals the most-recently-sacrificed
    /// creature's mana value plus `offset`, read from the resolution scratch
    /// (`GameState.sacrificed_mana_value`). Powers Birthing Pod's "search for a
    /// creature card with mana value equal to 1 plus the sacrificed creature's
    /// mana value." Evaluates to `mv == 0 + offset` when nothing was recorded.
    ManaValueEqualsSacrificedPlus(u32),
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
    /// True when the candidate's name matches the name the resolving
    /// source stamped via `Effect::NameCard` (its `named_card` field).
    /// Used by reveal-until-the-named-card effects (Spoils of the Vault):
    /// pair `NameCard { This }` with a `RevealUntilFind { find:
    /// NamedBySource, .. }`. Falls back to "no match" when the source has
    /// not named a card.
    NamedBySource,
    /// True when the candidate's mana value is ≤ the number of permanents
    /// the evaluating player controls that match the inner filter. Powers
    /// "with mana value less than or equal to the number of [X] you
    /// control" gates — Lay Down Arms ("…number of Plains you control").
    /// Battlefield/zone candidate; the count walks the battlefield for
    /// permanents matching `inner` under the evaluating controller.
    ManaValueAtMostControlledCount(Box<SelectionRequirement>),
    /// True when the candidate's mana value is ≤ the number of cards in its
    /// own controller's graveyard. Powers Drown in the Loch's "counter/
    /// destroy target with mana value ≤ cards in its controller's graveyard"
    /// gate. Works on stack spells and battlefield permanents alike (both
    /// carry a `controller`).
    ManaValueAtMostControllerGraveyard,
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

    /// Replace X-dependent atoms with concrete values
    /// (`ManaValueAtMostXFromCost` → `ManaValueAtMost(x)`), recursing
    /// through And/Or/Not. Called by `Effect::Search` with the resolving
    /// spell's paid X (Chord of Calling, CR 601.2b).
    pub fn resolve_x(&self, x: u32) -> Self {
        match self {
            Self::ManaValueAtMostXFromCost => Self::ManaValueAtMost(x),
            Self::And(a, b) => Self::And(Box::new(a.resolve_x(x)), Box::new(b.resolve_x(x))),
            Self::Or(a, b) => Self::Or(Box::new(a.resolve_x(x)), Box::new(b.resolve_x(x))),
            Self::Not(inner) => Self::Not(Box::new(inner.resolve_x(x))),
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
        SelectionRequirement::ControlledByOpponent => Some("an opponent controls".to_string()),
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
    #[serde(skip)]
    pub static_abilities: Vec<StaticAbility>,
    /// Equip bonus for Equipment tokens (CR 301 — Mabel's Cragflame,
    /// Kellan's Boots, etc.). When `Some`, the resulting token carries
    /// this `equipped_bonus`; pair it with a `Keyword::Equip(cost)` in
    /// `keywords` so the token can be equipped. `None` for non-Equipment
    /// tokens. Defaults to `None` via `#[serde(default)]`.
    #[serde(default)]
    pub equipped_bonus: Option<EquipBonus>,
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
    /// CR 606.3 override — "You may activate the loyalty abilities of [this]
    /// twice each turn rather than only once." (Urza, Planeswalker.)
    #[serde(default)]
    pub loyalty_twice_each_turn: bool,
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
    /// "This spell costs {1} less to cast for each [filter] card in your
    /// graveyard" — the graveyard-counting sibling of `affinity_filter`.
    /// Generic-only, clamped by the caller. Powers Tolarian Terror /
    /// The Dawning Archaic (instant-or-sorcery), Murktide-class delve-flavor
    /// reductions, etc. Defaults to `None` for snapshot back-compat.
    #[serde(default)]
    pub affinity_graveyard_filter: Option<SelectionRequirement>,
    /// "Equipped creature gets +P/+T and has [keywords]." Read by
    /// `compute_battlefield` for any Equipment whose `attached_to` points at
    /// a creature on the battlefield — the bonus is emitted as layer-7 (P/T)
    /// and layer-6 (keyword) continuous effects on the equipped creature.
    /// `None` for non-Equipment cards (and for Equipment whose only relevant
    /// effect is an activated ability, e.g. Lightning Greaves' grant-on-
    /// activate approximation). Defaults to `None` for snapshot back-compat.
    #[serde(default)]
    pub equipped_bonus: Option<EquipBonus>,
    /// CR 702.95 — Soulbond bonus. When `Some`, this card carries the Soulbond
    /// keyword and, while paired (`CardInstance.soulbond_partner`), confers
    /// this bonus on BOTH itself and its partner. Defaults to `None`.
    #[serde(default)]
    pub soulbond_bonus: Option<SoulbondBonus>,
    /// CR 601.2b/601.2f — additional cost(s) paid as the spell is cast
    /// ("As an additional cost to cast this spell, …"). Paid during
    /// casting, not folded into resolution: the spell can't be cast unless
    /// every cost is payable, and a sacrifice/discard/life payment happens
    /// immediately (death/discard triggers fire before the spell resolves).
    /// Powers Necrotic Fumes, Tend the Pests, Witherbloom Sacrosanct.
    /// Defaults to empty via `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub additional_cast_cost: Vec<AdditionalCastCost>,
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
    /// miracle alt-cost (`granted_alt_cast_cost_eot` + `may_play_until`).
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
    // Skipped on the wire like `TokenDefinition.static_abilities` — a Room
    // instance round-trips by card name + `unlocked_doors`, and the live
    // definition is rebuilt from the catalog factory on load.
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
    pub name: &'static str,
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
}

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
    /// "As an additional cost, discard N card(s)."
    Discard { count: u32 },
    /// "As an additional cost to cast this spell, return N permanent(s) you
    /// control matching `filter` to their owner's hand." Devour in Flames
    /// ("return a land you control"). Auto-picker bounces the lowest-impact
    /// matches (tapped first, then lowest mana value).
    ReturnToHand {
        filter: SelectionRequirement,
        #[serde(default = "one_u32")]
        count: u32,
    },
    /// "As an additional cost to cast this spell, exile a [filter] card from
    /// your graveyard." The exiled card's mana value becomes the spell's X
    /// (read at resolution via `Value::XFromCost`) — Draconic Intervention
    /// ("X is the exiled card's mana value"). Auto-picker exiles the lowest-MV
    /// match. Cast is rejected if no matching card is in the graveyard.
    ExileFromGraveyard {
        filter: SelectionRequirement,
    },
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
    /// Optional board-count scaling (CR 613 layer 7c): the attached creature
    /// gets an additional `per_power`/`per_toughness` for each permanent
    /// matching `filter` the source's controller controls, on top of the flat
    /// `power`/`toughness` (Nettlecyst — "+1/+1 for each artifact and/or
    /// enchantment you control"). `None` for the common static-bonus case.
    #[serde(default)]
    pub scale: Option<EquipScale>,
    /// Triggered abilities granted to the equipped creature (CR 702.6e). Each
    /// fires as though printed on the equipped creature — `EventScope::
    /// SelfSource` reads the creature, and a `DealsCombatDamageToPlayer` body
    /// can refer to the damaged player via `PlayerRef::Target(0)`. The Sword
    /// cycle's combat-damage triggers. Empty for the common static-bonus case.
    #[serde(default)]
    pub triggered_abilities: Vec<crate::effect::TriggeredAbility>,
    /// When true, `triggered_abilities` resolve with the **Equipment** as the
    /// trigger source (so `Selector::This` reads the Equipment, not the equipped
    /// creature). Umezawa's Jitte's "whenever equipped creature deals combat
    /// damage, put two charge counters on this Equipment". `false` keeps the
    /// default CR 702.6e behavior (the ability fires off the creature).
    #[serde(default)]
    pub triggers_on_equipment: bool,
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
/// `EquipBonus.scale`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipScale {
    pub filter: SelectionRequirement,
    pub per_power: i32,
    pub per_toughness: i32,
    /// When set, the count is the number of counters of this kind on the
    /// source permanent itself (Lion Sash — "+1/+1 for each +1/+1 counter on
    /// this Equipment") rather than the controlled-permanent count by `filter`.
    #[serde(default)]
    pub count_self_counters: Option<CounterType>,
}

/// Characteristic-defining dynamic P/T formula. Read by
/// `compute_battlefield` to inject a layer-7 `SetPowerToughness` for
/// the named card. Each variant encodes both the power and toughness
/// expression so the engine doesn't have to know the printed Oracle's
/// wording — just the two scalars to set.
///
/// Stored on `CardDefinition.dynamic_pt`; adding a new dynamic-P/T card
/// sets that field (no engine-side table).
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
    /// Power = toughness = the number of creatures of `creature_type` the
    /// controller controls (counting the source). Pack Rat.
    CreaturesOfTypeControlled { creature_type: CreatureType },
    /// Power = toughness = `base` + the number of lands the controller
    /// controls. Lumra, Bellow of the Woods (base 0/0).
    LandsControlled { base: i32 },
    /// Power = toughness = `base` + the number of artifacts the controller
    /// controls (counting the source). Broodstar (base 0/0, CR 604.3 CDA).
    ArtifactsControlled { base: i32 },
    /// Power = number of instant and sorcery cards in the controller's
    /// graveyard and exile; toughness = `base_t`. Crackling Drake (0/4).
    InstantsSorceriesInGraveyardAndExile { base_t: i32 },
    /// Imprint CDA (CR 604.3): P/T of the creature card exiled with this
    /// permanent; printed base when nothing is exiled. Duplicant.
    ExiledWithSourcePt { base_p: i32, base_t: i32 },
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

    /// Printed colors from the mana cost's colored pips (CR 105.2), with
    /// the Devoid CDA (CR 702.114) yielding colorless.
    pub fn printed_colors(&self) -> Vec<crate::mana::Color> {
        use crate::mana::ManaSymbol;
        if self.keywords.contains(&Keyword::Devoid) {
            return Vec::new();
        }
        let mut colors = Vec::new();
        let mut push = |c: crate::mana::Color| {
            if !colors.contains(&c) {
                colors.push(c);
            }
        };
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
        }
    }

    /// Spend-restriction context for activating an ability of this card
    /// ("… or activate abilities of artifacts" — Power Depot).
    pub fn ability_spend_kind(&self) -> crate::mana::SpellKind {
        crate::mana::SpellKind { artifact: self.is_artifact(), ..Default::default() }
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
    /// CR 702.103 — the Bestow mana cost, if this card has Bestow.
    pub fn has_bestow(&self) -> Option<&ManaCost> {
        self.bestow.as_ref()
    }
    /// CR 715 — the adventure half, if this card has Adventure.
    pub fn has_adventure(&self) -> Option<&Adventure> {
        self.adventure.as_deref()
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
    pub fn has_kicker(&self) -> Option<&ManaCost> {
        // Offspring (CR 702.166) is an optional additional cast cost that
        // reuses the Kicker pipeline (pay it → `SpellWasKicked` → ETB mints a
        // 1/1 token copy). A card carries one or the other, not both.
        self.keywords.iter().find_map(|kw| match kw {
            Keyword::Kicker(cost) | Keyword::Offspring(cost) => Some(cost),
            _ => None,
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
    /// CR 702.35 — the Madness cost if this card has `Keyword::Madness`.
    pub fn madness_cost(&self) -> Option<&ManaCost> {
        self.keywords.iter().find_map(|kw| {
            if let Keyword::Madness(cost) = kw { Some(cost) } else { None }
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
    /// CR 702.95 — the creature this one is Soulbond-paired with, if any.
    /// Either member of a pair points at the other. Cleared when either
    /// creature leaves the battlefield (SBA in `stack.rs`).
    pub soulbond_partner: Option<CardId>,
    pub kicked: bool,
    /// CR 702.157 — how many times this spell's optional Squad cost was paid.
    /// Persists onto the permanent so its ETB mints one token copy per payment
    /// (`Value::SquadCount`). Defaults to 0 (no Squad / not paid).
    pub squad_count: u32,
    /// CR 702.176 — true if this spell was cast paying its optional Bargain
    /// cost (sacrifice an artifact, enchantment, or token). Read at resolution
    /// by `Predicate::SpellWasBargained`.
    pub bargained: bool,
    /// CR 702.27 — true if this spell was cast paying its optional Buyback
    /// cost. On resolution the resolver returns the card to its owner's
    /// hand instead of the graveyard.
    pub bought_back: bool,
    /// CR 702.41 — true if this spell was cast paying its optional Entwine
    /// cost: its `ChooseMode` runs every mode in order.
    pub entwined: bool,
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
    pub is_token: bool,
    /// CR 606.3 — loyalty activations so far this turn (normally capped at
    /// one; two with `CardDefinition.loyalty_twice_each_turn`).
    pub loyalty_uses_this_turn: u8,
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
    /// True if this card was cast from a graveyard via its Flashback
    /// cost. On resolution the resolver routes the card to exile instead
    /// of the owner's graveyard. Replaces an earlier overload of the
    /// `kicked` flag so that a card with both Kicker and Flashback can
    /// be disambiguated cleanly.
    pub cast_via_flashback: bool,
    /// True if this card was cast from exile on its current trip through the
    /// stack (suspend/foretell/plot/impulse free or alt-cost casts). Powers
    /// "whenever you cast a spell from exile" payoffs (Nassari, Dean of
    /// Expression). Cleared when the card leaves the stack.
    pub cast_from_exile: bool,
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
    /// Keywords removed until end of turn via `Effect::LoseKeywordThisTurn`
    /// (Shadowspear's "creatures your opponents control lose hexproof and
    /// indestructible until end of turn"). Removal beats printed/granted/
    /// counter sources for the turn; cleared at Cleanup.
    pub removed_keywords_eot: Vec<Keyword>,
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
    /// CR 702.142 — set when this creature is declared as an attacker;
    /// gates Boast activated abilities ("activate only if this attacked
    /// this turn"). Cleared in per-turn cleanup. Transient — not
    /// serialized (defaults to false on snapshot reload).
    pub attacked_this_turn: bool,
    /// CR 702.39 — Provoke: the attacker this creature must block this
    /// combat if able. Set when an attacker provokes it (untap + force
    /// block); cleared at end of combat. Transient — not serialized.
    pub must_block: Option<CardId>,
    /// CR 603.6e — set on a card in exile that a permanent's "exile until
    /// ~ leaves" ability put there. When that source leaves the
    /// battlefield the engine returns this card to `ExileLink::return_to`.
    /// `None` for ordinary (permanent) exile.
    pub exiled_by: Option<ExileLink>,
    /// CR 720-style "exiled with [a permanent]": a permanent tag stamped on a
    /// card in exile that some source put there and keeps caring about (e.g.
    /// Keen-Eyed Curator's "card types among cards exiled with this
    /// creature"). Unlike `exiled_by`, the card never returns — this is a
    /// pure association used by counting effects. `None` for ordinary exile.
    pub exiled_with: Option<CardId>,
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
    /// Alternative cost the controller may pay to cast this card via its
    /// `may_play_until` permission instead of casting it for free — the
    /// "miracle {N}" cost granted by Lorehold, the Historian. Read by
    /// `cast_from_zone_without_paying`; its lifetime tracks `may_play_until`
    /// (cleared together by the expiry sweep and when a cast consumes the
    /// permission). `None` for ordinary free may-play grants.
    pub granted_alt_cast_cost_eot: Option<crate::mana::ManaCost>,
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
    /// CR 702.171 — true while this permanent is saddled (a marker set by a
    /// Saddle activation, until end of turn). Read by `Predicate::SourceSaddled`
    /// to gate "whenever this attacks while saddled" triggers. Cleared by
    /// `clear_end_of_turn_effects`.
    pub saddled: bool,
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
    /// CR 701.35 — Detain. `Some(detainer)` while this permanent is detained:
    /// it can't attack or block and its activated abilities can't be activated
    /// until the detaining player's next turn (cleared at the start of that
    /// player's turn). `None` for the common undetained case. Round-trips with
    /// `#[serde(default)]`.
    pub detained_by: Option<usize>,
    /// CR 712.16 — the two component cards of a melded permanent. Non-empty
    /// only on a melded object; when it leaves the battlefield, the parts go
    /// to that zone instead and the melded shell ceases to exist.
    pub meld_parts: Vec<CardInstance>,
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
            soulbond_partner: None,
            kicked: false,
            squad_count: 0,
            bargained: false,
            bought_back: false,
            entwined: false,
            bestowed: false,
            face_down: false,
            transformed: false,
            front_face: None,
            unlocked_doors: 0,
            face_up_def: None,
            cloaked: false,
            is_token: false,
            loyalty_uses_this_turn: 0,
            evoked: false,
            dashed: false,
            cast_from_suspend: false,
            cast_from_escape: false,
            blitzed: false,
            impending_counters: 0,
            cast_from_hand: false,
            cast_target_was_battlefield: false,
            cast_via_flashback: false,
            cast_from_exile: false,
            chosen_creature_type: None,
            once_per_turn_used: Vec::new(),
            granted_keywords_eot: Vec::new(),
            removed_keywords_eot: Vec::new(),
            keyword_counters: std::collections::HashMap::new(),
            may_play_until: None,
            dealt_deathtouch_damage: false,
            regeneration_shields: 0,
            skip_next_untap: false,
            attacked_this_turn: false,
            must_block: None,
            exiled_by: None,
            exiled_with: None,
            encoded_on: None,
            granted_flashback_eot: None,
            granted_alt_cast_cost_eot: None,
            named_card: None,
            chosen_color: None,
            goaded_by: Vec::new(),
            monstrous: false,
            suspected: false,
            adventuring: false,
            on_adventure: false,
            saddled: false,
            split_cast: None,
            entered_turn: None,
            detained_by: None,
            meld_parts: Vec::new(),
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
        let minus_one_zero = self.counter_count(CounterType::MinusOneMinusZero) as i32;
        self.definition.base_power() + self.power_bonus + plus - minus - minus_one_zero
    }

    pub fn toughness(&self) -> i32 {
        let plus = self.counter_count(CounterType::PlusOnePlusOne) as i32;
        let minus = self.counter_count(CounterType::MinusOneMinusOne) as i32;
        let minus_zero_one = self.counter_count(CounterType::MinusZeroMinusOne) as i32;
        self.definition.base_toughness() + self.toughness_bonus + plus - minus - minus_zero_one
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
        if self.removed_keywords_eot.contains(kw) {
            return false;
        }
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

    /// CR 708 — flip this permanent face down: stash the real definition in
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

    pub fn clear_end_of_turn_effects(&mut self) {
        self.power_bonus = 0;
        self.toughness_bonus = 0;
        self.loyalty_uses_this_turn = 0;
        self.once_per_turn_used.clear();
        self.granted_keywords_eot.clear();
        self.removed_keywords_eot.clear();
        self.granted_flashback_eot = None;
        self.granted_alt_cast_cost_eot = None;
        self.dealt_deathtouch_damage = false;
        // CR 701.15g — unused regeneration shields expire at end of turn.
        self.regeneration_shields = 0;
        // CR 702.171 — "saddled until end of turn" ends here.
        self.saddled = false;
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
    /// CR 702.95 Soulbond partner. `#[serde(default)]` so older snapshots load
    /// as `None`.
    #[serde(default)]
    soulbond_partner: Option<CardId>,
    kicked: bool,
    /// CR 702.27 buyback flag. `#[serde(default)]` so older snapshots load
    /// as `false`.
    #[serde(default)]
    bought_back: bool,
    /// CR 702.41 entwine flag. `#[serde(default)]` for back-compat.
    #[serde(default)]
    entwined: bool,
    /// CR 702.103 bestowed flag. `#[serde(default)]` so older snapshots load
    /// as `false`.
    #[serde(default)]
    bestowed: bool,
    face_down: bool,
    /// CR 709.5c — Room unlocked-door bitmask. `#[serde(default)]` for
    /// back-compat; the live definition is rebuilt on load.
    #[serde(default)]
    unlocked_doors: u8,
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
    is_token: bool,
    #[serde(default)]
    loyalty_uses_this_turn: u8,
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
    impending_counters: u32,
    cast_from_hand: bool,
    /// `#[serde(default)]` so snapshots predating the field deserialize
    /// as `false` (matching the old "not cast via flashback" path).
    #[serde(default)]
    cast_via_flashback: bool,
    #[serde(default)]
    cast_from_exile: bool,
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
    /// Until-end-of-turn keyword removals (Shadowspear). Shares
    /// `granted_keywords_eot`'s lifetime. `#[serde(default)]` for back-compat.
    #[serde(default)]
    removed_keywords_eot: Vec<Keyword>,
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
    /// CR 201.3 named card (Pithing Needle / Phyrexian Revoker). Persistent
    /// state; `#[serde(default)]` so older snapshots load as `None`.
    #[serde(default)]
    named_card: Option<String>,
    /// Chosen color (Coldsteel Heart-style mana rocks). `#[serde(default)]`
    /// so older snapshots load as `None`.
    #[serde(default)]
    chosen_color: Option<crate::mana::Color>,
    /// CR 701.38 goad — players who have goaded this creature.
    /// `#[serde(default)]` so older snapshots load as empty.
    #[serde(default)]
    goaded_by: Vec<usize>,
    /// CR 701.31 monstrous flag. `#[serde(default)]` so older snapshots load
    /// as `false`.
    #[serde(default)]
    monstrous: bool,
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
    /// CR 702.171 saddled marker. `#[serde(default)]` so older snapshots
    /// load as `false`.
    #[serde(default)]
    saddled: bool,
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
    /// CR 701.35 Detain marker. `#[serde(default)]` so older snapshots load as
    /// `None`.
    #[serde(default)]
    detained_by: Option<usize>,
    /// CR 712.16 melded-component cards. `#[serde(default)]` so older
    /// snapshots load as empty.
    #[serde(default)]
    meld_parts: Vec<CardInstance>,
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
                .map(|f| f.name.to_string())
                .unwrap_or_else(|| self.definition.name.to_string()),
            owner: self.owner,
            controller: self.controller,
            tapped: self.tapped,
            damage: self.damage,
            summoning_sick: self.summoning_sick,
            power_bonus: self.power_bonus,
            toughness_bonus: self.toughness_bonus,
            counters: self.counters.iter().map(|(k, v)| (*k, *v)).collect(),
            attached_to: self.attached_to,
            soulbond_partner: self.soulbond_partner,
            kicked: self.kicked,
            bought_back: self.bought_back,
            entwined: self.entwined,
            bestowed: self.bestowed,
            face_down: self.face_down,
            unlocked_doors: self.unlocked_doors,
            face_down_permanent: self.face_up_def.is_some(),
            cloaked: self.cloaked,
            transformed: self.transformed,
            is_token: self.is_token,
            loyalty_uses_this_turn: self.loyalty_uses_this_turn,
            evoked: self.evoked,
            dashed: self.dashed,
            cast_from_suspend: self.cast_from_suspend,
            cast_from_escape: self.cast_from_escape,
            blitzed: self.blitzed,
            impending_counters: self.impending_counters,
            cast_from_hand: self.cast_from_hand,
            cast_via_flashback: self.cast_via_flashback,
            cast_from_exile: self.cast_from_exile,
            chosen_creature_type: self.chosen_creature_type,
            once_per_turn_used: self.once_per_turn_used.clone(),
            may_play_until: self.may_play_until,
            keyword_counters: self
                .keyword_counters
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
            granted_keywords_eot: self.granted_keywords_eot.clone(),
            removed_keywords_eot: self.removed_keywords_eot.clone(),
            granted_flashback_eot: self.granted_flashback_eot.clone(),
            granted_alt_cast_cost_eot: self.granted_alt_cast_cost_eot.clone(),
            named_card: self.named_card.clone(),
            chosen_color: self.chosen_color,
            goaded_by: self.goaded_by.clone(),
            monstrous: self.monstrous,
            suspected: self.suspected,
            adventuring: self.adventuring,
            on_adventure: self.on_adventure,
            saddled: self.saddled,
            split_cast: self.split_cast,
            exiled_by: self.exiled_by,
            exiled_with: self.exiled_with,
            entered_turn: self.entered_turn,
            detained_by: self.detained_by,
            meld_parts: self.meld_parts.clone(),
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
        c.soulbond_partner = wire.soulbond_partner;
        c.kicked = wire.kicked;
        c.bought_back = wire.bought_back;
        c.entwined = wire.entwined;
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
        // CR 709.5 — restore a Room permanent's unlocked doors (the live
        // definition is the union of unlocked doors' abilities).
        if wire.unlocked_doors != 0 && c.definition.room.is_some() {
            c.unlocked_doors = wire.unlocked_doors;
            c.definition = Arc::new(c.definition.room_definition_with(wire.unlocked_doors));
        }
        c.is_token = wire.is_token;
        c.loyalty_uses_this_turn = wire.loyalty_uses_this_turn;
        c.evoked = wire.evoked;
        c.dashed = wire.dashed;
        c.cast_from_suspend = wire.cast_from_suspend;
        c.cast_from_escape = wire.cast_from_escape;
        c.blitzed = wire.blitzed;
        c.impending_counters = wire.impending_counters;
        c.cast_from_hand = wire.cast_from_hand;
        c.cast_via_flashback = wire.cast_via_flashback;
        c.cast_from_exile = wire.cast_from_exile;
        c.chosen_creature_type = wire.chosen_creature_type;
        c.once_per_turn_used = wire.once_per_turn_used;
        c.may_play_until = wire.may_play_until;
        c.keyword_counters = wire.keyword_counters.into_iter().collect();
        c.granted_keywords_eot = wire.granted_keywords_eot;
        c.removed_keywords_eot = wire.removed_keywords_eot;
        c.granted_flashback_eot = wire.granted_flashback_eot;
        c.granted_alt_cast_cost_eot = wire.granted_alt_cast_cost_eot;
        c.named_card = wire.named_card;
        c.chosen_color = wire.chosen_color;
        c.goaded_by = wire.goaded_by;
        c.monstrous = wire.monstrous;
        c.suspected = wire.suspected;
        c.adventuring = wire.adventuring;
        c.on_adventure = wire.on_adventure;
        c.saddled = wire.saddled;
        c.split_cast = wire.split_cast;
        c.exiled_by = wire.exiled_by;
        c.exiled_with = wire.exiled_with;
        c.entered_turn = wire.entered_turn;
        c.detained_by = wire.detained_by;
        c.meld_parts = wire.meld_parts;
        Ok(c)
    }
}
