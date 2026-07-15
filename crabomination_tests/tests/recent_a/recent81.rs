//! Functionality tests for `catalog::sets::decks::recent81`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
use crabomination::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

// ── Sacrifice outlets ────────────────────────────────────────────────────────

#[test]
fn vampiric_rites_sacrifices_for_life_and_a_card() {
    let mut g = two_player_game();
    let rites = g.add_card_to_battlefield(0, catalog::vampiric_rites());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rites, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Vampiric Rites");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 life");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 0,
        "the creature was sacrificed");
}

#[test]
fn blasting_station_sacrifices_to_ping() {
    let mut g = two_player_game();
    let station = g.add_card_to_battlefield(0, catalog::blasting_station());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: station, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Blasting Station");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "1 damage to opponent");
}

// ── Additional-cost draw ───────────────────────────────────────────────────────

#[test]
fn seize_the_spoils_discards_for_two_cards_and_a_treasure() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::plains()); }
    let discard_fodder = g.add_card_to_hand(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::seize_the_spoils());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Seize the Spoils");
    drain_stack(&mut g);
    // -Seize -discard +2 drawn = net -0 from the initial `hand` (which counted Seize + fodder).
    assert_eq!(g.players[0].hand.len(), hand - 2 + 2, "discarded one, drew two");
    let _ = discard_fodder;
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.controller == 0),
        "created a Treasure");
}

#[test]
fn blood_divination_sacrifices_a_creature_for_three_cards() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let id = g.add_card_to_hand(0, catalog::blood_divination());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Blood Divination");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "cast it, drew three");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_creature()).count(), 0,
        "the creature was sacrificed as an additional cost");
}

// ── Curiosity aura family ──────────────────────────────────────────────────────

#[test]
fn snake_umbra_pumps_and_draws_on_combat_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let umbra = g.add_card_to_hand(0, catalog::snake_umbra());
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, umbra, Target::Permanent(bear));
    // +1/+1 → 3/3.
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "Snake Umbra grants +1/+1");
    let hand = g.players[0].hand.len();
    g.clear_sickness(bear);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew from combat damage");
}

#[test]
fn curious_obsession_sacrifices_when_you_didnt_attack() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::curious_obsession());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    cast_at(&mut g, aura, Target::Permanent(bear));
    assert!(g.battlefield.iter().any(|c| c.id == aura), "aura attached");
    // End step, no attack this turn → the Aura sacrifices itself.
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == aura), "Aura sacrificed (didn't attack)");
}

// ── Lifegain matters ───────────────────────────────────────────────────────────

#[test]
fn ageless_entity_grows_on_lifegain() {
    let mut g = two_player_game();
    let ae = g.add_card_to_battlefield(0, catalog::ageless_entity());
    let bless = g.add_card_to_hand(0, catalog::chaplains_blessing());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    cast(&mut g, bless); // gain 5
    let counters = g.battlefield_find(ae).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 5, "5 +1/+1 counters from gaining 5 life");
}

#[test]
fn sunbond_grants_lifegain_growth() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sunbond = g.add_card_to_hand(0, catalog::sunbond());
    let bless = g.add_card_to_hand(0, catalog::chaplains_blessing());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast_at(&mut g, sunbond, Target::Permanent(bear));
    cast(&mut g, bless); // gain 5
    let counters = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 5, "enchanted creature grew by 5");
}

#[test]
fn nyx_fleece_ram_gains_life_each_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nyx_fleece_ram());
    let life = g.players[0].life;
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 at upkeep");
}

#[test]
fn wall_of_reverence_gains_life_equal_to_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wall_of_reverence());
    let big = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power
    let life = g.players[0].life;
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained life = target creature's power");
    let _ = big;
}

// ── Constellation ──────────────────────────────────────────────────────────────

#[test]
fn grim_guardian_drains_on_enchantment_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grim_guardian());
    let life = g.players[1].life;
    // A second enchantment entering triggers constellation.
    let aura = g.add_card_to_hand(0, catalog::nyx_fleece_ram()); // enchantment creature
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aura);
    assert_eq!(g.players[1].life, life - 1, "each opponent lost 1 from constellation");
}

#[test]
fn underworld_coinsmith_activated_drain() {
    let mut g = two_player_game();
    let cs = g.add_card_to_battlefield(0, catalog::underworld_coinsmith());
    let opp = g.players[1].life;
    let me = g.players[0].life;
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cs, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Coinsmith");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, me - 1, "paid 1 life");
}

// ── Misc ───────────────────────────────────────────────────────────────────────

#[test]
fn fecundity_draws_when_a_creature_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fecundity());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::plains()); }
    // Bolt my own creature so the full SBA+dispatch path fires Fecundity.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    let hand = g.players[0].hand.len();
    cast_at(&mut g, bolt, Target::Permanent(bear));
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "bolt cast, then drew when the creature died");
}

#[test]
fn mask_of_griselbrand_draws_on_equipped_death() {
    let mut g = two_player_game();
    let mask = g.add_card_to_battlefield(0, catalog::mask_of_griselbrand());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: mask, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink),
        "equipped creature has lifelink");
    let hand = g.players[0].hand.len();
    g.remove_to_graveyard_with_triggers(bear);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew cards equal to its power");
}

#[test]
fn sanctuary_cat_is_a_one_two() {
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::sanctuary_cat());
    let cp = g.computed_permanent(cat).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 2));
}

#[test]
fn chaplains_blessing_gains_five() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::chaplains_blessing());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    cast(&mut g, id);
    assert_eq!(g.players[0].life, life + 5);
}

#[test]
fn vicious_hunger_pings_and_gains() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vicious_hunger());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
    cast_at(&mut g, id, Target::Permanent(victim));
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "2 damage killed the 2/2");
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}

#[test]
fn life_goes_on_scales_with_a_death() {
    let mut g = two_player_game();
    // No creature died: gain 4.
    let id = g.add_card_to_hand(0, catalog::life_goes_on());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    cast(&mut g, id);
    assert_eq!(g.players[0].life, life + 4, "gained 4 with no death");
    // Now with a death this turn: gain 8.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(bear);
    let id2 = g.add_card_to_hand(0, catalog::life_goes_on());
    let life2 = g.players[0].life;
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    cast(&mut g, id2);
    assert_eq!(g.players[0].life, life2 + 8, "gained 8 after a creature died");
}

#[test]
fn feed_the_clan_scales_with_ferocious() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::feed_the_clan());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert_eq!(g.players[0].life, life + 5, "gained 5 without ferocious");
    // With a 4-power creature: gain 10.
    g.add_card_to_battlefield(0, catalog::craw_wurm());
    let id2 = g.add_card_to_hand(0, catalog::feed_the_clan());
    let life2 = g.players[0].life;
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id2);
    assert_eq!(g.players[0].life, life2 + 10, "gained 10 with a power-4 creature");
}

#[test]
fn silverflame_ritual_counters_each_creature() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverflame_ritual());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn renewed_faith_gains_and_cycles() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::renewed_faith());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(g.players[0].life, life + 6, "gained 6 on cast");
    // Cycling: gain 2 via the cycle trigger.
    let id2 = g.add_card_to_hand(0, catalog::renewed_faith());
    for _ in 0..2 { g.add_card_to_library(0, catalog::plains()); }
    let life2 = g.players[0].life;
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Cycle { card_id: id2, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life2 + 2, "gained 2 from the cycle trigger");
}
