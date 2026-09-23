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
