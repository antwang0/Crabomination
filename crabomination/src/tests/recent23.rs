//! Functionality tests for `catalog::sets::decks::recent23` —
//! `Keyword::AssignsCombatDamageByToughness` (CR 510.1c).

use crate::catalog;
use crate::card::Keyword;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::TurnStep;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Doran makes every creature assign combat damage equal to its toughness: an
/// unblocked 0/5 Doran deals 5 to the defending player.
#[test]
fn doran_attacks_for_toughness() {
    let mut g = two_player_game();
    let doran = g.add_card_to_battlefield(0, catalog::doran_the_siege_tower()); // 0/5
    g.clear_sickness(doran);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: doran,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 15, "0/5 Doran assigns 5 (toughness)");
}

/// Doran's substitution is unconditional even when power exceeds toughness: a
/// 3/1 attacker assigns only 1.
#[test]
fn doran_caps_high_power_attacker_at_toughness() {
    let mut g = two_player_game();
    let doran = g.add_card_to_battlefield(0, catalog::doran_the_siege_tower());
    g.clear_sickness(doran);
    let bolt = g.add_card_to_battlefield(0, catalog::goblin_piker()); // 2/1
    g.clear_sickness(bolt);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bolt,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 19, "2/1 under Doran assigns 1 (toughness)");
}

/// Tapestry Warden only affects your creatures whose toughness exceeds their
/// power: a 1/4 Wall assigns 4, while a 2/1 you control assigns its normal 2.
#[test]
fn tapestry_warden_only_buffs_high_toughness() {
    let mut g = two_player_game();
    let warden = g.add_card_to_battlefield(0, catalog::tapestry_warden());
    g.clear_sickness(warden);
    // Warden itself is 3/4 (T>P) → assigns 4.
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: warden,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 16, "3/4 Warden assigns 4 (toughness)");
}

/// A creature you control with power ≥ toughness is left alone by Tapestry
/// Warden (a 2/1 still assigns 2, not 1).
#[test]
fn tapestry_warden_ignores_low_toughness() {
    let mut g = two_player_game();
    let warden = g.add_card_to_battlefield(0, catalog::tapestry_warden());
    g.clear_sickness(warden);
    let piker = g.add_card_to_battlefield(0, catalog::goblin_piker()); // 2/1
    g.clear_sickness(piker);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: piker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 18, "2/1 unaffected → assigns 2 (power)");
}

/// Ancient Lumberknot reuses Tapestry Warden's static: a 1/4 it controls (T>P)
/// assigns 4, attacking unblocked.
#[test]
fn ancient_lumberknot_buffs_high_toughness() {
    let mut g = two_player_game();
    let knot = g.add_card_to_battlefield(0, catalog::ancient_lumberknot()); // 1/4
    g.clear_sickness(knot);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: knot,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 16, "1/4 Lumberknot assigns 4 (toughness)");
}

/// Thrumming Hivepool's lord static grants double strike + haste to Slivers,
/// and its Affinity for Slivers reduces its {6} cost by {1} per Sliver (so two
/// Slivers let it cast for {4}).
#[test]
fn thrumming_hivepool_affinity_and_lord() {
    let mut g = two_player_game();
    let s1 = g.add_card_to_battlefield(0, catalog::muscle_sliver());
    g.add_card_to_battlefield(0, catalog::muscle_sliver());
    let pool = g.add_card_to_battlefield(0, catalog::thrumming_hivepool());
    assert!(
        g.computed_permanent(s1)
            .is_some_and(|c| c.keywords.contains(&Keyword::DoubleStrike)
                && c.keywords.contains(&Keyword::Haste)),
        "Slivers gain double strike + haste"
    );
    // Affinity: {6} reduced by {1} per Sliver (2 on board) → {4} generic.
    let inst = g.battlefield.iter().find(|c| c.id == pool).unwrap().clone();
    let reduced = crate::game::actions::cost_reduction_for_spell(&g, 0, &inst, None);
    assert_eq!(reduced, 2, "Affinity for Slivers gives {{2}} off with two Slivers");
}

/// Bill the Pony enters with two Food and can sacrifice one to grant the
/// toughness-damage keyword to a target creature you control until end of turn.
#[test]
fn bill_the_pony_etb_food_and_grant() {
    let mut g = two_player_game();
    let bill = g.move_card_to_battlefield_for_test(0, catalog::bill_the_pony());
    g.clear_sickness(bill);
    drain_stack(&mut g);
    let foods = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.is_token)
        .count();
    assert_eq!(foods, 2, "ETB makes two Food tokens");

    // Grant the keyword to Bill (a 1/4) by sacrificing a Food.
    g.perform_action(GameAction::ActivateAbility {
        card_id: bill,
        ability_index: 0,
        target: Some(Target::Permanent(bill)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("activate sac-a-Food grant");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(bill)
            .is_some_and(|c| c.keywords.contains(&Keyword::AssignsCombatDamageByToughness)),
        "Bill now assigns combat damage by toughness"
    );
    let foods_after = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.is_token)
        .count();
    assert_eq!(foods_after, 1, "one Food sacrificed");
}

/// Bedhead Beastie is a 5/6 with menace and Mountaincycling {2}.
#[test]
fn bedhead_beastie_keywords() {
    let d = catalog::bedhead_beastie();
    assert_eq!((d.power, d.toughness), (5, 6));
    assert!(d.keywords.contains(&Keyword::Menace));
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Typecycling(_))));
}

/// Daggermaw Megalodon is a 5/7 with vigilance and Islandcycling {2}.
#[test]
fn daggermaw_megalodon_keywords() {
    let d = catalog::daggermaw_megalodon();
    assert_eq!((d.power, d.toughness), (5, 7));
    assert!(d.keywords.contains(&Keyword::Vigilance));
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Typecycling(_))));
}

/// Boilerbilges Ripper sacrifices another creature on ETB to deal 2 to any
/// target (auto-decider sacrifices the fodder and pings the opponent).
#[test]
fn boilerbilges_ripper_sac_pings() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.move_card_to_battlefield_for_test(0, catalog::boilerbilges_ripper());
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "fodder sacrificed");
    assert_eq!(g.players[1].life, 18, "dealt 2 to opponent");
}

/// Bashful Beastie manifests dread when it dies (a face-down 2/2 enters).
#[test]
fn bashful_beastie_dies_manifest_dread() {
    let mut g = two_player_game();
    let beastie = g.add_card_to_battlefield(0, catalog::bashful_beastie());
    // Seed library so manifest dread has cards to look at.
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let mut evs = g.remove_to_graveyard_with_triggers(beastie);
    evs.push(GameEvent::CreatureDied { card_id: beastie });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.face_down),
        "manifest dread put a face-down creature onto the battlefield"
    );
}

/// Bear Trap has flash and can sacrifice itself to deal 3 to a creature.
#[test]
fn bear_trap_sac_burns_creature() {
    let mut g = two_player_game();
    let trap = g.add_card_to_battlefield(0, catalog::bear_trap());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: trap,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("activate Bear Trap");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 took 3 and died");
    assert!(!g.battlefield.iter().any(|c| c.id == trap), "Bear Trap sacrificed");
}


/// Frantic Strength gives the enchanted creature +2/+2 and trample.
#[test]
fn frantic_strength_pumps_and_tramples() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::frantic_strength());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Frantic Strength");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Trample), "granted trample");
}

/// Most Valuable Slayer's attack trigger gives an attacking creature +1/+0 and
/// first strike.
#[test]
fn most_valuable_slayer_pumps_attacker() {
    let mut g = two_player_game();
    let slayer = g.add_card_to_battlefield(0, catalog::most_valuable_slayer()); // 2/4
    g.clear_sickness(slayer);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: slayer,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(slayer).unwrap();
    assert_eq!(cp.power, 3, "attack trigger pumped +1/+0");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "gained first strike");
}

/// Twist Reality's first mode counters a spell on the stack.
#[test]
fn twist_reality_counters_spell() {
    let mut g = two_player_game();
    // Opponent casts a spell.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts bolt");
    // Counter it with Twist Reality (mode 0).
    let twist = g.add_card_to_hand(0, catalog::twist_reality());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: twist,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    }).expect("cast Twist Reality countering");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "bolt was countered (no damage)");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "countered bolt hits graveyard");
}

/// Vengeful Possession steals a creature until end of turn and untaps it.
#[test]
fn vengeful_possession_steals_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::vengeful_possession());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("cast Vengeful Possession");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.controller, 0, "gained control of the creature");
    assert!(!g.battlefield.iter().find(|c| c.id == bear).unwrap().tapped, "untapped");
    assert!(cp.keywords.contains(&Keyword::Haste), "gained haste");
}

/// Unstoppable Plan untaps your nonland permanents at your end step.
#[test]
fn unstoppable_plan_untaps_at_end_step() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::unstoppable_plan());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().find(|c| c.id == bear).unwrap().tapped, "untapped at end step");
}

/// Gearseeker Serpent's Affinity for artifacts discounts its generic cost.
#[test]
fn gearseeker_serpent_affinity_for_artifacts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    let serp = g.add_card_to_battlefield(0, catalog::gearseeker_serpent());
    let inst = g.battlefield.iter().find(|c| c.id == serp).unwrap().clone();
    let reduced = crate::game::actions::cost_reduction_for_spell(&g, 0, &inst, None);
    assert_eq!(reduced, 3, "three artifacts give {{3}} off");
}

/// Aetherjacket sacrifices itself to destroy another artifact.
#[test]
fn aetherjacket_sacs_to_destroy_artifact() {
    let mut g = two_player_game();
    let jacket = g.add_card_to_battlefield(0, catalog::aetherjacket());
    let target = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: jacket,
        ability_index: 0,
        target: Some(Target::Permanent(target)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("activate Aetherjacket");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == target), "target artifact destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == jacket), "Aetherjacket sacrificed");
}

/// Dynamite Diver deals 1 to any target when it dies.
#[test]
fn dynamite_diver_dies_pings() {
    let mut g = two_player_game();
    let diver = g.add_card_to_battlefield(0, catalog::dynamite_diver());
    let mut evs = g.remove_to_graveyard_with_triggers(diver);
    evs.push(GameEvent::CreatureDied { card_id: diver });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "dies trigger pinged the opponent for 1");
}

/// Gas Guzzler starts your engines, enters tapped, and its max-speed ability is
/// gated until speed 4.
#[test]
fn gas_guzzler_starts_engines_enters_tapped() {
    let mut g = two_player_game();
    let guz = g.move_card_to_battlefield_for_test(0, catalog::gas_guzzler());
    assert!(g.battlefield.iter().find(|c| c.id == guz).unwrap().tapped, "enters tapped");
    assert_eq!(g.players[0].speed, 1, "Start your engines! sets speed to 1");
}

/// Chitin Gravestalker's graveyard affinity discounts {1} per artifact/creature
/// card in your graveyard.
#[test]
fn chitin_gravestalker_graveyard_affinity() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
    g.add_card_to_graveyard(0, catalog::sol_ring()); // artifact
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // neither — no discount
    let inst = crate::card::CardInstance::new(
        crate::card::CardId(9999),
        catalog::chitin_gravestalker(),
        0,
    );
    let reduced = crate::game::actions::cost_reduction_for_spell(&g, 0, &inst, None);
    assert_eq!(reduced, 2, "two matching gy cards give {{2}} off");
}

/// Unnerving Grasp bounces a target permanent and manifests dread.
#[test]
fn unnerving_grasp_bounces_and_manifests() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::unnerving_grasp());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Unnerving Grasp");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bear returned to hand");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.face_down),
        "manifest dread put a face-down creature onto the battlefield");
}

/// Fanged Flames deals 4 and exiles the creature instead of letting it die.
#[test]
fn fanged_flames_exiles_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::fanged_flames());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fanged Flames");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "lethal creature was exiled, not killed");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "not in graveyard");
}

/// Splitskin Doll draws, then discards when you control no other small creature.
#[test]
fn splitskin_doll_discards_without_small_creature() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard
    let hand_before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::splitskin_doll());
    drain_stack(&mut g);
    // Drew 1, then discarded 1 (no other power-≤2 creature) → net hand unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before, "draw then discard nets zero");
}

/// With another small creature out, Splitskin Doll keeps the drawn card.
#[test]
fn splitskin_doll_keeps_card_with_small_creature() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::goblin_piker()); // 2/1, power ≤2
    let hand_before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::splitskin_doll());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew and kept (no discard)");
}

/// Skittering Surveyor fetches a basic land to hand on ETB.
#[test]
fn skittering_surveyor_fetches_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest)),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::skittering_surveyor());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "fetched the Forest to hand");
}

/// Agonasaur Rex's cycle trigger puts two +1/+1 counters on a creature and
/// grants it trample + indestructible.
#[test]
fn agonasaur_rex_cycle_buffs_creature() {
    use crate::card::{CounterType, Keyword};
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rex = g.add_card_to_hand(0, catalog::agonasaur_rex());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Target the bear with the reflexive cycle trigger.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    g.perform_action(GameAction::Cycle { card_id: rex, x_value: None }).expect("cycle Agonasaur Rex");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "two +1/+1 counters");
    assert!(cp.keywords.contains(&Keyword::Trample) && cp.keywords.contains(&Keyword::Indestructible),
        "granted trample + indestructible");
}

/// Marketwatch Phantom gains flying when another small creature you control
/// enters.
#[test]
fn marketwatch_phantom_gains_flying() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let phantom = g.add_card_to_battlefield(0, catalog::marketwatch_phantom());
    assert!(!g.computed_permanent(phantom).unwrap().keywords.contains(&Keyword::Flying),
        "no flying yet");
    // Cast a 2/1 (power ≤2) through the real ETB funnel so the trigger fires.
    let piker = g.add_card_to_hand(0, catalog::goblin_piker());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: piker, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a small creature");
    drain_stack(&mut g);
    assert!(g.computed_permanent(phantom).unwrap().keywords.contains(&Keyword::Flying),
        "gained flying when a small creature entered");
}
