//! Star Trek: the fourth batch of EDHREC closers (`decks::trek_singles4`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
use crabomination::card::Value;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn attack(g: &mut GameState, attackers: &[CardId], defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let atk = attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(defender) }).collect();
    g.perform_action(GameAction::DeclareAttackers(atk)).expect("attack");
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// Worf, Still Not Merry (CR 508.3a / 508.4): one tapped, attacking Faerie per
/// attacking non-Faerie creature — a Faerie attacker adds none.
#[test]
fn worf_makes_a_faerie_per_non_faerie_attacker() {
    let mut g = pod(3);
    let worf = ready(&mut g, 0, catalog::worf_still_not_merry());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sprite = ready(&mut g, 0, catalog::spellstutter_sprite());
    attack(&mut g, &[worf, bear, sprite], 2);
    let faeries: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Faerie").collect();
    assert_eq!(faeries.len(), 2, "Worf and the Bears, not the Sprite");
    for f in faeries {
        assert!(f.tapped);
        assert!(g.attacking.iter().any(|a| a.attacker == f.id), "joined the attack");
    }
}

/// La Forge, Thoughtful as Ever: counts permanents you control that YOU put a
/// counter on this turn — any kind, any permanent type; a counter an opponent
/// put on your permanent, or one you put on theirs, doesn't count. The
/// registry clears at cleanup.
#[test]
fn la_forge_counts_permanents_you_put_counters_on() {
    let mut g = pod(3);
    let laforge = ready(&mut g, 0, catalog::la_forge_thoughtful_as_ever());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let rock = ready(&mut g, 0, catalog::sol_ring());
    let theirs_on_mine = ready(&mut g, 0, catalog::hill_giant());
    let mine_on_theirs = ready(&mut g, 1, catalog::hill_giant());
    let put = |g: &mut GameState, placer: usize, id: CardId, kind: CounterType| {
        let ctx = EffectContext::for_spell(placer, Some(Target::Permanent(id)), 0, 0);
        let fx = Effect::AddCounter { what: Selector::Target(0), kind, amount: Value::ONE };
        let ev = g.resolve_effect(&fx, &ctx).expect("counter");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(g);
    };
    put(&mut g, 0, bear, CounterType::PlusOnePlusOne);
    put(&mut g, 0, bear, CounterType::Shield);
    put(&mut g, 0, rock, CounterType::Charge);
    put(&mut g, 1, theirs_on_mine, CounterType::MinusOneMinusOne);
    put(&mut g, 0, mine_on_theirs, CounterType::PlusOnePlusOne);
    g.priority.player_with_priority = 0;
    activate(&mut g, laforge, 0, None);
    assert_eq!(named(&g, 0, "Faerie"), 2, "the Bears (once) and Sol Ring");
    // Next turn the registry is empty.
    g.do_cleanup(&mut Vec::new());
    g.battlefield_find_mut(laforge).unwrap().tapped = false;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    activate(&mut g, laforge, 0, None);
    assert_eq!(named(&g, 0, "Faerie"), 2);
}

/// Scotty, Chief Engineer: tapping an untapped artifact you control is the
/// whole cost; the target can't be blocked this turn.
#[test]
fn scotty_taps_an_artifact_to_make_a_creature_unblockable() {
    let mut g = pod(2);
    let scotty = ready(&mut g, 0, catalog::scotty_chief_engineer());
    let rock = ready(&mut g, 0, catalog::sol_ring());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, scotty, 0, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(rock).unwrap().tapped, "the artifact paid");
    assert!(!g.battlefield_find(scotty).unwrap().tapped);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Unblockable));
    // No untapped artifact left: the ability can't be activated.
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: scotty,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert!(err.is_err());
}

/// Lieutenant Hikaru Sulu (CR 701.44a): becoming tapped by attacking makes
/// him explore — a land on top goes to hand.
#[test]
fn sulu_explores_when_he_becomes_tapped() {
    let mut g = pod(2);
    let sulu = ready(&mut g, 0, catalog::lieutenant_hikaru_sulu());
    let hand = g.players[0].hand.len();
    attack(&mut g, &[sulu], 1);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the Island explored into hand");
    // A nonland on top: a +1/+1 counter instead.
    let mut g = pod(2);
    let sulu = ready(&mut g, 0, catalog::lieutenant_hikaru_sulu());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].library.rotate_right(1);
    attack(&mut g, &[sulu], 1);
    assert_eq!(g.battlefield_find(sulu).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Subcommander T'Pol: the second mode taps the target and stuns it.
#[test]
fn t_pol_taps_and_stuns() {
    let mut g = pod(3);
    let giant = ready(&mut g, 2, catalog::hill_giant());
    let tpol = g.add_card_to_hand(0, catalog::subcommander_t_pol());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    g.perform_action(GameAction::CastSpell { card_id: tpol, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let gi = g.battlefield_find(giant).unwrap();
    assert!(gi.tapped);
    assert_eq!(gi.counter_count(CounterType::Stun), 1);
}

/// Starfleet Communicator (CR 707.10f): casting it copies it, and the copy is
/// a token Communicator; each equipped attacker draws one per OTHER creature
/// you control with a Communicator attached, then may put that many lands.
#[test]
fn starfleet_communicator_copies_itself_and_counts_other_carriers() {
    let mut g = pod(3);
    let comm = g.add_card_to_hand(0, catalog::starfleet_communicator());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: comm, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let comms: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.definition.name == "Starfleet Communicator").map(|c| c.id).collect();
    assert_eq!(comms.len(), 2, "the card and its token copy");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Starfleet Communicator" && c.is_token));
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::hill_giant());
    for (&eq, host) in comms.iter().zip([a, b]) {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::Equip { equipment: eq, target: host }).expect("equip");
        drain_stack(&mut g);
    }
    g.add_card_to_hand(0, catalog::forest());
    let (hand, lands) = (g.players[0].hand.len(), g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count());
    g.decider = Box::new(ScriptedDecider::new([]));
    attack(&mut g, &[a], 1);
    // X = 1 (the Giant): draw one, put up to one land.
    let now_lands = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(g.players[0].hand.len() + (now_lands - lands), hand + 1);
    assert!(now_lands <= lands + 1);
}

/// Antagonist Incorporator: an artifact creature connecting may pay {1} for a
/// Borg token; a nonartifact creature has no such ability.
#[test]
fn antagonist_incorporator_grants_paid_borg_on_connect() {
    let mut g = pod(2);
    let inc = ready(&mut g, 0, catalog::antagonist_incorporator());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, &[inc, bear], 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Borg"), 1, "one artifact creature connected");
}

/// Borg Cube (CR 607.2a): with nothing exiled with it, entering exiles the
/// target creature (linked); attacking then assimilates that card instead of
/// exiling anew — under your control with a +1/+1 counter, a Borg artifact.
#[test]
fn borg_cube_exiles_then_assimilates_its_own_exile() {
    let mut g = pod(3);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let cube = g.add_card_to_hand(0, catalog::borg_cube());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: cube, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == giant && c.exiled_with == Some(cube)), "linked exile");
    let bear = ready(&mut g, 2, catalog::grizzly_bears());
    g.clear_sickness(cube);
    g.battlefield_find_mut(cube).unwrap().counters.add(CounterType::Charge, 12);
    attack(&mut g, &[cube], 2);
    let gi = g.battlefield_find(giant).expect("assimilated");
    assert_eq!(gi.controller, 0);
    assert_eq!(gi.counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.battlefield_find(bear).is_some(), "assimilated instead of exiling");
}

/// Death Is Irrelevant: your end step assimilates a creature card from an
/// opponent's graveyard; your artifact creatures are indestructible.
#[test]
fn death_is_irrelevant_assimilates_at_end_step() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::death_is_irrelevant());
    let giant = g.add_card_to_graveyard(2, catalog::hill_giant());
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    let gi = g.battlefield_find(giant).expect("assimilated");
    assert_eq!(gi.controller, 0);
    assert!(g.computed_permanent(giant).unwrap().keywords().contains(&Keyword::Indestructible), "now a Borg artifact");
}

/// Gwyndala, the Progeny (CR 707.2): her Heirloom enters attached (+1/+1);
/// {2} makes every Equipment token a copy of the target Equipment until end
/// of turn — the token stays attached and now grants Bonesplitter's bonus.
#[test]
fn gwyndala_turns_equipment_tokens_into_copies() {
    let mut g = pod(2);
    let gw = g.add_card_to_hand(0, catalog::gwyndala_the_progeny());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: gw, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let pt = |g: &GameState| g.computed_permanent(gw).map(|c| (c.power, c.toughness)).unwrap();
    assert_eq!(pt(&g), (2, 2), "Neuroflux Heirloom attached");
    let splitter = ready(&mut g, 0, catalog::bonesplitter());
    activate(&mut g, gw, 0, Some(Target::Permanent(splitter)));
    assert_eq!(pt(&g), (3, 1), "the Heirloom is a Bonesplitter now");
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Bonesplitter"));
}

/// Ensign Pavel Chekov (CR 101.4): each opponent exiles their top X, X the
/// differently named lands you control (two Forests count once); you may play
/// one land of each opponent's from exile.
#[test]
fn ensign_chekov_exiles_and_offers_a_land_per_opponent() {
    let mut g = pod(3);
    for def in [catalog::forest(), catalog::forest(), catalog::island()] {
        g.add_card_to_battlefield(0, def);
    }
    let lib = |g: &GameState, s: usize| g.players[s].library.len();
    let (l1, l2) = (lib(&g, 1), lib(&g, 2));
    let chekov = g.add_card_to_hand(0, catalog::ensign_pavel_chekov());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: chekov, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!((lib(&g, 1), lib(&g, 2)), (l1 - 2, l2 - 2), "X = 2 distinct names");
    let playable: Vec<CardId> = g.exile.iter().filter(|c| c.owner != 0 && c.may_play_until.as_ref().is_some_and(|m| m.player == 0)).map(|c| c.id).collect();
    assert_eq!(playable.len(), 2, "one land per opponent");
    g.players[0].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(playable[0])).expect("play an opponent's land from exile");
    assert_eq!(g.battlefield_find(playable[0]).unwrap().controller, 0);
}

/// Borg Queen, Perfection Manifest: entering assimilates a creature card from
/// an opponent's graveyard (never your own); it becomes a Borg artifact
/// creature and gets her +2/+0.
#[test]
fn borg_queen_manifest_assimilates_from_an_opponents_graveyard() {
    let mut g = pod(3);
    let mine = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_graveyard(2, catalog::hill_giant());
    let queen = g.add_card_to_hand(0, catalog::borg_queen_perfection_manifest());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: queen, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mine));
    let cp = g.computed_permanent(theirs).expect("assimilated");
    assert_eq!((cp.power, cp.toughness), (3 + 1 + 2, 3 + 1), "counter and anthem");
}

/// Q, Stacking the Deck (CR 113.6 / 707.12): the nonland card Q digs to gains
/// an ability that works only from exile, only on your turn; it exiles a card
/// from hand and casts a COPY, paying its cost — the exiled card stays put.
#[test]
fn q_digs_a_card_whose_copies_you_cast_from_exile() {
    let mut g = pod(3);
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let q = g.add_card_to_hand(0, catalog::q_stacking_the_deck());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: q, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "dug to the Bears");
    let fodder = g.add_card_to_hand(0, catalog::island());
    let idx = g.exile.iter().find(|c| c.id == bear).unwrap().definition.activated_abilities.len();
    flood(&mut g, 0);
    activate(&mut g, bear, idx, None);
    assert!(g.exile.iter().any(|c| c.id == fodder), "the hand card paid");
    assert!(g.exile.iter().any(|c| c.id == bear), "the original stays exiled");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears" && c.is_token).count(), 1);
    // Not on an opponent's turn.
    g.add_card_to_hand(0, catalog::island());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    let off_turn = g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert!(off_turn.is_err());
}
