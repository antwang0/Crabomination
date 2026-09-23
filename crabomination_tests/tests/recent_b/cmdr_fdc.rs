//! Commander: the Keen Engineering precon's missing cards (FDC, Sai, Master
//! Thopterist — `decks::cmdr_sai`).

use crabomination::card::{CardDefinition, CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn etb(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield_entering(0, def);
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
    id
}

fn try_cast(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    try_cast(g, 0, id, targets).expect("cast");
}

/// Declare `attacks` for the active seat, then no blocks, to end of combat.
/// `mid` runs with priority on `responder` after the declaration.
fn combat(
    g: &mut GameState,
    attacks: Vec<Attack>,
    responder: usize,
    mid: impl FnOnce(&mut GameState),
) {
    for a in &attacks {
        g.clear_sickness(a.attacker);
    }
    let active = g.active_player_idx;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = active;
    g.declare_attackers(attacks).expect("attack");
    drain_stack(g);
    g.priority.player_with_priority = responder;
    mid(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

// ── Keen Engineering (Sai, Master Thopterist) ───────────────────────────────

fn activate(g: &mut GameState, id: CardId, index: usize, x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// +1/+1 per artifact you control, and an attack digs six for an artifact.
#[test]
fn adaptive_omnitool_scales_and_digs_for_an_artifact() {
    let mut g = main_phase();
    let tool = g.add_card_to_battlefield(0, catalog::adaptive_omnitool());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bears);
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: tool, target: bears }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bears), (4, 4), "two artifacts");
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let ring = g.add_card_to_library(0, catalog::sol_ring());
    g.add_card_to_library(0, catalog::forest());
    combat(&mut g, vec![Attack { attacker: bears, target: AttackTarget::Player(1) }], 0, |_| {});
    assert!(g.players[0].hand.iter().any(|c| c.id == ring), "the artifact came up");
}

#[test]
fn darksteel_juggernaut_counts_your_artifacts() {
    let mut g = main_phase();
    let jugg = g.add_card_to_battlefield(0, catalog::darksteel_juggernaut());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(1, catalog::sol_ring());
    assert_eq!(pt(&g, jugg), (2, 2), "itself and the Ring");
}

/// CR 724 — the enchanted creature untaps only while its controller is the
/// monarch; the Aura makes its caster the monarch.
#[test]
fn cr_724_fall_from_favor_locks_until_its_victim_takes_the_crown() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
    let aura = g.add_card_to_hand(0, catalog::fall_from_favor());
    cast(&mut g, aura, &[Target::Permanent(victim)]);
    assert!(g.battlefield_find(victim).unwrap().tapped);
    assert_eq!(g.monarch, Some(0));
    let untap = |g: &mut GameState| {
        g.active_player_idx = 1;
        g.do_untap();
    };
    untap(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped, "not the monarch");
    g.monarch = Some(1);
    untap(&mut g);
    assert!(!g.battlefield_find(victim).unwrap().tapped, "the monarch untaps");
}

/// Ruling 2021-11-19 — one additional {C} per colorless tap however much it
/// made, nothing for a colored tap; colorless creatures +2/+2; 2 life a
/// colorless spell.
#[test]
fn forsaken_monument_adds_one_colorless_and_rewards_colorless_spells() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::forsaken_monument());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let jugg = g.add_card_to_battlefield(0, catalog::darksteel_juggernaut());
    assert_eq!(pt(&g, jugg), (5, 5), "three artifacts, plus two");
    g.perform_action(GameAction::ActivateAbility {
        card_id: ring,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap the Ring");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap the Forest");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 3);
    let life = g.players[0].life;
    let signet = g.add_card_to_hand(0, catalog::mind_stone());
    cast(&mut g, signet, &[]);
    assert_eq!(g.players[0].life, life + 2);
}

#[test]
fn foundry_and_launch_mishap_make_thopters() {
    let mut g = main_phase();
    let foundry = g.add_card_to_battlefield(0, catalog::foundry_of_the_consuls());
    activate(&mut g, foundry, 1, None).expect("sac for Thopters");
    assert_eq!(count_named(&g, 0, "Thopter"), 2);
    assert!(g.battlefield_find(foundry).is_none());

    // Seat 1 casts a creature; seat 0 answers with Launch Mishap.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    flood(&mut g, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bears");
    g.priority.player_with_priority = 0;
    let mishap = g.add_card_to_hand(0, catalog::launch_mishap());
    cast(&mut g, mishap, &[Target::Permanent(bears)]);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears), "countered");
    assert_eq!(count_named(&g, 0, "Thopter"), 3);
}

/// The {U}, {T}, return-an-artifact cost pays for a free artifact from hand.
#[test]
fn master_transmuter_swaps_an_artifact_in_from_hand() {
    let mut g = main_phase();
    let mt = g.add_card_to_battlefield(0, catalog::master_transmuter());
    g.clear_sickness(mt);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let jugg = g.add_card_to_hand(0, catalog::darksteel_juggernaut());
    activate(&mut g, mt, 0, None).expect("activate");
    assert!(g.battlefield_find(jugg).is_some(), "the Juggernaut went in");
    assert!(g.players[0].hand.iter().any(|c| c.id == ring) || g.battlefield_find(ring).is_some());
}

/// Rulings 2021-11-19 — flashed in during declare attackers, it points an
/// attacker at another legal defender; outside that step it doesn't trigger.
/// At three seats the headless chooser sends it at its hostile opponent.
#[test]
fn misleading_signpost_redirects_an_attacker_during_declare_attackers() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 1;
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.clear_sickness(giant);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: giant, target: AttackTarget::Player(0) }])
        .expect("attack");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    let post = g.add_card_to_hand(0, catalog::misleading_signpost());
    try_cast(&mut g, 0, post, &[]).expect("flash");
    assert_eq!(g.attacking[0].target, AttackTarget::Player(2));

    // A main-phase Signpost is only a mana rock.
    let mut g = main_phase();
    let post = g.add_card_to_hand(0, catalog::misleading_signpost());
    cast(&mut g, post, &[]);
    assert!(g.battlefield_find(post).is_some());
}

#[test]
fn research_thief_draws_for_artifact_creatures_that_connect() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::research_thief());
    let jugg = g.add_card_to_battlefield(0, catalog::darksteel_juggernaut());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand = g.players[0].hand.len();
    combat(
        &mut g,
        vec![
            Attack { attacker: jugg, target: AttackTarget::Player(1) },
            Attack { attacker: bears, target: AttackTarget::Player(1) },
        ],
        0,
        |_| {},
    );
    assert_eq!(g.players[0].hand.len(), hand + 1, "the Juggernaut, not the Bears");
}

#[test]
fn shimmer_dragon_hexproof_at_four_artifacts_and_taps_two_to_draw() {
    let mut g = main_phase();
    let dragon = g.add_card_to_battlefield(0, catalog::shimmer_dragon());
    let hexproof = |g: &GameState| g.computed_permanent(dragon).unwrap().keywords().contains(&Keyword::Hexproof);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::sol_ring());
    }
    assert!(!hexproof(&g));
    g.add_card_to_battlefield(0, catalog::sol_ring());
    assert!(hexproof(&g));
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    activate(&mut g, dragon, 0, None).expect("tap two artifacts");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Sol Ring" && c.tapped).count(), 2);
}

#[test]
fn skysovereign_shoots_an_opposing_creature_on_entry() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    etb(&mut g, catalog::skysovereign_consul_flagship());
    assert!(g.battlefield_find(giant).is_none(), "3 damage kills the 3/3");
}

/// Rulings 2021-03-19 — {X} destroys each nonland permanent with mana value
/// exactly X controlled by a player the Hellkite dealt combat damage to this
/// turn; the other seat's permanents and lands are untouched.
#[test]
fn steel_hellkite_sweeps_mana_value_x_from_the_players_it_hit() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    let kite = g.add_card_to_battlefield(0, catalog::steel_hellkite());
    let hit_bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hit_giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let hit_land = g.add_card_to_battlefield(1, catalog::forest());
    let safe_bears = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    combat(&mut g, vec![Attack { attacker: kite, target: AttackTarget::Player(1) }], 0, |_| {});
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    activate(&mut g, kite, 1, Some(2)).expect("X = 2");
    assert!(g.battlefield_find(hit_bears).is_none());
    assert!(g.battlefield_find(hit_giant).is_some(), "mana value 4");
    assert!(g.battlefield_find(hit_land).is_some(), "a land");
    assert!(g.battlefield_find(safe_bears).is_some(), "seat 2 wasn't hit");
    assert!(activate(&mut g, kite, 1, Some(4)).is_err(), "once each turn");
}
