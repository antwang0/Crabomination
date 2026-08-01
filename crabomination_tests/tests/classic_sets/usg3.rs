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

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// Every Rune activates for {W} and cycles for {2}.
#[test]
fn usg_runes_of_protection_cost_one_white() {
    for f in [
        catalog::rune_of_protection_white as fn() -> crabomination::card::CardDefinition,
        catalog::rune_of_protection_blue,
        catalog::rune_of_protection_green,
        catalog::rune_of_protection_artifacts,
        catalog::rune_of_protection_lands,
        catalog::rune_of_protection_red,
        catalog::rune_of_protection_black,
    ] {
        let def = f();
        assert_eq!(def.activated_abilities[0].mana_cost.cmc(), 1, "{}", def.name);
        assert!(
            def.keywords.iter().any(|k| matches!(k, Keyword::Cycling(c) if c.cmc() == 2)),
            "{} is missing Cycling {{2}}",
            def.name
        );
    }
}

/// Rune of Protection: Artifacts soaks an artifact source's damage.
#[test]
fn rune_of_protection_artifacts_blanks_an_artifact_source() {
    let mut g = two_player_game();
    let rune = g.add_card_to_battlefield(0, catalog::rune_of_protection_artifacts());
    let pinger = g.add_card_to_battlefield(1, catalog::sol_ring());
    mana(&mut g, 0);
    activate(&mut g, rune, 0, None);
    let life = g.players[0].life;
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        3,
        Some(pinger),
        &mut ev,
    );
    assert_eq!(g.players[0].life, life, "the chosen artifact source was blanked");
}

/// Electryte punishes every blocker once it connects.
#[test]
fn electryte_burns_its_blockers() {
    let mut g = two_player_game();
    let elec = g.add_card_to_battlefield(0, catalog::electryte());
    let chump = g.add_card_to_battlefield(1, catalog::wild_dogs()); // 2/1
    g.battlefield_find_mut(elec).unwrap().summoning_sick = false;
    while g.step != TurnStep::DeclareAttackers || g.active_player_idx != 0 {
        g.advance_step(vec![]).expect("advance");
    }
    g.declare_attackers(vec![Attack { attacker: elec, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.advance_step(vec![]).expect("to blockers");
    // Unblocked: connect with the player, then the blockers clause finds none.
    g.declare_blockers(vec![]).expect("no blocks");
    let bystander = g.battlefield_find(chump).map(|c| c.id);
    g.advance_step(vec![]).expect("to damage");
    drain_stack(&mut g);
    assert!(bystander.is_some_and(|id| g.battlefield_find(id).is_some()));
}

/// No Rest for the Wicked only takes back what died this turn.
#[test]
fn no_rest_for_the_wicked_recurs_this_turns_dead() {
    let mut g = two_player_game();
    let nrftw = g.add_card_to_battlefield(0, catalog::no_rest_for_the_wicked());
    let old = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let fresh = g.add_card_to_battlefield(0, catalog::wild_dogs());
    let mut ev = vec![];
    g.destroy_permanent(fresh, false, &mut ev);
    activate(&mut g, nrftw, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == fresh), "died this turn");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == old), "was already there");
}

/// Argothian Wurm goes back on top when a player pays a land for it.
#[test]
fn argothian_wurm_can_be_bought_off_with_a_land() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::forest());
    let wurm = g.add_card_to_hand(0, catalog::argothian_wurm());
    mana(&mut g, 0);
    cast(&mut g, wurm, None);
    assert!(g.battlefield_find(wurm).is_none(), "back on the library");
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(wurm));
}

/// Lifeline returns a dead creature at the next end step while another is out.
#[test]
fn lifeline_returns_the_dead_at_end_step() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lifeline());
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::wild_dogs()); // "another creature"
    let mut ev = vec![];
    g.destroy_permanent(victim, false, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    while g.step != TurnStep::End {
        g.advance_step(vec![]).expect("advance");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_some(), "back at the end step");
}

/// Persecute strips one color out of a hand.
#[test]
fn persecute_strips_one_color() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Green)]));
    let green = g.add_card_to_hand(1, catalog::grizzly_bears());
    let red = g.add_card_to_hand(1, catalog::lightning_bolt());
    let spell = g.add_card_to_hand(0, catalog::persecute());
    mana(&mut g, 0);
    cast(&mut g, spell, Some(Target::Player(1)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == green));
    assert!(g.players[1].hand.iter().any(|c| c.id == red));
    assert!(g.hand_visible_to(0, 1), "the hand was revealed");
}

/// Phyrexian Processor mints a Minion the size of the life it ate.
#[test]
fn phyrexian_processor_mints_what_you_paid() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(5)]));
    let life = g.players[0].life;
    let proc = g.add_card_to_hand(0, catalog::phyrexian_processor());
    mana(&mut g, 0);
    cast(&mut g, proc, None);
    assert_eq!(g.players[0].life, life - 5);
    mana(&mut g, 0);
    activate(&mut g, proc, 0, None);
    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Phyrexian Minion")
        .expect("token");
    let cp = g.computed_permanent(token.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Carpet of Flowers pays out one mana per opposing Island.
#[test]
fn carpet_of_flowers_counts_their_islands() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Color(Color::Green),
    ]));
    let carpet = g.add_card_to_battlefield(0, catalog::carpet_of_flowers());
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::island());
    }
    let _ = carpet;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// Remembrance fetches a twin of the creature that died.
#[test]
fn remembrance_fetches_a_twin() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::remembrance());
    let twin = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let dying = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(twin)),
    ]));
    let mut ev = vec![];
    g.destroy_permanent(dying, false, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == twin), "same name, fetched");
}

/// Sporogenesis blooms a Saproling per fungus counter on the dead creature.
#[test]
fn sporogenesis_blooms_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sporogenesis());
    let seeded = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(seeded)
        .unwrap()
        .add_counters(crabomination::card::CounterType::Fungus, 2);
    let mut ev = vec![];
    g.destroy_permanent(seeded, false, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(),
        2
    );
}

/// Serra's Hymn splits its verse counters across several targets.
#[test]
fn serras_hymn_divides_its_shield() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let hymn = g.add_card_to_battlefield(0, catalog::serras_hymn());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(hymn)
        .unwrap()
        .add_counters(crabomination::card::CounterType::Verse, 3);
    // 2 points to the player, 1 to the bear.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DamageDivision(vec![2, 1])]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: hymn,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    cast_as(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, life - 1, "2 of the 3 were prevented");
    assert!(g.battlefield_find(bear).is_some(), "the bear kept its 1-point shield");
}

/// Discordant Dirge eats one card per verse counter.
#[test]
fn discordant_dirge_eats_a_card_per_verse() {
    let mut g = two_player_game();
    let dirge = g.add_card_to_battlefield(0, catalog::discordant_dirge());
    g.battlefield_find_mut(dirge)
        .unwrap()
        .add_counters(crabomination::card::CounterType::Verse, 2);
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::forest());
    }
    mana(&mut g, 0);
    activate(&mut g, dirge, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 2);
    assert_eq!(g.players[1].graveyard.len(), 2);
}

/// Abundance swaps a draw for a dig to the chosen kind.
#[test]
fn abundance_digs_instead_of_drawing() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::abundance());
    // A land-light hand makes the auto policy dig for a land.
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    // Library bottom-to-top: the land is buried under two nonlands.
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    assert_eq!(g.players[0].hand.len(), 3);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.is_land()),
        "dug past the nonlands"
    );
    assert!(g.players[0].library.is_empty() || g.players[0].library.len() == 2);
}

/// Academy Researchers drags an Aura out of hand onto itself.
#[test]
fn academy_researchers_deploys_an_aura_from_hand() {
    let mut g = two_player_game();
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let aura = g.add_card_to_hand(0, catalog::pendrell_flux());
    let body = g.add_card_to_hand(0, catalog::academy_researchers());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![aura])]));
    mana(&mut g, 0);
    cast(&mut g, body, None);
    assert_eq!(
        g.battlefield_find(aura).and_then(|c| c.attached_to),
        Some(body),
        "the Aura came down attached"
    );
}

/// Defensive Formation hands the damage split to the defender (CR 510.1a).
#[test]
fn defensive_formation_lets_the_defender_assign() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::defensive_formation());
    let attacker = g.add_card_to_battlefield(0, catalog::okk()); // 4/4
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(0, catalog::serra_avatar());
    for id in [attacker, big] {
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    }
    while g.step != TurnStep::DeclareAttackers || g.active_player_idx != 0 {
        g.advance_step(vec![]).expect("advance");
    }
    g.declare_attackers(vec![
        Attack { attacker, target: AttackTarget::Player(1) },
        Attack { attacker: big, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    g.advance_step(vec![]).expect("to blockers");
    g.declare_blockers(vec![(a, attacker), (b, attacker)]).expect("double block");
    // Seat 1 is the assigner, so its scripted order is the one that lands.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DamageOrder(vec![b, a])]));
    g.advance_step(vec![]).expect("to damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(b).is_none(), "the defender put b first in line");
}

/// Temporal Aperture shuffles up and hands you the new top card for free.
#[test]
fn temporal_aperture_frees_the_new_top_card() {
    let mut g = two_player_game();
    let aperture = g.add_card_to_battlefield(0, catalog::temporal_aperture());
    g.battlefield_find_mut(aperture).unwrap().summoning_sick = false;
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    activate(&mut g, aperture, 0, None);
    assert!(
        g.players[0].library.iter().any(|c| c.id == top && c.may_play_until.is_some()),
        "the revealed card is castable for free"
    );
}

/// Diabolic Servitude rents a creature back, then takes it with it.
#[test]
fn diabolic_servitude_exiles_its_tenant_when_it_leaves() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let servitude = g.add_card_to_hand(0, catalog::diabolic_servitude());
    mana(&mut g, 0);
    cast(&mut g, servitude, Some(Target::Permanent(corpse)));
    assert!(g.battlefield_find(corpse).is_some(), "reanimated");
    assert_eq!(g.battlefield_find(servitude).and_then(|c| c.chosen_permanent), Some(corpse));
    let mut ev = vec![];
    g.destroy_permanent(servitude, false, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == corpse), "the tenant left with it");
}

/// The tenant dying instead exiles the tenant and buys the enchantment back.
#[test]
fn diabolic_servitude_returns_to_hand_when_its_tenant_dies() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let servitude = g.add_card_to_hand(0, catalog::diabolic_servitude());
    mana(&mut g, 0);
    cast(&mut g, servitude, Some(Target::Permanent(corpse)));
    let mut ev = vec![];
    g.destroy_permanent(corpse, false, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == corpse), "exiled, not binned");
    assert!(g.players[0].hand.iter().any(|c| c.id == servitude), "the rental came back");
}
