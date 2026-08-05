//! Functionality tests for `catalog::sets::decks::tdm2` (TDM's last six gaps).

use crabomination::card::{CardDefinition, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::effects::EntityRef;
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 12);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn etb(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, def);
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
    id
}

fn swing(g: &mut GameState, attacker: CardId) {
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// A red Dragon under Call the Spirit Dragons is indestructible and takes a
/// +1/+1 counter from the upkeep trigger.
#[test]
fn call_the_spirit_dragons_grows_and_shields_dragons() {
    let mut g = main_phase();
    etb(&mut g, catalog::call_the_spirit_dragons());
    let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    assert!(g.computed_permanent(dragon).unwrap().keywords.contains(&Keyword::Indestructible));

    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(dragon).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "the red pick grew the only Dragon"
    );
}

/// Kotis exiles as many cards as damage dealt and hands out free-cast
/// permission only up to that mana value.
#[test]
fn kotis_exiles_and_frees_only_cheap_cards() {
    let mut g = main_phase();
    let kotis = g.add_card_to_battlefield(0, catalog::kotis_the_fangkeeper());
    // Top two: a 2-drop (castable off 2 damage) then a 7-drop (not).
    let cheap = g.add_card_to_library(1, catalog::grizzly_bears());
    let dear = g.add_card_to_library(1, catalog::shivan_dragon());
    swing(&mut g, kotis);

    assert_eq!(g.exile.iter().filter(|c| c.id == cheap || c.id == dear).count(), 2);
    assert!(g.exile.iter().find(|c| c.id == cheap).unwrap().may_play_until.is_some());
    assert!(g.exile.iter().find(|c| c.id == dear).unwrap().may_play_until.is_none());
}

/// Mardu Siegebreaker banishes a creature on ETB and swings a temporary copy
/// of it, which is sacrificed at the next end step.
#[test]
fn mardu_siegebreaker_swings_a_copy_of_the_exiled_creature() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mardu = etb(&mut g, catalog::mardu_siegebreaker());
    assert!(g.exile.iter().any(|c| c.id == bear), "the bear is exiled with Mardu");

    g.clear_sickness(mardu);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: mardu, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let copy = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Grizzly Bears")
        .map(|c| c.id)
        .expect("one copy per opponent");
    assert!(g.attacking.iter().any(|a| a.attacker == copy), "the copy joined the attack");

    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == copy), "the copy is sacrificed at end step");
}

/// New Way Forward soaks the next hit, reflects it at the source's controller
/// and draws that many cards.
#[test]
fn new_way_forward_reflects_the_prevented_damage() {
    let mut g = main_phase();
    let bolt_source = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let spell = g.add_card_to_hand(0, catalog::new_way_forward());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    flood(&mut g, 0);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);

    let life_before = (g.players[0].life, g.players[1].life);
    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 5, Some(bolt_source), &mut ev);
    assert_eq!(g.players[0].life, life_before.0, "damage prevented");
    assert_eq!(g.players[1].life, life_before.1 - 5, "reflected at the source's controller");
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 5, "drew that many cards");
}

/// Taigam copies your second spell each turn and suspends the original with
/// four time counters instead of letting it resolve.
#[test]
fn taigam_copies_then_suspends_the_second_spell() {
    let mut g = main_phase();
    etb(&mut g, catalog::taigam_master_opportunist());
    flood(&mut g, 0);
    for _ in 0..2 {
        let id = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast bolt");
        drain_stack(&mut g);
    }
    // First bolt resolved (3), the second was copied (3) and then suspended.
    assert_eq!(g.players[1].life, 20 - 6);
    let suspended = g
        .exile
        .iter()
        .find(|c| c.definition.name == "Lightning Bolt")
        .expect("second bolt suspended in exile");
    assert!(suspended.granted_suspend);
    assert_eq!(suspended.counter_count(CounterType::Time), 4);
}

/// Ugin's cast trigger exiles a colored permanent, and his −11 tutors every
/// colorless nonland card into a free-cast pile.
#[test]
fn ugin_exiles_on_cast_and_frees_colorless_cards() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ugin = g.add_card_to_hand(0, catalog::ugin_eye_of_the_storms());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: ugin,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Ugin");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "the colored permanent is exiled");

    let walker = g
        .battlefield
        .iter()
        .find(|c| c.definition.card_types.contains(&CardType::Planeswalker))
        .map(|c| c.id)
        .expect("Ugin on the battlefield");
    g.battlefield_find_mut(walker).unwrap().add_counters(CounterType::Loyalty, 11);
    let rock = g.add_card_to_library(0, catalog::sol_ring());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: walker,
        ability_index: 2,
        target: None,
        x_value: None,
    })
        .expect("ultimate");
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == rock).expect("Sol Ring tutored to exile");
    assert!(exiled.may_play_until.is_some(), "and castable this turn");
}
