//! Functionality tests for the MH3 batch-5 cards in `catalog::sets::mh3e`.

use crate::card::CounterType;
use crate::catalog;
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;




/// Reiterating Bolt's Replicate—Pay {E}{E}{E} copies the spell once per payment.
#[test]
fn reiterating_bolt_energy_replicate_copies() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let spell = g.add_card_to_hand(0, catalog::reiterating_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].energy = 6;
    g.perform_action(GameAction::CastSpellReplicate {
        card_id: spell, times: 2,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast with Replicate x2");
    assert_eq!(g.players[0].energy, 0, "3 energy per replication x2 = 6 spent");
    assert_eq!(g.stack.len(), 3, "original spell plus two copies");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != victim), "the 3/3 takes lethal");
}

/// Reiterating Bolt's Replicate is rejected without enough energy.
#[test]
fn reiterating_bolt_replicate_needs_energy() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::reiterating_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].energy = 3; // needs 6 for x2
    assert!(g.perform_action(GameAction::CastSpellReplicate {
        card_id: spell, times: 2,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "replicate x2 needs six energy");
}

/// Volatile Stormdrake can't be the target of an opponent's ability.
#[test]
fn volatile_stormdrake_hexproof_from_abilities() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(0, catalog::volatile_stormdrake());
    let opp_pinger = g.add_card_to_battlefield(1, catalog::rootwater_hunter());
    let own_pinger = g.add_card_to_battlefield(0, catalog::rootwater_hunter());
    assert!(g.ability_target_has_protection(&Target::Permanent(drake), opp_pinger),
        "an opponent's ability can't target it");
    assert!(!g.ability_target_has_protection(&Target::Permanent(drake), own_pinger),
        "your own ability still can");
}

/// Reiterating Bolt deals 3 damage to a creature.
#[test]
fn reiterating_bolt_deals_three() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let spell = g.add_card_to_hand(0, catalog::reiterating_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(crate::game::types::Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Reiterating Bolt");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != victim), "3 damage kills the 3/3");
}

/// Planar Genesis deploys a revealed land tapped when one is on top.
#[test]
fn planar_genesis_deploys_a_land() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    // Top of library: a land plus three nonlands.
    let land = g.add_card_to_library(0, catalog::forest());
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::planar_genesis());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Planar Genesis");
    drain_stack(&mut g);
    let deployed = g.battlefield.iter().find(|c| c.id == land).expect("land deployed");
    assert!(deployed.tapped, "the land enters tapped");
}

/// With no land revealed, Planar Genesis puts a card into hand instead.
#[test]
fn planar_genesis_takes_a_card_when_no_land() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::planar_genesis());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Planar Genesis");
    drain_stack(&mut g);
    // Spell leaves hand (-1), one revealed card goes to hand (+1) = net unchanged.
    assert_eq!(g.players[0].hand.len(), before, "a card was put into hand");
}

/// Vega draws when you cast a spell from your graveyard (not from hand).
#[test]
fn vega_draws_on_graveyard_cast() {
    // Isolate Vega's extra draw by comparing library depletion with vs.
    // without Vega on the battlefield when flashing back the same spell.
    fn deplete_with(vega: bool) -> usize {
        let mut g = two_player_game();
        for _ in 0..20 { g.add_card_to_library(0, catalog::island()); }
        if vega {
            g.add_card_to_battlefield(0, catalog::vega_the_watcher());
        }
        let id = g.add_card_to_library(0, catalog::faithless_looting());
        let pos = g.players[0].library.iter().position(|c| c.id == id).unwrap();
        let card = g.players[0].library.remove(pos);
        g.players[0].graveyard.push(card);
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(1);
        let before = g.players[0].library.len();
        g.perform_action(GameAction::CastFlashback {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("flashback Faithless Looting");
        drain_stack(&mut g);
        before - g.players[0].library.len()
    }
    // Faithless Looting draws 2; Vega adds exactly one more.
    assert_eq!(deplete_with(false), 2, "without Vega only the spell's draws");
    assert_eq!(deplete_with(true), 3, "Vega draws one extra on the graveyard cast");
}

/// Chthonian Nightmare enters and grants three energy.
#[test]
fn chthonian_nightmare_etb_gives_energy() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::chthonian_nightmare());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Chthonian Nightmare");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 3, "ETB grants three energy");
}

/// Pay X {E} + sac a creature + bounce self to reanimate a MV-X creature.
#[test]
fn chthonian_nightmare_reanimates_by_energy_x() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let nightmare = g.add_card_to_battlefield(0, catalog::chthonian_nightmare());
    g.players[0].energy = 5;
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Target: a MV-2 creature card in the graveyard (Grizzly Bears is {1}{G}).
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: nightmare, ability_index: 0,
        target: Some(crate::game::types::Target::Permanent(dead)),
        additional_targets: vec![], x_value: Some(2),
    }).expect("activate for X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 3, "spent two energy");
    assert!(g.battlefield.iter().any(|c| c.id == dead), "MV-2 creature reanimated");
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "fodder creature sacrificed");
    assert!(g.players[0].hand.iter().any(|c| c.id == nightmare), "enchantment returned to hand");
}

/// Volatile Stormdrake steals a small creature (energy covers its mana value).
#[test]
fn volatile_stormdrake_steals_when_energy_pays() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    // Opponent's MV-2 Grizzly Bears; four energy easily covers the 2 upkeep.
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let drake_h = g.add_card_to_hand(0, catalog::volatile_stormdrake());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: drake_h,
        target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Volatile Stormdrake");
    drain_stack(&mut g);
    // Exchange: you now control the prey; opponent controls the Drake.
    assert_eq!(g.battlefield_find(prey).map(|c| c.controller), Some(0), "gained control of the prey");
    assert_eq!(g.battlefield_find(drake_h).map(|c| c.controller), Some(1), "opponent got the Drake");
    assert_eq!(g.players[0].energy, 2, "gained 4 energy, paid 2 for the MV-2 upkeep");
}

/// With no energy banked, a high-mana-value stolen creature is sacrificed.
#[test]
fn volatile_stormdrake_sacrifices_when_short_on_energy() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    // Opponent's MV-8 creature (Twisted Riddlekeeper): four energy can't cover the upkeep → sacrifice.
    let prey = g.add_card_to_battlefield(1, catalog::twisted_riddlekeeper());
    let drake_h = g.add_card_to_hand(0, catalog::volatile_stormdrake());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: drake_h,
        target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Volatile Stormdrake");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != prey), "couldn't pay the upkeep → prey sacrificed");
}

/// Jolted Awake grants two energy and reanimates a MV-2 card by paying it.
#[test]
fn jolted_awake_energy_reanimates() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let spell = g.add_card_to_hand(0, catalog::jolted_awake());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(crate::game::types::Target::Permanent(dead)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Jolted Awake");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == dead), "MV-2 creature reanimated");
    assert_eq!(g.players[0].energy, 0, "the two energy gained paid the MV-2 return");
}

/// Cycling Jolted Awake for {2} draws a card.
#[test]
fn jolted_awake_cycles() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::jolted_awake());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle for {2}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before, "discard one, draw one");
}

/// Lethal Throwdown's modified-sacrifice mode destroys a target and draws.
#[test]
fn lethal_throwdown_modified_mode_draws() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    // A modified fodder creature (a +1/+1 counter counts as a modification).
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(fodder).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::lethal_throwdown());
    g.players[0].mana_pool.add(Color::Black, 1);
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(crate::game::types::Target::Permanent(victim)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast in modified-sacrifice mode");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != fodder), "modified creature sacrificed");
    assert!(g.battlefield.iter().all(|c| c.id != victim), "target destroyed");
    // -1 for the spell leaving hand, +1 for the draw = net 0 change.
    assert_eq!(g.players[0].hand.len(), before - 1 + 1, "drew a card for sacrificing a modified creature");
}

/// Pyretic Rebirth returns a graveyard card and burns for its mana value.
#[test]
fn pyretic_rebirth_returns_and_burns() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    // A MV-3 artifact/creature card in the graveyard (Ornithopter is {0}; use
    // a bear-ish 3-drop: Grizzly Bears is MV 2). Use a 3/3 for a clean number.
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant());
    let mv = catalog::hill_giant().cost.cmc();
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::pyretic_rebirth());
    for c in [Color::Black, Color::Red] { g.players[0].mana_pool.add(c, 1); }
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(crate::game::types::Target::Permanent(dead)),
        additional_targets: vec![crate::game::types::Target::Permanent(victim)],
        mode: None, x_value: None,
    }).expect("cast Pyretic Rebirth");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "graveyard card returned to hand");
    // Hill Giant is 3/3; MV 4 damage from the returned Hill Giant is lethal.
    assert!(mv >= 3 && g.battlefield_find(victim).is_none(), "victim took its mana value ({mv}) in damage and died");
}

/// Argent Dais gains an oil counter only when two or more creatures attack.
#[test]
fn argent_dais_oil_on_multi_attack() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let dais_h = g.add_card_to_hand(0, catalog::argent_dais());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: dais_h, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Argent Dais");
    drain_stack(&mut g);
    let dais = dais_h;
    assert_eq!(g.battlefield_find(dais).unwrap().counter_count(CounterType::Oil), 2, "enters with two oil");
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]).expect("declare two attackers");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(dais).unwrap().counter_count(CounterType::Oil), 3, "oil added on 2+ attack");
}

/// The activated ability exiles another nonland permanent; its controller draws.
#[test]
fn argent_dais_exiles_and_draws() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let dais_h = g.add_card_to_hand(0, catalog::argent_dais());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: dais_h, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Argent Dais");
    drain_stack(&mut g);
    let dais = dais_h;
    g.clear_sickness(dais);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    g.players[0].mana_pool.add_colorless(2);
    let opp_hand = g.players[1].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: dais, ability_index: 0,
        target: Some(crate::game::types::Target::Permanent(victim)),
        additional_targets: vec![], x_value: None,
    }).expect("activate exile ability");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "victim exiled");
    assert_eq!(g.players[1].hand.len(), opp_hand + 2, "controller of the victim draws two");
    assert_eq!(g.battlefield_find(dais).unwrap().counter_count(CounterType::Oil), 0, "spent two oil");
}

/// Glimpse the Impossible exiles three, and uncast cards become Spawn at end.
#[test]
fn glimpse_the_impossible_exiles_then_spawns() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::glimpse_the_impossible());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Glimpse the Impossible");
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 3, "three cards exiled");
    // Fire the next-end-step penalty for the uncast cards.
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Eldrazi Spawn").count(), 3,
        "each uncast card makes an Eldrazi Spawn");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 3,
        "uncast cards go to graveyard");
}

/// Unstable Amulet enters with two energy and pings on a graveyard cast.
#[test]
fn unstable_amulet_etb_energy_and_ping() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let amulet = g.add_card_to_hand(0, catalog::unstable_amulet());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: amulet, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Unstable Amulet");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 2, "ETB grants two energy");
    // Flashback a spell from the graveyard → the amulet pings the opponent.
    let opp_life = g.players[1].life;
    let looting = g.add_card_to_library(0, catalog::faithless_looting());
    let pos = g.players[0].library.iter().position(|c| c.id == looting).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card);
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastFlashback { card_id: looting, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("flashback from graveyard");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 1, "graveyard cast pings the opponent for 1");
}

/// The {T}, Pay {E}{E} ability exiles the top card and lets you play it.
#[test]
fn unstable_amulet_impulse_ability() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let amulet = g.add_card_to_battlefield(0, catalog::unstable_amulet());
    g.clear_sickness(amulet);
    g.players[0].energy = 2;
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: amulet, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate impulse for {T} + {E}{E}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 0, "paid two energy");
    assert!(g.exile.iter().any(|c| c.id == top), "top card exiled and playable");
}

/// Izzet Generatorium boosts energy gains by one and gates its draw on 4+ spent.
#[test]
fn izzet_generatorium_energy_bonus_and_draw_gate() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let generatorium = g.add_card_to_battlefield(0, catalog::izzet_generatorium());
    g.clear_sickness(generatorium);
    // Amulet's ETB "get {E}{E}" becomes three with the Generatorium's +1.
    let amulet = g.add_card_to_hand(0, catalog::unstable_amulet());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: amulet, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Unstable Amulet");
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 3, "two energy plus the Generatorium's one");
    // Draw ability is locked until 4+ energy is spent this turn.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: generatorium, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).is_err(), "draw locked with under four energy spent");
    // Spend four energy this turn (routed through the shared chokepoint).
    g.players[0].energy = 4;
    g.spend_energy(0, 4);
    assert!(g.players[0].energy_spent_this_turn >= 4, "spent 4+ energy this turn");
    let hand = g.players[0].hand.len();
    let _ = g.add_card_to_library(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: generatorium, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("draw now unlocked");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card once 4+ energy was spent");
}
