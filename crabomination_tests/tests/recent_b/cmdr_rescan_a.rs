//! Commander: one-card gaps of the 2026-10-14 ranks 1-5500 rescan
//! (`decks::cmdr_rescan_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>, mode: Option<usize>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: more,
        mode,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn finish_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

/// Invigorated Rampage's second mode splits +2/+0 and trample over exactly
/// two creatures.
#[test]
fn invigorated_rampage_second_mode_pumps_two() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::invigorated_rampage());
    cast_with(&mut g, spell, Some(Target::Permanent(a)), vec![Target::Permanent(b)], Some(1));
    for id in [a, b] {
        assert_eq!(pt(&g, id), (4, 2));
        assert!(g.permanent_has_keyword(id, &Keyword::Trample));
    }
}

/// Swarm of Bloodflies enters with two counters and grows when another
/// creature — anyone's — dies.
#[test]
fn swarm_of_bloodflies_grows_on_any_death() {
    let mut g = pod(2);
    let swarm = g.add_card_to_hand(0, catalog::swarm_of_bloodflies());
    cast_with(&mut g, swarm, None, vec![], None);
    assert_eq!(plus(&g, swarm), 2);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Permanent(theirs)), vec![], None);
    assert_eq!(plus(&g, swarm), 3);
}

/// Bloodcrazed Paladin counts the creatures that died this turn as it enters.
#[test]
fn bloodcrazed_paladin_counts_this_turns_deaths() {
    let mut g = pod(2);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Permanent(theirs)), vec![], None);
    let paladin = g.add_card_to_hand(0, catalog::bloodcrazed_paladin());
    cast_with(&mut g, paladin, None, vec![], None);
    assert_eq!(plus(&g, paladin), 1);
}

/// Homicidal Seclusion pumps a lone creature and switches off at two.
#[test]
fn homicidal_seclusion_needs_exactly_one_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::homicidal_seclusion());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(pt(&g, a), (5, 3));
    assert!(g.permanent_has_keyword(a, &Keyword::Lifelink));
    ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(pt(&g, a), (2, 2));
}

/// The Scarlet Witch's discount is her power, only on instants and sorceries
/// of mana value 4 or greater: Concentrate ({2}{U}{U}) casts off {U}{U}.
#[test]
fn scarlet_witch_discounts_big_instants_by_her_power() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_scarlet_witch());
    let spell = g.add_card_to_hand(0, catalog::concentrate());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{U}{U} pays for a discounted Concentrate");
}

/// Gnoll Hunter's pack tactics wants six total attacking power.
#[test]
fn gnoll_hunter_pack_tactics() {
    let mut g = pod(2);
    let gnoll = ready(&mut g, 0, catalog::gnoll_hunter());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    declare(&mut g, vec![(gnoll, 1), (angel, 1)]);
    assert_eq!(plus(&g, gnoll), 1, "2 + 4 = 6 power attacked");
}

/// Dragonscale General's bolster counts tapped creatures at your end step.
#[test]
fn dragonscale_general_bolsters_by_tapped_creatures() {
    let mut g = pod(2);
    let general = ready(&mut g, 0, catalog::dragonscale_general());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    g.battlefield_find_mut(general).unwrap().tapped = true;
    g.battlefield_find_mut(angel).unwrap().tapped = true;
    g.step = TurnStep::PostCombatMain;
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::End);
    assert_eq!(plus(&g, elf), 2, "two tapped creatures, onto the least toughness");
}

/// Mutant Chain Reaction kills a flyer and leaves a Mutagen behind.
#[test]
fn mutant_chain_reaction_destroys_and_makes_mutagen() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::mutant_chain_reaction());
    cast_with(&mut g, spell, Some(Target::Permanent(angel)), vec![], None);
    assert!(g.battlefield_find(angel).is_none());
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Mutagen"));
}

/// CR 903 pods: Gargantos's combat damage to one opponent is dealt to each
/// other opponent too.
#[test]
fn gargantos_splashes_combat_damage_to_other_opponents() {
    let mut g = pod(4);
    let garg = ready(&mut g, 0, catalog::gargantos_the_endbringer());
    g.battlefield_find_mut(garg).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    declare(&mut g, vec![(garg, 2)]);
    finish_combat(&mut g);
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    assert_eq!(life, vec![20, 17, 17, 17]);
}

/// Davvol drains you 1 and adds {B} per creature entering under your control.
#[test]
fn davvol_adds_black_mana_per_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::davvol_evincar_of_rath());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_with(&mut g, bear, None, vec![], None);
    assert_eq!(g.players[0].life, 19);
}

/// Kinbinding's Kithkin enters at combat and pumps the team by one.
#[test]
fn kinbinding_mints_at_combat_and_counts_entries() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::kinbinding());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::BeginCombat);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Kithkin"));
    // Only the Kithkin entered this turn: the setup bear skipped the tally.
    assert_eq!(pt(&g, bear), (3, 3));
}

/// Gimli's Axe grants menace only to a legendary bearer.
#[test]
fn gimlis_axe_menace_needs_a_legend() {
    let mut g = pod(2);
    let axe = ready(&mut g, 0, catalog::gimlis_axe());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(axe).unwrap().attached_to = Some(bear);
    assert_eq!(pt(&g, bear), (5, 2));
    assert!(!g.permanent_has_keyword(bear, &Keyword::Menace));
    let witch = ready(&mut g, 0, catalog::the_scarlet_witch());
    g.battlefield_find_mut(axe).unwrap().attached_to = Some(witch);
    assert!(g.permanent_has_keyword(witch, &Keyword::Menace));
}

/// Weather Maker banks a charge counter on landfall.
#[test]
fn weather_maker_charges_on_landfall() {
    let mut g = pod(2);
    let wm = ready(&mut g, 0, catalog::weather_maker());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wm).unwrap().counter_count(CounterType::Charge), 1);
}
