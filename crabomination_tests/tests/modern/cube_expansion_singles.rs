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

// ── New cube cards (push claude/modern_decks) ──────────────────────────

#[test]
fn collective_brutality_escalate_runs_two_modes_paying_discard() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // P0 needs a spare card to pay the escalate "discard a card" cost.
    let fodder = g.add_card_to_hand(0, catalog::island());
    // P1 holds a card to be discarded by mode 1.
    g.add_card_to_hand(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::collective_brutality());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Escalate to modes 1 (opp discards) + 2 (drain). Base mode = 1.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![1, 2])]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Collective Brutality castable");
    drain_stack(&mut g);
    // Escalate cost discarded P0's spare card.
    assert!(!g.players[0].hand.iter().any(|c| c.id == fodder), "escalate cost discarded a card");
    // Mode 1 made the opponent discard their card; mode 2 drained 2.
    assert!(g.players[1].hand.is_empty(), "opponent discarded to mode 1");
    assert_eq!(g.players[1].life, 18, "mode 2 drains opponent for 2");
    assert_eq!(g.players[0].life, 22, "mode 2 gains controller 2");
}

#[test]
fn collective_brutality_mode_two_drains() {
    let mut g = two_player_game();
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::collective_brutality());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(2), x_value: None,
    }).expect("Collective Brutality castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2);
    assert_eq!(g.players[0].life, my_life + 2);
}

#[test]
fn cam_and_farrik_pumps_on_noncreature_cast() {
    let mut g = two_player_game();
    let cam = g.add_card_to_battlefield(0, catalog::cam_and_farrik());
    g.clear_sickness(cam);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let p_before = g.battlefield.iter().find(|c| c.id == cam).unwrap().power();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let p_after = g.battlefield.iter().find(|c| c.id == cam).unwrap().power();
    assert_eq!(p_after, p_before + 2);
}

#[test]
fn keen_eyed_curator_exiles_graveyard_cards_and_buffs_at_four_types() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::keen_eyed_curator());
    g.clear_sickness(id);
    // Seed the opponent's graveyard with four distinct card types.
    let creature = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let instant = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let land = g.add_card_to_graveyard(1, catalog::forest());
    let artifact = g.add_card_to_graveyard(1, catalog::ornithopter());
    // Base 2/2 with nothing exiled.
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == id).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3));
    // Exile each, tagging them with the Curator (one activation per card).
    for (i, card) in [creature, instant, land, artifact].into_iter().enumerate() {
        for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
        g.players[0].mana_pool.add_colorless(20);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: Some(Target::Permanent(card)), additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("exile a graveyard card");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == card), "card {i} is exiled");
    }
    // Four card types among exiled-with cards → +4/+4 and trample.
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == id).unwrap();
    assert_eq!((c.power, c.toughness), (7, 7), "+4/+4 at four card types");
    assert!(c.keywords.contains(&crabomination::card::Keyword::Trample));
}

#[test]
fn intervention_pact_gains_three_life_and_sets_delayed_trigger() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::intervention_pact());
    // Free cast ({0})
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 5);
    assert!(!g.delayed_triggers.is_empty(), "Should have a delayed PayOrLoseGame trigger");
}

#[test]
fn gush_draws_two_cards() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::gush());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // Cast -1 (Gush) from hand + draw 2 = net +1
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn gush_alt_cost_returns_two_islands_and_draws() {
    // Free Gush: pay no mana, return two Islands you control to hand.
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let i1 = g.add_card_to_battlefield(0, catalog::island());
    let i2 = g.add_card_to_battlefield(0, catalog::island());
    let _plains = g.add_card_to_battlefield(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::gush());
    // No mana in pool — only the alt cost can pay.
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Gush castable via return-two-Islands alt cost");
    drain_stack(&mut g);

    // Both Islands bounced to hand; Plains stayed.
    assert!(!g.battlefield.iter().any(|c| c.id == i1 || c.id == i2),
        "two Islands returned to hand");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Plains").count(), 1);
    // Drew two cards (Gush itself left hand to the stack/graveyard).
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Gush"));
}

#[test]
fn gush_alt_cost_rejected_without_two_islands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::island()); // only one Island
    let id = g.add_card_to_hand(0, catalog::gush());
    let err = g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "can't pay Gush's alt cost with only one Island");
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "Gush stays in hand on failure");
}

// ── Cube expansion cards ──────────────────────────────────────────────────────

#[test]
fn back_to_basics_prevents_nonbasic_land_untap() {
    let mut g = two_player_game();
    let _btb = g.add_card_to_battlefield(0, catalog::back_to_basics());
    // Tap a nonbasic land.
    let nonbasic = g.add_card_to_battlefield(0, catalog::razortide_bridge());
    g.battlefield.iter_mut().find(|c| c.id == nonbasic).unwrap().tapped = true;
    // Also tap a basic land for comparison.
    let basic = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield.iter_mut().find(|c| c.id == basic).unwrap().tapped = true;

    g.do_untap();

    // Basic land should untap.
    assert!(!g.battlefield.iter().find(|c| c.id == basic).unwrap().tapped,
        "Basic land should untap normally");
    // Nonbasic land should stay tapped.
    assert!(g.battlefield.iter().find(|c| c.id == nonbasic).unwrap().tapped,
        "Nonbasic land should stay tapped under Back to Basics");
}

// ── Overload cards ────────────────────────────────────────────────────────────

#[test]
fn blustersquall_taps_target_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::blustersquall());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    assert!(g.battlefield.iter().find(|c| c.id == target).unwrap().tapped,
        "Blustersquall should tap target creature");
}

#[test]
fn blustersquall_overload_taps_all_opponent_creatures() {
    let mut g = two_player_game();
    let c1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let own = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::blustersquall());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: spell, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    assert!(g.battlefield.iter().find(|c| c.id == c1).unwrap().tapped);
    assert!(g.battlefield.iter().find(|c| c.id == c2).unwrap().tapped);
    assert!(!g.battlefield.iter().find(|c| c.id == own).unwrap().tapped,
        "Own creatures should NOT be tapped by Overload");
}

#[test]
fn electrickery_deals_1_to_target() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::electrickery());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    let bear = g.battlefield.iter().find(|c| c.id == target).unwrap();
    assert_eq!(bear.damage, 1, "Electrickery should deal 1 damage");
}

#[test]
fn electrickery_overload_deals_1_to_each_opp_creature() {
    let mut g = two_player_game();
    let c1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::electrickery());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: spell, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    for id in [c1, c2] {
        let c = g.battlefield.iter().find(|c| c.id == id).unwrap();
        assert_eq!(c.damage, 1, "Electrickery Overload should deal 1 to each");
    }
}

#[test]
fn teleportal_pumps_and_grants_unblockable() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::teleportal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(creature)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);

    let c = g.battlefield.iter().find(|c| c.id == creature).unwrap();
    assert_eq!(c.power(), 3, "Should get +1/+0");
    assert!(c.has_keyword(&crabomination::card::Keyword::Unblockable));
}

// ── Modern cube supplement ──────────────────────────────────────────────────

#[test]
fn dreadhorde_arcanist_attack_returns_instant_from_graveyard() {
    let mut g = two_player_game();
    // Put an instant card in P0's graveyard.
    let bolt_id = g.add_card_to_library(0, catalog::lightning_bolt());
    let pos = g.players[0].library.iter().position(|c| c.id == bolt_id).unwrap();
    let bolt_card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(bolt_card);

    let arcanist = g.add_card_to_battlefield(0, catalog::dreadhorde_arcanist());
    g.clear_sickness(arcanist);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;

    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: arcanist,
        target: AttackTarget::Player(1),
    }]))
    .expect("Dreadhorde Arcanist attacks");
    drain_stack(&mut g);

    // The attack trigger should move the Lightning Bolt from graveyard to hand.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Arcanist attack should return an IS card from graveyard to hand");
    assert_eq!(g.players[0].graveyard.len(), gy_before - 1,
        "Graveyard should lose one card");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_id),
        "Lightning Bolt should now be in hand");
}

#[test]
fn baleful_mastery_full_cost_exiles_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::baleful_mastery());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);

    let p1_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Baleful Mastery castable for {3}{B}");
    drain_stack(&mut g);

    // Bear should be exiled.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be exiled from battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile");
    // At full cost, opponent should NOT draw a card.
    assert_eq!(g.players[1].hand.len(), p1_hand_before,
        "At full cost, opponent should not draw a card");
}

#[test]
fn baleful_mastery_alt_cost_exiles_and_opp_draws() {
    let mut g = two_player_game();
    // Opponent needs library so they can draw.
    g.add_card_to_library(1, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::baleful_mastery());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let p1_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: spell,
        pitch_card: None,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Baleful Mastery alt-castable for {1}{B}");
    drain_stack(&mut g);

    // Bear should be exiled.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be exiled from battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile");
    // At alt cost, opponent SHOULD draw a card.
    assert_eq!(g.players[1].hand.len(), p1_hand_before + 1,
        "At alt cost, opponent should draw 1 card");
}

#[test]
fn parallax_nexus_enters_with_counters_and_forces_discard() {
    let mut g = two_player_game();
    // Give opponent a card to discard.
    g.add_card_to_hand(1, catalog::grizzly_bears());

    // Cast the enchantment so the ETB-counters pipeline fires
    // (`add_card_to_battlefield` bypasses `enters_with_counters`).
    let nexus = g.add_card_to_hand(0, catalog::parallax_nexus());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: nexus, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Parallax Nexus castable for {1}{B}{B}");
    drain_stack(&mut g);

    // Verify it enters with 5 fade counters (Fading 5).
    let n = g.battlefield.iter().find(|c| c.id == nexus).unwrap();
    assert_eq!(n.counter_count(CounterType::Fade), 5,
        "Parallax Nexus should enter with 5 fade counters");

    let opp_hand_before = g.players[1].hand.len();

    // Activate the {0} ability to force an opponent discard.
    g.perform_action(GameAction::ActivateAbility {
        card_id: nexus,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("Parallax Nexus activation should work");
    drain_stack(&mut g);

    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1,
        "Opponent should have discarded one card");
}

// ── Cube expansion: body-only stubs ─────────────────────────────────────────

#[test]
fn enduring_innocence_draws_on_nontoken_creature_etb() {
    let mut g = two_player_game();
    // Seed the library so the draw has something to pull.
    g.add_card_to_library(0, catalog::island());
    let _innocence = g.add_card_to_battlefield(0, catalog::enduring_innocence());

    // Cast a creature (goes through the stack → ETB triggers fire).
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bears castable");
    drain_stack(&mut g);

    // Net hand: cast bear (-1) + draw from Enduring Innocence (+1) = 0.
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "Enduring Innocence should draw 1 when a nontoken creature ETBs (net 0 from cast + draw)"
    );
}

/// Enduring Innocence returns from death as a noncreature enchantment.
#[test]
fn enduring_innocence_returns_as_enchantment_when_it_dies() {
    let mut g = two_player_game();
    let innocence = g.add_card_to_battlefield(0, catalog::enduring_innocence());
    // Bolt it (2/1, lethal) so SBA dispatches CreatureDied → revive trigger.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(innocence)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt Enduring Innocence");
    drain_stack(&mut g);

    let back = g.battlefield_find(innocence).expect("returned to the battlefield");
    assert_eq!(back.controller, 0, "returns under its owner's control");
    assert!(!back.definition.card_types.contains(&CardType::Creature),
        "returns as a noncreature enchantment");
    assert!(back.definition.card_types.contains(&crabomination::card::CardType::Enchantment));
}

#[test]
fn thundertrap_trainer_etb_takes_noncreature_nonland_from_top_four() {
    let mut g = two_player_game();
    // Top four: a creature, a land, an instant, a sorcery. Only the instant
    // and sorcery (noncreature, nonland) go to hand; creature + land bottomed.
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::lightning_bolt());      // 4th
    g.add_card_to_library(0, catalog::sinkhole());            // 3rd (sorcery)
    g.add_card_to_library(0, catalog::forest());              // 2nd (land)
    g.add_card_to_library(0, catalog::grizzly_bears());       // top (creature)
    let id = g.add_card_to_battlefield(0, catalog::thundertrap_trainer());
    let hand_before = g.players[0].hand.len();
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "the instant and sorcery go to hand");
}

#[test]
fn thundertrap_trainer_offspring_mints_one_one_copy() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::thundertrap_trainer());
    // Base {1}{U} + Offspring {4} = {5}{U}.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast with Offspring paid");
    drain_stack(&mut g);
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Thundertrap Trainer" && c.id != id);
    let copy = copy.expect("Offspring mints a token copy");
    let cp = g.compute_battlefield();
    let tok = cp.iter().find(|c| c.id == copy.id).unwrap();
    assert_eq!((tok.power, tok.toughness), (1, 1), "Offspring token is 1/1");
}

#[test]
fn thundertrap_trainer_no_offspring_no_copy() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::thundertrap_trainer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id); // unkicked
    let copies = g.battlefield.iter()
        .filter(|c| c.definition.name == "Thundertrap Trainer").count();
    assert_eq!(copies, 1, "no Offspring paid → no token copy");
}

#[test]
fn corpse_dance_reanimates_creature_from_graveyard() {
    let mut g = two_player_game();
    // Put a creature in P0's graveyard.
    let bear_id = g.add_card_to_library(0, catalog::grizzly_bears());
    let pos = g.players[0]
        .library
        .iter()
        .position(|c| c.id == bear_id)
        .unwrap();
    let bear_card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(bear_card);

    let bf_creatures_before = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.card_types.contains(&CardType::Creature))
        .count();

    let spell = g.add_card_to_hand(0, catalog::corpse_dance());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Corpse Dance castable");
    drain_stack(&mut g);

    let bf_creatures_after = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.card_types.contains(&CardType::Creature))
        .count();
    assert!(
        bf_creatures_after > bf_creatures_before,
        "Corpse Dance should put a creature onto the battlefield"
    );
}

#[test]
fn basking_rootwalla_pump_once_per_turn() {
    let mut g = two_player_game();
    let rootwalla = g.add_card_to_battlefield(0, catalog::basking_rootwalla());
    g.clear_sickness(rootwalla);

    let base_power = g.computed_permanent(rootwalla).unwrap().power;
    assert_eq!(base_power, 1, "Basking Rootwalla base power should be 1");

    // Pay {1}{G} to activate the pump.
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: rootwalla,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    })
    .expect("Rootwalla pump activates");
    drain_stack(&mut g);

    let pumped = g.computed_permanent(rootwalla).unwrap();
    assert_eq!(pumped.power, 3, "Rootwalla should be 3/3 after pump");
    assert_eq!(pumped.toughness, 3, "Rootwalla should be 3/3 after pump");
}

#[test]
fn blazing_rootwalla_madness_zero_and_pump() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let rw = g.add_card_to_battlefield(0, catalog::blazing_rootwalla());
    g.clear_sickness(rw);
    // Madness {0}: the keyword is present so a discard offers a free cast.
    assert!(g.battlefield_find(rw).unwrap().definition.keywords
        .iter().any(|k| matches!(k, Keyword::Madness(_))), "carries Madness");
    // {1}{R}: +1/+1 until end of turn.
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rw, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("pump activates");
    drain_stack(&mut g);
    let pumped = g.computed_permanent(rw).unwrap();
    assert_eq!((pumped.power, pumped.toughness), (2, 2), "1/1 → 2/2 after +1/+1");
}

// ── Push XIX: cube creature tests ──────────────────────────────────────

// ── Push: new modern creatures ──────────────────────────────────────────

#[test]
fn blade_splicer_etb_creates_golem_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blade_splicer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Blade Splicer castable");
    drain_stack(&mut g);
    // Blade Splicer (1/1) + Golem token (3/3)
    assert_eq!(g.battlefield.len(), bf_before + 2);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Golem"),
        "A 3/3 Golem token should be on the battlefield");
}

/// Torpor Orb (CR 614): a creature's ETB trigger doesn't fire while it's in
/// play — Blade Splicer enters but mints no Golem.
#[test]
fn torpor_orb_suppresses_creature_etb_triggers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::torpor_orb());
    let splicer = g.add_card_to_battlefield(0, catalog::blade_splicer());
    g.fire_self_etb_triggers(splicer, 0);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Golem"),
        "Torpor Orb suppresses the ETB Golem token");
}

/// Tocatli Honor Guard suppresses creature ETB triggers exactly like
/// Torpor Orb (1/3 creature body).
#[test]
fn tocatli_honor_guard_suppresses_creature_etb_triggers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tocatli_honor_guard());
    let splicer = g.add_card_to_battlefield(1, catalog::blade_splicer());
    g.fire_self_etb_triggers(splicer, 1);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Golem"),
        "Tocatli Honor Guard suppresses an opponent's ETB token too");
}

/// Hushbringer suppresses both creature ETB *and* death triggers — a dying
/// Wurmcoil Engine mints no Wurm tokens.
#[test]
fn hushbringer_suppresses_creature_dies_triggers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hushbringer());
    let wurm = g.add_card_to_battlefield(0, catalog::wurmcoil_engine());
    // Lethal damage → SBA death → dies trigger would mint two tokens.
    g.battlefield_find_mut(wurm).unwrap().damage = 6;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_none(), "Wurmcoil died to lethal damage");
    assert!(!g.battlefield.iter().any(|c| c.definition.name.contains("Wurm")),
        "Hushbringer suppresses the death-trigger Wurm tokens");
}

#[test]
fn vendilion_clique_is_3_1_legendary_flash_flying() {
    use crabomination::card::Keyword;
    let card = catalog::vendilion_clique();
    assert_eq!(card.name, "Vendilion Clique");
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 1);
    assert!(card.keywords.contains(&Keyword::Flash));
    assert!(card.keywords.contains(&Keyword::Flying));
    assert!(card.supertypes.iter().any(|s| matches!(s, crabomination::card::Supertype::Legendary)));

    // ETB hand-disruption: a SelfSource EntersBattlefield trigger carrying
    // the look-choose-bottom-and-draw primitive.
    use crabomination::card::{EventKind, EventScope};
    use crabomination::effect::Effect;
    let etb_disrupt = card.triggered_abilities.iter().any(|ta| {
        ta.event.kind == EventKind::EntersBattlefield
            && ta.event.scope == EventScope::SelfSource
            && matches!(ta.effect, Effect::BottomChosenFromHandAndDraw { .. })
    });
    assert!(etb_disrupt, "Vendilion Clique should have its ETB bottom-and-draw trigger");
}

#[test]
fn vendilion_clique_etb_bottoms_chosen_card_and_target_draws() {
    // P0 ETBs Vendilion; the caster picks a nonland card from the opponent's
    // hand. That card goes to the bottom of the opponent's library and the
    // opponent draws a replacement off the top.
    let mut g = two_player_game();
    // Opponent (P1) hand: a nonland card we'll bottom.
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    // P1 library top = the replacement they'll draw. add_card_to_library
    // appends to the bottom; with an otherwise-empty library it's also the
    // top, so it's drawn before the freshly-bottomed bear.
    let replacement = g.add_card_to_library(1, catalog::lightning_bolt());

    let id = g.add_card_to_hand(0, catalog::vendilion_clique());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);

    // Force the caster's choice (the bear) deterministically.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![bear])]));

    let p1_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Vendilion Clique castable for {1}{U}{U}");
    drain_stack(&mut g);

    assert!(
        !g.players[1].hand.iter().any(|c| c.id == bear),
        "the chosen bear should have left the opponent's hand",
    );
    assert_eq!(
        g.players[1].library.last().map(|c| c.id),
        Some(bear),
        "the chosen bear should be on the bottom of the opponent's library",
    );
    assert!(
        g.players[1].hand.iter().any(|c| c.id == replacement),
        "the opponent should have drawn a replacement off the top",
    );
    assert_eq!(
        g.players[1].hand.len(),
        p1_hand_before,
        "net opponent hand size is unchanged (-1 bottomed, +1 drawn)",
    );
}

#[test]
fn torrential_gearhulk_is_5_6_artifact_flash() {
    use crabomination::card::Keyword;
    let card = catalog::torrential_gearhulk();
    assert_eq!(card.name, "Torrential Gearhulk");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 6);
    assert!(card.keywords.contains(&Keyword::Flash));
    assert!(card.card_types.contains(&CardType::Artifact));
    assert!(card.card_types.contains(&CardType::Creature));

    // ETB "cast target instant from your graveyard without paying" rider:
    // a SelfSource EntersBattlefield trigger carrying the free-cast-from-
    // graveyard primitive with `exile_after`.
    use crabomination::card::{EventKind, EventScope};
    use crabomination::effect::Effect;
    let etb_free_cast = card.triggered_abilities.iter().any(|ta| {
        ta.event.kind == EventKind::EntersBattlefield
            && ta.event.scope == EventScope::SelfSource
            && matches!(
                ta.effect,
                Effect::CastWithoutPayingImmediate {
                    source_zone: crabomination::card::Zone::Graveyard,
                    exile_after: true,
                    ..
                }
            )
    });
    assert!(
        etb_free_cast,
        "Torrential Gearhulk should have an ETB free-cast-from-graveyard (exile_after) trigger",
    );
}

#[test]
fn torrential_gearhulk_etb_casts_instant_from_graveyard_and_exiles_it() {
    // Seed a Lightning Bolt in P0's graveyard, ETB Gearhulk, accept the
    // "cast without paying?" prompt: the Bolt is cast for free (auto-
    // targeted) and, per the printed exile rider, ends up in exile rather
    // than back in the graveyard.
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    // A legal auto-target for the free Bolt (opponent's creature).
    let _opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::torrential_gearhulk());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);

    // Accept the OptionalTrigger ("Cast without paying?").
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Torrential Gearhulk castable for {4}{U}{U}");
    drain_stack(&mut g);

    assert!(
        g.battlefield.iter().any(|c| c.id == id),
        "Gearhulk should resolve onto the battlefield",
    );
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Bolt should have left the graveyard when cast",
    );
    assert!(
        g.exile.iter().any(|c| c.id == bolt),
        "Bolt should be exiled after resolving (the printed exile rider)",
    );
}

#[test]
fn kitesail_larcenist_etb_exiles_opponent_nonland() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::kitesail_larcenist());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Kitesail Larcenist castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Opponent's creature should be exiled");
}

#[test]
fn grave_titan_etb_creates_two_zombie_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::grave_titan());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grave Titan castable");
    drain_stack(&mut g);
    // Grave Titan + 2 Zombies
    assert_eq!(g.battlefield.len(), bf_before + 3);
    let zombie_count = g.battlefield.iter()
        .filter(|c| c.definition.name == "Zombie")
        .count();
    assert_eq!(zombie_count, 2, "Should create 2 Zombie tokens on ETB");
}

#[test]
fn shriekmaw_etb_destroys_nonblack_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::shriekmaw());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Shriekmaw castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Opponent's nonblack creature should be destroyed");
}

#[test]
fn phyrexian_obliterator_is_5_5_trample() {
    use crabomination::card::Keyword;
    let card = catalog::phyrexian_obliterator();
    assert_eq!(card.name, "Phyrexian Obliterator");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 5);
    assert!(card.keywords.contains(&Keyword::Trample));

    // Damage-retaliation: a SelfSource DealtDamage trigger → sacrifice
    // (count = the damage amount).
    use crabomination::card::{EventKind, EventScope};
    use crabomination::effect::{Effect, Value};
    let retaliate = card.triggered_abilities.iter().any(|ta| {
        ta.event.kind == EventKind::DealtDamage
            && ta.event.scope == EventScope::SelfSource
            && matches!(
                &ta.effect,
                Effect::Sacrifice { count: Value::TriggerEventAmount, .. }
            )
    });
    assert!(retaliate, "Phyrexian Obliterator should retaliate on being dealt damage");
}

#[test]
fn phyrexian_obliterator_damage_forces_opponent_to_sacrifice_that_many() {
    // 3 damage to the Obliterator → the opponent sacrifices 3 permanents.
    // (P0 bolts its own Obliterator to deliver the damage — the EachOpponent
    // approximation has the opponent sacrifice regardless of who dealt it.)
    let mut g = two_player_game();
    let _oblit = g.add_card_to_battlefield(0, catalog::phyrexian_obliterator()); // 5/5, survives 3
    // Opponent board: four permanents, so post-sacrifice there's a remainder
    // to count against.
    for _ in 0..4 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let opp_perms_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(opp_perms_before, 4);

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(_oblit)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);

    assert!(
        g.battlefield_find(_oblit).is_some(),
        "Obliterator (5/5) survives 3 damage",
    );
    let opp_perms_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(
        opp_perms_after,
        opp_perms_before - 3,
        "opponent sacrifices 3 permanents (= the 3 damage dealt)",
    );
}

#[test]
fn glorybringer_attack_deals_4_damage_to_opponent_creature() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let glory = g.add_card_to_battlefield(0, catalog::glorybringer());
    g.clear_sickness(glory);
    // Opponent has a 5-toughness creature
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: glory,
        target: AttackTarget::Player(1),
    }]))
    .expect("Glorybringer attacks");
    drain_stack(&mut g);
    // Grizzly Bears has 2 toughness; 4 damage kills it
    assert!(g.players[1].graveyard.iter().any(|c| c.id == opp_creature),
        "Glorybringer should deal 4 damage to the targeted creature, killing it");
}

#[test]
fn inferno_titan_etb_deals_3_damage_to_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inferno_titan());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Inferno Titan castable");
    drain_stack(&mut g);
    // Grizzly Bears has 2 toughness; 3 damage kills it
    assert!(g.players[1].graveyard.iter().any(|c| c.id == opp_bear),
        "Inferno Titan ETB should deal 3 damage, killing the bear");
}

#[test]
fn thundermaw_hellkite_is_5_5_flying_haste() {
    use crabomination::card::Keyword;
    let card = catalog::thundermaw_hellkite();
    assert_eq!(card.name, "Thundermaw Hellkite");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 5);
    assert!(card.keywords.contains(&Keyword::Flying));
    assert!(card.keywords.contains(&Keyword::Haste));
    assert_eq!(card.triggered_abilities.len(), 1, "ETB trigger");
}

#[test]
fn craterhoof_behemoth_is_5_5_haste_trample() {
    use crabomination::card::Keyword;
    let card = catalog::craterhoof_behemoth();
    assert_eq!(card.name, "Craterhoof Behemoth");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 5);
    assert!(card.keywords.contains(&Keyword::Haste));
    assert!(card.keywords.contains(&Keyword::Trample));
    assert_eq!(card.triggered_abilities.len(), 1, "ETB pump trigger");
}

#[test]
fn thragtusk_etb_gains_5_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::thragtusk());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Thragtusk castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 5,
        "Thragtusk ETB should gain 5 life");
}

#[test]
fn courser_of_kruphix_gains_life_on_land_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::courser_of_kruphix());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.priority.player_with_priority = 0;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::PlayLand(land))
        .expect("Forest plays");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1,
        "Courser should gain 1 life when a land enters");
}

#[test]
fn wurmcoil_engine_is_6_6_deathtouch_lifelink() {
    use crabomination::card::Keyword;
    let card = catalog::wurmcoil_engine();
    assert_eq!(card.name, "Wurmcoil Engine");
    assert_eq!(card.power, 6);
    assert_eq!(card.toughness, 6);
    assert!(card.keywords.contains(&Keyword::Deathtouch));
    assert!(card.keywords.contains(&Keyword::Lifelink));
    assert!(card.card_types.contains(&CardType::Artifact));
    assert_eq!(card.triggered_abilities.len(), 1, "death trigger");
}

