//! Commander: the Silverquill Influence precon (SOC, Killian,
//! `decks::cmdr_killian`).

use crabomination::card::{CardId, CounterType};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn sac(g: &mut GameState, seat: usize, id: CardId) {
    let mut evs = Vec::new();
    g.sacrifice_one(id, seat, &mut evs);
    drain_stack(g);
}

fn step(g: &mut GameState, s: TurnStep) {
    g.step = s;
    g.fire_step_triggers(s);
    drain_stack(g);
}

/// Seat `seat` (made active) declares `attacks`.
fn declare(g: &mut GameState, seat: usize, attacks: Vec<(CardId, usize)>) -> Result<(), String> {
    g.active_player_idx = seat;
    for (a, _) in &attacks {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, d)| Attack { attacker, target: AttackTarget::Player(d) }).collect(),
    ))
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Killian — an enchantment of yours entering taps and goads an opponent's
/// creature (CR 701.15); a creature enchanted by your Aura attacking draws.
#[test]
fn killian_goads_on_enchantments_and_draws_on_enchanted_attacks() {
    let mut g = pod(3);
    stock_libraries(&mut g, 5);
    g.add_card_to_battlefield(0, catalog::killian_decisive_mentor());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pi = g.add_card_to_hand(0, catalog::parasitic_impetus());
    cast(&mut g, 0, pi, Some(Target::Permanent(bear))).expect("cast the Aura");
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped, "tapped by Killian");
    assert!(g.goaded_by_player(b, 0), "goaded by Killian's controller");
    // The enchanted bear attacks seat 2: Killian's controller draws.
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().tapped = false;
    let h = g.players[0].hand.len();
    declare(&mut g, 1, vec![(bear, 2)]).expect("attack");
    assert_eq!(g.players[0].hand.len(), h + 1);
}

/// Killian's draw ignores a creature enchanted only by an opponent's Aura.
#[test]
fn killian_ignores_other_players_auras() {
    let mut g = pod(3);
    stock_libraries(&mut g, 5);
    g.add_card_to_battlefield(0, catalog::killian_decisive_mentor());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rg = g.add_card_to_hand(1, catalog::raffines_guidance());
    g.active_player_idx = 1;
    cast(&mut g, 1, rg, Some(Target::Permanent(bear))).expect("cast");
    let h = g.players[0].hand.len();
    declare(&mut g, 1, vec![(bear, 2)]).expect("attack");
    assert_eq!(g.players[0].hand.len(), h);
}

/// Eriette — a creature enchanted by your Aura can't attack you (CR 508.1c),
/// and your end step drains each opponent for your Aura count (you gain X once).
#[test]
fn eriette_shields_you_and_drains_per_aura() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::eriette_of_the_charmed_apple());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rg = g.add_card_to_hand(0, catalog::raffines_guidance());
    cast(&mut g, 0, rg, Some(Target::Permanent(bear))).expect("cast");
    assert!(declare(&mut g, 1, vec![(bear, 0)]).is_err(), "can't attack Eriette's controller");
    let mut g2 = pod(3);
    g2.add_card_to_battlefield(0, catalog::eriette_of_the_charmed_apple());
    let bear = g2.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..2 {
        let rg = g2.add_card_to_hand(0, catalog::raffines_guidance());
        cast(&mut g2, 0, rg, Some(Target::Permanent(bear))).expect("cast");
    }
    let (l0, l1, l2) = (g2.players[0].life, g2.players[1].life, g2.players[2].life);
    step(&mut g2, TurnStep::End);
    assert_eq!((g2.players[0].life, g2.players[1].life, g2.players[2].life), (l0 + 2, l1 - 2, l2 - 2));
}

/// Raffine's Guidance pumps its host and can be cast from the graveyard.
#[test]
fn raffines_guidance_recasts_from_graveyard() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rg = g.add_card_to_hand(0, catalog::raffines_guidance());
    cast(&mut g, 0, rg, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(pt(&g, bear), (3, 3));
    let rg = g.battlefield.iter().find(|c| c.definition.name == "Raffine's Guidance").unwrap().id;
    sac(&mut g, 0, rg);
    assert_eq!(pt(&g, bear), (2, 2));
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: rg,
        pitch_card: None,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from graveyard");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (3, 3));
}

/// Coercive Impetus — +1/+1; the host attacking draws its caster a card for 1
/// life.
#[test]
fn coercive_impetus_pumps_and_draws() {
    let mut g = pod(3);
    stock_libraries(&mut g, 5);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ci = g.add_card_to_hand(0, catalog::coercive_impetus());
    cast(&mut g, 0, ci, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(pt(&g, bear), (3, 3));
    step(&mut g, TurnStep::BeginCombat);
    assert!(g.goaded_by_player(g.battlefield_find(bear).unwrap(), 0));
    let (h, l) = (g.players[0].hand.len(), g.players[0].life);
    declare(&mut g, 1, vec![(bear, 2)]).expect("attack");
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (h + 1, l - 1));
}

/// Changing Loyalty — the enchanted creature dying returns under the Aura's
/// controller.
#[test]
fn changing_loyalty_steals_the_dead() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cl = g.add_card_to_hand(0, catalog::changing_loyalty());
    cast(&mut g, 0, cl, Some(Target::Permanent(bear))).expect("cast");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(bear))).expect("bolt");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1, "returns under seat 0");
}

/// Herald of Amity casts an Aura from the top eight for free (CR 601.2).
#[test]
fn herald_of_amity_casts_an_aura_free() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::raffines_guidance());
    let herald = g.add_card_to_hand(0, catalog::herald_of_amity());
    cast(&mut g, 0, herald, None).expect("cast");
    let herald = named(&g, 0, "Herald of Amity")[0];
    let rg = named(&g, 0, "Raffine's Guidance");
    assert_eq!(rg.len(), 1, "the Aura was cast");
    let host = g.battlefield_find(rg[0]).unwrap().attached_to.unwrap();
    assert!(host == bear || host == herald);
}

/// Defacing Duskmage prepares on an opponent's second draw of a turn; Vandal's
/// Edit draws two and costs each player 2 life.
#[test]
fn defacing_duskmage_prepares_on_second_draw() {
    let mut g = pod(2);
    stock_libraries(&mut g, 8);
    let dm = g.add_card_to_battlefield(0, catalog::defacing_duskmage());
    let opt = g.add_card_to_hand(1, catalog::opt());
    cast(&mut g, 1, opt, None).expect("opt");
    assert_eq!(g.battlefield_find(dm).unwrap().counter_count(CounterType::Prepared), 0);
    let opt = g.add_card_to_hand(1, catalog::opt());
    cast(&mut g, 1, opt, None).expect("opt");
    assert_eq!(g.battlefield_find(dm).unwrap().counter_count(CounterType::Prepared), 1);
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    let (h, l0, l1) = (g.players[0].hand.len(), g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::CastPrepareSpell { creature_id: dm, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Vandal's Edit");
    drain_stack(&mut g);
    assert_eq!((g.players[0].hand.len(), g.players[0].life, g.players[1].life), (h + 2, l0 - 2, l1 - 2));
}

/// Eiganjo Dynastorian prepares when you attack with two or more; Replenish
/// returns your graveyard's enchantments.
#[test]
fn eiganjo_dynastorian_prepares_replenish() {
    let mut g = pod(2);
    let ed = g.add_card_to_battlefield(0, catalog::eiganjo_dynastorian());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::forum_filibuster());
    declare(&mut g, 0, vec![(ed, 1), (bear, 1)]).expect("attack");
    assert_eq!(g.battlefield_find(ed).unwrap().counter_count(CounterType::Prepared), 1);
    g.step = TurnStep::PostCombatMain;
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastPrepareSpell { creature_id: ed, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Replenish");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Forum Filibuster").len(), 1);
}

/// Forum Filibuster makes an Inkling each upkeep and returns an Aura from the
/// graveyard onto it.
#[test]
fn forum_filibuster_makes_inkling_and_returns_aura() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::forum_filibuster());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rg = g.add_card_to_hand(0, catalog::raffines_guidance());
    cast(&mut g, 0, rg, Some(Target::Permanent(bear))).expect("cast");
    let rg = named(&g, 0, "Raffine's Guidance")[0];
    sac(&mut g, 0, rg);
    step(&mut g, TurnStep::Upkeep);
    let ink = named(&g, 0, "Inkling");
    assert_eq!(ink.len(), 1);
    assert_eq!(pt(&g, ink[0]), (3, 2), "the Aura returned attached to the Inkling");
}

/// Intermediate Chirography — Inkling on entry; level 2 counters your first
/// life loss each turn onto a creature (CR 716.2a).
#[test]
fn intermediate_chirography_levels() {
    let mut g = pod(2);
    let ic = g.add_card_to_hand(0, catalog::intermediate_chirography());
    cast(&mut g, 0, ic, None).expect("cast");
    let ic = named(&g, 0, "Intermediate Chirography")[0];
    let ink = named(&g, 0, "Inkling");
    assert_eq!(ink.len(), 1);
    activate(&mut g, 0, ic, 0, None).expect("level 2");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0))).expect("bolt");
    assert_eq!(g.battlefield_find(ink[0]).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0))).expect("bolt");
    assert_eq!(g.battlefield_find(ink[0]).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "once per turn");
}

/// Intermediate Chirography level 3: a creature of yours modified only by an
/// Aura you control dying makes an Inkling at the end step; an unmodified
/// one doesn't. Regression (CR 700.9 / 603.10a): the death snapshot's
/// attachments were invisible, so the card read "modified" as "had counters".
#[test]
fn intermediate_chirography_level_3_counts_an_aura_modified_death() {
    // One Inkling from the Class entering; a second only for the modified death.
    for (aura, inklings) in [(false, 1), (true, 2)] {
        let mut g = pod(2);
        let ic = g.add_card_to_hand(0, catalog::intermediate_chirography());
        cast(&mut g, 0, ic, None).expect("cast");
        let ic = named(&g, 0, "Intermediate Chirography")[0];
        activate(&mut g, 0, ic, 0, None).expect("level 2");
        activate(&mut g, 0, ic, 1, None).expect("level 3");
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        if aura {
            let r = g.add_card_to_battlefield(0, catalog::rancor());
            g.battlefield_find_mut(r).unwrap().attached_to = Some(bear);
        }
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        cast(&mut g, 1, bolt, Some(Target::Permanent(bear))).expect("bolt");
        step(&mut g, TurnStep::End);
        assert_eq!(named(&g, 0, "Inkling").len(), inklings);
    }
}

/// Scriv attaches a Contract to an opponent's creature; that creature
/// attacking another opponent gets +2/+0, attacking you costs its controller 2.
#[test]
fn scriv_contracts() {
    let mut g = pod(3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let scriv = g.add_card_to_hand(0, catalog::scriv_the_obligator());
    cast(&mut g, 0, scriv, None).expect("cast");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Contract" && c.attached_to == Some(bear)));
    declare(&mut g, 1, vec![(bear, 2)]).expect("attack");
    assert_eq!(pt(&g, bear), (4, 2));
    let mut g2 = pod(3);
    let bear = g2.add_card_to_battlefield(1, catalog::grizzly_bears());
    let scriv = g2.add_card_to_hand(0, catalog::scriv_the_obligator());
    cast(&mut g2, 0, scriv, None).expect("cast");
    let l = g2.players[1].life;
    declare(&mut g2, 1, vec![(bear, 0)]).expect("attack Scriv's controller");
    assert_eq!(g2.players[1].life, l - 2);
    assert_eq!(pt(&g2, bear), (2, 2));
}

/// Chains of Custody exiles an opponent's nonland permanent until it leaves.
#[test]
fn chains_of_custody_exiles_until_it_leaves() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cc = g.add_card_to_hand(0, catalog::chains_of_custody());
    cast(&mut g, 0, cc, Some(Target::Permanent(bear))).expect("cast");
    if g.battlefield_find(theirs).is_some() {
        // The ETB target was picked automatically; point it explicitly.
        panic!("the opponent's bear should be exiled");
    }
    let cc = named(&g, 0, "Chains of Custody")[0];
    sac(&mut g, 0, cc);
    assert_eq!(named(&g, 1, "Grizzly Bears").len(), 1, "returned");
}

/// Turbulent Moor enters tapped unless opponents control eight lands; Sunlit
/// Marsh always enters tapped.
#[test]
fn silverquill_duals_enter_tapped() {
    let mut g = pod(2);
    let tm = g.add_card_to_hand(0, catalog::turbulent_moor());
    g.perform_action(GameAction::PlayLand(tm)).expect("land");
    assert!(g.battlefield_find(tm).unwrap().tapped);
    let mut g = pod(2);
    for _ in 0..8 {
        g.add_card_to_battlefield(1, catalog::plains());
    }
    let tm = g.add_card_to_hand(0, catalog::turbulent_moor());
    g.perform_action(GameAction::PlayLand(tm)).expect("land");
    assert!(!g.battlefield_find(tm).unwrap().tapped);
}
