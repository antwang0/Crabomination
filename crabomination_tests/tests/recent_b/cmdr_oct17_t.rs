//! Commander: the 2026-10-17 (`3chree`) fourth closer batch
//! (`decks::cmdr_oct17_t`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::catalog;
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
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
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
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target, additional_targets: vec![], x_value: None, mode: None })?;
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
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

/// Wayward Guide-Beast connecting bounces your only land to your hand.
#[test]
fn wayward_guide_beast_bounces_a_land_on_hit() {
    let mut g = pod(2);
    let beast = ready(&mut g, 0, catalog::wayward_guide_beast());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let opp = g.players[1].life;
    attack_unblocked(&mut g, vec![(beast, 1)]);
    assert_eq!(g.players[1].life, opp - 2);
    assert!(g.battlefield_find(forest).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "returned to its owner's hand");
}

/// Riftsweeper shuffles an opponent's exiled card into that opponent's library.
#[test]
fn riftsweeper_shuffles_exiled_card_into_owners_library() {
    let mut g = pod(2);
    let exiled = g.add_card_to_exile(1, catalog::grizzly_bears());
    let rs = g.add_card_to_hand(0, catalog::riftsweeper());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(exiled))]));
    cast(&mut g, rs, None).expect("riftsweeper");
    assert!(!g.exile.iter().any(|c| c.id == exiled));
    assert!(g.players[1].library.iter().any(|c| c.id == exiled), "owner's library");
}

/// Icy Reception's counter mode hits a legendary noncreature spell (Jace) but
/// can't target a plain sorcery.
#[test]
fn icy_reception_counters_legendary_spells_only() {
    let opp_casts = |def: crabomination::card::CardDefinition| {
        let mut g = pod(2);
        let s = g.add_card_to_hand(1, def);
        g.players[1].mana_pool.add(Color::Blue, 2);
        g.players[1].mana_pool.add_colorless(1);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell { card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("opponent's spell");
        g.priority.player_with_priority = 0;
        let ir = g.add_card_to_hand(0, catalog::icy_reception());
        flood(&mut g, 0);
        let r = g.perform_action(GameAction::CastSpell {
            card_id: ir,
            target: Some(Target::Permanent(s)),
            additional_targets: vec![],
            mode: Some(0),
            x_value: None,
        });
        drain_stack(&mut g);
        (g, s, r.is_ok())
    };
    let (_, _, ok) = opp_casts(catalog::divination());
    assert!(!ok, "a nonlegendary sorcery is no legal target");
    let (g, jace, ok) = opp_casts(catalog::jace_beleren());
    assert!(ok);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == jace), "unpaid {{3}}: countered");
}

/// Punishing Punch costs {G} with two creature cards in the graveyard, and the
/// lifelinking Nighthawk is the damage source (gains 4).
#[test]
fn punishing_punch_discounts_and_creature_deals_damage() {
    let mut g = pod(2);
    let hawk = ready(&mut g, 0, catalog::vampire_nighthawk());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let pp = g.add_card_to_hand(0, catalog::punishing_punch());
    let go = |g: &mut GameState| {
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: pp,
            target: Some(Target::Permanent(hawk)),
            additional_targets: vec![Target::Permanent(giant)],
            mode: None,
            x_value: None,
        })
    };
    assert!(go(&mut g).is_err(), "no discount yet");
    g.players[0].mana_pool.empty();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    go(&mut g).expect("{G} with two creature cards");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.players[0].life, life + 4, "twice 2 power, lifelink from the creature");
}

/// Compel Brutality mode 1: Jace (loyalty 3) deals 3 to a 3/3 and keeps his loyalty.
#[test]
fn compel_brutality_planeswalker_deals_its_loyalty() {
    let mut g = pod(2);
    let jace = ready(&mut g, 0, catalog::jace_beleren());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let cb = g.add_card_to_hand(0, catalog::compel_brutality());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: cb,
        target: Some(Target::Permanent(jace)),
        additional_targets: vec![Target::Permanent(giant)],
        mode: Some(1),
        x_value: None,
    })
    .expect("compel");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 3);
}

/// Grafted Growth attaches to a land and can put its counter on a Vehicle.
#[test]
fn grafted_growth_counters_a_vehicle() {
    let mut g = pod(2);
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let copter = ready(&mut g, 0, catalog::smugglers_copter());
    let gg = g.add_card_to_hand(0, catalog::grafted_growth());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(copter))]));
    cast(&mut g, gg, Some(Target::Permanent(forest))).expect("aura");
    assert_eq!(g.battlefield_find(gg).unwrap().attached_to, Some(forest));
    assert_eq!(g.battlefield_find(copter).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Recursive Recruitment: two 2/2 Cadets from hand; flashed back over six
/// graveyard cards, each Cadet gets two counters.
#[test]
fn recursive_recruitment_grows_cadets_on_flashback() {
    let cadets = |g: &GameState| {
        let mut v: Vec<i32> = g
            .battlefield
            .iter()
            .filter(|c| c.definition.name == "Cadet")
            .map(|c| g.computed_permanent(c.id).unwrap().power)
            .collect();
        v.sort();
        v
    };
    let mut g = pod(2);
    let rr = g.add_card_to_hand(0, catalog::recursive_recruitment());
    flood(&mut g, 0);
    cast(&mut g, rr, None).expect("from hand");
    assert_eq!(cadets(&g), vec![2, 2]);
    // Six Forests; Recursive Recruitment itself is on the stack as it resolves.
    for _ in 0..6 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFlashback { card_id: rr, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("flashback");
    drain_stack(&mut g);
    assert_eq!(cadets(&g), vec![2, 2, 4, 4], "six cards in the graveyard: two counters each");
    assert!(g.exile.iter().any(|c| c.id == rr));
}

/// Doublecast copies only the next instant: Bolt then Bolt deals 6 + 3.
#[test]
fn doublecast_copies_the_next_instant_once() {
    let mut g = pod(2);
    let dc = g.add_card_to_hand(0, catalog::doublecast());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, dc, None).expect("doublecast");
    let opp = g.players[1].life;
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    }
    assert_eq!(g.players[1].life, opp - 9);
}

/// Vizier of Deferment exiles an attacker until the end step; a creature that
/// didn't attack or block stays.
#[test]
fn vizier_of_deferment_flickers_only_combatants() {
    let bears_of = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.name == "Grizzly Bears").count();
    for attacked in [true, false] {
        let mut g = pod(2);
        let bear = ready(&mut g, 1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().attacked_this_turn = attacked;
        let v = g.add_card_to_hand(0, catalog::vizier_of_deferment());
        flood(&mut g, 0);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        cast(&mut g, v, None).expect("vizier");
        assert_eq!(bears_of(&g), if attacked { 0 } else { 1 }, "attacked {attacked}");
        step_to(&mut g, 1, TurnStep::Upkeep);
        assert_eq!(bears_of(&g), 1, "back at the end step");
    }
}

/// Catharsis: {W}{W} makes two Kithkin; {R}{R} pumps and hastes your team;
/// evoked with {W}{W} it still makes Kithkin and is sacrificed.
#[test]
fn catharsis_reads_the_colors_spent() {
    for (white, evoke) in [(true, false), (false, false), (true, true)] {
        let mut g = pod(2);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let c = g.add_card_to_hand(0, catalog::catharsis());
        g.players[0].mana_pool.add(if white { Color::White } else { Color::Red }, 2);
        if !evoke {
            g.players[0].mana_pool.add_colorless(4);
        }
        if evoke {
            g.perform_action(GameAction::CastSpellAlternative { card_id: c, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None })
                .expect("evoke");
        } else {
            g.perform_action(GameAction::CastSpell { card_id: c, target: None, additional_targets: vec![], mode: None, x_value: None })
                .expect("hardcast");
        }
        drain_stack(&mut g);
        let kithkin = g.battlefield.iter().filter(|x| x.definition.name == "Kithkin").count();
        assert_eq!(kithkin, if white { 2 } else { 0 }, "white {white} evoke {evoke}");
        let bp = g.computed_permanent(bear).unwrap();
        assert_eq!(bp.power, if white { 2 } else { 3 });
        assert_eq!(g.permanent_has_keyword(bear, &Keyword::Haste), !white);
        assert_eq!(g.battlefield_find(c).is_some(), !evoke);
    }
}

/// Siren Lookout explores on entry: a land on top goes to hand, no counter.
#[test]
fn siren_lookout_explores_a_land_to_hand() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::siren_lookout());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    cast(&mut g, s, None).expect("lookout");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "forest on top went to hand");
    assert_eq!(g.battlefield_find(s).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Enter the Unknown: a nonland reveal grows the creature; one extra land play.
#[test]
fn enter_the_unknown_explores_and_grants_a_land_drop() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let id = g.next_id();
    g.players[0].add_to_library_top(id, catalog::lightning_bolt());
    let s = g.add_card_to_hand(0, catalog::enter_the_unknown());
    flood(&mut g, 0);
    cast(&mut g, s, Some(Target::Permanent(bear))).expect("enter the unknown");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].extra_land_plays, 1);
}

/// Gaze in Wonder taps two creatures, then the Butterflies wait in exile.
#[test]
fn velvetwing_butterflies_gaze_taps_two() {
    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b2 = ready(&mut g, 1, catalog::grizzly_bears());
    let v = g.add_card_to_hand(0, catalog::velvetwing_butterflies());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastAdventure {
        card_id: v,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b2)],
        mode: None,
        x_value: None,
    })
    .expect("gaze in wonder");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b2).unwrap().tapped);
    assert!(g.exile.iter().any(|c| c.id == v && c.on_adventure));
}

/// Web Shot kills a flier; Ettercap is then cast from exile with reach.
#[test]
fn ettercap_web_shot_then_creature() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let e = g.add_card_to_hand(0, catalog::ettercap());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastAdventure { card_id: e, target: Some(Target::Permanent(angel)), additional_targets: vec![], mode: None, x_value: None })
        .expect("web shot");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none());
    g.perform_action(GameAction::CastAdventureCreature { card_id: e, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("ettercap from exile");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(e, &Keyword::Reach));
}

/// Gloom Ripper: X = 2 Elves on the battlefield (itself included) + 1 Elf card
/// in the graveyard = 3: +3/+0 to one of yours, -0/-3 to the opponent's.
#[test]
fn gloom_ripper_counts_elves_on_board_and_in_graveyard() {
    let mut g = pod(2);
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let gr = g.add_card_to_hand(0, catalog::gloom_ripper());
    flood(&mut g, 0);
    cast(&mut g, gr, None).expect("gloom ripper");
    assert_eq!(pt(&g, angel), (4, 1));
    assert_eq!(pt(&g, gr).0 + pt(&g, elf).0, 4 + 1 + 3);
}

/// Ravenous Tyrannosaurus' attack trigger: 6 to a 2/2, 4 excess to its controller.
#[test]
fn ravenous_tyrannosaurus_spills_excess_onto_the_controller() {
    let mut g = pod(2);
    let t = ready(&mut g, 0, catalog::ravenous_tyrannosaurus());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: t, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].life, life - 4);
}

/// Knight of the Holy Nimbus regenerates from destruction until an opponent
/// pays {2}; its controller can't activate that ability.
#[test]
fn knight_of_the_holy_nimbus_regenerates_until_an_opponent_pays() {
    let mut g = pod(2);
    let k = ready(&mut g, 0, catalog::knight_of_the_holy_nimbus());
    let blade = g.add_card_to_hand(1, catalog::doom_blade());
    g.players[1].mana_pool.add(Color::Black, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: blade, target: Some(Target::Permanent(k)), additional_targets: vec![], mode: None, x_value: None })
        .expect("doom blade");
    drain_stack(&mut g);
    assert!(g.battlefield_find(k).is_some_and(|c| c.tapped), "regenerated");
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    assert!(activate(&mut g, k, None).is_err(), "only opponents may activate");
    g.players[1].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 1;
    activate(&mut g, k, None).expect("opponent pays {2}");
    let blade = g.add_card_to_hand(1, catalog::doom_blade());
    g.players[1].mana_pool.add(Color::Black, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: blade, target: Some(Target::Permanent(k)), additional_targets: vec![], mode: None, x_value: None })
        .expect("second doom blade");
    drain_stack(&mut g);
    assert!(g.battlefield_find(k).is_none());
}

/// Kenku Artificer animates Sol Ring into a 3/3 flier.
#[test]
fn kenku_artificer_animates_a_noncreature_artifact() {
    let mut g = pod(2);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let k = g.add_card_to_hand(0, catalog::kenku_artificer());
    flood(&mut g, 0);
    cast(&mut g, k, None).expect("kenku");
    assert_eq!(pt(&g, ring), (3, 3));
    assert!(g.permanent_has_keyword(ring, &Keyword::Flying));
}

/// Deceit reads the colors spent: {U}{U} bounces, {B}{B} strips a nonland card.
#[test]
fn deceit_reads_the_colors_spent_on_it() {
    for blue in [true, false] {
        let mut g = pod(2);
        let bear = ready(&mut g, 1, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.add_card_to_hand(1, catalog::forest());
        let d = g.add_card_to_hand(0, catalog::deceit());
        g.players[0].mana_pool.add(if blue { Color::Blue } else { Color::Black }, 2);
        g.players[0].mana_pool.add_colorless(4);
        cast(&mut g, d, None).expect("hardcast");
        assert_eq!(g.battlefield_find(bear).is_none(), blue, "bounced only with UU");
        assert_eq!(g.players[1].graveyard.iter().any(|c| c.id == bolt), !blue, "discard only with BB");
        assert!(g.battlefield_find(d).is_some());
    }
}

/// Titan's Revenge deals X and returns to hand on a won clash.
#[test]
fn titans_revenge_burns_and_returns_on_a_won_clash() {
    let mut g = pod(2);
    g.players[0].library.clear();
    g.players[1].library.clear();
    g.add_card_to_library(0, catalog::colossal_dreadmaw());
    g.add_card_to_library(1, catalog::island());
    let tr = g.add_card_to_hand(0, catalog::titans_revenge());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell { card_id: tr, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("X = 3");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
    assert!(g.players[0].hand.iter().any(|c| c.id == tr), "won the clash");
}

/// Theorix Charm: mode 1 shrinks a 2/2 to death; mode 2 mills three and draws one.
#[test]
fn theorix_charm_shrinks_or_mills_and_draws() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::theorix_charm());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: c, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: Some(1), x_value: None })
        .expect("-2/-2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    let c = g.add_card_to_hand(0, catalog::theorix_charm());
    flood(&mut g, 0);
    let (hand, gy, lib) = (g.players[0].hand.len(), g.players[0].graveyard.len(), g.players[0].library.len());
    g.perform_action(GameAction::CastSpell { card_id: c, target: None, additional_targets: vec![], mode: Some(2), x_value: None })
        .expect("mill and draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1);
    assert_eq!(g.players[0].graveyard.len(), gy + 3 + 1, "three milled plus the charm");
    assert_eq!(g.players[0].library.len(), lib - 4);
}

/// Seal from Existence exiles the opponent's creature until it leaves; it has ward.
#[test]
fn seal_from_existence_exiles_until_it_leaves() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let seal = g.add_card_to_hand(0, catalog::seal_from_existence());
    flood(&mut g, 0);
    cast(&mut g, seal, None).expect("seal");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.exile.iter().any(|c| c.id == bear));
    assert!(g.computed_permanent(seal).unwrap().keywords().iter().any(|k| matches!(k, Keyword::Ward(_))));
    let dis = g.add_card_to_hand(0, catalog::disenchant());
    cast(&mut g, dis, Some(Target::Permanent(seal))).expect("own spell ignores ward");
    assert!(g.battlefield_find(bear).is_some(), "bear returns when the seal leaves");
}

/// Winterthorn Blessing counters yours, taps and freezes theirs; flashback exiles it.
#[test]
fn winterthorn_blessing_counters_and_freezes() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let wb = g.add_card_to_hand(0, catalog::winterthorn_blessing());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: wb,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(theirs).unwrap().tapped, "skips its next untap");
    g.active_player_idx = 0;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFlashback {
        card_id: wb,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("flashback");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.exile.iter().any(|c| c.id == wb));
}

/// Uneasy Partings costs {2}{U} against an attacking nontoken creature, full
/// price otherwise; the owner's default puts it on the bottom.
#[test]
fn uneasy_partings_discounted_against_an_attacker() {
    let mut g = pod(2);
    let idle = ready(&mut g, 1, catalog::grizzly_bears());
    let attacker = ready(&mut g, 1, catalog::grizzly_bears());
    g.attacking.push(crabomination::game::Attack {
        attacker,
        target: crabomination::game::AttackTarget::Player(0),
    });
    let up = g.add_card_to_hand(0, catalog::uneasy_partings());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(cast(&mut g, up, Some(Target::Permanent(idle))).is_err(), "not attacking: {{3}}{{U}}");
    cast(&mut g, up, Some(Target::Permanent(attacker))).expect("{2}{U} against the attacker");
    assert!(g.battlefield_find(attacker).is_none());
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(attacker), "bottom of its owner's library");
}

/// Answered Prayers: a creature entering gains 1 and animates the enchantment as a 3/3 flier.
#[test]
fn answered_prayers_gains_and_animates() {
    let mut g = pod(2);
    let ap = ready(&mut g, 0, catalog::answered_prayers());
    assert!(!g.computed_permanent(ap).unwrap().card_types().contains(&crabomination::card::CardType::Creature));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let life = g.players[0].life;
    cast(&mut g, bear, None).expect("bear");
    assert_eq!(g.players[0].life, life + 1);
    let c = g.computed_permanent(ap).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3));
    assert!(c.keywords().contains(&Keyword::Flying));
    assert!(c.card_types().contains(&crabomination::card::CardType::Enchantment), "in addition to its types");
}

/// Reaping Willow enters as a 1/4; removing its two counters reanimates a
/// small creature and leaves it 3/6; with no counters left it can't again.
#[test]
fn reaping_willow_spends_counters_to_reanimate() {
    let mut g = pod(2);
    let rw = g.add_card_to_hand(0, catalog::reaping_willow());
    flood(&mut g, 0);
    cast(&mut g, rw, None).expect("willow");
    assert_eq!(pt(&g, rw), (1, 4));
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    activate(&mut g, rw, Some(Target::Permanent(bear))).expect("reanimate the bear");
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(pt(&g, rw), (3, 6));
    flood(&mut g, 0);
    assert!(activate(&mut g, rw, Some(Target::Permanent(giant))).is_err(), "no counters left");
}

/// Falcon's Wing Harness attaches on entry: +1/+1, flying, ward.
#[test]
fn falcons_wing_harness_attaches_on_entry() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let h = g.add_card_to_hand(0, catalog::falcons_wing_harness());
    flood(&mut g, 0);
    cast(&mut g, h, None).expect("harness");
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3));
    assert!(c.keywords().contains(&Keyword::Flying));
    assert!(c.keywords().iter().any(|k| matches!(k, Keyword::Ward(_))));
}

/// Battlefront Krushok grants its blocking limit only to countered creatures.
#[test]
fn battlefront_krushok_shares_with_countered_creatures() {
    let mut g = pod(2);
    let k = ready(&mut g, 0, catalog::battlefront_krushok());
    let plain = ready(&mut g, 0, catalog::grizzly_bears());
    let grown = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(grown).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let has = |g: &GameState, id| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::CantBeBlockedByMoreThanOne);
    assert!(has(&g, k));
    assert!(has(&g, grown));
    assert!(!has(&g, plain));
}

/// Blessings of Nature puts all four counters on a single target; miracle {G}.
#[test]
fn blessings_of_nature_distributes_four() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::blessings_of_nature());
    flood(&mut g, 0);
    cast(&mut g, s, Some(Target::Permanent(bear))).expect("blessings");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
    assert!(catalog::blessings_of_nature().miracle.is_some());
}

/// Baleful Beholder: mode 0 strips the opponent's enchantment; mode 1 gives menace.
#[test]
fn baleful_beholder_modes() {
    let mut g = pod(2);
    let anthem = ready(&mut g, 1, catalog::glorious_anthem());
    let bb = g.add_card_to_hand(0, catalog::baleful_beholder());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    flood(&mut g, 0);
    cast(&mut g, bb, None).expect("beholder");
    assert!(g.battlefield_find(anthem).is_none());
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bb = g.add_card_to_hand(0, catalog::baleful_beholder());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    flood(&mut g, 0);
    cast(&mut g, bb, None).expect("beholder");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Menace));
}
