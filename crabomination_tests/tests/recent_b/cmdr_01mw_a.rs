//! Commander: the 2026-10-18 (`01MWhJbe`) first closer batch
//! (`decks::cmdr_01mw_a`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

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

fn cast(g: &mut GameState, card_id: CardId) {
    g.perform_action(GameAction::CastSpell { card_id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn lore(g: &mut GameState, saga: CardId) {
    let add = crabomination::effect::Effect::AddCounter {
        what: crabomination::effect::Selector::ExactObjects(vec![saga]),
        kind: CounterType::Lore,
        amount: crabomination::card::Value::ONE,
    };
    let evs = g.resolve_effect(&add, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Origin of Iron Man: chapter I's tap is locked to the Saga; chapter III
/// deploys an artifact of mana value 5 or less from hand and leaves a bigger
/// one behind.
#[test]
fn origin_of_iron_man_locks_then_deploys_a_small_artifact() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::origin_of_iron_man());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let i = catalog::origin_of_iron_man().saga_chapters[0].1.clone();
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(saga);
    ctx.targets = vec![Target::Permanent(bear)];
    let evs = g.resolve_effect(&i, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped);
    assert_eq!(b.untap_locked_while_present, Some(saga));
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    let colossus = g.add_card_to_hand(0, catalog::darksteel_colossus());
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    lore(&mut g, saga);
    assert!(g.battlefield_find(stone).is_some(), "MV 2 artifact deployed");
    assert!(g.players[0].hand.iter().any(|c| c.id == colossus), "MV 11 stays in hand");
}

/// Odric, Blood-Cursed counts each listed keyword once: Serra Angel (flying,
/// vigilance) and Goblin Guide (haste) beside Odric make three Blood.
#[test]
fn odric_blood_cursed_counts_distinct_keywords() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::serra_angel());
    ready(&mut g, 0, catalog::serra_angel());
    ready(&mut g, 0, catalog::goblin_guide());
    let odric = g.add_card_to_hand(0, catalog::odric_blood_cursed());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, odric);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Blood").count(), 3);
}

/// Kellan, Daring Traveler: attacking takes a small creature off the top;
/// Journey On counts the opponents who control an artifact, at three seats.
#[test]
fn kellan_reveals_on_attack_and_maps_per_artifact_opponent() {
    let mut g = pod(3);
    let kellan = ready(&mut g, 0, catalog::kellan_daring_traveler());
    let bear = g.next_id();
    g.players[0].add_to_library_top(bear, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: kellan, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));

    let mut g = pod(3);
    ready(&mut g, 1, catalog::sol_ring());
    let k = g.add_card_to_hand(0, catalog::kellan_daring_traveler());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.perform_action(GameAction::CastAdventure { card_id: k, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Journey On");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Map").count(), 2);
}

/// Invasion of Regatha's ETB splits 4 to an opponent and 1 to a creature;
/// CR 614.5 — Disciples of the Inferno adds 2 to a noncreature source's damage
/// to an opponent or any creature, and nothing to a creature source's.
#[test]
fn invasion_of_regatha_burns_and_its_back_adds_two() {
    let mut g = pod(3);
    let inv = ready(&mut g, 0, catalog::invasion_of_regatha());
    let bear = ready(&mut g, 2, catalog::grizzly_bears());
    let life = g.players[1].life;
    let etb = catalog::invasion_of_regatha().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(inv);
    ctx.targets = vec![Target::Player(1), Target::Permanent(bear)];
    let evs = g.resolve_effect(&etb, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 4);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1);

    let mut g = pod(2);
    let disciples = ready(&mut g, 0, *catalog::invasion_of_regatha().back_face.unwrap());
    let own = ready(&mut g, 0, catalog::grizzly_bears());
    use crabomination::game::effects::EntityRef;
    let rock = ready(&mut g, 0, catalog::sol_ring());
    assert_eq!(g.scale_damage_to(Some(rock), EntityRef::Player(1), 1), 3, "noncreature source → opponent");
    assert_eq!(g.scale_damage_to(Some(rock), EntityRef::Permanent(own), 1), 3, "any creature, yours too");
    assert_eq!(g.scale_damage_to(Some(rock), EntityRef::Player(0), 1), 1, "not to you");
    assert_eq!(g.scale_damage_to(Some(disciples), EntityRef::Player(1), 4), 4, "a creature source");
}

/// Thorin, Mountain-king: Bonesplitter moves onto the bear, which then bites
/// the giant for 4; with nothing to attach, the bite never happens.
#[test]
fn thorin_mountain_king_equips_then_bites() {
    let run = |with_equipment: bool| {
        let mut g = pod(2);
        let thorin = ready(&mut g, 0, catalog::thorin_mountain_king());
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        if with_equipment {
            ready(&mut g, 0, catalog::bonesplitter());
        }
        let giant = ready(&mut g, 1, catalog::hill_giant());
        let etb = catalog::thorin_mountain_king().triggered_abilities[0].effect.clone();
        let mut ctx = EffectContext::for_spell(0, None, 0, 0);
        ctx.source = Some(thorin);
        ctx.targets = vec![Target::Permanent(bear), Target::Permanent(giant)];
        let evs = g.resolve_effect(&etb, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        g.battlefield_find(giant).is_none()
    };
    assert!(run(true), "a 4-power bite kills the giant");
    assert!(!run(false), "nothing attached, no bite");
}

/// Dwarven Hammer's ETB payment mints a Dwarf Berserker wearing the Hammer.
#[test]
fn dwarven_hammer_arrives_on_a_berserker() {
    let mut g = pod(2);
    let hammer = g.add_card_to_hand(0, catalog::dwarven_hammer());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, hammer);
    let host = g.battlefield_find(hammer).and_then(|c| c.attached_to).expect("attached");
    let berserker = g.battlefield_find(host).unwrap();
    assert_eq!(berserker.definition.name, "Dwarf Berserker");
    assert_eq!(g.computed_permanent(host).unwrap().power, 5);
}

/// Vow to Erebor untaps and pumps; only a Dwarf picks up an Equipment.
#[test]
fn vow_to_erebor_equips_only_a_dwarf() {
    let run = |def: crabomination::card::CardDefinition| {
        let mut g = pod(2);
        let who = ready(&mut g, 0, def);
        g.battlefield_find_mut(who).unwrap().tapped = true;
        let axe = ready(&mut g, 0, catalog::bonesplitter());
        let vow = g.add_card_to_hand(0, catalog::vow_to_erebor());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: vow,
            target: Some(Target::Permanent(who)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert!(!g.battlefield_find(who).unwrap().tapped);
        g.battlefield_find(axe).unwrap().attached_to == Some(who)
    };
    assert!(run(catalog::bombur_gentle_dreamer()), "a Dwarf");
    assert!(!run(catalog::grizzly_bears()), "not a Dwarf");
}

/// CR 500.8 — Cyclonus's back face connects: it converts, and the extra
/// beginning phase it makes comes right after this combat, so the turn then
/// resumes at the postcombat main (not the end step).
#[test]
fn cr_500_8_cyclonus_adds_a_beginning_phase_after_combat() {
    let mut g = pod(2);
    let cy = ready(&mut g, 0, catalog::cyclonus_the_saboteur());
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(cy);
    let flip = crabomination::effect::Effect::Transform { what: crabomination::effect::Selector::This };
    g.resolve_effect(&flip, &ctx).unwrap();
    assert_eq!(g.battlefield_find(cy).unwrap().definition.name, "Cyclonus, Cybertronian Fighter");
    let land = ready(&mut g, 0, catalog::island());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.turn_number = 3;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: cy, target: AttackTarget::Player(1) }]))
        .expect("attack");
    let turn = g.turn_number;
    let hand = g.players[0].hand.len();
    let mut seen = Vec::new();
    for _ in 0..60 {
        if g.step == TurnStep::End || g.is_game_over() {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
        if seen.last() != Some(&g.step) {
            seen.push(g.step);
        }
    }
    let at = seen.iter().position(|s| *s == TurnStep::EndCombat).expect("combat ended");
    assert_eq!(
        &seen[at + 1..],
        &[TurnStep::Upkeep, TurnStep::Draw, TurnStep::PostCombatMain, TurnStep::End],
        "untap, upkeep, draw, then the postcombat main"
    );
    assert_eq!(g.turn_number, turn);
    assert!(!g.battlefield_find(land).unwrap().tapped, "the extra untap step");
    assert_eq!(g.players[0].hand.len(), hand + 1, "the extra draw step");
    assert_eq!(g.battlefield_find(cy).unwrap().definition.name, "Cyclonus, the Saboteur", "converted");
}

/// Wicked Slumber's two stun counters: one on each of two targets, both on a
/// lone target.
#[test]
fn wicked_slumber_splits_two_stun_counters() {
    let stuns = |targets: usize| {
        let mut g = pod(2);
        let a = ready(&mut g, 1, catalog::grizzly_bears());
        let b = ready(&mut g, 1, catalog::hill_giant());
        let eff = catalog::wicked_slumber().effect.clone();
        let mut ctx = EffectContext::for_spell(0, None, 0, 0);
        ctx.targets = [Target::Permanent(a), Target::Permanent(b)][..targets].to_vec();
        let evs = g.resolve_effect(&eff, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        let n = |id| g.battlefield_find(id).map(|c| (c.tapped, c.counter_count(CounterType::Stun))).unwrap();
        (n(a), n(b))
    };
    assert_eq!(stuns(1), ((true, 2), (false, 0)));
    assert_eq!(stuns(2), ((true, 1), (true, 1)));
}

/// Tetsuo's burn mode deals the greatest mana value among its Equipment:
/// Bonesplitter (1) and Lightning Greaves (2) make 2.
#[test]
fn tetsuo_burns_for_its_greatest_equipment_mana_value() {
    let mut g = pod(2);
    let t = ready(&mut g, 0, catalog::tetsuo_imperial_champion());
    for def in [catalog::bonesplitter(), catalog::lightning_greaves()] {
        let e = ready(&mut g, 0, def);
        g.battlefield_find_mut(e).unwrap().attached_to = Some(t);
    }
    let life = g.players[1].life;
    let crabomination::effect::Effect::ChooseMode(modes) = catalog::tetsuo_imperial_champion().triggered_abilities[0].effect.clone()
    else {
        panic!("modal")
    };
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(t);
    ctx.targets = vec![Target::Player(1)];
    let evs = g.resolve_effect(&modes[0], &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(g.players[1].life, life - 2);
}

/// Unscythe: a creature its wearer damaged dies, is exiled, and a Zombie
/// takes its place.
#[test]
fn unscythe_exiles_its_victims_for_zombies() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let scythe = ready(&mut g, 0, catalog::unscythe_killer_of_kings());
    g.battlefield_find_mut(scythe).unwrap().attached_to = Some(bear);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bite = crabomination::effect::Effect::DealDamageEqualToPower {
        source: crabomination::effect::Selector::ExactObjects(vec![bear]),
        target: crabomination::effect::Selector::ExactObjects(vec![giant]),
    };
    let evs = g.resolve_effect(&bite, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == giant), "exiled");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Zombie" && c.controller == 0));
}

/// Thorin, King of Durin's Folk: a Dwarf entering makes a Treasure, and each
/// artifact token pumps the other Dwarves (not Thorin, not a non-Dwarf).
#[test]
fn thorin_king_of_durins_folk_mints_and_pumps() {
    let mut g = pod(2);
    let thorin = ready(&mut g, 0, catalog::thorin_king_of_durins_folk());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let dwarf = g.add_card_to_hand(0, catalog::bombur_gentle_dreamer());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, dwarf);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1);
    let p = |g: &GameState, id| g.computed_permanent(id).unwrap().power;
    assert_eq!(p(&g, dwarf), 6, "Bombur 5 + 1 Treasure");
    assert_eq!(p(&g, thorin), 4);
    assert_eq!(p(&g, bear), 2);
}

/// CR 702.62a — Gandalf of the Secret Fire: Shock cast from hand on an
/// opponent's turn resolves and lands in exile with three time counters and
/// suspend; on your own turn it goes to the graveyard.
#[test]
fn cr_702_62a_gandalf_suspends_off_turn_spells() {
    let run = |active: usize| {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::gandalf_of_the_secret_fire());
        g.active_player_idx = active;
        g.priority.player_with_priority = 0;
        let d = g.add_card_to_hand(0, catalog::shock());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        let life = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: d,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2, "resolved");
        g.exile.iter().find(|c| c.id == d).map(|c| (c.counter_count(CounterType::Time), c.granted_suspend))
    };
    assert_eq!(run(1), Some((3, true)), "opponent's turn");
    assert_eq!(run(0), None, "your own turn");
}
