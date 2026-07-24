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
    g.decider = Box::new(crabomination::decision::AutoDecider::default());
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
