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
    assert_eq!(g.computed_permanent(kavu).unwrap().colors.to_vec(), vec![Color::White]);
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
    let kws = g.computed_permanent(bears).unwrap().keywords.clone();
    assert!(kws.contains(&Keyword::Protection(Color::Red)));
    assert!(kws.contains(&Keyword::Protection(Color::Blue)));
    assert!(!kws.contains(&Keyword::Protection(Color::Green)), "no Forest, no pro-green");
}

/// A Lair stays only if you bounce a real land.
#[test]
fn lair_bounces_a_land_to_stay() {
    let mut g = main_phase();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let lair = g.add_card_to_hand(0, catalog::dromars_cavern());
    g.perform_action(GameAction::PlayLand(lair)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == lair), "the Lair stayed");
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "the Forest bounced");
}

/// With no other land, the Lair sacrifices itself.
#[test]
fn lair_sacrifices_itself_with_nothing_to_bounce() {
    let mut g = main_phase();
    let lair = g.add_card_to_hand(0, catalog::crosiss_catacombs());
    g.perform_action(GameAction::PlayLand(lair)).expect("play");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == lair));
}

/// Ertai's Trickery only answers a kicked spell.
#[test]
fn ertais_trickery_only_counters_a_kicked_spell() {
    let mut g = main_phase();
    let angel = g.add_card_to_hand(1, catalog::desolation_angel());
    g.active_player_idx = 1;
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: angel,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let trickery = g.add_card_to_hand(0, catalog::ertais_trickery());
    cast(&mut g, 0, trickery, Some(Target::Permanent(angel)));
    assert!(g.battlefield.iter().any(|c| c.id == angel), "unkicked — it resolved");
}

/// Ertai eats an enchantment to counter.
#[test]
fn ertai_the_corrupted_counters_by_sacrificing() {
    let mut g = main_phase();
    let ertai = g.add_card_to_battlefield(0, catalog::ertai_the_corrupted());
    g.clear_sickness(ertai);
    g.add_card_to_battlefield(0, catalog::pacifism());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    activate(&mut g, 0, ertai, 0, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, 20, "the Bolt never resolved");
}

/// Exotic Disease drains by domain.
#[test]
fn exotic_disease_drains_by_domain() {
    let mut g = main_phase();
    for land in [catalog::plains, catalog::island, catalog::swamp, catalog::mountain] {
        g.add_card_to_battlefield(0, land());
    }
    let disease = g.add_card_to_hand(0, catalog::exotic_disease());
    cast(&mut g, 0, disease, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 16);
    assert_eq!(g.players[0].life, 24);
}

/// Gaea's Herald stops a counterspell aimed at a creature.
#[test]
fn gaeas_herald_protects_creature_spells() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::gaeas_herald());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    let _ = g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bears), "the creature resolved anyway");
}

/// Gaea's Might scales with your basic land types.
#[test]
fn gaeas_might_scales_with_domain() {
    let mut g = main_phase();
    for land in [catalog::forest, catalog::mountain] {
        g.add_card_to_battlefield(0, land());
    }
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let might = g.add_card_to_hand(0, catalog::gaeas_might());
    cast(&mut g, 0, might, Some(Target::Permanent(bears)));
    assert_eq!(g.computed_permanent(bears).unwrap().power, 4);
}

/// Gainsay only hits blue.
#[test]
fn gainsay_counters_a_blue_spell() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(1, catalog::counterspell());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let gainsay = g.add_card_to_hand(0, catalog::gainsay());
    cast(&mut g, 0, gainsay, Some(Target::Permanent(spell)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == spell));
}

/// Gerrard's Command untaps and pumps in one.
#[test]
fn gerrards_command_untaps_and_pumps() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bears).unwrap().tapped = true;
    let command = g.add_card_to_hand(0, catalog::gerrards_command());
    cast(&mut g, 0, command, Some(Target::Permanent(bears)));
    assert!(!g.battlefield.iter().find(|c| c.id == bears).unwrap().tapped);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 5);
}

/// Hobble pins a creature and cantrips.
#[test]
fn hobble_pins_and_draws() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let hobble = g.add_card_to_hand(0, catalog::hobble());
    let before = g.players[0].hand.len();
    cast(&mut g, 0, hobble, Some(Target::Permanent(bears)));
    assert_eq!(g.players[0].hand.len(), before, "aura out, card in");
    assert!(g.computed_permanent(bears).unwrap().keywords.contains(&Keyword::CantAttack));
}

/// Escape Routes rebuys a white creature.
#[test]
fn escape_routes_returns_a_white_creature() {
    let mut g = main_phase();
    let routes = g.add_card_to_battlefield(0, catalog::escape_routes());
    let lions = g.add_card_to_battlefield(0, catalog::savannah_lions());
    activate(&mut g, 0, routes, 0, Some(Target::Permanent(lions)));
    assert!(g.players[0].hand.iter().any(|c| c.id == lions));
}

/// Dromar's Charm's counter mode works.
#[test]
fn dromars_charm_counters() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let charm = g.add_card_to_hand(0, catalog::dromars_charm());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: charm,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20);
}

/// Fleetfoot Panther bounces one of your own on the way in.
#[test]
fn fleetfoot_panther_rebuys_a_green_creature() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let panther = g.add_card_to_hand(0, catalog::fleetfoot_panther());
    cast(&mut g, 0, panther, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears));
}

/// Dralnu's Crusade rebuilds every Goblin.
#[test]
fn dralnus_crusade_reshapes_goblins() {
    let mut g = main_phase();
    let goblin = g.add_card_to_battlefield(0, catalog::goblin_guide());
    g.add_card_to_battlefield(0, catalog::dralnus_crusade());
    let cp = g.computed_permanent(goblin).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert_eq!(cp.colors.to_vec(), vec![Color::Black]);
    assert!(cp.subtypes.creature_types.contains(&crabomination::card::CreatureType::Zombie));
}

/// Falling Timber fogs a second creature when kicked.
#[test]
fn falling_timber_kicked_fogs_two() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::forest());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    let timber = g.add_card_to_hand(0, catalog::falling_timber());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: timber,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "both attackers were fogged");
}

/// Honorable Scout pays two per black-or-red creature they have.
#[test]
fn honorable_scout_gains_per_black_or_red_creature() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::goblin_guide());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let scout = g.add_card_to_hand(0, catalog::honorable_scout());
    cast(&mut g, 0, scout, Some(Target::Player(1)));
    assert_eq!(g.players[0].life, 22, "one red creature");
}

/// Hunting Drake decks a red creature.
#[test]
fn hunting_drake_tucks_a_red_creature() {
    let mut g = main_phase();
    let goblin = g.add_card_to_battlefield(1, catalog::goblin_guide());
    let drake = g.add_card_to_hand(0, catalog::hunting_drake());
    cast(&mut g, 0, drake, Some(Target::Permanent(goblin)));
    assert!(g.players[1].library.iter().any(|c| c.id == goblin));
}

/// Insolence bites the controller whenever the creature taps.
#[test]
fn insolence_bites_on_tap() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bears);
    let insolence = g.add_card_to_hand(0, catalog::insolence());
    cast(&mut g, 0, insolence, Some(Target::Permanent(bears)));
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bears,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "tapping to attack cost them 2");
}

/// Kavu Recluse turns a land into a Forest.
#[test]
fn kavu_recluse_makes_a_forest() {
    let mut g = main_phase();
    let recluse = g.add_card_to_battlefield(0, catalog::kavu_recluse());
    g.clear_sickness(recluse);
    let island = g.add_card_to_battlefield(0, catalog::island());
    activate(&mut g, 0, recluse, 0, Some(Target::Permanent(island)));
    assert!(
        g.computed_permanent(island)
            .unwrap()
            .subtypes
            .land_types
            .contains(&crabomination::card::LandType::Forest)
    );
}

/// Keldon Mantle lends trample for {G}.
#[test]
fn keldon_mantle_grants_trample() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mantle = g.add_card_to_hand(0, catalog::keldon_mantle());
    cast(&mut g, 0, mantle, Some(Target::Permanent(bears)));
    let mantle = g.battlefield.iter().find(|c| c.definition.name == "Keldon Mantle").unwrap().id;
    activate(&mut g, 0, mantle, 2, None);
    assert!(g.computed_permanent(bears).unwrap().keywords.contains(&Keyword::Trample));
}

/// Maggot Carrier bleeds the whole table.
#[test]
fn maggot_carrier_drains_everyone() {
    let mut g = main_phase();
    let carrier = g.add_card_to_hand(0, catalog::maggot_carrier());
    cast(&mut g, 0, carrier, None);
    assert_eq!(g.players[0].life, 19);
    assert_eq!(g.players[1].life, 19);
}

/// Hull Breach's third mode gets both.
#[test]
fn hull_breach_both_mode() {
    let mut g = main_phase();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let pacifism = g.add_card_to_battlefield(1, catalog::pacifism());
    let breach = g.add_card_to_hand(0, catalog::hull_breach());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: breach,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![Target::Permanent(pacifism)],
        mode: Some(2),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != ring && c.id != pacifism));
}
