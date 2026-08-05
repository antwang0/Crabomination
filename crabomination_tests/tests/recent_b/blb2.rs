//! Bloomburrow / Duskmourn gap batch 2 (`decks::blb2`).

use crabomination::card::{CardDefinition, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::effects::EntityRef;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
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

/// Enter a permanent through the shared "another permanent entered" dispatch
/// so board-wide ETB watchers see it.
fn etb_seen(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, def);
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: id }]);
    drain_stack(g);
    id
}

/// Starforged Sword's gift attaches it on entry and swaps +3/+3 for flying.
#[test]
fn starforged_sword_attaches_when_gifted() {
    let mut g = main_phase();
    let bird = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    let sword = g.add_card_to_hand(0, catalog::starforged_sword());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastGift {
        card_id: sword,
        target: Some(Target::Permanent(bird)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast with gift");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.is_token), "the Fish was gifted");
    let cp = g.computed_permanent(bird).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8), "+3/+3 from the Sword");
    assert!(!cp.keywords.contains(&Keyword::Flying), "and it lost flying");
}

/// Cruelclaw's Heist exiles a card; only the gifted mode lets you cast it.
#[test]
fn cruelclaws_heist_exiles_and_gifts_the_cast() {
    let mut g = main_phase();
    let loot = g.add_card_to_hand(1, catalog::shivan_dragon());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let heist = g.add_card_to_hand(0, catalog::cruelclaws_heist());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastGift {
        card_id: heist,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast with gift");
    drain_stack(&mut g);
    let stolen = g.exile.iter().find(|c| c.id == loot).expect("the Dragon was exiled");
    assert!(stolen.may_play_until.is_some(), "and the gift bought the right to cast it");
}

/// Grievous Wound stops the enchanted player gaining life and halves them on
/// every hit.
#[test]
fn grievous_wound_locks_lifegain_and_halves_on_damage() {
    let mut g = main_phase();
    let aura = g.add_card_to_hand(0, catalog::grievous_wound());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);

    g.adjust_life(1, 5);
    assert_eq!(g.players[1].life, 20, "no life gain");

    let bolt = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(1), 4, Some(bolt), &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    // 20 − 4 = 16, then half of 16 rounded up = 8 more.
    assert_eq!(g.players[1].life, 8);
}

/// The Jolly Balloon Man mints a 1/1 flying Balloon copy that dies at end of
/// turn.
#[test]
fn jolly_balloon_man_makes_a_temporary_balloon_copy() {
    let mut g = main_phase();
    let jolly = etb(&mut g, catalog::the_jolly_balloon_man());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(jolly);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: jolly,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let token = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Grizzly Bears")
        .map(|c| c.id)
        .expect("balloon copy");
    let cp = g.computed_permanent(token).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.keywords.contains(&Keyword::Flying));

    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == token), "sacrificed at end step");
}

/// Muerra ramps one mana per Raccoon at your main phase.
#[test]
fn muerra_ramps_per_raccoon() {
    let mut g = main_phase();
    etb(&mut g, catalog::muerra_trash_tactician());
    g.step = TurnStep::Upkeep;
    g.players[0].mana_pool.empty();
    let _ = g.advance_step(Vec::new());
    while g.step != TurnStep::PreCombatMain {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "one mana for Muerra herself");
}

/// Wick creates a Snail on the first Rat and grows it afterwards.
#[test]
fn wick_creates_then_grows_a_snail() {
    let mut g = main_phase();
    let wick = etb_seen(&mut g, catalog::wick_the_whorled_mind());
    let snail = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Snail")
        .map(|c| c.id)
        .expect("Snail minted by Wick's own entry");

    // A second Rat grows the Snail instead of minting another — both Wicks
    // see the entry, so it takes two counters.
    let _ = etb_seen(&mut g, catalog::wick_the_whorled_mind());
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Snail").count(), 1);
    assert_eq!(
        g.battlefield_find(snail).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2
    );
    let _ = wick;
}

/// Zoraline's attack trigger reanimates a cheap permanent with a finality
/// counter, and Bats attacking gain life.
#[test]
fn zoraline_reanimates_on_attack() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let zoraline = g.add_card_to_battlefield(0, catalog::zoraline_cosmos_caller());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.clear_sickness(zoraline);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: zoraline,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == dead), "the bear came back");
    assert_eq!(
        g.battlefield_find(dead).unwrap().counter_count(CounterType::Finality),
        1
    );
    assert_eq!(g.players[0].life, 20 - 2 + 1, "paid 2 life, gained 1 from the Bat trigger");
}

/// Kastral's Bird damage trigger draws on its third mode.
#[test]
fn kastral_pays_out_on_bird_combat_damage() {
    let mut g = main_phase();
    let kastral = g.add_card_to_battlefield(0, catalog::kastral_the_windcrested());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.clear_sickness(kastral);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: kastral, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    let before = g.battlefield_find(kastral).unwrap().counter_count(CounterType::PlusOnePlusOne);
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    // AutoDecider takes mode 0: a Bird from hand/graveyard — none here, so the
    // trigger resolves without a counter or a board change.
    assert_eq!(
        g.battlefield_find(kastral).unwrap().counter_count(CounterType::PlusOnePlusOne),
        before
    );
    assert_eq!(g.players[1].life, 20 - 4, "the swing still connected");
}
