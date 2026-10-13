//! Commander: Eriette's, Zo-Zu's, Charix's, Dion's, Gyruda's, Redshift's and
//! Zirda's last gaps (`decks::cmdr_edhrec30`).

use crabomination::card::{CardId, CounterType, Keyword};
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

fn cast(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
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

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn has_kw(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|cp| cp.keywords().contains(&kw))
}

/// Indestructibility makes the enchanted permanent indestructible; Aura
/// Finesse moves it to another creature and draws.
#[test]
fn indestructibility_moves_with_aura_finesse() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let aura = g.add_card_to_hand(0, catalog::indestructibility());
    cast(&mut g, 0, aura, &[Target::Permanent(bear)]).expect("aura");
    assert!(has_kw(&g, bear, Keyword::Indestructible));
    g.add_card_to_library(0, catalog::island());
    let finesse = g.add_card_to_hand(0, catalog::aura_finesse());
    cast(&mut g, 0, finesse, &[Target::Permanent(aura), Target::Permanent(giant)]).expect("finesse");
    assert!(has_kw(&g, giant, Keyword::Indestructible));
    assert!(!has_kw(&g, bear, Keyword::Indestructible));
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Dingus Egg bills a dead land's controller 2.
#[test]
fn dingus_egg_burns_on_land_death() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::dingus_egg());
    let land = ready(&mut g, 1, catalog::forest());
    let stone = g.add_card_to_hand(0, catalog::stone_rain());
    cast(&mut g, 0, stone, &[Target::Permanent(land)]).expect("stone rain");
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 20);
}

/// Tunnel Ignus punishes an opponent's second land in a turn (CR 603.4
/// intervening "if").
#[test]
fn tunnel_ignus_punishes_second_land() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::tunnel_ignus());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let a = g.add_card_to_hand(1, catalog::forest());
    g.perform_action(GameAction::PlayLand(a)).expect("land 1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "first land is free");
    g.add_card_to_library(1, catalog::forest());
    let ramp = g.add_card_to_hand(1, catalog::rampant_growth());
    cast(&mut g, 1, ramp, &[]).expect("ramp");
    assert_eq!(g.players[1].life, 17);
}

/// Wildfire takes four lands from each player and burns every creature.
#[test]
fn wildfire_razes() {
    let mut g = pod(2);
    for seat in [0, 1] {
        for _ in 0..5 {
            g.add_card_to_battlefield(seat, catalog::mountain());
        }
    }
    let crab = ready(&mut g, 1, catalog::fortress_crab());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wildfire());
    cast(&mut g, 0, id, &[]).expect("cast");
    g.check_state_based_actions();
    for seat in [0, 1] {
        assert_eq!(named(&g, seat, "Mountain"), 1);
    }
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(crab).is_some(), "6 toughness survives 4");
}

/// Crab Umbra untaps its creature and saves it from destruction (CR 702.89).
#[test]
fn crab_umbra_untaps_and_armors() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let umbra = g.add_card_to_hand(0, catalog::crab_umbra());
    cast(&mut g, 0, umbra, &[Target::Permanent(bear)]).expect("aura");
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    activate(&mut g, umbra, None);
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    let doom = g.add_card_to_hand(1, catalog::doom_blade());
    cast(&mut g, 1, doom, &[Target::Permanent(bear)]).expect("doom blade");
    assert!(g.battlefield_find(bear).is_some());
    assert!(g.battlefield_find(umbra).is_none());
}

/// Vexing Scuttler's cast trigger returns an instant from the graveyard.
#[test]
fn vexing_scuttler_returns_a_spell() {
    let mut g = pod(2);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::vexing_scuttler());
    cast(&mut g, 0, id, &[]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
}

/// Cavalier of Dawn trades an opposing permanent for a Golem.
#[test]
fn cavalier_of_dawn_golemizes() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let id = g.add_card_to_hand(0, catalog::cavalier_of_dawn());
    cast(&mut g, 0, id, &[Target::Permanent(wurm)]).expect("cast");
    assert!(g.battlefield_find(wurm).is_none());
    assert_eq!(named(&g, 1, "Golem"), 1);
}

/// Diamond Knight grows on spells of its chosen color.
#[test]
fn diamond_knight_grows_on_chosen_color() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    let id = g.add_card_to_hand(0, catalog::diamond_knight());
    cast(&mut g, 0, id, &[]).expect("cast");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, &[Target::Player(1)]).expect("bolt");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Fiendslayer Paladin can't be targeted by an opponent's red spell, but its
/// controller's own red spell may target it.
#[test]
fn fiendslayer_paladin_dodges_opposing_red_spells() {
    let mut g = pod(2);
    let pal = ready(&mut g, 0, catalog::fiendslayer_paladin());
    let theirs = g.add_card_to_hand(1, catalog::lightning_bolt());
    assert!(cast(&mut g, 1, theirs, &[Target::Permanent(pal)]).is_err());
    let mine = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, mine, &[Target::Permanent(pal)]).expect("own spell targets");
}

/// Undercover Operative copying your own creature enters with a shield
/// counter; copying an opponent's doesn't.
#[test]
fn undercover_operative_shields_copies_of_your_creatures() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::craw_wurm());
    let theirs = ready(&mut g, 1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![mine])]));
    let id = g.add_card_to_hand(0, catalog::undercover_operative());
    cast(&mut g, 0, id, &[]).expect("cast");
    let op = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(op.definition.name, "Craw Wurm");
    assert_eq!(op.counter_count(CounterType::Shield), 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![theirs])]));
    let id2 = g.add_card_to_hand(0, catalog::undercover_operative());
    cast(&mut g, 0, id2, &[]).expect("cast");
    let op2 = g.battlefield_find(id2).expect("on battlefield");
    assert_eq!(op2.definition.name, "Hill Giant");
    assert_eq!(op2.counter_count(CounterType::Shield), 0);
}

/// Ghastly Mimicry (Mirrorhall Mimic's back) copies the enchanted creature
/// each upkeep, as a Spirit.
#[test]
fn ghastly_mimicry_copies_each_upkeep() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let def = catalog::mirrorhall_mimic();
    let back = (*def.back_face.clone().unwrap()).clone();
    let aura = g.add_card_to_battlefield(0, back);
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Grizzly Bears"), 2);
}

/// Bhaal's Invoker's {8} hits each opponent for 4.
#[test]
fn bhaals_invoker_scorches() {
    let mut g = pod(3);
    let id = ready(&mut g, 0, catalog::bhaals_invoker());
    activate(&mut g, id, None);
    assert_eq!((g.players[1].life, g.players[2].life, g.players[0].life), (16, 16, 20));
}

/// Reckless Handling tutors an artifact; discarding an artifact at random
/// burns each opponent.
#[test]
fn reckless_handling_tutors_and_maybe_burns() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::reckless_handling());
    cast(&mut g, 0, id, &[]).expect("cast");
    assert!(g.players[0].hand.is_empty(), "the only card in hand, Sol Ring, was discarded");
    assert_eq!(g.players[1].life, 18);
}

/// Cogwork Assembler copies an artifact with haste, gone at end step.
#[test]
fn cogwork_assembler_borrows_an_artifact() {
    let mut g = pod(2);
    let cog = ready(&mut g, 0, catalog::cogwork_assembler());
    let ring = ready(&mut g, 1, catalog::sol_ring());
    activate(&mut g, cog, Some(Target::Permanent(ring)));
    assert_eq!(named(&g, 0, "Sol Ring"), 1);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Sol Ring"), 0);
}

/// Thran Spider gives you and an opponent a tapped Powerstone each.
#[test]
fn thran_spider_shares_powerstones() {
    let mut g = pod(2);
    let id = g.add_card_to_hand(0, catalog::thran_spider());
    cast(&mut g, 0, id, &[]).expect("cast");
    for seat in [0, 1] {
        let ps = g.battlefield.iter().find(|c| c.controller == seat && c.definition.name == "Powerstone").expect("token");
        assert!(ps.tapped);
    }
}
