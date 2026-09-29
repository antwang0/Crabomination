//! Printed rules text — each card's Oracle wording — for the hover preview
//! and the Alt peek.
//!
//! Cards render art-only at table size, so the preview's text panel is where
//! a player reads what a card does. It used to phrase each ability from the
//! engine's own shapes (`effect_short_text`), which covered the common
//! triggers and fell back to "Triggered ability:" for the rest.
//! `assets/oracle.tsv`, compiled in, holds the Oracle text of every card the
//! catalog knows, keyed by each name a card is shown under; a card it does
//! not hold — a token, a card added since the table was built — keeps the
//! phrased lines.
//!
//! One line per name: the name, then five tab-separated fields per face —
//! name (empty: the line's own), mana cost, type line, stats (`3/2`,
//! `Loyalty 4`, `Defense 5`, or empty) and rules text, its paragraph breaks
//! written `\n`. A split, adventure or prepare card lists every face under
//! each of its names; a double-faced card lists only the face the name is on.
//!
//! Rebuild it from `scripts/.scryfall_cache.json` after the catalog grows:
//!
//! ```text
//! CRAB_BLESS_ORACLE=1 cargo test -p crabomination_client oracle -- --nocapture
//! ```

use std::collections::HashMap;
use std::sync::OnceLock;

const TABLE: &str = include_str!("../../assets/oracle.tsv");

/// One face of a card as printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OracleFace {
    pub name: &'static str,
    pub cost: &'static str,
    pub type_line: &'static str,
    /// `3/2`, `Loyalty 4`, `Defense 5`, or empty.
    pub stats: &'static str,
    text: &'static str,
}

impl OracleFace {
    /// The rules text's paragraphs, in print order.
    pub fn paragraphs(&self) -> impl Iterator<Item = &'static str> {
        self.text.split("\\n").filter(|p| !p.is_empty())
    }
}

/// The printed faces of the card shown as `name`, or `None` when the table
/// doesn't hold it.
pub fn printed(name: &str) -> Option<&'static [OracleFace]> {
    static INDEX: OnceLock<HashMap<&'static str, Vec<OracleFace>>> = OnceLock::new();
    INDEX.get_or_init(|| parse(TABLE)).get(name).map(Vec::as_slice)
}

fn parse(table: &'static str) -> HashMap<&'static str, Vec<OracleFace>> {
    table
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let key = fields.next()?;
            let rest: Vec<&'static str> = fields.collect();
            let faces = rest
                .chunks_exact(5)
                .map(|f| OracleFace {
                    name: if f[0].is_empty() { key } else { f[0] },
                    cost: f[1],
                    type_line: f[2],
                    stats: f[3],
                    text: f[4],
                })
                .collect::<Vec<_>>();
            (!faces.is_empty()).then_some((key, faces))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::fmt::Write as _;
    use std::path::Path;

    /// Layouts whose faces are two sides of one card: a name shows its own
    /// face only. Every other multi-face layout (split, adventure, flip,
    /// prepare, …) prints all its faces on one side.
    const TWO_SIDED: &[&str] = &["transform", "modal_dfc", "meld", "reversible_card"];

    /// The table for the catalog as it stands, from the Scryfall cache.
    fn build() -> (String, usize, Vec<&'static str>) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let raw = std::fs::read_to_string(root.join("scripts/.scryfall_cache.json"))
            .expect("scripts/.scryfall_cache.json");
        let cache: serde_json::Map<String, Value> = serde_json::from_str(&raw).expect("cache is a JSON object");
        // A name the cache could not resolve is stored as a string, not an
        // object; those are skipped. Cache keys first, then each entry's own
        // name and its faces' names, so a face never shadows a card.
        let mut by_name: HashMap<String, &Value> = HashMap::new();
        for (key, card) in cache.iter().filter(|(_, v)| v.is_object()) {
            by_name.insert(key.to_lowercase(), card);
        }
        for card in cache.values().filter(|v| v.is_object()) {
            let faces = card.get("card_faces").and_then(Value::as_array).into_iter().flatten();
            for name in std::iter::once(card).chain(faces).filter_map(|o| o.get("name")?.as_str()) {
                by_name.entry(name.to_lowercase()).or_insert(card);
            }
        }

        let mut names: BTreeSet<&'static str> = BTreeSet::new();
        for factory in crabomination::catalog::all_known_factories() {
            let def = factory();
            names.insert(def.name);
            if let Some(back) = &def.back_face {
                names.insert(back.name);
            }
        }

        let mut out = String::new();
        let mut missing = Vec::new();
        for &name in &names {
            let Some(card) = by_name.get(&name.to_lowercase()) else {
                missing.push(name);
                continue;
            };
            let layout = card.get("layout").and_then(Value::as_str).unwrap_or("");
            let faces: Vec<&Value> = match card.get("card_faces").and_then(Value::as_array) {
                None => vec![card],
                Some(faces) => {
                    let own = faces
                        .iter()
                        .filter(|f| f.get("name").and_then(Value::as_str).is_some_and(|n| n.eq_ignore_ascii_case(name)))
                        .collect::<Vec<_>>();
                    if TWO_SIDED.contains(&layout) && !own.is_empty() { own } else { faces.iter().collect() }
                }
            };
            out.push_str(name);
            for face in faces {
                let field = |k: &str| face.get(k).and_then(Value::as_str).unwrap_or("");
                let stats = match (field("power"), field("toughness"), field("loyalty"), field("defense")) {
                    (p, t, _, _) if !p.is_empty() && !t.is_empty() => format!("{p}/{t}"),
                    (_, _, l, _) if !l.is_empty() => format!("Loyalty {l}"),
                    (_, _, _, d) if !d.is_empty() => format!("Defense {d}"),
                    _ => String::new(),
                };
                let face_name = if field("name") == name { "" } else { field("name") };
                let text = field("oracle_text").replace('\n', "\\n");
                for s in [face_name, field("mana_cost"), field("type_line"), &text] {
                    assert!(!s.contains('\t'), "{name}: a tab in {s:?}");
                }
                write!(out, "\t{face_name}\t{}\t{}\t{stats}\t{text}", field("mana_cost"), field("type_line")).unwrap();
            }
            out.push('\n');
        }
        (out, names.len(), missing)
    }

    /// Rewrites `assets/oracle.tsv` under `CRAB_BLESS_ORACLE=1`; a normal
    /// run never touches the repo.
    #[test]
    fn bless_the_oracle_table() {
        if std::env::var_os("CRAB_BLESS_ORACLE").is_none() {
            return;
        }
        let (table, total, missing) = build();
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/oracle.tsv");
        std::fs::write(&path, &table).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
        eprintln!(
            "blessed {}: {} of {total} names, {} bytes; not in the cache: {missing:?}",
            path.display(),
            total - missing.len(),
            table.len(),
        );
    }

    #[test]
    fn a_plain_card_reads_as_printed() {
        let [bolt] = printed("Lightning Bolt").expect("Lightning Bolt") else { panic!("one face") };
        assert_eq!(bolt.name, "Lightning Bolt");
        assert_eq!(bolt.type_line, "Instant");
        assert_eq!(bolt.paragraphs().collect::<Vec<_>>(), ["Lightning Bolt deals 3 damage to any target."]);
    }

    /// Each side of a double-faced card is its own entry.
    #[test]
    fn a_double_faced_card_shows_the_side_it_is_on() {
        let [front] = printed("Delver of Secrets").expect("front") else { panic!("one face") };
        assert_eq!((front.type_line, front.stats), ("Creature — Human Wizard", "1/1"));
        let [back] = printed("Insectile Aberration").expect("back") else { panic!("one face") };
        assert_eq!((back.stats, back.paragraphs().collect::<Vec<_>>()), ("3/2", vec!["Flying"]));
    }

    /// An adventurer prints both halves on one side.
    #[test]
    fn an_adventure_card_shows_both_halves() {
        let faces = printed("Bonecrusher Giant").expect("Bonecrusher Giant");
        let names: Vec<_> = faces.iter().map(|f| (f.name, f.cost)).collect();
        assert_eq!(names, [("Bonecrusher Giant", "{2}{R}"), ("Stomp", "{1}{R}")]);
    }

    #[test]
    fn paragraphs_split_on_the_escaped_breaks() {
        let table = parse("Card\t\t{1}\tArtifact\t\tFirst.\\nSecond.\n");
        let face = table["Card"][0];
        assert_eq!(face.name, "Card");
        assert_eq!(face.paragraphs().collect::<Vec<_>>(), ["First.", "Second."]);
    }
}
