use super::*;

#[test]
fn cmc_sums_pip_values() {
    assert_eq!(cost(&[r()]).cmc(), 1, "Lightning Bolt {{R}}");
    assert_eq!(cost(&[generic(3), w(), w()]).cmc(), 5, "Serra Angel {{3}}{{W}}{{W}}");
    assert_eq!(ManaCost::default().cmc(), 0, "free spell");
}

#[test]
fn pay_drains_matching_color() {
    let mut pool = ManaPool::new();
    pool.add(Color::Red, 1);
    pool.pay(&cost(&[r()])).unwrap();
    assert_eq!(pool.amount(Color::Red), 0);
}

#[test]
fn pay_drains_any_color_for_generic_pips() {
    let mut pool = ManaPool::new();
    pool.add(Color::Green, 2);
    pool.pay(&cost(&[generic(1), g()])).unwrap();
    assert_eq!(pool.total(), 0);
}

#[test]
fn pay_fails_atomically_when_short_on_color() {
    // {W}{U} against a WW pool: must fail without consuming any white.
    let mut pool = ManaPool::new();
    pool.add(Color::White, 2);
    assert!(pool.pay(&cost(&[w(), u()])).is_err());
    assert_eq!(pool.amount(Color::White), 2);
}

#[test]
fn pay_fails_atomically_when_short_on_generic() {
    let mut pool = ManaPool::new();
    pool.add(Color::Red, 1);
    assert!(pool.pay(&cost(&[generic(3)])).is_err());
    assert_eq!(pool.amount(Color::Red), 1);
}

#[test]
fn pay_fails_on_wrong_color() {
    let mut pool = ManaPool::new();
    pool.add(Color::Green, 1);
    assert!(pool.pay(&cost(&[r()])).is_err());
    assert_eq!(pool.amount(Color::Green), 1);
}

#[test]
fn empty_drains_all_mana() {
    let mut pool = ManaPool::new();
    pool.add(Color::Red, 2);
    pool.add(Color::Green, 3);
    pool.empty();
    assert_eq!(pool.total(), 0);
}

#[test]
fn pay_clamps_snow_counter_after_draining_non_snow_buckets() {
    // Regression: `ManaPool.snow` used to drift above `total()` because
    // non-{S} payments drained the colored buckets without decrementing
    // the snow tally. A subsequent {S} payment then succeeded against
    // non-snow mana.
    let mut pool = ManaPool::new();
    pool.add_snow(Color::Red, 1);    // red=1, snow=1
    pool.pay(&cost(&[r()])).unwrap(); // red=0, snow should clamp to 0
    assert_eq!(pool.snow_amount(), 0,
        "snow counter must not exceed total mana after a non-snow payment");

    // Same check via spend_generic.
    let mut pool = ManaPool::new();
    pool.add_snow(Color::Red, 2);
    pool.spend_generic(2);
    assert_eq!(pool.snow_amount(), 0);
}

#[test]
fn color_short_name_and_display_render_wubrg() {
    assert_eq!(Color::White.short_name(), 'W');
    assert_eq!(Color::Blue.short_name(),  'U');
    assert_eq!(Color::Black.short_name(), 'B');
    assert_eq!(Color::Red.short_name(),   'R');
    assert_eq!(Color::Green.short_name(), 'G');
    // Display matches short_name and inlines into format!().
    assert_eq!(format!("{{{}}}", Color::Red), "{R}");
}

#[test]
fn distinct_colors_counts_unique_colored_pips() {
    use crate::mana::{hybrid, phyrexian, snow_mana, x};
    assert_eq!(cost(&[r()]).distinct_colors(), 1, "mono");
    assert_eq!(cost(&[r(), w()]).distinct_colors(), 2, "two colors");
    assert_eq!(cost(&[w(), w(), b(), r()]).distinct_colors(), 3, "duplicates collapse");
    assert_eq!(cost(&[generic(3)]).distinct_colors(), 0, "generic doesn't count");
    assert_eq!(ManaCost::default().distinct_colors(), 0, "free spell");
    // Hybrid contributes both halves; Phyrexian contributes its colored half.
    assert_eq!(cost(&[hybrid(Color::White, Color::Black)]).distinct_colors(), 2);
    assert_eq!(cost(&[phyrexian(Color::Black)]).distinct_colors(), 1);
    // X, Snow, Colorless pips don't count.
    assert_eq!(cost(&[x(), x(), u()]).distinct_colors(), 1);
    assert_eq!(cost(&[snow_mana()]).distinct_colors(), 0);
    assert_eq!(cost(&[crate::mana::colorless(2)]).distinct_colors(), 0);
}

// ── Cost reduction (CR 601.2f / 117.7c) ─────────────────────────────────────

#[test]
fn reduce_generic_drains_a_single_generic_pip() {
    // {2}{R}{R} → reduce_generic(2) → {R}{R}.
    let mut c = cost(&[generic(2), r(), r()]);
    let applied = c.reduce_generic(2);
    assert_eq!(applied, 2);
    assert_eq!(c.cmc(), 2);
}

#[test]
fn reduce_generic_clamps_at_available_pips() {
    // {1}{B}{B} with reduce(5): only 1 generic exists; clamp at 1.
    let mut c = cost(&[generic(1), b(), b()]);
    assert_eq!(c.reduce_generic(5), 1);
    assert_eq!(c.cmc(), 2, "{{1}}{{B}}{{B}} → {{B}}{{B}}");
}

#[test]
fn reduce_generic_splits_multiple_generic_pips() {
    // {2}{1}{G}: drain {2}, leave {1} intact.
    let mut c = cost(&[generic(2), generic(1), g()]);
    assert_eq!(c.reduce_generic(2), 2);
    assert_eq!(c.cmc(), 2, "{{2}}{{1}}{{G}} → {{1}}{{G}}");
}

#[test]
fn reduce_generic_preserves_colored_colorless_and_x_pips() {
    // CR 601.2f / 117.7c — only Generic pips are reducible.
    use crate::mana::{colorless, x};
    let mut c = cost(&[w(), r(), colorless(1), x()]);
    let before = c.symbols.clone();
    assert_eq!(c.reduce_generic(99), 0);
    assert_eq!(c.symbols, before, "colored / {{C}} / X pips preserved");
}

// ── CR 105 — Colors: ColorSet predicate helpers ─────────────────────────────

#[test]
fn color_set_predicates_match_cr_105_2() {
    use crate::mana::ColorSet;

    let mono = ColorSet::single(Color::White);
    assert!(mono.is_monocolored());
    assert!(!mono.is_multicolored());
    assert!(!mono.is_colorless());

    let mut multi = ColorSet::empty();
    multi.insert(Color::Red);
    multi.insert(Color::Green);
    assert!(multi.is_multicolored());
    assert!(!multi.is_monocolored());

    let colorless = ColorSet::empty();
    assert!(colorless.is_colorless());
    assert!(colorless.is_empty());

    let all = ColorSet::all();
    assert_eq!(all.len(), 5);
    assert!(all.is_multicolored());
}

// ── ManaCost::summary() printed-Oracle rendering ────────────────────────────

#[test]
fn mana_cost_summary_renders_each_pip_kind() {
    use crate::mana::{colorless, hybrid, phyrexian, snow_mana, x, ManaCost};
    assert_eq!(cost(&[generic(2), w(), b()]).summary(), "{2}{W}{B}");
    assert_eq!(cost(&[x(), x(), r()]).summary(), "{X}{X}{R}");
    assert_eq!(cost(&[hybrid(Color::White, Color::Black)]).summary(), "{W/B}");
    assert_eq!(cost(&[phyrexian(Color::Black)]).summary(), "{B/P}");
    assert_eq!(cost(&[snow_mana(), u()]).summary(), "{S}{U}");
    assert_eq!(cost(&[generic(1), colorless(2)]).summary(), "{1}{C}{C}");
    assert_eq!(ManaCost::new(vec![]).summary(), "{0}", "free spell renders as {{0}}");
}

// ── Monocolored hybrid pips ({n/C}) — CR 202.3f / CR 107.4f ─────────────────

#[test]
fn mono_hybrid_mana_value_uses_generic_side() {
    use crate::mana::mono_hybrid;
    // CR 202.3f: a monocolored hybrid pip {2/R} contributes 2 to mana value.
    assert_eq!(cost(&[mono_hybrid(2, Color::Red)]).cmc(), 2);
    assert_eq!(
        cost(&[mono_hybrid(2, Color::Red), mono_hybrid(2, Color::Red), mono_hybrid(2, Color::Red)]).cmc(),
        6,
        "Magmablood Archaic mono-hybrid cost has mana value 6",
    );
}

#[test]
fn mono_hybrid_summary_renders_n_over_color() {
    use crate::mana::mono_hybrid;
    assert_eq!(cost(&[mono_hybrid(2, Color::Red)]).summary(), "{2/R}");
    assert_eq!(cost(&[mono_hybrid(2, Color::Green)]).summary(), "{2/G}");
}

#[test]
fn mono_hybrid_pays_colored_side_when_available() {
    use crate::mana::mono_hybrid;
    let mut pool = ManaPool::new();
    pool.add(Color::Red, 1);
    // One red pays the colored side of {2/R} (1 mana, not 2).
    assert!(pool.pay(&cost(&[mono_hybrid(2, Color::Red)])).is_ok());
    assert_eq!(pool.total(), 0, "exactly the one red was spent");
}

#[test]
fn mono_hybrid_pays_generic_side_when_no_color() {
    use crate::mana::mono_hybrid;
    let mut pool = ManaPool::new();
    pool.add_colorless(2);
    // No red — pay {2} generic for the {2/R} pip.
    assert!(pool.pay(&cost(&[mono_hybrid(2, Color::Red)])).is_ok());
    assert_eq!(pool.total(), 0);
}

#[test]
fn mono_hybrid_one_generic_is_insufficient() {
    use crate::mana::mono_hybrid;
    let mut pool = ManaPool::new();
    pool.add_colorless(1);
    // One colorless can't pay {2/R} (needs {2} or one red).
    assert!(pool.pay(&cost(&[mono_hybrid(2, Color::Red)])).is_err());
    assert_eq!(pool.total(), 1, "pool unchanged on failed payment");
}

#[test]
fn mono_hybrid_contributes_color_identity() {
    use crate::mana::mono_hybrid;
    assert_eq!(cost(&[mono_hybrid(2, Color::Red)]).colors(), vec![Color::Red]);
    assert_eq!(cost(&[mono_hybrid(2, Color::Green)]).distinct_colors(), 1);
}
