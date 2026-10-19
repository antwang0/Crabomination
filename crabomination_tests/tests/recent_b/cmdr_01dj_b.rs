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

// ── The party pair and Captain America, Unbowed (`decks::cmdr_01dj_r`) ────

fn begin_combat(g: &mut GameState) {
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(g);
}

/// Base Camp's coloured mana pays for a Rogue (Merfolk Looter) but not a
/// Bear.
#[test]
fn base_camp_funds_only_party_types() {
    for (rogue, ok) in [(true, true), (false, false)] {
        let mut g = pod(2);
        let camp = ready(&mut g, 0, catalog::base_camp());
        g.perform_action(GameAction::ActivateAbility {
            card_id: camp,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap for party mana");
        drain_stack(&mut g);
        let (def, colour) = if rogue {
            (catalog::merfolk_looter(), Color::Blue)
        } else {
            (catalog::grizzly_bears(), Color::Green)
        };
        g.players[0].mana_pool.add(colour, 1);
        let id = g.add_card_to_hand(0, def);
        let cast = g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        assert_eq!(cast.is_ok(), ok, "rogue={rogue}");
    }
}

/// The Destined Warrior's combat pump is +1/+0 alone and +3/+0 with a full
/// party (CR 700.8: a Warrior, a Rogue, a Wizard and a Cleric).
#[test]
fn the_destined_warrior_pumps_harder_with_a_full_party() {
    for (full, want) in [(false, 3), (true, 5)] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::the_destined_warrior());
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        if full {
            ready(&mut g, 0, catalog::merfolk_looter());
            ready(&mut g, 0, catalog::prodigal_sorcerer());
            ready(&mut g, 0, catalog::soul_warden());
        }
        begin_combat(&mut g);
        assert_eq!(power(&g, bear), want, "full party: {full}");
    }
}

/// Skyclave Plunder looks at 3 + party (here 2: a Rogue and a Wizard) and
/// keeps three.
#[test]
fn skyclave_plunder_digs_deeper_with_a_party() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::merfolk_looter());
    ready(&mut g, 0, catalog::prodigal_sorcerer());
    let lib = g.players[0].library.len();
    let s = g.add_card_to_hand(0, catalog::skyclave_plunder());
    let hand = g.players[0].hand.len();
    cast_targets(&mut g, s, &[]);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3);
    assert_eq!(g.players[0].library.len(), lib - 3);
}

/// Practiced Tactics deals twice the party size to an attacker.
#[test]
fn practiced_tactics_scales_with_the_party() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::merfolk_looter());
    ready(&mut g, 1, catalog::soul_warden());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    attack(&mut g, &[giant]);
    let s = g.add_card_to_hand(1, catalog::practiced_tactics());
    g.players[1].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: s,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "4 damage kills the 3/3");
}

/// Infinity Formula attaches as it enters: +1/+2, and the equipped
/// creature's attack gains 2 life.
#[test]
fn infinity_formula_attaches_and_gains_on_attack() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let f = g.add_card_to_hand(0, catalog::infinity_formula());
    cast_targets(&mut g, f, &[]);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4));
    let life = g.players[0].life;
    attack(&mut g, &[bear]);
    assert_eq!(g.players[0].life, life + 2);
}

// ── T'Pol, the Enterprise-D and Arzakon (`decks::cmdr_01dj_s`) ─────────────

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// T'Pol taps an opposing creature and puts two stun counters on it.
#[test]
fn tpol_stuns_an_opposing_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::tpol_vulcan_representative());
    cast_targets(&mut g, t, &[]);
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped);
    assert_eq!(b.counter_count(CounterType::Stun), 2);
}

/// Cybernetic Specialist puts a graveyard card on the bottom of its owner's
/// library.
#[test]
fn cybernetic_specialist_recycles_a_graveyard_card() {
    let mut g = pod(2);
    let spec = ready(&mut g, 0, catalog::cybernetic_specialist());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    activate(&mut g, spec, 0, Some(Target::Permanent(bolt)));
    assert!(g.players[0].graveyard.is_empty());
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bolt));
}

/// CR 702.184 — stationing the Enterprise-D puts charge counters on it and,
/// the first time each turn, exiles the top card to play this turn; it is a
/// 4/5 flying creature at seven.
#[test]
fn enterprise_d_stations_and_exiles_the_top_card() {
    let mut g = pod(2);
    let ship = g.add_card_to_battlefield(0, catalog::uss_enterprise_d_galaxy_class());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let top = g.players[0].library.first().map(|c| c.id).unwrap();
    g.battlefield_find_mut(ship).unwrap().add_counters(CounterType::Charge, 4);
    activate(&mut g, ship, 0, None);
    assert_eq!(g.battlefield_find(ship).unwrap().counter_count(CounterType::Charge), 7);
    assert!(g.exile.iter().any(|c| c.id == top), "the top card is exiled");
    let cp = g.computed_permanent(ship).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
    assert!(cp.keywords().contains(&Keyword::Flying));
    let _ = giant;
}

/// Arzakon's −3 makes a Black Lotus (three mana of one colour), and its −7
/// has every player shuffle hand and graveyard away and draw seven.
#[test]
fn arzakon_makes_a_lotus_and_resets_hands() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::arzakon());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::Loyalty, 10);
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: a, ability_index: 1, target: None, x_value: None })
        .expect("−3");
    drain_stack(&mut g);
    let lotus = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Black Lotus")
        .map(|c| c.id)
        .expect("a Black Lotus token");
    activate(&mut g, lotus, 0, None);
    assert_eq!(g.players[0].mana_pool.total(), 3);
    assert!(g.battlefield_find(lotus).is_none(), "sacrificed");

    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::arzakon());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::Loyalty, 10);
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: a, ability_index: 2, target: None, x_value: None })
        .expect("−7");
    drain_stack(&mut g);
    for p in 0..2 {
        assert_eq!(g.players[p].hand.len(), 7, "seat {p}");
        assert!(g.players[p].graveyard.is_empty(), "seat {p}");
    }
}

/// Tam proliferates once per planeswalker type you control: two walkers of
/// different types add two loyalty to each.
#[test]
fn tam_proliferates_per_planeswalker_type() {
    let mut g = pod(2);
    let tam = ready(&mut g, 0, catalog::tam_the_possibility());
    let a = g.add_card_to_battlefield(0, catalog::arzakon());
    let t = g.add_card_to_battlefield(0, catalog::liliana_of_the_veil());
    let before = [a, t].map(|id| g.battlefield_find(id).unwrap().counter_count(CounterType::Loyalty));
    flood(&mut g, 0);
    activate(&mut g, tam, 0, None);
    for (i, id) in [a, t].into_iter().enumerate() {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Loyalty), before[i] + 2);
    }
}

// ── Three-gap closers (`decks::cmdr_01dj_t`) ───────────────────────────────

/// Soldier On: two +1/+1 counters and an untap.
#[test]
fn soldier_on_counters_and_untaps() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let s = g.add_card_to_hand(0, catalog::soldier_on());
    cast_targets(&mut g, s, &[Target::Permanent(bear)]);
    let b = g.battlefield_find(bear).unwrap();
    assert!(!b.tapped);
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Hawkeye's Shot costs {2} less at a tapped creature: three mana exiles a
/// tapped Bear and can't pay for an untapped one.
#[test]
fn hawkeyes_shot_is_cheaper_at_a_tapped_creature() {
    for (tapped, ok) in [(true, true), (false, false)] {
        let mut g = pod(2);
        let bear = ready(&mut g, 1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = tapped;
        let s = g.add_card_to_hand(0, catalog::hawkeyes_shot());
        g.players[0].mana_pool.add(Color::White, 3);
        let cast = g.perform_action(GameAction::CastSpell {
            card_id: s,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        assert_eq!(cast.is_ok(), ok, "tapped={tapped}");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).is_none(), ok);
    }
}

/// Entrust the Spark trades a planeswalker for one from the library.
#[test]
fn entrust_the_spark_swaps_a_planeswalker() {
    let mut g = pod(2);
    let arz = g.add_card_to_battlefield(0, catalog::arzakon());
    let lili = g.add_card_to_library(0, catalog::liliana_of_the_veil());
    let s = g.add_card_to_hand(0, catalog::entrust_the_spark());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast_targets(&mut g, s, &[]);
    assert!(g.battlefield_find(arz).is_none(), "sacrificed");
    assert!(g.battlefield_find(lili).is_some(), "searched onto the battlefield");
}

/// Mouser Mark III can't attack alone among artifacts; another artifact
/// frees it.
#[test]
fn mouser_mark_iii_needs_another_artifact() {
    for (ring, ok) in [(false, false), (true, true)] {
        let mut g = pod(2);
        let m = ready(&mut g, 0, catalog::mouser_mark_iii());
        if ring {
            g.add_card_to_battlefield(0, catalog::sol_ring());
        }
        g.step = TurnStep::DeclareAttackers;
        let r = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: m,
            target: AttackTarget::Player(1),
        }]));
        assert_eq!(r.is_ok(), ok, "ring={ring}");
    }
}

/// Nobody bounces another artifact you control and scries; Rock Soldiers
/// destroys a noncreature artifact.
#[test]
fn nobody_bounces_and_rock_soldiers_destroys_artifacts() {
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let n = g.add_card_to_hand(0, catalog::nobody());
    cast_targets(&mut g, n, &[Target::Permanent(ring)]);
    assert!(g.players[0].hand.iter().any(|c| c.id == ring));

    let mut g = pod(2);
    let theirs = g.add_card_to_battlefield(1, catalog::sol_ring());
    let rs = g.add_card_to_hand(0, catalog::rock_soldiers());
    cast_targets(&mut g, rs, &[Target::Permanent(theirs)]);
    assert!(g.battlefield_find(theirs).is_none());
}

/// Transmogrant Altar turns a creature into {C}{C}{C}, or (sorcery speed)
/// into a 3/3 Zombie artifact creature.
#[test]
fn transmogrant_altar_sacrifices_for_mana_or_a_zombie() {
    let mut g = pod(2);
    let altar = g.add_card_to_battlefield(0, catalog::transmogrant_altar());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    activate(&mut g, altar, 0, None);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].mana_pool.total(), 3);

    let mut g = pod(2);
    let altar = g.add_card_to_battlefield(0, catalog::transmogrant_altar());
    ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 2);
    activate(&mut g, altar, 1, None);
    let z = g.battlefield.iter().find(|c| c.definition.name == "Zombie").expect("a Zombie");
    assert!(z.definition.card_types.contains(&crabomination::card::CardType::Artifact));
}

/// Ceti Eel mills two, then grows per artifact/creature card in your
/// graveyard (two Bears milled + one already there = three).
#[test]
fn ceti_eel_grows_from_the_graveyard() {
    let mut g = pod(2);
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::sol_ring());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let e = g.add_card_to_hand(0, catalog::ceti_eel());
    cast_targets(&mut g, e, &[]);
    assert_eq!(g.battlefield_find(e).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// V'Ger's entry, second mode: opposing creatures get -1/-0 until your next
/// turn.
#[test]
fn vger_shrinks_opposing_attackers() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let v = g.add_card_to_hand(0, catalog::vger_the_intruder());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    cast_targets(&mut g, v, &[]);
    assert_eq!(power(&g, bear), 1);
}

/// Boarded Window: attackers aimed at you get -1/-0; after you've taken 4
/// damage in a turn, the end step exiles it.
#[test]
fn boarded_window_weakens_attackers_then_breaks() {
    let mut g = pod(2);
    let w = g.add_card_to_battlefield(1, catalog::boarded_window());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    attack(&mut g, &[giant]);
    assert_eq!(power(&g, giant), 2);
    g.players[1].damage_taken_this_turn = 4;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == w));
}

// ── Four-gap closers (`decks::cmdr_01dj_u`) ────────────────────────────────

/// Bulette grows at your end step only if a creature died this turn.
#[test]
fn bulette_grows_after_a_death() {
    for (death, want) in [(false, 0), (true, 1)] {
        let mut g = pod(2);
        let bul = ready(&mut g, 0, catalog::bulette());
        if death {
            let bear = ready(&mut g, 1, catalog::grizzly_bears());
            let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
            cast_targets(&mut g, bolt, &[Target::Permanent(bear)]);
        }
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bul).unwrap().counter_count(CounterType::PlusOnePlusOne), want);
    }
}

/// Vivien's Invocation deploys a creature from the top seven, which then
/// deals its power to an opposing creature.
#[test]
fn viviens_invocation_deploys_and_bites() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let giant = g.add_card_to_library(0, catalog::hill_giant());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::viviens_invocation());
    cast_targets(&mut g, s, &[]);
    assert!(g.battlefield_find(giant).is_some(), "the Giant entered");
    assert!(g.battlefield_find(bear).is_none(), "and dealt 3 to the Bears");
}

/// Gloomwidow's Feast kills a flyer; a blue or black one also leaves you a
/// Spider.
#[test]
fn gloomwidows_feast_spiders_on_a_blue_flyer() {
    let mut g = pod(2);
    let crow = ready(&mut g, 1, catalog::storm_crow());
    let s = g.add_card_to_hand(0, catalog::gloomwidows_feast());
    cast_targets(&mut g, s, &[Target::Permanent(crow)]);
    assert!(g.battlefield_find(crow).is_none());
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Spider"));
}

/// Blizzard Brawl: with three snow permanents your creature gets +1/+0 and
/// indestructible before the fight, so the Bears beat a Hill Giant and live.
#[test]
fn blizzard_brawl_wins_with_snow() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::snow_covered_forest());
    }
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let s = g.add_card_to_hand(0, catalog::blizzard_brawl());
    cast_targets(&mut g, s, &[Target::Permanent(bear), Target::Permanent(giant)]);
    assert!(g.battlefield_find(giant).is_none(), "3 damage kills the 3/3");
    assert!(g.battlefield_find(bear).is_some(), "indestructible");
}

/// Energy (CR 107.16): Aether Theorist and Minister of Inquiries pay {E} for
/// their tap abilities; Electrostatic Pummeler's {E}{E}{E} doubles its power.
#[test]
fn energy_creatures_spend_their_energy() {
    let mut g = pod(2);
    let mi = g.add_card_to_hand(0, catalog::minister_of_inquiries());
    cast_targets(&mut g, mi, &[]);
    assert_eq!(g.players[0].energy, 2);
    g.clear_sickness(mi);
    let lib = g.players[1].library.len();
    activate(&mut g, mi, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].library.len(), lib - 3);
    assert_eq!(g.players[0].energy, 1);

    let ep = g.add_card_to_hand(0, catalog::electrostatic_pummeler());
    cast_targets(&mut g, ep, &[]);
    assert_eq!(g.players[0].energy, 4);
    activate(&mut g, ep, 0, None);
    assert_eq!(power(&g, ep), 2);
    assert_eq!(g.players[0].energy, 1);
}

// ── Pharaoh Rama-Tut (`decks::cmdr_01dj_v`) ────────────────────────────────

/// Umara Wizard gains flying when you cast an instant.
#[test]
fn umara_wizard_flies_on_an_instant() {
    let mut g = pod(2);
    let uw = ready(&mut g, 0, catalog::umara_wizard());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_targets(&mut g, bolt, &[Target::Player(1)]);
    assert!(g.computed_permanent(uw).unwrap().keywords().contains(&Keyword::Flying));
}

/// CR 702.35 — Welcome to the Fold hard-cast steals only toughness 2 or
/// less; discarded and cast for madness with X = 3, it takes a 3/3.
#[test]
fn welcome_to_the_fold_reaches_further_with_madness() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let s = g.add_card_to_hand(0, catalog::welcome_to_the_fold());
    cast_targets(&mut g, s, &[Target::Permanent(giant)]);
    assert_eq!(g.battlefield_find(giant).unwrap().controller, 1, "toughness 3 is out of reach");

    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    g.players[0].hand.clear();
    g.add_card_to_hand(0, catalog::welcome_to_the_fold());
    g.players[0].mana_pool.add(Color::Blue, 5);
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Discard {
                who: crabomination::effect::Selector::You,
                amount: crabomination::effect::Value::ONE,
                random: false,
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(giant).unwrap().controller, 0, "madness X = 3");
}
