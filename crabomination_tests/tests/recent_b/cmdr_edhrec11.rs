//! Commander: one-card EDHREC gaps of EDHREC's top-600 commanders
//! (`decks::cmdr_edhrec11`).

use crabomination::card::{CardId, CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
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

fn bolt(g: &mut GameState, caster: usize, target: Target) {
    let b = g.add_card_to_hand(caster, catalog::lightning_bolt());
    flood(g, caster);
    g.priority.player_with_priority = caster;
    g.perform_action(GameAction::CastSpell { card_id: b, target: Some(target), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(g);
}


/// Red Dragon breathes 4 on each opponent.
#[test]
fn red_dragon_breathes_fire() {
    let mut g = pod(3);
    let (a, b) = (g.players[1].life, g.players[2].life);
    let d = g.add_card_to_hand(0, catalog::red_dragon());
    cast(&mut g, d, &[]).expect("cast");
    assert_eq!((g.players[1].life, g.players[2].life), (a - 4, b - 4));
}

/// Thallid: three spore counters make a Saproling.
#[test]
fn thallid_grows_spores() {
    let mut g = pod(2);
    let t = ready(&mut g, 0, catalog::thallid());
    for _ in 0..3 {
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
    }
    activate(&mut g, t, 0, None).expect("spores");
    assert_eq!(tokens_named(&g, "Saproling"), 1);
}

/// Myrkul: a nontoken creature of yours dying may become an enchantment-only
/// token copy.
#[test]
fn myrkul_preserves_the_dead() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::myrkul_lord_of_bones());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    bolt(&mut g, 1, perm(bear));
    let copy = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Grizzly Bears").expect("copy").id;
    let cp = g.computed_permanent(copy).unwrap();
    assert!(cp.card_types().contains(&CardType::Enchantment) && !cp.card_types().contains(&CardType::Creature));
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// Barrenton Medic untaps for a -1/-1 counter.
#[test]
fn barrenton_medic_untaps_for_a_counter() {
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::barrenton_medic());
    g.battlefield_find_mut(m).unwrap().tapped = true;
    activate(&mut g, m, 1, None).expect("untap");
    let c = g.battlefield_find(m).unwrap();
    assert!(!c.tapped);
    assert_eq!(c.counter_count(CounterType::MinusOneMinusOne), 1);
}


/// The Council of Four: an opponent's second spell on their turn makes you a
/// Knight.
#[test]
fn council_of_four_rewards_second_spells() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_council_of_four());
    g.active_player_idx = 1;
    for _ in 0..2 {
        let b = g.add_card_to_hand(1, catalog::grizzly_bears());
        flood(&mut g, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell { card_id: b, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast");
        drain_stack(&mut g);
    }
    assert_eq!(tokens_named(&g, "Knight"), 1);
}

/// Scryb Ranger: return a Forest to untap a creature.
#[test]
fn scryb_ranger_untaps_for_a_forest() {
    let mut g = pod(2);
    let s = ready(&mut g, 0, catalog::scryb_ranger());
    let forest = ready(&mut g, 0, catalog::forest());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    activate(&mut g, s, 0, Some(perm(bear))).expect("untap");
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest));
}

/// Titania's Command: Bears and counters.
#[test]
fn titanias_command_two_modes() {
    let mut g = pod(2);
    let c = g.add_card_to_hand(0, catalog::titanias_command());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellSpree { card_id: c, spree_modes: vec![2, 3], target: None, additional_targets: vec![], x_value: None })
        .expect("cast choosing Bears and counters");
    drain_stack(&mut g);
    assert_eq!(tokens_named(&g, "Bear"), 2);
    assert!(g.battlefield.iter().filter(|c| c.definition.name == "Bear").all(|c| c.counter_count(CounterType::PlusOnePlusOne) == 2));
}

/// Temporal Trespass takes an extra turn and exiles itself.
#[test]
fn temporal_trespass_takes_a_turn() {
    let mut g = pod(2);
    let t = g.add_card_to_hand(0, catalog::temporal_trespass());
    cast(&mut g, t, &[]).expect("cast");
    assert_eq!(g.players[0].extra_turns, 1);
    assert!(g.exile.iter().any(|c| c.id == t));
}

/// Songs of the Damned: {B} per creature card in your graveyard.
#[test]
fn songs_of_the_damned_counts_the_dead() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let s = g.add_card_to_hand(0, catalog::songs_of_the_damned());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3);
}

/// Virtus: combat damage halves that player's life.
#[test]
fn virtus_has_deathtouch() {
    let mut g = pod(2);
    let v = ready(&mut g, 0, catalog::virtus_the_veiled());
    assert!(g.computed_permanent(v).unwrap().keywords().contains(&crabomination::card::Keyword::Deathtouch));
}

/// Oath of Liliana: ETB each opponent sacrifices a creature.
#[test]
fn oath_of_liliana_edicts() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    let o = g.add_card_to_hand(0, catalog::oath_of_liliana());
    cast(&mut g, o, &[]).expect("cast");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Merrow Commerce untaps your Merfolk at your end step.
#[test]
fn merrow_commerce_untaps_merfolk() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::merrow_commerce());
    let m = ready(&mut g, 0, catalog::merrow_harbinger());
    g.battlefield_find_mut(m).unwrap().tapped = true;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(m).unwrap().tapped);
}

/// Lightning-Rig Crew pings each opponent.
#[test]
fn lightning_rig_crew_pings() {
    let mut g = pod(3);
    let c = ready(&mut g, 0, catalog::lightning_rig_crew());
    let (a, b) = (g.players[1].life, g.players[2].life);
    activate(&mut g, c, 0, None).expect("tap");
    assert_eq!((g.players[1].life, g.players[2].life), (a - 1, b - 1));
}

/// Craterclaw Colossus pumps your team by your artifact count.
#[test]
fn craterclaw_colossus_pumps_by_artifacts() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::sol_ring());
    let c = g.add_card_to_hand(0, catalog::craterclaw_colossus());
    cast(&mut g, c, &[]).expect("cast");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "two artifacts: Sol Ring and the Colossus");
}

/// Knight-Captain of Eos: two Soldiers on entry.
#[test]
fn knight_captain_brings_soldiers() {
    let mut g = pod(2);
    let k = g.add_card_to_hand(0, catalog::knight_captain_of_eos());
    cast(&mut g, k, &[]).expect("cast");
    assert_eq!(tokens_named(&g, "Soldier"), 2);
}

/// Citizen's Crowbar arrives with a Citizen wearing it.
#[test]
fn citizens_crowbar_suits_up_a_citizen() {
    let mut g = pod(2);
    let c = g.add_card_to_hand(0, catalog::citizens_crowbar());
    cast(&mut g, c, &[]).expect("cast");
    let citizen = g.battlefield.iter().find(|x| x.is_token && x.definition.name == "Citizen").expect("Citizen").id;
    assert_eq!(g.battlefield_find(c).unwrap().attached_to, Some(citizen));
    assert_eq!(g.computed_permanent(citizen).unwrap().power, 2);
}

/// Liliana Vess's −8 takes every graveyard's creatures.
#[test]
fn liliana_vess_ultimate_raises_the_dead() {
    let mut g = pod(2);
    let l = ready(&mut g, 0, catalog::liliana_vess());
    g.battlefield_find_mut(l).unwrap().counters.insert(CounterType::Loyalty, 8);
    let a = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: l, ability_index: 2, target: None, x_value: None })
        .expect("ultimate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).map(|c| c.controller), Some(0));
    assert!(g.battlefield_find(b).is_some());
}

/// Shared Summons searches for two creature cards with different names.
#[test]
fn shared_summons_fetches_different_names() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for c in [catalog::grizzly_bears(), catalog::grizzly_bears(), catalog::hill_giant()] {
        g.add_card_to_library(0, c);
    }
    let ss = g.add_card_to_hand(0, catalog::shared_summons());
    cast(&mut g, ss, &[]).expect("cast");
    let mut names: Vec<&str> = g.players[0].hand.iter().map(|c| c.definition.name).collect();
    names.sort();
    assert_eq!(names, ["Grizzly Bears", "Hill Giant"]);
}

/// Gandalf the White — "If a legendary permanent or an artifact entering or
/// leaving the battlefield causes a triggered ability of a permanent you
/// control to trigger, that ability triggers an additional time." The
/// leaving half reads the left object's last known information (CR 603.10a).
#[test]
fn gandalf_the_white_doubles_triggers_a_leaving_legend_causes() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::gandalf_the_white());
    g.add_card_to_battlefield(0, catalog::zulaport_cutthroat());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hound = g.add_card_to_battlefield(0, catalog::isamaru_hound_of_konda());
    let life = g.players[1].life;
    for (victim, lost) in [(bear, 1), (hound, 2)] {
        let before = g.players[1].life;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, bolt, &[Target::Permanent(victim)]).expect("bolt");
        assert_eq!(before - g.players[1].life, lost, "a nonlegendary death once, a legend's twice");
    }
    assert_eq!(life - g.players[1].life, 3);
}

/// Roshan: creature cards you own off the battlefield are Assassins too (it
/// typed only permanents); an opponent's aren't.
#[test]
fn roshan_makes_your_creature_cards_assassins_everywhere() {
    use crabomination::card::{CreatureType, SelectionRequirement as R};
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::roshan_hidden_magister());
    let assassin = R::HasCreatureType(CreatureType::Assassin);
    let mine = g.add_card_to_hand(0, catalog::grizzly_bears());
    let card = g.find_card_anywhere(mine).unwrap().clone();
    assert!(g.evaluate_requirement_on_card(&assassin, &card, 0));
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let card = g.find_card_anywhere(theirs).unwrap().clone();
    assert!(!g.evaluate_requirement_on_card(&assassin, &card, 0));
}
