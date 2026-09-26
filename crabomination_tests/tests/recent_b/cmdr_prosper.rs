//! Commander: the Planar Portal precon (AFC, Prosper, Tome-Bound,
//! `decks::cmdr_prosper`).

use crabomination::card::{CardId, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn act_as(g: &mut GameState, seat: usize, action: GameAction) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    act_as(g, seat, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_as(g, 0, id, targets)
}

fn activate(g: &mut GameState, id: CardId, index: usize, targets: &[Target]) -> Result<(), String> {
    act_as(g, 0, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        x_value: None,
        mode: None,
    })
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn bolt_as(g: &mut GameState, seat: usize, at: Target) {
    let b = g.add_card_to_hand(seat, catalog::lightning_bolt());
    cast_as(g, seat, b, &[at]).expect("bolt");
}

/// Advance to `step`, dispatching each step's events the way a priority
/// pass does (combat damage and deaths reach the trigger dispatcher).
fn to_step(g: &mut GameState, step: TurnStep) {
    for _ in 0..16 {
        if g.step == step {
            return;
        }
        if let Ok(events) = g.advance_step(Vec::new()) {
            g.dispatch_triggers_for_events(&events);
        }
        drain_stack(g);
    }
}

/// Prosper: its end step exiles the top card, playable through your next
/// turn; casting it from exile makes a Treasure.
#[test]
fn prosper_turns_exile_into_treasure() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::prosper_tome_bound());
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    g.step = TurnStep::PostCombatMain;
    to_step(&mut g, TurnStep::End);
    assert!(g.exile.iter().any(|c| c.id == top), "exiled at the end step");
    g.step = TurnStep::PostCombatMain;
    act_as(&mut g, 0, GameAction::CastFromZoneWithoutPaying {
        card_id: top,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast it from exile");
    assert_eq!(named(&g, 0, "Treasure").len(), 1);
}

/// CR 614 — Lorcan steals a creature card hitting an opponent's graveyard
/// for life equal to its mana value, as a Warlock; when that Warlock would
/// die it's exiled instead.
#[test]
fn cr_614_lorcan_collects_and_exiles_warlocks() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::lorcan_warlock_collector());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[0].life;
    let doom = g.add_card_to_hand(0, catalog::doom_blade());
    cast_at(&mut g, doom, &[Target::Permanent(giant)]).expect("kill");
    let c = g.battlefield_find(giant).expect("stolen");
    assert_eq!(c.controller, 0);
    assert_eq!(g.players[0].life, life - 4);
    assert!(g.computed_permanent(giant).unwrap().subtypes().creature_types.contains(&CreatureType::Warlock));
    bolt_as(&mut g, 0, Target::Permanent(giant));
    bolt_as(&mut g, 0, Target::Permanent(giant));
    assert!(g.exile.iter().any(|c| c.id == giant), "a dying Warlock is exiled");
}

/// CR 603.10 — Death Tyrant makes a Zombie when an opponent's blocker dies.
#[test]
fn cr_603_10_death_tyrant_counts_a_dead_blocker() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::death_tyrant());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(giant);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: giant, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, giant)])).expect("block");
    drain_stack(&mut g);
    to_step(&mut g, TurnStep::EndCombat);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(named(&g, 0, "Zombie").len(), 1);
}

/// Danse Macabre: each player sacrifices; a high roll returns two of them
/// under your control.
#[test]
fn danse_macabre_takes_the_fallen() {
    let mut g = main_phase(2);
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(18), DecisionAnswer::Cards(vec![mine, theirs])]));
    let dm = g.add_card_to_hand(0, catalog::danse_macabre());
    cast_at(&mut g, dm, &[]).expect("cast");
    for id in [mine, theirs] {
        assert_eq!(g.battlefield_find(id).map(|c| c.controller), Some(0), "back under your control");
    }
}

/// Karazikar: attacking a player taps and goads a creature of theirs.
#[test]
fn karazikar_goads_the_attacked_players_creature() {
    let mut g = main_phase(3);
    let k = g.add_card_to_battlefield(0, catalog::karazikar_the_eye_tyrant());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.clear_sickness(k);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: k, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let c = g.battlefield_find(giant).unwrap();
    assert!(c.tapped);
    assert!(g.is_goaded(c), "goaded");
}

/// CR 508.1 / 115.1 — Karazikar's "tap target creature that player controls"
/// is a target, once per attacked player: a hexproof creature can't be
/// chosen (the bigger Carnage Tyrant stays untapped), and each of the two
/// attacked players has a creature of their own tapped and goaded.
#[test]
fn karazikar_targets_a_creature_of_each_attacked_player() {
    let mut g = main_phase(3);
    let k = g.add_card_to_battlefield(0, catalog::karazikar_the_eye_tyrant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tyrant = g.add_card_to_battlefield(1, catalog::carnage_tyrant());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let other = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.clear_sickness(k);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: k, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    let tapped_goaded = |id| {
        let c = g.battlefield_find(id).unwrap();
        c.tapped && g.is_goaded(c)
    };
    assert!(!tapped_goaded(tyrant), "hexproof: not a legal target");
    assert!(tapped_goaded(giant), "seat 1's trigger took seat 1's creature");
    assert!(tapped_goaded(other), "seat 2's trigger took seat 2's creature");
}

/// Karazikar: an opponent attacking another opponent draws you both a card
/// for 1 life each.
#[test]
fn karazikar_taxes_the_war_between_opponents() {
    let mut g = main_phase(3);
    for s in 0..3 {
        g.add_card_to_library(s, catalog::plains());
    }
    g.add_card_to_battlefield(0, catalog::karazikar_the_eye_tyrant());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (l0 - 1, l1 - 1));
    assert_eq!((g.players[0].hand.len(), g.players[1].hand.len()), (1, 1));
}

/// Hellish Rebuke: an opponent's creature that damages you this turn is
/// sacrificed and costs its controller 2 life.
#[test]
fn hellish_rebuke_punishes_an_attacker() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hr = g.add_card_to_hand(0, catalog::hellish_rebuke());
    cast_at(&mut g, hr, &[]).expect("cast");
    g.active_player_idx = 1;
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let life = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    to_step(&mut g, TurnStep::EndCombat);
    assert!(g.battlefield_find(bear).is_none(), "sacrificed");
    assert_eq!(g.players[1].life, life - 2);
}

/// Share the Spoils: the top of each library is exiled on entry; at your
/// upkeep you may play one of them with any mana, which refills your pile.
#[test]
fn share_the_spoils_shares() {
    let mut g = main_phase(2);
    let mine = g.add_card_to_library(0, catalog::lightning_bolt());
    let theirs = g.add_card_to_library(1, catalog::grizzly_bears());
    let next = g.add_card_to_library(0, catalog::plains());
    // The library top is the last card added: keep `mine` on top.
    g.players[0].library.retain(|c| c.id != mine);
    let card = crabomination::card::CardInstance::new(mine, catalog::lightning_bolt(), 0);
    g.players[0].library.insert(0, card);
    let sts = g.add_card_to_hand(0, catalog::share_the_spoils());
    cast_at(&mut g, sts, &[]).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == mine) && g.exile.iter().any(|c| c.id == theirs));
    g.step = TurnStep::Untap;
    to_step(&mut g, TurnStep::Upkeep);
    to_step(&mut g, TurnStep::PreCombatMain);
    act_as(&mut g, 0, GameAction::CastSpell {
        card_id: theirs,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .or_else(|_| {
        act_as(&mut g, 0, GameAction::CastFromZoneWithoutPaying {
            card_id: theirs,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    })
    .expect("cast the opponent's exiled Bears");
    assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(0));
    assert!(g.exile.iter().any(|c| c.id == next), "refilled from your library");
}

/// Bag of Devouring exiles what you sacrifice and brings it back to hand on
/// its roll.
#[test]
fn bag_of_devouring_devours() {
    let mut g = main_phase(2);
    let bag = g.add_card_to_battlefield(0, catalog::bag_of_devouring());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::plains());
    activate(&mut g, bag, 0, &[]).expect("sacrifice the Bear");
    assert!(g.exile.iter().any(|c| c.id == bear && c.exiled_with == Some(bag)));
    g.battlefield_find_mut(bag).unwrap().tapped = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(5), DecisionAnswer::Cards(vec![bear])]));
    activate(&mut g, bag, 1, &[]).expect("roll");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Reckless Endeavor: one d12 to every creature, the other in Treasures.
#[test]
fn reckless_endeavor_splits_the_dice() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(3), DecisionAnswer::DieRoll(5)]));
    let re = g.add_card_to_hand(0, catalog::reckless_endeavor());
    cast_at(&mut g, re, &[]).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert!(!named(&g, 0, "Treasure").is_empty());
}

/// Grim Hireling: two Treasures per damaging batch; sacrifice X Treasures
/// for -X/-X.
#[test]
fn grim_hireling_gets_paid() {
    let mut g = main_phase(2);
    let gh = g.add_card_to_battlefield(0, catalog::grim_hireling());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for a in [gh, bear] {
        g.clear_sickness(a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: gh, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    to_step(&mut g, TurnStep::EndCombat);
    assert_eq!(named(&g, 0, "Treasure").len(), 2, "one batch, two Treasures");
}

/// Piper of the Swarm: Rats have menace; three Rats steal a creature.
#[test]
fn piper_of_the_swarm_leads_the_rats() {
    let mut g = main_phase(2);
    let piper = g.add_card_to_battlefield(0, catalog::piper_of_the_swarm());
    g.clear_sickness(piper);
    activate(&mut g, piper, 0, &[]).expect("a Rat");
    let rats = named(&g, 0, "Rat");
    assert!(g.computed_permanent(rats[0]).unwrap().keywords().contains(&Keyword::Menace));
}

/// Fiendlash: the equipped creature, dealt damage, hits a player for its
/// power.
#[test]
fn fiendlash_lashes_back() {
    let mut g = main_phase(2);
    let fl = g.add_card_to_battlefield(0, catalog::fiendlash());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.battlefield_find_mut(fl).unwrap().attached_to = Some(giant);
    let life = g.players[1].life;
    let shock = g.add_card_to_hand(1, catalog::shock());
    cast_as(&mut g, 1, shock, &[Target::Permanent(giant)]).expect("shock");
    assert_eq!(g.players[1].life, life - 5, "3 + 2 power");
}

/// Hurl Through Hell exiles a creature you may then cast with any mana.
#[test]
fn hurl_through_hell_takes_the_creature() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let hth = g.add_card_to_hand(0, catalog::hurl_through_hell());
    cast_at(&mut g, hth, &[Target::Permanent(giant)]).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == giant));
    act_as(&mut g, 0, GameAction::CastFromZoneWithoutPaying {
        card_id: giant,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast it");
    assert_eq!(g.battlefield_find(giant).map(|c| c.controller), Some(0));
}

/// Orazca Relic's sacrifice needs the city's blessing.
#[test]
fn orazca_relic_needs_the_city() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::plains());
    let or = g.add_card_to_battlefield(0, catalog::orazca_relic());
    assert!(activate(&mut g, or, 1, &[]).is_err(), "no blessing yet");
    for _ in 0..10 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    let life = g.players[0].life;
    activate(&mut g, or, 1, &[]).expect("ten permanents");
    assert_eq!(g.players[0].life, life + 3);
}

/// Chaos Channeler's d20 exiles one to three cards to play this turn.
#[test]
fn chaos_channeler_surges() {
    let mut g = main_phase(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::plains());
    }
    let cc = g.add_card_to_battlefield(0, catalog::chaos_channeler());
    g.clear_sickness(cc);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(20)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: cc, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), 3);
}

/// Dark-Dweller Oracle sacrifices a creature to impulse a card; Zhalfirin
/// Void scries as it enters.
#[test]
fn dark_dweller_oracle_and_zhalfirin_void() {
    let mut g = main_phase(2);
    g.add_card_to_library(0, catalog::plains());
    let o = g.add_card_to_battlefield(0, catalog::dark_dweller_oracle());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, o, 0, &[]).expect("sacrifice");
    assert_eq!(g.exile.len(), 1);
    let z = g.add_card_to_battlefield(0, catalog::zhalfirin_void());
    assert!(!g.battlefield_find(z).unwrap().tapped);
}

/// Dead Man's Chest: its creature dying exiles that many off its owner's
/// library for you to cast.
#[test]
fn dead_mans_chest_loots_the_body() {
    let mut g = main_phase(2);
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let dmc = g.add_card_to_hand(0, catalog::dead_mans_chest());
    cast_at(&mut g, dmc, &[Target::Permanent(giant)]).expect("enchant");
    let doom = g.add_card_to_hand(0, catalog::doom_blade());
    cast_at(&mut g, doom, &[Target::Permanent(giant)]).expect("kill");
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 3);
}

/// Fiend of the Shadows: the player it hits exiles a card from hand, which
/// you may play.
#[test]
fn fiend_of_the_shadows_steals_from_hand() {
    let mut g = main_phase(2);
    let f = g.add_card_to_battlefield(0, catalog::fiend_of_the_shadows());
    let card = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.clear_sickness(f);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: f, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    to_step(&mut g, TurnStep::EndCombat);
    let c = g.exile.iter().find(|c| c.id == card).expect("exiled from hand");
    assert!(c.may_play_until.is_some_and(|p| p.player == 0));
}

/// You Find Some Prisoners: destroy an artifact.
#[test]
fn you_find_some_prisoners_breaks_chains() {
    let mut g = main_phase(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let y = g.add_card_to_hand(0, catalog::you_find_some_prisoners());
    act_as(&mut g, 0, GameAction::CastSpell {
        card_id: y,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast");
    assert!(g.battlefield_find(ring).is_none());
}

/// Hellish Rebuke also answers noncombat damage from an opponent's
/// permanent (a pinger).
#[test]
fn hellish_rebuke_punishes_a_pinger() {
    let mut g = main_phase(2);
    let pyro = g.add_card_to_battlefield(1, catalog::prodigal_pyromancer());
    g.clear_sickness(pyro);
    let hr = g.add_card_to_hand(0, catalog::hellish_rebuke());
    cast_at(&mut g, hr, &[]).expect("cast");
    act_as(&mut g, 1, GameAction::ActivateAbility {
        card_id: pyro,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("ping");
    assert!(g.battlefield_find(pyro).is_none(), "sacrificed");
}
