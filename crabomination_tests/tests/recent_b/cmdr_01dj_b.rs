//! Commander: the 2026-10-19 (`01DJHkWk`) one-deck closers, second file
//! (`decks::cmdr_01dj_p` onward).

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
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 6);
    }
}

fn cast_targets(g: &mut GameState, card_id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers
            .iter()
            .map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn power(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).unwrap().power
}

/// CR 714.2b — Keldon Warcaller's attack trigger puts a lore counter on a
/// Saga you control, and the counter fires that chapter: History of Benalia
/// at chapter I goes to II and makes its second Knight.
#[test]
fn keldon_warcaller_advances_a_saga() {
    let mut g = pod(2);
    let saga = g.add_card_to_hand(0, catalog::history_of_benalia());
    cast_targets(&mut g, saga, &[]);
    let knights = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Knight").count();
    assert_eq!(knights(&g), 1);
    let wc = ready(&mut g, 0, catalog::keldon_warcaller());
    attack(&mut g, &[wc]);
    assert_eq!(g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore), 2);
    assert_eq!(knights(&g), 2, "chapter II made a second Knight");
}


/// Mind Transfer Protocol: a noncreature artifact becomes a 4/5 artifact
/// creature until end of turn, and you draw.
#[test]
fn mind_transfer_protocol_animates_and_draws() {
    let mut g = pod(2);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let mtp = g.add_card_to_hand(0, catalog::mind_transfer_protocol());
    let hand = g.players[0].hand.len();
    cast_targets(&mut g, mtp, &[Target::Permanent(ring)]);
    let cp = g.computed_permanent(ring).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
    assert!(cp.card_types().contains(&crabomination::card::CardType::Artifact));
    assert_eq!((cp.power, cp.toughness), (4, 5));
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
}

/// Donatello's Science Lesson taps two creatures and two different players
/// each draw — the third player does not.
#[test]
fn donatellos_science_lesson_taps_two_and_two_players_draw() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    let dsl = g.add_card_to_hand(0, catalog::donatellos_science_lesson());
    let hands: Vec<usize> = (0..3).map(|s| g.players[s].hand.len()).collect();
    cast_targets(
        &mut g,
        dsl,
        &[Target::Permanent(a), Target::Permanent(b), Target::Player(0), Target::Player(1)],
    );
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
    assert_eq!(g.players[0].hand.len(), hands[0], "cast one, drew one");
    assert_eq!(g.players[1].hand.len(), hands[1] + 1);
    assert_eq!(g.players[2].hand.len(), hands[2]);
}

/// CR 506.5 — "attacks alone": Agents of S.H.I.E.L.D. pumps a lone
/// attacker, and nothing when two attack.
#[test]
fn agents_of_shield_pumps_only_a_lone_attacker() {
    for (n, want) in [(1usize, 3), (2, 2)] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::agents_of_shield());
        let bears: Vec<CardId> = (0..n).map(|_| ready(&mut g, 0, catalog::grizzly_bears())).collect();
        attack(&mut g, &bears);
        assert_eq!(power(&g, bears[0]), want, "{n} attacker(s)");
    }
}

/// CR 506.5 — Luke Cage, Power Man attacking alone is a 4/5 indestructible.
#[test]
fn luke_cage_power_man_hardens_alone() {
    let mut g = pod(2);
    let luke = ready(&mut g, 0, catalog::luke_cage_power_man());
    attack(&mut g, &[luke]);
    let cp = g.computed_permanent(luke).unwrap();
    assert_eq!(cp.power, 4);
    assert!(cp.keywords().contains(&Keyword::Indestructible));
}

/// Fire of Orthanc destroys a land; the ground creature can't block this
/// turn, the flyer can.
#[test]
fn fire_of_orthanc_destroys_and_grounds_blockers() {
    let mut g = pod(2);
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bird = ready(&mut g, 1, catalog::storm_crow());
    let fire = g.add_card_to_hand(0, catalog::fire_of_orthanc());
    cast_targets(&mut g, fire, &[Target::Permanent(land)]);
    assert!(g.battlefield_find(land).is_none());
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::CantBlock));
    assert!(!g.computed_permanent(bird).unwrap().keywords().contains(&Keyword::CantBlock));
}


/// CR 115.1 — Cranial Archive's player slot carries a filter (it targeted
/// through a bare slot the walker can't see) and still resolves: the target
/// player's graveyard goes into their library, and its controller draws.
#[test]
fn cranial_archive_shuffles_the_target_players_graveyard() {
    let mut g = pod(3);
    let arc = ready(&mut g, 0, catalog::cranial_archive());
    g.add_card_to_graveyard(2, catalog::grizzly_bears());
    g.add_card_to_graveyard(2, catalog::lightning_bolt());
    let lib = g.players[2].library.len();
    let hand = g.players[0].hand.len();
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: arc,
        ability_index: 0,
        target: Some(Target::Player(2)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.players[2].graveyard.is_empty());
    assert_eq!(g.players[2].library.len(), lib + 2);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 506.5 — Strategic Intervention: a lone attacker gets +1/+1 and the
/// defending player's creature is tapped.
#[test]
fn strategic_intervention_pumps_and_taps_a_blocker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::strategic_intervention());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wall = ready(&mut g, 1, catalog::hill_giant());
    attack(&mut g, &[bear]);
    assert_eq!(power(&g, bear), 3);
    assert!(g.battlefield_find(wall).unwrap().tapped);
}

/// Crowd of True Believers pumps only a creature attacking alone, and gains
/// a life.
#[test]
fn crowd_of_true_believers_backs_a_lone_attacker() {
    let mut g = pod(2);
    let crowd = ready(&mut g, 0, catalog::crowd_of_true_believers());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let home = ready(&mut g, 0, catalog::grizzly_bears());
    attack(&mut g, &[bear]);
    let act = |g: &mut GameState, t: CardId| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: crowd,
            ability_index: 0,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(act(&mut g, home).is_err(), "the creature at home isn't attacking");
    act(&mut g, bear).expect("the lone attacker");
    drain_stack(&mut g);
    assert_eq!(power(&g, bear), 3);
    assert_eq!(g.players[0].life, 21);
}

/// Widow's Bite: one mode alone; with teamwork 3 both — deathtouch on yours
/// and -2/-2 kills theirs.
#[test]
fn widows_bite_both_with_teamwork() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::widows_bite());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: s,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("one mode");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());

    let mut g = pod(2);
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::widows_bite());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: s,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![Target::Permanent(bear)],
        x_value: None,
    })
    .expect("both modes, with teamwork");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    let cp = g.computed_permanent(giant).unwrap();
    assert!(cp.keywords().contains(&Keyword::Deathtouch));
    assert!(g.battlefield_find(giant).unwrap().tapped, "teamwork tapped the Giant");
}

// ── Sharae of Numbing Depths' Merfolk (`decks::cmdr_01dj_q`) ───────────────

/// CR 701.4a — Champions of the Shoal beholds and exiles a Merfolk from hand
/// as it's cast, stuns a creature on entering, and returns the exiled card
/// when it leaves.
#[test]
fn champions_of_the_shoal_beholds_stuns_and_returns() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let fish = g.add_card_to_hand(0, catalog::wanderbrine_trapper());
    let champ = g.add_card_to_hand(0, catalog::champions_of_the_shoal());
    cast_targets(&mut g, champ, &[]);
    assert!(g.exile.iter().any(|c| c.id == fish), "the beheld Merfolk is exiled");
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped && b.counter_count(CounterType::Stun) == 1);
    let bolt = g.add_card_to_hand(0, catalog::murder());
    cast_targets(&mut g, bolt, &[Target::Permanent(champ)]);
    assert!(g.players[0].hand.iter().any(|c| c.id == fish), "it comes back");
}

/// Eclipsed Merrow finds a Merfolk or a Plains/Island in the top four.
#[test]
fn eclipsed_merrow_finds_a_merfolk() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let fish = g.add_card_to_library(0, catalog::wanderbrine_trapper());
    let m = g.add_card_to_hand(0, catalog::eclipsed_merrow());
    cast_targets(&mut g, m, &[]);
    assert!(g.players[0].hand.iter().any(|c| c.id == fish));
    assert_eq!(g.players[0].library.len(), 3);
}

/// Meanders Guide taps another Merfolk to reanimate a small creature card.
#[test]
fn meanders_guide_taps_a_merfolk_to_reanimate() {
    let mut g = pod(2);
    let guide = ready(&mut g, 0, catalog::meanders_guide());
    let fish = ready(&mut g, 0, catalog::wanderbrine_trapper());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, &[guide]);
    assert!(g.battlefield_find(fish).unwrap().tapped);
    assert!(g.battlefield_find(dead).is_some(), "the Bears return");
}

/// Wanderbrine Trapper taps itself and another creature to tap an opposing
/// creature.
#[test]
fn wanderbrine_trapper_taps_an_opposing_creature() {
    let mut g = pod(2);
    let trap = ready(&mut g, 0, catalog::wanderbrine_trapper());
    let helper = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::hill_giant());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: trap,
        ability_index: 0,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
    assert!(g.battlefield_find(helper).unwrap().tapped && g.battlefield_find(trap).unwrap().tapped);
}

/// Turtles in Time bounces every creature, the takers shuffle hand and
/// graveyard away and draw seven, and the spell exiles itself.
#[test]
fn turtles_in_time_resets_the_takers() {
    let mut g = pod(3);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.add_card_to_graveyard(2, catalog::lightning_bolt());
    let tit = g.add_card_to_hand(0, catalog::turtles_in_time());
    // APNAP from the caster: seat 0 and 2 take it, seat 1 keeps the Bears.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(true),
    ]));
    cast_targets(&mut g, tit, &[]);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bounced, and seat 1 declined");
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[2].hand.len(), 7);
    assert!(g.players[2].graveyard.is_empty());
    assert!(g.exile.iter().any(|c| c.id == tit));
}
