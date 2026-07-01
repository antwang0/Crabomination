//! Functionality tests for `catalog::sets::decks::recent53` — monarch,
//! artifact hate, and white-weenie staples.

use crate::card::Keyword;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::*;
use crate::mana::Color;

#[test]
fn by_force_destroys_x_artifacts() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::mind_stone());
    let b = g.add_card_to_battlefield(1, catalog::mind_stone());
    let spell = g.add_card_to_hand(0, catalog::by_force());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast {X=2}{R} destroying two artifacts");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both artifacts destroyed");
}

#[test]
fn palace_jailer_takes_the_crown_and_a_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let jailer = g.add_card_to_battlefield(0, catalog::palace_jailer());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
    g.fire_self_etb_triggers(jailer, 0);
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0), "controller became the monarch");
    assert!(g.battlefield_find(victim).is_none(), "opponent's creature exiled");

    // CR 724 — when an opponent takes the crown, the creature comes back.
    let mut events = vec![];
    g.set_monarch(1, &mut events);
    assert!(g.battlefield_find(victim).is_some(), "creature returns when the monarchy moves");
}

#[test]
fn palace_jailer_keeps_the_creature_if_it_dies_while_monarch() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let jailer = g.add_card_to_battlefield(0, catalog::palace_jailer());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
    g.fire_self_etb_triggers(jailer, 0);
    drain_stack(&mut g);
    // Jailer leaving does NOT release a monarch-guarded exile (unlike O-Ring).
    g.remove_from_battlefield_to_graveyard_raw(jailer);
    assert!(g.battlefield_find(victim).is_none(), "creature stays exiled while you're still the monarch");
}

#[test]
fn loxodon_smiter_cant_be_countered() {
    assert!(catalog::loxodon_smiter().keywords.contains(&Keyword::CantBeCountered));
}

#[test]
fn leonin_vanguard_pumps_with_a_full_board() {
    let mut g = two_player_game();
    let leonin = g.add_card_to_battlefield(0, catalog::leonin_vanguard());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    g.step = TurnStep::BeginCombat;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let v = cp.iter().find(|c| c.id == leonin).unwrap();
    assert_eq!((v.power, v.toughness), (2, 2), "buffed with 3 creatures");
    assert_eq!(g.players[0].life, life + 1, "gained a life");
}

#[test]
fn giada_scales_entering_angels() {
    let mut g = two_player_game();
    // Giada + one more Angel already out = two Angels you control.
    g.add_card_to_battlefield(0, catalog::giada_font_of_hope());
    g.add_card_to_battlefield(0, catalog::serra_angel());
    // A third Angel enters (reanimation/move path) with +1/+1 per existing Angel.
    let newcomer = g.move_card_to_battlefield_for_test(0, catalog::serra_angel());
    let cp = g.compute_battlefield();
    let a = cp.iter().find(|c| c.id == newcomer).unwrap();
    // Serra Angel is 4/4; two Angels already controlled → +2/+2 → 6/6.
    assert_eq!((a.power, a.toughness), (6, 6), "entered with two +1/+1 counters");
}

#[test]
fn hopeful_initiate_removes_counters_from_among_creatures() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let init = g.add_card_to_battlefield(0, catalog::hopeful_initiate());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ench = g.add_card_to_battlefield(1, catalog::mark_of_asylum());
    // Spread two +1/+1 counters across the two creatures.
    g.battlefield.iter_mut().find(|c| c.id == init).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.battlefield.iter_mut().find(|c| c.id == ally).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: init, ability_index: 0, target: Some(Target::Permanent(ench)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("activates by removing two counters from among creatures");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
    assert_eq!(g.battlefield_find(init).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

#[test]
fn sanctum_prelate_locks_chosen_mana_value() {
    let mut g = two_player_game();
    let prelate = g.add_card_to_battlefield(0, catalog::sanctum_prelate());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(1)]));
    g.fire_self_etb_triggers(prelate, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(prelate).unwrap().chosen_number, Some(1));

    // A MV-1 noncreature spell (Lightning Bolt) is locked; a MV-1 creature and a
    // MV-2 noncreature are fine.
    assert!(g.noncreature_spell_cast_locked(&catalog::lightning_bolt()), "MV-1 noncreature locked");
    assert!(!g.noncreature_spell_cast_locked(&catalog::grizzly_bears()), "creature never locked");
    assert!(!g.noncreature_spell_cast_locked(&catalog::mind_stone()), "MV-2 noncreature unaffected");
}

#[test]
fn old_rutstein_mills_and_branches_by_type() {
    let mut g = two_player_game();
    let rutstein = g.add_card_to_battlefield(0, catalog::old_rutstein());
    // Land on top → mills a land → Treasure.
    let land = g.next_id();
    g.players[0].add_to_library_top(land, catalog::forest());
    g.fire_self_etb_triggers(rutstein, 0);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
        "milled land made a Treasure",
    );
    // Creature on top → Insect token.
    let crea = g.next_id();
    g.players[0].add_to_library_top(crea, catalog::grizzly_bears());
    g.fire_self_etb_triggers(rutstein, 0);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Insect"),
        "milled creature made an Insect",
    );
}

#[test]
fn custodi_lich_edicts_and_crowns() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lich = g.add_card_to_battlefield(0, catalog::custodi_lich());
    g.fire_self_etb_triggers(lich, 0);
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0), "took the crown");
    assert!(g.battlefield_find(opp).is_none(), "opponent sacrificed its only creature");
}
