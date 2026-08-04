//! Arabian Nights (ARN) — `catalog::sets::arn`.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    activate_n(g, seat, id, 0, target)
}

fn activate_n(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn upkeep(g: &mut GameState) {
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

// ── Vanilla / keyword bodies ───────────────────────────────────────────────

#[test]
fn keyword_bodies_are_printed_correctly() {
    let mut g = main_phase();
    for (def, kw) in [
        (catalog::flying_men(), Keyword::Flying),
        (catalog::moorish_cavalry(), Keyword::Trample),
        (catalog::stone_throwing_devils(), Keyword::FirstStrike),
        (catalog::dancing_scimitar(), Keyword::Flying),
        (catalog::repentant_blacksmith(), Keyword::Protection(Color::Red)),
        (catalog::war_elephant(), Keyword::Banding),
    ] {
        let name = def.name;
        let id = g.add_card_to_battlefield(0, def);
        assert!(
            g.computed_permanent(id).expect(name).keywords.contains(&kw),
            "{name} is missing {kw:?}",
        );
    }
}

// ── Creatures ──────────────────────────────────────────────────────────────

#[test]
fn giant_tortoise_only_hunkers_down_while_untapped() {
    let mut g = main_phase();
    let t = g.add_card_to_battlefield(0, catalog::giant_tortoise());
    assert_eq!(g.computed_permanent(t).expect("tortoise").toughness, 4);
    g.battlefield_find_mut(t).expect("tortoise").tapped = true;
    assert_eq!(g.computed_permanent(t).expect("tortoise").toughness, 1);
}

#[test]
fn serendib_efreet_bleeds_its_controller() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::serendib_efreet());
    upkeep(&mut g);
    assert_eq!(g.players[0].life, 19);
}

#[test]
fn junun_efreet_dies_when_the_rent_goes_unpaid() {
    let mut g = main_phase();
    let e = g.add_card_to_battlefield(0, catalog::junun_efreet());
    upkeep(&mut g);
    assert!(g.battlefield_find(e).is_none(), "no black mana floating, so it goes");
}

#[test]
fn el_hajjaj_drinks_the_damage_it_deals() {
    let mut g = main_phase();
    let ghoul = g.add_card_to_battlefield(0, catalog::el_hajjaj());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(bear),
        1,
        Some(ghoul),
        &mut evs,
    );
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21);
}

#[test]
fn khabal_ghoul_counts_the_turn_s_dead() {
    let mut g = main_phase();
    let ghoul = g.add_card_to_battlefield(0, catalog::khabal_ghoul());
    for seat in 0..2 {
        let bear = g.add_card_to_battlefield(seat, catalog::grizzly_bears());
        g.destroy_permanent(bear, false, &mut Vec::new());
    }
    drain_stack(&mut g);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ghoul).expect("ghoul").counter_count(CounterType::PlusOnePlusOne),
        2,
    );
}

#[test]
fn hasran_ogress_bites_an_unpaid_attack() {
    let mut g = main_phase();
    let ogress = g.add_card_to_battlefield(0, catalog::hasran_ogress());
    g.clear_sickness(ogress);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: ogress, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 17, "no mana floating, so it bites");
}

#[test]
fn sindbad_keeps_lands_and_bins_the_rest() {
    let mut g = main_phase();
    let sindbad = g.add_card_to_battlefield(0, catalog::sindbad());
    g.clear_sickness(sindbad);
    // Top of library is a Mountain.
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, sindbad, None);
    assert_eq!(g.players[0].hand.len(), hand + 1, "a land sticks");
    let bolt = g.next_id();
    g.players[0].add_to_library_top(bolt, catalog::lightning_bolt());
    g.battlefield_find_mut(sindbad).expect("sindbad").tapped = false;
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, sindbad, None);
    assert_eq!(g.players[0].hand.len(), hand, "a nonland is discarded again");
}

#[test]
fn sorceress_queen_shrinks_anything_but_herself() {
    let mut g = main_phase();
    let q = g.add_card_to_battlefield(0, catalog::sorceress_queen());
    g.clear_sickness(q);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, q, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).expect("bear");
    assert_eq!((cp.power, cp.toughness), (0, 2));
}

#[test]
fn singing_tree_mutes_an_attacker() {
    let mut g = main_phase();
    let tree = g.add_card_to_battlefield(1, catalog::singing_tree());
    g.clear_sickness(tree);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    activate(&mut g, 1, tree, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 0);
}

#[test]
fn hurr_jackal_strips_regeneration() {
    let mut g = main_phase();
    let jackal = g.add_card_to_battlefield(0, catalog::hurr_jackal());
    g.clear_sickness(jackal);
    let wall = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, jackal, Some(Target::Permanent(wall)));
    assert!(g.battlefield_find(wall).expect("bear").cant_regenerate_this_turn);
}

#[test]
fn king_suleiman_executes_a_djinn() {
    let mut g = main_phase();
    let king = g.add_card_to_battlefield(0, catalog::king_suleiman());
    g.clear_sickness(king);
    let djinn = g.add_card_to_battlefield(1, catalog::junun_efreet());
    activate(&mut g, 0, king, Some(Target::Permanent(djinn)));
    assert!(g.battlefield_find(djinn).is_none());
}

#[test]
fn brass_man_stays_down_until_you_pay() {
    let mut g = main_phase();
    let man = g.add_card_to_battlefield(0, catalog::brass_man());
    g.battlefield_find_mut(man).expect("brass man").tapped = true;
    upkeep(&mut g);
    assert!(g.battlefield_find(man).expect("brass man").tapped, "nothing floating");
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    upkeep(&mut g);
    assert!(!g.battlefield_find(man).expect("brass man").tapped);
}

#[test]
fn erhnam_djinn_hands_an_opponent_forestwalk() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::erhnam_djinn());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    upkeep(&mut g);
    assert!(
        g.computed_permanent(bear)
            .expect("bear")
            .keywords
            .contains(&Keyword::Landwalk(crabomination::card::LandType::Forest)),
    );
}

#[test]
fn abu_jafar_takes_its_blocker_with_it() {
    let mut g = main_phase();
    let abu = g.add_card_to_battlefield(0, catalog::abu_jafar());
    g.clear_sickness(abu);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: abu, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, abu)])).expect("block");
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(abu).is_none(), "0/1 died");
    assert!(g.battlefield_find(bear).is_none(), "and took its blocker along");
}

#[test]
fn ali_from_cairo_floors_your_life_at_one() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::ali_from_cairo());
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 50, None, &mut evs);
    assert_eq!(g.players[0].life, 1);
}

#[test]
fn aladdin_holds_an_artifact_only_while_he_lives() {
    let mut g = main_phase();
    let aladdin = g.add_card_to_battlefield(0, catalog::aladdin());
    g.clear_sickness(aladdin);
    let chalice = g.add_card_to_battlefield(1, catalog::urzas_chalice());
    activate(&mut g, 0, aladdin, Some(Target::Permanent(chalice)));
    assert_eq!(g.battlefield_find(chalice).expect("chalice").controller, 0);
    g.destroy_permanent(aladdin, false, &mut Vec::new());
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    assert_eq!(g.battlefield_find(chalice).expect("chalice").controller, 1);
}

#[test]
fn old_man_of_the_sea_cant_grab_something_bigger() {
    let mut g = main_phase();
    let old = g.add_card_to_battlefield(0, catalog::old_man_of_the_sea()); // 2/3
    g.clear_sickness(old);
    let big = g.add_card_to_battlefield(1, catalog::junun_efreet()); // 3/3
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: old,
            ability_index: 0,
            target: Some(Target::Permanent(big)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
    );
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    activate(&mut g, 0, old, Some(Target::Permanent(small)));
    assert_eq!(g.battlefield_find(small).expect("bear").controller, 0);
}

#[test]
fn ghazban_ogre_defects_to_the_life_leader() {
    let mut g = main_phase();
    let ogre = g.add_card_to_battlefield(0, catalog::ghazban_ogre());
    g.players[1].life = 30;
    upkeep(&mut g);
    assert_eq!(g.battlefield_find(ogre).expect("ogre").controller, 1);
}

#[test]
fn merchant_ship_needs_an_island_to_sail_at() {
    let mut g = main_phase();
    let ship = g.add_card_to_battlefield(0, catalog::merchant_ship());
    g.add_card_to_battlefield(0, catalog::island());
    g.clear_sickness(ship);
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![Attack { attacker: ship, target: AttackTarget::Player(1) }])
            .is_err(),
        "the defender controls no Island",
    );
    g.add_card_to_battlefield(1, catalog::island());
    assert!(
        g.declare_attackers(vec![Attack { attacker: ship, target: AttackTarget::Player(1) }])
            .is_ok(),
    );
}

#[test]
fn merchant_ship_sinks_without_islands() {
    let mut g = main_phase();
    let ship = g.add_card_to_battlefield(0, catalog::merchant_ship());
    upkeep(&mut g);
    assert!(g.battlefield_find(ship).is_none());
}

#[test]
fn serendib_djinn_eats_a_land_and_bites_for_islands() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::serendib_djinn());
    let isle = g.add_card_to_battlefield(0, catalog::island());
    upkeep(&mut g);
    assert!(g.battlefield_find(isle).is_none(), "the Island was eaten");
    assert_eq!(g.players[0].life, 17);
}

#[test]
fn ifh_biff_efreet_can_be_fired_by_anyone() {
    let mut g = main_phase();
    let efreet = g.add_card_to_battlefield(0, catalog::ifh_biff_efreet());
    let flier = g.add_card_to_battlefield(0, catalog::flying_men());
    activate(&mut g, 1, efreet, None);
    assert!(g.battlefield_find(flier).is_none(), "the 1/1 flier died");
    assert_eq!(g.players[0].life, 19);
    assert_eq!(g.players[1].life, 19);
}

// ── Spells / auras ─────────────────────────────────────────────────────────

#[test]
fn army_of_allah_pumps_only_attackers() {
    let mut g = main_phase();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let idle = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])
        .expect("attack");
    let spell = g.add_card_to_hand(0, catalog::army_of_allah());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.computed_permanent(atk).expect("attacker").power, 4);
    assert_eq!(g.computed_permanent(idle).expect("idle").power, 2);
}

#[test]
fn unstable_mutation_pumps_then_rots() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::unstable_mutation());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 5);
    upkeep(&mut g);
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 4);
}

#[test]
fn fishliver_oil_grants_islandwalk() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::fishliver_oil());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    assert!(
        g.computed_permanent(bear)
            .expect("bear")
            .keywords
            .contains(&Keyword::Landwalk(crabomination::card::LandType::Island)),
    );
}

// ── Artifacts ──────────────────────────────────────────────────────────────

#[test]
fn bottle_of_suleiman_pays_out_on_heads() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let bottle = g.add_card_to_battlefield(0, catalog::bottle_of_suleiman());
    activate(&mut g, 0, bottle, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Djinn").count(), 1);
}

#[test]
fn bottle_of_suleiman_backfires_on_tails() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(false)]));
    let bottle = g.add_card_to_battlefield(0, catalog::bottle_of_suleiman());
    activate(&mut g, 0, bottle, None);
    assert_eq!(g.players[0].life, 15);
}

#[test]
fn ebony_horse_pulls_an_attacker_out_of_the_fight() {
    let mut g = main_phase();
    let horse = g.add_card_to_battlefield(0, catalog::ebony_horse());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    activate(&mut g, 0, horse, Some(Target::Permanent(bear)));
    assert!(!g.battlefield_find(bear).expect("bear").tapped);
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, 20, "its combat damage was prevented");
}

#[test]
fn jandors_ring_trades_the_last_draw() {
    let mut g = main_phase();
    let ring = g.add_card_to_battlefield(0, catalog::jandors_ring());
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    let drawn = g.players[0].last_drawn_card.expect("drew");
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, ring, None);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == drawn));
    assert_eq!(g.players[0].hand.len(), hand, "one out, one in");
}

#[test]
fn city_in_a_bottle_sweeps_and_locks_arabian_nights() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::city_in_a_bottle());
    let djinn = g.add_card_to_battlefield(1, catalog::junun_efreet());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    upkeep(&mut g);
    assert!(g.battlefield_find(djinn).is_none(), "an ARN card is sacrificed");
    assert!(g.battlefield_find(bear).is_some(), "a non-ARN card is spared");
    let men = g.add_card_to_hand(0, catalog::flying_men());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: men,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "and they can't be cast either",
    );
}

#[test]
fn cuombajj_witches_hand_the_opponent_the_second_point() {
    let mut g = main_phase();
    let witches = g.add_card_to_battlefield(0, catalog::cuombajj_witches());
    g.clear_sickness(witches);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Player(0))]));
    activate(&mut g, 0, witches, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 19, "the controller's point");
    assert_eq!(g.players[0].life, 19, "and the opponent's, aimed back");
}

#[test]
fn metamorphosis_turns_a_body_into_creature_only_mana() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // {1}{G}, MV 2
    let meta = g.add_card_to_hand(0, catalog::metamorphosis());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: meta,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].mana_pool.restricted_total(),
        3,
        "1 + the sacrificed creature's mana value, creature-spells only",
    );
}

#[test]
fn drop_of_honey_culls_the_smallest_then_expires() {
    let mut g = main_phase();
    let drop = g.add_card_to_battlefield(0, catalog::drop_of_honey());
    let small = g.add_card_to_battlefield(1, catalog::flying_men()); // 1/1
    let big = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    upkeep(&mut g);
    assert!(g.battlefield_find(small).is_none());
    assert!(g.battlefield_find(big).is_some());
    g.destroy_permanent(big, false, &mut Vec::new());
    drain_stack(&mut g);
    upkeep(&mut g);
    assert!(g.battlefield_find(drop).is_none(), "no creatures left, so it goes");
}

#[test]
fn cyclone_escalates_then_blows_over() {
    let mut g = main_phase();
    let cyc = g.add_card_to_battlefield(0, catalog::cyclone());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    upkeep(&mut g);
    assert_eq!(g.battlefield_find(cyc).expect("cyclone").counter_count(CounterType::Wind), 1);
    assert_eq!(g.players[0].life, 19);
    assert_eq!(g.battlefield_find(bear).expect("bear").damage, 1);
    // Second upkeep with nothing floating: it blows itself out.
    upkeep(&mut g);
    assert!(g.battlefield_find(cyc).is_none());
}

// ── Lands ──────────────────────────────────────────────────────────────────

#[test]
fn bazaar_of_baghdad_digs_two_and_pitches_three() {
    let mut g = main_phase();
    let bazaar = g.add_card_to_battlefield(0, catalog::bazaar_of_baghdad());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, bazaar, None);
    assert_eq!(g.players[0].hand.len(), hand + 2 - 3);
}

#[test]
fn diamond_valley_cashes_a_creature_for_its_toughness() {
    let mut g = main_phase();
    let valley = g.add_card_to_battlefield(0, catalog::diamond_valley());
    g.add_card_to_battlefield(0, catalog::dancing_scimitar()); // 1/5
    activate(&mut g, 0, valley, None);
    assert_eq!(g.players[0].life, 25);
}

#[test]
fn library_of_alexandria_only_draws_at_exactly_seven() {
    let mut g = main_phase();
    let lib = g.add_card_to_battlefield(0, catalog::library_of_alexandria());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: lib,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
    );
    while g.players[0].hand.len() < 7 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    let hand = g.players[0].hand.len();
    activate_n(&mut g, 0, lib, 1, None);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

#[test]
fn desert_pings_an_attacker_at_end_of_combat() {
    let mut g = main_phase();
    let des = g.add_card_to_battlefield(1, catalog::desert());
    let bear = g.add_card_to_battlefield(0, catalog::flying_men()); // 1/1
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::EndCombat;
    activate_n(&mut g, 1, des, 1, Some(Target::Permanent(bear)));
    let _ = g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn island_of_wak_wak_grounds_a_flier() {
    let mut g = main_phase();
    let isle = g.add_card_to_battlefield(0, catalog::island_of_wak_wak());
    let flier = g.add_card_to_battlefield(1, catalog::dancing_scimitar()); // 1/5 flying
    activate(&mut g, 0, isle, Some(Target::Permanent(flier)));
    assert_eq!(g.computed_permanent(flier).expect("scimitar").power, 0);
}

#[test]
fn elephant_graveyard_shields_an_elephant() {
    let mut g = main_phase();
    let yard = g.add_card_to_battlefield(0, catalog::elephant_graveyard());
    let ele = g.add_card_to_battlefield(0, catalog::war_elephant());
    activate_n(&mut g, 0, yard, 1, Some(Target::Permanent(ele)));
    assert_eq!(g.battlefield_find(ele).expect("elephant").regeneration_shields, 1);
}

#[test]
fn desert_nomads_walk_past_deserts_and_shrug_off_their_pings() {
    let mut g = main_phase();
    let nomads = g.add_card_to_battlefield(0, catalog::desert_nomads());
    let des = g.add_card_to_battlefield(1, catalog::desert());
    g.clear_sickness(nomads);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: nomads, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::EndCombat;
    activate_n(&mut g, 1, des, 1, Some(Target::Permanent(nomads)));
    assert_eq!(g.battlefield_find(nomads).expect("nomads").damage, 0);
}

// ── Third wave ─────────────────────────────────────────────────────────────

#[test]
fn magnetic_mountain_pins_blue_creatures_down() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::magnetic_mountain());
    let blue = g.add_card_to_battlefield(1, catalog::flying_men());
    g.battlefield_find_mut(blue).expect("flier").tapped = true;
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(blue).expect("flier").tapped, "blue doesn't untap");
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
    ]));
    g.players[1].mana_pool.add_colorless(4);
    upkeep(&mut g);
    assert!(!g.battlefield_find(blue).expect("flier").tapped, "four generic bought it back up");
}

#[test]
fn guardian_beast_shields_your_artifacts_while_it_stands() {
    let mut g = main_phase();
    let beast = g.add_card_to_battlefield(0, catalog::guardian_beast());
    let chalice = g.add_card_to_battlefield(0, catalog::urzas_chalice());
    assert!(
        g.computed_permanent(chalice)
            .expect("chalice")
            .keywords
            .contains(&Keyword::Indestructible),
    );
    g.battlefield_find_mut(beast).expect("beast").tapped = true;
    assert!(
        !g.computed_permanent(chalice)
            .expect("chalice")
            .keywords
            .contains(&Keyword::Indestructible),
        "tapping the Beast drops the shield",
    );
}

#[test]
fn sandals_of_abdallah_break_with_their_wearer() {
    let mut g = main_phase();
    let sandals = g.add_card_to_battlefield(0, catalog::sandals_of_abdallah());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, sandals, Some(Target::Permanent(bear)));
    assert!(
        g.computed_permanent(bear)
            .expect("bear")
            .keywords
            .contains(&Keyword::Landwalk(crabomination::card::LandType::Island)),
    );
    let mut evs = Vec::new();
    g.destroy_permanent(bear, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(sandals).is_none());
}

#[test]
fn nafs_asp_bills_the_victim_at_their_draw_step() {
    let mut g = main_phase();
    let asp = g.add_card_to_battlefield(0, catalog::nafs_asp());
    g.clear_sickness(asp);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: asp, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.turn_number == 1 {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    while g.step != TurnStep::PreCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, 20 - 1 - 1, "one from the bite, one from the venom");
}

#[test]
fn jihad_swells_while_the_chosen_colour_is_out() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Color(Color::Green)]));
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let knight = g.add_card_to_battlefield(0, catalog::moorish_cavalry()); // white 3/3
    let jihad = g.add_card_to_hand(0, catalog::jihad());
    cast(&mut g, 0, jihad, None);
    let cp = g.computed_permanent(knight).expect("knight");
    assert_eq!((cp.power, cp.toughness), (5, 4));
    g.destroy_permanent(bear, false, &mut Vec::new());
    drain_stack(&mut g);
    upkeep(&mut g);
    assert!(g.battlefield_find(jihad).is_none(), "no green left, so it goes");
}

#[test]
fn eye_for_an_eye_pays_the_next_hit_back() {
    let mut g = main_phase();
    let eye = g.add_card_to_hand(0, catalog::eye_for_an_eye());
    cast(&mut g, 0, eye, None);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 17);
    assert_eq!(g.players[1].life, 17);
    // One mirror, one hit.
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt2, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 14);
    assert_eq!(g.players[1].life, 17);
}

#[test]
fn aladdins_lamp_digs_x_deep_for_the_next_draw() {
    let mut g = main_phase();
    g.players[0].library.clear();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::mountain());
    }
    let wanted = g.add_card_to_library(0, catalog::grizzly_bears());
    let lamp = g.add_card_to_battlefield(0, catalog::aladdins_lamp());
    g.clear_sickness(lamp);
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(wanted))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: lamp,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(3),
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let mut events = vec![];
    assert!(g.draw_one(0, &mut events));
    // The dig kept the picked card on top; the other two were bottomed.
    assert!(g.players[0].hand.iter().any(|c| c.id == wanted));
}

#[test]
fn cr_729_shahrazad_bleeds_every_player_who_does_not_win_the_subgame() {
    let mut g = main_phase();
    let shah = g.add_card_to_hand(0, catalog::shahrazad());
    cast(&mut g, 0, shah, None);
    // Exactly one seat keeps its life; the other pays half, rounded up.
    let paid: Vec<usize> = (0..2).filter(|&p| g.players[p].life != 20).collect();
    assert!(paid.len() <= 2 && !paid.is_empty());
    for &p in &paid {
        assert_eq!(g.players[p].life, 10);
    }
}
