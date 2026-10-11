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
/// hexproof and indestructible; a Saga's final chapter (CR 714.2e) digs up the
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

/// Moonmist (Terra's list): Humans transform (CR 701.27c: a single-faced
/// one doesn't), and only Werewolves and Wolves deal combat damage (CR 615).
#[test]
fn moonmist_transforms_humans_and_fogs_the_rest() {
    let mut g = pod(2);
    let delver = ready(&mut g, 0, catalog::delver_of_secrets());
    let human = ready(&mut g, 0, catalog::elite_vanguard());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let mist = g.add_card_to_hand(0, catalog::moonmist());
    flood(&mut g);
    cast_x(&mut g, mist, None);
    assert!(g.battlefield_find(delver).unwrap().transformed, "Delver is a Human");
    assert!(!g.battlefield_find(human).unwrap().transformed);
    let life = g.players[1].life;
    connect(&mut g, bear);
    assert_eq!(g.players[1].life, life, "the Bear's damage was prevented");
}

/// Djeru and Hazoret (Atreus's list): hellbent vigilance and haste; the attack
/// exiles a legendary creature card from the top six to cast free this turn.
#[test]
fn djeru_and_hazoret_finds_a_free_legend() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let djeru = ready(&mut g, 0, catalog::djeru_and_hazoret());
    let legend = g.add_card_to_library(0, catalog::surtr_fiery_jotun());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::mountain());
    }
    g.add_card_to_hand(0, catalog::mountain());
    let kw = g.computed_permanent(djeru).unwrap().keywords().to_vec();
    assert!(kw.contains(&Keyword::Haste) && kw.contains(&Keyword::Vigilance), "one card in hand");
    g.add_card_to_hand(0, catalog::mountain());
    assert!(!g.computed_permanent(djeru).unwrap().keywords().contains(&Keyword::Haste));
    declare(&mut g, &[(djeru, 1)]);
    assert!(g.exile.iter().any(|c| c.id == legend));
    assert_eq!(g.players[0].library.len(), 3, "the rest went to the bottom");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: legend,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free");
    drain_stack(&mut g);
    assert!(g.battlefield_find(legend).is_some());
}

/// Katara, Waterbending Master (Atreus's list): a spell on an opponent's turn
/// is an experience counter; the attack loots one card per counter.
#[test]
fn katara_waterbending_master_banks_experience() {
    let mut g = pod(2);
    let katara = ready(&mut g, 0, catalog::katara_waterbending_master());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let opt = g.add_card_to_hand(0, catalog::opt());
    g.active_player_idx = 1;
    flood(&mut g);
    cast_x(&mut g, opt, None);
    assert_eq!(g.players[0].experience, 1);
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    declare(&mut g, &[(katara, 1)]);
    assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
}

/// Lae'zel (Atreus's list): CR 614.16 — one more of each kind on a creature
/// or planeswalker you control and on you; an artifact gets none.
#[test]
fn laezel_adds_one_counter_of_each_kind() {
    use crabomination::card::CounterType;
    use crabomination::effect::Value;
    let mut g = pod(2);
    ready(&mut g, 0, catalog::laezel_vlaakiths_champion());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let rock = ready(&mut g, 0, catalog::mind_stone());
    let jace = ready(&mut g, 0, catalog::jace_the_mind_sculptor());
    let loyalty = g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty);
    for (what, kind) in [(bear, CounterType::PlusOnePlusOne), (rock, CounterType::Charge), (jace, CounterType::Loyalty)] {
        run(&mut g, Effect::AddCounter { what: Selector::ExactObjects(vec![what]), kind, amount: Value::ONE });
    }
    let n = |g: &GameState, id, k| g.battlefield_find(id).unwrap().counter_count(k);
    assert_eq!(n(&g, bear, CounterType::PlusOnePlusOne), 2);
    assert_eq!(n(&g, rock, CounterType::Charge), 1, "an artifact isn't Lae'zel's");
    assert_eq!(n(&g, jace, CounterType::Loyalty), loyalty + 2);
    run(&mut g, Effect::AddExperience(Value::ONE));
    assert_eq!(g.players[0].experience, 2, "you get one more too");
}

/// Resolve `effect` as seat 0's.
fn run(g: &mut GameState, effect: Effect) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&effect, &ctx).expect("effect");
}

/// Reidane // Valkmira (Atreus's list): an opponent's noncreature spell of
/// mana value 4+ costs {2} more; Valkmira shaves 1 off an opponent's damage
/// to you (CR 615).
#[test]
fn reidane_taxes_and_valkmira_shields() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::reidane_god_of_the_worthy());
    let opp_walker = g.add_card_to_hand(1, catalog::jace_the_mind_sculptor());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.players[1].mana_pool.add_colorless(2);
    let cast = |id| GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None };
    assert!(g.perform_action(cast(opp_walker)).is_err(), "{{2}}{{U}}{{U}} + {{2}}");
    g.players[1].mana_pool.add_colorless(2);
    g.perform_action(cast(opp_walker)).expect("with the tax paid");

    let mut g = pod(2);
    let valkmira = g.add_card_to_hand(0, catalog::reidane_god_of_the_worthy());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellBack { card_id: valkmira, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Valkmira");
    drain_stack(&mut g);
    let life = g.players[0].life;
    bolt_player_zero(&mut g);
    assert_eq!(g.players[0].life, life - 2);
}

/// Seat 1 bolts seat 0, paying the ward {1} Valkmira may demand.
fn bolt_player_zero(g: &mut GameState) {
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(g);
}

/// Surtr, Fiery Jötun (Atreus's list): a historic spell (CR 700.6) is 3
/// damage; a plain creature spell isn't.
#[test]
fn surtr_burns_on_historic_spells() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::surtr_fiery_jotun());
    flood(&mut g);
    let life = g.players[1].life;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_x(&mut g, bear, None);
    assert_eq!(g.players[1].life, life);
    let rock = g.add_card_to_hand(0, catalog::mind_stone());
    cast_x(&mut g, rock, None);
    assert_eq!(g.players[1].life, life - 3);
}

/// World at War (Atreus's list): CR 500.8 — cast in the first main phase, the
/// added combat follows the second main phase, and it untaps the creatures
/// that attacked.
#[test]
fn world_at_war_attacks_again_after_the_second_main() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let war = g.add_card_to_hand(0, catalog::world_at_war());
    flood(&mut g);
    cast_x(&mut g, war, None);
    let life = g.players[1].life;
    let mut combats = Vec::new();
    for _ in 0..60 {
        if g.step == TurnStep::DeclareAttackers && combats.last() != Some(&g.combat_phases_this_turn) {
            combats.push(g.combat_phases_this_turn);
            assert!(!g.battlefield_find(bear).unwrap().tapped, "combat {}", g.combat_phases_this_turn);
            g.priority.player_with_priority = 0;
            g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
                .expect("attack");
        }
        if g.step == TurnStep::End {
            break;
        }
        g.priority.player_with_priority = g.active_player_idx;
        let _ = g.perform_action(GameAction::PassPriority);
    }
    assert_eq!(combats, vec![1, 2]);
    assert_eq!(g.players[1].life, life - 4, "the Bear connected twice");
}

/// Mjölnir, Storm Hammer (Kratos's list): it attaches to a legend as it
/// enters; the attack taps and stuns a defender's creature, then each
/// opponent takes 1 per tapped creature they control.
#[test]
fn mjolnir_storm_hammer_stuns_and_strikes() {
    use crabomination::card::CounterType;
    let mut g = pod(3);
    let legend = ready(&mut g, 0, catalog::surtr_fiery_jotun());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let other = ready(&mut g, 2, catalog::grizzly_bears());
    g.battlefield_find_mut(other).unwrap().tapped = true;
    let hammer = g.add_card_to_hand(0, catalog::mjolnir_storm_hammer());
    flood(&mut g);
    cast_x(&mut g, hammer, None);
    assert_eq!(g.battlefield_find(hammer).unwrap().attached_to, Some(legend));
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    declare(&mut g, &[(legend, 1)]);
    let bear = g.battlefield_find(foe).unwrap();
    assert!(bear.tapped);
    assert_eq!(bear.counter_count(CounterType::Stun), 1);
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 1, l2 - 1));
}

/// Gornog, the Red Reaper (Kratos's list): a Warrior's attack turns a
/// defender's creature into a Coward (types overwritten, no duration); a
/// Coward can't block Warriors, and each one pumps attacking Warriors.
#[test]
fn gornog_makes_cowards_that_cant_block_warriors() {
    use crabomination::card::CreatureType;
    let mut g = pod(2);
    let gornog = ready(&mut g, 0, catalog::gornog_the_red_reaper());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    declare(&mut g, &[(gornog, 1)]);
    let types = g.computed_permanent(foe).unwrap().subtypes().creature_types.clone();
    assert_eq!(types, vec![CreatureType::Coward]);
    assert_eq!(g.computed_permanent(gornog).unwrap().power, 3, "+1/+0 per Coward");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(foe, gornog)])).is_err(), "a Coward can't block a Warrior");
}

/// Kazuul, Tyrant of the Cliffs (Kratos's list): the attacker's controller
/// is asked for {3}; unpaid, Kazuul's controller gets a 3/3 Ogre.
#[test]
fn kazuul_taxes_attackers_or_makes_ogres() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::kazuul_tyrant_of_the_cliffs());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Ogre"), 1, "no mana to pay the {{3}}");
}

/// Spider-Punk (Kratos's list): spells and abilities can't be countered — a
/// Counterspell finds nothing to counter and a ward trigger lets the ability
/// through (CR 702.21a) — and damage can't be prevented.
#[test]
fn spider_punk_stops_spell_and_ability_counters() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::spider_punk());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("bear");
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Counterspell");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the Bear resolved");

    // An ability: Stifle on Elvish Visionary's enters trigger.
    g.add_card_to_library(0, catalog::island());
    let elf = g.add_card_to_battlefield(0, catalog::elvish_visionary());
    g.fire_self_etb_triggers(elf, 0);
    let draw = g.top_ability_of(elf).expect("the Visionary's draw is on the stack");
    let hand = g.players[0].hand.len();
    let stifle = g.add_card_to_hand(1, catalog::stifle());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: stifle,
        target: Some(Target::Permanent(draw)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Stifle");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the trigger resolved and drew");
}

/// Maximum Carnage (Kratos's list): I goads every creature (CR 701.15b), II
/// adds {R}{R}{R}, III deals 5 to each opponent.
#[test]
fn maximum_carnage_runs_its_chapters() {
    let mut g = pod(3);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let carnage = saga(&mut g, catalog::maximum_carnage());
    assert!(g.goaded_by_player(g.battlefield_find(bear).unwrap(), 0));
    g.players[0].mana_pool = Default::default();
    g.saga_advance(carnage);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
    let (l0, l1, l2) = (g.players[0].life, g.players[1].life, g.players[2].life);
    g.saga_advance(carnage);
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life, g.players[2].life), (l0, l1 - 5, l2 - 5));
}

/// Rhythm of the Wild says "creature spells YOU control": an opponent's
/// creature spell stays counterable (Leyline of Lifeforce's symmetric static
/// was the wrong one).
#[test]
fn rhythm_of_the_wild_protects_only_its_controllers_creatures() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::rhythm_of_the_wild());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("bear");
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counterspell");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "countered");
}

/// Eddie Brock (Eddie's list): the ETB reanimates a mana value 1 creature;
/// {3}{B}{R}{G} transforms him, and Venom's attack trades a creature for X
/// cards and a permanent of mana value X or less from hand.
#[test]
fn eddie_brock_becomes_venom() {
    let mut g = pod(2);
    let elf = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let eddie = g.add_card_to_hand(0, catalog::eddie_brock());
    flood(&mut g);
    cast_x(&mut g, eddie, None);
    assert!(g.battlefield_find(elf).is_some(), "mana value 1 came back");
    g.clear_sickness(eddie);
    activate(&mut g, eddie, 0);
    assert_eq!(g.battlefield_find(eddie).unwrap().definition.name, "Venom, Lethal Protector");
    let ogre = ready(&mut g, 0, catalog::gray_ogre());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.clear_sickness(eddie);
    let hand = g.players[0].hand.len();
    declare(&mut g, &[(eddie, 1)]);
    let sacrificed = g.battlefield_find(ogre).is_none() || g.battlefield_find(elf).is_none();
    assert!(sacrificed);
    assert!(g.players[0].hand.len() + usize::from(g.battlefield_find(bears).is_some()) > hand, "drew");
}

/// Carnage, Crimson Chaos (Eddie's list): the reanimated creature must attack
/// and is sacrificed once it connects.
#[test]
fn carnage_returns_a_suicidal_attacker() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let ogre = g.add_card_to_graveyard(0, catalog::gray_ogre());
    let carnage = g.add_card_to_hand(0, catalog::carnage_crimson_chaos());
    flood(&mut g);
    cast_x(&mut g, carnage, None);
    assert!(g.computed_permanent(ogre).unwrap().keywords().contains(&Keyword::MustAttack));
    g.clear_sickness(ogre);
    connect(&mut g, ogre);
    assert!(g.battlefield_find(ogre).is_none(), "sacrificed after connecting");
}

/// Gwenom, Remorseless (Eddie's list): attacking opens the library top for
/// the turn, paying life instead of mana (CR 401.6).
#[test]
fn gwenom_plays_off_the_top_for_life() {
    let mut g = pod(2);
    let gwenom = ready(&mut g, 0, catalog::gwenom_remorseless());
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    declare(&mut g, &[(gwenom, 1)]);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    g.players[0].mana_pool = Default::default();
    let life = g.players[0].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: top, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast off the top");
    drain_stack(&mut g);
    assert!(g.battlefield_find(top).is_some());
    assert_eq!(g.players[0].life, life - 2, "two life for a mana value 2 spell");
}

/// Venom, Deadly Devourer (Eddie's list): exiling a graveyard creature grows
/// a Symbiote by its toughness.
#[test]
fn venom_deadly_devourer_feeds_on_graveyards() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let venom = ready(&mut g, 0, catalog::venom_deadly_devourer());
    let angel = g.add_card_to_graveyard(1, catalog::serra_angel());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: venom,
        ability_index: 0,
        target: Some(Target::Permanent(angel)),
        additional_targets: vec![Target::Permanent(venom)],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == angel));
    assert_eq!(g.battlefield_find(venom).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

/// Behold the Sinister Six! (Eddie's list): up to six creature cards come
/// back from your graveyard.
#[test]
fn behold_the_sinister_six_reanimates_a_team() {
    let mut g = pod(2);
    let dead: Vec<CardId> = [catalog::grizzly_bears(), catalog::gray_ogre(), catalog::serra_angel()]
        .into_iter()
        .map(|d| g.add_card_to_graveyard(0, d))
        .collect();
    let six = g.add_card_to_hand(0, catalog::behold_the_sinister_six());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: six,
        target: Some(Target::Permanent(dead[0])),
        additional_targets: vec![Target::Permanent(dead[1]), Target::Permanent(dead[2])],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(dead.iter().all(|id| g.battlefield_find(*id).is_some()));
}

/// Behold the Sinister Six!: CR 601.2c — "with different names" binds the
/// targets together, so a second Grizzly Bears can't be chosen beside the
/// first (the cast is rejected), while two different names are fine.
#[test]
fn behold_the_sinister_six_rejects_two_targets_with_one_name() {
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let c = g.add_card_to_graveyard(0, catalog::gray_ogre());
    let six = g.add_card_to_hand(0, catalog::behold_the_sinister_six());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    let cast = |g: &mut GameState, extra: CardId| {
        g.perform_action(GameAction::CastSpell {
            card_id: six,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(extra)],
            mode: None,
            x_value: None,
        })
    };
    assert!(cast(&mut g, b).is_err(), "two Grizzly Bears share a name");
    cast(&mut g, c).expect("different names");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(c).is_some());
    assert!(g.battlefield_find(b).is_none());
}

fn set_loyalty(g: &mut GameState, id: CardId, n: u32) {
    use crabomination::card::CounterType;
    let c = g.battlefield_find_mut(id).unwrap();
    let have = c.counter_count(CounterType::Loyalty);
    c.add_counters(CounterType::Loyalty, n);
    c.remove_counters(CounterType::Loyalty, have);
}

/// Sanctum Lurker (Bolas's list): CR 701.71a — empower Jace 1 makes a Jace
/// token with one loyalty; a planeswalker at 0 loyalty stays (CR 704.5i is
/// waived) and has the granted +2.
#[test]
fn sanctum_lurker_empowers_jace_and_keeps_walkers() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let lurker = g.add_card_to_hand(0, catalog::sanctum_lurker());
    flood(&mut g);
    cast_x(&mut g, lurker, None);
    let jace = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Jace").map(|c| c.id).expect("a Jace token");
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 1);
    loyalty(&mut g, jace, 0, None);
    g.check_state_based_actions();
    assert!(g.battlefield_find(jace).is_some(), "0 loyalty, still here");
    g.battlefield_find_mut(jace).unwrap().loyalty_uses_this_turn = 0;
    let life = g.players[1].life;
    loyalty(&mut g, jace, 2, None);
    assert_eq!(g.players[1].life, life - 1, "the granted +2");
}

/// Liliana's Talent (Bolas's list): the enchanted planeswalker's granted −8
/// puts every graveyard's creature cards onto the battlefield under you.
#[test]
fn lilianas_talent_grants_a_mass_reanimation() {
    let mut g = pod(3);
    let bolas = ready(&mut g, 0, catalog::nicol_bolas_the_deceiver());
    set_loyalty(&mut g, bolas, 8);
    let a = g.add_card_to_graveyard(1, catalog::serra_angel());
    let b = g.add_card_to_graveyard(2, catalog::grizzly_bears());
    let talent = g.add_card_to_hand(0, catalog::lilianas_talent());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: talent, target: Some(Target::Permanent(bolas)), additional_targets: vec![], mode: None, x_value: None })
        .expect("talent");
    drain_stack(&mut g);
    loyalty(&mut g, bolas, 3, None);
    for id in [a, b] {
        assert_eq!(g.battlefield_find(id).map(|c| c.controller), Some(0));
    }
}

/// Nicol Bolas, Planeswalker (Bolas's list): +3 destroys a noncreature
/// permanent; −2 steals a creature.
#[test]
fn nicol_bolas_planeswalker_destroys_and_steals() {
    let mut g = pod(2);
    let bolas = ready(&mut g, 0, catalog::nicol_bolas_planeswalker());
    let rock = ready(&mut g, 1, catalog::mind_stone());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    loyalty(&mut g, bolas, 0, Some(Target::Permanent(rock)));
    assert!(g.battlefield_find(rock).is_none());
    g.battlefield_find_mut(bolas).unwrap().loyalty_uses_this_turn = 0;
    loyalty(&mut g, bolas, 1, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
}

/// Nicol Bolas, the Deceiver (Bolas's list): −3 destroys a creature and draws.
#[test]
fn nicol_bolas_the_deceiver_kills_and_draws() {
    let mut g = pod(2);
    let bolas = ready(&mut g, 0, catalog::nicol_bolas_the_deceiver());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    loyalty(&mut g, bolas, 1, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Dark Intimations (Bolas's list): from the graveyard, casting a Bolas
/// planeswalker exiles it and the planeswalker enters with one more loyalty.
#[test]
fn dark_intimations_feeds_a_bolas_walker() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let dark = g.add_card_to_graveyard(0, catalog::dark_intimations());
    let bolas = g.add_card_to_hand(0, catalog::nicol_bolas_the_deceiver());
    flood(&mut g);
    cast_x(&mut g, bolas, None);
    assert!(g.exile.iter().any(|c| c.id == dark));
    assert_eq!(g.battlefield_find(bolas).unwrap().counter_count(CounterType::Loyalty), 6);
}

/// Hour of Devastation (Bolas's list): indestructible creatures die, a
/// non-Bolas planeswalker takes 5, a Bolas one is spared.
#[test]
fn hour_of_devastation_spares_only_bolas() {
    let mut g = pod(2);
    let myr = ready(&mut g, 1, catalog::darksteel_myr());
    let bolas = ready(&mut g, 0, catalog::nicol_bolas_the_deceiver());
    let lili = ready(&mut g, 1, catalog::liliana_of_the_veil());
    let hour = g.add_card_to_hand(0, catalog::hour_of_devastation());
    flood(&mut g);
    cast_x(&mut g, hour, None);
    assert!(g.battlefield_find(myr).is_none(), "lost indestructible");
    assert!(g.battlefield_find(bolas).is_some());
    assert!(g.battlefield_find(lili).is_none(), "3 loyalty, 5 damage");
}

/// Angel of Suffering (Shilgengar's list): CR 615 — damage to you is
/// prevented and you mill twice as many cards.
#[test]
fn angel_of_suffering_turns_damage_into_mill() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::angel_of_suffering());
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life);
    assert_eq!(g.players[0].library.len(), 4, "six cards milled");
}

/// Firja, Judge of Valor (Shilgengar's list): the second spell of a turn
/// digs three: one to hand, two to the graveyard.
#[test]
fn firja_digs_on_the_second_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::firja_judge_of_valor());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    flood(&mut g);
    let a = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_x(&mut g, a, None);
    assert_eq!(g.players[0].library.len(), 3, "the first spell does nothing");
    let b = g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    cast_x(&mut g, b, None);
    assert_eq!(g.players[0].hand.len(), hand, "cast one, took one");
    assert_eq!(g.players[0].graveyard.len(), 2);
}

/// Renegade Reaper (Shilgengar's list): milling an Angel gains 4 life.
#[test]
fn renegade_reaper_gains_for_a_milled_angel() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::serra_angel());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let life = g.players[0].life;
    let reaper = g.add_card_to_hand(0, catalog::renegade_reaper());
    flood(&mut g);
    cast_x(&mut g, reaper, None);
    assert_eq!(g.players[0].graveyard.len(), 4);
    assert_eq!(g.players[0].life, life + 4);
}

/// Sanctuary Warden (Shilgengar's list): it enters with two shield counters
/// and spends one to draw and make a Citizen.
#[test]
fn sanctuary_warden_cashes_in_a_shield_counter() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let warden = g.add_card_to_hand(0, catalog::sanctuary_warden());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast_x(&mut g, warden, None);
    assert_eq!(g.battlefield_find(warden).unwrap().counter_count(CounterType::Shield), 1);
    assert_eq!(g.players[0].hand.len(), hand, "cast it, drew one");
    assert_eq!(named(&g, "Citizen"), 1);
}

/// Rampage of the Valkyries (Shilgengar's list): its Angel dying makes each
/// other player sacrifice a creature.
#[test]
fn rampage_of_the_valkyries_punishes_angel_deaths() {
    let mut g = pod(3);
    let foes: Vec<CardId> = [1, 2].iter().map(|&s| ready(&mut g, s, catalog::grizzly_bears())).collect();
    let rampage = g.add_card_to_hand(0, catalog::rampage_of_the_valkyries());
    flood(&mut g);
    cast_x(&mut g, rampage, None);
    let angel = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Angel").map(|c| c.id).expect("angel");
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: murder, target: Some(Target::Permanent(angel)), additional_targets: vec![], mode: None, x_value: None })
        .expect("murder");
    drain_stack(&mut g);
    assert!(foes.iter().all(|f| g.battlefield_find(*f).is_none()));
}

/// Lich-Knights' Conquest (Shilgengar's list): each sacrificed artifact,
/// enchantment or token returns a creature card.
#[test]
fn lich_knights_conquest_trades_permanents_for_creatures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mind_stone());
    ready(&mut g, 0, catalog::mind_stone());
    let dead: Vec<CardId> = [catalog::serra_angel(), catalog::gray_ogre()].into_iter().map(|d| g.add_card_to_graveyard(0, d)).collect();
    let conquest = g.add_card_to_hand(0, catalog::lich_knights_conquest());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    cast_x(&mut g, conquest, None);
    assert_eq!(named(&g, "Mind Stone"), 0);
    assert!(dead.iter().all(|id| g.battlefield_find(*id).is_some()), "one creature per sacrifice");
}

/// Raphael and Slash (Norin's list): alliance pings — 1 to a target
/// opponent, 2 to each — and Slash's attack makes a Mutant.
#[test]
fn raphael_and_slash_ping_on_alliance() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::raphael_tough_turtle());
    let slash = ready(&mut g, 0, catalog::slash_reptile_rampager());
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g);
    cast_x(&mut g, bear, None);
    assert_eq!(l1 + l2 - g.players[1].life - g.players[2].life, 2 + 2 + 1);
    declare(&mut g, &[(slash, 1)]);
    assert_eq!(named(&g, "Mutant"), 1);
}

/// Mechanized Warfare (Norin's list): CR 614.5 — a red source and an
/// artifact source each deal 1 more to an opponent; a green one doesn't.
#[test]
fn mechanized_warfare_boosts_red_and_artifact_sources() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mechanized_warfare());
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 4);
    // Steel Overseer is a 1/1 artifact creature: 1 + 1.
    let overseer = ready(&mut g, 0, catalog::steel_overseer());
    let life = g.players[1].life;
    connect(&mut g, overseer);
    assert_eq!(g.players[1].life, life - 2);
}

/// CR 120.8 / 614.7a — a source that would deal 0 damage deals none, so an
/// increase has nothing to replace: a 0/1 artifact under Mechanized Warfare
/// deals nothing.
#[test]
fn cr_120_8_zero_damage_is_not_increased() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mechanized_warfare());
    let myr = ready(&mut g, 0, catalog::darksteel_myr());
    let life = g.players[1].life;
    connect(&mut g, myr);
    assert_eq!(g.players[1].life, life);
}

/// Powerbalance (Norin's list): an opponent's spell lets you cast your top
/// card free when the mana values match; a mismatch stays on top.
#[test]
fn powerbalance_matches_mana_values() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::powerbalance());
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    let opp = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: opp, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("opponent's bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(top).is_some_and(|c| c.controller == 0), "mana value 2 for 2");
}

/// Wild Magic Surge (Norin's list): the destroyed creature is replaced by the
/// next permanent card sharing a type from its controller's library.
#[test]
fn wild_magic_surge_replaces_what_it_destroys() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    g.add_card_to_library(1, catalog::island());
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    let surge = g.add_card_to_hand(0, catalog::wild_magic_surge());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: surge, target: Some(Target::Permanent(angel)), additional_targets: vec![], mode: None, x_value: None })
        .expect("surge");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none());
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(1), "the Island was passed over");
}

/// Wheel of Misfortune (Norin's list): with the controller picking 1 and the
/// opponent 0, the controller takes 1 and wheels; the opponent keeps its hand.
#[test]
fn wheel_of_misfortune_wheels_the_high_pickers() {
    let mut g = pod(2);
    for seat in 0..2 {
        for _ in 0..8 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g.add_card_to_hand(1, catalog::island());
    let wheel = g.add_card_to_hand(0, catalog::wheel_of_misfortune());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(1), DecisionAnswer::Amount(0)]));
    let life = g.players[0].life;
    cast_x(&mut g, wheel, None);
    assert_eq!(g.players[0].life, life - 1);
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[1].hand.len(), 1);
}

// ── Katara, the Fearless's list ─────────────────────────────────────────────

fn cast_on(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// Aang and Katara: the enter trigger counts tapped artifacts and/or
/// creatures you control on resolution (CR 608.2h) — two tapped bears and a
/// tapped Sol Ring make three Allies; an untapped bear doesn't count.
#[test]
fn aang_and_katara_count_tapped_artifacts_and_creatures() {
    let mut g = pod(2);
    for _ in 0..2 {
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
    }
    ready(&mut g, 0, catalog::grizzly_bears());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    g.battlefield_find_mut(ring).unwrap().tapped = true;
    let aang = g.add_card_to_hand(0, catalog::aang_and_katara());
    flood(&mut g);
    cast_on(&mut g, aang, None);
    assert_eq!(named(&g, "Ally"), 3);
}

/// Aang, Swift Savior: flash lets him resolve over an opponent's creature
/// spell, and his enter trigger airbends that spell (CR 701.65a — airbend
/// exiles permanents and/or spells; its owner may cast it for {2}).
#[test]
fn aang_swift_savior_airbends_a_spell() {
    let mut g = pod(2);
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("opponent's bear");
    let aang = g.add_card_to_hand(0, catalog::aang_swift_savior());
    flood(&mut g);
    cast_on(&mut g, aang, None);
    let exiled = g.exile.iter().find(|c| c.id == bear).expect("the bear spell was airbent");
    assert!(exiled.may_play_until.is_some(), "its owner may cast it for {{2}}");
    assert!(g.battlefield_find(bear).is_none());
}

/// Aang, Swift Savior: Waterbend {8} transforms him (CR 701.67, CR 701.27a)
/// into Aang and La, whose attack puts a +1/+1 counter on each tapped creature
/// you control (attacking taps them, CR 508.1f).
#[test]
fn aang_swift_savior_waterbends_into_aang_and_la() {
    let mut g = pod(2);
    let aang = ready(&mut g, 0, catalog::aang_swift_savior());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbilityWaterbend {
        card_id: aang, ability_index: 0, target: None, additional_targets: vec![], x_value: None, helpers: vec![],
    })
    .expect("waterbend {8}");
    drain_stack(&mut g);
    let la = g.battlefield_find(aang).unwrap();
    assert!(la.transformed);
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == aang).unwrap().power, 5);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: aang, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}

/// Aang, at the Crossroads: the top five offer a creature with mana value 4
/// or less onto the battlefield (a 5-drop doesn't qualify); another creature
/// leaving schedules the transform for the next upkeep (CR 603.7a delayed
/// trigger), and Aang, Destined Savior earthbends at combat.
#[test]
fn aang_at_the_crossroads_digs_then_transforms_at_upkeep() {
    let mut g = pod(2);
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let aang = g.add_card_to_hand(0, catalog::aang_at_the_crossroads());
    flood(&mut g);
    cast_on(&mut g, aang, None);
    assert!(g.battlefield_find(bear).is_some(), "the bear came in");
    assert!(g.battlefield_find(angel).is_none(), "mana value 5 is too much");
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g);
    cast_on(&mut g, murder, Some(Target::Permanent(bear)));
    assert!(!g.battlefield_find(aang).unwrap().transformed, "not until the upkeep");
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(aang).unwrap().transformed);
}

/// Appa, the Vigilant: an Ally entering pumps the team and gives it flying
/// and vigilance until end of turn (CR 611.2c — creatures you control as the
/// ability resolves).
#[test]
fn appa_the_vigilant_lifts_the_team_on_ally_entry() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::appa_the_vigilant());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ally = g.add_card_to_hand(0, catalog::aang_swift_savior());
    flood(&mut g);
    cast_on(&mut g, ally, None);
    let v = g.compute_battlefield();
    let b = v.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 3));
    assert!(b.keywords().contains(&crabomination::card::Keyword::Flying));
    assert!(b.keywords().contains(&crabomination::card::Keyword::Vigilance));
}

/// Tale of Katara and Toph: a creature's first tap during your turn grows it
/// (the granted trigger, CR 613.1f); an opponent's creature doesn't have it.
#[test]
fn tale_of_katara_and_toph_grows_on_the_first_tap() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::tale_of_katara_and_toph());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    connect(&mut g, bear);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}

/// Tale of Momo: {2} cheaper once a creature left the battlefield under your
/// control this turn (CR 601.2f), and it tutors an Ally from the graveyard.
#[test]
fn tale_of_momo_discounts_after_a_creature_leaves() {
    let mut g = pod(2);
    let ally = g.add_card_to_graveyard(0, catalog::appa_the_vigilant());
    let momo = g.add_card_to_hand(0, catalog::tale_of_momo());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: momo, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "{{W}} alone is short before anything left"
    );
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 3);
    cast_on(&mut g, murder, Some(Target::Permanent(bear)));
    g.players[0].mana_pool.add(Color::White, 1);
    cast_on(&mut g, momo, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == ally));
}

// ── Azlask, the Swelling Scourge's list ─────────────────────────────────────

/// Emrakul's Hatcher: three Eldrazi Spawn on entry (CR 603.6a).
#[test]
fn emrakuls_hatcher_brings_three_spawn() {
    let mut g = pod(2);
    let hatcher = g.add_card_to_hand(0, catalog::emrakuls_hatcher());
    flood(&mut g);
    cast_on(&mut g, hatcher, None);
    assert_eq!(named(&g, "Eldrazi Spawn"), 3);
}

/// Brood Birthing: one Spawn on an empty board, three once a Spawn is there
/// (the condition is read on resolution, CR 608.2c).
#[test]
fn brood_birthing_scales_with_a_spawn() {
    let mut g = pod(2);
    let first = g.add_card_to_hand(0, catalog::brood_birthing());
    let second = g.add_card_to_hand(0, catalog::brood_birthing());
    flood(&mut g);
    cast_on(&mut g, first, None);
    assert_eq!(named(&g, "Eldrazi Spawn"), 1);
    cast_on(&mut g, second, None);
    assert_eq!(named(&g, "Eldrazi Spawn"), 4);
}

/// Minthara, Merciless Soul: a permanent of yours leaving earns an experience
/// counter at your end step (CR 603.4 intervening if), which pumps your
/// creatures +1/+0 (CR 613.4c) and raises her ward: with two counters an
/// opponent's Murder that can't pay {2} is countered (CR 702.21a).
#[test]
fn minthara_earns_experience_and_wards_with_it() {
    let mut g = pod(2);
    let minthara = ready(&mut g, 0, catalog::minthara_merciless_soul());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g);
    cast_on(&mut g, murder, Some(Target::Permanent(bear)));
    to_end_step(&mut g);
    assert_eq!(g.players[0].experience, 1);
    let v = g.compute_battlefield();
    assert_eq!(v.iter().find(|c| c.id == minthara).unwrap().power, 3);

    let mut g = pod(2);
    let minthara = ready(&mut g, 0, catalog::minthara_merciless_soul());
    g.players[0].experience = 2;
    let kill = g.add_card_to_hand(1, catalog::murder());
    g.players[1].mana_pool.add(Color::Black, 3);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: kill, target: Some(Target::Permanent(minthara)), additional_targets: vec![], mode: None, x_value: None })
        .expect("murder");
    drain_stack(&mut g);
    assert!(g.battlefield_find(minthara).is_some(), "ward {{2}} went unpaid");
}

/// Echoes of Eternity: a colorless spell is copied (a permanent spell's copy
/// becomes a token, CR 707.10f), and another colorless permanent's trigger
/// fires an additional time (CR 603.2d): Ichor Wellspring and its token copy
/// each draw twice on entry.
#[test]
fn echoes_of_eternity_copies_and_doubles_colorless() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::echoes_of_eternity());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    flood(&mut g);
    cast_on(&mut g, ring, None);
    assert_eq!(named(&g, "Sol Ring"), 2);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let well = g.add_card_to_hand(0, catalog::ichor_wellspring());
    flood(&mut g);
    cast_on(&mut g, well, None);
    assert_eq!(named(&g, "Ichor Wellspring"), 2);
    assert_eq!(g.players[0].hand.len(), 4);
}

// ── The Mycotyrant's list ───────────────────────────────────────────────────

/// Matzalantli, the Great Door: the transform needs four permanent types
/// among graveyard cards (instants don't count, CR 110.4a); The Core then taps
/// for X mana of one color, X = permanent cards in the graveyard.
#[test]
fn matzalantli_opens_on_four_permanent_types() {
    let mut g = pod(2);
    let door = ready(&mut g, 0, catalog::matzalantli_the_great_door());
    for def in [catalog::grizzly_bears(), catalog::sol_ring(), catalog::island(), catalog::lightning_bolt()] {
        g.add_card_to_graveyard(0, def);
    }
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility { card_id: door, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
            .is_err(),
        "three permanent types and an instant"
    );
    g.add_card_to_graveyard(0, catalog::tale_of_katara_and_toph());
    activate(&mut g, door, 1);
    let core = g.battlefield_find(door).unwrap();
    assert!(core.transformed);
    g.players[0].mana_pool.empty();
    g.battlefield_find_mut(door).unwrap().tapped = false;
    activate(&mut g, door, 0);
    assert_eq!(g.players[0].mana_pool.total(), 4, "four permanent cards");
}

/// Nemata, Primeval Warden: an opponent's dying creature is exiled instead
/// (CR 614.1a), and the "when you do" makes a Saproling.
#[test]
fn nemata_exiles_dying_opponents_creatures_for_saprolings() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::nemata_primeval_warden());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g);
    cast_on(&mut g, murder, Some(Target::Permanent(bear)));
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert_eq!(named(&g, "Saproling"), 1);
}

/// Slimefoot, the Stowaway: {4} makes a Saproling; a Saproling of yours dying
/// drains each opponent for 1 (CR 603.10a leaves-the-battlefield trigger).
#[test]
fn slimefoot_drains_when_a_saproling_dies() {
    let mut g = pod(3);
    let slime = ready(&mut g, 0, catalog::slimefoot_the_stowaway());
    flood(&mut g);
    activate(&mut g, slime, 0);
    let sap = g.battlefield.iter().find(|c| c.definition.name == "Saproling").unwrap().id;
    let murder = g.add_card_to_hand(0, catalog::murder());
    let (me, a, b) = (g.players[0].life, g.players[1].life, g.players[2].life);
    cast_on(&mut g, murder, Some(Target::Permanent(sap)));
    assert_eq!((g.players[0].life, g.players[1].life, g.players[2].life), (me + 1, a - 1, b - 1));
}

/// Sporecrown Thallid: other Fungi and Saprolings get +1/+1; a bear doesn't.
#[test]
fn sporecrown_thallid_pumps_fungi_and_saprolings() {
    let mut g = pod(2);
    let crown = ready(&mut g, 0, catalog::sporecrown_thallid());
    let slime = ready(&mut g, 0, catalog::slimefoot_the_stowaway());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let v = g.compute_battlefield();
    let pt = |id| v.iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)).unwrap();
    assert_eq!(pt(slime), (3, 4));
    assert_eq!(pt(bear), (2, 2));
    assert_eq!(pt(crown), (2, 2));
}

/// Fungal Rebirth: the permanent card returns, and the Saprolings come only
/// once a creature died this turn (CR 608.2c).
#[test]
fn fungal_rebirth_adds_saprolings_after_a_death() {
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let rebirth = g.add_card_to_hand(0, catalog::fungal_rebirth());
    flood(&mut g);
    cast_on(&mut g, rebirth, Some(Target::Permanent(ring)));
    assert!(g.players[0].hand.iter().any(|c| c.id == ring));
    assert_eq!(named(&g, "Saproling"), 0);

    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g);
    cast_on(&mut g, murder, Some(Target::Permanent(bear)));
    let ring2 = g.add_card_to_graveyard(0, catalog::sol_ring());
    let rebirth = g.add_card_to_hand(0, catalog::fungal_rebirth());
    cast_on(&mut g, rebirth, Some(Target::Permanent(ring2)));
    assert_eq!(named(&g, "Saproling"), 2);
}

/// Fungal Sprouting: X Saprolings for the greatest power you control.
#[test]
fn fungal_sprouting_counts_the_biggest_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::serra_angel());
    let sprout = g.add_card_to_hand(0, catalog::fungal_sprouting());
    flood(&mut g);
    cast_on(&mut g, sprout, None);
    assert_eq!(named(&g, "Saproling"), 4);
}

// ── Thranduil, the Elvenking's list ─────────────────────────────────────────

/// Elven Passage: the fetched basic enters tapped, and beholding an Elf (CR
/// 701.4-style reveal-or-choose; free) untaps it.
#[test]
fn elven_passage_untaps_with_an_elf_to_behold() {
    for with_elf in [false, true] {
        let mut g = pod(2);
        let passage = ready(&mut g, 0, catalog::elven_passage());
        let forest = g.add_card_to_library(0, catalog::forest());
        if with_elf {
            g.add_card_to_hand(0, catalog::llanowar_elves());
        }
        let life = g.players[0].life;
        activate(&mut g, passage, 0);
        assert_eq!(g.players[0].life, life - 1);
        assert!(g.battlefield_find(passage).is_none());
        assert_eq!(g.battlefield_find(forest).map(|c| c.tapped), Some(!with_elf));
    }
}

/// Dionus, Elvish Archdruid: an Elf tapping on your turn untaps with a
/// counter, once each turn (CR 603.2h triggers-only-once).
#[test]
fn dionus_untaps_and_grows_elves_once_a_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::dionus_elvish_archdruid());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    activate(&mut g, elf, 0);
    let c = g.battlefield_find(elf).unwrap();
    assert!(!c.tapped, "untapped");
    assert_eq!(c.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
    activate(&mut g, elf, 0);
    let c = g.battlefield_find(elf).unwrap();
    assert!(c.tapped, "only once each turn");
    assert_eq!(c.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}

/// Elrond, Moon-Reader: activating a creature's ability draws, once a turn
/// (CR 603.2h); the flicker returns the exiled permanent at the next end step
/// (CR 603.7).
#[test]
fn elrond_draws_once_and_flickers_till_end_step() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let elrond = ready(&mut g, 0, catalog::elrond_moon_reader());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: elrond, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("flicker");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
    assert!(g.battlefield_find(bear).is_none());
    flood(&mut g);
    activate(&mut g, elrond, 0);
    assert_eq!(g.players[0].hand.len(), 1, "once each turn");
    to_end_step(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0));
}

/// Maralen, Fae Ascendant: entering exiles the top two of an opponent's
/// library; one spell among them with mana value up to your Elves and Faeries
/// may be cast free, once each turn.
#[test]
fn maralen_casts_one_exiled_spell_a_turn() {
    let mut g = pod(2);
    let bolt = g.add_card_to_library(1, catalog::lightning_bolt());
    let shock = g.add_card_to_library(1, catalog::shock());
    let maralen = g.add_card_to_hand(0, catalog::maralen_fae_ascendant());
    flood(&mut g);
    cast_on(&mut g, maralen, None);
    assert!(g.exile.iter().any(|c| c.id == bolt) && g.exile.iter().any(|c| c.id == shock));
    g.players[0].mana_pool.empty();
    let life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("free bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: shock, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "once each turn"
    );
}

/// Thranduil's Company: a second land drop only beside another Elf (CR
/// 305.2), and landfall puts two counters on a creature of yours.
#[test]
fn thranduils_company_extra_land_needs_another_elf() {
    let mut g = pod(2);
    let company = ready(&mut g, 0, catalog::thranduils_company());
    assert_eq!(g.max_lands_per_turn(0), 1);
    ready(&mut g, 0, catalog::llanowar_elves());
    assert_eq!(g.max_lands_per_turn(0), 2);
    let land = g.add_card_to_hand(0, catalog::forest());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    let counters: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(counters, 2);
    assert!(g.battlefield_find(company).is_some());
}

/// Woodland Weavemaster: taps for mana equal to its power, which grows when
/// another Elf enters.
#[test]
fn woodland_weavemaster_taps_for_its_power() {
    let mut g = pod(2);
    let weaver = ready(&mut g, 0, catalog::woodland_weavemaster());
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_on(&mut g, elf, None);
    activate(&mut g, weaver, 0);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "Elf-only mana");
}

// ── Maralen, Fae Ascendant's list ───────────────────────────────────────────

/// Eclipsed Realms: {C}, or one restricted mana of any color for the chosen
/// type (CR 106.6); a headless seat names the listed type its deck plays.
#[test]
fn eclipsed_realms_makes_restricted_mana() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::llanowar_elves());
    }
    let realms = g.add_card_to_hand(0, catalog::eclipsed_realms());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(realms)).expect("land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(realms).unwrap().chosen_creature_type, Some(crabomination::card::CreatureType::Elf));
    activate(&mut g, realms, 1);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1);
}

/// Champions of the Perfect: beholding exiles an Elf card from hand as a cost
/// (no Elf, no cast — CR 601.2f); creature spells draw; leaving returns the
/// exiled card to its owner's hand (CR 607.2a link).
#[test]
fn champions_of_the_perfect_returns_its_beheld_elf() {
    let mut g = pod(2);
    let champ = g.add_card_to_hand(0, catalog::champions_of_the_perfect());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: champ, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "nothing to behold"
    );
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    cast_on(&mut g, champ, None);
    assert!(g.battlefield_find(champ).is_some());
    assert!(g.exile.iter().any(|c| c.id == elf), "the Elf was exiled");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    flood(&mut g);
    cast_on(&mut g, bear, None);
    assert_eq!(g.players[0].hand.len(), hand, "cast a bear, drew a card");
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g);
    cast_on(&mut g, murder, Some(Target::Permanent(champ)));
    assert!(g.players[0].hand.iter().any(|c| c.id == elf));
}

/// Glen Elendra Guardian: enters with a -1/-1 counter (CR 122.1); removing it
/// counters a noncreature spell and its controller draws.
#[test]
fn glen_elendra_guardian_spends_its_counter_to_counter() {
    let mut g = pod(2);
    let guard = g.add_card_to_hand(0, catalog::glen_elendra_guardian());
    flood(&mut g);
    cast_on(&mut g, guard, None);
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == guard).unwrap().power, 2);
    g.add_card_to_library(1, catalog::island());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    let life = g.players[0].life;
    let opp_hand = g.players[1].hand.len();
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: guard, ability_index: 0, target: Some(Target::Permanent(bolt)), additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "countered");
    assert_eq!(g.players[1].hand.len(), opp_hand + 1, "its controller drew");
    assert_eq!(g.battlefield_find(guard).unwrap().counter_count(crabomination::card::CounterType::MinusOneMinusOne), 0);
}

/// Morcant's Loyalist: an Elf lord; dying returns another Elf card (CR
/// 603.10a looks back to the battlefield).
#[test]
fn morcants_loyalist_pumps_and_recurs_an_elf() {
    let mut g = pod(2);
    let loyal = ready(&mut g, 0, catalog::morcants_loyalist());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    assert_eq!(g.compute_battlefield().iter().find(|c| c.id == elf).unwrap().power, 2);
    let dead_elf = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g);
    cast_on(&mut g, murder, Some(Target::Permanent(loyal)));
    assert!(g.players[0].hand.iter().any(|c| c.id == dead_elf));
}

/// Selfless Safewright: naming Elf, the other Elves gain hexproof and
/// indestructible until end of turn.
#[test]
fn selfless_safewright_shields_its_kin() {
    let mut g = pod(2);
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    let wright = g.add_card_to_hand(0, catalog::selfless_safewright());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(crabomination::card::CreatureType::Elf)]));
    cast_on(&mut g, wright, None);
    let v = g.compute_battlefield();
    let e = v.iter().find(|c| c.id == elf).unwrap();
    assert!(e.keywords().contains(&crabomination::card::Keyword::Hexproof));
    assert!(e.keywords().contains(&crabomination::card::Keyword::Indestructible));
}

/// Glen Elendra's Answer: counters an opponent's spell and makes a Faerie for
/// it (CR 701.6a).
#[test]
fn glen_elendras_answer_counters_and_makes_faeries() {
    let mut g = pod(2);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    let life = g.players[0].life;
    let answer = g.add_card_to_hand(0, catalog::glen_elendras_answer());
    flood(&mut g);
    cast_on(&mut g, answer, None);
    assert_eq!(g.players[0].life, life);
    assert_eq!(named(&g, "Faerie"), 1);
}

// ── Najeela, the Blade-Blossom's list ───────────────────────────────────────

fn attack_with(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// Blood-Chin Rager: attacking gives your Warriors menace (CR 702.111).
#[test]
fn blood_chin_rager_gives_warriors_menace() {
    let mut g = pod(2);
    let rager = ready(&mut g, 0, catalog::blood_chin_rager());
    let warleader = ready(&mut g, 0, catalog::kargan_warleader());
    attack_with(&mut g, &[rager]);
    let v = g.compute_battlefield();
    assert!(v.iter().find(|c| c.id == warleader).unwrap().keywords().contains(&crabomination::card::Keyword::Menace));
}

/// Grand Warlord Radha: two attackers add two red/green mana that survives
/// the step's end (CR 500.4 exception).
#[test]
fn grand_warlord_radha_adds_mana_per_attacker() {
    let mut g = pod(2);
    let radha = ready(&mut g, 0, catalog::grand_warlord_radha());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    attack_with(&mut g, &[radha, bear]);
    assert_eq!(g.players[0].mana_pool.total(), 2);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].mana_pool.total(), 2, "kept through combat");
}

/// Herald of Dromoka, Kargan Warleader, Rushblade Commander: the Warrior
/// lords grant vigilance, +1/+1 and haste to other Warriors.
#[test]
fn warrior_lords_pump_and_grant() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::herald_of_dromoka());
    ready(&mut g, 0, catalog::kargan_warleader());
    ready(&mut g, 0, catalog::rushblade_commander());
    let rager = ready(&mut g, 0, catalog::blood_chin_rager());
    let v = g.compute_battlefield();
    let r = v.iter().find(|c| c.id == rager).unwrap();
    assert_eq!((r.power, r.toughness), (3, 3));
    assert!(r.keywords().contains(&crabomination::card::Keyword::Vigilance));
    assert!(r.keywords().contains(&crabomination::card::Keyword::Haste));
}

/// Samut, Vizier of Naktamun: a creature that entered this turn connecting
/// draws a card; Samut (haste) qualifies on the turn it arrives.
#[test]
fn samut_draws_for_fresh_attackers() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let samut = g.add_card_to_hand(0, catalog::samut_vizier_of_naktamun());
    flood(&mut g);
    cast_on(&mut g, samut, None);
    let hand = g.players[0].hand.len();
    connect(&mut g, samut);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Druids' Repository: each attacker adds a charge counter; removing one adds
/// a mana (CR 605.1a mana ability).
#[test]
fn druids_repository_banks_attack_counters() {
    let mut g = pod(2);
    let repo = ready(&mut g, 0, catalog::druids_repository());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    attack_with(&mut g, &[a, b]);
    assert_eq!(g.battlefield_find(repo).unwrap().counter_count(crabomination::card::CounterType::Charge), 2);
    g.players[0].mana_pool.empty();
    activate(&mut g, repo, 0);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

// ── Toph, the First Metalbender's list ──────────────────────────────────────

fn land_creature_power(g: &GameState, id: CardId) -> Option<i32> {
    g.compute_battlefield()
        .iter()
        .find(|c| c.id == id && c.card_types().contains(&crabomination::card::CardType::Creature))
        .map(|c| c.power)
}

/// The Stasis Coffin: exiling it as a cost gives you protection from
/// everything until your next turn (CR 702.16j).
#[test]
fn the_stasis_coffin_protects_its_controller() {
    let mut g = pod(2);
    let coffin = ready(&mut g, 0, catalog::the_stasis_coffin());
    flood(&mut g);
    activate(&mut g, coffin, 0);
    assert!(g.battlefield_find(coffin).is_none());
    assert!(g.exile.iter().any(|c| c.id == coffin));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "can't be targeted"
    );
}

/// Avatar Kyoshi, Earthbender: beginning of combat earthbends 8 and untaps
/// the land (CR 701.66a).
#[test]
fn avatar_kyoshi_earthbends_eight_and_untaps() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::avatar_kyoshi_earthbender());
    let forest = ready(&mut g, 0, catalog::forest());
    g.battlefield_find_mut(forest).unwrap().tapped = true;
    g.step = TurnStep::PreCombatMain;
    while g.step != TurnStep::BeginCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(land_creature_power(&g, forest), Some(8));
    assert!(!g.battlefield_find(forest).unwrap().tapped);
}

/// Bumi, Eclectic Earthbender: ETB earthbend 1; attacking adds two counters
/// to each land creature.
#[test]
fn bumi_grows_land_creatures() {
    let mut g = pod(2);
    let forest = ready(&mut g, 0, catalog::forest());
    let bumi = g.add_card_to_hand(0, catalog::bumi_eclectic_earthbender());
    flood(&mut g);
    cast_on(&mut g, bumi, None);
    assert_eq!(land_creature_power(&g, forest), Some(1));
    g.clear_sickness(bumi);
    attack_with(&mut g, &[bumi]);
    assert_eq!(land_creature_power(&g, forest), Some(3));
}

/// Toph, Earthbending Master: landfall banks experience; attacking earthbends
/// that many.
#[test]
fn toph_earthbending_master_spends_experience_on_attack() {
    let mut g = pod(2);
    let toph = ready(&mut g, 0, catalog::toph_earthbending_master());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(forest)).expect("land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].experience, 1);
    attack_with(&mut g, &[toph]);
    assert_eq!(land_creature_power(&g, forest), Some(1));
}

/// Toph, Greatest Earthbender: earthbends the mana spent on her (four), and
/// land creatures gain double strike.
#[test]
fn toph_greatest_earthbender_earthbends_her_cost() {
    let mut g = pod(2);
    let forest = ready(&mut g, 0, catalog::forest());
    let toph = g.add_card_to_hand(0, catalog::toph_greatest_earthbender());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::Green, 2);
    cast_on(&mut g, toph, None);
    let v = g.compute_battlefield();
    let land = v.iter().find(|c| c.id == forest).unwrap();
    assert_eq!(land.power, 4);
    assert!(land.keywords().contains(&crabomination::card::Keyword::DoubleStrike));
}

/// The Legend of Kyoshi: I draws for the biggest creature; II earthbends by
/// hand size and makes the land an Island too; III flips into Avatar Kyoshi.
#[test]
fn the_legend_of_kyoshi_reads_power_then_hand() {
    let mut g = pod(2);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::serra_angel());
    let saga = g.add_card_to_hand(0, catalog::the_legend_of_kyoshi());
    flood(&mut g);
    cast_on(&mut g, saga, None);
    assert_eq!(g.players[0].hand.len(), 4, "Serra Angel's power");
}

/// Earthshape: the 3/3 land shields creatures with power 3 or less, not a
/// 4-power Angel, and you gain hexproof.
#[test]
fn earthshape_shields_the_small() {
    let mut g = pod(2);
    let forest = ready(&mut g, 0, catalog::forest());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    let shape = g.add_card_to_hand(0, catalog::earthshape());
    flood(&mut g);
    cast_on(&mut g, shape, Some(Target::Permanent(forest)));
    let v = g.compute_battlefield();
    let kw = |id| v.iter().find(|c| c.id == id).unwrap().keywords().contains(&crabomination::card::Keyword::Indestructible);
    assert!(kw(bear));
    assert!(!kw(angel));
    assert_eq!(land_creature_power(&g, forest), Some(3));
}
