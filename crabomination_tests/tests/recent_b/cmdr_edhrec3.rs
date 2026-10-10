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
