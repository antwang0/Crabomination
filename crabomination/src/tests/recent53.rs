//! Functionality tests for `catalog::sets::decks::recent53` — monarch,
//! artifact hate, and white-weenie staples.

use crate::card::Keyword;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

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
fn thorn_of_the_black_rose_takes_the_crown() {
    let mut g = two_player_game();
    let thorn = g.add_card_to_battlefield(0, catalog::thorn_of_the_black_rose());
    g.fire_self_etb_triggers(thorn, 0);
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0));
}

#[test]
fn throne_warden_grows_while_monarch() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let warden = g.add_card_to_battlefield(0, catalog::throne_warden());
    // Not the monarch → no growth.
    g.monarch = None;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(warden).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    // Monarch → +1/+1.
    g.monarch = Some(0);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(warden).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn skyline_despot_makes_a_dragon_while_monarch() {
    let mut g = two_player_game();
    let despot = g.add_card_to_battlefield(0, catalog::skyline_despot());
    g.fire_self_etb_triggers(despot, 0);
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0), "took the crown on ETB");
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Dragon"),
        "monarch upkeep minted a Dragon",
    );
}

#[test]
fn keeper_of_keys_unblockable_while_monarch() {
    let mut g = two_player_game();
    let keeper = g.add_card_to_battlefield(0, catalog::keeper_of_keys());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.fire_self_etb_triggers(keeper, 0);
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    assert!(
        cp.iter().find(|c| c.id == ally).unwrap().keywords.contains(&Keyword::Unblockable),
        "your creatures gained unblockable while you're the monarch",
    );
}

#[test]
fn judith_pings_on_nontoken_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::judith_the_scourge_diva());
    let doomed = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Bolt our own bear so the SBA dispatches CreatureDied to Judith; her
    // trigger then pings the opponent (scripted target).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(doomed)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "Judith dealt 1 on a nontoken death");
}

#[test]
fn marchesas_decree_bleeds_attackers() {
    let mut g = two_player_game();
    // Player 1 controls the enchantment (is monarch); player 0 attacks them.
    let decree = g.add_card_to_battlefield(1, catalog::marchesas_decree());
    g.fire_self_etb_triggers(decree, 1);
    drain_stack(&mut g);
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let before = g.players[0].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("declare attack on the monarch");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before - 1, "the attacker's controller lost 1 life");
}

#[test]
fn terror_of_the_peaks_pings_on_creature_entry() {
    let mut g = two_player_game();
    let terror = g.add_card_to_battlefield(0, catalog::terror_of_the_peaks());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let before = g.players[1].life;
    // Cast a 2/2; Terror pings for its power (2).
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    drain_stack(&mut g);
    let _ = terror;
    assert_eq!(g.players[1].life, before - 2, "Terror dealt the entrant's power");
}

#[test]
fn warstorm_surge_pings_for_entrant_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::warstorm_surge());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let before = g.players[1].life;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 2, "Warstorm Surge pinged for the entrant's power");
}

#[test]
fn tuktuk_returns_bigger() {
    let mut g = two_player_game();
    let tuk = g.add_card_to_battlefield(0, catalog::tuktuk_the_explorer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(tuk)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt Tuktuk");
    drain_stack(&mut g);
    let ret = g.battlefield.iter().find(|c| c.definition.name == "Tuktuk the Returned").expect("token made");
    assert_eq!((ret.definition.power, ret.definition.toughness), (5, 5));
}

#[test]
fn tine_shrike_has_infect() {
    assert!(catalog::tine_shrike().keywords.contains(&Keyword::Infect));
}

#[test]
fn balustrade_spy_mills_to_a_land() {
    use crate::game::Target;
    let mut g = two_player_game();
    // Stack player 1's library: two nonlands on top of a land.
    for def in [catalog::grizzly_bears(), catalog::lightning_bolt()] {
        let id = g.next_id();
        g.players[1].add_to_library_top(id, def);
    }
    let land = g.next_id();
    g.players[1].library.push(crate::card::CardInstance::new(land, catalog::forest(), 1));
    let spy = g.add_card_to_battlefield(0, catalog::balustrade_spy());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    g.fire_self_etb_triggers(spy, 0);
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Forest"), "milled through to a land");
    assert!(g.players[1].graveyard.len() >= 3, "the nonlands + the land were milled");
}

#[test]
fn ravos_anthems_and_recurs() {
    let mut g = two_player_game();
    let ravos = g.add_card_to_battlefield(0, catalog::ravos_soultender());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Anthem: other creature 2/2 → 3/3.
    let cp = g.compute_battlefield();
    let a = cp.iter().find(|c| c.id == ally).unwrap();
    assert_eq!((a.power, a.toughness), (3, 3), "+1/+1 to others");
    // Upkeep return of a graveyard creature (script "yes"; auto-pick the target).
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let _ = ravos;
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "returned the creature to hand");
}

#[test]
fn adriana_grants_melee_to_the_team() {
    let mut g = two_player_game();
    let adriana = g.add_card_to_battlefield(0, catalog::adriana_captain_of_the_guard());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(adriana);
    g.clear_sickness(ally);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: adriana, target: AttackTarget::Player(1) },
        Attack { attacker: ally, target: AttackTarget::Player(1) },
    ])).expect("attack with both");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    // Melee (flat +1/+1 approximation) fires for both attackers.
    assert_eq!(
        { let a = cp.iter().find(|c| c.id == ally).unwrap(); (a.power, a.toughness) },
        (3, 3),
        "granted melee pumped the ally",
    );
}

#[test]
fn regal_behemoth_doubles_land_mana_while_monarch() {
    let mut g = two_player_game();
    let behemoth = g.add_card_to_battlefield(0, catalog::regal_behemoth());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.fire_self_etb_triggers(behemoth, 0);
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0));
    // Tap the Forest: one {G} from the land + one extra while monarch.
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap forest for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "monarch got an extra mana");
}

#[test]
fn gallant_cavalry_brings_a_friend() {
    let mut g = two_player_game();
    let cav = g.add_card_to_battlefield(0, catalog::gallant_cavalry());
    g.fire_self_etb_triggers(cav, 0);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Knight"),
        "made a Knight token",
    );
}

#[test]
fn valiant_knight_lords_and_grants_double_strike() {
    let mut g = two_player_game();
    let valiant = g.add_card_to_battlefield(0, catalog::valiant_knight());
    let ally = g.add_card_to_battlefield(0, catalog::gallant_cavalry()); // a Knight
    // Anthem: the other Knight is 2/2 → 3/3.
    let cp = g.compute_battlefield();
    let a = cp.iter().find(|c| c.id == ally).unwrap();
    assert_eq!((a.power, a.toughness), (3, 3), "+1/+1 to other Knights");
    // Activated grant: Knights gain double strike.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: valiant, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate double-strike pump");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    assert!(
        cp.iter().find(|c| c.id == valiant).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "Knights gained double strike",
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
