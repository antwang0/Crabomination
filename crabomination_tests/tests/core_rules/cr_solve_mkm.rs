//! Comprehensive-Rules conformance for MKM's Solve keyword action (the Case
//! mechanic), multi-source deathtouch (CR 702.2c) through
//! `EachControlledCreatureDealsDamage`, and layer-7 P/T stacking (CR 613.7c/d).

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

fn is_solved(g: &crabomination::game::GameState, id: crabomination::card::CardId) -> bool {
    g.battlefield.iter().find(|c| c.id == id).map(|c| c.case_solved).unwrap_or(false)
}

/// CR 719.3a — "To solve — [condition]" is an end-step trigger on *its
/// controller's* end step, and once solved a Case stays solved even if the
/// condition later stops holding (719.3b).
#[test]
fn cr_719_3a_case_solves_at_controllers_end_step_and_persists() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
    assert!(g.players[0].hand.is_empty(), "solve condition (empty hand) holds");

    // The opponent's end step must not solve the controller's Case.
    g.active_player_idx = 1;
    let mut evs = vec![];
    g.process_case_solves(&mut evs);
    assert!(!is_solved(&g, case), "not solved on the opponent's end step");

    // The controller's end step solves it.
    g.active_player_idx = 0;
    let mut evs = vec![];
    g.process_case_solves(&mut evs);
    drain_stack(&mut g);
    assert!(is_solved(&g, case), "solved at the controller's end step");

    // Once solved, a later end step where the condition fails leaves it solved.
    g.add_card_to_hand(0, catalog::forest());
    let mut evs = vec![];
    g.process_case_solves(&mut evs);
    assert!(is_solved(&g, case), "solved state persists");
}

/// CR 702.2c — a deathtouch source dealing any nonzero damage is lethal. When
/// several of your creatures each ping a target, the deathtouch pinger's damage
/// is enough to destroy it even though its share is only 1.
#[test]
fn cr_702_2c_multi_source_ping_deathtouch_is_lethal() {
    let mut g = two_player_game();
    // A 0/5 wall survives 2 plain damage, but a deathtouch ping kills it.
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_omens()); // 0/4
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, plain
    let rats = g.add_card_to_battlefield(0, catalog::typhoid_rats()); // 1/1 deathtouch
    let effect = crabomination::effect::Effect::EachControlledCreatureDealsDamage {
        to: crabomination::effect::shortcut::target_filtered(
            crabomination::card::SelectionRequirement::Creature,
        ),
        amount: crabomination::effect::Value::ONE,
    };
    let ctx = EffectContext::for_ability(rats, 0, Some(Target::Permanent(wall)));
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(wall).is_none(), "deathtouch ping among the group is lethal");
}

/// CR 613.7c/d — a +1/+1 anthem (layer 7c) and a +1/+1 counter (layer 7d) both
/// apply to a creature's power and toughness.
#[test]
fn cr_613_7_anthem_and_counter_both_apply() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_battlefield(0, catalog::gaeas_anthem()); // +1/+1 to your creatures
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "2/2 + anthem + counter = 4/4");
}

/// CR 719.3c — the "Solved —" abilities are dead until the Case is solved,
/// and switch on the moment it is.
#[test]
fn cr_719_3c_solved_abilities_are_dead_until_solved() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
    let printed = |g: &crabomination::game::GameState| {
        g.battlefield_find(case).unwrap().definition.triggered_abilities.len()
    };
    assert_eq!(printed(&g), 1, "only the always-on ETB line is live");
    g.active_player_idx = 0;
    let mut evs = vec![];
    g.process_case_solves(&mut evs);
    drain_stack(&mut g);
    assert!(is_solved(&g, case));
    assert_eq!(printed(&g), 2, "the Solved upkeep trigger switched on");
}

/// CR 719.3b — the solved designation is battlefield-only: a Case that leaves
/// and comes back is unsolved again, with its Solved abilities off.
#[test]
fn cr_719_3b_solved_is_lost_when_the_case_leaves_the_battlefield() {
    let mut g = two_player_game();
    let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
    g.active_player_idx = 0;
    let mut evs = vec![];
    g.process_case_solves(&mut evs);
    drain_stack(&mut g);
    assert!(is_solved(&g, case));

    g.remove_to_graveyard_with_triggers(case);
    drain_stack(&mut g);
    let gy = g.players[0].graveyard.iter().find(|c| c.id == case).expect("in the graveyard");
    assert!(!gy.case_solved, "the designation didn't survive the zone change");
    assert_eq!(gy.definition.triggered_abilities.len(), 1, "Solved abilities came back off");
}

// ── CR 730 — Merging with Permanents (Mutate) ────────────────────────────────

/// CR 730.2a — a merged permanent has the *topmost* component's
/// characteristics, with every component's abilities unioned in.
#[test]
fn cr_730_2a_merged_permanent_takes_the_topmost_characteristics() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion()); // 3/2 Trample
    let recluse = g.add_card_to_hand(0, catalog::glowstone_recluse()); // 2/3 Reach
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(crabomination::game::types::GameAction::CastMutate {
        card_id: recluse,
        target: host,
        on_top: true,
        x_value: None,
    })
    .expect("cast for mutate");
    drain_stack(&mut g);

    let merged = g.battlefield_find(host).expect("the host is still the permanent");
    assert_eq!(merged.definition.name, "Glowstone Recluse", "top card names the pile");
    let cp = g.computed_permanent(host).unwrap();
    // 2/3 from the top card, +2/+2 from its own on-mutate counters.
    assert_eq!((cp.power, cp.toughness), (4, 5));
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Reach), "top card's keyword");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Trample), "component's keyword");
}

/// CR 730.2b — merging isn't entering the battlefield: an "another creature
/// enters" watcher doesn't fire for the merge.
#[test]
fn cr_730_2b_merging_is_not_entering_the_battlefield() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::soul_warden());
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let recluse = g.add_card_to_hand(0, catalog::glowstone_recluse());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;
    g.perform_action(crabomination::game::types::GameAction::CastMutate {
        card_id: recluse,
        target: host,
        on_top: true,
        x_value: None,
    })
    .expect("cast for mutate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "no creature entered");
}

/// CR 730.2d — a merged permanent is a token only if the topmost component is
/// a token. A real card mutated onto a token makes the pile a nontoken.
#[test]
fn cr_730_2d_token_status_follows_the_topmost_component() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::garruks_companion());
    g.battlefield_find_mut(host).unwrap().is_token = true;
    let recluse = g.add_card_to_hand(0, catalog::glowstone_recluse());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(crabomination::game::types::GameAction::CastMutate {
        card_id: recluse,
        target: host,
        on_top: true,
        x_value: None,
    })
    .expect("cast for mutate");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(host).unwrap().is_token, "the nontoken top card rules the pile");
}
