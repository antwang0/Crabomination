//! Commander: precon batches — Keen Engineering (FDC, Sai, `decks::cmdr_sai`)
//! and Reap the Tides (CMR, Aesi, `decks::cmdr_aesi`).

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword};
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
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
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

// ── Reap the Tides (Aesi, Tyrant of Gyre Strait) ────────────────────────────

/// CR 702.119a — emerge: sacrifice a creature and pay the emerge cost less its
/// mana value; the cast trigger still taps.
#[test]
fn cr_702_119a_elder_deep_fiend_emerges_off_a_sacrifice_and_taps() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let fiend = g.add_card_to_hand(0, catalog::elder_deep_fiend());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: fiend,
        pitch_card: Some(giant),
        target: Some(Target::Permanent(enemy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("emerge for {1}{U}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "the Giant paid for it");
    assert!(g.battlefield_find(fiend).is_some());
    assert!(g.battlefield_find(enemy).unwrap().tapped, "the cast trigger tapped it");
}

#[test]
fn memorial_to_genius_enters_tapped_and_cashes_in_for_two() {
    let mut g = main_phase();
    let m = etb(&mut g, catalog::memorial_to_genius());
    assert!(g.battlefield_find(m).unwrap().tapped);
    g.battlefield.find_by_id_mut(m).unwrap().tapped = false;
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand = g.players[0].hand.len();
    activate(&mut g, m, 1, None).expect("sac for cards");
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Green and blue each pump; a green-and-blue creature gets both. Green/blue
/// creatures untap on another player's untap step (CR 502.3).
#[test]
fn murkfiend_liege_pumps_simic_and_untaps_on_other_turns() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::murkfiend_liege());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let piker = g.add_card_to_battlefield(0, catalog::goblin_piker());
    let ooze = g.add_card_to_battlefield(0, catalog::murkfiend_liege());
    assert_eq!(pt(&g, bears), (4, 4), "one anthem from each Liege");
    assert_eq!(pt(&g, piker), (2, 1), "red, no pump");
    assert_eq!(pt(&g, ooze), (6, 6), "the other Liege's two anthems");
    for id in [bears, piker] {
        g.battlefield.find_by_id_mut(id).unwrap().tapped = true;
    }
    g.active_player_idx = 1;
    g.do_untap();
    assert!(!g.battlefield_find(bears).unwrap().tapped);
    assert!(g.battlefield_find(piker).unwrap().tapped);
}

/// A card per opposing noncreature spell; discard three to blink it, back
/// tapped at the next end step.
#[test]
fn nezahal_draws_off_opposing_spells_and_blinks_for_three_cards() {
    let mut g = main_phase();
    let nez = g.add_card_to_battlefield(0, catalog::nezahal_primal_tide());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let hand = g.players[0].hand.len();
    try_cast(&mut g, 1, bolt, &[Target::Player(0)]).expect("bolt");
    assert_eq!(g.players[0].hand.len(), hand + 1);

    let mut g = main_phase();
    let nez2 = g.add_card_to_battlefield(0, catalog::nezahal_primal_tide());
    let _ = nez;
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::forest());
    }
    activate(&mut g, nez2, 0, None).expect("discard three");
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Nezahal, Primal Tide"), "exiled");
    g.step = TurnStep::PostCombatMain;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Nezahal, Primal Tide").expect("back");
    assert!(back.tapped);
}

/// Kicked, every creature but Merfolk, Krakens, Leviathans, Octopuses and
/// Serpents goes home — Slinn Voda itself stays.
#[test]
fn slinn_voda_kicked_bounces_all_but_the_sea() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let kraken = g.add_card_to_battlefield(1, catalog::trench_behemoth());
    let voda = g.add_card_to_hand(0, catalog::slinn_voda_the_rising_deep());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: voda,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bears));
    assert!(g.battlefield_find(kraken).is_some() && g.battlefield_find(voda).is_some());
}

#[test]
fn sphinx_of_uthuun_splits_five_between_hand_and_graveyard() {
    let mut g = main_phase();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let (hand, gy) = (g.players[0].hand.len(), g.players[0].graveyard.len());
    etb(&mut g, catalog::sphinx_of_uthuun());
    assert_eq!(g.players[0].hand.len() - hand + g.players[0].graveyard.len() - gy, 5);
    assert!(g.players[0].hand.len() > hand);
}

#[test]
fn spitting_image_copies_a_creature() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::spitting_image());
    cast(&mut g, spell, &[Target::Permanent(giant)]);
    assert_eq!(count_named(&g, 0, "Hill Giant"), 1);
}

/// With no commander of yours in play every counter stays on the Hydra; with
/// one, the headless seat spreads them.
#[test]
fn stumpsquall_hydra_shares_x_counters_with_your_commander() {
    let mut g = main_phase();
    let hydra = g.add_card_to_hand(0, catalog::stumpsquall_hydra());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: hydra,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    })
    .expect("X = 4");
    drain_stack(&mut g);
    assert_eq!(pt(&g, hydra), (5, 5));

    let mut g = main_phase();
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast the commander");
    drain_stack(&mut g);
    let hydra = g.add_card_to_hand(0, catalog::stumpsquall_hydra());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: hydra,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    })
    .expect("X = 4");
    drain_stack(&mut g);
    let on = |id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(on(hydra) + on(cmd), 4);
    assert!(on(cmd) > 0, "the commander got a share");
}

/// Return a land: untap and hexproof. Landfall: an opposing creature must
/// attack on its controller's next turn.
#[test]
fn trench_behemoth_untaps_for_a_land_and_forces_an_attack() {
    let mut g = main_phase();
    let tb = g.add_card_to_battlefield(0, catalog::trench_behemoth());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield.find_by_id_mut(tb).unwrap().tapped = true;
    activate(&mut g, tb, 0, None).expect("return a land");
    assert!(g.players[0].hand.iter().any(|c| c.id == forest));
    assert!(!g.battlefield_find(tb).unwrap().tapped);
    assert!(g.computed_permanent(tb).unwrap().keywords().contains(&Keyword::Hexproof));
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::PlayLand(forest)).expect("landfall");
    drain_stack(&mut g);
    assert!(g.computed_permanent(enemy).unwrap().keywords().contains(&Keyword::MustAttack));
}

/// Two charge counters, one spent per off-color mana; none left, no ability.
#[test]
fn vivid_lands_spend_charge_counters_for_any_color() {
    let mut g = main_phase();
    let creek = g.add_card_to_hand(0, catalog::vivid_creek());
    g.perform_action(GameAction::PlayLand(creek)).expect("play it");
    drain_stack(&mut g);
    let land = g.battlefield_find(creek).unwrap();
    assert!(land.tapped);
    assert_eq!(land.counter_count(CounterType::Charge), 2);
    for n in [1, 0] {
        g.battlefield.find_by_id_mut(creek).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: creek,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("any color");
        assert_eq!(g.battlefield_find(creek).unwrap().counter_count(CounterType::Charge), n);
    }
    g.battlefield.find_by_id_mut(creek).unwrap().tapped = false;
    assert!(g
        .perform_action(GameAction::ActivateAbility {
            card_id: creek,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err());
}

// ── Corrupting Influence (Ixhel, Scion of Atraxa) ───────────────────────────

/// CR 122.1f / 704.5c — poison counters on every player, and proliferate
/// adds one more of each kind already there.
#[test]
fn ichor_rats_poisons_everyone_and_glistening_sphere_proliferates() {
    let mut g = main_phase();
    etb(&mut g, catalog::ichor_rats());
    assert_eq!((g.players[0].poison_counters, g.players[1].poison_counters), (1, 1));
    let sphere = etb(&mut g, catalog::glistening_sphere());
    assert!(g.battlefield_find(sphere).unwrap().tapped);
    assert!(g.players[1].poison_counters >= 2, "proliferate hit the opponent");
}

/// CR 702.166 — corrupted gates the three-mana ability on an opponent at
/// three poison.
#[test]
fn cr_702_166_glistening_sphere_corrupted_mana_needs_three_poison() {
    let mut g = main_phase();
    let sphere = g.add_card_to_battlefield(0, catalog::glistening_sphere());
    let tap3 = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: sphere,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(tap3(&mut g).is_err());
    g.players[1].poison_counters = 3;
    tap3(&mut g).expect("corrupted");
    let pool = &g.players[0].mana_pool;
    let total: u32 = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
        .iter()
        .map(|c| pool.amount(*c))
        .sum();
    assert_eq!(total, 3);
}

/// CR 903.3 — a commander of yours attacking proliferates; another creature
/// attacking does not.
#[test]
fn cr_903_3_norns_choirmaster_proliferates_on_commander_attacks() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::norns_choirmaster());
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast the commander");
    drain_stack(&mut g);
    g.players[1].poison_counters = 1;
    let after_entry = g.players[1].poison_counters;
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    combat(&mut g, vec![Attack { attacker: giant, target: AttackTarget::Player(1) }], 0, |_| {});
    assert_eq!(g.players[1].poison_counters, after_entry, "not a commander");
    g.step = TurnStep::PreCombatMain;
    combat(&mut g, vec![Attack { attacker: cmd, target: AttackTarget::Player(1) }], 0, |_| {});
    assert_eq!(g.players[1].poison_counters, after_entry + 1);
}

/// Each creature shrinks by ITS controller's poison count, not the table's.
#[test]
fn phyresis_outbreak_shrinks_by_each_controllers_poison() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[1].poison_counters = 2;
    let a = g.add_card_to_battlefield(1, catalog::hill_giant());
    let b = g.add_card_to_battlefield(2, catalog::hill_giant());
    let mine = g.add_card_to_battlefield(0, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::phyresis_outbreak());
    cast(&mut g, spell, &[]);
    assert!(g.battlefield_find(a).is_none(), "3 poison: -3/-3 kills the 3/3");
    assert_eq!(pt(&g, b), (2, 2), "1 poison");
    assert_eq!(pt(&g, mine), (3, 3));
}

/// CR 122.1f — every opponent's poison counts, summed across the table.
#[test]
fn vishgraz_grows_with_the_tables_poison_and_makes_mites() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.players[1].poison_counters = 2;
    g.players[2].poison_counters = 3;
    g.players[0].poison_counters = 5;
    let v = etb(&mut g, catalog::vishgraz_the_doomhive());
    assert_eq!(pt(&g, v), (8, 8), "2 + 3, not its own 5");
    assert_eq!(count_named(&g, 0, "Phyrexian Mite"), 3);
}

/// CR 603.2c — "to one or more players" is one batch however many seats
/// were hit: two attackers at two players proliferate once.
#[test]
fn cr_603_2c_contaminant_grafter_proliferates_once_across_players() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::contaminant_grafter());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[1].poison_counters = 1;
    g.players[2].poison_counters = 1;
    combat(
        &mut g,
        vec![
            Attack { attacker: a, target: AttackTarget::Player(1) },
            Attack { attacker: b, target: AttackTarget::Player(2) },
        ],
        0,
        |_| {},
    );
    assert_eq!((g.players[1].poison_counters, g.players[2].poison_counters), (2, 2), "one proliferate");
}

/// CR 702.166 — corrupted: one creature from your graveyard, plus one from
/// each opponent at three or more poison (the other opponent's is left).
#[test]
fn cr_702_166_geths_summons_takes_from_each_corrupted_opponent() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let mine = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_graveyard(1, catalog::hill_giant());
    let safe = g.add_card_to_graveyard(2, catalog::hill_giant());
    g.players[1].poison_counters = 3;
    let spell = g.add_card_to_hand(0, catalog::geths_summons());
    cast(&mut g, spell, &[]);
    assert_eq!(g.battlefield_find(mine).map(|c| c.controller), Some(0));
    assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(0));
    assert!(g.battlefield_find(safe).is_none());
}

/// CR 702.166 — dying while an opponent is corrupted exiles it and rebuys a
/// card per corrupted opponent.
#[test]
fn cr_702_166_glissas_retriever_rebuys_per_corrupted_opponent() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let r = g.add_card_to_battlefield(0, catalog::glissas_retriever());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.players[1].poison_counters = 3;
    g.players[2].poison_counters = 4;
    let hand = g.players[0].hand.len();
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, &[Target::Permanent(r)]);
    assert!(g.exile.iter().any(|c| c.id == r), "exiled, not in the graveyard");
    assert_eq!(g.players[0].hand.len(), hand + 2, "two corrupted opponents");
}

/// X is the mana spent; each corrupted opponent adds another Wurm.
#[test]
fn wurmquake_makes_x_x_wurms_per_corrupted_opponent() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[1].poison_counters = 3;
    let spell = g.add_card_to_hand(0, catalog::wurmquake());
    g.players[0].mana_pool.add(Color::Green, 6);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast for six");
    drain_stack(&mut g);
    let wurms: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Wurm").map(|c| c.id).collect();
    assert_eq!(wurms.len(), 2);
    assert!(wurms.iter().all(|&w| pt(&g, w) == (6, 6)));
}

/// CR 702.166 — at your end step each corrupted opponent exiles their top
/// card, which you may cast spending mana as though it were any color; an
/// opponent below three poison exiles nothing.
#[test]
fn cr_702_166_ixhel_exiles_from_corrupted_opponents_and_casts_with_any_mana() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.add_card_to_battlefield(0, catalog::ixhel_scion_of_atraxa());
    let bears = g.add_card_to_library(1, catalog::goblin_piker());
    let kept = g.add_card_to_library(2, catalog::goblin_piker());
    g.players[1].poison_counters = 3;
    g.step = TurnStep::PostCombatMain;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bears));
    assert!(g.players[2].library.iter().any(|c| c.id == kept), "not corrupted");
    // Next main phase: cast the red Piker with green mana.
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from exile with any-color mana");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bears).map(|c| c.controller), Some(0));
}

/// An opponent whose creatures hit you gets one poison counter per batch;
/// attacking a poisoned player draws the attacker a card (CR 506.2).
#[test]
fn norns_decree_poisons_attackers_and_rewards_attacking_the_poisoned() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::norns_decree());
    // Seat 1 attacks seat 0 with two creatures: one poison counter.
    g.active_player_idx = 1;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::forest());
    }
    let hand1 = g.players[1].hand.len();
    combat(
        &mut g,
        vec![
            Attack { attacker: a, target: AttackTarget::Player(0) },
            Attack { attacker: b, target: AttackTarget::Player(0) },
        ],
        1,
        |_| {},
    );
    assert_eq!(g.players[1].poison_counters, 1);
    assert_eq!(g.players[1].hand.len(), hand1, "seat 0 wasn't poisoned");
    // Seat 0 attacks the now-poisoned seat 1 and draws.
    g.active_player_idx = 0;
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    combat(&mut g, vec![Attack { attacker: giant, target: AttackTarget::Player(1) }], 0, |_| {});
    assert_eq!(g.players[0].hand.len(), hand0 + 1);
}

/// Rulings 2023-02-04 — the Grafted trigger: moving the Equipment to another
/// creature, or the Equipment leaving, sacrifices the old host; the host
/// leaving on its own does nothing more.
#[test]
fn grafted_equipment_sacrifices_the_host_it_leaves() {
    let mut g = main_phase();
    let exo = g.add_card_to_battlefield(0, catalog::grafted_exoskeleton());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::hill_giant());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: exo, target: a }).expect("equip a");
    drain_stack(&mut g);
    assert_eq!(pt(&g, a), (4, 4));
    assert!(g.computed_permanent(a).unwrap().keywords().contains(&Keyword::Infect));
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: exo, target: b }).expect("move to b");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none(), "the old host was sacrificed");
    assert!(g.battlefield_find(b).is_some());
    // The Equipment leaving takes its host with it.
    let shatter = g.add_card_to_hand(0, catalog::shatter());
    cast(&mut g, shatter, &[Target::Permanent(exo)]);
    assert!(g.battlefield_find(b).is_none(), "the host was sacrificed as the Equipment left");

    // Grafted Wargear shares the rider.
    let mut g = main_phase();
    let wg = g.add_card_to_battlefield(0, catalog::grafted_wargear());
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let d = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::Equip { equipment: wg, target: c }).expect("equip");
    drain_stack(&mut g);
    g.perform_action(GameAction::Equip { equipment: wg, target: d }).expect("re-equip");
    drain_stack(&mut g);
    assert!(g.battlefield_find(c).is_none());
}

// ── Sneak Attack (Anowon, the Ruin Thief) ───────────────────────────────────

/// CR 603.2c — one fire per damaged player, milling the batch's TOTAL Rogue
/// damage, and one card however many creatures were milled.
#[test]
fn cr_603_2c_anowon_mills_the_batch_total_and_draws_once() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::anowon_the_ruin_thief());
    let a = g.add_card_to_battlefield(0, catalog::nightveil_sprite());
    let b = g.add_card_to_battlefield(0, catalog::nightveil_sprite());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..8 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    combat(
        &mut g,
        vec![
            Attack { attacker: a, target: AttackTarget::Player(1) },
            Attack { attacker: b, target: AttackTarget::Player(1) },
            Attack { attacker: bear, target: AttackTarget::Player(1) },
        ],
        0,
        |_| {},
    );
    // Two pumped Sprites (2 each) are Rogues; the Bears (2) are not.
    assert_eq!(g.players[1].graveyard.len(), 4, "milled 2 + 2");
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw for the batch");
}

/// CR 601.2c — for each opponent, up to one of their nonland permanents.
#[test]
fn enigma_thief_bounces_one_permanent_from_each_opponent() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    let x = g.add_card_to_battlefield(1, catalog::sol_ring());
    let y = g.add_card_to_battlefield(2, catalog::hill_giant());
    etb(&mut g, catalog::enigma_thief());
    assert!(g.players[1].hand.iter().any(|c| c.id == x));
    assert!(g.players[2].hand.iter().any(|c| c.id == y));
}

/// Cast from the graveyard only while a black or green permanent is yours.
#[test]
fn marang_river_prowler_recasts_from_the_graveyard_with_a_black_permanent() {
    let mut g = main_phase();
    let p = g.add_card_to_graveyard(0, catalog::marang_river_prowler());
    let recast = |g: &mut GameState| {
        flood(g, 0);
        g.perform_action(GameAction::CastFlashback {
            card_id: p,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(recast(&mut g).is_err(), "no black or green permanent");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    recast(&mut g).expect("a green permanent");
    drain_stack(&mut g);
    assert!(g.battlefield_find(p).is_some());
}

#[test]
fn master_thief_steals_an_artifact() {
    let mut g = main_phase();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let thief = g.add_card_to_hand(0, catalog::master_thief());
    cast(&mut g, thief, &[]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ring).map(|c| c.controller), Some(0));
}

/// "An opponent has eight or more" is any opponent, not the first one
/// (CR 102.2): seat 2's graveyard enables the sacrifice while seat 1's is empty.
#[test]
fn cr_102_2_merfolk_windrobber_reads_any_opponents_graveyard() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let w = g.add_card_to_battlefield(0, catalog::merfolk_windrobber());
    g.add_card_to_library(0, catalog::island());
    assert!(activate(&mut g, w, 0, None).is_err());
    for _ in 0..8 {
        g.add_card_to_graveyard(2, catalog::island());
    }
    activate(&mut g, w, 0, None).expect("seat 2 has eight");
    assert!(g.battlefield_find(w).is_none());
}

/// X target creatures become unblockable and draw on a hit this turn.
#[test]
fn open_into_wonder_makes_x_creatures_unblockable_card_drawers() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::hill_giant());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let spell = g.add_card_to_hand(0, catalog::open_into_wonder());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("X = 2");
    drain_stack(&mut g);
    assert!(g.computed_permanent(b).unwrap().keywords().contains(&Keyword::Unblockable));
    let hand = g.players[0].hand.len();
    combat(
        &mut g,
        vec![
            Attack { attacker: a, target: AttackTarget::Player(1) },
            Attack { attacker: b, target: AttackTarget::Player(1) },
        ],
        0,
        |_| {},
    );
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// {2} cheaper when the target is legendary.
#[test]
fn price_of_fame_is_cheaper_on_a_legend() {
    let mut g = main_phase();
    let legend = g.add_card_to_battlefield(1, catalog::anowon_the_ruin_thief());
    let spell = g.add_card_to_hand(0, catalog::price_of_fame());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(legend)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{1}{B} against a legend");
    drain_stack(&mut g);
    assert!(g.battlefield_find(legend).is_none());
}

/// A creature from ANY graveyard, under your control, a black Zombie too.
#[test]
fn rise_from_the_grave_reanimates_as_a_black_zombie() {
    let mut g = main_phase();
    let giant = g.add_card_to_graveyard(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::rise_from_the_grave());
    cast(&mut g, spell, &[Target::Permanent(giant)]);
    let cp = g.computed_permanent(giant).expect("on the battlefield");
    assert_eq!(g.battlefield_find(giant).unwrap().controller, 0);
    assert!(cp.subtypes().creature_types.contains(&crabomination::card::CreatureType::Zombie));
    assert!(cp.colors.contains(Color::Black) && cp.colors.contains(Color::Red));
}

/// A hit halves the player's life, rounded up.
#[test]
fn scytheclaw_halves_the_life_of_the_player_it_hits() {
    let mut g = main_phase();
    etb(&mut g, catalog::scytheclaw());
    let germ = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Germ").unwrap().id;
    assert_eq!(pt(&g, germ), (1, 1));
    g.players[1].life = 40;
    combat(&mut g, vec![Attack { attacker: germ, target: AttackTarget::Player(1) }], 0, |_| {});
    assert_eq!(g.players[1].life, 19, "40 - 1 = 39, then 20 of it");
}

/// Rogues attacking mill each opponent two, once per declaration.
#[test]
fn soaring_thought_thief_mills_once_per_rogue_attack() {
    let mut g = main_phase();
    let t = g.add_card_to_battlefield(0, catalog::soaring_thought_thief());
    let s = g.add_card_to_battlefield(0, catalog::nightveil_sprite());
    for _ in 0..10 {
        g.add_card_to_library(1, catalog::island());
    }
    g.add_card_to_library(0, catalog::island());
    combat(
        &mut g,
        vec![
            Attack { attacker: t, target: AttackTarget::Player(1) },
            Attack { attacker: s, target: AttackTarget::Player(1) },
        ],
        0,
        |_| {},
    );
    assert_eq!(g.players[1].graveyard.len(), 2);
    for _ in 0..6 {
        g.add_card_to_graveyard(1, catalog::island());
    }
    assert_eq!(pt(&g, s).0, 2, "an opponent at eight: Rogues +1/+0");
}

/// Choose one or both (CR 700.2) — both modes, two targets.
#[test]
fn soul_manipulation_counters_and_rebuys() {
    let mut g = main_phase();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
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
    let sm = g.add_card_to_hand(0, catalog::soul_manipulation());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: sm,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![Target::Permanent(dead)],
        x_value: None,
    })
    .expect("both modes");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears), "countered");
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "rebought");
}

#[test]
fn sure_footed_infiltrator_taps_another_rogue_to_slip_through() {
    let mut g = main_phase();
    let inf = g.add_card_to_battlefield(0, catalog::sure_footed_infiltrator());
    assert!(activate(&mut g, inf, 0, None).is_err(), "no other Rogue");
    let s = g.add_card_to_battlefield(0, catalog::nightveil_sprite());
    activate(&mut g, inf, 0, None).expect("tap the Sprite");
    assert!(g.battlefield_find(s).unwrap().tapped);
    assert!(g.computed_permanent(inf).unwrap().keywords().contains(&Keyword::Unblockable));
}

/// A hit opens that player's graveyard for a creature spell this turn, any
/// color of mana.
#[test]
fn whispersteel_dagger_casts_from_the_hit_players_graveyard() {
    let mut g = main_phase();
    let dagger = g.add_card_to_battlefield(0, catalog::whispersteel_dagger());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: dagger, target: bears }).expect("equip");
    drain_stack(&mut g);
    let piker = g.add_card_to_graveyard(1, catalog::goblin_piker());
    combat(&mut g, vec![Attack { attacker: bears, target: AttackTarget::Player(1) }], 0, |_| {});
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: piker,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Piker with black mana");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(piker).map(|c| c.controller), Some(0));
}

/// CR 611.2c — "for as long as you control" ends when the Thief changes
/// hands, not only when it leaves.
#[test]
fn cr_611_2c_master_thiefs_steal_ends_when_it_changes_hands() {
    let mut g = main_phase();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let thief = g.add_card_to_hand(0, catalog::master_thief());
    cast(&mut g, thief, &[]);
    assert_eq!(g.battlefield_find(ring).map(|c| c.controller), Some(0));
    let steal = g.add_card_to_hand(1, catalog::act_of_treason());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    try_cast(&mut g, 1, steal, &[Target::Permanent(thief)]).expect("Act of Treason on the Thief");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(thief).map(|c| c.controller), Some(1));
    assert_eq!(g.battlefield_find(ring).map(|c| c.controller), Some(1), "the Ring went home");
}

// ── Jeskai Striker (Shiko and Narset, Unified) ──────────────────────────────

/// CR 603.4 — counters stop at three; spending them copies the next instant.
#[test]
fn adaptive_training_post_banks_three_and_copies_the_next_instant() {
    let mut g = main_phase();
    let post = g.add_card_to_battlefield(0, catalog::adaptive_training_post());
    for _ in 0..4 {
        let s = g.add_card_to_hand(0, catalog::opt());
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        cast(&mut g, s, &[]);
    }
    assert_eq!(g.battlefield_find(post).unwrap().counter_count(CounterType::Charge), 3);
    activate(&mut g, post, 0, None).expect("remove three");
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]);
    assert_eq!(g.players[1].life, life - 6, "copied");
}

/// Flurry: the second spell each turn adds a rally counter, then a Monk for
/// each counter.
#[test]
fn aligned_heart_makes_a_monk_per_rally_counter() {
    let mut g = main_phase();
    let heart = g.add_card_to_battlefield(0, catalog::aligned_heart());
    for _ in 0..2 {
        let s = g.add_card_to_hand(0, catalog::opt());
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        cast(&mut g, s, &[]);
    }
    assert_eq!(g.battlefield_find(heart).unwrap().counter_count(CounterType::Rally), 1);
    assert_eq!(count_named(&g, 0, "Monk"), 1);
}

/// The first instant each turn offers a free instant of lesser mana value
/// from hand; with none, First Mate Ragavan arrives hasty.
#[test]
fn baral_and_kari_zev_casts_a_lesser_instant_free_or_makes_ragavan() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::baral_and_kari_zev());
    let opt = g.add_card_to_hand(0, catalog::opt());
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
    }
    let fof = g.add_card_to_hand(0, catalog::fact_or_fiction());
    cast(&mut g, fof, &[]);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == opt), "Opt (MV 1 < 4) was cast free");
    assert_eq!(count_named(&g, 0, "First Mate Ragavan"), 0);

    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::baral_and_kari_zev());
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, &[Target::Player(1)]);
    let rag = g.battlefield.iter().find(|c| c.definition.name == "First Mate Ragavan").expect("Ragavan").id;
    assert!(g.computed_permanent(rag).unwrap().keywords().contains(&Keyword::Haste));
}

/// CR 506.2 — you get a Gold whenever the cursed player is attacked, and so
/// does an opponent who attacks them.
#[test]
fn curse_of_opulence_pays_the_curser_and_the_attacker() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let curse = g.add_card_to_hand(0, catalog::curse_of_opulence());
    cast(&mut g, curse, &[Target::Player(2)]);
    g.active_player_idx = 1;
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    combat(&mut g, vec![Attack { attacker: bears, target: AttackTarget::Player(2) }], 1, |_| {});
    assert_eq!(count_named(&g, 0, "Gold"), 1);
    assert_eq!(count_named(&g, 1, "Gold"), 1);
}

/// Cycling it destroys every artifact and enchantment.
#[test]
fn dismantling_wave_cycles_into_a_sweep() {
    let mut g = main_phase();
    let wave = g.add_card_to_hand(0, catalog::dismantling_wave());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let mine = g.add_card_to_battlefield(0, catalog::aligned_heart());
    g.add_card_to_library(0, catalog::island());
    flood(&mut g, 0);
    g.perform_action(GameAction::Cycle { card_id: wave, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none() && g.battlefield_find(mine).is_none());
}

/// Explosion: X damage to any target, and a target player draws X.
#[test]
fn explosion_deals_x_and_draws_x() {
    let mut g = main_phase();
    let card = g.add_card_to_hand(0, catalog::expansion_explosion());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    let life = g.players[1].life;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSplitRight {
        card_id: card,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Player(0)],
        mode: None,
        x_value: Some(2),
    })
    .expect("Explosion, X = 2");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2);
}

/// Goaded while enchanted, and a Treasure for the Aura's controller each time
/// the creature attacks.
#[test]
fn shiny_impetus_goads_and_pays_treasure() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::shiny_impetus());
    cast(&mut g, aura, &[Target::Permanent(victim)]);
    assert_eq!(pt(&g, victim), (4, 4));
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::BeginCombat);
    assert!(g.battlefield_find(victim).unwrap().cold_any(|k| k.goaded_by.contains(&0)), "goaded by seat 0");
    combat(&mut g, vec![Attack { attacker: victim, target: AttackTarget::Player(2) }], 1, |_| {});
    assert_eq!(count_named(&g, 0, "Treasure"), 1);
}

/// Cast with flash in response, it counters the spell into exile and casts
/// it for its controller... for you, free.
#[test]
fn transcendent_dragon_steals_the_spell_it_counters() {
    let mut g = main_phase();
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
    let dragon = g.add_card_to_hand(0, catalog::transcendent_dragon());
    cast(&mut g, dragon, &[]);
    assert_eq!(g.battlefield_find(bears).map(|c| c.controller), Some(0), "cast free by the Dragon's controller");
}

/// A card per target of each spell cast.
#[test]
fn voracious_bibliophile_draws_per_target() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::voracious_bibliophile());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// With a commander in play both modes run: flashback for instants and
/// sorceries in your graveyard.
#[test]
fn will_of_the_jeskai_grants_flashback_with_a_commander() {
    let mut g = main_phase();
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("commander");
    drain_stack(&mut g);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let will = g.add_card_to_hand(0, catalog::will_of_the_jeskai());
    cast(&mut g, will, &[]);
    let life = g.players[1].life;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFlashback {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flashback Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
}

// ── Sliver Swarm (Sliver Gravemother) ───────────────────────────────────────

use crabomination::card::CreatureType;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};

fn has_type(g: &GameState, id: CardId, t: CreatureType) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.subtypes().creature_types.contains(&t))
}

/// CR 704.5j — the legend rule sits out for your legendary Slivers only: two
/// Gravemothers both stay, two copies of a non-Sliver legend still collapse.
#[test]
fn cr_704_5j_gravemother_spares_legendary_slivers_only() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::sliver_gravemother());
    let b = g.add_card_to_battlefield(0, catalog::sliver_gravemother());
    let x = g.add_card_to_battlefield(0, catalog::anowon_the_ruin_thief());
    let y = g.add_card_to_battlefield(0, catalog::anowon_the_ruin_thief());
    g.check_state_based_actions();
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some());
    assert_eq!(
        [x, y].iter().filter(|id| g.battlefield_find(**id).is_some()).count(),
        1,
        "the non-Sliver legends still obey the rule"
    );
}

/// CR 702.141 — a Sliver creature card in your graveyard has encore {X}, X its
/// mana value: one hasty attacking token copy per opponent. A non-Sliver
/// creature card gets nothing.
#[test]
fn cr_702_141_gravemother_grants_encore_x_to_sliver_cards() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::sliver_gravemother());
    let s = g.add_card_to_graveyard(0, catalog::capricious_sliver());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    let short = g.perform_action(GameAction::ActivateAbility {
        card_id: s,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert!(short.is_err(), "X = 4, not 3");
    assert!(activate(&mut g, bear, 0, None).is_err(), "not a Sliver");
    activate(&mut g, s, 0, None).expect("encore {4}");
    assert_eq!(count_named(&g, 0, "Capricious Sliver"), 2, "one per opponent");
    assert!(g.exile.iter().any(|c| c.id == s));
}

/// Rukarumel — Slivers and nontoken creatures you control take the chosen
/// type; a non-Sliver token does not. {3}, {T} makes a Sliver.
#[test]
fn rukarumel_types_your_slivers_and_nontoken_creatures() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elf)]));
    let ruk = g.add_card_to_hand(0, catalog::rukarumel_biologist());
    cast(&mut g, ruk, &[]);
    assert!(has_type(&g, bear, CreatureType::Elf));
    assert!(!has_type(&g, theirs, CreatureType::Elf), "only yours");
    g.clear_sickness(ruk);
    activate(&mut g, ruk, 0, None).expect("{3}, {T}");
    let token = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.controller == 0)
        .map(|c| c.id)
        .expect("a Sliver token");
    assert!(has_type(&g, token, CreatureType::Sliver));
    assert!(has_type(&g, token, CreatureType::Elf), "a Sliver token is typed too");
}

/// Capricious Sliver — a connecting Sliver exiles your top card, playable.
#[test]
fn capricious_sliver_impulse_draws_on_a_hit() {
    let mut g = main_phase();
    let s = g.add_card_to_battlefield(0, catalog::capricious_sliver());
    let top = g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    combat(&mut g, vec![Attack { attacker: s, target: AttackTarget::Player(1) }], 0, |_| {});
    assert!(g.exile.iter().any(|c| c.id == top));
}

/// Descendants' Fury — sacrifice the connecting Sliver; reveal past the
/// non-Slivers to the next Sliver creature card, which enters; the misses
/// go to the bottom.
#[test]
fn descendants_fury_trades_a_connecting_sliver_for_the_next_one() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::descendants_fury());
    let s = g.add_card_to_battlefield(0, catalog::hollowhead_sliver());
    g.add_card_to_library(0, catalog::island());
    let miss = g.add_card_to_library(0, catalog::grizzly_bears());
    let hit = g.add_card_to_library(0, catalog::capricious_sliver());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    combat(&mut g, vec![Attack { attacker: s, target: AttackTarget::Player(1) }], 0, |_| {});
    assert!(g.battlefield_find(s).is_none(), "sacrificed");
    assert!(g.battlefield_find(hit).is_some(), "the next Sliver entered");
    assert_eq!(g.players[0].library.len(), 3);
    assert!(g.players[0].library.iter().skip(1).any(|c| c.id == miss), "the Bears went under");
}

/// Firewake Sliver — every player's Sliver creatures have haste; any Sliver
/// can be sacrificed for +2/+2 on a Sliver.
#[test]
fn firewake_sliver_hastes_all_slivers_and_sacs_for_a_pump() {
    let mut g = main_phase();
    let f = g.add_card_to_battlefield(0, catalog::firewake_sliver());
    let theirs = g.add_card_to_battlefield(1, catalog::hollowhead_sliver());
    assert!(g.computed_permanent(theirs).unwrap().keywords().contains(&Keyword::Haste));
    let s = g.add_card_to_battlefield(0, catalog::capricious_sliver());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: f,
        ability_index: 0,
        target: Some(Target::Permanent(s)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{1}, sacrifice");
    drain_stack(&mut g);
    assert!(g.battlefield_find(f).is_none());
    assert_eq!(pt(&g, s), (5, 5));
}

/// For the Ancestors — every card of the chosen type among the top six goes
/// to hand; the other four go under.
#[test]
fn for_the_ancestors_takes_the_chosen_type_from_six() {
    let mut g = main_phase();
    let mut slivers = vec![];
    for i in 0..7 {
        let id = if i % 3 == 0 {
            g.add_card_to_library(0, catalog::capricious_sliver())
        } else {
            g.add_card_to_library(0, catalog::island())
        };
        if i % 3 == 0 && i < 6 {
            slivers.push(id);
        }
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Sliver)]));
    let spell = g.add_card_to_hand(0, catalog::for_the_ancestors());
    cast(&mut g, spell, &[]);
    assert!(slivers.iter().all(|id| g.players[0].hand.iter().any(|c| c.id == *id)));
    assert_eq!(g.players[0].hand.len(), 2, "the seventh card's Sliver stays");
}

/// CR 702.107 — Hatchery Sliver grants replicate to Sliver spells at their
/// own mana cost; each copy of a permanent spell becomes a token.
#[test]
fn cr_702_107_hatchery_sliver_replicates_a_sliver_spell() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::hatchery_sliver());
    let spell = g.add_card_to_hand(0, catalog::capricious_sliver());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellReplicate {
        card_id: spell,
        times: 2,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("replicate twice");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Capricious Sliver"), 3);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let not_a_sliver = g.perform_action(GameAction::CastSpellReplicate {
        card_id: bear,
        times: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(not_a_sliver.is_err() || count_named(&g, 0, "Grizzly Bears") <= 1);
}

/// Hollowhead Sliver — your Slivers rummage.
#[test]
fn hollowhead_sliver_grants_tap_discard_draw() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::hollowhead_sliver());
    let s = g.add_card_to_battlefield(0, catalog::capricious_sliver());
    g.clear_sickness(s);
    g.add_card_to_hand(0, catalog::island());
    let drawn = g.add_card_to_library(0, catalog::forest());
    activate(&mut g, s, 0, None).expect("{T}, discard");
    assert!(g.players[0].hand.iter().any(|c| c.id == drawn));
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// CR 702.131 / 701.43 — a blocked Sliver afflicts 2; a nontoken Sliver dying
/// amasses Slivers 2 (a 0/0 Sliver Army with two counters).
#[test]
fn lazotep_sliver_afflicts_and_amasses_slivers() {
    let mut g = main_phase();
    let l = g.add_card_to_battlefield(0, catalog::lazotep_sliver());
    let wall = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.clear_sickness(l);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: l,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let life = g.players[1].life;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, l)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "afflict 2");
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    let s = g.add_card_to_battlefield(0, catalog::hatchery_sliver());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(s)]);
    let army = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.controller == 0)
        .map(|c| c.id)
        .expect("an Army");
    assert!(has_type(&g, army, CreatureType::Sliver));
    assert_eq!(pt(&g, army), (2, 2));
}

/// Regal Sliver — the first Sliver in makes you the monarch; the next pumps.
#[test]
fn regal_sliver_crowns_then_pumps() {
    let mut g = main_phase();
    let r = etb(&mut g, catalog::regal_sliver());
    assert_eq!(g.monarch, Some(0));
    assert_eq!(pt(&g, r), (3, 3));
    etb(&mut g, catalog::capricious_sliver());
    assert_eq!(pt(&g, r), (4, 4));
}

/// CR 701.38 — each Sliver of yours entering goads an opposing creature.
#[test]
fn cr_701_38_taunting_sliver_goads_on_entry() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::taunting_sliver());
    cast(&mut g, t, &[]);
    assert!(g.battlefield_find(bear).unwrap().goaded_by.contains(&0));
}

/// Titan of Littjara — it is the chosen type; entering, it loots once per
/// other creature of yours sharing a type with it.
#[test]
fn titan_of_littjara_loots_per_creature_sharing_its_type() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::capricious_sliver());
    g.add_card_to_battlefield(0, catalog::hollowhead_sliver());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::CreatureType(CreatureType::Sliver),
        DecisionAnswer::Bool(true),
    ]));
    let titan = g.add_card_to_hand(0, catalog::titan_of_littjara());
    cast(&mut g, titan, &[]);
    assert!(has_type(&g, titan, CreatureType::Sliver));
    assert_eq!(g.players[0].hand.len(), 1, "drew two, discarded one");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Pillar of Origins — its mana casts a creature of the chosen type only.
#[test]
fn pillar_of_origins_funds_only_the_chosen_type() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Sliver)]));
    let pillar = g.add_card_to_hand(0, catalog::pillar_of_origins());
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, pillar, &[]);
    g.players[0].mana_pool.empty();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let cast_with_pillar = |g: &mut GameState, id: CardId| {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(cast_with_pillar(&mut g, bolt).is_err(), "not a Sliver creature spell");
    let s = g.add_card_to_hand(0, catalog::hatchery_sliver());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: s,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{1} + the Pillar's green");
}

// ── Vampiric Bloodlust (Edgar Markov, C17) ──────────────────────────────────

fn pass_to_end_step(g: &mut GameState) {
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

/// Bloodlord of Vaasgoth — a Vampire creature spell cast after an opponent
/// took damage enters with three +1/+1 counters; a non-Vampire does not.
#[test]
fn bloodlord_grants_bloodthirst_to_vampire_spells() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::bloodlord_of_vaasgoth());
    let vamp = g.add_card_to_hand(0, catalog::kheru_mind_eater());
    cast(&mut g, vamp, &[]);
    assert_eq!(pt(&g, vamp), (1, 3), "no opponent was dealt damage");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(1)]);
    let vamp2 = g.add_card_to_hand(0, catalog::vein_drinker());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, vamp2, &[]);
    cast(&mut g, bear, &[]);
    assert_eq!(pt(&g, vamp2), (7, 7));
    assert_eq!(pt(&g, bear), (2, 2));
}

/// Consuming Vapors — the victim picks; you gain its toughness; rebound
/// exiles the card.
#[test]
fn consuming_vapors_edicts_for_toughness_and_rebounds() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::consuming_vapors());
    let life = g.players[0].life;
    cast(&mut g, spell, &[Target::Player(1)]);
    assert_eq!(count_named(&g, 1, "Hill Giant"), 0);
    assert_eq!(g.players[0].life, life + 3);
    assert!(g.exile.iter().any(|c| c.id == spell), "rebound");
}

/// CR 903.3 — Crimson Honor Guard burns the turn's player unless they
/// control a commander (anyone's).
#[test]
fn cr_903_3_crimson_honor_guard_spares_a_player_with_a_commander() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::crimson_honor_guard());
    let life = g.players[0].life;
    pass_to_end_step(&mut g);
    assert_eq!(g.players[0].life, life - 4);
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::crimson_honor_guard());
    let stolen = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[1].commanders.push(stolen);
    let life = g.players[0].life;
    pass_to_end_step(&mut g);
    assert_eq!(g.players[0].life, life, "an opponent's commander counts");
}

/// CR 506.2 — Curse of Disturbance / Vitality: seat 1 attacking the cursed
/// seat 2 pays both the curse's owner and the attacker.
#[test]
fn curses_pay_the_owner_and_an_attacking_opponent() {
    for (curse, zombie) in [
        (catalog::curse_of_disturbance as fn() -> CardDefinition, true),
        (catalog::curse_of_vitality, false),
    ] {
        let mut g = multi_player_game(3);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let c = g.add_card_to_hand(0, curse());
        cast(&mut g, c, &[Target::Player(2)]);
        g.active_player_idx = 1;
        let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let (l0, l1) = (g.players[0].life, g.players[1].life);
        combat(&mut g, vec![Attack { attacker: atk, target: AttackTarget::Player(2) }], 1, |_| {});
        if zombie {
            assert_eq!(count_named(&g, 0, "Zombie"), 1);
            assert_eq!(count_named(&g, 1, "Zombie"), 1);
        } else {
            assert_eq!((g.players[0].life, g.players[1].life), (l0 + 2, l1 + 2));
        }
    }
}

/// Dark Impostor — exiles a creature, grows, and borrows its activated
/// abilities.
#[test]
fn dark_impostor_borrows_an_exiled_creatures_ability() {
    let mut g = main_phase();
    let imp = g.add_card_to_battlefield(0, catalog::dark_impostor());
    let sorc = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: imp,
        ability_index: 0,
        target: Some(Target::Permanent(sorc)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("exile the Sorcerer");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == sorc));
    assert_eq!(pt(&g, imp), (3, 3));
    g.clear_sickness(imp);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: imp,
        ability_index: 1,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("the borrowed ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
}

/// Drana — X shrinks a creature's toughness and grows Drana's power.
#[test]
fn drana_trades_toughness_for_power() {
    let mut g = main_phase();
    let d = g.add_card_to_battlefield(0, catalog::drana_kalastria_bloodchief());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: d,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("X = 2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(pt(&g, d), (6, 4));
}

/// Fell the Mighty — every creature with power greater than the target's.
#[test]
fn fell_the_mighty_destroys_the_bigger_creatures() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::fell_the_mighty());
    cast(&mut g, spell, &[Target::Permanent(bear)]);
    assert!(g.battlefield_find(bear).is_some() && g.battlefield_find(other).is_some());
    assert!(g.battlefield_find(giant).is_none());
}

/// Kheru Mind-Eater — the player it hits exiles a card of their choice; you
/// may cast it.
#[test]
fn kheru_mind_eater_takes_a_card_you_may_cast() {
    let mut g = main_phase();
    let k = g.add_card_to_battlefield(0, catalog::kheru_mind_eater());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    combat(&mut g, vec![Attack { attacker: k, target: AttackTarget::Player(1) }], 0, |_| {});
    assert!(g.exile.iter().any(|c| c.id == bolt));
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from exile");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
}

/// Kindred Charge — a hasty copy of each creature of the chosen type, exiled
/// (not sacrificed) at the next end step.
#[test]
fn kindred_charge_copies_the_chosen_type_then_exiles_the_copies() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Bear)]));
    let spell = g.add_card_to_hand(0, catalog::kindred_charge());
    cast(&mut g, spell, &[]);
    assert_eq!(count_named(&g, 0, "Grizzly Bears"), 4);
    assert_eq!(count_named(&g, 0, "Hill Giant"), 1);
    pass_to_end_step(&mut g);
    assert_eq!(count_named(&g, 0, "Grizzly Bears"), 2);
    assert!(g.players[0].graveyard.iter().all(|c| c.definition.name != "Grizzly Bears"));
}

/// Licia — {1} cheaper per life gained this turn; pay 5 life for three
/// counters, once, on your turn only.
#[test]
fn licia_discounts_by_life_gained_and_grows_once_a_turn() {
    let mut g = main_phase();
    g.players[0].life_gained_this_turn = 3;
    let licia = g.add_card_to_hand(0, catalog::licia_sanguine_tribune());
    for c in [Color::Red, Color::White, Color::Black] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: licia,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{5} less 3");
    drain_stack(&mut g);
    let pay = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: licia,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    pay(&mut g).expect("pay 5 life");
    drain_stack(&mut g);
    assert_eq!(pt(&g, licia), (7, 7));
    assert!(pay(&mut g).is_err(), "once each turn");
}

/// Mathas — the bounty lands on an opposing creature; when it dies, its
/// controller's opponents each draw and gain 2.
#[test]
fn mathas_bounty_pays_the_opponents_of_the_dead_creatures_controller() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mathas_fiend_seeker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    pass_to_end_step(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Bounty), 1);
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(bear)]);
    assert_eq!(g.players[0].life, life + 2);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Outpost Siege — Khans impulse-draws each upkeep.
#[test]
fn outpost_siege_khans_exiles_the_top_card_each_upkeep() {
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let siege = g.add_card_to_hand(0, catalog::outpost_siege());
    cast(&mut g, siege, &[]);
    let top = g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::Upkeep);
    assert!(g.exile.iter().any(|c| c.id == top));
}

/// Vein Drinker — it fights, and grows when a creature it damaged dies.
#[test]
fn vein_drinker_fights_and_grows() {
    let mut g = main_phase();
    let v = g.add_card_to_battlefield(0, catalog::vein_drinker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(v);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: v,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{R}, {T}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(pt(&g, v), (5, 5));
}

// ── Guided By Nature (Freyalise, Llanowar's Fury, C14) ─────────────────────

fn equip(g: &mut GameState, gear: CardId, onto: CardId) {
    flood(g, 0);
    g.perform_action(GameAction::Equip { equipment: gear, target: onto }).expect("equip");
    drain_stack(g);
}

/// CR 701.16 — Assault Suit: the equipped creature can't be sacrificed (an
/// edict finds nothing), and it can't attack the Suit's controller after it
/// is lent out at an opponent's upkeep.
#[test]
fn cr_701_16_assault_suit_blocks_sacrifice_and_lends_the_creature() {
    let mut g = main_phase();
    let suit = g.add_card_to_battlefield(0, catalog::assault_suit());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    equip(&mut g, suit, bear);
    assert_eq!(pt(&g, bear), (4, 4));
    g.priority.player_with_priority = 1;
    let edict = g.add_card_to_hand(1, catalog::diabolic_edict());
    try_cast(&mut g, 1, edict, &[Target::Player(0)]).expect("edict");
    assert!(g.battlefield_find(bear).is_some(), "can't be sacrificed");
    // Seat 1's upkeep: lend it.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.controller, 1);
    assert!(!c.tapped, "untapped as it is lent");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let at_owner = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(0),
    }]));
    assert!(at_owner.is_err(), "can't attack the Suit's controller");
}

/// Creeperhulk — a creature of yours becomes a 5/5 trampler.
#[test]
fn creeperhulk_makes_a_five_five_trampler() {
    let mut g = main_phase();
    let hulk = g.add_card_to_battlefield(0, catalog::creeperhulk());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hulk,
        ability_index: 0,
        target: Some(Target::Permanent(elf)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{1}{G}");
    drain_stack(&mut g);
    assert_eq!(pt(&g, elf), (5, 5));
    assert!(g.computed_permanent(elf).unwrap().keywords().contains(&Keyword::Trample));
}

/// Drove of Elves counts your green permanents, itself included.
#[test]
fn drove_of_elves_counts_green_permanents() {
    let mut g = main_phase();
    let d = g.add_card_to_battlefield(0, catalog::drove_of_elves());
    g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(pt(&g, d), (2, 2));
}

/// Grave Sifter — each player returns the cards of their own named type.
#[test]
fn grave_sifter_returns_each_players_own_type() {
    let mut g = main_phase();
    let e1 = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let e2 = g.add_card_to_graveyard(0, catalog::elvish_mystic());
    let b0 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b1 = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let e3 = g.add_card_to_graveyard(1, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::CreatureType(CreatureType::Elf),
        DecisionAnswer::CreatureType(CreatureType::Bear),
    ]));
    etb(&mut g, catalog::grave_sifter());
    let in_hand = |g: &GameState, p: usize, id| g.players[p].hand.iter().any(|c| c.id == id);
    assert!(in_hand(&g, 0, e1) && in_hand(&g, 0, e2) && !in_hand(&g, 0, b0));
    assert!(in_hand(&g, 1, b1) && !in_hand(&g, 1, e3));
}

/// Grim Flowering draws per creature card in your graveyard.
#[test]
fn grim_flowering_draws_per_creature_card() {
    let mut g = main_phase();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::grim_flowering());
    cast(&mut g, spell, &[]);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Haunted Fengraf — the only creature card comes back.
#[test]
fn haunted_fengraf_returns_a_creature_card() {
    let mut g = main_phase();
    let f = g.add_card_to_battlefield(0, catalog::haunted_fengraf());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    activate(&mut g, f, 1, None).expect("{3}, {T}, sacrifice");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Hunting Triad makes three Elf Warriors.
#[test]
fn hunting_triad_makes_three_elf_warriors() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::hunting_triad());
    cast(&mut g, spell, &[]);
    assert_eq!(count_named(&g, 0, "Elf Warrior"), 3);
}

/// Lifeblood Hydra — X counters; dying, it pays out its power.
#[test]
fn lifeblood_hydra_pays_its_power_on_death() {
    let mut g = main_phase();
    let h = g.add_card_to_hand(0, catalog::lifeblood_hydra());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: h,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("X = 3");
    drain_stack(&mut g);
    assert_eq!(pt(&g, h), (3, 3));
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(h)]);
    assert_eq!(g.players[0].life, life + 3);
    assert_eq!(g.players[0].hand.len(), hand + 3);
}

/// Loreseeker's Stone costs {1} more per card in hand.
#[test]
fn loreseekers_stone_is_taxed_by_hand_size() {
    let mut g = main_phase();
    let s = g.add_card_to_battlefield(0, catalog::loreseekers_stone());
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let go = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: s,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    g.players[0].mana_pool.add_colorless(4);
    assert!(go(&mut g).is_err(), "{{3}} + 2 for the hand");
    g.players[0].mana_pool.add_colorless(1);
    go(&mut g).expect("{{5}}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 5);
}

/// Siege Behemoth — while it attacks, a blocked creature of yours hits the
/// player anyway.
#[test]
fn siege_behemoth_lets_blocked_creatures_hit_the_player() {
    let mut g = main_phase();
    let s = g.add_card_to_battlefield(0, catalog::siege_behemoth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_stone());
    g.clear_sickness(s);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: s, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, bear)])).expect("block the Bears");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, 20 - 7 - 2);
}

/// Sylvan Offering — you and an opponent each get an X/X Treefolk and X Elf
/// Warriors.
#[test]
fn sylvan_offering_gifts_both_halves() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::sylvan_offering());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("X = 2");
    drain_stack(&mut g);
    for p in [0, 1] {
        assert_eq!(count_named(&g, p, "Elf Warrior"), 2);
        let tf = g
            .battlefield
            .iter()
            .find(|c| c.controller == p && c.definition.name == "Treefolk")
            .map(|c| c.id)
            .expect("a Treefolk");
        assert_eq!(pt(&g, tf), (2, 2));
    }
}

/// Wave of Vitriol — artifacts, enchantments and nonbasic lands go; each land
/// buys its controller a tapped basic.
#[test]
fn wave_of_vitriol_trades_nonbasics_for_basics() {
    let mut g = main_phase();
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let tower = g.add_card_to_battlefield(1, catalog::command_tower());
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let basic = g.add_card_to_library(1, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::wave_of_vitriol());
    cast(&mut g, spell, &[]);
    assert!(g.battlefield_find(ring).is_none() && g.battlefield_find(tower).is_none());
    assert!(g.battlefield_find(forest).is_some());
    let b = g.battlefield_find(basic).expect("fetched");
    assert!(b.tapped && b.controller == 1);
}

/// Wolfcaller's Howl — a Wolf per opponent with four or more cards.
#[test]
fn wolfcallers_howl_counts_full_hands() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.add_card_to_battlefield(0, catalog::wolfcallers_howl());
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::forest());
    }
    for _ in 0..3 {
        g.add_card_to_hand(2, catalog::forest());
    }
    g.add_card_to_library(0, catalog::forest());
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Wolf"), 1);
}

/// Wren's Run Packmaster — champions an Elf; its Wolves have deathtouch.
#[test]
fn wrens_run_packmaster_champions_an_elf_and_arms_wolves() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let pm = g.add_card_to_hand(0, catalog::wrens_run_packmaster());
    cast(&mut g, pm, &[]);
    assert!(g.battlefield_find(pm).is_some(), "an Elf was championed");
    assert!(g.exile.iter().any(|c| c.id == elf));
    activate(&mut g, pm, 0, None).expect("{2}{G}");
    let wolf = g.battlefield.iter().find(|c| c.definition.name == "Wolf").map(|c| c.id).unwrap();
    assert!(g.computed_permanent(wolf).unwrap().keywords().contains(&Keyword::Deathtouch));
}

// ── Grave Danger (Gisa and Geralf, SCD) ─────────────────────────────────────

fn cast_from_gy(g: &mut GameState, id: CardId) -> Result<(), String> {
    flood(g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Gisa and Geralf — one Zombie creature spell from the graveyard per turn,
/// and only a Zombie.
#[test]
fn gisa_and_geralf_casts_one_zombie_from_the_graveyard_a_turn() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::gisa_and_geralf());
    let a = g.add_card_to_graveyard(0, catalog::loyal_subordinate());
    let b = g.add_card_to_graveyard(0, catalog::unbreathing_horde());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    assert!(cast_from_gy(&mut g, bear).is_err(), "not a Zombie");
    cast_from_gy(&mut g, a).expect("the first Zombie");
    assert!(g.battlefield_find(a).is_some());
    assert!(cast_from_gy(&mut g, b).is_err(), "once each turn");
}

/// The bot casts from its graveyard through a board permission (Gisa and
/// Geralf) — it had no block for Muldrotha-style grants at all.
#[test]
fn bot_casts_a_zombie_through_gisa_and_geralf() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::gisa_and_geralf());
    let z = g.add_card_to_graveyard(0, catalog::loyal_subordinate());
    // Post-combat: a fresh creature is not held for the second main.
    g.step = TurnStep::PostCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let mut bot = HeuristicBot::new();
    for _ in 0..16 {
        let action = bot.next_action(&g, 0).expect("bot acts");
        if let GameAction::CastSpell { card_id, .. } = action
            && card_id == z
        {
            return;
        }
        let _ = g.perform_action(action);
    }
    panic!("the bot never cast the Zombie from its graveyard");
}

/// Scourge of Nel Toth — {B}{B} plus two sacrificed creatures from the
/// graveyard; the alternative cost is not offered from hand.
#[test]
fn scourge_of_nel_toth_returns_for_two_creatures() {
    let mut g = main_phase();
    let s = g.add_card_to_graveyard(0, catalog::scourge_of_nel_toth());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: s,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{B}{B} and two creatures");
    drain_stack(&mut g);
    assert!(g.battlefield_find(s).is_some());
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    let mut g = main_phase();
    let h = g.add_card_to_hand(0, catalog::scourge_of_nel_toth());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    g.players[0].mana_pool.add(Color::Black, 2);
    let from_hand = g.perform_action(GameAction::CastSpellAlternative {
        card_id: h,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(from_hand.is_err(), "graveyard only");
}

/// Laboratory Drudge — draws at end step once a graveyard was used.
#[test]
fn laboratory_drudge_draws_after_a_graveyard_cast() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::laboratory_drudge());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    pass_to_end_step(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "nothing from a graveyard yet");
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::laboratory_drudge());
    g.add_card_to_battlefield(0, catalog::gisa_and_geralf());
    let z = g.add_card_to_graveyard(0, catalog::loyal_subordinate());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    cast_from_gy(&mut g, z).expect("cast");
    let hand = g.players[0].hand.len();
    pass_to_end_step(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Grimoire of the Dead — three study counters, then every creature card in
/// every graveyard joins you as a black Zombie.
#[test]
fn grimoire_of_the_dead_raises_every_graveyard() {
    let mut g = main_phase();
    let gr = g.add_card_to_battlefield(0, catalog::grimoire_of_the_dead());
    g.battlefield_find_mut(gr).unwrap().add_counters(CounterType::Study, 3);
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let mine = g.add_card_to_graveyard(0, catalog::hill_giant());
    activate(&mut g, gr, 1, None).expect("remove three, sacrifice");
    for id in [theirs, mine] {
        let c = g.battlefield_find(id).expect("raised");
        assert_eq!(c.controller, 0);
        assert!(has_type(&g, id, CreatureType::Zombie));
        assert!(g.computed_permanent(id).unwrap().colors.contains(Color::Black));
    }
}

/// Liliana, Untouched by Death — +1 drains when a Zombie is milled; −3 opens
/// the graveyard's Zombies.
#[test]
fn liliana_untouched_mills_for_a_drain_and_opens_the_graveyard() {
    let mut g = main_phase();
    let l = g.add_card_to_battlefield(0, catalog::liliana_untouched_by_death());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::loyal_subordinate());
    g.add_card_to_library(0, catalog::island());
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: l, ability_index: 0, target: None, x_value: None })
        .expect("+1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
    let z = g.players[0].graveyard.iter().find(|c| c.definition.name == "Loyal Subordinate").unwrap().id;
    g.battlefield_find_mut(l).unwrap().loyalty_uses_this_turn = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: l, ability_index: 2, target: None, x_value: None })
        .expect("−3");
    drain_stack(&mut g);
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: z,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Zombie from the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(z).is_some());
}

/// CR 207.2c lieutenant — Loyal Subordinate drains only with your own
/// commander on the battlefield.
#[test]
fn cr_207_2c_loyal_subordinate_needs_your_commander() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::loyal_subordinate());
    let life = g.players[1].life;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life, "no commander");
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::loyal_subordinate());
    let cmd = g.add_card_to_battlefield(0, catalog::gisa_and_geralf());
    g.players[0].commanders.push(cmd);
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!((g.players[1].life, g.players[2].life), (life - 3, life - 3));
}

/// Overseer of the Damned — an opponent's nontoken creature dying makes a
/// tapped Zombie.
#[test]
fn overseer_of_the_damned_raises_opponents_dead() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::overseer_of_the_damned());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(bear)]);
    let z = g.battlefield.iter().find(|c| c.definition.name == "Zombie").expect("a Zombie");
    assert!(z.tapped && z.controller == 0);
}

/// Unbreathing Horde — counters from Zombies on board and in the graveyard;
/// damage removes one counter instead.
#[test]
fn unbreathing_horde_counts_zombies_and_sheds_counters() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::loyal_subordinate());
    g.add_card_to_graveyard(0, catalog::laboratory_drudge());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let h = g.add_card_to_hand(0, catalog::unbreathing_horde());
    cast(&mut g, h, &[]);
    assert_eq!(pt(&g, h), (2, 2));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(h)]);
    assert_eq!(pt(&g, h), (1, 1), "the bolt only took a counter");
}

/// Vela the Night-Clad — your creatures leaving drain each opponent.
#[test]
fn vela_drains_when_your_creatures_leave() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::vela_the_night_clad());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Intimidate));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    cast(&mut g, bolt, &[Target::Permanent(bear)]);
    assert_eq!((g.players[1].life, g.players[2].life), (l1 - 1, l2 - 1));
}

/// Zombie Apocalypse — Zombies return tapped, then Humans die.
#[test]
fn zombie_apocalypse_raises_zombies_and_kills_humans() {
    let mut g = main_phase();
    let z = g.add_card_to_graveyard(0, catalog::loyal_subordinate());
    let human = g.add_card_to_battlefield(1, catalog::lilianas_devotee());
    let spell = g.add_card_to_hand(0, catalog::zombie_apocalypse());
    cast(&mut g, spell, &[]);
    assert!(g.battlefield_find(z).is_some_and(|c| c.tapped));
    assert!(g.battlefield_find(human).is_none());
}

/// Liliana's Devotee — Zombies +1/+0; a death this turn lets you buy a Zombie
/// at your end step.
#[test]
fn lilianas_devotee_buys_a_zombie_after_a_death() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::lilianas_devotee());
    let sub = g.add_card_to_battlefield(0, catalog::loyal_subordinate());
    assert_eq!(pt(&g, sub), (4, 1));
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(bear)]);
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    pass_to_end_step(&mut g);
    assert_eq!(count_named(&g, 0, "Zombie"), 1);
}

/// Havengul Lich — {1}: a creature card in any graveyard is castable this turn.
#[test]
fn havengul_lich_opens_a_creature_card_in_any_graveyard() {
    let mut g = main_phase();
    let lich = g.add_card_to_battlefield(0, catalog::havengul_lich());
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lich,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{1}");
    drain_stack(&mut g);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast it");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(0));
}

/// Lotleth Giant — undergrowth damage to a target opponent.
#[test]
fn lotleth_giant_burns_for_creature_cards() {
    let mut g = main_phase();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    let giant = g.add_card_to_hand(0, catalog::lotleth_giant());
    let life = g.players[1].life;
    cast(&mut g, giant, &[]);
    assert_eq!(g.players[1].life, life - 2);
}

// ── Forged in Stone (Nahiri, the Lithomancer, C14) ─────────────────────────

fn loyalty(g: &mut GameState, pw: CardId, index: usize) {
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw,
        ability_index: index,
        target: None,
        x_value: None,
    })
    .expect("loyalty");
    drain_stack(g);
}

/// Nahiri — +2 suits a fresh Kor Soldier; −2 fetches an Equipment card.
#[test]
fn nahiri_suits_up_a_soldier_and_puts_out_equipment() {
    let mut g = main_phase();
    let n = g.add_card_to_battlefield(0, catalog::nahiri_the_lithomancer());
    let spear = g.add_card_to_battlefield(0, catalog::moonsilver_spear());
    loyalty(&mut g, n, 0);
    let soldier = g.battlefield.iter().find(|c| c.definition.name == "Kor Soldier").unwrap().id;
    assert_eq!(g.battlefield_find(spear).unwrap().attached_to, Some(soldier));
    let mut g = main_phase();
    let n = g.add_card_to_battlefield(0, catalog::nahiri_the_lithomancer());
    let scythe = g.add_card_to_graveyard(0, catalog::strata_scythe());
    loyalty(&mut g, n, 1);
    assert!(g.battlefield_find(scythe).is_some());
}

/// Adarkar Valkyrie — the marked creature comes back under your control.
#[test]
fn adarkar_valkyrie_steals_the_dead() {
    let mut g = main_phase();
    let v = g.add_card_to_battlefield(0, catalog::adarkar_valkyrie());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(v);
    g.perform_action(GameAction::ActivateAbility {
        card_id: v,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{T}");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(bear)]);
    assert_eq!(g.battlefield_find(bear).map(|c| c.controller), Some(0));
}

/// Angel of the Dire Hour — flashed in from hand, it exiles every attacker.
#[test]
fn angel_of_the_dire_hour_exiles_attackers() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let angel = g.add_card_to_hand(1, catalog::angel_of_the_dire_hour());
    combat(
        &mut g,
        vec![Attack { attacker: bear, target: AttackTarget::Player(1) }],
        1,
        |g| {
            try_cast(g, 1, angel, &[]).expect("flash");
        },
    );
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// Arcane Lighthouse — an opponent's hexproof creature becomes targetable.
#[test]
fn arcane_lighthouse_strips_hexproof() {
    let mut g = main_phase();
    let lh = g.add_card_to_battlefield(0, catalog::arcane_lighthouse());
    let drove = g.add_card_to_battlefield(1, catalog::drove_of_elves());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(try_cast(&mut g, 0, bolt, &[Target::Permanent(drove)]).is_err(), "hexproof");
    activate(&mut g, lh, 1, None).expect("{1}, {T}");
    try_cast(&mut g, 0, bolt, &[Target::Permanent(drove)]).expect("no hexproof now");
    assert!(g.battlefield_find(drove).is_none());
}

/// Benevolent Offering — three Spirits each, then life by creature count.
#[test]
fn benevolent_offering_shares_spirits_and_life() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::benevolent_offering());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    cast(&mut g, spell, &[]);
    assert_eq!(count_named(&g, 0, "Spirit"), 3);
    assert_eq!(count_named(&g, 1, "Spirit"), 3);
    assert_eq!((g.players[0].life, g.players[1].life), (l0 + 6, l1 + 6));
}

/// Celestial Crusader pumps every other white creature, anyone's.
#[test]
fn celestial_crusader_pumps_all_white_creatures() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::celestial_crusader());
    let monk = g.add_card_to_battlefield(1, catalog::adarkar_valkyrie());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(pt(&g, monk), (5, 6));
    assert_eq!(pt(&g, bear), (2, 2));
}

/// Deploy to the Front / Nomads' Assembly / Geist-Honored Monk count creatures.
#[test]
fn creature_count_token_makers() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let d = g.add_card_to_hand(0, catalog::deploy_to_the_front());
    cast(&mut g, d, &[]);
    assert_eq!(count_named(&g, 0, "Soldier"), 2);
    let na = g.add_card_to_hand(0, catalog::nomads_assembly());
    cast(&mut g, na, &[]);
    assert_eq!(count_named(&g, 0, "Kor Soldier"), 3, "one per creature you control");
    let monk = g.add_card_to_hand(0, catalog::geist_honored_monk());
    cast(&mut g, monk, &[]);
    assert_eq!(pt(&g, monk), (9, 9), "3 + 3 + Bears + 2 Spirits + itself");
}

/// Hallowed Spiritkeeper — dying, a Spirit per creature card in your
/// graveyard (itself included).
#[test]
fn hallowed_spiritkeeper_leaves_spirits() {
    let mut g = main_phase();
    let k = g.add_card_to_battlefield(0, catalog::hallowed_spiritkeeper());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(k)]);
    assert_eq!(count_named(&g, 0, "Spirit"), 2);
}

/// Masterwork of Ingenuity enters as a copy of an Equipment.
#[test]
fn masterwork_of_ingenuity_copies_an_equipment() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::moonsilver_spear());
    let m = g.add_card_to_hand(0, catalog::masterwork_of_ingenuity());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, m, &[]);
    assert_eq!(count_named(&g, 0, "Moonsilver Spear"), 2);
}

/// Moonsilver Spear — the equipped creature's attack makes a 4/4 Angel.
#[test]
fn moonsilver_spear_makes_an_angel_on_attack() {
    let mut g = main_phase();
    let spear = g.add_card_to_battlefield(0, catalog::moonsilver_spear());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    equip(&mut g, spear, bear);
    combat(&mut g, vec![Attack { attacker: bear, target: AttackTarget::Player(1) }], 0, |_| {});
    assert_eq!(count_named(&g, 0, "Angel"), 1);
}

/// Strata Scythe — +1/+1 per land sharing the imprinted land's name.
#[test]
fn strata_scythe_counts_lands_named_like_its_imprint() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(1, catalog::plains());
    g.add_card_to_battlefield(1, catalog::island());
    let s = g.add_card_to_hand(0, catalog::strata_scythe());
    cast(&mut g, s, &[]);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    equip(&mut g, s, bear);
    assert_eq!(pt(&g, bear), (4, 4));
}

/// True Conviction — double strike and lifelink for your creatures.
#[test]
fn true_conviction_grants_double_strike_and_lifelink() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::true_conviction());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let kw = g.computed_permanent(bear).unwrap().keywords().to_vec();
    assert!(kw.contains(&Keyword::DoubleStrike) && kw.contains(&Keyword::Lifelink));
}

/// Twilight Shepherd — returns what died this turn; persist brings it back.
#[test]
fn twilight_shepherd_returns_the_turns_dead() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let old = g.add_card_to_graveyard(0, catalog::hill_giant());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Permanent(bear)]);
    let s = g.add_card_to_hand(0, catalog::twilight_shepherd());
    cast(&mut g, s, &[]);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == old), "it was there before");
}

// ── Seize Control (C15, Mizzix of the Izmagnus) ─────────────────────────────

fn cast_x(g: &mut GameState, id: CardId, x: u32, targets: &[Target]) {
    flood(g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: Some(x),
    })
    .expect("cast");
    drain_stack(g);
}

/// CR 110.2 / 608.3 — Aethersnatch takes an opponent's creature spell, which
/// then enters under its new controller; the owner is unchanged.
#[test]
fn cr_608_3_aethersnatch_steals_a_creature_spell() {
    let mut g = main_phase();
    let giant = g.add_card_to_hand(1, catalog::hill_giant());
    g.active_player_idx = 1;
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: giant,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("opponent casts");
    g.priority.player_with_priority = 0;
    let snatch = g.add_card_to_hand(0, catalog::aethersnatch());
    cast(&mut g, snatch, &[Target::Permanent(giant)]);
    let c = g.battlefield_find(giant).expect("the Giant resolved");
    assert_eq!((c.controller, c.owner), (0, 1));
}

/// Awaken the Sky Tyrant — an opponent's source damaging you trades it for a
/// 5/5 flying Dragon; the second source's trigger finds nothing to sacrifice.
#[test]
fn awaken_the_sky_tyrant_becomes_a_dragon_once() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::awaken_the_sky_tyrant());
    g.active_player_idx = 1;
    let attacks: Vec<Attack> = (0..2)
        .map(|_| Attack {
            attacker: g.add_card_to_battlefield(1, catalog::grizzly_bears()),
            target: AttackTarget::Player(0),
        })
        .collect();
    for a in &attacks {
        g.clear_sickness(a.attacker);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    // Damage events come back from the step change; the game loop
    // dispatches them (the `combat` helper drops them).
    while g.step != TurnStep::EndCombat {
        let ev = g.advance_step(Vec::new()).expect("step");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, 16, "both hit");
    assert_eq!(count_named(&g, 0, "Dragon"), 1);
    assert_eq!(count_named(&g, 0, "Awaken the Sky Tyrant"), 0);
}

/// Meteor Blast — X targets, 4 damage each (CR 601.2c: exactly X targets).
#[test]
fn meteor_blast_hits_x_targets_for_four() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::hill_giant());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    let m = g.add_card_to_hand(0, catalog::meteor_blast());
    cast_x(&mut g, m, 3, &[Target::Permanent(a), Target::Permanent(b), Target::Player(1)]);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert_eq!(g.players[1].life, 16);
}

/// Mystic Confluence — CR 700.2d lets a mode repeat; the default is counter
/// unless {3}, then draw two.
#[test]
fn cr_700_2d_mystic_confluence_counters_and_draws_two() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let giant = g.add_card_to_hand(1, catalog::hill_giant());
    g.active_player_idx = 1;
    g.players[1].mana_pool.add(Color::Red, 4);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: giant,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("opponent casts");
    g.priority.player_with_priority = 0;
    let m = g.add_card_to_hand(0, catalog::mystic_confluence());
    let before = g.players[0].hand.len();
    cast(&mut g, m, &[Target::Permanent(giant)]);
    assert!(g.battlefield_find(giant).is_none(), "no mana left to pay {{3}}");
    assert_eq!(g.players[0].hand.len(), before - 1 + 2);
}

/// Rite of the Raging Storm — each upkeep hands that player a Lightning
/// Rager, and a Rager can't attack the Rite's controller.
#[test]
fn rite_of_the_raging_storm_gives_ragers_that_cant_hit_you() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::rite_of_the_raging_storm());
    g.active_player_idx = 1;
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 1, "Lightning Rager"), 1);
    let rager = g.battlefield.iter().find(|c| c.controller == 1).unwrap().id;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    assert!(g
        .perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: rager,
            target: AttackTarget::Player(0),
        }]))
        .is_err());
}

/// Seal of the Guildpact — {1} less per chosen colour the spell is (generic
/// only, CR 601.2f).
#[test]
fn seal_of_the_guildpact_discounts_per_chosen_color() {
    let mut g = main_phase();
    let seal = g.add_card_to_battlefield(0, catalog::seal_of_the_guildpact());
    g.battlefield.find_by_id_mut(seal).unwrap().chosen_colors = vec![Color::Red, Color::Blue];
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: giant,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{3}{R} for three");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some());
}

/// Stolen Goods — the opponent exiles to a nonland card, you cast it free.
#[test]
fn stolen_goods_casts_the_opponents_card_free() {
    let mut g = main_phase();
    g.add_card_to_library(1, catalog::island());
    let giant = g.add_card_to_library(1, catalog::hill_giant());
    g.players[1].library.reverse();
    let s = g.add_card_to_hand(0, catalog::stolen_goods());
    cast(&mut g, s, &[Target::Player(1)]);
    assert!(g.exile.iter().any(|c| c.id == giant));
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: giant,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free cast from exile");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(giant).map(|c| c.controller), Some(0));
}

/// Gigantoplasm — copies a creature (CR 707.9b) and keeps its own
/// "{X}: base power and toughness X/X".
#[test]
fn gigantoplasm_copies_and_sets_its_base_size() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::hill_giant());
    let p = g.add_card_to_hand(0, catalog::gigantoplasm());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, p, &[]);
    assert_eq!(count_named(&g, 0, "Hill Giant"), 1);
    activate(&mut g, p, 0, Some(7)).expect("{X}");
    assert_eq!(pt(&g, p), (7, 7));
}

/// Lone Revenant — the intervening "if you control no other creatures"
/// (CR 603.4) gates its look-four.
#[test]
fn cr_603_4_lone_revenant_digs_only_alone() {
    for alone in [true, false] {
        let mut g = main_phase();
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::island());
        }
        let r = g.add_card_to_battlefield(0, catalog::lone_revenant());
        if !alone {
            g.add_card_to_battlefield(0, catalog::grizzly_bears());
        }
        let before = g.players[0].hand.len();
        combat(&mut g, vec![Attack { attacker: r, target: AttackTarget::Player(1) }], 0, |_| {});
        assert_eq!(g.players[0].hand.len(), before + usize::from(alone), "alone = {alone}");
    }
}

/// The simple five: Warchief Giant's haste + myriad, Etherium-Horn Sorcerer's
/// self-bounce, Jace's Archivist's wheel, Desperate Ravings' random discard,
/// Call the Skybreaker's Dragon-sized Elemental.
#[test]
fn seize_control_simple_cards() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(0, catalog::warchief_giant());
    assert!(g.computed_permanent(giant).unwrap().keywords().contains(&Keyword::Haste));
    let s = g.add_card_to_battlefield(0, catalog::etherium_horn_sorcerer());
    activate(&mut g, s, 0, None).expect("bounce");
    assert!(g.players[0].hand.iter().any(|c| c.id == s));

    let mut g = main_phase();
    for seat in [0, 1] {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::island());
    let a = g.add_card_to_battlefield(0, catalog::jaces_archivist());
    g.clear_sickness(a);
    activate(&mut g, a, 0, None).expect("wheel");
    assert_eq!((g.players[0].hand.len(), g.players[1].hand.len()), (2, 2));

    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let d = g.add_card_to_hand(0, catalog::desperate_ravings());
    cast(&mut g, d, &[]);
    assert_eq!(g.players[0].hand.len(), 1);

    let mut g = main_phase();
    let c = g.add_card_to_hand(0, catalog::call_the_skybreaker());
    cast(&mut g, c, &[]);
    assert_eq!(pt(&g, g.battlefield.iter().find(|c| c.definition.name == "Elemental").unwrap().id), (5, 5));
}

/// The bot casts "each of X targets" at the X its targets allow (Meteor
/// Blast), and answers a spell with Mystic Confluence and Aethersnatch.
#[test]
fn bot_casts_meteor_blast_confluence_and_aethersnatch() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = main_phase();
    let m = g.add_card_to_hand(0, catalog::meteor_blast());
    g.add_card_to_battlefield(1, catalog::hill_giant());
    g.add_card_to_battlefield(1, catalog::hill_giant());
    g.step = TurnStep::PostCombatMain;
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(2);
    match HeuristicBot::new().next_action(&g, 0) {
        Some(GameAction::CastSpell { card_id, x_value: Some(2), additional_targets, .. })
            if card_id == m && additional_targets.len() == 1 => {}
        other => panic!("expected Meteor Blast at X=2, got {other:?}"),
    }
    for (answer, blue) in [(catalog::mystic_confluence(), 5), (catalog::aethersnatch(), 6)] {
        let mut g = main_phase();
        let giant = g.add_card_to_hand(1, catalog::hill_giant());
        g.active_player_idx = 1;
        g.players[1].mana_pool.add(Color::Red, 4);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: giant,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("opponent casts");
        g.priority.player_with_priority = 0;
        let a = g.add_card_to_hand(0, answer);
        g.players[0].mana_pool.add(Color::Blue, blue);
        match HeuristicBot::new().next_action(&g, 0) {
            Some(GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. })
                if card_id == a && t == giant => {}
            other => panic!("expected an answer to the Giant, got {other:?}"),
        }
    }
}

// ── Rebellion Rising (ONC, Neyali, Suns' Vanguard) ──────────────────────────

fn soldier_token(g: &mut GameState, seat: usize) -> CardId {
    use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
    let id = g.add_token_to_battlefield(
        seat,
        &TokenDefinition {
            name: "Soldier".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
            ..Default::default()
        },
    );
    g.clear_sickness(id);
    id
}

fn advance_to_turn_of(g: &mut GameState, seat: usize) {
    loop {
        let ev = g.advance_step(Vec::new()).expect("step");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(g);
        if g.active_player_idx == seat && g.step == TurnStep::PreCombatMain {
            break;
        }
    }
}

/// CR 508.1 — Neyali's exiled card is playable only during turns you
/// attacked with a token: armed as it resolves, dormant on the next turn,
/// armed again when a token is declared as an attacker. Attacking tokens
/// have double strike.
#[test]
fn cr_508_1_neyali_plays_on_turns_a_token_attacked() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::neyali_suns_vanguard());
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    let t = soldier_token(&mut g, 0);
    let armed = |g: &GameState| {
        g.exile.iter().find(|c| c.id == top).and_then(|c| c.may_play_until).map(|p| p.player)
    };
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: t,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(t).unwrap().keywords().contains(&Keyword::DoubleStrike));
    assert_eq!(armed(&g), Some(0), "playable this turn");
    advance_to_turn_of(&mut g, 1);
    assert_ne!(armed(&g), Some(0), "dormant on a turn with no token attack");
    advance_to_turn_of(&mut g, 0);
    assert_ne!(armed(&g), Some(0), "not yet — no token has attacked");
    g.clear_sickness(t);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: t,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack again");
    drain_stack(&mut g);
    assert_eq!(armed(&g), Some(0), "armed again");
}

/// Roar of Resistance — tokens have haste; the paid trigger pumps only
/// creatures attacking an opponent.
#[test]
fn roar_of_resistance_pumps_attackers_of_opponents() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::roar_of_resistance());
    let t = soldier_token(&mut g, 0);
    assert!(g.computed_permanent(t).unwrap().keywords().contains(&Keyword::Haste));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g, 0);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: t,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(pt(&g, t), (3, 1));
}

/// Staff of the Storyteller — a Spirit on entry, a story counter per token
/// batch, and a counter pays for a card.
#[test]
fn staff_of_the_storyteller_counts_token_batches() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::island());
    let s = g.add_card_to_hand(0, catalog::staff_of_the_storyteller());
    cast(&mut g, s, &[]);
    assert_eq!(count_named(&g, 0, "Spirit"), 1);
    assert_eq!(g.battlefield_find(s).unwrap().counter_count(CounterType::Story), 1);
    let before = g.players[0].hand.len();
    activate(&mut g, s, 0, None).expect("draw");
    assert_eq!(g.players[0].hand.len(), before + 1);
    assert_eq!(g.battlefield_find(s).unwrap().counter_count(CounterType::Story), 0);
}

/// Goldwardens' Gambit — affinity for Equipment; five hasty Rebels, each
/// taking a free Equipment.
#[test]
fn goldwardens_gambit_suits_up_its_rebels() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::mace_of_the_valiant());
    let b = g.add_card_to_battlefield(0, catalog::mace_of_the_valiant());
    let gg = g.add_card_to_hand(0, catalog::goldwardens_gambit());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: gg,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{6}{R}{R} less two");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Rebel"), 5);
    for e in [a, b] {
        let host = g.battlefield_find(e).unwrap().attached_to.expect("attached");
        assert_eq!(g.battlefield_find(host).unwrap().definition.name, "Rebel");
    }
}

/// Call the Coppercoats — a Soldier per creature the targeted opponent has.
#[test]
fn call_the_coppercoats_matches_the_opponents_board() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::call_the_coppercoats());
    cast(&mut g, c, &[Target::Player(1)]);
    assert_eq!(count_named(&g, 0, "Human Soldier"), 2);
}

/// The rest of Rebellion Rising, one assertion each.
#[test]
fn rebellion_rising_cards() {
    // Collective Effort's base mode: destroy a power-4 creature.
    let mut g = main_phase();
    let big = g.add_card_to_battlefield(1, catalog::hill_giant());
    let ogre = g.add_card_to_battlefield(1, catalog::serra_angel());
    let e = g.add_card_to_hand(0, catalog::collective_effort());
    cast(&mut g, e, &[Target::Permanent(ogre)]);
    assert!(g.battlefield_find(ogre).is_none() && g.battlefield_find(big).is_some());

    // Elspeth Tirel's −5 keeps lands, tokens and herself.
    let mut g = main_phase();
    let el = g.add_card_to_battlefield(0, catalog::elspeth_tirel());
    g.battlefield.find_by_id_mut(el).unwrap().add_counters(CounterType::Loyalty, 5);
    let land = g.add_card_to_battlefield(1, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tok = soldier_token(&mut g, 1);
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: el,
        ability_index: 2,
        target: None,
        x_value: None,
    })
    .expect("−5");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(land).is_some() && g.battlefield_find(tok).is_some());

    // For Mirrodin!: Kemba's Banner arrives on a Rebel, +1/+1 per creature.
    let mut g = main_phase();
    let k = g.add_card_to_hand(0, catalog::kembas_banner());
    cast(&mut g, k, &[]);
    let rebel = g.battlefield_find(k).unwrap().attached_to.expect("on the Rebel");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, rebel), (4, 4));

    // Harmonious Archon: non-Archons are base 3/3; two Humans.
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let a = g.add_card_to_hand(0, catalog::harmonious_archon());
    cast(&mut g, a, &[]);
    assert_eq!((pt(&g, bear), count_named(&g, 0, "Human")), ((3, 3), 2));
    assert_eq!(pt(&g, a), (4, 5));

    // Hate Mirage: hasty copies of two opposing creatures.
    let mut g = main_phase();
    let x = g.add_card_to_battlefield(1, catalog::hill_giant());
    let y = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let h = g.add_card_to_hand(0, catalog::hate_mirage());
    cast(&mut g, h, &[Target::Permanent(x), Target::Permanent(y)]);
    assert_eq!((count_named(&g, 0, "Hill Giant"), count_named(&g, 0, "Grizzly Bears")), (1, 1));

    // Mace of the Valiant: a charge counter per entering creature.
    let mut g = main_phase();
    let m = g.add_card_to_battlefield(0, catalog::mace_of_the_valiant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    equip(&mut g, m, bear);
    let b2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, b2, &[]);
    assert_eq!(pt(&g, bear), (3, 3));

    // Otharri: an experience counter, then that many attacking Rebels.
    let mut g = main_phase();
    let o = g.add_card_to_battlefield(0, catalog::otharri_suns_glory());
    combat(&mut g, vec![Attack { attacker: o, target: AttackTarget::Player(1) }], 0, |_| {});
    assert_eq!(count_named(&g, 0, "Rebel"), 1);

    // Prava: tokens +1/+4 on your turn only.
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::prava_of_the_steel_legion());
    let t = soldier_token(&mut g, 0);
    assert_eq!(pt(&g, t), (2, 5));
    g.active_player_idx = 1;
    assert_eq!(pt(&g, t), (1, 1));

    // Silverwing Squadron: */* = your creatures; a Knight per opponent.
    let mut g = main_phase();
    let s = g.add_card_to_battlefield(0, catalog::silverwing_squadron());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, s), (2, 2));
    combat(&mut g, vec![Attack { attacker: s, target: AttackTarget::Player(1) }], 0, |_| {});
    assert_eq!(count_named(&g, 0, "Knight"), 1);

    // Vulshok Factory: {R} and a counter; the sacrifice makes a 1/1 Golem.
    let mut g = main_phase();
    let f = g.add_card_to_battlefield(0, catalog::vulshok_factory());
    activate(&mut g, f, 0, None).expect("tap for R");
    g.battlefield.find_by_id_mut(f).unwrap().tapped = false;
    activate(&mut g, f, 1, None).expect("make a Golem");
    let golem = g.battlefield.iter().find(|c| c.definition.name == "Golem").unwrap().id;
    assert_eq!(pt(&g, golem), (1, 1));
}

/// The bot answers an opponent's sweeper with a phase-out instant (Clever
/// Concealment) — no response path cast one before.
#[test]
fn bot_phases_out_its_board_under_a_wrath() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = main_phase();
    let giants: Vec<CardId> =
        (0..3).map(|_| g.add_card_to_battlefield(0, catalog::hill_giant())).collect();
    let cc = g.add_card_to_hand(0, catalog::clever_concealment());
    let wrath = g.add_card_to_hand(1, catalog::wrath_of_god());
    g.active_player_idx = 1;
    g.players[1].mana_pool.add(Color::White, 4);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: wrath, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("wrath");
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 4);
    match HeuristicBot::new().next_action(&g, 0) {
        Some(a @ GameAction::CastSpell { card_id, .. }) if card_id == cc => {
            g.perform_action(a).expect("cast");
        }
        other => panic!("expected Clever Concealment, got {other:?}"),
    }
    drain_stack(&mut g);
    // Phased out (CR 702.26), so the Wrath finds none of them.
    assert!(
        giants.iter().all(|id| !g.players[0].graveyard.iter().any(|c| c.id == *id)),
        "every Giant survives"
    );
}
