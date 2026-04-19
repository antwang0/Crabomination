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
