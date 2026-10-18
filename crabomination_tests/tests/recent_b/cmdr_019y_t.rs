//! Commander: the 2026-10-18 (`019Y6J4R`) twentieth closer batch
//! (`decks::cmdr_019y_t`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn activate(g: &mut GameState, card_id: CardId, ability_index: usize, target: Option<Target>) -> bool {
    let ok = g
        .perform_action(GameAction::ActivateAbility {
            card_id,
            ability_index,
            target,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_ok();
    drain_stack(g);
    ok
}

fn minus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::MinusOneMinusOne))
}

/// CR 701.68 — Blight N as an activation cost: the -1/-1 counters land on a
/// creature you control that survives them (the 3/3 giant, not the 1/2
/// Dissident, for blight 2). The exiled card is linked to the Dissident, and
/// on your turn it's castable for its cost plus three counters removed from
/// among your creatures (-1/-1 counters first).
#[test]
fn dawnhand_dissident_blights_and_recasts_its_exiles() {
    let mut g = pod(2);
    let dd = ready(&mut g, 0, catalog::dawnhand_dissident());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    assert!(activate(&mut g, dd, 1, Some(Target::Permanent(bear))), "{{T}}, Blight 2");
    assert!(g.exile.iter().any(|c| c.id == bear && c.exiled_with == Some(dd)), "linked exile");
    assert_eq!((minus(&g, giant), minus(&g, dd)), (2, 0));

    g.players[0].mana_pool.add(Color::Green, 2);
    let cast = |g: &mut GameState| {
        g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
    };
    assert!(cast(&mut g).is_err(), "only two counters to remove");
    let elf = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(elf).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.active_player_idx = 1;
    assert!(!g.may_play_castable(0).contains(&bear), "not on an opponent's turn");
    g.active_player_idx = 0;
    assert!(g.may_play_castable(0).contains(&bear), "offered to the caster");
    cast(&mut g).expect("cast from linked exile");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "the creature resolved");
    assert_eq!(minus(&g, giant), 0, "the -1/-1 counters went first");
    assert_eq!(g.battlefield_find(elf).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.players[0].mana_pool.total(), 0, "its mana cost was paid");
}

/// CR 701.68 — blight 1 alone: the Dissident is the only creature and
/// survives as a 0/1 after surveilling.
#[test]
fn dawnhand_dissident_blights_itself_to_surveil() {
    let mut g = pod(2);
    let dd = ready(&mut g, 0, catalog::dawnhand_dissident());
    assert!(activate(&mut g, dd, 0, None), "{{T}}, Blight 1");
    assert_eq!(minus(&g, dd), 1);
}

/// CR 903.4 / 106.1 — Cryptic Spires makes only its circled colors, which
/// for a two-color Commander deck are its commander's identity (Siani and
/// Yoshimaru: white and blue); it enters tapped, and it is colorless for
/// identity (no colored symbol in its text).
#[test]
fn cryptic_spires_taps_for_the_circled_colors() {
    use crabomination::mana::Color;
    let mut g = pod(2);
    let cmdr = g.add_card_to_battlefield(0, catalog::hanna_ships_navigator());
    g.players[0].commanders.push(cmdr);
    g.players[0].commander_identity = [Color::White, Color::Blue].into_iter().collect();
    let spires = g.add_card_to_battlefield(0, catalog::cryptic_spires());
    let identity = crabomination::color_identity::color_identity(&catalog::cryptic_spires());
    assert!(identity.is_empty(), "colorless identity");
    g.battlefield_find_mut(spires).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: spires, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap for mana");
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.total(), 1);
    assert_eq!(pool.amount(Color::White) + pool.amount(Color::Blue), 1, "white or blue only");
}

/// CR 707.9b — Absorbing Man's first-main-phase copy keeps his exceptions:
/// copying Sol Ring he's named Absorbing Man, a legendary 4/4 Human Villain
/// artifact creature with vigilance and Sol Ring's mana ability.
#[test]
fn absorbing_man_copies_an_artifact_with_his_exceptions() {
    use crabomination::card::{CardType, CreatureType, Keyword, Supertype};
    let mut g = pod(2);
    let man = ready(&mut g, 0, catalog::absorbing_man());
    ready(&mut g, 0, catalog::sol_ring());
    g.step = TurnStep::Draw;
    while g.step != TurnStep::PreCombatMain && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    let c = g.battlefield_find(man).expect("still here");
    assert_eq!(c.definition.name, "Absorbing Man");
    assert!(c.definition.supertypes.contains(&Supertype::Legendary));
    assert!(c.definition.card_types.contains(&CardType::Artifact), "a copy of Sol Ring");
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Villain));
    assert!(c.definition.keywords.contains(&Keyword::Vigilance));
    assert!(!c.definition.activated_abilities.is_empty(), "Sol Ring's mana ability");
    let cp = g.computed_permanent(man).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.card_types().contains(&CardType::Creature));
}

/// Hurkyl — one card per type among noncreature spells cast this turn: an
/// instant and an artifact cast take one Instant and one Artifact card from
/// the top five (not a second instant, not the creature); the rest go to the
/// bottom.
#[test]
fn hurkyl_takes_one_card_per_cast_type() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::hurkyl_master_wizard());
    g.players[0].library.clear();
    for def in [catalog::grizzly_bears(), catalog::lightning_bolt(), catalog::sol_ring(), catalog::shock(), catalog::forest()] {
        g.add_card_to_library(0, def);
    }
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("ring");
    drain_stack(&mut g);
    let hand_before = g.players[0].hand.len();
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    let names: Vec<&str> = g.players[0].hand.iter().skip(hand_before).map(|c| c.definition.name).collect();
    assert_eq!(names.len(), 2, "one instant and one artifact: {names:?}");
    assert!(names.contains(&"Sol Ring"));
    assert!(names.contains(&"Lightning Bolt") || names.contains(&"Shock"));
    assert_eq!(g.players[0].library.len(), 8);
    assert_eq!(g.players[0].library[0].definition.name, "Island", "the other three went to the bottom");
}
