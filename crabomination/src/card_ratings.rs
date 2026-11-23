//! Card ratings measured by play, for the sealed builder.
//!
//! The builder's own score (`CardBrief::base_score + quality`) is a
//! hand-written proxy that sees bodies and keywords and gives non-creature
//! spells nothing (removal ranks with filler); on FRA it rank-correlated with
//! Draftsim at 0.06 overall, −0.46 on commons (2026-10-23).
//!
//! These are each card's ridge value from `fra_card_eval` (16,000 v3-built
//! sealed pools with `--explore 3`, 960k games, margin regressed on copy
//! differences), in win-rate points per copy, played by the `fra-fixes`
//! pilot (`dflt` misplayed some cards outright). Draftsim agreement 0.32,
//! 17lands GIH 0.29 (≈ Draftsim's own). Only nonland cards in ≥30 decks are
//! listed; others keep `base_score + quality`.
//!
//! Used under `SimConfig::rated_cards` (default off), scored by
//! [`crate::draft::score_brief_rated`]. Regenerate with `fra_card_eval` +
//! `scripts/fra_card_eval.py` (`ratings.tsv`), then
//! `scripts/fra_card_table.py ratings.tsv 30` for the table and scale.

/// Value → builder-score scale: the listed cards' static score mean/spread
/// (7.32 ± 3.30) matched to their values' (0.89 ± 2.28), so rated and unrated
/// cards share one scale; an off-colour pip (−10,
/// [`crate::draft::score_brief_rated`]) costs about four rating points.
const SCALE: f32 = 1.448;
const OFFSET: f32 = 6.034;

/// `(card name, win-rate points per copy)`: the seed-7 method under
/// `fra-fixes`, 16,000 pools, `--explore 3` (2026-10-25, 960k games).
/// Exploration (three spells a deck swapped for left-out on-colour cards)
/// lifts cards the v3 builder seldom plays over the 30-deck cut (Guiding
/// Hydra +6.49, Way of the Wildspeaker +4.45). 260 listed, 12 under the cut.
/// vs the v3 builder: 66.55 / 65.58 % (`--ab`, seeds 43 / 97).
///
/// **Re-rated 2026-11-17** under `fra-fixes` + `counter_bar_low` (ML_NOTES
/// "the bot's blue cards"; Countersculpt −1.89 → −0.97); gate vs the previous
/// table (`--ab --table`) 51.51 / 51.32 best-of-16.
// Measured values, not constants: 3.14 is a card's rating, not pi.
#[allow(clippy::approx_constant)]
pub const FRA_SIM_VALUE: &[(&str, f32)] = &[
    ("Academic Ascent", 1.90),
    ("Aerid Konstrari", 5.79),
    ("Afterthought Sentry", -0.25),
    ("Ajani Resolute", 1.81),
    ("Ajani Unrelenting", 7.00),
    ("Ajani's Anguish", 3.21),
    ("Apex Witchstalker", 2.22),
    ("Arcane Amphisbaena", 1.79),
    ("Archive Arbiter", 2.99),
    ("Arni, Humble Scribe", -2.09),
    ("Arni, Renowned Champion", 1.28),
    ("Artifist Acumen", 0.50),
    ("Avatar of Burgeoning Echoes", 0.87),
    ("Awaken the Inferno", 3.14),
    ("Bestial Incursion", 4.73),
    ("Blazing Crescendo", 0.44),
    ("Blessed Ghoul", 0.87),
    ("Bloodline Recollector", -0.67),
    ("Bloombrute", 0.94),
    ("Blossom-Blessed Angel", 4.50),
    ("Break Under Pressure", 3.51),
    ("Budding Insurgent", 0.34),
    ("Campus Crier", 0.73),
    ("Carnivorous Cultivator", 3.45),
    ("Cast Away Doubt", 0.24),
    ("Chandra's Emberling", 0.05),
    ("Chandra, Chill of Compliance", -1.50),
    ("Chandra, Torch of Defiance", 1.23),
    ("Charge the Sanctum", 0.08),
    ("Clash of Elements", 1.57),
    ("Codie, Ravenous Codex", -1.08),
    ("Command the Stage", -0.26),
    ("Compel Brutality", 0.12),
    ("Countersculpt", -0.97),
    ("Craftwork Crusher", 2.49),
    ("Craterclaw Colossus", 2.09),
    ("Cruel Calculations", -4.22),
    ("Cryotheory Adept", -0.07),
    ("Curse-Marred Demon", 3.12),
    ("Danitha, Spear of Agony", 0.31),
    ("Danitha, Sword of Hope", 0.65),
    ("Dark Matter Manipulator", 0.37),
    ("Darklight Phoenix", 2.50),
    ("Denzilore Fatehold", 5.10),
    ("Desperate Futurescribe", 3.63),
    ("Diviner of Victory", 0.30),
    ("Divining Duelist", -0.82),
    ("Draconic Visitor", 5.32),
    ("Eardrum Rattler", -0.98),
    ("Edgar, Ancient Bloodlord", -2.33),
    ("Edgar, Moonlit Sovereign", 0.01),
    ("Emergency Phytomedic", 0.12),
    ("Emrakul, the Exigent Doom", -1.23),
    ("Enlightened Confidant", 1.30),
    ("Entrust the Spark", -4.78),
    ("Essence Burn", 0.96),
    ("Extended Absence", 3.32),
    ("Extrapolate the Impossible", -4.33),
    ("Eye of Jace", 1.46),
    ("Face Yourself", 1.31),
    ("Fatehold Charm", -1.46),
    ("Fatehold Chronologist", 0.83),
    ("Fateshaper Aspirant", 1.42),
    ("Fblthp, Impossibly Lost", -0.16),
    ("Fblthp, Knows the Way", 0.50),
    ("Ferocity of the Hunt", -0.13),
    ("Flickering Hound", -2.00),
    ("Flourishing Grapple", 0.55),
    ("Frostbite Pyromental", -1.09),
    ("Fulminous Forte", 2.36),
    ("Gallia, Tragic Host", 2.12),
    ("Gallia, the Merrymaker", -0.22),
    ("Gardenize", -4.22),
    ("Garruk, Curse Breaker", 2.28),
    ("Garruk, Veiled Butcher", 5.97),
    ("Geist of Saint Thalia", -0.48),
    ("Generous Revival", 3.79),
    ("Germinate Recruits", -3.73),
    ("Ghalta the Immovable", 6.80),
    ("Ghalta the Unstoppable", 4.22),
    ("Gideon the Oathless", 2.82),
    ("Gideon's Memorial", -8.60),
    ("Graft Surgeon", 1.71),
    ("Greenhouse Propagator", 0.97),
    ("Grim Repriser", -0.73),
    ("Guiding Hydra", 6.09),
    ("Hallway Heckler", -1.06),
    ("Hapatra, the Desert Fang", 2.44),
    ("Hapatra, the Desert Frost", 1.69),
    ("Heartstring Puller", 1.47),
    ("Heartwood Crafter", -2.38),
    ("Hexhaven Battalion", 3.04),
    ("Hexhaven Invigorator", 3.77),
    ("Hungering Puppetbeast", 2.33),
    ("Hunter's Axe", -2.82),
    ("Icy Reception", -0.68),
    ("Identity Echo", -3.19),
    ("Infinite Coursework", 2.33),
    ("Ingris Stingerquill", 4.43),
    ("Inspired Tethermage", 0.17),
    ("Jace's Machinations", 1.49),
    ("Jace, Reality Sculptor", 2.52),
    ("Jiang Yanggu, Alone", 2.35),
    ("Jiang Yanggu, Never Alone", 2.95),
    ("Karn, Argent Defender", -1.03),
    ("Karn, Gilded Guardian", -2.16),
    ("Keeper of the Quiet Hour", 0.99),
    ("Kindred Judgment", 2.54),
    ("Kiora of Fire and Ashes", 6.58),
    ("Kiora of Salt and Sand", -1.77),
    ("Konstrari Charm", 0.12),
    ("Konstrari Improviser", -0.73),
    ("Koth of the Homestead", 1.29),
    ("Koth, the Geomancer", 0.99),
    ("Kwia Vigorbloom", 7.14),
    ("Last Gasp", -0.58),
    ("Lich's Relic", 0.29),
    ("Liliana the Faultless", 0.34),
    ("Liliana the Repentant", 1.14),
    ("Living Library", -0.39),
    ("Loot, the Anomaly", -1.89),
    ("Loot, the Nexus", -0.35),
    ("Loyal Tutor", -3.47),
    ("Lyra, Archangel of Dawn", 4.91),
    ("Lyra, Tolarian Archangel", 3.92),
    ("Mabel, Bitter Recluse", 1.41),
    ("Mabel, Valley Hero", 0.34),
    ("Marwyn, the Clearcutter", -0.63),
    ("Marwyn, the Preserver", 0.47),
    ("Massacre Girl, Most Wanted", 2.80),
    ("Master of Barbs", 0.73),
    ("Medic's Kitesail", -3.07),
    ("Memory Trap", 2.26),
    ("Mind Meanderer", 2.77),
    ("Mindseeker Oculus", 1.63),
    ("Multiply by Zero", 1.21),
    ("Murmuring Volume", -1.34),
    ("No Admittance", 1.32),
    ("Null Summoner", 0.96),
    ("Omnipresence", -4.34),
    ("Overwrite the Multiverse", 3.44),
    ("Paradox Shaper", -2.52),
    ("Perfected Theory", 1.27),
    ("Pia, Aether Ascetic", -4.13),
    ("Pia, Determined Rebuilder", 2.15),
    ("Plan for All Outcomes", 2.60),
    ("Pompous Battlemage", -2.29),
    ("Precise Redaction", -1.14),
    ("Predictive Preparations", 1.68),
    ("Primal Witchstalker", 0.27),
    ("Proctor of Potential", 0.77),
    ("Proft, Consulting Detective", 0.45),
    ("Proft, Sinister Mastermind", -0.83),
    ("Prophesied End", -0.38),
    ("Protege's Awakening", 0.75),
    ("Prudent Fateseer", -0.37),
    ("Puppet Crafting", -2.47),
    ("Pyre Rhymer", 0.00),
    ("Rampart Hunter", 0.61),
    ("Rank Rat", 0.82),
    ("Recursive Recruitment", 5.25),
    ("Refute Destiny", 1.68),
    ("Repurposed Enforcer", 2.22),
    ("Rescue Girl, First Responder", -1.51),
    ("Restore with Empathy", 0.81),
    ("Return to the Light Realms", 0.45),
    ("Rewrite Regrets", 2.73),
    ("Rise of the Deathbringer", -0.57),
    ("Ruric Thar, Biomagus", 4.05),
    ("Ruric Thar, Magecrusher", 2.73),
    ("Saheeli, Consul of Oversight", 3.77),
    ("Saheeli, Jewel of Avishkar", 0.08),
    ("Samut, Hazoret's Champion", 1.04),
    ("Samut, Tyrant of Naktamun", -1.01),
    ("Sanctum Lurker", 4.19),
    ("Screeching Soulbreaker", 2.90),
    ("Seasoned Cryomancer", -0.47),
    ("Semester Foreseer", 1.18),
    ("Shatterwing Pegasus", 0.99),
    ("Silence the Echo", -1.09),
    ("Simulacrum Shaper", 1.60),
    ("Skilled Battlecarver", -1.19),
    ("Solarium Sentry", 0.81),
    ("Solitary Cell", 0.58),
    ("Solve for Disappointment", -1.05),
    ("Something Worth Saving", 0.89),
    ("Sphinx of False Conclusions", 5.97),
    ("Sphinx's Approach", -0.34),
    ("Stingcaster Mage", 0.35),
    ("Stingerquill Charm", 0.85),
    ("Stingerquill Voxmancer", 0.51),
    ("Stinging Vitriol", -0.12),
    ("Sureshot Sower", -0.04),
    ("Surgical Precision", 3.25),
    ("Surveillance Phantasm", 1.52),
    ("Tam's Resistance", 1.56),
    ("Tam, the Possibility", -1.93),
    ("Tarmogoyf", 2.95),
    ("Tenured Tethermage", -0.07),
    ("Terminal Criticism", 1.47),
    ("Tether Technician", 1.83),
    ("Tethermage's Advantage", 2.45),
    ("Tetsuko Umezawa, Fugitive", 2.11),
    ("Tetsuko Umezawa, Pursuer", 4.36),
    ("Teyo, Diamondblade Mage", -0.77),
    ("Teyo, Lightshield Expert", 0.23),
    ("Thalia, the Survivor", 3.24),
    ("The Echoverse Fulcrum", -3.35),
    ("The Theorist, Jace Beleren", 3.73),
    ("Theoretical Necromancer", 1.53),
    ("Theorist's Proxy", -0.97),
    ("Theorix Charm", -2.78),
    ("Theorix Metamage", 0.09),
    ("Tinybones, Pocket Nuisance", 2.12),
    ("Titanbones, Towering Heart", 2.69),
    ("Tomik, Izzet Sparkmage", -1.15),
    ("Tomik, Orzhov Lawmage", 0.49),
    ("Traxos, Academy Guardian", 2.38),
    ("Traxos, Scourge Eternal", 3.06),
    ("Twinned Vision", -0.22),
    ("Twisted Fates", 2.97),
    ("Uldaros Theorix", 1.12),
    ("Undulating Witness", 2.90),
    ("Unflinching Hortimancer", -0.44),
    ("Unsummon", -0.63),
    ("Variable Chaser", 2.38),
    ("Verdant Kraken", 4.41),
    ("Vigorbloom Charm", 0.24),
    ("Vigorbloom Vanguard", 1.25),
    ("Vindictive Triumph", -3.90),
    ("Vinelasher Adept", 1.35),
    ("Violent Echoes", 3.15),
    ("Void Extrapolator", 0.07),
    ("Vraska's Final Mercy", 0.88),
    ("Vraska, Soul of Stone", -2.20),
    ("Vraska, the Cutting Glare", 4.04),
    ("Warrior's Blades", -0.15),
    ("Way of the Cryomancer", -0.04),
    ("Way of the Deathbringer", 2.15),
    ("Way of the Healer", 1.98),
    ("Way of the Mentor", 0.57),
    ("Way of the Mind Sculptor", 1.17),
    ("Way of the Necromancer", -1.52),
    ("Way of the Paradox", 2.33),
    ("Way of the Pyromancer", -0.37),
    ("Way of the Warlord", 0.53),
    ("Way of the Wildspeaker", 4.12),
    ("Whiplash Wordsmith", -0.20),
    ("Winter, Team Player", -0.46),
    ("Winter, Tormented Loner", 1.08),
    ("Woodwork Prodigy", -0.24),
    ("Wrath of the Bloodmane", 0.98),
    ("Wrecking Gecko", 2.01),
    ("Yargle, Glutton of Urborg", 0.65),
    ("Yargle, Goliath of Otaria", 0.55),
    ("Yoshimaru, Beloved Companion", -0.16),
    ("Yoshimaru, Scrappy Stray", -1.67),
    ("Your Fate Ends Here", 3.34),
    ("Yuriko, Blade of the Mighty", 0.83),
    ("Yuriko, Hope from the Shadows", -1.90),
];

/// [`SCALE`] / [`OFFSET`] for [`SOS_SIM_VALUE`]: its 151 cards' static
/// scores (6.91 ± 3.18) matched to their values (−0.31 ± 1.61).
const SOS_SCALE: f32 = 1.975;
const SOS_OFFSET: f32 = 7.524;

/// `(card name, win-rate points per copy)` for Secrets of Strixhaven, by the
/// [`FRA_SIM_VALUE`] method (2026-10-26): `fra_card_eval --set sos --pilot
/// dflt --decks 16000 --rounds 30 --pairs 2 --seed 7 --explore 3` (`dflt`:
/// the SOS client's weights), then `scripts/fra_card_eval.py RUN` and
/// `scripts/fra_card_table.py ratings.tsv 30`. 151 of 152 nonland cards.
/// Ridge λ 300 by 5-fold CV (FRA's 30) — SOS values are flatter.
#[allow(clippy::approx_constant)]
pub const SOS_SIM_VALUE: &[(&str, f32)] = &[
    ("Additive Evolution", 2.10),
    ("Adrix and Nev, Twincasters", 0.17),
    ("Ambitious Augmenter", 2.00),
    ("Applied Geometry", 2.28),
    ("Arcane Omens", -1.35),
    ("Archaeomancer", -0.65),
    ("Archaic's Agony", -1.80),
    ("Archmage Emeritus", -0.64),
    ("Ark of Hunger", -3.53),
    ("Arnyn, Deathbloom Botanist", 0.88),
    ("Artistic Process", 1.32),
    ("Ascendant Dustspeaker", 0.71),
    ("Aziza, Mage Tower Captain", -2.88),
    ("Banishing Betrayal", -0.32),
    ("Berta, Wise Extrapolator", -2.07),
    ("Blech, Loafing Pest", 0.10),
    ("Bogwater Lumaret", -0.04),
    ("Burrog Banemaker", 0.45),
    ("Burrog Barrage", 0.42),
    ("Campus Composer", -0.03),
    ("Cauldron of Essence", 0.14),
    ("Charging Strifeknight", 0.88),
    ("Chase Inspiration", -2.16),
    ("Chelonian Tackle", -1.03),
    ("Choreographed Sparks", -4.68),
    ("Codie, Vociferous Codex", -7.25),
    ("Colorstorm Stallion", 2.07),
    ("Daydream", 0.13),
    ("Decorum Dissertation", -1.21),
    ("Deluge Virtuoso", 0.41),
    ("Divergent Equation", -1.88),
    ("Dualcaster Mage", -2.34),
    ("Duel Tactics", -0.23),
    ("Eager Glyphmage", 1.16),
    ("Echocasting Symposium", -1.31),
    ("Efflorescence", -1.21),
    ("Elemental Mascot", -0.55),
    ("Emeritus of Ideation", 3.38),
    ("Ennis, Debate Moderator", -1.64),
    ("Environmental Scientist", 2.56),
    ("Essenceknit Scholar", 0.17),
    ("Fix What's Broken", -2.71),
    ("Flow State", -0.66),
    ("Follow the Lumarets", -0.91),
    ("Foolish Fate", 2.22),
    ("Forum Necroscribe", -0.43),
    ("Fractal Anomaly", -1.12),
    ("Fractal Mascot", 0.36),
    ("Fractal Tender", -0.14),
    ("Fractalize", -1.57),
    ("Garrison Excavator", 0.81),
    ("Geometer's Arthropod", -2.03),
    ("Germination Practicum", 2.57),
    ("Glorious Decay", -0.22),
    ("Graduation Day", -2.04),
    ("Grapple with Death", 0.98),
    ("Grave Researcher", 0.74),
    ("Grim Haruspex", 0.43),
    ("Growth Curve", -1.07),
    ("Hardened Academic", -0.57),
    ("Homesickness", -1.04),
    ("Imperious Inkmage", -0.67),
    ("Improvisation Capstone", -0.03),
    ("Informed Inkwright", 0.34),
    ("Inkling Mascot", -1.97),
    ("Inkshape Demonstrator", -0.00),
    ("Interjection", -0.22),
    ("Lecturing Scornmage", -0.79),
    ("Library of Leng", -2.74),
    ("Living History", 0.63),
    ("Lluwen, Exchange Student", -0.70),
    ("Lorehold Charm", -3.10),
    ("Lorehold, the Historian", 0.80),
    ("Lumaret's Favor", 0.17),
    ("Magmablood Archaic", 0.53),
    ("Magus of the Library", -2.00),
    ("Mana Sculpt", -1.78),
    ("Masterful Flourish", 0.04),
    ("Melancholic Poet", -0.33),
    ("Mica, Reader of Ruins", 0.35),
    ("Mindful Biomancer", 0.56),
    ("Molten Note", 1.40),
    ("Moseo, Vein's New Dean", 1.70),
    ("Murmuring Mystic", -2.41),
    ("Muse's Encouragement", 0.34),
    ("Nita, Forum Conciliator", -2.46),
    ("Noxious Newt", 0.88),
    ("Old-Growth Educator", 0.30),
    ("Oracle's Restoration", 0.36),
    ("Paradox Surveyor", 0.25),
    ("Pest Mascot", -1.54),
    ("Planar Engineering", -1.14),
    ("Practiced Offense", -2.43),
    ("Practiced Scrollsmith", -1.98),
    ("Primary Research", -0.17),
    ("Prismari Charm", -2.14),
    ("Prismari, the Inspiration", -1.21),
    ("Procrastinate", -0.36),
    ("Proctor's Gaze", -0.61),
    ("Professor Dellian Fel", 2.66),
    ("Quandrix, the Proof", 1.76),
    ("Rabid Attack", -1.89),
    ("Ral Zarek, Guest Lecturer", -0.08),
    ("Rapier Wit", 1.43),
    ("Rapturous Moment", -2.12),
    ("Rearing Embermare", 1.29),
    ("Rehearsed Debater", 0.52),
    ("Resonating Lute", -4.15),
    ("Restoration Seminar", 0.02),
    ("Rubble Rouser", -2.14),
    ("Shattered Acolyte", 0.49),
    ("Shopkeeper's Bane", 1.52),
    ("Silverquill Charm", 0.15),
    ("Silverquill, the Disputant", 0.57),
    ("Slumbering Trudge", 4.24),
    ("Snarl Song", -0.16),
    ("Sneering Shadewriter", 1.30),
    ("Snooping Page", -1.45),
    ("Soaring Stoneglider", 0.59),
    ("Social Snub", -1.15),
    ("Spectacular Skywhale", -0.26),
    ("Spirit Mascot", -1.59),
    ("Splatter Technique", -0.18),
    ("Stand Up for Yourself", 2.10),
    ("Startled Relic Sloth", 0.66),
    ("Steal the Show", -1.29),
    ("Stirring Honormancer", -1.11),
    ("Stirring Hopesinger", 2.09),
    ("Stone Docent", 0.27),
    ("Strife Scholar", -0.37),
    ("Studious First-Year", 0.66),
    ("Suspend Aggression", 0.24),
    ("Tablet of Discovery", -0.85),
    ("Textbook Tabulator", -0.07),
    ("Thornfist Striker", 1.09),
    ("Tome Blast", -0.62),
    ("Top of the Class", -2.70),
    ("Topiary Lecturer", 0.76),
    ("Tragedy Feaster", 0.48),
    ("Traumatic Critique", -0.63),
    ("Unsubtle Mockery", 1.80),
    ("Vicious Rivalry", -1.16),
    ("Wander Off", 1.99),
    ("Wild Hypothesis", 0.83),
    ("Wildgrowth Archaic", 0.85),
    ("Wilt in the Heat", 0.22),
    ("Witherbloom, the Balancer", -1.38),
    ("Withering Curse", -3.90),
    ("Zaffai and the Tempests", -2.31),
    ("Zealous Lorecaster", -0.18),
    ("Zimone's Experiment", 0.57),
];

/// A re-rated table: name -> builder-score units (`None` under the deck cut).
type RatingTable = std::sync::Arc<crate::fxhash::HashMap<String, Option<i32>>>;

thread_local! {
    /// [`with_table`]'s override, `None` outside it.
    static OVERRIDE: std::cell::RefCell<Option<RatingTable>> =
        const { std::cell::RefCell::new(None) };
}

/// Run `f` with `table` (name -> builder-score units, `None` = measured but
/// under the cut) replacing the compiled tables on this thread, for racing a
/// re-rated table against the shipped one in one process (`fra_card_eval
/// --ab --table`). Names it lacks read the compiled tables.
pub type TableOverride = std::sync::Arc<crate::fxhash::HashMap<String, Option<i32>>>;
pub fn with_table<R>(table: TableOverride, f: impl FnOnce() -> R) -> R {
    OVERRIDE_ON.store(true, std::sync::atomic::Ordering::Relaxed);
    let prev = OVERRIDE.with(|o| o.borrow_mut().replace(table));
    let r = f();
    OVERRIDE.with(|o| *o.borrow_mut() = prev);
    r
}

/// Has any thread ever entered [`with_table`]? One relaxed load keeps the
/// override's thread-local off the builder's path in every process that
/// never races a table.
static OVERRIDE_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The builder's read of a card's rating: `brief.rated` (cached per card for
/// the process, from the compiled tables), or [`with_table`]'s override on
/// this thread. The cache is why the override has to be read here and not
/// only in [`rated_score`]: a brief is built once, before any race.
pub fn brief_rated(brief: &crate::cube::CardBrief) -> Option<i32> {
    if !OVERRIDE_ON.load(std::sync::atomic::Ordering::Relaxed) || brief.is_land {
        return brief.rated;
    }
    match OVERRIDE.with(|o| o.borrow().as_ref().and_then(|t| t.get(brief.def.name).copied())) {
        Some(listed) => listed,
        None => brief.rated,
    }
}

/// A card's measured value in builder-score units, when it has one.
pub fn rated_score(name: &str) -> Option<i32> {
    if !OVERRIDE_ON.load(std::sync::atomic::Ordering::Relaxed) {
        return compiled_score(name);
    }
    match OVERRIDE.with(|o| o.borrow().as_ref().and_then(|t| t.get(name).copied())) {
        Some(listed) => listed,
        None => compiled_score(name),
    }
}

fn compiled_score(name: &str) -> Option<i32> {
    static MAP: std::sync::OnceLock<crate::fxhash::HashMap<&'static str, i32>> = std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let fra = FRA_SIM_VALUE.iter().map(|&(n, v)| (n, (OFFSET + SCALE * v).round() as i32));
        let sos = SOS_SIM_VALUE.iter().map(|&(n, v)| (n, (SOS_OFFSET + SOS_SCALE * v).round() as i32));
        fra.chain(sos).collect()
    })
    .get(name)
    .copied()
}

/// The deck-shape term of the rated builder's best-of-N judge, a builder-
/// score penalty (≤ 0) over a deck's spells.
///
/// Additive card values price counts linearly but not curvature. On the
/// [`FRA_SIM_VALUE`] pools the per-deck residual is an inverted U in three
/// counts (`scripts/deck_shape_fit.py`, 2026-10-26): flat at 16–19
/// creatures, 8–11 spells of MV ≤ 2, 2–5 of MV ≥ 5. Modelled as `k · d²` win
/// points, `d` the distance outside the flat range (creatures k 0.2, cheap
/// 0.12, top 0.15), times [`SCALE`].
pub fn shape_penalty(spells: &[crate::cube::CardFactory]) -> i32 {
    let (mut creatures, mut cheap, mut top) = (0i32, 0i32, 0i32);
    for f in spells {
        let b = crate::cube::card_brief(*f);
        if b.is_land {
            continue;
        }
        creatures += i32::from(b.is_creature);
        cheap += i32::from(b.cmc <= 2);
        top += i32::from(b.cmc >= 5);
    }
    let out = |x: i32, lo: i32, hi: i32| (lo - x).max(x - hi).max(0) as f32;
    let pts = 0.2 * out(creatures, 16, 19).powi(2) + 0.12 * out(cheap, 8, 11).powi(2) + 0.15 * out(top, 2, 5).powi(2);
    -(SCALE * pts).round() as i32
}

/// Does the table rate this pool: are at least half its nonland cards
/// listed? The sealed opponent's builder ([`crate::selfplay::sealed_opponent_build`])
/// switches to the rated builder on such a pool and keeps v3 on any other
/// (a cube pool, a set with no table).
pub fn rates_pool(pool: &[crate::cube::CardFactory]) -> bool {
    let (mut rated, mut all) = (0usize, 0usize);
    for f in pool {
        let def = crate::cube::card_def(*f);
        if def.is_land() {
            continue;
        }
        all += 1;
        rated += usize::from(rated_score(def.name).is_some());
    }
    all > 0 && rated * 2 >= all
}

/// Is this a Secrets of Strixhaven pool: are at least half its nonland
/// cards on [`SOS_SIM_VALUE`]? The sealed opponent's learned builder
/// ([`crate::selfplay::sealed_opponent_build`]) picks its same-pool deck
/// value by set: SOS's here, FRA's otherwise.
pub fn sos_pool(pool: &[crate::cube::CardFactory]) -> bool {
    static NAMES: std::sync::OnceLock<crate::fxhash::HashSet<&'static str>> = std::sync::OnceLock::new();
    let names = NAMES.get_or_init(|| SOS_SIM_VALUE.iter().map(|&(n, _)| n).collect());
    let (mut sos, mut all) = (0usize, 0usize);
    for f in pool {
        let def = crate::cube::card_def(*f);
        if def.is_land() {
            continue;
        }
        all += 1;
        sos += usize::from(names.contains(def.name));
    }
    all > 0 && sos * 2 >= all
}

#[cfg(test)]
mod tests {
    use crate::draft::{ColorCounts, colors_of_picks, score_brief_rated};

    /// A listed card scores its rating, unlisted its static score; both pay
    /// 10 per off-colour pip. Memory Trap is why the table exists: removal
    /// the static score ranks as filler, measured ~+3 a copy.
    #[test]
    fn rated_cards_score_their_rating_and_pay_only_for_off_colour_pips() {
        let trap = crate::cube::card_brief(crate::catalog::memory_trap);
        let colors: ColorCounts = colors_of_picks(&[crate::catalog::memory_trap]);
        let r = super::rated_score("Memory Trap").expect("Memory Trap is rated");
        assert_eq!(trap.rated, Some(r));
        assert_eq!(score_brief_rated(trap, &colors), r, "on colour: the rating alone");
        let off: ColorCounts = colors_of_picks(&[crate::catalog::grizzly_bears]);
        assert_eq!(score_brief_rated(trap, &off), r - 10 * trap.pips_total as i32, "off colour: 10 a pip");
        assert!(r > trap.base_score + trap.quality, "measured above its static score");
        let bears = crate::cube::card_brief(crate::catalog::grizzly_bears);
        assert_eq!(bears.rated, None);
        assert_eq!(score_brief_rated(bears, &off), bears.base_score + bears.quality, "unrated, on colour");
    }

    /// Tam's Resistance is `{1}{G/U}`: a green deck casts it with no blue,
    /// so the rated builder lets it into G/W and charges it no off-colour
    /// pip there. Counting the hybrid as both colours kept a +2 card out of
    /// every G/x deck but G/U.
    #[test]
    fn a_hybrid_card_fits_a_deck_holding_either_colour() {
        use crate::mana::{Color, ColorSet};
        let tams = crate::cube::card_brief(crate::catalog::tams_resistance);
        let mut gw = ColorSet::single(Color::Green);
        gw.insert(Color::White);
        let mut wr = ColorSet::single(Color::White);
        wr.insert(Color::Red);
        assert!(!tams.pip_colors.is_subset_of(gw), "the pip count reads both halves");
        assert!(tams.fits(gw));
        assert!(!tams.fits(wr));
        assert_eq!(tams.off_pips(gw), 0);
        assert_eq!(tams.off_pips(wr), 1, "one symbol, one pip");
        let r = super::rated_score("Tam's Resistance").expect("rated");
        let mut seat = ColorCounts::default();
        seat.add(Color::Green, 1);
        seat.add(Color::White, 1);
        assert_eq!(score_brief_rated(tams, &seat), r);
    }
}
