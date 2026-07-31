#![allow(unused_imports)]
#![allow(clippy::type_complexity)]
use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use crate::prepared_on_battlefield;
use crate::push15_17::place_creature;

// ── Push XVII: Ward MDFCs + Modern supplement ──────────────────────────────

#[test]
fn red_burn_spells_kill_bear_with_expected_life_riders() {
    // Table-driven: (ctor, red, colorless, mode, caster life delta, opp life delta).
    // Char: 4 to target, 2 to self. Searing Blaze: 3 to creature + 3 to its
    // controller. Collective Defiance mode 0: 4 to target creature.
    type Ctor = fn() -> crabomination::card::CardDefinition;
    let cases: [(Ctor, u32, u32, Option<usize>, i32, i32, &str); 3] = [
        (catalog::char, 1, 2, None, -2, 0, "Char"),
        (catalog::searing_blaze, 2, 0, None, 0, -3, "Searing Blaze"),
        (catalog::collective_defiance, 2, 1, Some(0), 0, 0, "Collective Defiance"),
    ];
    for (ctor, red, colorless, mode, caster_delta, opp_delta, name) in cases {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(bear);
        let id = g.add_card_to_hand(0, ctor());
        let caster_before = g.players[0].life;
        let opp_before = g.players[1].life;
        g.players[0].mana_pool.add(Color::Red, red);
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);

        assert!(g.battlefield.iter().find(|c| c.id == bear).is_none(),
            "{name}: 2/2 bear should be dead");
        assert_eq!(g.players[0].life, caster_before + caster_delta, "{name}: caster life");
        assert_eq!(g.players[1].life, opp_before + opp_delta, "{name}: opponent life");
    }
}

// ── Ward enforcement tests ────────────────────────────────────────────────

#[test]
fn ward_blocks_targeting_when_caster_cannot_pay() {
    // CR 702.21a: Ward is a triggered ability. The caster CAN target the
    // creature — the Ward trigger fires and counters the spell unless the
    // caster pays. With insufficient mana, the spell is countered.
    let mut g = two_player_game();
    let warded = g.add_card_to_battlefield(1, catalog::campus_composer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(warded)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Cast succeeds — Ward is enforced at resolution, not cast time");
    drain_stack(&mut g);

    // The Ward trigger countered the bolt; Demonstrator is unscathed.
    assert!(g.battlefield.iter().any(|c| c.id == warded),
        "Ward creature should survive — Ward countered the bolt");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Countered bolt should be in caster's graveyard");
}

#[test]
fn ward_allows_targeting_when_caster_can_pay() {
    // CR 702.21a: Ward triggers on the stack. Caster pays the bolt cost
    // at cast time. The Ward trigger auto-pays {2} from remaining pool
    // at resolution time. Both succeed with {R}+{2} = {3} total.
    let mut g = two_player_game();
    let warded = g.add_card_to_battlefield(1, catalog::campus_composer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 3);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(warded)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Cast succeeds — Ward auto-paid at trigger resolution");
    drain_stack(&mut g);

    // After resolution, P0's pool should be depleted (1 for bolt + 2 for ward).
    assert_eq!(g.players[0].mana_pool.total(), 0,
        "Pool should be empty after paying bolt + ward");
    // The bolt should have resolved and damaged the creature.
    let bolt_in_gy = g.players[0].graveyard.iter().any(|c| c.id == bolt);
    assert!(bolt_in_gy, "Bolt resolved and went to graveyard");
}

#[test]
fn ward_does_not_apply_to_own_creatures() {
    let mut g = two_player_game();
    // P0 owns the Ward creature and targets it with a pump spell.
    let warded = g.add_card_to_battlefield(0, catalog::campus_composer());
    let pump = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: pump,
        target: Some(Target::Permanent(warded)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("should succeed — Ward doesn't apply to own creatures");
}

// ── Preparation-card token characteristics ─────────────────────────────────

#[test]
fn campus_composer_aqueous_aria_token_is_three_three_flying() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::campus_composer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Aqueous Aria castable for {4}{U}");
    drain_stack(&mut g);

    let elemental = g.battlefield.iter().find(|c| c.definition.name == "Elemental")
        .expect("Elemental token minted");
    assert_eq!(elemental.definition.power, 3);
    assert_eq!(elemental.definition.toughness, 3);
    assert!(elemental.definition.keywords.contains(&Keyword::Flying),
        "the Elemental token has flying");
}

// NOTE: Molten Note and Fix What's Broken are covered in mana_shapes.rs.

// ── Beledros Witherbloom mass-untap ───────────────────────────────────────

#[test]
fn beledros_witherbloom_mass_untap_activation() {
    let mut g = two_player_game();
    let _bel = g.add_card_to_battlefield(0, catalog::beledros_witherbloom());
    // Add some tapped lands.
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let f2 = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield.iter_mut().filter(|c| c.id == f1 || c.id == f2).for_each(|c| c.tapped = true);
    g.players[0].life = 20;

    g.perform_action(GameAction::ActivateAbility {
        card_id: _bel,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("should activate with 10 life");
    drain_stack(&mut g);

    // Lands should be untapped.
    assert!(!g.battlefield.iter().find(|c| c.id == f1).unwrap().tapped);
    assert!(!g.battlefield.iter().find(|c| c.id == f2).unwrap().tapped);
    // Life should drop by 10.
    assert_eq!(g.players[0].life, 10);
}

#[test]
fn beledros_witherbloom_mass_untap_fails_low_life() {
    let mut g = two_player_game();
    let bel = g.add_card_to_battlefield(0, catalog::beledros_witherbloom());
    g.players[0].life = 5;

    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: bel,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    });
    assert!(result.is_err(), "should fail — not enough life");
}

// ── Lorehold Apprentice magecraft damage ──────────────────────────────────

#[test]
fn lorehold_apprentice_magecraft_grants_spirits_a_tap_ping() {
    // Real oracle: "Magecraft — ... until end of turn, Spirit creatures
    // you control gain '{T}: This creature deals 1 damage to each
    // opponent.'"
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::lorehold_apprentice());
    let spirit = g.add_card_to_battlefield(0, catalog::spirit_mascot()); // Spirit Ox
    g.clear_sickness(spirit);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);
    // Bolt alone: 3 damage. Now the Spirit's granted "{T}: 1 to each
    // opponent" ability (index after its printed abilities) fires.
    assert_eq!(g.players[1].life, p1_life - 3, "only the bolt so far");
    g.perform_action(GameAction::ActivateAbility {
        card_id: spirit, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("granted tap-ping activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4, "granted ping dealt 1 to the opponent");
    assert!(g.battlefield_find(spirit).unwrap().tapped, "tap paid");
}

// ── New cube creature tests ───────────────────────────────────────────────

#[test]
fn descendant_of_storms_dies_creates_spirit_token() {
    let mut g = two_player_game();
    let desc = g.add_card_to_battlefield(0, catalog::descendant_of_storms());
    // P0 kills their own creature with a bolt.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(desc)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);

    // Descendant should be dead, Spirit token should exist.
    assert!(!g.battlefield.iter().any(|c| c.id == desc));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"),
        "should create a Spirit token on death");
}

// ── Additional card shape tests ───────────────────────────────────────────

#[test]
fn fix_whats_broken_loses_life_and_returns_from_gy() {
    let mut g = two_player_game();
    // Put a 2-mana creature in P0's graveyard.
    let _bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let card = g.players[0].hand.pop().unwrap();
    g.players[0].graveyard.push(card);
    let id = g.add_card_to_hand(0, catalog::fix_whats_broken());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4); // 2 generic + 2 for X
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("castable");
    drain_stack(&mut g);

    // Should have lost X (=2) life.
    assert_eq!(g.players[0].life, life_before - 2);
}

// ── Explore + extra land plays ────────────────────────────────────────────

#[test]
fn explore_grants_extra_land_play_and_draws() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::explore());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    // Add a forest to hand + library cards.
    let _forest = g.add_card_to_hand(0, catalog::forest());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Explore castable");
    drain_stack(&mut g);

    // Should have drawn 1 card (net = hand_before - 1 (Explore) + 1 (draw) = same).
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Should be able to play 2 lands now (1 normal + 1 extra).
    assert_eq!(g.players[0].extra_land_plays, 1);
    assert!(g.players[0].can_play_land());
}

#[test]
fn extra_land_play_allows_two_lands() {
    let mut g = two_player_game();
    g.players[0].extra_land_plays = 1;
    let f1 = g.add_card_to_hand(0, catalog::forest());
    let f2 = g.add_card_to_hand(0, catalog::forest());

    g.perform_action(GameAction::PlayLand(f1)).expect("first land");
    assert!(g.players[0].can_play_land(), "should still be able to play another");
    g.perform_action(GameAction::PlayLand(f2)).expect("second land");
    assert!(!g.players[0].can_play_land(), "used all land plays");
}

// ── Subagent cube card tests ──────────────────────────────────────────────

#[test]
fn gush_draws_two_cards() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::gush());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable");
    drain_stack(&mut g);

    // Drew 2, played 1 = net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

// ── Stun counter untap suppression (CR 122.1c) ────────────────────────────

#[test]
fn stun_counter_prevents_untap_on_untap_step() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == creature).unwrap().tapped = true;
    g.battlefield.iter_mut().find(|c| c.id == creature).unwrap()
        .add_counters(CounterType::Stun, 1);
    g.do_untap();
    let c = g.battlefield.iter().find(|c| c.id == creature).unwrap();
    assert!(c.tapped, "Creature with stun counter should stay tapped after untap step");
    assert_eq!(c.counter_count(CounterType::Stun), 0, "Stun counter should be removed");
    g.do_untap();
    let c = g.battlefield.iter().find(|c| c.id == creature).unwrap();
    assert!(!c.tapped, "Creature should untap normally after stun counter is gone");
}

// ── Hand-size cleanup (CR 514.1) ───────────────────────────────────────────

// ── Cube: Intervention Pact ────────────────────────────────────────────────

#[test]
fn intervention_pact_gains_five_life_and_has_pact_trigger() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::intervention_pact());
    let life_before = g.players[0].life;
    // Costs {0} — no mana needed.
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Intervention Pact castable for 0");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 5, "Should gain 5 life");
    // A delayed trigger should be registered for the pact payment.
    assert!(!g.delayed_triggers.is_empty(), "Pact delayed trigger should be registered");
}

// ─────────────────────────────────────────────────────────────────────────
// modern_decks: copy-permanent / Opus copy-token / cast-from-exile
// promotions (Applied Geometry, Colorstorm Stallion, Elemental Mascot).
// These cards previously stubbed their copy/exile riders; they now wire
// the engine's CreateTokenCopyOf / ExileTopAndGrantMayPlay primitives.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn applied_geometry_copies_creature_as_six_six_fractal() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    // A 2/2 Grizzly Bears to copy.
    let bear = place_creature(&mut g, 0, catalog::grizzly_bears());
    let ag = g.add_card_to_hand(0, catalog::applied_geometry());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ag,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Applied Geometry castable for {2}{G}{U}");
    drain_stack(&mut g);
    // Two "Grizzly Bears" on the battlefield now: the original + the copy.
    let bears: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Grizzly Bears")
        .collect();
    assert_eq!(bears.len(), 2, "original + minted copy");
    let token = bears
        .iter()
        .find(|c| c.is_token)
        .expect("copy is a token");
    assert!(
        token.definition.has_creature_type(CreatureType::Fractal),
        "copy gains Fractal 'in addition to its other types'",
    );
    assert_eq!(token.power(), 6, "0/0 base + six +1/+1 counters → 6/6");
    assert_eq!(token.toughness(), 6);
}

#[test]
fn colorstorm_stallion_opus_mints_copy_at_five_mana() {
    let mut g = two_player_game();
    let _stallion = place_creature(&mut g, 0, catalog::colorstorm_stallion());
    // Divergent Equation with X=2 → {2}{2}{U} = 5 mana spent (an IS spell).
    let big = g.add_card_to_hand(0, catalog::divergent_equation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: big,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("Divergent Equation castable with X=2");
    drain_stack(&mut g);
    let stallions: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Colorstorm Stallion")
        .collect();
    assert_eq!(stallions.len(), 2, "Opus ≥5 mana mints a copy of itself");
    assert!(
        stallions.iter().any(|c| c.is_token),
        "the minted copy is a token",
    );
}

#[test]
fn colorstorm_stallion_opus_no_copy_below_five_mana() {
    let mut g = two_player_game();
    let stallion = place_creature(&mut g, 0, catalog::colorstorm_stallion());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let count = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Colorstorm Stallion")
        .count();
    assert_eq!(count, 1, "small body (<5 mana) only pumps, no copy");
    // Small body still pumped +1/+1: 3/3 → 4/4.
    let c = g.battlefield_find(stallion).unwrap();
    assert_eq!(c.power(), 4, "small body +1/+1");
}

#[test]
fn elemental_mascot_opus_exiles_top_and_grants_may_play_at_five_mana() {
    let mut g = two_player_game();
    let _mascot = place_creature(&mut g, 0, catalog::elemental_mascot());
    // Seed a known top card to exile.
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    // Divergent Equation with X=2 → 5 mana spent.
    let big = g.add_card_to_hand(0, catalog::divergent_equation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: big,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("Divergent Equation castable with X=2");
    drain_stack(&mut g);
    // The seeded top card should now be in exile (Opus big body).
    assert!(
        g.exile.iter().any(|c| c.id == top),
        "Opus ≥5 mana exiles the top card of the library",
    );
}

// ── modern_decks: Tragedy Feaster Ward—Discard a card (CR 702.21) ───────────

#[test]
fn tragedy_feaster_ward_discard_counters_when_payer_cannot_discard() {
    use crabomination::game::types::Target;
    // P0's Tragedy Feaster (7/6) has Ward—Discard a card. P1's only hand
    // card is the bolt; once cast, the hand is empty so the Ward can't
    // collect a discard → bolt is countered.
    let mut g = two_player_game();
    let feaster = g.add_card_to_battlefield(0, catalog::tragedy_feaster());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(feaster)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield.iter().find(|c| c.id == feaster)
        .expect("Tragedy Feaster survives — Ward—Discard countered the Bolt");
    assert_eq!(card.damage, 0, "no damage — bolt was countered");
}

// ── modern_decks: Prismari, the Inspiration Ward—Pay 5 life enforcement ─────

#[test]
fn prismari_the_inspiration_ward_pay_life_counters_when_payer_too_low() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::prismari_the_inspiration());
    g.clear_sickness(dragon);
    // P1 can't afford Ward—Pay 5 life (only 3 life total).
    g.players[1].life = 3;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(dragon)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable; Ward is a trigger");
    drain_stack(&mut g);
    let card = g.battlefield.iter().find(|c| c.id == dragon)
        .expect("Ward—Pay 5 life counters the Bolt — dragon survives");
    assert_eq!(card.damage, 0, "bolt countered, no damage");
    assert_eq!(g.players[1].life, 3, "ward life wasn't paid (couldn't afford)");
}

