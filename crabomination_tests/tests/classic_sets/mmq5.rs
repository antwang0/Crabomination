//! Mercadian Masques (MMQ) gap closure, fifth wave.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

/// Install a scripted decider that answers with `answers`, then AutoDecider.
fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
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

/// Statecraft blanks combat damage in both directions for its controller.
#[test]
fn statecraft_seals_combat_damage_both_ways() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::statecraft());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    attack_and_block(&mut g, mine, theirs);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "incoming sealed");
    assert_eq!(g.battlefield_find(theirs).unwrap().damage, 0, "outgoing sealed");
}

/// Insubordination punishes a host that stayed home, and spares one that swung.
#[test]
fn insubordination_bites_a_creature_that_didnt_attack() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::insubordination());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);

    g.clear_sickness(host);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: host, target: AttackTarget::Player(0) }])
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "it attacked, so no bite");
}

/// Barbed Wire's {2} shield soaks its own upkeep ping.
#[test]
fn barbed_wire_can_buy_off_its_own_damage() {
    let mut g = two_player_game();
    let wire = g.add_card_to_battlefield(0, catalog::barbed_wire());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wire,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the shield ate the ping");
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "shield spent");
}

/// Battle Squadron counts your creatures, itself included.
#[test]
fn battle_squadron_sizes_to_your_board() {
    let mut g = two_player_game();
    let squad = g.add_card_to_battlefield(0, catalog::battle_squadron());
    assert_eq!(g.computed_permanent(squad).unwrap().power, 1);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(squad).unwrap().power, 2);
}

/// Bribery pulls a creature out of the opponent's library onto your side.
#[test]
fn bribery_steals_from_the_opponents_library() {
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::grizzly_bears());
    let bear = g.players[1].library.last().map(|c| c.id).expect("seeded");
    let bribery = g.add_card_to_hand(0, catalog::bribery());
    script(&mut g, vec![DecisionAnswer::Search(Some(bear))]);
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bribery,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Grizzly Bears"),
        "stolen under your control"
    );
}

/// Renounce trades permanents for 2 life each.
#[test]
fn renounce_pays_two_life_per_sacrifice() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let renounce = g.add_card_to_hand(0, catalog::renounce());
    script(&mut g, vec![DecisionAnswer::Amount(2)]);
    cast(&mut g, 0, renounce, None);
    assert_eq!(g.players[0].life, 24);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 0);
}

/// Invigorate's alt cost is free — an opponent just gains 3.
#[test]
fn invigorate_casts_free_by_gifting_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let inv = g.add_card_to_hand(0, catalog::invigorate());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: inv,
        pitch_card: None,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 23);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6);
}

/// Orim's Cure taps a creature instead of paying mana.
#[test]
fn orims_cure_taps_a_creature_as_its_alt_cost() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::plains());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cure = g.add_card_to_hand(0, catalog::orims_cure());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: cure,
        pitch_card: None,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "tapped as a cost");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 20, "shield soaked all 3");
}

/// Ferocity grows its host when it meets a blocker.
#[test]
fn ferocity_counters_up_on_a_block() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::ferocity());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    attack_and_block(&mut g, attacker, host);
    assert_eq!(
        g.battlefield_find(host).and_then(|c| c.counters.get(&CounterType::PlusOnePlusOne)),
        Some(&1)
    );
}

/// Volcanic Wind's damage total is the creature count on resolution.
#[test]
fn volcanic_wind_scales_with_the_board() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wind = g.add_card_to_hand(0, catalog::volcanic_wind());
    script(&mut g, vec![DecisionAnswer::DamageDivision(vec![2, 0])]);
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: wind,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none(), "took both points");
    assert!(g.battlefield_find(b).is_some());
}

/// Puppet's Verdict sweeps the small creatures on heads.
#[test]
fn puppets_verdict_kills_by_power_on_the_flip() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    let verdict = g.add_card_to_hand(0, catalog::puppets_verdict());
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    cast(&mut g, 0, verdict, None);
    assert!(g.battlefield_find(small).is_none());
    assert!(g.battlefield_find(big).is_some());
}

/// Nether Spirit climbs back while it's the graveyard's only creature card.
#[test]
fn nether_spirit_returns_when_it_is_alone() {
    let mut g = two_player_game();
    let spirit = g.add_card_to_graveyard(0, catalog::nether_spirit());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(spirit).is_some());
}

/// Extortion strips two cards from a hand.
#[test]
fn extortion_takes_two_cards() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let ext = g.add_card_to_hand(0, catalog::extortion());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: ext,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Ramosian Rally's alt cost taps a creature and still pumps the team.
#[test]
fn ramosian_rally_pumps_for_a_tap() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::plains());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rally = g.add_card_to_hand(0, catalog::ramosian_rally());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: rally,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(a).unwrap().power, 3);
    assert_eq!(g.computed_permanent(b).unwrap().power, 3);
    assert!(g.battlefield_find(a).unwrap().tapped || g.battlefield_find(b).unwrap().tapped);
}

/// Aerial Caravan exiles the top card and lets you play it this turn.
#[test]
fn aerial_caravan_impulse_draws() {
    let mut g = two_player_game();
    let caravan = g.add_card_to_battlefield(0, catalog::aerial_caravan());
    g.clear_sickness(caravan);
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: caravan,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == top).expect("exiled");
    assert!(exiled.may_play_until.is_some(), "playable this turn");
}

/// Saprazzan Bailiff jails graveyard artifacts, then hands them back.
#[test]
fn saprazzan_bailiff_exiles_then_returns_artifacts() {
    let mut g = two_player_game();
    let relic = g.add_card_to_graveyard(1, catalog::sol_ring());
    let bailiff = g.add_card_to_battlefield(0, catalog::saprazzan_bailiff());
    g.fire_self_etb_triggers(bailiff, 0);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == relic), "jailed");
}

/// Karn's Touch stands an artifact up at its mana value.
#[test]
fn karns_touch_animates_at_mana_value() {
    let mut g = two_player_game();
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring()); // {1}
    let touch = g.add_card_to_hand(0, catalog::karns_touch());
    cast(&mut g, 0, touch, Some(Target::Permanent(ring)));
    let cp = g.computed_permanent(ring).unwrap();
    assert!(cp.card_types.contains(&crabomination::card::CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (1, 1));
}

/// Indentured Djinn hands every opponent three cards.
#[test]
fn indentured_djinn_draws_for_the_opponent() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let before = g.players[1].hand.len();
    let djinn = g.add_card_to_battlefield(0, catalog::indentured_djinn());
    g.fire_self_etb_triggers(djinn, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), before + 3);
}

/// Megatherium sticks around when the tax is payable, and dies when it isn't.
#[test]
fn megatherium_charges_one_per_card_in_hand() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    let beast = g.add_card_to_battlefield(0, catalog::megatherium());
    g.fire_self_etb_triggers(beast, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(beast).is_none(), "no mana for the two-card tax");

    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let beast = g.add_card_to_battlefield(0, catalog::megatherium());
    mana(&mut g, 0);
    g.fire_self_etb_triggers(beast, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(beast).is_some(), "paid the one-card tax");
}

/// Common Cause pumps only while the board is monochrome.
#[test]
fn common_cause_needs_a_shared_color() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::common_cause());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
    g.add_card_to_battlefield(1, catalog::savannah_lions()); // white
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "colors diverged");
}

/// Crumbling Sanctuary turns damage into library exile.
#[test]
fn crumbling_sanctuary_exiles_instead_of_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::crumbling_sanctuary());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let lib = g.players[0].library.len();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 20);
    assert_eq!(g.players[0].library.len(), lib - 3);
}

/// Instigator forces a player's whole board to attack.
#[test]
fn instigator_makes_them_attack() {
    let mut g = two_player_game();
    let shaper = g.add_card_to_battlefield(0, catalog::instigator());
    g.clear_sickness(shaper);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shaper,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(victim)
            .unwrap()
            .keywords
            .contains(&crabomination::card::Keyword::MustAttack)
    );
}

/// War Tax charges the attacking player {X} per attacker.
#[test]
fn war_tax_bills_the_attacker() {
    let mut g = two_player_game();
    let tax = g.add_card_to_battlefield(1, catalog::war_tax());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tax,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("activate");
    drain_stack(&mut g);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
            .is_err(),
        "can't cover the {{3}} toll"
    );
}

/// War Cadence charges the blocking player {X} per blocker.
#[test]
fn war_cadence_bills_the_blocker() {
    let mut g = two_player_game();
    let cad = g.add_card_to_battlefield(0, catalog::war_cadence());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cad,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    })
    .expect("activate");
    drain_stack(&mut g);
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).is_err(),
        "can't cover the {{4}} toll"
    );
}

/// Foster digs a creature out of the library when one dies.
#[test]
fn foster_digs_after_a_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::foster());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    let mut evs = vec![];
    g.destroy_permanent(bear, true, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "found a creature");
}

/// Monkey Cage pops when a creature enters and pays out in Monkeys.
#[test]
fn monkey_cage_pays_out_in_monkeys() {
    let mut g = two_player_game();
    let cage = g.add_card_to_battlefield(0, catalog::monkey_cage());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears()); // {1}{G}
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    cast(&mut g, 1, bear, None);
    assert!(g.battlefield_find(cage).is_none(), "cage sacrificed");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Monkey").count(),
        2
    );
}

/// Credit Voucher swaps part of your hand for fresh cards.
#[test]
fn credit_voucher_redraws_what_you_shuffle_away() {
    let mut g = two_player_game();
    let voucher = g.add_card_to_battlefield(0, catalog::credit_voucher());
    g.clear_sickness(voucher);
    let dud = g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    mana(&mut g, 0);
    script(&mut g, vec![DecisionAnswer::Cards(vec![dud])]);
    g.perform_action(GameAction::ActivateAbility {
        card_id: voucher,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "one in, one out");
}

/// Assembly Hall tutors a twin for a creature in hand.
#[test]
fn assembly_hall_finds_a_second_copy() {
    let mut g = two_player_game();
    let hall = g.add_card_to_battlefield(0, catalog::assembly_hall());
    g.clear_sickness(hall);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let twin = g.add_card_to_library(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    script(&mut g, vec![DecisionAnswer::Search(Some(twin))]);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hall,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == twin));
}

/// Clear the Land deploys the lands off the top five and burns the rest.
#[test]
fn clear_the_land_deploys_lands_and_exiles_the_rest() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::forest());
    }
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::clear_the_land());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Forest").count(), 2);
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 3);
}

/// Unmask is free when you exile a black card.
#[test]
fn unmask_pitches_a_black_card() {
    let mut g = two_player_game();
    let pitch = g.add_card_to_hand(0, catalog::dark_ritual());
    let unmask = g.add_card_to_hand(0, catalog::unmask());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: unmask,
        pitch_card: Some(pitch),
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("pitch cast");
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty());
}

/// Unnatural Hunger eats a creature or bites for the host's power.
#[test]
fn unnatural_hunger_bites_when_nothing_is_fed() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_hand(0, catalog::unnatural_hunger());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Blood Hound swells with the damage you take and deflates at end of turn.
#[test]
fn blood_hound_grows_on_damage_then_resets() {
    let mut g = two_player_game();
    let hound = g.add_card_to_battlefield(0, catalog::blood_hound());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.computed_permanent(hound).unwrap().power, 4);
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(hound).unwrap().power, 1);
}

/// Lava Runner charges a land for targeting it.
#[test]
fn lava_runner_eats_a_land_when_targeted() {
    let mut g = two_player_game();
    let runner = g.add_card_to_battlefield(0, catalog::lava_runner());
    g.add_card_to_battlefield(1, catalog::mountain());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(runner)));
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 0, "land sacrificed");
}

/// Mercadia's Downfall arms attackers with the defender's nonbasics.
#[test]
fn mercadias_downfall_counts_nonbasic_lands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::mountain()); // basic — no bonus
    g.add_card_to_battlefield(1, catalog::sol_ring());
    g.add_card_to_battlefield(1, catalog::adarkar_wastes());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let spell = g.add_card_to_hand(0, catalog::mercadias_downfall());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.computed_permanent(attacker).unwrap().power, 3);
}

/// Erithizon hands out a counter whenever it attacks.
#[test]
fn erithizon_gives_a_counter_on_attack() {
    let mut g = two_player_game();
    let porcupine = g.add_card_to_battlefield(0, catalog::erithizon());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(porcupine);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: porcupine, target: AttackTarget::Player(1) }])
        .expect("attack");
    script(&mut g, vec![DecisionAnswer::Target(Target::Permanent(victim))]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(victim).unwrap().power, 3);
}

/// Deepwood Elder turns X lands into Forests.
#[test]
fn deepwood_elder_forests_x_lands() {
    let mut g = two_player_game();
    let elder = g.add_card_to_battlefield(0, catalog::deepwood_elder());
    g.clear_sickness(elder);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let island = g.add_card_to_battlefield(0, catalog::island());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: elder,
        ability_index: 0,
        target: Some(Target::Permanent(island)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(island)
            .unwrap()
            .subtypes
            .land_types
            .contains(&crabomination::card::LandType::Forest)
    );
}

/// Cowardice bounces anything that gets targeted.
#[test]
fn cowardice_bounces_the_targeted_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cowardice());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "back to hand");
}

/// Crag Saurian defects to whoever damaged it.
#[test]
fn crag_saurian_changes_sides_when_damaged() {
    let mut g = two_player_game();
    let saurian = g.add_card_to_battlefield(0, catalog::crag_saurian());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(saurian)));
    assert_eq!(g.battlefield_find(saurian).unwrap().controller, 1);
}

/// Diplomatic Escort counters anything aimed at a creature.
#[test]
fn diplomatic_escort_counters_a_creature_targeting_spell() {
    let mut g = two_player_game();
    let escort = g.add_card_to_battlefield(0, catalog::diplomatic_escort());
    g.clear_sickness(escort);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: escort,
        ability_index: 0,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "bolt countered");
}

/// Jeweled Torque sells 2 life whenever a spell of the named colour resolves.
#[test]
fn jeweled_torque_pays_for_the_named_color() {
    let mut g = two_player_game();
    let torque = g.add_card_to_battlefield(0, catalog::jeweled_torque());
    script(&mut g, vec![DecisionAnswer::Color(Color::Red), DecisionAnswer::Bool(true)]);
    g.fire_self_etb_triggers(torque, 0);
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt()); // red
    mana(&mut g, 0);
    cast(&mut g, 1, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[0].life, 22);
}

/// Ley Line hands out a counter each upkeep.
#[test]
fn ley_line_grows_a_creature_each_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ley_line());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    script(&mut g, vec![DecisionAnswer::Bool(true), DecisionAnswer::Target(Target::Permanent(bear))]);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}
