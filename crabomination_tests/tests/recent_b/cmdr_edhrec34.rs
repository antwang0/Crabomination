//! Commander: Niv-Mizzet Reborn's / Supreme's, Trelasarra's, Shanna's,
//! Ramses's, Varolz's and Horde of Notions's last gaps (`decks::cmdr_edhrec34`).

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

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g);
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn end_step(g: &mut GameState) {
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(g);
}

/// CR 702.133 — Niv-Mizzet, Supreme gives a two-color instant in your
/// graveyard jump-start; a one-color one stays put.
#[test]
fn cr_702_133_niv_mizzet_supreme_grants_jump_start_to_two_color_cards() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::niv_mizzet_supreme());
    let helix = g.add_card_to_graveyard(0, catalog::lightning_helix());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    flood(&mut g);
    let from_gy = |g: &mut GameState, id| {
        act(g, GameAction::CastFlashback {
            card_id: id,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(from_gy(&mut g, bolt).is_err(), "Lightning Bolt is one color");
    from_gy(&mut g, helix).expect("Lightning Helix jump-starts");
    assert!(g.exile.iter().any(|c| c.id == helix), "exiled after resolving");
    assert_eq!(g.players[1].life, 17);
}

/// Cosmic Rebirth: a mana value 3 or less permanent card returns to the
/// battlefield, a bigger one to hand; 3 life either way.
#[test]
fn cosmic_rebirth_reanimates_small_and_returns_big() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let life = g.players[0].life;
    let a = g.add_card_to_hand(0, catalog::cosmic_rebirth());
    cast(&mut g, a, Some(Target::Permanent(bear))).expect("small");
    assert!(g.battlefield_find(bear).is_some());
    let b = g.add_card_to_hand(0, catalog::cosmic_rebirth());
    cast(&mut g, b, Some(Target::Permanent(wurm))).expect("big");
    assert!(g.players[0].hand.iter().any(|c| c.id == wurm));
    assert_eq!(g.players[0].life, life + 6);
}

/// Shanna draws X at your end step, X capped by the life you gained this
/// turn; Bloombrute draws once a turn on life gain.
#[test]
fn shanna_draws_up_to_life_gained_and_bloombrute_once_a_turn() {
    let mut g = pod(2);
    stock_libraries(&mut g, 20);
    ready(&mut g, 0, catalog::shanna_purifying_blade());
    ready(&mut g, 0, catalog::bloombrute());
    let hand = g.players[0].hand.len();
    let helix = g.add_card_to_hand(0, catalog::lightning_helix());
    cast(&mut g, helix, Some(Target::Player(1))).expect("helix");
    let helix = g.add_card_to_hand(0, catalog::lightning_helix());
    cast(&mut g, helix, Some(Target::Player(1))).expect("helix");
    assert_eq!(g.players[0].hand.len(), hand + 1, "Bloombrute: one draw for two life gains");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(10)]));
    end_step(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1 + 6, "X capped at the 6 life gained");
}

/// Kwia Vigorbloom's once-a-turn Lotus taps for three mana of one color.
#[test]
fn kwia_vigorbloom_makes_one_lotus_a_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::kwia_vigorbloom());
    for _ in 0..2 {
        let helix = g.add_card_to_hand(0, catalog::lightning_helix());
        cast(&mut g, helix, Some(Target::Player(1))).expect("helix");
    }
    let lotus: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.definition.name == "Lotus").map(|c| c.id).collect();
    assert_eq!(lotus.len(), 1);
    g.players[0].mana_pool = Default::default();
    act(&mut g, GameAction::ActivateAbility {
        card_id: lotus[0],
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("lotus");
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// Brotherhood Spy gets +1/+0 and can't be blocked only beside a legendary
/// Assassin.
#[test]
fn brotherhood_spy_needs_a_legendary_assassin() {
    let mut g = pod(2);
    let spy = ready(&mut g, 0, catalog::brotherhood_spy());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(spy).unwrap().power, 1);
    ready(&mut g, 0, catalog::ramses_assassin_lord());
    let base = g.computed_permanent(spy).unwrap().power;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let cp = g.computed_permanent(spy).unwrap();
    assert_eq!(cp.power, base + 1);
    assert!(cp.keywords().contains(&Keyword::Unblockable));
}

/// Bristlebane Battler sheds a -1/-1 counter per creature entering after it.
#[test]
fn bristlebane_battler_grows_with_each_entry() {
    let mut g = pod(2);
    let battler = g.add_card_to_hand(0, catalog::bristlebane_battler());
    cast(&mut g, battler, None).expect("battler");
    assert_eq!(g.computed_permanent(battler).unwrap().power, 1);
    for _ in 0..2 {
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        cast(&mut g, bear, None).expect("bear");
    }
    let c = g.battlefield_find(battler).unwrap();
    assert_eq!(c.counter_count(CounterType::MinusOneMinusOne), 3);
}

/// Nyxathid shrinks by the chosen opponent's hand.
#[test]
fn nyxathid_counts_the_chosen_opponents_hand() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::island());
    }
    let nyx = g.add_card_to_hand(0, catalog::nyxathid());
    cast(&mut g, nyx, None).expect("nyxathid");
    assert_eq!(g.computed_permanent(nyx).unwrap().power, 4);
}

/// Torrent Elemental taps the defending player's creatures as it attacks and
/// comes back from exile tapped.
#[test]
fn torrent_elemental_taps_the_defenders_and_returns_from_exile() {
    let mut g = pod(3);
    let torrent = ready(&mut g, 0, catalog::torrent_elemental());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let bystander = ready(&mut g, 2, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    act(&mut g, GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: torrent,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    assert!(g.battlefield_find(theirs).unwrap().tapped);
    assert!(!g.battlefield_find(bystander).unwrap().tapped);
    let exiled = g.add_card_to_exile(0, catalog::torrent_elemental());
    g.step = TurnStep::PreCombatMain;
    flood(&mut g);
    act(&mut g, GameAction::ActivateAbility {
        card_id: exiled,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("return from exile");
    assert!(g.battlefield_find(exiled).is_some_and(|c| c.tapped));
}
