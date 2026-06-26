//! Edge of Eternities — Warp (cast cheap, exile at next end step, recast from
//! exile), Void (a nonland permanent left the battlefield or a spell was warped
//! this turn), Lander tokens, and assorted card behaviors.

use crate::card::{CardType, CounterType, Keyword};
use crate::catalog;
use crate::game::types::TurnStep;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Sacrifice a battlefield permanent, firing LTB / dies triggers (CR 701.16).
fn kill(g: &mut GameState, id: CardId) {
    use crate::game::types::Target;
    let ctl = g.battlefield_find(id).unwrap().controller;
    let ctx = crate::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
    g.resolve_effect(
        &crate::effect::Effect::SacrificePermanent { what: crate::effect::Selector::Target(0) },
        &ctx,
    )
    .unwrap();
    drain_stack(g);
}

/// Warp: cast Bygone Colossus for its {3} warp cost. It enters as a 9/9, and at
/// the next end step it's exiled with a `WhileExiled` may-play so it can be
/// recast from exile.
#[test]
fn warp_casts_cheap_then_exiles_at_end_step_and_grants_recast() {
    let mut g = two_player_game();
    let colossus = g.add_card_to_hand(0, catalog::bygone_colossus());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3); // the warp cost, not the {9} face
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: colossus, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("warp-cast Bygone Colossus for {3}");
    drain_stack(&mut g);
    let c = g.battlefield_find(colossus).expect("Colossus entered");
    assert_eq!((c.definition.power, c.definition.toughness), (9, 9));
    assert!(g.players[0].warped_spell_this_turn, "warping a spell satisfies Void");

    // At the next end step the warp delayed trigger exiles it.
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(colossus).is_none(), "warped permanent left the battlefield");
    let exiled = g.exile.iter().find(|c| c.id == colossus).expect("exiled by warp");
    assert!(exiled.may_play_until.is_some(), "recastable from exile");
}

/// Void inactive: Decode Transmissions draws two and you lose 2 life.
#[test]
fn void_inactive_decode_transmissions_self_loses_life() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let decode = g.add_card_to_hand(0, catalog::decode_transmissions());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: decode, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Decode Transmissions");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 - 2, "you lose 2 life with Void off");
    assert_eq!(g.players[1].life, life1, "opponent untouched");
}

/// Void active (a creature died this turn): Decode Transmissions instead makes
/// each opponent lose 2 life.
#[test]
fn void_active_decode_transmissions_opponent_loses_life() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    // Make a nonland permanent leave the battlefield this turn.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, bear);
    drain_stack(&mut g);
    assert!(g.nonland_permanent_left_bf_this_turn, "Void condition latched");

    let decode = g.add_card_to_hand(0, catalog::decode_transmissions());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: decode, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Decode Transmissions");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0, "you keep your life with Void on");
    assert_eq!(g.players[1].life, life1 - 2, "opponent loses 2 with Void on");
}

/// Lander: Biomechan Engineer's ETB mints a Lander; sacrificing it for {2} fetches
/// a basic land onto the battlefield tapped.
#[test]
fn lander_token_fetches_a_basic_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let plains = g.add_card_to_library(0, catalog::plains());
    let eng = g.add_card_to_hand(0, catalog::biomechan_engineer());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eng, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Biomechan Engineer");
    drain_stack(&mut g);
    let lander = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Lander")
        .expect("ETB created a Lander").id;
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: lander, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("crack the Lander");
    drain_stack(&mut g);
    let p = g.battlefield_find(plains).expect("Plains fetched to battlefield");
    assert!(p.tapped, "fetched land enters tapped");
    assert!(g.battlefield_find(lander).is_none(), "Lander was sacrificed");
}

/// Drix Fatemaker's static gives trample to your creatures with a +1/+1 counter.
#[test]
fn drix_fatemaker_grants_trample_to_countered_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::drix_fatemaker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // No counter yet → no trample.
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample),
        "a +1/+1 counter turns on the trample static"
    );
}

/// Broodguard Elite leaves with X counters and dumps them on a target creature.
#[test]
fn broodguard_elite_moves_counters_on_leave() {
    let mut g = two_player_game();
    let brood = g.add_card_to_battlefield(0, catalog::broodguard_elite());
    g.battlefield_find_mut(brood).unwrap().counters.insert(CounterType::PlusOnePlusOne, 3);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, brood);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3,
        "the leaving Broodguard moved its counters to the bear"
    );
}

/// Cosmic Epiphany draws one card per instant/sorcery in your graveyard.
#[test]
fn cosmic_epiphany_draws_per_instant_sorcery_in_graveyard() {
    let mut g = two_player_game();
    let id1 = g.next_id(); g.players[0].graveyard.push(crate::card::CardInstance::new(id1, catalog::lightning_bolt(), 0));
    let id2 = g.next_id(); g.players[0].graveyard.push(crate::card::CardInstance::new(id2, catalog::day_of_judgment(), 0));
    let id3 = g.next_id(); g.players[0].graveyard.push(crate::card::CardInstance::new(id3, catalog::grizzly_bears(), 0));
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let epiphany = g.add_card_to_hand(0, catalog::cosmic_epiphany());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: epiphany, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cosmic Epiphany");
    drain_stack(&mut g);
    // -1 (the spell left hand) + 2 drawn.
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2, "drew 2 (one per I/S in gy)");
}

/// Beyond the Quiet exiles every creature.
#[test]
fn beyond_the_quiet_exiles_all_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let quiet = g.add_card_to_hand(0, catalog::beyond_the_quiet());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: quiet, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Beyond the Quiet");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b));
}

/// Perimeter Patrol grows whenever an artifact you control enters.
#[test]
fn perimeter_patrol_pumps_on_artifact_etb() {
    let mut g = two_player_game();
    let patrol = g.add_card_to_battlefield(0, catalog::perimeter_patrol());
    assert_eq!(g.computed_permanent(patrol).unwrap().power, 3);
    let art = g.add_card_to_hand(0, catalog::memory_guardian());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: art, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Memory Guardian");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(patrol).unwrap().power, 4, "+1/+0 from an artifact entering");
}

/// Exalted Sunborn doubles tokens you create.
#[test]
fn exalted_sunborn_doubles_tokens() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::exalted_sunborn());
    // Knight Luminary's ETB normally makes one Human Soldier → doubled to two.
    let lum = g.add_card_to_hand(0, catalog::knight_luminary());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: lum, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Knight Luminary");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Human Soldier").count();
    assert_eq!(soldiers, 2, "token doubler made two Human Soldiers");
}

/// Memorial Team Leader pumps your other creatures only on your turn.
#[test]
fn memorial_team_leader_anthem_only_on_your_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::memorial_team_leader());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+0 on your turn");
    g.active_player_idx = 1;
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "no bonus on the opponent's turn");
}

/// Kavaron Skywarden grows at your end step while Void is active.
#[test]
fn kavaron_skywarden_grows_on_void_end_step() {
    let mut g = two_player_game();
    let skywarden = g.add_card_to_battlefield(0, catalog::kavaron_skywarden());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, fodder); // Void on
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(skywarden).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Void end step added a +1/+1 counter"
    );
}

/// Emergency Eject destroys a nonland permanent and gives its controller a Lander.
#[test]
fn emergency_eject_gives_controller_a_lander() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let eject = g.add_card_to_hand(0, catalog::emergency_eject());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: eject, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Emergency Eject");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim destroyed");
    let landers = g.battlefield.iter()
        .filter(|c| c.definition.name == "Lander" && c.controller == 1).count();
    assert_eq!(landers, 1, "the victim's controller got the Lander");
}

/// Edge Rover gives every player a Lander when it dies.
#[test]
fn edge_rover_each_player_gets_a_lander() {
    let mut g = two_player_game();
    let rover = g.add_card_to_battlefield(0, catalog::edge_rover());
    kill(&mut g, rover);
    drain_stack(&mut g);
    for p in 0..2 {
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Lander" && c.controller == p).count(),
            1, "player {p} got a Lander"
        );
    }
}

/// Cloudsculpt Technician gets +1/+0 only while you control an artifact.
#[test]
fn cloudsculpt_technician_pumps_with_an_artifact() {
    let mut g = two_player_game();
    let tech = g.add_card_to_battlefield(0, catalog::cloudsculpt_technician());
    assert_eq!(g.computed_permanent(tech).unwrap().power, 2, "no artifact → base power");
    g.add_card_to_battlefield(0, catalog::memory_guardian()); // an artifact
    assert_eq!(g.computed_permanent(tech).unwrap().power, 3, "controlling an artifact → +1/+0");
}

/// Brightspear Zealot grows after you've cast two spells this turn.
#[test]
fn brightspear_zealot_grows_after_two_spells() {
    let mut g = two_player_game();
    let zealot = g.add_card_to_battlefield(0, catalog::brightspear_zealot());
    assert_eq!(g.computed_permanent(zealot).unwrap().power, 2);
    g.players[0].spells_cast_this_turn = 2;
    assert_eq!(g.computed_permanent(zealot).unwrap().power, 4, "+2/+0 with two spells cast");
}

/// Dual-Sun Technique draws only when the buffed creature has a +1/+1 counter.
#[test]
fn dual_sun_technique_draws_with_a_counter() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    let tech = g.add_card_to_hand(0, catalog::dual_sun_technique());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: tech, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dual-Sun Technique");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike));
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "drew because the target had a counter");
}

/// Hymn of the Faller draws an extra card when Void is active.
#[test]
fn hymn_of_the_faller_void_draws_extra() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, fodder); // Void on
    let hymn = g.add_card_to_hand(0, catalog::hymn_of_the_faller());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Default AutoDecider keeps the surveilled card on top.
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: hymn, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hymn of the Faller");
    drain_stack(&mut g);
    // -1 (spell) + 2 drawn (base + Void).
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "Void granted the extra draw");
}

/// CR 702.184 + 721 — Station: tapping a creature adds charge counters equal to
/// its power, and reaching a `{N+}` band turns the Spacecraft into a creature
/// with that band's P/T and keywords.
#[test]
fn station_charges_from_tapped_power_then_band_makes_creature() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::wurmwall_sweeper());
    let beast = g.add_card_to_battlefield(0, catalog::hazard_of_the_dunes()); // 4/4
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Before stationing it's a noncreature artifact.
    assert!(!g.computed_permanent(ship).unwrap().card_types.contains(&CardType::Creature));
    g.perform_action(GameAction::ActivateAbility {
        card_id: ship, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("station the 4/4");
    drain_stack(&mut g);
    let s = g.battlefield_find(ship).unwrap();
    assert_eq!(s.counter_count(CounterType::Charge), 4, "charges = tapped creature's power");
    assert!(g.battlefield_find(beast).unwrap().tapped, "the stationed creature is tapped");
    let post = g.computed_permanent(ship).unwrap();
    assert!(post.card_types.contains(&CardType::Creature), "{{4+}} makes it a creature");
    assert_eq!((post.power, post.toughness), (2, 2));
    assert!(post.keywords.contains(&Keyword::Flying));
}

/// Charge counters accumulate across multiple stationings; a higher `{N+}`
/// band (Atmospheric Greenhouse, {8+}) only applies once the total is reached.
#[test]
fn station_accumulates_to_higher_band() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::atmospheric_greenhouse());
    let a = g.add_card_to_battlefield(0, catalog::hazard_of_the_dunes()); // 4/4
    let b = g.add_card_to_battlefield(0, catalog::hazard_of_the_dunes()); // 4/4
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let act = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: ship, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("station");
        drain_stack(g);
    };
    act(&mut g); // 4 charges — below {8+}
    assert!(!g.computed_permanent(ship).unwrap().card_types.contains(&CardType::Creature));
    let _ = (a, b);
    act(&mut g); // 8 charges — {8+} applies
    let post = g.computed_permanent(ship).unwrap();
    assert_eq!(g.battlefield_find(ship).unwrap().counter_count(CounterType::Charge), 8);
    assert!(post.card_types.contains(&CardType::Creature));
    assert_eq!((post.power, post.toughness), (5, 4));
    assert!(post.keywords.contains(&Keyword::Flying) && post.keywords.contains(&Keyword::Trample));
}

/// Station is sorcery-speed only (CR 702.184a) — it can't be activated with the
/// stack non-empty / at instant speed.
#[test]
fn station_is_sorcery_speed_only() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::wurmwall_sweeper());
    g.add_card_to_battlefield(0, catalog::hazard_of_the_dunes());
    g.step = TurnStep::Upkeep; // not a main phase
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: ship, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).is_err(), "station rejected outside a main phase");
}

/// Galvanizing Sawship's {3+} band grants flying + haste and a 6/5 body once
/// stationed.
#[test]
fn galvanizing_sawship_band_grants_flying_haste() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::galvanizing_sawship());
    g.add_card_to_battlefield(0, catalog::hazard_of_the_dunes()); // 4/4
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ship, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("station");
    drain_stack(&mut g);
    let post = g.computed_permanent(ship).unwrap();
    assert!(post.card_types.contains(&CardType::Creature));
    assert_eq!((post.power, post.toughness), (6, 5));
    assert!(post.keywords.contains(&Keyword::Flying) && post.keywords.contains(&Keyword::Haste));
}

/// Susurian Dirgecraft's ETB makes each opponent sacrifice a nontoken creature.
#[test]
fn susurian_dirgecraft_etb_each_opponent_sacrifices() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ship = g.add_card_to_hand(0, catalog::susurian_dirgecraft());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: ship, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Susurian Dirgecraft");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "opponent sacrificed their creature");
}

/// Pinnacle Kill-Ship's ETB deals 10 to a creature.
#[test]
fn pinnacle_kill_ship_etb_deals_ten() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ship = g.add_card_to_hand(0, catalog::pinnacle_kill_ship());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpell {
        card_id: ship, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Pinnacle Kill-Ship");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "the 2/2 took 10 and died");
}

/// Lumen-Class Frigate's {2+} band is a charge-gated anthem (other creatures
/// you control get +1/+1); the {12+} band makes it a 3/5 flier.
#[test]
fn lumen_frigate_charge_gated_anthem_band() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::lumen_class_frigate());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // No charges yet → no anthem.
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2);
    // Two stationings off the 2/2 → 2 charges → {2+} anthem turns on... but the
    // bear is now tapped; drop 2 charges directly to isolate the band layer.
    g.battlefield_find_mut(ship).unwrap().counters.insert(CounterType::Charge, 2);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "{{2+}} anthem grants +1/+1");
    assert!(!g.computed_permanent(ship).unwrap().card_types.contains(&CardType::Creature),
        "still a noncreature below the {{12+}} band");
    // Reach the {12+} band.
    g.battlefield_find_mut(ship).unwrap().counters.insert(CounterType::Charge, 12);
    let post = g.computed_permanent(ship).unwrap();
    assert!(post.card_types.contains(&CardType::Creature));
    assert_eq!((post.power, post.toughness), (3, 5));
    assert!(post.keywords.contains(&Keyword::Lifelink));
}

/// The server view surfaces a Station card's next `{N+}` charge threshold so
/// the client can show progress (CR 721).
#[test]
fn view_surfaces_station_next_threshold() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::wurmwall_sweeper()); // {4+}
    let v = crate::server::view::project(&g, 0);
    let pv = v.battlefield.iter().find(|p| p.id == ship).expect("ship in view");
    assert_eq!(pv.station_next_threshold, Some(4), "next threshold before any charges");
    // Add 4 charges → threshold reached → no next threshold.
    g.battlefield_find_mut(ship).unwrap().counters.insert(CounterType::Charge, 4);
    let v2 = crate::server::view::project(&g, 0);
    let pv2 = v2.battlefield.iter().find(|p| p.id == ship).unwrap();
    assert_eq!(pv2.station_next_threshold, None, "all bands active");
}

/// The server view surfaces the renowned flag (CR 702.93) so the client can
/// badge a creature whose Renown trigger has fired.
#[test]
fn view_surfaces_renowned_flag() {
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(0, catalog::knight_of_the_pilgrims_road());
    let pv = crate::server::view::project(&g, 0)
        .battlefield.iter().find(|p| p.id == knight).cloned().expect("knight in view");
    assert!(!pv.renowned, "not renowned yet");
    g.battlefield_find_mut(knight).unwrap().renowned = true;
    let pv2 = crate::server::view::project(&g, 0)
        .battlefield.iter().find(|p| p.id == knight).cloned().unwrap();
    assert!(pv2.renowned, "view reflects the renowned flag");
}

/// Harmonious Grovestrider's power/toughness equal the lands you control.
#[test]
fn harmonious_grovestrider_pt_tracks_lands() {
    let mut g = two_player_game();
    let strider = g.add_card_to_battlefield(0, catalog::harmonious_grovestrider());
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); }
    let cp = g.computed_permanent(strider).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "*/* = three lands");
}

/// Lightless Evangel grows when you sacrifice another creature or artifact.
#[test]
fn lightless_evangel_grows_on_sacrifice() {
    use crate::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let evangel = g.add_card_to_battlefield(0, catalog::lightless_evangel());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crate::game::effects::EffectContext::for_ability(evangel, 0, None);
    let evs = g
        .resolve_effect(
            &Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: crate::card::SelectionRequirement::Creature
                    .and(crate::card::SelectionRequirement::OtherThanSource),
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(evangel).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "sacrificing another creature added a +1/+1 counter"
    );
}

/// Frontline War-Rager grows at end step while you control two+ tapped creatures.
#[test]
fn frontline_war_rager_end_step_counter() {
    let mut g = two_player_game();
    let rager = g.add_card_to_battlefield(0, catalog::frontline_war_rager());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(rager).unwrap().tapped = true;
    g.battlefield_find_mut(other).unwrap().tapped = true;
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(rager).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "two tapped creatures → end-step counter"
    );
}

/// Gravkill exiles a creature (no graveyard, no regen).
#[test]
fn gravkill_exiles_target() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::gravkill());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Gravkill");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert!(g.exile.iter().any(|c| c.id == victim), "exiled, not in graveyard");
}

/// Invasive Maneuvers deals 5 instead of 3 while you control a Spacecraft.
#[test]
fn invasive_maneuvers_scales_with_spacecraft() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::galvanizing_sawship()); // a Spacecraft
    let victim = g.add_card_to_battlefield(1, catalog::hazard_of_the_dunes()); // 4/4
    let spell = g.add_card_to_hand(0, catalog::invasive_maneuvers());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Invasive Maneuvers");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "5 damage killed the 4/4");
}

/// Gravpack Monoist leaves a tapped 2/2 Robot when it dies.
#[test]
fn gravpack_monoist_dies_into_robot() {
    let mut g = two_player_game();
    let mono = g.add_card_to_battlefield(0, catalog::gravpack_monoist());
    kill(&mut g, mono);
    drain_stack(&mut g);
    let robot = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Robot")
        .expect("died into a Robot");
    assert!(robot.tapped, "the Robot enters tapped");
}

/// Illvoi Operative grows when you cast your second spell of the turn.
#[test]
fn illvoi_operative_second_spell_counter() {
    let mut g = two_player_game();
    let op = g.add_card_to_battlefield(0, catalog::illvoi_operative());
    let s1 = g.add_card_to_hand(0, catalog::bombard());
    let s2 = g.add_card_to_hand(0, catalog::bombard());
    let d1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let d2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    // First spell — no counter.
    g.perform_action(GameAction::CastSpell {
        card_id: s1, target: Some(Target::Permanent(d1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("first spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(op).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    // Second spell — counter.
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: s2, target: Some(Target::Permanent(d2)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("second spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(op).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "second spell each turn adds a +1/+1 counter");
}

/// Kavaron Turbodrone pumps and hastes a creature you control.
#[test]
fn kavaron_turbodrone_pumps_and_hastes() {
    let mut g = two_player_game();
    let drone = g.add_card_to_battlefield(0, catalog::kavaron_turbodrone());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: drone, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Turbodrone");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    assert!(cp.keywords.contains(&Keyword::Haste));
}

/// Plasma Bolt deals 2, or 3 with Void active.
#[test]
fn plasma_bolt_void_scales_damage() {
    // Void off: a 3-toughness creature survives 2 damage.
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::prowcatcher_specialist()); // 2/1
    let bolt = g.add_card_to_hand(0, catalog::plasma_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    // Void on: a nonland permanent left the battlefield this turn.
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, fodder);
    drain_stack(&mut g);
    let toughie = g.add_card_to_battlefield(1, catalog::hazard_of_the_dunes()); // 4/4
    let _ = victim;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(toughie)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Plasma Bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(toughie).unwrap().damage, 3, "Void active → 3 damage");
}

/// Seedship Agrarian mints a Lander when it taps and grows on landfall.
#[test]
fn seedship_agrarian_taps_for_lander_and_grows() {
    let mut g = two_player_game();
    let agr = g.add_card_to_battlefield(0, catalog::seedship_agrarian());
    g.battlefield_find_mut(agr).unwrap().tapped = false;
    // Tapping mints a Lander.
    let evs = vec![GameEvent::PermanentTapped { card_id: agr }];
    g.battlefield_find_mut(agr).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Lander"),
        "becoming tapped created a Lander");
}

/// Entropic Battlecruiser's {1+} band grants a counter-gated trigger: while it
/// has ≥1 charge, an opponent's discard costs them 3 life. Below the band the
/// trigger is inert.
#[test]
fn entropic_battlecruiser_charge_gated_discard_punisher() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::entropic_battlecruiser());
    let card = g.add_card_to_hand(1, catalog::forest());
    // No charges → the {1+} trigger isn't granted.
    g.dispatch_triggers_for_events(&[GameEvent::CardDiscarded { player: 1, card_id: card }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "no band, no life loss");
    // One charge turns on the band's trigger.
    g.battlefield_find_mut(ship).unwrap().counters.insert(CounterType::Charge, 1);
    g.dispatch_triggers_for_events(&[GameEvent::CardDiscarded { player: 1, card_id: card }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "{{1+}} band: opponent loses 3 on discard");
}

/// Synthesizer Labship's {2+} band fires a begin-combat trigger that animates
/// another artifact you control into a 2/2 flier; below the band nothing fires.
#[test]
fn synthesizer_labship_charge_gated_animate_band() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::synthesizer_labship());
    let relic = g.add_card_to_battlefield(0, catalog::wurmwall_sweeper()); // an artifact
    g.battlefield_find_mut(ship).unwrap().counters.insert(CounterType::Charge, 2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    let cp = g.computed_permanent(relic).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature), "artifact animated");
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Nutrient Block draws a card when it's sacrificed (left for the graveyard).
#[test]
fn nutrient_block_draws_on_leave() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let block = g.add_card_to_battlefield(0, catalog::nutrient_block());
    let hand0 = g.players[0].hand.len();
    kill(&mut g, block);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "leaving the battlefield drew a card");
}

/// Frenzied Baloth's static makes all combat damage unpreventable: a prevention
/// shield on the defending player is bypassed.
#[test]
fn frenzied_baloth_combat_damage_unpreventable() {
    use crate::game::types::{Attack, AttackTarget, PreventionShield, PreventionTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::frenzied_baloth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.prevention_shields.push(PreventionShield {
        target: PreventionTarget::Player(1),
        remaining: None,
        gain_life: false,
        source: None,
        one_event: false,
        reflect: false,
        source_controller: None,
    });
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    assert_eq!(g.players[1].life, 18, "prevention shield bypassed");
}

/// Frenzied Baloth's static makes the controller's creature spells uncounterable.
#[test]
fn frenzied_baloth_creature_spells_uncounterable() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::frenzied_baloth());
    let bear_id = g.add_card_to_hand(0, catalog::grizzly_bears());
    let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear = g.players[0].hand.iter().find(|c| c.id == bear_id).unwrap().clone();
    let bolt = g.players[0].hand.iter().find(|c| c.id == bolt_id).unwrap().clone();
    assert!(g.caster_grants_uncounterable(0, &bear), "your creature spell can't be countered");
    assert!(!g.caster_grants_uncounterable(0, &bolt), "noncreature spells still counterable");
}

/// Gravblade Heavy gains +1/+0 and deathtouch while you control an artifact.
#[test]
fn gravblade_heavy_artifact_conditional_buff() {
    let mut g = two_player_game();
    let gh = g.add_card_to_battlefield(0, catalog::gravblade_heavy());
    let cp = g.computed_permanent(gh).unwrap();
    assert_eq!(cp.power, 3, "no artifact → base 3/4");
    assert!(!cp.keywords.contains(&Keyword::Deathtouch));
    g.add_card_to_battlefield(0, catalog::wurmwall_sweeper()); // an artifact
    let cp2 = g.computed_permanent(gh).unwrap();
    assert_eq!(cp2.power, 4, "artifact → +1/+0");
    assert!(cp2.keywords.contains(&Keyword::Deathtouch));
}

/// Skystinger gets +5/+0 when it blocks a creature with flying.
#[test]
fn skystinger_pumps_blocking_flier() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    g.clear_sickness(flier);
    let sky = g.add_card_to_battlefield(0, catalog::skystinger());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: flier,
        target: AttackTarget::Player(0),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(sky, flier)])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(sky).unwrap().power, 8, "blocked a flier → +5/+0");
}

/// Resolve `effect` for `player` with the given target permanents bound to
/// slots 0,1,… (a lightweight stand-in for casting a targeted spell).
fn resolve_targeted(g: &mut GameState, player: usize, effect: crate::effect::Effect, targets: &[CardId]) {
    use crate::game::types::Target;
    let src = g.add_card_to_battlefield(player, catalog::grizzly_bears());
    let mut ctx = crate::game::effects::EffectContext::for_ability(src, player, None);
    ctx.targets = targets.iter().map(|id| Target::Permanent(*id)).collect();
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(g);
}

/// Honor puts a +1/+1 counter on a creature and draws.
#[test]
fn honor_counters_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hand0 = g.players[0].hand.len();
    resolve_targeted(&mut g, 0, catalog::honor().effect, &[bear]);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
}

/// Radiant Strike destroys a tapped creature (untapped non-artifact is an
/// illegal target) and gains 3 life.
#[test]
fn radiant_strike_destroys_tapped_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let life0 = g.players[0].life;
    resolve_targeted(&mut g, 0, catalog::radiant_strike().effect, &[bear]);
    assert!(g.battlefield_find(bear).is_none(), "tapped creature destroyed");
    assert_eq!(g.players[0].life, life0 + 3);
}

/// Luxknight Breacher enters with a +1/+1 counter per other creature/artifact.
#[test]
fn luxknight_breacher_scales_on_entry() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::wurmwall_sweeper()); // artifact
    let lux = g.move_card_to_battlefield_for_test(0, catalog::luxknight_breacher());
    assert_eq!(g.battlefield_find(lux).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "one counter per other creature + artifact");
}

/// Diplomatic Relations pumps your creature and makes it deal its power to an
/// opponent's creature.
#[test]
fn diplomatic_relations_pump_and_zap() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/2
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 survives 3
    resolve_targeted(&mut g, 0, catalog::diplomatic_relations().effect, &[mine, theirs]);
    assert!(g.computed_permanent(mine).unwrap().keywords.contains(&Keyword::Vigilance));
    assert_eq!(g.battlefield_find(theirs).unwrap().damage, 3, "took damage = pumped power");
}

/// Cut Propulsion: a flier takes twice its power in self-damage.
#[test]
fn cut_propulsion_doubles_on_flier() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    resolve_targeted(&mut g, 0, catalog::cut_propulsion().effect, &[angel]);
    assert!(g.battlefield_find(angel).is_none(), "8 damage to a 4/4 flier is lethal");
}

/// Mechan Navigator loots when it becomes tapped.
#[test]
fn mechan_navigator_loots_on_tap() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let nav = g.add_card_to_battlefield(0, catalog::mechan_navigator());
    g.add_card_to_hand(0, catalog::forest()); // something to discard
    let hand0 = g.players[0].hand.len();
    g.battlefield_find_mut(nav).unwrap().tapped = false;
    let evs = vec![GameEvent::PermanentTapped { card_id: nav }];
    g.battlefield_find_mut(nav).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0, "drew one, discarded one (net zero)");
}

/// Gigastorm Titan costs {3} less once you've cast another spell this turn.
#[test]
fn gigastorm_titan_cost_reduction() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crate::card::CardInstance::new(g.next_id(), catalog::gigastorm_titan(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no spell cast yet → full price");
    g.players[0].spells_cast_this_turn = 1;
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 3, "after a spell → {{3}} off");
}

/// Lashwhip Predator costs {2} less while opponents control 3+ creatures.
#[test]
fn lashwhip_predator_cost_reduction() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crate::card::CardInstance::new(g.next_id(), catalog::lashwhip_predator(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "<3 opp creatures → full price");
    for _ in 0..3 { g.add_card_to_battlefield(1, catalog::grizzly_bears()); }
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 2, "3 opp creatures → {{2}} off");
}

/// Sami's Curiosity gains 2 life and mints a Lander.
#[test]
fn samis_curiosity_gains_and_makes_lander() {
    let mut g = two_player_game();
    let life0 = g.players[0].life;
    resolve_targeted(&mut g, 0, catalog::samis_curiosity().effect, &[]);
    assert_eq!(g.players[0].life, life0 + 2);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Lander"));
}

/// Lithobraking makes a Lander, then sacrificing an artifact deals 2 to each
/// creature.
#[test]
fn lithobraking_sac_pings_all_creatures() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wurmwall_sweeper()); // an artifact to sacrifice
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 survives the ping
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    g.resolve_effect(&catalog::lithobraking().effect, &ctx).unwrap();
    drain_stack(&mut g);
    // An artifact was sacrificed (one of the Wurmwall / the new Lander), and the
    // reflexive trigger dealt 2 to each creature.
    assert_eq!(g.battlefield_find(victim).map(|c| c.damage), Some(2), "each creature took 2");
}

/// Rust Harvester grows and pings equal to its (post-counter) power.
#[test]
fn rust_harvester_grows_and_pings() {
    let mut g = two_player_game();
    let rh = g.add_card_to_battlefield(0, catalog::rust_harvester());
    g.clear_sickness(rh);
    // Seed an artifact card in the graveyard to exile as the cost.
    let gy_id = g.next_id();
    g.players[0].graveyard.push(crate::card::CardInstance::new(
        gy_id, catalog::wurmwall_sweeper(), 0));
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rh, ability_index: 0,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Rust Harvester");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rh).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[1].life, 18, "2/2-power ping (1 base + 1 counter)");
}

/// Nanoform Sentinel untaps another permanent when it becomes tapped.
#[test]
fn nanoform_sentinel_untaps_on_tap() {
    let mut g = two_player_game();
    let nano = g.add_card_to_battlefield(0, catalog::nanoform_sentinel());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.battlefield_find_mut(nano).unwrap().tapped = false;
    let evs = vec![GameEvent::PermanentTapped { card_id: nano }];
    g.battlefield_find_mut(nano).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&evs);
    // Bind the untap target to the tapped land.
    drain_stack(&mut g);
    assert!(!g.battlefield_find(land).unwrap().tapped, "the land was untapped");
}

/// Resolve `effect` for `player`, picking modal index `mode` with `targets`.
fn resolve_modal(g: &mut GameState, player: usize, effect: crate::effect::Effect, mode: usize, targets: &[CardId]) {
    use crate::game::types::Target;
    let src = g.add_card_to_battlefield(player, catalog::grizzly_bears());
    let mut ctx = crate::game::effects::EffectContext::for_ability(src, player, None);
    ctx.mode = mode;
    ctx.targets = targets.iter().map(|id| Target::Permanent(*id)).collect();
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(g);
}

/// Mechan Assembler mints a Robot when another artifact enters under your control.
#[test]
fn mechan_assembler_makes_robot_on_artifact_entry() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mechan_assembler());
    let before = g.battlefield.iter().filter(|c| c.definition.name == "Robot").count();
    let art = g.add_card_to_battlefield(0, catalog::wurmwall_sweeper()); // an artifact
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: art }]);
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.definition.name == "Robot").count();
    assert_eq!(after, before + 1, "another artifact entering mints a Robot");
}

/// Mm'menon puts a +1/+1 counter on a creature when an artifact enters.
#[test]
fn mmmenon_counters_on_artifact_entry() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mmmenon_uthros_exile());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let art = g.add_card_to_battlefield(0, catalog::wurmwall_sweeper());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: art }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Embrace Oblivion destroys a Spacecraft.
#[test]
fn embrace_oblivion_destroys_spacecraft() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(1, catalog::wurmwall_sweeper()); // a Spacecraft
    resolve_targeted(&mut g, 0, catalog::embrace_oblivion().effect, &[ship]);
    assert!(g.battlefield_find(ship).is_none(), "Spacecraft destroyed");
}

/// Scrounge for Eternity reanimates a creature from your graveyard and makes a
/// Lander.
#[test]
fn scrounge_reanimates_and_makes_lander() {
    let mut g = two_player_game();
    let id = g.next_id();
    g.players[0].graveyard.push(crate::card::CardInstance::new(id, catalog::grizzly_bears(), 0));
    resolve_targeted(&mut g, 0, catalog::scrounge_for_eternity().effect, &[id]);
    assert!(g.battlefield.iter().any(|c| c.id == id && c.controller == 0), "creature reanimated");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Lander"));
}

/// Ruinous Rampage's first mode deals 3 to each opponent.
#[test]
fn ruinous_rampage_burns_each_opponent() {
    let mut g = two_player_game();
    resolve_modal(&mut g, 0, catalog::ruinous_rampage().effect, 0, &[]);
    assert_eq!(g.players[1].life, 17, "each opponent took 3");
}

/// Drill Too Deep's second mode destroys a target artifact.
#[test]
fn drill_too_deep_destroys_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::wurmwall_sweeper());
    resolve_modal(&mut g, 0, catalog::drill_too_deep().effect, 1, &[art]);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
}

/// Reroute Systems' second mode deals 2 to a tapped creature.
#[test]
fn reroute_systems_zaps_tapped_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    resolve_modal(&mut g, 0, catalog::reroute_systems().effect, 1, &[bear]);
    assert!(g.battlefield_find(bear).is_none(), "2 damage kills the 2/2");
}

/// Mouth of the Storm shrinks opponents' creatures by 3 power on entry.
#[test]
fn mouth_of_the_storm_shrinks_opponents() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let mouth = g.move_card_to_battlefield_for_test(0, catalog::mouth_of_the_storm());
    drain_stack(&mut g);
    assert!(g.computed_permanent(mouth).unwrap().keywords.contains(&Keyword::Flying));
    assert_eq!(g.computed_permanent(foe).unwrap().power, 1, "-3/-0 on opponents");
}

/// Meltstrider Eulogist draws when a counter-bearing creature you control dies.
#[test]
fn meltstrider_eulogist_draws_on_countered_death() {
    // AnotherOfYours death triggers need the unified cast→SBA→dispatch path
    // (mirrors the Felisa test), so kill the bear with a real Murder.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::meltstrider_eulogist());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let hand0 = g.players[0].hand.len();
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: murder,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Murder on the bear");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "counter-bearing death drew a card");
}

/// Cryogen Relic draws when it leaves the battlefield.
#[test]
fn cryogen_relic_draws_on_leave() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let relic = g.add_card_to_battlefield(0, catalog::cryogen_relic());
    let hand0 = g.players[0].hand.len();
    kill(&mut g, relic);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "leaving drew a card");
}

/// Chrome Companion gains 1 life when it becomes tapped.
#[test]
fn chrome_companion_gains_on_tap() {
    let mut g = two_player_game();
    let dog = g.add_card_to_battlefield(0, catalog::chrome_companion());
    let life0 = g.players[0].life;
    g.battlefield_find_mut(dog).unwrap().tapped = false;
    let evs = vec![GameEvent::PermanentTapped { card_id: dog }];
    g.battlefield_find_mut(dog).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1);
}

/// The Station tap-cost auto-picker (non-UI seat) taps the *highest*-power
/// creature, since charges scale with its power (CR 702.184a).
#[test]
fn station_autopick_taps_highest_power_creature() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::wurmwall_sweeper());
    let weak = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let strong = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    g.clear_sickness(weak);
    g.clear_sickness(strong);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ship, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("station");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ship).unwrap().counter_count(CounterType::Charge), 4,
        "tapped the 4/4, not the 2/2");
    assert!(g.battlefield_find(strong).unwrap().tapped, "the strong creature was tapped");
    assert!(!g.battlefield_find(weak).unwrap().tapped, "the weak creature stayed up");
}

/// Hemosymbic Mite pumps another creature by its own power when it taps.
#[test]
fn hemosymbic_mite_pumps_by_power_on_tap() {
    let mut g = two_player_game();
    let mite = g.add_card_to_battlefield(0, catalog::hemosymbic_mite());
    g.battlefield_find_mut(mite).unwrap().add_counters(CounterType::PlusOnePlusOne, 2); // now 3/3
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mite).unwrap().tapped = false;
    let evs = vec![GameEvent::PermanentTapped { card_id: mite }];
    g.battlefield_find_mut(mite).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "+3/+3 from a 3-power Mite");
}

/// Genemorph Imago's landfall sets a creature's base P/T to 3/3.
#[test]
fn genemorph_imago_landfall_sets_base_pt() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::genemorph_imago());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: land }]);
    drain_stack(&mut g);
    assert_eq!((g.computed_permanent(bear).unwrap().power, g.computed_permanent(bear).unwrap().toughness), (3, 3));
}

/// Full Bore grants trample+haste only to a creature cast for its warp cost.
#[test]
fn full_bore_warp_rider() {
    let mut g = two_player_game();
    let normal = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let warped = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(warped).unwrap().warped = true;
    resolve_targeted(&mut g, 0, catalog::full_bore().effect, &[normal]);
    assert!(!g.computed_permanent(normal).unwrap().keywords.contains(&Keyword::Trample),
        "non-warped: no trample");
    resolve_targeted(&mut g, 0, catalog::full_bore().effect, &[warped]);
    assert!(g.computed_permanent(warped).unwrap().keywords.contains(&Keyword::Haste),
        "warped: gains haste");
}

/// Emissary Escort's power = greatest mana value among OTHER artifacts you
/// control (0 with none, excludes itself).
#[test]
fn emissary_escort_power_tracks_greatest_other_artifact_mv() {
    let mut g = two_player_game();
    let escort = g.add_card_to_battlefield(0, catalog::emissary_escort());
    assert_eq!(g.computed_permanent(escort).unwrap().power, 0, "no other artifacts → +0");
    assert_eq!(g.computed_permanent(escort).unwrap().toughness, 4);
    g.add_card_to_battlefield(0, catalog::mind_stone()); // MV 2 artifact
    assert_eq!(g.computed_permanent(escort).unwrap().power, 2, "greatest other artifact MV = 2");
}

/// Solar Blaze: each creature deals damage to itself equal to its power. A 3/3
/// dies; a 0/4 (power 0) is untouched.
#[test]
fn solar_blaze_each_creature_self_damages() {
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let escort = g.add_card_to_battlefield(1, catalog::emissary_escort()); // 0/4, power 0
    resolve_targeted(&mut g, 0, catalog::solar_blaze().effect, &[]);
    assert!(g.battlefield_find(giant).is_none(), "3/3 took 3, died");
    assert!(g.battlefield_find(escort).is_some(), "0-power creature took 0, survived");
}

/// Fungal Colossus costs {X} less, X = differently named lands you control.
#[test]
fn fungal_colossus_distinct_land_name_discount() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crate::card::CardInstance::new(g.next_id(), catalog::fungal_colossus(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0);
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::mountain());
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 3, "3 distinct land names");
    g.add_card_to_battlefield(0, catalog::forest()); // duplicate name, no extra
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 3, "duplicate name doesn't count");
}

/// Dark Endurance: {1} cheaper only when it targets a blocking creature; the
/// effect grants +2/+0 and indestructible.
#[test]
fn dark_endurance_blocking_discount_and_pump() {
    use crate::game::actions::cost_reduction_for_spell;
    use crate::game::types::Target;
    let mut g = two_player_game();
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(1, catalog::hill_giant());
    let idle = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = crate::card::CardInstance::new(g.next_id(), catalog::dark_endurance(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, Some(&Target::Permanent(idle))), 0);
    g.block_map.insert(blocker, attacker);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, Some(&Target::Permanent(blocker))), 1,
        "{{1}} off vs a blocking creature");
    resolve_targeted(&mut g, 0, catalog::dark_endurance().effect, &[blocker]);
    let cp = g.computed_permanent(blocker).unwrap();
    assert_eq!(cp.power, 4, "2/2 +2/+0");
    assert!(cp.keywords.contains(&Keyword::Indestructible));
}

/// Genemorph Imago's landfall sets a creature to base 3/3, or 5/5 once you
/// control six or more lands. Drives the card's real landfall effect.
#[test]
fn genemorph_imago_landfall_scales_with_lands() {
    let landfall = || catalog::genemorph_imago().triggered_abilities[0].effect.clone();
    // Five lands: landfall sets base 3/3.
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    for _ in 0..5 { g.add_card_to_battlefield(0, catalog::forest()); }
    resolve_targeted(&mut g, 0, landfall(), &[target]);
    assert_eq!(g.computed_permanent(target).unwrap().power, 3, "five lands → base 3/3");
    // Sixth land flips it to 5/5.
    g.add_card_to_battlefield(0, catalog::forest());
    resolve_targeted(&mut g, 0, landfall(), &[target]);
    assert_eq!(g.computed_permanent(target).unwrap().power, 5, "six lands → base 5/5");
}
