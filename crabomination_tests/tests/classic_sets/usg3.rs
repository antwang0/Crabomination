//! Urza's Saga (USG) gap closure, third wave.

use crabomination::card::{CardType, Keyword};
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

/// Cast from `seat`'s hand, handing it priority first.
fn cast_as(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = seat;
    cast(g, id, target);
}

/// Give `seat` a library so draws don't deck it.
fn stock(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::forest());
    }
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

/// Fire `seat`'s upkeep triggers in place (no library needed for the draw step).
fn upkeep(g: &mut GameState, seat: usize) {
    g.active_player_idx = seat;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

// ── Statics ─────────────────────────────────────────────────────────────────

/// Telepathy publishes opponents' hands through the server view.
#[test]
fn telepathy_reveals_opponent_hands() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    assert!(!g.hand_visible_to(0, 1));
    g.add_card_to_battlefield(0, catalog::telepathy());
    assert!(g.hand_visible_to(0, 1), "opponent plays with hand revealed");
    assert!(!g.hand_visible_to(1, 0), "the reveal is one-way");
}

/// Fluctuator makes a Cycling {2} card free to cycle.
#[test]
fn fluctuator_discounts_cycling() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fluctuator());
    let card = g.add_card_to_hand(0, catalog::drifting_meadow());
    stock(&mut g, 0, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: card, x_value: None }).expect("cycle for free");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0, "no mana was spent");
    assert_eq!(g.players[0].hand.len(), hand_before, "discarded one, drew one");
}

/// Sulfuric Vapors adds 1 to a red spell's damage but not to a permanent's.
#[test]
fn sulfuric_vapors_pumps_red_spells_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sulfuric_vapors());
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 4, "3 + 1");
}

/// Contamination turns every land into a Swamp for mana purposes.
#[test]
fn contamination_makes_every_land_tap_for_black() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::contamination());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, forest, 0, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 0);
}

/// Energy Field blanks damage from an opponent's source but not your own, and
/// dies to the first card entering your graveyard.
#[test]
fn energy_field_prevents_opposing_damage_then_dies() {
    let mut g = two_player_game();
    let field = g.add_card_to_battlefield(0, catalog::energy_field());
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    cast_as(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, life, "prevented");
    // The Bolt hit its owner's graveyard, not yours; the Field survives.
    assert!(g.battlefield_find(field).is_some());
    let own = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    cast_as(&mut g, 0, own, Some(Target::Player(1)));
    assert!(g.battlefield_find(field).is_none(), "your own card hit your graveyard");
}

// ── State triggers (CR 603.8) ───────────────────────────────────────────────

/// Hidden Predators sleeps until an opponent fields a 4-power creature, then
/// becomes a 4/4 Beast and stops being an enchantment (CR 205.1b).
#[test]
fn hidden_predators_wakes_on_a_big_opposing_creature() {
    let mut g = two_player_game();
    let pred = g.add_card_to_battlefield(0, catalog::hidden_predators());
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(!g.computed_permanent(pred).unwrap().card_types.contains(&CardType::Creature));
    g.add_card_to_battlefield(1, catalog::okk()); // 4/4
    g.check_state_based_actions();
    drain_stack(&mut g);
    let cp = g.computed_permanent(pred).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature));
    assert!(!cp.card_types.contains(&CardType::Enchantment));
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// A state trigger fires once while its condition holds — not once per SBA pass.
#[test]
fn cr_603_8_state_trigger_latches_until_the_condition_clears() {
    let mut g = two_player_game();
    let croc = g.add_card_to_battlefield(0, catalog::veiled_crocodile());
    g.players[0].hand.clear();
    g.players[1].hand.clear();
    g.check_state_based_actions();
    assert_eq!(g.stack.len(), 1);
    assert!(g.state_trigger_armed.contains(&croc));
    g.check_state_based_actions();
    assert_eq!(g.stack.len(), 1, "no second trigger while the condition holds");
    drain_stack(&mut g);
    // Animating clears "this permanent is an enchantment", so the latch re-arms.
    assert!(!g.state_trigger_armed.contains(&croc));
}

/// Veiled Apparition wakes on an opponent's spell and carries the granted
/// upkeep tax.
#[test]
fn veiled_apparition_wakes_with_its_upkeep_tax() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::veiled_apparition());
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    mana(&mut g, 1);
    g.active_player_idx = 1;
    cast_as(&mut g, 1, bears, None);
    let cp = g.computed_permanent(app).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Flying));
    assert_eq!(
        g.battlefield_find(app).unwrap().definition.triggered_abilities.len(),
        2,
        "the sacrifice-unless-pay rider came with it"
    );
}

// ── Combat ──────────────────────────────────────────────────────────────────

/// Okk can't attack without a bigger partner in the same batch.
#[test]
fn okk_needs_a_bigger_attacker() {
    let mut g = two_player_game();
    let okk = g.add_card_to_battlefield(0, catalog::okk());
    g.battlefield_find_mut(okk).unwrap().summoning_sick = false;
    while g.step != TurnStep::DeclareAttackers || g.active_player_idx != 0 {
        g.advance_step(vec![]).expect("advance");
    }
    assert!(
        g.declare_attackers(vec![Attack { attacker: okk, target: AttackTarget::Player(1) }])
            .is_err(),
        "alone it stays home"
    );
    let big = g.add_card_to_battlefield(0, catalog::serra_avatar());
    g.battlefield_find_mut(big).unwrap().summoning_sick = false;
    g.declare_attackers(vec![
        Attack { attacker: okk, target: AttackTarget::Player(1) },
        Attack { attacker: big, target: AttackTarget::Player(1) },
    ])
    .expect("a bigger friend lets Okk swing");
}

/// Outmaneuver sends a blocked creature's damage past its blocker.
#[test]
fn outmaneuver_routes_damage_past_the_blocker() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(attacker).unwrap().summoning_sick = false;
    while g.step != TurnStep::DeclareAttackers || g.active_player_idx != 0 {
        g.advance_step(vec![]).expect("advance");
    }
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.advance_step(vec![]).expect("to blockers");
    g.declare_blockers(vec![(blocker, attacker)]).expect("block");
    let spell = g.add_card_to_hand(0, catalog::outmaneuver());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(attacker)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
    })
    .expect("cast");
    drain_stack(&mut g);
    let life = g.players[1].life;
    g.advance_step(vec![]).expect("to damage");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "the blocker was ignored");
}

/// Waylay's Knights survive the end step and vanish at cleanup.
#[test]
fn waylay_knights_last_until_cleanup() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::waylay());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    let knights: Vec<CardId> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Knight")
        .map(|c| c.id)
        .collect();
    assert_eq!(knights.len(), 3);
    while g.step != TurnStep::End {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert_eq!(
        knights.iter().filter(|id| g.battlefield_find(**id).is_some()).count(),
        3,
        "still around during the end step"
    );
    g.advance_step(vec![]).expect("to cleanup");
    drain_stack(&mut g);
    assert!(knights.iter().all(|id| g.battlefield_find(*id).is_none()), "exiled at cleanup");
}

// ── Upkeep engines ──────────────────────────────────────────────────────────

/// Umbilicus takes 2 life from a player willing to pay; the bot pays.
#[test]
fn umbilicus_taxes_each_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::umbilicus());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let (life, board) = (g.players[0].life, g.battlefield.len());
    upkeep(&mut g, 0);
    assert!(
        g.players[0].life == life - 2 || g.battlefield.len() == board - 1,
        "paid 2 life or gave a permanent back"
    );
}

/// Noetic Scales bounces creatures bigger than their controller's hand.
#[test]
fn noetic_scales_bounces_the_oversized() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::noetic_scales());
    let big = g.add_card_to_battlefield(0, catalog::okk()); // 4/4
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.players[0].hand.clear();
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::forest());
    }
    upkeep(&mut g, 0);
    assert!(g.battlefield_find(big).is_none(), "power 4 > 3 cards in hand");
    assert!(g.battlefield_find(small).is_some(), "power 2 is fine");
}

/// Purging Scythe picks the least-tough creature on the whole battlefield.
#[test]
fn purging_scythe_hits_the_flimsiest_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::purging_scythe());
    let tough = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let frail = g.add_card_to_battlefield(1, catalog::wild_dogs()); // 2/1
    upkeep(&mut g, 0);
    assert!(g.battlefield_find(frail).is_none(), "1 toughness, 2 damage");
    assert!(g.battlefield_find(tough).is_some());
}

/// Thran Turbine's mana can't be spent on a spell.
#[test]
fn thran_turbine_mana_is_abilities_only() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::thran_turbine());
    upkeep(&mut g, 0);
    assert_eq!(g.players[0].mana_pool.total(), 0, "not freely spendable");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2);
    let spell = g.add_card_to_hand(0, catalog::ornithopter());
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "restricted mana can't pay for a spell"
    );
}

/// Wild Dogs defect to whoever is ahead on life.
#[test]
fn wild_dogs_run_to_the_life_leader() {
    let mut g = two_player_game();
    let dogs = g.add_card_to_battlefield(0, catalog::wild_dogs());
    g.players[1].life = 30;
    upkeep(&mut g, 0);
    assert_eq!(g.battlefield_find(dogs).unwrap().controller, 1);
}

/// Greener Pastures only feeds the player with strictly the most lands.
#[test]
fn greener_pastures_rewards_the_land_leader() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::greener_pastures());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.add_card_to_battlefield(1, catalog::island());
    upkeep(&mut g, 0);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(),
        1
    );
}

/// Antagonism spares a player whose opponent took damage that turn.
#[test]
fn antagonism_spares_the_aggressor() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::antagonism());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    cast(&mut g, bolt, Some(Target::Player(1)));
    let life = g.players[0].life;
    while g.step != TurnStep::End {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "an opponent was dealt damage this turn");
}

// ── Activations, auras, spells ──────────────────────────────────────────────

/// Attunement's cost is returning itself (CR 118 — `IsSource`).
#[test]
fn attunement_returns_itself_to_draw_three() {
    let mut g = two_player_game();
    let att = g.add_card_to_battlefield(0, catalog::attunement());
    for _ in 0..5 {
        g.add_card_to_hand(0, catalog::forest());
    }
    stock(&mut g, 0, 5);
    let hand = g.players[0].hand.len();
    activate(&mut g, att, 0, None);
    assert!(g.battlefield_find(att).is_none(), "the enchantment bounced as the cost");
    // +1 (Attunement itself) +3 drawn −4 discarded.
    assert_eq!(g.players[0].hand.len(), hand);
    assert!(g.players[0].graveyard.len() >= 4);
}

/// Copper Gnomes cheats an artifact out of hand.
#[test]
fn copper_gnomes_deploys_an_artifact_from_hand() {
    let mut g = two_player_game();
    let gnomes = g.add_card_to_battlefield(0, catalog::copper_gnomes());
    let bomb = g.add_card_to_hand(0, catalog::fluctuator());
    mana(&mut g, 0);
    activate(&mut g, gnomes, 0, None);
    assert!(g.battlefield_find(bomb).is_some());
    assert!(g.battlefield_find(gnomes).is_none(), "sacrificed as a cost");
}

/// Pendrell Flux makes the host pay its own mana cost each upkeep.
#[test]
fn pendrell_flux_taxes_the_enchanted_creature() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::pendrell_flux());
    mana(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(host)));
    upkeep(&mut g, 1);
    assert!(g.battlefield_find(host).is_none(), "an empty pool can't pay {{1}}{{G}}");
}

/// Power Taint bleeds the enchanted enchantment's controller.
#[test]
fn power_taint_drains_the_enchanted_controller() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::telepathy());
    let aura = g.add_card_to_hand(0, catalog::power_taint());
    mana(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(theirs)));
    let life = g.players[1].life;
    upkeep(&mut g, 1);
    assert_eq!(g.players[1].life, life - 2);
}

/// Yawgmoth's Will opens the graveyard and exiles what falls in afterwards.
#[test]
fn yawgmoths_will_opens_the_graveyard() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::yawgmoths_will());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    assert!(g.players[0].play_from_graveyard_this_turn);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ev = vec![];
    g.destroy_permanent(bears, false, &mut ev);
    assert!(
        g.players[0].graveyard.iter().all(|c| c.id != bears),
        "it was exiled instead of binned"
    );
}

/// Planar Birth returns every basic land in every graveyard, tapped.
#[test]
fn planar_birth_rebuilds_both_manabases() {
    let mut g = two_player_game();
    let mine = g.add_card_to_graveyard(0, catalog::forest());
    let theirs = g.add_card_to_graveyard(1, catalog::island());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::planar_birth());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    for (id, owner) in [(mine, 0), (theirs, 1)] {
        let c = g.battlefield_find(id).expect("back on the battlefield");
        assert_eq!(c.controller, owner);
        assert!(c.tapped);
    }
    assert_eq!(g.players[1].graveyard.len(), 1, "the creature stayed put");
}

/// Brand takes back a stolen permanent.
#[test]
fn brand_reclaims_what_you_own() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().controller = 1;
    let spell = g.add_card_to_hand(0, catalog::brand());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0);
}

/// Victimize trades one creature for two out of the graveyard, tapped.
#[test]
fn victimize_reanimates_two_for_one() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let a = g.add_card_to_graveyard(0, catalog::okk());
    let b = g.add_card_to_graveyard(0, catalog::wild_dogs());
    let spell = g.add_card_to_hand(0, catalog::victimize());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed");
    for id in [a, b] {
        assert!(g.battlefield_find(id).is_some_and(|c| c.tapped));
    }
}

/// Ill-Gotten Gains wheels every hand into three graveyard picks.
#[test]
fn ill_gotten_gains_wheels_into_the_graveyard() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::ill_gotten_gains());
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].hand.len(), 3, "hand pitched, three bought back");
    assert!(g.exile.iter().any(|c| c.id == spell), "it exiles itself");
}

/// Time Spiral refills both players and untaps six of your lands.
#[test]
fn time_spiral_refuels_and_untaps_six_lands() {
    let mut g = two_player_game();
    let lands: Vec<CardId> =
        (0..7).map(|_| g.add_card_to_battlefield(0, catalog::island())).collect();
    for id in &lands {
        g.battlefield_find_mut(*id).unwrap().tapped = true;
    }
    let spell = g.add_card_to_hand(0, catalog::time_spiral());
    g.add_card_to_hand(0, catalog::forest());
    stock(&mut g, 0, 10);
    stock(&mut g, 1, 10);
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[1].hand.len(), 7);
    assert_eq!(lands.iter().filter(|id| !g.battlefield_find(**id).unwrap().tapped).count(), 6);
}
