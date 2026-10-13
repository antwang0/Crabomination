//! Commander: two-card gaps of the top-2000 commanders (`decks::cmdr_rank2k_k`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: more, mode: None, x_value: None });
}

fn named_for(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// Descent of the Dragons replaces each destroyed creature with a Dragon for
/// its controller.
#[test]
fn descent_of_the_dragons_pays_each_controller() {
    let mut g = pod(3);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    let descent = g.add_card_to_hand(0, catalog::descent_of_the_dragons());
    cast(&mut g, descent, Some(Target::Permanent(a)), vec![Target::Permanent(b), Target::Permanent(mine)]);
    for seat in 0..3 {
        assert_eq!(named_for(&g, seat, "Dragon"), 1, "seat {seat}");
    }
}

/// Down in the Valley II turns on landfall Elves; III pumps the Elves.
#[test]
fn down_in_the_valley_breeds_and_rallies_elves() {
    let mut g = pod(2);
    let def = catalog::down_in_the_valley();
    let host = ready(&mut g, 0, catalog::down_in_the_valley());
    let ctx = EffectContext::for_ability(host, 0, None);
    let chapter = |n: u32| def.saga_chapters.iter().find(|(c, _)| *c == n).unwrap().1.clone();
    g.resolve_effect(&chapter(2), &ctx).unwrap();
    let forest = g.add_card_to_hand(0, catalog::forest());
    act(&mut g, GameAction::PlayLand(forest));
    let elf = g.battlefield.iter().find(|c| c.definition.name == "Elf").expect("landfall Elf").id;
    g.resolve_effect(&chapter(3), &ctx).unwrap();
    assert_eq!(g.computed_permanent(elf).unwrap().power, 2);
}

/// Drizzt brings Guenhwyvar and grows by the power gap of a bigger death.
#[test]
fn drizzt_grows_off_a_bigger_death() {
    let mut g = pod(2);
    let drizzt = g.add_card_to_hand(0, catalog::drizzt_dourden());
    cast(&mut g, drizzt, None, vec![]);
    assert_eq!(named_for(&g, 0, "Guenhwyvar"), 1);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let blade = g.add_card_to_hand(0, catalog::doom_blade());
    cast(&mut g, blade, Some(Target::Permanent(angel)), vec![]);
    assert_eq!(g.battlefield_find(drizzt).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Icingdeath leaves Frost Tongue behind, which taps a defender's creature
/// when its bearer attacks.
#[test]
fn icingdeath_leaves_frost_tongue() {
    let mut g = pod(2);
    let dragon = ready(&mut g, 0, catalog::icingdeath_frost_tyrant());
    let blade = g.add_card_to_hand(0, catalog::doom_blade());
    cast(&mut g, blade, Some(Target::Permanent(dragon)), vec![]);
    let tongue = g.battlefield.iter().find(|c| c.definition.name == "Icingdeath, Frost Tongue").expect("token").id;
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let blocker = ready(&mut g, 1, catalog::grizzly_bears());
    act(&mut g, GameAction::Equip { equipment: tongue, target: bear });
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocker).unwrap().tapped);
}

/// Karrthus takes every Dragon, untaps them, and lends them haste.
#[test]
fn karrthus_rules_the_dragons() {
    let mut g = pod(2);
    let shivan = ready(&mut g, 1, catalog::shivan_dragon());
    g.battlefield_find_mut(shivan).unwrap().tapped = true;
    let karrthus = g.add_card_to_hand(0, catalog::karrthus_tyrant_of_jund());
    cast(&mut g, karrthus, None, vec![]);
    let s = g.battlefield_find(shivan).unwrap();
    assert_eq!(s.controller, 0);
    assert!(!s.tapped);
    assert!(g.computed_permanent(shivan).unwrap().keywords().contains(&crabomination::card::Keyword::Haste));
}

/// Thranduil gives other Elves a mana ability and makes Elves on landfall.
#[test]
fn thranduil_feeds_the_elves() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::thranduil_the_strategist());
    let warrior = ready(&mut g, 0, catalog::elvish_warrior());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: warrior,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("the granted {T}: Add {G}");
    assert!(g.battlefield_find(warrior).unwrap().tapped);
    let forest = g.add_card_to_hand(0, catalog::forest());
    act(&mut g, GameAction::PlayLand(forest));
    assert_eq!(named_for(&g, 0, "Elf"), 1);
}
