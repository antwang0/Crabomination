//! Commander: the 2026-10-17 (`3chree`) fifth closer batch
//! (`decks::cmdr_oct17_w`).

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
    g.players[0].hostile_player_targets = true;
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
    for c in [
        Color::White,
        Color::Blue,
        Color::Black,
        Color::Red,
        Color::Green,
    ] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn step_to(g: &mut GameState, seat: usize, step: TurnStep) {
    for _ in 0..200 {
        if g.active_player_idx == seat && g.step == step {
            return;
        }
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
    panic!("never reached seat {seat} {step:?}");
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn attack_unblocked(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack {
                attacker,
                target: AttackTarget::Player(p),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

/// Keep Out's first mode deals 4 to a tapped creature.
#[test]
fn keep_out_burns_a_tapped_creature() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let ko = g.add_card_to_hand(0, catalog::keep_out());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: ko,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("keep out");
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(bear).is_none(),
        "4 damage kills the bear"
    );
}

/// Knockout Blow costs just {W} against a red attacker and gains 2 life.
#[test]
fn knockout_blow_discounted_on_red_attacker() {
    let mut g = pod(2);
    let piker = g.add_card_to_battlefield(1, catalog::goblin_piker());
    g.attacking.push(Attack {
        attacker: piker,
        target: AttackTarget::Player(0),
    });
    let kb = g.add_card_to_hand(0, catalog::knockout_blow());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, kb, Some(Target::Permanent(piker))).expect("{W} is enough");
    assert!(g.battlefield_find(piker).is_none());
    assert_eq!(g.players[0].life, 22);
}

/// Ghoulcaller's Chant's second mode returns two Zombie cards.
#[test]
fn ghoulcallers_chant_returns_two_zombies() {
    let mut g = pod(2);
    let z1 = g.add_card_to_graveyard(0, catalog::vulturous_zombie());
    let z2 = g.add_card_to_graveyard(0, catalog::soul_shackled_zombie());
    let gc = g.add_card_to_hand(0, catalog::ghoulcallers_chant());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: gc,
        target: Some(Target::Permanent(z1)),
        additional_targets: vec![Target::Permanent(z2)],
        mode: Some(1),
        x_value: None,
    })
    .expect("chant");
    drain_stack(&mut g);
    for z in [z1, z2] {
        assert!(g.players[0].hand.iter().any(|c| c.id == z), "zombie back");
    }
}

/// Voracious Wurm enters with a counter per life gained this turn.
#[test]
fn voracious_wurm_counts_life_gained() {
    let mut g = pod(2);
    g.players[0].life_gained_this_turn = 3;
    let vw = g.add_card_to_hand(0, catalog::voracious_wurm());
    flood(&mut g, 0);
    cast(&mut g, vw, None).expect("wurm");
    assert_eq!(pt(&g, vw), (5, 5));
}

/// Mouth of Ronom needs snow mana, then sacrifices itself for 4 damage.
#[test]
fn mouth_of_ronom_needs_snow() {
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::mouth_of_ronom());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    let act = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: m,
            ability_index: 1,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(act(&mut g).is_err(), "no snow mana");
    g.players[0].mana_pool.add_snow(Color::Green, 1);
    act(&mut g).expect("snow paid");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == m),
        "sacrificed"
    );
}

/// Oath of the Grey Host: Food each, then drain 3 + Treasure, then three
/// tapped flying Spirits.
#[test]
fn oath_of_the_grey_host_chapters() {
    let mut g = pod(2);
    let oath = g.add_card_to_hand(0, catalog::oath_of_the_grey_host());
    flood(&mut g, 0);
    cast(&mut g, oath, None).expect("oath");
    let count = |g: &GameState, seat: usize, name: &str| {
        g.battlefield
            .iter()
            .filter(|c| c.controller == seat && c.definition.name == name)
            .count()
    };
    assert_eq!(count(&g, 0, "Food"), 1);
    assert_eq!(count(&g, 1, "Food"), 1);
    step_to(&mut g, 1, TurnStep::Upkeep);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    assert_eq!(g.players[1].life, 17);
    assert_eq!(count(&g, 0, "Treasure"), 1);
    step_to(&mut g, 1, TurnStep::Upkeep);
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    let spirits: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 3);
    assert!(spirits.iter().all(|c| c.tapped), "enter tapped");
}

/// Sarkhan, Dragonsoul +2 pings each opponent and their creatures only.
#[test]
fn sarkhan_dragonsoul_plus_two_pings_opponents() {
    let mut g = pod(2);
    let pw = ready(&mut g, 0, catalog::sarkhan_dragonsoul());
    let piker = g.add_card_to_battlefield(1, catalog::goblin_piker());
    let mine = ready(&mut g, 0, catalog::goblin_piker());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("+2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(piker).is_none(), "their 2/1 died");
    assert!(g.battlefield_find(mine).is_some(), "mine untouched");
    assert_eq!(g.players[1].life, 19);
    assert_eq!(
        g.battlefield_find(pw)
            .unwrap()
            .counter_count(CounterType::Loyalty),
        7
    );
}


/// Ral, Izzet Viceroy −3 counts instants/sorceries in graveyard and exile.
#[test]
fn ral_izzet_viceroy_minus_three_counts_spells() {
    let mut g = pod(2);
    let pw = ready(&mut g, 0, catalog::ral_izzet_viceroy());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_exile(0, catalog::opt());
    g.add_card_to_exile(0, catalog::divination());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: 1,
        target: Some(Target::Permanent(angel)),
        x_value: None,
    })
    .expect("-3");
    drain_stack(&mut g);
    assert!(
        g.battlefield_find(angel).is_none(),
        "4 damage kills the 4/4"
    );
}

/// Molting Snakeskin gives +2/+0 and a {2}{B} regeneration ability.
#[test]
fn molting_snakeskin_pumps_and_regenerates() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ms = g.add_card_to_hand(0, catalog::molting_snakeskin());
    flood(&mut g, 0);
    cast(&mut g, ms, Some(Target::Permanent(bear))).expect("snakeskin");
    assert_eq!(pt(&g, bear), (4, 2));
    activate(&mut g, bear, None).expect("granted regenerate");
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g, 0);
    cast(&mut g, murder, Some(Target::Permanent(bear))).expect("murder");
    assert!(g.battlefield_find(bear).expect("regenerated").tapped);
}

/// Briarblade Adept's attack trigger shrinks an opposing creature.
#[test]
fn briarblade_adept_attack_shrinks() {
    let mut g = pod(2);
    let ba = ready(&mut g, 0, catalog::briarblade_adept());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    attack_unblocked(&mut g, vec![(ba, 1)]);
    assert_eq!(pt(&g, bear), (1, 1));
}

/// Shadowed Caravel grows when a creature you control explores.
#[test]
fn shadowed_caravel_grows_on_explore() {
    let mut g = pod(2);
    let sc = ready(&mut g, 0, catalog::shadowed_caravel());
    let id = g.next_id();
    g.players[0].add_to_library_top(id, catalog::lightning_bolt());
    let dfi = g.add_card_to_hand(0, catalog::dire_fleet_interloper());
    flood(&mut g, 0);
    cast(&mut g, dfi, None).expect("interloper");
    assert_eq!(
        g.battlefield_find(sc)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Bounty of Might's three separate clauses can all hit the same creature.
#[test]
fn bounty_of_might_stacks_on_one_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bom = g.add_card_to_hand(0, catalog::bounty_of_might());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bom,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(bear), Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("bounty");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (11, 11));
}

/// Scout the Wilderness kicked: a tapped basic plus two Soldiers.
#[test]
fn scout_the_wilderness_kicked_makes_soldiers() {
    let mut g = pod(2);
    let stw = g.add_card_to_hand(0, catalog::scout_the_wilderness());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: stw,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    assert!(
        g.battlefield
            .iter()
            .any(|c| c.controller == 0 && c.definition.name == "Forest" && c.tapped),
        "basic land fetched tapped"
    );
    assert_eq!(
        g.battlefield
            .iter()
            .filter(|c| c.is_token && c.definition.name == "Soldier")
            .count(),
        2
    );
}

/// Shipwreck Looter loots on entry only after you attacked this turn.
#[test]
fn shipwreck_looter_raid_loots() {
    let mut g = pod(2);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let sl = g.add_card_to_hand(0, catalog::shipwreck_looter());
    g.players[0].attacked_this_turn = true;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    cast(&mut g, sl, None).expect("looter");
    let _ = bolt;
    assert_eq!(g.players[0].hand.len(), 1, "drew one, discarded one");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Ambuscade Shaman dashed: haste, and its own entry pumps it to 4/4.
#[test]
fn ambuscade_shaman_dash_pumps_itself() {
    let mut g = pod(2);
    let ash = g.add_card_to_hand(0, catalog::ambuscade_shaman());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: ash,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("dash");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(ash, &Keyword::Haste));
    assert_eq!(pt(&g, ash), (4, 4));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None).expect("bear");
    assert_eq!(
        pt(&g, bear),
        (4, 4),
        "another creature entering is pumped too"
    );
}

/// Snake Cult Initiation: enchanted creature's combat damage gives 3 poison.
#[test]
fn snake_cult_initiation_grants_poisonous_three() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sci = g.add_card_to_hand(0, catalog::snake_cult_initiation());
    flood(&mut g, 0);
    cast(&mut g, sci, Some(Target::Permanent(bear))).expect("aura");
    assert!(g.permanent_has_keyword(bear, &Keyword::Poisonous(3)));
    attack_unblocked(&mut g, vec![(bear, 1)]);
    assert_eq!(g.players[1].poison_counters, 3);
    assert_eq!(g.players[1].life, 18);
}

/// Drowner Initiate: a blue spell lets you pay {1} to mill a player two.
#[test]
fn drowner_initiate_mills_on_blue_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::drowner_initiate());
    let opt = g.add_card_to_hand(0, catalog::opt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    cast(&mut g, opt, None).expect("opt");
    // hostile_player_targets: the trigger's player target is the opponent.
    assert_eq!(g.players[1].graveyard.len(), 2);
}

/// Dragon's Eye Savants unmorphs by revealing a blue card and looks at the
/// opponent's hand.
#[test]
fn dragons_eye_savants_unmorph_looks_at_hand() {
    let mut g = pod(2);
    let des = g.add_card_to_hand(0, catalog::dragons_eye_savants());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFaceDown { card_id: des })
        .expect("face down");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::TurnFaceUp { card_id: des })
            .is_err(),
        "no blue card to reveal"
    );
    g.add_card_to_hand(0, catalog::opt());
    g.perform_action(GameAction::TurnFaceUp { card_id: des })
        .expect("reveal Opt");
    drain_stack(&mut g);
    assert_eq!(pt(&g, des), (0, 6));
    assert!(g.hands_revealed_to.contains(&(0, 1)));
}

/// Goblin Morningstar on a 10+ makes a Goblin wearing it (2/1 trample).
#[test]
fn goblin_morningstar_high_roll_equips_token() {
    let mut g = pod(2);
    let gm = g.add_card_to_hand(0, catalog::goblin_morningstar());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(15)]));
    flood(&mut g, 0);
    cast(&mut g, gm, None).expect("morningstar");
    let tok = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Goblin")
        .map(|c| c.id)
        .expect("goblin token");
    assert_eq!(g.battlefield_find(gm).unwrap().attached_to, Some(tok));
    assert_eq!(pt(&g, tok), (2, 1));
    assert!(g.permanent_has_keyword(tok, &Keyword::Trample));
}

/// Pompous Battlemage enters prepared; Improvised Act rummages and its cast
/// triggers prowess.
#[test]
fn pompous_battlemage_improvised_act_rummages() {
    let mut g = pod(2);
    let pb = g.add_card_to_hand(0, catalog::pompous_battlemage());
    flood(&mut g, 0);
    cast(&mut g, pb, None).expect("battlemage");
    assert_eq!(
        g.battlefield_find(pb)
            .unwrap()
            .counter_count(CounterType::Prepared),
        1
    );
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: pb,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("improvised act");
    drain_stack(&mut g);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "discarded"
    );
    assert!(
        g.players[0]
            .hand
            .iter()
            .any(|c| c.definition.name == "Forest"),
        "drew"
    );
    assert_eq!(pt(&g, pb), (2, 2), "prowess");
    assert_eq!(
        g.battlefield_find(pb)
            .unwrap()
            .counter_count(CounterType::Prepared),
        0
    );
}

/// Isareth pays X=2 on attack to reanimate a 2-drop with a corpse counter; it
/// is exiled when it later dies.
#[test]
fn isareth_reanimates_with_corpse_counter() {
    let mut g = pod(2);
    let isa = ready(&mut g, 0, catalog::isareth_the_awakener());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    flood(&mut g, 0);
    attack_unblocked(&mut g, vec![(isa, 1)]);
    let c = g.battlefield_find(bear).expect("bear returned");
    assert_eq!(c.counter_count(CounterType::Corpse), 1);
    g.priority.player_with_priority = 0;
    flood(&mut g, 0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt");
    assert!(
        g.exile.iter().any(|c| c.id == bear),
        "exiled instead of dying"
    );
}
