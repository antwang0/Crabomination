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
