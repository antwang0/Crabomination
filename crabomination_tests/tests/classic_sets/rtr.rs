//! Functionality tests for Return to Ravnica (RTR) gap cards.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::*;

/// Static stat/keyword lines for the simple RTR creatures.
#[test]
fn rtr_stat_and_keyword_lines() {
    let peg = catalog::concordia_pegasus();
    assert_eq!((peg.power, peg.toughness), (1, 3));
    assert!(peg.keywords.contains(&Keyword::Flying));

    let imp = catalog::daggerdrome_imp();
    assert!(imp.keywords.contains(&Keyword::Flying) && imp.keywords.contains(&Keyword::Lifelink));

    let slug = catalog::catacomb_slug();
    assert_eq!((slug.power, slug.toughness), (2, 6));

    let brush = catalog::brushstrider();
    assert!(brush.keywords.contains(&Keyword::Vigilance));
}

/// Bellows Lizard firebreathes for +1/+0.
#[test]
fn bellows_lizard_firebreathes() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let lizard = g.add_card_to_battlefield(0, catalog::bellows_lizard());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lizard, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("firebreathe");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(lizard).unwrap().power, 2, "pumped to 2/1");
}

/// Centaur Healer gains 3 life on entry.
#[test]
fn centaur_healer_gains_life() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let healer = g.add_card_to_hand(0, catalog::centaur_healer());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: healer, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gained 3 life");
}

/// Crosstown Courier mills the player it damages by that much.
#[test]
fn crosstown_courier_mills_on_hit() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let courier = g.add_card_to_battlefield(0, catalog::crosstown_courier()); // 2/1
    g.clear_sickness(courier);
    for _ in 0..5 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let lib1 = g.players[1].library.len();
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: courier, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.resolve_combat().expect("combat");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib1 - 2, "milled 2 (combat damage)");
}

/// Centaur's Herald sacrifices itself to make a 3/3.
#[test]
fn centaurs_herald_makes_centaur() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let herald = g.add_card_to_battlefield(0, catalog::centaurs_herald());
    g.clear_sickness(herald);
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: herald, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for token");
    drain_stack(&mut g);
    assert!(g.battlefield_find(herald).is_none(), "Herald sacrificed");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Centaur" && c.definition.power == 3),
        "made a 3/3 Centaur token",
    );
}

/// Doorkeeper mills by the number of defenders you control.
#[test]
fn doorkeeper_mills_by_defenders() {
    use crabomination::game::types::Target;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let door = g.add_card_to_battlefield(0, catalog::doorkeeper());
    g.clear_sickness(door);
    g.add_card_to_battlefield(0, catalog::doorkeeper()); // a second defender
    for _ in 0..5 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let lib1 = g.players[1].library.len();
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: door, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("mill");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib1 - 2, "milled 2 (two defenders)");
}

/// Dead Reveler may enter unleashed with a +1/+1 counter.
#[test]
fn dead_reveler_unleash_counter() {
    use crabomination::card::CounterType;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::mana::Color;
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let rev = g.add_card_to_hand(0, catalog::dead_reveler());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rev, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cid = g.battlefield.iter().find(|c| c.definition.name == "Dead Reveler").unwrap().id;
    assert_eq!(g.battlefield_find(cid).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "entered unleashed with a +1/+1 counter");
}

/// Stat/keyword lines for the vanilla / french-vanilla RTR batch-2 creatures.
#[test]
fn rtr_batch2_stat_lines() {
    assert!(catalog::rubbleback_rhino().keywords.contains(&Keyword::Hexproof));
    assert!(catalog::skyline_predator().keywords.contains(&Keyword::Flash));
    assert!(catalog::towering_indrik().keywords.contains(&Keyword::Reach));
    assert!(catalog::tenement_crasher().keywords.contains(&Keyword::Haste));
    let thug = catalog::splatter_thug();
    assert!(thug.keywords.contains(&Keyword::FirstStrike) && thug.keywords.contains(&Keyword::Unleash));
}

/// Runewing draws a card when it dies.
#[test]
fn runewing_draws_on_death() {
    let mut g = two_player_game();
    let wing = g.add_card_to_battlefield(0, catalog::runewing()); // 2/2
    g.add_card_to_library(0, catalog::grizzly_bears());
    let h = g.players[0].hand.len();
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(wing), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert!(g.battlefield_find(wing).is_none(), "Runewing died");
    assert_eq!(g.players[0].hand.len(), h + 1, "drew on death");
}

/// Seller of Songbirds makes a 1/1 flying Bird on entry.
#[test]
fn seller_of_songbirds_makes_bird() {
    let mut g = two_player_game();
    let seller = g.add_card_to_hand(0, catalog::seller_of_songbirds());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: seller, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Bird"
        && c.definition.keywords.contains(&Keyword::Flying)), "1/1 flying Bird token");
}

/// Korozda Monitor scavenges from the graveyard to grow a creature.
#[test]
fn korozda_monitor_scavenges() {
    use crabomination::card::CounterType;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let mon = g.add_card_to_graveyard(0, catalog::korozda_monitor()); // power 3
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mon, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: vec![], x_value: None,
    }).expect("scavenge");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 3,
        "three +1/+1 counters (Monitor's power)");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == mon), "Monitor exiled by scavenge");
}

/// Tavern Swindler's coin flip gains 6 life on a win.
#[test]
fn tavern_swindler_wins_flip() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let sw = g.add_card_to_battlefield(0, catalog::tavern_swindler());
    g.clear_sickness(sw);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sw, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("flip");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 3 + 6, "paid 3 life, won 6");
}

/// Explosive Impact burns a player for 5.
#[test]
fn explosive_impact_deals_5() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::explosive_impact());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 5, "5 damage");
}

/// Auger Spree gives +4/-4.
#[test]
fn auger_spree_pumps_and_shrinks() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spree = g.add_card_to_hand(0, catalog::auger_spree());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spree, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // 2/2 → 6/-2 → dies to SBA.
    let _ = g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "toughness dropped to -2, creature died");
}

/// Avenging Arrow only hits a creature that dealt damage this turn.
#[test]
fn avenging_arrow_needs_damage_dealt() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let dealt = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(dealt).unwrap().dealt_damage_this_turn = true;
    let arrow = g.add_card_to_hand(0, catalog::avenging_arrow());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: arrow, target: Some(Target::Permanent(dealt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dealt).is_none(), "the damaging creature is destroyed");
}

/// Skull Rend burns each opponent and discards two at random.
#[test]
fn skull_rend_damages_and_discards() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    let rend = g.add_card_to_hand(0, catalog::skull_rend());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    let life = g.players[1].life;
    let h = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: rend, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 damage to the opponent");
    assert_eq!(g.players[1].hand.len(), h - 2, "discarded two");
}

/// Dynacharge's overload pumps every creature you control.
#[test]
fn dynacharge_overload_pumps_team() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let over = catalog::dynacharge().alternative_cost.unwrap().effect_override.unwrap();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&over, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.computed_permanent(a).unwrap().power, 4, "first creature +2/+0");
    assert_eq!(g.computed_permanent(b).unwrap().power, 4, "second creature +2/+0");
}

/// Batch-4 vanilla/hybrid stat lines.
#[test]
fn rtr_batch4_stat_lines() {
    let rs = catalog::risen_sanctuary();
    assert_eq!((rs.power, rs.toughness), (8, 8));
    assert!(rs.keywords.contains(&Keyword::Vigilance));
    assert!(catalog::rakdos_shred_freak().keywords.contains(&Keyword::Haste));
    let gl = catalog::golgari_longlegs();
    assert_eq!((gl.power, gl.toughness), (5, 4));
}

/// Frostburn Weird pumps +1/-1 with its hybrid ability.
#[test]
fn frostburn_weird_pumps() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let weird = g.add_card_to_battlefield(0, catalog::frostburn_weird()); // 1/4
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: weird, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(weird).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "1/4 -> 2/3");
}

/// Phantom General buffs your creature tokens only.
#[test]
fn phantom_general_buffs_tokens() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::phantom_general());
    // A real (nontoken) creature is unaffected.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // A token creature (via Seller of Songbirds' Bird) gets +1/+1.
    let seller = g.add_card_to_hand(0, catalog::seller_of_songbirds());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: seller, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let bird = g.battlefield.iter().find(|c| c.definition.name == "Bird").unwrap().id;
    assert_eq!(g.computed_permanent(bird).unwrap().power, 2, "1/1 token -> 2/2");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "nontoken bear unaffected");
}

/// Slum Reaper makes each player sacrifice a creature.
#[test]
fn slum_reaper_edicts_everyone() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let reaper = g.add_card_to_hand(0, catalog::slum_reaper());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: reaper, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "opponent's only creature sacrificed");
}

/// Soulsworn Spirit detains an opponent's creature on entry.
#[test]
fn soulsworn_spirit_detains() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
        [crabomination::decision::DecisionAnswer::Target(Target::Permanent(foe))],
    ));
    let spirit = g.add_card_to_hand(0, catalog::soulsworn_spirit());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spirit, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().detained_by.is_some(), "opponent's creature detained");
}

/// Chaos Imps has trample only while it carries a +1/+1 counter.
#[test]
fn chaos_imps_trample_gated_on_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let imps = g.add_card_to_battlefield(0, catalog::chaos_imps());
    assert!(!g.computed_permanent(imps).unwrap().keywords.contains(&Keyword::Trample),
        "no trample without a counter");
    g.battlefield_find_mut(imps).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(imps).unwrap().keywords.contains(&Keyword::Trample),
        "trample once it has a +1/+1 counter");
}

// ── RTR gap wave 5 (gaps4.rs) ────────────────────────────────────────────────

/// Stat/keyword lines for the wave-5 vanilla / french-vanilla creatures.
#[test]
fn rtr_wave5_stat_lines() {
    assert!(catalog::trained_caracal().keywords.contains(&Keyword::Lifelink));
    assert!(catalog::fencing_ace().keywords.contains(&Keyword::DoubleStrike));
    let hb = catalog::hover_barrier();
    assert_eq!((hb.power, hb.toughness), (0, 6));
    assert!(hb.keywords.contains(&Keyword::Defender) && hb.keywords.contains(&Keyword::Flying));
    let hp = catalog::hussar_patrol();
    assert!(hp.keywords.contains(&Keyword::Flash) && hp.keywords.contains(&Keyword::Vigilance));
    assert!(catalog::golgari_decoy().keywords.contains(&Keyword::AllMustBlock));
}

/// Izzet Keyrune taps for U or R, then animates into a 2/1 creature.
#[test]
fn izzet_keyrune_animates() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let rune = g.add_card_to_battlefield(0, catalog::izzet_keyrune());
    g.clear_sickness(rune);
    // Not a creature at rest.
    assert!(!g.computed_permanent(rune).unwrap().card_types.contains(&crabomination::card::CardType::Creature));
    // Animate for {U}{R}.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rune, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(rune).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature), "became a creature");
    assert_eq!((cp.power, cp.toughness), (2, 1), "2/1 Elemental");
}

/// Armory Guard has vigilance only while you control a Gate.
#[test]
fn armory_guard_gate_vigilance() {
    let mut g = two_player_game();
    let guard = g.add_card_to_battlefield(0, catalog::armory_guard());
    assert!(!g.computed_permanent(guard).unwrap().keywords.contains(&Keyword::Vigilance),
        "no vigilance without a Gate");
    g.add_card_to_battlefield(0, catalog::azorius_guildgate());
    assert!(g.computed_permanent(guard).unwrap().keywords.contains(&Keyword::Vigilance),
        "vigilance once you control a Gate");
}

/// Axebane Guardian taps for mana equal to your defenders.
#[test]
fn axebane_guardian_mana_scales_with_defenders() {
    let mut g = two_player_game();
    let axe = g.add_card_to_battlefield(0, catalog::axebane_guardian()); // itself a defender
    g.clear_sickness(axe);
    g.add_card_to_battlefield(0, catalog::hover_barrier()); // second defender
    g.perform_action(GameAction::ActivateAbility {
        card_id: axe, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("mana");
    assert_eq!(g.players[0].mana_pool.total(), 2, "two mana (two defenders)");
}

/// Lobber Crew pings each opponent and untaps when you cast a multicolored spell.
#[test]
fn lobber_crew_pings_and_untaps_on_multicolor() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let crew = g.add_card_to_battlefield(0, catalog::lobber_crew());
    g.clear_sickness(crew);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: crew, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "1 damage to opponent");
    assert!(g.battlefield_find(crew).unwrap().tapped, "tapped after activating");
    // Cast a multicolored spell → untap.
    let gold = g.add_card_to_hand(0, catalog::auger_spree()); // {1}{B}{R} multicolored
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: gold,
        target: Some(crabomination::game::types::Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast multicolored");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(crew).unwrap().tapped, "untapped by the multicolored cast");
}

/// Judge's Familiar sacrifices to counter an instant unless {1} is paid.
#[test]
fn judges_familiar_counters_unpaid_spell() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let bird = g.add_card_to_battlefield(0, catalog::judges_familiar());
    // Opponent casts Explosive Impact ({5}{R}) with no spare mana to pay the {1}.
    let bolt = g.add_card_to_hand(1, catalog::explosive_impact());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add_colorless(5);
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    // Sacrifice the Bird to counter the spell on the stack.
    g.perform_action(GameAction::ActivateAbility {
        card_id: bird, ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(bolt)),
        additional_targets: vec![], x_value: None,
    }).expect("sac to counter");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "spell countered into graveyard");
    assert!(g.battlefield_find(bird).is_none(), "Bird sacrificed");
}

/// Korozda Guildmage's second ability sacrifices a creature for Saprolings equal
/// to its toughness.
#[test]
fn korozda_guildmage_makes_saprolings() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hover_barrier()); // 0/6 nontoken fodder (sacrificed first)
    let mage = g.add_card_to_battlefield(0, catalog::korozda_guildmage());
    g.clear_sickness(mage);
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for saprolings");
    drain_stack(&mut g);
    let saps = g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count();
    assert_eq!(saps, 6, "six Saprolings (sacrificed 0/6's toughness)");
}

/// Rootborn Defenses gives your creatures indestructible.
#[test]
fn rootborn_defenses_grants_indestructible() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::rootborn_defenses());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Indestructible),
        "your creature gains indestructible");
}

/// Civic Saber pumps +1/+0 for each color of the equipped creature.
#[test]
fn civic_saber_scales_with_host_colors() {
    let mut g = two_player_game();
    // Watchwolf is G/W — two colors.
    let wolf = g.add_card_to_battlefield(0, catalog::watchwolf());
    let saber = g.add_card_to_battlefield(0, catalog::civic_saber());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: saber, target: wolf }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(wolf).unwrap().power, 3 + 2, "+2/+0 (two colors)");
}

/// Ogre Jailbreaker can attack despite defender while you control a Gate.
#[test]
fn ogre_jailbreaker_attacks_with_gate() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let ogre = g.add_card_to_battlefield(0, catalog::ogre_jailbreaker());
    g.clear_sickness(ogre);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: ogre, target: AttackTarget::Player(1) }])).is_err(),
        "can't attack without a Gate (defender)",
    );
    g.add_card_to_battlefield(0, catalog::golgari_guildgate());
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: ogre, target: AttackTarget::Player(1) }]))
        .expect("can attack once you control a Gate");
}

/// Druid's Deliverance prevents all combat damage to you this turn (CR 615).
#[test]
fn druids_deliverance_prevents_combat_damage_to_you() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    use crabomination::mana::Color;
    let mut g = two_player_game();
    // Player 1 has an attacker; it's player 1's turn.
    g.active_player_idx = 1;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(bear);
    // Player 0 casts Druid's Deliverance in response.
    let spell = g.add_card_to_hand(0, catalog::druids_deliverance());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let life = g.players[0].life;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(0),
    }])).expect("attack");
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.resolve_combat().expect("combat");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "combat damage to you prevented");
}

// ── RTR gap wave 6 (gaps5.rs) ────────────────────────────────────────────────

/// Stat lines for the wave-6 vanilla / french-vanilla creatures.
#[test]
fn rtr_wave6_stat_lines() {
    let arch = catalog::archweaver();
    assert_eq!((arch.power, arch.toughness), (5, 5));
    assert!(arch.keywords.contains(&Keyword::Reach) && arch.keywords.contains(&Keyword::Trample));
    let troll = catalog::lotleth_troll();
    assert!(troll.keywords.contains(&Keyword::Trample));
}

/// Lotleth Troll grows by discarding a creature card.
#[test]
fn lotleth_troll_discards_creature_for_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let troll = g.add_card_to_battlefield(0, catalog::lotleth_troll());
    g.clear_sickness(troll);
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a creature card to pitch
    g.perform_action(GameAction::ActivateAbility {
        card_id: troll, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("discard for counter");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(troll).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "grew a +1/+1 counter");
}

/// Cryptborn Horror enters with counters equal to opponents' life lost this turn.
#[test]
fn cryptborn_horror_enters_with_life_lost_counters() {
    use crabomination::card::CounterType;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    g.players[1].life -= 5; // opponent lost 5 life this turn
    g.players[1].life_lost_this_turn = 5;
    let horror = g.add_card_to_hand(0, catalog::cryptborn_horror());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: horror, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(horror).unwrap().counter_count(CounterType::PlusOnePlusOne), 5,
        "entered with 5 +1/+1 counters");
}

/// Stab Wound shrinks the creature and drains its controller each upkeep.
#[test]
fn stab_wound_drains_and_shrinks() {
    use crabomination::game::types::{Target, TurnStep};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::stab_wound());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    // -2/-2 kills the 2/2.
    assert!(g.battlefield_find(bear).is_none(), "2/2 dies to -2/-2");
    // Enchant a bigger creature and check the upkeep drain.
    let ogre = g.add_card_to_battlefield(1, catalog::risen_sanctuary()); // 8/8
    let aura2 = g.add_card_to_battlefield(0, catalog::stab_wound());
    g.battlefield_find_mut(aura2).unwrap().attached_to = Some(ogre);
    let life = g.players[1].life;
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "controller loses 2 at upkeep");
}

/// Pursuit of Flight pumps and grants an activated flying ability.
#[test]
fn pursuit_of_flight_pumps_and_grants_flying() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::pursuit_of_flight());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(!cp.keywords.contains(&Keyword::Flying), "no flying until activated");
    // Activate the granted {U}: flying ability (index 0 on the enchanted creature).
    g.clear_sickness(bear);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("grant flying");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying),
        "granted flying until end of turn");
}

/// Knightly Valor makes a Knight token on entry and buffs the host.
#[test]
fn knightly_valor_makes_knight_and_buffs() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::knightly_valor());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "granted vigilance");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Knight"
        && c.definition.keywords.contains(&Keyword::Vigilance)), "made a 2/2 vigilance Knight");
}

/// Hellhole Flailer sacrifices itself to burn for its power (CR — sac_cost
/// stamps Value::SacrificedPower).
#[test]
fn hellhole_flailer_sacs_for_power_damage() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let flailer = g.add_card_to_battlefield(0, catalog::hellhole_flailer()); // 3/2
    g.clear_sickness(flailer);
    // Unleash it up to a 4/3 so the sacrifice reads a pumped power.
    g.battlefield_find_mut(flailer).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: flailer, ability_index: 0,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("sac to burn");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flailer).is_none(), "Flailer sacrificed");
    assert_eq!(g.players[1].life, life - 4, "dealt 4 (its unleashed power)");
}

/// Chronic Flooding mills the enchanted land's controller when it taps for mana
/// (aura-granted `EventKind::Tapped` trigger keyed on the host land).
#[test]
fn chronic_flooding_mills_on_land_tap() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::island()); // opponent's land
    let aura = g.add_card_to_hand(0, catalog::chronic_flooding());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    for _ in 0..5 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let lib = g.players[1].library.len();
    // Land becomes tapped → enchanted-land trigger mills its controller (player 1).
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: land, actor: None }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib - 3, "controller milled 3 on tap");
}

/// Soul Tithe forces a pay-mana-value-or-sacrifice each upkeep (CR 701.16).
#[test]
fn soul_tithe_sacrifices_when_unpaid() {
    use crabomination::game::types::{Target, TurnStep};
    let mut g = two_player_game();
    // Risen Sanctuary ({5}{G}{W}, MV 7) belonging to the opponent.
    let ogre = g.add_card_to_battlefield(1, catalog::risen_sanctuary());
    let aura = g.add_card_to_hand(0, catalog::soul_tithe());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(ogre)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    // Opponent's upkeep with no mana → they can't pay 5 → sacrifice.
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ogre).is_none(), "sacrificed when the MV goes unpaid");
}

/// Soul Tithe lets the controller keep the permanent when they can pay its MV.
#[test]
fn soul_tithe_kept_when_paid() {
    use crabomination::game::types::{Target, TurnStep};
    let mut g = two_player_game();
    let ogre = g.add_card_to_battlefield(1, catalog::risen_sanctuary()); // {5}{G}{W}, MV 7
    let aura = g.add_card_to_hand(0, catalog::soul_tithe());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(ogre)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add_colorless(7); // enough to pay the {7}
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ogre).is_some(), "kept when the MV is paid");
}

// ── RTR gap wave 7 (gaps6.rs) ────────────────────────────────────────────────

/// Cast a targeted sorcery/instant `def` by player 0 at `target`.
fn cast_at(g: &mut GameState, def: crabomination::card::CardDefinition, tgt: crabomination::game::types::Target,
           colorless: u32, colors: &[(crabomination::mana::Color, u32)]) -> crabomination::card::CardId {
    let id = g.add_card_to_hand(0, def);
    g.players[0].mana_pool.add_colorless(colorless);
    for (c, n) in colors { g.players[0].mana_pool.add(*c, *n); }
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(tgt), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(g);
    id
}

/// Trostani's Judgment exiles a creature.
#[test]
fn trostanis_judgment_exiles() {
    use crabomination::game::types::Target;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast_at(&mut g, catalog::trostanis_judgment(), Target::Permanent(bear), 5, &[(Color::White, 1)]);
    assert!(g.battlefield_find(bear).is_none(), "creature exiled");
    assert!(g.players[1].graveyard.iter().all(|c| c.id != bear), "exiled, not in graveyard");
}

/// Rakdos's Return burns for X and forces X discards.
#[test]
fn rakdos_return_burns_and_discards() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    let hand = g.players[1].hand.len();
    let life = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::rakdos_return());
    g.players[0].mana_pool.add_colorless(2); // X=2
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 damage");
    assert_eq!(g.players[1].hand.len(), hand - 2, "discarded 2");
}

/// Thoughtflare draws four then discards two (net +2).
#[test]
fn thoughtflare_draws_four_discards_two() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let hand = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::thoughtflare());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // Spent the Thoughtflare from hand (-1), drew 4, discarded 2 → net +1 vs start.
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew 4, discarded 2 (net +2)");
}

/// Search Warrant gains life equal to the target's hand size.
#[test]
fn search_warrant_gains_life_per_hand() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    let life = g.players[0].life;
    cast_at(&mut g, catalog::search_warrant(), Target::Player(1), 0,
        &[(crabomination::mana::Color::White, 1), (crabomination::mana::Color::Blue, 1)]);
    assert_eq!(g.players[0].life, life + 4, "gained life = opponent's 4-card hand");
}

/// Rites of Reaping pumps one creature and shrinks another.
#[test]
fn rites_of_reaping_pumps_and_shrinks() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::risen_sanctuary()); // 8/8
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::rites_of_reaping());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 11, "+3/+3");
    assert!(g.battlefield_find(theirs).is_none(), "2/2 dies to -3/-3");
}

/// Inaction Injunction detains and cantrips.
#[test]
fn inaction_injunction_detains_and_draws() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    cast_at(&mut g, catalog::inaction_injunction(), Target::Permanent(foe), 1,
        &[(crabomination::mana::Color::Blue, 1)]);
    assert!(g.battlefield_find(foe).unwrap().detained_by.is_some(), "detained");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Treasured Find returns a graveyard card and exiles itself.
#[test]
fn treasured_find_recurs_then_exiles_self() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = cast_at(&mut g, catalog::treasured_find(), Target::Permanent(dead), 0,
        &[(crabomination::mana::Color::Black, 1), (crabomination::mana::Color::Green, 1)]);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "card returned to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id), "Treasured Find exiled itself, not in graveyard");
}

/// Chemister's Trick overload debuffs each opposing creature.
#[test]
fn chemisters_trick_overload_hits_each() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::risen_sanctuary()); // 8/8
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let over = catalog::chemisters_trick().alternative_cost.unwrap().effect_override.unwrap();
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&over, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.computed_permanent(a).unwrap().power, 6, "8/8 → 6/8");
    assert_eq!(g.computed_permanent(b).unwrap().power, 0, "2/2 → 0/2");
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::MustAttack), "must attack");
}

// ── Gap wave 8 (guild legends / rares) ───────────────────────────────────────

/// Collective Blessing pumps your creatures +3/+3.
#[test]
fn collective_blessing_anthem() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::collective_blessing());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5), "2/2 → 5/5");
}

/// Armada Wurm enters with a 5/5 trample Wurm token.
#[test]
fn armada_wurm_makes_token() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::armada_wurm());
    drain_stack(&mut g);
    let tokens: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Wurm" && c.definition.keywords.contains(&Keyword::Trample))
        .collect();
    assert_eq!(tokens.len(), 1, "one 5/5 Wurm token minted");
    assert_eq!((tokens[0].definition.power, tokens[0].definition.toughness), (5, 5));
}

/// Slime Molding makes an X/X Ooze.
#[test]
fn slime_molding_x_token() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let sm = g.add_card_to_hand(0, catalog::slime_molding());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sm, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast");
    drain_stack(&mut g);
    let ooze = g.battlefield.iter().find(|c| c.definition.name == "Ooze").expect("Ooze token");
    assert_eq!((g.computed_permanent(ooze.id).unwrap().power, g.computed_permanent(ooze.id).unwrap().toughness), (3, 3));
}

/// Dark Revenant returns to the top of its owner's library when it dies.
#[test]
fn dark_revenant_returns_to_top() {
    let mut g = two_player_game();
    let rev = g.add_card_to_battlefield(0, catalog::dark_revenant());
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(rev), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert!(g.battlefield_find(rev).is_none(), "Dark Revenant left the battlefield");
    assert_eq!(g.players[0].library.first().map(|c| c.definition.name), Some("Dark Revenant"),
        "put on top of library");
}

/// Gobbling Ooze grows by sacrificing another creature.
#[test]
fn gobbling_ooze_grows() {
    use crabomination::card::CounterType;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let ooze = g.add_card_to_battlefield(0, catalog::gobbling_ooze());
    g.clear_sickness(ooze);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ooze, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for counter");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.battlefield_find(ooze).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Hypersonic Dragon lets you cast sorceries outside your main phase.
#[test]
fn hypersonic_dragon_flash_sorcery() {
    use crabomination::mana::Color;
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hypersonic_dragon());
    let sorcery = g.add_card_to_hand(0, catalog::wrath_of_god());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 2);
    g.step = TurnStep::DeclareAttackers; // sorcery-illegal window
    g.perform_action(GameAction::CastSpell {
        card_id: sorcery, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hypersonic Dragon grants sorcery-as-flash");
}

/// Azorius Justiciar detains up to two opposing creatures on entry.
#[test]
fn azorius_justiciar_detains_two() {
    let mut g = two_player_game();
    g.step = crabomination::game::TurnStep::PreCombatMain;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let jus = g.add_card_to_hand(0, catalog::azorius_justiciar());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: jus, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast + detain two");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().detained_by.is_some(), "first detained");
    assert!(g.battlefield_find(b).unwrap().detained_by.is_some(), "second detained");
}

/// Traitorous Instinct steals a creature, untaps it, and gives haste + pump.
#[test]
fn traitorous_instinct_steals() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    cast_at(&mut g, catalog::traitorous_instinct(), Target::Permanent(foe), 3,
        &[(crabomination::mana::Color::Red, 1)]);
    let c = g.battlefield_find(foe).unwrap();
    assert_eq!(c.controller, 0, "gained control");
    assert!(!c.tapped, "untapped");
    assert_eq!(g.computed_permanent(foe).unwrap().power, 4, "2/2 → 4/2");
    assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::Haste), "gained haste");
}

/// Wayfaring Temple's P/T equals the number of creatures you control.
#[test]
fn wayfaring_temple_cda() {
    let mut g = two_player_game();
    let temple = g.add_card_to_battlefield(0, catalog::wayfaring_temple());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.computed_permanent(temple).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2), "2 creatures → 2/2");
}

/// Trostani gains life equal to another creature's toughness on entry.
#[test]
fn trostani_gains_life_on_etb() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::trostani_selesnyas_voice());
    let life = g.players[0].life;
    let big = g.add_card_to_hand(0, catalog::risen_sanctuary()); // 8/8
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast 8/8");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 8, "gained 8 (its toughness)");
}

/// Havoc Festival halves each player's life at their upkeep and blocks lifegain.
#[test]
fn havoc_festival_upkeep_halves_life() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::havoc_festival());
    g.players[0].life = 20;
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 10, "20 → 10 (half, rounded up)");
    // Lifegain is blocked by the static.
    g.adjust_life(0, 5);
    assert_eq!(g.players[0].life, 10, "can't gain life");
}

/// Seek the Horizon fetches up to three basics to hand.
#[test]
fn seek_the_horizon_fetches_basics() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let f: Vec<_> = (0..4).map(|_| g.add_card_to_library(0, catalog::forest())).collect();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f[0])),
        DecisionAnswer::Search(Some(f[1])),
        DecisionAnswer::Search(Some(f[2])),
    ]));
    let hand = g.players[0].hand.len();
    let seek = g.add_card_to_hand(0, catalog::seek_the_horizon());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: seek, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // Seek leaves the hand to the stack; three basics arrive → net +3 from the pre-cast size.
    assert_eq!(g.players[0].hand.len(), hand + 3, "fetched three basics");
}

/// Psychic Spiral mills the target equal to your graveyard size.
#[test]
fn psychic_spiral_mills_graveyard_size() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    for _ in 0..5 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let lib1 = g.players[1].library.len();
    cast_at(&mut g, catalog::psychic_spiral(), Target::Player(1), 4,
        &[(crabomination::mana::Color::Blue, 1)]);
    assert_eq!(g.players[1].library.len(), lib1 - 3, "milled 3 (graveyard size)");
    assert!(!g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the three bears were shuffled out of the graveyard");
}

/// Launch Party sacrifices a creature, destroys a target, and drains 2.
#[test]
fn launch_party_sac_destroy_drain() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let l1 = g.players[1].life;
    cast_at(&mut g, catalog::launch_party(), Target::Permanent(victim), 3,
        &[(crabomination::mana::Color::Black, 1)]);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert!(g.battlefield_find(victim).is_none(), "target destroyed");
    assert_eq!(g.players[1].life, l1 - 2, "controller lost 2");
}
// temp debug appended

/// Archon of the Triumvirate detains two nonland permanents when it attacks.
#[test]
fn archon_detains_two_on_attack() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let archon = g.add_card_to_battlefield(0, catalog::archon_of_the_triumvirate());
    g.clear_sickness(archon);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: archon, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().detained_by.is_some(), "first detained");
    assert!(g.battlefield_find(b).unwrap().detained_by.is_some(), "second detained");
}

/// Utvara Hellkite mints a 6/6 Dragon whenever a Dragon you control attacks.
#[test]
fn utvara_hellkite_makes_dragon_on_attack() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let utvara = g.add_card_to_battlefield(0, catalog::utvara_hellkite());
    g.clear_sickness(utvara);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: utvara, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let dragons = g.battlefield.iter()
        .filter(|c| c.definition.name == "Dragon" && c.definition.keywords.contains(&Keyword::Flying))
        .count();
    assert_eq!(dragons, 1, "one 6/6 Dragon token minted on the Hellkite's own attack");
}

/// Necropolis Regent grows a creature by the combat damage it deals to a player.
#[test]
fn necropolis_regent_adds_counters_on_combat_damage() {
    use crabomination::card::CounterType;
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::necropolis_regent());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(bear);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.resolve_combat().expect("combat");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "gained 2 counters (its combat damage)");
}

// ── Gap wave 9 (guildmages, counters, Overload) ──────────────────────────────

/// Tower Drake firebreathes +0/+1 for {W}.
#[test]
fn tower_drake_toughness_pump() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(0, catalog::tower_drake());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: drake, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(drake).unwrap().toughness, 2, "2/1 → 2/2");
}

/// Paralyzing Grasp keeps the enchanted creature from untapping.
#[test]
fn paralyzing_grasp_prevents_untap() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    cast_at(&mut g, catalog::paralyzing_grasp(), Target::Permanent(foe), 2,
        &[(crabomination::mana::Color::Blue, 1)]);
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(foe).unwrap().tapped, "still tapped after its untap step");
}

/// Essence Backlash counters a creature spell and burns its controller for its power.
#[test]
fn essence_backlash_counters_and_burns() {
    use crabomination::game::types::Target;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(1, catalog::risen_sanctuary()); // 8/8
    g.players[1].mana_pool.add_colorless(5);
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add(Color::White, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts 8/8");
    g.priority.player_with_priority = 0;
    let l1 = g.players[1].life;
    let eb = g.add_card_to_hand(0, catalog::essence_backlash());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eb, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("counter it");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "creature spell countered");
    assert_eq!(g.players[1].life, l1 - 8, "controller took 8 (its power)");
}

/// Counterflux counters an opponent's spell.
#[test]
fn counterflux_counters_opponent_spell() {
    use crabomination::game::types::Target;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts Bolt");
    g.priority.player_with_priority = 0;
    let cf = g.add_card_to_hand(0, catalog::counterflux());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: cf, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("counterflux");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "Bolt countered — no damage");
}

/// Mercurial Chemister's second ability exiles a graveyard I/S for damage = its MV.
#[test]
fn mercurial_chemister_gy_exile_burn() {
    use crabomination::game::types::Target;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let chem = g.add_card_to_battlefield(0, catalog::mercurial_chemister());
    g.clear_sickness(chem);
    let wrath = g.add_card_to_graveyard(0, catalog::wrath_of_god()); // MV 4
    let l1 = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: chem, ability_index: 1,
        target: Some(Target::Permanent(wrath)),
        additional_targets: vec![Target::Player(1)],
        x_value: None,
    }).expect("exile + burn");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == wrath), "the I/S card was exiled");
    assert_eq!(g.players[1].life, l1 - 4, "dealt 4 (its mana value)");
}

/// Grove of the Guardian sacrifices itself and taps two creatures to make an 8/8.
#[test]
fn grove_of_the_guardian_makes_elemental() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let grove = g.add_card_to_battlefield(0, catalog::grove_of_the_guardian());
    g.clear_sickness(grove);
    let c1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: grove, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("make elemental");
    drain_stack(&mut g);
    assert!(g.battlefield_find(grove).is_none(), "Grove sacrificed");
    assert!(g.battlefield_find(c1).unwrap().tapped && g.battlefield_find(c2).unwrap().tapped, "two creatures tapped");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Elemental"
        && c.definition.power == 8 && c.definition.keywords.contains(&Keyword::Vigilance)),
        "8/8 vigilant Elemental minted");
}

// ── Gap wave 10 (punisher / death payoffs / magecraft) ───────────────────────

/// Shrieking Affliction drains a hellbent opponent at their upkeep.
#[test]
fn shrieking_affliction_drains_low_hand() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::shrieking_affliction());
    g.players[1].hand.clear(); // 0 cards ≤ 1
    let l1 = g.players[1].life;
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 3, "opponent lost 3 (hellbent)");
}

/// Shrieking Affliction does nothing when the opponent holds 2+ cards.
#[test]
fn shrieking_affliction_skips_full_hand() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::shrieking_affliction());
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    let l1 = g.players[1].life;
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1, "no drain with a full hand");
}

/// Desecration Demon taps and grows when an opponent sacrifices to it.
#[test]
fn desecration_demon_sac_taps_and_grows() {
    use crabomination::card::CounterType;
    use crabomination::game::types::TurnStep;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let demon = g.add_card_to_battlefield(0, catalog::desecration_demon());
    let fodder = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "opponent sacrificed a creature");
    assert!(g.battlefield_find(demon).unwrap().tapped, "Demon tapped");
    assert_eq!(g.battlefield_find(demon).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "grew");
}

/// Death's Presence puts counters equal to the dead creature's power.
#[test]
fn deaths_presence_counters_on_death() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::deaths_presence());
    let target = g.add_card_to_battlefield(0, catalog::risen_sanctuary()); // 8/8 recipient
    let dying = g.add_card_to_battlefield(0, catalog::risen_sanctuary()); // 8/8 dies
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(dying), 8, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 8,
        "8 counters (dead creature's power)");
}

/// Pyroconvergence pings when you cast a multicolored spell.
#[test]
fn pyroconvergence_pings_on_multicolor_cast() {
    use crabomination::game::types::Target;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pyroconvergence());
    // Rites of Reaping is a gold {4}{B}{G} sorcery already in the RTR set.
    let gold = g.add_card_to_hand(0, catalog::rites_of_reaping());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::risen_sanctuary());
    let l1 = g.players[1].life;
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: gold, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("cast gold spell");
    // The magecraft-style trigger goes on the stack above the spell; drain it.
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 2, "Pyroconvergence dealt 2 to the opponent");
}

/// Firemind's Foresight tutors three instants of mana value 3, 2, and 1.
#[test]
fn fireminds_foresight_fetches_three_costs() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let one = g.add_card_to_library(0, catalog::lightning_bolt());   // {R} = MV1
    let two = g.add_card_to_library(0, catalog::dramatic_rescue());  // {W}{U} = MV2
    let three = g.add_card_to_library(0, catalog::counterflux());    // {U}{U}{R} = MV3
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(three)),
        DecisionAnswer::Search(Some(two)),
        DecisionAnswer::Search(Some(one)),
    ]));
    let ff = g.add_card_to_hand(0, catalog::fireminds_foresight());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ff, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    for (id, mv) in [(one, 1), (two, 2), (three, 3)] {
        assert!(g.players[0].hand.iter().any(|c| c.id == id), "fetched the MV{mv} instant");
    }
}

/// Jarad's power grows with creature cards in your graveyard, and his sac
/// ability drains each opponent for the sacrificed creature's power.
#[test]
fn jarad_grows_and_drains() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    let jarad = g.add_card_to_battlefield(0, catalog::jarad_golgari_lich_lord());
    assert_eq!(g.computed_permanent(jarad).unwrap().power, 5, "2 base + 3 creatures in gy");
    let fodder = g.add_card_to_battlefield(0, catalog::risen_sanctuary()); // 8/8 to sacrifice
    let opp = g.players[1].life;
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: jarad, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac-drain");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[1].life, opp - 8, "opponent drained for sacrificed power");
}

/// Jarad returns himself from the graveyard by sacrificing a Swamp and Forest.
#[test]
fn jarad_recurs_from_graveyard() {
    let mut g = two_player_game();
    let jarad = g.add_card_to_graveyard(0, catalog::jarad_golgari_lich_lord());
    let swamp = g.add_card_to_battlefield(0, catalog::swamp());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: jarad, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("recur");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == jarad), "Jarad back in hand");
    assert!(g.battlefield_find(swamp).is_none() && g.battlefield_find(forest).is_none(),
        "Swamp and Forest sacrificed");
}

/// Conjured Currency swaps control of itself with an opponent's permanent.
#[test]
fn conjured_currency_swaps_control() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let curr = g.add_card_to_battlefield(0, catalog::conjured_currency());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "took the bear");
    assert_eq!(g.battlefield_find(curr).unwrap().controller, 1, "gave up the currency");
}

/// Volatile Rig, on losing its death coin flip, explodes for 4 to everything.
#[test]
fn volatile_rig_explodes_on_death() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    let rig = g.add_card_to_battlefield(0, catalog::volatile_rig()); // 4/4
    let bystander = g.add_card_to_battlefield(1, catalog::risen_sanctuary()); // 8/8 survives 4
    // Lost dealt-damage flip → sacrifice; lost death flip → 4 to all.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false), DecisionAnswer::Bool(false),
    ]));
    let opp = g.players[1].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(rig), 4, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let death = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&death);
    drain_stack(&mut g);
    assert!(g.battlefield_find(rig).is_none(), "Rig gone");
    assert_eq!(g.players[1].life, opp - 4, "opponent took 4 from the blast");
    assert_eq!(g.battlefield_find(bystander).unwrap().damage, 4, "bystander took 4");
}

/// Oak Street Innkeeper gives your tapped creatures hexproof only on others' turns.
#[test]
fn oak_street_innkeeper_hexproofs_tapped_on_others_turns() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::oak_street_innkeeper());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.active_player_idx = 1; // opponent's turn
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof),
        "tapped creature is hexproof on the opponent's turn");
    g.battlefield_find_mut(bear).unwrap().tapped = false;
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof),
        "untapped creature isn't hexproof");
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.active_player_idx = 0; // your own turn
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof),
        "no hexproof during your own turn");
}

/// Urban Burgeoning untaps its enchanted land during an opponent's untap step.
#[test]
fn urban_burgeoning_untaps_on_opponents_untap() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let aura = g.add_card_to_battlefield(0, catalog::urban_burgeoning());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(land);
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.active_player_idx = 1; // opponent's untap step
    g.do_untap();
    assert!(!g.battlefield_find(land).unwrap().tapped,
        "enchanted land untapped on the opponent's untap step");
}

/// Street Sweeper destroys the Auras on a land when it attacks.
#[test]
fn street_sweeper_clears_land_auras() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let sweeper = g.add_card_to_battlefield(0, catalog::street_sweeper());
    g.clear_sickness(sweeper);
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let aura = g.add_card_to_battlefield(1, catalog::racecourse_fury());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(land);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sweeper, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "the land's Aura was destroyed");
    assert!(g.battlefield_find(land).is_some(), "the land itself survives");
}

/// Jarad's Orders fetches one creature to hand and one to the graveyard.
#[test]
fn jarads_orders_splits_hand_and_graveyard() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::grizzly_bears());
    let b = g.add_card_to_library(0, catalog::risen_sanctuary());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(a)),
        DecisionAnswer::Search(Some(b)),
    ]));
    let ord = g.add_card_to_hand(0, catalog::jarads_orders());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ord, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == a), "first pick to hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == b), "second pick to graveyard");
}

/// Racecourse Fury lets its enchanted land grant haste.
#[test]
fn racecourse_fury_grants_haste() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    let fury = g.add_card_to_battlefield(0, catalog::racecourse_fury());
    g.battlefield_find_mut(fury).unwrap().attached_to = Some(land);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
    // Index 0 is the land's own mana ability; the granted haste ability is 1.
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("grant haste");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste),
        "enchanted land granted haste");
}

/// Security Blockade mints a Knight and its land prevents 1 damage.
#[test]
fn security_blockade_knight_and_prevention() {
    use crabomination::game::effects::EntityRef;
    use crabomination::game::types::Target;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::plains());
    let aura = g.add_card_to_hand(0, catalog::security_blockade());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    let knights_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature()).count();
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count(),
        knights_before + 1, "made a Knight token");
    // Activate the granted prevention (index 1; 0 is the land's mana), then
    // 3 damage lands as 2.
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("prevent shield");
    drain_stack(&mut g);
    let life = g.players[0].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 3, None, &mut evs);
    assert_eq!(g.players[0].life, life - 2, "1 of 3 damage prevented");
}

/// Izzet Staticaster pings the target creature and every same-named creature.
#[test]
fn izzet_staticaster_pings_same_name() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let caster = g.add_card_to_battlefield(0, catalog::izzet_staticaster());
    g.clear_sickness(caster);
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(1, catalog::risen_sanctuary());
    g.perform_action(GameAction::ActivateAbility {
        card_id: caster, ability_index: 0, target: Some(Target::Permanent(bear1)),
        additional_targets: vec![], x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear1).unwrap().damage, 1, "target bear pinged");
    assert_eq!(g.battlefield_find(bear2).unwrap().damage, 1, "same-named bear pinged");
    assert_eq!(g.battlefield_find(other).unwrap().damage, 0, "different name untouched");
}

/// Mana Bloom enters with X charge counters and taps them for any color.
#[test]
fn mana_bloom_enters_with_counters_and_taps_for_mana() {
    use crabomination::card::CounterType;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let bloom = g.add_card_to_hand(0, catalog::mana_bloom());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: bloom, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast for X=3");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bloom).unwrap().counter_count(CounterType::Charge), 3,
        "entered with 3 charge counters");
    g.perform_action(GameAction::ActivateAbility {
        card_id: bloom, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bloom).unwrap().counter_count(CounterType::Charge), 2,
        "spent one counter");
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
}

/// CR 705.1 — Volatile Rig wins its dealt-damage coin flip (heads) and is not
/// sacrificed.
#[test]
fn cr_705_1_volatile_rig_survives_flip_on_heads() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    let rig = g.add_card_to_battlefield(0, catalog::volatile_rig());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(rig), 1, None, &mut evs); // non-lethal
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(rig).is_some(), "won the flip → not sacrificed");
}

/// CR 514.2 — the haste Racecourse Fury's land grants ends at cleanup.
#[test]
fn cr_514_2_racecourse_haste_expires_at_cleanup() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    let fury = g.add_card_to_battlefield(0, catalog::racecourse_fury());
    g.battlefield_find_mut(fury).unwrap().attached_to = Some(land);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("grant haste");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
    for card in g.battlefield.iter_mut() { card.clear_end_of_turn_effects(); }
    g.expire_end_of_turn_effects();
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste),
        "haste ended at cleanup");
}

/// Nivmagus Elemental exiles an instant spell it controls as an activation
/// cost, adds two +1/+1 counters, and the exiled spell doesn't resolve.
#[test]
fn nivmagus_elemental_eats_a_spell_for_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let niv = g.add_card_to_battlefield(0, catalog::nivmagus_elemental());
    // Put an instant on the stack (Player 0 controls it).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.cast_spell(bolt, Some(Target::Player(1)), vec![], None, None).expect("cast bolt");
    assert_eq!(g.stack.len(), 1, "bolt is on the stack");
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: niv, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("eat the spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(niv).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "gained two +1/+1 counters");
    assert!(g.stack.is_empty(), "the exiled spell left the stack");
    assert_eq!(g.players[1].life, opp_life, "the exiled bolt never resolved");
}

/// With no other spell you control on the stack, Nivmagus can't activate.
#[test]
fn nivmagus_elemental_needs_a_spell_to_eat() {
    let mut g = two_player_game();
    let niv = g.add_card_to_battlefield(0, catalog::nivmagus_elemental());
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: niv, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    });
    assert!(res.is_err(), "no spell to exile → activation rejected");
}

/// Faerie Impostor bounces another creature you control on ETB; with none, it
/// sacrifices itself.
#[test]
fn faerie_impostor_bounces_or_sacrifices() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let imp = g.add_card_to_battlefield(0, catalog::faerie_impostor());
    g.fire_self_etb_triggers(imp, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the bear was bounced");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear returned to hand");
    assert!(g.battlefield_find(imp).is_some(), "impostor stayed");

    // A second impostor entering with no other creature sacrifices itself.
    let lonely = g.add_card_to_battlefield(1, catalog::faerie_impostor());
    g.fire_self_etb_triggers(lonely, 1);
    drain_stack(&mut g);
    assert!(g.battlefield_find(lonely).is_none(), "lonely impostor sacrificed itself");
}

/// Righteous Authority scales +1/+1 with the enchanted creature's controller's
/// hand size.
#[test]
fn righteous_authority_scales_with_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_battlefield(0, catalog::righteous_authority());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    // Give player 0 a three-card hand.
    for _ in 0..3 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5), "2/2 + 3 cards in hand");
}

/// Slaughter Games names a card and exiles every copy from the opponent's
/// graveyard, hand, and library.
#[test]
fn slaughter_games_exiles_all_copies() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{GameAction, Target};
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let h = g.add_card_to_hand(1, catalog::grizzly_bears());
    let l = g.add_card_to_library(1, catalog::grizzly_bears());
    g.players[1].graveyard.push(crabomination::card::CardInstance::new(
        crabomination::card::CardId(9001), std::sync::Arc::new(catalog::grizzly_bears()), 1));
    let keep = g.add_card_to_hand(1, catalog::phantom_warrior());
    let spell = g.add_card_to_hand(0, catalog::slaughter_games());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard("Grizzly Bears".into())]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Slaughter Games");
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 3, "all three copies exiled");
    assert!(!g.players[1].hand.iter().any(|c| c.id == h), "hand copy gone");
    assert!(!g.players[1].library.iter().any(|c| c.id == l), "library copy gone");
    assert!(g.players[1].hand.iter().any(|c| c.id == keep), "the odd card stays");
}

/// Guild Feud deploys the best creature for each player and, when two enter,
/// they fight.
#[test]
fn guild_feud_deploys_and_fights() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::guild_feud());
    // Each library's top card is a creature (1/1 so both die in the fight).
    g.add_card_to_library(0, catalog::merfolk_of_the_pearl_trident());
    g.add_card_to_library(1, catalog::merfolk_of_the_pearl_trident());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    // Both 1/1s entered and fought to mutual death.
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Merfolk of the Pearl Trident").count(), 0,
        "the two deployed 1/1s fought and died");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Merfolk of the Pearl Trident"));
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Merfolk of the Pearl Trident"));
}

/// Grave Betrayal steals a creature you don't control that dies, returning it
/// under your control at the next end step with a +1/+1 counter as a Zombie.
#[test]
fn grave_betrayal_steals_the_dead() {
    use crabomination::card::{CounterType, CreatureType};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grave_betrayal());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // opponent's 2/2
    // Kill it with lethal damage so the SBA death dispatches to death-watchers.
    g.battlefield_find_mut(victim).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    // The delayed reanimation fires at the next end step.
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let reborn = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears")
        .expect("reanimated under Grave Betrayal's controller");
    assert_eq!(reborn.controller, 0, "now controlled by Grave Betrayal's owner");
    assert_eq!(*reborn.counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0), 1, "entered with a +1/+1 counter");
    assert!(reborn.definition.subtypes.creature_types.contains(&CreatureType::Zombie), "now a Zombie");
}


/// Angel of Serenity exiles up to three creatures on ETB; they return to their
/// owners' hands when it leaves.
#[test]
fn angel_of_serenity_exiles_then_returns() {
    use crabomination::game::types::Target;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_hand(0, catalog::angel_of_serenity());
    g.players[0].mana_pool.add(Color::White, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("cast Angel");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b), "both exiled on ETB");
    // Angel dies → exiled creatures go to their owner's hand.
    g.battlefield_find_mut(angel).unwrap().damage = 6;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == a) && g.players[1].hand.iter().any(|c| c.id == b),
        "returned to owner's hand when Angel left");
}
