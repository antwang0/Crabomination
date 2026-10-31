//! Commander: Ganax's, Aragorn's, Alisaie's, Gor Muldrak's and The Capitoline
//! Triad's last gaps (`decks::cmdr_edhrec32`).

use crabomination::card::{CardId, CounterType};
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
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_with(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    cast_with(g, seat, id, targets)
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn end_step(g: &mut GameState) {
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(g);
}

fn attack_and_hit(g: &mut GameState, attackers: &[CardId], defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    for &a in attackers {
        g.battlefield_find_mut(a).unwrap().tapped = false;
    }
    let attacks = attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) });
    g.perform_action(GameAction::DeclareAttackers(attacks.collect())).expect("attack");
    drain_stack(g);
    g.step = TurnStep::FirstStrikeDamage;
    let ev = g.resolve_first_strike_damage().expect("first strike");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn p1p1(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// CR 702.112b — Valeron Wardens draws when a creature you control becomes
/// renowned, here through Aragorn's granted renown 1; a creature that is
/// already renowned doesn't become renowned again (CR 702.112c).
#[test]
fn cr_702_112b_valeron_wardens_draws_when_aragorns_renown_lands() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::aragorn_hornburg_hero());
    ready(&mut g, 0, catalog::valeron_wardens());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    attack_and_hit(&mut g, &[bear], 2);
    let c = g.battlefield_find(bear).expect("bear");
    assert!(c.renowned);
    // Renown 1, then Aragorn doubles a renowned creature's counters — the
    // doubling only sees a creature renowned as the damage is dealt.
    assert_eq!(p1p1(&g, bear), 1);
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw for one creature becoming renowned");
    attack_and_hit(&mut g, &[bear], 1);
    assert_eq!(p1p1(&g, bear), 2, "already renowned: Aragorn doubles, renown is spent");
    assert_eq!(g.players[0].hand.len(), hand + 1, "no second renown, no second draw");
}

/// CR 702.131b — Andúril: an attack puts a +1/+1 counter on each creature you
/// control, two once you have the city's blessing.
#[test]
fn cr_702_131b_anduril_counters_scale_with_the_citys_blessing() {
    let mut g = pod(2);
    let sword = ready(&mut g, 0, catalog::anduril_narsil_reforged());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
    attack_and_hit(&mut g, &[bear], 1);
    assert_eq!((p1p1(&g, bear), p1p1(&g, giant)), (1, 1));
    g.players[0].city_blessing = true;
    attack_and_hit(&mut g, &[bear], 1);
    assert_eq!((p1p1(&g, bear), p1p1(&g, giant)), (3, 3));
}

/// Dragon Cultist's granted end-step trigger needs one source you controlled
/// to have dealt 5 damage — two sources dealing 3 each don't count.
#[test]
fn dragon_cultist_needs_one_source_dealing_five() {
    let mut g = pod(2);
    let ganax = ready(&mut g, 0, catalog::ganax_astral_hunter());
    g.players[0].commanders.push(ganax);
    ready(&mut g, 0, catalog::dragon_cultist());
    let shock = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, shock, &[Target::Player(1)]).expect("bolt");
    let shock = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, shock, &[Target::Player(1)]).expect("bolt");
    end_step(&mut g);
    assert_eq!(named(&g, 0, "Dragon"), 0, "6 damage, but from two sources");
    g.step = TurnStep::PreCombatMain;
    let axe = g.add_card_to_hand(0, catalog::lava_axe());
    cast(&mut g, 0, axe, &[Target::Player(1)]).expect("lava axe");
    end_step(&mut g);
    assert_eq!(named(&g, 0, "Dragon"), 1);
}

/// Acolyte of Bahamut: the first Dragon spell each turn costs {2} less; the
/// second pays full.
#[test]
fn acolyte_of_bahamut_discounts_the_first_dragon() {
    let mut g = pod(2);
    let ganax = ready(&mut g, 0, catalog::ganax_astral_hunter());
    g.players[0].commanders.push(ganax);
    ready(&mut g, 0, catalog::acolyte_of_bahamut());
    // Sync the granted static onto the commander.
    g.check_state_based_actions();
    let pay = |g: &mut GameState| {
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(2);
    };
    let first = g.add_card_to_hand(0, catalog::shivan_dragon());
    pay(&mut g);
    cast_with(&mut g, 0, first, &[]).expect("{2}{R}{R} pays for the first Dragon");
    let second = g.add_card_to_hand(0, catalog::shivan_dragon());
    pay(&mut g);
    assert!(cast_with(&mut g, 0, second, &[]).is_err(), "the second Dragon costs full");
}

/// Sunseed Nurturer gains 2 at your end step only with a power-5 creature.
#[test]
fn sunseed_nurturer_needs_a_big_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sunseed_nurturer());
    let life = g.players[0].life;
    end_step(&mut g);
    assert_eq!(g.players[0].life, life);
    ready(&mut g, 0, catalog::colossal_dreadmaw());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    end_step(&mut g);
    assert_eq!(g.players[0].life, life + 2);
}

/// Gor Muldrak: each player tied for the fewest creatures gets a Salamander,
/// and a Salamander's combat damage to Gor's controller is prevented (CR
/// 702.16e — you have protection from Salamanders).
#[test]
fn cr_702_16e_gor_muldrak_salamanders_feed_the_fewest_and_cant_hurt_you() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::gor_muldrak_amphinologist());
    ready(&mut g, 2, catalog::grizzly_bears());
    end_step(&mut g);
    assert_eq!(named(&g, 1, "Salamander Warrior"), 1, "seat 1 has none");
    assert_eq!(named(&g, 0, "Salamander Warrior") + named(&g, 2, "Salamander Warrior"), 0);
    let sal = g.battlefield.iter().find(|c| c.controller == 1).map(|c| c.id).expect("salamander");
    g.clear_sickness(sal);
    g.active_player_idx = 1;
    let life = g.players[0].life;
    attack_and_hit(&mut g, &[sal], 0);
    assert_eq!(g.players[0].life, life, "protection from Salamanders prevents the damage");
    let life = g.players[2].life;
    attack_and_hit(&mut g, &[sal], 2);
    assert_eq!(g.players[2].life, life - 4, "an unprotected player takes it");
}

/// Metamorphic Alteration: the enchanted creature becomes a copy of the chosen
/// creature for as long as the Aura stays attached (CR 707.2).
#[test]
fn metamorphic_alteration_copies_the_chosen_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![giant])]));
    let aura = g.add_card_to_hand(0, catalog::metamorphic_alteration());
    cast(&mut g, 0, aura, &[Target::Permanent(bear)]).expect("aura");
    let cp = g.computed_permanent(bear).expect("bear");
    assert_eq!(cp.def.name, "Hill Giant");
    assert_eq!((cp.power, cp.toughness), (3, 3));
    g.battlefield_find_mut(aura).unwrap().attached_to = None;
    g.check_state_based_actions();
    assert_eq!(g.computed_permanent(bear).expect("bear").def.name, "Grizzly Bears");
}

/// Metamorphic Alteration's copy is a static ability, not a trigger: it is in
/// place as the Aura resolves, with no window to respond (CR 707.2, 611.3a). The
/// values lock as it first applies — the chosen creature leaving changes nothing —
/// and moving the Aura copies the new host while the old one reverts (CR 611.2c).
#[test]
fn metamorphic_alteration_locks_the_copy_and_follows_the_aura() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let wolf = ready(&mut g, 1, catalog::savannah_lions());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![giant])]));
    let aura = g.add_card_to_hand(0, catalog::metamorphic_alteration());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    for _ in 0..2 {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.stack.is_empty(), "no copy trigger goes on the stack");
    assert_eq!(g.computed_permanent(bear).expect("bear").def.name, "Hill Giant");
    g.remove_from_battlefield_to_graveyard_raw(giant);
    assert_eq!(g.computed_permanent(bear).expect("bear").def.name, "Hill Giant", "locked in");
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let moved = g
        .resolve_effect(
            &crabomination::effect::Effect::Attach {
                what: crabomination::effect::Selector::ExactObjects(vec![aura]),
                to: crabomination::effect::Selector::ExactObjects(vec![wolf]),
            },
            &ctx,
        )
        .expect("move the Aura");
    g.dispatch_triggers_for_events(&moved);
    g.check_state_based_actions();
    assert_eq!(g.computed_permanent(wolf).expect("wolf").def.name, "Hill Giant", "the new host copies");
    assert_eq!(g.computed_permanent(bear).expect("bear").def.name, "Grizzly Bears", "the old host reverts");
}

/// The Capitoline Triad: {1} less per historic card in your graveyard; its
/// ability exiles the fewest historic cards worth 30 mana value for an emblem
/// that makes your creatures 9/9.
#[test]
fn the_capitoline_triad_discounts_and_crowns_your_creatures() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::sol_ring());
    }
    let triad = g.add_card_to_hand(0, catalog::the_capitoline_triad());
    g.players[0].mana_pool.add_colorless(6);
    cast_with(&mut g, 0, triad, &[]).expect("{10} minus four historic cards");
    g.clear_sickness(triad);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let activate = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        let r = g.perform_action(GameAction::ActivateAbility {
            card_id: triad,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        });
        drain_stack(g);
        r
    };
    assert!(activate(&mut g).is_err(), "four mana value isn't thirty");
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::the_capitoline_triad());
    }
    activate(&mut g).expect("thirty mana value");
    assert_eq!(g.players[0].graveyard.len(), 4, "three Triads exiled, the Sol Rings stay");
    let cp = g.computed_permanent(bear).expect("bear");
    assert_eq!((cp.power, cp.toughness), (9, 9));
}

/// "Exile any number of historic cards … with total mana value 30 or
/// greater": a manual seat names the set (it was always the auto-pick), here
/// three Triads and a Sol Ring.
#[test]
fn the_capitoline_triad_lets_a_manual_seat_name_the_exiled_set() {
    use crabomination::decision::DecisionAnswer;
    let mut g = pod(2);
    let rings: Vec<_> = (0..4).map(|_| g.add_card_to_graveyard(0, catalog::sol_ring())).collect();
    let triads: Vec<_> = (0..3).map(|_| g.add_card_to_graveyard(0, catalog::the_capitoline_triad())).collect();
    let triad = ready(&mut g, 0, catalog::the_capitoline_triad());
    g.players[0].manual_mana = true;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: triad,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("asks");
    assert!(g.pending_decision.is_some(), "the exiled set is asked");
    let mut pick = triads.clone();
    pick.push(rings[0]);
    g.submit_decision(DecisionAnswer::Cards(pick)).expect("thirty-one mana value");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 3, "the chosen Sol Ring went too");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == rings[0]));
}

/// Lavinia investigates on your second spell each turn; her {C}{C} only taps
/// on an opponent's turn.
#[test]
fn lavinia_investigates_on_the_second_spell_and_taps_off_turn() {
    let mut g = pod(2);
    let lavinia = ready(&mut g, 0, catalog::lavinia_foil_to_conspiracy());
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, 0, bolt, &[Target::Player(1)]).expect("bolt");
    }
    assert_eq!(named(&g, 0, "Clue"), 1);
    let tap = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: lavinia,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(tap(&mut g).is_err(), "not on your own turn");
    g.active_player_idx = 1;
    let before = g.players[0].mana_pool.colorless_amount();
    tap(&mut g).expect("an opponent's turn");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), before + 2);
}

/// Monk Class level 3: the upkeep-exiled card is castable only once you've
/// cast another spell this turn.
#[test]
fn monk_class_exiled_card_waits_for_another_spell() {
    let mut g = pod(2);
    let class = ready(&mut g, 0, catalog::monk_class());
    g.battlefield_find_mut(class).unwrap().class_level = 3;
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == top));
    g.step = TurnStep::PreCombatMain;
    let from_exile = |g: &mut GameState| {
        flood(g, 0);
        g.priority.player_with_priority = 0;
        let r = g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: top,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        });
        drain_stack(g);
        r
    };
    assert!(from_exile(&mut g).is_err(), "no other spell yet");
    let other = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, other, &[Target::Player(1)]).expect("first spell");
    from_exile(&mut g).expect("castable after another spell");
    assert_eq!(g.players[1].life, 14);
}
