//! Functionality tests for `catalog::sets::decks::recent215`.

use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::game::effects::{EffectContext, EntityRef};
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    let mut guard = 0;
    while g.step != step && guard < 60 {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
        guard += 1;
    }
}

/// Mild-Mannered Librarian's once-per-game ability turns it into a 3/3 Werewolf
/// and draws a card. A second activation is rejected.
#[test]
fn mild_mannered_librarian_transforms_once() {
    let mut g = two_player_game();
    let lib = g.add_card_to_battlefield(0, catalog::mild_mannered_librarian());
    g.add_card_to_library(0, catalog::forest());
    g.clear_sickness(lib);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: lib, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("transform");
    drain_stack(&mut g);
    let v = g.computed_permanent(lib).unwrap();
    assert_eq!((v.power, v.toughness), (3, 3), "1/1 + two counters");
    assert!(v.subtypes.creature_types.contains(&CreatureType::Werewolf), "became a Werewolf");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: lib, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "activate only once");
}

/// Mazemind Tome's fourth page counter exiles it and gains 4 life.
#[test]
fn mazemind_tome_cashes_out_at_four_pages() {
    let mut g = two_player_game();
    let tome = g.add_card_to_battlefield(0, catalog::mazemind_tome());
    g.add_card_to_library(0, catalog::forest());
    g.battlefield_find_mut(tome).unwrap().counters.insert(CounterType::Page, 3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life_before = g.players[0].life;
    // The scry ability adds the fourth page counter → exile + gain 4 life.
    g.perform_action(GameAction::ActivateAbility {
        card_id: tome, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("scry ability");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == tome), "exiled at 4 page counters");
    assert_eq!(g.players[0].life, life_before + 4, "gained 4 life");
}

/// Extravagant Replication clones a target nonland permanent at upkeep.
#[test]
fn extravagant_replication_copies_at_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::extravagant_replication());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    let bears = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "original bear + one token copy");
}

/// Lathril mints one Elf Warrior per point of combat damage dealt to a player.
#[test]
fn lathril_mints_elves_on_combat_damage() {
    let mut g = two_player_game();
    let lathril = g.add_card_to_battlefield(0, catalog::lathril_blade_of_the_elves());
    let effect = catalog::lathril_blade_of_the_elves().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { event_amount: 2, ..EffectContext::for_trigger(lathril, 0, None, 0) };
    g.resolve_effect(&effect, &ctx).unwrap();
    let elves = g.battlefield.iter().filter(|c| c.definition.name == "Elf Warrior").count();
    assert_eq!(elves, 2, "2 combat damage → 2 Elf Warriors");
}

/// Lathril's tap-ten-Elves ability drains each opponent for 10.
#[test]
fn lathril_drains_ten_by_tapping_elves() {
    let mut g = two_player_game();
    let lathril = g.add_card_to_battlefield(0, catalog::lathril_blade_of_the_elves());
    for _ in 0..10 { g.add_card_to_battlefield(0, catalog::llanowar_elves()); }
    g.clear_sickness(lathril);
    for c in g.battlefield.iter_mut() { c.summoning_sick = false; }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: lathril, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap ten elves");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 10, "opponent lost 10");
    assert_eq!(g.players[0].life, 30, "you gained 10");
}

/// Ayli gains life equal to a sacrificed creature's toughness.
#[test]
fn ayli_gains_life_on_sacrifice() {
    let mut g = two_player_game();
    let ayli = g.add_card_to_battlefield(0, catalog::ayli_eternal_pilgrim());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 fodder
    g.clear_sickness(ayli);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ayli, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sacrifice for life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained 2 (bear toughness)");
}

/// Kykar makes a 1/1 white flying Spirit when a noncreature spell is cast
/// (choosing the token mode).
#[test]
fn kykar_makes_spirit_on_noncreature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kykar_zephyr_awakener());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    // The mode is picked synchronously at trigger push (mode 0 has a target);
    // choose mode 1 (create Spirit).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits, 1, "one Spirit token");
    let sp = g.battlefield.iter().find(|c| c.definition.name == "Spirit").unwrap();
    assert!(sp.definition.keywords.contains(&Keyword::Flying), "flying Spirit");
}

/// Alesha grows on attack and, thanks to Raid, reanimates a small creature at
/// her end step.
#[test]
fn alesha_reanimates_at_end_step() {
    let mut g = two_player_game();
    let alesha = g.add_card_to_battlefield(0, catalog::alesha_who_laughs_at_fate());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 ≤ power 3 after attack
    g.clear_sickness(alesha);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: alesha, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(alesha).unwrap().power, 3, "grew to 3 power on attack");
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "reanimated the bear at end step");
}

/// Garna draws when an attacking creature you control dies, and pings each
/// opponent when a non-attacker dies.
#[test]
fn garna_draws_or_pings_on_death() {
    let mut g = two_player_game();
    let garna = g.add_card_to_battlefield(0, catalog::garna_bloodfist_of_keld());
    g.add_card_to_library(0, catalog::forest());
    let effect = catalog::garna_bloodfist_of_keld().triggered_abilities[0].effect.clone();
    // Non-attacker death → ping each opponent. (Dying creature is left on the
    // battlefield here so its `attacked_this_turn` is readable as trigger source.)
    let foe_life = g.players[1].life;
    let non_attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext {
        trigger_source: Some(EntityRef::Permanent(non_attacker)),
        ..EffectContext::for_trigger(garna, 0, None, 0)
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[1].life, foe_life - 1, "non-attacker death pings opponent");
    // Attacker death → draw a card.
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(attacker).unwrap().attacked_this_turn = true;
    let hand = g.players[0].hand.len();
    let ctx2 = EffectContext {
        trigger_source: Some(EntityRef::Permanent(attacker)),
        ..EffectContext::for_trigger(garna, 0, None, 0)
    };
    g.resolve_effect(&effect, &ctx2).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "attacker death draws");
}
