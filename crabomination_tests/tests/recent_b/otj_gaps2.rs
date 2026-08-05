//! Outlaws of Thunder Junction gap batch 2 (`decks::otj2`) — the build-around
//! legends plus Assimilation Aegis.

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

fn cast(g: &mut GameState, def: CardDefinition, target: Option<Target>) -> CardId {
    cast_mode(g, def, target, None)
}

fn cast_mode(
    g: &mut GameState,
    def: CardDefinition,
    target: Option<Target>,
    mode: Option<usize>,
) -> CardId {
    let id = g.add_card_to_hand(0, def);
    flood(g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
    id
}

/// Assimilation Aegis exiles a creature on entry; equipping turns the bearer
/// into a copy of that card, and unequipping ends the copy.
#[test]
fn assimilation_aegis_copies_the_exiled_card_while_attached() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let aegis = etb(&mut g, catalog::assimilation_aegis());
    assert!(g.exile.iter().any(|c| c.id == victim), "the Dragon is banished");

    let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: aegis, target: bearer }).expect("equip");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bearer).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "the bear is a Shivan Dragon");

    g.battlefield_find_mut(aegis).unwrap().attached_to = None;
    let _ = g.check_state_based_actions();
    assert_eq!(g.computed_permanent(bearer).unwrap().power, 2, "the copy ends on unattach");
}

/// Breeches copies your second spell on a won flip.
#[test]
fn breeches_copies_the_second_spell_on_heads() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),  // sacrifice the artifact
        DecisionAnswer::Bool(true),  // won flip
    ]));
    etb(&mut g, catalog::breeches_the_blastmaker());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    cast(&mut g, catalog::lightning_bolt(), Some(Target::Player(1)));
    cast(&mut g, catalog::lightning_bolt(), Some(Target::Player(1)));
    // Two bolts (6) plus one copy of the second (3).
    assert_eq!(g.players[1].life, 20 - 9);
}

/// Calamity swings two temporary copies of the creature that saddled it.
#[test]
fn calamity_copies_its_saddler_twice() {
    let mut g = main_phase();
    let calamity = etb(&mut g, catalog::calamity_galloping_inferno());
    let saddler = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(saddler);
    g.perform_action(GameAction::Saddle { mount: calamity, creatures: vec![saddler] })
        .expect("saddle");
    drain_stack(&mut g);

    g.clear_sickness(calamity);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: calamity, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let copies = g
        .battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == "Grizzly Bears")
        .count();
    assert_eq!(copies, 2, "the process repeats once");
}

/// Kellan free-casts a cheap permanent when you cast from outside your hand.
#[test]
fn kellan_free_casts_a_permanent_after_a_graveyard_cast() {
    let mut g = main_phase();
    etb(&mut g, catalog::kellan_the_kid());
    let freebie = g.add_card_to_hand(0, catalog::grizzly_bears());
    // A flashback cast is not from hand.
    let flash = g.add_card_to_graveyard(0, catalog::deep_analysis());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFlashback {
        card_id: flash,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flashback");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.id == freebie),
        "the free permanent came down off Kellan"
    );
}

/// Lilah plots your multicolored instants instead of binning them.
#[test]
fn lilah_plots_a_multicolored_spell_on_resolution() {
    let mut g = main_phase();
    etb(&mut g, catalog::lilah_undefeated_slickshot());
    let spell = cast(&mut g, catalog::terminate(), None);
    assert!(g.exile.iter().any(|c| c.id == spell), "exiled rather than binned");
    assert!(g.plotted_cards.contains(&spell), "and plotted");
}

/// Riku pays out one pick per mode chosen: a one-mode charm yields exactly
/// one of his three options (the impulse).
#[test]
fn riku_offers_one_pick_per_chosen_mode() {
    let mut g = main_phase();
    let riku = etb(&mut g, catalog::riku_of_many_paths());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    cast_mode(&mut g, catalog::abzan_charm(), None, Some(1));
    // One mode chosen → exactly one of Riku's modes resolves (the impulse).
    assert_eq!(
        g.battlefield_find(riku).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
        "the counter mode wasn't taken"
    );
    assert!(g.exile.iter().any(|c| c.may_play_until.is_some()), "the impulse mode resolved");
}

/// Taii Wakeen draws on an exact-lethal ping and pumps later noncombat damage.
#[test]
fn taii_wakeen_draws_on_exact_lethal_and_pumps_burn() {
    let mut g = main_phase();
    let taii = etb(&mut g, catalog::taii_wakeen_perfect_shot());
    g.clear_sickness(taii);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(bear), 2, Some(taii), &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "exact lethal drew a card");

    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: taii,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let life_before = g.players[1].life;
    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(1), 3, Some(taii), &mut ev);
    assert_eq!(g.players[1].life, life_before - 5, "3 + X");
}

/// The Gitrog eats its saddler to draw and deploy that many lands.
#[test]
fn the_gitrog_converts_its_saddler_into_cards_and_lands() {
    let mut g = main_phase();
    let gitrog = etb(&mut g, catalog::the_gitrog_ravenous_ride());
    let saddler = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(saddler);
    g.perform_action(GameAction::Saddle { mount: gitrog, creatures: vec![saddler] })
        .expect("saddle");
    drain_stack(&mut g);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let lands: Vec<CardId> =
        (0..2).map(|_| g.add_card_to_hand(0, catalog::forest())).collect();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),          // eat the saddler
        DecisionAnswer::Cards(lands.clone()), // deploy both lands
    ]));

    g.clear_sickness(gitrog);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: gitrog, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(!g.battlefield.iter().any(|c| c.id == saddler), "the saddler was eaten");
    assert_eq!(
        lands.iter().filter(|id| g.battlefield.iter().any(|c| c.id == **id)).count(),
        2,
        "both lands entered (power 2)"
    );
    assert!(g.battlefield.iter().filter(|c| c.id == lands[0]).all(|c| c.tapped));
}

/// The equip keyword is printed on the Aegis.
#[test]
fn assimilation_aegis_has_equip() {
    assert!(
        catalog::assimilation_aegis()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::Equip(_)))
    );
}
