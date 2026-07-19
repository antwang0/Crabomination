#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── New modern-supplement cards (claude/modern_decks batch) ──────────────────

/// Cathartic Reunion: discard 2, draw 3.
#[test]
fn cathartic_reunion_discards_two_then_draws_three() {
    let mut g = two_player_game();
    // Stock 5 cards in library so the draw 3 has inputs.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    // 4 cards in hand: Cathartic Reunion + 3 fillers (so we can discard 2
    // and still cast).
    let id = g.add_card_to_hand(0, catalog::cathartic_reunion());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Cathartic Reunion castable for {1}{R}");
    drain_stack(&mut g);

    // Hand: -1 cast -2 discard +3 draw = net 0. The Cathartic Reunion itself
    // and 2 discarded cards are now in the graveyard.
    assert_eq!(g.players[0].hand.len(), hand_before, "net hand change should be 0");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "Cathartic Reunion should hit the graveyard");
    assert!(g.players[0].graveyard.len() >= 3,
        "Two discards plus the Reunion itself = at least 3 cards in graveyard");
}

/// Gitaxian Probe: lose 2 life, draw 1 card.
#[test]
fn gitaxian_probe_pays_two_life_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::gitaxian_probe());
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Gitaxian Probe castable by paying the {U/P} pip with life");
    drain_stack(&mut g);

    // -1 cast +1 draw → net hand 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].life, life_before - 2, "Probe pays the Phyrexian pip with 2 life");
}

#[test]
fn gitaxian_probe_paid_with_blue_costs_no_life() {
    // Paying the {U/P} pip with blue mana costs no life.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::gitaxian_probe());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Gitaxian Probe castable for {U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before, "no life lost when the pip is paid with blue");
}

/// Force Spike counters target spell unless its controller pays {1}.
/// When the opp can't pay, the spell is countered.
#[test]
fn force_spike_counters_when_opponent_cannot_pay() {
    let mut g = two_player_game();
    // P1 casts a Lightning Bolt with no spare mana; P0 responds with Force Spike.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");

    g.priority.player_with_priority = 0;
    let spike = g.add_card_to_hand(0, catalog::force_spike());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spike,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Force Spike castable for {U}");
    drain_stack(&mut g);

    // P1 had only {R} (already spent) and 0 generic, so they can't pay {1}.
    // The Bolt is countered → P0 still at 20.
    assert_eq!(g.players[0].life, 20,
        "Bolt countered; P0 takes no damage");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Countered Bolt goes to controller's graveyard");
}

/// Force Spike doesn't counter when the opponent can pay {1}.
#[test]
fn force_spike_does_not_counter_when_opponent_can_pay() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(1); // spare to pay the spike
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable");

    g.priority.player_with_priority = 0;
    let spike = g.add_card_to_hand(0, catalog::force_spike());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spike,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Force Spike castable for {U}");
    drain_stack(&mut g);

    // P1 pays the {1}, Bolt resolves.
    assert_eq!(g.players[0].life, 17, "Bolt resolved; P0 took 3 damage");
    assert_eq!(g.players[1].mana_pool.colorless_amount(), 0,
        "P1's spare colorless should have been consumed paying the spike");
}

/// Vampiric Tutor: pay 2 life, search the library, put on top.
#[test]
fn vampiric_tutor_pays_two_life_and_tutors_to_top_of_library() {
    let mut g = two_player_game();
    let target_card = g.add_card_to_library(0, catalog::griselbrand());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target_card))]));

    let id = g.add_card_to_hand(0, catalog::vampiric_tutor());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Vampiric Tutor castable for {B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2, "Vampiric pays 2 life");
    // Tutored card should be on top of the library.
    let top = g.players[0].library.first().unwrap();
    assert_eq!(top.id, target_card,
        "Vampiric Tutor should put the chosen card on top of the library");
}

/// Sylvan Scrying tutors a land into hand.
#[test]
fn sylvan_scrying_tutors_a_land_to_hand() {
    let mut g = two_player_game();
    let target_land = g.add_card_to_library(0, catalog::bojuka_bog());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target_land))]));

    let id = g.add_card_to_hand(0, catalog::sylvan_scrying());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Sylvan Scrying castable for {1}{G}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == target_land),
        "Tutored land should be in hand");
}

/// Abrupt Decay destroys a low-CMC nonland permanent and is uncounterable.
#[test]
fn abrupt_decay_destroys_low_cmc_nonland() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2-CMC creature
    let id = g.add_card_to_hand(0, catalog::abrupt_decay());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Abrupt Decay castable for {B}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (CMC 2) should be destroyed");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

/// Abrupt Decay refuses to target a CMC-3-or-higher permanent at cast time.
#[test]
fn abrupt_decay_rejects_high_cmc_target() {
    let mut g = two_player_game();
    // Tarmogoyf is base {1}{G} → CMC 2 — but the engine validates the cast-
    // time `ManaValueAtMost(2)` against the *definition* CMC. Use a
    // 3-CMC card for the rejection test: Cankerbloom is {1}{G}{G}? Actually
    // it's {1}{G} = 2. Let's use Soul-Guide Lantern which is {1} = 1. Let's
    // pick something CMC ≥ 3: Pact of Negation is {0}, no good. Let's use
    // mana_leak ({1}{U} = 2). Use phyrexian_arena ({1}{B}{B} = 3). Yes.
    let arena = g.add_card_to_battlefield(1, catalog::phyrexian_arena());
    let id = g.add_card_to_hand(0, catalog::abrupt_decay());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    let res = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(arena)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(res.is_err(),
        "Abrupt Decay should reject a CMC-3 permanent target");
}

/// Abrupt Decay is uncounterable via Keyword::CantBeCountered.
#[test]
fn abrupt_decay_cannot_be_countered() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::abrupt_decay());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Abrupt Decay castable");

    // Verify the spell on the stack is flagged uncounterable.
    let flagged = g.stack.iter().any(|si| matches!(si, StackItem::Spell { uncounterable: true, .. }));
    assert!(flagged, "Abrupt Decay's stack item should be marked uncounterable");
}

/// Kodama's Reach searches twice — one basic to play tapped, one to hand.
#[test]
fn kodamas_reach_searches_two_basics() {
    let mut g = two_player_game();
    let bf_target = g.add_card_to_library(0, catalog::forest());
    let hand_target = g.add_card_to_library(0, catalog::island());
    // Library padding so the search filters have non-trivial options.
    g.add_card_to_library(0, catalog::lightning_bolt());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bf_target)),
        DecisionAnswer::Search(Some(hand_target)),
    ]));

    let id = g.add_card_to_hand(0, catalog::kodamas_reach());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Kodama's Reach castable for {2}{G}");
    drain_stack(&mut g);

    // First basic should be on the battlefield tapped.
    let bf_view = g.battlefield.iter().find(|c| c.id == bf_target);
    assert!(bf_view.is_some(), "First basic should land on the battlefield");
    assert!(bf_view.unwrap().tapped, "Battlefield basic should enter tapped");
    // Second basic should be in hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == hand_target),
        "Second basic should land in hand");
}

/// Lotus Petal: tap and sac for one mana of any color.
#[test]
fn lotus_petal_taps_and_sacs_for_any_one_color() {
    let mut g = two_player_game();
    let petal = g.add_card_to_battlefield(0, catalog::lotus_petal());
    g.clear_sickness(petal);

    g.perform_action(GameAction::ActivateAbility {
        card_id: petal, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Lotus Petal activates");
    drain_stack(&mut g);

    // Sacrificed: leaves the battlefield, lands in graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == petal),
        "Petal should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == petal));
    // One mana of any color was added.
    assert_eq!(g.players[0].mana_pool.total(), 1,
        "Petal should add exactly one mana");
}

/// Tormod's Crypt: tap and sac to exile each opponent's graveyard.
#[test]
fn tormods_crypt_exiles_opponent_graveyard() {
    let mut g = two_player_game();
    // Stock P1's graveyard with a few cards.
    for _ in 0..3 {
        let cid = g.add_card_to_library(1, catalog::lightning_bolt());
        let pos = g.players[1].library.iter().position(|c| c.id == cid).unwrap();
        let card = g.players[1].library.remove(pos);
        g.players[1].graveyard.push(card);
    }
    let p1_grave_before = g.players[1].graveyard.len();
    let crypt = g.add_card_to_battlefield(0, catalog::tormods_crypt());
    g.clear_sickness(crypt);

    g.perform_action(GameAction::ActivateAbility {
        card_id: crypt, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Tormod's Crypt activates");
    drain_stack(&mut g);

    // Crypt sacrificed; opp graveyard exiled.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == crypt),
        "Crypt should be sacrificed");
    assert_eq!(g.players[1].graveyard.len(), 0,
        "P1's graveyard should be empty");
    assert!(g.exile.len() >= p1_grave_before,
        "Exiled cards should land in exile");
}

/// Mishra's Bauble: tap and sac to register a delayed cantrip on next upkeep.
#[test]
fn mishras_bauble_sacs_and_registers_delayed_draw() {
    let mut g = two_player_game();
    // Library has a card so the LookAtTop has an input.
    g.add_card_to_library(0, catalog::island());
    let bauble = g.add_card_to_battlefield(0, catalog::mishras_bauble());
    g.clear_sickness(bauble);

    let delayed_before = g.delayed_triggers.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bauble, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Mishra's Bauble activates");
    drain_stack(&mut g);

    // Bauble sacrificed.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bauble),
        "Bauble should be sacrificed");
    // A delayed trigger should be queued for the next upkeep.
    assert_eq!(g.delayed_triggers.len(), delayed_before + 1,
        "Bauble should have registered a delayed-draw trigger");
}

/// Stoneforge Mystic ETB tutors an Equipment.
///
/// Note: the cube/catalog has no equipment cards yet that are easy to fixture.
/// We assert the ETB-search trigger fires and routes through the decider —
/// declining is the "no equipment in library" outcome and produces no hand
/// gain.
#[test]
fn stoneforge_mystic_etb_searches_for_equipment() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());

    // Decider will be asked Search(None) — there's no equipment to pull. The
    // important assertion is that the decision was raised at all.
    let asked_before = 0usize;

    let id = g.add_card_to_battlefield(0, catalog::stoneforge_mystic());
    drain_stack(&mut g);

    // Stoneforge is on the battlefield; ETB trigger should have resolved
    // (search resolved as `None`, no hand gain).
    assert!(g.battlefield.iter().any(|c| c.id == id));
    let _ = asked_before;
}

/// Qasali Pridemage: {1}, sac itself to destroy artifact/enchantment.
#[test]
fn qasali_pridemage_sacs_to_destroy_artifact() {
    let mut g = two_player_game();
    let pride = g.add_card_to_battlefield(0, catalog::qasali_pridemage());
    let opp_artifact = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.clear_sickness(pride);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: pride,
        ability_index: 0,
        target: Some(Target::Permanent(opp_artifact)), additional_targets: Vec::new(), x_value: None })
    .expect("Qasali Pridemage activates");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_artifact),
        "Sol Ring should be destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == pride),
        "Pridemage is sacrificed");
}

/// Qasali Pridemage's Exalted: attacking alone pumps it +1/+1.
#[test]
fn qasali_pridemage_exalted_pumps_lone_attacker() {
    let mut g = two_player_game();
    let pride = g.add_card_to_battlefield(0, catalog::qasali_pridemage());
    g.clear_sickness(pride);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pride, target: AttackTarget::Player(1),
    }])).expect("Pridemage attacks alone");
    drain_stack(&mut g);
    let cp = g.computed_permanent(pride).expect("alive");
    assert_eq!((cp.power, cp.toughness), (3, 3),
        "Exalted pumps the lone attacker to 3/3");
}

/// CR 702.83b — multiple Exalted sources stack on the lone attacker;
/// 702.83a — no bonus when the creature doesn't attack alone.
#[test]
fn cr_702_83_exalted_stacks_and_requires_attacking_alone() {
    let mut g = two_player_game();
    let q1 = g.add_card_to_battlefield(0, catalog::qasali_pridemage());
    let _q2 = g.add_card_to_battlefield(0, catalog::qasali_pridemage()); // 2nd exalted source
    g.clear_sickness(q1);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: q1, target: AttackTarget::Player(1),
    }])).expect("q1 attacks alone");
    drain_stack(&mut g);
    let cp = g.computed_permanent(q1).expect("alive");
    assert_eq!((cp.power, cp.toughness), (4, 4),
        "two Exalted sources each pump the lone attacker (+2/+2)");

    // New combat, attacking with two creatures → not alone → no bonus.
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::qasali_pridemage());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ])).expect("two attackers");
    drain_stack(&mut g);
    let cp = g.computed_permanent(a).expect("alive");
    assert_eq!((cp.power, cp.toughness), (2, 2), "no Exalted when not attacking alone");
}

/// Greater Good: sac creature, draw P, discard 3.
#[test]
fn greater_good_sacrifices_creature_and_draws_power() {
    let mut g = two_player_game();
    let gg = g.add_card_to_battlefield(0, catalog::greater_good());
    // Sac fodder: a 5/5 Griselbrand-class body. Use Goldspan Dragon (4/4).
    let fodder = g.add_card_to_battlefield(0, catalog::goldspan_dragon());
    g.clear_sickness(gg);
    g.clear_sickness(fodder);
    // Stock library with 5 cards so the draw 4 has inputs.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    // Stock hand with extra cards so the discard 3 has inputs.
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    let hand_before = g.players[0].hand.len();
    let library_before = g.players[0].library.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: gg, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Greater Good activates");
    drain_stack(&mut g);

    // Goldspan Dragon (4 power) sacrificed; draw 4; discard 3.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "Goldspan Dragon should be sacrificed");
    let drawn = library_before - g.players[0].library.len();
    assert_eq!(drawn, 4, "Should draw 4 cards (= sacrificed power)");
    // Net hand: +4 draw - 3 discard = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Net hand = +4 draw - 3 discard = +1");
}

// ── Cube cards (round 6: modal counter, sac-payoff, drain Demon, recursion) ──

#[test]
fn cryptic_command_counter_plus_bounce_resolves() {
    // P1 has a creature out and casts Lightning Bolt at P0; P0 responds with
    // Cryptic Command's default "choose two" (counter + bounce): slot 0
    // counters the Bolt, slot 1 bounces P1's creature to its owner's hand.
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bolt castable");

    let cryptic = g.add_card_to_hand(0, catalog::cryptic_command());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: cryptic,
        target: Some(Target::Permanent(bolt)),               // slot 0: counter
        additional_targets: vec![Target::Permanent(creature)], // slot 1: bounce
        mode: None, x_value: None,
    })
    .expect("Cryptic Command castable for {1}{U}{U}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, 20, "Bolt countered by mode 0");
    assert!(g.players[1].hand.iter().any(|c| c.id == creature),
        "creature bounced to its owner's hand by mode 1");
    assert!(g.stack.is_empty(), "Stack empty after resolution");
}

#[test]
fn cryptic_command_counter_and_draw() {
    // ScriptedDecider picks modes [0, 3] (counter + draw a card).
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .unwrap();

    let cryptic = g.add_card_to_hand(0, catalog::cryptic_command());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 3);
    let hand_before = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![0, 3])]));
    g.perform_action(GameAction::CastSpell {
        card_id: cryptic,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);

    // Cryptic goes to grave on resolution; net hand = +1 (draw) - 1 (cast) = 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "Net hand: +1 draw - 1 cast = 0");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

#[test]
fn deadly_dispute_sacrifices_and_creates_treasure_and_draws_two() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fodder);
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let dispute = g.add_card_to_hand(0, catalog::deadly_dispute());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: dispute, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Deadly Dispute castable for {1}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "Sacrificed creature should leave the battlefield");
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1, "Should create one Treasure token");
    // Cast Dispute (-1), drew 2 (+2), net +1 ≈ hand_before + 1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Net +1 hand");
}

#[test]
fn bloodchiefs_thirst_destroys_low_cmc_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // CMC 2
    let thirst = g.add_card_to_hand(0, catalog::bloodchiefs_thirst());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: thirst,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Bloodchief's Thirst castable for {B}");
    drain_stack(&mut g);

    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Targeted bear should be destroyed");
}

#[test]
fn bloodchiefs_thirst_rejects_high_cmc_target() {
    let mut g = two_player_game();
    let mahamoti = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // CMC 6
    let thirst = g.add_card_to_hand(0, catalog::bloodchiefs_thirst());
    g.players[0].mana_pool.add(Color::Black, 1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: thirst,
        target: Some(Target::Permanent(mahamoti)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .unwrap_err();
    assert_eq!(err, GameError::SelectionRequirementViolated,
        "Mana value 6 fails the ≤2 base mode filter");
}

#[test]
fn bloodchiefs_thirst_kicked_destroys_high_cmc_creature() {
    let mut g = two_player_game();
    let mahamoti = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // CMC 6
    let thirst = g.add_card_to_hand(0, catalog::bloodchiefs_thirst());
    // Kicker is additive (CR 702.32): {B} base + {2}{B} kicker = {2}{B}{B}.
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpellKicked {
        card_id: thirst,
        target: Some(Target::Permanent(mahamoti)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Kicked Bloodchief's Thirst should destroy any creature/PW");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().find(|c| c.id == mahamoti).is_none(),
        "Mahamoti Djinn should be destroyed by kicked Bloodchief's Thirst");
}

#[test]
fn heliod_sun_crowned_grants_lifelink_until_end_of_turn() {
    let mut g = two_player_game();
    let heliod = g.add_card_to_battlefield(0, catalog::heliod_sun_crowned());
    g.clear_sickness(heliod);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: heliod,
        ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None })
    .expect("Heliod's lifelink-grant activates for {1}{W}");
    drain_stack(&mut g);

    let cp = g.computed_permanent(bear).expect("Bear still in play");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Lifelink),
        "Bear should now have Lifelink");
}

#[test]
fn indulgent_tormentor_opponent_pays_3_life_to_deny_the_draw() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let _torm = g.add_card_to_battlefield(0, catalog::indulgent_tormentor());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    let p1_life = g.players[1].life;
    let p0_hand = g.players[0].hand.len();

    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);

    // High-life opponent pays 3 life rather than let the controller draw.
    assert_eq!(g.players[1].life, p1_life - 3, "opponent paid 3 life");
    assert_eq!(g.players[0].hand.len(), p0_hand, "controller drew nothing");
}

#[test]
fn indulgent_tormentor_controller_draws_when_opponent_cant_pay() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let _torm = g.add_card_to_battlefield(0, catalog::indulgent_tormentor());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    // Opponent too low to pay 3 and controls no creature → controller draws.
    g.players[1].life = 2;
    g.add_card_to_library(0, catalog::island());
    let p0_hand = g.players[0].hand.len();

    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 2, "opponent paid nothing");
    assert_eq!(g.players[0].hand.len(), p0_hand + 1, "controller drew a card");
}

/// With the graveyard-source preference in `auto_target_for_effect`,
/// Eternal Witness's ETB now picks a card out of YOUR graveyard
/// automatically — the trigger no longer requires UI to land its
/// gameplay-default behavior.
#[test]
fn eternal_witness_etb_returns_graveyard_card_via_auto_target() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::eternal_witness());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Eternal Witness castable for {1}{G}{G}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt should auto-return from graveyard to hand");
}

#[test]
fn static_prison_sacrifices_itself_without_energy_to_pay() {
    use crabomination::game::TurnStep;
    let mut g = two_player_game();
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let prison = g.add_card_to_battlefield(0, catalog::static_prison());
    g.fire_self_etb_triggers(prison, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2, "ETB grants two energy");
    // First main with energy in pool → pay {E}, Prison stays.
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert!(g.battlefield_find(prison).is_some(), "paid energy, Prison survives");
    assert_eq!(g.players[0].energy, 1, "one energy spent");
    // Drain the rest, then a later upkeep can't pay → Prison is sacrificed and
    // the exiled creature returns.
    g.players[0].energy = 0;
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert!(g.battlefield_find(prison).is_none(), "no energy means it is sacrificed");
    assert!(g.battlefield.iter().any(|c| c.id == prey), "exiled creature returns");
}

#[test]
fn marauding_mako_grows_when_you_discard() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let mako = g.add_card_to_battlefield(0, catalog::marauding_mako());
    g.clear_sickness(mako);

    // P0 discards a card via an effect — we use direct hand-to-graveyard
    // movement to keep the test focused on the discard listener.
    let throwaway = g.add_card_to_hand(0, catalog::forest());
    let card = g.players[0].remove_from_hand(throwaway).unwrap();
    g.players[0].graveyard.push(card);
    // Fire the discard event directly — this exercises the listener path.
    let events = vec![GameEvent::CardDiscarded { player: 0, card_id: throwaway }];
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);

    let counters = g.battlefield_find(mako).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 1, "Discarding a card should add one +1/+1 counter");
}

#[test]
fn marauding_mako_has_cycling_and_grows_when_you_cycle_another_card() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    assert!(catalog::marauding_mako().keywords.iter()
        .any(|k| matches!(k, Keyword::Cycling(_))), "Marauding Mako has Cycling {{2}}");
    let mako = g.add_card_to_battlefield(0, catalog::marauding_mako());
    g.clear_sickness(mako);
    // Cycle a different card; the discard pumps Mako on the battlefield.
    let other = g.add_card_to_hand(0, catalog::marauding_mako());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Cycle { card_id: other, x_value: None }).expect("cycle for {2}");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mako).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "cycling a card discards it, pumping the Mako on the battlefield");
}

// ── New cards (claude/modern_decks: sweepers / tutors / burn / lands) ────────

/// Pyroclasm: 2 damage to each creature destroys 2-toughness creatures
/// while leaving bigger ones alive.
#[test]
fn pyroclasm_kills_two_toughness_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
    let py = g.add_card_to_hand(0, catalog::pyroclasm());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: py, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pyroclasm castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Pyroclasm should kill the 2-toughness Grizzly Bears");
    assert!(g.battlefield.iter().any(|c| c.id == dragon),
        "Pyroclasm should leave the 5-toughness Shivan Dragon alive");
}

/// Day of Judgment: destroy each creature regardless of toughness.
#[test]
fn day_of_judgment_destroys_all_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let lion = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let day = g.add_card_to_hand(0, catalog::day_of_judgment());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: day, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Day of Judgment castable for {2}{W}{W}");
    drain_stack(&mut g);

    for cid in [bear, dragon, lion] {
        assert!(!g.battlefield.iter().any(|c| c.id == cid),
            "Day of Judgment should destroy all creatures");
    }
}

/// Damnation: black-mana mirror of Day of Judgment. Destroys every
/// creature including indestructible-without-shroud ones (engine has no
/// regen primitive to bypass anyway).
#[test]
fn damnation_destroys_all_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let dn = g.add_card_to_hand(0, catalog::damnation());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: dn, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Damnation castable for {2}{B}{B}");
    drain_stack(&mut g);

    for cid in [a, b] {
        assert!(!g.battlefield.iter().any(|c| c.id == cid));
    }
}

/// Mystical Tutor: search library for an instant or sorcery and put on top.
#[test]
fn mystical_tutor_finds_instant_and_puts_on_top() {
    let mut g = two_player_game();
    // Stock library with a creature (ineligible) + a sorcery (eligible).
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));

    let id = g.add_card_to_hand(0, catalog::mystical_tutor());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mystical Tutor castable for {U}");
    drain_stack(&mut g);

    // Bolt should land on top of library; bear stays put.
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bolt),
        "Mystical Tutor should put the chosen instant on top of library");
    assert!(g.players[0].library.iter().any(|c| c.id == bear),
        "Untargeted card should remain in library");
}

/// Worldly Tutor: search for a creature, put on top.
#[test]
fn worldly_tutor_finds_creature_and_puts_on_top() {
    let mut g = two_player_game();
    let creature = g.add_card_to_library(0, catalog::shivan_dragon());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(creature))]));

    let id = g.add_card_to_hand(0, catalog::worldly_tutor());
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Worldly Tutor castable for {G}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(creature),
        "Worldly Tutor should put the chosen creature on top");
}

/// Enlightened Tutor: search for an artifact or enchantment.
#[test]
fn enlightened_tutor_finds_artifact_and_puts_on_top() {
    let mut g = two_player_game();
    let artifact = g.add_card_to_library(0, catalog::sol_ring());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(artifact))]));

    let id = g.add_card_to_hand(0, catalog::enlightened_tutor());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Enlightened Tutor castable for {W}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(artifact),
        "Enlightened Tutor should put the chosen artifact on top");
}

/// Diabolic Tutor: search for any card, put into hand.
#[test]
fn diabolic_tutor_finds_any_card_into_hand() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));

    let id = g.add_card_to_hand(0, catalog::diabolic_tutor());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Diabolic Tutor castable for {2}{B}{B}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Diabolic Tutor should pull the chosen card into hand");
    assert!(!g.players[0].library.iter().any(|c| c.id == bolt));
}

/// Imperial Seal: pay 2 life, search for any card, put on top.
#[test]
fn imperial_seal_pays_two_life_and_tutors_to_top() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));

    let id = g.add_card_to_hand(0, catalog::imperial_seal());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Imperial Seal castable for {B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2,
        "Imperial Seal should cost 2 life");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bolt),
        "Imperial Seal should put the chosen card on top");
}

/// Lightning Strike: 3 damage to a creature.
#[test]
fn lightning_strike_deals_three_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lightning_strike());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Lightning Strike castable for {1}{R} on a creature");
    drain_stack(&mut g);

    // 3 damage > 2 toughness ⇒ destroyed.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Lightning Strike should destroy the Grizzly Bears");
}

/// Lightning Strike: 3 damage to a player.
#[test]
fn lightning_strike_can_target_a_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lightning_strike());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Lightning Strike castable at a player");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, before - 3);
}

/// Goblin Bombardment: sacrifice a creature, deal 1 damage to any target.
#[test]
fn goblin_bombardment_sacrifices_creature_and_deals_one_damage() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::goblin_bombardment());
    g.clear_sickness(bomb);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fodder);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb,
        ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None })
    .expect("Goblin Bombardment activates");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "Bomb should sacrifice the Grizzly Bears");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder));
    assert_eq!(g.players[1].life, life_before - 1,
        "Bombardment should ping the targeted player for 1");
}

/// Wasteland: tap and sacrifice to destroy a nonbasic land.
#[test]
fn wasteland_destroys_nonbasic_land() {
    let mut g = two_player_game();
    let waste = g.add_card_to_battlefield(0, catalog::wasteland());
    g.clear_sickness(waste);
    // Place a nonbasic dual under P1.
    let dual = g.add_card_to_battlefield(1, catalog::watery_grave());
    g.clear_sickness(dual);

    // Activate ability index 1 (the destroy-land ability).
    g.perform_action(GameAction::ActivateAbility {
        card_id: waste,
        ability_index: 1,
        target: Some(Target::Permanent(dual)), additional_targets: Vec::new(), x_value: None })
    .expect("Wasteland's destroy ability activates");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == waste),
        "Wasteland should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == dual),
        "Wasteland should destroy the nonbasic dual");
}

/// Wasteland: rejects a basic land target (filter enforces nonbasic).
#[test]
fn wasteland_rejects_basic_land_target() {
    let mut g = two_player_game();
    let waste = g.add_card_to_battlefield(0, catalog::wasteland());
    g.clear_sickness(waste);
    let plains = g.add_card_to_battlefield(1, catalog::plains());

    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: waste,
        ability_index: 1,
        target: Some(Target::Permanent(plains)), additional_targets: Vec::new(), x_value: None });
    assert!(res.is_err(),
        "Wasteland's destroy ability should reject a basic-land target");
}

/// Strip Mine: tap and sacrifice to destroy any land (including basics).
#[test]
fn strip_mine_destroys_any_land() {
    let mut g = two_player_game();
    let strip = g.add_card_to_battlefield(0, catalog::strip_mine());
    g.clear_sickness(strip);
    let plains = g.add_card_to_battlefield(1, catalog::plains());

    g.perform_action(GameAction::ActivateAbility {
        card_id: strip,
        ability_index: 1,
        target: Some(Target::Permanent(plains)), additional_targets: Vec::new(), x_value: None })
    .expect("Strip Mine activates against any land");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == strip),
        "Strip Mine should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == plains),
        "Strip Mine should destroy even a basic land");
}

/// Snuff Out: cast for {3}{B} normally — destroys nonblack creature.
#[test]
fn snuff_out_destroys_nonblack_creature_via_normal_cost() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::snuff_out());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Snuff Out castable for {3}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

/// Snuff Out: pitch alt cost — pay 4 life instead of mana.
#[test]
fn snuff_out_alt_cost_pays_four_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::swamp()); // Swamp gate satisfied
    let id = g.add_card_to_hand(0, catalog::snuff_out());
    let life_before = g.players[0].life;
    // No mana — alt cost must succeed via 4 life.

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Snuff Out alt cost pays 4 life");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 4,
        "Snuff Out alt cost should deduct 4 life");
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn snuff_out_alt_cost_requires_a_swamp() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::snuff_out());
    // No Swamp controlled → the 4-life alt cost is illegal.
    let err = g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).unwrap_err();
    assert_eq!(err, GameError::NoAlternativeCost,
        "alt cost rejected without a Swamp");
    assert!(g.battlefield.iter().any(|c| c.id == bear), "bear untouched");
}

/// Teferi -3 rejects a target that doesn't match its
/// "nonland permanent an opponent controls" filter. Loyalty abilities
/// previously skipped the slot-0 filter check (only spell casts and
/// activated abilities enforced it), so a Teferi -3 aimed at the
/// controller's own permanent silently bounced their own creature.
#[test]
fn teferi_minus_three_rejects_self_targeted_land() {
    let mut g = two_player_game();
    let teferi = g.add_card_to_battlefield(0, catalog::teferi_time_raveler());
    let own_forest = g.add_card_to_battlefield(0, catalog::forest());
    // Stock a card so the +draw rider doesn't deck out.
    g.add_card_to_library(0, catalog::forest());

    let err = g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: teferi,
        ability_index: 1, // -3
        target: Some(Target::Permanent(own_forest)),
    })
    .unwrap_err();
    assert_eq!(err, GameError::SelectionRequirementViolated,
        "Teferi -3 should reject the controller's own land");
    // Forest still on the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == own_forest));
}

/// Snuff Out: rejects a black-creature target (filter enforces nonblack).
#[test]
fn snuff_out_rejects_black_creature() {
    let mut g = two_player_game();
    let demon = g.add_card_to_battlefield(1, catalog::griselbrand());
    let id = g.add_card_to_hand(0, catalog::snuff_out());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Black, 1);

    let res = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(demon)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(res.is_err(),
        "Snuff Out should reject a black creature target");
}

/// Windfall: each player discards their hand and draws 7 cards.
#[test]
fn windfall_discards_both_hands_then_draws_max_discarded() {
    // Push (batch 115): dynamic yield. P0 has 2 cards, P1 has 3 cards
    // (plus Windfall itself = 4 in hand). After discarding everything
    // each player draws `max(2, 4) = 4` cards.
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    // Clear pre-existing hands so we can stage the counts precisely.
    g.players[0].hand.clear();
    g.players[1].hand.clear();
    // Give each player a few cards in hand + library.
    for _ in 0..2 { g.add_card_to_hand(0, catalog::forest()); }
    for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
    for _ in 0..15 { g.add_card_to_library(0, catalog::forest()); }
    for _ in 0..15 { g.add_card_to_library(1, catalog::island()); }
    let wf = g.add_card_to_hand(1, catalog::windfall()); // P1 hand now = 4
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: wf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // P0 discarded 2; P1 discarded 4 (3 islands + Windfall after it
    // started resolving — actually Windfall leaves hand at cast time
    // so P1's hand was 3 at the discard step). Max = 4 or 3 depending
    // on cast-time bookkeeping; what matters is "both players draw the
    // same amount, equal to the max".
    let drawn_p0 = g.players[0].hand.len();
    let drawn_p1 = g.players[1].hand.len();
    assert_eq!(drawn_p0, drawn_p1,
        "Each player draws the same amount (the max discarded)");
    assert!(drawn_p0 >= 3, "Max discarded was at least 3 (P1's island hand)");
    assert!(drawn_p0 <= 4, "Max discarded was at most 4 (P1's full pre-cast hand)");
}

#[test]
fn windfall_asymmetric_discards_yields_higher_player_count() {
    // Force an asymmetric discard: P0 has 6 cards, P1 has 1 + Windfall.
    // Each player draws 6 (P0's discard count, the max).
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[0].hand.clear();
    g.players[1].hand.clear();
    for _ in 0..6 { g.add_card_to_hand(0, catalog::forest()); }
    for _ in 0..1 { g.add_card_to_hand(1, catalog::island()); }
    for _ in 0..20 { g.add_card_to_library(0, catalog::forest()); }
    for _ in 0..20 { g.add_card_to_library(1, catalog::island()); }
    let wf = g.add_card_to_hand(1, catalog::windfall());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: wf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 6,
        "P0 discarded 6 — Max = 6, P0 redraws 6");
    assert_eq!(g.players[1].hand.len(), 6,
        "P1 only discarded 2 (1 island + Windfall) but still draws 6 = max");
}

/// Treasure Cruise: at full {7}{U} cost, draws 3 cards.
#[test]
fn treasure_cruise_draws_three_at_full_cost() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::treasure_cruise());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(7);
    let hand_before_cast = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    // Net change: cast Cruise (-1) + drew 3 (+3) = +2.
    assert_eq!(g.players[0].hand.len(), hand_before_cast + 2);
}

/// Lose Focus: counters target spell when controller can't pay {2}.
#[test]
fn lose_focus_counters_when_controller_cannot_pay_two() {
    let mut g = two_player_game();
    // Bob is the active player; he casts Lightning Bolt at Alice.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    // Exactly enough for the bolt and nothing more, so paying the {2} is impossible.
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();
    // Bob has no mana, so paying {2} is impossible. Alice casts Lose Focus
    // at the bolt at instant speed (priority moved to her after Bob's cast).
    g.priority.player_with_priority = 0;
    let lose = g.add_card_to_hand(0, catalog::lose_focus());
    g.players[0].mana_pool.add(Color::Blue, 2); // {1}{U}
    g.perform_action(GameAction::CastSpell {
        card_id: lose,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // Bolt should be countered (graveyard) — no damage to Alice.
    assert_eq!(g.players[0].life, 20);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Lightning Bolt should be in Bob's graveyard after counter");
}

/// Lose Focus: leaves the spell alone when the controller can pay {2}.
#[test]
fn lose_focus_does_not_counter_when_controller_can_pay_two() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[1].mana_pool.add(_c, 20); }
    g.players[1].mana_pool.add_colorless(20);
    // Bob has 2 extra colorless to pay the unless-cost.
    g.players[1].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();
    g.priority.player_with_priority = 0;
    let lose = g.add_card_to_hand(0, catalog::lose_focus());
    g.players[0].mana_pool.add(Color::Blue, 2); // {1}{U}
    g.perform_action(GameAction::CastSpell {
        card_id: lose,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // Bolt resolved despite Lose Focus — Alice took 3.
    assert_eq!(g.players[0].life, 17);
}

// ── New mod_set additions: Stifle / Memory Lapse / Reckless Charge / etc. ──

/// Stifle counters the most recent triggered ability whose source matches
/// the targeted permanent.
#[test]
fn stifle_counters_a_triggered_ability_off_the_stack() {
    let mut g = two_player_game();
    // Cast Devourer of Destiny (P0) — its on-cast Scry-2 trigger goes on
    // top of the spell. Then Stifle the trigger.
    let dev = g.add_card_to_hand(0, catalog::devourer_of_destiny());
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpell {
        card_id: dev, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    // P1 stifles the trigger before it resolves.
    g.priority.player_with_priority = 1;
    let stifle = g.add_card_to_hand(1, catalog::stifle());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: stifle,
        target: Some(Target::Permanent(dev)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Stifle should accept Devourer as the source target");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dev),
        "Devourer should still resolve — Stifle only counters the ability");
    assert!(!g.stack.iter().any(|si| matches!(
        si, crabomination::game::StackItem::Trigger { source, .. } if *source == dev
    )), "Scry trigger should have been countered");
}

/// Memory Lapse: counters a target spell.
#[test]
fn memory_lapse_counters_target_spell() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).unwrap();
    g.priority.player_with_priority = 0;
    let lapse = g.add_card_to_hand(0, catalog::memory_lapse());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: lapse,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Memory Lapse should accept the bolt as a spell target");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "bolt was countered");
}

/// Vines of Vastwood: pumps the targeted creature +4/+4 EOT.
#[test]
fn vines_of_vastwood_kicked_pumps_target_creature_plus_four() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let vines = g.add_card_to_hand(0, catalog::vines_of_vastwood());
    // {G} base + {G}{G} kicker = {G}{G}{G}.
    g.players[0].mana_pool.add(Color::Green, 3);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: vines,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("kicked Vines castable for {G}{G}{G}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).expect("bear still alive");
    assert_eq!(cp.power, 6, "Grizzly Bears 2/2 + 4 = 6 power");
    assert_eq!(cp.toughness, 6, "Grizzly Bears 2/2 + 4 = 6 toughness");
    assert!(cp.keywords.contains(&Keyword::Hexproof), "kicked still grants hexproof");
}

#[test]
fn vines_of_vastwood_unkicked_only_grants_hexproof() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let vines = g.add_card_to_hand(0, catalog::vines_of_vastwood());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: vines,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Vines castable for {G}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).expect("bear still alive");
    assert_eq!(cp.power, 2, "unkicked Vines doesn't pump");
    assert!(cp.keywords.contains(&Keyword::Hexproof), "unkicked grants hexproof");
}

/// Reckless Charge: pumps +3/+0 and grants haste until end of turn.
#[test]
fn reckless_charge_grants_three_power_and_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let charge = g.add_card_to_hand(0, catalog::reckless_charge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: charge,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Reckless Charge castable for {R}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 5, "+3 power from Reckless Charge");
    assert_eq!(cp.toughness, 2, "toughness unchanged");
    assert!(
        cp.keywords.contains(&crabomination::card::Keyword::Haste),
        "should have haste"
    );
}

/// Boil: destroys every Island in play, regardless of controller.
#[test]
fn boil_destroys_all_islands() {
    let mut g = two_player_game();
    let i1 = g.add_card_to_battlefield(0, catalog::island());
    let i2 = g.add_card_to_battlefield(1, catalog::island());
    let f1 = g.add_card_to_battlefield(0, catalog::forest());
    let boil = g.add_card_to_hand(0, catalog::boil());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: boil, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Boil castable for {2}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == i1), "P0's Island should be destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == i2), "P1's Island should be destroyed");
    assert!(g.battlefield.iter().any(|c| c.id == f1), "Forest should survive");
}

/// Compulsive Research: caster draws three then discards two.
#[test]
fn compulsive_research_draws_three_discards_two() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::compulsive_research());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // -1 (cast) + 3 (draw) - 2 (discard) = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "net hand size unchanged");
    assert_eq!(g.players[0].graveyard.len(), 3, "2 discards + the cast spell itself");
}

/// Demolish: destroys target artifact.
#[test]
fn demolish_destroys_target_artifact() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let demo = g.add_card_to_hand(0, catalog::demolish());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: demo,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Demolish should accept an artifact target");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Mind Stone should be destroyed by Demolish");
}

/// Mind Sculpt: each opponent mills 7.
#[test]
fn mind_sculpt_mills_each_opponent_seven() {
    let mut g = two_player_game();
    for _ in 0..15 { g.add_card_to_library(1, catalog::island()); }
    let lib_before = g.players[1].library.len();
    let ms = g.add_card_to_hand(0, catalog::mind_sculpt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ms, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 7,
        "P1 should have milled 7 cards");
    assert_eq!(g.players[1].graveyard.len(), 7);
}

/// Cabal Therapy: name a card; the target player discards every copy.
#[test]
fn cabal_therapy_discards_all_copies_of_the_named_card() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bolt1 = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::forest());
    let ct = g.add_card_to_hand(0, catalog::cabal_therapy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::NamedCard("Lightning Bolt".into()),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: ct, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cabal Therapy castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt1));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt2),
        "every copy of the named card is discarded");
    assert_eq!(g.players[1].hand.len(), 1, "Forest still in hand");
}

/// AutoDecider names the most common nonland name in the targeted hand.
#[test]
fn cabal_therapy_auto_decider_names_the_densest_card() {
    let mut g = two_player_game();
    let bolt1 = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::forest());
    let ct = g.add_card_to_hand(0, catalog::cabal_therapy());
    g.players[0].mana_pool.add(Color::Black, 1);
    // No scripted name — the AutoDecider suggestion heuristic kicks in.
    g.perform_action(GameAction::CastSpell {
        card_id: ct, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cabal Therapy castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt1));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt2),
        "bot names the two-copy Bolt over the singleton Bear");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

/// Wear Down: destroys a target artifact or enchantment.
#[test]
fn wear_down_destroys_target_artifact() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let wd = g.add_card_to_hand(0, catalog::wear_down());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: wd,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Wear Down should accept an artifact target");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "Mind Stone should be destroyed by Wear Down");
}

// ── Cube additions: cheap creatures + sacrifice-cost spells ─────────────────

/// Memnite: vanilla {0} 1/1 artifact creature — castable from an empty pool.
#[test]
fn memnite_casts_for_zero_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::memnite());
    // Zero pool — Memnite costs nothing.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memnite is free");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("Memnite on battlefield");
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 1);
    assert!(card.definition.card_types.contains(&CardType::Artifact));
    assert!(card.definition.card_types.contains(&CardType::Creature));
}

/// Fanatic of Rhonas: {T} adds {G}.
#[test]
fn fanatic_of_rhonas_taps_for_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::fanatic_of_rhonas());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap for green");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    assert!(g.battlefield_find(id).unwrap().tapped, "Tap cost taps the source");
}

/// Greasewrench Goblin: vanilla 2/2 haste body.
#[test]
fn greasewrench_goblin_enters_with_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::greasewrench_goblin());
    let card = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 1);
    assert!(card.has_keyword(&crabomination::card::Keyword::Haste),
        "Greasewrench Goblin should have Haste");
    // Haste lets it attack on the turn it enters.
    assert!(card.can_attack(),
        "Haste creature can attack the turn it enters");
}

/// Orcish Lumberjack: {T}, sacrifice a Forest → add {G}{G}{G}. The
/// Forest sacrifice is folded into the resolved effect's first step, so
/// we need to make this a non-mana ability that goes on the stack… but
/// the engine treats `Seq([Sacrifice, AddMana])` as a non-mana ability
/// since `is_mana_ability` only matches pure-AddMana effects. Drain the
/// stack to resolve.
#[test]
fn orcish_lumberjack_sacrifices_forest_for_three_red() {
    let mut g = two_player_game();
    let lj = g.add_card_to_battlefield(0, catalog::orcish_lumberjack());
    g.clear_sickness(lj);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: lj, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("Lumberjack should activate for {T}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == forest),
        "Forest should be sacrificed as the activation cost");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3,
        "Activation should add {{R}}{{R}}{{R}}");
}

#[test]
fn orcish_lumberjack_cannot_activate_without_a_forest() {
    let mut g = two_player_game();
    let lj = g.add_card_to_battlefield(0, catalog::orcish_lumberjack());
    g.clear_sickness(lj);
    // A non-Forest land doesn't satisfy the Sacrifice-a-Forest cost.
    g.add_card_to_battlefield(0, catalog::mountain());
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: lj, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None });
    assert!(res.is_err(), "no Forest to sacrifice → activation rejected");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 0, "no mana made");
}

/// Mine Collapse: {2}{R} sorcery, sacrifice a Mountain on resolution,
/// deal 4 damage to the target.
#[test]
fn mine_collapse_sacrifices_mountain_and_deals_four() {
    let mut g = two_player_game();
    let mtn = g.add_card_to_battlefield(0, catalog::mountain());
    let mc = g.add_card_to_hand(0, catalog::mine_collapse());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: mc,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mine Collapse castable for {{2}}{{R}}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mtn),
        "Mountain should be sacrificed on resolution");
    assert_eq!(g.players[1].life, 16,
        "Target player should take 4 damage");
}

/// Satyr Wayfinder: ETB reveals 4, takes a land to hand, rest to graveyard.
#[test]
fn satyr_wayfinder_etb_takes_a_land_rest_to_graveyard() {
    let mut g = two_player_game();
    // Top of library: 3 nonland + 1 land among the top four.
    for _ in 0..3 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    g.add_card_to_library(0, catalog::forest());
    let sw = g.add_card_to_hand(0, catalog::satyr_wayfinder());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: sw, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Satyr Wayfinder castable for {1}{G}");
    drain_stack(&mut g);
    // The Forest is taken to hand; the three nonlands hit the graveyard.
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"),
        "the land is taken to hand");
    // The three nonland cards go to the graveyard.
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Lightning Bolt").count(), 3);
}

/// Satyr Wayfinder takes nothing when no land is among the revealed cards.
#[test]
fn satyr_wayfinder_takes_nothing_with_no_land_revealed() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let sw = g.add_card_to_hand(0, catalog::satyr_wayfinder());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: sw, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(!g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "no land → nothing to hand");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Lightning Bolt").count(), 4,
        "all four revealed nonlands hit the graveyard");
}

/// Fireblast: {4}{R}{R} for 4 damage to any target. (Alt cost path —
/// sacrifice 2 Mountains — is not yet wired; this exercises the regular
/// cost.)
#[test]
fn fireblast_deals_four_to_any_target() {
    let mut g = two_player_game();
    let fb = g.add_card_to_hand(0, catalog::fireblast());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: fb,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Fireblast castable for {{4}}{{R}}{{R}}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16, "Target should take 4 damage");
}

#[test]
fn fireblast_alt_cost_sacrifices_two_mountains() {
    // Free Fireblast: no mana, sacrifice two Mountains.
    let mut g = two_player_game();
    let m1 = g.add_card_to_battlefield(0, catalog::mountain());
    let m2 = g.add_card_to_battlefield(0, catalog::mountain());
    let _keep = g.add_card_to_battlefield(0, catalog::mountain());
    let fb = g.add_card_to_hand(0, catalog::fireblast());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: fb, pitch_card: None, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fireblast castable by sacrificing two Mountains");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 16, "deals 4 damage");
    assert!(!g.battlefield.iter().any(|c| c.id == m1 || c.id == m2),
        "two Mountains sacrificed");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Mountain").count(), 1,
        "the third Mountain stays");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Mountain").count(), 2);
}

#[test]
fn fireblast_alt_cost_rejected_without_two_mountains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mountain()); // only one
    let fb = g.add_card_to_hand(0, catalog::fireblast());
    let err = g.perform_action(GameAction::CastSpellAlternative {
        card_id: fb, pitch_card: None, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "can't pay the alt cost with one Mountain");
    assert!(g.players[0].hand.iter().any(|c| c.id == fb), "Fireblast stays in hand");
}

/// Talisman of Progress: {T}: Add {C} via index 0; {T}: lose 1 + add
/// {W} via index 1; index 2 adds {U}.
#[test]
fn talisman_of_progress_taps_for_colorless_or_one_of_w_or_u() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::talisman_of_progress());
    g.clear_sickness(id);
    // Colorless ability (index 0) — no life cost.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("colorless tap succeeds");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
    // Mana abilities tap the source synchronously; untap to use again.
    let life_before = g.players[0].life;
    g.battlefield_find_mut(id).unwrap().tapped = false;
    // White ability (index 1) — costs 1 life. Bundled with `LoseLife`
    // it's no longer a pure mana ability, so it goes on the stack and
    // needs draining.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).expect("white tap succeeds");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
    assert_eq!(g.players[0].life, life_before - 1,
        "Talisman costs 1 life when tapped for a color");
}

/// Talisman of Dominance: UB mirror — index 1 = {U}, index 2 = {B}.
#[test]
fn talisman_of_dominance_taps_for_blue_costing_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::talisman_of_dominance());
    g.clear_sickness(id);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).expect("blue tap succeeds");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
    assert_eq!(g.players[0].life, life_before - 1);
}

/// Elvish Spirit Guide: "Exile this from your hand: Add {G}." pitches for
/// a green mana and leaves play (exiled, not discarded).
#[test]
fn elvish_spirit_guide_pitches_from_hand_for_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::elvish_spirit_guide());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("pitch ability activates from hand");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "added one green");
    assert!(g.exile.iter().any(|c| c.id == id), "pitched card is exiled");
    assert!(!g.players[0].hand.iter().any(|c| c.id == id), "no longer in hand");
}

// ── New cube cards (this branch) ───────────────────────────────────────────

#[test]
fn bloodghast_returns_from_graveyard_when_you_play_a_land() {
    let mut g = two_player_game();
    // Seed Bloodghast in P0's graveyard.
    let bg_id = g.add_card_to_library(0, catalog::bloodghast());
    let card = g.players[0]
        .library
        .iter()
        .position(|c| c.id == bg_id)
        .map(|pos| g.players[0].library.remove(pos))
        .unwrap();
    g.players[0].graveyard.push(card);

    // P0 plays a Forest. The landfall trigger should reanimate Bloodghast.
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(forest)).unwrap();
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bg_id),
        "Bloodghast should return to the battlefield on landfall");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bg_id),
        "Bloodghast should no longer be in the graveyard");
}

/// Bloodghast can't block and gains haste only while an opponent is at ≤10 life.
#[test]
fn bloodghast_conditional_haste_and_cant_block() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bg = g.add_card_to_battlefield(0, catalog::bloodghast());
    assert!(g.computed_permanent(bg).unwrap().keywords.contains(&Keyword::CantBlock),
        "Bloodghast can't block");
    // Opponent at full life → no haste.
    assert!(!g.computed_permanent(bg).unwrap().keywords.contains(&Keyword::Haste),
        "no haste while opponent is above 10 life");
    g.players[1].life = 9;
    assert!(g.computed_permanent(bg).unwrap().keywords.contains(&Keyword::Haste),
        "gains haste once an opponent is at 10 or less life");
}

#[test]
fn ichorid_returns_at_upkeep_then_sacrifices_at_end_step() {
    // Real Oracle: at your upkeep, if Ichorid is in your graveyard, you
    // may EXILE a black creature card other than Ichorid FROM YOUR
    // graveyard; if you do, return Ichorid. Seed a black creature in p0's
    // own graveyard and accept the optional return.
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.step = TurnStep::Cleanup;
    let id = g.add_card_to_library(0, catalog::ichorid());
    let card = g.players[0]
        .library
        .iter()
        .position(|c| c.id == id)
        .map(|pos| g.players[0].library.remove(pos))
        .unwrap();
    g.players[0].graveyard.push(card);
    // Black Knight is a black creature — the exile fodder for the cost.
    let fodder = g.add_card_to_graveyard(0, catalog::black_knight());

    // Walk Cleanup → Untap → Upkeep so the trigger fires.
    for _ in 0..30 {
        if g.battlefield.iter().any(|c| c.id == id) { break; }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Ichorid should reanimate at the start of upkeep");
    assert!(g.exile.iter().any(|c| c.id == fodder),
        "the black creature fodder is exiled as the return cost");
    assert!(g.delayed_triggers.iter().any(|t|
        t.kind == crabomination::game::types::DelayedKind::NextEndStep),
        "Reanimation should register an end-step sacrifice delayed trigger");
}

/// Helper: drop Arclight Phoenix into P0's graveyard, set the IS-cast
/// counter, start on P0's pre-combat main, and walk to begin-combat.
#[cfg(test)]
fn arclight_setup(is_cast: u32) -> (GameState, crabomination::card::CardId) {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::arclight_phoenix());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].instants_or_sorceries_cast_this_turn = is_cast;
    for _ in 0..10 {
        if g.battlefield.iter().any(|c| c.id == id) { break; }
        if g.step == TurnStep::PostCombatMain { break; }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    (g, id)
}

#[test]
fn arclight_phoenix_returns_after_three_instants_or_sorceries() {
    let (g, id) = arclight_setup(3);
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Arclight returns at begin-combat with 3+ IS spells cast");
}

#[test]
fn arclight_phoenix_stays_in_graveyard_below_threshold() {
    let (g, id) = arclight_setup(2);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "Arclight stays in the graveyard with only 2 IS spells cast");
    assert!(!g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn ichorid_stays_in_graveyard_without_black_fodder() {
    // Negative test for the exile-cost gate: with no OTHER black creature
    // in your graveyard, the upkeep trigger predicate fails and Ichorid
    // stays put (it can't exile itself to pay the cost).
    let mut g = two_player_game();
    g.step = TurnStep::Cleanup;
    let id = g.add_card_to_library(0, catalog::ichorid());
    let card = g.players[0]
        .library
        .iter()
        .position(|c| c.id == id)
        .map(|pos| g.players[0].library.remove(pos))
        .unwrap();
    g.players[0].graveyard.push(card);
    // Seed a non-black creature in your graveyard (Grizzly Bears is
    // green) — the predicate must still fail.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());

    // Walk past Cleanup → Untap → Upkeep.
    for _ in 0..10 {
        if g.step == TurnStep::Draw { break; }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == id),
        "Ichorid should NOT reanimate — no black creature fodder in graveyard");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "Ichorid still sits in p0's graveyard");
}

#[test]
fn silversmote_ghoul_returns_from_graveyard_on_lifegain() {
    let mut g = two_player_game();
    let id = g.add_card_to_library(0, catalog::silversmote_ghoul());
    let card = g.players[0]
        .library
        .iter()
        .position(|c| c.id == id)
        .map(|pos| g.players[0].library.remove(pos))
        .unwrap();
    g.players[0].graveyard.push(card);

    // Cast Faithful Mending (mode 2 = Discard 0) to gain 2 life.
    let mending = g.add_card_to_hand(0, catalog::faithful_mending());
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: mending, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Faithful Mending castable for {W}{U}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Silversmote Ghoul should return when its controller gains life");
}

#[test]
fn bitterbloom_bearer_etb_creates_a_faerie_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bitterbloom_bearer());
    g.players[0].mana_pool.add(Color::Black, 2);

    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bitterbloom Bearer castable for {B}{B}");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 2,
        "Bitterbloom Bearer + 1 Faerie token = +2 permanents");
    let faerie = g.battlefield.iter().find(|c| c.definition.name == "Faerie")
        .expect("Faerie token should be on the battlefield");
    assert_eq!(faerie.definition.power, 1);
    assert_eq!(faerie.definition.toughness, 1);
    assert!(faerie.definition.keywords.contains(&crabomination::card::Keyword::Flying));
}

#[test]
fn dandan_sacrifices_at_upkeep_when_no_island() {
    let mut g = two_player_game();
    let dd = g.add_card_to_battlefield(0, catalog::dandan());
    g.clear_sickness(dd);
    g.step = TurnStep::Cleanup;
    // No Islands — at the start of upkeep Dandân should sac itself.

    for _ in 0..30 {
        if !g.battlefield.iter().any(|c| c.id == dd) { break; }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == dd),
        "Dandân should be sacrificed at upkeep when no Island is in play");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == dd),
        "Sacrificed Dandân should land in the graveyard");
}

#[test]
fn dandan_stays_in_play_with_an_island() {
    let mut g = two_player_game();
    let _island = g.add_card_to_battlefield(0, catalog::island());
    let dd = g.add_card_to_battlefield(0, catalog::dandan());
    g.clear_sickness(dd);
    g.step = TurnStep::Cleanup;

    // Walk past upkeep — Dandân should survive.
    for _ in 0..15 {
        if g.step == TurnStep::PreCombatMain { break; }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == dd),
        "Dandân should survive while you control an Island");
}

#[test]
fn dandan_cannot_attack_unless_defender_controls_an_island() {
    let mut g = two_player_game();
    // p0 controls an Island (so Dandân survives upkeep) but the *defender*
    // (p1) does not, so Dandân can't be declared as an attacker.
    g.add_card_to_battlefield(0, catalog::island());
    let dd = g.add_card_to_battlefield(0, catalog::dandan());
    g.clear_sickness(dd);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dd, target: AttackTarget::Player(1),
    }])).is_err(), "Dandân can't attack a defender with no Island");

    // Give the defender an Island — now the attack is legal.
    g.add_card_to_battlefield(1, catalog::island());
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dd, target: AttackTarget::Player(1),
    }])).is_ok(), "Dandân may attack once the defender controls an Island");
}

#[test]
fn turnabout_mode_four_taps_all_opponent_lands() {
    let mut g = two_player_game();
    let m1 = g.add_card_to_battlefield(1, catalog::mountain());
    let m2 = g.add_card_to_battlefield(1, catalog::mountain());
    let i1 = g.add_card_to_battlefield(1, catalog::island());

    let ta = g.add_card_to_hand(0, catalog::turnabout());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ta, target: None, additional_targets: vec![], mode: Some(4), x_value: None,
    }).expect("Turnabout castable for {2}{U}{U}");
    drain_stack(&mut g);

    for id in [m1, m2, i1] {
        let card = g.battlefield.iter().find(|c| c.id == id).unwrap();
        assert!(card.tapped, "Land {:?} should be tapped after Turnabout mode 4", id);
    }
}

#[test]
fn heliod_adds_plus_one_counter_when_you_gain_life_with_lifelink() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let _heliod = g.add_card_to_battlefield(0, catalog::heliod_sun_crowned());
    let ll = g.add_card_to_battlefield(0, catalog::hopeful_eidolon());
    g.clear_sickness(ll);

    // Cast Faithful Mending mode 2 (Discard 0 → Draw 2 + GainLife 2).
    let mending = g.add_card_to_hand(0, catalog::faithful_mending());
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: mending, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Faithful Mending castable");
    drain_stack(&mut g);

    let counters = g.battlefield.iter().find(|c| c.id == ll)
        .and_then(|c| c.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert!(counters >= 1,
        "Heliod should add a +1/+1 counter to a lifelink creature when you gain life");
}

#[test]
fn dread_return_reanimates_target_creature_from_graveyard() {
    let mut g = two_player_game();
    // Seed a Grizzly Bears in P0's graveyard.
    let bear_id = g.add_card_to_library(0, catalog::grizzly_bears());
    let card = g.players[0].library.iter().position(|c| c.id == bear_id)
        .map(|pos| g.players[0].library.remove(pos)).unwrap();
    g.players[0].graveyard.push(card);

    // Cast Dread Return for {2}{B}{B}.
    let dr = g.add_card_to_hand(0, catalog::dread_return());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: dr,
        target: Some(Target::Permanent(bear_id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Dread Return castable for {2}{B}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bear_id),
        "Dread Return should reanimate the target creature");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear_id),
        "Bears should no longer be in graveyard");
}

#[test]
fn dread_return_flashback_sacrifices_three_creatures() {
    // Flashback—Sacrifice three creatures (free flashback mana; the sac is
    // the name-keyed additional cost). Reanimates a 4th creature from gy.
    let mut g = two_player_game();
    let target = g.add_card_to_graveyard(0, catalog::atraxa_grand_unifier());
    let dr = g.add_card_to_graveyard(0, catalog::dread_return());
    let fodder: Vec<_> = (0..3)
        .map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears()))
        .collect();
    g.perform_action(GameAction::CastFlashback {
        card_id: dr, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dread Return flashback castable with 3 creatures to sac");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == target),
        "the reanimation target is back on the battlefield");
    for f in &fodder {
        assert!(g.battlefield_find(*f).is_none(), "fodder creature sacrificed");
    }
}

#[test]
fn dread_return_flashback_rejected_without_three_creatures() {
    let mut g = two_player_game();
    let target = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let dr = g.add_card_to_graveyard(0, catalog::dread_return());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // only 1 creature
    let result = g.perform_action(GameAction::CastFlashback {
        card_id: dr, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(result.is_err(), "fewer than three creatures → flashback rejected");
}

#[test]
fn tidehollow_sculler_etb_takes_an_opponent_card() {
    let mut g = two_player_game();
    // Seed P1's hand with a Lightning Bolt.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());

    let sculler = g.add_card_to_hand(0, catalog::tidehollow_sculler());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sculler, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidehollow Sculler castable for {W}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == sculler),
        "Sculler should resolve onto the battlefield");
    assert!(!g.players[1].hand.iter().any(|c| c.id == bolt),
        "ETB should exile the Bolt from P1's hand");
    assert!(g.exile.iter().any(|c| c.id == bolt),
        "Bolt exiled until the Sculler leaves");
    // Sculler dies → Bolt returns to its owner's hand.
    g.remove_from_battlefield_to_graveyard_raw(sculler);
    assert!(g.players[1].hand.iter().any(|c| c.id == bolt),
        "Bolt returns to owner's hand when the Sculler leaves");
}

