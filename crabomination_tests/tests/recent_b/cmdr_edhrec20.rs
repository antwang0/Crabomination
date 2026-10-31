//! Commander: Maha, General Ferrous Rokiric and Greasefang's last gaps
//! (`decks::cmdr_edhrec20`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
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

fn cast_mode(g: &mut GameState, id: CardId, targets: &[Target], mode: Option<usize>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_mode(g, id, targets, None)
}

fn activate(g: &mut GameState, id: CardId, index: usize) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
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

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).expect("on the battlefield");
    (cp.power, cp.toughness)
}

/// Maha's -1/-1 package: each shrinks only the opponents' creatures until end
/// of turn (Shrivel hits everything), and M.O.D.O.K.'s is a static.
#[test]
fn maha_shrinkers_hit_the_opponents() {
    for (name, def, all) in [
        ("Make Obsolete", catalog::make_obsolete as fn() -> _, false),
        ("Shrivel", catalog::shrivel, true),
        ("Night Clubber", catalog::night_clubber, false),
        ("Plague Mare", catalog::plague_mare, false),
    ] {
        let mut g = pod(3);
        let mine = ready(&mut g, 0, catalog::hill_giant());
        let theirs = ready(&mut g, 1, catalog::hill_giant());
        let other = ready(&mut g, 2, catalog::hill_giant());
        let id = g.add_card_to_hand(0, def());
        cast(&mut g, id, &[]).expect(name);
        assert_eq!(pt(&g, theirs), (2, 2), "{name}");
        assert_eq!(pt(&g, other), (2, 2), "{name}: each opponent's");
        assert_eq!(pt(&g, mine), if all { (2, 2) } else { (3, 3) }, "{name}");
    }
    let mut g = pod(2);
    let theirs = ready(&mut g, 1, catalog::hill_giant());
    ready(&mut g, 0, catalog::m_o_d_o_k());
    assert_eq!(pt(&g, theirs), (2, 2));
}

/// M.O.D.O.K.'s connive costs 3 life and only works on your turn.
#[test]
fn modok_connives_for_life_on_your_turn() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let modok = ready(&mut g, 0, catalog::m_o_d_o_k());
    activate(&mut g, modok, 0).expect("connive");
    assert_eq!(g.players[0].life, 17);
    g.active_player_idx = 1;
    assert!(activate(&mut g, modok, 0).is_err(), "your turn only");
}

/// Gruesome Realization's first mode draws two for 2 life.
#[test]
fn gruesome_realization_draw_mode() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::gruesome_realization());
    cast_mode(&mut g, id, &[], Some(0)).expect("cast");
    assert_eq!((g.players[0].hand.len(), g.players[0].life), (2, 18));
}

/// Figure of Destiny climbs: each step needs the one before it.
#[test]
fn figure_of_destiny_levels_in_order() {
    let mut g = pod(2);
    let fig = ready(&mut g, 0, catalog::figure_of_destiny());
    activate(&mut g, fig, 2).expect("eight");
    assert_eq!(pt(&g, fig), (1, 1), "not a Warrior yet");
    activate(&mut g, fig, 0).expect("spirit");
    activate(&mut g, fig, 1).expect("warrior");
    assert_eq!(pt(&g, fig), (4, 4));
    activate(&mut g, fig, 2).expect("avatar");
    assert_eq!(pt(&g, fig), (8, 8));
    let cp = g.computed_permanent(fig).unwrap();
    let kw = cp.keywords();
    assert!(kw.contains(&Keyword::Flying) && kw.contains(&Keyword::FirstStrike));
}

/// Rem Karolus: a spell's damage to its controller's side is prevented, and
/// to an opponent's side is one more; combat damage is untouched.
#[test]
fn rem_karolus_bends_spell_damage() {
    let mut g = pod(2);
    let rem = ready(&mut g, 0, catalog::rem_karolus_stalwart_slayer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]).expect("bolt them");
    assert_eq!(g.players[1].life, 16, "3 plus 1");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(0)]).expect("bolt me");
    assert_eq!(g.players[0].life, 20, "prevented");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(rem)]).expect("bolt Rem");
    g.check_state_based_actions();
    assert!(g.battlefield_find(rem).is_none(), "Rem doesn't shield itself");
}

/// Intimidation Bolt: 3 to the target, and nothing else may attack.
#[test]
fn intimidation_bolt_grounds_the_rest() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::intimidation_bolt());
    cast(&mut g, bolt, &[Target::Permanent(giant)]).expect("cast");
    g.check_state_based_actions();
    assert!(g.battlefield_find(giant).is_none());
    g.step = TurnStep::DeclareAttackers;
    let r = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]));
    assert!(r.is_err(), "the bear can't attack this turn");
}

/// Intimidation Bolt's "other creatures" spares its own target: a survivor
/// may still attack (bug fix — the board selector read the target slot).
#[test]
fn intimidation_bolt_lets_a_surviving_target_attack() {
    let mut g = pod(2);
    let wall = ready(&mut g, 0, catalog::webweaver_changeling());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::intimidation_bolt());
    cast(&mut g, bolt, &[Target::Permanent(wall)]).expect("cast");
    g.step = TurnStep::DeclareAttackers;
    let attack = |a| vec![Attack { attacker: a, target: AttackTarget::Player(1) }];
    assert!(g.clone().perform_action(GameAction::DeclareAttackers(attack(bear))).is_err());
    g.perform_action(GameAction::DeclareAttackers(attack(wall))).expect("the target may attack");
}

/// "Other creatures can't attack this turn" is a rules effect, not a change to
/// characteristics, so it binds a creature that arrives after the spell resolved
/// (CR 611.2c covers only characteristics and control). Intimidation Bolt and the
/// whole board-wide `CantAttackThisTurn` class (Festival, kicked Orim's Chant).
#[test]
fn intimidation_bolt_binds_a_creature_that_enters_later() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let bolt = g.add_card_to_hand(0, catalog::intimidation_bolt());
    cast(&mut g, bolt, &[Target::Permanent(giant)]).expect("cast");
    let late = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    let r = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: late,
        target: AttackTarget::Player(1),
    }]));
    assert!(r.is_err(), "a creature entering after the Bolt can't attack either");
}

/// Justice Strike: the creature hits itself for its power.
#[test]
fn justice_strike_self_damage() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let id = g.add_card_to_hand(0, catalog::justice_strike());
    cast(&mut g, id, &[Target::Permanent(giant)]).expect("cast");
    g.check_state_based_actions();
    assert!(g.battlefield_find(giant).is_none());
}

/// Deafening Clarion's two modes together: 3 to each creature, then lifelink
/// for what's left of yours.
#[test]
fn deafening_clarion_both_modes() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let wall = ready(&mut g, 0, catalog::giant_ox());
    let id = g.add_card_to_hand(0, catalog::deafening_clarion());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![0, 1])]));
    cast(&mut g, id, &[]).expect("cast");
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(wall).is_some());
}

/// Crewed by exactly two creatures (Hotshot Mechanic counts as power 4), the
/// Mighty Servant draws two on its combat damage; crewed by one (Giant Ox, by
/// toughness) it doesn't (CR 702.122).
#[test]
fn mighty_servant_crewed_by_two_draws() {
    for two in [true, false] {
        let mut g = pod(2);
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        let servant = ready(&mut g, 0, catalog::mighty_servant_of_leuk_o());
        let crew = if two {
            vec![ready(&mut g, 0, catalog::hotshot_mechanic()), ready(&mut g, 0, catalog::grizzly_bears())]
        } else {
            vec![ready(&mut g, 0, catalog::giant_ox())]
        };
        g.perform_action(GameAction::Crew { vehicle: servant, crew_creatures: crew }).expect("crew");
        drain_stack(&mut g);
        g.clear_sickness(servant);
        let life = g.players[1].life;
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: servant,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        g.step = TurnStep::CombatDamage;
        let ev = g.resolve_combat().expect("damage");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 6);
        assert_eq!(g.players[0].hand.len(), if two { 2 } else { 0 }, "two crew: {two}");
    }
}

/// "Becomes crewed for the first time each turn, if it was crewed by exactly
/// two creatures": each crewing is counted on its own, and only the first one
/// each turn counts — one then one (two crewers this turn, never two at once),
/// or one then two, is no draw grant.
#[test]
fn mighty_servant_reads_only_the_first_crew_each_turn() {
    for second_pair in [false, true] {
        let mut g = pod(2);
        let servant = ready(&mut g, 0, catalog::mighty_servant_of_leuk_o());
        let ox = ready(&mut g, 0, catalog::giant_ox());
        g.perform_action(GameAction::Crew { vehicle: servant, crew_creatures: vec![ox] }).expect("crew by one");
        drain_stack(&mut g);
        let mut again = vec![ready(&mut g, 0, catalog::hotshot_mechanic())];
        if second_pair {
            again.push(ready(&mut g, 0, catalog::grizzly_bears()));
        }
        g.perform_action(GameAction::Crew { vehicle: servant, crew_creatures: again }).expect("crew again");
        drain_stack(&mut g);
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        g.clear_sickness(servant);
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: servant, target: AttackTarget::Player(1) }]))
            .expect("attack");
        drain_stack(&mut g);
        g.step = TurnStep::CombatDamage;
        let ev = g.resolve_combat().expect("damage");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 0, "second crew by a pair: {second_pair}");
    }
}

/// Giant Ox and Hotshot Mechanic only crew (CR 702.122b): neither saddles
/// Caustic Bronco's saddle 3 by toughness or with the +2 (CR 702.171a).
#[test]
fn giant_ox_and_hotshot_mechanic_dont_saddle() {
    for helper in [catalog::giant_ox as fn() -> _, catalog::hotshot_mechanic] {
        let mut g = pod(2);
        let bronco = ready(&mut g, 0, catalog::caustic_bronco());
        let rider = ready(&mut g, 0, helper());
        assert!(g.perform_action(GameAction::Saddle { mount: bronco, creatures: vec![rider] }).is_err());
        assert!(!g.battlefield_find(bronco).unwrap().saddled);
    }
}

/// Kitsune Ace: a Vehicle attacking gives it first strike (mode 0).
#[test]
fn kitsune_ace_first_strike_for_the_vehicle() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::kitsune_ace());
    let servant = ready(&mut g, 0, catalog::mighty_servant_of_leuk_o());
    let ox = ready(&mut g, 0, catalog::giant_ox());
    g.perform_action(GameAction::Crew { vehicle: servant, crew_creatures: vec![ox] }).expect("crew");
    drain_stack(&mut g);
    g.clear_sickness(servant);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: servant,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(servant).unwrap().keywords().contains(&Keyword::FirstStrike));
}

/// Born to Drive counts creatures and Vehicles you control; its channel makes
/// two Pilots.
#[test]
fn born_to_drive_scales_and_channels() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::mighty_servant_of_leuk_o());
    let aura = g.add_card_to_hand(0, catalog::born_to_drive());
    cast(&mut g, aura, &[Target::Permanent(bear)]).expect("cast");
    assert_eq!(pt(&g, bear), (4, 4), "the bear and the Vehicle");
    let other = g.add_card_to_hand(0, catalog::born_to_drive());
    activate(&mut g, other, 0).expect("channel");
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Pilot").count(), 2);
}

/// Okiba Reckoner Raid drains each opponent, then flips into Nezumi Road
/// Captain, whose Vehicles have menace.
#[test]
fn okiba_reckoner_raid_drains_then_flips() {
    let mut g = pod(3);
    let saga = g.add_card_to_hand(0, catalog::okiba_reckoner_raid());
    cast(&mut g, saga, &[]).expect("cast");
    assert_eq!((g.players[0].life, g.players[1].life, g.players[2].life), (21, 19, 19));
    for _ in 0..2 {
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let evs = g
            .resolve_effect(
                &Effect::AddCounter {
                    what: Selector::ExactObjects(vec![saga]),
                    kind: crabomination::card::CounterType::Lore,
                    amount: crabomination::card::Value::ONE,
                },
                &ctx,
            )
            .unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    let captain = g.battlefield.iter().find(|c| c.definition.name == "Nezumi Road Captain").map(|c| c.id);
    assert!(captain.is_some(), "returned transformed");
}
