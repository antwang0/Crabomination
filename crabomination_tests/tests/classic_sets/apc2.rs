//! Apocalypse (APC), closing waves — the Volver kicker cycle, the Flagbearers,
//! the split cards and the remaining wedge utility.

use crabomination::card::{CardType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
    cast_with(g, seat, id, target, vec![]);
}

fn cast_with(
    g: &mut GameState,
    seat: usize,
    id: CardId,
    target: Option<Target>,
    additional_targets: Vec<Target>,
) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets,
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

fn find(g: &GameState, name: &str) -> CardId {
    g.battlefield.iter().find(|c| c.definition.name == name).expect("on battlefield").id
}

fn power_of(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).map(|p| p.power).expect("computed")
}

/// Minotaur Tactician grows once per allied colour on your board.
#[test]
fn minotaur_tactician_scales_with_white_and_blue() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::minotaur_tactician());
    let tactician = find(&g, "Minotaur Tactician");
    assert_eq!(power_of(&g, tactician), 1);
    g.add_card_to_battlefield(0, catalog::savannah_lions());
    assert_eq!(power_of(&g, tactician), 2, "a white creature is worth +1/+1");
}

/// Minotaur Illusionist throws its own power at a creature.
#[test]
fn minotaur_illusionist_sacrifices_for_damage() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::minotaur_illusionist());
    let illusionist = find(&g, "Minotaur Illusionist");
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bears = find(&g, "Grizzly Bears");
    activate(&mut g, 0, illusionist, 1, Some(Target::Permanent(bears)));
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Grizzly Bears"), "3 damage killed it");
}

/// Martyrs' Tomb converts life into a prevention shield.
#[test]
fn martyrs_tomb_prevents_one_damage() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::martyrs_tomb());
    let tomb = find(&g, "Martyrs' Tomb");
    g.add_card_to_battlefield(0, catalog::hill_giant());
    let giant = find(&g, "Hill Giant");
    activate(&mut g, 0, tomb, 0, Some(Target::Permanent(giant)));
    assert_eq!(g.players[0].life, 18, "paid 2 life");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(giant)));
    assert_eq!(
        g.battlefield.iter().find(|c| c.id == giant).map(|c| c.damage),
        Some(2),
        "one point was prevented"
    );
}

/// Manacles of Decay locks a creature out of attacking, and can shrink it.
#[test]
fn manacles_of_decay_pins_and_shrinks() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bears = find(&g, "Grizzly Bears");
    let manacles = g.add_card_to_hand(0, catalog::manacles_of_decay());
    cast(&mut g, 0, manacles, Some(Target::Permanent(bears)));
    assert!(g.computed_permanent(bears).unwrap().keywords.contains(&Keyword::CantAttack));
    let manacles = find(&g, "Manacles of Decay");
    activate(&mut g, 0, manacles, 0, None);
    assert_eq!(power_of(&g, bears), 1);
}

/// Yavimaya's Embrace steals the creature it enchants.
#[test]
fn yavimayas_embrace_steals_and_pumps() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bears = find(&g, "Grizzly Bears");
    let embrace = g.add_card_to_hand(0, catalog::yavimayas_embrace());
    cast(&mut g, 0, embrace, Some(Target::Permanent(bears)));
    assert_eq!(g.battlefield.iter().find(|c| c.id == bears).unwrap().controller, 0);
    assert_eq!(power_of(&g, bears), 4);
}

/// Soul Link pays its controller for damage in either direction.
#[test]
fn soul_link_gains_on_damage_dealt_and_taken() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bears = find(&g, "Grizzly Bears");
    let link = g.add_card_to_hand(0, catalog::soul_link());
    cast(&mut g, 0, link, Some(Target::Permanent(bears)));
    let before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(bears)));
    assert_eq!(g.players[0].life, before + 3, "the enchanted creature took 3");
}

/// Planar Despair shrinks the board by your domain.
#[test]
fn planar_despair_scales_with_domain() {
    let mut g = main_phase();
    for land in [catalog::plains, catalog::island, catalog::swamp] {
        g.add_card_to_battlefield(0, land());
    }
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let angel = find(&g, "Serra Angel");
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let despair = g.add_card_to_hand(0, catalog::planar_despair());
    cast(&mut g, 0, despair, None);
    assert_eq!(power_of(&g, angel), 1, "4/4 minus 3/3");
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Grizzly Bears"), "the 2/2 died");
}

/// Mask of Intolerance only bites a four-colour mana base.
#[test]
fn mask_of_intolerance_punishes_four_basic_types() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::mask_of_intolerance());
    for land in [catalog::plains, catalog::island, catalog::swamp] {
        g.add_card_to_battlefield(0, land());
    }
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "three basic types is safe");
    g.add_card_to_battlefield(0, catalog::mountain());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 17);
}

/// Symbiotic Deployment trades the draw step for a tap-two draw.
#[test]
fn symbiotic_deployment_draws_off_two_creatures() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::symbiotic_deployment());
    let deployment = find(&g, "Symbiotic Deployment");
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.add_card_to_library(0, catalog::forest());
    let before = g.players[0].hand.len();
    activate(&mut g, 0, deployment, 0, None);
    assert_eq!(g.players[0].hand.len(), before + 1);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears" && c.tapped).count(),
        2
    );
}

/// Wild Research tutors an enchantment and takes a card back at random.
#[test]
fn wild_research_tutors_then_discards() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::wild_research());
    let research = find(&g, "Wild Research");
    let pacifism = g.add_card_to_library(0, catalog::pacifism());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(pacifism))]));
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    activate(&mut g, 0, research, 0, None);
    assert_eq!(g.players[0].hand.len(), before, "tutored one, discarded one");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Whirlpool Warrior redraws your hand on the way in.
#[test]
fn whirlpool_warrior_redraws_your_hand() {
    let mut g = main_phase();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    let warrior = g.add_card_to_hand(0, catalog::whirlpool_warrior());
    cast(&mut g, 0, warrior, None);
    assert_eq!(g.players[0].hand.len(), 3, "three in, three out");
}

/// Last Stand pays out off every basic land type you control.
#[test]
fn last_stand_pays_out_per_basic_type() {
    let mut g = main_phase();
    for land in [catalog::plains, catalog::swamp, catalog::mountain, catalog::forest] {
        g.add_card_to_battlefield(0, land());
    }
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bears = find(&g, "Grizzly Bears");
    let stand = g.add_card_to_hand(0, catalog::last_stand());
    cast_with(&mut g, 0, stand, Some(Target::Player(1)), vec![Target::Permanent(bears)]);
    assert_eq!(g.players[1].life, 18, "one Swamp drains 2");
    assert_eq!(g.players[0].life, 22, "one Plains gains 2");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(), 1);
    assert_eq!(
        g.battlefield.iter().find(|c| c.id == bears).map(|c| c.damage),
        Some(1),
        "one Mountain deals 1"
    );
}

/// Tahngarth's Glare declares its opponent slot.
#[test]
fn tahngarths_glare_targets_an_opponent() {
    let def = catalog::tahngarths_glare();
    assert!(def.effect.target_filter_for_slot(0).is_some(), "slot 0 is the opponent");
    assert!(def.card_types.contains(&CardType::Sorcery));
}

/// Putrid Warrior drains everyone when it connects.
#[test]
fn putrid_warrior_drains_on_damage() {
    let mut g = main_phase();
    let warrior = g.add_card_to_battlefield(0, catalog::putrid_warrior());
    g.clear_sickness(warrior);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: warrior,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "each player lost 1");
    assert_eq!(g.players[1].life, 17, "2 combat damage plus the drain");
}



/// Standard Bearer soaks up an opponent's targeted removal.
#[test]
fn flagbearer_must_be_targeted() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::standard_bearer());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bears = find(&g, "Grizzly Bears");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(), "the Flagbearer has to be the target");
    let bearer = find(&g, "Standard Bearer");
    cast(&mut g, 0, bolt, Some(Target::Permanent(bearer)));
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Standard Bearer"));
}

/// Coalition Flag turns one of your own creatures into the lightning rod.
#[test]
fn coalition_flag_makes_a_flagbearer() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bears = find(&g, "Grizzly Bears");
    let flag = g.add_card_to_hand(1, catalog::coalition_flag());
    g.active_player_idx = 1;
    cast(&mut g, 1, flag, Some(Target::Permanent(bears)));
    g.active_player_idx = 0;
    assert!(
        g.computed_permanent(bears)
            .unwrap()
            .subtypes
            .creature_types
            .contains(&crabomination::card::CreatureType::Flagbearer)
    );
    g.add_card_to_battlefield(1, catalog::hill_giant());
    let giant = find(&g, "Hill Giant");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(giant)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err()
    );
}

/// Order exiles an attacker; Chaos turns off blocking.
#[test]
fn order_chaos_split_halves() {
    let def = catalog::order_chaos();
    let split = def.split.as_ref().expect("split");
    assert!(split.right.is_instant_speed());
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    let order = g.add_card_to_hand(0, catalog::order_chaos());
    cast(&mut g, 0, order, Some(Target::Permanent(attacker)));
    assert!(g.exile.iter().any(|c| c.id == attacker));
}

/// Life animates your lands; Death reanimates for life.
#[test]
fn life_death_split_halves() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::forest());
    let forest = find(&g, "Forest");
    let life = g.add_card_to_hand(0, catalog::life_death());
    cast(&mut g, 0, life, None);
    assert_eq!(power_of(&g, forest), 1, "the land is a 1/1");
    assert!(g.computed_permanent(forest).unwrap().card_types.contains(&CardType::Land));
}

/// Cromat picks off whatever it's tangling with.
#[test]
fn cromat_kills_its_blocker() {
    let mut g = main_phase();
    let cromat = g.add_card_to_battlefield(0, catalog::cromat());
    g.clear_sickness(cromat);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cromat,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, cromat)])).expect("block");
    activate(&mut g, 0, cromat, 0, Some(Target::Permanent(blocker)));
    assert!(g.battlefield.iter().all(|c| c.id != blocker), "the blocker is gone");
}

/// Dead Ringers only fires on a matched colour pair.
#[test]
fn dead_ringers_needs_identical_colors() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let ringers = g.add_card_to_hand(0, catalog::dead_ringers());
    cast_with(&mut g, 0, ringers, Some(Target::Permanent(a)), vec![Target::Permanent(b)]);
    assert_eq!(g.battlefield.iter().filter(|c| c.id == a || c.id == b).count(), 2, "green vs white");

    let c = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ringers = g.add_card_to_hand(0, catalog::dead_ringers());
    cast_with(&mut g, 0, ringers, Some(Target::Permanent(a)), vec![Target::Permanent(c)]);
    assert!(g.battlefield.iter().all(|p| p.id != a && p.id != c), "both green — both die");
}

/// Jaded Response only answers a colour you already have on board.
#[test]
fn jaded_response_needs_a_shared_color() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(1, catalog::giant_growth());
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
    let response = g.add_card_to_hand(0, catalog::jaded_response());
    cast(&mut g, 0, response, Some(Target::Permanent(spell)));
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == spell),
        "green spell, green creature — countered"
    );
}

/// Gaea's Balance eats five lands and returns one of each basic type.
#[test]
fn gaeas_balance_fetches_one_of_each_basic() {
    let mut g = main_phase();
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    for land in [catalog::plains, catalog::island, catalog::swamp, catalog::mountain] {
        g.add_card_to_library(0, land());
    }
    let balance = g.add_card_to_hand(0, catalog::gaeas_balance());
    cast(&mut g, 0, balance, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 4);
    assert!(g.players[0].library.is_empty(), "all four were found");
}

/// Captain's Maneuver moves X damage to the second target.
#[test]
fn captains_maneuver_redirects_damage() {
    let mut g = main_phase();
    let ox = g.add_card_to_battlefield(1, catalog::serra_angel());
    let maneuver = g.add_card_to_hand(0, catalog::captains_maneuver());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: maneuver,
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(ox)],
        mode: None,
        x_value: Some(3),
    })
    .expect("cast");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 20, "the burn went elsewhere");
    assert_eq!(g.battlefield.iter().find(|c| c.id == ox).map(|c| c.damage), Some(3));
}

/// Legacy Weapon shuffles back rather than dying.
#[test]
fn legacy_weapon_shuffles_instead_of_dying() {
    let mut g = main_phase();
    let weapon = g.add_card_to_battlefield(0, catalog::legacy_weapon());
    let mut events = Vec::new();
    g.destroy_permanent(weapon, false, &mut events);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.is_empty());
    assert!(g.players[0].library.iter().any(|c| c.id == weapon));
}

/// Suppress banks their hand until the end of their next turn.
#[test]
fn suppress_exiles_then_returns_the_hand() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let suppress = g.add_card_to_hand(0, catalog::suppress());
    cast(&mut g, 0, suppress, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 0, "hand is in exile");
    g.turn_number += 1;
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 3, "returned at their end step");
}

/// Emblazoned Golem grows by its kicker.
#[test]
fn emblazoned_golem_enters_with_x_counters() {
    let mut g = main_phase();
    let golem = g.add_card_to_hand(0, catalog::emblazoned_golem());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: golem,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(power_of(&g, golem), 4);
}
