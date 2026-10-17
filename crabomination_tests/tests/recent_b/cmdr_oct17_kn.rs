//! Commander: the 2026-10-17 (`yl8zjb`) two-card-gap closers
//! (`decks::cmdr_oct17_kn`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::effect::{Effect, Selector, Value};
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

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), GameError> {
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })?;
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

fn shock(g: &mut GameState, at: CardId) {
    let s = g.add_card_to_hand(0, catalog::shock());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_with(g, s, Some(Target::Permanent(at)), None).expect("shock");
}

/// Elektra's reflexive trigger: the 2 to you buys 4 to a creature.
#[test]
fn elektra_pays_two_life_for_four_damage() {
    let mut g = pod(2);
    let ogre = ready(&mut g, 1, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::elektra_femme_fatale());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast_with(&mut g, e, None, None).expect("elektra");
    assert_eq!(g.players[0].life, life - 2);
    assert!(g.battlefield_find(ogre).is_none(), "4 damage kills the 2/2");
}

/// Increasing Confusion mills X from hand and 2X when flashed back.
#[test]
fn increasing_confusion_doubles_from_graveyard() {
    let mut g = pod(2);
    let c = g.add_card_to_hand(0, catalog::increasing_confusion());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_with(&mut g, c, Some(Target::Player(1)), Some(2)).expect("cast");
    assert_eq!(g.players[1].graveyard.len(), 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastFlashback {
        card_id: c,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("flashback");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 6, "2 + twice 2");
}

/// Merfolk Pupil loots again from the graveyard, exiling itself.
#[test]
fn merfolk_pupil_loots_from_graveyard() {
    let mut g = pod(2);
    let p = g.add_card_to_graveyard(0, catalog::merfolk_pupil());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: p,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("graveyard loot");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != p), "exiled as a cost");
    assert_eq!(g.players[0].graveyard.len(), 1, "the discarded card");
}

/// Zalto ventures each time it's dealt damage; Venus grows by the damage.
#[test]
fn zalto_ventures_and_venus_grows_when_damaged() {
    let mut g = pod(2);
    let zalto = ready(&mut g, 0, catalog::zalto_fire_giant_duke());
    let venus = ready(&mut g, 0, catalog::venus_torn_between_worlds());
    shock(&mut g, zalto);
    assert!(g.players[0].dungeon.is_some(), "ventured");
    shock(&mut g, venus);
    assert_eq!(g.battlefield_find(venus).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Sentinel taps a creature of the active opponent at their beginning of
/// combat, and does nothing on its controller's own turn.
#[test]
fn sentinel_taps_on_each_opponents_combat() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::sentinel_of_the_eternal_watch());
    let bear = ready(&mut g, 2, catalog::grizzly_bears());
    step_to(&mut g, 2, TurnStep::DeclareAttackers);
    assert!(g.battlefield_find(bear).unwrap().tapped, "tapped at seat 2's combat");
}

/// Hollow Trees banks a storage counter each upkeep it stays tapped and
/// cashes them for {G} each.
#[test]
fn hollow_trees_stores_while_tapped() {
    let mut g = pod(2);
    let trees = ready(&mut g, 0, catalog::hollow_trees());
    g.battlefield_find_mut(trees).unwrap().tapped = true;
    g.battlefield_find_mut(trees).unwrap().add_counters(CounterType::Storage, 2);
    g.battlefield_find_mut(trees).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: trees,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("cash out");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2);
}

/// CR 506.2 — Jurin pumps each attacker by the creatures of the player *it*
/// attacks, not Jurin's defender.
#[test]
fn jurin_pumps_each_attacker_by_its_own_defender() {
    let mut g = pod(3);
    let jurin = ready(&mut g, 0, catalog::jurin_leading_the_charge());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..3 {
        ready(&mut g, 1, catalog::grizzly_bears());
    }
    ready(&mut g, 2, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: jurin, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    let p = |g: &GameState, id| g.computed_permanent(id).unwrap().power;
    assert_eq!(p(&g, jurin), 4 + 3);
    assert_eq!(p(&g, bear), 2 + 1);
}

/// CR 615 — Old Fat Spider's chapter II: the opposing creature it picks deals
/// no combat damage while the Saga remains; chapter I hexproofs our own.
#[test]
fn old_fat_spider_shuts_off_an_opposing_creatures_damage() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let saga = g.add_card_to_hand(0, catalog::old_fat_spider_cant_see_me());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_with(&mut g, saga, None, None).expect("saga");
    let kw = |g: &GameState, id, k| g.permanent_has_keyword(id, &k);
    assert!(kw(&g, mine, Keyword::Hexproof), "chapter I on our creature");
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let add = Effect::AddCounter { what: Selector::ExactObjects(vec![saga]), kind: CounterType::Lore, amount: Value::ONE };
    let evs = g.resolve_effect(&add, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(kw(&g, theirs, Keyword::DealsNoDamage), "chapter II aims at the opponent's");
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: theirs, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    let life = g.players[0].life;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    assert_eq!(g.players[0].life, life, "its damage is prevented");
}

fn gamma() -> crabomination::card::CardDefinition {
    use crabomination::card::{CardDefinition, CardType, CreatureType, Subtypes};
    CardDefinition {
        name: "Gamma Test Brute",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gamma], ..Default::default() },
        power: 5,
        toughness: 5,
        ..Default::default()
    }
}

/// CR 701.4a / 601.2c — Hulk's Thunderclap's third target exists only when a
/// Gamma creature was beheld, and the behold needs one to show.
#[test]
fn hulks_thunderclap_beholds_for_its_third_target() {
    let mut g = pod(2);
    let brute = ready(&mut g, 0, gamma());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let s = g.add_card_to_hand(0, catalog::hulks_thunderclap());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: s,
        target: Some(Target::Permanent(brute)),
        additional_targets: vec![Target::Permanent(bear), Target::Permanent(ring)],
        mode: None,
        x_value: None,
    })
    .expect("beheld the Gamma");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "5 damage");
    assert!(g.battlefield_find(ring).is_none(), "the behold rider");

    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::hulks_thunderclap());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let kicked = g.perform_action(GameAction::CastSpellKicked {
        card_id: s,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    });
    assert!(kicked.is_err(), "no Gamma to behold");
    g.perform_action(GameAction::CastSpell {
        card_id: s,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("unbeheld");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

/// CR 701.4a — "behold a Merfolk or pay {2}": a Merfolk on the battlefield or
/// in hand waives the {2}; without one the cast costs {3}{U}.
#[test]
fn silvergill_mentor_beholds_or_pays() {
    let mut g = pod(2);
    let m = g.add_card_to_hand(0, catalog::silvergill_mentor());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(cast_with(&mut g, m, None, None).is_err(), "{{2}} more with no Merfolk");
    ready(&mut g, 0, catalog::silvergill_mentor());
    cast_with(&mut g, m, None, None).expect("a Merfolk on the battlefield");
    let merfolk = g.battlefield.iter().filter(|c| c.definition.name == "Merfolk").count();
    assert_eq!(merfolk, 1, "the cast Mentor's token");
}

/// Extraordinary Journey exiles up to X creatures; the owner recasts theirs
/// from exile (paying for it), and that entry draws the Journey's controller
/// a card — once a turn.
#[test]
fn extraordinary_journey_draws_when_an_exiled_creature_returns() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let j = g.add_card_to_hand(0, catalog::extraordinary_journey());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: j, target: None, additional_targets: vec![], mode: None, x_value: Some(1) })
        .expect("X = 1");
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == bear).expect("the bear is exiled");
    assert!(exiled.may_play_until.is_some(), "its owner may play it");
    let hand = g.players[0].hand.len();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("recast from exile");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.players[0].hand.len(), hand + 1, "cast from exile: a card");
}

/// CR 702.190 — Leonardo's sneak cost marks the cast, so his entry pumps the
/// team; cast normally, no pump.
#[test]
fn leonardo_pumps_only_when_sneaked() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let other = ready(&mut g, 0, catalog::grizzly_bears());
    let leo = g.add_card_to_hand(0, catalog::leonardo_leader_in_blue());
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: leo,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("sneak");
    drain_stack(&mut g);
    let p = |g: &GameState, id| g.computed_permanent(id).unwrap().power;
    assert_eq!(p(&g, other), 4, "+2/+0 from the sneak");
    assert_eq!(p(&g, leo), 4);

    let mut g = pod(2);
    let other = ready(&mut g, 0, catalog::grizzly_bears());
    let leo = g.add_card_to_hand(0, catalog::leonardo_leader_in_blue());
    g.players[0].mana_pool.add(Color::White, 1);
    cast_with(&mut g, leo, None, None).expect("hard cast");
    assert_eq!(p(&g, other), 2, "no sneak, no pump");
}

/// Prehistoric Pet bounces another creature of yours only on your turn.
#[test]
fn prehistoric_pet_bounces_only_on_your_turn() {
    let mut g = pod(2);
    let pet = ready(&mut g, 0, catalog::prehistoric_pet());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let act = |g: &mut GameState| {
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: pet,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    g.active_player_idx = 1;
    assert!(act(&mut g).is_err(), "not on an opponent's turn");
    g.active_player_idx = 0;
    act(&mut g).expect("your turn");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// CR 613 — Futurist Operative tapped is a 1/1 unblockable Human Citizen
/// (layers 4 and 7b); untapped it's the printed 3/4 Human Ninja.
#[test]
fn futurist_operative_shifts_while_tapped() {
    use crabomination::card::CreatureType;
    let mut g = pod(2);
    let op = ready(&mut g, 0, catalog::futurist_operative());
    let cp = g.computed_permanent(op).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4));
    g.battlefield_find_mut(op).unwrap().tapped = true;
    let cp = g.computed_permanent(op).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.keywords().contains(&Keyword::Unblockable));
    assert!(cp.subtypes().creature_types.contains(&CreatureType::Citizen));
    assert!(!cp.subtypes().creature_types.contains(&CreatureType::Ninja));
}

fn hit_player(g: &mut GameState, attacker: CardId, defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    drain_stack(g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

/// Sphinx Ambassador takes the searched creature only when the player it hit
/// names a different card: over the two creature names on that ballot exactly
/// one answer lets the Hill Giant go, and nobody else's library is touched.
#[test]
fn sphinx_ambassador_steals_an_unnamed_creature() {
    let mut steals = 0;
    for pick in 0..2 {
        let mut g = pod(3);
        let sphinx = ready(&mut g, 0, catalog::sphinx_ambassador());
        let giant = g.add_card_to_library(2, catalog::hill_giant());
        g.add_card_to_library(2, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Cards(vec![giant]),
            DecisionAnswer::Amount(pick),
            DecisionAnswer::Bool(true),
        ]));
        hit_player(&mut g, sphinx, 2);
        if g.battlefield_find(giant).is_some_and(|c| c.controller == 0) {
            steals += 1;
        } else {
            assert!(g.players[2].library.iter().any(|c| c.id == giant), "named: stays, shuffled in");
        }
        assert_eq!(g.players[1].library.len(), 10, "seat 1 untouched");
    }
    assert_eq!(steals, 1);
}

/// Dragon's Smile crewed and connecting: the player hit exiles four cards and
/// we may play them; the first spell from exile on our turn costs {0}.
#[test]
fn dragons_smile_exiles_what_it_hits_for_and_casts_one_free() {
    let mut g = pod(2);
    let smile = ready(&mut g, 0, catalog::dragons_smile());
    let crew = ready(&mut g, 0, catalog::hill_giant());
    g.perform_action(GameAction::Crew { vehicle: smile, crew_creatures: vec![crew] }).expect("crew 3");
    drain_stack(&mut g);
    g.players[1].library.clear();
    let bear = g.add_card_to_library(1, catalog::grizzly_bears());
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    hit_player(&mut g, smile, 1);
    let bear = if g.exile.iter().any(|c| c.id == bear) { bear } else { g.exile.iter().find(|c| c.owner == 1).unwrap().id };
    let exiled = g.exile.iter().filter(|c| c.owner == 1).count();
    assert_eq!(exiled, 4, "four damage, four cards");
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{0} for the first spell from exile");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some_and(|c| c.controller == 0));
}

/// Master of the Wild Hunt: three Wargs maul a 6/4 Craw Wurm for 9, and its 6
/// power comes back divided among them — headless, exactly lethal each.
#[test]
fn master_of_the_wild_hunt_pack_mauls_and_takes_the_answer() {
    let mut g = pod(2);
    let master = ready(&mut g, 0, catalog::master_of_the_wild_hunt());
    let wolves: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::bejeweled_warg())).collect();
    let big = ready(&mut g, 1, catalog::craw_wurm());
    g.perform_action(GameAction::ActivateAbility {
        card_id: master,
        ability_index: 0,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("pack attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "9 damage kills the 6/4");
    let alive = wolves.iter().filter(|w| g.battlefield_find(**w).is_some()).count();
    assert_eq!(alive, 0, "6 power split 2/2/2 kills all three 3/2 Wargs");
    assert!(wolves.iter().filter_map(|w| g.battlefield_find(*w)).all(|c| c.tapped));
}

/// Winnowing: each player keeps the chosen creature and whatever shares a
/// creature type with it. Headless, our pick spares the most of ours and an
/// opponent's spares the fewest.
#[test]
fn winnowing_keeps_only_the_chosen_types() {
    let mut g = pod(2);
    let b1 = ready(&mut g, 0, catalog::grizzly_bears());
    let b2 = ready(&mut g, 0, catalog::grizzly_bears());
    let pupil = ready(&mut g, 0, catalog::merfolk_pupil());
    let o1 = ready(&mut g, 1, catalog::grizzly_bears());
    let o2 = ready(&mut g, 1, catalog::grizzly_bears());
    let w = g.add_card_to_hand(0, catalog::winnowing());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast_with(&mut g, w, None, None).expect("winnow");
    assert!(g.battlefield_find(b1).is_some() && g.battlefield_find(b2).is_some(), "the Bears pair survives");
    assert!(g.battlefield_find(pupil).is_none(), "the Merfolk shares nothing");
    assert_eq!([o1, o2].iter().filter(|id| g.battlefield_find(**id).is_some()).count(), 2, "two Bears share a type");
}

/// CR 611.2c — Olinda's odor counter carries "at the beginning of your upkeep,
/// you lose 2 life" to the creature's controller: one creature per opponent.
#[test]
fn olinda_stinks_up_one_creature_per_opponent() {
    let mut g = pod(3);
    let b1 = ready(&mut g, 1, catalog::grizzly_bears());
    let b2 = ready(&mut g, 2, catalog::grizzly_bears());
    let o = g.add_card_to_hand(0, catalog::olinda_the_oblivious());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast_with(&mut g, o, None, None).expect("Olinda");
    for b in [b1, b2] {
        assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::Odor), 1);
    }
    let life = g.players[1].life;
    step_to(&mut g, 1, TurnStep::Draw);
    assert_eq!(g.players[1].life, life - 2, "seat 1's upkeep");
}

/// The Neutrinos blink a creature we own back in tapped and attacking.
#[test]
fn the_neutrinos_blink_an_attacker_in() {
    let mut g = pod(2);
    let n = ready(&mut g, 0, catalog::the_neutrinos());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: n, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.attacking.iter().any(|a| a.attacker == bear), "the bear joined the attack");
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// CR 702.188 / 614.12 — web-slung Scarlet Spider enters with counters equal
/// to the returned creature's mana value; hard-cast, none.
#[test]
fn scarlet_spider_counts_the_returned_creatures_mana_value() {
    let mut g = pod(2);
    let giant = ready(&mut g, 0, catalog::hill_giant());
    g.battlefield_find_mut(giant).unwrap().tapped = true;
    let s = g.add_card_to_hand(0, catalog::scarlet_spider_ben_reilly());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: s,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("web-slinging");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == giant), "the Giant went back");
    assert_eq!(g.battlefield_find(s).unwrap().counter_count(CounterType::PlusOnePlusOne), 4, "Hill Giant's MV");
}

/// CR 614 — Darigaaz dies into exile with three egg counters, sheds one each
/// upkeep of its owner, and hatches back onto the battlefield on the third.
#[test]
fn darigaaz_hatches_back_after_three_upkeeps() {
    let mut g = pod(2);
    let d = ready(&mut g, 0, catalog::darigaaz_reincarnated());
    let doom = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast_with(&mut g, doom, Some(Target::Permanent(d)), None).expect("murder");
    let eggs = |g: &GameState| g.exile.iter().find(|c| c.id == d).map(|c| c.counter_count(CounterType::Egg));
    assert_eq!(eggs(&g), Some(3), "exiled with three eggs");
    for left in [2, 1] {
        step_to(&mut g, 1, TurnStep::Upkeep);
        step_to(&mut g, 0, TurnStep::Draw);
        assert_eq!(eggs(&g), Some(left));
    }
    step_to(&mut g, 1, TurnStep::Upkeep);
    step_to(&mut g, 0, TurnStep::Draw);
    assert!(g.battlefield_find(d).is_some(), "hatched");
}

/// Bot Bashing Time's 6 damage kills into exile, not the graveyard.
#[test]
fn bot_bashing_time_exiles_what_it_kills() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let s = g.add_card_to_hand(0, catalog::bot_bashing_time());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast_with(&mut g, s, Some(Target::Permanent(wurm)), None).expect("bash");
    assert!(g.exile.iter().any(|c| c.id == wurm));
    assert!(g.players[1].graveyard.iter().all(|c| c.id != wurm));
}

/// Magik flips at the end step only after two sacrifices this turn.
#[test]
fn magik_transforms_after_two_sacrifices() {
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::magik_illyana_rasputin());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].permanents_sacrificed_this_turn = 1;
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert_eq!(g.battlefield_find(m).unwrap().definition.name, "Magik, Illyana Rasputin", "one sacrifice: no flip");
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::magik_illyana_rasputin());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].permanents_sacrificed_this_turn = 2;
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert_eq!(g.battlefield_find(m).unwrap().definition.name, "Darkchild, Demon Uncaged");
}

/// Merieke takes a creature for as long as we control her; untapping her (or
/// her leaving) destroys it.
#[test]
fn merieke_steals_until_she_untaps() {
    let mut g = pod(2);
    let m = ready(&mut g, 0, catalog::merieke_ri_berit());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    g.perform_action(GameAction::ActivateAbility {
        card_id: m,
        ability_index: 0,
        target: Some(Target::Permanent(wurm)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("steal");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wurm).unwrap().controller, 0);
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Untap { what: Selector::ExactObjects(vec![m]), up_to: None }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_none(), "untapped: the Wurm is destroyed");
}

/// CR 707.2 — the 'Dane becomes a copy of what it sacrificed and keeps the
/// ability, so it can shift again.
#[test]
fn the_ever_changing_dane_copies_its_sacrifice() {
    let mut g = pod(2);
    let dane = ready(&mut g, 0, catalog::the_ever_changing_dane());
    ready(&mut g, 0, catalog::craw_wurm());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dane,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("shift");
    drain_stack(&mut g);
    let cp = g.computed_permanent(dane).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 4), "a Craw Wurm now");
    assert!(!g.battlefield_find(dane).unwrap().definition.activated_abilities.is_empty(), "kept this ability");
}

/// Fly's granted trigger ventures when the enchanted creature connects.
#[test]
fn fly_ventures_on_combat_damage() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let f = g.add_card_to_hand(0, catalog::fly());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast_with(&mut g, f, Some(Target::Permanent(bear)), None).expect("Fly");
    assert!(g.permanent_has_keyword(bear, &Keyword::Flying));
    hit_player(&mut g, bear, 1);
    assert!(g.players[0].dungeon.is_some(), "ventured");
}

/// Helm of Kaldra assembles Kaldra only with all three named Equipment.
#[test]
fn helm_of_kaldra_assembles_kaldra() {
    let mut g = pod(2);
    let helm = ready(&mut g, 0, catalog::helm_of_kaldra());
    let act = |g: &mut GameState| {
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: helm,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("activate");
        drain_stack(g);
    };
    let kaldra = |g: &GameState| g.battlefield.iter().find(|c| c.definition.name == "Kaldra").map(|c| c.id);
    act(&mut g);
    assert!(kaldra(&g).is_none(), "the Helm alone makes nothing");
    let sword = ready(&mut g, 0, catalog::sword_of_kaldra());
    let shield = ready(&mut g, 0, catalog::shield_of_kaldra());
    act(&mut g);
    let k = kaldra(&g).expect("Kaldra");
    for e in [helm, sword, shield] {
        assert_eq!(g.battlefield_find(e).unwrap().attached_to, Some(k));
    }
}

/// Corrosive Ooze blocking an equipped attacker destroys its Equipment at end
/// of combat.
#[test]
fn corrosive_ooze_eats_the_attackers_equipment() {
    let mut g = pod(2);
    let ooze = ready(&mut g, 1, catalog::corrosive_ooze());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let kit = ready(&mut g, 0, catalog::shield_spy_kit());
    g.battlefield_find_mut(kit).unwrap().attached_to = Some(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(ooze, bear)])).expect("block");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(kit).is_none(), "the Spy Kit is destroyed at end of combat");
}

/// Bug fix (pod seed 9330): Guided Passage's "an opponent chooses" is one
/// opponent — in a 4-seat pod it named `EachOpponent` and resolved it singly.
#[test]
fn guided_passage_has_one_opponent_pick_in_a_pod() {
    let mut g = pod(4);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::guided_passage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand = g.players[0].hand.len();
    cast_with(&mut g, s, None, None).expect("Guided Passage");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "a creature and a land");
}

/// Displacer Beast ventures on entry and bounces itself for {3}{U}.
#[test]
fn displacer_beast_ventures_and_displaces() {
    let mut g = pod(2);
    let b = g.add_card_to_hand(0, catalog::displacer_beast());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_with(&mut g, b, None, None).expect("Beast");
    assert!(g.players[0].dungeon.is_some());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: b,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("displacement");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == b));
}

/// Kumano's chapter I pings each opponent and their planeswalkers, not us.
#[test]
fn kumano_faces_kakkazan_pings_each_opponent() {
    let mut g = pod(3);
    let s = g.add_card_to_hand(0, catalog::kumano_faces_kakkazan());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    cast_with(&mut g, s, None, None).expect("saga");
    assert_eq!(g.players[0].life, life[0]);
    assert_eq!(g.players[1].life, life[1] - 1);
    assert_eq!(g.players[2].life, life[2] - 1);
}

/// Kicked In Thrall to the Pit sacrifices the stolen creature at the end step.
#[test]
fn in_thrall_to_the_pit_kicked_sacrifices_at_end_step() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let s = g.add_card_to_hand(0, catalog::in_thrall_to_the_pit());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: s,
        target: Some(Target::Permanent(wurm)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wurm).unwrap().controller, 0);
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert!(g.battlefield_find(wurm).is_none(), "sacrificed at our end step");
}

/// Draining Whelk counters a spell and grows by its mana value.
#[test]
fn draining_whelk_counters_and_grows() {
    let mut g = pod(2);
    let wurm = g.add_card_to_hand(1, catalog::craw_wurm());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 2);
    g.players[1].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell { card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("wurm");
    let w = g.add_card_to_hand(0, catalog::draining_whelk());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell { card_id: w, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("flash Whelk");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == wurm), "countered");
    assert_eq!(g.battlefield_find(w).unwrap().counter_count(CounterType::PlusOnePlusOne), 6);
}

/// Predatory Advantage: a Lizard at the end step of an opponent who cast no
/// creature spell, none on our own turn.
#[test]
fn predatory_advantage_punishes_a_creatureless_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::predatory_advantage());
    let lizards = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Lizard").count();
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert_eq!(lizards(&g), 0, "not on our end step");
    step_to(&mut g, 0, TurnStep::Upkeep);
    assert_eq!(lizards(&g), 1, "seat 1 cast no creature");
}

/// CR 601.2c / 201.2 — Harness the Storm's target is a graveyard card named
/// like the spell just cast from hand; the recast pays its own cost.
#[test]
fn harness_the_storm_recasts_the_same_named_card() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::harness_the_storm());
    let old = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let other = g.add_card_to_graveyard(0, catalog::shock());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    cast_with(&mut g, bolt, Some(Target::Player(1)), None).expect("bolt");
    assert_eq!(g.players[1].life, life - 6, "both Bolts hit");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the recast paid {{R}}");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == old));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == other), "Shock's name differs");
}

/// Iron Man draws on attack only after an artifact entered this turn.
#[test]
fn iron_man_draws_after_an_artifact_entered() {
    let mut g = pod(2);
    let im = ready(&mut g, 0, catalog::iron_man_master_of_machines());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    g.players[0].mana_pool.add_colorless(1);
    cast_with(&mut g, ring, None, None).expect("ring");
    assert_eq!(g.computed_permanent(im).unwrap().power, 2, "+1/+0 per other artifact");
    let hand = g.players[0].hand.len();
    hit_player(&mut g, im, 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Hexhaven Battalion: three named Cadets and an empowered Jace token.
#[test]
fn hexhaven_battalion_makes_cadets_and_a_jace() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::hexhaven_battalion());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast_with(&mut g, s, None, None).expect("cast");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Cadet" && c.controller == 0).count(), 3);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.counter_count(CounterType::Loyalty) >= 2));
}

/// Courier of Comestibles makes a Food only when no card went to hand.
#[test]
fn courier_of_comestibles_falls_back_to_a_food_token() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let c = g.add_card_to_hand(0, catalog::courier_of_comestibles());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_with(&mut g, c, None, None).expect("courier");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Food").count(), 1, "no Food card to find");

    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let pizza = g.add_card_to_library(0, catalog::spicy_oatmeal_pizza());
    let c = g.add_card_to_hand(0, catalog::courier_of_comestibles());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_with(&mut g, c, None, None).expect("courier");
    assert!(g.players[0].hand.iter().any(|c| c.id == pizza), "found the Food card");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Food"));
}

/// Hallowed Respite: our blinked creature gains a counter, theirs comes back
/// to its owner tapped.
#[test]
fn hallowed_respite_counters_ours_and_taps_theirs() {
    let mut g = pod(2);
    let ours = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    for target in [ours, theirs] {
        let s = g.add_card_to_hand(0, catalog::hallowed_respite());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Blue, 1);
        cast_with(&mut g, s, Some(Target::Permanent(target)), None).expect("respite");
    }
    let o = g.battlefield_find(ours).unwrap();
    assert_eq!(o.counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(!o.tapped);
    let t = g.battlefield_find(theirs).unwrap();
    assert_eq!((t.controller, t.tapped, t.counter_count(CounterType::PlusOnePlusOne)), (1, true, 0));
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: index, target, additional_targets: vec![], x_value: None, mode: None })?;
    drain_stack(g);
    Ok(())
}

/// Lantern of Revealing deploys a land on top tapped; a nonland it may bottom.
#[test]
fn lantern_of_revealing_deploys_a_land_or_bottoms_the_rest() {
    let mut g = pod(2);
    let l = ready(&mut g, 0, catalog::lantern_of_revealing());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add_colorless(4);
    activate(&mut g, l, 1, None).expect("lantern");
    assert_eq!(g.players[0].library.len(), 9);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Forest" && c.tapped));

    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let l = ready(&mut g, 0, catalog::lantern_of_revealing());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add_colorless(4);
    activate(&mut g, l, 1, None).expect("lantern");
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bear), "the nonland went to the bottom");
}

/// Counterlash counters a creature spell and casts a creature from hand free;
/// an instant in hand shares no type with it.
#[test]
fn counterlash_casts_a_same_typed_spell_free() {
    let mut g = pod(2);
    let wurm = g.add_card_to_hand(1, catalog::craw_wurm());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 2);
    g.players[1].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell { card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("wurm");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let c = g.add_card_to_hand(0, catalog::counterlash());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear]), DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell { card_id: c, target: Some(Target::Permanent(wurm)), additional_targets: vec![], mode: None, x_value: None })
        .expect("counterlash");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == wurm), "countered");
    assert!(g.battlefield_find(bear).is_some_and(|c| c.controller == 0), "a creature cast free");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
}

/// Thornmantle Striker's -X/-X counts the Elves you control (itself included).
#[test]
fn thornmantle_striker_shrinks_by_elf_count() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::llanowar_elves());
    let ogre = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::thornmantle_striker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    cast_with(&mut g, s, None, None).expect("striker");
    assert!(g.battlefield_find(ogre).is_none(), "two Elves: -2/-2 kills the 2/2");
}

/// CR 702.62a — Reality Strobe bounces, then suspends itself with three time
/// counters.
#[test]
fn reality_strobe_bounces_and_resuspends() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::reality_strobe());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast_with(&mut g, s, Some(Target::Permanent(bear)), None).expect("strobe");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
    let e = g.exile.iter().find(|c| c.id == s).expect("exiled");
    assert_eq!(e.counter_count(CounterType::Time), 3);
}

/// Conduit of Storms banks {R} for the postcombat main phase; {3}{R}{R}
/// transforms it into colorless Conduit of Emrakul.
#[test]
fn conduit_of_storms_banks_red_for_second_main() {
    let mut g = pod(2);
    let c = ready(&mut g, 0, catalog::conduit_of_storms());
    hit_player(&mut g, c, 1);
    step_to(&mut g, 0, TurnStep::PostCombatMain);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, c, 0, None).expect("transform");
    let back = g.battlefield_find(c).unwrap();
    assert_eq!(back.definition.name, "Conduit of Emrakul");
    assert_eq!(g.computed_permanent(c).unwrap().power, 5);
}

/// Jerren replaces a dying nontoken Human (not an Elf) for 1 life; at
/// exactly 13 life the end step offers the {4}{B}{B} flip.
#[test]
fn jerren_recruits_and_flips_at_thirteen() {
    let mut g = pod(2);
    let j = ready(&mut g, 0, catalog::jerren_corrupted_bishop());
    let elf = ready(&mut g, 0, catalog::elvish_visionary());
    let soldier = ready(&mut g, 0, catalog::elite_vanguard());
    let humans = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Human").count();
    shock(&mut g, elf);
    assert_eq!(humans(&g), 0, "an Elf is no Human");
    let life = g.players[0].life;
    shock(&mut g, soldier);
    assert_eq!((humans(&g), g.players[0].life), (1, life - 1));
    g.players[0].life = 13;
    for _ in 0..6 {
        ready(&mut g, 0, catalog::swamp());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    step_to(&mut g, 0, TurnStep::End);
    assert_eq!(g.battlefield_find(j).unwrap().definition.name, "Ormendahl, the Corrupter");
}
