//! Functionality tests for the keyword-trigger shortcuts
//! (`effect::shortcut::{frenzy, afflict, afterlife}`) — CR 702.68 /
//! 702.131 / 702.135. Each test builds a synthetic creature carrying the
//! keyword trigger and drives combat (or a death) to observe the rider.
//! (Bushido / Flanking / Rampage already ship as `Keyword::*` combat
//! rules wired in `combat.rs`.)

use crate::card::{CardDefinition, CardType, Subtypes, TriggeredAbility};
use crate::catalog;
use crate::effect::shortcut;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::game::types::TurnStep;
use crate::mana::Color;

/// A bare N/M creature carrying the given triggered abilities.
fn body(name: &'static str, p: i32, t: i32, trig: Vec<TriggeredAbility>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        power: p,
        toughness: t,
        triggered_abilities: trig,
        ..Default::default()
    }
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

// ── CR 702.68 Frenzy ─────────────────────────────────────────────────────────

#[test]
fn cr_702_68_frenzy_pumps_only_when_unblocked() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Berserker", 2, 2, vec![shortcut::frenzy(3)]));
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // No blockers declared → frenzy fires for +3/+0.
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);
    let s = g.battlefield_find(atk).unwrap();
    assert_eq!((s.power(), s.toughness()), (5, 2), "frenzy 3 pumps an unblocked attacker");
}

#[test]
fn cr_702_68_frenzy_silent_when_blocked() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Berserker", 2, 2, vec![shortcut::frenzy(3)]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    let s = g.battlefield_find(atk).unwrap();
    assert_eq!((s.power(), s.toughness()), (2, 2), "frenzy does NOT fire when blocked");
}

// ── CR 702.147 Decayed ─────────────────────────────────────────────────────

fn decayed_creature(name: &'static str) -> CardDefinition {
    CardDefinition { keywords: vec![crate::card::Keyword::Decayed], ..body(name, 2, 2, vec![]) }
}

/// A decayed creature can't be declared as a blocker (CR 702.147a).
#[test]
fn cr_702_147_decayed_cant_block() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let zombie = g.add_card_to_battlefield(1, decayed_creature("Decayed Zombie"));
    g.clear_sickness(atk);
    g.clear_sickness(zombie);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    let err = g.perform_action(GameAction::DeclareBlockers(vec![(zombie, atk)]));
    assert!(err.is_err(), "a decayed creature can't block");
}

/// A decayed creature that attacks is sacrificed at end of combat (CR 702.147b).
#[test]
fn cr_702_147_decayed_sacrificed_after_attacking() {
    let mut g = two_player_game();
    let zombie = g.add_card_to_battlefield(0, decayed_creature("Decayed Zombie"));
    g.clear_sickness(zombie);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: zombie, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert!(!g.battlefield.iter().any(|c| c.id == zombie), "decayed attacker sacrificed at end of combat");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == zombie), "it went to the graveyard");
}

// ── CR 702.158 Connive ─────────────────────────────────────────────────────

// ── CR 702.39 Provoke ────────────────────────────────────────────────────────

#[test]
fn cr_702_39_provoke_untaps_and_forces_block() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Provoker", 2, 2, vec![shortcut::provoke()]));
    g.clear_sickness(atk);
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(blk).unwrap().tapped = true; // provoke should untap it

    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);

    let b = g.battlefield_find(blk).unwrap();
    assert!(!b.tapped, "provoke untaps the target");
    assert_eq!(b.must_block, Some(atk), "provoke links the target to the attacker");

    // Declaring no blockers is illegal — the provoked creature must block.
    advance_to(&mut g, TurnStep::DeclareBlockers);
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![])).is_err(),
        "provoked creature must block if able");
    // Blocking the provoker is legal.
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block the provoker");
}

#[test]
fn cr_702_39_provoke_doesnt_force_when_unable_to_block() {
    let mut g = two_player_game();
    // Flying attacker: a grounded provoked creature can't block it, so the
    // requirement doesn't bind.
    let mut def = body("Sky Provoker", 2, 2, vec![shortcut::provoke()]);
    def.keywords.push(crate::card::Keyword::Flying);
    let atk = g.add_card_to_battlefield(0, def);
    g.clear_sickness(atk);
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // no reach/flying

    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(blk).unwrap().must_block, Some(atk), "still provoked");

    advance_to(&mut g, TurnStep::DeclareBlockers);
    // Grounded creature can't block a flier, so declaring no blockers is legal.
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("can't block a flier — no requirement");
}

// ── CR 702.142 Boast ─────────────────────────────────────────────────────────

#[test]
fn cr_702_142_boast_rejected_before_attacking() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::dragonkin_berserker());
    g.clear_sickness(id);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    });
    assert!(res.is_err(), "Boast can't be activated by a creature that hasn't attacked");
}

#[test]
fn cr_702_142_boast_succeeds_after_attacking() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::dragonkin_berserker());
    g.clear_sickness(id);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // Now boast: {3}{R} put a +1/+1 counter on it.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("Boast activatable after attacking");
    drain_stack(&mut g);
    let s = g.battlefield_find(id).unwrap();
    assert_eq!((s.power(), s.toughness()), (3, 3), "Boast adds a +1/+1 counter");
}

#[test]
fn cr_702_142_boast_only_once_per_turn() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::dragonkin_berserker());
    g.battlefield_find_mut(id).unwrap().attacked_this_turn = true;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("first boast");
    drain_stack(&mut g);
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    });
    assert!(res.is_err(), "Boast is once per turn");
}

#[test]
fn cr_702_158_connive_draws_discards_and_counters_per_nonland() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::quandrix_cryptomancer()); // 2/2
    g.clear_sickness(atk);
    // Hand + library both nonland so the discard is guaranteed nonland → +1.
    g.add_card_to_hand(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::counterspell());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let s = g.battlefield_find(atk).unwrap();
    assert_eq!((s.power(), s.toughness()), (3, 3),
        "connive drew + pitched a nonland → one +1/+1 counter");
    assert_eq!(g.players[0].hand.len(), 1, "drew one, discarded one");
}

// ── CR 702.149 Training ────────────────────────────────────────────────────

#[test]
fn cr_702_149_training_counters_when_attacking_with_bigger_creature() {
    let mut g = two_player_game();
    let trainee = g.add_card_to_battlefield(0, body("Trainee", 2, 2, vec![shortcut::training()]));
    let mentor = g.add_card_to_battlefield(0, body("Veteran", 3, 3, vec![]));
    g.clear_sickness(trainee);
    g.clear_sickness(mentor);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: trainee, target: AttackTarget::Player(1) },
        Attack { attacker: mentor, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    let s = g.battlefield_find(trainee).unwrap();
    assert_eq!((s.power(), s.toughness()), (3, 3),
        "trains off the bigger co-attacker → one +1/+1 counter");
}

#[test]
fn cr_702_149_training_silent_without_bigger_co_attacker() {
    let mut g = two_player_game();
    let trainee = g.add_card_to_battlefield(0, body("Trainee", 2, 2, vec![shortcut::training()]));
    let small = g.add_card_to_battlefield(0, body("Runt", 1, 1, vec![]));
    g.clear_sickness(trainee);
    g.clear_sickness(small);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: trainee, target: AttackTarget::Player(1) },
        Attack { attacker: small, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    let s = g.battlefield_find(trainee).unwrap();
    assert_eq!((s.power(), s.toughness()), (2, 2), "no bigger co-attacker → no counter");
}

// ── CR 702.131 Afflict ───────────────────────────────────────────────────────

#[test]
fn cr_702_131_afflict_drains_defender_on_becoming_blocked() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Raptor", 3, 3, vec![shortcut::afflict(2)]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    let life_before = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 2, "afflict 2 drains the defending player");
}

// ── Combat math preview (ClientView.combat_preview) ──────────────────────────

#[test]
fn combat_preview_reports_unblocked_damage() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Brute", 3, 3, vec![]));
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let view = crate::server::view::project(&g, 0);
    let prev = view.combat_preview.expect("preview during combat");
    assert_eq!(prev.damage_to_players, vec![(1, 3)], "unblocked 3-power swing → 3 to P1");
    assert!(prev.dying_creatures.is_empty(), "no blocks, nothing dies");
}

#[test]
fn combat_preview_flags_a_losing_trade() {
    let mut g = two_player_game();
    // 3/3 attacker into a 2/2 blocker: blocker dies, attacker lives.
    let atk = g.add_card_to_battlefield(0, body("Brute", 3, 3, vec![]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    let prev = crate::server::view::project(&g, 0).combat_preview.expect("preview");
    assert_eq!(prev.dying_creatures, vec![blk], "the 2/2 blocker is projected to die");
    assert!(prev.damage_to_players.is_empty(), "no trample, no player damage");
}

#[test]
fn combat_preview_first_strike_attacker_survives_lethal_blocker() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    // 2/2 first-strike attacker into a 2/2 blocker: the attacker kills the
    // blocker before it deals damage, so only the blocker is projected to die.
    let mut striker = body("Striker", 2, 2, vec![]);
    striker.keywords = vec![Keyword::FirstStrike];
    let atk = g.add_card_to_battlefield(0, striker);
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    let prev = crate::server::view::project(&g, 0).combat_preview.expect("preview");
    assert_eq!(prev.dying_creatures, vec![blk],
        "first strike kills the blocker before it deals damage; attacker survives");
}

// ── CR 702.135 Afterlife ─────────────────────────────────────────────────────

#[test]
fn cr_702_135_afterlife_mints_spirits_on_death() {
    use crate::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, body("Cleric", 1, 1, vec![shortcut::afterlife(2)]));
    // Kill it: drop its toughness below 1 so SBA destroys it.
    g.battlefield_find_mut(c).unwrap().toughness_bonus -= 1;
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|p| p.controller == 0
            && p.is_token
            && p.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 2, "afterlife 2 mints two Spirit tokens");
    assert!(spirits.iter().all(|s| s.has_keyword(&Keyword::Flying)), "Spirits have flying");
}

// ── CR 702.19 Trample (with Deathtouch) ──────────────────────────────────────

#[test]
fn cr_702_19_deathtouch_trample_assigns_one_then_tramples_rest() {
    // CR 702.19e + 702.2c: a deathtouch+trample attacker need only assign 1
    // (lethal) to each blocker, tramping the remainder to the player.
    use crate::card::{CardType, Keyword, Subtypes};
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, CardDefinition {
        name: "Wurm",
        card_types: vec![CardType::Creature],
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample, Keyword::Deathtouch],
        ..Default::default()
    });
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(atk);
    let life = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, life - 4,
        "1 lethal to the 2/2 (deathtouch), 4 trample to the player");
    assert!(g.battlefield_find(blk).is_none(), "the blocker died to deathtouch");
}

// ── CR 702.137 Riot ──────────────────────────────────────────────────────────

#[test]
fn cr_702_137_riot_default_grants_haste() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::zhur_taa_goblin());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {R}{G}");
    drain_stack(&mut g);
    // AutoDecider picks mode 0 → permanent haste.
    let cp = g.computed_permanent(id).expect("Goblin in play");
    assert!(cp.keywords.contains(&Keyword::Haste), "Riot default mode grants haste");
    assert_eq!((cp.power, cp.toughness), (2, 2), "no counter in haste mode");
}

#[test]
fn cr_702_137_riot_counter_mode_grows_the_body() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(1)]));
    let id = g.add_card_to_hand(0, catalog::zhur_taa_goblin());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 3), "Riot counter mode → 3/3");
}

// ── CR 702.99 Extort ─────────────────────────────────────────────────────────

#[test]
fn cr_702_99_extort_drains_when_the_optional_cost_is_paid() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Yes to the extort "may pay {W/B}" prompt.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::basilica_screecher());
    // Cast Bolt ({R}, no generic so the floated W survives for extort).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 4, "Bolt 3 + extort drain 1");
    assert_eq!(g.players[0].life, my_life + 1, "extort gained the controller 1");
}

#[test]
fn cr_702_99_extort_does_nothing_when_declined() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::basilica_screecher());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bears castable");
    drain_stack(&mut g);
    // AutoDecider declines the optional cost → no drain.
    assert_eq!(g.players[1].life, opp_life, "extort declined → opponent untouched");
}

// ── CR 702.46 Soulshift ──────────────────────────────────────────────────────

#[test]
fn cr_702_46_soulshift_returns_a_spirit_from_graveyard() {
    use crate::card::{CardType, CreatureType, Subtypes};
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    use crate::mana::{b, cost, generic};
    let mut g = two_player_game();
    // Yes to the soulshift "may return" prompt.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    // A 2/2 Spirit with Soulshift 5.
    let warden = g.add_card_to_battlefield(0, CardDefinition {
        name: "Test Spirit Warden",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![shortcut::soulshift(5)],
        ..Default::default()
    });
    // A cheap Spirit waiting in the graveyard (MV 2 ≤ 5).
    let ghost = g.add_card_to_graveyard(0, CardDefinition {
        name: "Test Lost Ghost",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    });
    // Kill the warden via SBA.
    g.battlefield_find_mut(warden).unwrap().toughness_bonus -= 2;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == ghost),
        "Soulshift returned the graveyard Spirit to hand");
}

// ── Extort / Riot card bodies ────────────────────────────────────────────────

#[test]
fn extort_creatures_have_correct_bodies() {
    use crate::card::Keyword;
    let syndic = catalog::syndic_of_tithes();
    assert_eq!((syndic.power, syndic.toughness), (2, 3));
    assert!(!syndic.triggered_abilities.is_empty(), "Syndic has Extort");

    let tithe = catalog::tithe_drinker();
    assert!(tithe.keywords.contains(&Keyword::Lifelink), "Tithe Drinker has Lifelink");

    let pet = catalog::kingpins_pet();
    assert!(pet.keywords.contains(&Keyword::Flying), "Kingpin's Pet has Flying");
}

#[test]
fn frenzied_arynx_pump_ability_grows_it() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::frenzied_arynx());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("pump ability activatable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (6, 5), "Frenzied Arynx pumps to 6/5");
}

// ── ClientView.activatable_permanents (legal-plays hint) ─────────────────────

#[test]
fn activatable_permanents_lists_a_usable_ability() {
    let mut g = two_player_game();
    // Nantuko Husk: "Sacrifice a creature: +1/+1". With fodder available
    // and priority held, it should report as activatable.
    let husk = g.add_card_to_battlefield(0, catalog::nantuko_husk());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let view = crate::server::view::project(&g, 0);
    assert!(view.activatable_permanents.contains(&husk),
        "Nantuko Husk with sac fodder is activatable");
}

#[test]
fn activatable_permanents_empty_off_priority() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nantuko_husk());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 1; // not our priority
    let view = crate::server::view::project(&g, 0);
    assert!(view.activatable_permanents.is_empty(), "no affordance off-priority");
}

/// A pure mana ability (Llanowar Elves' `{T}: Add G`) is not a meaningful
/// instant-speed play: it never uses the stack and is auto-tapped on demand
/// during payment. It must be excluded from `activatable_permanents` so the
/// client doesn't stall priority on every step just because a mana dork is
/// available.
#[test]
fn activatable_permanents_excludes_pure_mana_abilities() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    // Untapped + not summoning-sick → the tap-for-mana ability would
    // otherwise be accepted by the engine.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == elf) {
        c.summoning_sick = false;
    }
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let view = crate::server::view::project(&g, 0);
    assert!(
        !view.activatable_permanents.contains(&elf),
        "a pure mana dork must not register as an instant-speed play",
    );
}

/// CR 116 priority: a non-active player who holds priority can activate
/// instant-speed abilities — e.g. crack a fetch land on the opponent's end
/// step. Verifies both that the engine hands priority to the non-active
/// player on the opponent's end step and that the fetch land surfaces as an
/// available play (the signal the client uses to stop auto-passing).
#[test]
fn crack_fetchland_on_opponents_end_step() {
    let mut g = two_player_game();
    // Seat 0 controls a fetch land. Seat 1 is the active player, on their
    // End step, having just passed priority to seat 0.
    let strand = g.add_card_to_battlefield(0, catalog::flooded_strand());
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    g.give_priority_to_active(); // seat 1 (active) receives priority first
    g.perform_action(GameAction::PassPriority).expect("active player passes");

    // CR 116.3b/116.4 — priority now sits with the non-active player.
    assert_eq!(g.player_with_priority(), 0, "non-active player gets priority");
    assert_eq!(g.active_player_idx, 1);
    assert_eq!(g.step, TurnStep::End);

    // The fetch land is reported as a legal instant-speed play right now.
    let view = crate::server::view::project(&g, 0);
    assert!(
        view.activatable_permanents.contains(&strand),
        "fetch land should be crackable on the opponent's end step",
    );

    // Cracking it is accepted and sacrifices the land (CR 605/701.16).
    g.perform_action(GameAction::ActivateAbility {
        card_id: strand,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("crack fetch on opponent's end step");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().all(|c| c.id != strand),
        "fetch land sacrificed after cracking",
    );
}

// ── CR 508.3a — create tokens tapped and attacking ──────────────────────────

#[test]
fn create_token_attacking_joins_combat_tapped() {
    use crate::card::{CreatureType, TokenDefinition};
    use crate::effect::{Effect, PlayerRef, Value};
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        equipped_bonus: None,
    };
    let trig = shortcut::on_attack(Effect::CreateTokenAttacking {
        who: PlayerRef::You,
        count: Value::Const(2),
        definition: soldier,
        cleanup: crate::effect::AttackingTokenCleanup::None,
    });
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Warcaller", 2, 2, vec![trig]));
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // Two Soldier tokens minted, tapped and attacking the same player.
    let tokens: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Soldier")
        .collect();
    assert_eq!(tokens.len(), 2, "two soldier tokens minted");
    assert!(tokens.iter().all(|c| c.tapped), "tokens enter tapped");
    let token_ids: Vec<_> = tokens.iter().map(|c| c.id).collect();
    for id in token_ids {
        assert!(g.attacking.iter().any(|a| a.attacker == id
            && a.target == AttackTarget::Player(1)), "token attacking P1");
    }
}

// ── CR 702.151 Enlist ─────────────────────────────────────────────────────────

#[test]
fn cr_702_151_enlist_taps_a_creature_and_adds_its_power() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Enlister", 2, 2, vec![shortcut::enlist()]));
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(atk);
    g.clear_sickness(helper);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // Helper tapped; attacker is 2+2 = 4 power until end of turn.
    assert!(g.battlefield_find(helper).unwrap().tapped, "enlisted creature is tapped");
    assert_eq!(g.battlefield_find(atk).unwrap().power(), 4, "attacker gains the helper's power");
}

#[test]
fn cr_702_151_enlist_does_nothing_with_no_eligible_helper() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Enlister", 2, 2, vec![shortcut::enlist()]));
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(atk).unwrap().power(), 2, "no helper → no pump");
}

// ── CR 702.169 Mobilize ──────────────────────────────────────────────────────

#[test]
fn cr_702_169_mobilize_tokens_attack_then_sacrifice_at_end_of_combat() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Mobilizer", 2, 2, vec![shortcut::mobilize(2)]));
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    fn warriors(g: &GameState) -> usize {
        g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Warrior").count()
    }
    // Two 1/1 red Warriors tapped and attacking.
    assert_eq!(warriors(&g), 2, "two warrior tokens minted");
    assert!(g.battlefield.iter().filter(|c| c.definition.name == "Warrior")
        .all(|c| c.tapped), "warriors enter tapped");
    // They deal their combat damage (P1 takes 2 attacker + 2 warriors = 4).
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 16, "attacker + two 1/1 warriors hit for 4");
    // Sacrificed as combat ends — gone before postcombat main.
    assert_eq!(warriors(&g), 0, "warriors sacrificed at end of combat");
}

// ── CR 509.1c MustBlock ("blocks each combat if able") ───────────────────────

#[test]
fn cr_509_1c_must_block_creature_is_forced_to_block() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Attacker", 2, 2, vec![]));
    let mut guard = body("Compelled Guard", 1, 4, vec![]);
    guard.keywords = vec![Keyword::MustBlock];
    let guard = g.add_card_to_battlefield(1, guard);
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    // Declaring no blocks is illegal — the guard must block.
    let err = g.perform_action(GameAction::DeclareBlockers(vec![])).unwrap_err();
    assert!(matches!(err, GameError::MustBeBlockedIfAble(id) if id == guard));
    // Assigning the guard is accepted.
    g.perform_action(GameAction::DeclareBlockers(vec![(guard, atk)])).expect("guard blocks");
}

// ── CR 702.16e Protection from color prevents combat damage ──────────────────

/// A red 2/2 attacker blocked by a 2/2 with protection from red: the blocker
/// takes no combat damage, but still deals its 2 back to the attacker.
#[test]
fn cr_702_16e_protection_prevents_combat_damage() {
    use crate::card::Keyword;
    use crate::mana::{cost, r};
    let mut g = two_player_game();
    let mut red_atk = body("Red Attacker", 2, 3, vec![]); // 2/3 survives the 2 back
    red_atk.cost = cost(&[r()]); // makes it red
    let atk = g.add_card_to_battlefield(0, red_atk);
    let mut prot = body("Warded", 2, 2, vec![]);
    prot.keywords = vec![Keyword::Protection(Color::Red)];
    let blk = g.add_card_to_battlefield(1, prot);
    g.clear_sickness(atk);

    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.battlefield_find(blk).unwrap().damage, 0, "protected blocker takes no red damage");
    assert_eq!(g.battlefield_find(atk).unwrap().damage, 2, "attacker still takes the blocker's 2");
}

// ── CR 724 The Monarch ───────────────────────────────────────────────────────

#[test]
fn cr_724_monarch_draws_at_their_end_step() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.monarch = Some(0);
    let before = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.players[0].hand.len(), before + 1, "monarch drew at their end step");
}

#[test]
fn cr_724_non_monarch_end_step_does_not_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.monarch = Some(1); // P1 is monarch, but it's P0's turn/end step
    let before = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.players[0].hand.len(), before, "active non-monarch does not draw");
}

#[test]
fn cr_724_combat_damage_to_monarch_steals_the_crown() {
    let mut g = two_player_game();
    g.monarch = Some(1); // the opponent starts as monarch
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0), "dealing combat damage to the monarch steals the crown");
}

#[test]
fn cr_724_become_monarch_effect_via_etb() {
    use crate::card::Effect;
    use crate::effect::PlayerRef;
    let mut g = two_player_game();
    let mut def = body("Crown Claimer", 2, 2,
        vec![shortcut::etb(Effect::BecomeMonarch { who: PlayerRef::You })]);
    def.cost = crate::mana::cost(&[crate::mana::generic(1)]);
    let id = g.add_card_to_hand(0, def);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0), "ETB BecomeMonarch made the controller the monarch");
}

#[test]
fn cr_724_is_monarch_predicate() {
    use crate::card::Predicate;
    use crate::effect::PlayerRef;
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    g.monarch = Some(0);
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    assert!(g.evaluate_predicate(&Predicate::IsMonarch { who: PlayerRef::You }, &ctx));
    assert!(!g.evaluate_predicate(&Predicate::IsMonarch { who: PlayerRef::EachOpponent }, &ctx));
}

// ── CR 505.1b additional combat phase ─────────────────────────────────────────

#[test]
fn cr_505_1b_additional_combat_phase_lets_attacker_strike_twice() {
    use crate::card::ActivatedAbility;
    use crate::effect::{Effect, Selector, Value};
    // A Hellkite-Charger-style body: `{0}: Untap this; additional combat
    // phase.` Activated during combat, it untaps the (now-tapped) attacker
    // and loops the turn back to a fresh combat.
    let charger = CardDefinition {
        name: "Test Charger",
        card_types: vec![CardType::Creature],
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::Untap { what: Selector::This, up_to: None },
                Effect::AdditionalCombatPhase { count: Value::Const(1) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, charger);
    g.clear_sickness(atk);
    let start = g.players[1].life;

    // First combat: attack, then activate the extra-combat ability.
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack 1");
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: atk, ability_index: 0, target: None, x_value: None,
    }).expect("extra-combat ability");
    drain_stack(&mut g);
    // Walk out of the first combat — End of Combat loops back to Begin Combat.
    advance_to(&mut g, TurnStep::BeginCombat);

    // Second combat: the untapped attacker strikes again.
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack 2");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);

    assert_eq!(g.players[1].life, start - 6, "attacker dealt 3 in each of two combats");
    assert_eq!(g.additional_combat_phases, 0, "the banked phase was consumed");
}

// ── CR 509.1 "Whenever this blocks" + Stun (Wall of Frost) ────────────────────

#[test]
fn wall_of_frost_stuns_the_creature_it_blocks() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    // P0 attacks with a 2/2; P1's Wall of Frost blocks it.
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_frost());
    g.clear_sickness(atk);
    let _ = wall;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, atk)])).expect("wall blocks");
    drain_stack(&mut g);
    // The blocked attacker carries a Stun counter (the BlockedAttacker selector
    // bound it correctly), and survives the 0-power wall.
    let a = g.battlefield_find(atk).expect("attacker survives the 0/7 wall");
    assert_eq!(a.counter_count(CounterType::Stun), 1, "blocked creature got a Stun counter");
}

// ── CR 508.1 attack tax (Propaganda / Ghostly Prison) ───────────────────────
// `StaticEffect::AttackTaxToController` — the defending player's pillowfort
// charges the attacker's controller a generic mana tax per attacking creature,
// paid at declare-attackers from floating mana. Mana empties on step changes,
// so each test funds the pool *after* advancing to DeclareAttackers.

#[test]
fn propaganda_tax_paid_lets_attacker_through() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::propaganda()); // defender's pillowfort
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack after paying the {2} tax");
    assert_eq!(g.attacking().len(), 1, "attacker declared");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the {{2}} tax was paid");
    assert!(g.battlefield_find(atk).unwrap().tapped, "attacker tapped");
}

#[test]
fn propaganda_tax_unpaid_rejects_attack() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::propaganda());
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.players[0].mana_pool.add_colorless(1); // one short of the {2} tax
    let err = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }]));
    assert!(err.is_err(), "can't attack without paying the full tax");
    assert!(g.attacking().is_empty(), "no attacker declared");
    assert!(!g.battlefield_find(atk).unwrap().tapped, "attacker not tapped (rolled back)");
    assert_eq!(g.players[0].mana_pool.total(), 1, "mana not spent (payment rolled back)");
}

#[test]
fn ghostly_prison_taxes_like_propaganda() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::ghostly_prison());
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack after paying Ghostly Prison's {2}");
    assert_eq!(g.attacking().len(), 1);
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

#[test]
fn stacked_tax_enchantments_are_additive() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::propaganda());
    g.add_card_to_battlefield(1, catalog::ghostly_prison());
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.players[0].mana_pool.add_colorless(4); // {2} + {2}
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("pay {4} for two stacked tax enchantments");
    assert_eq!(g.attacking().len(), 1);
    assert_eq!(g.players[0].mana_pool.total(), 0, "both taxes paid");
}

/// The tax is per attacking creature: two attackers into one Propaganda cost
/// {4}, and {2} is short.
#[test]
fn propaganda_tax_scales_per_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::propaganda());
    let a1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let a2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a1);
    g.clear_sickness(a2);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.players[0].mana_pool.add_colorless(2); // only enough for one attacker
    let err = g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a1, target: AttackTarget::Player(1) },
        Attack { attacker: a2, target: AttackTarget::Player(1) },
    ]));
    assert!(err.is_err(), "{{2}} can't cover the {{4}} tax for two attackers");
    g.players[0].mana_pool.add_colorless(2); // top up to {4}
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a1, target: AttackTarget::Player(1) },
        Attack { attacker: a2, target: AttackTarget::Player(1) },
    ])).expect("two attackers, {4} tax paid");
    assert_eq!(g.attacking().len(), 2);
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

#[test]
fn cr_508_1g_ghostly_prison_taxes_attackers() {
    use crate::game::types::{AttackTarget, Attack};
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Bear", 2, 2, vec![]));
    g.clear_sickness(atk);
    g.add_card_to_battlefield(1, catalog::ghostly_prison());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    // No mana floating: the attack is rejected (can't pay the {2} tax).
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).is_err(), "attack rejected without paying the tax");
    // Pay {2}: the attack goes through and the mana is consumed.
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack legal once tax is payable");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the tax was spent");
}

#[test]
fn cr_508_1g_windborn_muse_does_not_tax_planeswalker_attacks() {
    use crate::game::types::{AttackTarget, Attack};
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Bear", 2, 2, vec![]));
    g.clear_sickness(atk);
    // Player 1 has Windborn Muse (player-only tax) and a planeswalker.
    g.add_card_to_battlefield(1, catalog::windborn_muse());
    let pw = g.add_card_to_battlefield(1, catalog::teferi_time_raveler());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    // Attacking the planeswalker is free (Windborn Muse protects only the player).
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Planeswalker(pw),
    }])).expect("planeswalker attack is untaxed");
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

/// Sphere of Safety taxes {X} = enchantments you control; with only Sphere out
/// it counts itself, so the tax is {1}.
#[test]
fn sphere_of_safety_counts_itself() {
    use crate::game::types::{AttackTarget, Attack};
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Bear", 2, 2, vec![]));
    g.clear_sickness(atk);
    g.add_card_to_battlefield(1, catalog::sphere_of_safety());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("pay {1} (Sphere counts itself) to attack");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the {{1}} tax was paid");
}

/// Sphere of Safety's tax scales with the defender's enchantment count: a
/// second (non-tax) enchantment pushes the tax to {2}.
#[test]
fn sphere_of_safety_scales_with_enchantment_count() {
    use crate::game::types::{AttackTarget, Attack};
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Bear", 2, 2, vec![]));
    g.clear_sickness(atk);
    g.add_card_to_battlefield(1, catalog::sphere_of_safety());
    g.add_card_to_battlefield(1, catalog::glorious_anthem()); // 2nd, non-tax enchantment
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).is_err(), "{{1}} can't cover the {{2}} tax for two enchantments");
    g.players[0].mana_pool.add_colorless(1); // top up to {2}
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("pay {2} for two enchantments");
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

/// Settle the Wreckage exiles all attacking creatures.
#[test]
fn settle_the_wreckage_exiles_attackers() {
    use crate::game::types::{AttackTarget, Attack};
    use crate::mana::Color;
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Bear", 2, 2, vec![]));
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    let settle = g.add_card_to_hand(0, catalog::settle_the_wreckage());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: settle, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Settle the Wreckage");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == atk), "attacking creature exiled");
}

/// Beastmaster Ascension gains a quest counter whenever a creature you control
/// attacks.
#[test]
fn beastmaster_ascension_accrues_quest_counter_on_attack() {
    use crate::card::CounterType;
    use crate::game::types::{AttackTarget, Attack};
    let mut g = two_player_game();
    let asc = g.add_card_to_battlefield(0, catalog::beastmaster_ascension());
    let atk = g.add_card_to_battlefield(0, body("Bear", 2, 2, vec![]));
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(asc).unwrap().counter_count(CounterType::Quest), 1,
        "one quest counter per attacking creature");
}

/// CR 509.1g — "can't be blocked by more than one creature": a second blocker
/// on the same attacker is rejected; a single block is legal.
#[test]
fn cr_509_1g_cant_be_blocked_by_more_than_one() {
    use crate::card::Keyword;
    use crate::game::types::{AttackTarget, Attack};
    let mut g = two_player_game();
    let mut def = body("Sleek Skulker", 3, 3, vec![]);
    def.keywords = vec![Keyword::CantBeBlockedByMoreThanOne];
    let atk = g.add_card_to_battlefield(0, def);
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk), (b2, atk)])).is_err(),
        "two blockers on a single-block-only attacker is illegal"
    );
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk)])).expect("one blocker is fine");
}

// ── CR 702.26e — phasing out mid-combat leaves combat ─────────────────────────

#[test]
fn cr_702_26e_phase_out_removes_attacker_from_combat() {
    use crate::effect::{Effect, Selector};
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.attacking().len(), 1, "attacker declared");
    // Phase the attacker out mid-combat: it leaves combat and the battlefield.
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(atk)), 0, 0);
    let _ = g.resolve_effect(
        &Effect::PhaseOut { what: Selector::Target(0) }, &ctx,
    ).expect("phase out resolves");
    assert!(g.battlefield_find(atk).is_none(), "attacker phased out");
    assert!(g.attacking().is_empty(), "phased-out attacker no longer in combat");
}
