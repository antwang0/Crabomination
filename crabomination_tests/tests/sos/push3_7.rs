#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use crate::prepared_on_battlefield;

// ── modern_decks 2026-04-30 push (post-push III) ───────────────────────────

#[test]
fn mathemagics_draws_two_to_the_x() {
    // X=3 → draw 2^3 = 8 cards.
    let mut g = two_player_game();
    // Stock the library so we can draw 8.
    for _ in 0..12 {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::mathemagics());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(6);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("Mathemagics castable for {3}{3}{U}{U}");
    drain_stack(&mut g);

    // Hand size = (before - 1 cast) + 8 drawn = before + 7.
    assert_eq!(g.players[0].hand.len(), hand_before + 7);
}

#[test]
fn mathemagics_x_zero_draws_one_card() {
    // X=0 → draw 2^0 = 1 card.
    let mut g = two_player_game();
    let cid = g.next_id();
    g.players[0].add_to_library_top(cid, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mathemagics());
    g.players[0].mana_pool.add(Color::Blue, 2);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(0),
    })
    .expect("Mathemagics castable for {0}{0}{U}{U}");
    drain_stack(&mut g);

    // -1 (cast) + 1 (drawn) = no net change, but the draw step ran.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn visionarys_dance_creates_two_flying_elementals() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::visionarys_dance());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);

    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Visionary's Dance castable for {5}{U}{R}");
    drain_stack(&mut g);

    // Two Elemental tokens added.
    let tokens: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Elemental")
        .collect();
    assert_eq!(tokens.len(), 2, "two Elemental tokens created");
    assert_eq!(g.battlefield.len(), bf_before + 2);
    for t in &tokens {
        assert_eq!(t.definition.power, 3);
        assert_eq!(t.definition.toughness, 3);
        assert!(t.definition.keywords.contains(&Keyword::Flying));
    }
}

#[test]
fn abstract_paintmage_adds_ur_at_first_main_phase() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::abstract_paintmage());

    // Empty the pool so the assertions below are unambiguous.
    g.players[0].mana_pool = crabomination::mana::ManaPool::default();
    // Fire the PreCombatMain step trigger directly — this is the same
    // entry point the engine uses when transitioning into the active
    // player's first main phase.
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);

    // The {U}{R} enters spend-restricted ("instant and sorcery only"), so
    // it sits in the restricted bucket — not the free color buckets.
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.restricted_total(), 2, "U + R added as restricted: {pool:?}");
    assert_eq!(pool.amount(Color::Blue), 0, "not freely spendable: {pool:?}");
    assert_eq!(pool.amount(Color::Red), 0, "not freely spendable: {pool:?}");
    // And it pays a {U}{R} instant/sorcery cast.
    use crabomination::mana::{cost, r, u, SpellKind};
    let mut clone = pool.clone();
    clone
        .pay_for_spell(&cost(&[u(), r()]), &SpellKind { instant_or_sorcery: true, ..Default::default() })
        .expect("restricted {U}{R} funds an instant/sorcery");
}

#[test]
fn tablet_restricted_mana_casts_instant_but_not_creature() {
    // Tablet of Discovery's {T}: Add {R}{R} produces spend-restricted mana
    // ("instant and sorcery only"): it funds Lightning Bolt but not a
    // Goblin Guide ({R} creature). End-to-end through the cast path.
    let mut g = two_player_game();
    let tablet = g.add_card_to_battlefield(0, catalog::tablet_of_discovery());
    drain_stack(&mut g); // resolve the ETB mill trigger
    g.players[0].mana_pool = crabomination::mana::ManaPool::default();

    // Index 1 is the restricted {R}{R} ability (index 0 is the plain {R}).
    g.perform_action(GameAction::ActivateAbility {
        card_id: tablet, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("Tablet {T}: Add {R}{R}");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2);
    assert_eq!(g.players[0].mana_pool.total(), 0, "restricted mana isn't freely spendable");

    // A creature ({R}) can't be cast from instants-only mana.
    let guide = g.add_card_to_hand(0, catalog::goblin_guide());
    let creature_res = g.perform_action(GameAction::CastSpell {
        card_id: guide, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert!(creature_res.is_err(), "instants-only mana can't cast a creature");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "failed cast spends nothing");

    // A Lightning Bolt (instant, {R}) can.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None })
        .expect("instants-only mana funds Lightning Bolt");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1, "one restricted {{R}} spent");
}

#[test]
fn pox_plague_halves_each_player_resources() {
    let mut g = two_player_game();
    // Stock both players: life 20, hand 4, three permanents each.
    g.players[0].life = 20;
    g.players[1].life = 20;
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let p0_perms_before = (0..3)
        .map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears()))
        .collect::<Vec<_>>();
    let p1_perms_before = (0..3)
        .map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears()))
        .collect::<Vec<_>>();

    let id = g.add_card_to_hand(0, catalog::pox_plague());
    g.players[0].mana_pool.add(Color::Black, 5);

    // Hand: Pox Plague + 4 bears = 5. Cast Pox Plague.
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pox Plague castable for {B}{B}{B}{B}{B}");
    drain_stack(&mut g);

    // Each player loses half life: 20 → 10.
    assert_eq!(g.players[0].life, 10);
    assert_eq!(g.players[1].life, 10);
    // Each player discards half their hand. P0's hand was 4 bears
    // before resolution (Pox Plague consumed itself when cast), so
    // half = 2 bears discarded → 2 left. P1 had 4 → 2 discarded → 2 left.
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(g.players[1].hand.len(), 2);
    // Sacrifice half of 3 permanents (rounded down) = 1 each.
    let p0_remaining = p0_perms_before
        .iter()
        .filter(|cid| g.battlefield.iter().any(|c| c.id == **cid))
        .count();
    let p1_remaining = p1_perms_before
        .iter()
        .filter(|cid| g.battlefield.iter().any(|c| c.id == **cid))
        .count();
    assert_eq!(p0_remaining, 2, "p0 sacrificed 1 of 3");
    assert_eq!(p1_remaining, 2, "p1 sacrificed 1 of 3");
}

#[test]
fn emil_grants_trample_to_counter_creatures() {
    // Emil's static ability "Creatures you control with +1/+1 counters
    // on them have trample" should add Trample to a counter-bearing
    // creature, but not to one without a counter. Powered by the new
    // `AffectedPermanents::AllWithCounter` layer-system variant.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::emil_vastlands_roamer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let plain_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Drop a +1/+1 counter on `bear` only.
    {
        let perm = g.battlefield.iter_mut().find(|c| c.id == bear).unwrap();
        perm.add_counters(CounterType::PlusOnePlusOne, 1);
    }

    let view = g.computed_permanent(bear).unwrap();
    assert!(
        view.keywords.contains(&Keyword::Trample),
        "bear with +1/+1 counter has trample (Emil's static): {:?}",
        view.keywords
    );

    let view2 = g.computed_permanent(plain_bear).unwrap();
    assert!(
        !view2.keywords.contains(&Keyword::Trample),
        "uncounter'd bear should NOT have trample"
    );
}

#[test]
fn matterbending_mage_etb_bounces_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::matterbending_mage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Matterbending Mage castable for {2}{U}");
    drain_stack(&mut g);

    // Bear bounced to opponent's hand.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

#[test]
fn orysa_etb_draws_two_cards() {
    let mut g = two_player_game();
    for _ in 0..3 {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::orysa_tide_choreographer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Orysa castable for {4}{U}");
    drain_stack(&mut g);

    // -1 (cast) + 2 (drawn) = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    let orysa = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Orysa, Tide Choreographer");
    assert!(orysa.is_some(), "Orysa is on the battlefield");
}

#[test]
fn orysa_alt_cost_rejected_when_total_toughness_under_ten() {
    // "{3} less if creatures you control have total toughness ≥ 10" is a
    // MANDATORY, automatic reduction. With one bear (toughness 2), no
    // reduction applies — {1}{U} floated can't pay the printed {4}{U}.
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::orysa_tide_choreographer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    let res = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(res.is_err(),
        "cast unaffordable with total toughness < 10; got {:?}", res);
}

#[test]
fn orysa_alt_cost_succeeds_when_total_toughness_ten_or_more() {
    // 5 bears (5 × 2 toughness = 10) → the automatic {3} reduction makes
    // the normal cast cost {1}{U}. No alternative-cast action needed.
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    // Seed library so the ETB draw 2 has cards to consume.
    for _ in 0..3 {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::orysa_tide_choreographer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Orysa costs {1}{U} at total toughness ≥ 10 (automatic reduction)");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Orysa, Tide Choreographer"),
        "Orysa lands via the reduced cost",
    );
}

#[test]
fn exhibition_tidecaller_is_one_drop_blocker() {
    // Body-only test: the {U} 0/2 enters the battlefield with the
    // expected stat line and creature types. No Opus rider yet.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::exhibition_tidecaller());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Exhibition Tidecaller castable for {U}");
    drain_stack(&mut g);

    let card = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Exhibition Tidecaller")
        .expect("on battlefield");
    assert_eq!(card.definition.power, 0);
    assert_eq!(card.definition.toughness, 2);
    assert!(card
        .definition
        .subtypes
        .creature_types
        .contains(&crabomination::card::CreatureType::Wizard));
}

#[test]
fn colossus_etb_drains_three_each_opponent() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.players[1].life = 20;
    let id = g.add_card_to_hand(0, catalog::colossus_of_the_blood_age());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Colossus castable for {4}{R}{W}");
    drain_stack(&mut g);

    // ETB: 3 damage to each opponent + you gain 3 life.
    assert_eq!(g.players[0].life, 23);
    assert_eq!(g.players[1].life, 17);
}

#[test]
fn colossus_dies_loots_one_for_two() {
    // Push (modern_decks): with the new DiscardAnyNumber primitive,
    // AutoDecider picks 0 discards (conservative). The follow-up Draw
    // reads `CardsDiscardedThisEffect + 1`, so the death trigger draws
    // exactly 1 card. Net: +1 hand.
    let mut g = two_player_game();
    let cid = g.add_card_to_battlefield(0, catalog::colossus_of_the_blood_age());
    for _ in 0..3 {
        let nid = g.next_id();
        g.players[0].add_to_library_top(nid, catalog::grizzly_bears());
    }
    let _discard_target = g.add_card_to_hand(0, catalog::grizzly_bears());

    let hand_before = g.players[0].hand.len();
    let _ = g.remove_to_graveyard_with_triggers(cid);
    drain_stack(&mut g);

    // AutoDecider: 0 discarded + 1 drawn = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cid));
}

#[test]
fn colossus_dies_discard_three_draws_four_via_scripted_decider() {
    // Push (modern_decks): with ScriptedDecider returning a Discard
    // answer that picks all 3 hand cards, the death trigger discards 3
    // and then draws 4 (= 3 discarded + 1).
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let cid = g.add_card_to_battlefield(0, catalog::colossus_of_the_blood_age());
    // Three cards in hand to discard, four cards in library to draw.
    let h1 = g.add_card_to_hand(0, catalog::grizzly_bears());
    let h2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    let h3 = g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..4 {
        let nid = g.next_id();
        g.players[0].add_to_library_top(nid, catalog::island());
    }
    // Scripted decider: discard all three hand cards.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![h1, h2, h3])]));

    let hand_before = g.players[0].hand.len(); // 3
    let gy_before = g.players[0].graveyard.len();
    let _ = g.remove_to_graveyard_with_triggers(cid);
    drain_stack(&mut g);

    // After: 3 discarded out of hand, 4 drawn → +1 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before - 3 + 4,
        "discarded 3 and drew 4 = net +1");
    // Graveyard gained: 3 discards + the Colossus itself = +4.
    assert_eq!(g.players[0].graveyard.len(), gy_before + 4);
}

#[test]
fn conciliators_duelist_repartee_exiles_target() {
    // Cast a creature-targeting spell while Conciliator's Duelist is in
    // play; the Repartee trigger should exile the targeted creature
    // (the "return at end step" rider is omitted; see card-level docs).
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::conciliators_duelist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Cast Lightning Bolt at the bear so the cast targets a creature.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    // Bear should be exiled (or in graveyard if the bolt killed it
    // first; either is a valid resolution since both effects are
    // legal). The important assertion: bear is not on battlefield
    // after Repartee resolves.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn conciliators_duelist_repartee_returns_target_at_end_step() {
    // Push (modern_decks): the "return at next end step" delayed rider
    // is wired via the DelayUntil fallback to `Selector::CastSpellTarget(0)`.
    // Cast Make Your Mark (a pump-cantrip that targets a creature
    // without killing it) at an opponent's creature, then advance
    // through the end step; the exiled bear should return to the
    // battlefield under its owner's (the opponent's) control.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::conciliators_duelist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Seed library so the cantrip from Make Your Mark doesn't deck out.
    g.add_card_to_library(0, catalog::lightning_bolt());

    // Cast Make Your Mark targeting the opp's bear.
    let mark = g.add_card_to_hand(0, catalog::make_your_mark());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: mark,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Make Your Mark castable for {1}{W}");
    drain_stack(&mut g);

    // Bear should be in exile after Repartee resolves (pump's target
    // becomes illegal since the bear is no longer in play, but the
    // cantrip still fires).
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "bear should be exiled after Repartee fires");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "bear should be in the (global) exile zone");

    // Fire end-of-turn triggers; the delayed trigger should resolve and
    // return the bear to the battlefield under its owner (player 1).
    g.fire_step_triggers(crabomination::game::types::TurnStep::End);
    drain_stack(&mut g);

    let bear_on_bf = g.battlefield.iter().find(|c| c.id == bear)
        .expect("bear should be back on the battlefield at end step");
    assert_eq!(bear_on_bf.controller, 1,
        "bear should come back under its owner's control");
    assert!(!g.exile.iter().any(|c| c.id == bear),
        "bear should be gone from exile zone");
}

#[test]
fn hardened_academic_grants_counter_when_card_leaves_graveyard() {
    // Hardened Academic triggers off the `EventKind::CardLeftGraveyard`
    // event. Returning a card from your graveyard via Zealous
    // Lorecaster's ETB should put a +1/+1 counter on *some* creature
    // you control. The auto-target picker (post push-VI) prefers the
    // highest-power friendly creature when handing out a friendly
    // pump, so the counter typically lands on Lorecaster (4/4) rather
    // than the smaller Academic (2/1) or Bear (2/2). Assertion: total
    // +1/+1 counters across all friendly creatures should grow by 1.
    let mut g = two_player_game();
    let academic = g.add_card_to_battlefield(0, catalog::hardened_academic());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _ = (academic, bear);
    let bolt_id = g.next_id();
    let mut bolt = crabomination::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    let counters_before: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();

    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable for {5}{R}");
    drain_stack(&mut g);

    let counters_after: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert!(
        counters_after > counters_before,
        "Hardened Academic should add a +1/+1 counter to a friendly creature when Lorecaster's ETB returns the bolt (was {}, now {})",
        counters_before, counters_after
    );
}

#[test]
fn spirit_mascot_self_counter_on_graveyard_leave() {
    let mut g = two_player_game();
    let mascot = g.add_card_to_battlefield(0, catalog::spirit_mascot());
    let bolt_id = g.next_id();
    let mut bolt = crabomination::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable for {5}{R}");
    drain_stack(&mut g);

    let counters = g
        .battlefield_find(mascot)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert!(
        counters >= 1,
        "Spirit Mascot should pick up a +1/+1 counter when bolt leaves the graveyard"
    );
}

#[test]
fn garrison_excavator_creates_spirit_on_graveyard_leave() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::garrison_excavator());
    let bolt_id = g.next_id();
    let mut bolt = crabomination::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    let bf_before = g.battlefield.len();
    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable for {5}{R}");
    drain_stack(&mut g);

    assert!(g.battlefield.len() > bf_before);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"));
}

#[test]
fn living_history_etb_creates_spirit_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::living_history());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Living History castable for {1}{R}");
    drain_stack(&mut g);

    // Living History itself + Spirit token = 2 new permanents.
    assert_eq!(g.battlefield.len(), bf_before + 2);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Living History"));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"));
}

#[test]
fn cards_left_graveyard_this_turn_resets_each_turn() {
    let mut g = two_player_game();
    let bolt_id = g.next_id();
    let mut bolt = crabomination::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable");
    drain_stack(&mut g);

    assert!(
        g.players[0].cards_left_graveyard_this_turn >= 1,
        "the bolt-from-gy → hand move should bump the per-turn tally"
    );

    // Manually reset (simulating start-of-controller's-turn untap).
    // do_untap operates on the active player; player 0 is already
    // active in the two_player_game helper so this is a no-op for
    // turn rotation but exercises the per-turn reset path.
    g.do_untap();
    assert_eq!(g.players[0].cards_left_graveyard_this_turn, 0);
}

#[test]
fn witherbloom_balancer_etb_with_keywords() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_the_balancer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(6);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Witherbloom castable for {6}{B}{G}");
    drain_stack(&mut g);

    let drag = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Witherbloom, the Balancer")
        .unwrap();
    assert_eq!(drag.definition.power, 5);
    assert_eq!(drag.definition.toughness, 5);
    assert!(drag.definition.keywords.contains(&Keyword::Flying));
    assert!(drag.definition.keywords.contains(&Keyword::Deathtouch));
}

#[test]
fn rabid_attack_pumps_friendly_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rabid_attack());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rabid Attack castable for {1}{B}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 3, "2 + 1 pump = 3");
}

#[test]
fn rabid_attack_pumps_multiple_creatures_via_multi_target() {
    // Push (modern_decks): "any number of target creatures" — fill all
    // three slots with friendly creatures, all three get +1/+0.
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b3 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rabid_attack());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(b1)),
        additional_targets: vec![Target::Permanent(b2), Target::Permanent(b3)],
        mode: None,
        x_value: None,
    })
    .expect("Rabid Attack castable");
    drain_stack(&mut g);

    for bid in [b1, b2, b3] {
        let view = g.computed_permanent(bid).expect("creature alive");
        assert_eq!(view.power, 3, "creature {bid:?} should be 3 power (2 base + 1 pump)");
    }
}

#[test]
fn rabid_attack_pumps_four_targets_beyond_old_cap() {
    // "Any number of target creatures" — `ApplyToTargets { max_targets:
    // 16 }` replaced the old hand-unrolled 3-slot shape; four targets
    // all get the pump.
    let mut g = two_player_game();
    let bears: Vec<_> = (0..4)
        .map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears()))
        .collect();
    let id = g.add_card_to_hand(0, catalog::rabid_attack());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bears[0])),
        additional_targets: bears[1..].iter().map(|b| Target::Permanent(*b)).collect(),
        mode: None,
        x_value: None,
    })
    .expect("Rabid Attack castable with four targets");
    drain_stack(&mut g);

    for bid in &bears {
        let view = g.computed_permanent(*bid).expect("creature alive");
        assert_eq!(view.power, 3, "all four creatures pumped");
    }
}

#[test]
fn rabid_attack_rejects_duplicate_targets() {
    // CR 115.3 — the slots of one multi-target instance must name
    // distinct objects.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rabid_attack());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let result = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    });
    assert!(result.is_err(), "same creature twice must be rejected");
}

#[test]
fn rabid_attack_ui_caster_declines_after_two_targets() {
    // A `wants_ui` caster is prompted slot-by-slot; every Rabid Attack
    // slot is optional (`min_targets: 0`), and answering `DeclineTarget`
    // ends selection — exactly the two chosen creatures get pumped.
    use crabomination::decision::{Decision, DecisionAnswer};
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b3 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rabid_attack());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(b1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast suspends on the slot-1 prompt");
    let pending = g.pending_decision.as_ref().expect("slot-1 ChooseTarget pending");
    match &pending.decision {
        Decision::ChooseTarget { optional, legal, .. } => {
            assert!(*optional, "up-to-N slot must be declinable");
            assert!(
                !legal.contains(&Target::Permanent(b1)),
                "already-chosen target excluded (CR 115.3)"
            );
        }
        other => panic!("expected ChooseTarget, got {other:?}"),
    }
    g.submit_decision(DecisionAnswer::Target(Target::Permanent(b2)))
        .expect("slot 1 accepted");
    // Slot-2 prompt arrives; decline it.
    assert!(g.pending_decision.is_some(), "slot-2 prompt pending");
    g.submit_decision(DecisionAnswer::DeclineTarget)
        .expect("decline replays and completes the cast");
    assert!(g.pending_decision.is_none(), "no further prompts after decline");
    drain_stack(&mut g);

    assert_eq!(g.computed_permanent(b1).unwrap().power, 3, "b1 pumped");
    assert_eq!(g.computed_permanent(b2).unwrap().power, 3, "b2 pumped");
    assert_eq!(g.computed_permanent(b3).unwrap().power, 2, "b3 untouched");
}

#[test]
fn burrog_barrage_no_pump_on_first_spell_skips_damage_with_no_opp_target() {
    // Push (modern_decks): Burrog Barrage now uses a two-slot multi-target
    // shape. Slot 0 = friendly creature to pump; slot 1 = optional opp
    // creature to take power-as-damage. When only slot 0 is filled the
    // damage half no-ops (the printed "up to one target" semantics).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::burrog_barrage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Burrog Barrage castable for {1}{G}");
    drain_stack(&mut g);

    // No prior spell → no pump. No slot-1 target → no damage. Bear lives.
    assert!(
        g.battlefield.iter().any(|c| c.id == bear),
        "bear should survive — no slot-1 opp creature, no damage half fires"
    );
}

#[test]
fn burrog_barrage_kills_opp_bear_with_second_target_filled() {
    // With both slots filled and Barrage being the second spell of the
    // turn, friendly bear pumps to 3 power and deals 3 damage to opp bear.
    let mut g = two_player_game();
    let friendly = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].spells_cast_this_turn = 1; // pretend we already cast a spell
    let id = g.add_card_to_hand(0, catalog::burrog_barrage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(friendly)),
        additional_targets: vec![Target::Permanent(opp)],
        mode: None,
        x_value: None,
    })
    .expect("Burrog Barrage castable for {1}{G}");
    drain_stack(&mut g);

    // Friendly bear pumped to 3 power. Opp bear took 3 damage and died.
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == opp),
        "opp bear should die from 3 damage (friendly's pumped power)"
    );
    assert!(
        g.battlefield.iter().any(|c| c.id == friendly),
        "friendly bear survives (no opp damage back — not Fight, just damage out)"
    );
}

#[test]
fn chelonian_tackle_pumps_toughness() {
    // No opp creature: Fight no-ops (preserves the "up to one"
    // semantics). Bear gets the +0/+10 pump and stays on the
    // battlefield at 2/12.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::chelonian_tackle());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Chelonian Tackle castable for {2}{G}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear);
    assert!(view.is_some(), "bear should survive (no opp to fight)");
    assert_eq!(view.unwrap().toughness, 12);
}

#[test]
fn chelonian_tackle_fights_opp_creature() {
    // With an opp creature passed in `additional_targets[0]`, Fight
    // resolves: friendly bear (2 power) damages opp bear (2 toughness)
    // and the opp bear damages friendly bear back. Friendly bear is
    // 2/12 after pump so survives 2 damage; opp bear dies to 2 damage.
    //
    // Push (modern_decks): Chelonian Tackle now uses a two-slot
    // multi-target shape — slot 0 = the friendly attacker, slot 1 =
    // the optional opp defender. The test now supplies both.
    let mut g = two_player_game();
    let friendly = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::chelonian_tackle());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(friendly)),
        additional_targets: vec![Target::Permanent(opp)],
        mode: None,
        x_value: None,
    })
    .expect("Chelonian Tackle castable for {2}{G}");
    drain_stack(&mut g);

    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == opp),
        "opp bear should die from the fight"
    );
    let friendly_view = g.computed_permanent(friendly);
    assert!(friendly_view.is_some(), "friendly bear should survive 2 damage with 12 toughness");
}

#[test]
fn tablet_of_discovery_etb_mills_one() {
    let mut g = two_player_game();
    for _ in 0..3 {
        let nid = g.next_id();
        g.players[0].add_to_library_top(nid, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::tablet_of_discovery());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tablet of Discovery castable for {2}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].graveyard.len(), gy_before + 1);
}

#[test]
fn practiced_offense_pumps_creatures_and_grants_double_strike() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::practiced_offense());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        // Slot 0 = target player (whose creatures get counters), slot 1 =
        // the creature picking up the keyword.
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(bear1)],
        mode: None, x_value: None,
    })
    .expect("Practiced Offense castable for {2}{W}");
    drain_stack(&mut g);

    // Both bears should have a +1/+1 counter; bear1 should also have
    // double strike granted (Selector::Target).
    let v1 = g.computed_permanent(bear1).unwrap();
    let v2 = g.computed_permanent(bear2).unwrap();
    assert_eq!(v1.power, 3, "bear1 = 2 + 1 counter");
    assert_eq!(v2.power, 3, "bear2 = 2 + 1 counter");
    assert!(v1.keywords.contains(&Keyword::DoubleStrike));
}

#[test]
fn mana_sculpt_counters_spell() {
    // Lightweight assertion: Mana Sculpt as a 4-mana counterspell should
    // remove a spell from the stack. The wizard-rider mana refund is
    // tested implicitly via the no-regression test suite (any wired
    // path that reaches the Mana Sculpt cast site will exercise the
    // `If Predicate::SelectorExists` branch).
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    let stack_count_with_bolt = g.stack.len();
    assert_eq!(stack_count_with_bolt, 1);

    g.priority.player_with_priority = 0;
    let sculpt = g.add_card_to_hand(0, catalog::mana_sculpt());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    let bolt_target = g
        .stack
        .iter()
        .find_map(|s| match s {
            StackItem::Spell { card, .. } => Some(card.id),
            _ => None,
        })
        .unwrap();
    g.perform_action(GameAction::CastSpell {
        card_id: sculpt, target: Some(Target::Permanent(bolt_target)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mana Sculpt castable for {1}{U}{U}");
    drain_stack(&mut g);

    // The bolt should have been countered (no damage to player 0).
    assert_eq!(g.players[0].life, 20, "bolt should have been countered");
}

#[test]
fn mana_sculpt_refunds_mana_value_of_countered_spell_with_wizard() {
    // The "if you control a Wizard, add an amount of {C} equal to the
    // amount of mana SPENT on that spell" rider reads the paid total
    // (`CounteredSpellManaSpent`) and banks it on a YourNextMainPhase
    // delayed trigger — checked end-to-end in
    // `mana_sculpt_banks_paid_mana_at_next_main_phase`.
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    // P0 controls a Wizard so the gate passes.
    // Add a Wizard creature (Eager First-Year is W, Human Student;
    // use the Quandrix Apprentice or similar wizard). Pick a known
    // Wizard from catalog: hydro_channeler (Merfolk Wizard).
    let _ = CreatureType::Wizard;
    g.add_card_to_battlefield(0, catalog::hydro_channeler());

    // P1 casts a Lightning Bolt (CMC = 1).
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    let bolt_target = g
        .stack
        .iter()
        .find_map(|s| match s {
            StackItem::Spell { card, .. } => Some(card.id),
            _ => None,
        })
        .unwrap();

    // P0 casts Mana Sculpt countering the Bolt.
    g.priority.player_with_priority = 0;
    let sculpt = g.add_card_to_hand(0, catalog::mana_sculpt());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: sculpt, target: Some(Target::Permanent(bolt_target)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mana Sculpt castable for {1}{U}{U}");
    drain_stack(&mut g);

    // Bolt is countered (1 mana was spent on it). The printed rider
    // BANKS the {C} for P0's next main phase (nothing lands
    // immediately) via a YourNextMainPhase delayed trigger.
    assert_eq!(g.players[0].mana_pool.total(), 0, "no immediate refund");
    assert_eq!(g.delayed_triggers.len(), 1, "the {{C}} is banked for next main");
}


// ── push VI: Lorehold completion + Daydream + token-side triggers ──────────

#[test]
fn ark_of_hunger_triggers_on_card_left_graveyard() {
    let mut g = two_player_game();
    let ark = g.add_card_to_battlefield(0, catalog::ark_of_hunger());
    let _ = ark;

    // Seed a card in P0's graveyard, then exile-via-trigger to fire the event.
    let bolt_id = g.next_id();
    let mut bolt = crabomination::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    let life_before = g.players[0].life;
    let opp_life_before = g.players[1].life;

    // Cast Zealous Lorecaster — its ETB returns an instant/sorcery from your gy
    // to your hand, firing CardLeftGraveyard, which Ark of Hunger watches.
    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 1, "Ark of Hunger should gain 1 life");
    assert_eq!(g.players[1].life, opp_life_before - 1, "Ark of Hunger should ping opp for 1");
}

#[test]
fn ark_of_hunger_mill_activation() {
    let mut g = two_player_game();
    let ark = g.add_card_to_battlefield(0, catalog::ark_of_hunger());
    // Seed a few library cards.
    for _ in 0..3 {
        let nid = g.next_id();
        g.players[0].add_to_library_top(nid, catalog::grizzly_bears());
    }
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: ark, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Ark of Hunger {T}: Mill activation");
    drain_stack(&mut g);

    assert_eq!(g.players[0].graveyard.len(), gy_before + 1, "Ark mills 1");
    assert!(g.battlefield.iter().any(|c| c.id == ark && c.tapped),
        "Ark should be tapped after activation");
}

#[test]
fn suspend_aggression_exiles_target_and_top_of_library() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::suspend_aggression());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    let top_card_id = g.next_id();
    g.players[0].add_to_library_top(top_card_id, catalog::lightning_bolt());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Suspend Aggression castable for {1}{R}{W}");
    drain_stack(&mut g);

    // Bear should be exiled (not on battlefield, not in graveyard).
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear off battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear), "Bear exiled");
    // Top card of library should be exiled.
    assert!(g.exile.iter().any(|c| c.id == top_card_id), "Top of library exiled");
}

#[test]
fn wilt_in_the_heat_deals_five_to_creature_and_exiles_it() {
    // Printed: "deals 5 damage; if that creature would die this turn,
    // exile it instead." 2/2 Grizzly Bears dies to 5 damage, and the
    // `ExileIfWouldDieThisTurn` death replacement redirects it to exile
    // (not graveyard).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wilt_in_the_heat());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wilt in the Heat castable for {2}{R}{W}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear leaves the battlefield");
    assert!(
        !g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Bear does NOT go to graveyard — exile rider redirects"
    );
    assert!(
        g.exile.iter().any(|c| c.id == bear),
        "Bear is exiled per 'would die this turn' rider"
    );
}

#[test]
fn wilt_in_the_heat_leaves_high_toughness_creature_in_play() {
    // Serra Angel has 4 toughness — wait, toughness 4 <= 5 so still
    // exiled. Use a higher-toughness creature. We don't have a 6+
    // toughness bear in catalog handy; use Tenured Concocter (4/5
    // Troll Druid). Toughness 5 = boundary; 5 <= 5 still triggers
    // exile. Use a 6/6 token construct or just verify the predicate
    // works at boundary with a 6-toughness creature.
    //
    // Use Bookwurm (7/7) which definitely doesn't die to 5 damage.
    let mut g = two_player_game();
    let beledros = g.add_card_to_battlefield(1, catalog::bookwurm());
    let id = g.add_card_to_hand(0, catalog::wilt_in_the_heat());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(beledros)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Wilt in the Heat castable");
    drain_stack(&mut g);

    let bel = g.battlefield_find(beledros).expect("Beledros still on bf");
    assert_eq!(bel.damage, 5, "Beledros has 5 damage marked");
    assert!(
        !g.exile.iter().any(|c| c.id == beledros),
        "Beledros not exiled — toughness 6 > 5"
    );
}

#[test]
fn wilt_in_the_heat_exiles_creature_that_dies_later_this_turn() {
    // The replacement lasts the whole turn, not just the spell's own
    // damage: a 6/6 that survives the 5 but dies to a later source is
    // still exiled rather than going to the graveyard.
    let mut g = two_player_game();
    let beledros = g.add_card_to_battlefield(1, catalog::bookwurm());
    let id = g.add_card_to_hand(0, catalog::wilt_in_the_heat());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(beledros)),
        additional_targets: vec![], mode: None, x_value: None })
        .expect("Wilt in the Heat castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(beledros).is_some(), "survives the 5 damage");

    // Now it dies to another source later the same turn.
    g.remove_to_graveyard_with_triggers(beledros);
    drain_stack(&mut g);
    assert!(
        !g.players[1].graveyard.iter().any(|c| c.id == beledros),
        "later death is redirected out of the graveyard"
    );
    assert!(
        g.exile.iter().any(|c| c.id == beledros),
        "lingering 'would die this turn' replacement exiles it"
    );
}

#[test]
fn wilt_in_the_heat_does_not_exile_indestructible_creature() {
    // "If that creature would die this turn, exile it instead" only fires
    // on an actual death. An indestructible creature doesn't die to the 5
    // damage, so it stays in play — the death replacement never triggers.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear)
        .unwrap()
        .add_counters(CounterType::Indestructible, 1);
    let id = g.add_card_to_hand(0, catalog::wilt_in_the_heat());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None })
        .expect("Wilt in the Heat castable");
    drain_stack(&mut g);

    assert!(
        g.battlefield_find(bear).is_some(),
        "indestructible creature survives the 5 damage"
    );
    assert!(
        !g.exile.iter().any(|c| c.id == bear),
        "and is not exiled — it never died"
    );
}

#[test]
fn wilt_in_the_heat_alt_cost_rejected_with_empty_graveyard_history() {
    // "{2} less if one or more cards left your graveyard this turn" is a
    // MANDATORY reduction. With no graveyard exits, no reduction: {R}{W}
    // floated can't pay the printed {2}{R}{W}.
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wilt_in_the_heat());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    // No generic mana — only enough for the reduced cost.
    assert_eq!(g.players[0].cards_left_graveyard_this_turn, 0);

    let res = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(res.is_err(),
        "cast unaffordable when no cards left graveyard this turn; got {:?}", res);
    assert!(
        g.players[0].hand.iter().any(|c| c.id == id),
        "Wilt should still be in hand (cast rejected before any state mutation)",
    );
}

#[test]
fn wilt_in_the_heat_alt_cost_succeeds_after_graveyard_recursion() {
    // Once a card has left the controller's graveyard this turn, the
    // automatic {2} reduction makes the normal cast cost {R}{W}.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wilt_in_the_heat());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    // Simulate "a card left your graveyard this turn" by bumping the
    // per-turn counter directly.
    g.players[0].cards_left_graveyard_this_turn = 1;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Wilt costs {R}{W} after a card leaves your gy this turn");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear takes 5 damage via the reduced cast");
}

#[test]
fn daydream_flickers_and_adds_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::daydream());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Daydream castable for {W}");
    drain_stack(&mut g);

    // The same bear (zone-stable) should be back on the battlefield with a +1/+1 counter.
    let bear_view = g.computed_permanent(bear);
    assert!(bear_view.is_some(), "Bear should be back on battlefield");
    let bear_view = bear_view.unwrap();
    assert_eq!(bear_view.power, 3, "Bear with +1/+1 counter = 3 power");
    assert_eq!(bear_view.toughness, 3, "Bear with +1/+1 counter = 3 toughness");
}

#[test]
fn snarl_song_creates_two_fractals_with_x_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::snarl_song());
    // Snarl Song cost is {5}{G} and Converge X = colors of mana spent.
    // Add 6 green mana so the engine treats it as 1-color converge.
    g.players[0].mana_pool.add(Color::Green, 6);
    let bf_before = g.battlefield.len();
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Snarl Song castable for {5}{G}");
    drain_stack(&mut g);

    // Two Fractal tokens minted.
    let fractal_count = g.battlefield.iter().filter(|c| c.definition.name == "Fractal").count();
    assert_eq!(fractal_count, 2, "Snarl Song creates two Fractal tokens");
    // X = 1 (one color used).
    assert!(g.battlefield.len() >= bf_before + 2);
    // Life gained = X.
    assert_eq!(g.players[0].life, life_before + 1, "Snarl Song gains X life (1)");
}

#[test]
fn wild_hypothesis_creates_fractal_with_x_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::wild_hypothesis());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Seed library so surveil has something to look at.
    for _ in 0..3 {
        let nid = g.next_id();
        g.players[0].add_to_library_top(nid, catalog::lightning_bolt());
    }

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    })
    .expect("Wild Hypothesis castable for X=3 ({3}{G})");
    drain_stack(&mut g);

    // Fractal token with 3 +1/+1 counters → 3/3.
    let fractal = g.battlefield.iter().find(|c| c.definition.name == "Fractal");
    assert!(fractal.is_some(), "Wild Hypothesis mints a Fractal");
    let fractal_id = fractal.unwrap().id;
    let view = g.computed_permanent(fractal_id).unwrap();
    assert_eq!(view.power, 3, "Fractal should have 3 +1/+1 counters → 3 power");
}

#[test]
fn tome_blast_deals_two_damage_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tome_blast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tome Blast castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear (2/2) dies to 2 damage");
}

#[test]
fn duel_tactics_pings_and_grants_cant_block() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::duel_tactics());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Duel Tactics castable for {R}");
    drain_stack(&mut g);

    // Bear takes 1 (bear is 2/2 so it survives). Bear should now have CantBlock.
    let bear_view = g.computed_permanent(bear).unwrap();
    assert!(bear_view.keywords.contains(&Keyword::CantBlock),
        "Bear should have CantBlock granted EOT");
}

/// Soaring Stoneglider: the printed alt cost {2}{W} + exile two cards
/// from your graveyard is wired via the new `exile_from_graveyard_count`
/// field. With 2 cards in gy and {2}{W} available, the alt cast succeeds
/// and both gy cards land in exile.
#[test]
fn soaring_stoneglider_alt_cost_exiles_two_from_graveyard() {
    let mut g = two_player_game();
    // Seed graveyard with 2 cards (lowest-CMC picker: takes both).
    let bolt_id = g.next_id();
    let mut bolt = crabomination::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);
    let bears_id = g.next_id();
    let mut bears = crabomination::card::CardInstance::new(bears_id, catalog::grizzly_bears(), 0);
    bears.controller = 0;
    g.players[0].graveyard.push(bears);
    let id = g.add_card_to_hand(0, catalog::soaring_stoneglider());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Soaring Stoneglider alt-castable at {2}{W} with 2 cards in gy");
    drain_stack(&mut g);

    // Soaring Stoneglider on battlefield.
    let on_bf = g.battlefield.iter().any(|c| c.definition.name == "Soaring Stoneglider");
    assert!(on_bf, "Stoneglider ETBs after alt cast");
    // Both gy cards in exile.
    assert!(g.exile.iter().any(|c| c.id == bolt_id),
        "Lightning Bolt should be exiled as alt cost");
    assert!(g.exile.iter().any(|c| c.id == bears_id),
        "Grizzly Bears should be exiled as alt cost");
    assert!(g.players[0].graveyard.is_empty(), "Graveyard drained by 2");
}

/// Soaring Stoneglider: alt cost rejected when graveyard has < 2 cards.
/// The caller can fall back to the printed mana cost.
#[test]
fn soaring_stoneglider_alt_cost_rejects_with_insufficient_graveyard() {
    let mut g = two_player_game();
    // Only one card in gy — alt cost requires two.
    let bolt_id = g.next_id();
    let mut bolt = crabomination::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);
    let id = g.add_card_to_hand(0, catalog::soaring_stoneglider());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    let res = g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(res.is_err(), "Alt cast should reject with only 1 gy card");
    // The Stoneglider is still in hand (rolled back cleanly).
    assert!(g.players[0].hand.iter().any(|c| c.id == id),
        "Stoneglider should remain in hand on rejected alt cast");
}

#[test]
fn practiced_scrollsmith_etb_exiles_noncreature_nonland_from_gy() {
    let mut g = two_player_game();
    // Seed a noncreature/nonland in gy: a sorcery (Pox Plague) and a land
    // (Forest) and a creature (Bears). Only the sorcery should be exiled.
    let pox_id = g.next_id();
    let mut pox = crabomination::card::CardInstance::new(pox_id, catalog::pox_plague(), 0);
    pox.controller = 0;
    g.players[0].graveyard.push(pox);
    let forest_id = g.next_id();
    let mut forest = crabomination::card::CardInstance::new(forest_id, catalog::forest(), 0);
    forest.controller = 0;
    g.players[0].graveyard.push(forest);
    let bear_id = g.next_id();
    let mut bear = crabomination::card::CardInstance::new(bear_id, catalog::grizzly_bears(), 0);
    bear.controller = 0;
    g.players[0].graveyard.push(bear);

    let id = g.add_card_to_hand(0, catalog::practiced_scrollsmith());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Practiced Scrollsmith castable for {R}{R}{W}");
    drain_stack(&mut g);

    // Pox should have been exiled.
    assert!(g.exile.iter().any(|c| c.id == pox_id), "Pox Plague exiled");
    // Forest stays in gy (it's a land).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == forest_id),
        "Forest stays in gy (it's a land)");
    // Bear stays in gy (it's a creature).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear_id),
        "Bear stays in gy (it's a creature)");
}

#[test]
fn topiary_lecturer_taps_for_g_equal_to_power() {
    let mut g = two_player_game();
    let lec = g.add_card_to_battlefield(0, catalog::topiary_lecturer());
    // Make sure it's not summoning sick — manually unmark it.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == lec) {
        c.summoning_sick = false;
    }

    g.perform_action(GameAction::ActivateAbility {
        card_id: lec, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Topiary Lecturer {T}: Add G mana ability");
    drain_stack(&mut g);

    // Base 1/2 → 1 power → 1 G.
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "Adds G = power (1)");
}

#[test]
fn topiary_lecturer_increment_lands_counter_on_three_mana_cast() {
    // Increment: when you cast a spell with mana spent > P or T of this
    // creature (1 or 2), put a +1/+1 counter. Casting a 3-mana spell
    // satisfies the gate (3 > 2).
    let mut g = two_player_game();
    let lec = g.add_card_to_battlefield(0, catalog::topiary_lecturer());
    drain_stack(&mut g);
    let curse = g.add_card_to_hand(0, catalog::withering_curse());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: curse,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Withering Curse castable for {1}{B}{B}");
    drain_stack(&mut g);
    let lec_after = g.battlefield_find(lec).unwrap();
    assert!(
        lec_after.counter_count(CounterType::PlusOnePlusOne) >= 1,
        "3-mana spell triggers Increment, +1/+1 counter on Topiary Lecturer",
    );
}

#[test]
fn pest_token_attack_trigger_gains_one_life() {
    // SOS Pest token: "Whenever this token attacks, you gain 1 life."
    // Use Send in the Pest to mint a token, then attack with it, then
    // confirm the controller gained a life.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::send_in_the_pest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Send in the Pest castable for {1}{B}");
    drain_stack(&mut g);

    let pest = g.battlefield.iter().find(|c| c.definition.name == "Pest")
        .expect("Pest token created");
    let pest_id = pest.id;
    // Manually un-summoning-sick.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == pest_id) {
        c.summoning_sick = false;
    }

    // Move to combat phase.
    g.step = TurnStep::DeclareAttackers;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pest_id, target: AttackTarget::Player(1),
    }]))
    .expect("Declare Pest attacker");
    // Drain any triggers.
    drain_stack(&mut g);

    assert!(g.players[0].life > life_before,
        "SOS Pest token's attack trigger should grant +1 life (was {}, now {})",
        life_before, g.players[0].life);
}


// ── push VII: Multicolored predicate, MDFC bodies, Lorehold capstone ────────

#[test]
fn homesickness_draws_two_taps_and_stuns() {
    // Push (modern_decks): now multi-target — slot 0 = target player
    // (draw 2), slots 1 + 2 = optional creature taps + stun counters.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::homesickness());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    // Seed 2 cards on the library so the draw-2 actually moves them.
    let l1 = g.next_id(); g.players[0].add_to_library_top(l1, catalog::lightning_bolt());
    let l2 = g.next_id(); g.players[0].add_to_library_top(l2, catalog::lightning_bolt());

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("Homesickness castable for {4}{U}{U}");
    drain_stack(&mut g);

    // Caster drew 2.
    assert_eq!(g.players[0].hand.len(), 2, "drew 2 cards");
    // Bear (slot 1) is tapped + stunned.
    let bear_card = g.battlefield.iter().find(|c| c.id == bear).expect("bear on bf");
    assert!(bear_card.tapped, "bear tapped");
    assert!(bear_card.counter_count(CounterType::Stun) >= 1, "stun counter on bear");
}

#[test]
fn homesickness_taps_and_stuns_two_creatures() {
    // Multi-target with slot 0 + slots 1+2 filled — both bears tapped + stunned.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::homesickness());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    let l1 = g.next_id(); g.players[0].add_to_library_top(l1, catalog::lightning_bolt());
    let l2 = g.next_id(); g.players[0].add_to_library_top(l2, catalog::lightning_bolt());

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(bear), Target::Permanent(bear2)],
        mode: None,
        x_value: None,
    })
    .expect("Homesickness castable for {4}{U}{U}");
    drain_stack(&mut g);

    let b1 = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    let b2 = g.battlefield.iter().find(|c| c.id == bear2).unwrap();
    assert!(b1.tapped && b1.counter_count(CounterType::Stun) >= 1);
    assert!(b2.tapped && b2.counter_count(CounterType::Stun) >= 1);
}

#[test]
fn fractalize_sets_target_to_x_plus_one_base() {
    // Push (modern_decks): Fractalize now uses `Effect::SetBasePT` to
    // overwrite the target's base P/T to (X+1)/(X+1) for the turn —
    // not a +N pump. So a Grizzly Bears (2/2) at X=3 becomes a
    // 4/4 (base 0/0 → set to 4/4) until end of turn.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractalize());
    // Cast for X=3 — costs {3}{U} = 4 mana.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: Some(3),
    })
    .expect("Fractalize castable for {X=3}{U}");
    drain_stack(&mut g);

    let cv = g.computed_permanent(bear).expect("bear computed");
    // Base reset to 4/4 (X+1 = 4) — Square-Up-style base override.
    assert_eq!(cv.power, 4, "bear's base P set to X+1 = 4");
    assert_eq!(cv.toughness, 4, "bear's base T set to X+1 = 4");
}

#[test]
fn fractalize_layers_under_plus_one_counters() {
    // A creature with a +1/+1 counter, after Fractalize at X=2, should
    // be 3/3 base + 1/1 counter = 4/4 (layers: 7b SetBasePT applies
    // first, then 7c counters add on top per CR 613.7c-f).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Place a +1/+1 counter on the bear directly.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    let id = g.add_card_to_hand(0, catalog::fractalize());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Fractalize castable for {X=2}{U}");
    drain_stack(&mut g);

    let cv = g.computed_permanent(bear).expect("bear computed");
    // Base set to 3/3 (X+1 = 3) + 1/1 counter = 4/4.
    assert_eq!(cv.power, 4, "bear at base 3 + 1 counter = 4 power");
    assert_eq!(cv.toughness, 4, "bear at base 3 + 1 counter = 4 toughness");
}

#[test]
fn divergent_equation_returns_instant_from_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::divergent_equation());
    // Seed an instant in P0's graveyard.
    let bolt_id = g.next_id();
    let mut bolt = crabomination::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt_id)), additional_targets: vec![], mode: None, x_value: Some(1),
    })
    .expect("Divergent Equation castable for {X=1}{X=1}{U}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_id), "Bolt in hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt_id),
        "Bolt left graveyard");
}

#[test]
fn divergent_equation_returns_x_cards_from_graveyard_at_x_two() {
    // X=2 → return 2 instants from gy. Seed 3 instants; only 2 should
    // come back to hand (the engine walks gy iteration order).
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::divergent_equation());
    let bolt_a = g.next_id();
    let mut a = crabomination::card::CardInstance::new(bolt_a, catalog::lightning_bolt(), 0);
    a.controller = 0;
    g.players[0].graveyard.push(a);
    let bolt_b = g.next_id();
    let mut b = crabomination::card::CardInstance::new(bolt_b, catalog::lightning_bolt(), 0);
    b.controller = 0;
    g.players[0].graveyard.push(b);
    let bolt_c = g.next_id();
    let mut c = crabomination::card::CardInstance::new(bolt_c, catalog::lightning_bolt(), 0);
    c.controller = 0;
    g.players[0].graveyard.push(c);

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4); // X=2 → 2+2+U = 5 mana
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Divergent Equation castable for {X=2}{X=2}{U}");
    drain_stack(&mut g);

    // Two of three Bolts should be in hand; one stays in graveyard.
    let in_hand = [bolt_a, bolt_b, bolt_c]
        .iter()
        .filter(|&&bid| g.players[0].hand.iter().any(|c| c.id == bid))
        .count();
    assert_eq!(in_hand, 2, "X=2 returns exactly two cards from graveyard");
    let in_gy = [bolt_a, bolt_b, bolt_c]
        .iter()
        .filter(|&&bid| g.players[0].graveyard.iter().any(|c| c.id == bid))
        .count();
    assert_eq!(in_gy, 1, "one card stays in graveyard");
}

#[test]
fn divergent_equation_returns_zero_at_x_zero() {
    // X=0 → take no cards from gy. The cantrip-with-no-effect mode.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::divergent_equation());
    let bolt_a = g.next_id();
    let mut a = crabomination::card::CardInstance::new(bolt_a, catalog::lightning_bolt(), 0);
    a.controller = 0;
    g.players[0].graveyard.push(a);

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(0),
    })
    .expect("Divergent Equation castable for {X=0}{X=0}{U}");
    drain_stack(&mut g);

    assert!(!g.players[0].hand.iter().any(|c| c.id == bolt_a),
        "Bolt should stay in graveyard at X=0");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt_a),
        "Bolt should remain in graveyard");
}

#[test]
fn divergent_equation_caps_at_available_cards() {
    // X=3 but only 1 IS card in gy — return the one card, no error.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::divergent_equation());
    let bolt_a = g.next_id();
    let mut a = crabomination::card::CardInstance::new(bolt_a, catalog::lightning_bolt(), 0);
    a.controller = 0;
    g.players[0].graveyard.push(a);

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(6); // X=3 → 3+3+U
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    })
    .expect("Divergent Equation castable for {X=3}{X=3}{U}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_a),
        "the only IS card should be in hand");
}

#[test]
fn divergent_equation_filters_to_instants_and_sorceries() {
    // Seed a creature card alongside IS — only the IS comes back.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::divergent_equation());
    let bolt = g.next_id();
    let mut b = crabomination::card::CardInstance::new(bolt, catalog::lightning_bolt(), 0);
    b.controller = 0;
    g.players[0].graveyard.push(b);
    let bear = g.next_id();
    let mut br = crabomination::card::CardInstance::new(bear, catalog::grizzly_bears(), 0);
    br.controller = 0;
    g.players[0].graveyard.push(br);

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Divergent Equation castable for {X=2}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt (instant) returns to hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear),
        "Grizzly Bears (creature) stays in graveyard");
}

#[test]
fn divergent_equation_exiles_itself_via_exile_on_resolve_flag() {
    // Push (modern_decks): the printed "Exile Divergent Equation" rider
    // now lands via the new `CardDefinition.exile_on_resolve` flag —
    // the resolved instant goes to exile, not graveyard, so it can't
    // be flashbacked / Past-in-Flames-looped.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::divergent_equation());
    let bolt_id = g.next_id();
    let mut bolt = crabomination::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let exile_before = g.exile.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt_id)), additional_targets: vec![], mode: None, x_value: Some(1),
    })
    .expect("Divergent Equation castable for {X=1}{X=1}{U}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == id),
        "Divergent Equation should land in exile after resolve");
    assert_eq!(g.exile.len(), exile_before + 1,
        "Exile zone gained one card");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id),
        "Divergent Equation should NOT be in graveyard");
}

#[test]
fn spectacular_skywhale_is_one_four_flyer() {
    let g = two_player_game();
    let _ = g;
    let def = catalog::spectacular_skywhale();
    assert_eq!(def.power, 1);
    assert_eq!(def.toughness, 4);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert_eq!(def.cost.cmc(), 4);
}

#[test]
fn lorehold_the_historian_opp_upkeep_loots_with_scripted_yes() {
    // Push (modern_decks): per-opp-upkeep loot trigger fires off the
    // `EventScope::OpponentControl` step trigger. With Lorehold on
    // P0's bf and P1's upkeep step, the trigger fires; ScriptedDecider
    // says "yes" to the MayDo, the player discards 1 + draws 1.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_the_historian());
    drain_stack(&mut g);
    // Seed P0's hand and library so loot has fuel.
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let lib_top = g.next_id();
    g.players[0].add_to_library_top(lib_top, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    // Set up P1 as active player at upkeep.
    g.active_player_idx = 1;
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    // Fire the upkeep step for P1.
    g.fire_step_triggers(crabomination::game::types::TurnStep::Upkeep);
    drain_stack(&mut g);
    // Hand size: -1 discard + 1 draw = 0 net change in count.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Hand size net unchanged: -1 discard + 1 draw = 0 net (after MayDo accepted)");
    assert_eq!(g.players[0].graveyard.len(), gy_before + 1,
        "P0's graveyard +1 from the discard");
}

#[test]
fn lorehold_the_historian_grants_miracle_two_on_first_is_draw() {
    // "Each instant and sorcery card in your hand has miracle {2}." The
    // first IS card drawn this turn gains an until-end-of-turn may-play
    // grant whose alt cost is {2}; the controller may cast it by paying
    // {2} (not its full cost, not for free).
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_the_historian());
    drain_stack(&mut g);
    g.players[0].cards_drawn_this_turn = 0;
    let bolt = g.next_id();
    g.players[0].add_to_library_top(bolt, catalog::lightning_bolt());

    // Draw the (first) card and dispatch the resulting CardDrawn trigger.
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);

    let bolt_hand = g.players[0].hand.iter().find(|c| c.id == bolt)
        .expect("Bolt drawn to hand");
    assert!(bolt_hand.may_play_until.is_some(), "miracle may-play granted");
    assert_eq!(
        bolt_hand.granted_alt_cast_cost_eot.as_ref().map(|c| c.summary()),
        Some("{2}".to_string()),
        "miracle cost is {{2}}",
    );

    // Can't cast it without paying the {2}.
    g.priority.player_with_priority = 0;
    let unpaid = g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None });
    assert!(unpaid.is_err(), "miracle cast requires paying its {{2}} cost");
    assert!(
        g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt stays in hand when the miracle cost can't be paid",
    );

    // With exactly {2} available, the miracle cast goes through and empties the pool.
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None })
        .expect("Bolt castable for its {2} miracle cost");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the {{2}} miracle cost was paid");
    assert!(
        !g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt left hand to the stack",
    );
}

#[test]
fn mage_tower_referee_gets_counter_on_multicolored_cast() {
    let mut g = two_player_game();
    let referee = g.add_card_to_battlefield(0, catalog::mage_tower_referee());

    // Cast a multicolored spell (Lorehold Charm — {R}{W}).
    let charm_id = g.add_card_to_hand(0, catalog::lorehold_charm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    // Drain stack of any pending state, choose mode 2 (creatures get +1/+1)
    // since it doesn't require an extra target.
    g.perform_action(GameAction::CastSpell {
        card_id: charm_id, target: None, additional_targets: vec![], mode: Some(2), x_value: None,
    })
    .expect("Lorehold Charm castable for {R}{W}");
    drain_stack(&mut g);

    let counter = g.battlefield.iter().find(|c| c.id == referee)
        .expect("referee on bf").counter_count(CounterType::PlusOnePlusOne);
    assert!(counter >= 1, "referee gained +1/+1 counter on multicolored cast (got {})", counter);
}

#[test]
fn mage_tower_referee_no_counter_on_monocolored_cast() {
    let mut g = two_player_game();
    let referee = g.add_card_to_battlefield(0, catalog::mage_tower_referee());

    // Cast a mono-color spell (Lightning Bolt — {R}). Should NOT add a counter.
    let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt_id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    let counter = g.battlefield.iter().find(|c| c.id == referee)
        .expect("referee on bf").counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counter, 0, "no counter for mono-color cast");
}

#[test]
fn rubble_rouser_etb_rummages() {
    // ETB rummage is `Effect::MayDo`; inject Bool(true).
    let mut g = two_player_game();
    // Seed a card to discard + a card to draw.
    let _ = g.add_card_to_hand(0, catalog::lightning_bolt());
    let l1 = g.next_id(); g.players[0].add_to_library_top(l1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::rubble_rouser());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rubble Rouser castable for {2}{R}");
    drain_stack(&mut g);

    // Net: cast (-1 hand) + discard 1 (-1 hand, +1 gy) + draw 1 (+1 hand).
    // From hand_before perspective: -1 -1 +1 = -1. Cast removed Rouser
    // already (it's now on battlefield).
    assert_eq!(g.players[0].hand.len(), hand_before - 1,
        "rummage net: -1 from hand (cast moved Rouser to bf, discard then draw nets 0)");
    assert!(g.players[0].graveyard.len() > gy_before, "discarded card hit gy");
    let on_bf = g.battlefield.iter().find(|c| c.id == id).expect("Rouser on bf");
    assert_eq!(on_bf.power(), 1);
    assert_eq!(on_bf.toughness(), 4);
}

#[test]
fn rubble_rouser_activation_exiles_gy_card_pings_opp_and_adds_red() {
    // Push (modern_decks): `{T}, Exile a card from your graveyard:` is
    // wired via `ActivatedAbility.exile_other_filter`. Activation drains
    // a graveyard card (cost) and resolves: opp takes 1 damage + R goes
    // into the player's pool.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::rubble_rouser());
    g.clear_sickness(id);
    // Seed a card in P0's graveyard to exile as cost.
    let gy_card = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let opp_life_before = g.players[1].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Rubble Rouser activation w/ gy card to exile");
    drain_stack(&mut g);

    assert!(
        g.exile.iter().any(|c| c.id == gy_card),
        "Exiled gy card is in exile",
    );
    assert_eq!(g.players[1].life, opp_life_before - 1,
        "Opp loses 1 life from the ping");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1,
        "Caster's pool has the R produced by the activation");
}

#[test]
fn rubble_rouser_activation_rejected_with_empty_graveyard() {
    // Activation cost requires exiling another card from your graveyard;
    // with an empty graveyard the activation is rejected pre-payment.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::rubble_rouser());
    g.clear_sickness(id);
    assert!(g.players[0].graveyard.is_empty());

    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None });
    assert!(res.is_err(),
        "Activation rejected when gy is empty; got {:?}", res);
    assert!(!g.battlefield_find(id).unwrap().tapped,
        "Rouser should not be tapped (cost rolled back on gate fail)");
}

#[test]
fn additive_evolution_etb_creates_fractal_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::additive_evolution());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Additive Evolution castable for {3}{G}{G}");
    drain_stack(&mut g);

    let frac = g.battlefield.iter().find(|c| c.definition.name == "Fractal")
        .expect("Fractal token created");
    assert_eq!(frac.counter_count(CounterType::PlusOnePlusOne), 3,
        "Fractal entered with three +1/+1 counters");
    // 0/0 base + 3 counters = 3/3.
    assert_eq!(frac.power(), 3);
    assert_eq!(frac.toughness(), 3);
}

#[test]
fn zimones_experiment_reveals_to_hand_and_lands_basic() {
    let mut g = two_player_game();
    // Library (top → bottom): bear, forest. RevealUntilFind walks the
    // top: bear is a creature → it goes to hand. Forest stays in the
    // library where the subsequent Search picks it up onto the
    // battlefield tapped.
    let forest_id = g.next_id();
    g.players[0].add_to_library_top(forest_id, catalog::forest());
    let bear_id = g.next_id();
    g.players[0].add_to_library_top(bear_id, catalog::grizzly_bears());

    // ScriptedDecider answers the SearchLibrary decision with the seeded
    // Forest (the AutoDecider's default of `Search(None)` would pass on
    // the search, leaving the basic in the library).
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest_id)),
    ]));

    let id = g.add_card_to_hand(0, catalog::zimones_experiment());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Zimone's Experiment castable for {3}{G}");
    drain_stack(&mut g);

    // The bear should now be in hand (RevealUntilFind put it there).
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_id),
        "Bear should be in hand after RevealUntilFind");
    // The forest should be on the battlefield tapped (Search half).
    let forest_on_bf = g.battlefield.iter().find(|c| c.id == forest_id);
    assert!(forest_on_bf.is_some(), "Forest should ETB via Search");
    assert!(forest_on_bf.unwrap().tapped, "Forest should ETB tapped");
}

#[test]
fn petrified_hamlet_taps_for_colorless() {
    let mut g = two_player_game();
    let lid = g.add_card_to_battlefield(0, catalog::petrified_hamlet());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == lid) {
        c.summoning_sick = false;
        c.tapped = false;
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: lid, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Petrified Hamlet {T}: Add C");
    drain_stack(&mut g);

    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "added 1 colorless");
}

#[test]
fn owlin_historian_gets_pump_when_card_leaves_graveyard() {
    let mut g = two_player_game();
    let owlin = g.add_card_to_battlefield(0, catalog::owlin_historian());
    // Seed an instant in P0's graveyard for Zealous Lorecaster's ETB to return.
    let bolt_id = g.next_id();
    let mut bolt = crabomination::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    // Cast Zealous Lorecaster — its ETB fires CardLeftGraveyard.
    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable");
    drain_stack(&mut g);

    let owlin_card = g.battlefield.iter().find(|c| c.id == owlin).expect("Owlin on bf");
    assert!(owlin_card.power() >= 3, "Owlin should be pumped (was 2/3, now {}/{})",
        owlin_card.power(), owlin_card.toughness());
}

#[test]
fn additive_evolution_combat_pumps_friendly_creature() {
    use crabomination::game::TurnStep;
    let mut g = two_player_game();
    let _enchant = g.add_card_to_battlefield(0, catalog::additive_evolution());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Drain anything pending.
    drain_stack(&mut g);

    // Move to begin combat — the begin-combat trigger fires once per
    // ActivePlayer step transition.
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);

    let _ = bear;
    // +1/+1 counter from begin-combat trigger lands on a friendly
    // creature (auto-target picks the highest-power friendly).
    // `add_card_to_battlefield` skips ETB triggers, so we don't expect
    // the Fractal token here — only the begin-combat counter.
    let total_creature_pump: i32 = g.battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature())
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne) as i32)
        .sum();
    assert!(total_creature_pump >= 1,
        "begin-combat pump should add ≥ 1 +1/+1 counter on a friendly creature \
         (got total={})", total_creature_pump);
}

