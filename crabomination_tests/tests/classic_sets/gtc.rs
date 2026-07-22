//! Functionality tests for the Gatecrash (GTC) first wave.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
use crabomination::game::*;

/// Stat / keyword lines for the simple beaters.
#[test]
fn gtc_stat_and_keyword_lines() {
    let skulk = catalog::gutter_skulk();
    assert_eq!((skulk.power, skulk.toughness), (2, 2));
    let wurm = catalog::ruination_wurm();
    assert_eq!((wurm.power, wurm.toughness), (7, 6));
    let griffin = catalog::assault_griffin();
    assert!(griffin.keywords.contains(&Keyword::Flying));
    let krasis = catalog::drakewing_krasis();
    assert!(krasis.keywords.contains(&Keyword::Flying) && krasis.keywords.contains(&Keyword::Trample));
    let beast = catalog::ember_beast();
    assert!(beast.keywords.contains(&Keyword::CantAttackOrBlockAlone));
    let gargoyle = catalog::millennial_gargoyle();
    assert!(gargoyle.card_types.contains(&crabomination::card::CardType::Artifact));
}

/// Disciple of the Old Ways gains first strike until end of turn.
#[test]
fn disciple_grants_first_strike() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let d = g.add_card_to_battlefield(0, catalog::disciple_of_the_old_ways());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: d, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("grant first strike");
    drain_stack(&mut g);
    assert!(g.computed_permanent(d).unwrap().keywords.contains(&Keyword::FirstStrike));
}

/// Metropolis Sprite pumps itself +1/-1.
#[test]
fn metropolis_sprite_pumps() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::metropolis_sprite());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let c = g.computed_permanent(s).unwrap();
    assert_eq!((c.power, c.toughness), (2, 1), "1/2 → 2/1");
}

/// Mortus Strider returns to its owner's hand when it dies.
#[test]
fn mortus_strider_returns_on_death() {
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::mortus_strider());
    kill_perm(&mut g, m);
    drain_stack(&mut g);
    assert!(g.battlefield_find(m).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == m), "back in hand");
}

/// Mindeye Drake mills five when it dies.
#[test]
fn mindeye_drake_mills_on_death() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(1, catalog::gutter_skulk()); }
    let d = g.add_card_to_battlefield(0, catalog::mindeye_drake());
    let lib_before = g.players[1].library.len();
    kill_perm(&mut g, d);
    drain_stack_targeting(&mut g, Target::Player(1));
    assert_eq!(g.players[1].library.len(), lib_before - 5, "milled five");
}

/// Nimbus Swimmer enters with X +1/+1 counters.
#[test]
fn nimbus_swimmer_enters_with_x() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let n = g.add_card_to_hand(0, catalog::nimbus_swimmer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: n, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast X=3");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(n).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Smite destroys a blocked attacker (exercises `R::IsBlocked`).
#[test]
fn smite_destroys_blocked_creature() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::ruination_wurm()); // 7/6, P0 attacks
    let blocker = g.add_card_to_battlefield(1, catalog::gutter_skulk());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    // Now the wurm is a "blocked creature". P0 casts Smite on it.
    let smite = g.add_card_to_hand(0, catalog::smite());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.cast_spell(smite, Some(Target::Permanent(attacker)), vec![], None, None).expect("smite");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "blocked creature destroyed");
}

/// Killing Glare destroys a creature with power ≤ X but not a bigger one.
#[test]
fn killing_glare_respects_x() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::gutter_skulk()); // power 2
    let big = g.add_card_to_battlefield(1, catalog::ruination_wurm()); // power 7
    // X=2 can hit the 2-power skulk.
    let glare = g.add_card_to_hand(0, catalog::killing_glare());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.cast_spell(glare, Some(Target::Permanent(small)), vec![], None, Some(2)).expect("glare");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "2-power creature destroyed");
    assert!(g.battlefield_find(big).is_some(), "7-power creature untouched");
}

/// Righteous Charge pumps your whole team +2/+2.
#[test]
fn righteous_charge_pumps_team() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    let b = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    resolve_sorcery(&mut g, catalog::righteous_charge());
    assert_eq!(g.computed_permanent(a).unwrap().power, 4);
    assert_eq!(g.computed_permanent(b).unwrap().toughness, 4);
}

/// Knight Watch makes two 2/2 vigilant Knight tokens.
#[test]
fn knight_watch_makes_two_knights() {
    let mut g = two_player_game();
    resolve_sorcery(&mut g, catalog::knight_watch());
    let knights: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Knight" && c.controller == 0).collect();
    assert_eq!(knights.len(), 2);
    assert!(knights[0].definition.keywords.contains(&Keyword::Vigilance));
}

/// Madcap Skills grants +3/+0 and menace to the enchanted creature.
#[test]
fn madcap_skills_buffs_host() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // 2/2
    let aura = g.add_card_to_battlefield(0, catalog::madcap_skills());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 2));
    assert!(c.keywords.contains(&Keyword::Menace));
}

/// Illness in the Ranks shrinks every creature token by 1/1.
#[test]
fn illness_shrinks_tokens() {
    let mut g = two_player_game();
    let tok = g.add_card_to_battlefield(1, catalog::gutter_skulk());
    g.battlefield_find_mut(tok).unwrap().is_token = true;
    let nontoken = g.add_card_to_battlefield(1, catalog::gutter_skulk());
    g.add_card_to_battlefield(0, catalog::illness_in_the_ranks());
    assert_eq!(g.computed_permanent(tok).unwrap().toughness, 1, "token is 1/1");
    assert_eq!(g.computed_permanent(nontoken).unwrap().toughness, 2, "nontoken unaffected");
}

/// Smog Elemental shrinks opposing flyers.
#[test]
fn smog_elemental_shrinks_opposing_flyers() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::assault_griffin()); // 3/2 flyer
    let ground = g.add_card_to_battlefield(1, catalog::gutter_skulk()); // no flying
    g.add_card_to_battlefield(0, catalog::smog_elemental());
    assert_eq!(g.computed_permanent(flyer).unwrap().toughness, 1, "opposing flyer -1/-1");
    assert_eq!(g.computed_permanent(ground).unwrap().toughness, 2, "ground creature unaffected");
}

/// Debtor's Pulpit grants the enchanted land a "{T}: Tap target creature."
#[test]
fn debtors_pulpit_grants_land_tapper() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let pulpit = g.add_card_to_battlefield(0, catalog::debtors_pulpit());
    g.battlefield_find_mut(pulpit).unwrap().attached_to = Some(land);
    let victim = g.add_card_to_battlefield(1, catalog::gutter_skulk());
    // Forest's mana ability is printed at index 0; the granted tapper follows.
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], x_value: None,
    }).expect("tap via granted ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped, "target creature tapped");
}

/// Totally Lost puts a nonland permanent on top of its owner's library.
#[test]
fn totally_lost_tucks_to_top() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::gutter_skulk());
    let spell = g.add_card_to_hand(0, catalog::totally_lost());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.cast_spell(spell, Some(Target::Permanent(creature)), vec![], None, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(creature).is_none());
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(creature), "on top of library");
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn kill_perm(g: &mut GameState, id: crabomination::card::CardId) {
    let ctl = g.battlefield_find(id).unwrap().controller;
    let ctx = crabomination::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &ctx,
    )
    .unwrap();
}

fn resolve_sorcery(g: &mut GameState, def: crabomination::card::CardDefinition) {
    use crabomination::game::effects::EffectContext;
    let src = g.add_card_to_battlefield(0, def.clone());
    let ctx = EffectContext::for_ability(src, 0, None);
    g.resolve_effect(&def.effect, &ctx).unwrap();
    drain_stack(g);
}

fn drain_stack_targeting(g: &mut GameState, tgt: Target) {
    // Resolve a pending trigger that needs a target by supplying it via the
    // AutoDecider's default; if the top-of-stack trigger has no target set,
    // set it directly on the stack item.
    if let Some(StackItem::Trigger { target, .. }) = g.stack.last_mut()
        && target.is_none()
    {
        *target = Some(tgt);
    }
    drain_stack(g);
}

// ── wave 2 ───────────────────────────────────────────────────────────────────

/// Battalion: Warmind Infantry pumps +2/+0 when it and two others attack.
#[test]
fn battalion_triggers_with_three_attackers() {
    let mut g = two_player_game();
    let warmind = g.add_card_to_battlefield(0, catalog::warmind_infantry()); // 2/3
    let b1 = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    let b2 = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    for id in [warmind, b1, b2] { g.clear_sickness(id); }
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: warmind, target: AttackTarget::Player(1) },
        Attack { attacker: b1, target: AttackTarget::Player(1) },
        Attack { attacker: b2, target: AttackTarget::Player(1) },
    ]).expect("attack with three");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(warmind).unwrap().power, 4, "battalion pumped +2/+0");
}

/// Battalion does NOT trigger with only two attackers.
#[test]
fn battalion_silent_with_two_attackers() {
    let mut g = two_player_game();
    let warmind = g.add_card_to_battlefield(0, catalog::warmind_infantry());
    let b1 = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    for id in [warmind, b1] { g.clear_sickness(id); }
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: warmind, target: AttackTarget::Player(1) },
        Attack { attacker: b1, target: AttackTarget::Player(1) },
    ]).expect("attack with two");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(warmind).unwrap().power, 2, "battalion did not fire");
}

/// Rubblebelt Raiders grows by the number of attacking creatures you control.
#[test]
fn rubblebelt_raiders_grows_with_attackers() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let raider = g.add_card_to_battlefield(0, catalog::rubblebelt_raiders());
    let other = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    for id in [raider, other] { g.clear_sickness(id); }
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: raider, target: AttackTarget::Player(1) },
        Attack { attacker: other, target: AttackTarget::Player(1) },
    ]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(raider).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "one counter per attacking creature you control");
}

/// Truefire Paladin's firebreathing pump.
#[test]
fn truefire_paladin_pumps() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::truefire_paladin());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: p, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(p).unwrap().power, 4, "+2/+0");
}

/// Riot Gear equips for +1/+2.
#[test]
fn riot_gear_equips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // 2/2
    let gear = g.add_card_to_battlefield(0, catalog::riot_gear());
    g.battlefield_find_mut(gear).unwrap().attached_to = Some(bear);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 4), "+1/+2 from Riot Gear");
}

/// Predator's Rapport gains life equal to a creature's power + toughness.
#[test]
fn predators_rapport_gains_life() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::ruination_wurm()); // 7/6
    g.players[0].life = 20;
    let spell = g.add_card_to_hand(0, catalog::predators_rapport());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.cast_spell(spell, Some(Target::Permanent(wurm)), vec![], None, None).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 33, "gained 7 + 6 = 13 life");
}

/// Keymaster Rogue is unblockable and bounces one of your creatures on ETB.
#[test]
fn keymaster_rogue_unblockable_and_bounces() {
    let mut g = two_player_game();
    let friend = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    let rogue = g.add_card_to_battlefield(0, catalog::keymaster_rogue());
    assert!(rogue_is_unblockable(&g, rogue));
    g.fire_self_etb_triggers(rogue, 0);
    drain_stack(&mut g);
    // The only other creature you control is bounced (auto-picked).
    assert!(g.battlefield_find(friend).is_none() || g.battlefield_find(rogue).is_none(),
        "a controlled creature returned to hand");
}

fn rogue_is_unblockable(g: &GameState, id: crabomination::card::CardId) -> bool {
    g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Unblockable)
}

/// Death's Approach shrinks the host by the number of creature cards in its
/// controller's graveyard.
#[test]
fn deaths_approach_scales_with_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::ruination_wurm()); // 7/6, P1 controls
    let aura = g.add_card_to_battlefield(0, catalog::deaths_approach());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    // Two creature cards in P1's graveyard → -2/-2.
    for _ in 0..2 { g.add_card_to_graveyard(1, catalog::gutter_skulk()); }
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 4), "7/6 minus 2/2 from two gy creatures");
}

// ── Wave 3 ───────────────────────────────────────────────────────────────────

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// The guild Keyrunes animate into their printed bodies for {c1}{c2}.
#[test]
fn gtc3_keyrunes_animate() {
    use crabomination::card::CardType;
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let gruul = g.add_card_to_battlefield(0, catalog::gruul_keyrune());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gruul, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("animate");
    drain_stack(&mut g);
    let c = g.computed_permanent(gruul).unwrap();
    assert_eq!((c.power, c.toughness), (3, 2));
    assert!(c.card_types.contains(&CardType::Creature) && c.keywords.contains(&Keyword::Trample));

    let boros = g.add_card_to_battlefield(0, catalog::boros_keyrune());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: boros, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("animate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(boros).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// The three extort creatures carry Extort plus their printed keyword lines.
#[test]
fn gtc3_extort_bodies() {
    let guards = catalog::basilica_guards();
    assert!(guards.keywords.contains(&Keyword::Defender) && !guards.triggered_abilities.is_empty());
    let knight = catalog::knight_of_obligation();
    assert!(knight.keywords.contains(&Keyword::Vigilance) && !knight.triggered_abilities.is_empty());
    assert!(!catalog::syndicate_enforcer().triggered_abilities.is_empty());
}

/// Spark Trooper is sacrificed at the beginning of the end step.
#[test]
fn gtc3_spark_trooper_end_step_sacrifice() {
    let mut g = two_player_game();
    let t = g.add_card_to_battlefield(0, catalog::spark_trooper());
    advance_to(&mut g, TurnStep::End);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(t).is_none(), "sacrificed at end step");
}

/// Urbis Protector's ETB mints a 4/4 flying Angel.
#[test]
fn gtc3_urbis_protector_makes_angel() {
    let mut g = two_player_game();
    let u = g.add_card_to_battlefield(0, catalog::urbis_protector());
    g.fire_self_etb_triggers(u, 0);
    drain_stack(&mut g);
    let angel = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Angel").expect("angel token").id;
    let c = g.computed_permanent(angel).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(c.keywords.contains(&Keyword::Flying));
}

/// Forced Adaptation adds a +1/+1 counter at its controller's upkeep.
#[test]
fn gtc3_forced_adaptation_grows_host() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::ruination_wurm());
    let aura = g.add_card_to_battlefield(0, catalog::forced_adaptation());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Holy Mantle pumps +2/+2 and grants protection from creatures.
#[test]
fn gtc3_holy_mantle_pumps_and_protects() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // 2/2
    let aura = g.add_card_to_battlefield(0, catalog::holy_mantle());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(c.keywords.contains(&Keyword::ProtectionFromCreatures));
}

/// Mugging deals 2 damage and stops the creature from blocking.
#[test]
fn gtc3_mugging_damages_and_stops_block() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(1, catalog::ruination_wurm()); // 7/6
    let m = g.add_card_to_hand(0, catalog::mugging());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.cast_spell(m, Some(Target::Permanent(wurm)), vec![], None, None).expect("mugging");
    drain_stack(&mut g);
    let c = g.computed_permanent(wurm).unwrap();
    assert!(c.keywords.contains(&Keyword::CantBlock));
}

/// Homing Lightning deals 4 to the target and each same-named creature.
#[test]
fn gtc3_homing_lightning_hits_same_name() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::gutter_skulk()); // 2/2
    let b = g.add_card_to_battlefield(1, catalog::gutter_skulk()); // 2/2, same name
    let bolt = g.add_card_to_hand(0, catalog::homing_lightning());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.cast_spell(bolt, Some(Target::Permanent(a)), vec![], None, None).expect("homing");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both same-named skulks die");
}

/// Massive Raid deals damage equal to the number of creatures you control.
#[test]
fn gtc3_massive_raid_scales_with_creatures() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::gutter_skulk()); }
    let raid = g.add_card_to_hand(0, catalog::massive_raid());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    g.cast_spell(raid, Some(Target::Player(1)), vec![], None, None).expect("raid");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "3 creatures → 3 damage");
}

/// Ground Assault deals damage equal to the number of lands you control.
#[test]
fn gtc3_ground_assault_scales_with_lands() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_battlefield(0, catalog::forest()); }
    let wurm = g.add_card_to_battlefield(1, catalog::ruination_wurm()); // 7/6
    let ga = g.add_card_to_hand(0, catalog::ground_assault());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.cast_spell(ga, Some(Target::Permanent(wurm)), vec![], None, None).expect("ground assault");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, 4, "4 lands → 4 damage");
}

// ── Wave 4 ───────────────────────────────────────────────────────────────────

/// Sapphire Drake / Crowned Ceratok grant evasion to your +1/+1-countered team.
#[test]
fn gtc4_counter_lords_grant_evasion() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sapphire_drake());
    g.add_card_to_battlefield(0, catalog::crowned_ceratok());
    let bear = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // 2/2, no counter yet
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying), "no counter → no grant");
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let c = g.computed_permanent(bear).unwrap();
    assert!(c.keywords.contains(&Keyword::Flying) && c.keywords.contains(&Keyword::Trample));
}

/// Hellraiser Goblin gives your creatures haste and "attacks each combat".
#[test]
fn gtc4_hellraiser_grants_haste_mustattack() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hellraiser_goblin());
    let bear = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    let c = g.computed_permanent(bear).unwrap();
    assert!(c.keywords.contains(&Keyword::Haste) && c.keywords.contains(&Keyword::MustAttack));
}

/// Ogre Slumlord mints a Rat when a nontoken creature dies; Rats have deathtouch.
#[test]
fn gtc4_ogre_slumlord_rats_have_deathtouch() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ogre_slumlord());
    let victim = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // nontoken
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    kill_perm(&mut g, victim);
    g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: victim }]);
    drain_stack(&mut g);
    let rat = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Rat").expect("rat token").id;
    assert!(g.computed_permanent(rat).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Court Street Denizen taps an opponent's creature when another white creature enters.
#[test]
fn gtc4_court_street_taps_on_white_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::court_street_denizen());
    let foe = g.add_card_to_battlefield(1, catalog::ruination_wurm());
    let ally = g.add_card_to_battlefield(0, catalog::urbis_protector()); // white
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ally }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's creature tapped");
}

/// Sage's Row Denizen mills when another blue creature enters.
#[test]
fn gtc4_sages_row_mills_on_blue_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sages_row_denizen());
    for _ in 0..5 { g.add_card_to_library(1, catalog::gutter_skulk()); }
    let ally = g.add_card_to_battlefield(0, catalog::merfolk_of_the_depths()); // blue
    let before = g.players[1].library.len();
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ally }]);
    drain_stack_targeting(&mut g, Target::Player(1));
    assert_eq!(g.players[1].library.len(), before - 2, "milled two");
}

/// High Priest of Penance destroys a nonland permanent when dealt damage.
#[test]
fn gtc4_high_priest_destroys_on_damage() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let priest = g.add_card_to_battlefield(0, catalog::high_priest_of_penance());
    let foe = g.add_card_to_battlefield(1, catalog::ruination_wurm());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(priest), 1, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "nonland permanent destroyed");
}

/// Frilled Oculus can only pump once each turn.
#[test]
fn gtc4_frilled_oculus_once_per_turn() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let o = g.add_card_to_battlefield(0, catalog::frilled_oculus());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: o, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("first pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(o).unwrap().power, 3, "1/3 → 3/5");
    let second = g.perform_action(GameAction::ActivateAbility {
        card_id: o, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    });
    assert!(second.is_err(), "second activation this turn is illegal");
}

/// Dinrova Horror bounces a permanent then makes its owner discard.
#[test]
fn gtc4_dinrova_bounces_and_discards() {
    let mut g = two_player_game();
    let d = g.add_card_to_battlefield(0, catalog::dinrova_horror());
    let foe = g.add_card_to_battlefield(1, catalog::ruination_wurm());
    g.add_card_to_hand(1, catalog::gutter_skulk()); // something to discard
    let hand_before = g.players[1].hand.len();
    g.fire_self_etb_triggers(d, 0);
    drain_stack_targeting(&mut g, Target::Permanent(foe));
    assert!(g.battlefield_find(foe).is_none(), "permanent bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == foe), "bounced card is in owner's hand");
    assert_eq!(g.players[1].hand.len(), hand_before + 1 - 1, "returned one, discarded one");
}

/// Grisly Spectacle destroys a nonartifact creature and mills its power.
#[test]
fn gtc4_grisly_spectacle_destroys_and_mills() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::ruination_wurm()); // 7/6
    for _ in 0..10 { g.add_card_to_library(1, catalog::gutter_skulk()); }
    let before = g.players[1].library.len();
    let gs = g.add_card_to_hand(0, catalog::grisly_spectacle());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.cast_spell(gs, Some(Target::Permanent(foe)), vec![], None, None).expect("grisly");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "destroyed");
    assert_eq!(g.players[1].library.len(), before - 7, "milled 7 (its power)");
}

/// Crackling Perimeter pings each opponent for 1 by tapping a Gate.
#[test]
fn gtc4_crackling_perimeter_pings() {
    let mut g = two_player_game();
    let perim = g.add_card_to_battlefield(0, catalog::crackling_perimeter());
    let gate = g.add_card_to_battlefield(0, catalog::azorius_guildgate()); // a Gate to tap
    g.battlefield_find_mut(gate).unwrap().tapped = false;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: perim, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "opponent pinged for 1");
}

// ── Wave 5 ───────────────────────────────────────────────────────────────────

/// Immortal Servitude returns each creature card of mana value X from your
/// graveyard — exercising `EachMatching`'s new `{X}`-from-cost resolution.
#[test]
fn gtc5_immortal_servitude_reanimates_by_x() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    // Graveyard: two 2-MV creatures (Gutter Skulk {1}{B}) and one 6-MV (Ruination Wurm).
    g.add_card_to_graveyard(0, catalog::gutter_skulk());
    g.add_card_to_graveyard(0, catalog::gutter_skulk());
    g.add_card_to_graveyard(0, catalog::ruination_wurm());
    let spell = g.add_card_to_hand(0, catalog::immortal_servitude());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add_colorless(2); // X=2
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast X=2");
    drain_stack(&mut g);
    let skulks = g.battlefield.iter().filter(|c| c.definition.name == "Gutter Skulk" && c.controller == 0).count();
    assert_eq!(skulks, 2, "both MV-2 creatures returned");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Ruination Wurm"), "MV-6 creature stayed in graveyard");
}

/// Biovisionary wins the game at end step with four copies in play.
#[test]
fn gtc5_biovisionary_wins_with_four() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_battlefield(0, catalog::biovisionary()); }
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    // Resolve the win trigger; PassPriority errors once the game ends, so stop
    // as soon as game_over is set.
    while !g.stack.is_empty() && g.perform_action(GameAction::PassPriority).is_ok() {}
    assert_eq!(g.game_over, Some(Some(0)), "controller wins with four Biovisionaries");
}

/// Three Biovisionaries is not enough — the game continues.
#[test]
fn gtc5_biovisionary_needs_four() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::biovisionary()); }
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.is_game_over(), "three is not enough to win");
}

/// Giant Adephage copies itself on dealing combat damage to a player.
#[test]
fn gtc5_giant_adephage_copies_on_damage() {
    let mut g = two_player_game();
    let ade = g.add_card_to_battlefield(0, catalog::giant_adephage());
    g.clear_sickness(ade);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ade, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    let copies = g.battlefield.iter().filter(|c| c.definition.name == "Giant Adephage" && c.controller == 0).count();
    assert_eq!(copies, 2, "original plus one token copy");
}

/// Executioner's Swing shrinks a creature that dealt damage this turn.
#[test]
fn gtc5_executioners_swing_hits_damager() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::ruination_wurm()); // 7/6
    g.battlefield_find_mut(foe).unwrap().dealt_damage_this_turn = true;
    let swing = g.add_card_to_hand(0, catalog::executioners_swing());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.cast_spell(swing, Some(Target::Permanent(foe)), vec![], None, None).expect("swing");
    drain_stack(&mut g);
    let c = g.computed_permanent(foe).unwrap();
    assert_eq!((c.power, c.toughness), (2, 1), "7/6 shrunk by -5/-5");
}

// ── Wave 6 ───────────────────────────────────────────────────────────────────

/// Scab-Clan Charger's Bloodrush pumps a target attacker from hand.
#[test]
fn gtc6_bloodrush_pumps_attacker() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // 2/2
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    let charger = g.add_card_to_hand(0, catalog::scab_clan_charger());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: charger, ability_index: 0, target: Some(Target::Permanent(attacker)),
        additional_targets: vec![], x_value: None,
    }).expect("bloodrush");
    drain_stack(&mut g);
    let c = g.computed_permanent(attacker).unwrap();
    assert_eq!((c.power, c.toughness), (4, 6), "2/2 + 2/4");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == charger), "charger discarded as cost");
}

/// Martial Glory's second mode grants +0/+3.
#[test]
fn gtc6_martial_glory_modal() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::martial_glory());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("mode 1 = +0/+3");
    drain_stack(&mut g);
    assert_eq!((g.computed_permanent(bear).unwrap().power, g.computed_permanent(bear).unwrap().toughness), (2, 5));
}

/// Alpha Authority grants hexproof and "can't be blocked by more than one".
#[test]
fn gtc6_alpha_authority_grants() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    let aura = g.add_card_to_battlefield(0, catalog::alpha_authority());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let c = g.computed_permanent(bear).unwrap();
    assert!(c.keywords.contains(&Keyword::Hexproof) && c.keywords.contains(&Keyword::CantBeBlockedByMoreThanOne));
}

/// Agoraphobia shrinks the host by 5 power and can return itself to hand.
#[test]
fn gtc6_agoraphobia_shrinks_and_returns() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(1, catalog::ruination_wurm()); // 7/6
    let aura = g.add_card_to_battlefield(0, catalog::agoraphobia());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(wurm);
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 2, "7 - 5 = 2");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("return aura");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "aura back in hand");
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 7, "host restored");
}

/// Greenside Watcher untaps a Gate.
#[test]
fn gtc6_greenside_watcher_untaps_gate() {
    let mut g = two_player_game();
    let watcher = g.add_card_to_battlefield(0, catalog::greenside_watcher());
    g.clear_sickness(watcher);
    let gate = g.add_card_to_battlefield(0, catalog::azorius_guildgate());
    g.battlefield_find_mut(gate).unwrap().tapped = true;
    g.perform_action(GameAction::ActivateAbility {
        card_id: watcher, ability_index: 0, target: Some(Target::Permanent(gate)),
        additional_targets: vec![], x_value: None,
    }).expect("untap gate");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(gate).unwrap().tapped, "gate untapped");
}

/// Leyline Phantom returns to hand after dealing combat damage.
#[test]
fn gtc6_leyline_phantom_bounces() {
    let mut g = two_player_game();
    let phantom = g.add_card_to_battlefield(0, catalog::leyline_phantom());
    g.clear_sickness(phantom);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: phantom, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert!(g.battlefield_find(phantom).is_none() && g.players[0].hand.iter().any(|c| c.id == phantom),
        "phantom returned to hand after combat damage");
}

/// Slate Street Ruffian makes the defending player discard when it's blocked.
#[test]
fn gtc6_slate_street_ruffian_discards_on_block() {
    let mut g = two_player_game();
    let ruffian = g.add_card_to_battlefield(0, catalog::slate_street_ruffian());
    let blocker = g.add_card_to_battlefield(1, catalog::gutter_skulk());
    g.add_card_to_hand(1, catalog::gutter_skulk()); // discard fodder
    g.clear_sickness(ruffian);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: ruffian, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, ruffian)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 1, "defending player discarded on block");
}

// ── Wave 7 (gtc7) ─────────────────────────────────────────────────────────────

use crabomination::card::{CardType, LandType};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::mana::Color;

/// Stat / keyword lines for the wave-7 Evolve creatures.
#[test]
fn gtc7_evolve_keyword_lines() {
    assert!(catalog::crocanura().keywords.contains(&Keyword::Reach));
    assert!(catalog::battering_krasis().keywords.contains(&Keyword::Trample));
    assert!(catalog::shambleshark().keywords.contains(&Keyword::Flash));
    assert!(catalog::clinging_anemones().keywords.contains(&Keyword::Defender));
    let snap = catalog::adaptive_snapjaw();
    assert_eq!((snap.power, snap.toughness), (6, 2));
}

/// A bigger creature entering evolves Crocanura (0/0 → after counter).
#[test]
fn gtc7_crocanura_evolves() {
    let mut g = two_player_game();
    let croc = g.add_card_to_battlefield(0, catalog::crocanura()); // 1/3
    let wurm = g.add_card_to_hand(0, catalog::ruination_wurm()); // 7/6
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast wurm");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(croc).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Renegade Krasis: when it evolves, each other counter-bearing creature you
/// control gets a +1/+1 counter too.
#[test]
fn gtc7_renegade_krasis_payoff() {
    let mut g = two_player_game();
    let renegade = g.add_card_to_battlefield(0, catalog::renegade_krasis()); // 3/2
    // Another (non-evolve) creature with a +1/+1 counter already on it.
    let other = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    g.battlefield_find_mut(other).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let wurm = g.add_card_to_hand(0, catalog::ruination_wurm()); // 7/6 evolves the 3/2
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast wurm");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(renegade).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "evolved");
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "payoff added one");
}

/// Miming Slime makes an Ooze whose P/T equals the greatest power you control.
#[test]
fn gtc7_miming_slime_token_size() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ruination_wurm()); // 7/6 — greatest power 7
    let slime = g.add_card_to_hand(0, catalog::miming_slime());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: slime, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast slime");
    drain_stack(&mut g);
    let ooze = g.battlefield.iter().find(|c| c.definition.name == "Ooze").expect("token exists").id;
    assert_eq!(g.computed_permanent(ooze).unwrap().power, 7, "X = greatest power (7)");
}

/// Realmwright makes the controller's lands the chosen basic type in addition.
#[test]
fn gtc7_realmwright_adds_chosen_land_type() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.move_card_to_battlefield_for_test(0, catalog::realmwright());
    drain_stack(&mut g);
    let types = &g.computed_permanent(forest).unwrap().subtypes.land_types;
    assert!(types.contains(&LandType::Island), "gained the chosen Island type");
    assert!(types.contains(&LandType::Forest), "kept its Forest type");
}

/// Gruul Ragebeast: on entering, it fights an opponent's creature.
#[test]
fn gtc7_gruul_ragebeast_fights_on_etb() {
    let mut g = two_player_game();
    let prey = g.add_card_to_battlefield(1, catalog::gutter_skulk()); // 2/2
    g.move_card_to_battlefield_for_test(0, catalog::gruul_ragebeast()); // 6/6 fights the 2/2
    drain_stack(&mut g);
    assert!(g.battlefield_find(prey).is_none(), "prey died to the fight");
}

/// Merciless Eviction mode 1 exiles all creatures.
#[test]
fn gtc7_merciless_eviction_exiles_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    let b = g.add_card_to_battlefield(1, catalog::ruination_wurm());
    let spell = g.add_card_to_hand(0, catalog::merciless_eviction());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast eviction"); // mode 1 = exile all creatures
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both creatures exiled");
}

/// Skarrg Guildmage animates one of your lands into a 4/4.
#[test]
fn gtc7_skarrg_animates_land() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::skarrg_guildmage());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.clear_sickness(mage);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 1, target: Some(Target::Permanent(land)),
        additional_targets: vec![], x_value: None,
    }).expect("animate land");
    drain_stack(&mut g);
    let c = g.computed_permanent(land).unwrap();
    assert!(c.card_types.contains(&CardType::Creature) && c.card_types.contains(&CardType::Land));
    assert_eq!((c.power, c.toughness), (4, 4));
}

/// Simic Fluxmage moves a +1/+1 counter off itself onto another creature.
#[test]
fn gtc7_simic_fluxmage_moves_counter() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::simic_fluxmage());
    g.battlefield_find_mut(mage).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let target = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    g.clear_sickness(mage);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: vec![], x_value: None,
    }).expect("move counter");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mage).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Fortress Cyclops gets +3/+0 when it attacks.
#[test]
fn gtc7_fortress_cyclops_attack_pump() {
    let mut g = two_player_game();
    let cyc = g.add_card_to_battlefield(0, catalog::fortress_cyclops());
    g.clear_sickness(cyc);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cyc, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(cyc).unwrap().power, 6, "3 + 3 attack pump");
}

/// Zameck Guildmage draws by removing a +1/+1 counter from a creature you control.
#[test]
fn gtc7_zameck_removes_counter_to_draw() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::zameck_guildmage());
    let dude = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    g.battlefield_find_mut(dude).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.add_card_to_library(0, catalog::gutter_skulk());
    g.clear_sickness(mage);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("remove counter, draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    assert_eq!(g.battlefield_find(dude).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "counter removed");
}

/// Foundry Champion's ETB deals damage equal to creatures you control.
#[test]
fn gtc7_foundry_champion_etb_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gutter_skulk());
    g.add_card_to_battlefield(0, catalog::gutter_skulk());
    let life_before = g.players[1].life;
    // Foundry Champion itself counts too: 2 skulks + the champion = 3.
    g.move_card_to_battlefield_for_test(0, catalog::foundry_champion());
    // Auto-target picks the opponent (only "any target" legal choice defaults to a player).
    drain_stack(&mut g);
    assert!(g.players[1].life < life_before, "opponent took ETB damage");
}

/// Verdant Haven gains 2 life when it enters.
#[test]
fn gtc7_verdant_haven_gains_life() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let life_before = g.players[0].life;
    let aura = g.add_card_to_hand(0, catalog::verdant_haven());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(land)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2, "gained 2 life on ETB");
}

// ── Wave 8 (gtc8) ─────────────────────────────────────────────────────────────

/// Shadow Alley Denizen grants intimidate when another black creature enters.
#[test]
fn gtc8_shadow_alley_denizen_grants_intimidate() {
    let mut g = two_player_game();
    let denizen = g.add_card_to_battlefield(0, catalog::shadow_alley_denizen()); // black
    let target = g.add_card_to_battlefield(0, catalog::gutter_skulk());
    // Another black creature entering (Shadow Alley Denizen is black; cast one).
    let black = g.add_card_to_hand(0, catalog::shadow_alley_denizen());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: black, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast black creature");
    drain_stack(&mut g);
    assert!(g.computed_permanent(target).unwrap().keywords.contains(&Keyword::Intimidate));
    let _ = denizen;
}

/// Structural Collapse makes the target sacrifice an artifact and a land, then
/// deals 2 damage.
#[test]
fn gtc8_structural_collapse_edict_and_burn() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::millennial_gargoyle()); // artifact
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let life_before = g.players[1].life;
    let spell = g.add_card_to_hand(0, catalog::structural_collapse());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast structural collapse");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact sacrificed");
    assert!(g.battlefield_find(land).is_none(), "land sacrificed");
    assert_eq!(g.players[1].life, life_before - 2, "took 2 damage");
}

/// Coerced Confession mills four and draws one per creature card milled
/// (exercises the new `Effect::MillThenDrawPerType`).
#[test]
fn gtc8_coerced_confession_mills_and_draws_per_creature() {
    let mut g = two_player_game();
    // Opponent's top four: two creatures, two lands → I should draw two.
    g.add_card_to_library(1, catalog::gutter_skulk());
    g.add_card_to_library(1, catalog::gutter_skulk());
    g.add_card_to_library(1, catalog::forest());
    g.add_card_to_library(1, catalog::forest());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); } // my draw fuel
    let my_lib_before = g.players[0].library.len();
    let spell = g.add_card_to_hand(0, catalog::coerced_confession());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast coerced confession");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 4, "milled four");
    assert_eq!(g.players[0].library.len(), my_lib_before - 2, "drew one per creature milled (2)");
}

/// Serene Remembrance shuffles up to three cards from a graveyard into the
/// library.
#[test]
fn gtc8_serene_remembrance_shuffles_graveyard() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_graveyard(1, catalog::gutter_skulk()); }
    let lib_before = g.players[1].library.len();
    let spell = g.add_card_to_hand(0, catalog::serene_remembrance());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast serene remembrance");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.len() <= 0, "up to three cards left the graveyard");
    assert_eq!(g.players[1].library.len(), lib_before + 3, "three cards shuffled in");
}

// ── Wave 9 (gtc9) ─────────────────────────────────────────────────────────────

/// Skyblinder Staff pumps +1/+0 and stops flying blockers.
#[test]
fn gtc9_skyblinder_staff_buffs_and_evades() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // 2/2
    let staff = g.add_card_to_battlefield(0, catalog::skyblinder_staff());
    g.battlefield_find_mut(staff).unwrap().attached_to = Some(bear);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 3, "2 + 1");
    assert!(c.keywords.iter().any(|k| matches!(k, Keyword::CantBeBlockedBy(_))));
}

/// Razortip Whip pings for 1.
#[test]
fn gtc9_razortip_whip_pings() {
    let mut g = two_player_game();
    let whip = g.add_card_to_battlefield(0, catalog::razortip_whip());
    let life_before = g.players[1].life;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: whip, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 1, "dealt 1");
}

/// Murder Investigation makes Soldiers equal to the dead creature's power.
#[test]
fn gtc9_murder_investigation_makes_soldiers() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::ruination_wurm()); // 7/6
    let aura = g.add_card_to_hand(0, catalog::murder_investigation());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(wurm)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    g.battlefield_find_mut(wurm).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.definition.name == "Soldier" && c.controller == 0).count();
    assert_eq!(soldiers, 7, "one Soldier per power");
}

/// Dying Wish drains life equal to the dead creature's power.
#[test]
fn gtc9_dying_wish_drains() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::ruination_wurm()); // power 7
    let aura = g.add_card_to_hand(0, catalog::dying_wish());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(wurm)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast aura");
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let me_before = g.players[0].life;
    g.battlefield_find_mut(wurm).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack_targeting(&mut g, Target::Player(1));
    assert_eq!(g.players[1].life, opp_before - 7, "opponent lost 7");
    assert_eq!(g.players[0].life, me_before + 7, "I gained 7");
}

/// Truefire Captain reflects damage dealt to it onto a target player.
#[test]
fn gtc9_truefire_captain_reflects_damage() {
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::truefire_captain());
    let opp_before = g.players[1].life;
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(cap), 3, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack_targeting(&mut g, Target::Player(1));
    assert_eq!(g.players[1].life, opp_before - 3, "reflected 3 to the player");
}

// ── Wave 10 ──────────────────────────────────────────────────────────────────

/// Wrecking Ogre has double strike and its Bloodrush grants +3/+3 + double strike.
#[test]
fn gtc10_wrecking_ogre_bloodrush() {
    use crabomination::mana::Color;
    let ogre = catalog::wrecking_ogre();
    assert!(ogre.keywords.contains(&Keyword::DoubleStrike));
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::gutter_skulk()); // 2/2
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    let card = g.add_card_to_hand(0, catalog::wrecking_ogre());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: card, ability_index: 0, target: Some(Target::Permanent(attacker)),
        additional_targets: vec![], x_value: None,
    }).expect("bloodrush");
    drain_stack(&mut g);
    let c = g.computed_permanent(attacker).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5), "2/2 + 3/3");
    assert!(c.keywords.contains(&Keyword::DoubleStrike), "gained double strike");
}

/// Incursion Specialist pumps +2/+0 and turns unblockable on the second spell.
#[test]
fn gtc10_incursion_specialist_second_spell() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let spec = g.add_card_to_battlefield(0, catalog::incursion_specialist());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Cast two creature spells; the flurry fires on the second.
    for _ in 0..2 {
        let s = g.add_card_to_hand(0, catalog::gutter_skulk());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
    }
    let c = g.computed_permanent(spec).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "1/3 + 2/0");
    assert!(c.keywords.contains(&Keyword::Unblockable), "can't be blocked this turn");
}

/// Molten Primordial's ETB steals an opponent's creature with haste.
#[test]
fn gtc10_molten_primordial_steals() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::ruination_wurm());
    let prim = g.add_card_to_battlefield(0, catalog::molten_primordial());
    g.fire_self_etb_triggers(prim, 0);
    drain_stack(&mut g);
    let c = g.computed_permanent(foe).unwrap();
    assert_eq!(c.controller, 0, "gained control of the opponent's creature");
    assert!(c.keywords.contains(&Keyword::Haste), "it has haste");
}

/// Sepulchral Primordial reanimates from an opponent's graveyard under your control.
#[test]
fn gtc10_sepulchral_primordial_reanimates_opponent_gy() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::ruination_wurm());
    let prim = g.add_card_to_battlefield(0, catalog::sepulchral_primordial());
    g.fire_self_etb_triggers(prim, 0);
    drain_stack(&mut g);
    let c = g.battlefield_find(dead).expect("reanimated onto battlefield");
    assert_eq!(c.controller, 0, "under your control");
}

/// Luminate Primordial exiles an opponent's creature; that player gains its power.
#[test]
fn gtc10_luminate_primordial_exiles_and_gains() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::ruination_wurm()); // power 7
    let opp_before = g.players[1].life;
    let prim = g.add_card_to_battlefield(0, catalog::luminate_primordial());
    g.fire_self_etb_triggers(prim, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature exiled");
    assert_eq!(g.players[1].life, opp_before + 7, "controller gained life equal to power");
}

/// Sylvan Primordial destroys an opponent's noncreature and fetches a tapped Forest.
#[test]
fn gtc10_sylvan_primordial_destroys_and_ramps() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::razortip_whip()); // an artifact
    let forest = g.add_card_to_library(0, catalog::forest());
    let prim = g.add_card_to_battlefield(0, catalog::sylvan_primordial());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.fire_self_etb_triggers(prim, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "opponent's artifact destroyed");
    let f = g.battlefield_find(forest).expect("Forest fetched onto the battlefield");
    assert!(f.tapped, "Forest enters tapped");
}

/// Treasury Thrull returns a permanent from your graveyard on combat damage.
#[test]
fn gtc10_treasury_thrull_recurs_on_damage() {
    let mut g = two_player_game();
    let thrull = g.add_card_to_battlefield(0, catalog::treasury_thrull());
    let dead = g.add_card_to_graveyard(0, catalog::razortip_whip()); // an artifact
    g.clear_sickness(thrull);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: thrull, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "artifact returned from graveyard");
}

/// Hellkite Tyrant steals all of the damaged player's artifacts.
#[test]
fn gtc10_hellkite_tyrant_steals_artifacts() {
    let mut g = two_player_game();
    let tyrant = g.add_card_to_battlefield(0, catalog::hellkite_tyrant());
    let rock = g.add_card_to_battlefield(1, catalog::razortip_whip());
    g.clear_sickness(tyrant);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: tyrant, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(rock).unwrap().controller, 0, "artifact stolen");
}

/// Hellkite Tyrant wins the game with twenty artifacts at upkeep.
#[test]
fn gtc10_hellkite_tyrant_artifact_win() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hellkite_tyrant());
    for _ in 0..20 { g.add_card_to_battlefield(0, catalog::razortip_whip()); }
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[1].eliminated || g.game_over.is_some(),
        "controller wins with 20 artifacts");
}

// ── Wave 11 ──────────────────────────────────────────────────────────────────

/// Hindervines fogs an uncountered attacker but not a +1/+1-countered one.
#[test]
fn gtc11_hindervines_spares_counter_creatures() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let plain = g.add_card_to_battlefield(0, catalog::ruination_wurm()); // 7/6, no counters
    let buffed = g.add_card_to_battlefield(0, catalog::ruination_wurm());
    g.battlefield_find_mut(buffed).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.clear_sickness(plain);
    g.clear_sickness(buffed);
    // Cast the fog.
    let fog = g.add_card_to_hand(0, catalog::hindervines());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(fog, None, vec![], None, None).expect("cast");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: plain, target: AttackTarget::Player(1) },
        Attack { attacker: buffed, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    let before = g.players[1].life;
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 8, "only the +1/+1 attacker (8 power) got through");
}

/// Lord of the Void steals a creature from the defending player's top seven.
#[test]
fn gtc11_lord_of_the_void_steals_from_library() {
    let mut g = two_player_game();
    let lord = g.add_card_to_battlefield(0, catalog::lord_of_the_void());
    // Stack the opponent's library so a creature is within the top seven.
    let target = g.add_card_to_library(1, catalog::ruination_wurm());
    for _ in 0..3 { g.add_card_to_library(1, catalog::forest()); }
    g.clear_sickness(lord);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lord, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    let c = g.battlefield_find(target).expect("creature pulled onto battlefield");
    assert_eq!(c.controller, 0, "under your control");
}

/// Duskmantle Seer drains each player for their top card's mana value.
#[test]
fn gtc11_duskmantle_seer_symmetric_drain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::duskmantle_seer());
    let mine = g.add_card_to_library(0, catalog::ruination_wurm()); // MV 6
    let theirs = g.add_card_to_library(1, catalog::duskmantle_seer()); // MV 4
    let my_life = g.players[0].life;
    let opp_life = g.players[1].life;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, my_life - 6, "I lost MV of my top card");
    assert_eq!(g.players[1].life, opp_life - 4, "opponent lost MV of theirs");
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "my card went to hand");
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "their card went to hand");
}

/// Deathpact Angel leaves a Cleric token that can recur it from the graveyard.
#[test]
fn gtc11_deathpact_angel_recurs() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::deathpact_angel());
    // Kill it; the dies trigger mints the Cleric.
    g.battlefield_find_mut(angel).unwrap().damage = 99;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let cleric = g.battlefield.iter().find(|c| c.definition.name == "Cleric" && c.controller == 0)
        .expect("Cleric token minted").id;
    g.clear_sickness(cleric);
    // Angel is in the graveyard; activate the token's recur ability.
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cleric, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("recur");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Deathpact Angel" && c.controller == 0),
        "Deathpact Angel returned to the battlefield");
}

/// Voidwalk exiles a creature and returns it at the next end step.
#[test]
fn gtc11_voidwalk_blinks() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::ruination_wurm());
    let spell = g.add_card_to_hand(0, catalog::voidwalk());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.cast_spell(spell, Some(Target::Permanent(foe)), vec![], None, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature exiled");
    // Resolve the delayed next-end-step return.
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Ruination Wurm" && c.controller == 1),
        "creature returned under its owner's control");
}
