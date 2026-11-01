//! Star Trek: the fifth batch of EDHREC closers (`decks::trek_singles5`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// Fleet Captain Christopher Pike: "No" taps exactly two other creatures,
/// "Yes" untaps two; Pike can't name himself.
#[test]
fn pike_taps_and_untaps_two_others() {
    let mut g = pod(3);
    let pike = ready(&mut g, 0, catalog::fleet_captain_christopher_pike());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::hill_giant());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pike,
        ability_index: 1,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        x_value: None,
        mode: None,
    })
    .expect("tap two");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
    g.battlefield_find_mut(pike).unwrap().tapped = false;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pike,
        ability_index: 0,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        x_value: None,
        mode: None,
    })
    .expect("untap two");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(a).unwrap().tapped && !g.battlefield_find(b).unwrap().tapped);
}

/// Sam Kirk: an opponent's creature spell investigates; your own doesn't.
#[test]
fn sam_kirk_investigates_on_opponent_creature_spells() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::sam_kirk_xenoanthropologist());
    let clues = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Clue").count();
    let mine = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    cast(&mut g, mine, None);
    assert_eq!(clues(&g), 0);
    let theirs = g.add_card_to_hand(2, catalog::grizzly_bears());
    flood(&mut g, 2);
    g.active_player_idx = 2;
    g.priority.player_with_priority = 2;
    cast(&mut g, theirs, None);
    assert_eq!(clues(&g), 1);
}

/// Out of the Way: {2} less aimed at a green permanent; bounces and draws.
#[test]
fn out_of_the_way_discounts_green_targets() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::out_of_the_way());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bounced for {{1}}{{U}}");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
}

/// Lt. Arex (CR 604.3): */* counts differently named lands — two Forests and
/// an Island make a 2/2; entering draws three and may drop lands tapped.
#[test]
fn arex_counts_distinct_land_names() {
    let mut g = pod(2);
    for def in [catalog::forest(), catalog::forest(), catalog::island()] {
        g.add_card_to_battlefield(0, def);
    }
    g.add_card_to_hand(0, catalog::mountain());
    let arex = g.add_card_to_hand(0, catalog::lt_arex_ever_adapting());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([]));
    cast(&mut g, arex, None);
    let lands = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    let names = if lands == 4 { 3 } else { 2 };
    let cp = g.computed_permanent(arex).unwrap();
    assert_eq!((cp.power, cp.toughness), (names, names));
}

/// Warp Speed: lands enter untapped (a tapland included); the sacrifice costs
/// {X} less, X the differently named lands you control.
#[test]
fn warp_speed_untaps_lands_and_discounts_its_draw() {
    let mut g = pod(2);
    let ws = ready(&mut g, 0, catalog::warp_speed());
    let tl = g.add_card_to_hand(0, catalog::dismal_backwater());
    g.perform_action(GameAction::PlayLand(tl)).expect("land");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(tl).unwrap().tapped);
    for def in [catalog::forest(), catalog::plains(), catalog::swamp(), catalog::mountain(), catalog::island()] {
        g.add_card_to_battlefield(0, def);
    }
    // Six names: {7}{U}{U} costs {1}{U}{U}.
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ws,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("discounted");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Vash: cards leaving your graveyard draw once a turn, however many leave.
#[test]
fn vash_draws_once_a_turn_off_graveyard_exits() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::vash_freelance_archaeologist());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let hand = g.players[0].hand.len();
    for _ in 0..2 {
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let fx = crabomination::effect::Effect::ExileFromGraveyard {
            who: crabomination::effect::PlayerRef::You,
            count: crabomination::card::Value::ONE,
            filter: crabomination::card::SelectionRequirement::Any,
        };
        let ev = g.resolve_effect(&fx, &ctx).expect("exile");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Blastoneuron: a hasty token copy of a small graveyard creature, sacrificed
/// at the end step; the exile drains each opponent.
#[test]
fn blastoneuron_reanimates_a_copy_and_drains() {
    let mut g = pod(3);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bn = g.add_card_to_hand(0, catalog::blastoneuron());
    flood(&mut g, 0);
    cast(&mut g, bn, None);
    assert!(g.exile.iter().any(|c| c.id == bear));
    let token = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Grizzly Bears").map(|c| c.id).expect("copy");
    assert!(g.computed_permanent(token).unwrap().keywords().contains(&crabomination::card::Keyword::Haste));
    assert_eq!((g.players[1].life, g.players[2].life, g.players[0].life), (19, 19, 21));
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(token).is_none(), "sacrificed at the end step");
}

/// Sylvia Tilly (CR 603.12): at your combat, paying {1} and exiling a graveyard
/// card puts a +1/+1 counter on a creature; an empty graveyard offers nothing.
#[test]
fn tilly_pays_and_exiles_for_a_counter() {
    let mut g = pod(2);
    let tilly = ready(&mut g, 0, catalog::sylvia_tilly_joy_of_discovery());
    let card = g.add_card_to_graveyard(0, catalog::island());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == card));
    assert_eq!(g.battlefield_find(tilly).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}
