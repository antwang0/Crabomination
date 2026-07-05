//! Functionality tests for `catalog::sets::decks::abilitywords` and the new
//! ability-word condition predicates (Threshold / Metalcraft / Ferocious /
//! Hellbent / Formidable).

use crate::card::Keyword;
use crate::catalog;
use crate::game::two_player_game;
use crate::game::*;

/// Fill player `p`'s graveyard with `n` cards.
fn fill_gy(g: &mut GameState, p: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_graveyard(p, catalog::island());
    }
}

/// Resolve a spell `def` cast by player 0 against the given target slots.
fn resolve_spell(g: &mut GameState, def: crate::card::CardDefinition, targets: Vec<Target>) {
    let mut ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = targets;
    g.resolve_effect(&def.effect, &ctx).unwrap();
}

// ── Threshold ────────────────────────────────────────────────────────────────

#[test]
fn springing_tiger_threshold_pump() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::springing_tiger());
    assert_eq!(g.computed_permanent(id).unwrap().power, 3, "no threshold → base 3/3");
    fill_gy(&mut g, 0, 7);
    assert_eq!(g.computed_permanent(id).unwrap().power, 5, "threshold → +2/+2");
}

#[test]
fn krosan_beast_threshold_pump() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::krosan_beast());
    fill_gy(&mut g, 0, 7);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8), "1/1 + threshold +7/+7");
}

#[test]
fn mystic_enforcer_threshold_grants_flying() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mystic_enforcer());
    assert!(!g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying));
    fill_gy(&mut g, 0, 7);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!(cp.power, 6, "3/3 + threshold +3/+3");
    assert!(cp.keywords.contains(&Keyword::Flying), "threshold grants flying");
}

#[test]
fn anurid_barkripper_threshold_pump() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::anurid_barkripper());
    fill_gy(&mut g, 0, 7);
    assert_eq!(g.computed_permanent(id).unwrap().power, 4, "2/2 + threshold +2/+2");
}

// ── Metalcraft ───────────────────────────────────────────────────────────────

fn give_three_artifacts(g: &mut GameState, p: usize) {
    for _ in 0..3 {
        g.add_card_to_battlefield(p, catalog::ornithopter());
    }
}

#[test]
fn ardent_recruit_metalcraft_pump() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ardent_recruit());
    assert_eq!(g.computed_permanent(id).unwrap().power, 1, "no metalcraft → 1/1");
    give_three_artifacts(&mut g, 0);
    assert_eq!(g.computed_permanent(id).unwrap().power, 3, "metalcraft → +2/+2");
}

#[test]
fn snapsail_glider_metalcraft_flying() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::snapsail_glider());
    assert!(!g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying));
    // The glider counts as one artifact; add two more.
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::ornithopter());
    }
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying));
}

#[test]
fn auriok_sunchaser_metalcraft_pump_and_flying() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::auriok_sunchaser());
    give_three_artifacts(&mut g, 0);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!(cp.power, 3, "1/1 + metalcraft +2/+2");
    assert!(cp.keywords.contains(&Keyword::Flying));
}

#[test]
fn dispatch_taps_then_exiles_with_metalcraft() {
    // No metalcraft → taps the creature.
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve_spell(&mut g, catalog::dispatch(), vec![Target::Permanent(foe)]);
    assert!(g.battlefield_find(foe).is_some_and(|c| c.tapped), "no metalcraft → tapped");

    // Metalcraft → exiles it.
    let mut g = two_player_game();
    give_three_artifacts(&mut g, 0);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve_spell(&mut g, catalog::dispatch(), vec![Target::Permanent(foe)]);
    assert!(g.battlefield_find(foe).is_none(), "metalcraft → exiled");
}

// ── Ferocious ────────────────────────────────────────────────────────────────

#[test]
fn savage_punch_ferocious_pumps_before_fight() {
    let mut g = two_player_game();
    // A power-4 creature turns Ferocious on.
    g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 4/4 after pump
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    resolve_spell(&mut g, catalog::savage_punch(), vec![Target::Permanent(mine), Target::Permanent(foe)]);
    // Ferocious pumps mine to 4/4, so it kills the 3/3 and survives the 3 back.
    assert!(g.battlefield_find(foe).is_none(), "3/3 dies to the pumped 4/4");
    assert!(g.battlefield_find(mine).is_some(), "pumped 4/4 survives 3 damage");
}

// ── Formidable ───────────────────────────────────────────────────────────────

#[test]
fn circle_of_elders_only_taps_when_formidable() {
    let mut g = two_player_game();
    let circle = g.add_card_to_battlefield(0, catalog::circle_of_elders());
    g.clear_sickness(circle);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let act = GameAction::ActivateAbility {
        card_id: circle, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    };
    assert!(g.perform_action(act.clone()).is_err(), "not formidable → can't activate");
    // Push total power to 8+ (2/4 circle + 4/4 + 4/4).
    g.add_card_to_battlefield(0, catalog::serra_angel());
    g.add_card_to_battlefield(0, catalog::serra_angel());
    g.perform_action(act).expect("formidable → activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 3, "added three colorless");
}

#[test]
fn sabertooth_outrider_first_strike_when_formidable() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sabertooth_outrider()); // 4/2
    g.clear_sickness(id);
    g.add_card_to_battlefield(0, catalog::serra_angel()); // total power 8
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("advance");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::FirstStrike),
        "formidable attack trigger grants first strike");
}

// ── Hellbent ─────────────────────────────────────────────────────────────────

#[test]
fn cutthroat_il_dal_shadow_only_while_hellbent() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::cutthroat_il_dal());
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Shadow),
        "empty hand → hellbent → shadow");
    g.add_card_to_hand(0, catalog::island());
    assert!(!g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Shadow),
        "card in hand → not hellbent → no shadow");
}

#[test]
fn rakdos_pit_dragon_double_strike_while_hellbent() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::rakdos_pit_dragon());
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "empty hand → hellbent → double strike");
    g.add_card_to_hand(0, catalog::island());
    assert!(!g.computed_permanent(id).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

// ── Conditional burn ─────────────────────────────────────────────────────────

#[test]
fn bring_low_deals_more_to_countered_creature() {
    use crate::card::CounterType;
    // No counter → 3 damage (a 4-toughness creature survives).
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    resolve_spell(&mut g, catalog::bring_low(), vec![Target::Permanent(foe)]);
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 3, "no counter → 3 damage");

    // With a +1/+1 counter (→ 5/5) it deals 5 — lethal.
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.battlefield_find_mut(foe).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    resolve_spell(&mut g, catalog::bring_low(), vec![Target::Permanent(foe)]);
    assert!(g.battlefield_find(foe).is_none(), "counter → 5 damage kills the 5/5");
}

#[test]
fn sarkhans_rage_hurts_you_without_a_dragon() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let before = g.players[0].life;
    resolve_spell(&mut g, catalog::sarkhans_rage(), vec![Target::Permanent(foe)]);
    assert_eq!(g.players[0].life, before - 2, "no Dragon → 2 damage to you");

    // Control a Dragon → no self-damage.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rakdos_pit_dragon()); // a Dragon
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let before = g.players[0].life;
    resolve_spell(&mut g, catalog::sarkhans_rage(), vec![Target::Permanent(foe)]);
    assert_eq!(g.players[0].life, before, "controlling a Dragon → no self-damage");
}

// ── Refactored modern.rs cards keep working on the named predicates ───────────

#[test]
fn galvanic_blast_metalcraft_scales() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    resolve_spell(&mut g, catalog::galvanic_blast(), vec![Target::Permanent(foe)]);
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 2, "no metalcraft → 2 damage");

    let mut g = two_player_game();
    give_three_artifacts(&mut g, 0);
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    resolve_spell(&mut g, catalog::galvanic_blast(), vec![Target::Permanent(foe)]);
    assert!(g.battlefield_find(foe).is_none(), "metalcraft → 4 damage kills the 4/4");
}

#[test]
fn temur_battle_rage_double_strike_and_ferocious_trample() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve_spell(&mut g, catalog::temur_battle_rage(), vec![Target::Permanent(mine)]);
    let cp = g.computed_permanent(mine).unwrap();
    assert!(cp.keywords.contains(&Keyword::DoubleStrike), "base grants double strike");
    assert!(!cp.keywords.contains(&Keyword::Trample), "no ferocious → no trample");

    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::serra_angel()); // power 4 → ferocious
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve_spell(&mut g, catalog::temur_battle_rage(), vec![Target::Permanent(mine)]);
    let cp = g.computed_permanent(mine).unwrap();
    assert!(cp.keywords.contains(&Keyword::DoubleStrike));
    assert!(cp.keywords.contains(&Keyword::Trample), "ferocious grants trample");
}

#[test]
fn nimble_mongoose_threshold_pump() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::nimble_mongoose());
    assert_eq!(g.computed_permanent(id).unwrap().power, 1, "no threshold → 1/1");
    fill_gy(&mut g, 0, 7);
    assert_eq!(g.computed_permanent(id).unwrap().power, 3, "threshold → 3/3");
}
