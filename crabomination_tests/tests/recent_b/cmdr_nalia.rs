//! Commander: the Party Time precon (CLB, Nalia de'Arnise, `decks::cmdr_nalia`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Cleric, Rogue, Warrior and Wizard — a full party (CR 700.18).
fn full_party(g: &mut GameState) -> Vec<CardId> {
    [catalog::malakir_blood_priest, catalog::mages_attendant, catalog::mardu_strike_leader, catalog::galepowder_mage]
        .into_iter()
        .map(|f| g.add_card_to_battlefield(0, f()))
        .collect()
}

fn combat_begins(g: &mut GameState) {
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(g);
}

/// Nalia (CR 903.3): with a full party, your combat counters and deathtouch.
#[test]
fn nalia_rallies_a_full_party() {
    assert!(catalog::nalia_dearnise().can_be_commander);
    let mut g = pod(2);
    let n = g.add_card_to_battlefield(0, catalog::nalia_dearnise());
    let party = full_party(&mut g);
    combat_begins(&mut g);
    for id in party.iter().chain([&n]) {
        assert_eq!(g.battlefield_find(*id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert!(g.computed_permanent(*id).unwrap().keywords().contains(&Keyword::Deathtouch));
    }
}

/// Archpriest of Iona's power is the party's size; Malakir Blood-Priest
/// drains by it.
#[test]
fn party_size_counts() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::archpriest_of_iona());
    assert_eq!(g.computed_permanent(a).map(|c| c.power), Some(1), "itself, a Cleric");
    g.add_card_to_battlefield(0, catalog::mages_attendant());
    let bp = g.add_card_to_hand(0, catalog::malakir_blood_priest());
    flood(&mut g, 0);
    cast(&mut g, bp, &[]);
    // Two Clerics count once: Cleric + Rogue.
    assert_eq!(g.players[1].starting_life - g.players[1].life, 2);
    assert_eq!(g.computed_permanent(a).map(|c| c.power), Some(2));
}

/// Burakos drains the defending player by the party size and makes that
/// many Treasures; it is every role itself.
#[test]
fn burakos_leads_the_party() {
    let mut g = pod(2);
    let b = g.add_card_to_battlefield(0, catalog::burakos_party_leader());
    g.clear_sickness(b);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: b, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].starting_life - g.players[1].life, 1, "one creature, one party member");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count(), 1);
}

/// Calculating Lich: each creature attacking an opponent drains 1.
#[test]
fn calculating_lich_taxes_attacks() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::calculating_lich());
    let a = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    g.active_player_idx = 2;
    g.priority.player_with_priority = 2;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].starting_life - g.players[1].life, 1, "attacked opponent");
    assert_eq!(g.players[0].life, g.players[0].starting_life, "not us");
}

/// Deep Gnome Terramancer: a land put onto the battlefield (not played, CR
/// 305.1) under an opponent's control fetches a Plains; a played one doesn't.
#[test]
fn deep_gnome_terramancer_answers_ramp() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::deep_gnome_terramancer());
    let plains = g.add_card_to_library(0, catalog::plains());
    let played = g.add_card_to_hand(1, catalog::forest());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::PlayLand(played)).expect("land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(plains).is_none(), "a played land doesn't trigger");
    // Animist's Awakening puts the top Forest in with no question asked.
    let aa = g.add_card_to_hand(1, catalog::animists_awakening());
    flood(&mut g, 1);
    g.add_card_to_library(1, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: aa,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
    })
    .expect("ramp");
    drain_stack(&mut g);
    assert!(g.battlefield_find(plains).is_some_and(|c| c.tapped));
}

/// Solemn Doomguide grants unearth (CR 702.84) to a party-role creature card.
#[test]
fn solemn_doomguide_grants_unearth() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::solemn_doomguide());
    let lich = g.add_card_to_graveyard(0, catalog::calculating_lich());
    flood(&mut g, 0);
    let idx = catalog::calculating_lich().activated_abilities.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: lich,
        ability_index: idx,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("unearth");
    drain_stack(&mut g);
    assert!(g.battlefield_find(lich).is_some());
}

/// Thwart the Grave returns a creature card and a party-role one, cheaper by
/// the party.
#[test]
fn thwart_the_grave_returns_two() {
    let mut g = pod(2);
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let lich = g.add_card_to_graveyard(0, catalog::calculating_lich());
    let tg = g.add_card_to_hand(0, catalog::thwart_the_grave());
    flood(&mut g, 0);
    cast(&mut g, tg, &[Target::Permanent(wurm), Target::Permanent(lich)]);
    assert!(g.battlefield_find(wurm).is_some() && g.battlefield_find(lich).is_some());
}

/// Valiant Changeling costs {1} less per creature type among your creatures.
#[test]
fn valiant_changeling_discount() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mardu_strike_leader());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let vc = g.add_card_to_hand(0, catalog::valiant_changeling());
    // Human, Warrior, Bear: {5}{W}{W} less {3}.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, vc, &[]);
    assert!(g.battlefield_find(vc).is_some());
}

/// Multiclass Baldric: lifelink with a Cleric; full party prevents all damage
/// to the equipped creature.
#[test]
fn multiclass_baldric_by_role() {
    let mut g = pod(2);
    let mb = g.add_card_to_battlefield(0, catalog::multiclass_baldric());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mb).unwrap().attached_to = Some(bear);
    g.add_card_to_battlefield(0, catalog::malakir_blood_priest());
    let kws = g.computed_permanent(bear).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::Lifelink) && !kws.contains(&Keyword::Flying));
    full_party(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast(&mut g, bolt, &[Target::Permanent(bear)]);
    assert!(g.battlefield_find(bear).is_some(), "the damage was prevented");
}

/// Grim Hireling: a hit makes two Treasures; sacrificing both gives -2/-2.
#[test]
fn grim_hireling_spends_treasure() {
    let mut g = pod(2);
    let gh = g.add_card_to_battlefield(0, catalog::grim_hireling());
    g.clear_sickness(gh);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: gh, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count(), 2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gh,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// Folk Hero (a Background): your commander draws off a spell sharing its
/// creature type, once each turn (CR 603.3d) — an off-type spell first
/// doesn't spend it.
#[test]
fn folk_hero_draws_once_a_turn() {
    let mut g = pod(2);
    let cmdr = g.add_card_to_battlefield(0, catalog::nalia_dearnise());
    g.players[0].commanders.push(cmdr);
    g.add_card_to_battlefield(0, catalog::folk_hero());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    flood(&mut g, 0);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]);
    let hand = g.players[0].hand.len();
    for f in [catalog::mages_attendant, catalog::nashi_moon_sages_scion] {
        let rogue = g.add_card_to_hand(0, f());
        cast(&mut g, rogue, &[]);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw for two Rogue spells");
}
