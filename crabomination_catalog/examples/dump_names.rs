//! Dump every registered card name (one per line). Diff against a Scryfall
//! `set:<code>` name list to find unimplemented cards. Pass a set code as the
//! first arg to restrict to that catalog slice.
fn main() {
    let only = std::env::args().nth(1);
    for (set, fs) in crabomination_catalog::sets::all_factories::per_set_card_factories() {
        if only.as_deref().is_some_and(|s| s != *set) {
            continue;
        }
        for f in *fs {
            println!("{}", f().name);
        }
    }
}
