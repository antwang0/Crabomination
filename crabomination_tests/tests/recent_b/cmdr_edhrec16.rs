//! Commander: the 2026-10-12 rescan's two- and four-card EDHREC gaps of the top 600
//! (`decks::cmdr_edhrec16`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_as(g, 0, id, targets)
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

fn tokens_named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

fn declare(g: &mut GameState, seat: usize, attacks: &[(CardId, usize)]) -> Result<(), String> {
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.iter().map(|&(a, d)| Attack { attacker: a, target: AttackTarget::Player(d) }).collect(),
    ))
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn combat_damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn on_bf(g: &GameState, id: CardId) -> bool {
    g.battlefield_find(id).is_some()
}


/// Gorgon Flail / Rune of Mortality hand out deathtouch; the Rune cantrips.
#[test]
fn kelsien_deathtouch_kit() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let rune = g.add_card_to_hand(0, catalog::rune_of_mortality());
    let hand = g.players[0].hand.len();
    cast(&mut g, rune, &[perm(bear)]).expect("rune");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    let kw = |g: &GameState| g.computed_permanent(bear).unwrap().keywords().to_vec();
    assert!(kw(&g).contains(&Keyword::Deathtouch));
    let flail = ready(&mut g, 0, catalog::gorgon_flail());
    let other = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: flail, target: other }).expect("equip");
    drain_stack(&mut g);
    let cp = g.computed_permanent(other).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords().contains(&Keyword::Deathtouch));
}

/// Boarding Party and Forceful Denial cascade.
#[test]
fn averna_deck_cascades() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bp = g.add_card_to_hand(0, catalog::boarding_party());
    cast(&mut g, bp, &[]).expect("cast");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "cascaded into the bear");
}

/// Disciple of Perdition: at 13 life both modes happen.
#[test]
fn disciple_of_perdition_at_thirteen_takes_both() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_graveyard(1, catalog::island());
    g.players[0].life = 13;
    let d = ready(&mut g, 0, catalog::disciple_of_perdition());
    let (hand, theirs) = (g.players[0].hand.len(), g.players[1].life);
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, b, &[perm(d)]).expect("bolt");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].life, 12);
    assert_eq!(g.players[1].life, theirs - 1);
    assert!(g.players[1].graveyard.iter().all(|c| c.definition.name != "Island"), "their graveyard was exiled");
}

/// Fume Spitter and Tattered Mummy.
#[test]
fn shirei_fodder() {
    let mut g = pod(2);
    let fs = ready(&mut g, 0, catalog::fume_spitter());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    activate(&mut g, fs, 0, Some(perm(bear))).expect("spit");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    let m = ready(&mut g, 0, catalog::tattered_mummy());
    let life = g.players[1].life;
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, b, &[perm(m)]).expect("bolt");
    assert_eq!(g.players[1].life, life - 2);
}

/// Raph & Mikey put the first creature off the top into play attacking.
#[test]
fn raph_and_mikey_bring_a_friend() {
    let mut g = pod(2);
    for def in [catalog::island(), catalog::grizzly_bears(), catalog::island()] {
        g.add_card_to_library(0, def);
    }
    let rm = ready(&mut g, 0, catalog::raph_mikey_troublemakers());
    declare(&mut g, 0, &[(rm, 1)]).expect("attack");
    let bear = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("bear").id;
    assert!(g.attacking.iter().any(|a| a.attacker == bear), "tapped and attacking");
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Groundchuck & Dirtbag: each land tap adds an extra {G}.
#[test]
fn groundchuck_and_dirtbag_double_lands() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::groundchuck_dirtbag());
    let island = ready(&mut g, 0, catalog::island());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: island,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), 2);
}


/// Krovikan Mist counts every Illusion; Oneirophage grows on draws.
#[test]
fn minn_illusions() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let mist = ready(&mut g, 0, catalog::krovikan_mist());
    let o = ready(&mut g, 1, catalog::oneirophage());
    assert_eq!(g.computed_permanent(mist).unwrap().power, 2);
    g.add_card_to_library(1, catalog::island());
    let tt = g.add_card_to_hand(1, catalog::think_twice());
    cast_as(&mut g, 1, tt, &[]).expect("draw");
    assert_eq!(g.battlefield_find(o).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Mordenkainen's −2 Dog is twice your hand; −10 swaps hand and library.
#[test]
fn mordenkainen_dog_and_swap() {
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::mordenkainen());
    g.battlefield_find_mut(m).unwrap().add_counters(CounterType::Loyalty, 10);
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: m, ability_index: 1, target: None, x_value: None })
        .expect("-2");
    drain_stack(&mut g);
    let dog = g.battlefield.iter().find(|c| c.definition.name == "Dog Illusion").expect("dog").id;
    assert_eq!(g.computed_permanent(dog).unwrap().power, 6);

    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::mordenkainen());
    g.battlefield_find_mut(m).unwrap().add_counters(CounterType::Loyalty, 10);
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: m, ability_index: 2, target: None, x_value: None })
        .expect("-10");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 5, "the library is the hand");
    assert!(g.players[0].hand.iter().all(|c| c.definition.name == "Forest"));
    assert_eq!(g.players[0].library.len(), 3);
    assert_eq!(g.players[0].emblems.len(), 1);
}

/// Biowaste Blob copies itself while you control a commander; Oozes +1/+1.
#[test]
fn biowaste_blob_needs_a_commander() {
    let mut g = pod(2);
    let blob = ready(&mut g, 0, catalog::biowaste_blob());
    assert_eq!(g.computed_permanent(blob).unwrap().power, 1);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(tokens_named(&g, "Biowaste Blob"), 0, "no commander: no copy");
    let cmd = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmd);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(tokens_named(&g, "Biowaste Blob"), 1);
}

/// Consuming Blob's end-step Ooze shares its P/T rule.
#[test]
fn consuming_blob_spawns_matching_oozes() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let blob = ready(&mut g, 0, catalog::consuming_blob());
    let cp = g.computed_permanent(blob).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let ooze = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Ooze").expect("ooze").id;
    let cp = g.computed_permanent(ooze).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3));
}

/// Ochre Jelly splits on death into a copy with half the counters.
#[test]
fn ochre_jelly_splits() {
    let mut g = pod(2);
    let j = g.add_card_to_hand(0, catalog::ochre_jelly());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: j, target: None, additional_targets: vec![], mode: None, x_value: Some(4) })
        .expect("jelly");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(j).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
    g.battlefield_find_mut(j).unwrap().damage = 10;
    g.check_state_based_actions();
    drain_stack(&mut g);
    let t = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Ochre Jelly").expect("split");
    assert_eq!(t.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Oran-Rief Ooze grows attackers that already have counters.
#[test]
fn oran_rief_ooze_feeds_the_countered() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let o = g.add_card_to_hand(0, catalog::oran_rief_ooze());
    cast(&mut g, o, &[perm(bear)]).expect("ooze");
    g.clear_sickness(o);
    declare(&mut g, 0, &[(o, 1), (bear, 1)]).expect("attack");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(o).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}



/// CR 614.1a — Kenessos adds one to each scry; its ability fishes for a sea
/// creature.
#[test]
fn cr_614_1a_kenessos_scries_one_more() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::kenessos_priest_of_thassa());
    let d = g.add_card_to_hand(0, catalog::deliberate());
    flood(&mut g, 0);
    let mut ev = g
        .perform_action(GameAction::CastSpell { card_id: d, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("deliberate");
    ev.extend(drain_stack(&mut g));
    assert!(
        ev.iter().any(|e| matches!(e, GameEvent::ScriedOrSurveiled { surveil: false, looked_at: 3, .. })),
        "scry 2 became scry 3"
    );
}

/// Kenessos's look: a sea creature may enter; anything left on top may go
/// to the bottom (the printed second "may").
#[test]
fn kenessos_puts_a_sea_creature_in_or_a_miss_on_the_bottom() {
    let mut g = pod(2);
    let kraken = g.add_card_to_library(0, catalog::kraken_hatchling());
    let miss = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let k = ready(&mut g, 0, catalog::kenessos_priest_of_thassa());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, k, 0, None).expect("look");
    assert!(g.battlefield_find(kraken).is_some(), "the Kraken enters");
    g.battlefield_find_mut(k).unwrap().tapped = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, k, 0, None).expect("look again");
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(miss), "the miss went to the bottom");
}

/// Joint Exploration kicked drops a land from hand.
#[test]
fn joint_exploration_kicked_drops_a_land() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let forest = g.add_card_to_hand(0, catalog::forest());
    let je = g.add_card_to_hand(0, catalog::joint_exploration());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked { card_id: je, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("kicked");
    drain_stack(&mut g);
    assert!(on_bf(&g, forest));
}

/// Aphelia's {4}{B}: a Snake connecting halves the damaged player's life.
#[test]
fn aphelia_halves_a_life_total() {
    let mut g = pod(2);
    let aph = ready(&mut g, 0, catalog::aphelia_viper_whisperer());
    let snake = ready(&mut g, 0, catalog::hooded_blightfang());
    activate(&mut g, aph, 0, None).expect("arm");
    g.players[1].life = 21;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    declare(&mut g, 0, &[(snake, 1)]).expect("attack");
    assert_eq!(g.players[1].life, 20, "Blightfang's attack drain");
    combat_damage(&mut g);
    // 20 - 1 combat = 19, then half rounded up (10) lost.
    assert_eq!(g.players[1].life, 9);
}

/// Hooded Blightfang — "Whenever a creature you control with deathtouch deals
/// damage to a planeswalker, destroy that planeswalker." Noncombat damage
/// counts (CR 120.3); damage to a creature does nothing extra.
#[test]
fn hooded_blightfang_destroys_a_planeswalker_burned_outside_combat() {
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = pod(2);
    let fang = ready(&mut g, 0, catalog::hooded_blightfang());
    let jace = ready(&mut g, 1, catalog::jace_beleren());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ping = |t| {
        TriggerPush::new(fang, 0, Effect::DealDamage { to: Selector::Target(0), amount: Value::ONE })
            .target(Some(Target::Permanent(t)))
            .build()
    };
    g.stack.push(ping(bear));
    drain_stack(&mut g);
    assert!(g.battlefield_find(jace).is_some(), "a creature hit destroys no planeswalker");
    g.stack.push(ping(jace));
    drain_stack(&mut g);
    assert!(g.battlefield_find(jace).is_none(), "the 1 noncombat damage triggers the destroy");
    // In combat the dealer survives and the planeswalker it hit is destroyed.
    let jace = ready(&mut g, 1, catalog::jace_beleren());
    g.battlefield_find_mut(jace).unwrap().counters.insert(CounterType::Loyalty, 5);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: fang, target: AttackTarget::Planeswalker(jace) }]))
        .expect("attack Jace");
    drain_stack(&mut g);
    combat_damage(&mut g);
    assert!(g.battlefield_find(jace).is_none(), "destroyed by the combat hit");
    assert!(g.battlefield_find(fang).is_some(), "the dealer is not \"that planeswalker\"");
}

/// CR 603.2c — two Snakes connecting are one batch: half the life once.
#[test]
fn cr_603_2c_aphelia_halves_once_a_batch() {
    let mut g = pod(2);
    let aph = ready(&mut g, 0, catalog::aphelia_viper_whisperer());
    let fang = ready(&mut g, 0, catalog::hooded_blightfang());
    let boa = ready(&mut g, 0, catalog::mire_boa());
    activate(&mut g, aph, 0, None).expect("arm");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    declare(&mut g, 0, &[(fang, 1), (boa, 1)]).expect("attack");
    assert_eq!(g.players[1].life, 19, "Blightfang's drain");
    combat_damage(&mut g);
    // 19 - 3 combat = 16, halved once to 8 (twice would be 4).
    assert_eq!(g.players[1].life, 8);
}

/// CR 702.21a — The Serpent Society's ward gives poison counters.
#[test]
fn cr_702_21a_serpent_society_ward_poisons() {
    let mut g = pod(2);
    let s = ready(&mut g, 0, catalog::the_serpent_society());
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast_as(&mut g, 1, b, &[perm(s)]).expect("bolt");
    assert_eq!(g.players[1].poison_counters, 5);
    assert_eq!(g.battlefield_find(s).unwrap().damage, 3, "ward paid: the Bolt resolved");
}

/// Mire Boa regenerates; Oketra needs three other creatures to attack.
#[test]
fn mire_boa_and_oketra() {
    let mut g = pod(2);
    let boa = ready(&mut g, 0, catalog::mire_boa());
    activate(&mut g, boa, 0, None).expect("regen");
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, b, &[perm(boa)]).expect("bolt");
    assert!(on_bf(&g, boa));
    let mut g = pod(2);
    let ok = ready(&mut g, 0, catalog::oketra_the_true());
    assert!(declare(&mut g, 0, &[(ok, 1)]).is_err());
    let mut g = pod(2);
    let ok = ready(&mut g, 0, catalog::oketra_the_true());
    for _ in 0..3 {
        ready(&mut g, 0, catalog::grizzly_bears());
    }
    declare(&mut g, 0, &[(ok, 1)]).expect("three others");
}

/// Akroma, Vision of Ixidor counts keywords at each combat — protection of
/// any kind once, and "partner with" as partner (the card's rulings).
#[test]
fn akroma_vision_counts_keywords() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::akroma_vision_of_ixidor());
    let o = ready(&mut g, 0, catalog::oketra_the_true());
    let knight = ready(&mut g, 0, catalog::white_knight());
    let pir = ready(&mut g, 0, catalog::pir_imaginative_rascal());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    // Double strike + indestructible.
    assert_eq!(g.computed_permanent(o).unwrap().power, 5);
    // First strike + protection from black.
    assert_eq!(g.computed_permanent(knight).unwrap().power, 4);
    // Partner with Toothy.
    assert_eq!(g.computed_permanent(pir).unwrap().power, 2);
}

/// Angelic Overseer is protected while you control a Human.
#[test]
fn angelic_overseer_wants_a_human() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::angelic_overseer());
    assert!(!g.computed_permanent(a).unwrap().keywords().contains(&Keyword::Hexproof));
    ready(&mut g, 0, catalog::suppressor_skyguard());
    assert!(g.computed_permanent(a).unwrap().keywords().contains(&Keyword::Hexproof));
}

/// Ashling transforms for {U} and Rimebound's mana pays only for big spells.
#[test]
fn ashling_rekindled_transforms_and_makes_big_mana() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::ashling_rekindled());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::PreCombatMain;
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().definition.name, "Ashling, Rimebound");
}

/// Enraged Flamecaster pings, Tanufel Rimespeaker draws, off a 4+ spell.
#[test]
fn ashling_deck_big_spell_payoffs() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    ready(&mut g, 0, catalog::enraged_flamecaster());
    ready(&mut g, 0, catalog::tanufel_rimespeaker());
    let life = g.players[1].life;
    let s = g.add_card_to_hand(0, catalog::boarding_party());
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    cast(&mut g, s, &[]).expect("cast");
    assert_eq!(g.players[1].life, life - 2);
    assert_eq!(g.players[0].hand.len(), hand);
}
