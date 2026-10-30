//! Commander: one-card EDHREC gaps of EDHREC's top-600 commanders
//! (`decks::cmdr_edhrec9`).

use crabomination::card::{CardId, CounterType};
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

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
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

/// Smaug: noncombat damage dealt to it makes that many Treasures (a new
/// `EventKind::DealtNoncombatDamage`).
#[test]
fn smaug_turns_noncombat_damage_into_treasure() {
    let mut g = pod(2);
    let s = ready(&mut g, 0, catalog::smaug_the_impenetrable());
    bolt(&mut g, 1, perm(s));
    assert_eq!(tokens_named(&g, "Treasure"), 3);
}

/// CR 603.2 — combat damage to Smaug doesn't trigger it: nothing goes on the
/// stack (the trigger used to be put there and resolve to nothing).
#[test]
fn smaug_ignores_combat_damage() {
    use crabomination::game::types::GameEvent;
    let mut g = pod(2);
    let s = ready(&mut g, 0, catalog::smaug_the_impenetrable());
    let hit = GameEvent::DamageDealt {
        amount: 3, to_player: None, to_card: Some(s), combat: true, from_controller: Some(1), from_card: None,
    };
    g.dispatch_triggers_for_events(&[hit]);
    assert!(g.stack.is_empty(), "no trigger");
    assert_eq!(tokens_named(&g, "Treasure"), 0);
}

/// Heliod, the Warped Eclipse: spells cost {1} less per card the opponents
/// drew this turn (summed over every opponent).
#[test]
fn heliod_warped_eclipse_discounts_by_opponents_draws() {
    let mut g = pod(3);
    library(&mut g, 1, 3);
    library(&mut g, 2, 3);
    let h = ready(&mut g, 0, catalog::heliod_the_radiant_dawn());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: h, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("transform");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(h).unwrap().definition.name, "Heliod, the Warped Eclipse");
    g.players[1].cards_drawn_this_turn = 1;
    g.players[2].cards_drawn_this_turn = 1;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("two opposing draws take the Bears' {1} off: {G} pays");
}

/// Briarbridge Patrol: three Clues sacrificed this turn let the end step put
/// a creature from hand onto the battlefield.
#[test]
fn briarbridge_patrol_cashes_three_clues() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    ready(&mut g, 0, catalog::briarbridge_patrol());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..3 {
        let c = g.add_card_to_hand(0, catalog::confront_the_unknown());
        let pat = g.battlefield.iter().find(|c| c.definition.name == "Briarbridge Patrol").unwrap().id;
        cast(&mut g, c, &[perm(pat)]).expect("investigate");
    }
    let clues: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Clue").map(|c| c.id).collect();
    assert_eq!(clues.len(), 3);
    for c in clues {
        activate(&mut g, c, 0, None).expect("crack a Clue");
    }
    assert_eq!(g.players[0].clues_sacrificed_this_turn, 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}

/// CR 603.2c — "deals damage to one or more creatures": double-blocked, the
/// Patrol damages two creatures in one combat damage step and investigates once.
#[test]
fn briarbridge_patrol_investigates_once_for_two_blockers() {
    let mut g = pod(2);
    let pat = ready(&mut g, 0, catalog::briarbridge_patrol());
    let blockers = [ready(&mut g, 1, catalog::grizzly_bears()), ready(&mut g, 1, catalog::grizzly_bears())];
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: pat,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(blockers.iter().map(|&b| (b, pat)).collect())).expect("double block");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(blockers.iter().all(|&b| g.battlefield_find(b).is_none_or(|c| c.damage > 0)), "both blockers were dealt damage");
    assert_eq!(tokens_named(&g, "Clue"), 1);
}

/// Haze Frog: other creatures' combat damage is prevented; its own isn't
/// touched by the filter.
#[test]
fn haze_frog_fogs_the_others() {
    let mut g = pod(2);
    let frog = g.add_card_to_hand(0, catalog::haze_frog());
    cast(&mut g, frog, &[]).expect("cast");
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let _ = bear;
    assert!(g.battlefield_find(frog).is_some());
}

/// Apocalypse Hydra: X counters, doubled at five or more.
#[test]
fn apocalypse_hydra_doubles_at_five() {
    let mut g = pod(2);
    let h = g.add_card_to_hand(0, catalog::apocalypse_hydra());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: h, target: None, additional_targets: vec![], mode: None, x_value: Some(5) })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(h).unwrap().counter_count(CounterType::PlusOnePlusOne), 10);
}

/// Curious Colossus: the target opponent's creatures become 1/1 Cowards.
#[test]
fn curious_colossus_shrinks_a_board() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::curious_colossus());
    cast(&mut g, c, &[Target::Player(1)]).expect("cast");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.subtypes().creature_types.contains(&crabomination::card::CreatureType::Coward));
}

/// Stronghold Rats: shadow; a hit makes each player discard.
#[test]
fn stronghold_rats_has_shadow() {
    let mut g = pod(2);
    let r = ready(&mut g, 0, catalog::stronghold_rats());
    assert!(g.computed_permanent(r).unwrap().keywords().contains(&crabomination::card::Keyword::Shadow));
}

/// Angelic Ascension: exile a creature; its controller gets a 4/4 Angel.
#[test]
fn angelic_ascension_trades_for_an_angel() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let a = g.add_card_to_hand(0, catalog::angelic_ascension());
    cast(&mut g, a, &[perm(bear)]).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Angel" && c.controller == 1));
}

/// Kaervek shrinks every other creature.
#[test]
fn kaervek_shrinks_others() {
    let mut g = pod(2);
    let k = ready(&mut g, 0, catalog::kaervek_the_spiteful());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 1);
    assert_eq!(g.computed_permanent(k).unwrap().power, 3);
}

/// Urza's Workshop: metalcraft taps for each Urza's land.
#[test]
fn urzas_workshop_needs_metalcraft() {
    let mut g = pod(2);
    let w1 = ready(&mut g, 0, catalog::urzas_workshop());
    ready(&mut g, 0, catalog::urzas_workshop());
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility { card_id: w1, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
            .is_err(),
        "no metalcraft"
    );
    for _ in 0..3 {
        ready(&mut g, 0, catalog::sol_ring());
    }
    g.perform_action(GameAction::ActivateAbility { card_id: w1, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("metalcraft");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2);
}

/// Compy Swarm copies itself, tapped, at an end step after a death.
#[test]
fn compy_swarm_multiplies_after_a_death() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::compy_swarm());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    bolt(&mut g, 0, perm(bear));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let copy = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Compy Swarm").expect("a copy");
    assert!(copy.tapped);
}

/// Spitting Dilophosaurus: ETB -1/-1 counter; that opponent's creature
/// can't block.
#[test]
fn spitting_dilophosaurus_spits() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let d = g.add_card_to_hand(0, catalog::spitting_dilophosaurus());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(perm(bear))]));
    cast(&mut g, d, &[]).expect("cast");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&crabomination::card::Keyword::CantBlock));
}

/// Staff of Eden: draws one per permanent you control but don't own.
#[test]
fn staff_of_eden_counts_borrowed_permanents() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let staff = ready(&mut g, 0, catalog::staff_of_eden_vaults_key());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().controller = 0;
    let hand = g.players[0].hand.len();
    activate(&mut g, staff, 0, None).expect("tap");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Bre: the end step after life gain casts a cheap enough dug card for free.
#[test]
fn bre_casts_the_dug_card_within_the_life_gained() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::bre_of_clan_stoutarm());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].life_gained_this_turn = 3;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "cast free");
}

/// Mabel: an entering creature gets a counter on a creature that entered
/// this turn.
#[test]
fn mabel_grows_new_arrivals() {
    let mut g = pod(2);
    let m = g.add_card_to_hand(0, catalog::mabel_valley_hero());
    cast(&mut g, m, &[]).expect("cast");
    assert_eq!(g.battlefield_find(m).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Eirdu: the first main phase may pay {B} to become Isilu.
#[test]
fn eirdu_transforms_for_black() {
    let mut g = pod(2);
    let e = ready(&mut g, 0, catalog::eirdu_carrier_of_dawn());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(e).unwrap().definition.name, "Isilu, Carrier of Twilight");
}

/// Workshop Assistant returns another artifact card when it dies.
#[test]
fn workshop_assistant_recovers_an_artifact() {
    let mut g = pod(2);
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let wa = ready(&mut g, 0, catalog::workshop_assistant());
    bolt(&mut g, 1, perm(wa));
    assert!(g.players[0].hand.iter().any(|c| c.id == ring));
}

/// Guile: a counterspell of yours exiles the spell instead (CR 614.1a), and
/// you may play it free while it stays exiled.
#[test]
fn guile_turns_counters_into_theft() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::guile());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("their Bears");
    let retort = g.add_card_to_hand(0, catalog::wizards_retort());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: retort, target: Some(perm(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("counter");
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == bear).expect("exiled, not countered");
    assert_eq!(exiled.may_play_until.map(|m| m.player), Some(0));
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear));
}

/// Ertai's Scorn costs {1}{U} once an opponent has cast two spells this turn.
#[test]
fn ertais_scorn_gets_cheaper_after_two_opposing_spells() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::ertais_scorn());
    assert!(catalog::ertais_scorn().alternative_cost.is_some());
    g.players[1].spells_cast_this_game_turn = 1;
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("their Bears");
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: s,
        pitch_card: None,
        target: Some(perm(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{U}{U} pays the {1}{U} alternative");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

/// Loki: an ability of yours targeting draws a card, once a turn; a spell
/// targeting doesn't.
#[test]
fn loki_draws_when_an_ability_targets() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    ready(&mut g, 0, catalog::loki_god_of_mischief());
    let hand = g.players[0].hand.len();
    bolt(&mut g, 0, Target::Player(1));
    assert_eq!(g.players[0].hand.len(), hand, "a spell's targeting doesn't count");
    let plagon = ready(&mut g, 0, catalog::plagon_lord_of_the_beach());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, plagon, 0, Some(perm(bear))).expect("Plagon's ability targets");
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 115.1 — Loki hears a *player or permanent* becoming an ability's
/// target: Scavenging Ooze aiming at a graveyard card draws nothing (the
/// `ChoseTargets` listener drew for it), Glasses of Urza aiming at a player does.
#[test]
fn loki_ignores_an_ability_aimed_only_at_a_graveyard_card() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    ready(&mut g, 0, catalog::loki_god_of_mischief());
    let ooze = ready(&mut g, 0, catalog::scavenging_ooze());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    activate(&mut g, ooze, 0, Some(perm(dead))).expect("exile a graveyard card");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == dead), "the Ooze resolved");
    assert_eq!(g.players[0].hand.len(), hand, "a graveyard card isn't a permanent");
    let gl = ready(&mut g, 0, catalog::glasses_of_urza());
    activate(&mut g, gl, 0, Some(Target::Player(1))).expect("look");
    assert_eq!(g.players[0].hand.len(), hand + 1, "a player is");
}

/// Glasses of Urza: {T}: look at a player's hand (an ability that targets).
#[test]
fn glasses_of_urza_peeks() {
    let mut g = pod(2);
    let gl = ready(&mut g, 0, catalog::glasses_of_urza());
    activate(&mut g, gl, 0, Some(Target::Player(1))).expect("look");
    assert!(g.battlefield_find(gl).unwrap().tapped);
}

/// Uril grows +2/+2 per Aura on it.
#[test]
fn uril_grows_with_auras() {
    let mut g = pod(2);
    let u = ready(&mut g, 0, catalog::uril_the_miststalker());
    let runes = g.add_card_to_hand(0, catalog::runes_of_the_deus());
    cast(&mut g, runes, &[perm(u)]).expect("enchant Uril");
    let cp = g.computed_permanent(u).unwrap();
    assert_eq!((cp.power, cp.toughness), (9, 9), "5/5, +2/+2 for the Aura, Runes' +1/+1 for red and again for green");
    assert!(cp.keywords().contains(&crabomination::card::Keyword::DoubleStrike));
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Trample));
}

/// Zurgo is indestructible on your turn only.
#[test]
fn zurgo_is_indestructible_on_your_turn() {
    let mut g = pod(2);
    let z = ready(&mut g, 0, catalog::zurgo_helmsmasher());
    assert!(g.computed_permanent(z).unwrap().keywords().contains(&crabomination::card::Keyword::Indestructible));
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(z).unwrap().keywords().contains(&crabomination::card::Keyword::Indestructible));
}

/// Dual Shot pings two creatures.
#[test]
fn dual_shot_pings_two() {
    let mut g = pod(2);
    let a = ready(&mut g, 1, crabomination::card::CardDefinition { toughness: 1, ..catalog::grizzly_bears() });
    let b = ready(&mut g, 1, crabomination::card::CardDefinition { toughness: 1, ..catalog::grizzly_bears() });
    let d = g.add_card_to_hand(0, catalog::dual_shot());
    cast(&mut g, d, &[perm(a), perm(b)]).expect("cast");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Hanweir Militia Captain transforms with four creatures and its back face
/// counts them.
#[test]
fn hanweir_militia_captain_becomes_the_cult_leader() {
    let mut g = pod(2);
    let c = ready(&mut g, 0, catalog::hanweir_militia_captain());
    for _ in 0..3 {
        ready(&mut g, 0, catalog::grizzly_bears());
    }
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(c).unwrap().definition.name, "Westvale Cult Leader");
    assert_eq!(g.computed_permanent(c).unwrap().power, 4);
}

/// Riders of the Mark: after attacking, your end step trades it for Soldiers.
#[test]
fn riders_of_the_mark_return_for_soldiers() {
    let mut g = pod(2);
    let r = ready(&mut g, 0, catalog::riders_of_the_mark());
    g.battlefield_find_mut(r).unwrap().attacked_this_turn = true;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == r));
    assert_eq!(tokens_named(&g, "Human Soldier"), 4);
}

/// Desecrated Tomb: a creature card leaving your graveyard makes a Bat.
#[test]
fn desecrated_tomb_makes_bats() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::desecrated_tomb());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ff = g.add_card_to_hand(0, catalog::flood_of_recollection());
    let _ = ff;
    let ascend = g.add_card_to_hand(0, catalog::ascend_from_avernus());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: ascend, target: None, additional_targets: vec![], mode: None, x_value: Some(2) })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(tokens_named(&g, "Bat"), 1);
}

/// Nim Deathmantle watches YOUR graveyard: a creature you own dies under an
/// opponent's control and comes back to you wearing it; one you control but
/// don't own goes to its owner's graveyard and doesn't trigger it.
#[test]
fn nim_deathmantle_reads_the_owner() {
    let mut g = pod(2);
    let nim = ready(&mut g, 0, catalog::nim_deathmantle());
    let lent = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(lent).unwrap().owner = 0;
    let borrowed = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(borrowed).unwrap().owner = 1;
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    for id in [lent, borrowed] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, bolt, &[Target::Permanent(id)]).expect("bolt");
        g.check_state_based_actions();
        drain_stack(&mut g);
    }
    let back = g.battlefield_find(lent).expect("the owned one returns");
    assert_eq!(back.controller, 0);
    assert_eq!(g.battlefield_find(nim).unwrap().attached_to, Some(lent));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == borrowed));
}

/// Froghemoth exiles from the graveyard of the player it damaged ("their
/// graveyard"), never another opponent's (three seats).
#[test]
fn froghemoth_eats_only_the_damaged_players_graveyard() {
    let mut g = pod(3);
    let frog = ready(&mut g, 0, catalog::froghemoth());
    let mine = [g.add_card_to_graveyard(1, catalog::grizzly_bears()), g.add_card_to_graveyard(1, catalog::island())];
    let other = [g.add_card_to_graveyard(2, catalog::grizzly_bears()), g.add_card_to_graveyard(2, catalog::island())];
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: frog,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(mine.iter().all(|id| g.exile.iter().any(|c| c.id == *id)), "the damaged player's cards");
    assert!(other.iter().all(|id| g.players[2].graveyard.iter().any(|c| c.id == *id)), "not the other opponent's");
}

/// Rhys, the Evermore's "remove any number of counters" (CR 122.8): a count
/// per kind — here the stun counters go and the +1/+1 counter stays (it was
/// a fixed "harmful only" rule; any count is legal now).
#[test]
fn rhys_removes_the_harmful_counters_and_keeps_the_good_ones() {
    let mut g = pod(2);
    let rhys = ready(&mut g, 0, catalog::rhys_the_evermore());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    {
        let b = g.battlefield_find_mut(bear).unwrap();
        b.add_counters(CounterType::PlusOnePlusOne, 1);
        b.add_counters(CounterType::Stun, 2);
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(0), DecisionAnswer::Amount(2)]));
    activate(&mut g, rhys, 0, Some(Target::Permanent(bear))).expect("{W}, {T}");
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.counter_count(CounterType::Stun), b.counter_count(CounterType::PlusOnePlusOne)), (0, 1));
}
