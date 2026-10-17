//! Commander: the 2026-10-17 (`3chree`) fifth closer batch
//! (`decks::cmdr_oct17_v`).

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

/// Dire Fleet Interloper has menace and explores a nonland into a counter.
#[test]
fn dire_fleet_interloper_explores() {
    let mut g = pod(2);
    let id = g.next_id();
    g.players[0].add_to_library_top(id, catalog::lightning_bolt());
    let dfi = g.add_card_to_hand(0, catalog::dire_fleet_interloper());
    flood(&mut g, 0);
    cast(&mut g, dfi, None).expect("interloper");
    assert!(g.permanent_has_keyword(dfi, &Keyword::Menace));
    assert_eq!(
        g.battlefield_find(dfi)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        1,
        "nonland revealed: +1/+1 counter"
    );
}

/// Jungle Wayfinder lets every player fetch a basic land to hand.
#[test]
fn jungle_wayfinder_each_player_fetches_a_basic() {
    let mut g = pod(2);
    let jw = g.add_card_to_hand(0, catalog::jungle_wayfinder());
    flood(&mut g, 0);
    cast(&mut g, jw, None).expect("wayfinder");
    for seat in 0..2 {
        assert!(
            g.players[seat]
                .hand
                .iter()
                .any(|c| c.definition.name == "Forest"),
            "seat {seat} fetched a Forest"
        );
    }
}

/// Littjara Glade-Warden exiles a creature card from the graveyard to put
/// two +1/+1 counters on a creature.
#[test]
fn littjara_glade_warden_exiles_for_two_counters() {
    let mut g = pod(2);
    let lw = ready(&mut g, 0, catalog::littjara_glade_warden());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    assert!(g.permanent_has_keyword(lw, &Keyword::Changeling));
    flood(&mut g, 0);
    activate(&mut g, lw, Some(Target::Permanent(bear))).expect("activate");
    assert_eq!(
        g.battlefield_find(bear)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        2
    );
    assert!(
        g.exile.iter().any(|c| c.id == dead),
        "creature card exiled as cost"
    );
}

/// Warehouse Thief sacrifices itself to impulse the top card until the end
/// of your next turn.
#[test]
fn warehouse_thief_sacrifices_to_impulse() {
    let mut g = pod(2);
    let id = g.next_id();
    g.players[0].add_to_library_top(id, catalog::lightning_bolt());
    let wt = ready(&mut g, 0, catalog::warehouse_thief());
    flood(&mut g, 0);
    activate(&mut g, wt, None).expect("activate");
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == wt),
        "sacrificed itself"
    );
    assert!(
        g.exile
            .iter()
            .any(|c| c.id == id && c.may_play_until.is_some()),
        "top card exiled and playable"
    );
}

/// Sarkhan Vol +1 pumps and hastes your team; −2 steals and untaps a creature.
#[test]
fn sarkhan_vol_pumps_then_threatens() {
    let mut g = pod(2);
    let pw = ready(&mut g, 0, catalog::sarkhan_vol());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("+1");
    drain_stack(&mut g);
    assert_eq!(pt(&g, mine), (3, 3));
    assert!(g.permanent_has_keyword(mine, &Keyword::Haste));
    g.battlefield_find_mut(pw).unwrap().loyalty_uses_this_turn = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: 1,
        target: Some(Target::Permanent(theirs)),
        x_value: None,
    })
    .expect("-2");
    drain_stack(&mut g);
    let c = g.battlefield_find(theirs).unwrap();
    assert_eq!(c.controller, 0);
    assert!(!c.tapped);
    assert!(g.permanent_has_keyword(theirs, &Keyword::Haste));
    assert_eq!(
        g.battlefield_find(pw)
            .unwrap()
            .counter_count(CounterType::Loyalty),
        3
    );
}

/// Mine Worker gains 1 alone, 3 with Power Plant Worker and Tower Worker.
#[test]
fn mine_worker_gains_three_with_the_set() {
    let mut g = pod(2);
    let mw = ready(&mut g, 0, catalog::mine_worker());
    activate(&mut g, mw, None).expect("tap");
    assert_eq!(g.players[0].life, 21);
    let named = |n: &'static str| crabomination::card::CardDefinition {
        name: n,
        card_types: vec![crabomination::card::CardType::Creature],
        power: 2,
        toughness: 2,
        ..Default::default()
    };
    g.add_card_to_battlefield(0, named("Power Plant Worker"));
    g.add_card_to_battlefield(0, named("Tower Worker"));
    g.battlefield_find_mut(mw).unwrap().tapped = false;
    activate(&mut g, mw, None).expect("tap again");
    assert_eq!(g.players[0].life, 24);
}

/// Rimewind Taskmage needs four snow permanents; then it taps a permanent.
#[test]
fn rimewind_taskmage_requires_four_snow() {
    let mut g = pod(2);
    let rt = ready(&mut g, 0, catalog::rimewind_taskmage());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::snow_covered_forest());
    }
    flood(&mut g, 0);
    assert!(
        activate(&mut g, rt, Some(Target::Permanent(bear))).is_err(),
        "only three snow"
    );
    g.add_card_to_battlefield(0, catalog::snow_covered_forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, rt, Some(Target::Permanent(bear))).expect("four snow");
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Coral Trickster, unmorphed for {U}, taps a permanent.
#[test]
fn coral_trickster_taps_when_turned_up() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ct = g.add_card_to_hand(0, catalog::coral_trickster());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFaceDown { card_id: ct })
        .expect("face down");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ct).is_some_and(|c| c.face_down));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::TurnFaceUp { card_id: ct })
        .expect("face up");
    drain_stack(&mut g);
    assert_eq!(pt(&g, ct), (2, 1));
    assert!(
        g.battlefield_find(bear).unwrap().tapped,
        "opposing bear tapped"
    );
}

/// Take Possession (split second) steals a noncreature permanent.
#[test]
fn take_possession_steals_any_permanent() {
    let mut g = pod(2);
    assert!(
        catalog::take_possession()
            .keywords
            .contains(&Keyword::SplitSecond)
    );
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let tp = g.add_card_to_hand(0, catalog::take_possession());
    flood(&mut g, 0);
    cast(&mut g, tp, Some(Target::Permanent(land))).expect("take possession");
    assert_eq!(g.battlefield_find(land).unwrap().controller, 0);
}

/// Second Wind taps and untaps the enchanted creature via its own {T}.
#[test]
fn second_wind_taps_and_untaps_host() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sw = g.add_card_to_hand(0, catalog::second_wind());
    flood(&mut g, 0);
    cast(&mut g, sw, Some(Target::Permanent(bear))).expect("aura");
    activate(&mut g, sw, None).expect("tap host");
    assert!(g.battlefield_find(bear).unwrap().tapped);
    assert!(g.battlefield_find(sw).unwrap().tapped);
    g.battlefield_find_mut(sw).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sw,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("untap host");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped);
}

/// Shield of the Righteous: +0/+2, vigilance; the creature it blocks is stunned.
#[test]
fn shield_of_the_righteous_freezes_blocked_attacker() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let shield = g.add_card_to_battlefield(0, catalog::shield_of_the_righteous());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip {
        equipment: shield,
        target: bear,
    })
    .expect("equip {2}");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (2, 4));
    assert!(g.permanent_has_keyword(bear, &Keyword::Vigilance));
    let giant = ready(&mut g, 1, catalog::hill_giant());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: giant,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, giant)]))
        .expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).unwrap().skip_next_untap);
}

/// Temmet pumps a creature token and makes it unblockable at beginning of combat.
#[test]
fn temmet_boosts_a_token_at_combat() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::temmet_vizier_of_naktamun());
    let tok = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(tok).unwrap().is_token = true;
    step_to(&mut g, 0, TurnStep::BeginCombat);
    assert_eq!(pt(&g, tok), (3, 3));
    assert!(g.permanent_has_keyword(tok, &Keyword::Unblockable));
}

// Extra imports: none beyond the cmdr_oct17_u.rs test header.

/// Hypnotic Grifter's {3} connives: a drawn nonland discard grows it.
#[test]
fn hypnotic_grifter_connives_for_three() {
    let mut g = pod(2);
    let gr = ready(&mut g, 0, catalog::hypnotic_grifter());
    let id = g.next_id();
    g.players[0].add_to_library_top(id, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, gr, None).expect("{3}: connive");
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == id),
        "drew and discarded the Bolt"
    );
    assert_eq!(
        g.battlefield_find(gr)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        1
    );
    assert_eq!(pt(&g, gr), (2, 3));
}

/// Calciderm enters with four time counters and can't be targeted.
#[test]
fn calciderm_vanishing_four_and_shroud() {
    let mut g = pod(2);
    let cd = g.add_card_to_hand(0, catalog::calciderm());
    flood(&mut g, 0);
    cast(&mut g, cd, None).expect("calciderm");
    assert_eq!(
        g.battlefield_find(cd)
            .unwrap()
            .counter_count(CounterType::Time),
        4
    );
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(
        cast(&mut g, bolt, Some(Target::Permanent(cd))).is_err(),
        "shroud"
    );
}

/// Maximize Velocity pumps and hastes, then jump-starts from the graveyard.
#[test]
fn maximize_velocity_hastes_and_jump_starts() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mv = g.add_card_to_hand(0, catalog::maximize_velocity());
    flood(&mut g, 0);
    cast(&mut g, mv, Some(Target::Permanent(bear))).expect("cast");
    assert_eq!(pt(&g, bear), (3, 3));
    assert!(g.permanent_has_keyword(bear, &Keyword::Haste));
    g.add_card_to_hand(0, catalog::island());
    g.perform_action(GameAction::CastFlashback {
        card_id: mv,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("jump-start");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (4, 4));
    assert!(
        g.exile.iter().any(|c| c.id == mv),
        "exiled after jump-start"
    );
}

/// Mu Yanling: +2 makes a creature unblockable; −10 taps their team and
/// grants an extra turn.
#[test]
fn mu_yanling_unblockable_and_ultimate() {
    let mut g = pod(2);
    let pw = ready(&mut g, 0, catalog::mu_yanling());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::hill_giant());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        x_value: None,
    })
    .expect("+2");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(bear, &Keyword::Unblockable));
    g.battlefield_find_mut(pw)
        .unwrap()
        .add_counters(CounterType::Loyalty, 3);
    g.battlefield_find_mut(pw).unwrap().loyalty_uses_this_turn = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: 2,
        target: None,
        x_value: None,
    })
    .expect("-10");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
    assert!(
        !g.battlefield_find(bear).unwrap().tapped,
        "only opponents' creatures"
    );
    assert_eq!(g.players[0].extra_turns, 1);
}

/// Kayla's Command, counter + life modes: the chosen creature gets a counter
/// and double strike; you gain 2.
#[test]
fn kaylas_command_counter_double_strike_and_life() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let kc = g.add_card_to_hand(0, catalog::kaylas_command());
    flood(&mut g, 0);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpellSpree {
        card_id: kc,
        spree_modes: vec![1, 3],
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear)
            .unwrap()
            .counter_count(CounterType::PlusOnePlusOne),
        1
    );
    assert!(g.permanent_has_keyword(bear, &Keyword::DoubleStrike));
    assert_eq!(g.players[0].life, life + 2);
    assert!(
        g.battlefield
            .iter()
            .all(|c| c.definition.name != "Construct"),
        "token mode not chosen"
    );
}

/// Gideon, Martial Paragon: +2 untaps and pumps your team; 0 makes him a 5/5
/// indestructible creature.
#[test]
fn gideon_martial_paragon_untaps_and_animates() {
    let mut g = pod(2);
    let pw = ready(&mut g, 0, catalog::gideon_martial_paragon());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("+2");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(pt(&g, bear), (3, 3));
    g.battlefield_find_mut(pw).unwrap().loyalty_uses_this_turn = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: 1,
        target: None,
        x_value: None,
    })
    .expect("0");
    drain_stack(&mut g);
    assert_eq!(pt(&g, pw), (5, 5));
    assert!(g.permanent_has_keyword(pw, &Keyword::Indestructible));
}

/// Terror Ballista's attack sacrifices another creature to destroy an
/// opponent's creature.
#[test]
fn terror_ballista_sacrifices_to_destroy() {
    let mut g = pod(2);
    let tb = ready(&mut g, 0, catalog::terror_ballista());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack_unblocked(&mut g, vec![(tb, 1)]);
    assert!(g.battlefield_find(bear).is_none(), "bear sacrificed");
    assert!(g.battlefield_find(giant).is_none(), "giant destroyed");
    assert_eq!(g.players[1].life, 15);
}

/// Ipnu Rivulet sacrifices itself (a Desert) to mill target player four.
#[test]
fn ipnu_rivulet_sacrifices_a_desert_to_mill_four() {
    let mut g = pod(2);
    let ir = ready(&mut g, 0, catalog::ipnu_rivulet());
    flood(&mut g, 0);
    let before = g.players[1].library.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ir,
        ability_index: 2,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("mill four");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), before - 4);
    assert_eq!(g.players[1].graveyard.len(), 4);
    assert!(
        g.battlefield_find(ir).is_none(),
        "sacrificed itself as the Desert"
    );
}

/// Moonring Island's peek needs two blue permanents.
#[test]
fn moonring_island_peek_needs_two_blue_permanents() {
    let mut g = pod(2);
    let mi = ready(&mut g, 0, catalog::moonring_island());
    let peek = |g: &mut GameState| {
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: mi,
            ability_index: 1,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(peek(&mut g).is_err(), "no blue permanents");
    ready(&mut g, 0, catalog::storm_crow());
    ready(&mut g, 0, catalog::storm_crow());
    peek(&mut g).expect("two blue permanents");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mi).unwrap().tapped);
}

/// Appeal pumps by your creature count; Authority (aftermath) taps two of
/// theirs and gives your team vigilance.
#[test]
fn appeal_then_authority() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::grizzly_bears());
    let t1 = ready(&mut g, 1, catalog::hill_giant());
    let t2 = ready(&mut g, 1, catalog::grizzly_bears());
    let aa = g.add_card_to_hand(0, catalog::appeal_authority());
    flood(&mut g, 0);
    cast(&mut g, aa, Some(Target::Permanent(bear))).expect("Appeal");
    assert_eq!(pt(&g, bear), (4, 4), "+2/+2 for two creatures");
    assert!(g.permanent_has_keyword(bear, &Keyword::Trample));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastAftermath {
        card_id: aa,
        target: Some(Target::Permanent(t1)),
        additional_targets: vec![Target::Permanent(t2)],
        mode: None,
        x_value: None,
    })
    .expect("Authority");
    drain_stack(&mut g);
    assert!(g.battlefield_find(t1).unwrap().tapped);
    assert!(g.battlefield_find(t2).unwrap().tapped);
    assert!(g.permanent_has_keyword(bear, &Keyword::Vigilance));
    assert!(g.exile.iter().any(|c| c.id == aa));
}

/// Dig Up the Body with casualty: the copy and the original each mill two and
/// return a creature card.
#[test]
fn dig_up_the_body_casualty_copies() {
    let mut g = pod(2);
    let old = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let fodder = ready(&mut g, 0, catalog::grizzly_bears());
    let d = g.add_card_to_hand(0, catalog::dig_up_the_body());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Cards(vec![old]),
        DecisionAnswer::Cards(vec![fodder]),
    ]));
    g.perform_action(GameAction::CastSpellCasualty {
        card_id: d,
        sacrifice: fodder,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("casualty 1");
    drain_stack(&mut g);
    let in_hand = |id| g.players[0].hand.iter().any(|c| c.id == id);
    assert!(in_hand(old) && in_hand(fodder), "both bears returned");
    let forests = g.players[0]
        .graveyard
        .iter()
        .filter(|c| c.definition.name == "Forest")
        .count();
    assert_eq!(forests, 4, "milled two twice");
}

/// Temur Charger unmorphs only by revealing a green card, then grants trample.
#[test]
fn temur_charger_reveals_green_to_unmorph() {
    let mut g = pod(2);
    let tc = g.add_card_to_hand(0, catalog::temur_charger());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFaceDown { card_id: tc })
        .expect("face down");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::TurnFaceUp { card_id: tc })
            .is_err(),
        "no green card"
    );
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.perform_action(GameAction::TurnFaceUp { card_id: tc })
        .expect("reveal Bears");
    drain_stack(&mut g);
    assert_eq!(pt(&g, tc), (3, 1));
    assert!(g.permanent_has_keyword(tc, &Keyword::Trample));
}
