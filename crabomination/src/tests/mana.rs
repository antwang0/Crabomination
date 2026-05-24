use super::*;

#[test]
fn cmc_of_lightning_bolt_is_1() {
    let c = cost(&[r()]);
    assert_eq!(c.cmc(), 1);
}

#[test]
fn cmc_of_serra_angel_is_5() {
    let c = cost(&[generic(3), w(), w()]);
    assert_eq!(c.cmc(), 5);
}

#[test]
fn cmc_of_free_card_is_0() {
    assert_eq!(ManaCost::default().cmc(), 0);
}

#[test]
fn pay_exact_colored_cost() {
    let mut pool = ManaPool::new();
    pool.add(Color::Red, 1);
    assert!(pool.pay(&cost(&[r()])).is_ok());
    assert_eq!(pool.amount(Color::Red), 0);
}

#[test]
fn pay_fails_wrong_color() {
    let mut pool = ManaPool::new();
    pool.add(Color::Green, 1);
    let err = pool.pay(&cost(&[r()]));
    assert!(err.is_err());
    // Pool must be unchanged
    assert_eq!(pool.amount(Color::Green), 1);
}

#[test]
fn pay_generic_with_any_mana() {
    let mut pool = ManaPool::new();
    pool.add(Color::Green, 2);
    assert!(pool.pay(&cost(&[generic(2)])).is_ok());
    assert_eq!(pool.total(), 0);
}

#[test]
fn pay_mixed_cost_grizzly_bears() {
    // {1}{G} with GG in pool
    let mut pool = ManaPool::new();
    pool.add(Color::Green, 2);
    assert!(pool.pay(&cost(&[generic(1), g()])).is_ok());
    assert_eq!(pool.total(), 0);
}

#[test]
fn pay_fails_not_enough_generic() {
    let mut pool = ManaPool::new();
    pool.add(Color::Red, 1);
    let err = pool.pay(&cost(&[generic(3)]));
    assert!(err.is_err());
    assert_eq!(pool.amount(Color::Red), 1); // unchanged
}

#[test]
fn pay_dark_ritual_cost() {
    // Pay {B}, then manually add BBB to pool
    let mut pool = ManaPool::new();
    pool.add(Color::Black, 1);
    pool.pay(&cost(&[b()])).unwrap();
    pool.add(Color::Black, 3); // Dark Ritual's effect
    assert_eq!(pool.amount(Color::Black), 3);
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
fn pay_does_not_partially_drain_on_failure() {
    // {W}{U} cost, pool has WW but no U — colored check must fail atomically
    let mut pool = ManaPool::new();
    pool.add(Color::White, 2);
    let err = pool.pay(&cost(&[w(), u()]));
    assert!(err.is_err());
    assert_eq!(pool.amount(Color::White), 2); // still intact
}

#[test]
fn color_short_name_matches_mtg_abbreviations() {
    assert_eq!(Color::White.short_name(), 'W');
    assert_eq!(Color::Blue.short_name(),  'U');
    assert_eq!(Color::Black.short_name(), 'B');
    assert_eq!(Color::Red.short_name(),   'R');
    assert_eq!(Color::Green.short_name(), 'G');
}

#[test]
fn color_display_matches_short_name() {
    assert_eq!(format!("{}", Color::Blue), "U");
    assert_eq!(format!("{{{}}}", Color::Red), "{R}",
        "Color::Display can be inlined into format!() to render mana pips");
}

#[test]
fn distinct_colors_counts_colored_pips() {
    // Mono-color: 1.
    assert_eq!(cost(&[r()]).distinct_colors(), 1);
    // Multicolored ({R}{W}): 2.
    assert_eq!(cost(&[r(), w()]).distinct_colors(), 2);
    // Generic only: 0.
    assert_eq!(cost(&[generic(3)]).distinct_colors(), 0);
    // Three colors with duplicates ({W}{W}{B}{R}): 3 (W, B, R distinct).
    assert_eq!(cost(&[w(), w(), b(), r()]).distinct_colors(), 3);
    // Free spell: 0.
    assert_eq!(ManaCost::default().distinct_colors(), 0);
}

#[test]
fn distinct_colors_handles_hybrid_and_phyrexian() {
    use crate::mana::{hybrid, phyrexian};
    // Hybrid {W/B} contributes both halves.
    assert_eq!(cost(&[hybrid(Color::White, Color::Black)]).distinct_colors(), 2);
    // Phyrexian {B/P} contributes its colored side.
    assert_eq!(cost(&[phyrexian(Color::Black)]).distinct_colors(), 1);
    // Colorless artifacts ({2}) — 0 distinct colors.
    assert_eq!(cost(&[generic(2)]).distinct_colors(), 0);
}

#[test]
fn distinct_colors_does_not_count_x_or_snow_or_colorless() {
    use crate::mana::{snow_mana, x};
    assert_eq!(cost(&[x(), x(), u()]).distinct_colors(), 1);
    assert_eq!(cost(&[snow_mana()]).distinct_colors(), 0);
    assert_eq!(
        cost(&[crate::mana::colorless(2)]).distinct_colors(),
        0,
        "Colorless pips are not colored"
    );
}

// ── Cost reduction (CR 601.2f / 117.7c) ─────────────────────────────────────

#[test]
fn reduce_generic_drains_a_single_generic_pip() {
    // {2}{R}{R} → reduce_generic(2) → {R}{R} (2 generic + 0 left).
    let mut c = cost(&[generic(2), r(), r()]);
    let applied = c.reduce_generic(2);
    assert_eq!(applied, 2, "all 2 of the requested reduction applied");
    assert_eq!(c.cmc(), 2, "{{2}}{{R}}{{R}} → {{R}}{{R}} (CMC 4 → 2)");
    assert_eq!(c.distinct_colors(), 1, "{{R}}{{R}} is still 1 distinct color");
}

#[test]
fn reduce_generic_clamps_to_zero_when_request_exceeds_generic() {
    // {1}{B}{B} → reduce_generic(5): only 1 generic exists; clamp at 1.
    let mut c = cost(&[generic(1), b(), b()]);
    let applied = c.reduce_generic(5);
    assert_eq!(applied, 1, "only the available 1 generic pip drained");
    assert_eq!(c.cmc(), 2, "{{1}}{{B}}{{B}} → {{B}}{{B}}");
}

#[test]
fn reduce_generic_does_not_touch_colored_or_colorless_pips() {
    // {W}{R}{C}: no generic pips → reduce_generic(99) → unchanged.
    use crate::mana::colorless;
    let mut c = cost(&[w(), r(), colorless(1)]);
    let cmc_before = c.cmc();
    let applied = c.reduce_generic(99);
    assert_eq!(applied, 0, "no generic pips → no reduction");
    assert_eq!(c.cmc(), cmc_before, "colored / {{C}} pips preserved (CR 117.7c)");
}

#[test]
fn reduce_generic_splits_multiple_generic_pips() {
    // {2}{1}{G}: two Generic pips totaling 3. reduce_generic(2) should
    // drain the first ({2} → {0}) and leave the second ({1}) intact.
    let mut c = cost(&[generic(2), generic(1), g()]);
    let applied = c.reduce_generic(2);
    assert_eq!(applied, 2);
    assert_eq!(c.cmc(), 2, "{{2}}{{1}}{{G}} → {{1}}{{G}}");
}

#[test]
fn reduce_generic_does_not_touch_x_pips() {
    // {X}{R}{R}: reduce_generic(99) → unchanged (CR 601.2f / 117.7c —
    // X is determined when the spell is cast, not a generic).
    use crate::mana::x;
    let mut c = cost(&[x(), r(), r()]);
    let symbols_before = c.symbols.clone();
    let applied = c.reduce_generic(99);
    assert_eq!(applied, 0);
    assert_eq!(c.symbols, symbols_before, "X pips preserved");
}

// ── CR 105 — Colors: ColorSet predicate helpers ─────────────────────────────

#[test]
fn color_set_is_monocolored_for_single_color_per_cr_105_2a() {
    use crate::mana::ColorSet;
    let mut s = ColorSet::empty();
    s.insert(Color::White);
    assert!(s.is_monocolored());
    assert!(!s.is_multicolored());
    assert!(!s.is_colorless());
}

#[test]
fn color_set_is_multicolored_for_two_or_more_colors_per_cr_105_2b() {
    use crate::mana::ColorSet;
    let mut s = ColorSet::empty();
    s.insert(Color::Red);
    s.insert(Color::Green);
    assert!(!s.is_monocolored());
    assert!(s.is_multicolored());
    assert!(!s.is_colorless());
}

#[test]
fn color_set_is_colorless_for_empty_per_cr_105_2c() {
    use crate::mana::ColorSet;
    let s = ColorSet::empty();
    assert!(!s.is_monocolored());
    assert!(!s.is_multicolored());
    assert!(s.is_colorless());
    assert!(s.is_empty()); // synonyms
}

#[test]
fn color_set_all_five_is_multicolored() {
    use crate::mana::ColorSet;
    let s = ColorSet::all();
    assert!(!s.is_monocolored());
    assert!(s.is_multicolored());
    assert!(!s.is_colorless());
    assert_eq!(s.len(), 5);
}

// ── ManaCost::summary() printed-Oracle rendering ────────────────────────────

#[test]
fn mana_cost_summary_renders_two_white_one_black() {
    use crate::mana::{b, cost, generic, w};
    let c = cost(&[generic(2), w(), b()]);
    assert_eq!(c.summary(), "{2}{W}{B}");
}

#[test]
fn mana_cost_summary_renders_x_pip() {
    use crate::mana::{cost, r, x};
    let c = cost(&[x(), x(), r()]);
    assert_eq!(c.summary(), "{X}{X}{R}");
}

#[test]
fn mana_cost_summary_renders_hybrid_pip() {
    use crate::mana::{cost, hybrid, Color};
    let c = cost(&[hybrid(Color::White, Color::Black)]);
    assert_eq!(c.summary(), "{W/B}");
}

#[test]
fn mana_cost_summary_renders_phyrexian_pip() {
    use crate::mana::{cost, phyrexian, Color};
    let c = cost(&[phyrexian(Color::Black)]);
    assert_eq!(c.summary(), "{B/P}");
}

#[test]
fn mana_cost_summary_renders_snow_pip() {
    use crate::mana::{cost, snow_mana, u};
    let c = cost(&[snow_mana(), u()]);
    assert_eq!(c.summary(), "{S}{U}");
}

#[test]
fn mana_cost_summary_renders_zero_for_empty_cost() {
    use crate::mana::ManaCost;
    let c = ManaCost::new(vec![]);
    assert_eq!(c.summary(), "{0}");
}

#[test]
fn mana_cost_summary_renders_colorless_pips() {
    use crate::mana::{colorless, cost, generic};
    let c = cost(&[generic(1), colorless(2)]);
    assert_eq!(c.summary(), "{1}{C}{C}");
}
