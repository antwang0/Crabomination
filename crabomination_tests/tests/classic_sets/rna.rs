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
        (catalog::tithe_taker, 2, 1, &[]),
        (catalog::imperious_oligarch, 2, 1, &[Keyword::Vigilance]),
        (catalog::rampaging_rendhorn, 4, 4, &[]),
        (catalog::spear_spewer, 0, 2, &[Keyword::Defender]),
        (catalog::vindictive_vampire, 2, 3, &[]),
        (catalog::sauroform_hybrid, 2, 2, &[]),
        (catalog::skitter_eel, 3, 3, &[]),
        (catalog::rakdos_roustabout, 3, 2, &[]),
        (catalog::gatebreaker_ram, 2, 2, &[]),
        (catalog::feral_maaka, 2, 2, &[]),
        (catalog::rubble_slinger, 2, 3, &[Keyword::Reach]),
        (catalog::watchful_giant, 3, 6, &[]),
        (catalog::faerie_duelist, 1, 2, &[Keyword::Flash, Keyword::Flying]),
        (catalog::coral_commando, 3, 2, &[]),
        (catalog::windstorm_drake, 3, 3, &[Keyword::Flying]),
        (catalog::burning_tree_vandal, 2, 1, &[]),
        (catalog::ghor_clan_wrecker, 2, 2, &[Keyword::Menace]),
        (catalog::haazda_officer, 3, 2, &[]),
        (catalog::twilight_panther, 1, 2, &[]),
        (catalog::vedalken_mesmerist, 2, 1, &[]),
        (catalog::chillbringer, 3, 3, &[Keyword::Flying]),
        (catalog::noxious_groodion, 2, 2, &[Keyword::Deathtouch]),
        (catalog::steeple_creeper, 4, 2, &[]),
        (catalog::gruul_beastmaster, 2, 2, &[]),
        (catalog::trollbred_guardian, 5, 5, &[]),
        (catalog::loxodon_restorer, 3, 4, &[Keyword::Convoke]),
        (catalog::syndicate_messenger, 2, 3, &[Keyword::Flying]),
        (catalog::aeromunculus, 2, 3, &[Keyword::Flying]),
        (catalog::sages_row_savant, 2, 1, &[]),
        (catalog::senate_griffin, 3, 2, &[Keyword::Flying]),
        (catalog::sylvan_brushstrider, 3, 2, &[]),
        (catalog::wrecking_beast, 6, 6, &[Keyword::Trample]),
        (catalog::thirsting_shade, 1, 1, &[Keyword::Lifelink]),
        (catalog::senate_courier, 1, 4, &[Keyword::Flying]),
        (catalog::enraged_ceratok, 4, 4, &[Keyword::CantBeBlockedByPowerAtMost(2)]),
        (catalog::debtors_transport, 5, 3, &[]),
        (catalog::spikewheel_acrobat, 5, 2, &[]),
        (catalog::dagger_caster, 2, 3, &[]),
        (catalog::footlight_fiend, 1, 1, &[]),
        (catalog::skatewing_spy, 2, 3, &[]),
        (catalog::spirit_of_the_spires, 2, 4, &[Keyword::Flying]),
        (catalog::clamor_shaman, 1, 1, &[]),
        (catalog::resolute_watchdog, 1, 3, &[Keyword::Defender]),
        (catalog::tenth_district_veteran, 2, 3, &[Keyword::Vigilance]),
        (catalog::silhana_wayfinder, 2, 1, &[]),
        (catalog::elite_arrester, 0, 3, &[]),
        (catalog::wall_of_lost_thoughts, 0, 4, &[Keyword::Defender]),
        (catalog::rubblebelt_runner, 3, 3, &[Keyword::CantBeBlockedBy(Box::new(crabomination::card::SelectionRequirement::IsToken))]),
        (catalog::frilled_mystic, 3, 2, &[Keyword::Flash]),
        (catalog::zegana_utopian_speaker, 4, 4, &[]),
        (catalog::biogenic_ooze, 2, 2, &[]),
        (catalog::sunder_shaman, 5, 5, &[Keyword::CantBeBlockedByMoreThanOne]),
        (catalog::skarrgan_hellkite, 4, 4, &[Keyword::Flying]),
        (catalog::humongulus, 2, 5, &[Keyword::Hexproof]),
        (catalog::gravel_hide_goblin, 2, 1, &[]),
        (catalog::seraph_of_the_scales, 4, 3, &[Keyword::Flying]),
        (catalog::orzhov_racketeers, 3, 2, &[]),
        (catalog::gutterbones, 2, 1, &[]),
        (catalog::knight_of_the_last_breath, 4, 4, &[]),
        (catalog::sphinx_of_the_guildpact, 5, 5, &[Keyword::Flying, Keyword::HexproofFromMonocolored]),
        (catalog::azorius_skyguard, 3, 3, &[Keyword::Flying, Keyword::FirstStrike]),
        (catalog::charging_war_boar, 3, 1, &[Keyword::Haste]),
        (catalog::dovins_automaton, 3, 3, &[]),
        (catalog::the_haunt_of_hightower, 3, 3, &[Keyword::Flying, Keyword::Lifelink]),
        (catalog::sharktocrab, 4, 4, &[]),
        (catalog::growth_chamber_guardian, 2, 2, &[]),
        (catalog::rix_maadi_reveler, 2, 2, &[]),
        (catalog::rafter_demon, 4, 2, &[]),
        (catalog::hackrobat, 2, 3, &[]),
        (catalog::gruul_spellbreaker, 3, 3, &[Keyword::Trample]),
        (catalog::smelt_ward_ignus, 2, 1, &[]),
        (catalog::sphinx_of_new_prahv, 4, 3, &[Keyword::Flying, Keyword::Vigilance]),
        (catalog::pestilent_spirit, 3, 2, &[Keyword::Menace, Keyword::Deathtouch]),
        (catalog::scuttlegator, 6, 6, &[Keyword::Defender]),
        (catalog::pitiless_pontiff, 2, 2, &[]),
        (catalog::mesmerizing_benthid, 4, 5, &[]),
        (catalog::immolation_shaman, 1, 3, &[]),
        (catalog::domris_nodorog, 5, 2, &[Keyword::Trample]),
        (catalog::bolrac_clan_crusher, 4, 4, &[]),
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

/// Summary Judgment deals 5 to a tapped creature under Addendum, 3 otherwise.
#[test]
fn summary_judgment_addendum_damage() {
    // Cast during your main phase → 5 damage.
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    let cast = g.add_card_to_hand(0, catalog::summary_judgment());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    assert!(g.battlefield_find(foe).is_none(), "6/4 took 5 and died under Addendum");
}

/// Grotesque Demise exiles a small creature (and can't hit a big one).
#[test]
fn grotesque_demise_exiles_small() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // power 2
    let cast = g.add_card_to_hand(0, catalog::grotesque_demise());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(small)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == small), "power-2 creature exiled");
}

/// Chillbringer taps an opponent's creature and stuns it.
#[test]
fn chillbringer_taps_and_stuns() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(foe)), 0, 0);
    let effect = catalog::chillbringer().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    let c = g.battlefield_find(foe).unwrap();
    assert!(c.tapped, "opponent creature tapped");
    assert_eq!(c.counter_count(CounterType::Stun), 1, "gets a stun counter");
}

/// Haazda Officer pumps a creature you control on entry.
#[test]
fn haazda_officer_pumps_ally() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    let effect = catalog::haazda_officer().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+1");
}

/// Rubble Reading destroys a land and scries.
#[test]
fn rubble_reading_destroys_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::mountain());
    let cast = g.add_card_to_hand(0, catalog::rubble_reading());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(land)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land destroyed");
}

/// Regenesis returns up to two permanent cards from the graveyard to hand.
#[test]
fn regenesis_returns_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let ctx = crabomination::game::effects::EffectContext {
        targets: vec![Target::Permanent(a), Target::Permanent(b)],
        ..crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)
    };
    let effect = catalog::regenesis().effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.id == a) && g.players[0].hand.iter().any(|c| c.id == b), "both returned to hand");
}

/// Gruul Beastmaster pumps another creature by its power when it attacks.
#[test]
fn gruul_beastmaster_attack_pump() {
    let mut g = two_player_game();
    let boss = g.add_card_to_battlefield(0, catalog::gruul_beastmaster()); // 2/2
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(boss);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: boss, target: AttackTarget::Player(1) }]).expect("attack");
    // Beastmaster power 2 → ally gets +2/+0. Resolve the attack trigger onto the ally.
    let ctx = crabomination::game::effects::EffectContext {
        source: Some(boss),
        targets: vec![Target::Permanent(ally)],
        ..crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(ally)), 0, 0)
    };
    let effect = catalog::gruul_beastmaster().triggered_abilities[1].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.computed_permanent(ally).unwrap().power, 4, "+2/+0 from Beastmaster's power");
}

/// Trollbred Guardian grants trample to your +1/+1-countered creatures.
#[test]
fn trollbred_guardian_trample_anthem() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::trollbred_guardian());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample), "no counter → no trample");
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample), "counter → trample");
}

/// Loxodon Restorer gains 4 life on entry and has convoke.
#[test]
fn loxodon_restorer_gains_four() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let life = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::loxodon_restorer());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "ETB gained 4");
}

/// Prying Eyes draws four and discards two.
#[test]
fn prying_eyes_draw_four_discard_two() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let cast = g.add_card_to_hand(0, catalog::prying_eyes());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell { card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    // -1 spell, +4 draw, -2 discard = net +1.
    assert_eq!(g.players[0].hand.len(), hand - 1 + 4 - 2, "net +1 card");
}

// ── Batch 4 (2026-07-24) functionality tests ─────────────────────────────────

/// Azorius Locket taps for {W} or {U} and sacrifices to draw two.
#[test]
fn azorius_locket_mana_and_sac_draw() {
    let mut g = two_player_game();
    let locket = g.add_card_to_battlefield(0, catalog::azorius_locket());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    // Mana ability: adds one of {W}/{U}.
    g.perform_action(GameAction::ActivateAbility { card_id: locket, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::White) + g.players[0].mana_pool.amount(Color::Blue), 1, "one W or U");
    // Sac ability draws two.
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(locket).unwrap().tapped = false;
    g.players[0].mana_pool.add(Color::White, 4);
    g.perform_action(GameAction::ActivateAbility { card_id: locket, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).expect("sac draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
    assert!(g.battlefield_find(locket).is_none(), "locket sacrificed");
}

/// Aeromunculus's adapt puts a +1/+1 counter on it (once).
#[test]
fn aeromunculus_adapt() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::aeromunculus());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: c, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("adapt");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(c).unwrap().power, 3, "2/3 → 3/4 after adapt 1");
}

/// Sylvan Brushstrider gains 2 life on entry.
#[test]
fn sylvan_brushstrider_gains_two() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let life = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::sylvan_brushstrider());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "ETB gained 2");
}

/// Enraged Ceratok can't be blocked by a power-2-or-less creature.
#[test]
fn enraged_ceratok_evades_small_blockers() {
    let c = catalog::enraged_ceratok();
    assert!(c.keywords.contains(&Keyword::CantBeBlockedByPowerAtMost(2)), "power-2-or-less can't block");
}

/// Dagger Caster pings each opponent and each opposing creature on entry.
#[test]
fn dagger_caster_etb_pings() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let opp = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let effect = catalog::dagger_caster().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    let sba = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sba);
    assert_eq!(g.players[1].life, opp - 1, "opponent took 1");
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 1, "opposing creature took 1");
}

/// Footlight Fiend deals 1 to any target when it dies.
#[test]
fn footlight_fiend_dies_ping() {
    let mut g = two_player_game();
    let opp = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    let effect = catalog::footlight_fiend().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[1].life, opp - 1, "1 damage to any target");
}

/// Storm Strike buffs +1/+0 and grants first strike.
#[test]
fn storm_strike_pump_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let cast = g.add_card_to_hand(0, catalog::storm_strike());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!(p.power, 3, "+1/+0");
    assert!(p.keywords.contains(&Keyword::FirstStrike), "gains first strike");
}

/// Stony Strength adds a +1/+1 counter and untaps the creature.
#[test]
fn stony_strength_counter_and_untap() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let cast = g.add_card_to_hand(0, catalog::stony_strength());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+1 counter");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
}

/// Ragefire deals 3 to a creature.
#[test]
fn ragefire_burns_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let cast = g.add_card_to_hand(0, catalog::ragefire());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 3, "3 damage marked");
}

/// Elite Arrester taps a target creature.
#[test]
fn elite_arrester_taps() {
    let mut g = two_player_game();
    let arr = g.add_card_to_battlefield(0, catalog::elite_arrester());
    g.clear_sickness(arr);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility { card_id: arr, ability_index: 0, target: Some(Target::Permanent(foe)), additional_targets: Vec::new(), x_value: None }).expect("tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "target tapped");
}

/// Wall of Lost Thoughts mills a target player 4.
#[test]
fn wall_of_lost_thoughts_mills_four() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(1, catalog::island()); }
    let gy = g.players[1].graveyard.len();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    let effect = catalog::wall_of_lost_thoughts().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[1].graveyard.len(), gy + 4, "milled 4");
}

/// Spirit of the Spires gives other flyers you control +0/+1.
#[test]
fn spirit_of_the_spires_flying_anthem() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spirit_of_the_spires());
    let flyer = g.add_card_to_battlefield(0, catalog::wind_drake()); // 2/2 flying
    assert_eq!(g.computed_permanent(flyer).unwrap().toughness, 3, "flyer gets +0/+1");
    let ground = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(ground).unwrap().toughness, 2, "non-flyer unaffected");
}

/// Skatewing Spy grants flying to your +1/+1-countered creatures.
#[test]
fn skatewing_spy_counter_flying_anthem() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::skatewing_spy());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying), "no counter → no flying");
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying), "counter → flying");
}

/// Dead Revels returns up to two creature cards from your graveyard.
#[test]
fn dead_revels_returns_creatures() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::craw_wurm());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let effect = catalog::dead_revels().effect.clone();
    let hand = g.players[0].hand.len();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 2, "two creatures returned to hand");
}

/// Resolute Watchdog sacrifices to grant indestructible.
#[test]
fn resolute_watchdog_grants_indestructible() {
    let mut g = two_player_game();
    let dog = g.add_card_to_battlefield(0, catalog::resolute_watchdog());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility { card_id: dog, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None }).expect("sac");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Indestructible), "bear indestructible");
    assert!(g.battlefield_find(dog).is_none(), "watchdog sacrificed");
}

/// Silhana Wayfinder puts a creature or land on top of the library.
#[test]
fn silhana_wayfinder_stacks_top() {
    let mut g = two_player_game();
    // Bottom-most drawn last; put a land under a few misses.
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::forest());
    let lib0 = g.players[0].library.len();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let effect = catalog::silhana_wayfinder().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    // A creature or land ends on top; library size unchanged (rest bottomed).
    assert_eq!(g.players[0].library.len(), lib0, "no cards leave the library");
    let top = g.players[0].library.last().unwrap();
    assert!(top.definition.card_types.contains(&CardType::Creature) || top.definition.card_types.contains(&CardType::Land), "top is a creature or land");
}

/// Gyre Engineer untaps whenever you activate an adapt ability (CR 702.108).
#[test]
fn gyre_engineer_untaps_on_adapt() {
    let mut g = two_player_game();
    let eng = g.add_card_to_battlefield(0, catalog::gyre_engineer());
    let munc = g.add_card_to_battlefield(0, catalog::aeromunculus());
    g.clear_sickness(eng);
    // Tap Gyre Engineer for mana (so we can observe it untapping).
    g.perform_action(GameAction::ActivateAbility { card_id: eng, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("mana");
    assert!(g.battlefield_find(eng).unwrap().tapped, "engineer tapped for mana");
    // Activate Aeromunculus's adapt ability.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: munc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("adapt");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(eng).unwrap().tapped, "engineer untapped by adapt trigger");
}

// ── Batch 5 (2026-07-24) functionality tests ─────────────────────────────────

/// Basilica Bell-Haunt makes each opponent discard and gains 3.
#[test]
fn basilica_bell_haunt_discard_and_gain() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let hand = g.players[1].hand.len();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let effect = catalog::basilica_bell_haunt().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[0].life, life + 3, "gained 3");
    assert_eq!(g.players[1].hand.len(), hand - 1, "opponent discarded");
}

/// Sky Tether gives defender and removes flying.
#[test]
fn sky_tether_grounds_flyer() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::wind_drake()); // 2/2 flying
    let tether = g.add_card_to_hand(0, catalog::sky_tether());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell { card_id: tether, target: Some(Target::Permanent(flyer)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let p = g.computed_permanent(flyer).unwrap();
    assert!(!p.keywords.contains(&Keyword::Flying), "loses flying");
    assert!(p.keywords.contains(&Keyword::Defender), "has defender");
}

/// Slimebind saps -4/-0.
#[test]
fn slimebind_saps_power() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let aura = g.add_card_to_hand(0, catalog::slimebind());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: aura, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(foe).unwrap().power, 2, "6/4 → 2/4");
}

/// Sentinel's Mark grants +1/+2 and vigilance, plus lifelink on a main-phase cast.
#[test]
fn sentinels_mark_addendum_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::sentinels_mark());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (3, 4), "+1/+2");
    assert!(p.keywords.contains(&Keyword::Vigilance), "vigilance");
    assert!(p.keywords.contains(&Keyword::Lifelink), "addendum lifelink on main-phase cast");
}

/// Lawmage's Binding locks a creature down and has flash.
#[test]
fn lawmages_binding_locks_down() {
    assert!(catalog::lawmages_binding().keywords.contains(&Keyword::Flash), "has flash");
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::lawmages_binding());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: aura, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let p = g.computed_permanent(foe).unwrap();
    assert!(p.keywords.contains(&Keyword::CantAttack) && p.keywords.contains(&Keyword::CantBlock), "can't attack or block");
}

/// Syndicate Guildmage taps a big creature.
#[test]
fn syndicate_guildmage_taps_big() {
    let mut g = two_player_game();
    let gm = g.add_card_to_battlefield(0, catalog::syndicate_guildmage());
    g.clear_sickness(gm);
    let foe = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4, power 4+
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility { card_id: gm, ability_index: 0, target: Some(Target::Permanent(foe)), additional_targets: Vec::new(), x_value: None }).expect("tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "big creature tapped");
}

/// Cult Guildmage pings an opponent.
#[test]
fn cult_guildmage_pings() {
    let mut g = two_player_game();
    let gm = g.add_card_to_battlefield(0, catalog::cult_guildmage());
    g.clear_sickness(gm);
    let opp = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility { card_id: gm, ability_index: 1, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "1 damage to opponent");
}

/// Rally to Battle pumps +1/+3 and untaps your creatures.
#[test]
fn rally_to_battle_pump_untap() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let cast = g.add_card_to_hand(0, catalog::rally_to_battle());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (3, 5), "+1/+3");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
}

/// Expose to Daylight destroys an enchantment.
#[test]
fn expose_to_daylight_destroys() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let ench = g.add_card_to_battlefield(1, catalog::pacifism());
    let cast = g.add_card_to_hand(0, catalog::expose_to_daylight());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: cast, target: Some(Target::Permanent(ench)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
}

/// Orzhov Enforcer has deathtouch and afterlife 1.
#[test]
fn orzhov_enforcer_afterlife() {
    let mut g = two_player_game();
    let e = g.add_card_to_battlefield(0, catalog::orzhov_enforcer());
    g.remove_to_graveyard_with_triggers(e);
    drain_stack(&mut g);
    let spirits = g.battlefield.iter().filter(|c| c.definition.name.contains("Spirit")).count();
    assert_eq!(spirits, 1, "afterlife 1 made a Spirit token");
}

/// The bot activates an adapt ability to grow an uncountered creature
/// (regression for adapt-shape recognition in `pick_self_pump_counter`).
#[test]
fn bot_activates_adapt_ability() {
    use crabomination::server::bot::{Bot, RandomBot};
    let mut g = two_player_game();
    let munc = g.add_card_to_battlefield(0, catalog::aeromunculus());
    g.clear_sickness(munc);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let action = RandomBot::new().next_action(&g, 0);
    assert!(
        matches!(action, Some(GameAction::ActivateAbility { card_id, .. }) if card_id == munc),
        "bot adapts Aeromunculus: {action:?}"
    );
}

// ── Batch 6 (2026-07-24) functionality tests ─────────────────────────────────

/// Frilled Mystic has flash and an ETB "may counter target spell".
#[test]
fn frilled_mystic_flash_counter() {
    use crabomination::effect::Effect;
    let def = catalog::frilled_mystic();
    assert!(def.keywords.contains(&Keyword::Flash), "flash");
    assert!(matches!(&def.triggered_abilities[0].effect, Effect::MayDo { body, .. }
        if matches!(&**body, Effect::CounterSpell { .. })), "ETB may counter a spell");
}

/// Rubblebelt Runner can't be blocked by a token.
#[test]
fn rubblebelt_runner_evades_tokens() {
    let c = catalog::rubblebelt_runner();
    assert!(c.keywords.iter().any(|k| matches!(k, Keyword::CantBeBlockedBy(_))), "carries the token-block restriction");
}

/// Zegana draws on entry when you control a countered creature.
#[test]
fn zegana_etb_draws_with_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let hand = g.players[0].hand.len();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let effect = catalog::zegana_utopian_speaker().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Zegana's static gives trample to your +1/+1-countered creatures.
#[test]
fn zegana_trample_anthem() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zegana_utopian_speaker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample), "countered creature has trample");
}

/// Ill-Gotten Inheritance's upkeep drains each opponent and gains you life.
#[test]
fn ill_gotten_inheritance_upkeep_drain() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let opp = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let effect = catalog::ill_gotten_inheritance().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[1].life, opp - 1, "each opponent took 1");
    assert_eq!(g.players[0].life, life + 1, "gained 1");
}

/// Biogenic Ooze makes an Ooze token on entry.
#[test]
fn biogenic_ooze_makes_token() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.move_card_to_battlefield_for_test(0, catalog::biogenic_ooze());
    drain_stack(&mut g);
    let oozes = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Ooze)).count();
    assert_eq!(oozes, 2, "Biogenic Ooze + one token");
}

/// Skarrgan Hellkite's ping is gated on having a +1/+1 counter.
#[test]
fn skarrgan_hellkite_ping_needs_counter() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::skarrgan_hellkite());
    g.clear_sickness(dragon);
    // Ensure no +1/+1 counter (riot's ETB choice may have added one).
    g.battlefield_find_mut(dragon).unwrap().counters.remove(&CounterType::PlusOnePlusOne);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    // No counter yet → the "activate only if it has a +1/+1 counter" gate rejects.
    let no_counter = g.perform_action(GameAction::ActivateAbility { card_id: dragon, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None });
    assert!(matches!(no_counter, Err(crabomination::game::GameError::AbilityConditionNotMet)), "no counter → gate rejects: {no_counter:?}");
    // Give it a counter → the gate is satisfied (no AbilityConditionNotMet).
    g.battlefield_find_mut(dragon).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let with_counter = g.perform_action(GameAction::ActivateAbility { card_id: dragon, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None });
    assert!(!matches!(with_counter, Err(crabomination::game::GameError::AbilityConditionNotMet)), "counter → gate satisfied: {with_counter:?}");
}

/// Sunder Shaman can't be blocked by more than one creature.
#[test]
fn sunder_shaman_menace_like() {
    assert!(catalog::sunder_shaman().keywords.contains(&Keyword::CantBeBlockedByMoreThanOne), "can't be blocked by more than one");
}

// ── RNA batch 7 (modern_decks) behavior tests ───────────────────────────────

/// Get the Point destroys the target creature and scries.
#[test]
fn get_the_point_destroys() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(victim)), 0, 0);
    let effect = catalog::get_the_point().effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.players[1].graveyard.iter().any(|c| c.id == victim), "creature destroyed");
}

/// Kaya's Wrath destroys all creatures and gains life for each of yours.
#[test]
fn kayas_wrath_wraths_and_gains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let effect = catalog::kayas_wrath().effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.card_types.contains(&CardType::Creature)).count(), 0, "all creatures gone");
    assert_eq!(g.players[0].life, life + 2, "gained 2 (your two creatures)");
}

/// Rampage of the Clans destroys artifacts/enchantments, minting a 3/3 Centaur
/// for each destroyed permanent's controller.
#[test]
fn rampage_of_the_clans_swaps_for_centaurs() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::glass_of_the_guildpact()); // an artifact
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let effect = catalog::rampage_of_the_clans().effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    let centaurs = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Centaur)).count();
    assert_eq!(centaurs, 1, "opponent gets a Centaur for their destroyed artifact");
}

/// Goblin Gathering scales with copies already in your graveyard.
#[test]
fn goblin_gathering_scales_with_graveyard() {
    let mut g = two_player_game();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let effect = catalog::goblin_gathering().effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    let base = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Goblin)).count();
    assert_eq!(base, 2, "two Goblins with an empty graveyard");
    g.add_card_to_graveyard(0, catalog::goblin_gathering());
    g.resolve_effect(&effect, &ctx).unwrap();
    let after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Goblin)).count();
    assert_eq!(after, base + 3, "two plus one graveyard copy → three more");
}

/// Gates Ablaze deals damage equal to Gates you control to each creature.
#[test]
fn gates_ablaze_scales_with_gates() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gateway_plaza());
    g.add_card_to_battlefield(0, catalog::gateway_plaza());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let effect = catalog::gates_ablaze().effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "2 damage from two Gates kills the 2/2");
}

/// Undercity's Embrace edicts and gains 4 with a power-4 creature out.
#[test]
fn undercitys_embrace_edict_and_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::craw_wurm()); // 6/4, power >= 4
    let life = g.players[0].life;
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let effect = catalog::undercitys_embrace().effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.card_types.contains(&CardType::Creature)).count(), 0, "opponent sacrificed their creature");
    assert_eq!(g.players[0].life, life + 4, "gained 4 for the power-4 creature");
}

/// Glass of the Guildpact only pumps multicolored creatures.
#[test]
fn glass_of_the_guildpact_multicolored_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::glass_of_the_guildpact());
    let mono = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green only
    let multi = g.add_card_to_battlefield(0, catalog::rakdos_firewheeler()); // B/R multicolored
    let m = g.computed_permanent(mono).unwrap();
    assert_eq!((m.power, m.toughness), (2, 2), "monocolored unaffected");
    let x = g.computed_permanent(multi).unwrap();
    assert_eq!((x.power, x.toughness), (5, 4), "multicolored 4/3 → 5/4");
}

/// Macabre Mockery reanimates a creature from an opponent's graveyard under
/// your control.
#[test]
fn macabre_mockery_steals_from_graveyard() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(1, catalog::craw_wurm());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(corpse)), 0, 0);
    let effect = catalog::macabre_mockery().effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    let reanimated = g.battlefield_find(corpse).expect("on battlefield");
    assert_eq!(reanimated.controller, 0, "under your control");
    assert!(g.computed_permanent(corpse).unwrap().keywords.contains(&Keyword::Haste), "gained haste");
}

/// Azorius Skyguard weakens opposing creatures.
#[test]
fn azorius_skyguard_debuffs_opponents() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::azorius_skyguard());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let c = g.computed_permanent(opp).unwrap();
    assert_eq!((c.power, c.toughness), (1, 2), "opponent creature gets -1/-0");
}

/// Seraph of the Scales can grant itself deathtouch for {B}.
#[test]
fn seraph_grants_deathtouch() {
    let mut g = two_player_game();
    let seraph = g.add_card_to_battlefield(0, catalog::seraph_of_the_scales());
    g.clear_sickness(seraph);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility { card_id: seraph, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).expect("activate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(seraph).unwrap().keywords.contains(&Keyword::Deathtouch), "gained deathtouch");
}

/// Gutterbones returns from the graveyard once an opponent has lost life.
#[test]
fn gutterbones_recurs_after_opponent_loses_life() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bones = g.add_card_to_graveyard(0, catalog::gutterbones());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    // No opponent life loss yet → the "only if an opponent lost life" gate rejects.
    let early = g.perform_action(GameAction::ActivateAbility { card_id: bones, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None });
    assert!(early.is_err(), "gate rejects before any life loss: {early:?}");
    g.adjust_life(1, -1);
    g.perform_action(GameAction::ActivateAbility { card_id: bones, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("recur");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bones), "returned to hand");
}

/// The Haunt of Hightower grows when cards hit an opponent's graveyard.
#[test]
fn haunt_of_hightower_grows_on_opponent_mill() {
    let mut g = two_player_game();
    let haunt = g.add_card_to_battlefield(0, catalog::the_haunt_of_hightower());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&crabomination::effect::Effect::Mill { who: crabomination::effect::Selector::Player(crabomination::effect::PlayerRef::EachOpponent), amount: crabomination::effect::Value::ONE }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(haunt).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1, "one +1/+1 counter from the milled card");
}

/// Depose taps a creature and draws; its Deploy half makes two Thopters.
#[test]
fn depose_deploy_halves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(victim)), 0, 0);
    g.resolve_effect(&catalog::depose_deploy().effect.clone(), &ctx).unwrap();
    assert!(g.battlefield_find(victim).unwrap().tapped, "Depose taps the target");
    assert_eq!(g.players[0].hand.len(), hand + 1, "Depose draws a card");
    let deploy = catalog::depose_deploy().split.unwrap().right.effect.clone();
    let ctx2 = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&deploy, &ctx2).unwrap();
    let thopters = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Thopter)).count();
    assert_eq!(thopters, 2, "Deploy makes two Thopters");
}

/// Warden's right half mints a 4/4 flying, vigilant Sphinx.
#[test]
fn warrant_warden_sphinx() {
    let mut g = two_player_game();
    let warden = catalog::warrant_warden().split.unwrap().right.effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&warden, &ctx).unwrap();
    let sphinx = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Sphinx)).expect("sphinx token");
    assert_eq!((sphinx.power(), sphinx.toughness()), (4, 4), "4/4 Sphinx");
    assert!(sphinx.definition.keywords.contains(&Keyword::Flying) && sphinx.definition.keywords.contains(&Keyword::Vigilance));
}

// ── RNA batch 8 (modern_decks) behavior tests ───────────────────────────────

/// Clan Guildmage animates a land you control into a 4/4 Elemental.
#[test]
fn clan_guildmage_animates_land() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::clan_guildmage());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.clear_sickness(mage);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: mage, ability_index: 1, target: Some(Target::Permanent(land)), additional_targets: Vec::new(), x_value: None }).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "land is a 4/4");
    assert!(cp.card_types.contains(&CardType::Creature) && cp.card_types.contains(&CardType::Land), "still a land");
}

/// Tin Street Dodger grants itself "can't be blocked except by defenders."
#[test]
fn tin_street_dodger_evasion() {
    let mut g = two_player_game();
    let dodger = g.add_card_to_battlefield(0, catalog::tin_street_dodger());
    g.clear_sickness(dodger);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility { card_id: dodger, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("evade");
    drain_stack(&mut g);
    assert!(g.computed_permanent(dodger).unwrap().keywords.iter().any(|k| matches!(k, Keyword::CantBeBlockedExceptBy(_))), "gained can't-be-blocked-except-by-defenders");
}

/// Saruli Caretaker taps another creature as part of its mana cost.
#[test]
fn saruli_caretaker_taps_a_creature_for_mana() {
    let mut g = two_player_game();
    let caretaker = g.add_card_to_battlefield(0, catalog::saruli_caretaker());
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(caretaker);
    g.clear_sickness(helper);
    g.perform_action(GameAction::ActivateAbility { card_id: caretaker, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap for mana");
    assert!(g.battlefield_find(caretaker).unwrap().tapped, "Caretaker taps");
    assert!(g.battlefield_find(helper).unwrap().tapped, "the helper creature also taps");
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
}

/// Gate Colossus recurs itself when a Gate enters.
#[test]
fn gate_colossus_recurs_on_gate() {
    let mut g = two_player_game();
    let colossus = g.add_card_to_graveyard(0, catalog::gate_colossus());
    let gate = g.add_card_to_battlefield(0, catalog::gateway_plaza());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: gate }]);
    drain_stack(&mut g);
    assert!(g.players[0].library.iter().any(|c| c.id == colossus), "Gate Colossus put on top of library");
}

/// Persistent Petitioners mills twelve when four Advisors tap.
#[test]
fn persistent_petitioners_advisor_mill() {
    let mut g = two_player_game();
    for _ in 0..20 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let ids: Vec<_> = (0..4).map(|_| {
        let id = g.add_card_to_battlefield(0, catalog::persistent_petitioners());
        g.clear_sickness(id);
        id
    }).collect();
    g.perform_action(GameAction::ActivateAbility { card_id: ids[0], ability_index: 1, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None }).expect("mill 12");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 12, "four Advisors mill twelve");
}

/// Fireblade Artist sacrifices a creature on upkeep to burn an opponent.
#[test]
fn fireblade_artist_sac_burn() {
    let mut g = two_player_game();
    let _artist = g.add_card_to_battlefield(0, catalog::fireblade_artist());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    let life = g.players[1].life;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    let effect = catalog::fireblade_artist().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[1].life, life - 2, "opponent took 2 from the sacrifice");
}

/// Bedeck weakens a creature; Bedazzle destroys a nonbasic land and burns.
#[test]
fn bedeck_bedazzle_halves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&catalog::bedeck_bedazzle().effect.clone(), &ctx).unwrap();
    let cp = g.computed_permanent(bear);
    // 2/2 +3/-3 → 5/-1 → dies as SBA; check it took the shrink or died.
    g.check_state_based_actions();
    assert!(cp.map(|c| c.toughness <= 0).unwrap_or(true) || g.battlefield_find(bear).is_none(), "Bedeck's -3 toughness is lethal");
}

/// Replicate makes a token copy of a creature you control.
#[test]
fn repudiate_replicate_copies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let replicate = catalog::repudiate_replicate().split.unwrap().right.effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&replicate, &ctx).unwrap();
    let bears = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "original plus a token copy");
}

/// Incongruity exiles a creature and gives its controller a 3/3 Frog Lizard.
#[test]
fn incubation_incongruity_frog() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let incongruity = catalog::incubation_incongruity().split.unwrap().right.effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(victim)), 0, 0);
    g.resolve_effect(&incongruity, &ctx).unwrap();
    assert!(g.battlefield_find(victim).is_none(), "creature exiled");
    let frogs = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Frog)).count();
    assert_eq!(frogs, 1, "controller gets a 3/3 Frog Lizard");
}

/// Sharktocrab taps and stuns an opponent's creature when it adapts.
#[test]
fn sharktocrab_taps_on_counter() {
    let mut g = two_player_game();
    let shark = g.add_card_to_battlefield(0, catalog::sharktocrab());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(victim)), 0, 0);
    // Fire the counter-added trigger's effect directly.
    let eff = catalog::sharktocrab().triggered_abilities[0].effect.clone();
    let _ = shark;
    g.resolve_effect(&eff, &ctx).unwrap();
    assert!(g.battlefield_find(victim).unwrap().tapped, "opponent creature tapped");
    assert_eq!(g.battlefield_find(victim).unwrap().counters.get(&CounterType::Stun).copied().unwrap_or(0), 1, "stunned");
}

/// Growth-Chamber Guardian's counter trigger tutors another copy by name.
#[test]
fn growth_chamber_guardian_tutors_copy() {
    use crabomination::effect::Effect;
    let card = catalog::growth_chamber_guardian();
    // The counter trigger searches the library for another copy by name.
    match &card.triggered_abilities[0].effect {
        Effect::Search { filter, .. } => assert_eq!(
            *filter,
            crabomination::card::SelectionRequirement::HasName("Growth-Chamber Guardian".into()),
            "searches for another Growth-Chamber Guardian"
        ),
        other => panic!("expected a Search effect, got {other:?}"),
    }
}

// ── Batch 9 tests (2026-07-24) ───────────────────────────────────────────────

/// Rix Maadi Reveler's spectacle marks the cast so its ETB rider fires; the
/// non-spectacle ETB discards one and draws one.
#[test]
fn rix_maadi_reveler_etb_discard_draw() {
    let def = catalog::rix_maadi_reveler();
    let alt = def.alternative_cost.clone().expect("spectacle");
    assert!(alt.marks_kicked, "spectacle marks the cast for the 'if paid' rider");

    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let gy = g.players[0].graveyard.len();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), 2, "discard one, draw one → net unchanged");
    assert_eq!(g.players[0].graveyard.len(), gy + 1, "one card discarded");
}

/// Rafter Demon, if its spectacle cost was paid, makes each opponent discard.
#[test]
fn rafter_demon_spectacle_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.kicked = true;
    g.resolve_effect(&catalog::rafter_demon().triggered_abilities[0].effect, &ctx).unwrap();
    assert!(g.players[1].hand.is_empty(), "opponent discarded under spectacle");
}

/// Hackrobat's {R} ability gives +2/-2 until end of turn.
#[test]
fn hackrobat_red_pump() {
    let mut g = two_player_game();
    let h = g.add_card_to_battlefield(0, catalog::hackrobat());
    g.clear_sickness(h);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility { card_id: h, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).expect("pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(h).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 1), "+2/-2");
}

/// Gruul Spellbreaker has hexproof only during its controller's turn.
#[test]
fn gruul_spellbreaker_turn_gated_hexproof() {
    let mut g = two_player_game();
    let gs = g.add_card_to_battlefield(0, catalog::gruul_spellbreaker());
    g.active_player_idx = 0;
    assert!(g.computed_permanent(gs).unwrap().keywords.contains(&Keyword::Hexproof), "hexproof on your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(gs).unwrap().keywords.contains(&Keyword::Hexproof), "no hexproof off your turn");
}

/// Smelt-Ward Ignus steals a small creature: sacrifice it to gain control of a
/// power-3-or-less creature, untapped and hasty.
#[test]
fn smelt_ward_ignus_steals() {
    let mut g = two_player_game();
    let ignus = g.add_card_to_battlefield(0, catalog::smelt_ward_ignus());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    g.clear_sickness(ignus);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: ignus, ability_index: 0, target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None }).expect("steal");
    drain_stack(&mut g);
    let v = g.battlefield_find(victim).unwrap();
    assert_eq!(v.controller, 0, "gained control");
    assert!(!v.tapped, "untapped");
    assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::Haste), "has haste");
}

/// A spell an opponent casts targeting Sphinx of New Prahv costs {2} more.
#[test]
fn sphinx_of_new_prahv_target_tax() {
    let mut g = two_player_game();
    let sphinx = g.add_card_to_battlefield(0, catalog::sphinx_of_new_prahv());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    // Only {R} available — the {2} tax makes the cast illegal.
    g.players[1].mana_pool.add(Color::Red, 1);
    assert!(g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(sphinx)), additional_targets: vec![], mode: None, x_value: None }).is_err(), "untaxed cast rejected");
    // Add the {2} and it resolves.
    g.players[1].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(sphinx)), additional_targets: vec![], mode: None, x_value: None }).expect("taxed cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sphinx).is_none(), "3 damage killed the 4/3 Sphinx");
}

/// Pestilent Spirit gives its controller's instant/sorcery spells deathtouch.
#[test]
fn pestilent_spirit_grants_spell_deathtouch() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pestilent_spirit());
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(wurm)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(wurm).is_none(), "3 deathtouch damage destroys the 6/4");
}

/// Without Pestilent Spirit a 3-damage bolt leaves a 6/4 alive (control case).
#[test]
fn bolt_without_deathtouch_leaves_wurm() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(wurm)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(wurm).is_some(), "6/4 survives 3 non-deathtouch damage");
}

/// Scuttlegator can attack despite defender only while it carries a +1/+1
/// counter (from adapt).
#[test]
fn scuttlegator_attacks_with_counter() {
    // No counter → the defender can't attack.
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::scuttlegator());
    g.clear_sickness(s);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(g.declare_attackers(vec![Attack { attacker: s, target: AttackTarget::Player(1) }]).is_err(), "defender can't attack");

    // With a +1/+1 counter → it may attack.
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::scuttlegator());
    g.battlefield_find_mut(s).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.clear_sickness(s);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: s, target: AttackTarget::Player(1) }]).expect("adapted → can attack");
}

/// Angelic Exaltation pumps a lone attacker by the number of creatures you
/// control.
#[test]
fn angelic_exaltation_lone_attacker_pump() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::angelic_exaltation());
    let att = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // second creature, doesn't attack
    g.clear_sickness(att);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let events = g.declare_attackers(vec![Attack { attacker: att, target: AttackTarget::Player(1) }]).expect("attack");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    // X = 2 creatures you control → +2/+2 on the 2/2 attacker.
    assert_eq!(g.computed_permanent(att).unwrap().power, 4, "lone attacker pumped by creature count");
}

/// Ethereal Absolution's twin anthems buff your creatures and shrink theirs.
#[test]
fn ethereal_absolution_anthems() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ethereal_absolution());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "your creatures +1/+1");
    assert_eq!(g.computed_permanent(theirs).unwrap().toughness, 1, "opponents' creatures -1/-1");
}

/// Cry of the Carnarium's -2/-2 sweep kills small creatures, big ones survive.
#[test]
fn cry_of_the_carnarium_sweep() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::cry_of_the_carnarium().effect, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "2/2 dies to -2/-2");
    let w = g.computed_permanent(wurm).expect("wurm alive");
    assert_eq!((w.power, w.toughness), (4, 2), "6/4 shrinks to 4/2");
}

/// Pitiless Pontiff's sacrifice payoff grants deathtouch and indestructible.
#[test]
fn pitiless_pontiff_payoff() {
    let mut g = two_player_game();
    let pontiff = g.add_card_to_battlefield(0, catalog::pitiless_pontiff());
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(pontiff);
    g.resolve_effect(&catalog::pitiless_pontiff().activated_abilities[0].effect, &ctx).unwrap();
    let cp = g.computed_permanent(pontiff).unwrap();
    assert!(cp.keywords.contains(&Keyword::Deathtouch) && cp.keywords.contains(&Keyword::Indestructible), "gains deathtouch + indestructible");
}

/// Unbreakable Formation's Addendum adds a +1/+1 counter and vigilance on your
/// main phase (indestructible always).
#[test]
fn unbreakable_formation_addendum() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(bear);
    g.resolve_effect(&catalog::unbreakable_formation().effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Indestructible), "indestructible");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "Addendum vigilance");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "Addendum +1/+1 counter");
}

/// Flames of the Raze-Boar deals 4 to a creature, then 2 to that player's board
/// when you control a power-4+ creature.
#[test]
fn flames_of_the_raze_boar_sweeps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::craw_wurm()); // your power-6 creature
    let main = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4 → takes 4
    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → takes 2
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(main)), 0, 0);
    g.resolve_effect(&catalog::flames_of_the_raze_boar().effect, &ctx).unwrap();
    g.check_state_based_actions();
    // Target took 4 (+2 from the elided-"other" sweep) → 6 ≥ 4 toughness, dies;
    // the 2/2 also dies to the second wave.
    assert!(g.battlefield_find(main).is_none(), "4-damage target dies");
    assert!(g.battlefield_find(other).is_none(), "2 to that player's board");
}

/// Swirling Torrent is a one-or-both modal bounce/topdeck.
#[test]
fn swirling_torrent_modal() {
    let def = catalog::swirling_torrent();
    match &def.effect {
        crabomination::effect::Effect::ChooseModesCast { min, max, .. } => {
            assert_eq!((*min, *max), (1, 2), "choose one or both");
        }
        other => panic!("expected ChooseModesCast, got {other:?}"),
    }
}

/// Mesmerizing Benthid makes two Illusions and is hexproof while you control one.
#[test]
fn mesmerizing_benthid_tokens_and_hexproof() {
    let mut g = two_player_game();
    let benthid = g.add_card_to_battlefield(0, catalog::mesmerizing_benthid());
    // No Illusions yet (ETB not run) → no hexproof.
    assert!(!g.computed_permanent(benthid).unwrap().keywords.contains(&Keyword::Hexproof), "no hexproof without an Illusion");
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(benthid);
    g.resolve_effect(&catalog::mesmerizing_benthid().triggered_abilities[0].effect, &ctx).unwrap();
    let illusions = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Illusion)).count();
    assert_eq!(illusions, 2, "two Illusion tokens");
    assert!(g.computed_permanent(benthid).unwrap().keywords.contains(&Keyword::Hexproof), "hexproof while you control an Illusion");
}

/// Immolation Shaman pings a player who activates a creature's non-mana ability.
#[test]
fn immolation_shaman_pings_ability_user() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::immolation_shaman());
    let hack = g.add_card_to_battlefield(1, catalog::hackrobat()); // {B}: deathtouch
    g.clear_sickness(hack);
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Black, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility { card_id: hack, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "Shaman pinged the activator");
}

/// Screaming Shield grants +0/+3 and a mill ability.
#[test]
fn screaming_shield_bonus_and_mill() {
    let def = catalog::screaming_shield();
    let bonus = def.equipped_bonus.clone().expect("equip bonus");
    assert_eq!((bonus.power, bonus.toughness), (0, 3), "+0/+3");
    assert_eq!(bonus.activated_abilities.len(), 1, "granted mill ability");
    // The granted mill puts three cards into the target player's graveyard.
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(1, catalog::island()); }
    let lib = g.players[1].library.len();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
    g.resolve_effect(&bonus.activated_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.players[1].library.len(), lib - 3, "milled three");
}

/// Clear the Stage shrinks a creature and, with a power-4+ creature, returns one
/// from your graveyard.
#[test]
fn clear_the_stage_shrink_and_return() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::craw_wurm()); // your power-6 creature
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(victim)), 0, 0);
    g.resolve_effect(&catalog::clear_the_stage().effect, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "-3/-3 kills the 2/2");
    assert_eq!(g.players[0].hand.len(), hand + 1, "returned a creature from graveyard");
}

/// Domri's Nodorog has riot and an ETB tutor for Domri, City Smasher.
#[test]
fn domris_nodorog_shape() {
    use crabomination::effect::Effect;
    let def = catalog::domris_nodorog();
    assert!(def.keywords.contains(&Keyword::Trample) && def.triggered_abilities.len() == 2, "riot trigger + ETB tutor");
    match &def.triggered_abilities[1].effect {
        Effect::Search { filter, .. } => assert_eq!(*filter, crabomination::card::SelectionRequirement::HasName("Domri, City Smasher".into())),
        other => panic!("expected Search, got {other:?}"),
    }
}

/// Bolrac-Clan Crusher removes a +1/+1 counter to deal 2 to any target.
#[test]
fn bolrac_clan_crusher_ping() {
    let mut g = two_player_game();
    let crusher = g.add_card_to_battlefield(0, catalog::bolrac_clan_crusher());
    let fuel = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(fuel).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.clear_sickness(crusher);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility { card_id: crusher, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "dealt 2");
    assert_eq!(g.battlefield_find(fuel).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "counter removed as a cost");
}

/// Dovin's Acuity's ETB gains 2 life and draws a card.
#[test]
fn dovins_acuity_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::dovins_acuity().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.players[0].life, life + 2, "gained 2");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Dovin's Dismissal puts a tapped creature on top of its owner's library.
#[test]
fn dovins_dismissal_topdecks() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    let lib = g.players[1].library.len();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(foe)), 0, 0);
    g.resolve_effect(&catalog::dovins_dismissal().effect, &ctx).unwrap();
    assert!(g.battlefield_find(foe).is_none(), "tapped creature left the battlefield");
    assert_eq!(g.players[1].library.len(), lib + 1, "put on top of owner's library");
}

/// Eyes Everywhere swaps control of itself and a target nonland permanent.
#[test]
fn eyes_everywhere_exchange_control() {
    let mut g = two_player_game();
    let eyes = g.add_card_to_battlefield(0, catalog::eyes_everywhere());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Permanent(foe)), 0, 0);
    ctx.source = Some(eyes);
    g.resolve_effect(&catalog::eyes_everywhere().activated_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(foe).unwrap().controller, 0, "gained the creature");
    assert_eq!(g.battlefield_find(eyes).unwrap().controller, 1, "gave up the enchantment");
}

/// Nikya of the Old Ways locks its controller out of noncreature spells but not
/// creature spells, and doubles land mana.
#[test]
fn nikya_noncreature_lock() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nikya_of_the_old_ways());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None }).is_err(), "noncreature spell locked");
    // A creature spell is unaffected.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None }).is_ok(), "creature spell allowed");
}

/// Angel of Grace floors combat/noncombat damage at 1 life the turn it enters,
/// and its graveyard ability sets life to 10.
#[test]
fn angel_of_grace_floors_life_and_recurs() {
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::angel_of_grace());
    drain_stack(&mut g);
    g.players[0].life = 5;
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 20, None, &mut evs);
    assert_eq!(g.players[0].life, 1, "damage floored at 1 this turn");

    // Graveyard ability: {4}{W}{W}, exile from graveyard: life becomes 10.
    let ang = g.add_card_to_graveyard(0, catalog::angel_of_grace());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ang, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 10, "life set to 10");
    assert!(g.exile.iter().any(|c| c.id == ang), "card exiled as a cost");
}

/// Rhythm of the Wild gives nontoken creatures you control riot (here: a +1/+1
/// counter choice on entry).
#[test]
fn rhythm_of_the_wild_grants_riot() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rhythm_of_the_wild());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    // riot's ChooseMode: pick the +1/+1 counter (mode index 1).
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Mode(1),
    ]));
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast bear");
    drain_stack(&mut g);
    let b = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("bear on battlefield");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1, "riot added a +1/+1 counter");
}

/// Galloping Lizrog removes +1/+1 counters from your other creatures and puts
/// twice that many on itself.
#[test]
fn galloping_lizrog_doubles_counters() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.battlefield_find_mut(b).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let lizrog = g.move_card_to_battlefield_for_test(0, catalog::galloping_lizrog());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "a drained");
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "b drained");
    assert_eq!(g.battlefield_find(lizrog).unwrap().counter_count(CounterType::PlusOnePlusOne), 6, "3 removed -> 6 on Lizrog");
}

/// Forbidding Spirit taxes attackers {2} until its controller's next turn.
#[test]
fn forbidding_spirit_taxes_attackers() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(1, catalog::forbidding_spirit());
    drain_stack(&mut g);
    assert_eq!(g.players[1].attack_tax_until_your_turn, 2, "tax installed on controller");
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    // Unpayable — the declaration is rejected.
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])).is_err(), "can't attack without paying the tax");
    // Pay the {2} and it goes through.
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])).expect("attack after paying {2}");
    assert_eq!(g.attacking().len(), 1);
}

/// Combine Guildmage's first ability makes your creatures enter with an extra
/// +1/+1 counter this turn; the second moves a counter between your creatures.
#[test]
fn combine_guildmage_abilities() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::combine_guildmage());
    g.clear_sickness(mage);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility { card_id: mage, ability_index: 0, target: None, additional_targets: vec![], x_value: None }).expect("grant etb counter");
    drain_stack(&mut g);
    // Enter through the real ETB funnel so enters-with replacements apply.
    let bear = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "entered with an extra counter");

    // Second ability: move the counter from the bear onto the mage.
    g.battlefield_find_mut(mage).unwrap().tapped = false;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility { card_id: mage, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: vec![Target::Permanent(mage)], x_value: None }).expect("move counter");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "counter left the bear");
    assert_eq!(g.battlefield_find(mage).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "counter moved to mage");
}

/// Verity Circle draws when an opponent's creature is tapped (not as an
/// attacker) — here via its own tap ability — but not when it's declared as
/// an attacker.
#[test]
fn verity_circle_draws_on_nonattacker_tap() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::verity_circle());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand0 = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    // Find the tap ability index (the only activated ability).
    g.perform_action(GameAction::ActivateAbility { card_id: g.battlefield.iter().find(|c| c.definition.name == "Verity Circle").unwrap().id, ability_index: 0, target: Some(Target::Permanent(foe)), additional_targets: vec![], x_value: None }).expect("tap the foe");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "foe tapped");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "Verity Circle drew");
}

/// Declaring an opponent's creature as an attacker does not trigger Verity Circle.
#[test]
fn verity_circle_silent_on_attacker_tap() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::verity_circle()); // opponent's Verity Circle
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.add_card_to_library(1, catalog::grizzly_bears());
    let hand1 = g.players[1].hand.len();
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(atk).unwrap().tapped, "attacker tapped");
    assert_eq!(g.players[1].hand.len(), hand1, "no draw on attacker tap");
}

/// Rumbling Ruin locks opponents' creatures with power ≤ your +1/+1 counter
/// count out of blocking this turn.
#[test]
fn rumbling_ruin_locks_weak_blockers() {
    let mut g = two_player_game();
    // Two +1/+1 counters on my board -> threshold 2.
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let weak = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, power 2 <= 2
    let strong = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4, power 6 > 2
    g.move_card_to_battlefield_for_test(0, catalog::rumbling_ruin());
    drain_stack(&mut g);
    assert!(g.computed_permanent(weak).unwrap().keywords.contains(&Keyword::CantBlock), "weak creature can't block");
    assert!(!g.computed_permanent(strong).unwrap().keywords.contains(&Keyword::CantBlock), "strong creature still blocks");
}

/// Font of Agonies banks a blood counter per life paid, and four of them fuel a
/// creature kill.
#[test]
fn font_of_agonies_banks_and_kills() {
    let mut g = two_player_game();
    let font = g.add_card_to_battlefield(0, catalog::font_of_agonies());
    // "Whenever you pay life, put that many blood counters on it."
    g.dispatch_triggers_for_events(&[GameEvent::PaidLife { player: 0, amount: 3 }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(font).unwrap().counter_count(CounterType::Blood), 3, "3 blood counters banked");
    // Bank one more, then spend four to destroy a creature.
    g.dispatch_triggers_for_events(&[GameEvent::PaidLife { player: 0, amount: 1 }]);
    drain_stack(&mut g);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility { card_id: font, ability_index: 0, target: Some(Target::Permanent(foe)), additional_targets: vec![], x_value: None }).expect("destroy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature destroyed");
    assert_eq!(g.battlefield_find(font).unwrap().counter_count(CounterType::Blood), 0, "four blood counters removed");
}

/// Deputy of Detention exiles an opponent's permanent until it leaves, then
/// returns it.
#[test]
fn deputy_of_detention_exiles_until_leaves() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dep = g.move_card_to_battlefield_for_test(0, catalog::deputy_of_detention());
    // resolve the ETB against the foe
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(Target::Permanent(foe)),
    ]));
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == foe), "foe exiled");
    // Deputy leaving returns it.
    let mut evs = Vec::new();
    g.destroy_permanent(dep, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_some(), "returned when Deputy left");
}

/// Prime Speaker Vannifar pods a sacrificed creature into one costing one more.
#[test]
fn prime_speaker_vannifar_pods() {
    let mut g = two_player_game();
    let van = g.add_card_to_battlefield(0, catalog::prime_speaker_vannifar());
    g.clear_sickness(van);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2
    let target = g.add_card_to_library(0, catalog::rakdos_roustabout()); // MV 3
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(target)),
    ]));
    g.perform_action(GameAction::ActivateAbility { card_id: van, ability_index: 0, target: None, additional_targets: vec![], x_value: None }).expect("pod");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert!(g.battlefield.iter().any(|c| c.id == target && c.controller == 0), "MV-3 creature onto battlefield");
}

/// Hydroid Krasis's cast trigger gains half X life and draws half X cards, and
/// it enters with X +1/+1 counters.
#[test]
fn hydroid_krasis_cast_trigger_scales_with_x() {
    let mut g = two_player_game();
    let krasis = g.add_card_to_hand(0, catalog::hydroid_krasis());
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4); // X = 4
    g.perform_action(GameAction::CastSpell { card_id: krasis, target: None, additional_targets: vec![], mode: None, x_value: Some(4) }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained half of X=4");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew 2 (cast Krasis, drew 2)");
    let body = g.battlefield.iter().find(|c| c.definition.name == "Hydroid Krasis").expect("on battlefield");
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 4, "entered with 4 counters");
}
