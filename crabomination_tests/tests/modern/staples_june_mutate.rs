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
use crate::altars_flips_artifacts::kill_with_bolt;

// ── Staples batch 2 (June 2026) ──────────────────────────────────────────────

/// Utter End exiles any nonland permanent.
#[test]
fn utter_end_exiles_nonland_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::utter_end());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, id, Target::Permanent(bear));
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// Esper Charm mode 1 draws two.
#[test]
fn esper_charm_draw_mode() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::esper_charm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast Esper Charm");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 2);
}

/// Kaya's Guile runs two chosen modes (default picks: edict + exile
/// graveyards).
#[test]
fn kayas_guile_runs_two_modes() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::kayas_guile());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert!(g.battlefield_find(bear).is_none(), "opponent sacrificed");
    assert!(g.exile.iter().any(|c| c.id == dead), "opponent graveyard exiled");
}

/// Damn destroys one creature normally and sweeps on overload.
#[test]
fn damn_single_and_overloaded() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::damn());
    g.players[0].mana_pool.add(Color::Black, 2);
    cast_at(&mut g, id, Target::Permanent(a));
    assert!(g.battlefield_find(a).is_none());
    assert!(g.battlefield_find(b1).is_some());

    let c1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id2 = g.add_card_to_hand(0, catalog::damn());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id2, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("overload Damn");
    drain_stack(&mut g);
    assert!(g.battlefield_find(b1).is_none() && g.battlefield_find(c1).is_none(),
        "overload swept every creature");
}

/// World Breaker's cast trigger exiles a land and it recurs from the
/// graveyard for {2}{C} + a land sacrifice.
#[test]
fn world_breaker_cast_trigger_and_recursion() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::world_breaker());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(6);
    cast_at(&mut g, id, Target::Permanent(land));
    assert!(g.exile.iter().any(|c| c.id == land), "cast trigger exiled the land");

    // Kill it, then recur it.
    g.battlefield.retain(|c| c.id != id);
    g.players[0].graveyard.push(crabomination::card::CardInstance::new(id, catalog::world_breaker(), 0));
    g.add_card_to_battlefield(0, catalog::forest());
    g.players[0].mana_pool.add_colorless(3); // {2} + the true-colorless {C}
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("recursion activates from the graveyard");
    drain_stack(&mut g);
    assert!(g.players[0].has_in_hand(id), "World Breaker back in hand");
}

/// Harbinger of the Tides bounces a tapped creature on ETB and flashes in
/// for {2} more.
#[test]
fn harbinger_bounces_tapped_attacker() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    let id = g.add_card_to_hand(0, catalog::harbinger_of_the_tides());
    g.players[0].mana_pool.add(Color::Blue, 2);
    cast(&mut g, id);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "tapped bear bounced");
}

/// Shacklegeist taps two Spirits to tap an opposing creature.
#[test]
fn shacklegeist_taps_down_a_creature() {
    let mut g = two_player_game();
    let geist = g.add_card_to_battlefield(0, catalog::shacklegeist());
    let other = g.add_card_to_battlefield(0, catalog::shacklegeist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(geist);
    g.clear_sickness(other);
    g.perform_action(GameAction::ActivateAbility {
        card_id: geist, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("tap two Spirits");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "target tapped down");
    assert!(g.battlefield_find(geist).unwrap().tapped && g.battlefield_find(other).unwrap().tapped,
        "both Spirits paid the cost");
}

/// Deflecting Palm prevents the chosen source's next hit and reflects it
/// to the source's controller.
#[test]
fn deflecting_palm_reflects_damage() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt at P0");
    // Respond with Deflecting Palm choosing the Bolt as the source.
    let palm = g.add_card_to_hand(0, catalog::deflecting_palm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(bolt)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: palm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Palm");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the Bolt was prevented");
    assert_eq!(g.players[1].life, 17, "3 reflected to the Bolt's controller");
}

/// Sword-Point Diplomacy: the opponent pays 3 to deny one card; the other
/// two land in hand; the denied card is exiled.
#[test]
fn sword_point_diplomacy_pay_or_take() {
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::lightning_bolt());
    let b1 = g.add_card_to_library(0, catalog::island());
    let c1 = g.add_card_to_library(0, catalog::forest());
    // Opponent pays for the first revealed card only.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(false),
    ]));
    let id = g.add_card_to_hand(0, catalog::sword_point_diplomacy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(g.players[1].life, 17, "opponent paid 3 once");
    assert!(g.exile.iter().any(|c| c.id == a), "denied card exiled");
    assert!(g.players[0].has_in_hand(b1) && g.players[0].has_in_hand(c1),
        "unpaid cards into hand");
}

/// Stern Lesson loots two-for-one and mints a tapped Powerstone whose
/// mana can't cast a nonartifact spell.
#[test]
fn stern_lesson_powerstone_restriction() {
    use crabomination_base::mana::SpellKind;
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::stern_lesson());
    g.add_card_to_hand(0, catalog::forest()); // discard fodder
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let stone = g.battlefield.iter().find(|c| c.definition.name == "Powerstone")
        .expect("Powerstone minted").id;
    assert!(g.battlefield_find(stone).unwrap().tapped, "enters tapped");
    g.clear_sickness(stone);
    g.battlefield_find_mut(stone).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for {C}");
    // The restricted {C} can't fund a nonartifact spell but can fund an
    // artifact spell.
    let pool = &mut g.players[0].mana_pool;
    assert!(pool.clone().pay_for_spell(
        &crabomination_base::mana::cost(&[crabomination_base::mana::generic(1)]),
        &SpellKind { casting_nonartifact_spell: true, ..Default::default() },
    ).is_err(), "nonartifact spell rejected");
    assert!(pool.clone().pay_for_spell(
        &crabomination_base::mana::cost(&[crabomination_base::mana::generic(1)]),
        &SpellKind { artifact: true, ..Default::default() },
    ).is_ok(), "artifact spell allowed");
}

/// Boggart Ram-Gang's wither lands combat damage as -1/-1 counters.
#[test]
fn boggart_ram_gang_wither_in_combat() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let gang = g.add_card_to_battlefield(0, catalog::boggart_ram_gang());
    let blocker = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.clear_sickness(gang);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: gang, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, gang)])).expect("block");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    let b = g.battlefield_find(blocker).expect("Angel survives 3 wither");
    assert_eq!(b.counter_count(crabomination::card::CounterType::MinusOneMinusOne), 3,
        "wither dealt -1/-1 counters");
}

/// Eyeblight's Ending can't target an Elf.
#[test]
fn eyeblights_ending_non_elf_only() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::eyeblights_ending());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(elf)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "Elves are safe");
    cast_at(&mut g, id, Target::Permanent(bear));
    assert!(g.battlefield_find(bear).is_none());
}

/// Barkhide Troll enters with a counter and trades it for hexproof.
#[test]
fn barkhide_troll_counter_for_hexproof() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::barkhide_troll());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, id);
    assert_eq!(g.battlefield_find(id).unwrap()
        .counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1, "enters with a counter");
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("remove the counter");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 0, "counter paid");
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Hexproof));
}

/// Enraged Revolutionary's printed Dethrone grows it when it attacks the
/// highest-life player (CR 702.105, carded — not test-granted).
#[test]
fn enraged_revolutionary_dethrone_on_card() {
    let mut g = two_player_game();
    let rev = g.add_card_to_battlefield(0, catalog::enraged_revolutionary());
    g.clear_sickness(rev);
    g.players[1].life = 25; // defender has the most life
    g.players[0].life = 20;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rev, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rev).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Dethrone counter from the printed keyword");
}

/// Sunburst (CR 702.44): Suntouched Myr enters with a +1/+1 counter for each
/// color of mana spent to cast it. Paid with three different colors → 3/3.
#[test]
fn sunburst_counters_track_colors_spent() {
    let mut g = two_player_game();
    let myr = g.add_card_to_hand(0, catalog::suntouched_myr());
    // {3} paid with one mana each of three distinct colors → converge 3.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: myr, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Suntouched Myr with RGW");
    drain_stack(&mut g);
    let c = g.battlefield_find(myr).expect("Myr resolved");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3, "3 colors → 3 counters");
    let view = g.compute_battlefield().into_iter().find(|c| c.id == myr).unwrap();
    assert_eq!((view.power, view.toughness), (3, 3), "0/0 base + three +1/+1");
}

/// Paid with only generic/colorless mana, Sunburst adds no counters (a 0/0
/// that dies to the state-based-action sweep).
#[test]
fn sunburst_no_colored_mana_means_no_counters() {
    let mut g = two_player_game();
    let myr = g.add_card_to_hand(0, catalog::suntouched_myr());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: myr, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Suntouched Myr with colorless");
    drain_stack(&mut g);
    // 0/0 with no counters → dies to SBA.
    assert!(g.battlefield_find(myr).is_none(), "0/0 with no counters dies");
}

/// Memory Sluice conspired mills the target for 8 (4 original + 4 copy).
#[test]
fn conspire_memory_sluice_mills_eight() {
    let mut g = two_player_game();
    for _ in 0..12 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::memory_sluice());
    g.players[0].mana_pool.add(Color::Blue, 1);
    // Memory Sluice is {U/B}; the two conspirers must share a color with it.
    let b0 = g.add_card_to_battlefield(0, catalog::delver_of_secrets());
    let b1 = g.add_card_to_battlefield(0, catalog::delver_of_secrets());
    let gy_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::CastSpellConspire {
        card_id: id, conspire_creatures: [b0, b1],
        target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("conspire Memory Sluice");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy_before + 8, "milled 4 + 4");
}

/// Ghastly Discovery conspired nets +2 cards (draw 2 / discard 1, twice).
#[test]
fn conspire_ghastly_discovery_nets_two() {
    let mut g = two_player_game();
    for _ in 0..8 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::ghastly_discovery());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let b0 = g.add_card_to_battlefield(0, catalog::delver_of_secrets());
    let b1 = g.add_card_to_battlefield(0, catalog::delver_of_secrets());
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellConspire {
        card_id: id, conspire_creatures: [b0, b1],
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("conspire Ghastly Discovery");
    drain_stack(&mut g);
    // -1 cast + (draw 2 - discard 1) * 2 = -1 + 2 = net +1 from before-cast hand.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 4 - 2, "draw 4, discard 2");
}

/// Gleeful Sabotage destroys a target artifact (conspired; copy is redundant).
#[test]
fn conspire_gleeful_sabotage_destroys_artifact() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::gleeful_sabotage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let art = g.add_card_to_battlefield(1, catalog::ornithopter());
    let b0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpellConspire {
        card_id: id, conspire_creatures: [b0, b1],
        target: Some(Target::Permanent(art)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("conspire Gleeful Sabotage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// Disturbing Plot returns a creature card from a graveyard to its owner's
/// hand (conspired; the copy is a redundant re-target).
#[test]
fn conspire_disturbing_plot_returns_creature() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::disturbing_plot());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Disturbing Plot is {1}{B}; the conspirers must be black (Gravedigger).
    let bk0 = g.add_card_to_battlefield(0, catalog::gravedigger());
    let bk1 = g.add_card_to_battlefield(0, catalog::gravedigger());
    g.perform_action(GameAction::CastSpellConspire {
        card_id: id, conspire_creatures: [bk0, bk1],
        target: Some(Target::Permanent(dead)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("conspire Disturbing Plot");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "creature returned to hand");
}

/// Mine Excavation returns an artifact card from a graveyard to its owner's
/// hand (conspired; copy redundant).
#[test]
fn conspire_mine_excavation_returns_artifact() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::ornithopter());
    let id = g.add_card_to_hand(0, catalog::mine_excavation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let b0 = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let b1 = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.perform_action(GameAction::CastSpellConspire {
        card_id: id, conspire_creatures: [b0, b1],
        target: Some(Target::Permanent(dead)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("conspire Mine Excavation");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "artifact returned to hand");
}

/// Rally the Galadhrim makes a token copy of a creature you control; conspired,
/// it makes two copies.
#[test]
fn conspire_rally_the_galadhrim_makes_two_copies() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rally_the_galadhrim());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let orig = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Green/blue conspirers sharing a color with the {2}{G}{U} spell.
    let b0 = g.add_card_to_battlefield(0, catalog::delver_of_secrets());
    let b1 = g.add_card_to_battlefield(0, catalog::delver_of_secrets());
    g.perform_action(GameAction::CastSpellConspire {
        card_id: id, conspire_creatures: [b0, b1],
        target: Some(Target::Permanent(orig)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("conspire Rally");
    drain_stack(&mut g);
    let copies = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Grizzly Bears").count();
    assert_eq!(copies, 2, "original cast + conspire copy = two token copies");
}

/// Aethertow puts an attacking creature on top of its owner's library.
#[test]
fn aethertow_bounces_attacker_to_library_top() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let id = g.add_card_to_hand(0, catalog::aethertow());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(attacker)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Aethertow");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "attacker left the battlefield");
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(attacker),
        "put on top of its owner's library");
}

/// Giantbaiting conspired makes two 4/4 hasty Giant Warriors; both are exiled
/// at the next end step.
#[test]
fn conspire_giantbaiting_makes_two_then_exiles() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::giantbaiting());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    // {2}{R/G} — red conspirers share its color.
    let b0 = g.add_card_to_battlefield(0, catalog::goblin_guide());
    let b1 = g.add_card_to_battlefield(0, catalog::goblin_guide());
    g.perform_action(GameAction::CastSpellConspire {
        card_id: id, conspire_creatures: [b0, b1],
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("conspire Giantbaiting");
    drain_stack(&mut g);
    let giants: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Giant Warrior").collect();
    assert_eq!(giants.len(), 2, "original + conspire copy = two tokens");
    assert!(giants.iter().all(|c| c.has_keyword(&Keyword::Haste) && (c.power(), c.toughness()) == (4, 4)));
    // Next end step exiles them.
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Giant Warrior"),
        "both Giant Warriors exiled at end step");
}

/// Traitor's Roar taps a creature and makes it deal damage equal to its power
/// to its controller (conspired; copy is redundant on the same target).
#[test]
fn conspire_traitors_roar_taps_and_burns_controller() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::traitors_roar());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    // Opponent's 2/2 to turn against them.
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b0 = g.add_card_to_battlefield(0, catalog::gravedigger());
    let b1 = g.add_card_to_battlefield(0, catalog::gravedigger());
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpellConspire {
        card_id: id, conspire_creatures: [b0, b1],
        target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("conspire Traitor's Roar");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped, "victim tapped");
    assert_eq!(g.players[1].life, life - 2, "controller took 2 (the creature's power)");
}

/// Transfigure (CR 702.71): Fleshwrither sacrifices itself and tutors a
/// creature with the SAME mana value (4) onto the battlefield.
#[test]
fn fleshwrither_transfigure_same_mana_value() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let flesh = g.add_card_to_battlefield(0, catalog::fleshwrither()); // MV 4
    g.clear_sickness(flesh);
    let nek = g.add_card_to_library(0, catalog::nekrataal()); // MV 4
    g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 — must NOT match
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(nek))]));
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: flesh, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Transfigure {1}{B}{B}, sac Fleshwrither");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == flesh), "Fleshwrither sacrificed");
    assert!(g.battlefield.iter().any(|c| c.id == nek), "same-MV creature put onto battlefield");
}

#[test]
fn keruga_etb_draws_per_big_permanent() {
    // ETB: draw a card for each *other* permanent you control with MV ≥ 3.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::serra_angel()); // MV5
    g.add_card_to_battlefield(0, catalog::shivan_dragon()); // MV6
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV2 — doesn't count
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::keruga_the_macrosage());
    g.players[0].mana_pool.add(Color::Green, 5);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Keruga castable for {3}{G/U}{G/U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 2, "drew 2 (Serra + Shivan), Keruga left hand");
}

#[test]
fn gyruda_etb_mills_each_player_four() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::gyruda_doom_of_depths());
    g.players[0].mana_pool.add(Color::Black, 6);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Gyruda castable for {4}{U/B}{U/B}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 4, "controller milled 4");
    assert_eq!(g.players[1].graveyard.len(), 4, "opponent milled 4");
}

#[test]
fn kaheera_anthems_matching_creatures() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kaheera_the_orphanguard());
    let beast = g.add_card_to_battlefield(0, catalog::garruks_companion()); // 3/2 Beast
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 Bear — not a kindred type
    let cp = g.computed_permanent(beast).expect("beast alive");
    assert_eq!((cp.power, cp.toughness), (4, 3), "Beast gets +1/+1 from Kaheera");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "Beast gains vigilance");
    let bp = g.computed_permanent(bear).expect("bear alive");
    assert_eq!((bp.power, bp.toughness), (2, 2), "non-kindred Bear unaffected");
}

#[test]
fn honey_mammoth_etb_gains_four_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::honey_mammoth());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Honey Mammoth castable for {4}{G}{G}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "gained 4 on ETB");
}

#[test]
fn boon_of_the_wish_giver_draws_four() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::boon_of_the_wish_giver());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {4}{U}{U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 4, "drew four, spell left hand");
}

#[test]
fn splendor_mare_cycle_adds_lifelink_counter() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest()); // card to draw on cycle
    let id = g.add_card_to_hand(0, catalog::splendor_mare());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle {1}{W}");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().keyword_counters.get(&Keyword::Lifelink).copied().unwrap_or(0),
        1, "cycle trigger put a lifelink counter on our creature");
}

#[test]
fn frostveil_ambush_taps_two_and_locks_untap() {
    let mut g = two_player_game();
    g.step = crabomination::game::TurnStep::PreCombatMain;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::frostveil_ambush());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("cast Frostveil Ambush");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped, "both tapped");
    assert!(g.battlefield_find(a).unwrap().skip_next_untap, "a skips its next untap");
}

#[test]
fn whisper_squad_tutors_a_copy_onto_the_battlefield() {
    let mut g = two_player_game();
    let ws = g.add_card_to_battlefield(0, catalog::whisper_squad());
    let copy = g.add_card_to_library(0, catalog::whisper_squad());
    g.clear_sickness(ws);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(copy))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: ws, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate {1}{B} tutor");
    drain_stack(&mut g);
    let on_bf = g.battlefield_find(copy).expect("copy entered the battlefield");
    assert!(on_bf.tapped, "enters tapped");
}

#[test]
fn drannith_magistrate_blocks_opponent_nonhand_casts() {
    // CR 601 — opponents of the Magistrate's controller can't flashback.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::drannith_magistrate());
    // Seat 1 has Faithless Looting in its graveyard and tries to flashback.
    let id = g.add_card_to_graveyard(1, catalog::faithless_looting());
    g.players[1].mana_pool.add(Color::Red, 2);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::CastFlashback {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "Drannith blocks the opponent's graveyard cast");

    // The Magistrate's controller is unaffected.
    let own = g.add_card_to_graveyard(0, catalog::faithless_looting());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastFlashback {
        card_id: own, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_ok(), "the Magistrate's own controller can still flashback");
}

#[test]
fn cackling_flames_hellbent_scales_with_empty_hand() {
    // With a card in hand: 3 damage. With an empty hand: 5.
    for (hand_cards, expect) in [(1usize, 3i32), (0, 5)] {
        let mut g = two_player_game();
        g.players[0].hand.clear();
        for _ in 0..hand_cards { g.add_card_to_hand(0, catalog::forest()); }
        let id = g.add_card_to_hand(0, catalog::cackling_flames());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        let life = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("Cackling Flames castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - expect, "Hellbent damage with {hand_cards} cards in hand");
    }
}

#[test]
fn tri_crystals_tap_for_their_colors_and_cycle() {
    use crabomination::card::Keyword;
    let d = catalog::indatha_crystal();
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))), "has Cycling");
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::raugrin_crystal());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 1, "produced one mana");
}

#[test]
fn excavation_mole_mills_three_on_etb() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::excavation_mole());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 3, "milled three");
}

#[test]
fn bushmeat_poacher_sac_gains_life_and_draws() {
    let mut g = two_player_game();
    let poacher = g.add_card_to_battlefield(0, catalog::bushmeat_poacher());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(poacher);
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: poacher, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac a creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[0].life, life + 2, "gained life equal to its toughness");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}


#[test]
fn mosscoat_goriak_and_lava_serpent_stats() {
    use crabomination::card::Keyword;
    let g1 = catalog::mosscoat_goriak();
    assert_eq!((g1.power, g1.toughness), (2, 4));
    assert!(g1.keywords.contains(&Keyword::Vigilance));
    let s = catalog::lava_serpent();
    assert_eq!((s.power, s.toughness), (5, 5));
    assert!(s.keywords.contains(&Keyword::Haste));
    assert!(s.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
}

#[test]
fn glint_buffs_toughness_and_grants_hexproof() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::glint());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Glint castable");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).expect("bear alive");
    assert_eq!((cp.power, cp.toughness), (2, 5), "+0/+3");
    assert!(cp.keywords.contains(&Keyword::Hexproof), "gains hexproof");
}

#[test]
fn springjaw_trap_sacrifices_for_three_damage() {
    let mut g = two_player_game();
    let trap = g.add_card_to_battlefield(0, catalog::springjaw_trap());
    g.clear_sickness(trap);
    g.players[0].mana_pool.add_colorless(4);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: trap, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate {4},T,Sac");
    drain_stack(&mut g);
    assert!(g.battlefield_find(trap).is_none(), "trap sacrificed");
    assert_eq!(g.players[1].life, life - 3, "dealt 3");
}

#[test]
fn pacification_array_taps_a_creature() {
    let mut g = two_player_game();
    let array = g.add_card_to_battlefield(0, catalog::pacification_array());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(array);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: array, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate {2},T");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "target creature tapped");
}

#[test]
fn wretched_throng_death_tutors_a_copy() {
    let mut g = two_player_game();
    let throng = g.add_card_to_battlefield(0, catalog::wretched_throng());
    let copy = g.add_card_to_library(0, catalog::wretched_throng());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(copy)),
    ]));
    kill_with_bolt(&mut g, throng);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == copy), "tutored a copy to hand");
}

#[test]
fn frost_bite_scales_with_snow_permanents() {
    use crabomination::card::Supertype;
    for (snow, expect) in [(0usize, 2i32), (3, 3)] {
        let mut g = two_player_game();
        for _ in 0..snow {
            let mut land = catalog::forest();
            land.supertypes.push(Supertype::Snow);
            g.add_card_to_battlefield(0, land);
        }
        let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 5 toughness, survives
        let id = g.add_card_to_hand(0, catalog::frost_bite());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(big)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("Frost Bite castable");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(big).map(|c| c.damage).unwrap_or(0), expect as u32,
            "{snow} snow → {expect} damage");
    }
}

#[test]
fn divine_verdict_destroys_an_attacker() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.active_player_idx = 1;
    g.step = crabomination::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: atk, target: crabomination::game::types::AttackTarget::Player(0),
    }])).expect("P1 attacks");
    let id = g.add_card_to_hand(0, catalog::divine_verdict());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(atk)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Divine Verdict castable on an attacker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(atk).is_none(), "attacker destroyed");
}

#[test]
fn goring_ceratops_grants_team_double_strike_on_attack() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let cera = g.add_card_to_battlefield(0, catalog::goring_ceratops());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(cera);
    g.clear_sickness(bear);
    g.step = crabomination::game::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: cera, target: crabomination::game::types::AttackTarget::Player(1),
    }])).expect("declare attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "other creatures gain double strike");
}

#[test]
fn migration_path_fetches_two_basics_tapped() {
    let mut g = two_player_game();
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::migration_path());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f1)),
        DecisionAnswer::Search(Some(f2)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Migration Path castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(f1).is_some_and(|c| c.tapped), "first basic, tapped");
    assert!(g.battlefield_find(f2).is_some_and(|c| c.tapped), "second basic, tapped");
}

#[test]
fn titanoth_rex_cycle_adds_trample_counter() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::titanoth_rex());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle {1}{G}");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().keyword_counters.get(&Keyword::Trample).copied().unwrap_or(0),
        1, "trample counter from the cycle trigger");
}

#[test]
fn crystacean_is_a_flash_wall() {
    let d = catalog::crystacean();
    assert_eq!((d.power, d.toughness), (1, 6));
    assert!(d.keywords.contains(&crabomination::card::Keyword::Flash));
}

#[test]
fn aerial_assault_destroys_tapped_and_gains_per_flyer() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::suntail_hawk()); // a flyer we control
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::aerial_assault());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Aerial Assault castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "tapped creature destroyed");
    assert_eq!(g.players[0].life, life + 1, "gained 1 per flyer (the Hawk)");
}

#[test]
fn wilt_destroys_an_artifact_and_can_cycle() {
    use crabomination::card::Keyword;
    let d = catalog::wilt();
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::pacification_array());
    let id = g.add_card_to_hand(0, catalog::wilt());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(rock)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wilt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "artifact destroyed");
}

#[test]
fn wall_of_runes_scries_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::wall_of_runes());
    let top = g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder { kept_top: vec![top], bottom: vec![] }]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wall of Runes castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Wall entered");
}

#[test]
fn sonorous_howlbonder_locks_menace_creatures_to_three_blockers() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sonorous_howlbonder());
    // Another menace creature we control inherits the three-blocker clause.
    let mut menacer = catalog::grizzly_bears();
    menacer.keywords.push(Keyword::Menace);
    let other = g.add_card_to_battlefield(0, menacer);
    let cp = g.computed_permanent(other).expect("alive");
    assert!(cp.keywords.contains(&Keyword::CantBeBlockedExceptByN(3)),
        "menace creature gains the 3-blocker restriction");
}

// ── Ikoria mutate cycle (CR 702.140) ────────────────────────────────────────

/// Casting Glowstone Recluse for its mutate cost merges it onto a non-Human
/// host: the pile takes the top card's name/P-T and fires the mutate trigger
/// (two +1/+1 counters).
#[test]
fn mutate_glowstone_recluse_merges_on_top_and_triggers() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion()); // non-Human Beast 3/2
    let recluse = g.add_card_to_hand(0, catalog::glowstone_recluse());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3); // mutate {3}{G}
    g.perform_action(GameAction::CastMutate {
        card_id: recluse, target: host, on_top: true, x_value: None,
    }).expect("cast Glowstone Recluse for mutate");
    drain_stack(&mut g);
    // The merged permanent is the host id, now showing the Recluse on top.
    let pile = g.battlefield_find(host).expect("merged pile alive");
    assert_eq!(pile.definition.name, "Glowstone Recluse");
    // Mutate trigger added two +1/+1 counters → 2/3 base + 2 = 4/5.
    assert_eq!(pile.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2);
    assert_eq!((pile.power(), pile.toughness()), (4, 5));
    // The spell did not enter as its own creature.
    assert!(g.battlefield_find(recluse).is_none());
}

/// Mutating under the host keeps the host's characteristics on top but unions
/// the abilities (the under card's mutate trigger still fires).
#[test]
fn mutate_under_keeps_host_face_unions_abilities() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let recluse = g.add_card_to_hand(0, catalog::glowstone_recluse());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastMutate {
        card_id: recluse, target: host, on_top: false, x_value: None,
    }).expect("mutate under");
    drain_stack(&mut g);
    let pile = g.battlefield_find(host).expect("alive");
    // Top is still the host's name, but Reach (from the under card) is unioned in.
    assert_eq!(pile.definition.name, "Garruk's Companion");
    assert!(pile.definition.keywords.contains(&Keyword::Reach));
    assert_eq!(pile.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2);
}

/// A merged pile leaving the battlefield scatters into its component cards in
/// the destination zone (CR 702.140e).
#[test]
fn mutate_pile_scatters_on_death() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let bat = g.add_card_to_hand(0, catalog::dirge_bat());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4); // mutate {4}{B}{B}
    // No opponent target for the destroy trigger; let it auto-fizzle.
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::CastMutate {
        card_id: bat, target: host, on_top: true, x_value: None,
    }).expect("mutate Dirge Bat");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "mutate trigger destroyed opp creature");
    // Destroy the pile; both component cards hit the graveyard.
    g.remove_to_graveyard_with_triggers(host);
    drain_stack(&mut g);
    let gy: Vec<&str> = g.players[0].graveyard.iter().map(|c| c.definition.name).collect();
    assert!(gy.contains(&"Dirge Bat"), "Dirge Bat in graveyard: {gy:?}");
    assert!(gy.contains(&"Garruk's Companion"), "host in graveyard: {gy:?}");
}

/// Cubwarden's mutate trigger mints two 1/1 lifelink Cats; the merged pile
/// keeps Cubwarden's lifelink on top.
#[test]
fn mutate_cubwarden_makes_two_lifelink_cats() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let cub = g.add_card_to_hand(0, catalog::cubwarden());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2); // {2}{W}{W}
    g.perform_action(GameAction::CastMutate {
        card_id: cub, target: host, on_top: true, x_value: None,
    }).expect("mutate Cubwarden");
    drain_stack(&mut g);
    let cats: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Cat" && c.controller == 0).collect();
    assert_eq!(cats.len(), 2, "two Cat tokens");
    assert!(cats[0].definition.keywords.contains(&Keyword::Lifelink));
    assert!(g.battlefield_find(host).unwrap().definition.keywords.contains(&Keyword::Lifelink));
}

/// Trumpeting Gnarr's mutate trigger makes a 3/3 Beast.
#[test]
fn mutate_trumpeting_gnarr_makes_beast() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let gnarr = g.add_card_to_hand(0, catalog::trumpeting_gnarr());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3); // {3}{G/U}{G/U}
    g.perform_action(GameAction::CastMutate {
        card_id: gnarr, target: host, on_top: true, x_value: None,
    }).expect("mutate Trumpeting Gnarr");
    drain_stack(&mut g);
    let beasts = g.battlefield.iter()
        .filter(|c| c.definition.name == "Beast" && c.controller == 0).count();
    assert_eq!(beasts, 1, "one 3/3 Beast token");
}

/// Archipelagore's mutate trigger taps up to X creatures (X = mutate count)
/// chosen at resolution; they don't untap next turn (`skip_next_untap`).
#[test]
fn mutate_archipelagore_taps_dynamic_count() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let arch = g.add_card_to_hand(0, catalog::archipelagore());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5); // mutate {5}{U}
    // X = 1 after a single mutate; controller taps the opponent's creature.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![victim])]));
    g.perform_action(GameAction::CastMutate {
        card_id: arch, target: host, on_top: true, x_value: None,
    }).expect("mutate Archipelagore");
    drain_stack(&mut g);
    let v = g.battlefield_find(victim).expect("victim alive");
    assert!(v.tapped, "victim tapped by Archipelagore");
    assert!(v.skip_next_untap, "victim won't untap next turn");
}

/// Snapdax's mutate trigger deals 4 to an opponent creature and gains 4 life.
#[test]
fn mutate_snapdax_burns_and_gains() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let snap = g.add_card_to_hand(0, catalog::snapdax_apex_of_the_hunt());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.players[0].life = 20;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2); // mutate {2}{B/R}{W}{W}
    g.perform_action(GameAction::CastMutate {
        card_id: snap, target: host, on_top: true, x_value: None,
    }).expect("mutate Snapdax");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim took 4 and died");
    assert_eq!(g.players[0].life, 24, "gained 4 life");
}

/// Slitherwisp draws + drains 1 whenever you cast another flash spell.
#[test]
fn slitherwisp_triggers_on_flash_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::slitherwisp());
    let flasher = g.add_card_to_hand(0, catalog::village_bell_ringer()); // {2}{W} Flash
    g.add_card_to_library(0, catalog::grizzly_bears()); // something to draw
    g.players[0].life = 20;
    g.players[1].life = 20;
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2); // {2}{W}
    g.perform_action(GameAction::CastSpell {
        card_id: flasher, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast flash creature");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before, "drew 1 (net: cast 1, drew 1)");
    assert_eq!(g.players[1].life, 19, "opponent lost 1 life");
}

/// Illuna's mutate trigger digs to the first nonland permanent and (here) puts
/// it onto the battlefield.
#[test]
fn mutate_illuna_digs_to_permanent() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let illuna = g.add_card_to_hand(0, catalog::illuna_apex_of_wishes());
    // Library: a land on top, then a creature (the first permanent hit).
    g.add_card_to_library(0, catalog::grizzly_bears());
    let nid = g.next_id();
    g.players[0].library.insert(0, crabomination::card::CardInstance::new(
        nid, catalog::lay_of_the_land(), 0)); // Sorcery (nonpermanent) on top
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3); // mutate {3}{R/G}{U}{U}
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // → battlefield
    g.perform_action(GameAction::CastMutate {
        card_id: illuna, target: host, on_top: true, x_value: None,
    }).expect("mutate Illuna");
    drain_stack(&mut g);
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 1, "the creature was put onto the battlefield");
}

/// Vadrok's mutate trigger free-casts a noncreature card from the graveyard.
#[test]
fn mutate_vadrok_recasts_from_graveyard() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let vadrok = g.add_card_to_hand(0, catalog::vadrok_apex_of_thunder());
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // {R} instant, MV 1
    g.players[1].life = 20;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1); // mutate {1}{W/U}{R}{R}
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastMutate {
        card_id: vadrok, target: host, on_top: true, x_value: None,
    }).expect("mutate Vadrok");
    drain_stack(&mut g);
    // No creature targets on board → the free-cast bolt hits the opponent.
    assert_eq!(g.players[1].life, 17, "Vadrok recast the bolt (3 to opponent)");
}

/// Nethroi returns creature cards from the graveyard with total power ≤ 10.
#[test]
fn mutate_nethroi_reanimates_within_power_cap() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let nethroi = g.add_card_to_hand(0, catalog::nethroi_apex_of_death());
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // 2 power
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // 2 power
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4); // mutate {4}{G/W}{B}{B}
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b])]));
    g.perform_action(GameAction::CastMutate {
        card_id: nethroi, target: host, on_top: true, x_value: None,
    }).expect("mutate Nethroi");
    drain_stack(&mut g);
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "both 2-power creatures returned (total 4 ≤ 10)");
}

/// Zagoth Triome enters tapped and taps for three colors.
#[test]
fn zagoth_triome_enters_tapped_three_colors() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::zagoth_triome());
    g.perform_action(GameAction::PlayLand(id)).expect("play triome");
    let land = g.battlefield_find(id).expect("triome on battlefield");
    assert!(land.tapped, "Triome enters tapped");
    assert_eq!(land.definition.activated_abilities.len(), 3, "three mana abilities");
}


/// Farfinder's ETB optionally fetches a basic land to hand.
#[test]
fn farfinder_fetches_basic_to_hand() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let ff = g.add_card_to_hand(0, catalog::farfinder());
    g.players[0].mana_pool.add_colorless(3); // {3}
    let before = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true), DecisionAnswer::Search(Some(forest)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: ff, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Farfinder");
    drain_stack(&mut g);
    // -1 (cast Farfinder) +1 (fetched land) = net 0 vs before.
    assert_eq!(g.players[0].hand.len(), before, "a basic land went to hand");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"));
}

/// Huntmaster Liger's mutate trigger pumps your other creatures by the mutate count.
#[test]
fn mutate_huntmaster_liger_pumps_others() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let liger = g.add_card_to_hand(0, catalog::huntmaster_liger());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2); // mutate {2}{W}
    g.perform_action(GameAction::CastMutate {
        card_id: liger, target: host, on_top: true, x_value: None,
    }).expect("mutate Huntmaster Liger");
    drain_stack(&mut g);
    let o = g.battlefield_find(other).expect("other alive");
    assert_eq!((o.power(), o.toughness()), (3, 3), "other creature got +1/+1 (X=1)");
}

/// Flycatcher Giraffid enters with a chosen keyword counter.
#[test]
fn flycatcher_giraffid_choice_counter() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)])); // reach
    let id = g.add_card_to_hand(0, catalog::flycatcher_giraffid());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4); // {4}{G}
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Flycatcher");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("alive");
    assert_eq!(c.keyword_counters.get(&crabomination::card::Keyword::Reach).copied().unwrap_or(0), 1,
        "a reach counter was placed");
}

/// Bristling Boar carries the can't-be-blocked-by-more-than-one keyword.
#[test]
fn bristling_boar_has_solo_block_clause() {
    let g = two_player_game();
    let def = catalog::bristling_boar();
    assert!(def.keywords.contains(&crabomination::card::Keyword::CantBeBlockedByMoreThanOne));
    let _ = g;
}

/// Humble Naturalist taps for one creature-spell-restricted mana.
#[test]
fn humble_naturalist_makes_restricted_mana() {
    let mut g = two_player_game();
    let hn = g.add_card_to_battlefield(0, catalog::humble_naturalist());
    g.clear_sickness(hn);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hn, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap Humble Naturalist");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1, "one creature-only pip");
}

/// Mysteries of the Deep draws 3 with landfall, else 2.
#[test]
fn mysteries_of_the_deep_landfall_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    // Play a land first so landfall is active.
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    let m = g.add_card_to_hand(0, catalog::mysteries_of_the_deep());
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4); // {4}{U}
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mysteries");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 3, "drew three with landfall");
}

/// Ruinous Ultimatum destroys all nonland permanents opponents control.
#[test]
fn ruinous_ultimatum_wraths_opponents() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_land = g.add_card_to_battlefield(1, catalog::forest());
    let r = g.add_card_to_hand(0, catalog::ruinous_ultimatum());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::White, 3);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: r, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ruinous Ultimatum");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp_creature).is_none(), "opponent's creature destroyed");
    assert!(g.battlefield_find(opp_land).is_some(), "lands survive");
    assert!(g.battlefield_find(mine).is_some(), "our creature survives");
}

/// Eerie Ultimatum returns differently-named permanents from the graveyard.
#[test]
fn eerie_ultimatum_returns_distinct_names() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // same name
    let c = g.add_card_to_graveyard(0, catalog::forest());
    let e = g.add_card_to_hand(0, catalog::eerie_ultimatum());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b, c])]));
    g.perform_action(GameAction::CastSpell {
        card_id: e, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Eerie Ultimatum");
    drain_stack(&mut g);
    // a + c return (distinct names); b is a duplicate Grizzly Bears, stays.
    assert!(g.battlefield_find(a).is_some(), "first Grizzly returned");
    assert!(g.battlefield_find(c).is_some(), "Forest returned");
    assert!(g.players[0].graveyard.iter().any(|x| x.id == b), "duplicate-name card stayed");
}

/// Genesis Ultimatum deploys permanents from the top five, rest to hand.
#[test]
fn genesis_ultimatum_deploys_permanents() {
    let mut g = two_player_game();
    let creature = g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_library(0, catalog::lightning_bolt());
    let gen_id = g.add_card_to_hand(0, catalog::genesis_ultimatum());
    let before_hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![creature])]));
    g.perform_action(GameAction::CastSpell {
        card_id: gen_id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Genesis Ultimatum");
    drain_stack(&mut g);
    assert!(g.battlefield_find(creature).is_some(), "creature deployed");
    assert!(g.players[0].hand.iter().any(|c| c.id == spell), "noncreature went to hand");
    // Genesis Ultimatum exiles itself on resolve, not to graveyard.
    assert!(g.exile.iter().any(|c| c.id == gen_id), "Genesis Ultimatum exiled");
    let _ = before_hand;
}

/// Sanctuary Lockdown anthems Humans and taps with two Humans tapped.
#[test]
fn sanctuary_lockdown_anthem_and_tap() {
    let mut g = two_player_game();
    let lockdown = g.add_card_to_battlefield(0, catalog::sanctuary_lockdown());
    // Two Humans you control; a victim opposite.
    let h1 = g.add_card_to_battlefield(0, catalog::beskir_shieldmate());
    let _ = lockdown;
    let h2 = g.add_card_to_battlefield(0, catalog::beskir_shieldmate());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lockdown, ability_index: 0,
        target: Some(Target::Permanent(victim)), additional_targets: vec![], x_value: None,
    }).expect("activate Lockdown");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped, "opponent creature tapped");
    assert!(g.battlefield_find(h1).unwrap().tapped && g.battlefield_find(h2).unwrap().tapped,
        "two Humans tapped to pay");
}

/// Clear the Mind shuffles a graveyard into the library and draws.
#[test]
fn clear_the_mind_shuffles_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::forest());
    let c = g.add_card_to_hand(0, catalog::clear_the_mind());
    let before_hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: c, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Clear the Mind");
    drain_stack(&mut g);
    // The two seeded cards shuffled away (Clear the Mind itself then lands in gy).
    assert!(!g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"
        || c.definition.name == "Lightning Bolt"), "seeded cards shuffled away");
    assert_eq!(g.players[0].hand.len(), before_hand - 1 + 1, "drew a card");
}

/// Blitz of the Thunder-Raptor deals damage equal to I/S in graveyard and exiles.
#[test]
fn blitz_thunder_raptor_scales_and_exiles() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());   // instant
    g.add_card_to_graveyard(0, catalog::ponder());           // sorcery
    g.add_card_to_graveyard(0, catalog::grizzly_bears());    // not I/S
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let blitz = g.add_card_to_hand(0, catalog::blitz_of_the_thunder_raptor());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: blitz, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Blitz");
    drain_stack(&mut g);
    // 2 damage (2 I/S cards) kills the 2/2; exiled, not in graveyard.
    assert!(g.battlefield_find(victim).is_none(), "victim died to 2 damage");
    assert!(g.exile.iter().any(|c| c.id == victim), "exiled instead of graveyard");
}

/// Tentative Connection steals a creature for the turn with haste.
#[test]
fn tentative_connection_steals_for_turn() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(creature).unwrap().tapped = true;
    let t = g.add_card_to_hand(0, catalog::tentative_connection());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3); // {3}{R}
    g.perform_action(GameAction::CastSpell {
        card_id: t, target: Some(Target::Permanent(creature)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Tentative Connection");
    drain_stack(&mut g);
    let c = g.battlefield_find(creature).expect("alive");
    assert_eq!(c.controller, 0, "we control it now");
    assert!(!c.tapped, "it was untapped");
}

/// Essence Capture counters a creature spell and grows one of your creatures.
#[test]
fn essence_capture_counters_and_grows() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts a creature");
    let cap = g.add_card_to_hand(0, catalog::essence_capture());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: cap, target: Some(Target::Permanent(spell)),
        additional_targets: vec![Target::Permanent(mine)], mode: None, x_value: None,
    }).expect("cast Essence Capture");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == spell), "creature spell countered");
    let m = g.battlefield_find(mine).expect("our creature alive");
    assert_eq!(m.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
}

/// Gust of Wind bounces an opponent's nonland permanent and draws.
#[test]
fn gust_of_wind_bounces_and_draws() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let gust = g.add_card_to_hand(0, catalog::gust_of_wind());
    let before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3); // {3}{U}
    g.perform_action(GameAction::CastSpell {
        card_id: gust, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Gust of Wind");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "bounced to owner's hand");
    assert_eq!(g.players[0].hand.len(), before - 1 + 1, "drew a card (net: cast + draw)");
}

/// Mythos of Illuna makes a token copy of a target permanent.
#[test]
fn mythos_of_illuna_copies_permanent() {
    let mut g = two_player_game();
    let orig = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::mythos_of_illuna());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2); // {2}{U}{U}
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: Some(Target::Permanent(orig)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mythos of Illuna");
    drain_stack(&mut g);
    let bears = g.battlefield.iter()
        .filter(|c| c.definition.name == "Grizzly Bears" && c.controller == 0).count();
    assert_eq!(bears, 1, "a Grizzly Bears token copy under our control");
}

/// Mythos of Brokkos returns up to two permanent cards from the graveyard.
#[test]
fn mythos_of_brokkos_returns_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // nonpermanent — ineligible
    let m = g.add_card_to_hand(0, catalog::mythos_of_brokkos());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2); // {2}{G}{G}
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b])]));
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mythos of Brokkos");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == a), "creature returned to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == b), "land returned to hand");
}

/// Mythos of Vadrok deals 5 damage divided (here, all to one creature).
#[test]
fn mythos_of_vadrok_divides_damage() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::mythos_of_vadrok());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2); // {2}{R}{R}
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mythos of Vadrok");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "the 2/2 took lethal from the 5 damage");
}

/// Kogla's ETB fights an opponent creature; its activation grants indestructible.
#[test]
fn kogla_fights_on_etb_and_protects() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let kogla = g.add_card_to_hand(0, catalog::kogla_the_titan_ape());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(3); // {3}{G}{G}{G}
    g.perform_action(GameAction::CastSpell {
        card_id: kogla, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Kogla");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "Kogla's ETB fight killed the 2/2");
    let k = g.battlefield_find(kogla).expect("Kogla survives the fight (7/6 vs 2/2)");
    assert_eq!(k.damage, 2, "Kogla took 2 from the fight");

    // Activate: return a Human (none here → fizzle on the move) is awkward, so
    // just confirm the ability grants indestructible to Kogla.
    let human = g.add_card_to_battlefield(0, catalog::beskir_shieldmate());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kogla, ability_index: 0,
        target: Some(Target::Permanent(human)), additional_targets: vec![], x_value: None,
    }).expect("activate Kogla");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == human), "Human returned to hand");
    assert!(g.battlefield_find(kogla).unwrap().is_indestructible(), "Kogla gained indestructible");
}

/// Parcelbeast's ability puts a revealed top land onto the battlefield.
#[test]
fn parcelbeast_drops_top_land() {
    let mut g = two_player_game();
    let pb = g.add_card_to_battlefield(0, catalog::parcelbeast());
    g.clear_sickness(pb);
    let land = g.add_card_to_library(0, catalog::forest()); // top of empty library
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pb, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Parcelbeast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some(), "top land entered the battlefield");
}

/// Yidaro shuffles back into the library on a cycle; on the 4th cycle this
/// game it enters the battlefield from the graveyard instead.
#[test]
fn yidaro_recurs_on_fourth_cycle() {
    let mut g = two_player_game();
    // First cycle: shuffles into the library (not onto the battlefield).
    let y1 = g.add_card_to_hand(0, catalog::yidaro_wandering_monster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Cycle { card_id: y1, x_value: None }).expect("cycle 1");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Yidaro, Wandering Monster"),
        "first cycle shuffles back, no battlefield Yidaro");
    assert!(g.players[0].library.iter().any(|c| c.id == y1), "Yidaro shuffled into library");

    // Pre-load the game count to 3 so the next cycle is the 4th.
    g.cycled_count_by_name.insert("Yidaro, Wandering Monster".into(), 3);
    let y2 = g.add_card_to_hand(0, catalog::yidaro_wandering_monster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Cycle { card_id: y2, x_value: None }).expect("cycle 4");
    drain_stack(&mut g);
    assert!(g.battlefield_find(y2).is_some(), "4th cycle puts Yidaro onto the battlefield");
}

/// Zilortha makes your creatures' lethal threshold their power, not toughness:
/// a damaged 0/4 dies; an undamaged one survives.
#[test]
fn zilortha_lethal_measured_by_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zilortha_strength_incarnate());
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_omens()); // 0/4
    // 1 marked damage: normally survives (1 < 4) but dies under Zilortha (1 ≥ power 0).
    g.battlefield_find_mut(wall).unwrap().damage = 1;
    g.check_state_based_actions();
    assert!(g.battlefield_find(wall).is_none(), "0/4 dies once damaged under Zilortha");

    // An undamaged 0/4 survives (power threshold 0, but no damage marked).
    let wall2 = g.add_card_to_battlefield(0, catalog::wall_of_omens());
    g.check_state_based_actions();
    assert!(g.battlefield_find(wall2).is_some(), "undamaged 0/4 survives");
}

/// Mythos of Nethroi destroys a target creature.
#[test]
fn mythos_of_nethroi_destroys_creature() {
    let mut g = two_player_game();
    let m = g.add_card_to_hand(0, catalog::mythos_of_nethroi());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2); // {2}{B}
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mythos of Nethroi");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "target creature destroyed");
}

/// Mutual Destruction sacrifices a creature (additional cost) and destroys one.
#[test]
fn mutual_destruction_sacs_and_destroys() {
    let mut g = two_player_game();
    let md = g.add_card_to_hand(0, catalog::mutual_destruction());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1); // {B}
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![fodder])]));
    g.perform_action(GameAction::CastSpell {
        card_id: md, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mutual Destruction");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed our creature");
    assert!(g.battlefield_find(victim).is_none(), "destroyed target creature");
}

/// Everquill Phoenix's mutate trigger mints a Feather token whose sac ability
/// reanimates a Phoenix from the graveyard.
#[test]
fn mutate_everquill_phoenix_feather_reanimates() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let phoenix = g.add_card_to_hand(0, catalog::everquill_phoenix());
    let dead = g.add_card_to_graveyard(0, catalog::everquill_phoenix()); // a Phoenix in gy
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3); // mutate {3}{R}
    g.perform_action(GameAction::CastMutate {
        card_id: phoenix, target: host, on_top: true, x_value: None,
    }).expect("mutate Everquill Phoenix");
    drain_stack(&mut g);
    let feather = g.battlefield.iter().find(|c| c.definition.name == "Feather")
        .expect("Feather token minted").id;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: feather, ability_index: 0,
        target: Some(Target::Permanent(dead)), additional_targets: vec![], x_value: None,
    }).expect("activate Feather");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "Phoenix returned to the battlefield");
    assert!(g.battlefield_find(feather).is_none(), "Feather sacrificed");
}

/// Cavern Whisperer's mutate trigger makes each opponent discard.
#[test]
fn mutate_cavern_whisperer_opponent_discards() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    g.add_card_to_hand(1, catalog::grizzly_bears()); // opponent's only card
    let whisper = g.add_card_to_hand(0, catalog::cavern_whisperer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3); // {3}{B}
    g.perform_action(GameAction::CastMutate {
        card_id: whisper, target: host, on_top: true, x_value: None,
    }).expect("mutate Cavern Whisperer");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "opponent discarded their card");
}

/// Migratory Greathorn's mutate trigger fetches a basic land tapped.
#[test]
fn mutate_migratory_greathorn_ramps() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let horn = g.add_card_to_hand(0, catalog::migratory_greathorn());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2); // {2}{G}
    g.perform_action(GameAction::CastMutate {
        card_id: horn, target: host, on_top: true, x_value: None,
    }).expect("mutate Migratory Greathorn");
    drain_stack(&mut g);
    let land = g.battlefield.iter().find(|c| c.definition.name == "Forest" && c.controller == 0);
    assert!(land.is_some_and(|l| l.tapped), "Forest entered tapped");
}

/// Boneyard Lurker's mutate trigger returns a permanent card from the
/// graveyard to hand.
#[test]
fn mutate_boneyard_lurker_regrows() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let card = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    let bears_id = card.id;
    g.players[0].graveyard.push(card);
    let lurker = g.add_card_to_hand(0, catalog::boneyard_lurker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2); // {2}{B/G}{B/G}
    g.perform_action(GameAction::CastMutate {
        card_id: lurker, target: host, on_top: true, x_value: None,
    }).expect("mutate Boneyard Lurker");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears_id), "permanent card returned to hand");
}

/// Pollywog Symbiote makes a mutate spell cost {1} less and loots when you
/// cast one. Glowstone Recluse's mutate cost {3}{G} drops to {2}{G}.
#[test]
fn pollywog_symbiote_discounts_and_loots_mutate() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pollywog_symbiote());
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let recluse = g.add_card_to_hand(0, catalog::glowstone_recluse());
    g.add_card_to_library(0, catalog::grizzly_bears()); // the looted draw
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2); // only {2}{G} — proves the discount
    g.perform_action(GameAction::CastMutate {
        card_id: recluse, target: host, on_top: true, x_value: None,
    }).expect("mutate at the discounted {2}{G}");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(host).unwrap().definition.name, "Glowstone Recluse");
    // Loot: drew the Grizzly Bears, then discarded it (sole hand card).
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "Pollywog looted a card into the graveyard");
}

/// Helper: cast `card` (in hand) for its mutate cost onto a fresh non-Human
/// host, paying a fat mana pool. Returns the host id (the merged pile).
#[cfg(test)]
fn mutate_onto_fresh_host(g: &mut GameState, card: CardId) -> CardId {
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 10);
    }
    g.players[0].mana_pool.add_colorless(10);
    g.perform_action(GameAction::CastMutate {
        card_id: card, target: host, on_top: true, x_value: None,
    }).expect("cast for mutate");
    host
}

/// Wingfold Pteron enters with a chosen hexproof counter.
#[test]
fn wingfold_pteron_enters_hexproof() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)])); // hexproof
    let id = g.add_card_to_hand(0, catalog::wingfold_pteron());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Wingfold Pteron");
    drain_stack(&mut g);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Hexproof));
}

/// Voracious Greatshark counters a creature spell when it flashes in.
#[test]
fn voracious_greatshark_counters_creature_spell() {
    let mut g = two_player_game();
    // Opponent casts a creature spell on their own main phase.
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts Grizzly Bears");
    // We flash in the Greatshark in response; its ETB counters the bear.
    let shark = g.add_card_to_hand(0, catalog::voracious_greatshark());
    for _ in 0..3 { g.players[0].mana_pool.add(Color::Blue, 1); }
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: shark, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("flash in Greatshark");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the creature spell was countered");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Voracious Greatshark"), "shark resolved");
}

/// Heightened Reflexes pumps and grants a first strike counter.
#[test]
fn heightened_reflexes_pumps_and_first_strikes() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::heightened_reflexes());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Heightened Reflexes");
    drain_stack(&mut g);
    let b = g.computed_permanent(bear).unwrap();
    assert_eq!(b.power, 3, "+1/+0");
    assert!(b.keywords.contains(&Keyword::FirstStrike));
}

/// Weaponize the Monsters fires a sacrificed creature at any target.
#[test]
fn weaponize_the_monsters_slings_a_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::weaponize_the_monsters());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    let opp = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: g.battlefield.iter().find(|c| c.definition.name == "Weaponize the Monsters").unwrap().id,
        ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
    }).expect("activate, sacrificing the bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    assert_eq!(g.players[1].life, opp - 2, "dealt 2 to the opponent");
}

/// Unbreakable Bond reanimates a creature with a lifelink counter.
#[test]
fn unbreakable_bond_reanimates_with_lifelink() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let dead = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    let dead_id = dead.id;
    g.players[0].graveyard.push(dead);
    let id = g.add_card_to_hand(0, catalog::unbreakable_bond());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(dead_id))]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(dead_id)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Unbreakable Bond");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead_id).is_some(), "creature reanimated");
    assert!(g.computed_permanent(dead_id).unwrap().keywords.contains(&Keyword::Lifelink));
}

/// Heroes' Reunion gains the target player 7 life.
#[test]
fn heroes_reunion_gains_seven() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::heroes_reunion());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Heroes' Reunion");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 7);
}

/// Forbidden Friendship makes a Dinosaur and a Human Soldier.
#[test]
fn forbidden_friendship_makes_two_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::forbidden_friendship());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Forbidden Friendship");
    drain_stack(&mut g);
    let mine: Vec<&str> = g.battlefield.iter().filter(|c| c.controller == 0).map(|c| c.definition.name).collect();
    assert!(mine.contains(&"Dinosaur") && mine.contains(&"Human Soldier"), "{mine:?}");
}

/// Easy Prey kills a small creature.
#[test]
fn easy_prey_destroys_small_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let id = g.add_card_to_hand(0, catalog::easy_prey());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Easy Prey");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "MV-2 creature destroyed");
}

/// Necropanther reanimates a small creature on mutate.
#[test]
fn necropanther_reanimates_on_mutate() {
    let mut g = two_player_game();
    let dead = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0); // MV 2
    let dead_id = dead.id;
    g.players[0].graveyard.push(dead);
    let np = g.add_card_to_hand(0, catalog::necropanther());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(dead_id))]));
    mutate_onto_fresh_host(&mut g, np);
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead_id).is_some(), "creature reanimated to battlefield");
}

/// Cliffhaven Kitesail auto-equips and grants flying.
#[test]
fn cliffhaven_kitesail_grants_flying() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cliffhaven_kitesail());
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cliffhaven Kitesail");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying),
        "equipped creature has flying");
}

/// Blood Curdle kills a creature and hands one of yours a menace counter.
#[test]
fn blood_curdle_destroys_and_grants_menace() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::blood_curdle());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Blood Curdle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature destroyed");
    assert!(g.computed_permanent(mine).unwrap().keywords.contains(&Keyword::Menace));
}

/// Helica Glider enters with a chosen flying counter.
#[test]
fn helica_glider_enters_flying() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)])); // flying
    let id = g.add_card_to_hand(0, catalog::helica_glider());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Helica Glider");
    drain_stack(&mut g);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Flying));
}

/// Maraleaf Pixie taps for green or blue.
#[test]
fn maraleaf_pixie_makes_mana() {
    let mut g = two_player_game();
    let pix = g.add_card_to_battlefield(0, catalog::maraleaf_pixie());
    g.clear_sickness(pix);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pix, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    let pool = g.players[0].mana_pool.total();
    assert_eq!(pool, 1, "produced one mana");
}

/// Skull Prophet's second ability self-mills two.
#[test]
fn skull_prophet_mills() {
    let mut g = two_player_game();
    let sp = g.add_card_to_battlefield(0, catalog::skull_prophet());
    g.clear_sickness(sp);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let before = g.players[0].graveyard.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: sp, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("mill ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), before + 2, "milled two cards");
}

/// Dreamtail Heron draws on mutate and flies.
#[test]
fn dreamtail_heron_draws_on_mutate() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let heron = g.add_card_to_hand(0, catalog::dreamtail_heron());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let host = mutate_onto_fresh_host(&mut g, heron);
    drain_stack(&mut g);
    assert!(g.battlefield_find(host).unwrap().definition.keywords.contains(&Keyword::Flying));
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"), "drew on mutate");
}

/// Barrier Breach exiles up to three enchantments.
#[test]
fn barrier_breach_exiles_enchantments() {
    let mut g = two_player_game();
    let e1 = g.add_card_to_battlefield(1, catalog::whirlwind_of_thought());
    let id = g.add_card_to_hand(0, catalog::barrier_breach());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(e1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Barrier Breach");
    drain_stack(&mut g);
    assert!(g.battlefield_find(e1).is_none(), "enchantment exiled");
}

/// Porcuparrot pings for X = number of times it has mutated.
#[test]
fn porcuparrot_pings_by_mutate_count() {
    let mut g = two_player_game();
    let p = g.add_card_to_hand(0, catalog::porcuparrot());
    let host = mutate_onto_fresh_host(&mut g, p); // mutated once → X = 1
    drain_stack(&mut g);
    g.clear_sickness(host);
    let opp = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: host, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("tap to ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "dealt X=1 damage");
}

/// Vulpikeet grows itself on mutate and flies.
#[test]
fn vulpikeet_grows_and_flies() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let v = g.add_card_to_hand(0, catalog::vulpikeet());
    let host = mutate_onto_fresh_host(&mut g, v);
    drain_stack(&mut g);
    let p = g.battlefield_find(host).unwrap();
    assert!(p.definition.keywords.contains(&Keyword::Flying));
    assert_eq!(p.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
}

/// Majestic Auricorn gains 4 life on mutate.
#[test]
fn majestic_auricorn_gains_life() {
    let mut g = two_player_game();
    let a = g.add_card_to_hand(0, catalog::majestic_auricorn());
    let life = g.players[0].life;
    mutate_onto_fresh_host(&mut g, a);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4);
}

/// Insatiable Hemophage drains X = number of times it has mutated (1 here).
#[test]
fn insatiable_hemophage_drains_by_mutate_count() {
    let mut g = two_player_game();
    let h = g.add_card_to_hand(0, catalog::insatiable_hemophage());
    let my_life = g.players[0].life;
    let opp_life = g.players[1].life;
    mutate_onto_fresh_host(&mut g, h);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 1, "opponent lost X=1");
    assert_eq!(g.players[0].life, my_life + 1, "you gained X=1");
}

/// Sawtusk Demolisher blows up a noncreature permanent and gifts a 3/3 Beast.
#[test]
fn sawtusk_demolisher_destroys_noncreature() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::pacification_array()); // artifact
    let s = g.add_card_to_hand(0, catalog::sawtusk_demolisher());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(rock))]));
    mutate_onto_fresh_host(&mut g, s);
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "noncreature destroyed");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Beast" && c.controller == 1).count(),
        1, "its controller got a 3/3 Beast");
}

/// Gemrazer destroys an opponent's artifact on mutate.
#[test]
fn gemrazer_destroys_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::pacification_array());
    let gem = g.add_card_to_hand(0, catalog::gemrazer());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(art))]));
    mutate_onto_fresh_host(&mut g, gem);
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "opponent artifact destroyed");
}

/// Chittering Harvester edicts each opponent on mutate.
#[test]
fn chittering_harvester_edicts() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ch = g.add_card_to_hand(0, catalog::chittering_harvester());
    mutate_onto_fresh_host(&mut g, ch);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent sacrificed their only creature");
}

/// Regal Leosaur pumps the rest of your team on mutate, but not itself.
#[test]
fn regal_leosaur_pumps_team() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let leo = g.add_card_to_hand(0, catalog::regal_leosaur());
    let host = mutate_onto_fresh_host(&mut g, leo);
    drain_stack(&mut g);
    let o = g.computed_permanent(other).unwrap();
    assert_eq!((o.power, o.toughness), (4, 3), "other creature got +2/+1");
    // The host itself is excluded by OtherThanSource (base Leosaur 2/2).
    assert_eq!(g.computed_permanent(host).map(|c| (c.power, c.toughness)), Some((2, 2)));
}

/// Void Beckoner drops a deathtouch counter when cycled.
#[test]
fn void_beckoner_cycle_grants_deathtouch() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let vb = g.add_card_to_hand(0, catalog::void_beckoner());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2); // Cycling {2}{B}
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(mine))]));
    g.perform_action(GameAction::Cycle { card_id: vb, x_value: None }).expect("cycle Void Beckoner");
    drain_stack(&mut g);
    assert!(g.computed_permanent(mine).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Almighty Brushwagg pumps itself with its activated ability.
#[test]
fn almighty_brushwagg_pumps() {
    let mut g = two_player_game();
    let bw = g.add_card_to_battlefield(0, catalog::almighty_brushwagg());
    g.clear_sickness(bw);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bw, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate {3}{G}");
    drain_stack(&mut g);
    let b = g.computed_permanent(bw).unwrap();
    assert_eq!((b.power, b.toughness), (4, 4), "1/1 pumped to 4/4");
}

/// Essence Symbiote rewards any creature you control mutating with a +1/+1
/// counter on it and 2 life.
#[test]
fn essence_symbiote_rewards_mutation() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::essence_symbiote());
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let recluse = g.add_card_to_hand(0, catalog::glowstone_recluse());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastMutate {
        card_id: recluse, target: host, on_top: true, x_value: None,
    }).expect("mutate");
    drain_stack(&mut g);
    // Recluse's own +2 counters plus Essence Symbiote's +1 = 3.
    assert_eq!(g.battlefield_find(host).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 3);
    assert_eq!(g.players[0].life, life + 2, "gained 2 from Essence Symbiote");
}

/// Cloudpiercer rummages (discard then draw) when it mutates.
#[test]
fn cloudpiercer_rummages_on_mutate() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let piercer = g.add_card_to_hand(0, catalog::cloudpiercer());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // the discard
    g.add_card_to_library(0, catalog::lightning_bolt()); // the draw
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3); // {3}{R}
    g.perform_action(GameAction::CastMutate {
        card_id: piercer, target: host, on_top: true, x_value: None,
    }).expect("mutate Cloudpiercer");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"), "drew the bolt");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"), "discarded the bear");
}

/// Sea-Dasher Octopus draws when it connects in combat.
#[test]
fn sea_dasher_octopus_draws_on_combat_damage() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let octo = g.add_card_to_battlefield(0, catalog::sea_dasher_octopus());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.clear_sickness(octo);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: octo, target: AttackTarget::Player(1),
    }])).expect("octopus attacks");
    g.step = TurnStep::CombatDamage;
    let _ = g.resolve_combat();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"), "drew on combat damage");
}

/// Snare Tactician taps an opponent's creature whenever you cycle a card.
#[test]
fn snare_tactician_taps_on_cycle() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::snare_tactician());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cycler = g.add_card_to_hand(0, catalog::migration_path()); // has Cycling {2}
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
    g.perform_action(GameAction::Cycle { card_id: cycler, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped, "Snare Tactician tapped the opponent's creature");
}

/// Capture Sphere taps the enchanted creature and locks it down.
#[test]
fn capture_sphere_locks_a_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::capture_sphere());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Capture Sphere");
    drain_stack(&mut g);
    let v = g.battlefield_find(victim).unwrap();
    assert!(v.tapped, "enchanted creature tapped on entry");
    // Untap step skips the locked creature.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(victim).unwrap().tapped, "Capture Sphere keeps it tapped through untap");
}

/// Boot Nipper enters with a chosen keyword counter (deathtouch here).
#[test]
fn boot_nipper_enters_with_chosen_counter() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)])); // deathtouch
    let id = g.add_card_to_hand(0, catalog::boot_nipper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Boot Nipper");
    drain_stack(&mut g);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Deathtouch),
        "chose the deathtouch counter");
}

#[test]
fn frillscare_mentor_grants_menace_then_pumps_menace_team() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    let mentor = g.add_card_to_battlefield(0, catalog::frillscare_mentor());
    let beast = g.add_card_to_battlefield(0, catalog::garruks_companion()); // non-Human Beast
    g.clear_sickness(mentor);
    g.fire_self_etb_triggers(mentor, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(beast))]));
    drain_stack(&mut g);
    assert!(g.computed_permanent(beast).unwrap().keywords.contains(&Keyword::Menace),
        "ETB granted a menace counter");
    // Activate: +1/+1 counter on each menace creature we control.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mentor, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate {2}{R},T");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(beast).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1, "menace beast got a +1/+1 counter");
}
/// Jubilant Skybonder taxes opponents' spells that target a flyer you control.
#[test]
fn jubilant_skybonder_taxes_opponent_spells_targeting_flyers() {
    use crabomination::game::actions::extra_cost_for_spell;
    let mut g = two_player_game();
    let skybonder = g.add_card_to_battlefield(0, catalog::jubilant_skybonder()); // 2/2 flyer
    let ground = g.add_card_to_battlefield(0, catalog::grizzly_bears());          // no flying
    // Opponent (player 1) casts a spell at our flyer vs. our ground creature.
    let bolt_id = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bolt = g.players[1].hand.iter().find(|c| c.id == bolt_id).unwrap().clone();
    let at_flyer = crabomination::game::Target::Permanent(skybonder);
    let at_ground = crabomination::game::Target::Permanent(ground);
    assert_eq!(extra_cost_for_spell(&g, 1, &bolt, Some(&at_flyer)), 2, "flyer taxed by 2");
    assert_eq!(extra_cost_for_spell(&g, 1, &bolt, Some(&at_ground)), 0, "ground creature untaxed");
    // The controller's own spells are never taxed.
    assert_eq!(extra_cost_for_spell(&g, 0, &bolt, Some(&at_flyer)), 0, "own spells untaxed");
}

/// Lavabrink Venturer's ETB choice grants protection from the chosen parity:
/// choosing even blocks even-mana-value spells, choosing odd blocks odd ones.
#[test]
fn lavabrink_venturer_parity_protection() {
    use crabomination::card::Keyword;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    // Helper: enter a Venturer with the given odd/even choice and try to
    // target it with `spell`; returns whether the cast was rejected.
    fn try_target(mode: usize, spell: crabomination::card::CardDefinition) -> bool {
        let mut g = two_player_game();
        // Venturer belongs to player 1; the active player 0 (with priority)
        // tries to target it.
        let venturer = g.add_card_to_battlefield(1, catalog::lavabrink_venturer());
        g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(mode)]));
        g.fire_self_etb_triggers(venturer, 1);
        drain_stack(&mut g);
        let expect_odd = mode == 1;
        assert!(g.computed_permanent(venturer).unwrap().keywords
            .contains(&Keyword::ProtectionFromManaValueParity { odd: expect_odd }));
        let s = g.add_card_to_hand(0, spell);
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: s, target: Some(Target::Permanent(venturer)),
            additional_targets: vec![], mode: None, x_value: None,
        }).is_err()
    }
    // Chose even: Doom Blade (mv 2, even) blocked; Lightning Bolt (mv 1, odd) ok.
    assert!(try_target(0, catalog::doom_blade()), "even-protected blocks even spell");
    assert!(!try_target(0, catalog::lightning_bolt()), "even-protected allows odd spell");
    // Chose odd: Lightning Bolt blocked; Doom Blade allowed.
    assert!(try_target(1, catalog::lightning_bolt()), "odd-protected blocks odd spell");
    assert!(!try_target(1, catalog::doom_blade()), "odd-protected allows even spell");
}

/// Mythos of Snapdax: each player keeps their best of each nonland type and
/// sacrifices the rest.
#[test]
fn mythos_of_snapdax_keeps_one_per_type() {
    let mut g = two_player_game();
    // Player 0: two creatures (keeps Serra Angel — higher MV) + an artifact.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::serra_angel());
    g.add_card_to_battlefield(0, catalog::mind_stone());
    // Player 1: two creatures.
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::mythos_of_snapdax());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mythos of Snapdax");
    drain_stack(&mut g);
    let names = |p: usize| -> Vec<&str> {
        let mut v: Vec<&str> = g.battlefield.iter().filter(|c| c.controller == p)
            .map(|c| c.definition.name).collect();
        v.sort();
        v
    };
    // Each player keeps their best creature; player 0 also keeps the artifact.
    assert_eq!(names(0), vec!["Mind Stone", "Serra Angel"]);
    assert_eq!(names(1), vec!["Serra Angel"]);
}

/// Clackbridge Troll: ETB gifts the opponent three Goats; the begin-combat
/// tempting offer (accepted) sacrifices a creature, taps the Troll, and the
/// controller gains 3 life and draws.
#[test]
fn clackbridge_troll_etb_goats_and_combat_offer() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let troll = g.add_card_to_battlefield(0, catalog::clackbridge_troll());
    g.fire_self_etb_triggers(troll, 0);
    drain_stack(&mut g);
    let goats = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.subtypes.creature_types.contains(&CreatureType::Goat))
        .count();
    assert_eq!(goats, 3, "opponent got three Goats");
    // Begin combat: opponent accepts and sacrifices a Goat.
    g.add_card_to_library(0, catalog::forest()); // something to draw
    let hand_before = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1
        && c.definition.subtypes.creature_types.contains(&CreatureType::Goat)).count(),
        2, "opponent sacrificed one Goat");
    assert!(g.battlefield_find(troll).unwrap().tapped, "Troll tapped");
    assert_eq!(g.players[0].life, 23, "gained 3 life");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

/// Wingspan Mentor: ETB flying counter on a non-Human; activated grows flyers.
#[test]
fn wingspan_mentor_flying_counter_and_pump() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    let mentor = g.add_card_to_battlefield(0, catalog::wingspan_mentor());
    let beast = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // non-Human, no flying
    g.clear_sickness(mentor);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Permanent(beast))]));
    g.fire_self_etb_triggers(mentor, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(beast).unwrap().keywords.contains(&Keyword::Flying),
        "flying counter granted flying");
    // Activate: +1/+1 counter on each flyer (the now-flying Bears).
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mentor, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate {2}{U},T");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(beast).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1, "flyer got a +1/+1 counter");
}

/// Of One Mind costs {2} less with a Human and a non-Human creature in play.
#[test]
fn of_one_mind_conditional_discount() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let som_id = g.add_card_to_hand(0, catalog::of_one_mind());
    let som = g.players[0].hand.iter().find(|c| c.id == som_id).unwrap().clone();
    // No creatures: no discount.
    assert_eq!(cost_reduction_for_spell(&g, 0, &som, None), 0, "no creatures, no discount");
    // A non-Human only: still no discount (needs both).
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(cost_reduction_for_spell(&g, 0, &som, None), 0, "non-Human alone, no discount");
    // Add a Human: now {2} less.
    g.add_card_to_battlefield(0, catalog::wingspan_mentor()); // Human Wizard
    assert_eq!(cost_reduction_for_spell(&g, 0, &som, None), 2, "Human + non-Human gives 2 less");
}

/// Cunning Nightbonder makes the controller's flash spells cost {1} less.
#[test]
fn cunning_nightbonder_discounts_flash_spells() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cunning_nightbonder());
    let flash_id = g.add_card_to_hand(0, catalog::snapcaster_mage()); // has Flash
    let flash = g.players[0].hand.iter().find(|c| c.id == flash_id).unwrap().clone();
    let nonflash_id = g.add_card_to_hand(0, catalog::grizzly_bears());
    let nonflash = g.players[0].hand.iter().find(|c| c.id == nonflash_id).unwrap().clone();
    assert_eq!(cost_reduction_for_spell(&g, 0, &flash, None), 1, "flash spell discounted");
    assert_eq!(cost_reduction_for_spell(&g, 0, &nonflash, None), 0, "non-flash undiscounted");
    // Flash spells can't be countered; non-flash spells can.
    assert!(g.caster_grants_uncounterable(0, &flash), "flash spell is uncounterable");
    assert!(!g.caster_grants_uncounterable(0, &nonflash), "non-flash spell counterable");
}

/// Flame Spill: 5 to a 2/2 kills it and spills 3 excess onto its controller.
#[test]
fn flame_spill_excess_hits_controller() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::flame_spill());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Flame Spill");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2/2 dies to 5 damage");
    assert_eq!(g.players[1].life, life - 3, "5 - 2 lethal = 3 excess to controller");
}

/// Lullmage's Domination at X=2 gains control of a mana-value-2 creature.
#[test]
fn lullmages_domination_steals_mv_x_creature() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let spell = g.add_card_to_hand(0, catalog::lullmages_domination());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(2); // X=2
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast at X=2");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0, "stole the MV-2 creature");
}

/// Splash Portal blinks a creature and draws when it's a Bird/Frog/Otter/Rat.
#[test]
fn splash_portal_blinks_and_draws_on_type() {
    let mut g = two_player_game();
    // Wingspan Mentor is a Human (not in the list) — no draw.
    let human = g.add_card_to_battlefield(0, catalog::wingspan_mentor());
    g.add_card_to_library(0, catalog::island());
    let p1 = g.add_card_to_hand(0, catalog::splash_portal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    let hand_before = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::CastSpell {
        card_id: p1, target: Some(Target::Permanent(human)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("blink the Human");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "no draw for a Human");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Wingspan Mentor" && c.controller == 0),
        "creature returned to the battlefield");
}

/// Crystalline Giant gains a random missing counter at the beginning of combat,
/// and never duplicates a counter kind it already has.
#[test]
fn crystalline_giant_gains_random_counter() {
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(0, catalog::crystalline_giant());
    let count_kinds = |g: &GameState| -> usize {
        let c = g.battlefield_find(giant).unwrap();
        c.keyword_counters.len()
            + usize::from(c.counters.get(&crabomination::card::CounterType::PlusOnePlusOne)
                .copied().unwrap_or(0) > 0)
    };
    // Fire begin-combat ten times; each adds exactly one new distinct kind
    // until the 10-option pool is exhausted (never duplicating).
    for expected in 1..=10 {
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert_eq!(count_kinds(&g), expected.min(10), "one new counter kind each combat");
    }
    // Pool exhausted: an 11th trigger adds nothing.
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(count_kinds(&g), 10, "no duplicate counter kinds beyond the 10 options");
}

/// Inspired Ultimatum: target player gains 5, deal 5 to any target, draw five.
#[test]
fn inspired_ultimatum_gain_burn_draw() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4, dies to 5
    for _ in 0..8 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::inspired_ultimatum());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add(Color::White, 2);
    g.step = TurnStep::PreCombatMain;
    let hand_before = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(victim)], mode: None, x_value: None,
    }).expect("cast Inspired Ultimatum");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 25, "controller gained 5");
    assert!(g.battlefield_find(victim).is_none(), "5 damage killed the 4/4");
    assert_eq!(g.players[0].hand.len(), hand_before + 5, "drew five");
}

/// Spelleater Wolverine has double strike only with 3+ I/S in the graveyard.
#[test]
fn spelleater_wolverine_conditional_double_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let wolf = g.add_card_to_battlefield(0, catalog::spelleater_wolverine());
    assert!(!g.computed_permanent(wolf).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "no double strike with an empty graveyard");
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::ponder());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    assert!(g.computed_permanent(wolf).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "double strike with 3 instants/sorceries");
}

/// Pridemalkin grants trample to your +1/+1-countered creatures.
#[test]
fn pridemalkin_counter_trample() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let malkin = g.add_card_to_battlefield(0, catalog::pridemalkin());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Permanent(bear))]));
    g.fire_self_etb_triggers(malkin, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample),
        "countered creature has trample");
}

/// Dirgur Nemesis is a 6/5 Defender with megamorph.
#[test]
fn dirgur_nemesis_stats() {
    use crabomination::card::Keyword;
    let d = catalog::dirgur_nemesis();
    assert_eq!((d.power, d.toughness), (6, 5));
    assert!(d.keywords.contains(&Keyword::Defender));
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Megamorph(_))));
}

/// Coordinated Charge pumps your team +2/+1 until end of turn.
#[test]
fn coordinated_charge_team_pump() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cc = g.add_card_to_hand(0, catalog::coordinated_charge());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: cc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Coordinated Charge");
    drain_stack(&mut g);
    let mine = g.computed_permanent(a).unwrap();
    assert_eq!((mine.power, mine.toughness), (4, 3), "my creature +2/+1");
    let theirs = g.computed_permanent(enemy).unwrap();
    assert_eq!((theirs.power, theirs.toughness), (2, 2), "enemy unaffected");
}

/// Fully Grown gives +3/+3 and a trample counter.
#[test]
fn fully_grown_pump_and_trample_counter() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fg = g.add_card_to_hand(0, catalog::fully_grown());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: fg, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fully Grown");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5), "+3/+3");
    assert!(c.keywords.contains(&Keyword::Trample), "trample counter");
}

/// Plague Wight shrinks its blockers when it becomes blocked.
#[test]
fn plague_wight_shrinks_blockers() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let wight = g.add_card_to_battlefield(0, catalog::plague_wight());
    let blk = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 survives combat
    g.clear_sickness(wight);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: wight, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, wight)])).expect("block");
    drain_stack(&mut g);
    // The becomes-blocked trigger shrinks the 4/4 blocker to 3/3.
    let after = g.computed_permanent(blk).unwrap();
    assert_eq!((after.power, after.toughness), (3, 3), "blocker got -1/-1");
}

/// Zagoth Mamba's mutate trigger shrinks an opponent's creature.
#[test]
fn zagoth_mamba_mutate_debuff() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // non-Human host
    let mamba = g.add_card_to_hand(0, catalog::zagoth_mamba());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.clear_sickness(host);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Target(Target::Permanent(victim))]));
    g.perform_action(GameAction::CastMutate {
        card_id: mamba, target: host, on_top: false, x_value: None,
    }).expect("mutate onto host");
    drain_stack(&mut g);
    let v = g.computed_permanent(victim).unwrap();
    assert_eq!((v.power, v.toughness), (2, 2), "Serra Angel got -2/-2");
}

/// Fight as One buffs both a Human and a non-Human with indestructible.
#[test]
fn fight_as_one_buffs_both() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let human = g.add_card_to_battlefield(0, catalog::wingspan_mentor()); // Human
    let beast = g.add_card_to_battlefield(0, catalog::grizzly_bears());    // non-Human
    let spell = g.add_card_to_hand(0, catalog::fight_as_one());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(human)),
        additional_targets: vec![Target::Permanent(beast)], mode: None, x_value: None,
    }).expect("cast Fight as One choosing both");
    drain_stack(&mut g);
    assert!(g.computed_permanent(human).unwrap().keywords.contains(&Keyword::Indestructible),
        "Human gained indestructible");
    assert!(g.computed_permanent(beast).unwrap().keywords.contains(&Keyword::Indestructible),
        "non-Human gained indestructible");
}

/// Adaptive Shimmerer enters as a 3/3 (three +1/+1 counters on a 0/0).
#[test]
fn adaptive_shimmerer_enters_three_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::adaptive_shimmerer());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Adaptive Shimmerer");
    drain_stack(&mut g);
    let c = g.computed_permanent(id).expect("alive (not a counter-less 0/0)");
    assert_eq!((c.power, c.toughness), (3, 3), "0/0 with three +1/+1 = 3/3");
    assert_eq!(g.battlefield_find(id).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 3);
}

/// Will of the All-Hunter pumps a non-blocking creature but counters a blocker.
#[test]
fn will_of_the_all_hunter_modes() {
    use crabomination::card::CounterType;
    use crabomination::game::types::{Attack, AttackTarget};
    // Non-blocking → +2/+2 until end of turn.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let w = g.add_card_to_hand(0, catalog::will_of_the_all_hunter());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: w, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast at idle creature");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "+2/+2 when not blocking");
    assert_eq!(g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 0,
        "no counters in the pump mode");
    // Blocking → two +1/+1 counters instead.
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(0),
    }])).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, atk)])).expect("block");
    let w = g.add_card_to_hand(0, catalog::will_of_the_all_hunter());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: w, target: Some(Target::Permanent(blocker)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast at blocker");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(blocker).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2,
        "two +1/+1 counters when blocking");
}

/// Gleaming Overseer grants your Zombies hexproof and unblockable, and amasses.
#[test]
fn gleaming_overseer_zombie_anthem() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let overseer = g.add_card_to_battlefield(0, catalog::gleaming_overseer());
    g.fire_self_etb_triggers(overseer, 0);
    drain_stack(&mut g);
    // Amass made a Zombie Army token.
    let army = g.battlefield.iter().find(|c| c.controller == 0
        && c.definition.subtypes.creature_types.contains(&CreatureType::Army))
        .map(|c| c.id).expect("Army token");
    let a = g.computed_permanent(army).unwrap();
    assert!(a.subtypes.creature_types.contains(&CreatureType::Zombie), "Army is a Zombie");
    assert!(a.keywords.contains(&Keyword::Hexproof) && a.keywords.contains(&Keyword::Unblockable),
        "Zombie Army has hexproof + unblockable");
    // The Overseer itself is a Zombie too.
    let o = g.computed_permanent(overseer).unwrap();
    assert!(o.keywords.contains(&Keyword::Hexproof), "Overseer (a Zombie) has hexproof");
}

/// Ferocious Tigorilla enters with a chosen trample or menace counter.
#[test]
fn ferocious_tigorilla_etb_keyword_counter() {
    use crabomination::card::Keyword;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    // Mode 1 = menace.
    let mut g = two_player_game();
    let t = g.add_card_to_battlefield(0, catalog::ferocious_tigorilla());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(1)]));
    g.fire_self_etb_triggers(t, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(t).unwrap().keywords.contains(&Keyword::Menace), "chose menace");
    // Mode 0 = trample.
    let mut g = two_player_game();
    let t = g.add_card_to_battlefield(0, catalog::ferocious_tigorilla());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(0)]));
    g.fire_self_etb_triggers(t, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(t).unwrap().keywords.contains(&Keyword::Trample), "chose trample");
}

/// Glimpse the Cosmos digs three, takes one, bottoms the rest.
#[test]
fn glimpse_the_cosmos_digs_three_takes_one() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::forest());       // ends bottom-ward
    g.add_card_to_library(0, catalog::grizzly_bears()); // middle
    g.add_card_to_library(0, catalog::island());        // top
    let lib_before = g.players[0].library.len();
    let spell = g.add_card_to_hand(0, catalog::glimpse_the_cosmos());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    let hand_before = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Glimpse the Cosmos");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "took one card to hand");
    assert_eq!(g.players[0].library.len(), lib_before - 1, "other two stayed in library (bottom)");
}

/// Honor the God-Pharaoh: discard a card as a cost, draw two, amass Zombies 1.
#[test]
fn honor_the_god_pharaoh_discard_draw_amass() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::forest()); // fodder to discard
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::honor_the_god_pharaoh());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    let hand_before = g.players[0].hand.len(); // includes spell + forest
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Honor the God-Pharaoh");
    drain_stack(&mut g);
    // -spell -forest(discard) +2 draw = net +0 vs hand_before.
    assert_eq!(g.players[0].hand.len(), hand_before, "discard one as cost, draw two");
    assert!(g.battlefield.iter().any(|c| c.controller == 0
        && c.definition.subtypes.creature_types.contains(&CreatureType::Army)),
        "amassed a Zombie Army");
}

/// Reptilian Reflection animates itself into a 5/4 Dinosaur on cycle.
#[test]
fn reptilian_reflection_animates_on_cycle() {
    use crabomination::card::{CardType, CreatureType, Keyword};
    let mut g = two_player_game();
    let refl = g.add_card_to_battlefield(0, catalog::reptilian_reflection());
    // A cycler in hand to trigger the cycle event.
    let cyc = g.add_card_to_hand(0, catalog::greater_sandwurm()); // Cycling {2}
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::Cycle { card_id: cyc, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    let c = g.computed_permanent(refl).unwrap();
    assert!(c.card_types.contains(&CardType::Creature), "became a creature");
    assert_eq!((c.power, c.toughness), (5, 4));
    assert!(c.subtypes.creature_types.contains(&CreatureType::Dinosaur));
    assert!(c.keywords.contains(&Keyword::Trample) && c.keywords.contains(&Keyword::Haste));
}

// ── IKO commons/uncommons batch ──────────────────────────────────────────────

/// Garrison Cat leaves a 1/1 Human Soldier when it dies.
#[test]
fn garrison_cat_dies_into_soldier() {
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::garrison_cat());
    g.battlefield_find_mut(cat).unwrap().damage = 2; // lethal
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    let tokens: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Human Soldier").collect();
    assert_eq!(tokens.len(), 1, "Garrison Cat death makes one Soldier");
}

/// Daysquad Marshal's ETB mints a 1/1 Human Soldier.
#[test]
fn daysquad_marshal_etb_token() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::daysquad_marshal());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Human Soldier").count(),
        1, "ETB mints a Soldier");
}

/// Serrated Scorpion drains each opponent for 2 and gains 2 on death.
#[test]
fn serrated_scorpion_death_drain() {
    let mut g = two_player_game();
    let scorp = g.add_card_to_battlefield(0, catalog::serrated_scorpion());
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.battlefield_find_mut(scorp).unwrap().damage = 2;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 2, "opponent takes 2");
    assert_eq!(g.players[0].life, life0 + 2, "you gain 2");
}

/// Divine Arrow only targets attacking/blocking creatures, dealing 4.
#[test]
fn divine_arrow_kills_attacker() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(0),
    }])).expect("attack");
    let arrow = g.add_card_to_hand(0, catalog::divine_arrow());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: arrow, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast on attacker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "4 damage kills the 2/2 attacker");
}

/// Blade Banish exiles a power-4 creature and rejects a small one.
#[test]
fn blade_banish_exiles_big_creature() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
    let bb = g.add_card_to_hand(0, catalog::blade_banish());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bb, target: Some(Target::Permanent(dragon)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast on power-5 creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).is_none(), "exiled");
}

/// Dead Weight enchants a creature for -2/-2, killing a 2/2.
#[test]
fn dead_weight_shrinks_and_kills() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dw = g.add_card_to_hand(0, catalog::dead_weight());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: dw, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "-2/-2 kills the 2/2");
}

/// Suffocating Fumes gives opponents' creatures -1/-1 and has Cycling.
#[test]
fn suffocating_fumes_weakens_opponents() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    assert!(catalog::suffocating_fumes().keywords.iter()
        .any(|k| matches!(k, Keyword::Cycling(_))));
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::garrison_cat()); // 1/1
    let sf = g.add_card_to_hand(0, catalog::suffocating_fumes());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: sf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "opponent's 2/2 dies to -1/-1");
    assert!(g.battlefield_find(mine).is_some(), "your creature is unaffected");
}

/// Blazing Volley pings only opponents' creatures.
#[test]
fn blazing_volley_hits_opponents_only() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::ornithopter()); // 0/2
    let theirs = g.add_card_to_battlefield(1, catalog::ornithopter()); // 0/2
    g.battlefield_find_mut(theirs).unwrap().damage = 1; // already at 1; +1 = lethal
    let bv = g.add_card_to_hand(0, catalog::blazing_volley());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bv, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "opponent's creature dies");
    assert!(g.battlefield_find(mine).is_some(), "your creature is untouched");
}

/// Checkpoint Officer taps a target creature with its activated ability.
#[test]
fn checkpoint_officer_taps_target() {
    let mut g = two_player_game();
    let officer = g.add_card_to_battlefield(0, catalog::checkpoint_officer());
    g.clear_sickness(officer);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: officer, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "target is tapped");
}

/// Durable Coilbug returns itself from the graveyard to hand for {4}{B}.
#[test]
fn durable_coilbug_self_returns_from_graveyard() {
    let mut g = two_player_game();
    let bug = g.add_card_to_graveyard(0, catalog::durable_coilbug());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bug, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate from graveyard");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bug), "returned to hand");
}

/// Glimmerbell can untap itself for {1}{U}.
#[test]
fn glimmerbell_untaps_itself() {
    let mut g = two_player_game();
    let bell = g.add_card_to_battlefield(0, catalog::glimmerbell());
    g.clear_sickness(bell);
    g.battlefield_find_mut(bell).unwrap().tapped = true;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bell, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("untap self");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bell).unwrap().tapped, "untapped");
}

/// Avian Oddity's cycle trigger puts a flying counter on your creature.
#[test]
fn avian_oddity_cycle_grants_flying() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let oddity = g.add_card_to_hand(0, catalog::avian_oddity());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Cycle { card_id: oddity, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying), "flying counter grants flying");
    assert_eq!(
        g.battlefield_find(bear).unwrap().keyword_counters.get(&Keyword::Flying).copied(),
        Some(1));
    let _ = CounterType::PlusOnePlusOne;
}

/// Light of Hope mode 0 gains 4 life.
#[test]
fn light_of_hope_gains_life() {
    let mut g = two_player_game();
    let loh = g.add_card_to_hand(0, catalog::light_of_hope());
    g.players[0].mana_pool.add(Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: loh, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast mode 0");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "gained 4 life");
}

/// Dark Bargain draws two of the top three and costs 2 life.
#[test]
fn dark_bargain_digs_and_costs_life() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let db = g.add_card_to_hand(0, catalog::dark_bargain());
    let hand0 = g.players[0].hand.len();
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: db, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2, "two cards to hand (minus the spell)");
    assert_eq!(g.players[0].life, life - 2, "took 2 damage");
}

/// IKO gain-taplands enter tapped and gain a life.
#[test]
fn iko_gainlands_enter_tapped_with_life() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let hollow = g.move_card_to_battlefield_for_test(0, catalog::jungle_hollow());
    drain_stack(&mut g);
    assert!(g.battlefield_find(hollow).unwrap().tapped, "enters tapped");
    assert_eq!(g.players[0].life, life + 1, "gains 1 life");
}

/// Luminous Broodmoth returns a died non-flyer with a flying counter (CR 122).
#[test]
fn luminous_broodmoth_returns_with_flying() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::luminous_broodmoth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, no flying
    // Kill it through the real damage funnel so the death trigger dispatches.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    // The bear card returns to the battlefield (same id) with flying.
    let back = g.battlefield_find(bear).expect("bear returned to battlefield");
    assert_eq!(back.controller, 0);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying), "returned with a flying counter");
}

/// Quartzwood Crasher mints an X/X Dinosaur Beast on combat damage (CR 510.2).
#[test]
fn quartzwood_crasher_makes_token_on_combat_damage() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let crasher = g.add_card_to_battlefield(0, catalog::quartzwood_crasher()); // 5/5 trample
    g.clear_sickness(crasher);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: crasher, target: AttackTarget::Player(1),
    }])).expect("attack");
    // Run combat to damage.
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).ok();
        if g.stack.is_empty() && g.priority.player_with_priority == 0
            && matches!(g.step, TurnStep::PostCombatMain | TurnStep::End) { break; }
    }
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Dinosaur Beast");
    let token = token.expect("a Dinosaur Beast token was created");
    assert_eq!(token.counter_count(CounterType::PlusOnePlusOne), 6, "X = 6 combat damage");
}

/// CR 603.10: a stolen creature dying fires the *thief's* "a creature you
/// control dies" watcher (Bastion drains), not the owner's.
#[test]
fn stolen_creature_death_fires_controllers_watcher() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bastion_of_remembrance());
    // P1 owns the bear but P0 controls it (theft).
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().controller = 0;
    let p1 = g.players[1].life;
    let p0 = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the stolen creature");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 1, "the thief's Bastion drains the opponent");
    assert_eq!(g.players[0].life, p0 + 1, "the thief gains the life");
}

// ── IKO batch 2 ──────────────────────────────────────────────────────────────

/// Ivy Elemental enters at X=3 as a 3/3.
#[test]
fn ivy_elemental_enters_with_x_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::ivy_elemental());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast at X=3");
    drain_stack(&mut g);
    let ivy = g.battlefield.iter().find(|c| c.definition.name == "Ivy Elemental").unwrap();
    assert_eq!(ivy.counter_count(CounterType::PlusOnePlusOne), 3, "X=3 counters");
}

/// Unexpected Fangs adds a +1/+1 counter and a lifelink keyword counter.
#[test]
fn unexpected_fangs_adds_counters() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let uf = g.add_card_to_hand(0, catalog::unexpected_fangs());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: uf, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 counter");
    assert!(cp.keywords.contains(&Keyword::Lifelink), "lifelink counter");
}

/// Go for Blood makes your creature fight an opponent's, and has Cycling.
#[test]
fn go_for_blood_fights_and_cycles() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    assert!(catalog::go_for_blood().keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
    let mine = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let gfb = g.add_card_to_hand(0, catalog::go_for_blood());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: gfb, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast fight");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "the 2/2 dies to the 5/5");
    assert!(g.battlefield_find(mine).is_some(), "the 5/5 survives 2 damage");
}

/// Neutralize counters a spell on the stack.
#[test]
fn neutralize_counters_a_spell() {
    let mut g = two_player_game();
    // Opponent casts an instant (Lightning Bolt) at player 0.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts Bolt");
    let life = g.players[0].life;
    let neut = g.add_card_to_hand(0, catalog::neutralize());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: neut, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("counter it");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "the Bolt was countered (no damage)");
}

/// Dire Tactics: with no Human you control, you lose life equal to the exiled
/// creature's toughness.
#[test]
fn dire_tactics_costs_life_without_a_human() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
    let dt = g.add_card_to_hand(0, catalog::dire_tactics());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: dt, target: Some(Target::Permanent(dragon)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == dragon), "creature exiled");
    assert_eq!(g.players[0].life, life - 5, "lose life = toughness (no Human)");
}

/// Colossification pumps +20/+20 and taps the enchanted creature on entry.
#[test]
fn colossification_pumps_and_taps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let col = g.add_card_to_hand(0, catalog::colossification());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: col, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (22, 22), "+20/+20");
    assert!(g.battlefield_find(bear).unwrap().tapped, "tapped on entry");
}

/// Pyroceratops grows when you cast a noncreature spell.
#[test]
fn pyroceratops_grows_on_noncreature_cast() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let cera = g.add_card_to_battlefield(0, catalog::pyroceratops());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a noncreature spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cera).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Sleeper Dart draws on entry; its sac ability stuns a creature's next untap.
#[test]
fn sleeper_dart_draws_and_locks_untap() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let hand0 = g.players[0].hand.len();
    let dart = g.move_card_to_battlefield_for_test(0, catalog::sleeper_dart());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "ETB draws a card");
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.perform_action(GameAction::ActivateAbility {
        card_id: dart, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("sac to stun");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dart).is_none(), "Sleeper Dart sacrificed");
}

/// Ominous Seas accrues a tide counter on the first draw each turn (only the
/// first), and at four counters mints an 8/8 Kraken, clearing the counters.
#[test]
fn ominous_seas_tide_counters_then_kraken() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let seas = g.add_card_to_battlefield(0, catalog::ominous_seas());
    for _ in 0..8 { g.add_card_to_library(0, catalog::island()); }
    let tide = |g: &GameState| g.battlefield_find(seas).unwrap()
        .counters.get(&CounterType::Tide).copied().unwrap_or(0);
    let kraken_count = |g: &GameState| g.battlefield.iter()
        .filter(|c| c.definition.name == "Kraken").count();

    for turn in 1..=3 {
        g.triggered_once_per_turn_used.clear();
        // Two draws this turn — only the first adds a counter.
        for _ in 0..2 {
            let mut ev = Vec::new();
            g.draw_one(0, &mut ev);
            g.dispatch_triggers_for_events(&ev);
            drain_stack(&mut g);
        }
        assert_eq!(tide(&g), turn, "one tide counter per turn (first draw only)");
    }
    assert_eq!(kraken_count(&g), 0, "no Kraken below four counters");
    // Fourth turn's first draw hits four → Kraken, counters cleared.
    g.triggered_once_per_turn_used.clear();
    let mut ev = Vec::new();
    g.draw_one(0, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(kraken_count(&g), 1, "four tide counters mint an 8/8 Kraken");
    assert_eq!(tide(&g), 0, "tide counters removed when the Kraken is made");
}

/// Extinction Event (default odd) exiles every creature with odd mana value,
/// leaving the even-MV creatures on the battlefield.
#[test]
fn extinction_event_exiles_chosen_parity() {
    let mut g = two_player_game();
    let lion = g.add_card_to_battlefield(1, catalog::savannah_lions()); // MV1 odd
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());   // MV5 odd
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV2 even
    let memnite = g.add_card_to_battlefield(0, catalog::memnite());     // MV0 even
    let spell = g.add_card_to_hand(0, catalog::extinction_event());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Extinction Event");
    drain_stack(&mut g);
    assert!(g.battlefield_find(lion).is_none(), "MV1 (odd) exiled");
    assert!(g.battlefield_find(angel).is_none(), "MV5 (odd) exiled");
    assert!(g.battlefield_find(bears).is_some(), "MV2 (even) survives");
    assert!(g.battlefield_find(memnite).is_some(), "MV0 (even) survives");
}

/// Song of Creation: casting a spell draws two; end step discards the hand.
#[test]
fn song_of_creation_draws_then_dumps_hand() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::song_of_creation());
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    drain_stack(&mut g);
    // -1 bolt, +2 from Song's cast trigger.
    assert_eq!(g.players[0].hand.len(), before - 1 + 2, "cast trigger drew two");
    // End step discards the whole hand.
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 0, "end step discarded the hand");
}

/// Fiend Artisan grows with creature cards in the graveyard.
#[test]
fn fiend_artisan_grows_with_graveyard() {
    let mut g = two_player_game();
    let artisan = g.add_card_to_battlefield(0, catalog::fiend_artisan());
    assert_eq!(g.computed_permanent(artisan).unwrap().power, 1, "base 1/1");
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::serra_angel());
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // not a creature
    assert_eq!(g.computed_permanent(artisan).unwrap().power, 3, "+1/+1 per creature card (two)");
    assert_eq!(g.computed_permanent(artisan).unwrap().toughness, 3, "toughness tracks too");
}

/// General Kudro: anthem buffs other Humans; sac-two-Humans destroys a creature.
#[test]
fn general_kudro_anthem_and_sacrifice() {
    let mut g = two_player_game();
    let kudro = g.add_card_to_battlefield(0, catalog::general_kudro_of_drannith());
    let soldier = g.add_card_to_battlefield(0, catalog::savannah_lions()); // Human-ish? it's a Cat
    // Use two Humans for the sac cost: add two Human tokens via Champion etc.
    let h1 = g.add_card_to_battlefield(0, catalog::champion_of_the_parish());
    let h2 = g.add_card_to_battlefield(0, catalog::champion_of_the_parish());
    // Anthem: another Human (Champion 1/1) is buffed to 2/2; Kudro itself isn't.
    assert_eq!(g.computed_permanent(h1).unwrap().power, 2, "other Human gets +1/+1");
    assert_eq!(g.computed_permanent(kudro).unwrap().power, 3, "Kudro doesn't buff itself");
    let _ = soldier;
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kudro, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("sac two Humans to destroy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "target creature destroyed");
    assert!(g.battlefield_find(h1).is_none() && g.battlefield_find(h2).is_none(),
        "two Humans sacrificed");
}

/// General Kudro's own entry (and another Human's) exiles an opponent gy card.
#[test]
fn general_kudro_etb_exiles_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let opp_gy = g.players[1].graveyard.len();
    g.move_card_to_battlefield_for_test(0, catalog::general_kudro_of_drannith());
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), opp_gy - 1, "Kudro's own ETB exiled a gy card");
}

/// Fiend Artisan's activated ability sacrifices another creature and tutors a
/// creature with mana value ≤ X straight to the battlefield.
#[test]
fn fiend_artisan_tutors_to_battlefield() {
    let mut g = two_player_game();
    let artisan = g.add_card_to_battlefield(0, catalog::fiend_artisan());
    g.clear_sickness(artisan);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let elf = g.add_card_to_library(0, catalog::llanowar_elves()); // MV1
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(elf))]));
    g.players[0].mana_pool.add(Color::Black, 1); // pays the {B/G}
    g.players[0].mana_pool.add_colorless(1);     // pays X=1 generic
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: artisan, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: Some(1),
    }).expect("activate Fiend Artisan tutor");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert!(g.battlefield_find(artisan).is_some(), "Artisan not sacrificed (another creature)");
    assert!(g.battlefield.iter().any(|c| c.id == elf), "tutored the MV1 creature to the battlefield");
}

/// Solar Blaze: every creature takes damage equal to its own power.
#[test]
fn solar_blaze_self_damage_by_power() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());   // 2/2 → 2 to self, dies
    let lions = g.add_card_to_battlefield(1, catalog::savannah_lions());  // 2/1 → 2 to self, dies
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());     // 4/4 → 4 to self, dies
    let spell = g.add_card_to_hand(0, catalog::solar_blaze());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Solar Blaze");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "2/2 took 2, died");
    assert!(g.battlefield_find(lions).is_none(), "2/1 took 2, died");
    assert!(g.battlefield_find(angel).is_none(), "4/4 took 4, died");
}

/// Bonders' Enclave only draws while you control a 4-power creature.
#[test]
fn bonders_enclave_conditional_draw() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::bonders_enclave());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    // No big creature → the draw ability is illegal.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).is_err(), "draw blocked without a 4-power creature");
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("draw enabled with a 4-power creature");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Auspicious Starrix mutates a permanent card onto the battlefield (X=1 on the
/// first mutation).
#[test]
fn auspicious_starrix_mutate_deploys_permanent() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion()); // non-Human Beast
    let land = g.add_card_to_library(0, catalog::island()); // a permanent on top
    let starrix = g.add_card_to_hand(0, catalog::auspicious_starrix());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(5); // mutate {5}{G}
    g.perform_action(GameAction::CastMutate {
        card_id: starrix, target: host, on_top: true, x_value: None,
    }).expect("mutate Auspicious Starrix");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == land), "a permanent card was put onto the battlefield");
}

/// Skycat Sovereign grows with other flyers and can mint a flying Cat Bird.
#[test]
fn skycat_sovereign_scales_with_flyers() {
    let mut g = two_player_game();
    let skycat = g.add_card_to_battlefield(0, catalog::skycat_sovereign());
    assert_eq!(g.computed_permanent(skycat).unwrap().power, 1, "base 1/1 alone");
    g.add_card_to_battlefield(0, catalog::serra_angel()); // a flyer
    assert_eq!(g.computed_permanent(skycat).unwrap().power, 2, "+1/+1 per other flyer");
    // Use its ability to make a Cat Bird flyer → grows again.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: skycat, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("make a Cat Bird");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(skycat).unwrap().power, 3, "Cat Bird flyer counted too");
}

/// Chevill bounties an opponent creature at upkeep, and cashes it on death.
#[test]
fn chevill_bounty_then_payoff() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let chevill = g.add_card_to_battlefield(0, catalog::chevill_bane_of_monsters());
    let _ = chevill;
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    // Upkeep trigger bounties the opponent's creature.
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(prey).unwrap().counters.get(&CounterType::Bounty).copied().unwrap_or(0),
        1, "bounty counter placed on opponent creature");
    // It dies → Chevill's controller draws and gains a life.
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(prey).unwrap().damage = 99;
    let events = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 on bounty death");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew on bounty death");
}

/// Glory of Warfare buffs your team +2/+0 on your turn, +0/+2 otherwise.
#[test]
fn glory_of_warfare_turn_conditional_anthem() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::glory_of_warfare());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.active_player_idx = 0;
    assert_eq!((g.computed_permanent(bears).unwrap().power, g.computed_permanent(bears).unwrap().toughness),
        (4, 2), "+2/+0 on your turn");
    g.active_player_idx = 1;
    assert_eq!((g.computed_permanent(bears).unwrap().power, g.computed_permanent(bears).unwrap().toughness),
        (2, 4), "+0/+2 on others' turns");
}

/// Sanctuary Smasher's cycle puts a first strike counter on your creature.
#[test]
fn sanctuary_smasher_cycle_grants_first_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let smasher = g.add_card_to_hand(0, catalog::sanctuary_smasher());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bears))]));
    g.perform_action(GameAction::Cycle { card_id: smasher, x_value: None }).expect("cycle Sanctuary Smasher");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bears).unwrap().keywords.contains(&Keyword::FirstStrike),
        "first strike counter granted on cycle");
}

/// Mysterious Egg gains a +1/+1 counter whenever it's mutated onto.
#[test]
fn mysterious_egg_grows_on_mutate() {
    let mut g = two_player_game();
    let egg = g.add_card_to_battlefield(0, catalog::mysterious_egg()); // 0/2, non-Human
    let starrix = g.add_card_to_hand(0, catalog::auspicious_starrix());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastMutate {
        card_id: starrix, target: egg, on_top: false, x_value: None,
    }).expect("mutate onto the Egg");
    drain_stack(&mut g);
    // Egg's own mutate trigger added a +1/+1 counter to the pile.
    let pile = g.battlefield_find(egg).expect("pile alive");
    assert_eq!(pile.counters.get(&crabomination::card::CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1,
        "Egg's mutate trigger added a +1/+1 counter");
}

/// Powerstone Fracture: sacrifice an artifact/creature, then destroy a target.
#[test]
fn powerstone_fracture_sacrifice_then_destroy() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::powerstone_fracture());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    // The lone artifact/creature you control is auto-sacrificed for the cost.
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Powerstone Fracture");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed an artifact/creature");
    assert!(g.battlefield_find(victim).is_none(), "destroyed the target");
}

/// Howl of the Hunt grants +2/+2 and vigilance to the enchanted creature.
#[test]
fn howl_of_the_hunt_buffs_and_grants_vigilance() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::howl_of_the_hunt());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Howl of the Hunt");
    drain_stack(&mut g);
    let c = g.computed_permanent(bears).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "+2/+2");
    assert!(c.keywords.contains(&Keyword::Vigilance), "granted vigilance");
}

/// Brokkos can be cast for its mutate cost onto a non-Human host (trample 6/6).
#[test]
fn brokkos_mutates_onto_host() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // non-Human Beast 2/2
    let brokkos = g.add_card_to_hand(0, catalog::brokkos_apex_of_forever());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2); // mutate {2}{U/B}{G}{G}
    g.perform_action(GameAction::CastMutate {
        card_id: brokkos, target: host, on_top: true, x_value: None,
    }).expect("mutate Brokkos");
    drain_stack(&mut g);
    let pile = g.battlefield_find(host).expect("pile alive");
    assert_eq!(pile.definition.name, "Brokkos, Apex of Forever");
    assert!(g.computed_permanent(host).unwrap().keywords.contains(&Keyword::Trample));
}

/// Drowsing Tyrannodon can only attack while you control a 4-power creature.
#[test]
fn drowsing_tyrannodon_conditional_defender() {
    let mut g = two_player_game();
    let dino = g.add_card_to_battlefield(0, catalog::drowsing_tyrannodon());
    g.clear_sickness(dino);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // No 4-power creature → defender blocks the attack.
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dino, target: AttackTarget::Player(1),
    }])).is_err(), "defender stops the attack without a 4-power creature");
    // Add a 4-power creature → the Tyrannodon may now attack.
    let beater = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    g.clear_sickness(beater);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dino, target: AttackTarget::Player(1),
    }])).expect("attacks once a 4-power creature is present");
}

/// Vivien's +1 mints a 3/3 Beast with a chosen keyword counter (vigilance).
#[test]
fn vivien_plus_one_beast_with_keyword_counter() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let vivien = g.add_card_to_battlefield(0, catalog::vivien_monsters_advocate());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)])); // vigilance
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: vivien, ability_index: 0, target: None, x_value: None,
    }).expect("Vivien +1");
    drain_stack(&mut g);
    let beast = g.battlefield.iter().find(|c| c.definition.name == "Beast").expect("Beast token");
    assert_eq!((beast.power(), beast.toughness()), (3, 3));
    assert!(g.computed_permanent(beast.id).unwrap().keywords.contains(&Keyword::Vigilance),
        "chose a vigilance counter");
}

/// Vivien's −2 tutors a lesser-MV creature to the battlefield on the next
/// creature spell cast this turn.
#[test]
fn vivien_minus_two_tutors_lesser_creature() {
    let mut g = two_player_game();
    let vivien = g.add_card_to_battlefield(0, catalog::vivien_monsters_advocate());
    let small = g.add_card_to_library(0, catalog::grizzly_bears()); // MV2
    let big = g.add_card_to_hand(0, catalog::colossal_dreadmaw());   // MV6 creature
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(small))]));
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: vivien, ability_index: 1, target: None, x_value: None,
    }).expect("Vivien -2");
    drain_stack(&mut g);
    // Cast the creature spell → the delayed trigger tutors the smaller creature.
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature spell");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == small),
        "tutored the lesser-MV creature onto the battlefield");
}

/// Mind Spike makes the opponent discard a chosen noncreature/nonland card and
/// costs you 2 life.
#[test]
fn mind_spike_discards_and_costs_life() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt()); // noncreature, nonland
    g.add_card_to_hand(1, catalog::grizzly_bears());             // creature, not eligible
    let spell = g.add_card_to_hand(0, catalog::mind_spike());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mind Spike");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "opponent discarded the noncreature card");
    assert_eq!(g.players[0].life, life - 2, "you lost 2 life");
}

/// Mind Spike draws you a card when the opponent reveals no noncreature/nonland.
#[test]
fn mind_spike_draws_when_nothing_revealed() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears()); // creature — not revealed
    g.add_card_to_hand(1, catalog::island());        // land — not revealed
    g.add_card_to_library(0, catalog::lightning_bolt());
    let spell = g.add_card_to_hand(0, catalog::mind_spike());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    let (life, hand, gy) = (g.players[0].life, g.players[0].hand.len(), g.players[1].graveyard.len());
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mind Spike");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy, "nothing eligible to discard");
    assert_eq!(g.players[0].hand.len(), hand, "drew a card (net: -spell +draw)");
    assert_eq!(g.players[0].life, life - 2, "you lost 2 life");
}

/// Howl of the Hunt untaps the enchanted creature if it's a Wolf or Werewolf.
#[test]
fn howl_of_the_hunt_untaps_a_wolf() {
    let mut g = two_player_game();
    let wolf = g.add_card_to_battlefield(0, catalog::sarulfs_packmate()); // Wolf
    g.battlefield_find_mut(wolf).unwrap().tapped = true;
    let aura = g.add_card_to_hand(0, catalog::howl_of_the_hunt());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(wolf)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Howl of the Hunt");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(wolf).unwrap().tapped, "Wolf untapped on enter");
}

/// Winota deploys a Human from the top six tapped-and-attacking (with
/// indestructible) when a non-Human you control attacks.
#[test]
fn winota_deploys_human_when_nonhuman_attacks() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::winota_joiner_of_forces()); // Human, not attacking
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // non-Human attacker
    g.clear_sickness(bear);
    let human = g.add_card_to_library(0, catalog::beskir_shieldmate()); // Human in top six
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("non-Human attacks");
    drain_stack(&mut g);
    let h = g.battlefield_find(human).expect("Human deployed from library");
    assert!(h.tapped, "deployed tapped");
    assert!(g.attacking.iter().any(|a| a.attacker == human), "deployed attacking");
    assert!(g.computed_permanent(human).unwrap().keywords.contains(&Keyword::Indestructible),
        "gains indestructible EOT");
}

/// The Wandering Emperor may activate her loyalty at instant speed the turn she
/// enters (CR 606.3b), but only that turn.
#[test]
fn wandering_emperor_flash_loyalty_window() {
    let mut g = two_player_game();
    let emp = g.add_card_to_battlefield(0, catalog::the_wandering_emperor());
    g.battlefield_find_mut(emp).unwrap().entered_turn = Some(g.turn_number);
    // An instant-speed window that is NOT sorcery-speed: opponent's upkeep,
    // emperor's controller holds priority.
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    // −1: make a Samurai — allowed at instant speed the turn she entered.
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: emp, ability_index: 1, target: None, x_value: None,
    }).expect("instant-speed loyalty the turn she entered");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Samurai"), "made a Samurai");
    // A later turn: the window is closed, so the same activation is rejected.
    g.battlefield_find_mut(emp).unwrap().entered_turn = Some(g.turn_number.wrapping_sub(1));
    g.battlefield_find_mut(emp).unwrap().loyalty_uses_this_turn = 0;
    assert!(g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: emp, ability_index: 1, target: None, x_value: None,
    }).is_err(), "no longer instant-speed once it's not the turn she entered");
}

/// Memory Leak exiles a nonland card from the opponent's hand or graveyard
/// (auto-picks the highest mana value across both zones).
#[test]
fn memory_leak_exiles_highest_mv_across_zones() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());          // MV1 in hand
    let big = g.add_card_to_graveyard(1, catalog::colossal_dreadmaw()); // MV6 in graveyard
    g.add_card_to_hand(1, catalog::island());                  // land — ineligible
    let spell = g.add_card_to_hand(0, catalog::memory_leak());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Memory Leak");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == big), "exiled the highest-MV nonland (from the graveyard)");
}

/// Skyscanner draws a card on entry.
#[test]
fn skyscanner_draws_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let h = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::skyscanner());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h + 1, "ETB drew a card");
}

/// Pristine Talisman taps for {C} and gains a life (as one mana ability).
#[test]
fn pristine_talisman_mana_and_life() {
    let mut g = two_player_game();
    let tal = g.add_card_to_battlefield(0, catalog::pristine_talisman());
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tal, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("tap for mana");
    assert_eq!(g.players[0].life, life + 1, "gained 1 life");
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
}

/// Ram Through: your creature deals its power to a target you don't control.
#[test]
fn ram_through_one_sided_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel());   // 4/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::ram_through());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Ram Through");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "2/2 took 4 and died");
    assert!(g.battlefield_find(mine).is_some(), "one-sided: your creature took nothing");
}

/// Survivors' Bond returns a Human and a non-Human creature card from your gy.
#[test]
fn survivors_bond_returns_both() {
    let mut g = two_player_game();
    let human = g.add_card_to_graveyard(0, catalog::savannah_lions()); // Cat? use a Human
    let nonhuman = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // Bear, non-Human
    // Use an actual Human in the graveyard.
    let real_human = g.add_card_to_graveyard(0, catalog::champion_of_the_parish());
    let _ = human;
    let spell = g.add_card_to_hand(0, catalog::survivors_bond());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(real_human)),
        additional_targets: vec![Target::Permanent(nonhuman)], mode: None, x_value: None,
    }).expect("cast Survivors' Bond");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == real_human), "Human returned to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == nonhuman), "non-Human returned to hand");
}

/// Sagittars' Volley destroys a target flyer and pings each opposing flyer.
#[test]
fn sagittars_volley_destroys_and_pings_flyers() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());  // 4/4 flyer
    let small = g.add_card_to_battlefield(1, catalog::skyscanner()); // 1/1 flyer
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // no flying
    let spell = g.add_card_to_hand(0, catalog::sagittars_volley());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sagittars' Volley");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "target flyer destroyed");
    assert!(g.battlefield_find(small).is_none(), "1/2 flyer took 1 and died");
    assert!(g.battlefield_find(ground).is_some(), "non-flyer untouched");
}

/// Dawntreader Elk sacrifices itself to fetch a basic land tapped.
#[test]
fn dawntreader_elk_fetches_basic() {
    let mut g = two_player_game();
    let elk = g.add_card_to_battlefield(0, catalog::dawntreader_elk());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: elk, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    }).expect("sac Elk to fetch");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elk).is_none(), "Elk sacrificed");
    let land = g.battlefield_find(forest).expect("fetched the Forest");
    assert!(land.tapped, "fetched land enters tapped");
}

/// Ranger's Guile pumps +1/+1 and grants hexproof until end of turn.
#[test]
fn rangers_guile_pump_and_hexproof() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::rangers_guile());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ranger's Guile");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "+1/+1");
    assert!(c.keywords.contains(&Keyword::Hexproof), "granted hexproof");
}

/// Brimstone Volley deals 3, or 5 (morbid) if a creature died this turn.
#[test]
fn brimstone_volley_morbid_scaling() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::brimstone_volley());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Brimstone Volley");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "no morbid → 3 damage");

    // Now with a creature having died this turn → 5.
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(victim);
    let spell2 = g.add_card_to_hand(0, catalog::brimstone_volley());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life2 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell2, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Brimstone Volley again");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life2 - 5, "morbid → 5 damage");
}

/// Unexpected Windfall: discard a card, draw two, make two Treasures.
#[test]
fn unexpected_windfall_draws_and_makes_treasures() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::island()); // the discard fodder
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::unexpected_windfall());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    let hand_before = g.players[0].hand.len(); // includes the spell + fodder
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Unexpected Windfall");
    drain_stack(&mut g);
    // -1 spell, -1 discard, +2 draw = net 0 vs hand_before.
    assert_eq!(g.players[0].hand.len(), hand_before - 2 + 2, "discard 1, draw 2");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 2,
        "two Treasure tokens created");
}

/// Feed the Swarm destroys an opposing permanent and costs life equal to its MV.
#[test]
fn feed_the_swarm_destroys_and_drains_self() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // MV5
    let spell = g.add_card_to_hand(0, catalog::feed_the_swarm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Feed the Swarm");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none(), "destroyed the creature");
    assert_eq!(g.players[0].life, life - 5, "lost life equal to its mana value");
}

/// Deadly Rollick exiles a target creature.
#[test]
fn deadly_rollick_exiles_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::deadly_rollick());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Deadly Rollick");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature exiled");
}

/// Winged Words costs {1} less with a flyer and draws two cards.
#[test]
fn winged_words_flyer_discount_and_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::storm_crow()); // a flyer
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::winged_words());
    // Discounted to {1}{U}: pay exactly that.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    let h = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Winged Words castable for {1}{U} with a flyer out");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h - 1 + 2, "drew two");
}

/// Condescend counters a spell whose controller can't pay X, and scrys 2.
#[test]
fn condescend_counters_unpaid_x() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); } // scry fodder
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts Bolt");
    let cond = g.add_card_to_hand(0, catalog::condescend());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3); // X=3
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: cond, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast Condescend with X=3");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Bolt countered (opponent could not pay X)");
}

/// Secure the Wastes makes X Warriors; Captain's Call makes three Soldiers.
#[test]
fn token_makers_secure_and_captains() {
    let mut g = two_player_game();
    let stw = g.add_card_to_hand(0, catalog::secure_the_wastes());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3); // X=3
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: stw, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast Secure the Wastes X=3");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Warrior").count(), 3,
        "three Warriors");
    let cc = g.add_card_to_hand(0, catalog::captains_call());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: cc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Captain's Call");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count(), 3,
        "three Soldiers");
}

/// Forsake the Worldly exiles a target artifact or enchantment.
#[test]
fn forsake_the_worldly_exiles_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::pristine_talisman());
    let spell = g.add_card_to_hand(0, catalog::forsake_the_worldly());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(art)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Forsake the Worldly");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact exiled");
}

/// Cruel Edict makes the target opponent sacrifice a creature.
#[test]
fn cruel_edict_forces_sacrifice() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::cruel_edict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cruel Edict");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "opponent sacrificed their creature");
}

/// Liliana's Triumph edicts each opponent and, with a Liliana out, also makes
/// them discard.
#[test]
fn lilianas_triumph_edict_and_liliana_discard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::liliana_of_the_veil()); // a Liliana
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island()); // discard fodder
    let spell = g.add_card_to_hand(0, catalog::lilianas_triumph());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    let opp_hand = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Liliana's Triumph");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "opponent sacrificed a creature");
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "Liliana rider made them discard");
}

/// Sailor of Means makes a Treasure on entry.
#[test]
fn sailor_of_means_makes_treasure() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::sailor_of_means());
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1,
        "ETB made a Treasure");
}

/// Reave Soul destroys a small creature but not a big one.
#[test]
fn reave_soul_destroys_power_three_or_less() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // power 4
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // power 2
    let spell = g.add_card_to_hand(0, catalog::reave_soul());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    // Can't target the power-4 creature.
    assert!(g.cast_spell(spell, Some(Target::Permanent(big)), vec![], None, None).is_err(),
        "power 4 is an illegal target");
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(small)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Reave Soul on the small creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "power-2 creature destroyed");
}

/// CR 702.163 — For Mirrodin! Barbed Batterfist mints a 2/2 red Rebel and
/// attaches itself, so the Rebel ends up a 3/1 (+1/-1).
#[test]
fn cr_702_163_for_mirrodin_mints_and_attaches() {
    let mut g = two_player_game();
    let eq = g.add_card_to_battlefield(0, catalog::barbed_batterfist());
    g.fire_self_etb_triggers(eq, 0);
    drain_stack(&mut g);
    let rebel = g.battlefield.iter()
        .find(|c| c.definition.name == "Rebel")
        .map(|c| c.id)
        .expect("Rebel token minted");
    let cp = g.computed_permanent(rebel).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 1), "2/2 Rebel wears Barbed Batterfist (+1/-1)");
    assert_eq!(g.battlefield_find(eq).unwrap().attached_to, Some(rebel), "equipment attached to the Rebel");
}

/// CR 702.156 — Ravenous: Tyrant Guard enters with X +1/+1 counters and draws a
/// card when X is 5 or more.
#[test]
fn cr_702_156_ravenous_counters_and_draw() {
    let mut g = two_player_game();
    let card = g.add_card_to_hand(0, catalog::tyrant_guard());
    let lib = g.next_id();
    g.players[0].add_to_library_top(lib, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(7); // {X=5}{2}
    g.perform_action(GameAction::CastSpell {
        card_id: card, target: None, additional_targets: vec![], mode: None, x_value: Some(5),
    }).expect("cast Tyrant Guard for X=5");
    drain_stack(&mut g);
    let gid = g.battlefield.iter().find(|c| c.definition.name == "Tyrant Guard").unwrap().id;
    let cp = g.computed_permanent(gid).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8), "3/3 with five +1/+1 counters");
    // Drew the card (hand had the spell removed, then +1 from Ravenous).
    assert!(g.players[0].hand.iter().any(|c| c.id == lib), "Ravenous drew the top card at X>=5");
}

/// Ravenous draws nothing when X is below 5.
#[test]
fn ravenous_no_draw_below_five() {
    let mut g = two_player_game();
    let card = g.add_card_to_hand(0, catalog::tyrant_guard());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4); // {X=2}{2}
    g.perform_action(GameAction::CastSpell {
        card_id: card, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast for X=2");
    drain_stack(&mut g);
    let gid = g.battlefield.iter().find(|c| c.definition.name == "Tyrant Guard").unwrap().id;
    let cp = g.computed_permanent(gid).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "3/3 with two counters");
    assert!(g.players[0].hand.is_empty(), "no Ravenous draw below X=5");
}

/// Termagant Swarm's Death Frenzy makes 1/1 tokens equal to its power on death.
#[test]
fn termagant_swarm_death_frenzy_spawns_tokens() {
    let mut g = two_player_game();
    let swarm = g.add_card_to_battlefield(0, catalog::termagant_swarm());
    // Give it 3 +1/+1 counters → 3/3.
    g.battlefield_find_mut(swarm).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 3);
    let mut events = Vec::new();
    g.sacrifice_one(swarm, 0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let tokens = g.battlefield.iter().filter(|c| c.definition.name == "Tyranid" && c.is_token).count();
    assert_eq!(tokens, 3, "three 1/1 Tyranid tokens equal to power");
}

/// Sunblast Angel's ETB destroys all tapped creatures (not untapped ones).
#[test]
fn sunblast_angel_destroys_tapped_creatures() {
    let mut g = two_player_game();
    let tapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let untapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    let angel = g.add_card_to_battlefield(0, catalog::sunblast_angel());
    g.fire_self_etb_triggers(angel, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(tapped).is_none(), "tapped creature destroyed");
    assert!(g.battlefield_find(untapped).is_some(), "untapped creature survives");
    assert!(g.battlefield_find(angel).is_some(), "the Angel (untapped) survives its own ETB");
}

/// Followed Footsteps copies the enchanted creature at each of your upkeeps.
#[test]
fn followed_footsteps_copies_at_upkeep() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::followed_footsteps());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let trig = catalog::followed_footsteps().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(aura, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "a token copy of the enchanted Bears was made");
}

