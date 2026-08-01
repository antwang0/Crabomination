//! Mercadian Masques (MMQ) gap closure, first wave.

use crabomination::card::{CounterType, Keyword};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn activate_x(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>, x: u32) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: Some(x),
    })
    .expect("activate");
    drain_stack(g);
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

/// The extra generic mana a static tax adds to `card` for `caster`.
fn tax_on(g: &GameState, card: CardId, caster: usize) -> u32 {
    let inst = g.players[caster].hand.iter().find(|c| c.id == card).expect("in hand");
    crabomination::game::actions::extra_cost_for_spell(g, caster, inst, None)
}

/// Put a fresh card straight into player 0's graveyard.
fn to_gy(g: &mut GameState, def: crabomination::card::CardDefinition) {
    let id = g.add_card_to_hand(0, def);
    let i = g.players[0].hand.iter().position(|c| c.id == id).unwrap();
    let c = g.players[0].hand.remove(i);
    g.players[0].graveyard.push(c);
}

fn stock(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::forest());
    }
}

/// Seat 0's `attacker` attacks seat 1 and is blocked by seat 1's `blocker`.
fn attack_and_block(g: &mut GameState, attacker: CardId, blocker: CardId) {
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    drain_stack(g);
}

/// Run the pending combat through the damage step.
fn combat_damage(g: &mut GameState) {
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

// ── Rebel / Mercenary tutor chains ──────────────────────────────────────────

/// Ramosian Sergeant fetches a Rebel with mana value 2 or less onto the
/// battlefield, and skips over one that's too expensive.
#[test]
fn ramosian_sergeant_fetches_a_cheap_rebel() {
    let mut g = two_player_game();
    let sarge = g.add_card_to_battlefield(0, catalog::ramosian_sergeant());
    g.battlefield_find_mut(sarge).unwrap().summoning_sick = false;
    g.add_card_to_library(0, catalog::jhovall_rider()); // {4}{W} — too big
    let prize = g.add_card_to_library(0, catalog::fresh_volunteers()); // {1}{W} — legal
    mana(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(prize))]));
    activate(&mut g, sarge, 0, None);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Fresh Volunteers"),
        "the mana-value-2 Rebel came down"
    );
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Jhovall Rider"));
}

/// Cateran Persuader only reaches mana value 1; the chain's next link reaches 2.
#[test]
fn mercenary_chain_respects_its_mana_value_cap() {
    let mut g = two_player_game();
    let persuader = g.add_card_to_battlefield(0, catalog::cateran_persuader());
    let brute = g.add_card_to_battlefield(0, catalog::cateran_brute());
    for id in [persuader, brute] {
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    }
    let fiend = g.add_card_to_library(0, catalog::misshapen_fiend()); // {1}{B} = 2
    mana(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(fiend)),
        DecisionAnswer::Search(Some(fiend)),
    ]));
    activate(&mut g, persuader, 0, None);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Misshapen Fiend"), "MV 2 > 1");
    activate(&mut g, brute, 0, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Misshapen Fiend"));
}

/// Cateran Overlord eats another creature to regenerate.
#[test]
fn cateran_overlord_regenerates_by_eating_a_creature() {
    let mut g = two_player_game();
    let lord = g.add_card_to_battlefield(0, catalog::cateran_overlord());
    let snack = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, lord, 0, None);
    assert!(g.battlefield_find(snack).is_none(), "the snack was the cost");
    assert!(g.battlefield_find(lord).unwrap().regeneration_shields > 0);
}

// ── Spellshapers ────────────────────────────────────────────────────────────

/// Bog Witch converts a card in hand into {B}{B}{B}.
#[test]
fn bog_witch_turns_a_card_into_three_black() {
    let mut g = two_player_game();
    let witch = g.add_card_to_battlefield(0, catalog::bog_witch());
    g.battlefield_find_mut(witch).unwrap().summoning_sick = false;
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    activate(&mut g, witch, 0, None);
    assert_eq!(g.players[0].hand.len(), 0, "the discard was a real cost");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3);
}

/// A Spellshaper can't be activated with an empty hand — the discard is a cost.
#[test]
fn spellshaper_needs_a_card_to_discard() {
    let mut g = two_player_game();
    let peddler = g.add_card_to_battlefield(0, catalog::tonic_peddler());
    g.battlefield_find_mut(peddler).unwrap().summoning_sick = false;
    g.players[0].hand.clear();
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: peddler,
            ability_index: 0,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "no card to pay the discard"
    );
}

/// Cackling Witch's {X} rides the activation's chosen X.
#[test]
fn cackling_witch_pumps_by_x() {
    let mut g = two_player_game();
    let witch = g.add_card_to_battlefield(0, catalog::cackling_witch());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(witch).unwrap().summoning_sick = false;
    g.add_card_to_hand(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    activate_x(&mut g, witch, 0, Some(Target::Permanent(bears)), 3);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 5);
}

/// Hammer Mage sweeps artifacts up to the chosen X, leaving bigger ones alone.
#[test]
fn hammer_mage_sweeps_artifacts_by_mana_value() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::hammer_mage());
    g.battlefield_find_mut(mage).unwrap().summoning_sick = false;
    let small = g.add_card_to_battlefield(1, catalog::iron_lance()); // {2}
    let big = g.add_card_to_battlefield(1, catalog::power_matrix()); // {4}
    g.add_card_to_hand(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    activate_x(&mut g, mage, 0, None, 2);
    assert!(g.battlefield_find(small).is_none());
    assert!(g.battlefield_find(big).is_some());
}

/// Overtaker's Threaten untaps, steals, and hastes the target.
#[test]
fn overtaker_steals_a_creature_for_the_turn() {
    let mut g = two_player_game();
    let taker = g.add_card_to_battlefield(0, catalog::overtaker());
    g.battlefield_find_mut(taker).unwrap().summoning_sick = false;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    g.add_card_to_hand(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    activate(&mut g, taker, 0, Some(Target::Permanent(victim)));
    let c = g.battlefield_find(victim).unwrap();
    assert_eq!(c.controller, 0);
    assert!(!c.tapped);
    assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::Haste));
}

// ── "Becomes blocked" ───────────────────────────────────────────────────────

/// Deepwood Wolverine punches back when it's blocked.
#[test]
fn deepwood_wolverine_grows_when_blocked() {
    let mut g = two_player_game();
    let wolverine = g.add_card_to_battlefield(0, catalog::deepwood_wolverine());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    attack_and_block(&mut g, wolverine, blocker);
    assert_eq!(g.computed_permanent(wolverine).unwrap().power, 3);
}

/// Saprazzan Raider bounces itself out of a block.
#[test]
fn saprazzan_raider_bails_on_a_block() {
    let mut g = two_player_game();
    let raider = g.add_card_to_battlefield(0, catalog::saprazzan_raider());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    attack_and_block(&mut g, raider, blocker);
    assert!(g.battlefield_find(raider).is_none(), "returned to hand");
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Alley Grifters strips a card from the defending player when blocked.
#[test]
fn alley_grifters_mugs_the_defender() {
    let mut g = two_player_game();
    let grifters = g.add_card_to_battlefield(0, catalog::alley_grifters());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    attack_and_block(&mut g, grifters, blocker);
    assert_eq!(g.players[1].hand.len(), 0);
}

/// Ignoble Soldier's damage is blanked once it's blocked.
#[test]
fn ignoble_soldier_deals_no_damage_after_being_blocked() {
    let mut g = two_player_game();
    let soldier = g.add_card_to_battlefield(0, catalog::ignoble_soldier()); // 3/1
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    attack_and_block(&mut g, soldier, blocker);
    combat_damage(&mut g);
    assert!(g.battlefield_find(blocker).is_some(), "the 3 damage was prevented");
    assert!(g.battlefield_find(soldier).is_none(), "it still took 2");
}

// ── Auras / prevention ──────────────────────────────────────────────────────

/// Muzzle blanks the host's damage in both combat and noncombat.
#[test]
fn muzzle_blanks_the_hosts_damage() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let muzzle = g.add_card_to_hand(1, catalog::muzzle());
    mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    cast(&mut g, muzzle, Some(Target::Permanent(attacker)));
    attack_and_block(&mut g, attacker, blocker);
    combat_damage(&mut g);
    assert!(g.battlefield_find(blocker).is_some(), "muzzled");
    assert!(g.battlefield_find(attacker).is_some(), "it still took only 2");
}

/// Inviolability blanks damage *to* the host but leaves its own damage alone.
#[test]
fn inviolability_only_blocks_incoming_damage() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(1, catalog::inviolability());
    mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    cast(&mut g, aura, Some(Target::Permanent(blocker)));
    attack_and_block(&mut g, attacker, blocker);
    combat_damage(&mut g);
    assert!(g.battlefield_find(blocker).is_some(), "took no damage");
    assert!(g.battlefield_find(attacker).is_none(), "but still dealt its own");
}

/// Ancestral Mask counts every *other* enchantment on the battlefield, both
/// sides included.
#[test]
fn ancestral_mask_counts_enchantments_on_both_sides() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mask = g.add_card_to_hand(0, catalog::ancestral_mask());
    g.add_card_to_battlefield(0, catalog::moonlit_wake());
    g.add_card_to_battlefield(1, catalog::intimidation());
    mana(&mut g, 0);
    cast(&mut g, mask, Some(Target::Permanent(bears)));
    // Two other enchantments → +4/+4 on a 2/2.
    assert_eq!(g.computed_permanent(bears).unwrap().power, 6);
    let _ = mask;
}

/// Ivory Mask gives its controller shroud — even their own spells can't target
/// them (CR 702.18).
#[test]
fn ivory_mask_stops_even_your_own_targeting() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ivory_mask());
    assert!(g.check_target_legality(&Target::Player(0), 1).is_err(), "opponents can't");
    assert!(g.check_target_legality(&Target::Player(0), 0).is_err(), "and neither can you");
    assert!(g.check_target_legality(&Target::Player(1), 0).is_ok());
}

// ── Statics ─────────────────────────────────────────────────────────────────

/// Magistrate's Veto stops white and blue creatures from blocking.
#[test]
fn magistrates_veto_stops_white_and_blue_blockers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::magistrates_veto());
    let attacker = g.add_card_to_battlefield(1, catalog::hill_giant());
    let white = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let green = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(attacker).unwrap().summoning_sick = false;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    assert!(g.declare_blockers(vec![(white, attacker)]).is_err(), "white can't block");
    g.declare_blockers(vec![(green, attacker)]).expect("green can");
}

/// High Seas taxes red and green creature spells only.
#[test]
fn high_seas_taxes_red_and_green_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::high_seas());
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears()); // green {1}{G}
    let lions = g.add_card_to_hand(1, catalog::savannah_lions()); // white {W}
    assert_eq!(tax_on(&g, bears, 1), 1, "green creature spell taxed");
    assert_eq!(tax_on(&g, lions, 1), 0, "white creature spell isn't");
}

/// Lumbering Satyr hands forestwalk to every creature, both sides included.
#[test]
fn lumbering_satyr_gives_everyone_forestwalk() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lumbering_satyr());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(theirs)
            .unwrap()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::Landwalk(crabomination::card::LandType::Forest)))
    );
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// A storage land banks counters and cashes any number of them at once.
#[test]
fn storage_land_banks_then_pays_out() {
    let mut g = two_player_game();
    let bazaar = g.add_card_to_battlefield(0, catalog::mercadian_bazaar());
    for _ in 0..3 {
        g.battlefield_find_mut(bazaar).unwrap().tapped = false;
        activate(&mut g, bazaar, 0, None);
    }
    assert_eq!(g.battlefield_find(bazaar).unwrap().counter_count(CounterType::Storage), 3);
    g.battlefield_find_mut(bazaar).unwrap().tapped = false;
    activate_x(&mut g, bazaar, 1, None, 3);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
    assert_eq!(g.battlefield_find(bazaar).unwrap().counter_count(CounterType::Depletion), 0);
}

/// A depletion land enters with two counters and sacrifices itself on the
/// second tap.
#[test]
fn depletion_land_dies_when_the_last_counter_is_spent() {
    let mut g = two_player_game();
    let bog = g.add_card_to_hand(0, catalog::peat_bog());
    g.perform_action(GameAction::PlayLand(bog)).expect("play land");
    drain_stack(&mut g);
    let c = g.battlefield_find(bog).unwrap();
    assert!(c.tapped, "enters tapped");
    assert_eq!(c.counter_count(CounterType::Depletion), 2);

    g.battlefield_find_mut(bog).unwrap().tapped = false;
    activate(&mut g, bog, 0, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 2);
    assert!(g.battlefield_find(bog).is_some(), "one counter left");

    g.battlefield_find_mut(bog).unwrap().tapped = false;
    activate(&mut g, bog, 0, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 4);
    assert!(g.battlefield_find(bog).is_none(), "spent — sacrificed");
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Sever Soul kills a nonblack creature and refunds its toughness as life.
#[test]
fn sever_soul_pays_back_the_victims_toughness() {
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let soul = g.add_card_to_hand(0, catalog::sever_soul());
    let life = g.players[0].life;
    mana(&mut g, 0);
    cast(&mut g, soul, Some(Target::Permanent(giant)));
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.players[0].life, life + 3);
}

/// Wave of Reckoning has every creature shoot itself for its own power.
#[test]
fn wave_of_reckoning_makes_every_creature_shoot_itself() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 — dies
    let wall = g.add_card_to_battlefield(1, catalog::alabaster_wall()); // 0/4 — lives
    let wave = g.add_card_to_hand(0, catalog::wave_of_reckoning());
    mana(&mut g, 0);
    cast(&mut g, wave, None);
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.battlefield_find(wall).is_some());
}

/// Collective Unconscious draws one card per creature you control.
#[test]
fn collective_unconscious_draws_per_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hill_giant());
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // theirs — not counted
    stock(&mut g, 0, 5);
    let spell = g.add_card_to_hand(0, catalog::collective_unconscious());
    mana(&mut g, 0);
    let before = g.players[0].hand.len();
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].hand.len(), before - 1 + 2);
}

/// Ghoul's Feast scales off creature cards in your graveyard.
#[test]
fn ghouls_feast_scales_off_the_graveyard() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        to_gy(&mut g, catalog::hill_giant());
    }
    to_gy(&mut g, catalog::lightning_bolt()); // not a creature
    let feast = g.add_card_to_hand(0, catalog::ghouls_feast());
    mana(&mut g, 0);
    cast(&mut g, feast, Some(Target::Permanent(bears)));
    assert_eq!(g.computed_permanent(bears).unwrap().power, 5);
}

/// Hunted Wumpus lets *other* players cheat a creature in, not its controller.
#[test]
fn hunted_wumpus_only_helps_the_opponents() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::hill_giant());
    g.add_card_to_hand(1, catalog::hill_giant());
    let wumpus = g.add_card_to_hand(0, catalog::hunted_wumpus());
    mana(&mut g, 0);
    cast(&mut g, wumpus, None);
    assert_eq!(g.players[0].hand.len(), 1, "your own Giant stayed put");
    assert_eq!(g.players[1].hand.len(), 0, "theirs came down");
}

/// Forced March sweeps creatures up to the chosen X.
#[test]
fn forced_march_sweeps_by_mana_value() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant()); // MV 4
    let march = g.add_card_to_hand(0, catalog::forced_march());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: march,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.battlefield_find(giant).is_some());
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// A Ramos stone taps for its colour and can be cracked for one more.
#[test]
fn ramos_stone_taps_or_cracks_for_mana() {
    let mut g = two_player_game();
    let heart = g.add_card_to_battlefield(0, catalog::heart_of_ramos());
    activate(&mut g, heart, 0, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    activate(&mut g, heart, 1, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2);
    assert!(g.battlefield_find(heart).is_none(), "sacrificed as the cost");
}

/// Task Force toughens up whenever anything targets it.
#[test]
fn task_force_hardens_when_targeted() {
    let mut g = two_player_game();
    let tf = g.add_card_to_battlefield(0, catalog::task_force());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Permanent(tf)));
    // 1/3 + 0/+3, minus the 3 damage → survives.
    assert!(g.battlefield_find(tf).is_some());
    assert_eq!(g.computed_permanent(tf).unwrap().toughness, 6);
}

/// Skulking Fugitive dies the moment anything targets it.
#[test]
fn skulking_fugitive_folds_to_being_targeted() {
    let mut g = two_player_game();
    let fugitive = g.add_card_to_battlefield(0, catalog::skulking_fugitive());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Permanent(fugitive)));
    assert!(g.battlefield_find(fugitive).is_none());
}
