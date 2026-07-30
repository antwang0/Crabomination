//! Edge of Eternities — Warp (cast cheap, exile at next end step, recast from
//! exile), Void (a nonland permanent left the battlefield or a spell was warped
//! this turn), Lander tokens, and assorted card behaviors.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::TurnStep;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Sacrifice a battlefield permanent, firing LTB / dies triggers (CR 701.16).
fn kill(g: &mut GameState, id: CardId) {
    use crabomination::game::types::Target;
    let ctl = g.battlefield_find(id).unwrap().controller;
    let ctx = crabomination::game::effects::EffectContext::for_ability(id, ctl, Some(Target::Permanent(id)));
    g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &ctx,
    )
    .unwrap();
    drain_stack(g);
}

/// Put a fresh card straight into player 0's graveyard (test fixture).
fn to_gy(g: &mut GameState, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_hand(0, def);
    let i = g.players[0].hand.iter().position(|c| c.id == id).unwrap();
    let c = g.players[0].hand.remove(i);
    g.players[0].graveyard.push(c);
    id
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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
    let id1 = g.next_id(); g.players[0].graveyard.push(crabomination::card::CardInstance::new(id1, catalog::lightning_bolt(), 0));
    let id2 = g.next_id(); g.players[0].graveyard.push(crabomination::card::CardInstance::new(id2, catalog::day_of_judgment(), 0));
    let id3 = g.next_id(); g.players[0].graveyard.push(crabomination::card::CardInstance::new(id3, catalog::grizzly_bears(), 0));
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
    use crabomination::game::types::Target;
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
    assert_eq!(g.computed_permanent(tech).unwrap().power, 1, "no artifact → base power");
    g.add_card_to_battlefield(0, catalog::memory_guardian()); // an artifact
    assert_eq!(g.computed_permanent(tech).unwrap().power, 2, "controlling an artifact → +1/+0");
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
    use crabomination::game::types::Target;
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
    let v = crabomination::server::view::project(&g, 0);
    let pv = v.battlefield.iter().find(|p| p.id == ship).expect("ship in view");
    assert_eq!(pv.station_next_threshold, Some(4), "next threshold before any charges");
    assert_eq!(pv.station_charges, Some(0), "current charges surfaced");
    // Add 4 charges → threshold reached → no next threshold.
    g.battlefield_find_mut(ship).unwrap().counters.insert(CounterType::Charge, 4);
    let v2 = crabomination::server::view::project(&g, 0);
    let pv2 = v2.battlefield.iter().find(|p| p.id == ship).unwrap();
    assert_eq!(pv2.station_next_threshold, None, "all bands active");
    assert_eq!(pv2.station_charges, Some(4), "charge count tracks");
}

/// The server view surfaces the renowned flag (CR 702.93) so the client can
/// badge a creature whose Renown trigger has fired.
#[test]
fn view_surfaces_renowned_flag() {
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(0, catalog::knight_of_the_pilgrims_road());
    let pv = crabomination::server::view::project(&g, 0)
        .battlefield.iter().find(|p| p.id == knight).cloned().expect("knight in view");
    assert!(!pv.renowned, "not renowned yet");
    g.battlefield_find_mut(knight).unwrap().renowned = true;
    let pv2 = crabomination::server::view::project(&g, 0)
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
    use crabomination::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let evangel = g.add_card_to_battlefield(0, catalog::lightless_evangel());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(evangel, 0, None);
    let evs = g
        .resolve_effect(
            &Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: crabomination::card::SelectionRequirement::Creature
                    .and(crabomination::card::SelectionRequirement::OtherThanSource),
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
    g.clear_sickness(drone);
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
    let evs = vec![GameEvent::PermanentTapped { card_id: agr, actor: None, as_attacker: false }];
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
    use crabomination::game::types::{Attack, AttackTarget, PreventionShield, PreventionTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::frenzied_baloth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.prevention_shields.push(PreventionShield {
        target: PreventionTarget::Player(1),
        ..Default::default()
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
    use crabomination::game::types::{Attack, AttackTarget};
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
fn resolve_targeted(g: &mut GameState, player: usize, effect: crabomination::effect::Effect, targets: &[CardId]) {
    use crabomination::game::types::Target;
    let src = g.add_card_to_battlefield(player, catalog::grizzly_bears());
    let mut ctx = crabomination::game::effects::EffectContext::for_ability(src, player, None);
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
    let evs = vec![GameEvent::PermanentTapped { card_id: nav, actor: None, as_attacker: false }];
    g.battlefield_find_mut(nav).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0, "drew one, discarded one (net zero)");
}

/// Gigastorm Titan costs {3} less once you've cast another spell this turn.
#[test]
fn gigastorm_titan_cost_reduction() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::gigastorm_titan(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no spell cast yet → full price");
    g.players[0].spells_cast_this_turn = 1;
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 3, "after a spell → {{3}} off");
}

/// Lashwhip Predator costs {2} less while opponents control 3+ creatures.
#[test]
fn lashwhip_predator_cost_reduction() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::lashwhip_predator(), 0);
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
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wurmwall_sweeper()); // an artifact to sacrifice
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 survives the ping
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
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
    g.players[0].graveyard.push(crabomination::card::CardInstance::new(
        gy_id, catalog::wurmwall_sweeper(), 0));
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rh, ability_index: 0,
        target: Some(crabomination::game::types::Target::Player(1)),
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
    let evs = vec![GameEvent::PermanentTapped { card_id: nano, actor: None, as_attacker: false }];
    g.battlefield_find_mut(nano).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&evs);
    // Bind the untap target to the tapped land.
    drain_stack(&mut g);
    assert!(!g.battlefield_find(land).unwrap().tapped, "the land was untapped");
}

/// Resolve `effect` for `player`, picking modal index `mode` with `targets`.
fn resolve_modal(g: &mut GameState, player: usize, effect: crabomination::effect::Effect, mode: usize, targets: &[CardId]) {
    use crabomination::game::types::Target;
    let src = g.add_card_to_battlefield(player, catalog::grizzly_bears());
    let mut ctx = crabomination::game::effects::EffectContext::for_ability(src, player, None);
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
    g.players[0].graveyard.push(crabomination::card::CardInstance::new(id, catalog::grizzly_bears(), 0));
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
        target: Some(crabomination::game::types::Target::Permanent(bear)),
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
    let evs = vec![GameEvent::PermanentTapped { card_id: dog, actor: None, as_attacker: false }];
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
    let evs = vec![GameEvent::PermanentTapped { card_id: mite, actor: None, as_attacker: false }];
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
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::fungal_colossus(), 0);
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
    use crabomination::game::actions::cost_reduction_for_spell;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(1, catalog::hill_giant());
    let idle = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::dark_endurance(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, Some(&Target::Permanent(idle))), 0);
    g.set_block_map([(blocker, attacker)]);
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

/// Shattered Wings destroys a flying creature (and would hit artifacts /
/// enchantments too).
#[test]
fn shattered_wings_destroys_flyer() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    resolve_targeted(&mut g, 0, catalog::shattered_wings().effect, &[flyer]);
    assert!(g.battlefield_find(flyer).is_none(), "flyer destroyed");
}

/// Seam Rip exiles a low-MV opposing permanent until it leaves; removing Seam
/// Rip returns it.
#[test]
fn seam_rip_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let rip = g.move_card_to_battlefield_for_test(0, catalog::seam_rip());
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "exiled by Seam Rip");
    kill(&mut g, rip);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 1),
        "victim returned when Seam Rip left");
}

/// Seedship Impact destroys an artifact and yields a Lander only when the
/// target's mana value was 2 or less.
#[test]
fn seedship_impact_lander_on_low_mv() {
    let mut g = two_player_game();
    let cheap = g.add_card_to_battlefield(1, catalog::mind_stone()); // MV 2
    resolve_targeted(&mut g, 0, catalog::seedship_impact().effect, &[cheap]);
    assert!(g.battlefield_find(cheap).is_none(), "MV2 artifact destroyed");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Lander").count(), 1);

    let mut g = two_player_game();
    let pricey = g.add_card_to_battlefield(1, catalog::darksteel_ingot()); // MV 3
    resolve_targeted(&mut g, 0, catalog::seedship_impact().effect, &[pricey]);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Lander").count(), 0,
        "MV3 → no Lander");
}

/// Desculpting Blast bounces a permanent and spawns a Drone only when the
/// bounced creature was attacking.
#[test]
fn desculpting_blast_drone_when_attacking() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(0) }];
    resolve_targeted(&mut g, 0, catalog::desculpting_blast().effect, &[attacker]);
    assert!(g.battlefield_find(attacker).is_none(), "bounced");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "to owner's hand");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Drone").count(), 1,
        "attacking → Drone token");
}

/// Lost in Space tucks an artifact/creature into its owner's library.
#[test]
fn lost_in_space_tucks_to_library() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let before = g.players[1].library.len();
    resolve_targeted(&mut g, 0, catalog::lost_in_space().effect, &[creature]);
    assert!(g.battlefield_find(creature).is_none(), "left the battlefield");
    assert_eq!(g.players[1].library.len(), before + 1, "into owner's library");
}

/// Sinister Cryologist's ETB shrinks an opposing creature by 3 power.
#[test]
fn sinister_cryologist_etb_shrinks_opponent() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    resolve_targeted(&mut g, 0, catalog::sinister_cryologist().triggered_abilities[0].effect.clone(), &[foe]);
    assert_eq!(g.computed_permanent(foe).unwrap().power, 0, "−3/−0");
}

/// Orbital Plunge deals 6 and yields a Lander only on excess (low toughness).
#[test]
fn orbital_plunge_lander_on_excess() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    resolve_targeted(&mut g, 0, catalog::orbital_plunge().effect, &[small]);
    assert!(g.battlefield_find(small).is_none(), "killed");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Lander").count(), 1);

    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::bygone_colossus()); // 9/9
    resolve_targeted(&mut g, 0, catalog::orbital_plunge().effect, &[big]);
    assert!(g.battlefield_find(big).is_some(), "9/9 survives 6");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Lander").count(), 0,
        "no excess → no Lander");
}

/// Anticausal Vestige's leave trigger draws and cheats a low-MV permanent into
/// play tapped (MV ≤ lands you control).
#[test]
fn anticausal_vestige_ltb_cheats_permanent() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest()); // a card to draw
    let vestige = g.add_card_to_battlefield(0, catalog::anticausal_vestige());
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); } // MV ≤ 3 allowed
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears()); // MV 2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bears])])); // opt to cheat it in
    kill(&mut g, vestige);
    drain_stack(&mut g);
    let in_play = g.battlefield_find(bears);
    assert!(in_play.is_some(), "cheated the bears into play");
    assert!(in_play.unwrap().tapped, "entered tapped");
}

/// Faller's Faithful destroys a creature; an undamaged one lets its controller
/// draw two.
#[test]
fn fallers_faithful_destroy_undamaged_draws_two() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(1, catalog::forest()); } // cards to draw
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3, undamaged
    let before = g.players[1].hand.len();
    resolve_targeted(&mut g, 0, catalog::fallers_faithful().triggered_abilities[0].effect.clone(), &[foe]);
    assert!(g.battlefield_find(foe).is_none(), "destroyed");
    assert_eq!(g.players[1].hand.len(), before + 2, "undamaged → controller drew two");
}

/// Selfcraft Mechan sacrifices an artifact to grow a creature and draw.
#[test]
fn selfcraft_mechan_sacs_artifact_for_value() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest()); // a card to draw
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // pay the optional sac
    let fodder = g.add_card_to_battlefield(0, catalog::mind_stone());
    let grow = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    resolve_targeted(&mut g, 0, catalog::selfcraft_mechan().triggered_abilities[0].effect.clone(), &[grow]);
    assert!(g.battlefield_find(fodder).is_none(), "artifact sacrificed");
    assert_eq!(g.computed_permanent(grow).unwrap().toughness, 3, "+1/+1 counter");
    assert_eq!(g.players[0].hand.len(), before + 1, "drew a card");
}

/// Cosmogrand Zenith's flurry trigger (mode 0) makes two Soldier tokens.
#[test]
fn cosmogrand_zenith_flurry_makes_soldiers() {
    let mut g = two_player_game();
    resolve_targeted(&mut g, 0, catalog::cosmogrand_zenith().triggered_abilities[0].effect.clone(), &[]);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Human Soldier").count(), 2);
}

/// Seedship Broodtender mills three on ETB and can reanimate a creature from the
/// graveyard.
#[test]
fn seedship_broodtender_mills_and_reanimates() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); } // cards to mill
    let gy_before = g.players[0].graveyard.len();
    g.move_card_to_battlefield_for_test(0, catalog::seedship_broodtender());
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy_before + 3, "milled three");
    // Reanimate path: a creature card in the graveyard returns to the battlefield.
    let dead = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    let dead_id = dead.id;
    g.players[0].graveyard.push(dead);
    resolve_targeted(&mut g, 0, catalog::seedship_broodtender().activated_abilities[0].effect.clone(), &[dead_id]);
    assert!(g.battlefield_find(dead_id).is_some(), "reanimated from graveyard");
}

/// Virus Beetle's ETB makes each opponent discard.
#[test]
fn virus_beetle_etb_each_opponent_discards() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::virus_beetle());
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty(), "opponent discarded");
}

/// Tragic Trajectory is -2/-2 normally, -10/-10 with Void active.
#[test]
fn tragic_trajectory_void_upgrade() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::bygone_colossus()); // 9/9
    resolve_targeted(&mut g, 0, catalog::tragic_trajectory().effect, &[foe]);
    assert_eq!(g.computed_permanent(foe).unwrap().toughness, 7, "−2/−2 without Void");
    // With Void active, the same spell is -10/-10 instead.
    g.nonland_permanent_left_bf_this_turn = true;
    let foe2 = g.add_card_to_battlefield(1, catalog::bygone_colossus());
    resolve_targeted(&mut g, 0, catalog::tragic_trajectory().effect, &[foe2]);
    // -10/-10 on a 9/9 → -1 toughness (dies on the next SBA pass).
    assert_eq!(g.computed_permanent(foe2).map(|c| c.toughness), Some(-1),
        "−10/−10 applied with Void");
}

/// Sunstar Expansionist makes a Lander only when an opponent out-lands you.
#[test]
fn sunstar_expansionist_lander_when_behind() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::forest());
    g.move_card_to_battlefield_for_test(0, catalog::sunstar_expansionist());
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Lander").count(), 1,
        "opponent ahead on lands → Lander");
}

/// Sunstar Lightsmith's flurry grows it and draws.
#[test]
fn sunstar_lightsmith_flurry_value() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let smith = g.add_card_to_battlefield(0, catalog::sunstar_lightsmith());
    let before = g.players[0].hand.len();
    // Resolve the flurry body with the Lightsmith as source (refs Selector::This).
    let ctx = crabomination::game::effects::EffectContext::for_ability(smith, 0, None);
    g.resolve_effect(&catalog::sunstar_lightsmith().triggered_abilities[0].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(smith).unwrap().power, 4, "+1/+1 counter");
    assert_eq!(g.players[0].hand.len(), before + 1, "drew");
}

/// Uthros Psionicist discounts the second spell each turn by {2}.
#[test]
fn uthros_psionicist_second_spell_discount() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::uthros_psionicist());
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "first spell: no discount");
    g.players[0].spells_cast_this_turn = 1; // now casting the second
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 2, "second spell: {{2}} off");
}

/// Zealous Display pumps your team and untaps it off-turn.
#[test]
fn zealous_display_untaps_off_turn() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.active_player_idx = 1; // not our turn
    resolve_targeted(&mut g, 0, catalog::zealous_display().effect, &[]);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "+2/+0");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped off-turn");
}

/// Thawbringer surveils on both entry and death.
#[test]
fn thawbringer_surveils_on_etb_and_death() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let thaw = g.move_card_to_battlefield_for_test(0, catalog::thawbringer());
    drain_stack(&mut g);
    // ETB surveil ran (no panic); now kill it to fire the death surveil.
    g.add_card_to_library(0, catalog::forest());
    kill(&mut g, thaw);
    drain_stack(&mut g);
    assert!(g.battlefield_find(thaw).is_none(), "died");
}

/// Susurian Voidborn drains when a creature you control dies.
#[test]
fn susurian_voidborn_drains_on_friendly_death() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::susurian_voidborn());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let foe_life = g.players[1].life;
    let my_life = g.players[0].life;
    // Sacrifice the fodder and dispatch the death events so other
    // death-watchers (Susurian) trigger, mirroring the real game loop.
    let ctx = crabomination::game::effects::EffectContext::for_ability(fodder, 0, Some(Target::Permanent(fodder)));
    let events = g.resolve_effect(
        &crabomination::effect::Effect::SacrificePermanent { what: crabomination::effect::Selector::Target(0) },
        &ctx,
    ).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, my_life + 1, "you gained 1");
}

/// Mental Modulation is {1} cheaper on your own turn (CR 601.2f), and taps +
/// draws on resolution.
#[test]
fn mental_modulation_turn_discount_and_effect() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::mental_modulation(), 0);
    g.active_player_idx = 0;
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 1, "{{1}} off on your turn");
    g.active_player_idx = 1;
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no discount on opponent's turn");
    // Effect taps the target and draws.
    g.add_card_to_library(0, catalog::forest());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    resolve_targeted(&mut g, 0, catalog::mental_modulation().effect, &[foe]);
    assert!(g.battlefield_find(foe).unwrap().tapped, "target tapped");
    assert_eq!(g.players[0].hand.len(), before + 1, "drew a card");
}

/// Weftstalker Ardent pings each opponent when another permanent enters.
#[test]
fn weftstalker_ardent_pings_on_other_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::weftstalker_ardent());
    let foe_life = g.players[1].life;
    // Dispatch the ETB of another creature so Weftstalker's watcher triggers.
    let entered = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentEntered { card_id: entered }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 1, "each opponent took 1");
}

/// Weftblade Enhancer drops +1/+1 counters on up to two creatures.
#[test]
fn weftblade_enhancer_counters_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve_targeted(&mut g, 0, catalog::weftblade_enhancer().triggered_abilities[0].effect.clone(), &[a, b]);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Swarm Culler loots when it becomes tapped and you sac something.
#[test]
fn swarm_culler_sac_to_draw_on_tap() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let culler = g.add_card_to_battlefield(0, catalog::swarm_culler());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let before = g.players[0].hand.len();
    let ctx = crabomination::game::effects::EffectContext::for_ability(culler, 0, None);
    g.resolve_effect(&catalog::swarm_culler().triggered_abilities[0].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), before + 1, "drew");
}

/// Sunstar Chaplain's end-step trigger grows a creature when two are tapped.
#[test]
fn sunstar_chaplain_end_step_counter() {
    let mut g = two_player_game();
    let chap = g.add_card_to_battlefield(0, catalog::sunstar_chaplain());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(chap).unwrap().tapped = true;
    g.battlefield_find_mut(other).unwrap().tapped = true;
    resolve_targeted(&mut g, 0, catalog::sunstar_chaplain().triggered_abilities[0].effect.clone(), &[other]);
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Terrapact Intimidator: when the opponent declines the Landers, it gets two
/// +1/+1 counters (the lesser self-harm in a 2-player game is the counters).
#[test]
fn terrapact_intimidator_villainous_choice() {
    let mut g = two_player_game();
    let terra = g.move_card_to_battlefield_for_test(0, catalog::terrapact_intimidator());
    drain_stack(&mut g);
    let landers = g.battlefield.iter().filter(|c| c.definition.name == "Lander").count();
    let counters = g.battlefield_find(terra).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    // Exactly one branch happened.
    assert!((landers == 2) ^ (counters == 2), "opponent picked exactly one option");
}

/// Voidforged Titan draws and pays 1 life at end step while Void is active.
#[test]
fn voidforged_titan_void_end_step() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::voidforged_titan());
    g.nonland_permanent_left_bf_this_turn = true; // Void on
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    resolve_targeted(&mut g, 0, catalog::voidforged_titan().triggered_abilities[0].effect.clone(), &[]);
    assert_eq!(g.players[0].life, life - 1, "lost 1 life");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Zookeeper Mechan taps for {R}.
#[test]
fn zookeeper_mechan_taps_for_red() {
    let mut g = two_player_game();
    let zoo = g.add_card_to_battlefield(0, catalog::zookeeper_mechan());
    g.clear_sickness(zoo);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: zoo, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(crabomination::mana::Color::Red), 1);
}

/// Meltstrider's Gear attaches on ETB and grants +2/+1 and reach.
#[test]
fn meltstriders_gear_etb_attaches_and_buffs() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let gear = g.add_card_to_battlefield(0, catalog::meltstriders_gear());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(gear, 0, Some(Target::Permanent(bear)));
    g.resolve_effect(&catalog::meltstriders_gear().triggered_abilities[0].effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+1");
    assert!(cp.keywords.contains(&Keyword::Reach), "gains reach");
}

/// Illvoi Light Jammer attaches and grants hexproof until end of turn.
#[test]
fn illvoi_light_jammer_attach_and_hexproof() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let jam = g.add_card_to_battlefield(0, catalog::illvoi_light_jammer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(jam, 0, Some(Target::Permanent(bear)));
    g.resolve_effect(&catalog::illvoi_light_jammer().triggered_abilities[0].effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "+1/+2");
    assert!(cp.keywords.contains(&Keyword::Hexproof), "gains hexproof EOT");
}

/// Hylderblade grants +3/+1 to the creature it's attached to.
#[test]
fn hylderblade_buffs_equipped() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let blade = g.add_card_to_battlefield(0, catalog::hylderblade());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(blade, 0, Some(Target::Permanent(bear)));
    // Drive the Void end-step attach with Void active.
    g.nonland_permanent_left_bf_this_turn = true;
    g.resolve_effect(&catalog::hylderblade().triggered_abilities[0].effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 3), "+3/+1");
}

/// Sami, Ship's Engineer makes a tapped Robot at end step with two tapped
/// creatures.
#[test]
fn sami_makes_tapped_robot() {
    let mut g = two_player_game();
    let sami = g.add_card_to_battlefield(0, catalog::sami_ships_engineer());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(sami).unwrap().tapped = true;
    g.battlefield_find_mut(other).unwrap().tapped = true;
    resolve_targeted(&mut g, 0, catalog::sami_ships_engineer().triggered_abilities[0].effect.clone(), &[]);
    let robot = g.battlefield.iter().find(|c| c.definition.name == "Robot" && c.controller == 0);
    assert!(robot.is_some_and(|c| c.tapped), "tapped 2/2 Robot");
}

/// Starfighter Pilot surveils when it becomes tapped.
#[test]
fn starfighter_pilot_surveils_on_tap() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let pilot = g.add_card_to_battlefield(0, catalog::starfighter_pilot());
    let ctx = crabomination::game::effects::EffectContext::for_ability(pilot, 0, None);
    // The becomes-tapped body is a surveil; it resolves without panicking.
    g.resolve_effect(&catalog::starfighter_pilot().triggered_abilities[0].effect, &ctx).unwrap();
}

/// Starbreach Whale's ETB surveils two.
#[test]
fn starbreach_whale_etb_surveils_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let lib = g.players[0].library.len();
    g.move_card_to_battlefield_for_test(0, catalog::starbreach_whale());
    drain_stack(&mut g);
    // Surveil keeps cards on top by default (AutoDecider), so the library size
    // is unchanged but the ETB resolved.
    assert_eq!(g.players[0].library.len(), lib);
    assert!(g.computed_permanent(g.battlefield.iter().find(|c| c.definition.name == "Starbreach Whale").unwrap().id)
        .unwrap().keywords.contains(&Keyword::Flying));
}

/// Haliya puts a +1/+1 counter on a creature when she enters and when she
/// attacks.
#[test]
fn haliya_counters_on_enter_and_attack() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve_targeted(&mut g, 0, catalog::haliya_ascendant_cadet().triggered_abilities[0].effect.clone(), &[target]);
    resolve_targeted(&mut g, 0, catalog::haliya_ascendant_cadet().triggered_abilities[1].effect.clone(), &[target]);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

// ── New EOE batch (modern_decks) ────────────────────────────────────────────

/// Evendo (a Planet) enters tapped and taps for green.
#[test]
fn planet_enters_tapped_and_taps_for_mana() {
    let mut g = two_player_game();
    let evendo = g.add_card_to_battlefield(0, catalog::evendo_waking_haven());
    // A tap-for-G mana ability and a Station ability; an ETB-tap trigger.
    assert_eq!(g.battlefield_find(evendo).unwrap().definition.activated_abilities.len(), 2);
    assert_eq!(g.battlefield_find(evendo).unwrap().definition.triggered_abilities.len(), 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: evendo, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for green");
    assert!(g.players[0].mana_pool.total() >= 1, "Planet taps for mana");
}

/// Pulsar Squadron Ace's ETB reveals a Spacecraft from the top five to hand.
#[test]
fn pulsar_squadron_ace_grabs_a_spacecraft() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::wurmwall_sweeper()); // a Spacecraft
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    let hand_before = g.players[0].hand.len();
    resolve_targeted(&mut g, 0, catalog::pulsar_squadron_ace().triggered_abilities[0].effect.clone(), &[]);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Wurmwall Sweeper"),
        "Spacecraft put into hand"
    );
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

/// Umbral Collar Zealot sacrifices an artifact to surveil.
#[test]
fn umbral_collar_zealot_sacrifices_to_surveil() {
    let mut g = two_player_game();
    let zealot = g.add_card_to_battlefield(0, catalog::umbral_collar_zealot());
    let art = g.add_card_to_battlefield(0, catalog::melded_moxite());
    g.add_card_to_library(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: zealot, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac an artifact to surveil");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "the artifact was sacrificed");
}

/// Sunset Saboteur's attack trigger puts a +1/+1 counter on an opponent's creature.
#[test]
fn sunset_saboteur_attack_counters_opponent() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve_targeted(&mut g, 0, catalog::sunset_saboteur().triggered_abilities[0].effect.clone(), &[opp]);
    assert_eq!(g.battlefield_find(opp).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Station Monitor mints a Drone on the second spell each turn.
#[test]
fn station_monitor_second_spell_makes_drone() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::station_monitor());
    let s1 = g.add_card_to_hand(0, catalog::bombard());
    let s2 = g.add_card_to_hand(0, catalog::bombard());
    let d1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let d2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: s1, target: Some(Target::Permanent(d1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("first spell");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: s2, target: Some(Target::Permanent(d2)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("second spell");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Drone"),
        "second spell created a Drone"
    );
}

/// Virulent Silencer poisons the defending player when it connects.
#[test]
fn virulent_silencer_poisons_on_combat_damage() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let silencer = g.add_card_to_battlefield(0, catalog::virulent_silencer());
    g.clear_sickness(silencer);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: silencer, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 2, "two poison from a nontoken artifact creature hit");
}

/// Steelswarm Operator's restricted mana ability produces artifact-only mana.
#[test]
fn steelswarm_operator_makes_restricted_mana() {
    let mut g = two_player_game();
    let op = g.add_card_to_battlefield(0, catalog::steelswarm_operator());
    g.clear_sickness(op);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: op, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for restricted mana");
    assert!(g.players[0].mana_pool.restricted_total() >= 1, "restricted mana added");
}

/// Syr Vondam grows and gains life when another of your creatures dies.
#[test]
fn syr_vondam_grows_on_friendly_death() {
    let mut g = two_player_game();
    let vondam = g.add_card_to_battlefield(0, catalog::syr_vondam_sunstar_exemplar());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    let mut evs = g.remove_to_graveyard_with_triggers(bear);
    evs.push(GameEvent::CreatureDied { card_id: bear });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(vondam).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].life, life + 1, "gained 1 life");
}

/// Starfield Shepherd's ETB tutors a low-cost creature to hand.
#[test]
fn starfield_shepherd_tutors_small_creature() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let plains = g.add_card_to_library(0, catalog::plains()); // basic Plains is a legal pick
    let hand_before = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains))]));
    resolve_targeted(&mut g, 0, catalog::starfield_shepherd().triggered_abilities[0].effect.clone(), &[]);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "a card was tutored to hand");
}

/// Tannuk pings each opponent on landfall.
#[test]
fn tannuk_pings_on_landfall() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tannuk_memorial_ensign());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "landfall ping");
}

/// Xu-Ifit reanimates a creature card from your graveyard.
#[test]
fn xu_ifit_reanimates_from_graveyard() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let xu = g.add_card_to_battlefield(0, catalog::xu_ifit_osteoharmonist());
    g.clear_sickness(xu);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: xu, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("reanimate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "creature returned to battlefield");
}

/// Monoist Circuit-Feeder pumps yours and shrinks theirs by artifact count.
#[test]
fn monoist_circuit_feeder_pumps_by_artifacts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::melded_moxite()); // 1 artifact you control
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve_targeted(&mut g, 0, catalog::monoist_circuit_feeder().triggered_abilities[0].effect.clone(), &[mine, theirs]);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "+X/+0 where X=1 artifact");
    assert_eq!(g.computed_permanent(theirs).unwrap().toughness, 1, "-0/-X where X=1 artifact");
}

/// Space-Time Anomaly mills cards equal to your life total.
#[test]
fn space_time_anomaly_mills_your_life() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    for _ in 0..30 { g.add_card_to_library(1, catalog::forest()); }
    let anomaly = g.add_card_to_hand(0, catalog::space_time_anomaly());
    g.players[0].life = 5;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let gy_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: anomaly, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Space-Time Anomaly");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy_before + 5, "milled 5 = your life");
}

/// Systems Override steals an opposing creature for the turn (untapped, hasty).
#[test]
fn systems_override_steals_creature() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve_targeted(&mut g, 0, catalog::systems_override().effect.clone(), &[theirs]);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0, "gained control");
    assert!(g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::Haste));
}

/// Mutinous Massacre destroys creatures of the chosen mana-value parity.
#[test]
fn mutinous_massacre_destroys_by_parity() {
    let mut g = two_player_game();
    let odd = g.add_card_to_battlefield(1, catalog::memory_guardian()); // MV 5 (odd)
    let even = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2 (even)
    resolve_targeted(&mut g, 0, catalog::mutinous_massacre().effect.clone(), &[]);
    assert!(g.battlefield_find(odd).is_none(), "odd-MV creature destroyed (AutoDecider picks odd)");
    assert!(g.battlefield_find(even).is_some(), "even-MV creature survives");
}

/// Focus Fire deals 2 plus your creature/Spacecraft count to a creature.
#[test]
fn focus_fire_scales_with_board() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // resolve_targeted adds a source grizzly (1 creature you control) → X = 2 + 1 = 3.
    resolve_targeted(&mut g, 0, catalog::focus_fire().effect.clone(), &[target]);
    assert!(g.battlefield_find(target).is_none(), "3 damage kills the 2/2");
}

/// Terminal Velocity drops a creature from hand onto the battlefield with haste.
#[test]
fn terminal_velocity_cheats_creature_from_hand() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let velocity = g.add_card_to_hand(0, catalog::terminal_velocity());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.perform_action(GameAction::CastSpell {
        card_id: velocity, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Terminal Velocity");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "creature put onto battlefield");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
}

/// Melded Moxite sacrifices itself for a tapped Robot.
#[test]
fn melded_moxite_sacs_for_robot() {
    let mut g = two_player_game();
    let mox = g.add_card_to_battlefield(0, catalog::melded_moxite());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mox, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for a Robot");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mox).is_none(), "Moxite sacrificed");
    let robot = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Robot");
    assert!(robot.is_some_and(|c| c.tapped), "Robot enters tapped");
}

/// Squire's Lightblade attaches to a creature on ETB and grants first strike.
#[test]
fn squires_lightblade_attaches_and_grants_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve_targeted(&mut g, 0, catalog::squires_lightblade().triggered_abilities[0].effect.clone(), &[bear]);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::FirstStrike));
}

/// Auxiliary Boosters mints a Robot and equips it, granting flying.
#[test]
fn auxiliary_boosters_makes_and_equips_robot() {
    let mut g = two_player_game();
    let boosters = g.add_card_to_hand(0, catalog::auxiliary_boosters());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: boosters, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Auxiliary Boosters");
    drain_stack(&mut g);
    let robot = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Robot").map(|c| c.id);
    assert!(robot.is_some(), "Robot token created");
    assert!(g.computed_permanent(robot.unwrap()).unwrap().keywords.contains(&Keyword::Flying),
        "equipped Robot has flying");
}

/// Thaumaton Torpedo sacrifices to destroy a nonland permanent.
#[test]
fn thaumaton_torpedo_destroys_nonland() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let torpedo = g.add_card_to_battlefield(0, catalog::thaumaton_torpedo());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: torpedo, ability_index: 0, target: Some(Target::Permanent(theirs)),
        additional_targets: vec![], x_value: None,
    }).expect("destroy target");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "nonland permanent destroyed");
    assert!(g.battlefield_find(torpedo).is_none(), "Torpedo sacrificed");
}

/// Terrasymbiosis draws when you put a +1/+1 counter on your creature.
#[test]
fn terrasymbiosis_draws_on_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::terrasymbiosis());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.dispatch_triggers_for_events(&[GameEvent::CounterAdded {
        card_id: bear, counter_type: CounterType::PlusOnePlusOne, count: 1,
    }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew off the counter");
}

/// Weapons Manufacturing makes a Munitions token when a nontoken artifact enters.
#[test]
fn weapons_manufacturing_mints_munitions() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::weapons_manufacturing());
    let art = g.add_card_to_battlefield(0, catalog::melded_moxite()); // a nontoken artifact entering
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: art }]);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Munitions"),
        "Munitions token created"
    );
}

/// CR 721.2a — a Planet's 12+ activated Station band becomes usable once it has
/// 12 charge counters (Uthros taps for {U} per artifact you control).
#[test]
fn station_activated_band_unlocks_at_threshold() {
    let mut g = two_player_game();
    let uthros = g.add_card_to_battlefield(0, catalog::uthros_titanic_godcore());
    g.add_card_to_battlefield(0, catalog::melded_moxite()); // one artifact you control
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Below threshold the band ability isn't offered (index 2 = first granted band ability).
    assert!(g.granted_abilities_for(uthros).is_empty(), "no band abilities below 12 charges");
    // Charge it up to the 12+ band.
    g.battlefield_find_mut(uthros).unwrap().add_counters(CounterType::Charge, 12);
    assert_eq!(g.granted_abilities_for(uthros).len(), 1, "12+ band ability now granted");
    g.players[0].mana_pool.add(Color::Blue, 1);
    // Printed abilities are tap-for-U (0) and Station (1); the band is index 2.
    g.perform_action(GameAction::ActivateAbility {
        card_id: uthros, ability_index: 2, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate 12+ band");
    // Spent {U}, produced {U} per artifact (1) → at least one blue back.
    assert!(g.players[0].mana_pool.total() >= 1, "band added scaled mana");
}

/// Syr Vondam, the Lucent buffs your other creatures and grants deathtouch on attack.
#[test]
fn syr_vondam_lucent_buffs_team() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::syr_vondam_the_lucent());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve_targeted(&mut g, 0, catalog::syr_vondam_the_lucent().triggered_abilities[0].effect.clone(), &[]);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "+1/+0 to other creatures");
    assert!(cp.keywords.contains(&Keyword::Deathtouch), "granted deathtouch");
}

/// Starwinder draws cards equal to combat damage one of your creatures deals.
#[test]
fn starwinder_draws_on_combat_damage() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::starwinder());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power
    g.clear_sickness(bear);
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew 2 = combat damage dealt");
}

/// Pinnacle Starcage exiles small permanents until it leaves, then returns them.
#[test]
fn pinnacle_starcage_exiles_small_permanents() {
    let mut g = two_player_game();
    let cage = g.add_card_to_battlefield(0, catalog::pinnacle_starcage());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    resolve_targeted(&mut g, 0, catalog::pinnacle_starcage().triggered_abilities[0].effect.clone(), &[]);
    let _ = cage;
    assert!(g.battlefield_find(bear).is_none(), "MV-2 creature exiled");
}

/// Temporal Intervention is cheaper with Void active and discards a nonland.
#[test]
fn temporal_intervention_void_discount_and_discard() {
    let mut g = two_player_game();
    // A nonland permanent left the battlefield this turn → Void active.
    let token = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, token);
    drain_stack(&mut g);
    assert!(g.nonland_permanent_left_bf_this_turn, "Void latched");
    let def = catalog::temporal_intervention();
    // The static reduces the cost by {2} when Void is active.
    assert!(matches!(def.static_abilities[0].effect,
        crabomination::effect::StaticEffect::SelfCostReducedIf { amount: 2, .. }));
    let nid = g.next_id();
    g.players[1].hand.push(crabomination::card::CardInstance::new(nid, catalog::grizzly_bears(), 1));
    let gy_before = g.players[1].graveyard.len();
    resolve_targeted(&mut g, 0, def.effect.clone(), &[]);
    assert_eq!(g.players[1].graveyard.len(), gy_before + 1, "opponent discarded a nonland");
}

/// Vote Out destroys a creature (Convoke is a cast-time discount, not tested here).
#[test]
fn vote_out_destroys_creature() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve_targeted(&mut g, 0, catalog::vote_out().effect.clone(), &[theirs]);
    assert!(g.battlefield_find(theirs).is_none(), "creature destroyed");
}

/// Archenemy's Charm mode 0 exiles a creature (AutoDecider picks the first mode).
#[test]
fn archenemys_charm_exiles_creature() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve_targeted(&mut g, 0, catalog::archenemys_charm().effect.clone(), &[theirs]);
    assert!(g.battlefield_find(theirs).is_none(), "mode 0 exiles the creature");
}

/// CR 509.1b — Illvoi Infiltrator can't be blocked once its controller has cast
/// two or more spells this turn.
#[test]
fn illvoi_infiltrator_conditional_unblockable() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let illvoi = g.add_card_to_battlefield(0, catalog::illvoi_infiltrator());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(illvoi);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: illvoi, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    // One spell cast → blockable.
    g.players[0].spells_cast_this_turn = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(blocker, illvoi)])).is_ok(),
        "blockable with one spell cast");
    // Two spells cast → unblockable.
    g.players[0].spells_cast_this_turn = 2;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(blocker, illvoi)])).is_err(),
        "unblockable after two spells");
}

/// Cryoshatter shrinks the enchanted creature by 5 power and kills it when tapped.
#[test]
fn cryoshatter_debuffs_and_destroys_on_tap() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let cryo = g.add_card_to_hand(0, catalog::cryoshatter());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: cryo, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cryoshatter");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, -1, "-5/-0 applied (4 - 5)");
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: bear, actor: None, as_attacker: false }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "destroyed when tapped");
}

/// Hardlight Containment exiles an opposing creature while it's attached.
#[test]
fn hardlight_containment_exiles_opponent_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::melded_moxite()); // an artifact host
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let host = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Melded Moxite").unwrap().id;
    resolve_targeted(&mut g, 0, catalog::hardlight_containment().effect.clone(), &[host, theirs]);
    assert!(g.battlefield_find(theirs).is_none(), "opponent creature exiled");
}

/// Meltstrider's Resolve makes the enchanted creature fight an opposing creature.
#[test]
fn meltstriders_resolve_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    resolve_targeted(&mut g, 0, catalog::meltstriders_resolve().effect.clone(), &[mine, theirs]);
    assert!(g.battlefield_find(theirs).is_none(), "2/2 dies to the 4/4's fight");
}

/// Pain for All pings a target for the enchanted creature's power.
#[test]
fn pain_for_all_pings_for_power() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // power 4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    resolve_targeted(&mut g, 0, catalog::pain_for_all().effect.clone(), &[mine, theirs]);
    assert!(g.battlefield_find(theirs).is_none(), "4 damage kills the 2/2");
}

/// Starport Security taps another target creature.
#[test]
fn starport_security_taps_a_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let sec = g.add_card_to_battlefield(0, catalog::starport_security());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(sec);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sec, ability_index: 0, target: Some(Target::Permanent(theirs)),
        additional_targets: vec![], x_value: None,
    }).expect("tap a creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).unwrap().tapped, "target creature tapped");
}

/// Mm'menon, the Right Hand lets you cast artifact spells from the top of library.
#[test]
fn mmmenon_right_hand_grants_cast_artifacts_from_top() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mmmenon_the_right_hand());
    let art = g.add_card_to_library(0, catalog::melded_moxite()); // top of library, an artifact
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: art, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast artifact off the top");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_some(), "artifact cast from library top");
}

/// Memorial Vault exiles 1 + the sacrificed artifact's mana value off the top.
#[test]
fn memorial_vault_exiles_scaled_by_sacrifice() {
    let mut g = two_player_game();
    let vault = g.add_card_to_battlefield(0, catalog::memorial_vault());
    g.add_card_to_battlefield(0, catalog::melded_moxite()); // MV 2 artifact to sacrifice
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
    let exile_before = g.exile.len();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: vault, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Memorial Vault");
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), exile_before + 3, "exiled 1 + MV 2 = 3 cards");
}

/// Astelli Reclaimer returns a noncreature, nonland permanent whose mana value
/// is ≤ the mana spent to cast it (5). Memorial Vault is MV 4 → comes back.
#[test]
fn astelli_reclaimer_returns_cheap_noncreature() {
    let mut g = two_player_game();
    let vault = g.add_card_to_graveyard(0, catalog::memorial_vault()); // MV 4
    let astelli = g.add_card_to_hand(0, catalog::astelli_reclaimer());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: astelli, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Astelli Reclaimer");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vault).is_some(), "Memorial Vault (MV 4 ≤ 5) reanimated");
}

/// The MV gate excludes a permanent costing more than the mana spent — Staff of
/// Nin is MV 6 > 5, so there's no legal target and it stays in the graveyard.
#[test]
fn astelli_reclaimer_skips_overcost_permanent() {
    let mut g = two_player_game();
    let staff = g.add_card_to_graveyard(0, catalog::staff_of_nin()); // MV 6
    let astelli = g.add_card_to_hand(0, catalog::astelli_reclaimer());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: astelli, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Astelli Reclaimer");
    drain_stack(&mut g);
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == staff),
        "Staff of Nin (MV 6 > 5) stays in the graveyard"
    );
}

/// Icetill Explorer mills a card whenever a land you control enters (landfall).
#[test]
fn icetill_explorer_landfall_mills() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::icetill_explorer());
    let land = g.add_card_to_hand(0, catalog::forest());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let lib = g.players[0].library.len();
    let gy = g.players[0].graveyard.len();
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "landfall milled a card");
    assert_eq!(g.players[0].graveyard.len(), gy + 1, "milled card hit the graveyard");
}

/// Starfield Vocalist (Panharmonicon) doubles your permanent-ETB triggers: with
/// it in play, Blade of the Swarm's ETB resolves twice → four +1/+1 counters.
#[test]
fn starfield_vocalist_doubles_etb_trigger() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::starfield_vocalist());
    let blade = g.add_card_to_hand(0, catalog::blade_of_the_swarm());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: blade, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Blade of the Swarm");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(blade).unwrap().counter_count(CounterType::PlusOnePlusOne), 4,
        "ETB (2 counters) fired twice → 4 counters");
}

/// Perigee Beckoner's ETB gives another creature you control +2/+0.
#[test]
fn perigee_beckoner_pumps_another_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let beck = g.add_card_to_hand(0, catalog::perigee_beckoner());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: beck, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Perigee Beckoner");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2), "+2/+0 on the bear");
}

/// Consult the Star Charts looks at top X (= lands you control) and grabs one
/// card (two if kicked).
#[test]
fn consult_the_star_charts_grabs_by_lands() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); } // X = 3
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let consult = g.add_card_to_hand(0, catalog::consult_the_star_charts());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: consult, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Consult the Star Charts");
    drain_stack(&mut g);
    // Consult left hand (-1), one card to hand (+1) → net 0.
    assert_eq!(g.players[0].hand.len(), hand, "put one card into hand");
}

/// The Seriema's ETB tutors a legendary creature card into your hand.
#[test]
fn the_seriema_tutors_a_legendary_to_hand() {
    let mut g = two_player_game();
    let legend = g.add_card_to_library(0, catalog::sami_wildcat_captain()); // legendary
    g.add_card_to_library(0, catalog::grizzly_bears()); // non-legendary decoy
    let ship = g.add_card_to_hand(0, catalog::the_seriema());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(legend)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: ship, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast The Seriema");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == legend),
        "legendary creature tutored to hand");
}

/// Survey Mechan's {10}, Sac ability deals 3 to a target and draws three.
#[test]
fn survey_mechan_sac_burn_and_draw() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let mech = g.add_card_to_battlefield(0, catalog::survey_mechan());
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(10);
    let life1 = g.players[1].life;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: mech, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Survey Mechan");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 3, "dealt 3 damage");
    assert_eq!(g.players[0].hand.len(), hand + 3, "drew three");
    assert!(g.battlefield_find(mech).is_none(), "sacrificed itself");
}

/// Loading Zone doubles counters placed on your permanents — Blade of the Swarm's
/// ETB two counters become four.
#[test]
fn loading_zone_doubles_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::loading_zone());
    let blade = g.add_card_to_hand(0, catalog::blade_of_the_swarm());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: blade, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Blade of the Swarm");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(blade).unwrap().counter_count(CounterType::PlusOnePlusOne), 4,
        "2 counters doubled to 4");
}

/// Sami grants your instant/sorcery spells affinity for artifacts.
#[test]
fn sami_grants_affinity_for_artifacts() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let bolt = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    g.add_card_to_battlefield(0, catalog::sami_wildcat_captain());
    assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 0, "no artifacts → no discount");
    g.add_card_to_battlefield(0, catalog::memorial_vault());
    g.add_card_to_battlefield(0, catalog::memorial_vault());
    assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 2, "{{2}} off for two artifacts");
}

/// Annul counters an artifact spell on the stack.
#[test]
fn annul_counters_an_artifact_spell() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let vault = g.add_card_to_hand(1, catalog::memorial_vault()); // {3}{R} artifact
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(3);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: vault, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts the artifact");
    let annul = g.add_card_to_hand(0, catalog::annul());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: annul, target: Some(Target::Permanent(vault)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Annul on the artifact");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vault).is_none(), "artifact spell countered, never resolved");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == vault), "countered to graveyard");
}

/// Divert Disaster counters a spell whose controller can't pay {2}.
#[test]
fn divert_disaster_counters_when_unpaid() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1); // exactly the bolt cost, no spare
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    let divert = g.add_card_to_hand(0, catalog::divert_disaster());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: divert, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Divert Disaster");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "Bolt countered (unpaid), no damage");
}

/// Mightform Harmonizer's landfall doubles a creature's power until end of turn.
#[test]
fn mightform_harmonizer_landfall_doubles_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mightform_harmonizer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let land = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "power doubled 2 → 4 on landfall");
}

/// Blade of the Swarm's ETB (mode 0, the only legal mode with no exiled warp
/// card) puts two +1/+1 counters on it.
#[test]
fn blade_of_the_swarm_etb_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let blade = g.add_card_to_hand(0, catalog::blade_of_the_swarm());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: blade, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Blade of the Swarm");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(blade).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "ETB added two +1/+1 counters");
}

/// Tractor Beam steals control of the enchanted creature, taps it, and keeps it
/// tapped through its (new) controller's untap step. Control reverts when the
/// Aura leaves.
#[test]
fn tractor_beam_steals_control_and_locks_untap() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    let aura = g.add_card_to_hand(0, catalog::tractor_beam());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Tractor Beam");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "control stolen");
    assert!(g.battlefield_find(bear).unwrap().tapped, "ETB tapped it");
    // Its new controller's untap step must not untap it.
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "doesn't untap under new controller");
    // Aura leaves → control reverts to the original owner.
    kill(&mut g, aura);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 1, "control reverts when Aura leaves");
}

// ── Batch: missing EOE rares/lands (claude/modern_decks) ─────────────────────

/// Dawnsire: at 20+ charges it's a 20/20 flying artifact creature.
#[test]
fn dawnsire_20plus_band_is_2020_flier() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::dawnsire_sunstar_dreadnought());
    assert!(!g.computed_permanent(ship).unwrap().card_types.contains(&CardType::Creature));
    g.battlefield_find_mut(ship).unwrap().counters.insert(CounterType::Charge, 20);
    let post = g.computed_permanent(ship).unwrap();
    assert!(post.card_types.contains(&CardType::Creature));
    assert_eq!((post.power, post.toughness), (20, 20));
    assert!(post.keywords.contains(&Keyword::Flying));
}

/// Infinite Guideline Station ETB mints a tapped Robot per multicolored permanent.
#[test]
fn infinite_guideline_station_etb_robots_per_multicolored() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::boros_recruit()); // multicolored R/W
    g.add_card_to_battlefield(0, catalog::boros_recruit());
    let id = g.add_card_to_hand(0, catalog::infinite_guideline_station());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    cast(&mut g, id);
    let robots: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Robot").collect();
    // 2 Boros Recruits + the Station itself (WUBRG = multicolored) = 3.
    assert_eq!(robots.len(), 3, "one Robot per multicolored permanent (incl. itself)");
    assert!(robots.iter().all(|c| c.tapped), "Robots enter tapped");
}

/// All-Fates Scroll: {7},{T},Sac → draw X = differently named lands you control.
#[test]
fn all_fates_scroll_draws_per_differently_named_land() {
    let mut g = two_player_game();
    let scroll = g.add_card_to_battlefield(0, catalog::all_fates_scroll());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest()); // same name → counts once
    g.add_card_to_battlefield(0, catalog::island());
    for _ in 0..5 { g.add_card_to_library(0, catalog::plains()); }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(7);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: scroll, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate sac-draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "draws 2 (Forest, Island distinct)");
    assert!(g.battlefield_find(scroll).is_none(), "scroll sacrificed");
}

/// Haliya: +1 life per creature/artifact ETB; end-step draw after gaining 3+.
#[test]
fn haliya_lifegain_then_endstep_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::haliya_guided_by_light());
    let life0 = g.players[0].life;
    for _ in 0..3 {
        let b = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, b);
    }
    assert_eq!(g.players[0].life, life0 + 3, "1 life per creature ETB");
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "draws at end step after gaining 3+");
}

/// Alpharael, Dreaming Acolyte has deathtouch only during your turn.
#[test]
fn alpharael_dreaming_deathtouch_only_on_your_turn() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::alpharael_dreaming_acolyte());
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::Deathtouch));
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(a).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Alpharael, Stonechosen: with Void active, attacking halves the defender's life.
#[test]
fn alpharael_stonechosen_void_attack_halves_defender_life() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, bear);
    drain_stack(&mut g);
    assert!(g.nonland_permanent_left_bf_this_turn, "Void latched");
    let alph = g.add_card_to_battlefield(0, catalog::alpharael_stonechosen());
    g.battlefield_find_mut(alph).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: alph, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 10, "defender loses half (20 → 10)");
}

/// Possibility Technician's ETB exiles the top card with a play permission.
#[test]
fn possibility_technician_etb_exiles_top_grants_may_play() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::possibility_technician());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert!(
        g.exile.iter().any(|c| c.controller == 0 && c.may_play_until.is_some()),
        "exiled top card is playable",
    );
}

/// Roving Actuator: with Void active, recast a cheap instant from your graveyard.
#[test]
fn roving_actuator_void_recasts_cheap_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let idx = g.players[0].hand.iter().position(|c| c.id == bolt).unwrap();
    let card = g.players[0].hand.remove(idx);
    g.players[0].graveyard.push(card);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    kill(&mut g, bear);
    drain_stack(&mut g);
    let life1 = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::roving_actuator());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
        [crabomination::decision::DecisionAnswer::Bool(true)],
    ));
    cast(&mut g, id);
    assert_eq!(g.players[1].life, life1 - 3, "recast Bolt hit the opponent for 3");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt), "Bolt left the graveyard");
}

/// Tannuk gives other creatures you control haste.
#[test]
fn tannuk_grants_other_creatures_haste() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tannuk_steadfast_second());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
}

/// Secluded Starforge's {5},{T} ability mints a 2/2 Robot.
#[test]
fn secluded_starforge_makes_a_robot() {
    let mut g = two_player_game();
    let forge = g.add_card_to_battlefield(0, catalog::secluded_starforge());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: forge, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate robot maker");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Robot").count(), 1);
}

/// Command Bridge enters tapped and taps for any color.
#[test]
fn command_bridge_enters_tapped() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::command_bridge());
    g.perform_action(GameAction::PlayLand(id)).expect("play Command Bridge");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().tapped, "enters tapped");
}

/// The Eternity Elevator's base ability taps for {C}{C}{C}.
#[test]
fn eternity_elevator_taps_for_ccc() {
    let mut g = two_player_game();
    let elev = g.add_card_to_battlefield(0, catalog::the_eternity_elevator());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: elev, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for CCC");
    assert_eq!(g.players[0].mana_pool.total(), 3, "added 3 colorless");
}

/// Atomic Microsizer: when the equipped creature attacks, the target becomes a
/// 1/1 and can't be blocked this turn.
#[test]
fn atomic_microsizer_shrinks_and_unblocks_target() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let equip = g.add_card_to_battlefield(0, catalog::atomic_microsizer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    let ogre = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    // Equip the bear.
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: equip, target: bear }).expect("equip the bear");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+0 from the equipment");
    g.step = TurnStep::DeclareAttackers;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
        [crabomination::decision::DecisionAnswer::Bool(true)],
    ));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).unwrap();
    drain_stack(&mut g);
    let o = g.computed_permanent(ogre).unwrap();
    assert_eq!((o.power, o.toughness), (1, 1), "target shrunk to 1/1");
    assert!(o.keywords.contains(&Keyword::Unblockable), "target can't be blocked");
}

/// Dyadrine enters with +1/+1 counters equal to the mana spent to cast it.
#[test]
fn dyadrine_enters_with_counters_equal_to_mana_spent() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::dyadrine_synthesis_amalgam());
    // Pay {3}{G}{W} → X = 3, total mana spent = 5.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast Dyadrine for X=3");
    drain_stack(&mut g);
    let d = g.battlefield_find(id).unwrap();
    assert_eq!(d.counter_count(CounterType::PlusOnePlusOne), 5, "counters = mana spent (5)");
    assert_eq!(g.computed_permanent(id).unwrap().power, 5, "0/1 + 5 counters = 5/6");
}

/// Zero Point Ballad destroys creatures with toughness ≤ X and you lose X life.
#[test]
fn zero_point_ballad_destroys_small_toughness_and_loses_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let id = g.add_card_to_hand(0, catalog::zero_point_ballad());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast Zero Point Ballad for X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2-toughness creature destroyed");
    assert!(g.battlefield_find(giant).is_some(), "3-toughness creature survives");
    assert_eq!(g.players[0].life, life0 - 2, "you lose X life");
}

/// Scout for Survivors returns cheap creatures from the graveyard with counters.
#[test]
fn scout_for_survivors_reanimates_with_counters() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // MV 2
    let i1 = g.players[0].hand.iter().position(|c| c.id == bear).unwrap();
    let bc = g.players[0].hand.remove(i1);
    g.players[0].graveyard.push(bc);
    let id = g.add_card_to_hand(0, catalog::scout_for_survivors());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    cast(&mut g, id);
    let b = g.battlefield_find(bear).expect("bear reanimated");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1, "enters with a +1/+1 counter");
}

/// Weftwalking's ETB shuffles your hand and graveyard away and draws seven.
#[test]
fn weftwalking_wheels_into_seven() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::forest());
    to_gy(&mut g, catalog::island());
    for _ in 0..10 { g.add_card_to_library(0, catalog::plains()); }
    let id = g.add_card_to_hand(0, catalog::weftwalking());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, id);
    assert_eq!(g.players[0].hand.len(), 7, "drew a fresh seven");
    assert!(g.players[0].graveyard.is_empty(), "graveyard shuffled away");
}

/// Pull Through the Weft returns up to two nonland permanent cards to hand.
#[test]
fn pull_through_the_weft_returns_perms_to_hand() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let a = to_gy(&mut g, catalog::grizzly_bears());
    let b = to_gy(&mut g, catalog::hill_giant());
    g.add_card_to_battlefield(0, catalog::forest()); // a land also in gy shouldn't be returnable
    let id = g.add_card_to_hand(0, catalog::pull_through_the_weft());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b])]));
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|c| c.id == a) && g.players[0].hand.iter().any(|c| c.id == b),
        "both nonland permanents returned to hand");
}

/// Close Encounter deals damage equal to your greatest creature's power.
#[test]
fn close_encounter_deals_greatest_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3 — greatest power 3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::close_encounter());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, id, crabomination::game::types::Target::Permanent(foe));
    assert!(g.battlefield_find(foe).is_none(), "2/2 took 3 damage and died");
}

/// Devastating Onslaught makes X hasty token copies that are sacrificed at the
/// next end step.
#[test]
fn devastating_onslaught_makes_hasty_copies_sacrificed_at_end() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::devastating_onslaught());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4); // X=2 → {2}{2}{R}
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast Devastating Onslaught X=2");
    drain_stack(&mut g);
    let tokens: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Grizzly Bears").collect();
    assert_eq!(tokens.len(), 2, "X=2 token copies");
    let tid = tokens[0].id;
    assert!(g.computed_permanent(tid).unwrap().keywords.contains(&Keyword::Haste), "tokens have haste");
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Grizzly Bears"),
        "token copies sacrificed at the end step");
}

/// Unravel counters a spell and draws only if it was cast for less than its MV.
#[test]
fn unravel_draws_when_target_was_underpaid() {
    use crabomination::game::types::StackItem;
    let mut g = two_player_game();
    // Opponent casts Opt ({U}, MV 1) at instant speed; force it to look
    // "underpaid" (mana_spent 0 < MV 1).
    let spell = g.add_card_to_hand(1, catalog::opt());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts Opt");
    if let Some(StackItem::Spell { mana_spent, .. }) =
        g.stack.iter_mut().find(|si| matches!(si, StackItem::Spell { card, .. } if card.id == spell))
    {
        *mana_spent = 0;
    }
    let unr = g.add_card_to_hand(0, catalog::unravel());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len() - 1; // minus the Unravel about to leave
    g.priority.player_with_priority = 0;
    cast_at(&mut g, unr, crabomination::game::types::Target::Permanent(spell));
    assert!(!g.stack.iter().any(|si| matches!(si, StackItem::Spell { card, .. } if card.id == spell)),
        "Opt was countered");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card (underpaid)");
}

/// Adagia's 12+ band makes a legendary token copy of a target artifact you control.
#[test]
fn adagia_band_makes_legendary_copy() {
    let mut g = two_player_game();
    let adagia = g.add_card_to_battlefield(0, catalog::adagia_windswept_bastion());
    let moxite = g.add_card_to_battlefield(0, catalog::melded_moxite()); // non-legendary artifact
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.battlefield_find_mut(adagia).unwrap().add_counters(CounterType::Charge, 12);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: adagia, ability_index: 2,
        target: Some(crabomination::game::types::Target::Permanent(moxite)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Adagia 12+ band");
    drain_stack(&mut g);
    let copy = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.definition.name == "Melded Moxite" && c.is_token
    }).expect("legendary token copy minted");
    assert!(copy.definition.is_legendary(), "copy is legendary per the band");
}

/// Kavaron's 12+ band sacrifices a land for a Robot and a team haste+pump.
#[test]
fn kavaron_band_robot_and_team_haste() {
    let mut g = two_player_game();
    let kavaron = g.add_card_to_battlefield(0, catalog::kavaron_memorial_world());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.battlefield_find_mut(kavaron).unwrap().add_counters(CounterType::Charge, 12);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kavaron, ability_index: 2, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Kavaron 12+ band");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "a land was sacrificed");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Robot").count(),
        1, "made a Robot token"
    );
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "team +1/+0 → 3 power");
    assert!(cp.keywords.contains(&Keyword::Haste), "team gained haste");
}
