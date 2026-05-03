use super::*;

#[test]
fn cmc_calculations() {
    assert_eq!(cost(&[r()]).cmc(), 1);               // {R}
    assert_eq!(cost(&[generic(3), w(), w()]).cmc(), 5); // {3}{W}{W}
    assert_eq!(ManaCost::default().cmc(), 0);         // free spell
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
fn color_short_name_and_display() {
    let cases = [
        (Color::White, 'W'),
        (Color::Blue,  'U'),
        (Color::Black, 'B'),
        (Color::Red,   'R'),
        (Color::Green, 'G'),
    ];
    for (color, letter) in cases {
        assert_eq!(color.short_name(), letter);
        assert_eq!(format!("{color}"), letter.to_string());
    }
    assert_eq!(format!("{{{}}}", Color::Red), "{R}",
        "Color::Display inlines into format! to render mana pips");
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
