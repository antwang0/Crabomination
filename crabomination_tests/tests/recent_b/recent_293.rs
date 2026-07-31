//! Tests for the recent293 Ravnica batch 3 (Transmute, Haunt, Graft, Dredge,
//! and assorted guild spells).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState, TurnStep};
use crabomination::mana::Color;

/// Destroy `id` via a sacrifice effect, then let its death triggers resolve.
fn kill(g: &mut GameState, id: CardId) {
    let ctrl = g.battlefield_find(id).unwrap().controller;
    let ctx = EffectContext::for_ability(id, ctrl, Some(Target::Permanent(id)));
    g.resolve_effect(&Effect::SacrificePermanent { what: Selector::Target(0) }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&[]);
    drain_stack(g);
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 6);
    }
    g.players[0].mana_pool.add_colorless(8);
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn count_tokens(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

// ── Transmute ───────────────────────────────────────────────────────────────

#[test]
fn muddle_the_mixture_counters_an_instant() {
    let mut g = two_player_game();
    // P1 puts an instant on the stack.
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    let helix = g.add_card_to_hand(1, catalog::lightning_helix());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: helix, target: Some(Target::Player(0)), additional_targets: vec![], mode: None,
        x_value: None,
    }).expect("helix on stack");
    // P0 counters it with Muddle.
    let muddle = g.add_card_to_hand(0, catalog::muddle_the_mixture());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: muddle, target: Some(Target::Permanent(helix)), additional_targets: vec![], mode: None,
        x_value: None,
    }).expect("cast muddle");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == helix), "helix was countered");
}

#[test]
fn muddle_transmutes_for_a_same_mv_card() {
    // Transmute {1}{U}{U}: discard Muddle (MV 2) to fetch an MV-2 card.
    let mut g = two_player_game();
    let target = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
    let muddle = g.add_card_to_hand(0, catalog::muddle_the_mixture());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: muddle, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("transmute");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == target), "MV-2 card fetched");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == muddle), "Muddle discarded");
}

#[test]
fn dizzy_spell_weakens_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::dizzy_spell());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let p = g.computed_permanent(bear).unwrap();
    assert_eq!((p.power, p.toughness), (-1, 2), "-3/-0 on a 2/2");
}

#[test]
fn shred_memory_exiles_from_one_graveyard() {
    let mut g = two_player_game();
    let ids: Vec<CardId> =
        (0..3).map(|_| g.add_card_to_graveyard(1, catalog::grizzly_bears())).collect();
    let spell = g.add_card_to_hand(0, catalog::shred_memory());
    flood(&mut g);
    // Resolution-time choice of which graveyard cards to exile.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(ids)]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 0, "all three cards exiled from the single graveyard");
}

#[test]
fn clutch_of_the_undercity_bounces_and_drains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::clutch_of_the_undercity());
    flood(&mut g);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "returned to owner's hand");
    assert_eq!(g.players[1].life, life - 3, "controller lost 3 life");
}

#[test]
fn brainspoil_only_hits_unenchanted_creatures() {
    let mut g = two_player_game();
    let plain = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let enchanted = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(1, catalog::pillory_of_the_sleepless());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(enchanted);
    let spell = g.add_card_to_hand(0, catalog::brainspoil());
    flood(&mut g);
    // Enchanted creature is an illegal target.
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(enchanted)), additional_targets: vec![],
        mode: None, x_value: None,
    }).is_err(), "can't target an enchanted creature");
    // The unenchanted one is destroyed.
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(plain)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(plain).is_none(), "unenchanted creature destroyed");
}

#[test]
fn dimir_infiltrator_is_unblockable() {
    let mut g = two_player_game();
    let inf = g.add_card_to_battlefield(0, catalog::dimir_infiltrator());
    assert!(g.computed_permanent(inf).unwrap().keywords.contains(&Keyword::Unblockable));
}

#[test]
fn netherborn_phalanx_drains_per_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let phalanx = g.add_card_to_battlefield(0, catalog::netherborn_phalanx());
    g.fire_self_etb_triggers(phalanx, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "opponent loses 1 per their 2 creatures");
}

// ── Haunt ───────────────────────────────────────────────────────────────────

#[test]
fn blind_hunter_drains_on_enter_and_when_haunted_dies() {
    let mut g = two_player_game();
    let foe_start = g.players[1].life;
    let my_start = g.players[0].life;
    let hunter = g.add_card_to_battlefield(0, catalog::blind_hunter());
    g.fire_self_etb_triggers(hunter, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_start - 2, "ETB drained 2");
    assert_eq!(g.players[0].life, my_start + 2, "ETB gained 2");
    // It dies haunting an opponent's creature; when that creature dies, drain again.
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    kill(&mut g, hunter);
    let foe_mid = g.players[1].life;
    kill(&mut g, victim);
    assert_eq!(g.players[1].life, foe_mid - 2, "haunt payoff drained 2 more");
}

#[test]
fn belfry_spirit_makes_two_bats_on_enter() {
    let mut g = two_player_game();
    let spirit = g.add_card_to_battlefield(0, catalog::belfry_spirit());
    g.fire_self_etb_triggers(spirit, 0);
    drain_stack(&mut g);
    assert_eq!(count_tokens(&g, "Bat"), 2);
}

// ── Boros / Gruul ───────────────────────────────────────────────────────────

#[test]
fn sunhome_enforcer_gains_life_on_combat_damage() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let sunhome = g.add_card_to_battlefield(0, catalog::sunhome_enforcer());
    g.clear_sickness(sunhome);
    let life = g.players[0].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sunhome, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained life equal to the 2 combat damage");
}

#[test]
fn rumbling_slum_pings_each_player_on_upkeep() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.add_card_to_battlefield(0, catalog::rumbling_slum());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 - 1);
    assert_eq!(g.players[1].life, l1 - 1);
}

// ── Simic ───────────────────────────────────────────────────────────────────

#[test]
fn simic_sky_swallower_has_shroud() {
    let mut g = two_player_game();
    let swallower = g.add_card_to_battlefield(1, catalog::simic_sky_swallower());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(swallower)), additional_targets: vec![],
        mode: None, x_value: None,
    }).is_err(), "shroud blocks targeting");
    let c = g.computed_permanent(swallower).unwrap();
    assert!(c.keywords.contains(&Keyword::Flying) && c.keywords.contains(&Keyword::Trample));
}

#[test]
fn novijen_sages_grafts_and_draws() {
    let mut g = two_player_game();
    // Real placement applies the printed `enters_with_counters` (Graft 4).
    let sages = g.move_card_to_battlefield_for_test(0, catalog::novijen_sages());
    // Graft 4: enters as a 4/4.
    let p = g.computed_permanent(sages).unwrap();
    assert_eq!((p.power, p.toughness), (4, 4), "enters with four +1/+1 counters");
    g.clear_sickness(sages);
    g.add_card_to_library(0, catalog::grizzly_bears());
    flood(&mut g);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: sages, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("remove two counters, draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    assert_eq!(g.computed_permanent(sages).unwrap().power, 2, "two +1/+1 counters removed");
}

// ── Orzhov auras / legends / bounce ─────────────────────────────────────────

#[test]
fn pillory_locks_down_and_bleeds() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(1, catalog::pillory_of_the_sleepless());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(creature);
    let comp = g.computed_permanent(creature).unwrap();
    assert!(comp.keywords.contains(&Keyword::CantAttack));
    assert!(comp.keywords.contains(&Keyword::CantBlock));
    // Its controller (player 1) loses 1 at the beginning of their upkeep.
    g.active_player_idx = 1;
    let life = g.players[1].life;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
}

#[test]
fn ghost_council_blinks_and_retriggers_drain() {
    let mut g = two_player_game();
    let council = g.add_card_to_battlefield(0, catalog::ghost_council_of_orzhova());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(council);
    flood(&mut g);
    let (foe, me) = (g.players[1].life, g.players[0].life);
    g.perform_action(GameAction::ActivateAbility {
        card_id: council, ability_index: 0, target: None,
        additional_targets: vec![Target::Permanent(fodder)], x_value: None, mode: None,
    }).expect("sac to blink");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "the fodder creature was sacrificed");
    // Ghost Council exiled; advance to the end step so it returns and re-drains.
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Ghost Council of Orzhova"),
        "returned at the next end step");
    assert_eq!(g.players[1].life, foe - 1, "the returning ETB drained the opponent");
    assert_eq!(g.players[0].life, me + 1, "and gained you 1");
}

#[test]
fn seeds_of_strength_pumps_three_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::seeds_of_strength());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b), Target::Permanent(c)], mode: None,
        x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    for id in [a, b, c] {
        assert_eq!(g.computed_permanent(id).unwrap().power, 3, "+1/+1");
    }
}

#[test]
fn vedalken_dismisser_tucks_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dismisser = g.add_card_to_battlefield(0, catalog::vedalken_dismisser());
    g.fire_self_etb_triggers(dismisser, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "removed from the battlefield");
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(bear), "on top of owner's library");
}

#[test]
fn nightmare_void_discards_a_chosen_card() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::nightmare_void());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "the chosen card was discarded");
    assert!(catalog::nightmare_void().keywords.contains(&Keyword::Dredge(2)), "Nightmare Void has dredge 2");
}

#[test]
fn vedalken_entrancer_mills_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let entrancer = g.add_card_to_battlefield(0, catalog::vedalken_entrancer());
    g.clear_sickness(entrancer);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: entrancer, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None, mode: None,
    }).expect("mill");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 2, "milled two cards");
}

#[test]
fn beacon_hawk_firebreathes_toughness() {
    let mut g = two_player_game();
    let hawk = g.add_card_to_battlefield(0, catalog::beacon_hawk());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hawk, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("pump");
    drain_stack(&mut g);
    let p = g.computed_permanent(hawk).unwrap();
    assert_eq!((p.power, p.toughness), (1, 2), "+0/+1");
}
