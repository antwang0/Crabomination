//! Commander: EDHREC average-deck gaps, third batch (`decks::cmdr_edhrec3`):
//! Ketramose, the New Dawn's and Niko, Light of Hope's lists.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

/// The Eagles Are Coming!: unkicked, one creature however many are named
/// (CR 601.2c); kicked, all of them — and a 4/4 flying Bird Soldier for each
/// at the next upkeep (CR 603.7), whoever's it is.
#[test]
fn the_eagles_are_coming_trades_creatures_for_birds() {
    for kicked in [false, true] {
        let mut g = pod(3);
        let a = ready(&mut g, 0, catalog::grizzly_bears());
        let b = ready(&mut g, 0, catalog::grizzly_bears());
        let eagles = g.add_card_to_hand(0, catalog::the_eagles_are_coming());
        flood(&mut g);
        let (target, additional_targets) = (Some(Target::Permanent(a)), vec![Target::Permanent(b)]);
        let action = if kicked {
            GameAction::CastSpellKicked { card_id: eagles, target, additional_targets, mode: None, x_value: None }
        } else {
            GameAction::CastSpell { card_id: eagles, target, additional_targets, mode: None, x_value: None }
        };
        g.priority.player_with_priority = 0;
        g.perform_action(action).expect("cast");
        drain_stack(&mut g);
        let returned = [a, b].iter().filter(|id| g.players[0].hand.iter().any(|c| c.id == **id)).count();
        assert_eq!(returned, if kicked { 2 } else { 1 });
        assert_eq!(named(&g, "Bird Soldier"), 0, "not yet");
        while !(g.step == TurnStep::Upkeep && g.active_player_idx == 1) {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
        assert_eq!(named(&g, "Bird Soldier"), returned, "the next upkeep is seat 1's");
    }
}

/// Imoti: a mana value 6 spell gains cascade (CR 702.85a) and finds the
/// first cheaper nonland card.
#[test]
fn imoti_gives_big_spells_cascade() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::imoti_celebrant_of_bounty());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    flood(&mut g);
    cast_x(&mut g, wurm, None);
    assert!(g.battlefield_find(wurm).is_some());
    assert!(g.battlefield_find(bear).is_some(), "cascaded into the bear");
}

/// Leyline Immersion: ward {2} and five spell-only mana for the legend it
/// enchants.
#[test]
fn leyline_immersion_turns_a_legend_into_a_mana_engine() {
    let mut g = pod(2);
    let isamaru = ready(&mut g, 0, catalog::isamaru_hound_of_konda());
    let aura = ready(&mut g, 0, catalog::leyline_immersion());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(isamaru);
    let cp = g.computed_permanent(isamaru).unwrap();
    assert!(cp.keywords().iter().any(|k| matches!(k, crabomination::card::Keyword::Ward(_))));
    activate(&mut g, isamaru, 0);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 5, "spell-only mana");
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast_x(&mut g, ring, None);
    assert!(g.battlefield_find(ring).is_some(), "spell-only mana casts a spell");
}

/// Nicol Bolas, God-Pharaoh: +1 strips two cards from each opponent's hand,
/// −4 deals 7 to an opponent, +2 exiles to a nonland card castable free, −12
/// exiles the opponents' nonland permanents.
#[test]
fn nicol_bolas_god_pharaoh_four_abilities() {
    let mut g = pod(3);
    let bolas = ready(&mut g, 0, catalog::nicol_bolas_god_pharaoh());
    g.battlefield_find_mut(bolas).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 7);
    let reset = |g: &mut GameState| g.battlefield_find_mut(bolas).unwrap().loyalty_uses_this_turn = 0;
    for seat in [1, 2] {
        for _ in 0..3 {
            g.add_card_to_hand(seat, catalog::island());
        }
    }
    loyalty(&mut g, bolas, 1, None);
    assert_eq!((g.players[1].hand.len(), g.players[2].hand.len()), (1, 1));

    reset(&mut g);
    let life = g.players[1].life;
    loyalty(&mut g, bolas, 2, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 7);

    reset(&mut g);
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::island());
    loyalty(&mut g, bolas, 0, Some(Target::Player(1)));
    assert!(g.exile.iter().any(|c| c.id == bear));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("free cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(0));

    reset(&mut g);
    let theirs = ready(&mut g, 2, catalog::hill_giant());
    let land = ready(&mut g, 2, catalog::island());
    g.battlefield_find_mut(bolas).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 20);
    loyalty(&mut g, bolas, 3, None);
    assert!(g.battlefield_find(theirs).is_none() && g.battlefield_find(land).is_some());
    assert!(g.battlefield_find(bear).is_some(), "your own permanents stay");
}

/// Emergent Ultimatum: three monocolored cards, the opponent sends one back
/// (shuffled in), the rest are cast free; the Ultimatum exiles itself.
#[test]
fn emergent_ultimatum_casts_what_the_opponent_leaves() {
    let mut g = pod(2);
    let picks = [
        g.add_card_to_library(0, catalog::grizzly_bears()),
        g.add_card_to_library(0, catalog::hill_giant()),
        g.add_card_to_library(0, catalog::savannah_lions()),
    ];
    let ult = g.add_card_to_hand(0, catalog::emergent_ultimatum());
    flood(&mut g);
    cast_x(&mut g, ult, None);
    let on_field = picks.iter().filter(|id| g.battlefield_find(**id).is_some()).count();
    let in_library = picks.iter().filter(|id| g.players[0].library.iter().any(|c| c.id == **id)).count();
    assert_eq!((on_field, in_library), (2, 1));
    assert!(g.exile.iter().any(|c| c.id == ult), "exiles itself");
}

/// Navigation Orb: one land onto the battlefield tapped, one into hand.
#[test]
fn navigation_orb_splits_two_lands() {
    let mut g = pod(2);
    let orb = ready(&mut g, 0, catalog::navigation_orb());
    let a = g.add_card_to_library(0, catalog::plains());
    let b = g.add_card_to_library(0, catalog::azorius_guildgate());
    flood(&mut g);
    activate(&mut g, orb, 0);
    let on_field: Vec<CardId> = [a, b].into_iter().filter(|id| g.battlefield_find(*id).is_some()).collect();
    assert_eq!(on_field.len(), 1);
    assert!(g.battlefield_find(on_field[0]).unwrap().tapped);
    assert_eq!([a, b].iter().filter(|id| g.players[0].hand.iter().any(|c| c.id == **id)).count(), 1);
}

/// District Guide: a Gate counts as well as a basic.
#[test]
fn district_guide_finds_a_gate() {
    let mut g = pod(2);
    let gate = g.add_card_to_library(0, catalog::azorius_guildgate());
    let guide = g.add_card_to_hand(0, catalog::district_guide());
    flood(&mut g);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    cast_x(&mut g, guide, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == gate));
}

/// Nine-Fingers Keene: a Gate from the top nine; at nine Gates the rest come
/// to hand (no draw, CR 121.5), otherwise they go to the bottom.
#[test]
fn nine_fingers_keene_digs_for_gates() {
    for gates_before in [0, 8] {
        let mut g = pod(2);
        for _ in 0..gates_before {
            ready(&mut g, 0, catalog::azorius_guildgate());
        }
        let keene = ready(&mut g, 0, catalog::nine_fingers_keene());
        let gate = g.add_card_to_library(0, catalog::azorius_guildgate());
        for _ in 0..8 {
            g.add_card_to_library(0, catalog::island());
        }
        let hand = g.players[0].hand.len();
        connect(&mut g, keene);
        assert!(g.battlefield_find(gate).is_some(), "the Gate entered");
        let expected = if gates_before == 8 { hand + 8 } else { hand };
        assert_eq!(g.players[0].hand.len(), expected, "gates before: {gates_before}");
        assert_eq!(g.players[0].cards_drawn_this_turn, 0, "not a draw");
    }
}

/// Guild Summit: tapping two Gates draws two; a Gate entering draws one.
#[test]
fn guild_summit_draws_per_gate() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::azorius_guildgate());
    ready(&mut g, 0, catalog::azorius_guildgate());
    let summit = g.add_card_to_hand(0, catalog::guild_summit());
    flood(&mut g);
    let hand = g.players[0].hand.len();
    cast_x(&mut g, summit, None);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Azorius Guildgate" && c.tapped).count(), 2);
    let land = g.add_card_to_hand(0, catalog::azorius_guildgate());
    let hand = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("play gate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1);
}

/// Nasty End: three cards off a legendary sacrifice, two otherwise.
#[test]
fn nasty_end_draws_more_for_a_legend() {
    for legendary in [false, true] {
        let mut g = pod(2);
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::island());
        }
        if legendary {
            ready(&mut g, 0, catalog::isamaru_hound_of_konda());
        } else {
            ready(&mut g, 0, catalog::grizzly_bears());
        }
        let end = g.add_card_to_hand(0, catalog::nasty_end());
        flood(&mut g);
        let hand = g.players[0].hand.len();
        cast_x(&mut g, end, None);
        assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 0, "sacrificed");
        assert_eq!(g.players[0].hand.len(), hand - 1 + if legendary { 3 } else { 2 });
    }
}

/// Cryptolith Fragment: its mana costs every player a life; at your upkeep
/// with everyone at 10 or less it transforms (CR 701.28) into Aurora of
/// Emrakul.
#[test]
fn cryptolith_fragment_bleeds_the_table_and_transforms() {
    let mut g = pod(3);
    let frag = ready(&mut g, 0, catalog::cryptolith_fragment());
    let lives: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    activate(&mut g, frag, 0);
    assert!(g.players.iter().zip(&lives).all(|(p, l)| p.life == l - 1));
    for p in g.players.iter_mut() {
        p.life = 9;
    }
    for seat in 0..3 {
        for _ in 0..4 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g.battlefield_find_mut(frag).unwrap().tapped = false;
    while !(g.step == TurnStep::Upkeep && g.active_player_idx == 0) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(frag).unwrap().definition.name, "Aurora of Emrakul");
}

/// Orcus, Prince of Undeath: X = 2 — the first mode shrinks every other
/// creature by 2 and costs 2 life; the second returns creature cards worth
/// at most 2 in total, hasted.
#[test]
fn orcus_withers_or_raises() {
    let mut g = pod(2);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let orcus = g.add_card_to_hand(0, catalog::orcus_prince_of_undeath());
    flood(&mut g);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Mode(0)]));
    let life = g.players[0].life;
    cast_x(&mut g, orcus, Some(2));
    g.check_state_based_actions();
    assert!(g.battlefield_find(theirs).is_none(), "a 2/2 at -2/-2 dies");
    assert!(g.battlefield_find(orcus).is_some());
    assert_eq!(g.players[0].life, life - 2);

    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let orcus = g.add_card_to_hand(0, catalog::orcus_prince_of_undeath());
    flood(&mut g);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Mode(1)]));
    cast_x(&mut g, orcus, Some(2));
    assert!(g.battlefield_find(bear).is_some(), "the bear fits X = 2");
    assert!(g.battlefield_find(giant).is_none(), "the giant (4) doesn't");
}

/// Sanctum of Stone Fangs: at your precombat main, drain one per Shrine.
#[test]
fn sanctum_of_stone_fangs_drains_per_shrine() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::sanctum_of_stone_fangs());
    for seat in 0..3 {
        g.add_card_to_library(seat, catalog::island());
    }
    let lives: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    g.step = TurnStep::Upkeep;
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!([g.players[0].life, g.players[1].life, g.players[2].life], [lives[0] + 1, lives[1] - 1, lives[2] - 1]);
}

/// Sarkhan's Unsealing: a power-4 creature spell bolts for 4; a power-7 one
/// hits each opponent and everything they control for 4.
#[test]
fn sarkhans_unsealing_rewards_big_casts() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sarkhans_unsealing());
    let theirs = ready(&mut g, 1, catalog::hill_giant());
    let giant = g.add_card_to_hand(0, catalog::craw_wurm());
    flood(&mut g);
    let life = g.players[1].life;
    cast_x(&mut g, giant, None);
    let dealt = (life - g.players[1].life) + i32::from(g.battlefield_find(theirs).is_none()) * 4;
    assert_eq!(dealt, 4, "one 4-damage trigger off a 6/4");
}

/// Ob Nixilis, the Adversary: casualty X — sacrificing a 2-power creature
/// copies it non-legendary with 2 starting loyalty (CR 702.153a), so both
/// stay (no legend rule); the original keeps its printed 3.
#[test]
fn ob_nixilis_casualty_copy_has_the_sacrificed_power_as_loyalty() {
    let mut g = pod(2);
    let giant = ready(&mut g, 0, catalog::grizzly_bears());
    let ob = g.add_card_to_hand(0, catalog::ob_nixilis_the_adversary());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellCasualty {
        card_id: ob,
        sacrifice: giant,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("casualty cast");
    drain_stack(&mut g);
    g.check_state_based_actions();
    let walkers: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Ob Nixilis, the Adversary").collect();
    assert_eq!(walkers.len(), 2, "the copy isn't legendary");
    let copy = walkers.iter().find(|c| c.is_token).expect("the copy is a token");
    assert_eq!(copy.counter_count(crabomination::card::CounterType::Loyalty), 2);
    let original = walkers.iter().find(|c| !c.is_token).expect("the original");
    assert_eq!(original.counter_count(crabomination::card::CounterType::Loyalty), 3);
}

/// Dream Devourer: a hand card without foretell can be foretold (CR
/// 702.143), which pumps the Devourer; the {2}-cheaper cost stays with the
/// card, so it's cast for {1}{R} on a later turn even after the Devourer is
/// gone.
#[test]
fn dream_devourer_foretells_anything_for_two_less() {
    let mut g = pod(2);
    for seat in 0..2 {
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let devourer = ready(&mut g, 0, catalog::dream_devourer());
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Foretell { card_id: giant }).expect("foretell a hand card");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == giant && c.face_down));
    assert_eq!(g.computed_permanent(devourer).unwrap().power, 2, "+2/+0 for the foretell");
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![devourer]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    while g.active_player_idx == 0 {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    while !(g.step == TurnStep::PreCombatMain && g.active_player_idx == 0) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastForetold { card_id: giant, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast for {1}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some());
}

/// Raphael: other fiends get +1/+1 and lifelink; at the end step after a
/// creature card hit your graveyard, a Devil.
#[test]
fn raphael_pumps_fiends_and_makes_devils() {
    let mut g = pod(2);
    for seat in 0..2 {
        g.add_card_to_library(seat, catalog::island());
    }
    ready(&mut g, 0, catalog::raphael_fiendish_savior());
    let devourer = ready(&mut g, 0, catalog::dream_devourer());
    let cp = g.computed_permanent(devourer).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 4));
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Lifelink));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![bear]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    to_end_step(&mut g);
    assert_eq!(named(&g, "Devil"), 1);
}

/// Varragoth: boast only after attacking (CR 702.142), once a turn.
#[test]
fn varragoth_boasts_a_tutor_to_the_top() {
    let mut g = pod(2);
    let varragoth = ready(&mut g, 0, catalog::varragoth_bloodsky_sire());
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::grizzly_bears());
    flood(&mut g);
    let boast = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: varragoth,
            ability_index: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(boast(&mut g).is_err(), "it hasn't attacked");
    g.battlefield_find_mut(varragoth).unwrap().attacked_this_turn = true;
    boast(&mut g).expect("boast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), 2, "the card went back on top");
    assert!(boast(&mut g).is_err(), "once each turn");
}

/// Burning-Rune Demon: two different names, never itself; the opponent's
/// pick goes to hand, the other to the graveyard (its "may" body, resolved
/// as the bot plays it).
#[test]
fn burning_rune_demon_splits_two_cards() {
    let mut g = pod(2);
    let other = g.add_card_to_library(0, catalog::burning_rune_demon());
    g.add_card_to_library(0, catalog::hill_giant());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let demon = ready(&mut g, 0, catalog::burning_rune_demon());
    let body = match &catalog::burning_rune_demon().triggered_abilities[0].effect {
        Effect::MayDo { body, .. } => (**body).clone(),
        other => panic!("expected a may body, got {other:?}"),
    };
    let ctx = EffectContext::for_ability(demon, 0, None);
    let evs = g.resolve_effect(&body, &ctx).expect("search");
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].graveyard.len(), 1);
    assert!(g.players[0].library.iter().any(|c| c.id == other), "never a second Burning-Rune Demon");
}

/// Horn of Gondor: a Human Soldier on entry, then one per Human you control.
#[test]
fn horn_of_gondor_musters_per_human() {
    let mut g = pod(2);
    let horn = g.add_card_to_hand(0, catalog::horn_of_gondor());
    flood(&mut g);
    cast_x(&mut g, horn, None);
    assert_eq!(named(&g, "Human Soldier"), 1);
    activate(&mut g, horn, 0);
    assert_eq!(named(&g, "Human Soldier"), 2, "one Human, one more Soldier");
}

/// Horn of Valhalla: its adventure (CR 715.3) makes X Soldiers and exiles
/// it; cast later, the Equipment pumps by your creature count.
#[test]
fn horn_of_valhalla_calls_soldiers_then_scales() {
    let mut g = pod(2);
    let horn = g.add_card_to_hand(0, catalog::horn_of_valhalla());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastAdventure { card_id: horn, target: None, additional_targets: vec![], mode: None, x_value: Some(2) })
        .expect("Ysgard's Call");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Soldier"), 2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let gear = ready(&mut g, 0, catalog::horn_of_valhalla());
    g.battlefield_find_mut(gear).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2 + 3, 2 + 3), "three creatures: two Soldiers and the bear");
}

/// Preeminent Captain: its attack brings a Soldier from hand in attacking
/// (CR 508.4).
#[test]
fn preeminent_captain_brings_a_soldier_in_attacking() {
    let mut g = pod(2);
    let captain = ready(&mut g, 0, catalog::preeminent_captain());
    let vet = g.add_card_to_hand(0, catalog::valiant_veteran());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Cards(vec![vet]),
    ]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: captain, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vet).is_some_and(|c| c.tapped));
    assert!(g.attacking.iter().any(|a| a.attacker == vet));
}

/// Rescue Retriever: counters on the other Soldiers, and damage to an
/// attacking Soldier is prevented (CR 615) — a Bolt too, not only combat.
#[test]
fn rescue_retriever_shields_attacking_soldiers() {
    let mut g = pod(2);
    let vet = ready(&mut g, 0, catalog::siege_veteran());
    let dog = g.add_card_to_hand(0, catalog::rescue_retriever());
    flood(&mut g);
    cast_x(&mut g, dog, None);
    assert_eq!(g.battlefield_find(vet).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: vet, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let bolt = Effect::DealDamage { to: Selector::ExactObjects(vec![vet]), amount: crabomination::card::Value::Const(3) };
    g.resolve_effect(&bolt, &ctx).expect("bolt");
    assert_eq!(g.battlefield_find(vet).unwrap().damage, 0, "prevented");
}

/// Siege Veteran: a counter at your beginning of combat, and a Soldier
/// token when another nontoken Soldier dies.
#[test]
fn siege_veteran_replaces_fallen_soldiers() {
    let mut g = pod(2);
    let siege = ready(&mut g, 0, catalog::siege_veteran());
    let other = ready(&mut g, 0, catalog::valiant_veteran());
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::ExactObjects(vec![other]) }, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Soldier"), 1);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Soldier").unwrap();
    assert!(token.definition.card_types.contains(&crabomination::card::CardType::Artifact));
    g.add_card_to_library(0, catalog::island());
    g.step = TurnStep::PreCombatMain;
    while g.step != TurnStep::BeginCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    let counters: u32 = [siege]
        .iter()
        .chain(g.battlefield.iter().filter(|c| c.definition.name == "Soldier").map(|c| &c.id))
        .filter_map(|id| g.battlefield_find(*id))
        .map(|c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(counters, 1, "one counter on a creature you control");
}

/// Valiant Veteran: other Soldiers get +1/+1; from the graveyard it exiles
/// itself to put a counter on each Soldier.
#[test]
fn valiant_veteran_lords_and_rallies_from_the_grave() {
    let mut g = pod(2);
    let siege = ready(&mut g, 0, catalog::siege_veteran());
    let vet = ready(&mut g, 0, catalog::valiant_veteran());
    assert_eq!(g.computed_permanent(siege).unwrap().power, 3);
    assert_eq!(g.computed_permanent(vet).unwrap().power, 2, "not itself");
    let gy = g.add_card_to_graveyard(0, catalog::valiant_veteran());
    flood(&mut g);
    activate(&mut g, gy, 0);
    assert!(g.exile.iter().any(|c| c.id == gy));
    assert_eq!(g.battlefield_find(siege).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(vet).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}

/// Balor: its attack picks one or more modes, each at a different opponent —
/// in a four-seat pod all three land, one per opponent; in a duel only one.
#[test]
fn balor_spreads_its_modes_over_different_opponents() {
    for seats in [4, 2] {
        let mut g = pod(seats);
        let balor = ready(&mut g, 0, catalog::balor());
        for seat in 1..seats {
            for _ in 0..2 {
                g.add_card_to_hand(seat, catalog::island());
            }
            for _ in 0..4 {
                g.add_card_to_library(seat, catalog::island());
            }
            ready(&mut g, seat, catalog::sol_ring());
        }
        let lives: Vec<i32> = g.players.iter().map(|p| p.life).collect();
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: balor, target: AttackTarget::Player(1) }]))
            .expect("attack");
        drain_stack(&mut g);
        let burned = (1..seats).filter(|&s| g.players[s].life == lives[s] - 2).count();
        let sacrificed = (1..seats).filter(|&s| !g.battlefield.iter().any(|c| c.controller == s)).count();
        let wheeled = (1..seats).filter(|&s| g.players[s].graveyard.len() == 3).count();
        if seats == 4 {
            assert_eq!((burned, sacrificed, wheeled), (1, 1, 1), "one mode per opponent");
        } else {
            assert_eq!(burned + sacrificed + wheeled, 1, "a duel's one opponent takes one mode");
        }
    }
}

/// Gyruda, Doom of Depths: each player mills four, and an even-mana-value
/// creature card from among them comes to your side.
#[test]
fn gyruda_takes_an_even_creature_from_the_mill() {
    let mut g = pod(2);
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    let elves = g.add_card_to_library(0, catalog::llanowar_elves());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let gyruda = g.add_card_to_hand(0, catalog::gyruda_doom_of_depths());
    flood(&mut g);
    cast_x(&mut g, gyruda, None);
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(0), "the bear (2) is even");
    assert!(g.battlefield_find(elves).is_none(), "the elf (1) is odd");
}

/// Ardyn, the Usurper: "up to one" — no creature card in any graveyard is
/// no target and no token; with one, a 5/5 black token that's only a Demon
/// (CR 707.9b).
#[test]
fn ardyn_makes_a_demon_copy_or_nothing() {
    let mut g = pod(2);
    for seat in 0..2 {
        g.add_card_to_library(seat, catalog::island());
    }
    ready(&mut g, 0, catalog::ardyn_the_usurper());
    let tokens = |g: &GameState| g.battlefield.iter().filter(|c| c.is_token).count();
    g.step = TurnStep::PreCombatMain;
    while g.step != TurnStep::BeginCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(tokens(&g), 0);

    let mut g = pod(2);
    ready(&mut g, 0, catalog::ardyn_the_usurper());
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    while g.step != TurnStep::BeginCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear));
    let token = g.battlefield.iter().find(|c| c.is_token).expect("a token");
    let cp = g.computed_permanent(token.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert_eq!(cp.subtypes().creature_types, vec![crabomination::card::CreatureType::Demon]);
}

/// Declare `attacks` (attacker, defending seat) for seat 0 and resolve the
/// attack triggers, staying in combat.
fn declare(g: &mut GameState, attacks: &[(CardId, usize)]) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let attacks = attacks.iter().map(|&(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect();
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
    drain_stack(g);
}

fn begin_combat(g: &mut GameState) {
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(g);
}

/// Fire Nation Turret (Fire Lord Zuko's list): the combat trigger's +2/+0 and
/// firebending 2 (CR 702.189a: two {R} on attack); fifty charge counters buy
/// 50 damage.
#[test]
fn fire_nation_turret_lends_firebending_and_fires_at_fifty() {
    let mut g = pod(2);
    let turret = ready(&mut g, 0, catalog::fire_nation_turret());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    begin_combat(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
    declare(&mut g, &[(bear, 1)]);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2);
    g.battlefield_find_mut(turret).unwrap().add_counters(crabomination::card::CounterType::Charge, 50);
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: turret,
        ability_index: 1,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("remove fifty");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 50);
    assert_eq!(g.battlefield_find(turret).unwrap().counter_count(crabomination::card::CounterType::Charge), 0);
}

/// Commander Liara Portyr (Fire Lord Zuko's list): attacking two players
/// exiles two cards, castable this turn, and exile casts cost {2} less.
#[test]
fn commander_liara_portyr_scales_with_the_players_attacked() {
    let mut g = pod(3);
    let liara = ready(&mut g, 0, catalog::commander_liara_portyr());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ogres: Vec<CardId> = (0..3).map(|_| g.add_card_to_library(0, catalog::gray_ogre())).collect();
    declare(&mut g, &[(liara, 1), (bear, 2)]);
    let exiled: Vec<CardId> = ogres.iter().copied().filter(|id| g.exile.iter().any(|c| c.id == *id)).collect();
    assert_eq!(exiled.len(), 2, "two players attacked");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    // Gray Ogre is {2}{R}: {2} less leaves {R}.
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    let cast = GameAction::CastFromZoneWithoutPaying {
        card_id: exiled[0],
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    assert!(g.perform_action(cast).is_err(), "the {{R}} still has to be paid");
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: exiled[0],
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Ogre from exile for {R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(exiled[0]).is_some());
}

/// Fire Lord Ozai (Fire Lord Zuko's list): the attack sacrifice adds {R} per
/// point of the victim's power; {6} exiles each opponent's top card and only
/// one of them may be played.
#[test]
fn fire_lord_ozai_feeds_on_a_sacrifice_and_steals_one_card() {
    let mut g = pod(3);
    let ozai = ready(&mut g, 0, catalog::fire_lord_ozai());
    let ogre = ready(&mut g, 0, catalog::gray_ogre());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    declare(&mut g, &[(ozai, 1)]);
    assert!(g.battlefield_find(ogre).is_none(), "sacrificed");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2);

    let mut g = pod(3);
    let ozai = ready(&mut g, 0, catalog::fire_lord_ozai());
    let a = g.add_card_to_library(1, catalog::grizzly_bears());
    let b = g.add_card_to_library(2, catalog::grizzly_bears());
    flood(&mut g);
    activate(&mut g, ozai, 0);
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: a,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("the first free cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).map(|c| c.controller), Some(0));
    assert!(g
        .perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: b,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(), "only one of those cards");
}

/// Iroh, Dragon of the West (Fire Lord Zuko's list): a creature with a
/// counter gains firebending 2 at the beginning of combat; one without doesn't.
#[test]
fn iroh_dragon_of_the_west_lights_up_countered_creatures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::iroh_dragon_of_the_west());
    let grown = ready(&mut g, 0, catalog::grizzly_bears());
    let plain = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(grown).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 1);
    begin_combat(&mut g);
    declare(&mut g, &[(plain, 1)]);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 0);
    let mut g2 = pod(2);
    ready(&mut g2, 0, catalog::iroh_dragon_of_the_west());
    let grown = ready(&mut g2, 0, catalog::grizzly_bears());
    g2.battlefield_find_mut(grown).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 1);
    begin_combat(&mut g2);
    declare(&mut g2, &[(grown, 1)]);
    assert_eq!(g2.players[0].mana_pool.amount(Color::Red), 2);
}

/// The Legend of Roku (Fire Lord Zuko's list): I exiles three to play, II
/// adds a mana, III returns it as Avatar Roku (CR 714.2 / 712).
#[test]
fn the_legend_of_roku_becomes_avatar_roku() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::mountain());
    }
    let saga = g.add_card_to_hand(0, catalog::the_legend_of_roku());
    flood(&mut g);
    cast_x(&mut g, saga, None);
    assert_eq!(g.exile.iter().filter(|c| c.may_play_until.is_some()).count(), 3);
    g.players[0].mana_pool = Default::default();
    g.saga_advance(saga);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1);
    g.saga_advance(saga);
    drain_stack(&mut g);
    let roku = g.battlefield_find(saga).expect("returned transformed");
    assert_eq!(roku.definition.name, "Avatar Roku");
}

fn attach(g: &mut GameState, gear: CardId, host: CardId) {
    g.battlefield_find_mut(gear).unwrap().attached_to = Some(host);
}

fn attached(g: &GameState, host: CardId, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.attached_to == Some(host) && c.definition.name == name).count()
}

/// Arna Kennerüd (Arna's list): a modified attacker's counters double (CR
/// 701.10) and its nontoken Equipment is copied onto it (CR 707.2); an
/// unmodified attacker gets nothing.
#[test]
fn arna_doubles_counters_and_copies_attachments() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::arna_kennerud_skycaptain());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let plain = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 2);
    let gear = g.add_card_to_battlefield(0, catalog::bonesplitter());
    attach(&mut g, gear, bear);
    declare(&mut g, &[(bear, 1), (plain, 1)]);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 4);
    assert_eq!(attached(&g, bear, "Bonesplitter"), 2, "the original and a token copy");
    assert!(g.battlefield.iter().all(|c| c.attached_to != Some(plain)));
}

/// Assassin Gauntlet (Arna's list): the ETB taps the opponent's team and
/// attaches to your creature; a connecting host loots.
#[test]
fn assassin_gauntlet_taps_a_board_and_loots() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let foes: Vec<CardId> = (0..2).map(|_| ready(&mut g, 1, catalog::grizzly_bears())).collect();
    g.add_card_to_library(0, catalog::island());
    let gauntlet = g.add_card_to_hand(0, catalog::assassin_gauntlet());
    flood(&mut g);
    cast_x(&mut g, gauntlet, None);
    assert!(foes.iter().all(|f| g.battlefield_find(*f).unwrap().tapped));
    assert_eq!(g.battlefield_find(gauntlet).unwrap().attached_to, Some(bear));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    let hand = g.players[0].hand.len();
    connect(&mut g, bear);
    assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Biorganic Carapace (Arna's list): it attaches as it enters, and a hit
/// draws one card per modified creature you control (CR 700.9).
#[test]
fn biorganic_carapace_draws_per_modified_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    let other = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(other).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 1);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let carapace = g.add_card_to_hand(0, catalog::biorganic_carapace());
    flood(&mut g);
    cast_x(&mut g, carapace, None);
    let host = g.battlefield_find(carapace).unwrap().attached_to.expect("attached on entry");
    let hand = g.players[0].hand.len();
    connect(&mut g, host);
    let want = if host == other { 1 } else { 2 };
    assert_eq!(g.players[0].hand.len(), hand + want, "the host and the countered Bear are modified");
}

/// Ardenn (Arna's list): at the beginning of combat your Equipment gathers
/// on the target creature.
#[test]
fn ardenn_gathers_equipment_on_one_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ardenn_intrepid_archaeologist());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::serra_angel());
    let s1 = g.add_card_to_battlefield(0, catalog::bonesplitter());
    let s2 = g.add_card_to_battlefield(0, catalog::bonesplitter());
    attach(&mut g, s1, a);
    begin_combat(&mut g);
    let hosts: Vec<Option<CardId>> = [s1, s2].iter().map(|s| g.battlefield_find(*s).unwrap().attached_to).collect();
    assert!(hosts.iter().all(|h| h.is_some()), "{hosts:?}");
    assert_eq!(hosts[0], hosts[1], "both on one creature");
}

/// Halvar, God of Battle (Arna's list): equipped creatures have double
/// strike; Sword of the Realms (the MDFC back) returns its dead host to hand.
#[test]
fn halvar_and_sword_of_the_realms() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    ready(&mut g, 0, catalog::halvar_god_of_battle());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let gear = g.add_card_to_battlefield(0, catalog::bonesplitter());
    attach(&mut g, gear, bear);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::DoubleStrike));

    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sword = g.add_card_to_hand(0, catalog::halvar_god_of_battle());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellBack { card_id: sword, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast the Sword");
    drain_stack(&mut g);
    g.perform_action(GameAction::Equip { equipment: sword, target: bear }).expect("equip");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 4);
    assert!(c.keywords().contains(&Keyword::Vigilance));
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.perform_action(GameAction::CastSpell { card_id: murder, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("murder");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "back to its owner's hand");
}

/// A Saga of seat 0's, cast and drained through chapter I.
fn saga(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    flood(g);
    cast_x(g, id, None);
    id
}

/// Tom Bombadil (Terra's list): four lore counters among your Sagas give him
/// hexproof and indestructible; a Saga's final chapter (CR 714.2c) digs up the
/// next Saga, once each turn.
#[test]
fn tom_bombadil_chains_sagas_once_a_turn() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let tom = ready(&mut g, 0, catalog::tom_bombadil());
    let next = g.add_card_to_library(0, catalog::fable_of_the_mirror_breaker());
    g.add_card_to_library(0, catalog::mountain());
    let a = saga(&mut g, catalog::the_apprentices_folly());
    let b = saga(&mut g, catalog::fable_of_the_mirror_breaker());
    assert!(!g.computed_permanent(tom).unwrap().keywords().contains(&Keyword::Hexproof), "two lore counters");
    g.saga_advance(b);
    drain_stack(&mut g);
    g.saga_advance(a);
    drain_stack(&mut g);
    let kw = g.computed_permanent(tom).unwrap().keywords().to_vec();
    assert!(kw.contains(&Keyword::Hexproof) && kw.contains(&Keyword::Indestructible), "four lore counters");
    g.saga_advance(a);
    drain_stack(&mut g);
    assert!(g.battlefield_find(next).is_some(), "the Folly's III found the Fable");
    g.saga_advance(b);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), 1, "once each turn: the Mountain stays put");
}

/// The Apprentice's Folly (Terra's list): a hasty, nonlegendary Reflection
/// copy (CR 707.9b); a creature sharing a name with your token is no longer a
/// legal target; III sacrifices every Reflection.
#[test]
fn the_apprentices_folly_reflects_then_shatters() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = pod(2);
    let tom = ready(&mut g, 0, catalog::tom_bombadil());
    let folly = saga(&mut g, catalog::the_apprentices_folly());
    let copies: Vec<CardId> = g.battlefield.iter().filter(|c| c.is_token).map(|c| c.id).collect();
    assert_eq!(copies.len(), 1);
    let cp = g.computed_permanent(copies[0]).unwrap();
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Reflection));
    assert!(cp.keywords().contains(&Keyword::Haste));
    assert!(g.battlefield_find(tom).is_some(), "not legendary: no legend-rule loss");
    g.saga_advance(folly);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1, "Tom shares a token's name");
    g.saga_advance(folly);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 0);
}

/// The Kami War (Terra's list): I exiles, II bounces and makes each opponent
/// discard, III flips it; O-Kagachi's attack returns the defending player's
/// pick and grows by its mana value.
#[test]
fn the_kami_war_becomes_o_kagachi() {
    let mut g = pod(3);
    let foe = ready(&mut g, 1, catalog::serra_angel());
    for seat in [1, 2] {
        g.add_card_to_hand(seat, catalog::island());
    }
    let war = saga(&mut g, catalog::the_kami_war());
    assert!(g.exile.iter().any(|c| c.id == foe));
    g.saga_advance(war);
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty() && g.players[2].hand.is_empty(), "each opponent discarded");
    assert!(g.exile.iter().any(|c| c.id == foe), "a slot saying \"permanent\" never reaches exile (CR 109.2)");
    g.saga_advance(war);
    drain_stack(&mut g);
    let kagachi = g.battlefield_find(war).expect("returned transformed");
    assert_eq!(kagachi.definition.name, "O-Kagachi Made Manifest");
    g.clear_sickness(war);
    let ogre = g.add_card_to_graveyard(0, catalog::gray_ogre());
    g.add_card_to_graveyard(0, catalog::mountain());
    declare(&mut g, &[(war, 2)]);
    assert!(g.players[0].hand.iter().any(|c| c.id == ogre), "the only nonland card");
    assert_eq!(g.computed_permanent(war).unwrap().power, 9);
}

/// Moonmist (Terra's list): Humans transform (a modal DFC can't, CR
/// 701.27c), and only Werewolves and Wolves deal combat damage (CR 615).
#[test]
fn moonmist_transforms_humans_and_fogs_the_rest() {
    let mut g = pod(2);
    let delver = ready(&mut g, 0, catalog::delver_of_secrets());
    let esika = ready(&mut g, 0, catalog::esika_god_of_the_tree());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let mist = g.add_card_to_hand(0, catalog::moonmist());
    flood(&mut g);
    cast_x(&mut g, mist, None);
    assert!(g.battlefield_find(delver).unwrap().transformed, "Delver is a Human");
    let mut events = vec![];
    g.transform_permanent(esika, &mut events);
    assert!(!g.battlefield_find(esika).unwrap().transformed, "a modal DFC doesn't transform");
    let life = g.players[1].life;
    connect(&mut g, bear);
    assert_eq!(g.players[1].life, life, "the Bear's damage was prevented");
}

