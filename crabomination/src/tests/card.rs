use super::*;
use crate::catalog;

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

#[test]
fn indestructible_creature_does_not_die_from_damage() {
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    c.definition.keywords.push(Keyword::Indestructible);
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
    c.definition.keywords.push(Keyword::Defender);
    assert!(!c.can_attack());
}
