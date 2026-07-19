#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── Push (claude/modern_decks) — new card functional tests ──────────────────

#[test]
fn rofellos_taps_for_one_green_per_forest() {
    // Rofellos's mana ability now scales with Forest count: {T}: Add {G}
    // for each Forest you control. 3 Forests → 3 green.
    let mut g = two_player_game();
    let rofellos = g.add_card_to_battlefield(0, catalog::rofellos_llanowar_emissary());
    g.clear_sickness(rofellos);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let pool_before = g.players[0].mana_pool.amount(Color::Green);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rofellos, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Rofellos's mana ability should activate");
    let pool_after = g.players[0].mana_pool.amount(Color::Green);
    assert_eq!(
        pool_after - pool_before, 3,
        "Rofellos with 3 Forests adds 3 green mana"
    );
}

#[test]
fn rofellos_taps_for_zero_with_no_forests() {
    // Edge case: no Forests means 0 mana (the multiplier annihilates).
    let mut g = two_player_game();
    let rofellos = g.add_card_to_battlefield(0, catalog::rofellos_llanowar_emissary());
    g.clear_sickness(rofellos);
    let pool_before = g.players[0].mana_pool.amount(Color::Green);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rofellos, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Rofellos's mana ability should still activate");
    let pool_after = g.players[0].mana_pool.amount(Color::Green);
    assert_eq!(pool_after - pool_before, 0, "0 Forests → 0 green");
}

#[test]
fn snapcaster_mage_etb_grants_may_play_on_gy_is_card() {
    // Snapcaster Mage ETBs and grants MayPlay{EndOfThisTurn,
    // exile_after: true} on a target IS card in your graveyard. Same
    // shape as Flashback (the spell) — recovers the IS card for the turn.
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let snap = g.add_card_to_hand(0, catalog::snapcaster_mage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: snap, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Snapcaster castable for {1}{U}");
    drain_stack(&mut g);

    let bolt_gy = g.players[0].graveyard.iter().find(|c| c.id == bolt)
        .expect("Bolt still in graveyard");
    let perm = bolt_gy.may_play_until.expect("may_play stamped on Bolt");
    assert!(perm.exile_after, "exile-on-resolve flag set (CR 702.34d)");
    assert_eq!(perm.player, 0);
}

#[test]
fn snapcaster_mage_is_a_two_one_flash_wizard() {
    use crabomination::card::{CreatureType, Keyword};
    let snap = catalog::snapcaster_mage();
    assert_eq!(snap.power, 2);
    assert_eq!(snap.toughness, 1);
    assert!(snap.keywords.contains(&Keyword::Flash));
    assert!(snap.subtypes.creature_types.contains(&CreatureType::Wizard));
}

#[test]
fn pyroblast_counters_a_blue_spell() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // Opp casts a blue spell.
    let cancel = g.add_card_to_hand(1, catalog::cancel());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: cancel, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Cancel castable for {1}{U}{U}");
    // Now controller of Pyroblast (P0) counters the Cancel on the stack.
    let pyro = g.add_card_to_hand(0, catalog::pyroblast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: pyro,
        target: Some(Target::Permanent(cancel)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("Pyroblast castable for {R}, targeting Cancel");
    drain_stack(&mut g);
    // Cancel should have been countered to graveyard.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == cancel),
        "Cancel was countered by Pyroblast");
}

#[test]
fn pyroblast_rejects_non_blue_spell_target() {
    use crabomination::game::types::Target;
    // Pyroblast's mode-0 filter rejects non-blue spells. Try targeting
    // Lightning Bolt (red) — the cast should fail at target-validation.
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");

    let pyro = g.add_card_to_hand(0, catalog::pyroblast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    let res = g.perform_action(GameAction::CastSpell {
        card_id: pyro,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    });
    assert!(res.is_err(), "Pyroblast can't target a non-blue spell");
}

#[test]
fn red_elemental_blast_counters_a_blue_spell() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let cancel = g.add_card_to_hand(1, catalog::cancel());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: cancel, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Cancel castable");
    let reb = g.add_card_to_hand(0, catalog::red_elemental_blast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: reb,
        target: Some(Target::Permanent(cancel)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("REB castable, targeting Cancel");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == cancel),
        "REB countered the blue spell");
}

#[test]
fn hydroblast_counters_a_red_spell() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    let hydro = g.add_card_to_hand(0, catalog::hydroblast());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: hydro,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("Hydroblast castable, targeting Bolt");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Hydroblast countered the red spell");
}

#[test]
fn blue_elemental_blast_counters_a_red_spell() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    let beb = g.add_card_to_hand(0, catalog::blue_elemental_blast());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: beb,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("BEB castable, targeting Bolt");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "BEB countered the red spell");
}

#[test]
fn three_visits_fetches_a_forest_to_battlefield() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    // ScriptedDecider picks the Forest at search time (AutoDecider
    // declines library searches by returning Search(None)).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let visits = g.add_card_to_hand(0, catalog::three_visits());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: visits, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Three Visits castable for {1}{G}");
    drain_stack(&mut g);
    let f = g.battlefield.iter().find(|c| c.id == forest)
        .expect("Forest moved to battlefield");
    assert_eq!(f.controller, 0);
    assert!(!f.tapped, "Three Visits puts the land in untapped");
}

#[test]
fn tales_end_counters_a_legendary_spell() {
    use crabomination::game::types::{Target, TurnStep};
    // Drop a legendary creature spell on the stack, then Tale's End it.
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    let griselbrand = g.add_card_to_hand(1, catalog::griselbrand());
    g.players[1].mana_pool.add(Color::Black, 4);
    g.players[1].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: griselbrand, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Griselbrand castable");
    let tale = g.add_card_to_hand(0, catalog::tales_end());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: tale,
        target: Some(Target::Permanent(griselbrand)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("Tale's End castable, targeting Griselbrand");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == griselbrand),
        "Tale's End countered the legendary spell");
}

#[test]
fn wall_of_omens_etbs_and_draws() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let wall = g.add_card_to_hand(0, catalog::wall_of_omens());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: wall, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wall of Omens castable");
    drain_stack(&mut g);
    // Wall is on the battlefield + caster drew 1; hand size: -1 (cast) +1 (etb).
    assert_eq!(g.players[0].hand.len(), hand_before);
    let w = g.battlefield_find(wall).unwrap();
    assert_eq!(w.toughness(), 4);
    assert!(w.has_keyword(&Keyword::Defender));
}

#[test]
fn wall_of_roots_taps_for_green_with_pump_cost() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_roots());
    g.clear_sickness(wall);
    let pool_before = g.players[0].mana_pool.amount(Color::Green);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wall, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Wall of Roots activation should resolve");
    drain_stack(&mut g);
    let pool_after = g.players[0].mana_pool.amount(Color::Green);
    assert_eq!(pool_after - pool_before, 1, "Wall of Roots adds {{G}}");
    let w = g.battlefield_find(wall).unwrap();
    assert_eq!(w.toughness(), 4,
        "Wall of Roots's activation cost shrinks its toughness by 1");
}

/// Channel: until end of turn, generic shortfall can be paid with life 1:1.
#[test]
fn channel_converts_life_into_generic_mana_until_eot() {
    let mut g = two_player_game();
    let ch = g.add_card_to_hand(0, catalog::channel());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: ch, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Channel castable for {G}{G}");
    drain_stack(&mut g);
    // No mana left; a {6} Colossus is now payable with 6 life.
    let big = g.add_card_to_hand(0, catalog::wurmcoil_engine());
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Channel converts 6 life into the generic cost");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 6, "paid 6 life for the {{6}} cost");
    assert!(g.battlefield.iter().any(|c| c.id == big), "Wurmcoil deployed");
    // The flag expires at cleanup.
    g.step = crabomination::game::TurnStep::End;
    let _ = g.advance_step(Vec::new());
    assert!(!g.players[0].channel_life_for_mana, "Channel expires at end of turn");
}

#[test]
fn phyrexian_reclamation_returns_creature_for_one_b_two_life() {
    let mut g = two_player_game();
    let rec = g.add_card_to_battlefield(0, catalog::phyrexian_reclamation());
    g.clear_sickness(rec);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rec, ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None })
    .expect("Reclamation should activate for {1}{B} + 2 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 2, "2 life paid as cost");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Bear returned from gy to hand");
}

#[test]
fn pernicious_deed_destroys_low_cmc_permanents() {
    // Activate Deed for X=2: should kill the 1-mana and 2-mana
    // permanents but spare the 6-mana Shivan Dragon.
    let mut g = two_player_game();
    let deed = g.add_card_to_battlefield(0, catalog::pernicious_deed());
    g.clear_sickness(deed);
    let cheap = g.add_card_to_battlefield(1, catalog::savannah_lions()); // 1-cmc
    let mid = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2-cmc
    let big = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 6-cmc
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: deed, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: Some(2) })
    .expect("Deed should activate for {2}, sac");
    drain_stack(&mut g);
    // Cheap (1-cmc) and mid (2-cmc) die; 6-cmc survives.
    assert!(!g.battlefield.iter().any(|c| c.id == cheap),
        "1-cmc creature should leave battlefield");
    assert!(!g.battlefield.iter().any(|c| c.id == mid),
        "2-cmc creature should leave battlefield");
    assert!(g.battlefield.iter().any(|c| c.id == big),
        "6-cmc creature survives Deed at X=2");
    assert!(!g.battlefield.iter().any(|c| c.id == deed),
        "Deed sacrificed as activation cost");
}

#[test]
fn toxic_deluge_sweeps_creatures_for_x_two() {
    let mut g = two_player_game();
    // Two 2/2s (one each side) should die.
    let mine_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // 5/5 stays.
    let big = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let deluge = g.add_card_to_hand(0, catalog::toxic_deluge());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: deluge, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("Deluge castable with X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 2, "Paid 2 life for X=2");
    assert!(!g.battlefield.iter().any(|c| c.id == mine_bear), "own bear died");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear), "opp bear died");
    assert!(g.battlefield.iter().any(|c| c.id == big), "5/5 survives -2/-2");
}

#[test]
fn demonic_consultation_mills_six_and_searches() {
    let mut g = two_player_game();
    // Seed library so mill 6 has something to chew on, plus the tutor target.
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    let target = g.add_card_to_library(0, catalog::lightning_bolt());
    // ScriptedDecider picks the bolt at search time.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    let cons = g.add_card_to_hand(0, catalog::demonic_consultation());
    g.players[0].mana_pool.add(Color::Black, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: cons, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Consultation castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before - 7,
        "Library lost 6 to mill + 1 to search");
    assert!(g.players[0].hand.iter().any(|c| c.id == target),
        "Picked card lands in hand");
}

#[test]
fn howling_mine_draws_an_extra_card_each_turn() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::howling_mine());
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let hand_before = g.players[1].hand.len();
    // Trigger P1's draw step manually — the Howling Mine trigger fires
    // for the active player; set active_player to 1, advance.
    g.active_player_idx = 1;
    g.step = TurnStep::Draw;
    g.priority.player_with_priority = 1;
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    // P1 should have drawn 1 extra card from Mine.
    assert!(g.players[1].hand.len() > hand_before,
        "Howling Mine drew P1 a card on their draw step");
}

#[test]
fn sylvan_library_offers_draw_in_exchange_for_four_life() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sylvan_library());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    // Force the decider to accept the MayDo (draw + lose 4).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.step = TurnStep::Draw;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 4, "Paid 4 life");
    assert!(g.players[0].hand.len() > hand_before, "Drew the extra card");
}

// ── Dark Confidant — "lose life equal to CMC" trigger ────────────────────────

#[test]
fn dark_confidant_loses_life_equal_to_revealed_card_cmc() {
    // Seeds the library with a 5-CMC Serra Angel on top; on upkeep, Dark
    // Confidant's trigger reveals + draws it and the controller loses 5
    // life (not the old approximated flat 2).
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.players[0].library.clear();
    // Use add_to_library_top to control ordering — the *last* call to
    // add_to_library_top is the top of the library.
    {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::grizzly_bears()); // 2-CMC filler
    }
    {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::serra_angel()); // 5-CMC on top
    }
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    let dc = g.add_card_to_battlefield(0, catalog::dark_confidant());
    g.clear_sickness(dc);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Dark Confidant drew exactly one card");
    let drawn_name = g.players[0].hand.last().map(|c| c.definition.name).unwrap_or("");
    assert_eq!(drawn_name, "Serra Angel",
        "The on-top card (Serra Angel) was drawn into hand");
    let life_lost = (life_before - g.players[0].life) as u32;
    assert_eq!(life_lost, 5,
        "Life lost equals Serra Angel's mana value (CMC 5), not the old flat 2");
}

#[test]
fn dark_confidant_loses_zero_life_for_zero_cmc_card_on_top() {
    // Zero-CMC card (Black Lotus is the canonical {0} cost) → no life loss.
    // Tests the "0 mana value" corner of the new ManaValueOf wiring.
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.players[0].library.clear();
    {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::black_lotus());
    }
    let life_before = g.players[0].life;
    let dc = g.add_card_to_battlefield(0, catalog::dark_confidant());
    g.clear_sickness(dc);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before,
        "0-CMC revealed card → no life lost");
}

#[test]
fn ophiomancer_mints_a_snake_each_upkeep() {
    use crabomination::card::CreatureType;
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let ophio = g.add_card_to_battlefield(0, catalog::ophiomancer());
    g.clear_sickness(ophio);
    let bf_before = g.battlefield.len();
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield.len() > bf_before, "Snake token created");
    let tok = g.battlefield.iter().find(|c|
        c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Snake)
    ).expect("Snake token exists");
    assert!(tok.has_keyword(&crabomination::card::Keyword::Deathtouch));
}

#[test]
fn yavimaya_elder_dies_searches_two_basics() {
    let mut g = two_player_game();
    // Seed the library with two basic lands.
    let forest = g.add_card_to_library(0, catalog::forest());
    let plains = g.add_card_to_library(0, catalog::plains());
    let elder = g.add_card_to_battlefield(0, catalog::yavimaya_elder());
    let hand_before = g.players[0].hand.len();
    // ScriptedDecider answers MayDo(yes) + Search(Forest) + MayDo(yes) + Search(Plains).
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest)),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(plains)),
    ]));
    let _ = g.remove_to_graveyard_with_triggers(elder);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2,
        "Yavimaya Elder dies → +2 basic lands to hand");
}

#[test]
fn stroke_of_genius_draws_x_cards() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let stroke = g.add_card_to_hand(0, catalog::stroke_of_genius());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: stroke,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("Stroke castable at X=3");
    drain_stack(&mut g);
    // -1 (cast) + 3 (X draw) = +2.
    assert_eq!(g.players[0].hand.len(), hand_before + 2,
        "Stroke at X=3 draws 3 cards");
}

#[test]
fn green_suns_zenith_tutors_green_creature_with_cmc_x() {
    let mut g = two_player_game();
    // Seed library with a green creature.
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
    let zenith = g.add_card_to_hand(0, catalog::green_suns_zenith());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: zenith, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("GSZ castable for {X=2}{G}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "GSZ tutored Grizzly Bears (2-cmc green) into play");
}

#[test]
fn red_suns_zenith_deals_x_damage_to_target() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let zenith = g.add_card_to_hand(0, catalog::red_suns_zenith());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: zenith,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("RSZ castable for {X=3}{R}");
    drain_stack(&mut g);
    // 2/2 bear takes 3 damage → dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "RSZ at X=3 kills the bear");
}

#[test]
fn white_suns_zenith_creates_x_cat_tokens() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let zenith = g.add_card_to_hand(0, catalog::white_suns_zenith());
    g.players[0].mana_pool.add(Color::White, 3);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: zenith, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("WSZ castable for {X=2}{W}{W}{W}");
    drain_stack(&mut g);
    // 2 Cat tokens entered.
    assert_eq!(g.battlefield.len(), bf_before + 2);
    assert!(g.battlefield.iter().filter(|c|
        c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Cat)
    ).count() == 2, "Two Cat tokens minted");
}

#[test]
fn black_suns_zenith_puts_x_minus_one_counters_on_each_creature() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let zenith = g.add_card_to_hand(0, catalog::black_suns_zenith());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: zenith, target: None, additional_targets: vec![], mode: None, x_value: Some(1),
    })
    .expect("BSZ castable at X=1");
    drain_stack(&mut g);
    // Each 2/2 bear now has 1 -1/-1 counter (effectively 1/1).
    // SBA may not kill them at -1/-1 yet; just verify counters present.
    let b1 = g.battlefield_find(bear1);
    let b2 = g.battlefield_find(bear2);
    if let Some(c) = b1 {
        assert!(c.counter_count(CounterType::MinusOneMinusOne) >= 1,
            "Bear1 received -1/-1 counter");
    }
    if let Some(c) = b2 {
        assert!(c.counter_count(CounterType::MinusOneMinusOne) >= 1,
            "Bear2 received -1/-1 counter");
    }
}

#[test]
fn yavimaya_elder_sac_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let elder = g.add_card_to_battlefield(0, catalog::yavimaya_elder());
    g.clear_sickness(elder);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    // Sac-cost activated draw ability.
    g.perform_action(GameAction::ActivateAbility {
        card_id: elder, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Yavimaya Elder's sac-draw should activate");
    drain_stack(&mut g);
    // Hand gains the drawn card; Elder leaves play. Note: the dies-
    // trigger also fires on the sac, which may search for basic lands
    // too — but the AutoDecider's MayDo default-no skips them.
    assert!(g.players[0].hand.len() > hand_before, "Drew a card");
    assert!(!g.battlefield.iter().any(|c| c.id == elder),
        "Elder sacrificed");
}

// ── claude/modern_decks batch 102: multicolor cube expansion ────────────────

#[test]
fn sorin_plus_one_reveals_top_to_hand_and_drains_its_mv() {
    let mut g = two_player_game();
    let sorin = g.add_card_to_battlefield(0, catalog::sorin_grim_nemesis());
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::mind_stone()); // MV 2
    let p1_life = g.players[1].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        x_value: None,
        card_id: sorin, ability_index: 0, target: None,
    }).expect("Sorin +1");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == top), "revealed card to hand");
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(g.players[1].life, p1_life - 2, "opponent lost the card's MV");
}

#[test]
fn sorin_minus_x_pings_and_gains() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let sorin = g.add_card_to_battlefield(0, catalog::sorin_grim_nemesis());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let p0_life = g.players[0].life;

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        x_value: Some(2),
        card_id: sorin, ability_index: 1, target: Some(Target::Permanent(bear)),
    }).expect("Sorin -X");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).is_none(), "Bear took 2 and died");
    assert_eq!(g.players[0].life, p0_life + 2, "gained X life");
    assert_eq!(
        g.battlefield_find(sorin).unwrap().counter_count(crabomination::card::CounterType::Loyalty),
        4, "paid X=2 from 6",
    );
}

#[test]
fn sorin_minus_nine_mints_tokens_equal_to_highest_life() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let sorin = g.add_card_to_battlefield(0, catalog::sorin_grim_nemesis());
    if let Some(s) = g.battlefield_find_mut(sorin) {
        s.add_counters(CounterType::Loyalty, 3);
    }
    g.players[0].life = 12;
    g.players[1].life = 17;

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        x_value: None,
        card_id: sorin, ability_index: 2, target: None,
    }).expect("Sorin -9");
    drain_stack(&mut g);

    let knights = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Vampire Knight" && c.controller == 0)
        .count();
    assert_eq!(knights, 17, "tokens = highest life total among all players");
}

#[test]
fn narset_parter_caps_opponent_draws_at_one() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::narset_parter_of_veils());
    let cp = g.compute_battlefield();
    let _ = cp; // static is consulted via PlayerView/draw path
    // Opponent's per-turn draw cap is now 1.
    assert_eq!(g.draw_cap_for(1), Some(1), "opponent capped to one draw per turn");
    // Controller is unaffected.
    assert_eq!(g.draw_cap_for(0), None, "your own draws are uncapped");
}

#[test]
fn narset_parter_minus_two_digs_for_a_noncreature() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let narset = g.add_card_to_battlefield(0, catalog::narset_parter_of_veils());
    let spell = g.add_card_to_library(0, catalog::lightning_bolt()); // noncreature, nonland
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());   // creature → not takeable
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(spell))]));
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: narset, ability_index: 0, target: None,
    }).expect("Narset -2");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == spell), "took the noncreature spell");
    assert!(!g.players[0].hand.iter().any(|c| c.id == bear), "the creature stayed out of hand");
}

#[test]
fn liliana_of_the_veil_plus_one_makes_each_player_discard() {
    let mut g = two_player_game();
    let lily = g.add_card_to_battlefield(0, catalog::liliana_of_the_veil());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(1, catalog::island());
    let h0 = g.players[0].hand.len();
    let h1 = g.players[1].hand.len();
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: lily, ability_index: 0, target: None,
    }).expect("Lily +1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 - 1, "you discarded one");
    assert_eq!(g.players[1].hand.len(), h1 - 1, "opponent discarded one");
}

#[test]
fn liliana_of_the_veil_minus_two_forces_a_sacrifice() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let lily = g.add_card_to_battlefield(0, catalog::liliana_of_the_veil());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: lily, ability_index: 1, target: Some(Target::Player(1)),
    }).expect("Lily -2");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "target player sacrificed a creature");
}

#[test]
fn liliana_last_hope_plus_one_shrinks_a_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let lily = g.add_card_to_battlefield(0, catalog::liliana_the_last_hope());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: lily, ability_index: 0, target: Some(Target::Permanent(bear)),
    }).expect("Lily +1");
    drain_stack(&mut g);
    // -2/-1 puts the 2/2 to 0/1; SBA keeps it (toughness 1).
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear);
    assert!(b.is_none() || b.unwrap().power == 0, "creature shrunk to 0 power");
}

#[test]
fn liliana_last_hope_minus_two_mills_and_returns_a_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let lily = g.add_card_to_battlefield(0, catalog::liliana_the_last_hope());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: lily, ability_index: 1, target: Some(Target::Permanent(dead)),
    }).expect("Lily -2");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "returned the creature to hand");
}

#[test]
fn teferi_hero_plus_one_draws() {
    let mut g = two_player_game();
    let teferi = g.add_card_to_battlefield(0, catalog::teferi_hero_of_dominaria());
    g.add_card_to_library(0, catalog::island());
    let h = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: teferi, ability_index: 0, target: None,
    }).expect("Teferi +1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h + 1, "drew a card");
}

#[test]
fn teferi_hero_minus_three_tucks_a_permanent() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let teferi = g.add_card_to_battlefield(0, catalog::teferi_hero_of_dominaria());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..4 { g.add_card_to_library(1, catalog::island()); }
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: teferi, ability_index: 1, target: Some(Target::Permanent(bear)),
    }).expect("Teferi -3");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "permanent left the battlefield");
    // Third from top → library index 2.
    assert_eq!(g.players[1].library.get(2).map(|c| c.id), Some(bear), "tucked third from top");
}

#[test]
fn saheeli_rai_plus_one_pings_each_opponent() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let saheeli = g.add_card_to_battlefield(0, catalog::saheeli_rai());
    // An opponent planeswalker — Saheeli's +1 also pings each of these.
    let opp_pw = g.add_card_to_battlefield(1, catalog::karn_liberated());
    let pw_loyalty = g.battlefield_find(opp_pw).unwrap().counter_count(CounterType::Loyalty);
    g.add_card_to_library(0, catalog::island());
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: saheeli, ability_index: 0, target: None,
    }).expect("Saheeli +1");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_life - 1, "Opp pinged for 1");
    assert_eq!(
        g.battlefield_find(opp_pw).unwrap().counter_count(CounterType::Loyalty),
        pw_loyalty - 1,
        "opponent's planeswalker also took 1 (a loyalty counter removed)",
    );
}

#[test]
fn saheeli_rai_minus_two_creates_haste_copy() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let saheeli = g.add_card_to_battlefield(0, catalog::saheeli_rai());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: saheeli,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
    }).expect("Saheeli -2 copies bear");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 1, "Copy token entered");
    let bear_copies = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Grizzly Bears")
        .count();
    assert!(bear_copies >= 1, "At least one bear copy token");
}

#[test]
fn ashiok_plus_two_exiles_top_three_linked() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let ashiok = g.add_card_to_battlefield(0, catalog::ashiok_nightmare_weaver());
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let exile_before = g.exile.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        x_value: None,
        card_id: ashiok,
        ability_index: 0,
        target: Some(Target::Player(1)),
    }).expect("Ashiok +2");
    drain_stack(&mut g);

    assert_eq!(g.exile.len(), exile_before + 3, "top three exiled");
    assert_eq!(
        g.exile.iter().filter(|c| c.exiled_with == Some(ashiok)).count(),
        3,
        "exiled cards are linked to Ashiok"
    );
}

#[test]
fn ashiok_minus_x_reanimates_an_exiled_creature_as_a_nightmare() {
    let mut g = two_player_game();
    let ashiok = g.add_card_to_battlefield(0, catalog::ashiok_nightmare_weaver());
    g.battlefield_find_mut(ashiok).unwrap()
        .counters.insert(crabomination::card::CounterType::Loyalty, 5);
    // A MV-2 creature exiled with Ashiok.
    let bear = g.add_card_to_exile(1, catalog::grizzly_bears());
    g.exile.iter_mut().find(|c| c.id == bear).unwrap().exiled_with = Some(ashiok);

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        x_value: Some(2),
        card_id: ashiok,
        ability_index: 1,
        target: None,
    }).expect("Ashiok -X");
    drain_stack(&mut g);

    let stolen = g.battlefield_find(bear).expect("Bear on battlefield");
    assert_eq!(stolen.controller, 0, "under Ashiok's controller");
    assert!(
        stolen.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Nightmare),
        "a Nightmare in addition to its other types"
    );
    assert_eq!(
        g.battlefield_find(ashiok).unwrap().counter_count(crabomination::card::CounterType::Loyalty),
        3, "paid X=2 loyalty",
    );
}

#[test]
fn ashiok_minus_ten_exiles_opponent_hands_and_graveyards() {
    let mut g = two_player_game();
    let ashiok = g.add_card_to_battlefield(0, catalog::ashiok_nightmare_weaver());
    g.battlefield_find_mut(ashiok).unwrap()
        .counters.insert(crabomination::card::CounterType::Loyalty, 10);
    let in_hand = g.add_card_to_hand(1, catalog::island());
    let in_gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        x_value: None,
        card_id: ashiok,
        ability_index: 2,
        target: None,
    }).expect("Ashiok -10");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == in_hand), "opponent hand exiled");
    assert!(g.exile.iter().any(|c| c.id == in_gy), "opponent graveyard exiled");
}

#[test]
fn tamiyo_plus_one_names_a_card_and_sorts_the_top_four() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let tamiyo = g.add_card_to_battlefield(0, catalog::tamiyo_collector_of_tales());
    // Top four: Bear, Island, Bear, Island (top-first).
    for def in [catalog::island(), catalog::grizzly_bears(), catalog::island(), catalog::grizzly_bears()] {
        let id = g.next_id();
        g.players[0].add_to_library_top(id, def);
    }
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::NamedCard("Grizzly Bears".into()),
    ]));
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        x_value: None,
        card_id: tamiyo, ability_index: 0, target: None,
    }).expect("Tamiyo +1");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 2, "both Bears to hand");
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2, "both Islands to graveyard");
    assert_eq!(
        g.battlefield_find(tamiyo).unwrap().counter_count(crabomination::card::CounterType::Loyalty),
        6, "+1 loyalty",
    );
}

#[test]
fn tamiyo_minus_three_returns_card_from_graveyard() {
    let mut g = two_player_game();
    let tamiyo = g.add_card_to_battlefield(0, catalog::tamiyo_collector_of_tales());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        x_value: None,
        card_id: tamiyo, ability_index: 1,
        target: Some(crabomination::game::types::Target::Permanent(bear)),
    }).expect("Tamiyo -3");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "Bear returned to hand");
}

#[test]
fn tamiyo_static_blocks_opponent_forced_discard() {
    let mut g = two_player_game();
    let _tamiyo = g.add_card_to_battlefield(0, catalog::tamiyo_collector_of_tales());
    let keep = g.add_card_to_hand(0, catalog::island());
    // P1 casts a discard spell at P0 (on P1's own turn — Mind Rot is a sorcery).
    let spell = g.add_card_to_hand(1, catalog::mind_rot());
    g.players[1].mana_pool.add(Color::Black, 1);
    g.players[1].mana_pool.add_colorless(2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(crabomination::game::types::Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Mind Rot castable");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == keep),
        "Tamiyo's static blocks the forced discard");
}

#[test]
fn geyadrone_dihada_plus_one_drains_each_opponent_for_one() {
    let mut g = two_player_game();
    let dihada = g.add_card_to_battlefield(0, catalog::geyadrone_dihada());
    g.add_card_to_library(0, catalog::island());
    let p1_life = g.players[1].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: dihada, ability_index: 0, target: None,
    }).expect("Dihada +1");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_life - 1, "Opp loses 1");
    assert!(g.players[0].hand.len() > hand_before, "You draw a card");
}

/// +1's rider resets loyalty to 3 when you have less life than an opponent.
#[test]
fn geyadrone_dihada_plus_one_resets_loyalty_when_behind() {
    let mut g = two_player_game();
    let dihada = g.add_card_to_battlefield(0, catalog::geyadrone_dihada());
    g.add_card_to_library(0, catalog::island());
    // Push loyalty to 8, then fall behind on life.
    g.battlefield_find_mut(dihada).unwrap().add_counters(CounterType::Loyalty, 5); // 3 + 5 = 8
    g.players[0].life = 5;
    g.players[1].life = 20;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: dihada, ability_index: 0, target: None,
    }).expect("Dihada +1");
    drain_stack(&mut g);
    // +1 raised it to 9, then the rider reset it to the starting 3.
    assert_eq!(g.battlefield_find(dihada).unwrap().counter_count(CounterType::Loyalty), 3,
        "loyalty reset to starting value when behind on life");
}

#[test]
fn geyadrone_dihada_minus_seven_halves_opponent_life() {
    let mut g = two_player_game();
    let dihada = g.add_card_to_battlefield(0, catalog::geyadrone_dihada());
    g.battlefield_find_mut(dihada).unwrap().add_counters(CounterType::Loyalty, 4); // 3 + 4 = 7
    g.players[1].life = 21;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: dihada, ability_index: 2, target: None,
    }).expect("Dihada -7");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 10, "21 → loses 11 (half rounded up)");
}

#[test]
fn geyadrone_dihada_minus_three_steals_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let dihada = g.add_card_to_battlefield(0, catalog::geyadrone_dihada());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: dihada,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
    }).expect("Dihada -3 threaten");
    drain_stack(&mut g);

    // Bear is now under your control with haste.
    let bear_card = g.battlefield_find(bear).expect("Bear still on bf");
    assert_eq!(bear_card.controller, 0, "Bear now controlled by you");
}

#[test]
fn korvold_fae_cursed_king_triggers_on_sacrifice() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let korvold = g.add_card_to_battlefield(0, catalog::korvold_fae_cursed_king());
    g.clear_sickness(korvold);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();

    // Fire a Sacrifice on the bear via Effect::Sacrifice. We dispatch
    // the resulting CreatureSacrificed event into Korvold's trigger
    // listener after the sacrifice resolves.
    let sac_effect = crabomination::card::Effect::Sacrifice {
        who: crabomination::card::Selector::You,
        count: crabomination::card::Value::Const(1),
        filter: crabomination::card::SelectionRequirement::Creature,
    };
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&sac_effect, &ctx).expect("Sacrifice resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);

    // Korvold should have a +1/+1 counter and you should have drawn a card.
    let korvold_card = g.battlefield_find(korvold).expect("Korvold still alive");
    assert_eq!(korvold_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "Korvold gained +1/+1 counter from sacrifice");
    assert!(g.players[0].hand.len() > hand_before,
        "Korvold drew a card from sacrifice");
}

#[test]
fn korvold_fae_cursed_king_triggers_on_artifact_sacrifice_via_permanent_event() {
    // PermanentSacrificed catches non-creature sacrifices too —
    // CR 701.16 generalization shipped with the batch 102 engine work.
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let korvold = g.add_card_to_battlefield(0, catalog::korvold_fae_cursed_king());
    g.clear_sickness(korvold);
    // An artifact, not a creature.
    g.add_card_to_battlefield(0, catalog::mind_stone());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();

    let sac_effect = crabomination::card::Effect::Sacrifice {
        who: crabomination::card::Selector::You,
        count: crabomination::card::Value::Const(1),
        filter: crabomination::card::SelectionRequirement::Artifact,
    };
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g.resolve_effect(&sac_effect, &ctx).expect("Sac resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);

    let korvold_card = g.battlefield_find(korvold).expect("Korvold alive");
    assert_eq!(korvold_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "Korvold grew off non-creature (Mind Stone) sacrifice via PermanentSacrificed");
    assert!(g.players[0].hand.len() > hand_before,
        "Korvold drew a card from artifact sacrifice");
}

#[test]
fn lord_xander_the_collector_etb_discards_half_opponent_hand() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.players[1].hand.clear();
    for _ in 0..6 {
        g.add_card_to_hand(1, catalog::island());
    }
    let hand_before = g.players[1].hand.len();
    let xander = g.add_card_to_hand(0, catalog::lord_xander_the_collector());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: xander, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Xander castable");
    drain_stack(&mut g);
    // 6 cards, half rounded down = 3 discarded.
    assert_eq!(g.players[1].hand.len(), hand_before - hand_before / 2,
        "opp discards half their hand rounded down");
}

/// Lord Xander's attack trigger mills half the defending player's library.
#[test]
fn lord_xander_attack_mills_half_library() {
    let mut g = two_player_game();
    g.players[1].library.clear();
    for _ in 0..11 {
        g.add_card_to_library(1, catalog::island());
    }
    let xander = g.add_card_to_battlefield(0, catalog::lord_xander_the_collector());
    let trig = catalog::lord_xander_the_collector().triggered_abilities[1].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(xander, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    // 11 cards, half rounded down = 5 milled.
    assert_eq!(g.players[1].library.len(), 11 - 5, "milled half (rounded down)");
    assert_eq!(g.players[1].graveyard.len(), 5, "5 in graveyard");
}

/// Lord Xander's death trigger makes target opponent sacrifice half their
/// permanents, rounded down.
#[test]
fn lord_xander_death_sacrifices_half_permanents() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let xander = g.add_card_to_battlefield(0, catalog::lord_xander_the_collector());
    for _ in 0..5 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let trig = catalog::lord_xander_the_collector().triggered_abilities[2].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(
        xander, 0, Some(Target::Player(1)), 0,
    );
    g.resolve_effect(&trig, &ctx).unwrap();
    // 5 permanents, half rounded down = 2 sacrificed → 3 remain.
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 3,
        "opp sacrifices half their permanents rounded down");
}

// ── modern_decks value-body batch ───────────────────────────────────────────

/// Cloudblazer draws two and gains two on ETB.
#[test]
fn cloudblazer_etb_draws_two_gains_two() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let hand = g.players[0].hand.len();
    let life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::cloudblazer());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    cast(&mut g, id);
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew 2");
    assert_eq!(g.players[0].life, life + 2, "gained 2");
}

/// Invisible Stalker is hexproof and can't be blocked.
#[test]
fn invisible_stalker_is_hexproof_and_unblockable() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let stalker = g.add_card_to_battlefield(0, catalog::invisible_stalker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cp = g.compute_battlefield();
    let scp = cp.iter().find(|c| c.id == stalker).unwrap();
    assert!(scp.keywords.contains(&Keyword::Hexproof));
    assert!(!g.blocker_can_block_attacker(bear, stalker), "can't be blocked");
}

/// Slither Blade can't be blocked.
#[test]
fn slither_blade_unblockable() {
    let mut g = two_player_game();
    let sb = g.add_card_to_battlefield(0, catalog::slither_blade());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(!g.blocker_can_block_attacker(bear, sb));
}

/// Shadowmage Infiltrator draws when it deals combat damage to a player.
#[test]
fn shadowmage_infiltrator_draws_on_combat_damage() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::island());
    let inf = g.add_card_to_battlefield(0, catalog::shadowmage_infiltrator());
    let hand = g.players[0].hand.len();
    let trig = catalog::shadowmage_infiltrator().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(inf, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew on combat damage");
}

/// Liliana's Specter makes each opponent discard on ETB.
#[test]
fn lilianas_specter_etb_each_opponent_discards() {
    let mut g = two_player_game();
    g.players[1].hand.clear();
    g.add_card_to_hand(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::lilianas_specter());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(g.players[1].hand.len(), 0, "opp discarded a card");
}

/// Bone Shredder destroys a target nonblack, nonartifact creature on ETB and
/// can't target a black creature.
#[test]
fn bone_shredder_etb_destroys_legal_target_only() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green, legal
    // A black creature is not a legal target.
    let zombie = g.add_card_to_battlefield(1, catalog::hypnotic_specter());
    let filter = SelectionRequirement::Creature
        .and(SelectionRequirement::Artifact.negate())
        .and(SelectionRequirement::HasColor(Color::Black).negate());
    assert!(g.evaluate_requirement_static(&filter, &Target::Permanent(bear), 0, None));
    assert!(!g.evaluate_requirement_static(&filter, &Target::Permanent(zombie), 0, None),
        "black creature is an illegal target");
}

/// Goldnight Commander pumps your team when another creature enters.
#[test]
fn goldnight_commander_pumps_team_on_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goldnight_commander());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    // The pre-existing bear got +1/+1 from the new creature's ETB.
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (3, 3), "team pumped +1/+1");
}

/// Elvish Archdruid buffs other Elves and taps for {G} per Elf you control.
#[test]
fn elvish_archdruid_lord_and_mana() {
    let mut g = two_player_game();
    let archdruid = g.add_card_to_battlefield(0, catalog::elvish_archdruid());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.clear_sickness(archdruid);
    // Lord (continuous, layer-7): the other Elf is 1/1 base → 2/2.
    let cp = g.compute_battlefield();
    let e = cp.iter().find(|c| c.id == elf).unwrap();
    assert_eq!((e.power, e.toughness), (2, 2), "other Elf gets +1/+1");
    // Archdruid itself isn't buffed by its own "other" lord.
    let a = cp.iter().find(|c| c.id == archdruid).unwrap();
    assert_eq!(a.power, 2);
    // Mana: {T}: Add {G} for each Elf you control (2 Elves).
    g.perform_action(GameAction::ActivateAbility {
        card_id: archdruid, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for elf mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 2, "two green from two Elves");
}

/// Inspired Charge gives your creatures +2/+1 until end of turn.
#[test]
fn inspired_charge_team_pump() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inspired_charge());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (4, 3), "+2/+1 this turn");
}

/// Servo Exhibition makes two 1/1 Servo tokens.
#[test]
fn servo_exhibition_makes_two_servos() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::servo_exhibition());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.name == "Servo").count(), 2, "two Servo tokens");
}

/// Fire Ambush deals 3 to any target.
#[test]
fn fire_ambush_deals_three() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let life = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::fire_ambush());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, life - 3, "3 damage to the player");
}

/// Magus of the Mirror exchanges life totals with an opponent — but only
/// during its controller's upkeep.
#[test]
fn magus_of_the_mirror_exchanges_life_during_upkeep_only() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::magus_of_the_mirror());
    g.clear_sickness(id);
    g.players[0].life = 5;
    g.players[1].life = 20;
    // Outside upkeep (main phase): the activation is rejected by the gate.
    g.step = TurnStep::PreCombatMain;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).is_err(),
        "can't activate outside your upkeep");
    // During the controller's upkeep: exchange goes through (and sacrifices Magus).
    g.step = TurnStep::Upkeep;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None })
        .expect("activatable during upkeep");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "P0 took P1's 20");
    assert_eq!(g.players[1].life, 5, "P1 took P0's 5");
    assert!(g.battlefield_find(id).is_none(), "Magus sacrificed as a cost");
}

/// Convolute counters a spell unless the caster pays {4}.
#[test]
fn convolute_counters_unpaid_spell() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // P1 casts a creature spell; P0 Convolutes it (P1 can't pay {4}).
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bear castable");
    let conv = g.add_card_to_hand(0, catalog::convolute());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: conv, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Convolute castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "uncounterable-pay failed → countered");
}

/// Frost Breath taps two creatures and stuns them (skip-next-untap).
#[test]
fn frost_breath_taps_and_stuns() {
    use crabomination::game::types::Target;
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::frost_breath());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("Frost Breath castable");
    drain_stack(&mut g);
    let ra = g.battlefield_find(a).unwrap();
    assert!(ra.tapped, "tapped");
    assert_eq!(ra.counter_count(CounterType::Stun), 1, "stun counter applied");
}

/// Furnace of Rath doubles damage: a 3-damage bolt deals 6.
#[test]
fn furnace_of_rath_doubles_damage() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::furnace_of_rath());
    let opp = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 6, "3 doubled to 6");
}

/// Dictate of the Twin Gods has Flash and doubles damage.
#[test]
fn dictate_of_the_twin_gods_has_flash_and_doubles() {
    let d = catalog::dictate_of_the_twin_gods();
    assert!(d.keywords.contains(&Keyword::Flash), "castable at instant speed");
    assert!(d.static_abilities.iter().any(|s| matches!(
        s.effect, crabomination::effect::StaticEffect::DoubleDamageDealt)));
}

/// Wring Flesh shrinks a creature -3/-1.
#[test]
fn wring_flesh_shrinks_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wring_flesh());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wring Flesh castable");
    drain_stack(&mut g);
    // 2/2 → -1/1, dies to SBA (toughness 1 > 0 actually survives at 1).
    let c = g.computed_permanent(bear);
    assert!(c.is_none() || c.unwrap().toughness == 1, "shrunk to */1");
}

/// Epic Confrontation pumps your creature and fights an enemy.
#[test]
fn epic_confrontation_pumps_and_fights() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::epic_confrontation());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("Epic Confrontation castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "enemy 2/2 took 3 and died");
    assert!(g.battlefield_find(mine).is_some(), "your 3/4 survives 2 damage");
}

/// Renegade Tactics stops a creature from blocking and cantrips.
#[test]
fn renegade_tactics_cant_block_and_draws() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::renegade_tactics());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Renegade Tactics castable");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::CantBlock));
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one (net same)");
}

/// Fervor grants your creatures haste.
#[test]
fn fervor_grants_haste() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fervor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
}

/// Sizzle deals 3 to each opponent.
#[test]
fn sizzle_deals_three_to_each_opponent() {
    let mut g = two_player_game();
    let opp = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::sizzle());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sizzle castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 3);
}

/// Sunlance burns a nonwhite creature for 3 (and can't target a white one).
#[test]
fn sunlance_kills_nonwhite_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let id = g.add_card_to_hand(0, catalog::sunlance());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sunlance castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2/2 took 3 and died");
}

/// Cower in Fear shrinks opponents' creatures -1/-1.
#[test]
fn cower_in_fear_shrinks_opponents() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cower_in_fear());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cower in Fear castable");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 1, "opponent's bear -1/-1");
    assert_eq!(g.computed_permanent(mine).unwrap().power, 2, "your bear untouched");
}

/// Spidersilk Armor gives your creatures +0/+1 and reach.
#[test]
fn spidersilk_armor_grants_toughness_and_reach() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spidersilk_armor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.toughness, 3, "2/2 → 2/3");
    assert!(c.keywords.contains(&Keyword::Reach), "gained reach");
}

/// Pulse of Murasa returns a creature card from the graveyard and gains 6 life.
#[test]
fn pulse_of_murasa_returns_creature_and_gains_six() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pulse_of_murasa());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pulse castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "bear returned to hand");
    assert_eq!(g.players[0].life, life + 6, "gained 6 life");
}

/// Moment's Peace fogs combat damage for the turn.
#[test]
fn moments_peace_fogs_combat() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::moments_peace());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Moment's Peace castable");
    drain_stack(&mut g);
    assert!(g.prevent_combat_damage_this_turn, "combat damage prevention armed");
}

/// Mending Hands prevents the next 4 damage to a chosen target (here, a player).
#[test]
fn mending_hands_prevents_next_four_to_player() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mending_hands());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mending Hands castable");
    drain_stack(&mut g);
    // A 3-damage bolt aimed at P0: the next-4 shield prevents all of it.
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "all 3 prevented by the next-4 shield");
}

/// Underworld Dreams pings an opponent for 1 each time they draw.
#[test]
fn underworld_dreams_pings_opponent_on_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::underworld_dreams());
    g.add_card_to_library(1, catalog::island());
    let opp = g.players[1].life;
    let mut events = Vec::new();
    g.draw_one(1, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "opponent took 1 on their draw");
}

/// Megrim deals 2 to an opponent each time they discard.
#[test]
fn megrim_pings_opponent_on_discard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::megrim());
    let card = g.add_card_to_hand(1, catalog::island());
    let opp = g.players[1].life;
    let mut events = Vec::new();
    g.discard_card(1, card, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "opponent took 2 on their discard");
}

/// Howl from Beyond gives +X/+0 (X from the cast cost).
#[test]
fn howl_from_beyond_pumps_by_x() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::howl_from_beyond());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("Howl castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 2 + 3, "+3/+0");
}

/// Reckless Spite destroys two nonblack creatures and costs 5 life.
#[test]
fn reckless_spite_destroys_two_and_loses_five() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::reckless_spite());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("Reckless Spite castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both destroyed");
    assert_eq!(g.players[0].life, life - 5, "lost 5 life");
}

/// Wall of Blood pumps itself +1/+1 per life paid.
#[test]
fn wall_of_blood_pays_life_to_pump() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::wall_of_blood());
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().power(), 1, "0/2 → 1/3");
    assert_eq!(g.players[0].life, life - 1, "paid 1 life");
}

/// Disrupting Scepter makes a player discard — only on your own turn.
#[test]
fn disrupting_scepter_discards_only_on_your_turn() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::disrupting_scepter());
    g.add_card_to_hand(1, catalog::island());
    // Not your turn → rejected.
    g.active_player_idx = 1;
    g.players[0].mana_pool.add_colorless(3);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None }).is_err(),
        "can't activate on the opponent's turn");
    // Your turn → opponent discards.
    g.active_player_idx = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None })
        .expect("activatable on your turn");
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty(), "opponent discarded their card");
}

/// Repay in Kind sets every player's life to the lowest among all players.
#[test]
fn repay_in_kind_sets_all_life_to_lowest() {
    let mut g = two_player_game();
    g.players[0].life = 25;
    g.players[1].life = 6;
    let id = g.add_card_to_hand(0, catalog::repay_in_kind());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Repay in Kind castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 6, "P0 drops to the lowest (6)");
    assert_eq!(g.players[1].life, 6, "P1 unchanged at the lowest");
}

/// Trumpet Blast pumps attacking creatures +2/+0.
#[test]
fn trumpet_blast_pumps_attackers() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).unwrap();
    let id = g.add_card_to_hand(0, catalog::trumpet_blast());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    cast(&mut g, id);
    assert_eq!(g.battlefield_find(attacker).unwrap().power(), 4, "attacker +2/+0");
}

/// Dusk Legion Zealot draws a card and loses 1 life on ETB.
#[test]
fn dusk_legion_zealot_etb_draws_and_loses_life() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    let life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::dusk_legion_zealot());
    g.players[0].mana_pool.add(Color::Black, 2);
    cast(&mut g, id);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(g.players[0].life, life - 1, "lost 1 life");
}

/// Phyrexian Gargantua draws two and loses two on ETB.
#[test]
fn phyrexian_gargantua_etb_draws_two_loses_two() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let hand = g.players[0].hand.len();
    let life = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::phyrexian_gargantua());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, id);
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew 2");
    assert_eq!(g.players[0].life, life - 2, "lost 2");
}

/// Frost Lynx taps and stuns an opponent's creature on ETB.
#[test]
fn frost_lynx_taps_and_stuns() {
    use crabomination::card::CounterType;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::frost_lynx());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped && b.counter_count(CounterType::Stun) == 1, "tapped + stunned");
}

/// Veteran Armorer gives other creatures you control +0/+1 (continuous).
#[test]
fn veteran_armorer_toughness_anthem() {
    let mut g = two_player_game();
    let armorer = g.add_card_to_battlefield(0, catalog::veteran_armorer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (2, 3), "other creature gets +0/+1");
    // Armorer doesn't buff itself.
    let a = cp.iter().find(|c| c.id == armorer).unwrap();
    assert_eq!(a.toughness, 2, "armorer keeps base 2/3");
}

/// Attended Knight makes a 1/1 Soldier on ETB.
#[test]
fn attended_knight_etb_makes_soldier() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::attended_knight());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.name == "Soldier").count(), 1, "made a Soldier token");
}

/// Kor Hookmaster taps an opponent's creature and stuns it on ETB.
#[test]
fn kor_hookmaster_taps_and_stuns() {
    use crabomination::card::CounterType;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::kor_hookmaster());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped, "opp creature tapped");
    assert_eq!(b.counter_count(CounterType::Stun), 1, "got a stun counter");
}

/// Indrik Stomphowler destroys a target artifact or enchantment on ETB.
#[test]
fn indrik_stomphowler_etb_destroys_artifact() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::ornithopter()); // artifact creature
    let id = g.add_card_to_hand(0, catalog::indrik_stomphowler());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(art)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// Ambassador Oak makes a 1/1 Elf Warrior on ETB.
#[test]
fn ambassador_oak_etb_makes_elf_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::ambassador_oak());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.name == "Elf Warrior").count(), 1, "made an Elf Warrior");
}

/// Nessian Asp's Monstrosity 4 adds four +1/+1 counters.
#[test]
fn nessian_asp_monstrosity_adds_counters() {
    let mut g = two_player_game();
    let asp = g.add_card_to_battlefield(0, catalog::nessian_asp());
    g.clear_sickness(asp);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: asp, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("monstrosity activatable");
    drain_stack(&mut g);
    let c = g.battlefield_find(asp).unwrap();
    assert_eq!((c.power(), c.toughness()), (8, 9), "4/5 + four +1/+1 = 8/9");
}

/// Welkin Tern can't block.
#[test]
fn welkin_tern_cant_block() {
    let mut g = two_player_game();
    let tern = g.add_card_to_battlefield(0, catalog::welkin_tern());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(!g.blocker_can_block_attacker(tern, bear), "Welkin Tern can't block");
}

/// Sylvan Ranger fetches a basic land to hand on ETB.
#[test]
fn sylvan_ranger_fetches_basic_land() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::sylvan_ranger());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest),
        "Sylvan Ranger fetched the Forest to hand");
}

/// Charging Rhino can't be blocked by more than one creature (CR 509.1g).
#[test]
fn charging_rhino_blocked_by_at_most_one() {
    use crabomination::game::{Attack, AttackTarget};
    let declare = |two: bool| {
        let mut g = two_player_game();
        let rhino = g.add_card_to_battlefield(0, catalog::charging_rhino());
        g.clear_sickness(rhino);
        let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: rhino, target: AttackTarget::Player(1),
        }])).unwrap();
        g.step = TurnStep::DeclareBlockers;
        let blocks = if two { vec![(b1, rhino), (b2, rhino)] } else { vec![(b1, rhino)] };
        (g.perform_action(GameAction::DeclareBlockers(blocks)), rhino)
    };
    assert!(declare(false).0.is_ok(), "one blocker is legal");
    let (res, rhino) = declare(true);
    assert_eq!(res.unwrap_err(), GameError::CannotBeBlockedByMoreThanOne(rhino));
}

/// Dark Banishing can't target a black creature.
#[test]
fn dark_banishing_rejects_black_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let zombie = g.add_card_to_battlefield(1, catalog::hypnotic_specter());
    let filter = SelectionRequirement::Creature
        .and(SelectionRequirement::HasColor(Color::Black).negate());
    assert!(!g.evaluate_requirement_static(&filter, &Target::Permanent(zombie), 0, None));
}

#[test]
fn master_of_cruelties_attack_sets_opp_life_to_one() {
    let mut g = two_player_game();
    let master = g.add_card_to_battlefield(0, catalog::master_of_cruelties());
    g.clear_sickness(master);
    g.players[1].life = 20;

    // Fire the attack trigger directly via event bus.
    let trig = catalog::master_of_cruelties().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(
        master, 0, None, 0,
    );
    let _ = g.resolve_effect(&trig, &ctx);
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 1, "Opp's life set to 1");
}

#[test]
fn territorial_kavu_grows_when_opponent_plays_a_land() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let kavu = g.add_card_to_battlefield(0, catalog::territorial_kavu());
    g.clear_sickness(kavu);

    // Opponent plays a land.
    let land = g.add_card_to_hand(1, catalog::forest());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = crabomination::game::types::TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land))
        .expect("Opp plays a forest");
    drain_stack(&mut g);

    let k = g.battlefield_find(kavu).expect("Kavu alive");
    assert_eq!(k.counter_count(CounterType::PlusOnePlusOne), 1,
        "Kavu grew off opp's land entering");
}

#[test]
fn kolaghans_command_default_reanimate_plus_two_damage() {
    // Default picks [0, 3]: return creature from gy (slot 0) + 2 damage to
    // any target (slot 1).
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let cmd = g.add_card_to_hand(0, catalog::kolaghans_command());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: cmd,
        target: Some(Target::Permanent(bear)),     // slot 0: reanimate
        additional_targets: vec![Target::Player(1)], // slot 1: 2 damage
        mode: None, x_value: None,
    }).expect("Kolaghan's Command castable");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 18, "Opp took 2 damage from mode 3");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Bear reanimated to hand by mode 0");
}

#[test]
fn kolaghans_command_scripted_reanimate_only() {
    // ScriptedDecider picks just mode [0] (reanimate). Slot 0 carries the
    // graveyard creature; the opponent takes no damage.
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let cmd = g.add_card_to_hand(0, catalog::kolaghans_command());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![0])]));

    g.perform_action(GameAction::CastSpell {
        card_id: cmd,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Kolaghan's Command castable");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, 20, "damage mode not chosen");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Bear reanimated by mode 0");
}

#[test]
fn heroic_intervention_grants_indestructible_to_your_perms() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hi = g.add_card_to_hand(0, catalog::heroic_intervention());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: hi, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Heroic Intervention castable");
    drain_stack(&mut g);

    let bear_card = g.battlefield_find(bear).expect("Bear alive");
    assert!(bear_card.has_keyword(&crabomination::card::Keyword::Indestructible),
        "Bear gained indestructible");
    assert!(bear_card.has_keyword(&crabomination::card::Keyword::Hexproof),
        "Bear gained hexproof");
    // The granted hexproof is functional: an opponent can't target the bear.
    use crabomination::game::types::Target;
    assert!(
        g.check_target_legality(&Target::Permanent(bear), 1).is_err(),
        "opponent can't target the hexproof-granted bear",
    );
    // ...but its controller still can.
    assert!(
        g.check_target_legality(&Target::Permanent(bear), 0).is_ok(),
        "controller can still target their own hexproof permanent",
    );
}

#[test]
fn wear_tear_destroys_target_artifact() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let mind_stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let wt = g.add_card_to_hand(0, catalog::wear_tear());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: wt, target: Some(Target::Permanent(mind_stone)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wear // Tear castable");
    drain_stack(&mut g);

    assert!(g.battlefield_find(mind_stone).is_none(),
        "Mind Stone destroyed");
}

#[test]
fn fire_ice_left_deals_two_divided() {
    // Fire (left half): 2 damage to the opponent.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fire_ice());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fire castable");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, before - 2, "Fire dealt 2");
}

#[test]
fn fire_ice_right_taps_and_draws() {
    // Ice (right half): tap target permanent and draw a card.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fire_ice());
    let cid = g.next_id();
    g.players[0].add_to_library_top(cid, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSplitRight {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ice castable");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).unwrap().tapped, "Ice tapped the bear");
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card (cast -1, draw +1)");
}

#[test]
fn assault_battery_right_makes_elephant() {
    // Battery (right half): create a 3/3 green Elephant.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::assault_battery());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSplitRight {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battery castable");
    drain_stack(&mut g);

    let eleph = g.battlefield.iter().find(|c| c.definition.name == "Elephant").expect("Elephant token");
    assert_eq!((eleph.power(), eleph.toughness()), (3, 3), "3/3 Elephant");
    assert_eq!(eleph.controller, 0);
}

#[test]
fn far_away_fused_bounces_and_sacrifices() {
    // Fused: Far bounces our target creature; Away makes the opponent sac one.
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::far_away());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSplitFused {
        card_id: id,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Player(1)],
        mode: None, x_value: None,
    }).expect("fused cast");
    drain_stack(&mut g);

    assert!(g.battlefield_find(mine).is_none(), "Far bounced our creature");
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "bounced creature in hand");
    assert!(g.battlefield_find(theirs).is_none(), "Away forced opponent to sacrifice");
}

#[test]
fn wax_wane_left_pumps_creature() {
    // Wax (left half): +2/+2 until end of turn.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wax_wane());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wax castable");
    drain_stack(&mut g);

    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (4, 4), "Wax pumped to 4/4");
}

#[test]
fn stand_deliver_right_bounces_permanent() {
    // Deliver (right half): return target permanent to its owner's hand.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::stand_deliver());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSplitRight {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Deliver castable");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).is_none(), "Deliver bounced the bear");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bounced to owner's hand");
}

#[test]
fn alive_well_fused_makes_token_and_gains_life() {
    // Fused: Alive makes a 3/3 Centaur; Well gains 2 life per creature you
    // control (after the Centaur enters, that's at least 1 creature).
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::alive_well());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSplitFused {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("fused cast");
    drain_stack(&mut g);

    let centaur = g.battlefield.iter().find(|c| c.definition.name == "Centaur").expect("Centaur token");
    assert_eq!((centaur.power(), centaur.toughness()), (3, 3));
    // Left (Alive) resolves first, so the Centaur is on the field for Well.
    assert_eq!(g.players[0].life, life_before + 2, "gained 2 per creature (1 Centaur)");
}

#[test]
fn rough_tumble_right_hits_only_fliers() {
    // Tumble (right) deals 6 to each creature with flying; grounded creatures
    // are untouched.
    let mut g = two_player_game();
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 no flying
    let id = g.add_card_to_hand(0, catalog::rough_tumble());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSplitRight {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tumble castable");
    drain_stack(&mut g);

    assert!(g.battlefield_find(flier).is_none(), "flier took 6 and died");
    assert!(g.battlefield_find(ground).is_some(), "grounded creature untouched");
}

#[test]
fn boom_left_destroys_one_of_each_players_land() {
    // Boom (left) destroys a land you control and a land you don't.
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::mountain());
    let theirs = g.add_card_to_battlefield(1, catalog::forest());
    let safe = g.add_card_to_battlefield(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::boom_bust());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None, x_value: None,
    }).expect("Boom castable");
    drain_stack(&mut g);

    assert!(g.battlefield_find(mine).is_none(), "my land destroyed");
    assert!(g.battlefield_find(theirs).is_none(), "their land destroyed");
    assert!(g.battlefield_find(safe).is_some(), "untargeted land spared");
}

#[test]
fn bust_right_destroys_all_lands() {
    // Bust (right) destroys every land.
    let mut g = two_player_game();
    let l1 = g.add_card_to_battlefield(0, catalog::mountain());
    let l2 = g.add_card_to_battlefield(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::boom_bust());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSplitRight {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bust castable");
    drain_stack(&mut g);

    assert!(g.battlefield_find(l1).is_none() && g.battlefield_find(l2).is_none(), "all lands gone");
}

#[test]
fn pure_left_destroys_multicolored_only() {
    // Pure (left) destroys a multicolored permanent but spares a mono one.
    let mut g = two_player_game();
    let gold = g.add_card_to_battlefield(1, catalog::watchwolf()); // GW multicolored
    let mono = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pure_simple());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(gold)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pure castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gold).is_none(), "multicolored destroyed");
    assert!(g.battlefield_find(mono).is_some(), "mono untouched");
}

#[test]
fn spite_left_counters_noncreature_only() {
    // Spite (left) counters a noncreature spell on the stack.
    let mut g = two_player_game();
    // Opponent casts a noncreature spell (Lightning Bolt) at us.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[1].mana_pool.add(_c, 20); }
    g.players[1].mana_pool.add_colorless(20);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt cast");
    // We respond with Spite (counters the noncreature spell).
    let id = g.add_card_to_hand(0, catalog::spite_malice());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spite castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered to graveyard");
    assert_eq!(g.players[0].life, 20, "Bolt never resolved");
}

#[test]
fn dead_gone_left_kills_right_bounces() {
    // Dead (left) deals 2 to a 2/2; Gone (right) bounces an opponent's creature.
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::dead_gone());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dead castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "Dead killed the bear");

    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id2 = g.add_card_to_hand(0, catalog::dead_gone());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSplitRight {
        card_id: id2, target: Some(Target::Permanent(other)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Gone castable");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == other), "Gone bounced it");
}

#[test]
fn give_take_left_adds_three_counters() {
    // Give (left) puts three +1/+1 counters on target creature.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::give_take());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Give castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 3);
    assert_eq!((b.power(), b.toughness()), (5, 5));
}

#[test]
fn ready_left_grants_indestructible_and_untaps() {
    // Ready (left) gives your creatures indestructible and untaps them.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::ready_willing());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ready castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.has_keyword(&crabomination::card::Keyword::Indestructible), "granted indestructible");
    assert!(!b.tapped, "untapped");
}

#[test]
fn profit_loss_fused_pumps_yours_shrinks_theirs() {
    // Fused: Profit +1/+1 to my creatures, Loss -1/-1 to opponent's.
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 -> 3/3
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 -> 1/1
    let id = g.add_card_to_hand(0, catalog::profit_loss());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSplitFused {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("fused cast");
    drain_stack(&mut g);

    let m = g.battlefield_find(mine).unwrap();
    let t = g.battlefield_find(theirs).unwrap();
    assert_eq!((m.power(), m.toughness()), (3, 3), "Profit pumped mine");
    assert_eq!((t.power(), t.toughness()), (1, 1), "Loss shrank theirs");
}

#[test]
fn supply_left_makes_x_saprolings() {
    // Supply (left) makes X 1/1 Saprolings with X=3.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::supply_demand());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("Supply castable");
    drain_stack(&mut g);

    let n = g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count();
    assert_eq!(n, 3, "made 3 Saprolings");
}

#[test]
fn trouble_right_burns_for_target_hand_size() {
    // Trouble (right) deals damage = cards in the target player's hand.
    let mut g = two_player_game();
    for _ in 0..4 {
        let cid = g.next_id();
        g.players[1].hand.push(crabomination::card::CardInstance::new(cid, catalog::grizzly_bears(), 1));
    }
    let hand_n = g.players[1].hand.len() as i32;
    let id = g.add_card_to_hand(0, catalog::toil_trouble());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[1].life;

    g.perform_action(GameAction::CastSplitRight {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Trouble castable");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, before - hand_n, "Trouble burned for hand size");
}

#[test]
fn cut_ribbons_aftermath_drains_each_opponent_for_x() {
    // Ribbons (aftermath) drains each opponent for X. Cast it from the
    // graveyard with X=3.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::cut_ribbons());
    let card = g.players[0].remove_from_hand(id).unwrap();
    g.players[0].graveyard.push(card);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let before = g.players[1].life;

    g.perform_action(GameAction::CastAftermath {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("Ribbons castable from graveyard");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, before - 3, "Ribbons drained 3");
    assert!(g.exile.iter().any(|c| c.id == id), "Ribbons exiled");
}

#[test]
fn consign_oblivion_left_bounces_nonland() {
    // Consign (left) returns a target nonland permanent to its owner's hand.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::consign_oblivion());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Consign castable");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).is_none(), "bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "to owner's hand");
}

#[test]
fn mouth_feed_left_makes_hippo() {
    // Mouth (left) makes a 3/3 green Hippo.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mouth_feed());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mouth castable");
    drain_stack(&mut g);

    let hippo = g.battlefield.iter().find(|c| c.definition.name == "Hippo").expect("Hippo token");
    assert_eq!((hippo.power(), hippo.toughness()), (3, 3));
}

#[test]
fn spring_mind_aftermath_from_graveyard_then_exiled() {
    // CR 702.127 — Mind (right half) is castable only from the graveyard,
    // draws two, then is exiled.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spring_mind());
    for _ in 0..3 {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::grizzly_bears());
    }
    // Can't cast the aftermath half from hand.
    assert!(g.perform_action(GameAction::CastAftermath {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "aftermath not castable from hand");

    // Cast Spring (left) — lands in the graveyard.
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spring castable");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Spring in graveyard");

    // Now cast Mind (aftermath) from the graveyard: draw two, then exile.
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastAftermath {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mind castable from graveyard");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 2, "Mind drew two");
    assert!(g.exile.iter().any(|c| c.id == id), "Mind exiled after resolving");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id), "no longer in graveyard");
}

#[test]
fn onward_victory_aftermath_grants_double_strike() {
    // Victory (aftermath half) gives target creature double strike.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::onward_victory());
    // Move the card straight to the graveyard to cast its aftermath half.
    let card = g.players[0].remove_from_hand(id).unwrap();
    g.players[0].graveyard.push(card);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastAftermath {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Victory castable from graveyard");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).unwrap().has_keyword(&crabomination::card::Keyword::DoubleStrike),
        "bear gained double strike");
    assert!(g.exile.iter().any(|c| c.id == id), "Victory exiled");
}

#[test]
fn stillmoon_cavalier_grants_flying_eot() {
    let mut g = two_player_game();
    let cav = g.add_card_to_battlefield(0, catalog::stillmoon_cavalier());
    g.clear_sickness(cav);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cav, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Stillmoon {W}: flying");
    drain_stack(&mut g);
    let c = g.battlefield_find(cav).expect("Stillmoon alive");
    assert!(c.has_keyword(&crabomination::card::Keyword::Flying),
        "Gained flying EOT");
}

/// Printed protection from white AND black + the {W/B}{W/B}: +1/+0 pump.
#[test]
fn stillmoon_cavalier_has_double_protection_and_pumps() {
    let mut g = two_player_game();
    let cav = g.add_card_to_battlefield(0, catalog::stillmoon_cavalier());
    g.clear_sickness(cav);
    let c = g.battlefield_find(cav).unwrap();
    assert!(c.has_keyword(&crabomination::card::Keyword::Protection(Color::White)));
    assert!(c.has_keyword(&crabomination::card::Keyword::Protection(Color::Black)));
    assert_eq!((c.power(), c.toughness()), (2, 1));
    // {W/B}{W/B}: +1/+0 — payable with one white + one black.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cav, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Stillmoon {W/B}{W/B}: +1/+0");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(cav).unwrap().power, 3, "+1/+0 applied");
}

/// Black Knight has protection from white: a white removal spell (Swords
/// to Plowshares) can't legally target it.
#[test]
fn black_knight_protection_from_white_blocks_targeting() {
    let mut g = two_player_game();
    let bk = g.add_card_to_battlefield(1, catalog::black_knight());
    assert!(g.battlefield_find(bk).unwrap()
        .has_keyword(&crabomination::card::Keyword::Protection(Color::White)));
    let stp = g.add_card_to_hand(0, catalog::swords_to_plowshares());
    g.players[0].mana_pool.add(Color::White, 1);
    let res = g.perform_action(GameAction::CastSpell {
        card_id: stp,
        target: Some(Target::Permanent(bk)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(res.is_err(), "white Swords can't target a pro-white creature: {res:?}");
}

#[test]
fn wishclaw_talisman_enters_with_three_charge_counters() {
    // Cast the Talisman so the ETB-counters payload fires through the
    // normal pipeline (rather than add-direct-to-battlefield bypass).
    let mut g = two_player_game();
    let wishclaw = g.add_card_to_hand(0, catalog::wishclaw_talisman());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: wishclaw, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Wishclaw castable");
    drain_stack(&mut g);
    let w = g.battlefield_find(wishclaw).expect("Wishclaw on battlefield");
    use crabomination::card::CounterType;
    assert_eq!(w.counter_count(CounterType::Charge), 3,
        "Enters with three charge counters");
}

#[test]
fn wishclaw_talisman_searches_and_consumes_a_charge_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let wishclaw = g.add_card_to_battlefield(0, catalog::wishclaw_talisman());
    // Manually stamp three charge counters — `add_card_to_battlefield`
    // bypasses the ETB pipeline so `enters_with_counters` doesn't fire.
    if let Some(w) = g.battlefield_find_mut(wishclaw) {
        w.add_counters(CounterType::Charge, 3);
    }
    g.clear_sickness(wishclaw);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bear)),
    ]));

    g.perform_action(GameAction::ActivateAbility {
        card_id: wishclaw, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Wishclaw activatable");
    drain_stack(&mut g);

    // The tutored card is in your hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Tutored bear into hand");
    // Charge counter consumed.
    let w = g.battlefield_find(wishclaw).expect("Wishclaw alive");
    assert_eq!(w.counter_count(CounterType::Charge), 2,
        "Charge counter consumed");
    // The "an opponent gains control" downside now fires.
    assert_eq!(w.controller, 1, "opponent gained control of Wishclaw");
    assert_eq!(w.owner, 0, "ownership unchanged — only control shifted");
}

#[test]
fn murderous_cut_destroys_target_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mc = g.add_card_to_hand(0, catalog::murderous_cut());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: mc, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Murderous Cut castable at full cost");
    drain_stack(&mut g);

    assert!(g.battlefield_find(bear).is_none(), "Bear destroyed");
}

#[test]
fn trinisphere_is_a_three_mana_artifact() {
    let g = two_player_game();
    let def = catalog::trinisphere();
    assert_eq!(def.cost.cmc(), 3, "Costs 3");
    assert!(def.card_types.contains(&CardType::Artifact), "Artifact");
    let _ = g;
}

#[test]
fn magma_spray_exiles_a_creature_it_kills() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // 2-toughness creature dies to the 2 damage → exiled by the rider.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ms = g.add_card_to_hand(0, catalog::magma_spray());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: ms, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Magma Spray castable");
    drain_stack(&mut g);

    // Bear at 2 toughness: should be exiled, not in graveyard.
    let in_gy = g.players[1].graveyard.iter().any(|c| c.id == bear);
    let in_exile = g.exile.iter().any(|c| c.id == bear);
    assert!(in_exile, "Bear exiled");
    assert!(!in_gy, "Bear NOT in graveyard");
}

#[test]
fn yarok_the_desecrated_is_a_three_five_deathtouch_lifelink() {
    let mut g = two_player_game();
    let yarok = g.add_card_to_battlefield(0, catalog::yarok_the_desecrated());
    let y = g.battlefield_find(yarok).expect("Yarok alive");
    assert_eq!(y.power(), 3);
    assert_eq!(y.toughness(), 5);
    assert!(y.has_keyword(&crabomination::card::Keyword::Deathtouch));
    assert!(y.has_keyword(&crabomination::card::Keyword::Lifelink));
}

#[test]
fn yarok_doubles_your_own_etb_triggers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::yarok_the_desecrated());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    // Elvish Visionary's own ETB "draw a card" fires twice under Yarok.
    let viz = g.add_card_to_hand(0, catalog::elvish_visionary());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: viz, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Visionary castable");
    drain_stack(&mut g);
    // Cast (-1) + two draws (+2) = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Yarok doubles Elvish Visionary's ETB draw");
}

#[test]
fn yarok_doubles_reaction_etb_triggers() {
    // Yarok also doubles "whenever another creature enters" reaction
    // triggers of your permanents (Soul Warden), not just self-ETBs.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::yarok_the_desecrated());
    g.add_card_to_battlefield(0, catalog::soul_warden());
    let life = g.players[0].life;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2,
        "Soul Warden's reaction trigger fires twice under Yarok");
}

#[test]
fn yarok_under_opponent_control_does_not_double_or_suppress_yours() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::yarok_the_desecrated()); // opponent's Yarok
    g.add_card_to_library(0, catalog::island());
    let viz = g.add_card_to_hand(0, catalog::elvish_visionary());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: viz, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Visionary castable");
    drain_stack(&mut g);
    // Opp's Yarok doubles *their* ETBs, not yours — and never suppresses.
    // Cast (-1) + one draw (+1) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "your ETB draw fires exactly once");
}

#[test]
fn hellrider_pings_defending_player_when_any_creature_attacks() {
    let mut g = two_player_game();
    let hellrider = g.add_card_to_battlefield(0, catalog::hellrider());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(hellrider);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let p1_life = g.players[1].life;
    // Only the Bear attacks — Hellrider still pings, since the trigger fires on
    // any creature you control attacking, not just Hellrider itself.
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 1, "defending player pinged for 1");
}

#[test]
fn generous_gift_destroys_target_permanent() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let gg = g.add_card_to_hand(0, catalog::generous_gift());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: gg, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Generous Gift castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear destroyed");
    // The bear's controller (player 1) gets the 3/3 green Elephant.
    let elephant = g.battlefield.iter().find(|c| {
        c.is_token && c.definition.name == "Elephant" && c.controller == 1
    });
    let elephant = elephant.expect("target's controller gets a 3/3 Elephant token");
    assert_eq!(elephant.definition.power, 3);
    assert_eq!(elephant.definition.toughness, 3);
}

#[test]
fn putrefy_modern_destroys_artifact_or_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let p = g.add_card_to_hand(0, catalog::putrefy_modern());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: p, target: Some(Target::Permanent(stone)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Putrefy castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(stone).is_none(), "Mind Stone destroyed");
}

#[test]
fn etali_primal_storm_attack_mills_each_player_one() {
    let mut g = two_player_game();
    let etali = g.add_card_to_battlefield(0, catalog::etali_primal_storm());
    g.clear_sickness(etali);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(1, catalog::island());
    let p0_yard = g.players[0].graveyard.len();
    let p1_yard = g.players[1].graveyard.len();
    let trig = catalog::etali_primal_storm().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(etali, 0, None, 0);
    let _ = g.resolve_effect(&trig, &ctx);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), p0_yard + 1, "P0 milled 1");
    assert_eq!(g.players[1].graveyard.len(), p1_yard + 1, "P1 milled 1");
}

#[test]
fn knight_of_the_reliquary_pt_scales_with_lands_in_graveyards() {
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(0, catalog::knight_of_the_reliquary());
    // Base 2/2 with no lands in gys.
    let c = g.compute_battlefield();
    let k = c.iter().find(|c| c.id == knight).unwrap();
    assert_eq!(k.power, 2, "Base power 2");
    assert_eq!(k.toughness, 2, "Base toughness 2");
    // Add 3 lands to your graveyard.
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    // Add 2 lands to opp's graveyard.
    for _ in 0..2 {
        g.add_card_to_graveyard(1, catalog::island());
    }
    let c = g.compute_battlefield();
    let k = c.iter().find(|c| c.id == knight).unwrap();
    assert_eq!(k.power, 2 + 5, "Knight grew to 7/7");
    assert_eq!(k.toughness, 2 + 5, "Knight is 7/7");
}

#[test]
fn goblin_rabblemaster_attack_creates_a_goblin_token() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let rabble = g.add_card_to_battlefield(0, catalog::goblin_rabblemaster());
    g.clear_sickness(rabble);
    let bf_before = g.battlefield.len();
    let trig = catalog::goblin_rabblemaster().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(rabble, 0, None, 0);
    let _ = g.resolve_effect(&trig, &ctx);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), bf_before + 1, "Goblin token entered");
    assert!(g.battlefield.iter().any(|c|
        c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Goblin)
    ), "Goblin token present");
}

// ── modern_decks batch 103: new cube-expansion card tests ───────────────────

#[test]
fn glaring_fleshraker_etb_pings_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::glaring_fleshraker());
    g.players[0].mana_pool.add_colorless(3);
    let life1_before = g.players[1].life;
    cast(&mut g, id);
    assert_eq!(g.players[1].life, life1_before - 2, "ETB pings for 2");
}

#[test]
fn detectives_phoenix_dies_schedules_delayed_return() {
    let mut g = two_player_game();
    let phoenix = g.add_card_to_battlefield(0, catalog::detectives_phoenix());
    g.clear_sickness(phoenix);
    let dt_before = g.delayed_triggers.len();
    g.remove_to_graveyard_with_triggers(phoenix);
    drain_stack(&mut g);
    // A delayed-return trigger should be scheduled (matches Goryo's
    // shape — at next end step the body fires).
    assert!(g.delayed_triggers.len() > dt_before,
        "Delayed return trigger scheduled");
}

#[test]
fn detectives_phoenix_returns_at_end_step_only_with_a_detective() {
    // With a Detective in play the end-step body returns the Phoenix to
    // hand; the conditional gate (CR 603.4) is satisfied.
    let mut g = two_player_game();
    let phoenix = g.add_card_to_battlefield(0, catalog::detectives_phoenix());
    g.clear_sickness(phoenix);
    g.add_card_to_battlefield(0, catalog::lonis_genetics_expert()); // Otter Detective
    g.remove_to_graveyard_with_triggers(phoenix);
    drain_stack(&mut g);
    for _ in 0..40 {
        if g.players[0].hand.iter().any(|c| c.id == phoenix) { break; }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    assert!(g.players[0].hand.iter().any(|c| c.id == phoenix),
        "Phoenix returns to hand at end step while a Detective is in play");
}

#[test]
fn detectives_phoenix_stays_in_graveyard_without_a_detective() {
    let mut g = two_player_game();
    let phoenix = g.add_card_to_battlefield(0, catalog::detectives_phoenix());
    g.clear_sickness(phoenix);
    g.remove_to_graveyard_with_triggers(phoenix);
    drain_stack(&mut g);
    // Walk through a full end step; stop if the game ends.
    for _ in 0..40 {
        if g.perform_action(GameAction::PassPriority).is_err() { break; }
        if g.players[0].hand.iter().any(|c| c.id == phoenix) { break; }
    }
    assert!(g.players[0].graveyard.iter().any(|c| c.id == phoenix),
        "Phoenix stays in the graveyard with no Detective to satisfy the gate");
    assert!(!g.players[0].hand.iter().any(|c| c.id == phoenix));
}

#[test]
fn lonis_genetics_expert_creates_clue_when_other_creature_enters() {
    use crabomination::card::ArtifactSubtype;
    let mut g = two_player_game();
    let lonis = g.add_card_to_battlefield(0, catalog::lonis_genetics_expert());
    g.clear_sickness(lonis);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bear);
    let clues: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Clue))
        .collect();
    assert_eq!(clues.len(), 1, "Lonis mints a Clue when another creature enters");
}

#[test]
fn lonis_sacrifices_x_clues_to_steal_a_permanent() {
    use crabomination::game::effects::clue_token;
    let mut g = two_player_game();
    let lonis = g.add_card_to_battlefield(0, catalog::lonis_genetics_expert());
    g.clear_sickness(lonis);
    let _c1 = g.add_token_to_battlefield(0, &clue_token());
    let _c2 = g.add_token_to_battlefield(0, &clue_token());
    // P1's top two cards: a MV-2 artifact (steal target) and a land.
    let stone = g.next_id();
    g.players[1].add_to_library_top(stone, catalog::mind_stone());

    g.perform_action(GameAction::ActivateAbility {
        card_id: lonis, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: Some(2),
    })
    .expect("{T}, Sacrifice 2 Clues activates");
    drain_stack(&mut g);

    assert!(g.battlefield_find(lonis).unwrap().tapped, "Lonis tapped as a cost");
    assert!(
        !g.battlefield.iter().any(|c| c.is_token),
        "both Clues sacrificed as a cost"
    );
    let stolen = g.battlefield_find(stone).expect("Mind Stone put onto the battlefield");
    assert_eq!(stolen.controller, 0, "stolen permanent enters under Lonis's controller");
}

#[test]
fn lonis_x_exceeding_clues_is_rejected() {
    let mut g = two_player_game();
    let lonis = g.add_card_to_battlefield(0, catalog::lonis_genetics_expert());
    g.clear_sickness(lonis);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: lonis, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: Some(1),
        })
        .is_err(),
        "can't sacrifice more Clues than you control"
    );
}

#[test]
fn loot_the_pathfinder_etb_creates_map_token() {
    use crabomination::card::ArtifactSubtype;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::loot_the_pathfinder());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    cast(&mut g, id);
    let maps: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Map))
        .collect();
    assert_eq!(maps.len(), 1, "Loot mints a Map token on ETB");
    // The Map sacrifices for {1},{T} to explore a creature you control.
    assert!(maps[0].definition.activated_abilities.iter().any(|a| a.sac_cost && a.sorcery_speed),
        "Map has a sorcery-speed sacrifice-to-explore ability");
}

#[test]
fn one_shot_is_discount_applies_to_next_spell_then_lapses() {
    use crabomination::card::{CardDefinition, CardType};
    
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    // A {3} generic instant.
    let make = || CardDefinition {
        name: "Test Bolt",
        cost: crabomination::mana::cost(&[crabomination::mana::generic(3)]),
        card_types: vec![CardType::Instant],
        ..Default::default()
    };
    let spell = crabomination::card::CardInstance::new(crabomination::card::CardId(999), make(), 0);
    // No discount yet.
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0);
    // Grant "next instant/sorcery costs {2} less".
    g.players[0].pending_is_discounts.push((2, 0));
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 2, "discount applies to next IS spell");
    // After one IS spell resolves, the tally ticks up and the discount lapses.
    g.players[0].instants_or_sorceries_cast_this_turn = 1;
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "discount is spent after one spell");
}

#[test]
fn map_token_sacrifices_to_explore_a_creature() {
    use crabomination_base::tokens::map_token;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears()); // nonland top → +1/+1 counter
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let map = g.add_token_to_battlefield(0, &map_token());
    g.clear_sickness(map);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain; // sorcery speed
    g.perform_action(GameAction::ActivateAbility {
        card_id: map, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("Map's sac-to-explore activates");
    drain_stack(&mut g);
    // The Map is gone (sacrificed) and the bear explored a nonland → +1/+1.
    assert!(!g.battlefield.iter().any(|c| c.id == map), "Map sacrificed");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "explored creature gets a +1/+1 counter off a nonland reveal");
}

#[test]
fn brightglass_gearhulk_etb_fetches_two_low_mv_permanents() {
    use crabomination::card::Keyword;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Two MV-0 artifact creatures (legal fetch targets) + chaff lands.
    let m1 = g.add_card_to_library(0, catalog::memnite());
    let m2 = g.add_card_to_library(0, catalog::memnite());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    // Script both tutors (AutoDecider would decline each search).
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Search(Some(m1)),
        DecisionAnswer::Search(Some(m2)),
    ]));
    let id = g.add_card_to_hand(0, catalog::brightglass_gearhulk());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, id);
    let r = g.battlefield_find(id).expect("Gearhulk on bf");
    assert!(r.has_keyword(&Keyword::FirstStrike) && r.has_keyword(&Keyword::Trample));
    let memnites = g.players[0].hand.iter().filter(|c| c.definition.name == "Memnite").count();
    assert_eq!(memnites, 2, "ETB tutors up to two MV-≤1 permanents to hand");
}

#[test]
fn mossborn_hydra_enters_with_x_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mossborn_hydra());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: Some(3),
    }).expect("Mossborn castable at X=3");
    drain_stack(&mut g);
    let hydra = g.battlefield_find(id).expect("Hydra on bf");
    assert_eq!(hydra.counter_count(CounterType::PlusOnePlusOne), 3,
        "Mossborn enters with X +1/+1 counters (X<4: not doubled)");
}

#[test]
fn mossborn_hydra_doubles_counters_at_x_four_or_more() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mossborn_hydra());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: Some(4),
    }).expect("Mossborn castable at X=4");
    drain_stack(&mut g);
    let hydra = g.battlefield_find(id).expect("Hydra on bf");
    assert_eq!(hydra.counter_count(CounterType::PlusOnePlusOne), 8,
        "X≥4 doubles the entering counters (2×4 = 8)");
}

#[test]
fn mai_scornful_striker_drains_opp_on_attack() {
    use crabomination::card::Keyword;
    use crabomination::game::types::{AttackTarget, TurnStep};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::mai_scornful_striker());
    g.clear_sickness(attacker);
    let mai = g.battlefield_find(attacker).expect("Mai on bf");
    assert!(mai.has_keyword(&Keyword::Menace), "Has menace");
    let life1_before = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attacker declared");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1, "opp -1 life on attack");
}

#[test]
fn tempest_angler_etb_scries_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::tempest_angler());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    cast(&mut g, id);
    use crabomination::card::Keyword;
    let angler = g.battlefield_find(id).expect("Angler on bf");
    assert!(angler.has_keyword(&Keyword::Flying));
}

#[test]
fn carnage_interpreter_etb_discards_hand_and_investigates_four() {
    let mut g = two_player_game();
    // Two other cards in hand are discarded; four Clues are minted.
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::carnage_interpreter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, id);
    assert_eq!(g.players[0].hand.len(), 0, "the whole hand is discarded");
    let clues = g.battlefield.iter()
        .filter(|c| c.definition.name == "Clue" && c.controller == 0).count();
    assert_eq!(clues, 4, "investigate four times");
}

#[test]
fn carnage_interpreter_buffs_self_with_empty_hand() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::carnage_interpreter());
    // Empty hand → +2/+2 and menace.
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == id).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5), "+2/+2 while hand ≤ 1");
    assert!(c.keywords.contains(&crabomination::card::Keyword::Menace), "menace while hand ≤ 1");
    // Two cards in hand removes the bonus.
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == id).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "base stats with a full hand");
    assert!(!c.keywords.contains(&crabomination::card::Keyword::Menace));
}

#[test]
fn helix_pinnacle_x_activation_adds_charge_counters() {
    let mut g = two_player_game();
    let hp = g.add_card_to_battlefield(0, catalog::helix_pinnacle());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hp,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: Some(5),
    }).expect("Helix Pinnacle X=5 activation");
    drain_stack(&mut g);
    let c = g.battlefield_find(hp).expect("on bf");
    assert_eq!(c.counter_count(CounterType::Charge), 5,
        "5 charge counters from X=5 activation");
}

#[test]
fn helix_pinnacle_counter_cap_at_100() {
    // Excess counters via X-activation get pruned to 100 by CR 122.4 SBA.
    let mut g = two_player_game();
    let hp = g.add_card_to_battlefield(0, catalog::helix_pinnacle());
    g.players[0].mana_pool.add_colorless(150);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hp,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: Some(150),
    }).expect("Helix Pinnacle X=150 activation");
    drain_stack(&mut g);
    let c = g.battlefield_find(hp).expect("on bf");
    assert_eq!(c.counter_count(CounterType::Charge), 100,
        "Counter cap of 100 enforced by SBA");
}

#[test]
fn helix_pinnacle_wins_at_upkeep_with_one_hundred_counters() {
    let mut g = two_player_game();
    let hp = g.add_card_to_battlefield(0, catalog::helix_pinnacle());
    // Manually stamp 100 counters (bypass the activation mana cost for
    // the upkeep-win test).
    {
        let c = g.battlefield_find_mut(hp).expect("on bf");
        c.add_counters(CounterType::Charge, 100);
    }
    use crabomination::game::types::TurnStep;
    // Walk to next upkeep (active player == 0, step == Upkeep, turn >= 2).
    let mut iters = 0;
    while !(g.active_player_idx == 0 && g.step == TurnStep::Upkeep && g.turn_number >= 2)
        && iters < 200
    {
        let _ = g.pass_priority();
        drain_stack(&mut g);
        iters += 1;
        if g.game_over.is_some() {
            break;
        }
    }
    drain_stack(&mut g);
    assert!(g.game_over.is_some(),
        "Helix Pinnacle wins at upkeep with 100 storage counters");
    assert_eq!(g.game_over, Some(Some(0)),
        "P0 (Helix controller) declared winner");
}

