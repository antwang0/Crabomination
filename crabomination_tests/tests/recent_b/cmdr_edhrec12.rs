//! Commander: the 2026-10-12 rescan's ≤3-card EDHREC gaps of the top 600
//! (`decks::cmdr_edhrec12`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
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

fn cast_as(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
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

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_as(g, 0, id, targets)
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

fn tokens_named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

fn declare(g: &mut GameState, seat: usize, attacks: &[(CardId, usize)]) -> Result<(), String> {
    g.active_player_idx = seat;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.iter().map(|&(a, d)| Attack { attacker: a, target: AttackTarget::Player(d) }).collect(),
    ))
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn combat_damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn on_bf(g: &GameState, id: CardId) -> bool {
    g.battlefield_find(id).is_some()
}

/// Primal Surge: permanents keep coming off the top until a nonpermanent.
#[test]
fn primal_surge_keeps_putting_permanents() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 8]));
    for def in [catalog::grizzly_bears(), catalog::forest(), catalog::lightning_bolt(), catalog::grizzly_bears()] {
        g.add_card_to_library(0, def);
    }
    let s = g.add_card_to_hand(0, catalog::primal_surge());
    cast(&mut g, s, &[]).expect("cast");
    let mine = |n: &str| g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == n).count();
    assert_eq!((mine("Grizzly Bears"), mine("Forest")), (1, 1), "the two permanents above the Bolt");
    assert!(g.exile.iter().any(|c| c.definition.name == "Lightning Bolt"), "the Bolt stops it in exile");
    assert_eq!(g.players[0].library.len(), 1, "the bear under the Bolt stays");
}

/// Relic Golem: no attacks until an opponent has eight graveyard cards; its
/// {2},{T} mills two.
#[test]
fn relic_golem_waits_for_a_full_graveyard() {
    let mut g = pod(2);
    for _ in 0..20 {
        g.add_card_to_library(1, catalog::island());
    }
    let golem = ready(&mut g, 0, catalog::relic_golem());
    assert!(declare(&mut g, 0, &[(golem, 1)]).is_err(), "seven or fewer: can't attack");
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, golem, 0, Some(Target::Player(1))).expect("mill");
    assert_eq!(g.players[1].graveyard.len(), 2);
    for _ in 0..6 {
        let c = g.players[1].library.remove(0);
        g.players[1].graveyard.push(c);
    }
    g.clear_sickness(golem);
    g.battlefield_find_mut(golem).unwrap().tapped = false;
    declare(&mut g, 0, &[(golem, 1)]).expect("eight: attacks");
}

/// Shakedown Heavy: a defender who takes the deal untaps it out of combat and
/// its controller draws.
#[test]
fn shakedown_heavy_buys_its_way_out() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let heavy = ready(&mut g, 0, catalog::shakedown_heavy());
    let hand = g.players[0].hand.len();
    declare(&mut g, 0, &[(heavy, 1)]).expect("attack");
    assert_eq!(g.players[0].hand.len(), hand + 1, "the defender had it draw");
    assert!(!g.attacking.iter().any(|a| a.attacker == heavy), "removed from combat");
    assert!(!g.battlefield_find(heavy).unwrap().tapped, "and untapped");
}

/// CR 506.2 / 802.1 — Suppressor Skyguard shields only an attack that left
/// one of the attacker's other opponents alone.
#[test]
fn cr_506_2_suppressor_skyguard_shields_a_singled_out_player() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::suppressor_skyguard());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    let life = g.players[0].life;
    declare(&mut g, 1, &[(a, 0), (b, 0)]).expect("attack");
    combat_damage(&mut g);
    assert_eq!(g.players[0].life, life, "seat 2 was left alone: prevented");

    let mut g = pod(3);
    ready(&mut g, 0, catalog::suppressor_skyguard());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    let life = g.players[0].life;
    declare(&mut g, 1, &[(a, 0), (b, 2)]).expect("attack");
    combat_damage(&mut g);
    assert_eq!(g.players[0].life, life - 2, "everyone was attacked: no shield");
}

/// Zethi: multikicked twice, it exiles two instants with kick counters; an
/// attack copies them and casts the copies.
#[test]
fn zethi_stores_and_replays_instants() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::think_twice());
    }
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    let z = g.add_card_to_hand(0, catalog::zethi_arcane_blademaster());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellMultikicked {
        card_id: z,
        times: 2,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let kicked = g.exile.iter().filter(|c| c.counter_count(CounterType::Kick) == 1).count();
    assert_eq!(kicked, 2, "X = 2: two instants, each with a kick counter");
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 2]));
    g.clear_sickness(z);
    let hand = g.players[0].hand.len();
    declare(&mut g, 0, &[(z, 1)]).expect("attack");
    assert_eq!(g.players[0].hand.len(), hand + 2, "both Think Twice copies cast and drew");
    assert_eq!(g.exile.iter().filter(|c| c.counter_count(CounterType::Kick) == 1).count(), 2, "the originals stay");
}

/// CR 114.4 — Karn, Living Legacy's −7 emblem gives each artifact "{T}: 1
/// damage to any target"; its −1 digs X deep.
#[test]
fn cr_114_4_karn_living_legacy_emblem_pings() {
    let mut g = pod(2);
    let karn = ready(&mut g, 0, catalog::karn_living_legacy());
    g.battlefield_find_mut(karn).unwrap().add_counters(CounterType::Loyalty, 7);
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: karn,
        ability_index: 2,
        target: None,
        x_value: None,
    })
    .expect("-7");
    drain_stack(&mut g);
    assert_eq!(g.players[0].emblems.len(), 1);
    let rock = ready(&mut g, 0, catalog::sol_ring());
    let life = g.players[1].life;
    // Index 1: Sol Ring's printed mana ability, then the emblem's grant.
    activate(&mut g, rock, 1, Some(Target::Player(1))).expect("ping");
    assert_eq!(g.players[1].life, life - 1);
}

/// Fomori Vault: X = artifacts you control.
#[test]
fn fomori_vault_digs_by_artifacts() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let vault = ready(&mut g, 0, catalog::fomori_vault());
    ready(&mut g, 0, catalog::sol_ring());
    ready(&mut g, 0, catalog::sol_ring());
    g.add_card_to_hand(0, catalog::island());
    let hand = g.players[0].hand.len();
    activate(&mut g, vault, 1, None).expect("dig");
    assert_eq!(g.players[0].hand.len(), hand, "discarded one, kept one");
    assert_eq!(g.players[0].library.len(), 4);
}

/// Chrome Dome: other artifact creatures +1/+0; {5} makes a hasty copy that
/// is sacrificed at the next end step.
#[test]
fn chrome_dome_copies_an_artifact() {
    let mut g = pod(2);
    let dome = ready(&mut g, 0, catalog::chrome_dome());
    let golem = ready(&mut g, 0, catalog::relic_golem());
    assert_eq!(g.computed_permanent(golem).unwrap().power, 7);
    activate(&mut g, dome, 0, Some(perm(golem))).expect("copy");
    assert_eq!(tokens_named(&g, "Relic Golem"), 1);
}

/// CR 702.85 — Averna: a cascade's exiled land enters tapped instead of going
/// to the bottom.
#[test]
fn cr_702_85_averna_keeps_a_cascaded_land() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::averna_the_chaos_bloom());
    for def in [catalog::forest(), catalog::grizzly_bears(), catalog::island()] {
        g.add_card_to_library(0, def);
    }
    let p = g.add_card_to_hand(0, catalog::sakashimas_protege());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 4]));
    cast(&mut g, p, &[]).expect("cast");
    let forest = g.battlefield.iter().find(|c| c.definition.name == "Forest").expect("the Forest entered");
    assert!(forest.tapped && forest.controller == 0);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "and the hit was cast");
}

/// Release to the Wind: the exiled card's owner may cast it for free.
#[test]
fn release_to_the_wind_lets_the_owner_recast() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let r = g.add_card_to_hand(0, catalog::release_to_the_wind());
    cast(&mut g, r, &[perm(bear)]).expect("cast");
    assert!(g.exile.iter().any(|c| c.id == bear));
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    if let Err(e) =
        g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
    {
        panic!("free recast: {e:?}");
    }
    drain_stack(&mut g);
    assert!(on_bf(&g, bear));
}

/// Thunderous Snapper and Gilanra draw off big spells; Gilanra's only off its
/// own mana.
#[test]
fn snapper_and_gilanra_draw_off_big_spells() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::thunderous_snapper());
    let s = g.add_card_to_hand(0, catalog::primal_surge());
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(false); 4]));
    cast(&mut g, s, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
}

/// CR 106.6 — Gilanra draws only off its own mana: a six-drop funded by
/// another source's marked mana (Barracks of the Thousand's mark) draws
/// nothing; one funded by Gilanra's {G} draws.
#[test]
fn gilanra_reads_only_its_own_mark() {
    use crabomination::mana::SpendRestriction;
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::gilanra_caller_of_wirewood());
    for (mark, draws) in [(0u8, 0usize), (1, 1)] {
        let wurm = g.add_card_to_hand(0, catalog::wurmcoil_engine());
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool = Default::default();
        g.players[0].mana_pool.add_colorless(5);
        g.players[0].mana_pool.add_restricted(Color::Green, 1, SpendRestriction::MarksCast(mark));
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell { card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand - 1 + draws, "mark {mark}");
    }
}

/// Heartwood Storyteller: a noncreature spell lets each of the caster's
/// opponents draw.
#[test]
fn heartwood_storyteller_feeds_the_other_players() {
    let mut g = pod(3);
    for s in 0..3 {
        g.add_card_to_library(s, catalog::island());
    }
    ready(&mut g, 0, catalog::heartwood_storyteller());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 4]));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let hands: Vec<usize> = g.players.iter().map(|p| p.hand.len()).collect();
    cast_as(&mut g, 1, bolt, &[Target::Player(2)]).expect("bolt");
    assert_eq!(g.players[0].hand.len(), hands[0] + 1);
    assert_eq!(g.players[2].hand.len(), hands[2] + 1);
    assert_eq!(g.players[1].hand.len(), hands[1] - 1, "the caster draws nothing");
}

/// Rootweaver Druid: an opponent who searches gives one basic to the Druid's
/// controller.
#[test]
fn rootweaver_druid_takes_a_cut() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::forest());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let d = g.add_card_to_hand(0, catalog::rootweaver_druid());
    cast(&mut g, d, &[]).expect("cast");
    let forests = |s: usize| g.battlefield.iter().filter(|c| c.controller == s && c.definition.name == "Forest").count();
    assert_eq!((forests(0), forests(1)), (1, 2));
}

/// Haytham Kenway: one creature per opponent exiled until it leaves.
#[test]
fn haytham_kenway_holds_a_creature_per_opponent() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    let h = g.add_card_to_hand(0, catalog::haytham_kenway());
    cast(&mut g, h, &[perm(a), perm(b)]).expect("cast");
    assert!(!on_bf(&g, a) && !on_bf(&g, b));
    bolt_at(&mut g, h);
    assert!(on_bf(&g, a) && on_bf(&g, b), "back when Haytham leaves");
}

fn bolt_at(g: &mut GameState, id: CardId) {
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(g, 1, b, &[perm(id)]).expect("bolt");
}

/// Dream Fracture: the countered spell's controller draws, and so do you.
#[test]
fn dream_fracture_counters_and_both_draw() {
    let mut g = pod(2);
    for s in 0..2 {
        g.add_card_to_library(s, catalog::island());
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    let df = g.add_card_to_hand(0, catalog::dream_fracture());
    let (h0, h1, life) = (g.players[0].hand.len(), g.players[1].hand.len(), g.players[0].life);
    cast(&mut g, df, &[Target::Permanent(bolt)]).expect("fracture");
    assert_eq!(g.players[0].life, life);
    assert_eq!((g.players[0].hand.len(), g.players[1].hand.len()), (h0, h1 + 1));
}

/// Ghastly Gloomhunter: kicked, it enters with two +1/+1 counters.
#[test]
fn ghastly_gloomhunter_kicked_is_bigger() {
    let mut g = pod(2);
    let gh = g.add_card_to_hand(0, catalog::ghastly_gloomhunter());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked { card_id: gh, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("kicked");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(gh).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Batwing Brume paid with {B}: each player loses 1 per attacker they control.
#[test]
fn batwing_brume_black_drains_attackers() {
    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    g.attacking = vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ];
    let brume = g.add_card_to_hand(0, catalog::batwing_brume());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Black, 2);
    let life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: brume, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("brume");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
}

/// CR 302.6 — Frenzied Saddlebrute lets a summoning-sick creature attack its
/// controller's opponents, but not the Saddlebrute's controller.
#[test]
fn cr_302_6_frenzied_saddlebrute_waives_sickness_toward_its_opponents() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::frenzied_saddlebrute());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    declare(&mut g, 0, &[(mine, 1)]).expect("sick, but attacks an opponent");

    let mut g = pod(3);
    ready(&mut g, 0, catalog::frenzied_saddlebrute());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(declare(&mut g, 1, &[(theirs, 0)]).is_err(), "not at the Saddlebrute's controller");
    g.step = TurnStep::PreCombatMain;
    declare(&mut g, 1, &[(theirs, 2)]).expect("at another of seat 0's opponents");

    let mut g = pod(2);
    let sick = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(declare(&mut g, 0, &[(sick, 1)]).is_err(), "no Saddlebrute: still sick");
}

/// Winds of Qal Sisma with ferocious: only the opponents' creatures' combat
/// damage is prevented.
#[test]
fn winds_of_qal_sisma_ferocious_is_one_sided() {
    let mut g = pod(2);
    let big = ready(&mut g, 0, catalog::relic_golem());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let w = g.add_card_to_hand(0, catalog::winds_of_qal_sisma());
    cast(&mut g, w, &[]).expect("winds");
    g.attacking = vec![Attack { attacker: theirs, target: AttackTarget::Player(0) }];
    let life = g.players[0].life;
    combat_damage(&mut g);
    assert_eq!(g.players[0].life, life, "their bear's damage is prevented");
    let _ = big;
}

/// Ognis: a hasty attacker makes a tapped Treasure.
#[test]
fn ognis_taxes_hasty_attackers() {
    let mut g = pod(2);
    let o = ready(&mut g, 0, catalog::ognis_the_dragons_lash());
    declare(&mut g, 0, &[(o, 1)]).expect("attack");
    let t = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Treasure").expect("treasure");
    assert!(t.tapped);
}

/// Kalain: each Treasure mana spent on a creature is a +1/+1 counter.
#[test]
fn kalain_counts_treasure_mana() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::kalain_reclusive_painter());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.mark_from_treasure(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("bear");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// The Balrog: each permanent sacrificed this turn, by anyone, is {1} off.
#[test]
fn the_balrog_discounts_sacrifices() {
    let mut g = pod(2);
    g.players[0].permanents_sacrificed_this_turn = 2;
    g.players[1].permanents_sacrificed_this_turn = 3;
    let b = g.add_card_to_hand(0, catalog::the_balrog_durins_bane());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: b, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("five off: {B}{R}");
}

/// CR 601.2f — Even the Score sheds its {U}{U}{U} once an opponent has drawn
/// four cards this turn.
#[test]
fn cr_601_2f_even_the_score_colored_discount() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let e = g.add_card_to_hand(0, catalog::even_the_score());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    let act = |g: &mut GameState| {
        g.perform_action(GameAction::CastSpell { card_id: e, target: None, additional_targets: vec![], mode: None, x_value: Some(2) })
    };
    assert!(act(&mut g).is_err(), "no discount yet");
    g.players[1].cards_drawn_this_turn = 4;
    act(&mut g).expect("discounted to {X}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Game Plan: everyone wheels to seven and the spell exiles itself.
#[test]
fn game_plan_wheels_everyone() {
    let mut g = pod(2);
    for s in 0..2 {
        for _ in 0..10 {
            g.add_card_to_library(s, catalog::island());
        }
        g.add_card_to_graveyard(s, catalog::island());
    }
    let gp = g.add_card_to_hand(0, catalog::game_plan());
    cast(&mut g, gp, &[]).expect("cast");
    assert_eq!((g.players[0].hand.len(), g.players[1].hand.len()), (7, 7));
    assert!(g.players[0].graveyard.is_empty() && g.exile.iter().any(|c| c.id == gp));
}

/// Mister Negative: swap down to the opponent's lower life, draw the loss.
#[test]
fn mister_negative_trades_life_for_cards() {
    let mut g = pod(2);
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    g.players[0].life = 30;
    g.players[1].life = 24;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let m = g.add_card_to_hand(0, catalog::mister_negative());
    let hand = g.players[0].hand.len();
    cast(&mut g, m, &[Target::Player(1)]).expect("cast");
    assert_eq!((g.players[0].life, g.players[1].life), (24, 30));
    assert_eq!(g.players[0].hand.len(), hand - 1 + 6);
}

/// CR 704.5a — Pact Weapon attached: its controller survives at 0 life;
/// unattached, the SBA applies.
#[test]
fn cr_704_5a_pact_weapon_keeps_you_alive_while_attached() {
    let mut g = pod(2);
    let pact = ready(&mut g, 0, catalog::pact_weapon());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, pact, 0, Some(perm(bear))).expect("equip by discarding");
    assert_eq!(g.battlefield_find(pact).unwrap().attached_to, Some(bear));
    g.players[0].life = 0;
    g.check_state_based_actions();
    assert!(g.players[0].is_alive(), "attached: no CR 704.5a loss");
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, b, &[perm(bear)]).expect("kill the bearer");
    g.check_state_based_actions();
    assert!(!g.players[0].is_alive(), "unattached: the loss applies");
}

/// Tainted Sigil: gain the turn's total life lost.
#[test]
fn tainted_sigil_refunds_the_turns_losses() {
    let mut g = pod(3);
    let s = ready(&mut g, 0, catalog::tainted_sigil());
    g.players[1].life_lost_this_turn = 4;
    g.players[2].life_lost_this_turn = 3;
    let life = g.players[0].life;
    activate(&mut g, s, 0, None).expect("sigil");
    assert_eq!(g.players[0].life, life + 7);
}

/// CR 602.5b — Baba Lysaga sacrifices up to three; three card types pay off.
#[test]
fn cr_602_5b_baba_lysaga_wants_three_types() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let baba = ready(&mut g, 0, catalog::baba_lysaga_night_witch());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::sol_ring());
    ready(&mut g, 0, catalog::forest());
    ready(&mut g, 0, catalog::island());
    let (mine, theirs, hand) = (g.players[0].life, g.players[1].life, g.players[0].hand.len());
    activate(&mut g, baba, 0, None).expect("activate");
    assert_eq!((g.players[0].life, g.players[1].life), (mine + 3, theirs - 3));
    assert_eq!(g.players[0].hand.len(), hand + 3);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 2, "Baba and one land stay");
}

/// Erinis: attacking returns a land from the graveyard.
#[test]
fn erinis_regrows_a_land_on_attack() {
    let mut g = pod(2);
    let land = g.add_card_to_graveyard(0, catalog::forest());
    let e = ready(&mut g, 0, catalog::erinis_gloom_stalker());
    declare(&mut g, 0, &[(e, 1)]).expect("attack");
    assert!(on_bf(&g, land));
}

/// Aspect of Mongoose: shroud, and back to hand when it hits the graveyard.
#[test]
fn aspect_of_mongoose_returns_to_hand() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let a = g.add_card_to_hand(0, catalog::aspect_of_mongoose());
    cast(&mut g, a, &[perm(bear)]).expect("aura");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&crabomination::card::Keyword::Shroud));
    g.battlefield_find_mut(bear).unwrap().damage = 5;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == a));
}

/// CR 702.24 — Sheltering Ancient's cumulative upkeep feeds an opponent's
/// creature; with none to feed, it is sacrificed.
#[test]
fn cr_702_24_sheltering_ancient_feeds_the_opponent() {
    let mut g = pod(2);
    let anc = ready(&mut g, 0, catalog::sheltering_ancient());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::Upkeep;
    g.process_cumulative_upkeep();
    drain_stack(&mut g);
    assert!(on_bf(&g, anc));
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);

    let mut g = pod(2);
    let anc = ready(&mut g, 0, catalog::sheltering_ancient());
    g.step = TurnStep::Upkeep;
    g.process_cumulative_upkeep();
    drain_stack(&mut g);
    assert!(!on_bf(&g, anc), "no opposing creature: unpaid, sacrificed");
}

/// Tolarian Contempt: rejection counters, then one per opponent tucked.
#[test]
fn tolarian_contempt_tucks_the_rejected() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::tolarian_contempt());
    cast(&mut g, t, &[]).expect("cast");
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Rejection), 1);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[1].library.iter().any(|c| c.id == bear));
}

/// Regna's Sanction: a foe keeps one creature untapped.
#[test]
fn regnas_sanction_taps_all_but_one() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..3 {
        ready(&mut g, 1, catalog::grizzly_bears());
    }
    let r = g.add_card_to_hand(0, catalog::regnas_sanction());
    cast(&mut g, r, &[]).expect("cast");
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "you are a friend");
    let untapped = g.battlefield.iter().filter(|c| c.controller == 1 && !c.tapped).count();
    assert_eq!(untapped, 1, "seat 1, a foe, keeps one");
}

/// Yasharn (CR 118.3): no life or nonland sacrifice may pay a cost.
#[test]
fn cr_118_3_yasharn_blocks_life_and_sacrifice_costs() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::yasharn_implacable_earth());
    let stirge = ready(&mut g, 0, catalog::stirge());
    assert!(activate(&mut g, stirge, 0, None).is_err(), "pay 1 life + sacrifice: refused");
    let sigil = ready(&mut g, 1, catalog::tainted_sigil());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: sigil,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "a nonland self-sacrifice: refused for every player"
    );
}

/// Yasharn (CR 118.3) covers a SPELL's sacrifice costs too: Village Rites'
/// "sacrifice a creature" can't be paid, and Elder Deep-Fiend can't emerge.
#[test]
fn cr_118_3_yasharn_blocks_spell_sacrifice_costs() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::yasharn_implacable_earth());
    ready(&mut g, 0, catalog::grizzly_bears());
    let rites = g.add_card_to_hand(0, catalog::village_rites());
    assert!(cast(&mut g, rites, &[]).is_err(), "an additional sacrifice cost");
    let fiend = g.add_card_to_hand(0, catalog::elder_deep_fiend());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    let emerge = g.perform_action(GameAction::CastSpellAlternative {
        card_id: fiend,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    });
    assert!(emerge.is_err() || g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "no emerge sacrifice");
}

/// Arctic Treeline, Stirge, Dragonlair Spider, The Vision, Your Temple Is
/// Under Attack, Fascination and Aether Membrane — printed shapes.
#[test]
fn edhrec12_printed_shapes() {
    let t = catalog::arctic_treeline();
    assert!(t.supertypes.contains(&crabomination::card::Supertype::Snow));
    assert_eq!(t.activated_abilities.len(), 2);
    let mut g = pod(2);
    ready(&mut g, 0, catalog::dragonlair_spider());
    let b = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast_as(&mut g, 1, b, &[Target::Player(0)]).expect("bolt");
    assert_eq!(tokens_named(&g, "Insect"), 1, "Dragonlair Spider");
    let v = ready(&mut g, 0, catalog::the_vision());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let b = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, b, &[Target::Player(1)]).expect("bolt");
    let kws = g.computed_permanent(v).unwrap().keywords().to_vec();
    assert!(kws.contains(&crabomination::card::Keyword::DoubleStrike), "The Vision's first mode");
}

/// CR 104.3e / 611.2 — Vorpal Sword's {5}{B}{B}{B}: this turn the Sword's
/// granted trigger makes the player its equipped creature hits lose the game
/// (seat 1 of three; the game goes on). Unactivated, a hit is just damage.
#[test]
fn vorpal_sword_activated_makes_a_hit_lethal() {
    for activated in [false, true] {
        let mut g = pod(3);
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        let sword = ready(&mut g, 0, catalog::vorpal_sword());
        g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
        if activated {
            activate(&mut g, sword, 0, None).expect("activate");
        }
        declare(&mut g, 0, &[(bear, 1)]).expect("attack");
        combat_damage(&mut g);
        assert_eq!(g.players[1].eliminated, activated, "activated: {activated}");
        assert!(!g.players[2].eliminated);
    }
}
