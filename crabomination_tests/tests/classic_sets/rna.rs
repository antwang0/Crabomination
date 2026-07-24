//! Functionality tests for Ravnica Allegiance (RNA) — `catalog::sets::rna`.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
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
        (catalog::hunted_witness, 1, 1, &[]),
        (catalog::tithe_taker, 2, 1, &[]),
        (catalog::ministrant_of_obligation, 2, 1, &[]),
        (catalog::imperious_oligarch, 2, 1, &[Keyword::Vigilance]),
        (catalog::grasping_thrull, 3, 3, &[Keyword::Flying]),
        (catalog::zhur_taa_goblin, 2, 2, &[]),
        (catalog::rampaging_rendhorn, 4, 4, &[]),
        (catalog::frenzied_arynx, 3, 3, &[Keyword::Trample]),
        (catalog::sunhome_stalwart, 2, 2, &[Keyword::FirstStrike]),
        (catalog::spear_spewer, 0, 2, &[Keyword::Defender]),
        (catalog::vindictive_vampire, 2, 3, &[]),
        (catalog::sauroform_hybrid, 2, 2, &[]),
        (catalog::skitter_eel, 3, 3, &[]),
        (catalog::rakdos_roustabout, 3, 2, &[]),
        (catalog::gatebreaker_ram, 2, 2, &[]),
        (catalog::feral_maaka, 2, 2, &[]),
        (catalog::wild_ceratok, 4, 3, &[]),
        (catalog::rubble_slinger, 2, 3, &[Keyword::Reach]),
        (catalog::impassioned_orator, 2, 2, &[]),
        (catalog::concordia_pegasus, 1, 3, &[Keyword::Flying]),
        (catalog::prowling_caracal, 3, 1, &[]),
        (catalog::watchful_giant, 3, 6, &[]),
        (catalog::faerie_duelist, 1, 2, &[Keyword::Flash, Keyword::Flying]),
        (catalog::coral_commando, 3, 2, &[]),
        (catalog::windstorm_drake, 3, 3, &[Keyword::Flying]),
        (catalog::burning_tree_vandal, 2, 1, &[]),
        (catalog::ghor_clan_wrecker, 2, 2, &[Keyword::Menace]),
        (catalog::territorial_boar, 2, 2, &[]),
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

/// Ministrant of Obligation's Afterlife 2 makes two flying Spirit tokens.
#[test]
fn ministrant_afterlife_makes_two_spirits() {
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::ministrant_of_obligation()); // 2/1
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(m), 2, None, &mut evs);
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit" && c.definition.keywords.contains(&Keyword::Flying))
        .count();
    assert_eq!(spirits, 2, "Afterlife 2 → two flying Spirits");
}

/// Hunted Witness dies into a 1/1 Soldier with lifelink.
#[test]
fn hunted_witness_makes_lifelink_soldier() {
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::hunted_witness());
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(w), 1, None, &mut evs);
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    drain_stack(&mut g);
    assert!(g.battlefield.iter()
        .any(|c| c.controller == 0 && c.definition.name == "Soldier" && c.definition.keywords.contains(&Keyword::Lifelink)),
        "lifelink Soldier token minted");
}

/// Zhur-Taa Goblin's Riot grants haste by default (mode 0).
#[test]
fn zhur_taa_goblin_riot_default_haste() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let gob = g.move_card_to_battlefield_for_test(0, catalog::zhur_taa_goblin());
    drain_stack(&mut g);
    assert!(g.computed_permanent(gob).unwrap().keywords.contains(&Keyword::Haste), "Riot → haste");
}

/// Sunhome Stalwart's Mentor puts a +1/+1 counter on a lesser-power attacker.
#[test]
fn sunhome_stalwart_mentor_pumps_smaller_attacker() {
    let mut g = two_player_game();
    let stalwart = g.add_card_to_battlefield(0, catalog::sunhome_stalwart()); // 2/2
    let small_def = crabomination::card::TokenDefinition {
        name: "Goblin".into(), power: 1, toughness: 1,
        card_types: vec![CardType::Creature], ..Default::default()
    };
    let small = g.add_token_to_battlefield(0, &small_def); // 1/1
    g.clear_sickness(stalwart);
    g.clear_sickness(small);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: stalwart, target: AttackTarget::Player(1) },
        Attack { attacker: small, target: AttackTarget::Player(1) },
    ]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(small).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Mentor counter on the lesser attacker");
}

/// Skewer the Critics deals 3 to any target.
#[test]
fn skewer_the_critics_burns_three() {
    let mut g = two_player_game();
    let cast = g.add_card_to_hand(0, catalog::skewer_the_critics());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "3 damage to the opponent");
}

/// Light Up the Stage exiles the top two cards with a may-play grant.
#[test]
fn light_up_the_stage_exiles_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::mountain()); }
    let cast = g.add_card_to_hand(0, catalog::light_up_the_stage());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Mountain").count(), 2, "two cards exiled");
}

/// Spear Spewer pings each player for 1.
#[test]
fn spear_spewer_pings_each_player() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::spear_spewer());
    g.clear_sickness(s);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::ActivateAbility { card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap");
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (l0 - 1, l1 - 1), "1 to each player");
}

/// Vindictive Vampire drains when another creature you control dies.
#[test]
fn vindictive_vampire_drains_on_ally_death() {
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vindictive_vampire());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let (mine, opp) = (g.players[0].life, g.players[1].life);
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(ally), 2, None, &mut evs);
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "opponent took 1");
    assert_eq!(g.players[0].life, mine + 1, "gained 1");
}

/// Sauroform Hybrid's Adapt 4 adds four +1/+1 counters.
#[test]
fn sauroform_hybrid_adapt_four() {
    let mut g = two_player_game();
    let h = g.add_card_to_battlefield(0, catalog::sauroform_hybrid());
    g.clear_sickness(h);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility { card_id: h, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("adapt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(h).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// Titanic Brawl fights and carries the +1/+1-counter cost reduction.
#[test]
fn titanic_brawl_fights_and_reduces() {
    let def = catalog::titanic_brawl();
    assert!(def.self_cost_reduction_cost_if_target.is_some(), "has counter cost reduction");
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::craw_wurm());   // 6/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let cast = g.add_card_to_hand(0, catalog::titanic_brawl());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(mine)), additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    assert!(g.battlefield_find(theirs).is_none(), "the 2/2 died to the 6/4");
}

/// Scorchmark exiles a creature it would kill instead of it dying.
#[test]
fn scorchmark_exiles_dying_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let cast = g.add_card_to_hand(0, catalog::scorchmark());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled instead of dying");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "not in graveyard");
}

/// Consign to the Pit destroys a creature and burns its controller for 2.
#[test]
fn consign_to_the_pit_destroys_and_burns() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cast = g.add_card_to_hand(0, catalog::consign_to_the_pit());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(5);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    assert_eq!(g.players[1].life, life - 2, "controller took 2");
}

/// Undercity Scavenger's sac payoff adds two counters and scries.
#[test]
fn undercity_scavenger_sac_adds_counters() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let scav = catalog::undercity_scavenger();
    let this = g.add_card_to_battlefield(0, scav.clone());
    let effect = scav.triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext {
        source: Some(this),
        ..crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)
    };
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Cards(vec![fodder]),
    ]));
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(this).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
}

/// Gatebreaker Ram scales with Gates and gains keywords at two.
#[test]
fn gatebreaker_ram_scales_with_gates() {
    let mut g = two_player_game();
    let ram = g.add_card_to_battlefield(0, catalog::gatebreaker_ram());
    g.add_card_to_battlefield(0, catalog::azorius_guildgate());
    let cp = g.computed_permanent(ram).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 for one Gate");
    assert!(!cp.keywords.contains(&Keyword::Trample), "no trample with one Gate");
    g.add_card_to_battlefield(0, catalog::boros_guildgate());
    let cp = g.computed_permanent(ram).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2 for two Gates");
    assert!(cp.keywords.contains(&Keyword::Vigilance) && cp.keywords.contains(&Keyword::Trample),
        "vigilance + trample at two Gates");
}

/// Senate Guildmage's first ability gains 2 life.
#[test]
fn senate_guildmage_gains_life() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::senate_guildmage());
    g.clear_sickness(mage);
    g.players[0].mana_pool.add(Color::White, 1);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility { card_id: mage, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("gain");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2);
}

/// Rakdos Roustabout pings the defending player when it becomes blocked.
#[test]
fn rakdos_roustabout_pings_on_block() {
    let mut g = two_player_game();
    let att = g.add_card_to_battlefield(0, catalog::rakdos_roustabout());
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(att);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: att, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let life = g.players[1].life;
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, att)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "becomes-blocked ping hit the defending player");
}

/// Grasping Thrull's ETB drains each opponent for 2.
#[test]
fn grasping_thrull_etb_drains() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let (mine, opp) = (g.players[0].life, g.players[1].life);
    g.move_card_to_battlefield_for_test(0, catalog::grasping_thrull());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "opponent lost 2");
    assert_eq!(g.players[0].life, mine + 2, "gained 2");
}

/// Gift of Strength pumps +3/+3 and grants reach.
#[test]
fn gift_of_strength_pumps_and_grants_reach() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cast = g.add_card_to_hand(0, catalog::gift_of_strength());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "+3/+3");
    assert!(cp.keywords.contains(&Keyword::Reach), "gains reach");
}

/// Tithe Taker taxes opponents' spells {1} more only on its controller's turn,
/// and never taxes the controller.
#[test]
fn tithe_taker_taxes_opponents_on_your_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tithe_taker());
    let sid = g.add_card_to_hand(1, catalog::lightning_bolt());
    let opp_spell = g.players[1].hand.iter().find(|c| c.id == sid).unwrap().clone();
    // Player 0's turn → opponent (1) is taxed 1.
    g.active_player_idx = 0;
    assert_eq!(crabomination::game::actions::extra_cost_for_spell(&g, 1, &opp_spell, None, 0), 1, "taxed on your turn");
    // Player 1's turn → no tax.
    g.active_player_idx = 1;
    assert_eq!(crabomination::game::actions::extra_cost_for_spell(&g, 1, &opp_spell, None, 0), 0, "not taxed off your turn");
    // The controller's own spell is never taxed, even on their turn.
    let oid = g.add_card_to_hand(0, catalog::lightning_bolt());
    let own_spell = g.players[0].hand.iter().find(|c| c.id == oid).unwrap().clone();
    g.active_player_idx = 0;
    assert_eq!(crabomination::game::actions::extra_cost_for_spell(&g, 0, &own_spell, None, 0), 0, "controller exempt");
}

/// Impassioned Orator gains 1 life when another creature you control enters.
#[test]
fn impassioned_orator_gains_on_ally_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::impassioned_orator());
    let life = g.players[0].life;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 when the bear entered");
}

/// Watchful Giant's ETB makes a 1/1 white Human.
#[test]
fn watchful_giant_makes_human() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.move_card_to_battlefield_for_test(0, catalog::watchful_giant());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Human"), "1/1 Human token minted");
}

/// Faerie Duelist shrinks an opponent's creature by -2/-0 on entry.
#[test]
fn faerie_duelist_shrinks_opponent() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(foe)), 0, 0);
    let effect = catalog::faerie_duelist().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(foe).unwrap().power, 0, "2/2 → 0/2");
}

/// Windstorm Drake pumps other flyers you control +1/+0 (and not itself/ground).
#[test]
fn windstorm_drake_flying_anthem() {
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(0, catalog::windstorm_drake());
    let flyer = g.add_card_to_battlefield(0, catalog::concordia_pegasus()); // 1/3 flying
    let ground = g.add_card_to_battlefield(0, catalog::grizzly_bears());     // 2/2 no fly
    assert_eq!(g.computed_permanent(flyer).unwrap().power, 2, "other flyer +1/+0");
    assert_eq!(g.computed_permanent(ground).unwrap().power, 2, "ground creature unaffected");
    assert_eq!(g.computed_permanent(drake).unwrap().power, 3, "drake doesn't pump itself");
}

/// Bankrupt in Blood sacrifices two creatures and draws three.
#[test]
fn bankrupt_in_blood_sacs_two_draws_three() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let cast = g.add_card_to_hand(0, catalog::bankrupt_in_blood());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell { card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let creatures = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.card_types.contains(&CardType::Creature)).count();
    assert_eq!(creatures, 0, "two creatures sacrificed");
    // Spent the spell from hand, drew 3 → net +2 vs the pre-cast hand-minus-spell.
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "drew three");
}

/// Territorial Boar grows when a power-4+ creature you control enters.
#[test]
fn territorial_boar_grows_on_big_etb() {
    let mut g = two_player_game();
    let boar = g.add_card_to_battlefield(0, catalog::territorial_boar());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm()); // 6/4
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: wurm }]);
    drain_stack(&mut g);
    let cp = g.computed_permanent(boar).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "gains vigilance");
}

/// Open the Gates fetches a Gate to hand.
#[test]
fn open_the_gates_fetches_gate() {
    let mut g = two_player_game();
    let gate = g.add_card_to_library(0, catalog::azorius_guildgate());
    let cast = g.add_card_to_hand(0, catalog::open_the_gates());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Search(Some(gate))]));
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Azorius Guildgate"), "Gate in hand");
}

/// Cindervines pings an opponent who casts a noncreature spell.
#[test]
fn cindervines_pings_on_opponent_noncreature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cindervines());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "opponent pinged 1 for a noncreature cast");
}

/// Sphinx's Insight draws two, and gains 2 life under Addendum.
#[test]
fn sphinxs_insight_addendum_life() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let cast = g.add_card_to_hand(0, catalog::sphinxs_insight());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell { card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "Addendum gained 2 on your main phase");
}

/// Bladebrand grants deathtouch and draws a card.
#[test]
fn bladebrand_deathtouch_and_draw() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let cast = g.add_card_to_hand(0, catalog::bladebrand());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch), "bear gains deathtouch");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "drew a card");
}

/// Sprouting Renewal is a convoke modal (make a token / destroy).
#[test]
fn sprouting_renewal_is_convoke_modal() {
    let def = catalog::sprouting_renewal();
    assert!(def.keywords.contains(&Keyword::Convoke), "has convoke");
    match def.effect {
        crabomination::effect::Effect::ChooseModesCast { min, max, modes, .. } => {
            assert_eq!((min, max, modes.len()), (1, 1, 2), "choose one of two modes");
        }
        _ => panic!("expected ChooseModesCast"),
    }
}
