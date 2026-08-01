//! Planeshift (PLS) — Domain, the Kavu, and the Dragon Charms.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Allied Strategies draws by domain.
#[test]
fn allied_strategies_draws_per_basic_type() {
    let mut g = main_phase();
    for land in [catalog::plains, catalog::island, catalog::swamp] {
        g.add_card_to_battlefield(0, land());
    }
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::allied_strategies());
    cast(&mut g, 0, spell, Some(Target::Player(0)));
    assert_eq!(g.players[0].hand.len(), 3, "three basic types");
}

/// Alpha Kavu shrinks a Kavu's power and props up its toughness.
#[test]
fn alpha_kavu_swaps_a_kavus_stats() {
    let mut g = main_phase();
    let alpha = g.add_card_to_battlefield(0, catalog::alpha_kavu());
    let other = g.add_card_to_battlefield(0, catalog::caldera_kavu());
    activate(&mut g, 0, alpha, 0, Some(Target::Permanent(other)));
    let cp = g.computed_permanent(other).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 3));
}

/// Amphibious Kavu grows when a blue creature blocks it.
#[test]
fn amphibious_kavu_swells_against_blue() {
    let mut g = main_phase();
    let kavu = g.add_card_to_battlefield(0, catalog::amphibious_kavu());
    g.clear_sickness(kavu);
    let blocker = g.add_card_to_battlefield(1, catalog::wall_of_air());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: kavu,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, kavu)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(kavu).unwrap().power, 5);
}

/// Arctic Merfolk's kicker is a bounce, not mana.
#[test]
fn arctic_merfolk_kicked_by_bouncing_a_creature() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let merfolk = g.add_card_to_hand(0, catalog::arctic_merfolk());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: merfolk,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != bears), "the Bears paid the kicker");
    assert_eq!(g.computed_permanent(merfolk).unwrap().power, 2);
}

/// Aura Blast trades for an enchantment and replaces itself.
#[test]
fn aura_blast_destroys_and_draws() {
    let mut g = main_phase();
    let pacifism = g.add_card_to_battlefield(1, catalog::pacifism());
    g.add_card_to_library(0, catalog::forest());
    let blast = g.add_card_to_hand(0, catalog::aura_blast());
    let before = g.players[0].hand.len();
    cast(&mut g, 0, blast, Some(Target::Permanent(pacifism)));
    assert!(g.battlefield.iter().all(|c| c.id != pacifism));
    assert_eq!(g.players[0].hand.len(), before, "spell out, card in");
}

/// Bog Down takes a third card when kicked.
#[test]
fn bog_down_kicked_takes_three() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let bog = g.add_card_to_hand(0, catalog::bog_down());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: bog,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1);
    assert!(g.battlefield.iter().all(|c| !c.definition.is_land()), "two lands paid the kicker");
}

/// Caldera Kavu can become any colour.
#[test]
fn caldera_kavu_recolors_itself() {
    let mut g = main_phase();
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Color(Color::White),
    ]));
    let kavu = g.add_card_to_battlefield(0, catalog::caldera_kavu());
    activate(&mut g, 0, kavu, 1, None);
    assert_eq!(g.computed_permanent(kavu).unwrap().colors, vec![Color::White]);
}

/// Confound only answers a spell aimed at a creature.
#[test]
fn confound_counters_a_creature_targeting_spell() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let confound = g.add_card_to_hand(0, catalog::confound());
    cast(&mut g, 0, confound, Some(Target::Permanent(bolt)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "countered");
    assert!(g.battlefield.iter().any(|c| c.id == bears), "the Bears lived");
}

/// Crosis's Charm's second mode kills a nonblack creature outright.
#[test]
fn crosiss_charm_mode_one_kills() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let charm = g.add_card_to_hand(0, catalog::crosiss_charm());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: charm,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != bears));
}

/// Darigaaz's Charm burns for three on its middle mode.
#[test]
fn darigaazs_charm_burns_for_three() {
    let mut g = main_phase();
    let charm = g.add_card_to_hand(0, catalog::darigaazs_charm());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: charm,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Daring Leap hands over two keywords and a pump.
#[test]
fn daring_leap_grants_flying_and_first_strike() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let leap = g.add_card_to_hand(0, catalog::daring_leap());
    cast(&mut g, 0, leap, Some(Target::Permanent(bears)));
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Flying) && cp.keywords.contains(&Keyword::FirstStrike));
}

/// Dark Suspicions bills the difference in hand sizes.
#[test]
fn dark_suspicions_taxes_the_bigger_hand() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::dark_suspicions());
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "4 cards vs 1");
}

/// Deadapult turns a Zombie into two damage.
#[test]
fn deadapult_fires_a_zombie() {
    let mut g = main_phase();
    let catapult = g.add_card_to_battlefield(0, catalog::deadapult());
    g.add_card_to_battlefield(0, catalog::zombie_boa());
    activate(&mut g, 0, catapult, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 18);
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Zombie Boa"));
}

/// Death Bomb eats a creature and drains the victim's controller.
#[test]
fn death_bomb_kills_and_drains() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bomb = g.add_card_to_hand(0, catalog::death_bomb());
    cast(&mut g, 0, bomb, Some(Target::Permanent(victim)));
    assert!(g.battlefield.iter().all(|c| c.id != victim));
    assert_eq!(g.players[1].life, 18);
}

/// Destructive Flow eats a nonbasic each upkeep.
#[test]
fn destructive_flow_eats_a_nonbasic() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::destructive_flow());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::secluded_steppe());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().all(|c| c.definition.name != "Secluded Steppe"),
        "the nonbasic went"
    );
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Forest"), "the basic stayed");
}

/// Dominaria's Judgment hands out protection per basic type you control.
#[test]
fn dominarias_judgment_grants_domain_protection() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::mountain());
    g.add_card_to_battlefield(0, catalog::island());
    let judgment = g.add_card_to_hand(0, catalog::dominarias_judgment());
    cast(&mut g, 0, judgment, None);
    let kws = g.computed_permanent(bears).unwrap().keywords;
    assert!(kws.contains(&Keyword::Protection(Color::Red)));
    assert!(kws.contains(&Keyword::Protection(Color::Blue)));
    assert!(!kws.contains(&Keyword::Protection(Color::Green)), "no Forest, no pro-green");
}
