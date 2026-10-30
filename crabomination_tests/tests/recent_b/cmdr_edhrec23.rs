//! Commander: the 2026-10-13 rescan's one-card gaps that needed more than a
//! catalog entry (`decks::cmdr_edhrec23`).

use crabomination::card::{CardId, CreatureType, Keyword};
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

fn cast_x(g: &mut GameState, seat: usize, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, seat, id, targets, None)
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).expect("on the battlefield");
    (cp.power, cp.toughness)
}

/// Raised by Giants (CR 903.3): your commander creatures are base 10/10
/// Giants; another creature isn't.
#[test]
fn raised_by_giants_grows_only_commanders() {
    let mut g = pod(2);
    let cmdr = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    let other = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::raised_by_giants());
    assert_eq!(pt(&g, cmdr), (10, 10));
    assert!(g.computed_permanent(cmdr).unwrap().subtypes().creature_types.contains(&CreatureType::Giant));
    assert_eq!(pt(&g, other), (2, 2));
}

/// Kamahl's Druidic Vow puts lands and legendary permanents with mana value X
/// or less onto the battlefield and bins the rest; it needs a legend to cast
/// (CR 205.4e).
#[test]
fn kamahls_druidic_vow_deploys_lands_and_legends() {
    let mut g = pod(2);
    for def in [catalog::forest, catalog::grizzly_bears, catalog::owen_grady_raptor_trainer] {
        g.add_card_to_library(0, def());
    }
    let vow = g.add_card_to_hand(0, catalog::kamahls_druidic_vow());
    assert!(cast_x(&mut g, 0, vow, &[], Some(3)).is_err(), "no legend in play");
    ready(&mut g, 0, catalog::kraven_the_hunter());
    cast_x(&mut g, 0, vow, &[], Some(3)).expect("cast");
    let on = |n: &str| g.battlefield.iter().any(|c| c.definition.name == n);
    assert!(on("Forest") && on("Owen Grady, Raptor Trainer"));
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Demonic Embrace: +3/+1, flying and a Demon; castable from the graveyard
/// for 3 life and a discard on top of its mana cost — additional costs, not
/// an alternative one (CR 601.2f), so a plain cast from the graveyard pays them.
#[test]
fn demonic_embrace_returns_from_the_graveyard() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_graveyard(0, catalog::demonic_embrace());
    g.add_card_to_hand(0, catalog::island());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("graveyard cast");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (5, 3));
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords().contains(&Keyword::Flying));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Demon));
    assert_eq!(g.players[0].life, 17);
    assert!(g.players[0].hand.is_empty(), "discarded");
}

/// Trepanation Blade: the defending player mills down to a land and the
/// attacker gets +1/+0 per card revealed.
#[test]
fn trepanation_blade_pumps_per_revealed_card() {
    let mut g = pod(2);
    for def in [catalog::grizzly_bears, catalog::lightning_bolt, catalog::forest, catalog::island] {
        g.add_card_to_library(1, def());
    }
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let blade = ready(&mut g, 0, catalog::trepanation_blade());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: blade, target: bear }).expect("equip");
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 3, "two spells and the land");
    assert_eq!(pt(&g, bear), (5, 2));
}

/// Kraven draws and grows when an opponent's biggest creature dies, not a
/// smaller one.
#[test]
fn kraven_hunts_the_biggest() {
    let mut g = pod(3);
    let kraven = ready(&mut g, 0, catalog::kraven_the_hunter());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let small = ready(&mut g, 1, catalog::grizzly_bears());
    let big = ready(&mut g, 1, catalog::hill_giant());
    for id in [small, big] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, 0, bolt, &[Target::Permanent(id)]).expect("bolt");
        g.check_state_based_actions();
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), 1, "only the Hill Giant counted");
    assert_eq!(pt(&g, kraven), (5, 4));
}

/// Display of Power copies a target instant.
#[test]
fn display_of_power_copies_spells() {
    let mut g = pod(2);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    let dop = g.add_card_to_hand(0, catalog::display_of_power());
    cast(&mut g, 0, dop, &[Target::Permanent(bolt)]).expect("copy");
    // The copy's controller may retarget it, and points it at their opponent.
    assert_eq!((g.players[0].life, g.players[1].life), (17, 17), "the bolt and its copy");
}

/// Fires of Invention: free spells up to your land count, at most two a
/// turn, and none on another player's turn.
#[test]
fn fires_of_invention_free_but_limited() {
    let mut g = pod(2);
    for _ in 0..2 {
        ready(&mut g, 0, catalog::forest());
    }
    ready(&mut g, 0, catalog::fires_of_invention());
    let bears: Vec<CardId> = (0..3).map(|_| g.add_card_to_hand(0, catalog::grizzly_bears())).collect();
    for &b in &bears[..2] {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: b,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("free");
        drain_stack(&mut g);
    }
    flood(&mut g, 0);
    assert!(cast(&mut g, 0, bears[2], &[]).is_err(), "a third spell");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.players[0].spells_cast_this_game_turn = 0;
    assert!(cast(&mut g, 0, bolt, &[Target::Player(1)]).is_err(), "not on another turn");
}

/// Interdimensional Web Watch: its two exiled cards are playable, and its
/// mana pays for a spell cast from exile but not from hand.
#[test]
fn web_watch_mana_only_for_exile_casts() {
    let mut g = pod(2);
    let exiled_bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let watch = g.add_card_to_hand(0, catalog::interdimensional_web_watch());
    cast(&mut g, 0, watch, &[]).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == exiled_bear), "exiled with a may-play");
    g.players[0].mana_pool = Default::default();
    g.clear_sickness(watch);
    let in_hand = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    let from_hand = g.perform_action(GameAction::CastSpell {
        card_id: in_hand,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(from_hand.is_err(), "Web Watch mana can't pay for a hand spell");
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: exiled_bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Web Watch pays for the exiled bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(exiled_bear).is_some());
}

/// Parker Luck: two target players each lose the other's top card's mana
/// value and put their own into their hand.
#[test]
fn parker_luck_crosses_the_reveals() {
    let mut g = pod(3);
    g.add_card_to_library(1, catalog::craw_wurm());
    g.add_card_to_library(2, catalog::lightning_bolt());
    let def = catalog::parker_luck();
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(1), Target::Player(2)];
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("resolve");
    assert_eq!((g.players[1].life, g.players[2].life), (19, 14), "the bolt's 1 and the wurm's 6");
    assert_eq!(g.players[1].hand[0].definition.name, "Craw Wurm");
    assert_eq!(g.players[2].hand[0].definition.name, "Lightning Bolt");
    assert_eq!(g.players[0].life, 20);
}

/// Vaevictis Asmadi, the Dire: each targeted permanent's controller
/// sacrifices it and deploys their top card if it's a permanent.
#[test]
fn vaevictis_sacrifices_and_redeploys() {
    let mut g = pod(3);
    let vae = ready(&mut g, 0, catalog::vaevictis_asmadi_the_dire());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let giant = ready(&mut g, 2, catalog::hill_giant());
    g.add_card_to_library(1, catalog::craw_wurm());
    g.add_card_to_library(2, catalog::lightning_bolt());
    let def = catalog::vaevictis_asmadi_the_dire();
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(vae);
    ctx.targets = vec![Target::Permanent(bear), Target::Permanent(giant)];
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("resolve");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(giant).is_none());
    let wurm = g.battlefield.iter().find(|c| c.definition.name == "Craw Wurm").expect("deployed");
    assert_eq!(wurm.controller, 1);
    assert_eq!(g.players[2].library.len(), 1, "the Bolt stays on top");
}

/// Desert Were-Worm grows per Mountain; a 12-power attack untaps the
/// attackers and banks an extra combat, once a turn.
#[test]
fn desert_were_worm_extra_combat_at_twelve_power() {
    let mut g = pod(2);
    for _ in 0..6 {
        ready(&mut g, 0, catalog::mountain());
    }
    let worm = ready(&mut g, 0, catalog::desert_were_worm());
    assert_eq!(pt(&g, worm), (12, 5));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: worm, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(worm).unwrap().tapped, "untapped");
    assert_eq!(g.additional_combat_phases, 1);
}

/// Thorn Mammoth fights when another creature of yours enters.
#[test]
fn thorn_mammoth_fights_on_entries() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::thorn_mammoth());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    cast(&mut g, 0, elf, &[]).expect("cast");
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "the Mammoth fought the bear");
}

/// Thicket Crasher gives other Elementals trample.
#[test]
fn thicket_crasher_lends_trample() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::thicket_crasher());
    let other = ready(&mut g, 0, catalog::thicket_crasher());
    let champ = ready(&mut g, 0, catalog::champion_of_the_path());
    assert!(g.computed_permanent(champ).unwrap().keywords().contains(&Keyword::Trample));
    assert!(g.computed_permanent(other).unwrap().keywords().contains(&Keyword::Trample));
}

/// Elminster's Simulacrum copies one creature per opponent.
#[test]
fn elminsters_simulacrum_copies_per_opponent() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::hill_giant());
    let id = g.add_card_to_hand(0, catalog::elminsters_simulacrum());
    cast(&mut g, 0, id, &[Target::Permanent(a), Target::Permanent(b)]).expect("cast");
    let mine: Vec<&str> = g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).map(|c| c.definition.name).collect();
    assert_eq!(mine.len(), 2, "{mine:?}");
}
