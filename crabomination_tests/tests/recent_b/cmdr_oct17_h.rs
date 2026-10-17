//! Commander: the 2026-10-17 (`01Htv6cF`) one-card-gap closers
//! (`decks::cmdr_oct17_h`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].hostile_player_targets = true;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>, x: Option<u32>) -> Result<(), GameError> {
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: more, mode: None, x_value: x })?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    cast_with(g, id, target, vec![], None)
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: index, target, additional_targets: vec![], x_value: None, mode: None })?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).unwrap();
    (cp.power, cp.toughness)
}

/// The Karlov dual cyclers: {2}{W}{U}, {T}, sacrifice: draw a card.
#[test]
fn skybridge_towers_sacrifices_to_draw() {
    let mut g = pod(2);
    let land = ready(&mut g, 0, catalog::skybridge_towers());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    activate(&mut g, land, 2, None).expect("sac-draw");
    assert!(g.battlefield_find(land).is_none());
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Flaming Fist Officer grows when another creature you control leaves —
/// bounced, not only dying — and not for an opponent's.
#[test]
fn flaming_fist_officer_counts_any_departure() {
    let mut g = pod(2);
    let officer = ready(&mut g, 0, catalog::flaming_fist_officer());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    for t in [bears, theirs] {
        let s = g.add_card_to_hand(0, catalog::machine_over_matter());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, s, Some(Target::Permanent(t))).expect("bounce");
    }
    assert_eq!(g.battlefield_find(officer).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Machine Over Matter costs {U} with an artifact creature out.
#[test]
fn machine_over_matter_is_cheaper_with_an_artifact_creature() {
    let mut g = pod(2);
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let s = g.add_card_to_hand(0, catalog::machine_over_matter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    assert!(cast(&mut g, s, Some(Target::Permanent(ring))).is_err(), "{{1}}{{U}} with one mana");
    ready(&mut g, 0, catalog::afterthought_sentry());
    cast(&mut g, s, Some(Target::Permanent(ring))).expect("{U} with a Sentry");
    assert!(g.battlefield_find(ring).is_none());
}

/// Keldon Warlord counts non-Wall creatures; Serpent of the Endless Sea
/// counts Islands (CR 604.3 characteristic-defining abilities).
#[test]
fn keldon_warlord_and_serpent_count_their_board() {
    let mut g = pod(2);
    let warlord = ready(&mut g, 0, catalog::keldon_warlord());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::roving_keep());
    assert_eq!(pt(&g, warlord), (2, 2), "itself and the Bears, not the Wall");
    let serpent = ready(&mut g, 0, catalog::serpent_of_the_endless_sea());
    for _ in 0..3 {
        ready(&mut g, 0, catalog::island());
    }
    assert_eq!(pt(&g, serpent), (3, 3));
}

/// Ring of Three Wishes: three wish counters, each activation spends one.
#[test]
fn ring_of_three_wishes_spends_wish_counters() {
    let mut g = pod(2);
    let ring = g.add_card_to_hand(0, catalog::ring_of_three_wishes());
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, ring, None).expect("Ring");
    assert_eq!(g.battlefield_find(ring).unwrap().counter_count(CounterType::Wish), 3);
    g.players[0].mana_pool.add_colorless(5);
    let hand = g.players[0].hand.len();
    activate(&mut g, ring, 0, None).expect("wish");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.battlefield_find(ring).unwrap().counter_count(CounterType::Wish), 2);
}

/// Serra's Boon: +1/+2 on a white creature, -2/-1 otherwise.
#[test]
fn serras_boon_depends_on_color() {
    for (def, want) in [(catalog::savannah_lions(), (3, 3)), (catalog::grizzly_bears(), (0, 1))] {
        let mut g = pod(2);
        let c = ready(&mut g, 1, def);
        let s = g.add_card_to_hand(0, catalog::serras_boon());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, s, Some(Target::Permanent(c))).expect("Boon");
        assert_eq!(pt(&g, c), want);
    }
}

/// Consulate Crackdown exiles opponents' artifacts until it leaves (CR
/// 610.3), then they return.
#[test]
fn consulate_crackdown_holds_artifacts_until_it_leaves() {
    let mut g = pod(3);
    ready(&mut g, 1, catalog::sol_ring());
    ready(&mut g, 2, catalog::sol_ring());
    let mine = ready(&mut g, 0, catalog::sol_ring());
    let c = g.add_card_to_hand(0, catalog::consulate_crackdown());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, c, None).expect("Crackdown");
    let rings = |g: &GameState| g.battlefield.iter().filter(|p| p.definition.name == "Sol Ring").count();
    assert_eq!(rings(&g), 1);
    assert!(g.battlefield_find(mine).is_some());
    let s = g.add_card_to_hand(0, catalog::machine_over_matter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, s, Some(Target::Permanent(c))).expect("bounce Crackdown");
    assert_eq!(rings(&g), 3);
}

/// Break Under Pressure takes the opponent's greatest-mana-value creature.
#[test]
fn break_under_pressure_takes_the_biggest() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let crusher = ready(&mut g, 1, catalog::copper_host_crusher());
    let life = g.players[0].life;
    let s = g.add_card_to_hand(0, catalog::break_under_pressure());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, s, Some(Target::Player(1))).expect("Break");
    assert!(g.battlefield_find(crusher).is_none(), "the hexproof 8/8 goes — no creature is targeted");
    assert!(g.battlefield_find(bears).is_some());
    assert_eq!(g.players[0].life, life + 2);
}

/// Broadcast Takeover borrows every opponent artifact for the turn.
#[test]
fn broadcast_takeover_borrows_artifacts() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::afterthought_sentry());
    let b2 = ready(&mut g, 2, catalog::sol_ring());
    g.battlefield_find_mut(a).unwrap().tapped = true;
    let s = g.add_card_to_hand(0, catalog::broadcast_takeover());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, s, None).expect("Takeover");
    for id in [a, b2] {
        let p = g.battlefield_find(id).unwrap();
        assert_eq!(p.controller, 0);
        assert!(!p.tapped);
    }
}

/// Unwilling Recruit adds +X/+0 to the borrowed creature.
#[test]
fn unwilling_recruit_pumps_by_x() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::unwilling_recruit());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(2);
    cast_with(&mut g, s, Some(Target::Permanent(bears)), vec![], Some(2)).expect("Recruit");
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0);
    assert_eq!(pt(&g, bears), (4, 2));
}

/// Undercity Uprising: deathtouch first, so a 1/1 fighting an 8/8 kills it.
#[test]
fn undercity_uprising_fights_with_deathtouch() {
    let mut g = pod(2);
    let ninth = ready(&mut g, 0, catalog::ninth_bridge_patrol());
    let big = ready(&mut g, 1, catalog::keldon_warlord());
    for _ in 0..5 {
        ready(&mut g, 1, catalog::grizzly_bears());
    }
    let s = g.add_card_to_hand(0, catalog::undercity_uprising());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_with(&mut g, s, Some(Target::Permanent(ninth)), vec![Target::Permanent(big)], None).expect("Uprising");
    assert!(g.battlefield_find(big).is_none(), "deathtouch damage");
}

/// Dundoolin Weaver's return needs three creatures as it enters (CR 603.4).
#[test]
fn dundoolin_weaver_needs_three_creatures() {
    for (others, back) in [(1usize, false), (2, true)] {
        let mut g = pod(2);
        let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        for _ in 0..others {
            ready(&mut g, 0, catalog::grizzly_bears());
        }
        let w = g.add_card_to_hand(0, catalog::dundoolin_weaver());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, w, None).expect("Weaver");
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == bears), back, "{others} other creatures");
    }
}

/// Anchor to the Aether puts the creature on top of its owner's library.
#[test]
fn anchor_to_the_aether_tops_the_creature() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::anchor_to_the_aether());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, s, Some(Target::Permanent(bears))).expect("Anchor");
    assert!(g.players[1].library.iter().any(|c| c.id == bears));
    assert!(g.battlefield_find(bears).is_none());
}

// ── Second round (`decks::cmdr_oct17_i`) ───────────────────────────────────

/// Gift of Growth: +2/+2, or +4/+4 kicked (CR 702.33).
#[test]
fn gift_of_growth_scales_with_kicker() {
    for (kicked, want) in [(false, (4, 4)), (true, (6, 6))] {
        let mut g = pod(2);
        let bears = ready(&mut g, 0, catalog::grizzly_bears());
        g.battlefield_find_mut(bears).unwrap().tapped = true;
        let s = g.add_card_to_hand(0, catalog::gift_of_growth());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        let target = Some(Target::Permanent(bears));
        if kicked {
            g.perform_action(GameAction::CastSpellKicked { card_id: s, target, additional_targets: vec![], mode: None, x_value: None }).expect("kicked");
            drain_stack(&mut g);
        } else {
            cast(&mut g, s, target).expect("Gift");
        }
        assert_eq!(pt(&g, bears), want, "kicked: {kicked}");
        assert!(!g.battlefield_find(bears).unwrap().tapped);
    }
}

/// Henge Walker's adamant counter needs three mana of one color (CR 207.2c).
#[test]
fn henge_walker_adamant_counter() {
    for (red, counters) in [(2, 0u32), (3, 1)] {
        let mut g = pod(2);
        let h = g.add_card_to_hand(0, catalog::henge_walker());
        g.players[0].mana_pool.add(Color::Red, red);
        g.players[0].mana_pool.add(Color::Blue, 3 - red);
        cast(&mut g, h, None).expect("Henge Walker");
        assert_eq!(g.battlefield_find(h).unwrap().counter_count(CounterType::PlusOnePlusOne), counters, "{red} red");
    }
}

/// Golden Urn gains the charge counters it had as it was sacrificed (CR
/// 608.2h last-known information).
#[test]
fn golden_urn_reads_counters_last_known() {
    let mut g = pod(2);
    let urn = ready(&mut g, 0, catalog::golden_urn());
    g.battlefield_find_mut(urn).unwrap().add_counters(CounterType::Charge, 3);
    let life = g.players[0].life;
    activate(&mut g, urn, 0, None).expect("Urn");
    assert!(g.battlefield_find(urn).is_none());
    assert_eq!(g.players[0].life, life + 3);
}

/// Meadowboon evoked still leaves its counters on the target player's
/// creatures (CR 702.74a: the sacrifice is a leave-the-battlefield event).
#[test]
fn meadowboon_evoke_leaves_counters() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::meadowboon());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: m,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("evoke");
    for _ in 0..4 {
        drain_stack(&mut g);
        if g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne) > 0 {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    assert!(g.battlefield_find(m).is_none(), "evoked: sacrificed");
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Bolg's Company: haste only beside another Goblin; its mana ability eats a
/// Goblin for {B}{R}.
#[test]
fn bolgs_company_needs_another_goblin() {
    let mut g = pod(2);
    let bolg = g.add_card_to_battlefield(0, catalog::bolgs_company());
    let haste = |g: &GameState| g.computed_permanent(bolg).unwrap().keywords().contains(&crabomination::card::Keyword::Haste);
    assert!(!haste(&g));
    let gob = ready(&mut g, 0, catalog::goblin_mountaineer());
    assert!(haste(&g));
    activate(&mut g, bolg, 0, None).expect("mana");
    assert!(g.battlefield_find(gob).is_none());
    assert_eq!(g.players[0].mana_pool.amount(Color::Black) + g.players[0].mana_pool.amount(Color::Red), 2);
}

/// Creakwood Safewright sheds a -1/-1 counter at your end step only with an
/// Elf card in your graveyard.
#[test]
fn creakwood_safewright_recovers_with_an_elf_in_the_yard() {
    for (elf, want) in [(false, 3u32), (true, 2)] {
        let mut g = pod(2);
        let c = g.add_card_to_hand(0, catalog::creakwood_safewright());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, c, None).expect("Safewright");
        if elf {
            g.add_card_to_graveyard(0, catalog::llanowar_elves());
        }
        let mut guard = 0;
        while g.active_player_idx == 0 && guard < 50 {
            let _ = g.perform_action(GameAction::PassPriority);
            drain_stack(&mut g);
            guard += 1;
        }
        assert_eq!(g.battlefield_find(c).unwrap().counter_count(CounterType::MinusOneMinusOne), want, "elf: {elf}");
    }
}

/// Mindwrack Liege pumps other blue and red creatures, each once.
#[test]
fn mindwrack_liege_pumps_blue_and_red() {
    let mut g = pod(2);
    let liege = ready(&mut g, 0, catalog::mindwrack_liege());
    let red = ready(&mut g, 0, catalog::goblin_mountaineer());
    let green = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(pt(&g, red), (2, 2));
    assert_eq!(pt(&g, green), (2, 2));
    assert_eq!(pt(&g, liege), (4, 4));
}

/// Solitary Cell holds a cheap opponent permanent until it leaves.
#[test]
fn solitary_cell_exiles_until_it_leaves() {
    let mut g = pod(2);
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let cell = g.add_card_to_hand(0, catalog::solitary_cell());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, cell, None).expect("Cell");
    let _ = g.perform_action(GameAction::PassPriority);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none());
    let s = g.add_card_to_hand(0, catalog::machine_over_matter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, s, Some(Target::Permanent(cell))).expect("bounce Cell");
    assert!(g.battlefield.iter().any(|p| p.definition.name == "Sol Ring" && p.controller == 1));
}

/// Goblinslide pays {1} per noncreature spell for a hasty Goblin.
#[test]
fn goblinslide_pays_for_goblins() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::goblinslide());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    let s = g.add_card_to_hand(0, catalog::join_shields());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, s, None).expect("Join Shields");
    assert_eq!(g.battlefield.iter().filter(|p| p.definition.name == "Goblin").count(), 1);
}

/// I Am Iron Man makes a nonartifact creature an artifact creature (CR
/// 205.1b), only until end of turn.
#[test]
fn i_am_iron_man_adds_the_artifact_type() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::i_am_iron_man());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, s, Some(Target::Permanent(bears))).expect("I Am Iron Man");
    let cp = g.computed_permanent(bears).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Artifact));
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Flying));
}
