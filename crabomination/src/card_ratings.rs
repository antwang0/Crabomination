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
//! games, every match's margin regressed on (copies in deck A − copies in
//! deck B) — in win-rate points per copy. Played by the `fra-fixes` pilot
//! since 2026-10-25 (the `dflt` pilot misplayed some cards outright: Icy
//! Reception never countered, Living Library attacked as a 0/4, burn missed
//! lethal): Draftsim agreement 0.32 overall (0.67 on rares; `dflt` 0.28 /
//! 0.57); 17lands GIH agreement 0.29, about Draftsim's own 0.30. Only nonland cards in at least 30 decks
//! are listed; anything else keeps its static score.
//!
//! Used only under `SimConfig::rated_cards` (default off), which replaces a
//! listed card's colour-independent score with [`rated_score`]. Regenerate
//! with `fra_card_eval` + `scripts/fra_card_eval.py` (its `ratings.tsv`).

/// The value → builder-score scale: the mean and spread of the listed cards'
/// static scores (7.84 ± 3.09) matched to their values' (1.08 ± 2.39), so a
/// rating moves a card within the range the colour terms were tuned against
/// (an on-colour pip is +6) rather than swamping them.
const SCALE: f32 = 1.295;
const OFFSET: f32 = 6.444;

/// `(card name, win-rate points per copy)`, from the seed-7 run re-played
/// under the `fra-fixes` pilot (2026-10-25: same 4,000 pools and schedule as
/// the 2026-10-23 `dflt` run, so each move is the pilot's play of the card).
// Measured values, not constants: 3.14 is a card's rating, not pi.
#[allow(clippy::approx_constant)]
pub const FRA_SIM_VALUE: &[(&str, f32)] = &[
    ("Academic Ascent", 2.93),
    ("Aerid Konstrari", 5.47),
    ("Afterthought Sentry", -0.40),
    ("Apex Witchstalker", 2.69),
    ("Arcane Amphisbaena", 1.81),
    ("Archive Arbiter", 3.58),
    ("Arni, Humble Scribe", -1.51),
    ("Arni, Renowned Champion", 1.54),
    ("Artifist Acumen", 0.27),
    ("Avatar of Burgeoning Echoes", 1.70),
    ("Awaken the Inferno", 4.09),
    ("Bestial Incursion", 5.74),
    ("Blazing Crescendo", 1.59),
    ("Blessed Ghoul", 0.68),
    ("Bloodline Recollector", -0.74),
    ("Bloombrute", 0.81),
    ("Blossom-Blessed Angel", 4.85),
    ("Break Under Pressure", 4.19),
    ("Budding Insurgent", 0.72),
    ("Campus Crier", 0.78),
    ("Carnivorous Cultivator", 2.57),
    ("Cast Away Doubt", 1.22),
    ("Chandra's Emberling", -0.19),
    ("Chandra, Chill of Compliance", -2.68),
    ("Chandra, Torch of Defiance", 2.90),
    ("Charge the Sanctum", 1.13),
    ("Clash of Elements", 2.35),
    ("Codie, Ravenous Codex", -1.10),
    ("Command the Stage", -0.60),
    ("Compel Brutality", 0.42),
    ("Countersculpt", -1.25),
    ("Craftwork Crusher", 3.25),
    ("Craterclaw Colossus", 0.90),
    ("Cruel Calculations", -3.75),
    ("Cryotheory Adept", -0.21),
    ("Curse-Marred Demon", 3.91),
    ("Danitha, Spear of Agony", -0.16),
    ("Danitha, Sword of Hope", 0.57),
    ("Dark Matter Manipulator", 1.98),
    ("Darklight Phoenix", 3.32),
    ("Denzilore Fatehold", 5.32),
    ("Desperate Futurescribe", 3.54),
    ("Diviner of Victory", 0.39),
    ("Divining Duelist", -0.52),
    ("Draconic Visitor", 6.21),
    ("Eardrum Rattler", -0.87),
    ("Edgar, Ancient Bloodlord", -1.46),
    ("Edgar, Moonlit Sovereign", 0.57),
    ("Emergency Phytomedic", 0.34),
    ("Emrakul, the Exigent Doom", -0.14),
    ("Enlightened Confidant", 1.89),
    ("Entrust the Spark", -6.16),
    ("Essence Burn", 1.94),
    ("Extended Absence", 3.88),
    ("Extrapolate the Impossible", -4.26),
    ("Face Yourself", 2.13),
    ("Fatehold Charm", -1.05),
    ("Fatehold Chronologist", 0.94),
    ("Fateshaper Aspirant", -0.55),
    ("Fblthp, Impossibly Lost", -0.32),
    ("Fblthp, Knows the Way", -0.40),
    ("Ferocity of the Hunt", 0.33),
    ("Flickering Hound", -2.26),
    ("Frostbite Pyromental", -0.81),
    ("Fulminous Forte", 1.57),
    ("Gallia, Tragic Host", 2.56),
    ("Gallia, the Merrymaker", -0.71),
    ("Gardenize", -4.80),
    ("Garruk, Curse Breaker", 4.50),
    ("Garruk, Veiled Butcher", 5.83),
    ("Geist of Saint Thalia", 0.23),
    ("Generous Revival", 3.75),
    ("Germinate Recruits", -3.55),
    ("Ghalta the Immovable", 6.69),
    ("Ghalta the Unstoppable", 5.08),
    ("Gideon the Oathless", 3.42),
    ("Graft Surgeon", 1.77),
    ("Greenhouse Propagator", 1.51),
    ("Grim Repriser", -1.37),
    ("Hallway Heckler", -1.06),
    ("Hapatra, the Desert Fang", 3.07),
    ("Hapatra, the Desert Frost", 1.50),
    ("Heartstring Puller", 1.21),
    ("Heartwood Crafter", -2.04),
    ("Hexhaven Battalion", 3.11),
    ("Hexhaven Invigorator", 3.05),
    ("Hungering Puppetbeast", 2.67),
    ("Icy Reception", -0.71),
    ("Infinite Coursework", 1.65),
    ("Ingris Stingerquill", 5.57),
    ("Inspired Tethermage", 0.55),
    ("Jace's Machinations", 3.26),
    ("Jace, Reality Sculptor", 2.58),
    ("Jiang Yanggu, Alone", 3.21),
    ("Jiang Yanggu, Never Alone", 2.49),
    ("Karn, Argent Defender", -0.66),
    ("Karn, Gilded Guardian", -1.69),
    ("Keeper of the Quiet Hour", 0.67),
    ("Kindred Judgment", 3.25),
    ("Kiora of Fire and Ashes", 7.49),
    ("Kiora of Salt and Sand", -2.50),
    ("Konstrari Charm", 0.65),
    ("Konstrari Improviser", -1.17),
    ("Koth of the Homestead", -1.18),
    ("Koth, the Geomancer", 1.41),
    ("Kwia Vigorbloom", 8.62),
    ("Last Gasp", 0.04),
    ("Liliana the Faultless", -0.39),
    ("Liliana the Repentant", 0.87),
    ("Living Library", -0.25),
    ("Loot, the Anomaly", -2.69),
    ("Loot, the Nexus", -0.55),
    ("Lyra, Archangel of Dawn", 5.30),
    ("Lyra, Tolarian Archangel", 3.77),
    ("Mabel, Bitter Recluse", 1.19),
    ("Mabel, Valley Hero", 0.94),
    ("Marwyn, the Clearcutter", -0.22),
    ("Marwyn, the Preserver", 0.72),
    ("Massacre Girl, Most Wanted", 2.76),
    ("Master of Barbs", 0.17),
    ("Medic's Kitesail", -4.33),
    ("Memory Trap", 2.58),
    ("Mind Meanderer", 3.09),
    ("Mindseeker Oculus", 1.46),
    ("Multiply by Zero", 2.01),
    ("Murmuring Volume", -1.97),
    ("No Admittance", 0.61),
    ("Null Summoner", 1.31),
    ("Omnipresence", -4.11),
    ("Overwrite the Multiverse", 4.54),
    ("Paradox Shaper", -3.19),
    ("Pia, Aether Ascetic", -3.70),
    ("Pia, Determined Rebuilder", 1.58),
    ("Pompous Battlemage", -1.30),
    ("Precise Redaction", -2.38),
    ("Predictive Preparations", 2.50),
    ("Primal Witchstalker", 0.01),
    ("Proctor of Potential", 1.10),
    ("Proft, Consulting Detective", 0.42),
    ("Proft, Sinister Mastermind", -0.54),
    ("Prophesied End", 0.56),
    ("Protege's Awakening", 1.08),
    ("Prudent Fateseer", -0.68),
    ("Pyre Rhymer", 0.10),
    ("Rampart Hunter", 0.46),
    ("Rank Rat", 0.77),
    ("Recursive Recruitment", 6.48),
    ("Refute Destiny", 3.05),
    ("Repurposed Enforcer", 2.61),
    ("Rescue Girl, First Responder", -1.11),
    ("Restore with Empathy", 1.20),
    ("Return to the Light Realms", 0.78),
    ("Rewrite Regrets", 2.53),
    ("Ruric Thar, Biomagus", 4.30),
    ("Ruric Thar, Magecrusher", 3.04),
    ("Saheeli, Consul of Oversight", 4.35),
    ("Saheeli, Jewel of Avishkar", 0.24),
    ("Samut, Hazoret's Champion", 0.30),
    ("Samut, Tyrant of Naktamun", -1.45),
    ("Sanctum Lurker", 4.73),
    ("Screeching Soulbreaker", 3.32),
    ("Seasoned Cryomancer", -0.15),
    ("Semester Foreseer", 1.45),
    ("Shatterwing Pegasus", 1.37),
    ("Silence the Echo", -0.90),
    ("Simulacrum Shaper", 0.66),
    ("Skilled Battlecarver", -1.43),
    ("Solarium Sentry", 0.90),
    ("Solve for Disappointment", -0.51),
    ("Something Worth Saving", 0.85),
    ("Sphinx of False Conclusions", 1.19),
    ("Sphinx's Approach", -0.66),
    ("Stingerquill Charm", 0.69),
    ("Stingerquill Voxmancer", 1.76),
    ("Stinging Vitriol", -0.38),
    ("Sureshot Sower", 0.39),
    ("Surgical Precision", 4.66),
    ("Surveillance Phantasm", 1.82),
    ("Tam's Resistance", 1.99),
    ("Tam, the Possibility", -1.32),
    ("Tenured Tethermage", 0.57),
    ("Terminal Criticism", 1.87),
    ("Tether Technician", 1.61),
    ("Tethermage's Advantage", 2.33),
    ("Tetsuko Umezawa, Fugitive", 3.07),
    ("Tetsuko Umezawa, Pursuer", 5.26),
    ("Teyo, Diamondblade Mage", -1.53),
    ("Teyo, Lightshield Expert", -1.95),
    ("Thalia, the Survivor", 3.14),
    ("The Theorist, Jace Beleren", 3.63),
    ("Theoretical Necromancer", 1.74),
    ("Theorist's Proxy", -1.21),
    ("Theorix Charm", -2.66),
    ("Theorix Metamage", 0.17),
    ("Tinybones, Pocket Nuisance", 1.82),
    ("Titanbones, Towering Heart", 3.66),
    ("Tomik, Izzet Sparkmage", -1.19),
    ("Tomik, Orzhov Lawmage", 0.37),
    ("Traxos, Academy Guardian", 2.77),
    ("Traxos, Scourge Eternal", 3.31),
    ("Twinned Vision", -0.16),
    ("Twisted Fates", 3.73),
    ("Uldaros Theorix", 1.96),
    ("Undulating Witness", 3.33),
    ("Unflinching Hortimancer", -0.10),
    ("Unsummon", -1.03),
    ("Variable Chaser", 1.89),
    ("Verdant Kraken", 5.07),
    ("Vigorbloom Charm", 0.24),
    ("Vigorbloom Vanguard", 1.36),
    ("Vindictive Triumph", -3.28),
    ("Vinelasher Adept", -2.97),
    ("Violent Echoes", 3.03),
    ("Void Extrapolator", 0.09),
    ("Vraska's Final Mercy", 1.74),
    ("Vraska, the Cutting Glare", 4.20),
    ("Whiplash Wordsmith", -0.50),
    ("Winter, Team Player", 0.16),
    ("Winter, Tormented Loner", 0.82),
    ("Woodwork Prodigy", -0.41),
    ("Wrath of the Bloodmane", 1.25),
    ("Wrecking Gecko", 2.40),
    ("Yargle, Glutton of Urborg", 0.69),
    ("Yargle, Goliath of Otaria", 1.10),
    ("Yoshimaru, Beloved Companion", -1.13),
    ("Yoshimaru, Scrappy Stray", -3.85),
    ("Your Fate Ends Here", 3.57),
    ("Yuriko, Blade of the Mighty", 2.27),
    ("Yuriko, Hope from the Shadows", -0.79),
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
