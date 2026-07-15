//! Functionality tests for `catalog::sets::decks::recent29` — Tarkir:
//! Dragonstorm staples, plus the new `SelectionRequirement::WithAnyCounter`
//! filter (Delta Bloodflies).

use crabomination::catalog;
use crabomination::card::{CardId, CounterType};
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == controller && c.definition.name == name)
        .count()
}

/// Stand player 0 at PreCombatMain with priority and a full mana pool.
fn ready(g: &mut GameState) {
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..10 {
        g.players[0].mana_pool.add_colorless(1);
    }
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 4);
    }
}

/// Enter a card under `player`'s control through the real ETB funnel so
/// self-source ETB triggers and enters-with-counters fire.
fn etb_bf(g: &mut GameState, player: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.move_card_to_battlefield_for_test(player, def);
    drain_stack(g);
    id
}

/// Delta Bloodflies drains each opponent on attack while you control a
/// counter-bearing creature (exercises `WithAnyCounter`).
#[test]
fn delta_bloodflies_drains_with_counter() {
    let mut g = two_player_game();
    let delta = g.add_card_to_battlefield(0, catalog::delta_bloodflies());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.clear_sickness(delta);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.declare_attackers(vec![Attack { attacker: delta, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "opponent lost 1 life");
}

/// Without a counter-bearing creature, Delta Bloodflies' attack drains nothing.
#[test]
fn delta_bloodflies_no_counter_no_drain() {
    let mut g = two_player_game();
    let delta = g.add_card_to_battlefield(0, catalog::delta_bloodflies());
    g.clear_sickness(delta);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.declare_attackers(vec![Attack { attacker: delta, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "no drain without a counter on a creature");
}

/// Meticulous Artisan mints a Treasure on entry.
#[test]
fn meticulous_artisan_etb_treasure() {
    let mut g = two_player_game();
    etb_bf(&mut g, 0, catalog::meticulous_artisan());
    assert_eq!(count_named(&g, 0, "Treasure"), 1, "ETB Treasure");
}

/// Iridescent Tiger adds five mana on entry.
#[test]
fn iridescent_tiger_etb_mana() {
    let mut g = two_player_game();
    etb_bf(&mut g, 0, catalog::iridescent_tiger());
    assert_eq!(g.players[0].mana_pool.total(), 5, "WUBRG added");
}

/// Unburied Earthcarver sacrifices a creature to grow.
#[test]
fn unburied_earthcarver_sac_grows() {
    let mut g = two_player_game();
    let ue = g.add_card_to_battlefield(0, catalog::unburied_earthcarver());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    ready(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ue,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.battlefield_find(ue).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Unrooted Ancestor sacrifices a creature for indestructibility + a tap.
#[test]
fn unrooted_ancestor_indestructible() {
    let mut g = two_player_game();
    let ua = g.add_card_to_battlefield(0, catalog::unrooted_ancestor());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    ready(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ua,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ua).unwrap();
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Indestructible), "gained indestructible");
    assert!(g.battlefield_find(ua).unwrap().tapped, "tapped itself");
}

/// Gurmag Rakshasa's ETB shrinks an opponent's creature and pumps yours.
#[test]
fn gurmag_rakshasa_etb_modal() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies to -2/-2
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    etb_bf(&mut g, 0, catalog::gurmag_rakshasa());
    assert!(g.battlefield_find(foe).is_none(), "opponent's 2/2 shrank to 0/0 and died");
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "your creature got +2/+2");
}

/// Fleeting Effigy returns itself to hand at its controller's end step.
#[test]
fn fleeting_effigy_self_bounce() {
    let mut g = two_player_game();
    let fe = g.add_card_to_battlefield(0, catalog::fleeting_effigy());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(fe).is_none(), "left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == fe), "returned to hand");
}

/// Host of the Hereafter enters with two +1/+1 counters and relays them on
/// death.
#[test]
fn host_of_the_hereafter_counters() {
    let mut g = two_player_game();
    let host = etb_bf(&mut g, 0, catalog::host_of_the_hereafter());
    let cp = g.computed_permanent(host).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "entered as 4/4");
    let heir = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(host);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(heir).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "counters moved onto the other creature",
    );
}

/// Overwhelming Surge deals 3 to a creature (both modes run by default).
#[test]
fn overwhelming_surge_burns() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let art = g.add_card_to_battlefield(1, catalog::mind_stone());
    let os = g.add_card_to_hand(0, catalog::overwhelming_surge());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: os,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![Target::Permanent(art)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "3 damage killed the 2/2");
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// Marshal of the Lost pumps a creature by the number of attackers.
#[test]
fn marshal_of_the_lost_attack_pump() {
    let mut g = two_player_game();
    let marshal = g.add_card_to_battlefield(0, catalog::marshal_of_the_lost());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(marshal);
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: marshal, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    drain_stack(&mut g);
    // Two attackers → +2/+2 on the auto-picked target.
    let buffed = g.battlefield.iter().any(|c| {
        let cp = g.computed_permanent(c.id).unwrap();
        c.controller == 0 && (cp.power, cp.toughness) == (4, 4)
    });
    assert!(buffed, "a creature got +2/+2 from two attackers");
}

/// Embermouth Sentinel tutors a basic land to the top of the library.
#[test]
fn embermouth_sentinel_tutors_to_top() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    etb_bf(&mut g, 0, catalog::embermouth_sentinel());
    assert_eq!(g.players[0].library[0].definition.name, "Forest", "basic on top");
}

/// Rainveil Rejuvenator taps for green equal to its power.
#[test]
fn rainveil_rejuvenator_mana() {
    let mut g = two_player_game();
    let rr = g.add_card_to_battlefield(0, catalog::rainveil_rejuvenator());
    g.clear_sickness(rr);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rr,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "added two green");
}

/// Synchronized Charge distributes two +1/+1 counters onto your creature.
#[test]
fn synchronized_charge_distributes() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sc = g.add_card_to_hand(0, catalog::synchronized_charge());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: sc,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Watcher of the Wayside mills the opponent and gains you life.
#[test]
fn watcher_of_the_wayside_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let gy = g.players[1].graveyard.len();
    etb_bf(&mut g, 0, catalog::watcher_of_the_wayside());
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
    assert_eq!(g.players[1].graveyard.len(), gy + 2, "opponent milled 2");
}

/// Teeming Dragonstorm makes two Soldiers on entry.
#[test]
fn teeming_dragonstorm_makes_soldiers() {
    let mut g = two_player_game();
    etb_bf(&mut g, 0, catalog::teeming_dragonstorm());
    assert_eq!(count_named(&g, 0, "Soldier"), 2, "two 2/2 Soldiers");
}

/// Ainok Wayfarer mills three and takes a land into hand.
#[test]
fn ainok_wayfarer_grabs_land() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    etb_bf(&mut g, 0, catalog::ainok_wayfarer());
    assert_eq!(g.players[0].hand.len(), hand + 1, "took a land to hand");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "the land");
}

/// Tersa Lightshatter loots on entry (draw then discard, graveyard grows).
#[test]
fn tersa_lightshatter_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    let gy = g.players[0].graveyard.len();
    etb_bf(&mut g, 0, catalog::tersa_lightshatter());
    assert_eq!(g.players[0].graveyard.len(), gy + 1, "discarded one");
}

/// Temur Tawnyback loots on entry.
#[test]
fn temur_tawnyback_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
    let gy = g.players[0].graveyard.len();
    etb_bf(&mut g, 0, catalog::temur_tawnyback());
    assert_eq!(g.players[0].graveyard.len(), gy + 1, "discarded one");
}

/// Focus the Mind draws three and discards one on resolution.
#[test]
fn focus_the_mind_draws() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let fm = g.add_card_to_hand(0, catalog::focus_the_mind());
    ready(&mut g);
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: fm,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 3, "drew three");
}

/// Sage of the Skies copies itself when it's your second spell of the turn.
#[test]
fn sage_of_the_skies_copies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let sage = g.add_card_to_hand(0, catalog::sage_of_the_skies());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("first spell");
    drain_stack(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: sage, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("second spell");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Sage of the Skies"), 2, "original + token copy");
}
