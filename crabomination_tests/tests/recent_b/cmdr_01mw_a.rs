//! Commander: the 2026-10-18 (`01MWhJbe`) first closer batch
//! (`decks::cmdr_01mw_a`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
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

/// Thorin, Mountain-king: cast for real, the ETB targets the Equipment (CR
/// 601.2c, slots 1..) and moves both onto one creature, which then bites the
/// giant — a reflexive trigger (CR 603.12); with nothing to attach, no bite.
#[test]
fn thorin_mountain_king_equips_then_bites() {
    let run = |with_equipment: bool| {
        let mut g = pod(2);
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        let gear = with_equipment.then(|| [ready(&mut g, 0, catalog::bonesplitter()), ready(&mut g, 0, catalog::short_sword())]);
        let giant = ready(&mut g, 1, catalog::hill_giant());
        let thorin = g.add_card_to_hand(0, catalog::thorin_mountain_king());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, thorin);
        drain_stack(&mut g);
        if let Some([a, b]) = gear {
            let host = g.battlefield_find(a).unwrap().attached_to.expect("Bonesplitter attached");
            assert_eq!(g.battlefield_find(b).unwrap().attached_to, Some(host), "both on one creature");
            assert!([bear, thorin].contains(&host));
        }
        g.battlefield_find(giant).is_none()
    };
    assert!(run(true), "the equipped creature's bite kills the giant");
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
    // A five-mana Aura on it doesn't count: N is among Equipment only.
    let aura = crabomination::card::CardDefinition {
        name: "Heavy Aura",
        cost: crabomination::mana::cost(&[crabomination::mana::generic(5)]),
        card_types: vec![crabomination::card::CardType::Enchantment],
        subtypes: crabomination::card::Subtypes {
            enchantment_subtypes: vec![crabomination::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        ..Default::default()
    };
    for def in [catalog::bonesplitter(), catalog::lightning_greaves(), aura] {
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

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// CR 702.141 — Araumi of the Dead Tide at three seats: the cost exiles two
/// graveyard cards (one per opponent) but never the target; the Hill Giant
/// gains encore at its own {3}{R} and makes one hasty copy per opponent.
#[test]
fn cr_702_141_araumi_grants_encore_at_the_cards_cost() {
    let mut g = pod(3);
    let araumi = ready(&mut g, 0, catalog::araumi_of_the_dead_tide());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let one = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    assert!(activate(&mut g, araumi, 0, Some(Target::Permanent(giant))).is_err(), "one other card can't pay for two opponents");
    let two = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    activate(&mut g, araumi, 0, Some(Target::Permanent(giant))).expect("Araumi");
    for id in [one, two] {
        assert!(g.exile.iter().any(|c| c.id == id), "cost card exiled");
    }
    assert!(g.players[0].graveyard.iter().any(|c| c.id == giant), "the target stays");
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let idx = catalog::hill_giant().activated_abilities.len();
    activate(&mut g, giant, idx, None).expect("encore");
    let copies = g.battlefield.iter().filter(|c| c.definition.name == "Hill Giant" && c.controller == 0).count();
    assert_eq!(copies, 2, "one per opponent");
}

/// The Cloning of Shredder: chapter I exiles a legendary creature card and
/// mints a non-legendary Mutant copy; chapter II mints another off the same
/// exiled card, and the two copies coexist (no legend rule).
#[test]
fn the_cloning_of_shredder_mints_mutant_copies() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::the_cloning_of_shredder());
    let bombur = g.add_card_to_graveyard(0, catalog::bombur_gentle_dreamer());
    let i = catalog::the_cloning_of_shredder().saga_chapters[0].1.clone();
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(saga);
    ctx.targets = vec![Target::Permanent(bombur)];
    let evs = g.resolve_effect(&i, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bombur));
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 1);
    lore(&mut g, saga);
    let copies: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Bombur, Gentle Dreamer").collect();
    assert_eq!(copies.len(), 2);
    for c in copies {
        assert!(c.is_token);
        assert!(!c.definition.supertypes.contains(&crabomination::card::Supertype::Legendary));
        assert!(c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Mutant));
    }
}

/// Detritivore counts nonbasic land cards in every opponent's graveyard at
/// three seats — not basics, not its controller's own.
#[test]
fn detritivore_counts_nonbasics_across_opponents_graveyards() {
    let mut g = pod(3);
    let d = ready(&mut g, 0, catalog::detritivore());
    g.add_card_to_graveyard(1, catalog::command_tower());
    g.add_card_to_graveyard(1, catalog::forest());
    g.add_card_to_graveyard(2, catalog::evolving_wilds());
    g.add_card_to_graveyard(0, catalog::command_tower());
    let cp = g.computed_permanent(d).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Mind Carver: +1/+0, or +3/+1 once an opponent has eight cards in their
/// graveyard.
#[test]
fn mind_carver_grows_on_a_deep_opponent_graveyard() {
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let mc = ready(&mut g, 0, catalog::mind_carver());
    g.battlefield_find_mut(mc).unwrap().attached_to = Some(bear);
    let pt = |g: &GameState| {
        let cp = g.computed_permanent(bear).unwrap();
        (cp.power, cp.toughness)
    };
    assert_eq!(pt(&g), (3, 2));
    for _ in 0..8 {
        g.add_card_to_graveyard(2, catalog::forest());
    }
    assert_eq!(pt(&g), (5, 3));
}

/// Ajani Vengeant's −7 destroys only the target player's lands.
#[test]
fn ajani_vengeant_ultimate_razes_one_player() {
    let mut g = pod(3);
    let ajani = ready(&mut g, 0, catalog::ajani_vengeant());
    let lands: Vec<_> = (0..3).map(|s| ready(&mut g, s, catalog::forest())).collect();
    let ult = catalog::ajani_vengeant().loyalty_abilities[2].effect.clone();
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(ajani);
    ctx.targets = vec![Target::Player(2)];
    g.resolve_effect(&ult, &ctx).unwrap();
    assert!(g.battlefield_find(lands[2]).is_none());
    assert!(g.battlefield_find(lands[1]).is_some() && g.battlefield_find(lands[0]).is_some());
}

/// CR 702.193 — Donald Blake's power-up: 3/5 flying, and a God Warrior Hero
/// with no other creature types.
#[test]
fn cr_702_193_donald_blake_powers_up_into_a_god() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = pod(2);
    let db = ready(&mut g, 0, catalog::donald_blake_guise_of_thor());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    activate(&mut g, db, 0, None).expect("power-up");
    let cp = g.computed_permanent(db).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 5));
    assert!(cp.keywords().contains(&Keyword::Flying));
    let types = g.computed_permanent(db).unwrap().subtypes().creature_types.clone();
    assert!(types.contains(&CreatureType::God) && types.contains(&CreatureType::Hero));
    assert!(!types.contains(&CreatureType::Human));
}

/// End Hostilities takes the attachments on creatures with them; an
/// unattached Equipment survives.
#[test]
fn end_hostilities_takes_attachments_too() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let on = ready(&mut g, 1, catalog::bonesplitter());
    g.battlefield_find_mut(on).unwrap().attached_to = Some(bear);
    let off = ready(&mut g, 1, catalog::bonesplitter());
    g.resolve_effect(&catalog::end_hostilities().effect, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(on).is_none());
    assert!(g.battlefield_find(off).is_some());
}

/// Bring to Life animates a noncreature artifact as a 0/0 with four
/// +1/+1 counters, still an artifact.
#[test]
fn bring_to_life_animates_an_artifact() {
    let mut g = pod(2);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let fae = g.add_card_to_hand(0, catalog::animating_faerie());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastAdventure {
        card_id: fae,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bring to Life");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ring).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.card_types().contains(&crabomination::card::CardType::Artifact));
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
}

/// Arcee, Acrobatic Coupe: a spell whose SECOND target is a creature you
/// control still counts ("a spell that targets one or more creatures you
/// control"), and the counters are "that many" — one here — before Arcee
/// converts back.
#[test]
fn arcee_coupe_counts_every_target_slot() {
    let mut g = pod(2);
    let arcee = ready(&mut g, 0, catalog::arcee_sharpshooter());
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(arcee);
    let flip = crabomination::effect::Effect::Transform { what: crabomination::effect::Selector::This };
    g.resolve_effect(&flip, &ctx).unwrap();
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let ws = g.add_card_to_hand(0, catalog::wicked_slumber());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: ws,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![Target::Permanent(mine)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let a = g.battlefield_find(arcee).unwrap();
    assert_eq!(a.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(a.definition.name, "Arcee, Sharpshooter", "converted back");
}

/// CR 608.2c / 101.2 — a free cast a lock forbids is skipped, not an error:
/// under Codie, Vociferous Codex ("you can't cast permanent spells"),
/// Improvisation Capstone resolves, casts the Shock it exiled and leaves the
/// Hill Giant in exile (fresh-seed pod sweep 9209 aborted on this).
#[test]
fn cr_101_2_a_locked_free_cast_is_skipped() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::codie_vociferous_codex());
    for def in [catalog::hill_giant(), catalog::shock()] {
        let id = g.next_id();
        g.players[0].add_to_library_top(id, def);
    }
    let cap = g.add_card_to_hand(0, catalog::improvisation_capstone());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.players[0].mana_pool.add_colorless(5);
    let life = g.players[1].life;
    cast(&mut g, cap);
    assert!(g.exile.iter().any(|c| c.definition.name == "Hill Giant"), "the permanent stays exiled");
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Hill Giant"));
    assert_eq!(g.players[1].life, life - 2, "the Shock was cast");
}

/// Dáin of the Ancient Halls hits each opponent for the Dwarves you control
/// (Dáin and Nori: 2 apiece at three seats); Nori's attack hands an attacking
/// creature first strike.
#[test]
fn dain_and_nori_attack_together() {
    use crabomination::card::Keyword;
    let mut g = pod(3);
    let dain = ready(&mut g, 0, catalog::dain_of_the_ancient_halls());
    let nori = ready(&mut g, 0, catalog::nori_teller_of_tales());
    let life = [g.players[1].life, g.players[2].life];
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: dain, target: AttackTarget::Player(1) },
        Attack { attacker: nori, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [life[0] - 2, life[1] - 2]);
    let fs = |id| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::FirstStrike);
    assert!(fs(dain) || fs(nori), "one attacker gained first strike");
}

/// Turtle Van's attack feeds the creature that crewed it — doubled for a
/// Mutant Ninja Turtle (1 → 2), a single counter for a bear.
#[test]
fn turtle_van_doubles_for_a_ninja_turtle_crew() {
    let run = |crew: crabomination::card::CardDefinition| {
        let mut g = pod(2);
        let van = ready(&mut g, 0, catalog::turtle_van());
        let who = ready(&mut g, 0, crew);
        g.battlefield_find_mut(van).unwrap().crewed_by.push(who);
        let atk = catalog::turtle_van().triggered_abilities[0].effect.clone();
        let mut ctx = EffectContext::for_spell(0, None, 0, 0);
        ctx.source = Some(van);
        ctx.targets = vec![Target::Permanent(who)];
        let evs = g.resolve_effect(&atk, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        g.battlefield_find(who).unwrap().counter_count(CounterType::PlusOnePlusOne)
    };
    assert_eq!(run(catalog::leonardo_sewer_samurai()), 2);
    assert_eq!(run(catalog::grizzly_bears()), 1);
}

/// Vision Quest at X = 4 fetches a four-drop artifact creature from the
/// graveyard with four +1/+1 counters and haste.
#[test]
fn vision_quest_fetches_with_counters_and_haste() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let golem = g.add_card_to_graveyard(0, catalog::burnished_hart());
    let vq = g.add_card_to_hand(0, catalog::vision_quest());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell { card_id: vq, target: None, additional_targets: vec![], mode: None, x_value: Some(4) })
        .expect("cast");
    drain_stack(&mut g);
    let c = g.battlefield_find(golem).expect("on the battlefield");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 4);
    assert!(g.computed_permanent(golem).unwrap().keywords().contains(&Keyword::Haste));
}

/// Lithomantic Barrage: 5 to a blue creature, 1 to a green one.
#[test]
fn lithomantic_barrage_punishes_white_and_blue() {
    let hit = |def: crabomination::card::CardDefinition| {
        let mut g = pod(2);
        let c = ready(&mut g, 1, def);
        let mut ctx = EffectContext::for_spell(0, None, 0, 0);
        ctx.targets = vec![Target::Permanent(c)];
        g.resolve_effect(&catalog::lithomantic_barrage().effect, &ctx).unwrap();
        g.battlefield_find(c).map(|c| c.damage)
    };
    assert_eq!(hit(catalog::craw_wurm()), Some(1));
    assert_eq!(hit(catalog::serra_angel()), None, "5 kills the 4/4 Angel");
}

/// Warbeast of Gorgoroth: a power-4 creature of yours dying amasses Orcs 2;
/// a bear dying doesn't.
#[test]
fn warbeast_of_gorgoroth_amasses_on_big_deaths() {
    let run = |def: crabomination::card::CardDefinition| {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::warbeast_of_gorgoroth());
        let c = ready(&mut g, 0, def);
        let kill = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::ExactObjects(vec![c]) };
        let evs = g.resolve_effect(&kill, &EffectContext::for_spell(1, None, 0, 0)).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        g.battlefield.iter().filter(|c| c.definition.name.contains("Army")).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).sum::<u32>()
    };
    assert_eq!(run(catalog::craw_wurm()), 2);
    assert_eq!(run(catalog::grizzly_bears()), 0);
}

/// Surtland Flinger flings a sacrificed Giant for twice its power: Hill Giant
/// (3) is 6 to the defending player.
#[test]
fn surtland_flinger_doubles_a_flung_giant() {
    let mut g = pod(2);
    let flinger = ready(&mut g, 0, catalog::surtland_flinger());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: flinger, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "sacrificed");
    assert_eq!(g.players[1].life, life - 6);
}

/// Biolume Egg: sacrificed, it comes back as Biolume Serpent at the
/// beginning of the next end step.
#[test]
fn biolume_egg_hatches_after_a_sacrifice() {
    let mut g = pod(2);
    let egg = ready(&mut g, 0, catalog::biolume_egg());
    let sac = crabomination::effect::Effect::Sacrifice {
        who: crabomination::effect::Selector::You,
        count: crabomination::card::Value::ONE,
        filter: crabomination::card::SelectionRequirement::Creature,
    };
    let evs = g.resolve_effect(&sac, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(egg).is_none());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Biolume Serpent" && c.controller == 0));
}

/// Master the Way draws first, then deals damage equal to the new hand size.
#[test]
fn master_the_way_counts_the_hand_after_drawing() {
    let mut g = pod(2);
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let mtw = g.add_card_to_hand(0, catalog::master_the_way());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell { card_id: mtw, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "two Forests plus the draw");
}

/// Divine Congregation counts the target player's creatures (2 life each).
#[test]
fn divine_congregation_counts_the_target_players_creatures() {
    let mut g = pod(3);
    for _ in 0..3 {
        ready(&mut g, 2, catalog::grizzly_bears());
    }
    ready(&mut g, 1, catalog::grizzly_bears());
    let dc = g.add_card_to_hand(0, catalog::divine_congregation());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell { card_id: dc, target: Some(Target::Player(2)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 6);
}

/// Pardic Dragon: an opponent's spell while it's suspended lets that player
/// add a time counter.
#[test]
fn pardic_dragon_opponents_may_delay_it() {
    let mut g = pod(2);
    let pd = g.add_card_to_hand(0, catalog::pardic_dragon());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.perform_action(GameAction::Suspend { card_id: pd }).expect("suspend");
    let time = |g: &GameState| g.exile.iter().find(|c| c.id == pd).map(|c| c.counter_count(CounterType::Time));
    assert_eq!(time(&g), Some(2));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::shock());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("shock");
    let pardic_triggers = g
        .stack
        .iter()
        .filter(|si| matches!(si, crabomination::game::types::StackItem::Trigger { source, .. } if *source == pd))
        .count();
    assert_eq!(pardic_triggers, 1, "one trigger per opponent spell");
    drain_stack(&mut g);
    assert_eq!(time(&g), Some(3));
}

/// Tadeas: a reach attacker untaps and gains skulk for the combat; Tadeas
/// itself has hexproof only while not attacking.
#[test]
fn tadeas_untaps_reach_attackers_with_skulk() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let tadeas = ready(&mut g, 0, catalog::tadeas_juniper_ascendant());
    let spider = ready(&mut g, 0, catalog::sporecap_spider());
    let kw = |g: &GameState, id, k: Keyword| g.computed_permanent(id).unwrap().keywords().contains(&k);
    assert!(kw(&g, tadeas, Keyword::Hexproof));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: spider, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(spider).unwrap().tapped, "untapped");
    assert!(kw(&g, spider, Keyword::Skulk));
}

/// Thing Swing: twice the power, and {2} cheaper aimed from a Hero.
#[test]
fn thing_swing_hits_for_double_power() {
    let mut g = pod(2);
    let hero = ready(&mut g, 0, catalog::wakandan_royal_guard());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let ts = g.add_card_to_hand(0, catalog::thing_swing());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ts,
        target: Some(Target::Permanent(hero)),
        additional_targets: vec![Target::Permanent(wurm)],
        mode: None,
        x_value: None,
    })
    .expect("{1}{G} with a Hero");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_none(), "8 damage kills the 6/4");
}

/// Goblin Javelineer pings its blocker; Tormentor's Helm's wearer, blocked,
/// hits the player it attacked — seat 2 at three seats, not seat 1.
#[test]
fn becomes_blocked_triggers_find_the_blocker_and_defender() {
    let mut g = pod(3);
    let jav = ready(&mut g, 0, catalog::goblin_javelineer());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let helm = ready(&mut g, 0, catalog::tormentors_helm());
    g.battlefield_find_mut(helm).unwrap().attached_to = Some(bear);
    let wall = ready(&mut g, 2, catalog::wall_of_mist());
    let wall2 = ready(&mut g, 2, catalog::glacial_wall());
    let lives = [g.players[1].life, g.players[2].life];
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: jav, target: AttackTarget::Player(2) },
        Attack { attacker: bear, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 2;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, jav), (wall2, bear)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wall).unwrap().damage, 1, "Javelineer's ping");
    assert_eq!([g.players[1].life, g.players[2].life], [lives[0], lives[1] - 1], "the Helm hits the defender");
}

/// Primal Adversary paid twice: the reflexive trigger (CR 603.12) puts two
/// +1/+1 counters on it and targets up to two lands, which stand up as 3/3
/// haste Wolves that are still lands; the third Forest stays a land.
#[test]
fn primal_adversary_wakes_lands_per_payment() {
    use crabomination::card::CardType;
    let mut g = pod(2);
    let lands: Vec<_> = (0..3).map(|_| ready(&mut g, 0, catalog::forest())).collect();
    let pa = g.add_card_to_hand(0, catalog::primal_adversary());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
    ]));
    cast(&mut g, pa);
    assert_eq!(g.battlefield_find(pa).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let wolves = lands
        .iter()
        .filter(|&&l| {
            let cp = g.computed_permanent(l).unwrap();
            cp.card_types().contains(&CardType::Creature) && cp.card_types().contains(&CardType::Land) && cp.power == 3
        })
        .count();
    assert_eq!(wolves, 2);
}

/// Whirlwind stays home; a creature that entered this turn attacks seat 2 at
/// three seats, and seat 2's creature (not seat 1's) can't block.
#[test]
fn whirlwind_locks_a_blocker_of_the_attacked_player() {
    use crabomination::card::Keyword;
    let mut g = pod(3);
    ready(&mut g, 0, catalog::whirlwind_killer_cyclone());
    let fresh = ready(&mut g, 0, catalog::goblin_javelineer());
    g.battlefield_find_mut(fresh).unwrap().entered_turn = Some(g.turn_number);
    let theirs = ready(&mut g, 2, catalog::grizzly_bears());
    let other = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: fresh, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(&mut g);
    let cant = |id| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::CantBlock);
    assert!(cant(theirs));
    assert!(!cant(other));
}

/// CR 702.193 — beside Hulk, Gamma Goliath, Abomination's power-up costs {3}
/// less ({2}{R/G}{R/G}); it adds a counter and fights the opposing bear.
#[test]
fn cr_702_193_hulk_discounts_abominations_power_up() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::hulk_gamma_goliath());
    let abom = ready(&mut g, 0, catalog::abomination_terrifying_titan());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, abom, 0, Some(Target::Permanent(bear))).expect("four mana with Hulk out");
    assert_eq!(g.battlefield_find(abom).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.battlefield_find(bear).is_none(), "fought");
}

/// Alhammarret's named card is locked for its opponents only.
#[test]
fn alhammarret_locks_the_name_for_opponents() {
    let mut g = pod(2);
    let al = ready(&mut g, 0, catalog::alhammarret_high_arbiter());
    g.battlefield_find_mut(al).unwrap().named_card = Some("Shock".into());
    let mine = g.add_card_to_hand(0, catalog::shock());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: mine, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("its controller may cast it");
    drain_stack(&mut g);
    g.priority.player_with_priority = 1;
    let theirs = g.add_card_to_hand(1, catalog::shock());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    assert!(g
        .perform_action(GameAction::CastSpell { card_id: theirs, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .is_err());
}

/// Infinite Coursework taps its host, strips its abilities and keeps it
/// tapped through its controller's untap step.
#[test]
fn infinite_coursework_shuts_a_creature_down() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let ic = g.add_card_to_hand(0, catalog::infinite_coursework());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: ic, target: Some(Target::Permanent(angel)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).unwrap().tapped);
    assert!(!g.computed_permanent(angel).unwrap().keywords().contains(&Keyword::Flying));
    g.active_player_idx = 1;
    g.step = TurnStep::Untap;
    g.do_untap();
    assert!(g.battlefield_find(angel).unwrap().tapped, "doesn't untap");
}

/// Jetfire turns two +1/+1 counters into two {C} for *target player* (CR
/// 605.1a: a targeted mana-adding ability uses the stack) and converts.
#[test]
fn jetfire_cashes_counters_into_mana_and_converts() {
    let mut g = pod(2);
    let jf = ready(&mut g, 0, catalog::jetfire_ingenious_scientist());
    g.battlefield_find_mut(jf).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: jf, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: Some(2), mode: None })
        .expect("activate");
    assert!(!g.stack.is_empty(), "on the stack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].mana_pool.restricted_total(), 2, "the target's spend-restricted colorless");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 0);
    let c = g.battlefield_find(jf).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(c.definition.name, "Jetfire, Air Guardian");
}

/// Graft Surgeon enters with a counter and passes its counters on as it dies;
/// Fairgrounds Trumpeter grows at the end step after a counter landed.
#[test]
fn graft_surgeon_passes_counters_and_trumpeter_notices() {
    let mut g = pod(2);
    let trumpet = ready(&mut g, 0, catalog::fairgrounds_trumpeter());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let gs = g.add_card_to_hand(0, catalog::graft_surgeon());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, gs);
    assert_eq!(g.battlefield_find(gs).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let dies = crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::ExactObjects(vec![gs]) };
    let evs = g.resolve_effect(&dies, &EffectContext::for_spell(1, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let total = |g: &GameState| {
        [bear, trumpet].iter().map(|&id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)).sum::<u32>()
    };
    assert_eq!(total(&g), 1, "the counter moved to one of them");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(total(&g), 2, "the Trumpeter's own end-step counter");
}

/// The Lady of Otaria comes down free by tapping three Dwarves.
#[test]
fn the_lady_of_otaria_taps_three_dwarves() {
    let mut g = pod(2);
    let dwarves: Vec<_> = [catalog::nori_teller_of_tales(), catalog::bombur_gentle_dreamer(), catalog::dain_of_the_ancient_halls()]
        .into_iter()
        .map(|d| ready(&mut g, 0, d))
        .collect();
    let lady = g.add_card_to_hand(0, catalog::the_lady_of_otaria());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: lady,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(lady).is_some());
    assert!(dwarves.iter().all(|&d| g.battlefield_find(d).unwrap().tapped));
}

/// Predictive Preparations cast with flashback asks for its second target
/// the way the hand cast does: the flashback path never posed slot 1, so a
/// player's flashback could only name one creature (a play-test report).
#[test]
fn predictive_preparations_flashback_asks_for_the_second_creature() {
    use crabomination::decision::Decision;
    use crabomination::game::types::Target;
    use crabomination::mana::Color;
    let mut g = pod(2);
    g.players[0].wants_ui = true;
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let prep = g.add_card_to_graveyard(0, catalog::predictive_preparations());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFlashback {
        card_id: prep,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("the flashback asks for the second target");
    let pd = g.pending_decision.as_ref().expect("slot 1 is posed");
    assert!(
        matches!(&pd.decision, Decision::ChooseTarget { optional: true, legal, .. } if legal.contains(&Target::Permanent(b))),
        "the other creature is offered: {:?}",
        pd.decision
    );
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Target(Target::Permanent(b))))
        .expect("the pick casts the flashback");
    drain_stack(&mut g);
    for id in [a, b] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }
    assert!(g.exile.iter().any(|c| c.id == prep), "flashback exiles the card");
}
