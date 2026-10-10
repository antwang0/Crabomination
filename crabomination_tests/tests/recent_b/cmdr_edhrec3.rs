//! Commander: EDHREC average-deck gaps, third batch (`decks::cmdr_edhrec3`):
//! Ketramose, the New Dawn's and Niko, Light of Hope's lists.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast_x(g: &mut GameState, id: CardId, x_value: Option<u32>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, ability_index: usize) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn loyalty(g: &mut GameState, id: CardId, ability_index: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index, target, x_value: None })
        .expect("loyalty");
    drain_stack(g);
}

fn connect(g: &mut GameState, attacker: CardId) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(g);
    }
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Flickering Hound: casting a creature spell blinks another creature you
/// control (CR 603.2 cast trigger; the returned card is a new object, CR
/// 400.7, so its ETB fires again).
#[test]
fn flickering_hound_blinks_on_creature_casts() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::flickering_hound());
    let visionary = ready(&mut g, 0, catalog::elvish_visionary());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g);
    let hand = g.players[0].hand.len();
    cast_x(&mut g, bear, None);
    assert!(g.battlefield_find(visionary).is_some(), "it came back");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "as a new object its ETB drew again");
}

/// Unlicensed Hearse: two cards from one graveyard, linked to it; its
/// characteristic-defining P/T counts them (CR 604.3).
#[test]
fn unlicensed_hearse_grows_with_its_exiles() {
    let mut g = pod(2);
    let hearse = ready(&mut g, 0, catalog::unlicensed_hearse());
    for _ in 0..3 {
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
    }
    let pt = |g: &GameState| {
        let c = g.computed_permanent(hearse).unwrap();
        (c.power, c.toughness)
    };
    assert_eq!(pt(&g), (0, 0));
    activate(&mut g, hearse, 0);
    assert_eq!(g.players[1].graveyard.len(), 1);
    assert_eq!(g.exile.iter().filter(|c| c.exiled_with == Some(hearse)).count(), 2);
    assert_eq!(pt(&g), (2, 2));
}

/// Abdel Adrian: the exile lasts until it leaves (CR 610.3) and pays a
/// Soldier per card; when it goes, the cards come back.
#[test]
fn abdel_adrian_trades_permanents_for_soldiers_until_it_leaves() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    ready(&mut g, 0, catalog::plains());
    let abdel = g.add_card_to_hand(0, catalog::abdel_adrian_gorions_ward());
    flood(&mut g);
    cast_x(&mut g, abdel, None);
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(ring).is_none());
    assert_eq!(named(&g, "Plains"), 1, "lands stay");
    assert_eq!(named(&g, "Soldier"), 2);
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Exile { what: Selector::ExactObjects(vec![abdel]) }, &ctx).expect("exile");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Grizzly Bears") + named(&g, "Sol Ring"), 2, "both return");
}

/// Senu: its own cost exiles it (CR 602.1); from exile its trigger functions
/// (CR 113.6) and returns it attacking when a legendary attacker goes
/// unblocked (CR 509.3g / 508.4) — never off a nonlegendary one.
#[test]
fn senu_returns_attacking_beside_an_unblocked_legend() {
    let mut g = pod(2);
    let senu = ready(&mut g, 0, catalog::senu_keen_eyed_protector());
    let life = g.players[0].life;
    activate(&mut g, senu, 0);
    assert_eq!(g.players[0].life, life + 2);
    assert!(g.exile.iter().any(|c| c.id == senu));

    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    connect(&mut g, bear);
    assert!(g.exile.iter().any(|c| c.id == senu), "a nonlegendary attacker brings nothing");

    let opp = g.players[1].life;
    let isamaru = ready(&mut g, 0, catalog::isamaru_hound_of_konda());
    connect(&mut g, isamaru);
    assert!(g.battlefield_find(senu).is_some(), "Senu came back");
    assert_eq!(g.players[1].life, opp - 2 - 2, "and dealt its combat damage");
}

/// Battle Angels of Tyr: each clause compares the damaged player against
/// every other player, you included, strictly.
#[test]
fn battle_angels_reward_hitting_the_leader() {
    let mut g = pod(2);
    let angels = ready(&mut g, 0, catalog::battle_angels_of_tyr());
    g.add_card_to_library(0, catalog::island());
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::island());
    }
    ready(&mut g, 1, catalog::island());
    g.players[0].life = 40;
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    connect(&mut g, angels);
    assert_eq!(g.players[0].hand.len(), hand + 1, "more cards: draw");
    assert_eq!(named(&g, "Treasure"), 1, "more lands: Treasure");
    assert_eq!(g.players[0].life, life, "less life: no gain");
}

/// Candlekeep Sage: the Background's granted ability (CR 903.3b's commander
/// creature) draws as the commander enters and as it leaves.
#[test]
fn candlekeep_sage_draws_on_commander_entry_and_exit() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::candlekeep_sage());
    let cmdr = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    flood(&mut g);
    let hand = g.players[0].hand.len();
    cast_x(&mut g, cmdr, None);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "entry draw");
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![cmdr]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "leave draw");
}

/// Niko Aris: X Shards on entry (CR 107.3m); −1 deals two per card drawn
/// to a tapped creature; +1 makes an attacker unblockable and returns it
/// once it deals damage.
#[test]
fn niko_aris_shards_bolts_and_bounces() {
    let mut g = pod(2);
    let niko = g.add_card_to_hand(0, catalog::niko_aris());
    flood(&mut g);
    cast_x(&mut g, niko, Some(2));
    assert_eq!(named(&g, "Shard"), 2);

    loyalty(&mut g, niko, 2, None);
    assert_eq!(named(&g, "Shard"), 3);

    let giant = ready(&mut g, 1, catalog::hill_giant());
    g.battlefield_find_mut(giant).unwrap().tapped = true;
    g.players[0].cards_drawn_this_turn = 2;
    g.battlefield_find_mut(niko).unwrap().loyalty_uses_this_turn = 0;
    loyalty(&mut g, niko, 1, Some(Target::Permanent(giant)));
    assert!(g.battlefield_find(giant).is_none(), "4 damage kills a 3/3");
}

/// Niko Aris +1: the creature can't be blocked, and its damage sends it home.
#[test]
fn niko_aris_plus_one_returns_the_creature_after_damage() {
    let mut g = pod(2);
    let niko = ready(&mut g, 0, catalog::niko_aris());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::grizzly_bears());
    loyalty(&mut g, niko, 0, Some(Target::Permanent(bear)));
    let opp = g.players[1].life;
    connect(&mut g, bear);
    assert_eq!(g.players[1].life, opp - 2);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "back to hand");
}

/// Emperor of Bones: counters on it return a creature card it exiled, with a
/// finality counter and haste, and a CR 603.7 delayed trigger sacrifices it —
/// the finality counter then exiles it instead (CR 122.1g).
#[test]
fn emperor_of_bones_borrows_what_it_exiled() {
    let mut g = pod(2);
    let emperor = ready(&mut g, 0, catalog::emperor_of_bones());
    let giant = g.add_card_to_exile(1, catalog::hill_giant());
    g.exile.iter_mut().find(|c| c.id == giant).unwrap().exiled_with = Some(emperor);
    flood(&mut g);
    activate(&mut g, emperor, 0);
    let back = g.battlefield_find(giant).expect("the giant came back");
    assert_eq!(back.controller, 0);
    assert_eq!(back.counter_count(crabomination::card::CounterType::Finality), 1);
    assert!(g.computed_permanent(giant).unwrap().keywords().contains(&crabomination::card::Keyword::Haste));
    to_end_step(&mut g);
    assert!(g.battlefield_find(giant).is_none());
    assert!(g.exile.iter().any(|c| c.id == giant), "finality exiles it");
}

/// Kaya, Orzhov Usurper +1: two cards from one graveyard; 2 life only when a
/// creature card was among them.
#[test]
fn kaya_orzhov_usurper_gains_for_an_exiled_creature() {
    let mut g = pod(2);
    let kaya = ready(&mut g, 0, catalog::kaya_orzhov_usurper());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let life = g.players[0].life;
    loyalty(&mut g, kaya, 0, None);
    assert_eq!(g.players[1].graveyard.len(), 0);
    assert_eq!(g.players[0].life, life + 2);
    g.add_card_to_graveyard(1, catalog::island());
    g.battlefield_find_mut(kaya).unwrap().loyalty_uses_this_turn = 0;
    loyalty(&mut g, kaya, 0, None);
    assert_eq!(g.players[0].life, life + 2, "a land gains nothing");
}

fn to_end_step(g: &mut GameState) {
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// Axgard Armory: an Aura and an Equipment in one activation, each its own
/// search (CR 701.23).
#[test]
fn axgard_armory_fetches_an_aura_and_an_equipment() {
    let mut g = pod(2);
    let armory = ready(&mut g, 0, catalog::axgard_armory());
    let aura = g.add_card_to_library(0, catalog::pacifism());
    let gear = g.add_card_to_library(0, catalog::bonesplitter());
    g.add_card_to_library(0, catalog::island());
    flood(&mut g);
    activate(&mut g, armory, 1);
    let hand: Vec<CardId> = g.players[0].hand.iter().map(|c| c.id).collect();
    assert!(hand.contains(&aura) && hand.contains(&gear));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == armory), "sacrificed");
}

/// Tournament Grounds: its colored mana pays for a Knight (CR 106.6) and
/// for nothing else.
#[test]
fn tournament_grounds_mana_only_casts_knights_and_equipment() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::tournament_grounds());
    let b = ready(&mut g, 0, catalog::tournament_grounds());
    let lions = g.add_card_to_hand(0, catalog::savannah_lions());
    let knight = g.add_card_to_hand(0, catalog::knight_of_the_white_orchid());
    for id in [a, b] {
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 2,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap for W");
    }
    g.priority.player_with_priority = 0;
    let lions_cast = g.perform_action(GameAction::CastSpell { card_id: lions, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert!(lions_cast.is_err() || g.players[0].hand.iter().any(|c| c.id == lions), "a Cat isn't a Knight");
    cast_x(&mut g, knight, None);
    assert!(g.battlefield_find(knight).is_some());
}

/// Danitha, Benalia's Hope: an Equipment from hand enters attached to her.
#[test]
fn danitha_benalias_hope_brings_her_own_gear() {
    let mut g = pod(2);
    let gear = g.add_card_to_hand(0, catalog::bonesplitter());
    let danitha = g.add_card_to_hand(0, catalog::danitha_benalias_hope());
    flood(&mut g);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    cast_x(&mut g, danitha, None);
    assert_eq!(g.battlefield_find(gear).and_then(|c| c.attached_to), Some(danitha));
    assert_eq!(g.computed_permanent(danitha).unwrap().power, 6, "Bonesplitter's +2/+0");
}

/// Merry: first strike only while equipped; a draw only when another
/// legendary creature attacks beside it.
#[test]
fn merry_draws_beside_another_legend() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let merry = ready(&mut g, 0, catalog::merry_esquire_of_rohan());
    let fs = |g: &GameState| g.computed_permanent(merry).unwrap().keywords().contains(&crabomination::card::Keyword::FirstStrike);
    assert!(!fs(&g));
    let gear = ready(&mut g, 0, catalog::bonesplitter());
    g.battlefield_find_mut(gear).unwrap().attached_to = Some(merry);
    assert!(fs(&g), "equipped: first strike");

    let isamaru = ready(&mut g, 0, catalog::isamaru_hound_of_konda());
    let hand = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: merry, target: AttackTarget::Player(1) },
        Attack { attacker: isamaru, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Scouting Hawk: Keen Sight's intervening "if" (CR 603.4) — behind on
/// lands, a basic Plains enters tapped.
#[test]
fn scouting_hawk_catches_up_on_lands() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::island());
    let plains = g.add_card_to_library(0, catalog::plains());
    let hawk = g.add_card_to_hand(0, catalog::scouting_hawk());
    flood(&mut g);
    cast_x(&mut g, hawk, None);
    assert!(g.battlefield_find(plains).is_some_and(|c| c.tapped));
}

/// Lofty Denial: {1} to keep the spell, {4} with a flier on your side —
/// read as it resolves.
#[test]
fn lofty_denial_taxes_four_with_a_flier() {
    for flier in [false, true] {
        let mut g = pod(2);
        if flier {
            ready(&mut g, 0, catalog::battlefield_raptor());
        }
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.players[1].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
            .expect("bolt");
        let denial = g.add_card_to_hand(0, catalog::lofty_denial());
        flood(&mut g);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell { card_id: denial, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None })
            .expect("denial");
        let life = g.players[0].life;
        drain_stack(&mut g);
        let expected = if flier { life } else { life - 3 };
        assert_eq!(g.players[0].life, expected, "flier: {flier}");
    }
}
