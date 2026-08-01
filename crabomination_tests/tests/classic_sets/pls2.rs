//! Planeshift (PLS) closure — Familiars, Domain payoffs and the bounce cycle.

use crabomination::card::Keyword;
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

/// The total cost reduction seat 0 gets on a fresh copy of `def`.
fn discount(g: &GameState, def: crabomination::card::CardDefinition) -> u32 {
    let card = crabomination::card::CardInstance::new(CardId(9999), def, 0);
    crabomination::game::actions::cost_reduction_for_spell(g, 0, &card, None)
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

/// Nightscape Familiar shaves {1} off blue and red spells but not white ones.
#[test]
fn nightscape_familiar_discounts_blue_and_red() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::nightscape_familiar());
    assert_eq!(discount(&g, catalog::gainsay()), 1, "{{1}}{{U}} → {{U}}");
    assert_eq!(discount(&g, catalog::aura_blast()), 0, "white is untouched");
}

/// Stratadon's Domain discount is {1} per basic land type.
#[test]
fn stratadon_costs_one_less_per_basic_type() {
    let mut g = main_phase();
    for land in [catalog::plains, catalog::island, catalog::swamp] {
        g.add_card_to_battlefield(0, land());
    }
    assert_eq!(discount(&g, catalog::stratadon()), 3, "{{1}} per basic type");
}

/// Draco's Domain discount is {2} per basic land type.
#[test]
fn draco_costs_two_less_per_basic_type() {
    let mut g = main_phase();
    for land in [catalog::plains, catalog::island, catalog::swamp, catalog::mountain] {
        g.add_card_to_battlefield(0, land());
    }
    assert_eq!(discount(&g, catalog::draco()), 8, "{{2}} per basic type");
}

/// Magnigoth Treefolk gains landwalk for each basic type its controller has.
#[test]
fn magnigoth_treefolk_walks_shared_basic_types() {
    let mut g = main_phase();
    let tree = g.add_card_to_battlefield(0, catalog::magnigoth_treefolk());
    g.clear_sickness(tree);
    g.add_card_to_battlefield(0, catalog::island());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: tree,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    // No Island on the defending side yet — the block is legal.
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(blocker, tree)])).is_ok());
    g.add_card_to_battlefield(1, catalog::island());
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, tree)])).is_err(),
        "islandwalk from the shared basic type"
    );
}

/// Lashknife Barrier shaves a point off damage to its controller's creatures.
#[test]
fn lashknife_barrier_shaves_one_damage() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::forest());
    let barrier = g.add_card_to_hand(0, catalog::lashknife_barrier());
    cast(&mut g, 0, barrier, None);
    let wurm = g.add_card_to_battlefield(0, catalog::shivan_wurm());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(wurm)));
    assert_eq!(g.battlefield_find(wurm).map(|c| c.damage), Some(2), "3 − 1");
}

/// Sawtooth Loon draws two, then buries two.
#[test]
fn sawtooth_loon_draws_two_and_bottoms_two() {
    let mut g = main_phase();
    let ally = g.add_card_to_battlefield(0, catalog::silver_drake());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let loon = g.add_card_to_hand(0, catalog::sawtooth_loon());
    let hand_before = g.players[0].hand.len();
    cast(&mut g, 0, loon, Some(Target::Permanent(ally)));
    // -1 Loon cast, +1 bounced Drake, +2 drawn, -2 bottomed.
    assert_eq!(g.players[0].hand.len(), hand_before, "net zero");
    assert_eq!(g.players[0].library.len(), 4, "two drawn, two returned");
}

/// Planeswalker's Mirth gains life equal to the revealed card's mana value.
#[test]
fn planeswalkers_mirth_gains_the_revealed_mana_value() {
    let mut g = main_phase();
    let mirth = g.add_card_to_battlefield(0, catalog::planeswalkers_mirth());
    g.add_card_to_hand(1, catalog::draco()); // the only card — MV 16
    let before = g.players[0].life;
    activate(&mut g, 0, mirth, 0, Some(Target::Player(1)));
    assert_eq!(g.players[0].life, before + 16);
}

/// Shifting Sky repaints every nonland permanent the chosen color.
#[test]
fn shifting_sky_repaints_nonlands() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sky = g.add_card_to_hand(0, catalog::shifting_sky());
    cast(&mut g, 0, sky, None);
    let colors = &g.computed_permanent(bears).unwrap().colors;
    assert_eq!(colors.len(), 1, "exactly the chosen color");
    assert!(!colors.contains(&Color::Green), "green was replaced");
}

/// Natural Emergence animates its controller's lands.
#[test]
fn natural_emergence_animates_your_lands() {
    let mut g = main_phase();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let theirs = g.add_card_to_battlefield(1, catalog::forest());
    g.add_card_to_battlefield(0, catalog::natural_emergence());
    let cp = g.computed_permanent(forest).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
    assert!(
        !g.computed_permanent(theirs)
            .unwrap()
            .card_types
            .contains(&crabomination::card::CardType::Creature),
        "only yours"
    );
}

/// Phyrexian Tyranny bills the drawing player two life when they don't pay.
#[test]
fn phyrexian_tyranny_taxes_draws() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::phyrexian_tyranny());
    g.add_card_to_library(1, catalog::forest());
    let before = g.players[1].life;
    let mut evs = Vec::new();
    g.draw_one(1, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 2, "no mana floated to pay {{2}}");
}

/// March of Souls wraths the board and hands back fliers.
#[test]
fn march_of_souls_replaces_creatures_with_spirits() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let march = g.add_card_to_hand(0, catalog::march_of_souls());
    cast(&mut g, 0, march, None);
    let spirits = |seat| {
        g.battlefield
            .iter()
            .filter(|c| c.controller == seat && c.definition.name == "Spirit")
            .count()
    };
    assert_eq!((spirits(0), spirits(1)), (1, 2));
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Grizzly Bears"));
}

/// Mire Kavu is bigger beside a Swamp.
#[test]
fn mire_kavu_grows_with_a_swamp() {
    let mut g = main_phase();
    let kavu = g.add_card_to_battlefield(0, catalog::mire_kavu());
    assert_eq!(g.computed_permanent(kavu).unwrap().power, 3);
    g.add_card_to_battlefield(0, catalog::swamp());
    assert_eq!(g.computed_permanent(kavu).unwrap().power, 4);
}

/// Mogg Sentry swells on an opponent's spell.
#[test]
fn mogg_sentry_swells_on_opponent_spells() {
    let mut g = main_phase();
    let sentry = g.add_card_to_battlefield(0, catalog::mogg_sentry());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.computed_permanent(sentry).unwrap().power, 3);
}

/// Nemata trades a Saproling for a Saproling anthem.
#[test]
fn nemata_makes_and_eats_saprolings() {
    let mut g = main_phase();
    let nemata = g.add_card_to_battlefield(0, catalog::nemata_grove_guardian());
    activate(&mut g, 0, nemata, 0, None);
    activate(&mut g, 0, nemata, 0, None);
    let saps: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Saproling")
        .map(|c| c.id)
        .collect();
    assert_eq!(saps.len(), 2);
    activate(&mut g, 0, nemata, 1, None);
    let left = g.battlefield.iter().find(|c| c.definition.name == "Saproling").unwrap().id;
    assert_eq!(g.computed_permanent(left).unwrap().power, 2, "+1/+1 until end of turn");
}

/// Warped Devotion turns every bounce into a discard.
#[test]
fn warped_devotion_punishes_bounces() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::warped_devotion());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::forest());
    let bounce = g.add_card_to_hand(0, catalog::rushing_river());
    cast(&mut g, 0, bounce, Some(Target::Permanent(bears)));
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Forest"));
}

/// Phyrexian Scuta's kicker is three life for two counters.
#[test]
fn phyrexian_scuta_kicked_by_paying_life() {
    let mut g = main_phase();
    let scuta = g.add_card_to_hand(0, catalog::phyrexian_scuta());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: scuta,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 17);
    assert_eq!(g.computed_permanent(scuta).unwrap().power, 5);
}

/// Rushing River bounces a second permanent when kicked.
#[test]
fn rushing_river_kicked_bounces_two() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::forest());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::pacifism());
    let river = g.add_card_to_hand(0, catalog::rushing_river());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: river,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != a && c.id != b));
}

/// Voice of All is untouchable in the color it named.
#[test]
fn voice_of_all_has_protection_from_the_chosen_color() {
    let mut g = main_phase();
    let hand = g.add_card_to_hand(0, catalog::voice_of_all());
    cast(&mut g, 0, hand, None);
    let voice = g.battlefield.iter().find(|c| c.definition.name == "Voice of All").unwrap().id;
    let color = g.battlefield_find(voice).unwrap().chosen_color.expect("a color was chosen");
    assert!(
        g.computed_permanent(voice)
            .unwrap()
            .keywords
            .contains(&Keyword::Protection(color))
    );
}

/// Mogg Jailer stays home against a small untapped blocker.
#[test]
fn mogg_jailer_cant_attack_into_small_blockers() {
    let mut g = main_phase();
    let jailer = g.add_card_to_battlefield(0, catalog::mogg_jailer());
    g.clear_sickness(jailer);
    let wall = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: jailer,
            target: AttackTarget::Player(1),
        }]))
        .is_err()
    );
    g.battlefield.iter_mut().find(|c| c.id == wall).unwrap().tapped = true;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: jailer,
            target: AttackTarget::Player(1),
        }]))
        .is_ok()
    );
}

/// Terminal Moraine cracks for a basic.
#[test]
fn terminal_moraine_fetches_a_basic() {
    let mut g = main_phase();
    let moraine = g.add_card_to_battlefield(0, catalog::terminal_moraine());
    let forest_id = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(forest_id))]));
    activate(&mut g, 0, moraine, 1, None);
    let forest = g.battlefield_find(forest_id).expect("fetched");
    assert!(forest.tapped);
    assert!(g.battlefield.iter().all(|c| c.id != moraine), "sacrificed");
}

/// Sunken Hope bounces a creature at every upkeep.
#[test]
fn sunken_hope_bounces_each_upkeep() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::sunken_hope());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != bears));
}
