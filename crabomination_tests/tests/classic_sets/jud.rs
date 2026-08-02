//! Judgment (JUD) — the block-closing set: Threshold payoffs, the Dwarf
//! tribe and the white Nomad/Cleric shell.

use crabomination::card::{CardId, Keyword, LandType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    cast_x(g, seat, id, target, None);
}

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn fill_graveyard(g: &mut GameState, seat: usize) {
    for _ in 0..7 {
        g.add_card_to_graveyard(seat, catalog::forest());
    }
}

/// Ancestor's Chosen banks a life per card in your graveyard.
#[test]
fn ancestors_chosen_gains_life_for_the_dead() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let spell = g.add_card_to_hand(0, catalog::ancestors_chosen());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].life, 27);
}

/// Battlewise Aven only gets its bonus past Threshold.
#[test]
fn battlewise_aven_hardens_at_threshold() {
    let mut g = main_phase();
    let aven = g.add_card_to_battlefield(0, catalog::battlewise_aven());
    assert_eq!(g.computed_permanent(aven).unwrap().power, 2);
    fill_graveyard(&mut g, 0);
    let cp = g.computed_permanent(aven).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// Benevolent Bodyguard trades itself for a colour of protection.
#[test]
fn benevolent_bodyguard_buys_protection() {
    let mut g = main_phase();
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let guard = g.add_card_to_battlefield(0, catalog::benevolent_bodyguard());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    activate(&mut g, 0, guard, 0, Some(Target::Permanent(friend)));
    assert!(
        g.computed_permanent(friend).unwrap().keywords.contains(&Keyword::Protection(Color::Red))
    );
}

/// Chastise kills an attacker and banks its power as life.
#[test]
fn chastise_kills_an_attacker_for_life() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(attacker).unwrap().summoning_sick = false;
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    let spell = g.add_card_to_hand(0, catalog::chastise());
    cast(&mut g, 0, spell, Some(Target::Permanent(attacker)));
    assert!(g.battlefield_find(attacker).is_none());
    assert_eq!(g.players[0].life, 22);
}

/// Aven Fogbringer bounces a land on the way in.
#[test]
fn aven_fogbringer_bounces_a_land() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let bird = g.add_card_to_hand(0, catalog::aven_fogbringer());
    cast(&mut g, 0, bird, Some(Target::Permanent(land)));
    assert!(g.battlefield_find(land).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == land));
}

/// Envelop counters a sorcery and nothing else.
#[test]
fn envelop_only_counters_sorceries() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    let sorcery = g.add_card_to_hand(1, catalog::crush_of_wurms());
    mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: sorcery,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast wurms");
    let envelop = g.add_card_to_hand(0, catalog::envelop());
    cast(&mut g, 0, envelop, Some(Target::Permanent(sorcery)));
    assert!(!g.battlefield.iter().any(|c| c.is_token), "the Wurms never landed");
}

/// Keep Watch draws one per attacker.
#[test]
fn keep_watch_draws_per_attacker() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for id in [a, b] {
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    }
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    let spell = g.add_card_to_hand(0, catalog::keep_watch());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Guiltfeeder drains for the defender's graveyard when it gets through.
#[test]
fn guiltfeeder_drains_for_the_graveyard() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 1);
    let feeder = g.add_card_to_battlefield(0, catalog::guiltfeeder());
    g.battlefield_find_mut(feeder).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: feeder,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 13);
}

/// Earsplitting Rats strips a card from each hand.
#[test]
fn earsplitting_rats_strips_each_hand() {
    let mut g = main_phase();
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(1, catalog::forest());
    let rats = g.add_card_to_hand(0, catalog::earsplitting_rats());
    cast(&mut g, 0, rats, None);
    assert_eq!(g.players[0].hand.len(), 0);
    assert_eq!(g.players[1].hand.len(), 0);
}

/// Arcane Teachings pumps its host and hands it a tap-to-ping.
#[test]
fn arcane_teachings_pumps_and_pings() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(host).unwrap().summoning_sick = false;
    let victim = g.add_card_to_battlefield(1, catalog::mystic_familiar());
    let aura = g.add_card_to_hand(0, catalog::arcane_teachings());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    assert_eq!(g.computed_permanent(host).unwrap().power, 4);
    activate(&mut g, 0, host, 0, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(victim).unwrap().damage, 1);
}

/// Firecat Blitz mints X hasty Cats and takes them back at end of turn.
#[test]
fn firecat_blitz_mints_transient_cats() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::firecat_blitz());
    cast_x(&mut g, 0, spell, None, Some(3));
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 3);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 0, "exiled at end of turn");
}

/// Lightning Surge scales past Threshold.
#[test]
fn lightning_surge_scales_at_threshold() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let spell = g.add_card_to_hand(0, catalog::lightning_surge());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14);
}

/// Grizzly Fate doubles its Bears past Threshold.
#[test]
fn grizzly_fate_doubles_at_threshold() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let spell = g.add_card_to_hand(0, catalog::grizzly_fate());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 4);
}

/// Exoskeletal Armor is sized by every graveyard's dead.
#[test]
fn exoskeletal_armor_counts_all_graveyards() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::exoskeletal_armor());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    assert_eq!(g.computed_permanent(host).unwrap().power, 4, "2/2 plus two dead creatures");
}

/// Krosan Wayfarer trades itself for a land drop.
#[test]
fn krosan_wayfarer_deploys_a_land() {
    let mut g = main_phase();
    let land = g.add_card_to_hand(0, catalog::forest());
    let wayfarer = g.add_card_to_battlefield(0, catalog::krosan_wayfarer());
    activate(&mut g, 0, wayfarer, 0, None);
    assert!(g.battlefield_find(land).is_some());
}

/// Krosan Verge fetches a Forest and a Plains, both tapped.
#[test]
fn krosan_verge_fetches_a_pair() {
    let mut g = main_phase();
    let forest = g.add_card_to_library(0, catalog::forest());
    let plains = g.add_card_to_library(0, catalog::plains());
    let verge = g.add_card_to_battlefield(0, catalog::krosan_verge());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
        DecisionAnswer::Search(Some(plains)),
    ]));
    activate(&mut g, 0, verge, 1, None);
    let fetched: Vec<LandType> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.is_land() && c.tapped)
        .flat_map(|c| c.definition.subtypes.land_types.clone())
        .collect();
    assert!(fetched.contains(&LandType::Forest) && fetched.contains(&LandType::Plains));
}

/// Anurid Brushhopper blinks itself out and comes back at end of turn.
#[test]
fn anurid_brushhopper_blinks_out_of_removal() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let frog = g.add_card_to_battlefield(0, catalog::anurid_brushhopper());
    activate(&mut g, 0, frog, 0, None);
    assert!(g.battlefield_find(frog).is_none(), "exiled");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Anurid Brushhopper"),
        "back at end of turn"
    );
}

/// Balthor the Defiled lords over Minions and mass-reanimates on the way out.
#[test]
fn balthor_lords_and_reanimates() {
    let mut g = main_phase();
    let balthor = g.add_card_to_battlefield(0, catalog::balthor_the_defiled());
    let minion = g.add_card_to_battlefield(0, catalog::cabal_trainee());
    assert_eq!(g.computed_permanent(minion).unwrap().power, 2, "Minion lord");
    let dead = g.add_card_to_graveyard(0, catalog::nantuko_shade());
    activate(&mut g, 0, balthor, 0, None);
    assert!(g.battlefield_find(dead).is_some(), "the black creature came back");
}

/// Epic Struggle wins the game once the board is wide enough.
#[test]
fn epic_struggle_wins_at_twenty_creatures() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::epic_struggle());
    for _ in 0..20 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.is_game_over() && g.players[0].is_alive(), "you win at twenty creatures");
}
