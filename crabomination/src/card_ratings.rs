//! Card ratings measured by play, for the sealed builder.
//!
//! The builder's own card score (`CardBrief::base_score + quality`) is
//! a hand-written proxy: it sees a creature's body and keywords and gives
//! every non-creature spell nothing, so removal ranks with vanilla filler
//! and a colourless artifact creature (+5 for fitting any deck) outranks
//! both. On Reality Fracture it rank-correlates with Draftsim's ratings at
//! 0.06 overall and **−0.46 on commons** (`fra_card_eval`, 2026-10-23).
//!
//! These are the measured alternative: each card's ridge value from
//! `fra_card_eval` — 16,000 FRA sealed pools built by the v3 builder (three
//! spells a deck swapped for left-out pool cards, `--explore 3`), 960k
//! games, every match's margin regressed on (copies in deck A − copies in
//! deck B) — in win-rate points per copy. Played by the `fra-fixes` pilot
//! since 2026-10-25 (the `dflt` pilot misplayed some cards outright: Icy
//! Reception never countered, Living Library attacked as a 0/4, burn missed
//! lethal): Draftsim agreement 0.32 overall (0.67 on rares; `dflt` 0.28 /
//! 0.57; this table 0.29 overall); 17lands GIH agreement 0.29,
//! about Draftsim's own 0.30. Only nonland cards in at least 30 decks are
//! listed; anything else scores its static `base_score + quality`.
//!
//! Used only under `SimConfig::rated_cards` (default off), whose scorer is
//! [`crate::draft::score_brief_rated`]. Regenerate with `fra_card_eval` +
//! `scripts/fra_card_eval.py` (its `ratings.tsv`), then
//! `scripts/fra_card_table.py ratings.tsv 30` for this table and the scale.

/// The value → builder-score scale: the mean and spread of the listed cards'
/// static scores (7.32 ± 3.30) matched to their values' (1.10 ± 2.37), so a
/// rated card and an unrated one (scored `base_score + quality`) share one
/// scale, and an off-colour pip (−10, [`crate::draft::score_brief_rated`])
/// costs about four rating points.
const SCALE: f32 = 1.391;
const OFFSET: f32 = 5.784;

/// `(card name, win-rate points per copy)`, from the seed-7 method under the
/// `fra-fixes` pilot on 16,000 pools with `--explore 3` (2026-10-25; 30
/// rounds x 2 pairs, 960k games). The v3 builder seldom plays some cards,
/// so the 4,000-pool run left them under the 30-deck cut on a static score
/// that misses them: Guiding Hydra (25 decks there, static 4; 170 here,
/// +6.49) and Way of the Wildspeaker (17 decks even on 16,000 pools without
/// exploration; 241 here, +4.45). Exploration swaps three of each deck's
/// spells for left-out on-colour pool cards. 260 cards listed, 12 under the
/// cut. Against the v3 builder this table's builds win 66.55 / 65.58 %
/// (`--ab`, 2,000 pools x 16, seeds 43 / 97), the unexplored 16,000-pool
/// table's 66.19 / 66.02 %, the 4,000-pool table's (under the old scorer)
/// 63.34 / 62.57 %.
// Measured values, not constants: 3.14 is a card's rating, not pi.
#[allow(clippy::approx_constant)]
pub const FRA_SIM_VALUE: &[(&str, f32)] = &[
    ("Academic Ascent", 2.38),
    ("Aerid Konstrari", 5.98),
    ("Afterthought Sentry", 0.13),
    ("Ajani Resolute", 2.77),
    ("Ajani Unrelenting", 7.53),
    ("Ajani's Anguish", 2.98),
    ("Apex Witchstalker", 2.67),
    ("Arcane Amphisbaena", 1.84),
    ("Archive Arbiter", 3.76),
    ("Arni, Humble Scribe", -2.05),
    ("Arni, Renowned Champion", 1.57),
    ("Artifist Acumen", 0.47),
    ("Avatar of Burgeoning Echoes", 0.86),
    ("Awaken the Inferno", 3.72),
    ("Bestial Incursion", 5.69),
    ("Blazing Crescendo", 1.20),
    ("Blessed Ghoul", 0.96),
    ("Bloodline Recollector", -0.48),
    ("Bloombrute", 1.25),
    ("Blossom-Blessed Angel", 4.93),
    ("Break Under Pressure", 3.51),
    ("Budding Insurgent", 0.85),
    ("Campus Crier", 0.89),
    ("Carnivorous Cultivator", 3.00),
    ("Cast Away Doubt", 0.60),
    ("Chandra's Emberling", 0.35),
    ("Chandra, Chill of Compliance", -2.02),
    ("Chandra, Torch of Defiance", 2.06),
    ("Charge the Sanctum", 0.87),
    ("Clash of Elements", 2.01),
    ("Codie, Ravenous Codex", -0.64),
    ("Command the Stage", -0.03),
    ("Compel Brutality", 0.60),
    ("Countersculpt", -1.65),
    ("Craftwork Crusher", 2.23),
    ("Craterclaw Colossus", 2.43),
    ("Cruel Calculations", -3.16),
    ("Cryotheory Adept", 0.09),
    ("Curse-Marred Demon", 3.29),
    ("Danitha, Spear of Agony", 0.46),
    ("Danitha, Sword of Hope", 0.99),
    ("Dark Matter Manipulator", 0.72),
    ("Darklight Phoenix", 2.72),
    ("Denzilore Fatehold", 5.42),
    ("Desperate Futurescribe", 3.59),
    ("Diviner of Victory", 0.41),
    ("Divining Duelist", -0.41),
    ("Draconic Visitor", 5.70),
    ("Eardrum Rattler", -0.58),
    ("Edgar, Ancient Bloodlord", -1.73),
    ("Edgar, Moonlit Sovereign", 0.84),
    ("Emergency Phytomedic", 0.50),
    ("Emrakul, the Exigent Doom", -0.46),
    ("Enlightened Confidant", 1.35),
    ("Entrust the Spark", -4.79),
    ("Essence Burn", 1.38),
    ("Extended Absence", 3.89),
    ("Extrapolate the Impossible", -3.99),
    ("Eye of Jace", 1.65),
    ("Face Yourself", 1.93),
    ("Fatehold Charm", -1.15),
    ("Fatehold Chronologist", 1.32),
    ("Fateshaper Aspirant", -0.26),
    ("Fblthp, Impossibly Lost", 0.25),
    ("Fblthp, Knows the Way", 0.24),
    ("Ferocity of the Hunt", -0.10),
    ("Flickering Hound", -1.75),
    ("Flourishing Grapple", 0.93),
    ("Frostbite Pyromental", -1.36),
    ("Fulminous Forte", 2.36),
    ("Gallia, Tragic Host", 2.56),
    ("Gallia, the Merrymaker", -0.08),
    ("Gardenize", -4.48),
    ("Garruk, Curse Breaker", 3.19),
    ("Garruk, Veiled Butcher", 6.67),
    ("Geist of Saint Thalia", -0.09),
    ("Generous Revival", 4.21),
    ("Germinate Recruits", -4.09),
    ("Ghalta the Immovable", 7.09),
    ("Ghalta the Unstoppable", 4.98),
    ("Gideon the Oathless", 3.47),
    ("Gideon's Memorial", -8.39),
    ("Graft Surgeon", 2.08),
    ("Greenhouse Propagator", 1.21),
    ("Grim Repriser", -0.64),
    ("Guiding Hydra", 6.49),
    ("Hallway Heckler", -0.78),
    ("Hapatra, the Desert Fang", 2.40),
    ("Hapatra, the Desert Frost", 2.18),
    ("Heartstring Puller", 1.83),
    ("Heartwood Crafter", -1.78),
    ("Hexhaven Battalion", 3.10),
    ("Hexhaven Invigorator", 3.71),
    ("Hungering Puppetbeast", 3.18),
    ("Hunter's Axe", -2.75),
    ("Icy Reception", -0.85),
    ("Identity Echo", -2.77),
    ("Infinite Coursework", 2.13),
    ("Ingris Stingerquill", 4.56),
    ("Inspired Tethermage", 0.86),
    ("Jace's Machinations", 1.44),
    ("Jace, Reality Sculptor", 2.58),
    ("Jiang Yanggu, Alone", 2.81),
    ("Jiang Yanggu, Never Alone", 3.26),
    ("Karn, Argent Defender", -0.85),
    ("Karn, Gilded Guardian", -1.61),
    ("Keeper of the Quiet Hour", 1.25),
    ("Kindred Judgment", 3.62),
    ("Kiora of Fire and Ashes", 7.37),
    ("Kiora of Salt and Sand", -1.78),
    ("Konstrari Charm", 0.21),
    ("Konstrari Improviser", -0.55),
    ("Koth of the Homestead", -0.99),
    ("Koth, the Geomancer", 1.51),
    ("Kwia Vigorbloom", 7.20),
    ("Last Gasp", -0.11),
    ("Lich's Relic", 0.36),
    ("Liliana the Faultless", 1.15),
    ("Liliana the Repentant", 0.96),
    ("Living Library", -0.01),
    ("Loot, the Anomaly", -2.44),
    ("Loot, the Nexus", 0.30),
    ("Loyal Tutor", -3.40),
    ("Lyra, Archangel of Dawn", 5.56),
    ("Lyra, Tolarian Archangel", 4.14),
    ("Mabel, Bitter Recluse", 1.73),
    ("Mabel, Valley Hero", 0.65),
    ("Marwyn, the Clearcutter", -0.29),
    ("Marwyn, the Preserver", 0.77),
    ("Massacre Girl, Most Wanted", 3.71),
    ("Master of Barbs", 0.74),
    ("Medic's Kitesail", -2.58),
    ("Memory Trap", 2.50),
    ("Mind Meanderer", 2.82),
    ("Mindseeker Oculus", 1.67),
    ("Multiply by Zero", 1.82),
    ("Murmuring Volume", -0.90),
    ("No Admittance", 1.49),
    ("Null Summoner", 0.86),
    ("Omnipresence", -3.44),
    ("Overwrite the Multiverse", 3.33),
    ("Paradox Shaper", -2.74),
    ("Perfected Theory", 2.02),
    ("Pia, Aether Ascetic", -3.29),
    ("Pia, Determined Rebuilder", 1.96),
    ("Plan for All Outcomes", 2.90),
    ("Pompous Battlemage", -1.98),
    ("Precise Redaction", -1.94),
    ("Predictive Preparations", 2.52),
    ("Primal Witchstalker", 0.34),
    ("Proctor of Potential", 1.24),
    ("Proft, Consulting Detective", 0.93),
    ("Proft, Sinister Mastermind", -0.74),
    ("Prophesied End", -0.05),
    ("Protege's Awakening", 0.98),
    ("Prudent Fateseer", -0.25),
    ("Puppet Crafting", -2.84),
    ("Pyre Rhymer", 0.08),
    ("Rampart Hunter", 0.69),
    ("Rank Rat", 0.85),
    ("Recursive Recruitment", 6.27),
    ("Refute Destiny", 2.08),
    ("Repurposed Enforcer", 2.49),
    ("Rescue Girl, First Responder", -0.99),
    ("Restore with Empathy", 0.66),
    ("Return to the Light Realms", 0.80),
    ("Rewrite Regrets", 3.07),
    ("Rise of the Deathbringer", -0.95),
    ("Ruric Thar, Biomagus", 4.58),
    ("Ruric Thar, Magecrusher", 3.31),
    ("Saheeli, Consul of Oversight", 4.42),
    ("Saheeli, Jewel of Avishkar", 0.21),
    ("Samut, Hazoret's Champion", 0.70),
    ("Samut, Tyrant of Naktamun", -0.84),
    ("Sanctum Lurker", 4.51),
    ("Screeching Soulbreaker", 3.15),
    ("Seasoned Cryomancer", -0.30),
    ("Semester Foreseer", 1.59),
    ("Shatterwing Pegasus", 1.33),
    ("Silence the Echo", -0.76),
    ("Simulacrum Shaper", 1.58),
    ("Skilled Battlecarver", -1.16),
    ("Solarium Sentry", 1.04),
    ("Solitary Cell", 0.38),
    ("Solve for Disappointment", -0.74),
    ("Something Worth Saving", 1.08),
    ("Sphinx of False Conclusions", 1.64),
    ("Sphinx's Approach", -0.19),
    ("Stingcaster Mage", 0.78),
    ("Stingerquill Charm", 0.55),
    ("Stingerquill Voxmancer", 0.86),
    ("Stinging Vitriol", -0.09),
    ("Sureshot Sower", 0.17),
    ("Surgical Precision", 3.32),
    ("Surveillance Phantasm", 1.77),
    ("Tam's Resistance", 1.59),
    ("Tam, the Possibility", -1.76),
    ("Tarmogoyf", 2.89),
    ("Tenured Tethermage", 0.18),
    ("Terminal Criticism", 2.15),
    ("Tether Technician", 1.68),
    ("Tethermage's Advantage", 2.74),
    ("Tetsuko Umezawa, Fugitive", 2.82),
    ("Tetsuko Umezawa, Pursuer", 4.89),
    ("Teyo, Diamondblade Mage", -1.57),
    ("Teyo, Lightshield Expert", -1.46),
    ("Thalia, the Survivor", 3.67),
    ("The Echoverse Fulcrum", -2.87),
    ("The Theorist, Jace Beleren", 3.74),
    ("Theoretical Necromancer", 1.83),
    ("Theorist's Proxy", -0.89),
    ("Theorix Charm", -2.48),
    ("Theorix Metamage", 0.36),
    ("Tinybones, Pocket Nuisance", 2.16),
    ("Titanbones, Towering Heart", 3.41),
    ("Tomik, Izzet Sparkmage", -0.79),
    ("Tomik, Orzhov Lawmage", 1.00),
    ("Traxos, Academy Guardian", 2.70),
    ("Traxos, Scourge Eternal", 3.95),
    ("Twinned Vision", 0.16),
    ("Twisted Fates", 3.15),
    ("Uldaros Theorix", 1.17),
    ("Undulating Witness", 3.17),
    ("Unflinching Hortimancer", -0.14),
    ("Unsummon", -0.51),
    ("Variable Chaser", 2.57),
    ("Verdant Kraken", 5.44),
    ("Vigorbloom Charm", 0.34),
    ("Vigorbloom Vanguard", 1.71),
    ("Vindictive Triumph", -4.07),
    ("Vinelasher Adept", -2.86),
    ("Violent Echoes", 3.44),
    ("Void Extrapolator", 0.29),
    ("Vraska's Final Mercy", 1.09),
    ("Vraska, Soul of Stone", -1.76),
    ("Vraska, the Cutting Glare", 4.29),
    ("Warrior's Blades", -0.22),
    ("Way of the Cryomancer", 0.39),
    ("Way of the Deathbringer", 2.71),
    ("Way of the Healer", 2.46),
    ("Way of the Mentor", 0.46),
    ("Way of the Mind Sculptor", 1.49),
    ("Way of the Necromancer", -1.68),
    ("Way of the Paradox", 2.78),
    ("Way of the Pyromancer", -0.22),
    ("Way of the Warlord", 1.21),
    ("Way of the Wildspeaker", 4.45),
    ("Whiplash Wordsmith", -0.06),
    ("Winter, Team Player", 0.05),
    ("Winter, Tormented Loner", 1.36),
    ("Woodwork Prodigy", 0.44),
    ("Wrath of the Bloodmane", 1.53),
    ("Wrecking Gecko", 2.60),
    ("Yargle, Glutton of Urborg", 1.05),
    ("Yargle, Goliath of Otaria", 1.30),
    ("Yoshimaru, Beloved Companion", -0.65),
    ("Yoshimaru, Scrappy Stray", -2.95),
    ("Your Fate Ends Here", 3.74),
    ("Yuriko, Blade of the Mighty", 2.27),
    ("Yuriko, Hope from the Shadows", -1.62),
];

/// [`SCALE`] / [`OFFSET`] for [`SOS_SIM_VALUE`]: its 151 cards' static
/// scores (6.91 ± 3.18) matched to their values (−0.31 ± 1.61).
const SOS_SCALE: f32 = 1.975;
const SOS_OFFSET: f32 = 7.524;

/// `(card name, win-rate points per copy)` for Secrets of Strixhaven, by
/// the same method as [`FRA_SIM_VALUE`] (2026-10-26): `fra_card_eval --set
/// sos --pilot dflt --decks 16000 --rounds 30 --pairs 2 --seed 7 --explore 3`
/// (960k games; `dflt` because the SOS client plays default weights), then
/// `scripts/fra_card_eval.py RUN` and `scripts/fra_card_table.py ratings.tsv
/// 30`. 151 of the sheet's 152 nonland cards (Sylvan Library made 26 decks).
/// Ridge λ 300 by 5-fold CV (FRA's: 30) — the SOS values are flatter.
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

thread_local! {
    /// [`with_table`]'s override, `None` outside it.
    static OVERRIDE: std::cell::RefCell<Option<std::sync::Arc<crate::fxhash::HashMap<String, Option<i32>>>>> =
        const { std::cell::RefCell::new(None) };
}

/// Run `f` with `table` (name -> builder-score units, `None` for a card the
/// re-rate measured but left under its deck cut) in place of the compiled
/// tables on this thread: the paired race of a re-rated table against the
/// shipped one (`fra_card_eval --ab --table`), both builders on the same
/// pool in one process. A card the override does not name at all is read
/// from the compiled tables as usual (the other set's cards).
pub type TableOverride = std::sync::Arc<crate::fxhash::HashMap<String, Option<i32>>>;
pub fn with_table<R>(table: TableOverride, f: impl FnOnce() -> R) -> R {
    let prev = OVERRIDE.with(|o| o.borrow_mut().replace(table));
    let r = f();
    OVERRIDE.with(|o| *o.borrow_mut() = prev);
    r
}

/// A card's measured value in builder-score units, when it has one.
pub fn rated_score(name: &str) -> Option<i32> {
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

/// The deck-shape term of the rated builder's best-of-N judge, in
/// builder-score units (a penalty, ≤ 0), over a deck's spells.
///
/// Per-card values are additive, so they cannot see that a deck needs
/// enough creatures and a curve: a deck's count of creature cards is the sum
/// of its cards' creature indicators, which the card values already price
/// linearly, but not its curvature. On the 16,000 FRA pools behind
/// [`FRA_SIM_VALUE`] the per-deck residual of the card-value fit is an
/// inverted U in all three counts (`scripts/deck_shape_fit.py`, 2026-10-26):
/// flat at 16–19 creatures and −0.7 / −1.6 / −2.3 win points at 14 / 13 / 12
/// (−1.0 / −2.4 at 22 / 23); flat at 8–11 spells of mana value ≤ 2 (−0.7 at
/// 5–6, −0.6 at 13–15); flat at 2–5 of mana value ≥ 5 (−0.7 at 1, −0.9 at
/// 0). Modelled as `k · d²` win points, `d` the distance outside the flat
/// range (creatures k 0.2, cheap 0.12, top 0.15), times [`SCALE`].
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

#[cfg(test)]
mod tests {
    use crate::draft::{ColorCounts, colors_of_picks, score_brief_rated};

    /// A listed card scores its rating, with no bonus for on-colour pips
    /// and 10 a pip off the seat's colours; an unlisted card scores its
    /// colour-independent static score the same way. Memory Trap is the case
    /// the table exists for: removal the static score ranks as filler
    /// (quality 0), measured at about +3 a copy.
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
