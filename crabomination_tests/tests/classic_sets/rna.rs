//! Functionality tests for Ravnica Allegiance (RNA) — `catalog::sets::rna`.

use crabomination::card::{CardType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// Stat / keyword lines for the RNA batch.
#[test]
fn rna_stat_and_keyword_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::catacomb_crocodile, 3, 7, &[]),
        (catalog::azorius_knight_arbiter, 2, 5, &[Keyword::Vigilance, Keyword::Unblockable]),
        (catalog::carrion_imp, 2, 3, &[Keyword::Flying]),
        (catalog::civic_stalwart, 3, 3, &[]),
        (catalog::blade_juggler, 3, 2, &[]),
        (catalog::devkarin_dissident, 2, 2, &[]),
        (catalog::passwall_adept, 1, 3, &[]),
        (catalog::rakdos_firewheeler, 4, 3, &[]),
        (catalog::gyre_engineer, 1, 1, &[]),
        (catalog::vizkopa_vampire, 3, 1, &[Keyword::Lifelink]),
        (catalog::rubblebelt_recluse, 6, 5, &[Keyword::MustAttack]),
        (catalog::rakdos_trumpeter, 1, 3, &[Keyword::Menace]),
        (catalog::griffin_protector, 2, 3, &[Keyword::Flying]),
        (catalog::ironshell_beetle, 1, 1, &[]),
    ];
    for (f, p, t, kws) in table {
        let c = f();
        assert_eq!((c.power, c.toughness), (*p, *t), "{} P/T", c.name);
        for kw in *kws {
            assert!(c.keywords.contains(kw), "{} should have {:?}", c.name, kw);
        }
    }
}

/// Civic Stalwart pumps the team +1/+1 on entry.
#[test]
fn civic_stalwart_pumps_team() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::civic_stalwart());
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "bear +1/+1 until EOT");
}

/// Bring to Trial exiles a power-4+ creature (and can't hit a smaller one).
#[test]
fn bring_to_trial_exiles_big_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let cast = g.add_card_to_hand(0, catalog::bring_to_trial());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(big)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == big), "power-6 creature exiled");
}

/// Carrion Imp's ETB exiles a creature card from a graveyard and gains 2 life.
#[test]
fn carrion_imp_exiles_and_gains() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(corpse)), 0, 0);
    let imp = catalog::carrion_imp();
    let effect = imp.triggered_abilities[0].effect.clone();
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.exile.iter().any(|c| c.id == corpse), "creature card exiled from graveyard");
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}

/// Rakdos Firewheeler burns an opponent for 2 and a creature for 2 on entry.
#[test]
fn rakdos_firewheeler_double_burn() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Player(1), Target::Permanent(foe)],
        ..crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0)
    };
    let fw = catalog::rakdos_firewheeler();
    let effect = fw.triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    assert_eq!(g.players[1].life, opp - 2, "opponent took 2");
    assert!(g.battlefield_find(foe).is_none(), "the 2/2 took 2 and died");
}

/// Devkarin Dissident can firebreathe +2/+2.
#[test]
fn devkarin_dissident_pumps_self() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::devkarin_dissident());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility { card_id: elf, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(elf).unwrap().power, 4, "+2/+2 → 4/4");
}

/// Passwall Adept makes a creature unblockable for the turn.
#[test]
fn passwall_adept_grants_unblockable() {
    let mut g = two_player_game();
    let adept = g.add_card_to_battlefield(0, catalog::passwall_adept());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: adept, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None }).expect("activate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable), "bear is unblockable");
}

/// Burn Bright pumps the whole team +2/+0.
#[test]
fn burn_bright_team_pump() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cast = g.add_card_to_hand(0, catalog::burn_bright());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0");
}

/// Applied Biomancy is a true "choose one or both" modal (pump / bounce).
#[test]
fn applied_biomancy_is_one_or_both_modal() {
    match catalog::applied_biomancy().effect {
        crabomination::effect::Effect::ChooseModesCast { modes, min, max, .. } => {
            assert_eq!((modes.len(), min, max), (2, 1, 2), "two modes, choose one or both");
        }
        _ => panic!("expected ChooseModesCast"),
    }
}

/// Gyre Engineer taps for {G}{U}.
#[test]
fn gyre_engineer_taps_for_gu() {
    let mut g = two_player_game();
    let eng = g.add_card_to_battlefield(0, catalog::gyre_engineer());
    g.clear_sickness(eng);
    let (idx, _) = g.effective_mana_abilities(eng).into_iter().next().expect("mana ability");
    g.perform_action(GameAction::ActivateAbility { card_id: eng, ability_index: idx, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
}

/// Blade Juggler is castable for its Spectacle cost after an opponent lost life.
#[test]
fn blade_juggler_has_spectacle() {
    let def = catalog::blade_juggler();
    assert!(def.alternative_cost.is_some(), "Blade Juggler has a Spectacle alt-cost");
    assert!(def.card_types.contains(&CardType::Creature));
}

/// Arrester's Zeal grants flying when cast during your main phase (Addendum),
/// but not when cast at instant speed off your main phase.
#[test]
fn arresters_zeal_addendum_flying() {
    use crabomination::card::Keyword;
    // On your main phase → +2/+2 and flying.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cast = g.add_card_to_hand(0, catalog::arresters_zeal());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Flying), "Addendum grants flying on your main phase");

    // On the opponent's turn → +2/+2 only, no flying.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cast = g.add_card_to_hand(0, catalog::arresters_zeal());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(!cp.keywords.contains(&Keyword::Flying), "no Addendum off your main phase");
}

/// Arrester's Admonition bounces a creature and draws under its Addendum.
#[test]
fn arresters_admonition_addendum_draw() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cast = g.add_card_to_hand(0, catalog::arresters_admonition());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature bounced");
    // Spent one card (the spell) but drew one via Addendum → net hand unchanged.
    assert_eq!(g.players[0].hand.len(), hand, "Addendum drew a card");
}

/// Ironshell Beetle puts a +1/+1 counter on a creature when it enters.
#[test]
fn ironshell_beetle_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    let effect = catalog::ironshell_beetle().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}

/// Griffin Protector pumps itself when another creature you control enters.
#[test]
fn griffin_protector_self_pump_on_other_etb() {
    let mut g = two_player_game();
    let griffin = g.add_card_to_battlefield(0, catalog::griffin_protector());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(griffin).unwrap().power, 3, "Griffin gets +1/+1 when another creature enters");
}
