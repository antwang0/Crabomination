//! Commander: the Wade into Battle precon (C15, Kalemne, `decks::cmdr_kalemne`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn cast_raw(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    flood(g);
    cast_raw(g, id, targets).expect("cast");
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn has(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).unwrap().keywords().contains(&kw)
}

/// Anya counts opponents *below* half their starting life (CR 103.4): +3/+3
/// each, and indestructible while there is one. Exactly half doesn't count.
#[test]
fn anya_grows_per_opponent_below_half_life() {
    let mut g = main_phase(3);
    let start = g.players[1].starting_life;
    let anya = g.add_card_to_battlefield(0, catalog::anya_merciless_angel());
    g.players[1].life = start / 2;
    assert_eq!(pt(&g, anya), (4, 4), "exactly half is not less than half");
    assert!(!has(&g, anya, Keyword::Indestructible));
    g.players[1].life = start / 2 - 1;
    g.players[2].life = 1;
    assert_eq!(pt(&g, anya), (10, 10));
    assert!(has(&g, anya, Keyword::Indestructible));
    g.players[0].life = 1;
    assert_eq!(pt(&g, anya), (10, 10), "your own life doesn't count");
}

/// Arbiter of Knollridge: every life total becomes the highest.
#[test]
fn arbiter_of_knollridge_levels_life_up() {
    let mut g = main_phase(3);
    g.players[0].life = 3;
    g.players[1].life = 31;
    g.players[2].life = 12;
    let arb = g.add_card_to_hand(0, catalog::arbiter_of_knollridge());
    cast(&mut g, arb, &[]);
    assert!(g.players.iter().all(|p| p.life == 31));
}

/// Giants: Borderland Behemoth counts the *other* Giants you control; Sunrise
/// Sovereign pumps and grants trample to the others, not itself.
#[test]
fn giant_lords_count_the_other_giants() {
    let mut g = main_phase(2);
    let beh = g.add_card_to_battlefield(0, catalog::borderland_behemoth());
    assert_eq!(pt(&g, beh), (4, 4));
    let hill = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.add_card_to_battlefield(1, catalog::hill_giant());
    assert_eq!(pt(&g, beh), (8, 8), "an opponent's Giant doesn't count");
    let sov = g.add_card_to_battlefield(0, catalog::sunrise_sovereign());
    assert_eq!(pt(&g, beh), (14, 14), "two others, +2/+2 from the Sovereign");
    assert_eq!(pt(&g, hill), (5, 5));
    assert!(has(&g, hill, Keyword::Trample));
    assert_eq!(pt(&g, sov), (5, 5));
    assert!(!has(&g, sov, Keyword::Trample));
}

/// CR 508.1d — Curse of the Nightly Hunt: the enchanted player's creatures
/// must attack; nobody else's.
#[test]
fn curse_of_the_nightly_hunt_forces_attacks() {
    let mut g = main_phase(3);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let curse = g.add_card_to_hand(0, catalog::curse_of_the_nightly_hunt());
    cast(&mut g, curse, &[Target::Player(1)]);
    assert!(has(&g, theirs, Keyword::MustAttack));
    assert!(!has(&g, other, Keyword::MustAttack));
}

/// CR 601.2b — Disaster Radius needs a creature card to reveal and deals its
/// mana value to each opposing creature only.
#[test]
fn disaster_radius_reveals_for_its_damage() {
    let mut g = main_phase(2);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dr = g.add_card_to_hand(0, catalog::disaster_radius());
    flood(&mut g);
    assert!(cast_raw(&mut g, dr, &[]).is_err(), "nothing to reveal");
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::magma_giant());
    cast(&mut g, dr, &[]);
    assert!(g.battlefield_find(wurm).is_none(), "seven from the Magma Giant kills the 6/4");
    assert!(g.battlefield_find(mine).is_some());
    assert_eq!(g.players[0].hand.len(), 2, "revealed cards stay in hand");
}

/// Hamletback Goliath takes another creature's power as +1/+1 counters.
#[test]
fn hamletback_goliath_feeds_on_entrants() {
    let mut g = main_phase(2);
    let gol = g.add_card_to_battlefield(0, catalog::hamletback_goliath());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hill = g.add_card_to_hand(1, catalog::hill_giant());
    g.players[1].mana_pool.add(Color::Red, 4);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: hill, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("opponent casts");
    drain_stack(&mut g);
    assert_eq!(pt(&g, gol), (9, 9), "an opponent's creature counts too");
}

/// CR 615 — Hostility prevents your spells' damage to an opponent and mints
/// a 3/1 per point; damage to a creature is untouched. Put into a graveyard
/// it shuffles into its owner's library.
#[test]
fn hostility_turns_burn_into_shamans() {
    let mut g = main_phase(2);
    let host = g.add_card_to_battlefield(0, catalog::hostility());
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]);
    assert_eq!(g.players[1].life, life);
    let shamans = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Elemental Shaman").count();
    assert_eq!(shamans, 3);

    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(bear)]);
    assert!(g.battlefield_find(bear).is_none(), "creatures still take it");

    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, &[Target::Permanent(host)]);
    assert!(g.players[0].graveyard.iter().all(|c| c.definition.name != "Hostility"));
    assert!(g.players[0].library.iter().any(|c| c.definition.name == "Hostility"));
}

/// CR 701.37 — Kalemne's Captain becoming monstrous exiles every artifact and
/// enchantment, yours included.
#[test]
fn kalemnes_captain_monstrous_exiles_artifacts_and_enchantments() {
    let mut g = main_phase(2);
    let cap = g.add_card_to_battlefield(0, catalog::kalemnes_captain());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let theirs = g.add_card_to_battlefield(1, catalog::mind_stone());
    flood(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cap,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("monstrosity");
    drain_stack(&mut g);
    assert_eq!(pt(&g, cap), (8, 8));
    assert!(g.battlefield_find(ring).is_none() && g.battlefield_find(theirs).is_none());
}

/// Magma Giant: 2 to each creature and each player. Thundercloud Shaman:
/// your Giant count to each non-Giant.
#[test]
fn giant_sweepers() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mg = g.add_card_to_hand(0, catalog::magma_giant());
    cast(&mut g, mg, &[]);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, g.players[0].starting_life - 2);
    assert_eq!(g.players[1].life, g.players[1].starting_life - 2);

    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let ts = g.add_card_to_hand(0, catalog::thundercloud_shaman());
    cast(&mut g, ts, &[]);
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, 2, "two Giants: Magma and the Shaman");
    assert_eq!(g.battlefield_find(ts).unwrap().damage, 0, "Giants are spared");
}

/// Stinkdrinker Daredevil: a Giant costs {2} less.
#[test]
fn stinkdrinker_daredevil_discounts_giants() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::stinkdrinker_daredevil());
    let hill = g.add_card_to_hand(0, catalog::hill_giant());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast_raw(&mut g, hill, &[]).expect("{3}{R} for {1}{R}");
    assert!(g.battlefield_find(hill).is_some());
}

/// Dream Pillager — "you may cast spells from among those cards": a land it
/// exiles can't be played; a spell can be cast.
#[test]
fn dream_pillager_exiles_spells_to_cast_not_lands_to_play() {
    let mut g = main_phase(2);
    let pillager = g.add_card_to_battlefield(0, catalog::dream_pillager());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let land = g.add_card_to_library(0, catalog::mountain());
    let body = catalog::dream_pillager().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(pillager);
    ctx.event_amount = 2;
    let ev = g.resolve_effect(&body, &ctx).expect("trigger");
    g.dispatch_triggers_for_events(&ev);
    assert!(g.exile.iter().any(|c| c.id == land) && g.exile.iter().any(|c| c.id == bears));
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::PlayLand(land)).is_err(), "a land can't be played this way");
    flood(&mut g);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("a spell can be cast");
}
