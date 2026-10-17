//! Commander: the 2026-10-17 (`019Y6J4R`) eleventh closer batch
//! (`decks::cmdr_019y_k`).

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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn kill(g: &mut GameState, id: CardId) {
    let destroy = crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::ExactObjects(vec![id]),
    };
    let evs = g.resolve_effect(&destroy, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Seat 1 casts a Grizzly Bears with exactly its cost, leaving it on the
/// stack with nothing left to pay a counter's tax.
fn opposing_bear(g: &mut GameState) -> CardId {
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("bear");
    g.priority.player_with_priority = 0;
    bear
}

fn cast_at(g: &mut GameState, id: CardId, target: Target) -> Result<(), GameError> {
    cast(g, id, Some(target))
}

/// CR 702.8a — Whisper Agent has flash: it can be cast on an opponent's turn.
#[test]
fn whisper_agent_flashes_in_on_an_opponents_turn() {
    let mut g = pod(2);
    let wa = g.add_card_to_hand(0, catalog::whisper_agent());
    flood(&mut g, 0);
    g.active_player_idx = 1;
    cast(&mut g, wa, None).expect("flash");
    assert!(g.battlefield_find(wa).is_some());
}

/// Scribe of the Mindful sacrifices itself to regrow an instant.
#[test]
fn scribe_of_the_mindful_regrows_an_instant() {
    let mut g = pod(2);
    let scribe = ready(&mut g, 0, catalog::scribe_of_the_mindful());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    activate(&mut g, scribe, 0, Some(Target::Permanent(bolt))).expect("activate");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
    assert!(g.battlefield_find(scribe).is_none(), "sacrificed as a cost");
}

/// CR 700.4 — Deadbridge Shaman's death makes the opponent discard.
#[test]
fn deadbridge_shaman_dies_into_a_discard() {
    let mut g = pod(2);
    let ds = ready(&mut g, 0, catalog::deadbridge_shaman());
    g.add_card_to_hand(1, catalog::forest());
    kill(&mut g, ds);
    assert!(g.players[1].hand.is_empty());
}

/// Noxious Dragon's death destroys a small creature (mana value 3 or less).
#[test]
fn noxious_dragon_takes_a_small_creature_with_it() {
    let mut g = pod(2);
    let nd = ready(&mut g, 0, catalog::noxious_dragon());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    kill(&mut g, nd);
    assert!(g.battlefield_find(bear).is_none());
}

/// Wild Wanderer fetches a basic land onto the battlefield tapped.
#[test]
fn wild_wanderer_fetches_a_tapped_basic() {
    let mut g = pod(2);
    let ww = g.add_card_to_hand(0, catalog::wild_wanderer());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, ww, None).expect("cast");
    let land = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Forest").expect("a Forest");
    assert!(land.tapped);
}

/// Weaver of Currents taps for {C}{C}.
#[test]
fn weaver_of_currents_adds_two_colorless() {
    let mut g = pod(2);
    let wc = ready(&mut g, 0, catalog::weaver_of_currents());
    activate(&mut g, wc, 0, None).expect("mana");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2);
}

/// Contract Killing destroys and pays out two Treasures.
#[test]
fn contract_killing_destroys_and_makes_treasure() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let ck = g.add_card_to_hand(0, catalog::contract_killing());
    flood(&mut g, 0);
    cast_at(&mut g, ck, Target::Permanent(bear)).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count(), 2);
}

/// Aerial Guide's attack trigger gives another attacker flying (CR 508.1).
#[test]
fn aerial_guide_lends_flying_to_another_attacker() {
    let mut g = pod(2);
    let ag = ready(&mut g, 0, catalog::aerial_guide());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: ag, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&crabomination::card::Keyword::Flying));
}

/// Viashino Branchrider unkicked: a hasty 1/1 that pumps for {2}{R}.
#[test]
fn viashino_branchrider_pumps() {
    let mut g = pod(2);
    let vb = g.add_card_to_hand(0, catalog::viashino_branchrider());
    flood(&mut g, 0);
    cast(&mut g, vb, None).expect("cast");
    activate(&mut g, vb, 0, None).expect("pump");
    let cp = g.computed_permanent(vb).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 1));
}

/// CR 603.4 — Lion Vulture's end-step trigger checks that an opponent lost life.
#[test]
fn lion_vulture_feeds_on_lost_life() {
    let mut g = pod(2);
    let lv = ready(&mut g, 0, catalog::lion_vulture());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(lv).unwrap().counter_count(CounterType::PlusOnePlusOne), 0, "nobody lost life");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    cast_at(&mut g, bolt, Target::Player(1)).expect("bolt");
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(lv).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 702.12b — Soul Sear strips indestructible, so 5 damage kills Darksteel Myr.
#[test]
fn soul_sear_kills_through_indestructible() {
    let mut g = pod(2);
    let myr = ready(&mut g, 1, catalog::darksteel_myr());
    let ss = g.add_card_to_hand(0, catalog::soul_sear());
    flood(&mut g, 0);
    cast_at(&mut g, ss, Target::Permanent(myr)).expect("cast");
    assert!(g.battlefield_find(myr).is_none());
}

/// Sage's Dousing counters a spell whose controller can't pay {3}, and draws
/// with a Wizard on board.
#[test]
fn sages_dousing_counters_and_draws_with_a_wizard() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::coastal_wizard());
    let bear = opposing_bear(&mut g);
    let sd = g.add_card_to_hand(0, catalog::sages_dousing());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    cast_at(&mut g, sd, Target::Permanent(bear)).expect("cast");
    assert!(g.battlefield_find(bear).is_none(), "countered");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
}

/// Calculated Dismissal counters an unpaid spell; spell mastery adds a scry.
#[test]
fn calculated_dismissal_counters_an_unpaid_spell() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let bear = opposing_bear(&mut g);
    let cd = g.add_card_to_hand(0, catalog::calculated_dismissal());
    flood(&mut g, 0);
    cast_at(&mut g, cd, Target::Permanent(bear)).expect("cast");
    assert!(g.battlefield_find(bear).is_none(), "countered");
}

/// Raid (CR 207.2c) — Ruin Raider draws off the top only after you attacked,
/// and costs life equal to the card's mana value.
#[test]
fn ruin_raider_pays_life_for_its_card() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ruin_raider());
    g.step = TurnStep::End;
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "no attack, no card");
    g.players[0].attacked_this_turn = true;
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::hill_giant());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == top));
    assert_eq!(g.players[0].life, 20 - 4);
}

/// Sifter Wurm gains life equal to the top card's mana value after its scry.
#[test]
fn sifter_wurm_gains_the_top_cards_mana_value() {
    let mut g = pod(2);
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::hill_giant());
    let sw = g.add_card_to_hand(0, catalog::sifter_wurm());
    flood(&mut g, 0);
    cast(&mut g, sw, None).expect("cast");
    assert_eq!(g.players[0].life, 24);
}

/// Armor of Faith: +1/+1, and {W} for +0/+1 more this turn.
#[test]
fn armor_of_faith_pads_the_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aof = g.add_card_to_hand(0, catalog::armor_of_faith());
    flood(&mut g, 0);
    cast_at(&mut g, aof, Target::Permanent(bear)).expect("cast");
    activate(&mut g, aof, 0, None).expect("pump");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4));
}

/// Aether Meltdown: two energy and -4/-0 on the enchanted creature.
#[test]
fn aether_meltdown_shrinks_and_charges() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let am = g.add_card_to_hand(0, catalog::aether_meltdown());
    flood(&mut g, 0);
    cast_at(&mut g, am, Target::Permanent(giant)).expect("cast");
    assert_eq!(g.players[0].energy, 2);
    assert_eq!(g.computed_permanent(giant).unwrap().power, -1);
}

/// CR 613.1f — Stasis Field: base 0/2, defender, and the flyer loses flying.
#[test]
fn stasis_field_grounds_a_flyer() {
    let mut g = pod(2);
    let flyer = ready(&mut g, 1, catalog::aerial_guide());
    let sf = g.add_card_to_hand(0, catalog::stasis_field());
    flood(&mut g, 0);
    cast_at(&mut g, sf, Target::Permanent(flyer)).expect("cast");
    let cp = g.computed_permanent(flyer).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 2));
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Defender));
    assert!(!cp.keywords().contains(&crabomination::card::Keyword::Flying));
}

/// Lay Claim takes any permanent — here a land.
#[test]
fn lay_claim_steals_a_land() {
    let mut g = pod(2);
    let land = ready(&mut g, 1, catalog::forest());
    let lc = g.add_card_to_hand(0, catalog::lay_claim());
    flood(&mut g, 0);
    cast_at(&mut g, lc, Target::Permanent(land)).expect("cast");
    assert_eq!(g.battlefield_find(land).unwrap().controller, 0);
}

/// Enthralling Hold can't target an untapped creature (CR 115.1), but takes a
/// tapped one.
#[test]
fn enthralling_hold_needs_a_tapped_target() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let eh = g.add_card_to_hand(0, catalog::enthralling_hold());
    flood(&mut g, 0);
    assert!(cast_at(&mut g, eh, Target::Permanent(bear)).is_err(), "untapped");
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    cast_at(&mut g, eh, Target::Permanent(bear)).expect("tapped");
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
}

/// Coastal Wizard bounces itself and another creature, only before attackers
/// on your turn.
#[test]
fn coastal_wizard_bounces_both() {
    let mut g = pod(2);
    let cw = ready(&mut g, 0, catalog::coastal_wizard());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    assert!(activate(&mut g, cw, 0, Some(Target::Permanent(bear))).is_err(), "not your turn");
    g.active_player_idx = 0;
    activate(&mut g, cw, 0, Some(Target::Permanent(bear))).expect("bounce");
    assert!(g.players[0].hand.iter().any(|c| c.id == cw));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

/// Puresight Merrow's {Q} ability untaps it and may exile the top card.
#[test]
fn puresight_merrow_untaps_to_exile_the_top() {
    let mut g = pod(2);
    let pm = ready(&mut g, 0, catalog::puresight_merrow());
    g.battlefield_find_mut(pm).unwrap().tapped = true;
    flood(&mut g, 0);
    activate(&mut g, pm, 0, None).expect("activate");
    assert!(!g.battlefield_find(pm).unwrap().tapped);
}

/// CR 700.8 — Ardent Electromancer adds {R} per party member (itself a Wizard,
/// plus a Cleric).
#[test]
fn ardent_electromancer_counts_the_party() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::scribe_of_the_mindful());
    let ae = g.add_card_to_hand(0, catalog::ardent_electromancer());
    flood(&mut g, 0);
    let red = g.players[0].mana_pool.amount(Color::Red);
    cast(&mut g, ae, None).expect("cast");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), red - 1 + 2);
}
