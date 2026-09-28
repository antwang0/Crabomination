//! Descriptor tokens: what a card *does*, as a bag of hashed words the table
//! net embeds (`crabomination_nn::table`, ML_NOTES "One net for every mode").
//!
//! The name embedding only knows cards the training data played — 23 of the
//! ~6,750 cards in the Commander precons, for the v5 champion. Everything
//! else reached the net as "unknown" plus a few dozen generic features. A
//! descriptor gives every card in the catalog an identity built from its
//! definition, so a card never seen in training still reads as "a flier
//! with a damage trigger" rather than as nothing.
//!
//! **The words come from the definition's `Debug` text**, which is the one
//! description every one of the catalog's cards has without a hand-written
//! extractor per effect kind. A lexer walks it and keeps:
//!
//! * every capitalised identifier — variant and struct names (`DealDamage`,
//!   `Flying`, `Creature`, `Opponent`) — bare, and qualified by the field it
//!   sits under (`effect.DealDamage`, `card_types.Creature` as opposed to a
//!   targeting filter's `Creature`);
//! * small integers under their field, bucketed (`amount=3`, `power=11-20`);
//! * `true` under its field (`loyalty_twice_each_turn=true`).
//!
//! String literals (the card's own name, reminder text) are skipped, and
//! what every card carries is dropped: `None` (dozens of unset optional
//! fields), a zero under any field (`hand_modifier=0`, `defense=0`), and the
//! wrapper structs every definition prints (`CardDefinition`, `ManaCost`,
//! `Subtypes`). Those were ~a quarter of a vanilla creature's words and said
//! nothing about it.
//! Tokens are FNV-1a hashed into `1..DESC_BUCKETS` and deduplicated, so a
//! descriptor is a sorted set.
//!
//! ⚠ **The tokens follow the engine's type names.** Renaming an `Effect`
//! variant moves its token to a different bucket — a trained net then reads
//! that word as an untrained row, a quiet degradation rather than a wrong
//! answer. Worth knowing before a sweeping rename lands between a training
//! run and its gate.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::sync::Arc;

use crabomination_nn::table::DESC_BUCKETS;

use crate::card::{CardDefinition, CardInstance};
use crate::fxhash::HashMap;

/// FNV-1a over the token's bytes, into `1..DESC_BUCKETS`. Stable across
/// builds and platforms by construction — the bucket is what a trained net
/// learned, so a hash that moved would retire every net.
fn bucket(word: &str) -> u16 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in word.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    1 + (h % (DESC_BUCKETS as u64 - 1)) as u16
}

/// Integers are bucketed: exact to ten, then two coarse ranges. "Deal 3"
/// and "deal 4" are different cards; "deal 14" and "deal 17" barely are.
fn number_word(n: i64) -> &'static str {
    const EXACT: [&str; 11] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];
    match n {
        i64::MIN..=-1 => "neg",
        0..=10 => EXACT[n as usize],
        11..=20 => "11-20",
        _ => "21+",
    }
}

/// The descriptor of one definition: the sorted, deduplicated buckets of the
/// words the module doc lists.
pub fn definition_tokens(def: &CardDefinition) -> Vec<u16> {
    let text = format!("{def:?}");
    let b = text.as_bytes();
    let mut out: BTreeSet<u16> = BTreeSet::new();
    // The field whose value the lexer is inside, and the one it was inside
    // at each open bracket.
    let mut field: &str = "";
    let mut stack: Vec<&str> = Vec::new();
    let mut word = String::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        match c {
            b'"' => {
                // A string literal, escapes and all.
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    i += if b[i] == b'\\' { 2 } else { 1 };
                }
                i += 1;
            }
            b'(' | b'[' | b'{' => {
                stack.push(field);
                i += 1;
            }
            b')' | b']' | b'}' => {
                field = stack.pop().unwrap_or("");
                i += 1;
            }
            b'-' | b'0'..=b'9' => {
                let neg = c == b'-';
                let start = if neg { i + 1 } else { i };
                let mut j = start;
                while j < b.len() && b[j].is_ascii_digit() {
                    j += 1;
                }
                if j == start {
                    // A bare '-' (an arrow, a range): not a number.
                    i += 1;
                    continue;
                }
                // `0x55d1..` (a pointer) or `3.5`: the run is not one integer.
                let glued =
                    j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_' || b[j] == b'.');
                let mut end = j;
                while end < b.len()
                    && (b[end].is_ascii_alphanumeric() || b[end] == b'_' || b[end] == b'.')
                {
                    end += 1;
                }
                let n: i64 = text[start..j].parse().unwrap_or(i64::MAX);
                let n = if neg { -n } else { n };
                if !glued && !field.is_empty() && n != 0 {
                    word.clear();
                    word.push_str(field);
                    word.push('=');
                    word.push_str(number_word(n));
                    out.insert(bucket(&word));
                }
                i = end;
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                let ident = &text[start..i];
                let mut j = i;
                while j < b.len() && b[j] == b' ' {
                    j += 1;
                }
                let colon = j < b.len() && b[j] == b':' && b.get(j + 1) != Some(&b':');
                if c.is_ascii_lowercase() || c == b'_' {
                    if colon {
                        field = ident;
                    } else if ident == "true" && !field.is_empty() {
                        word.clear();
                        word.push_str(field);
                        word.push_str("=true");
                        out.insert(bucket(&word));
                    }
                } else if !matches!(ident, "None" | "CardDefinition" | "ManaCost" | "Subtypes") {
                    out.insert(bucket(ident));
                    if !field.is_empty() {
                        word.clear();
                        word.push_str(field);
                        word.push('.');
                        word.push_str(ident);
                        out.insert(bucket(&word));
                    }
                }
            }
            _ => i += 1,
        }
    }
    out.into_iter().collect()
}

/// What identifies a definition for the memo: the name (by address and
/// length — the same card presents the same literal, and a shared address
/// with a different length is a different string), printed P/T and the
/// ability counts.
///
/// Two *different* definitions share a key only when they have the same
/// name, stats and ability counts — in practice two tokens named alike
/// ("Soldier") from different makers. The first one seen on a thread then
/// describes both. That is the one way a descriptor can depend on history,
/// and it is a token's description, not its name or its features.
#[derive(Hash, PartialEq, Eq)]
struct Key {
    name_ptr: usize,
    name_len: usize,
    pt: (i32, i32),
    counts: [u16; 5],
}

fn key(def: &CardDefinition) -> Key {
    Key {
        name_ptr: def.name.as_ptr() as usize,
        name_len: def.name.len(),
        pt: (def.power, def.toughness),
        counts: [
            def.keywords.len() as u16,
            def.static_abilities.len() as u16,
            def.activated_abilities.len() as u16,
            def.triggered_abilities.len() as u16,
            def.loyalty_abilities.len() as u16,
        ],
    }
}

thread_local! {
    /// Per thread, so actors never contend; a card's `Debug` walk runs once
    /// per thread, not once per encode.
    static MEMO: RefCell<HashMap<Key, Arc<[u16]>>> = RefCell::new(HashMap::default());
}

/// This object's descriptor, memoized — see [`Key`] for what "the same
/// definition" means here.
pub fn tokens_of(c: &CardInstance) -> Arc<[u16]> {
    let def: &CardDefinition = &c.definition;
    let k = key(def);
    if let Some(t) = MEMO.with(|m| m.borrow().get(&k).cloned()) {
        return t;
    }
    let t: Arc<[u16]> = definition_tokens(def).into();
    MEMO.with(|m| m.borrow_mut().insert(k, t.clone()));
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    fn jaccard(a: &[u16], b: &[u16]) -> f64 {
        let a: BTreeSet<u16> = a.iter().copied().collect();
        let b: BTreeSet<u16> = b.iter().copied().collect();
        a.intersection(&b).count() as f64 / a.union(&b).count() as f64
    }

    /// A descriptor is a pure function of the definition: sorted, unique,
    /// inside the bucket range, and the same on every call.
    #[test]
    fn descriptors_are_deterministic_sorted_sets() {
        for f in [catalog::lightning_bolt, catalog::serra_angel, catalog::llanowar_elves] {
            let def = f();
            let t = definition_tokens(&def);
            assert!(!t.is_empty(), "{} has no tokens", def.name);
            assert!(t.windows(2).all(|w| w[0] < w[1]), "{}: not a sorted set", def.name);
            assert!(t.iter().all(|&x| x >= 1 && (x as usize) < DESC_BUCKETS));
            assert_eq!(t, definition_tokens(&f()), "{}: two walks disagree", def.name);
        }
    }

    /// The point of the descriptor: similar cards share most of their words
    /// and different ones don't. Two burn spells are closer to each other
    /// than either is to a vanilla creature, and the flier carries its
    /// keyword under the keyword field.
    #[test]
    fn similar_cards_share_words() {
        let bolt = definition_tokens(&catalog::lightning_bolt());
        let shock = definition_tokens(&catalog::shock());
        let bears = definition_tokens(&catalog::grizzly_bears());
        let angel = definition_tokens(&catalog::serra_angel());
        assert!(
            jaccard(&bolt, &shock) > jaccard(&bolt, &bears),
            "bolt~shock {:.2} vs bolt~bears {:.2}",
            jaccard(&bolt, &shock),
            jaccard(&bolt, &bears)
        );
        assert!(angel.contains(&bucket("keywords.Flying")));
        assert!(!bears.contains(&bucket("keywords.Flying")));
        assert!(bears.contains(&bucket("card_types.Creature")));
        assert!(bolt.contains(&bucket("card_types.Instant")));
        // The card's own name is a string literal and never becomes a word.
        assert!(!bolt.contains(&bucket("Lightning")));
    }

    #[test]
    fn numbers_bucket_and_pointers_do_not_count() {
        assert_eq!(number_word(3), "3");
        assert_eq!(number_word(14), "11-20");
        assert_eq!(number_word(99), "21+");
        assert_eq!(number_word(-2), "neg");
        let bears = definition_tokens(&catalog::grizzly_bears());
        assert!(bears.contains(&bucket("power=2")) && bears.contains(&bucket("toughness=2")));
        // Zeroes and the wrappers every definition prints carry nothing.
        assert!(
            !bears.contains(&bucket("defense=0")) && !bears.contains(&bucket("CardDefinition"))
        );
    }
}
