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
//! `fra_card_eval` — 4,000 FRA sealed pools built by the v3 builder, 240k
//! `dflt` games, every match's margin regressed on (copies in deck A − copies
//! in deck B) — in win-rate points per copy. Split-half reliability 0.95;
//! Draftsim agreement 0.28 overall (0.57 on rares); 17lands GIH agreement
//! 0.28, about Draftsim's own 0.31. Only nonland cards in at least 30 decks
//! are listed; anything else keeps its static score.
//!
//! Used only under `SimConfig::rated_cards` (default off), which replaces a
//! listed card's colour-independent score with [`rated_score`]. Regenerate
//! with `fra_card_eval` + `scripts/fra_card_eval.py` (its `ratings.tsv`).

/// The value → builder-score scale: the mean and spread of the listed cards'
/// static scores (7.77 ± 3.08) matched to their values' (1.18 ± 2.48), so a
/// rating moves a card within the range the colour terms were tuned against
/// (an on-colour pip is +6) rather than swamping them.
const SCALE: f32 = 1.242;
const OFFSET: f32 = 6.307;

/// `(card name, win-rate points per copy)`, from the 2026-10-23 seed-7 run.
pub const FRA_SIM_VALUE: &[(&str, f32)] = &[
    ("Academic Ascent", 0.95),
    ("Aerid Konstrari", 5.00),
    ("Afterthought Sentry", -0.24),
    ("Apex Witchstalker", 2.64),
    ("Arcane Amphisbaena", 2.35),
    ("Archive Arbiter", 3.00),
    ("Arni, Humble Scribe", -0.76),
    ("Arni, Renowned Champion", 1.82),
    ("Artifist Acumen", 1.23),
    ("Avatar of Burgeoning Echoes", 2.58),
    ("Awaken the Inferno", 4.04),
    ("Bestial Incursion", 5.84),
    ("Blazing Crescendo", -1.07),
    ("Blessed Ghoul", 0.77),
    ("Bloodline Recollector", 0.02),
    ("Bloombrute", 1.56),
    ("Blossom-Blessed Angel", 4.99),
    ("Break Under Pressure", 3.95),
    ("Budding Insurgent", 0.88),
    ("Campus Crier", 1.09),
    ("Carnivorous Cultivator", 3.60),
    ("Cast Away Doubt", 1.71),
    ("Chandra's Emberling", 0.44),
    ("Chandra, Chill of Compliance", -0.48),
    ("Chandra, Torch of Defiance", 4.55),
    ("Charge the Sanctum", -2.27),
    ("Clash of Elements", 1.87),
    ("Codie, Ravenous Codex", -0.23),
    ("Command the Stage", 1.34),
    ("Compel Brutality", 0.71),
    ("Countersculpt", -1.74),
    ("Craftwork Crusher", 3.94),
    ("Craterclaw Colossus", 1.98),
    ("Cruel Calculations", -4.08),
    ("Cryotheory Adept", -0.09),
    ("Curse-Marred Demon", 3.41),
    ("Danitha, Spear of Agony", -0.05),
    ("Danitha, Sword of Hope", 1.09),
    ("Denzilore Fatehold", 5.59),
    ("Desperate Futurescribe", 3.55),
    ("Diviner of Victory", -0.16),
    ("Divining Duelist", -0.12),
    ("Draconic Visitor", 5.88),
    ("Eardrum Rattler", -0.70),
    ("Edgar, Ancient Bloodlord", -1.18),
    ("Edgar, Moonlit Sovereign", 0.89),
    ("Emergency Phytomedic", 0.37),
    ("Emrakul, the Exigent Doom", 0.89),
    ("Enlightened Confidant", -0.59),
    ("Entrust the Spark", -3.59),
    ("Essence Burn", -0.82),
    ("Extended Absence", 4.12),
    ("Face Yourself", 3.06),
    ("Fatehold Charm", -0.92),
    ("Fatehold Chronologist", 1.10),
    ("Fateshaper Aspirant", -0.09),
    ("Fblthp, Impossibly Lost", 0.35),
    ("Fblthp, Knows the Way", 0.09),
    ("Ferocity of the Hunt", 0.67),
    ("Flickering Hound", -1.53),
    ("Frostbite Pyromental", -1.65),
    ("Fulminous Forte", 2.10),
    ("Gallia, Tragic Host", 2.72),
    ("Gallia, the Merrymaker", -0.43),
    ("Gardenize", -4.79),
    ("Garruk, Curse Breaker", 4.95),
    ("Garruk, Veiled Butcher", 5.78),
    ("Geist of Saint Thalia", -0.28),
    ("Generous Revival", 3.25),
    ("Germinate Recruits", -4.40),
    ("Ghalta the Immovable", 7.54),
    ("Ghalta the Unstoppable", 5.88),
    ("Gideon the Oathless", 3.51),
    ("Gideon's Memorial", -8.95),
    ("Graft Surgeon", 1.93),
    ("Greenhouse Propagator", 1.78),
    ("Grim Repriser", -1.27),
    ("Hallway Heckler", -0.65),
    ("Hapatra, the Desert Fang", 3.02),
    ("Hapatra, the Desert Frost", 2.42),
    ("Heartstring Puller", 1.37),
    ("Heartwood Crafter", 0.42),
    ("Hexhaven Battalion", 3.29),
    ("Hexhaven Invigorator", 3.80),
    ("Hungering Puppetbeast", 3.10),
    ("Icy Reception", -1.34),
    ("Infinite Coursework", 3.41),
    ("Inspired Tethermage", 0.14),
    ("Jace's Machinations", 2.56),
    ("Jace, Reality Sculptor", 4.18),
    ("Jiang Yanggu, Alone", 3.07),
    ("Jiang Yanggu, Never Alone", 3.30),
    ("Karn, Argent Defender", -0.41),
    ("Karn, Gilded Guardian", -1.28),
    ("Keeper of the Quiet Hour", 1.13),
    ("Kindred Judgment", 5.65),
    ("Kiora of Fire and Ashes", 8.56),
    ("Kiora of Salt and Sand", -0.96),
    ("Konstrari Charm", 1.52),
    ("Konstrari Improviser", -0.70),
    ("Koth of the Homestead", -0.88),
    ("Koth, the Geomancer", 0.95),
    ("Kwia Vigorbloom", 9.54),
    ("Last Gasp", 0.17),
    ("Liliana the Faultless", 1.24),
    ("Liliana the Repentant", -0.46),
    ("Living Library", -0.92),
    ("Loot, the Anomaly", -1.66),
    ("Loot, the Nexus", -0.41),
    ("Lyra, Archangel of Dawn", 4.67),
    ("Lyra, Tolarian Archangel", 3.51),
    ("Mabel, Bitter Recluse", 2.75),
    ("Mabel, Valley Hero", 2.21),
    ("Marwyn, the Clearcutter", -2.58),
    ("Marwyn, the Preserver", 0.33),
    ("Massacre Girl, Most Wanted", 4.02),
    ("Master of Barbs", 0.62),
    ("Medic's Kitesail", -2.90),
    ("Memory Trap", 3.14),
    ("Mind Meanderer", 3.14),
    ("Mindseeker Oculus", 1.88),
    ("Multiply by Zero", 2.10),
    ("Murmuring Volume", -2.60),
    ("No Admittance", -1.17),
    ("Null Summoner", 0.78),
    ("Omnipresence", -4.25),
    ("Paradox Shaper", -3.25),
    ("Pia, Aether Ascetic", -3.33),
    ("Pia, Determined Rebuilder", 1.07),
    ("Pompous Battlemage", -0.61),
    ("Precise Redaction", -0.25),
    ("Predictive Preparations", 2.29),
    ("Primal Witchstalker", 1.34),
    ("Proctor of Potential", 0.73),
    ("Proft, Consulting Detective", 0.54),
    ("Proft, Sinister Mastermind", -0.92),
    ("Prophesied End", 0.32),
    ("Protege's Awakening", 1.61),
    ("Prudent Fateseer", -0.67),
    ("Pyre Rhymer", 0.00),
    ("Rampart Hunter", 1.40),
    ("Rank Rat", 1.11),
    ("Recursive Recruitment", 3.34),
    ("Refute Destiny", 0.59),
    ("Repurposed Enforcer", 2.55),
    ("Rescue Girl, First Responder", -0.93),
    ("Restore with Empathy", 1.38),
    ("Return to the Light Realms", 1.94),
    ("Rewrite Regrets", 1.15),
    ("Ruric Thar, Biomagus", 3.79),
    ("Ruric Thar, Magecrusher", 4.03),
    ("Saheeli, Consul of Oversight", 3.82),
    ("Saheeli, Jewel of Avishkar", -0.13),
    ("Samut, Hazoret's Champion", -0.10),
    ("Samut, Tyrant of Naktamun", -1.19),
    ("Sanctum Lurker", 4.49),
    ("Screeching Soulbreaker", 3.08),
    ("Seasoned Cryomancer", -0.64),
    ("Shatterwing Pegasus", 1.54),
    ("Silence the Echo", -0.55),
    ("Simulacrum Shaper", 1.96),
    ("Skilled Battlecarver", -1.49),
    ("Solarium Sentry", 1.05),
    ("Solve for Disappointment", 0.69),
    ("Something Worth Saving", 1.17),
    ("Sphinx of False Conclusions", 0.92),
    ("Sphinx's Approach", 0.09),
    ("Stingcaster Mage", -0.54),
    ("Stingerquill Charm", -0.90),
    ("Sureshot Sower", 0.26),
    ("Surgical Precision", 4.15),
    ("Surveillance Phantasm", 1.42),
    ("Tam's Resistance", 1.97),
    ("Tam, the Possibility", -1.90),
    ("Tarmogoyf", 2.19),
    ("Tenured Tethermage", 0.55),
    ("Terminal Criticism", -0.82),
    ("Tether Technician", 2.08),
    ("Tethermage's Advantage", -0.40),
    ("Tetsuko Umezawa, Fugitive", 4.46),
    ("Tetsuko Umezawa, Pursuer", 3.94),
    ("Teyo, Diamondblade Mage", -1.76),
    ("Teyo, Lightshield Expert", -1.42),
    ("Thalia, the Survivor", 4.02),
    ("The Theorist, Jace Beleren", 5.46),
    ("Theoretical Necromancer", 1.82),
    ("Theorist's Proxy", -0.62),
    ("Theorix Charm", -2.54),
    ("Tinybones, Pocket Nuisance", 1.79),
    ("Titanbones, Towering Heart", 3.33),
    ("Tomik, Izzet Sparkmage", -0.67),
    ("Tomik, Orzhov Lawmage", 0.15),
    ("Traxos, Academy Guardian", 2.53),
    ("Traxos, Scourge Eternal", 3.55),
    ("Twinned Vision", 0.60),
    ("Twisted Fates", 4.80),
    ("Uldaros Theorix", 0.26),
    ("Undulating Witness", 2.92),
    ("Unflinching Hortimancer", -0.29),
    ("Unsummon", -1.53),
    ("Variable Chaser", 2.24),
    ("Verdant Kraken", 6.24),
    ("Vigorbloom Charm", 0.30),
    ("Vigorbloom Vanguard", 1.65),
    ("Vindictive Triumph", -2.31),
    ("Vinelasher Adept", -3.39),
    ("Violent Echoes", 4.06),
    ("Vraska's Final Mercy", 0.60),
    ("Vraska, Soul of Stone", -1.83),
    ("Vraska, the Cutting Glare", 5.35),
    ("Warrior's Blades", 1.33),
    ("Winter, Team Player", -0.28),
    ("Winter, Tormented Loner", 0.22),
    ("Woodwork Prodigy", 1.15),
    ("Wrath of the Bloodmane", 1.54),
    ("Wrecking Gecko", 2.50),
    ("Yargle, Glutton of Urborg", 1.14),
    ("Yargle, Goliath of Otaria", 1.74),
    ("Yoshimaru, Beloved Companion", 0.75),
    ("Yoshimaru, Scrappy Stray", -1.16),
    ("Your Fate Ends Here", 4.83),
    ("Yuriko, Blade of the Mighty", 3.65),
    ("Yuriko, Hope from the Shadows", -0.76),
];

/// A card's measured value in builder-score units, when it has one.
pub fn rated_score(name: &str) -> Option<i32> {
    static MAP: std::sync::OnceLock<crate::fxhash::HashMap<&'static str, i32>> = std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        FRA_SIM_VALUE.iter().map(|&(n, v)| (n, (OFFSET + SCALE * v).round() as i32)).collect()
    })
    .get(name)
    .copied()
}

#[cfg(test)]
mod tests {
    use crate::draft::{ColorCounts, colors_of_picks, score_brief_quality, score_brief_rated};

    /// A listed card trades its static `base_score + quality` for its rating
    /// and keeps its colour term; an unlisted card scores exactly as before.
    /// Memory Trap is the case the table exists for: removal the static
    /// score ranks as filler (quality 0), measured at about +3 a copy.
    #[test]
    fn rated_cards_replace_only_the_colour_independent_score() {
        let trap = crate::cube::card_brief(crate::catalog::memory_trap);
        let colors: ColorCounts = colors_of_picks(&[crate::catalog::memory_trap]);
        let r = super::rated_score("Memory Trap").expect("Memory Trap is rated");
        assert_eq!(trap.rated, Some(r));
        assert_eq!(score_brief_rated(trap, &colors), score_brief_quality(trap, &colors) - trap.base_score - trap.quality + r);
        assert!(r > trap.base_score + trap.quality, "measured above its static score");
        let bears = crate::cube::card_brief(crate::catalog::grizzly_bears);
        assert_eq!(bears.rated, None);
        assert_eq!(score_brief_rated(bears, &colors), score_brief_quality(bears, &colors));
    }
}
