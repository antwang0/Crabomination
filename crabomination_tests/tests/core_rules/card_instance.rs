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

/// CR 105.2c — a card whose mana cost carries no coloured pip takes its colour
/// from a colour indicator, and `printed_color_set` is the only place the
/// engine reads one. Three shipped cards had none (`audit_printed_body`'s
/// colour column, 2026-09-12): a `{0}` Kobold and two cards with no mana cost
/// at all, each colorless to `R::HasColor`, protection- and
/// hexproof-from-colour, devotion and every "shares a colour" read.
#[test]
fn a_colour_indicator_is_the_printed_colour_when_the_cost_has_no_pip() {
    use crabomination::mana::{Color, ColorSet, ManaSymbol};
    let set = |cs: &[Color]| {
        let mut s = ColorSet::empty();
        for c in cs {
            s.insert(*c);
        }
        s
    };
    for (def, want) in [
        (catalog::rograkh_son_of_rohgahh(), set(&[Color::Red])),
        (catalog::evermind(), set(&[Color::Blue])),
        (
            catalog::ragnarok_divine_deliverance(),
            set(&[Color::Black, Color::Green]),
        ),
    ] {
        let name = def.name;
        assert!(
            !def.cost
                .symbols
                .iter()
                .any(|s| matches!(s, ManaSymbol::Colored(_))),
            "{name} prints a coloured pip, so this is not the indicator case",
        );
        let c = CardInstance::new(CardId(0), def, 0);
        assert_eq!(c.printed_color_set(), want, "{name}");
    }
}

/// `combat_keywords` is a one-pass restatement of `has_keyword` for eleven
/// unit keywords (PERF `(-330)`); the two must agree over every source a
/// keyword can come from — printed, EOT-granted, keyword counter (CR 122.1b)
/// — and over both removal lists, which beat all three.
#[test]
fn combat_keywords_agrees_with_has_keyword() {
    use crabomination::card::combat_kw;
    let base = || CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    let check = |c: &CardInstance, what: &str| {
        let m = c.combat_keywords();
        for (bit, kw) in combat_kw::ALL {
            assert_eq!(
                m & bit != 0,
                c.has_keyword(&kw),
                "{what}: {kw:?} disagrees (mask {m:#x})"
            );
        }
    };
    check(&base(), "vanilla");
    for (_, kw) in combat_kw::ALL {
        // Printed.
        let mut c = base();
        std::sync::Arc::make_mut(c.definition_mut()).keywords.push(kw.clone());
        check(&c, "printed");
        // Printed, then stripped for the turn and permanently.
        let mut d = c.clone();
        d.removed_keywords_eot.push(kw.clone());
        check(&d, "printed + removed_eot");
        let mut d = c.clone();
        d.removed_keywords.push(kw.clone());
        check(&d, "printed + removed");
        // Granted until end of turn.
        let mut c = base();
        c.granted_keywords_eot.push(kw.clone());
        check(&c, "granted_eot");
        let mut d = c.clone();
        d.removed_keywords_eot.push(kw.clone());
        check(&d, "granted_eot + removed_eot");
        // CR 122.1b keyword counter.
        let mut c = base();
        c.keyword_counters.add(kw.clone(), 1);
        check(&c, "counter");
        let mut d = c.clone();
        d.removed_keywords.push(kw.clone());
        check(&d, "counter + removed");
    }
    // A keyword outside the family sets no bit, and a removal of one strips
    // nothing.
    let mut c = base();
    std::sync::Arc::make_mut(c.definition_mut()).keywords.push(Keyword::Vigilance);
    std::sync::Arc::make_mut(c.definition_mut()).keywords.push(Keyword::Flying);
    c.removed_keywords_eot.push(Keyword::Vigilance);
    check(&c, "off-family");
    assert_eq!(c.combat_keywords(), combat_kw::FLYING);
}

/// `KeywordCounters` stores no zero-valued entry — `add` and `remove_up_to`
/// drop one that reaches zero, and `insert(kw, 0)` removes rather than
/// stores. `has_tag`'s soundness as a negative test and
/// `combat_keywords`' one-pass read both rest on it.
#[test]
fn a_zero_keyword_counter_is_no_counter() {
    use crabomination::card::combat_kw;
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    c.keyword_counters.insert(Keyword::Flying, 0);
    assert!(c.keyword_counters.is_empty(), "insert(_, 0) stored an entry");
    assert!(!c.has_keyword(&Keyword::Flying));
    assert_eq!(c.combat_keywords() & combat_kw::FLYING, 0);
    c.keyword_counters.insert(Keyword::Flying, 2);
    assert!(c.has_keyword(&Keyword::Flying));
    c.keyword_counters.insert(Keyword::Flying, 0);
    assert!(c.keyword_counters.is_empty(), "insert(_, 0) kept a stale entry");
    assert_eq!(c.combat_keywords() & combat_kw::FLYING, 0);
    // `remove_up_to` to exactly zero is the same rule from the other side.
    c.keyword_counters.add(Keyword::Trample, 1);
    c.keyword_counters.remove_up_to(&Keyword::Trample, 1);
    assert!(c.keyword_counters.is_empty());
}

/// `has_keyword_tag` is `has_keyword` with the value ignored, so it must see
/// the same four sources and honour the same removal precedence. Before it
/// existed each value-agnostic caller hand-rolled the scan over a subset:
/// `has_toxic`/`has_modular` read printed + `granted_keywords_eot` and
/// stopped, so a Toxic granted by a keyword counter (CR 122.1b) read as
/// absent. Table-driven over the four sources x three valued keywords.
#[test]
fn has_keyword_tag_reads_the_same_four_sources_as_has_keyword() {
    let base = || CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    for kw in [Keyword::Toxic(2), Keyword::Modular(1), Keyword::Ward(
        crabomination::card::WardCost::Life(2),
    )] {
        let sample = &kw;
        assert!(!base().has_keyword_tag(sample), "{kw:?}: vanilla answered true");

        let mut c = base();
        std::sync::Arc::make_mut(c.definition_mut()).keywords.push(kw.clone());
        assert!(c.has_keyword_tag(sample), "{kw:?}: printed not seen");
        let mut d = c.clone();
        d.removed_keywords_eot.push(kw.clone());
        assert!(!d.has_keyword_tag(sample), "{kw:?}: removed_eot not honoured");
        let mut d = c.clone();
        d.removed_keywords.push(kw.clone());
        assert!(!d.has_keyword_tag(sample), "{kw:?}: removed not honoured");

        let mut c = base();
        c.granted_keywords_eot.push(kw.clone());
        assert!(c.has_keyword_tag(sample), "{kw:?}: granted_eot not seen");

        let mut c = base();
        c.keyword_counters.add(kw.clone(), 1);
        assert!(c.has_keyword_tag(sample), "{kw:?}: keyword counter not seen");
        // `has_keyword` and `has_keyword_tag` agree on the exact value too.
        assert_eq!(c.has_keyword(sample), c.has_keyword_tag(sample));
    }
}

/// The value is genuinely ignored — a granted Toxic 3 answers a Toxic 1 ask —
/// and a removal is matched by value, so stripping Toxic 1 leaves Toxic 3.
#[test]
fn has_keyword_tag_ignores_the_value_but_a_removal_does_not() {
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    c.granted_keywords_eot.push(Keyword::Toxic(3));
    assert!(c.has_keyword_tag(&Keyword::Toxic(1)));
    assert!(!c.has_keyword(&Keyword::Toxic(1)));
    c.removed_keywords_eot.push(Keyword::Toxic(1));
    assert!(c.has_keyword_tag(&Keyword::Toxic(1)), "Toxic 1's removal ate Toxic 3");
    c.removed_keywords_eot.push(Keyword::Toxic(3));
    assert!(!c.has_keyword_tag(&Keyword::Toxic(1)));
}

/// CR 702.180 / 702.43 — the two value-agnostic accessors ride
/// `has_keyword_tag`, so a granted or countered Toxic/Modular counts.
#[test]
fn toxic_and_modular_see_grants_and_keyword_counters() {
    let mut c = CardInstance::new(CardId(0), catalog::grizzly_bears(), 0);
    assert!(!c.has_toxic() && !c.has_modular());
    c.granted_keywords_eot.push(Keyword::Toxic(1));
    assert!(c.has_toxic());
    c.keyword_counters.add(Keyword::Modular(1), 1);
    assert!(c.has_modular());
    c.removed_keywords.push(Keyword::Toxic(1));
    assert!(!c.has_toxic(), "a permanent removal did not strip a granted Toxic");
}
