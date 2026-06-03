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
        subtypes: Subtypes::default(),
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
        name: "Wurm", card_types: vec![CardType::Creature], subtypes: Subtypes::default(),
        power: 5, toughness: 5,
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
        power: 2, toughness: 2,
        triggered_abilities: vec![shortcut::soulshift(5)],
        ..Default::default()
    });
    // A cheap Spirit waiting in the graveyard (MV 2 ≤ 5).
    let ghost = g.add_card_to_graveyard(0, CardDefinition {
        name: "Test Lost Ghost",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1, toughness: 1,
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
