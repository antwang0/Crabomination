use crabomination::card::{CardId, CardInstance, CardType, CounterType, Keyword};
use crabomination::catalog;

#[test]
fn grizzly_bears_base_stats() {
    let c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    assert_eq!(c.power(), 2);
    assert_eq!(c.toughness(), 2);
}

#[test]
fn new_creature_has_summoning_sickness() {
    let c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    assert!(c.summoning_sick);
    assert!(!c.can_attack());
}

#[test]
fn haste_creature_can_attack_with_summoning_sickness() {
    let c = CardInstance::new(CardId(0), catalog::goblin_guide(), 0);
    assert!(c.summoning_sick);
    assert!(c.can_attack());
}

#[test]
fn tapped_creature_cannot_attack() {
    let mut c = CardInstance::new(CardId(0), catalog::goblin_guide(), 0);
    c.tapped = true;
    assert!(!c.can_attack());
}

#[test]
fn tapped_creature_cannot_block() {
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    c.summoning_sick = false;
    c.tapped = true;
    assert!(!c.can_block());
}

#[test]
fn creature_dies_at_lethal_damage() {
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    c.damage = 2;
    assert!(c.is_dead());
}

/// The definition memo is keyed to the definition, not to "some write reached
/// this card" (PERF `(-91)`): a tap or a damage point keeps it, and each of the
/// two definition writes drops it. The seven accessors' `debug_assert!`s only
/// audit this where assertions are compiled in; this asserts the *answer*, so
/// a write path that forgot to clear fails in any profile.
#[test]
fn definition_memo_outlives_a_plain_write_and_dies_with_the_definition() {
    use crabomination::mana::{Color, ColorSet};
    let (blue, green) = (ColorSet::single(Color::Blue), ColorSet::single(Color::Green));

    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    assert_eq!(c.printed_color_set(), green); // memoize
    c.tapped = true;
    c.damage = 1;
    c.add_counters(CounterType::PlusOnePlusOne, 1);
    assert_eq!(c.printed_color_set(), green);

    // in-place rewrite of a uniquely-owned definition — the pointer does not move
    std::sync::Arc::make_mut(c.definition_mut()).color_override = Some(vec![Color::Blue]);
    assert_eq!(c.printed_color_set(), blue);

    // wholesale replacement, and back
    let printed = c.definition.arc();
    let mut swapped = crabomination::card::CardDefinition::clone(&printed);
    swapped.color_override = Some(vec![Color::White]);
    c.set_definition(std::sync::Arc::new(swapped));
    assert_eq!(c.printed_color_set(), ColorSet::single(Color::White));
    c.set_definition(printed);
    assert_eq!(c.printed_color_set(), blue);
}

/// A memo travels with the definition it describes: assigning one card's
/// definition to another carries the memo, so the reader cannot see an answer
/// computed for a definition the card no longer has.
#[test]
fn definition_memo_travels_with_an_assigned_definition() {
    use crabomination::mana::{Color, ColorSet};
    let mut bear = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    let bolt = CardInstance::new(CardId(1), catalog::lightning_bolt(), 0);
    assert_eq!(bolt.printed_color_set(), ColorSet::single(Color::Red)); // memoize on bolt
    bear.definition = bolt.definition.clone();
    assert_eq!(bear.printed_color_set(), ColorSet::single(Color::Red));
}

#[test]
fn indestructible_creature_does_not_die_from_damage() {
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    std::sync::Arc::make_mut(c.definition_mut()).keywords.push(Keyword::Indestructible);
    c.damage = 99;
    assert!(!c.is_dead());
}

#[test]
fn pump_keeps_creature_alive_through_damage() {
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    c.damage = 2;
    c.toughness_bonus = 3; // now 5 toughness
    assert!(!c.is_dead());
}

#[test]
fn plus_one_counters_increase_stats() {
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    c.add_counters(CounterType::PlusOnePlusOne, 2);
    assert_eq!(c.power(), 4);
    assert_eq!(c.toughness(), 4);
}

#[test]
fn minus_one_counters_reduce_stats_and_can_kill() {
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    c.add_counters(CounterType::MinusOneMinusOne, 3);
    assert_eq!(c.toughness(), -1);
    assert!(c.is_dead());
}

#[test]
fn clear_end_of_turn_resets_bonuses() {
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    c.power_bonus = 3;
    c.toughness_bonus = 3;
    c.clear_end_of_turn_effects();
    assert_eq!(c.power(), 2);
    assert_eq!(c.toughness(), 2);
}

#[test]
fn serra_angel_has_flying_and_vigilance() {
    let c = CardInstance::new(CardId(0), catalog::serra_angel(), 0);
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Vigilance));
}

#[test]
fn land_is_not_creature() {
    let def = catalog::forest();
    assert!(def.is_land());
    assert!(!def.is_creature());
}

#[test]
fn cmc_checks() {
    assert_eq!(catalog::lightning_bolt().cost.cmc(), 1); // {R}
    assert_eq!(catalog::grizzly_bears().cost.cmc(), 2); // {1}{G}
    assert_eq!(catalog::serra_angel().cost.cmc(), 5); // {3}{W}{W}
}

#[test]
fn enchantment_creature_has_both_types() {
    let def = catalog::hopeful_eidolon();
    assert!(def.is_creature());
    assert!(def.card_types.contains(&CardType::Enchantment));
}

#[test]
fn flash_is_instant_speed() {
    let mut def = catalog::grizzly_bears();
    assert!(!def.is_instant_speed());
    def.keywords.push(Keyword::Flash);
    assert!(def.is_instant_speed());
}

#[test]
fn defender_cannot_attack() {
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    c.summoning_sick = false;
    std::sync::Arc::make_mut(c.definition_mut()).keywords.push(Keyword::Defender);
    assert!(!c.can_attack());
}

/// `ComputedPermanent`'s four characteristics are `Overlay`s whose projection
/// into `CardDefinition` is a **type**, not a stored `fn` pointer: 104 -> 72
/// bytes on a struct built 289,098 times and `Arc`-allocated 201,780 times a
/// six-game `cube` run. PERF's padding probe prices 8 bytes on it at `fixed`
/// +0.040 % / `cube` +0.058 %, so the four pointers were ~0.16-0.23 % on
/// width alone; the direct read was the rest of the measured -0.481 / -0.394
/// / -0.497 %.
#[test]
fn computed_permanent_carries_no_projection_pointers() {
    assert_eq!(std::mem::size_of::<crabomination::game::layers::ComputedPermanent>(), 72);
}
