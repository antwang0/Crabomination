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

// ── Sacrifice altars + aristocrat death payoffs ────────────────────────────

#[test]
fn ashnods_altar_sacrifices_a_creature_for_two_colorless() {
    let mut g = two_player_game();
    let altar = g.add_card_to_battlefield(0, catalog::ashnods_altar());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: altar, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("altar activates by sacrificing a creature");
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "fodder sacrificed");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2, "added two colorless");
}

/// CR 602.5b — a `wants_ui` activator chooses which creature to sacrifice for
/// an activated ability's "Sacrifice a creature" cost (Ashnod's Altar) instead
/// of the engine auto-dumping the weakest. Activation suspends on a
/// `ChooseTarget`; the chosen creature is the one sacrificed.
#[test]
fn ashnods_altar_ui_activator_chooses_creature_to_sacrifice() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let altar = g.add_card_to_battlefield(0, catalog::ashnods_altar());
    let sacked = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let kept = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    g.perform_action(GameAction::ActivateAbility {
        card_id: altar, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("activation suspends for the sacrifice choice");

    let pd = g.pending_decision.as_ref().expect("a sacrifice choice is pending");
    assert_eq!(pd.acting_player(), 0);
    match &pd.decision {
        crabomination::decision::Decision::ChooseTarget { legal, .. } => {
            assert_eq!(legal.len(), 2, "both creatures are sacrifice options");
        }
        other => panic!("expected ChooseTarget, got {other:?}"),
    }

    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Target(
        Target::Permanent(sacked),
    )))
    .expect("submit the sacrifice choice");

    assert!(!g.battlefield.iter().any(|c| c.id == sacked), "chosen creature sacrificed");
    assert!(g.battlefield.iter().any(|c| c.id == kept), "unchosen creature survives");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2, "added two colorless");
}

#[test]
fn bastion_of_remembrance_drains_when_your_creature_dies() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bastion_of_remembrance());
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt your own creature");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, p0 + 1, "you gained 1 (bolt didn't hit you)");
}

#[test]
fn dictate_of_erebos_forces_opponent_sacrifice_on_your_death() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dictate_of_erebos());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(mine)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt your own creature");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "opponent sacrificed their creature");
}

// ── Equipment: Trusty Machete / Cranial Plating ────────────────────────────

#[test]
fn trusty_machete_grants_plus_two_plus_one() {
    let mut g = two_player_game();
    let eq = g.add_card_to_battlefield(0, catalog::trusty_machete());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: eq, target: bear })
        .expect("equip for {1}");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "2/2 + 2/1");
}

#[test]
fn cranial_plating_scales_with_artifacts() {
    let mut g = two_player_game();
    let plating = g.add_card_to_battlefield(0, catalog::cranial_plating());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::mind_stone());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: plating, target: bear })
        .expect("equip for {1}");
    // Artifacts: Plating, Sol Ring, Mind Stone = 3 → +3/+0.
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 2), "2/2 + 3/0");
}

// ── Obstinate / Ravenous Baloth + Penumbra Wurm ────────────────────────────

#[test]
fn obstinate_baloth_gains_four_life_on_etb() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let id = g.add_card_to_battlefield(0, catalog::obstinate_baloth());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4);
}

#[test]
fn ravenous_baloth_sacs_a_beast_for_four_life() {
    let mut g = two_player_game();
    let baloth = g.add_card_to_battlefield(0, catalog::ravenous_baloth());
    let beast = g.add_card_to_battlefield(0, catalog::obstinate_baloth()); // a Beast
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: baloth, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sacrifice a Beast to gain life");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == beast), "Beast sacrificed");
    assert_eq!(g.players[0].life, life + 4);
}

#[test]
fn penumbra_wurm_leaves_a_wurm_token_when_it_dies() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::penumbra_wurm());
    let _ = g.remove_to_graveyard_with_triggers(wurm);
    drain_stack(&mut g);
    let tok = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Wurm")
        .expect("Wurm token created on death");
    assert!(tok.definition.keywords.contains(&Keyword::Trample), "token has trample");
}

// ── Hanweir Garrison / Pyre Charger ────────────────────────────────────────

#[test]
fn hanweir_garrison_makes_two_attacking_tokens() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let garrison = g.add_card_to_battlefield(0, catalog::hanweir_garrison());
    g.clear_sickness(garrison);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: garrison, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    let tokens: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Human").collect();
    assert_eq!(tokens.len(), 2, "two Human tokens created");
    assert!(tokens.iter().all(|c| c.tapped), "tokens are tapped and attacking");
}

#[test]
fn pyre_charger_firebreathing_pumps_power() {
    let mut g = two_player_game();
    let pc = g.add_card_to_battlefield(0, catalog::pyre_charger());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("firebreathing activates");
    drain_stack(&mut g);
    let cp = g.computed_permanent(pc).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 1), "1/1 + 1/0");
}

// ── Solemnity (CR 122.1 — counters can't be placed) ────────────────────────

#[test]
fn solemnity_blocks_add_counter() {
    use crabomination::card::CounterType;
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::solemnity());
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        crabomination::card::CardId(0), 0, Some(Target::Permanent(bear)),
    );
    g.resolve_effect(&Effect::AddCounter {
        what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2),
    }, &ctx).unwrap();
    let n = g.battlefield.iter().find(|c| c.id == bear).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(n, 0, "Solemnity dropped the +1/+1 counters");
}

#[test]
fn solemnity_blocks_proliferate() {
    use crabomination::card::CounterType;
    use crabomination::effect::Effect;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    g.add_card_to_battlefield(0, catalog::solemnity());
    let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    g.resolve_effect(&Effect::Proliferate, &ctx).unwrap();
    let n = g.battlefield.iter().find(|c| c.id == bear).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(n, 1, "proliferate did not grow the counter under Solemnity");
}

#[test]
fn solemnity_blocks_enters_with_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::solemnity());
    let id = g.add_card_to_hand(0, catalog::murktide_regent());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {5}{U}{U}");
    drain_stack(&mut g);
    let n = g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(n, 0, "Murktide entered with no counters under Solemnity");
}

// ── White beaters + Blinking Spirit ────────────────────────────────────────

#[test]
fn blinking_spirit_returns_itself_to_hand_for_zero() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::blinking_spirit());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{0}: return to hand");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "back in its owner's hand");
}

#[test]
fn white_fliers_have_their_keywords() {
    use crabomination::card::Keyword;
    let sky = catalog::leonin_skyhunter();
    assert!(sky.keywords.contains(&Keyword::Flying) && sky.power == 2 && sky.toughness == 2);
    let serra = catalog::serra_avenger();
    assert!(serra.keywords.contains(&Keyword::Flying) && serra.keywords.contains(&Keyword::Vigilance));
}

// ── Edict-on-death (Grave Pact / Butcher of Malakir) ───────────────────────

pub(crate) fn kill_with_bolt(g: &mut crabomination::game::GameState, victim: CardId) {
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the creature");
    drain_stack(g);
}

#[test]
fn grave_pact_forces_opponent_sacrifice_on_your_creature_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grave_pact());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    kill_with_bolt(&mut g, mine);
    assert!(g.battlefield_find(theirs).is_none(), "opponent sacrificed a creature");
}

#[test]
fn butcher_of_malakir_edicts_when_your_creature_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::butcher_of_malakir());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    kill_with_bolt(&mut g, mine);
    assert!(g.battlefield_find(theirs).is_none(), "opponent sacrificed a creature");
}

// ── Red combat tricks + menace anthem ──────────────────────────────────────

#[test]
fn assault_strobe_grants_double_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::assault_strobe());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, id, Target::Permanent(bear));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

#[test]
fn uncaged_fury_pumps_and_grants_double_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::uncaged_fury());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, id, Target::Permanent(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "2/2 + 1/1");
    assert!(cp.keywords.contains(&Keyword::DoubleStrike));
}

#[test]
fn goblin_war_drums_grants_menace_to_your_creatures() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goblin_war_drums());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.computed_permanent(mine).unwrap().keywords.contains(&Keyword::Menace), "yours get menace");
    assert!(!g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::Menace), "opponent's don't");
}

// ── Green bodies + Eldrazi Spawn ramp ──────────────────────────────────────

#[test]
fn nest_invader_makes_an_eldrazi_spawn() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::nest_invader());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Nest Invader");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count(), 1,
        "one Eldrazi Spawn token");
}

#[test]
fn kozileks_predator_makes_two_spawns() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::kozileks_predator());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Kozilek's Predator");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count(), 2,
        "two Eldrazi Spawn tokens");
}

#[test]
fn greater_basilisk_has_deathtouch() {
    use crabomination::card::Keyword;
    let b = catalog::greater_basilisk();
    assert!(b.keywords.contains(&Keyword::Deathtouch) && (b.power, b.toughness) == (3, 5));
}

// ── Green burn + combat trick ──────────────────────────────────────────────

#[test]
fn hornet_sting_pings_any_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::hornet_sting());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, 19, "1 damage to the opponent");
}

#[test]
fn titanic_growth_pumps_four_four() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::titanic_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, id, Target::Permanent(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "2/2 + 4/4 = 6/6");
}

// ── White removal + utility ────────────────────────────────────────────────

#[test]
fn reprisal_destroys_a_big_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::reprisal());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, id, Target::Permanent(big));
    assert!(g.battlefield_find(big).is_none(), "the power-4 creature is destroyed");
}

#[test]
fn icatian_javelineers_pings_once_then_runs_dry() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let jav = g.add_card_to_battlefield(0, catalog::icatian_javelineers());
    g.battlefield_find_mut(jav).unwrap().add_counters(CounterType::Charge, 1);
    g.clear_sickness(jav);
    g.perform_action(GameAction::ActivateAbility {
        card_id: jav, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("ping for 1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "dealt 1 to the opponent");
    // Untap it so the only blocker to re-activation is the missing counter.
    g.battlefield_find_mut(jav).unwrap().tapped = false;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: jav, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).is_err(), "can't ping again with no counter");
}

#[test]
fn leonin_relic_warder_exiles_then_returns_on_death() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(1, catalog::ur_golems_eye());
    let warder = g.add_card_to_hand(0, catalog::leonin_relic_warder());
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: warder, target: Some(Target::Permanent(relic)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Leonin Relic-Warder castable for {W}{W}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(relic).is_none(), "artifact exiled by the Warder's ETB");
    // Kill the Warder — the exiled artifact returns (CR 603.6e).
    g.battlefield_find_mut(warder).unwrap().damage = 5;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == relic), "artifact returns on the Warder's death");
}

// ── Tempo illusions + Spiketail Hatchling ──────────────────────────────────

#[test]
fn jaces_phantasm_grows_with_opponent_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::jaces_phantasm());
    assert_eq!(g.computed_permanent(id).unwrap().power, 1, "1/1 with a small graveyard");
    for _ in 0..10 { g.add_card_to_graveyard(1, catalog::forest()); }
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "+4/+4 once an opponent has ten in graveyard");
}

#[test]
fn phantasmal_bear_sacrifices_when_targeted() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::phantasmal_bear());
    g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: bear, caster: 1 }]);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != bear), "sacrificed when it became a target");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear), "in its owner's graveyard");
}

#[test]
fn spiketail_hatchling_counters_unless_paid() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt on the stack");
    // P1 has spent their only mana and can't pay the {1}.
    g.priority.player_with_priority = 0;
    let spike = g.add_card_to_battlefield(0, catalog::spiketail_hatchling());
    g.perform_action(GameAction::ActivateAbility {
        card_id: spike, ability_index: 0, target: Some(Target::Permanent(bolt)), additional_targets: Vec::new(), x_value: None,
    }).expect("sacrifice Spiketail to counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "bolt countered (controller couldn't pay)");
    assert!(g.battlefield.iter().all(|c| c.id != spike), "Spiketail Hatchling sacrificed");
}

// ── Utility artifacts (Icy Manipulator + monoliths) ────────────────────────

#[test]
fn icy_manipulator_taps_a_target() {
    let mut g = two_player_game();
    let icy = g.add_card_to_battlefield(0, catalog::icy_manipulator());
    g.clear_sickness(icy);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: icy, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("Icy Manipulator taps a target");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "the target creature is tapped");
}

#[test]
fn basalt_monolith_makes_three_and_needs_three_to_untap() {
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::basalt_monolith());
    g.clear_sickness(m);
    g.perform_action(GameAction::ActivateAbility {
        card_id: m, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for {C}{C}{C}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 3, "three colorless");
    // CR 502.3 — doesn't untap normally.
    g.do_untap();
    assert!(g.battlefield_find(m).unwrap().tapped, "Basalt Monolith stays tapped through untap");
    // Pay {3} to untap.
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: m, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("pay 3 to untap");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(m).unwrap().tapped, "untapped after paying three");
}

// ── Misc cube creatures ────────────────────────────────────────────────────

#[test]
fn kokusho_dies_drains_five() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::kokusho_the_evening_star());
    g.battlefield_find_mut(id).unwrap().damage = 5;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 15, "each opponent lost 5");
    assert_eq!(g.players[0].life, 25, "you gained the 5 lost");
}

#[test]
fn ophidian_draws_when_unblocked() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_battlefield(0, catalog::ophidian());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let effect = catalog::ophidian().triggered_abilities[0].effect.clone();
    let before = g.players[0].hand.len();
    let ctx = crabomination::game::effects::EffectContext::for_ability(id, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), before + 1, "Ophidian drew when unblocked");
}

#[test]
fn legion_loyalist_battalion_grants_first_strike_and_trample() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let ll = g.add_card_to_battlefield(0, catalog::legion_loyalist());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let effect = catalog::legion_loyalist().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(ll, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::FirstStrike) && cp.keywords.contains(&Keyword::Trample),
        "battalion grants first strike + trample to your creatures");
}

#[test]
fn torch_courier_sacrifices_to_grant_haste() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let tc = g.add_card_to_battlefield(0, catalog::torch_courier());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: tc, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("sacrifice Torch Courier for haste");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != tc), "Torch Courier was sacrificed");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste),
        "the target creature gained haste");
}

// ── Chance Encounter (CR 705.1 — win-a-flip payoff) ────────────────────────

#[test]
fn chance_encounter_gains_luck_on_won_flip() {
    use crabomination::card::CounterType;
    use crabomination::effect::{Effect, Value};
    let mut g = two_player_game();
    let ce = g.add_card_to_battlefield(0, catalog::chance_encounter());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // heads
    let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    let events = g.resolve_effect(&Effect::FlipCoin {
        count: Value::Const(1),
        on_heads: Box::new(Effect::Noop),
        on_tails: Box::new(Effect::Noop),
    }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let n = g.battlefield.iter().find(|c| c.id == ce).unwrap()
        .counters.get(&CounterType::Luck).copied().unwrap_or(0);
    assert_eq!(n, 1, "won flip put a luck counter on Chance Encounter");
}

#[test]
fn chance_encounter_wins_the_game_at_ten_luck() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let ce = g.add_card_to_battlefield(0, catalog::chance_encounter());
    g.battlefield.iter_mut().find(|c| c.id == ce).unwrap()
        .add_counters(CounterType::Luck, 10);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.is_game_over(), "ten luck counters wins the game at upkeep");
    assert!(g.players[1].eliminated, "the opponent was eliminated");
}

// CR 705.1 — "Whenever you lose a coin flip" trigger event. No common
// printed card uses it, so the listener is a synthetic permanent: it adds a
// +1/+1 counter to itself each time its controller loses a flip.
#[test]
fn lost_coin_flip_fires_lose_trigger() {
    use crabomination::card::{CardDefinition, CardId, CardType, CounterType};
    use crabomination::effect::{Effect, EventKind, EventScope, EventSpec, Selector, TriggeredAbility, Value};
    let mut g = two_player_game();
    let mut def = CardDefinition {
        name: "Flip Loser",
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        ..Default::default()
    };
    def.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::LostCoinFlip, EventScope::YourControl),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    }];
    let id = g.add_card_to_battlefield(0, def);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)])); // tails → lose
    let ctx = crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None);
    let events = g.resolve_effect(&Effect::FlipCoin {
        count: Value::Const(1),
        on_heads: Box::new(Effect::Noop),
        on_tails: Box::new(Effect::Noop),
    }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let n = g.battlefield.iter().find(|c| c.id == id).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(n, 1, "losing the flip fired the lose-flip trigger");
}

// ── Coin-flip cycle (CR 705) + CR 506.4 RemoveFromCombat ────────────────────

#[test]
fn mijae_djinn_lost_flip_removed_from_combat() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mijae_djinn());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)])); // tails → lose
    g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert!(!g.attacking.iter().any(|a| a.attacker == id),
        "losing the flip removes Mijae Djinn from combat");
}

#[test]
fn mijae_djinn_won_flip_stays_attacking() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mijae_djinn());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // heads → win
    g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert!(g.attacking.iter().any(|a| a.attacker == id),
        "winning the flip keeps Mijae Djinn attacking");
}

#[test]
fn ydwen_efreet_lost_block_removed_from_combat() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(1, catalog::blade_of_the_sixth_pride()); // 3/1 vanilla
    let blk = g.add_card_to_battlefield(0, catalog::ydwen_efreet());
    g.clear_sickness(atk);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(0) }]).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)])); // tails → lose
    let events = g.declare_blockers(vec![(blk, atk)]).expect("block");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(!g.block_map.contains_key(&blk),
        "losing the flip removes Ydwen Efreet from combat as a blocker");
}

#[test]
fn squee_returns_from_graveyard_at_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::squee_goblin_nabob());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // MayDo yes
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Squee, Goblin Nabob"),
        "Squee returns from the graveyard to hand at upkeep");
}

#[test]
fn stitch_in_time_banks_extra_turn_on_win() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::stitch_in_time());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // heads
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stitch in Time castable for {1}{U}{R}");
    drain_stack(&mut g);
    assert!(g.players[0].extra_turns >= 1, "winning the flip banks an extra turn");
}

#[test]
fn krark_copies_your_spell_on_won_flip() {
    let mut g = two_player_game();
    let _k = g.add_card_to_battlefield(0, catalog::krark_the_thumbless());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // heads → copy
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lightning Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "Bolt + its Krark copy each deal 3");
}

#[test]
fn krark_returns_your_spell_on_lost_flip() {
    let mut g = two_player_game();
    let _k = g.add_card_to_battlefield(0, catalog::krark_the_thumbless());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)])); // tails → return
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lightning Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "losing the flip returns the Bolt before it resolves");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "the spell is returned to its owner's hand");
}

#[test]
fn boggart_shenanigans_pings_when_a_goblin_dies() {
    use crabomination::decision::DecisionAnswer::Target as T;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::boggart_shenanigans());
    let gob = g.add_card_to_battlefield(0, catalog::mogg_fanatic());
    let foe_life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),   // MayDo: yes
        T(Target::Player(1)),         // ability target
    ]));
    // Kill the Goblin with a Bolt so SBA dispatches CreatureDied.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(gob)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt goblin");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 1, "a dying Goblin pings the chosen player for 1");
}

#[test]
fn mudbrawler_cohort_grows_with_another_red_creature() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::mudbrawler_cohort());
    let b = g.add_card_to_battlefield(0, catalog::mudbrawler_cohort());
    let pt = |g: &GameState, id| g.compute_battlefield().iter()
        .find(|c| c.id == id).map(|c| (c.power, c.toughness));
    assert_eq!(pt(&g, a), Some((2, 2)), "each Cohort sees the other red creature");
    g.remove_to_graveyard_with_triggers(b);
    assert_eq!(pt(&g, a), Some((1, 1)), "back to 1/1 with no other red creature");
}

#[test]
fn goblin_bomb_upkeep_flip_adds_fuse_on_win() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::goblin_bomb());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true), // MayDo: yes, flip
        DecisionAnswer::Bool(true), // coin: heads → win
    ]));
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let n = g.battlefield_find(id).unwrap()
        .counters.get(&CounterType::Fuse).copied().unwrap_or(0);
    assert_eq!(n, 1, "winning the upkeep flip adds a fuse counter");
}

#[test]
fn goblin_bomb_detonates_at_five_fuse() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::goblin_bomb());
    // Under five counters, the payoff can't be activated.
    g.battlefield_find_mut(id).unwrap().add_counters(CounterType::Fuse, 4);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None }).is_err(),
        "four fuse counters is not enough to detonate");
    g.battlefield_find_mut(id).unwrap().add_counters(CounterType::Fuse, 1); // → 5
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None })
        .expect("detonates at five fuse counters");
    drain_stack(&mut g);
    assert!(g.players[1].life <= 0, "Goblin Bomb deals 20 to the target player");
}

#[test]
fn fiery_gambit_three_wins_fires_all_tiers() {
    let mut g = two_player_game();
    let gambit = g.add_card_to_hand(0, catalog::fiery_gambit());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    // win, again, win, again, win, stop → 3 wins.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true), DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true), DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true), DecisionAnswer::Bool(false),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: gambit, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None }).expect("Fiery Gambit castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "1+ wins: 3 damage kills the 2/2");
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3, "2+ wins: draw three (minus the cast Gambit)");
    assert_eq!(g.players[1].life, 15, "3+ wins: each opponent loses 5");
}

#[test]
fn fiery_gambit_lost_flip_does_nothing() {
    let mut g = two_player_game();
    let gambit = g.add_card_to_hand(0, catalog::fiery_gambit());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)])); // lose first flip
    g.perform_action(GameAction::CastSpell {
        card_id: gambit, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "losing the first flip cancels everything");
    assert_eq!(g.players[1].life, 20, "no life loss on a lost gambit");
}

// ── Utility artifacts ───────────────────────────────────────────────────────

#[test]
fn meteorite_etb_pings_and_taps_for_any_color() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let id = g.add_card_to_battlefield(0, catalog::meteorite());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "ETB deals 2 to any target");
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None }).expect("mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "{{T}}: add one mana of any color");
}

#[test]
fn basilisk_collar_grants_deathtouch_and_lifelink() {
    let mut g = two_player_game();
    let collar = g.add_card_to_battlefield(0, catalog::basilisk_collar());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: collar, target: bear }).expect("equip {2}");
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Deathtouch), "deathtouch granted");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Lifelink), "lifelink granted");
}

#[test]
fn hammer_of_bogardan_recurs_from_graveyard_at_upkeep() {
    let mut g = two_player_game();
    let hammer = g.add_card_to_graveyard(0, catalog::hammer_of_bogardan());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Outside your upkeep the recur ability is illegal.
    g.step = TurnStep::PreCombatMain;
    for _ in 0..3 { g.players[0].mana_pool.add(Color::Red, 1); }
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: hammer, ability_index: 0, target: None, additional_targets: vec![], x_value: None }).is_err(),
        "recur is upkeep-only");
    g.step = TurnStep::Upkeep;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hammer, ability_index: 0, target: None, additional_targets: vec![], x_value: None }).expect("recur at upkeep");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == hammer), "Hammer returns to hand");
}

#[test]
fn stalking_stones_animates_into_a_3_3() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::stalking_stones());
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: vec![], x_value: None }).expect("animate for {6}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature), "becomes a creature");
    assert_eq!((cp.power, cp.toughness), (3, 3), "a 3/3 Elemental");
}

#[test]
fn minds_eye_draws_when_opponent_draws_and_you_pay() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::minds_eye());
    for _ in 0..2 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    for _ in 0..2 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // MayPay: yes
    let my_hand = g.players[0].hand.len();
    let mut ev = vec![];
    g.draw_one(1, &mut ev); // opponent draws
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), my_hand + 1, "paying {{1}} draws you a card");
}

#[test]
fn fire_diamond_enters_tapped_then_taps_for_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fire_diamond());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Fire Diamond castable for {2}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().tapped, "Diamond enters tapped");
    g.battlefield_find_mut(id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None }).expect("tap for red");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "{{T}}: add {{R}}");
}

#[test]
fn pristine_talisman_taps_for_colorless_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pristine_talisman());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None }).expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "adds one colorless mana");
    assert_eq!(g.players[0].life, 21, "and you gain 1 life");
}

#[test]
fn sunbeam_spellbomb_gains_five_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sunbeam_spellbomb());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None }).expect("gain-life mode");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 25, "{{W}}, Sac: gain 5 life");
    assert!(!g.battlefield.iter().any(|c| c.id == id), "Sunbeam Spellbomb is sacrificed");
}

#[test]
fn necrogen_spellbomb_makes_target_discard() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::necrogen_spellbomb());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let before = g.players[1].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None })
        .expect("discard mode");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), before - 1, "target player discards a card");
}

#[test]
fn elixir_of_immortality_recycles_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::elixir_of_immortality());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None }).expect("recycle");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 25, "gain 5 life");
    assert!(g.players[0].graveyard.is_empty(), "graveyard shuffled into library");
    assert!(g.players[0].library.iter().any(|c| c.id == id), "Elixir shuffled into library");
}

#[test]
fn crystal_shard_bounces_unpaid_creature() {
    let mut g = two_player_game();
    let shard = g.add_card_to_battlefield(0, catalog::crystal_shard());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1); // {U},{T} mode (index 1)
    g.perform_action(GameAction::ActivateAbility {
        card_id: shard, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None })
        .expect("bounce mode");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "unpaid creature returns to its owner's hand");
}

#[test]
fn erratic_portal_bounces_unpaid_creature() {
    let mut g = two_player_game();
    let portal = g.add_card_to_battlefield(0, catalog::erratic_portal());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: portal, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None })
        .expect("bounce");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "unpaid creature returns to its owner's hand");
}

// CR 706.6 — "Whenever you roll one or more dice" fires once per roll
// instruction (not once per die). The synthetic listener gains 1 life.
#[test]
fn rolled_dice_fires_once_per_roll() {
    use crabomination::card::{CardDefinition, CardId, CardType};
    use crabomination::effect::{Effect, EventKind, EventScope, EventSpec, Selector, TriggeredAbility, Value};
    let mut g = two_player_game();
    let mut def = CardDefinition {
        name: "Die Watcher",
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    };
    def.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::RolledDice, EventScope::YourControl),
        effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
    }];
    g.add_card_to_battlefield(0, def);
    let life0 = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(3), DecisionAnswer::DieRoll(5)]));
    let ctx = crabomination::game::effects::EffectContext::for_ability(CardId(0), 0, None);
    // Roll two dice in one instruction → trigger fires exactly once.
    let events = g.resolve_effect(&Effect::RollDie {
        sides: 6,
        count: Value::Const(2),
        modifier: Value::Const(0),
        reroll_at_most: 0,
        results: vec![],
        on_doubles: None,
    }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1, "two dice in one roll → +1 life once");
}

// ── Ancient Copper Dragon (CR 706.4 — Value::LastDieRoll) ──────────────────

#[test]
fn ancient_copper_dragon_makes_treasures_equal_to_d20() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::ancient_copper_dragon());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(7)]));
    let effect = catalog::ancient_copper_dragon().triggered_abilities[0].effect.clone();
    let before = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
    let ctx = crabomination::game::effects::EffectContext::for_ability(dragon, 0, None);
    g.resolve_effect(&effect, &ctx).unwrap();
    let after = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(after - before, 7, "d20 = 7 → seven Treasure tokens");
}

// ── Rest in Peace / Leyline of the Void (CR 614.6 graveyard hate) ──────────

#[test]
fn rest_in_peace_etb_exiles_all_graveyards() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::rest_in_peace());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {1}{W}");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.is_empty() && g.players[1].graveyard.is_empty(),
        "all graveyards emptied");
    assert_eq!(g.exile.len(), 2, "both cards exiled");
}

#[test]
fn rest_in_peace_exiles_dying_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rest_in_peace());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_from_battlefield_to_graveyard_raw(bear);
    assert!(g.players[0].graveyard.is_empty(), "creature did not reach the graveyard");
    assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled instead");
}

#[test]
fn leyline_of_the_void_exiles_only_opponents_cards() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    // Player 0 controls the Leyline; mills hit each library.
    g.add_card_to_battlefield(0, catalog::leyline_of_the_void());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(1, catalog::forest());
    let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    // Mill self — controller's own card still goes to the graveyard.
    g.resolve_effect(&Effect::Mill { who: Selector::You, amount: Value::Const(1) }, &ctx).unwrap();
    assert_eq!(g.players[0].graveyard.len(), 1, "own milled card stays in graveyard");
    // Mill the opponent — exiled instead.
    let octx = crabomination::game::effects::EffectContext::for_ability(
        crabomination::card::CardId(0), 0, Some(Target::Player(1)),
    );
    g.resolve_effect(&Effect::Mill { who: Selector::Target(0), amount: Value::Const(1) }, &octx).unwrap();
    assert!(g.players[1].graveyard.is_empty(), "opponent's milled card exiled");
    assert_eq!(g.exile.len(), 1, "one card exiled");
}

// ── Inspiration / Opportunity (targeted draw) ──────────────────────────────

#[test]
fn opportunity_draws_target_player_four() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::opportunity());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Opportunity at self");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 4);
}

