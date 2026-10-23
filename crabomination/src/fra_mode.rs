//! Reality Fracture (FRA) sealed: the set's booster sheet and a pack roller,
//! so a sealed pool can be opened from FRA instead of Secrets of Strixhaven.
//!
//! [`crate::selfplay::sealed_pool`] reads [`SealedSet::current`] —
//! `CRAB_SEALED_SET=fra` in the environment — so every caller that opens a
//! sealed pool (the self-play trainer's actors, `bot_ladder --decks sealed`,
//! the client's sealed opponent) switches set together, and the default stays
//! SOS: a training field, a gate deck and a recorded reference all keep their
//! meaning unless the variable is set.
//!
//! **The pack.** Fourteen cards: one rare or mythic (mythic one time in
//! eight), three uncommons, ten commons, no duplicates within a pack. A Play
//! Booster's wildcard, foil and land slots are folded into the common and
//! rare counts; what a sealed pool needs is the rarity mix, and this keeps it
//! (84 cards from six packs: six rare-slot cards, eighteen uncommons). Rarities
//! are Scryfall's for set `fra` (280 non-basic cards, fetched 2026-10-23).

use crate::card::CardDefinition;
use rand::{Rng, RngExt};

type CardFactory = fn() -> CardDefinition;

/// A card's rarity on the booster sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
}

use Rarity::{Common as C, Mythic as M, Rare as R, Uncommon as U};

/// The FRA booster sheet by collector number: every non-basic card, by its
/// front-face name (a prepare creature's spell half is not a separate card).
pub const FRA_SHEET: &[(&str, Rarity)] = &[
    ("Emrakul, the Exigent Doom", M),
    ("Academic Ascent", C),
    ("Blossom-Blessed Angel", C),
    ("Campus Crier", C),
    ("Enlightened Confidant", M),
    ("Fateshaper Aspirant", C),
    ("Flickering Hound", R),
    ("Generous Revival", U),
    ("Germinate Recruits", R),
    ("Graft Surgeon", C),
    ("Guiding Hydra", R),
    ("Hexhaven Battalion", C),
    ("Kindred Judgment", R),
    ("Loyal Tutor", R),
    ("Memory Trap", C),
    ("Predictive Preparations", C),
    ("Prophesied End", U),
    ("Refute Destiny", U),
    ("Repurposed Enforcer", R),
    ("Return to the Light Realms", M),
    ("Shatterwing Pegasus", C),
    ("Surgical Precision", C),
    ("Unflinching Hortimancer", C),
    ("Your Fate Ends Here", U),
    ("Countersculpt", U),
    ("Cruel Calculations", R),
    ("Cryotheory Adept", C),
    ("Diviner of Victory", R),
    ("Divining Duelist", C),
    ("Icy Reception", C),
    ("Infinite Coursework", C),
    ("Jace's Machinations", R),
    ("Mindseeker Oculus", C),
    ("Perfected Theory", U),
    ("Plan for All Outcomes", U),
    ("Precise Redaction", U),
    ("Protege's Awakening", C),
    ("Seasoned Cryomancer", M),
    ("Semester Foreseer", C),
    ("Sphinx of False Conclusions", R),
    ("Sphinx's Approach", C),
    ("Surveillance Phantasm", C),
    ("The Theorist, Jace Beleren", M),
    ("Theorist's Proxy", R),
    ("Undulating Witness", C),
    ("Unsummon", C),
    ("Variable Chaser", R),
    ("Apex Witchstalker", C),
    ("Bloodline Recollector", M),
    ("Break Under Pressure", U),
    ("Cast Away Doubt", C),
    ("Dark Matter Manipulator", R),
    ("Darklight Phoenix", M),
    ("Extended Absence", C),
    ("Extrapolate the Impossible", R),
    ("Last Gasp", C),
    ("Lich's Relic", R),
    ("Multiply by Zero", U),
    ("Overwrite the Multiverse", M),
    ("Rampart Hunter", C),
    ("Rank Rat", C),
    ("Rewrite Regrets", U),
    ("Rise of the Deathbringer", R),
    ("Sanctum Lurker", R),
    ("Screeching Soulbreaker", C),
    ("Silence the Echo", C),
    ("Solve for Disappointment", C),
    ("Terminal Criticism", U),
    ("Theoretical Necromancer", C),
    ("Void Extrapolator", C),
    ("Vraska's Final Mercy", R),
    ("Ajani's Anguish", R),
    ("Artifist Acumen", C),
    ("Awaken the Inferno", C),
    ("Blazing Crescendo", C),
    ("Chandra's Emberling", C),
    ("Command the Stage", U),
    ("Craterclaw Colossus", M),
    ("Curse-Marred Demon", R),
    ("Draconic Visitor", R),
    ("Eardrum Rattler", C),
    ("Essence Burn", U),
    ("Face Yourself", R),
    ("Fulminous Forte", U),
    ("Hallway Heckler", C),
    ("Heartstring Puller", C),
    ("Identity Echo", R),
    ("Master of Barbs", R),
    ("No Admittance", C),
    ("Pompous Battlemage", R),
    ("Pyre Rhymer", R),
    ("Skilled Battlecarver", C),
    ("Stingcaster Mage", M),
    ("Tether Technician", C),
    ("Violent Echoes", U),
    ("Wrath of the Bloodmane", C),
    ("Arcane Amphisbaena", C),
    ("Bestial Incursion", C),
    ("Budding Insurgent", C),
    ("Carnivorous Cultivator", R),
    ("Compel Brutality", C),
    ("Flourishing Grapple", U),
    ("Gardenize", R),
    ("Greenhouse Propagator", C),
    ("Heartwood Crafter", U),
    ("Hexhaven Invigorator", M),
    ("Hungering Puppetbeast", R),
    ("Hunter's Axe", U),
    ("Inspired Tethermage", C),
    ("Omnipresence", M),
    ("Puppet Crafting", R),
    ("Restore with Empathy", U),
    ("Simulacrum Shaper", R),
    ("Something Worth Saving", C),
    ("Sureshot Sower", C),
    ("Tarmogoyf", M),
    ("Tethermage's Advantage", C),
    ("Verdant Kraken", R),
    ("Vinelasher Adept", C),
    ("Wrecking Gecko", C),
    ("Aerid Konstrari", M),
    ("Avatar of Burgeoning Echoes", M),
    ("Blessed Ghoul", C),
    ("Bloombrute", U),
    ("Charge the Sanctum", C),
    ("Clash of Elements", U),
    ("Craftwork Crusher", U),
    ("Denzilore Fatehold", M),
    ("Desperate Futurescribe", U),
    ("Emergency Phytomedic", C),
    ("Entrust the Spark", R),
    ("Fatehold Charm", U),
    ("Fatehold Chronologist", C),
    ("Ferocity of the Hunt", C),
    ("Frostbite Pyromental", R),
    ("Grim Repriser", U),
    ("Ingris Stingerquill", M),
    ("Konstrari Charm", U),
    ("Konstrari Improviser", C),
    ("Kwia Vigorbloom", M),
    ("Mind Meanderer", U),
    ("Null Summoner", R),
    ("Paradox Shaper", U),
    ("Primal Witchstalker", U),
    ("Proctor of Potential", R),
    ("Prudent Fateseer", U),
    ("Recursive Recruitment", U),
    ("Solarium Sentry", R),
    ("Solitary Cell", R),
    ("Stingerquill Charm", U),
    ("Stingerquill Voxmancer", U),
    ("Stinging Vitriol", R),
    ("Tam's Resistance", C),
    ("Tenured Tethermage", R),
    ("Theorix Charm", U),
    ("Theorix Metamage", C),
    ("Twinned Vision", C),
    ("Twisted Fates", U),
    ("Uldaros Theorix", M),
    ("Vigorbloom Charm", U),
    ("Vigorbloom Vanguard", U),
    ("Vindictive Triumph", R),
    ("Warrior's Blades", U),
    ("Whiplash Wordsmith", C),
    ("Woodwork Prodigy", U),
    ("Afterthought Sentry", C),
    ("Archive Arbiter", U),
    ("Codie, Ravenous Codex", R),
    ("The Echoverse Fulcrum", M),
    ("Eye of Jace", U),
    ("Keeper of the Quiet Hour", C),
    ("Living Library", C),
    ("Medic's Kitesail", C),
    ("Murmuring Volume", C),
    ("Dedicated Commons", C),
    ("Deserted Beach", R),
    ("Fatehold Annex", C),
    ("Formidable Commons", C),
    ("Hall of Echoes", R),
    ("Haunted Ridge", R),
    ("Hexhaven Dueling Arena", U),
    ("Innovative Commons", C),
    ("Konstrari Annex", C),
    ("Meticulous Commons", C),
    ("Overgrown Farmland", R),
    ("Rockfall Vale", R),
    ("Roiling Canopy", R),
    ("Room of Refuge", C),
    ("Shipwreck Marsh", R),
    ("Stingerquill Annex", C),
    ("Theorist's Sanctum", R),
    ("Theorix Annex", C),
    ("Transformative Commons", C),
    ("Vigorbloom Annex", C),
    ("Ajani Resolute", M),
    ("Danitha, Sword of Hope", U),
    ("Ghalta the Immovable", U),
    ("Gideon's Memorial", R),
    ("Koth of the Homestead", U),
    ("Liliana the Faultless", R),
    ("Lyra, Archangel of Dawn", R),
    ("Rescue Girl, First Responder", U),
    ("Saheeli, Consul of Oversight", U),
    ("Teyo, Lightshield Expert", U),
    ("Thalia, the Survivor", U),
    ("Tomik, Orzhov Lawmage", U),
    ("Way of the Healer", U),
    ("Way of the Mentor", U),
    ("Yoshimaru, Beloved Companion", U),
    ("Yuriko, Blade of the Mighty", U),
    ("Arni, Humble Scribe", U),
    ("Chandra, Chill of Compliance", M),
    ("Fblthp, Impossibly Lost", U),
    ("Geist of Saint Thalia", U),
    ("Hapatra, the Desert Frost", U),
    ("Jace, Reality Sculptor", R),
    ("Lyra, Tolarian Archangel", R),
    ("Proft, Consulting Detective", U),
    ("Ruric Thar, Biomagus", U),
    ("Samut, Tyrant of Naktamun", R),
    ("Tetsuko Umezawa, Fugitive", U),
    ("Traxos, Academy Guardian", U),
    ("Way of the Cryomancer", U),
    ("Way of the Mind Sculptor", U),
    ("Yargle, Goliath of Otaria", U),
    ("Yuriko, Hope from the Shadows", U),
    ("Danitha, Spear of Agony", U),
    ("Gallia, Tragic Host", U),
    ("Garruk, Veiled Butcher", M),
    ("Gideon the Oathless", R),
    ("Liliana the Repentant", R),
    ("Loot, the Anomaly", U),
    ("Mabel, Bitter Recluse", U),
    ("Massacre Girl, Most Wanted", U),
    ("Proft, Sinister Mastermind", U),
    ("Teyo, Diamondblade Mage", U),
    ("Tinybones, Pocket Nuisance", U),
    ("Way of the Deathbringer", U),
    ("Way of the Necromancer", U),
    ("Winter, Tormented Loner", U),
    ("Yargle, Glutton of Urborg", U),
    ("Ajani Unrelenting", M),
    ("Arni, Renowned Champion", U),
    ("Chandra, Torch of Defiance", M),
    ("Gallia, the Merrymaker", U),
    ("Jiang Yanggu, Alone", U),
    ("Kiora of Fire and Ashes", U),
    ("Koth, the Geomancer", U),
    ("Marwyn, the Clearcutter", U),
    ("Pia, Determined Rebuilder", U),
    ("Samut, Hazoret's Champion", R),
    ("Tetsuko Umezawa, Pursuer", U),
    ("Tomik, Izzet Sparkmage", U),
    ("Way of the Pyromancer", U),
    ("Way of the Warlord", U),
    ("Winter, Team Player", U),
    ("Edgar, Moonlit Sovereign", U),
    ("Fblthp, Knows the Way", U),
    ("Garruk, Curse Breaker", M),
    ("Ghalta the Unstoppable", U),
    ("Jiang Yanggu, Never Alone", U),
    ("Loot, the Nexus", U),
    ("Marwyn, the Preserver", U),
    ("Pia, Aether Ascetic", U),
    ("Ruric Thar, Magecrusher", U),
    ("Titanbones, Towering Heart", U),
    ("Way of the Paradox", U),
    ("Way of the Wildspeaker", U),
    ("Yoshimaru, Scrappy Stray", U),
    ("Edgar, Ancient Bloodlord", U),
    ("Hapatra, the Desert Fang", U),
    ("Karn, Gilded Guardian", R),
    ("Kiora of Salt and Sand", U),
    ("Mabel, Valley Hero", U),
    ("Saheeli, Jewel of Avishkar", U),
    ("Tam, the Possibility", R),
    ("Vraska, Soul of Stone", R),
    ("Vraska, the Cutting Glare", R),
    ("Karn, Argent Defender", R),
    ("Traxos, Scourge Eternal", U),
];

/// Which set a sealed pool opens. See the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealedSet {
    /// Secrets of Strixhaven — the default, and every recorded reference's.
    Sos,
    /// Reality Fracture.
    Fra,
}

impl SealedSet {
    /// `CRAB_SEALED_SET` (`sos` | `fra`), read once per process. An unknown
    /// value is a panic rather than a quiet SOS: a gate that meant to run on
    /// FRA and silently didn't is the measurement this switch exists for.
    pub fn current() -> SealedSet {
        static SET: std::sync::OnceLock<SealedSet> = std::sync::OnceLock::new();
        *SET.get_or_init(|| match std::env::var("CRAB_SEALED_SET").as_deref() {
            Err(_) | Ok("") | Ok("sos") => SealedSet::Sos,
            Ok("fra") => SealedSet::Fra,
            Ok(other) => panic!("CRAB_SEALED_SET={other}: expected sos or fra"),
        })
    }
}

/// The sheet resolved to catalog factories, bucketed by rarity. Built once.
pub struct FraPacks {
    pub common: Vec<CardFactory>,
    pub uncommon: Vec<CardFactory>,
    pub rare: Vec<CardFactory>,
    pub mythic: Vec<CardFactory>,
}

impl FraPacks {
    pub fn get() -> &'static FraPacks {
        static PACKS: std::sync::OnceLock<FraPacks> = std::sync::OnceLock::new();
        PACKS.get_or_init(|| {
            let mut p = FraPacks { common: vec![], uncommon: vec![], rare: vec![], mythic: vec![] };
            for &(name, rarity) in FRA_SHEET {
                let f = crate::card_registry::factory_by_name(name)
                    .unwrap_or_else(|| panic!("FRA sheet card {name} is not in the catalog"));
                match rarity {
                    C => p.common.push(f),
                    U => p.uncommon.push(f),
                    R => p.rare.push(f),
                    M => p.mythic.push(f),
                }
            }
            p
        })
    }

    /// Every card on the sheet once — the pool the vocabulary must cover.
    pub fn all(&self) -> impl Iterator<Item = CardFactory> + '_ {
        self.common.iter().chain(&self.uncommon).chain(&self.rare).chain(&self.mythic).copied()
    }

    /// Roll one 14-card pack.
    pub fn roll<G: Rng>(&self, rng: &mut G) -> Vec<CardFactory> {
        let mut pack = Vec::with_capacity(14);
        let rare_sheet = if rng.random_range(0..8) == 0 { &self.mythic } else { &self.rare };
        pack.push(rare_sheet[rng.random_range(0..rare_sheet.len())]);
        for (sheet, n) in [(&self.uncommon, 3), (&self.common, 10)] {
            let mut taken: Vec<usize> = Vec::with_capacity(n);
            while taken.len() < n {
                let i = rng.random_range(0..sheet.len());
                if !taken.contains(&i) {
                    taken.push(i);
                    pack.push(sheet[i]);
                }
            }
        }
        pack
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn the_fra_sheet_resolves_and_rolls_the_rarity_mix() {
        let p = FraPacks::get();
        assert_eq!(
            (p.common.len(), p.uncommon.len(), p.rare.len(), p.mythic.len()),
            (81, 109, 64, 26),
            "Scryfall's FRA counts"
        );
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        for _ in 0..50 {
            let pack = p.roll(&mut rng);
            assert_eq!(pack.len(), 14);
            let distinct: std::collections::HashSet<usize> = pack.iter().map(|f| *f as usize).collect();
            assert_eq!(distinct.len(), 14, "no duplicates within a pack");
        }
    }
}
