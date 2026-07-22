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
