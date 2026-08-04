//! Conspiracy (CNS) — CR 315 conspiracy cards, `catalog::sets::cns`.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
    g
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
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

fn activate_n(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn upkeep(g: &mut GameState) {
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

/// CR 315.5 — a face-up conspiracy's static abilities affect the game from
/// the command zone.
#[test]
fn cr_315_5_weight_advantage_works_from_the_command_zone() {
    let mut g = main_phase();
    g.seat_conspiracy(0, catalog::weight_advantage(), None);
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_omens());
    assert!(
        g.computed_permanent(wall)
            .expect("wall")
            .keywords
            .contains(&Keyword::AssignsCombatDamageByToughness)
    );
}

/// CR 315.5b — a face-down conspiracy has no characteristics, so nothing
/// applies until it is turned face up (CR 702.106b).
#[test]
fn cr_315_5b_a_face_down_agenda_grants_nothing_until_revealed() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::immediate_action(), Some("Grizzly Bears"));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).expect("bear").keywords.contains(&Keyword::Haste));
    assert!(g.reveal_hidden_agenda(0, agenda));
    assert!(g.computed_permanent(bear).expect("bear").keywords.contains(&Keyword::Haste));
}

/// CR 702.106 — the revealed name gates the grant; other creatures miss out.
#[test]
fn hidden_agenda_only_names_one_card() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::immediate_action(), Some("Grizzly Bears"));
    g.reveal_hidden_agenda(0, agenda);
    let other = g.add_card_to_battlefield(0, catalog::hill_giant());
    assert!(!g.computed_permanent(other).expect("giant").keywords.contains(&Keyword::Haste));
}

#[test]
fn power_play_claims_the_first_turn() {
    let mut g = main_phase();
    g.seat_conspiracy(1, catalog::power_play(), None);
    assert_eq!(g.apply_starting_player_conspiracies(), 1);
    assert_eq!(g.active_player_idx, 1);
}

#[test]
fn hymn_of_the_wilds_discounts_creatures_and_locks_out_spells() {
    let mut g = main_phase();
    g.seat_conspiracy(0, catalog::hymn_of_the_wilds(), None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err()
    );
    // The first creature spell of the turn costs {1} less.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, 0, bear, None);
    assert!(g.battlefield_find(bear).is_some());
}

#[test]
fn bragos_favor_discounts_the_named_spell() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::bragos_favor(), Some("Hill Giant"));
    g.reveal_hidden_agenda(0, agenda);
    let giant = g.add_card_to_hand(0, catalog::hill_giant()); // {3}{R}
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, 0, giant, None);
    assert!(g.battlefield_find(giant).is_some());
}

#[test]
fn muzzios_preparations_grows_the_named_creature() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::muzzios_preparations(), Some("Grizzly Bears"));
    g.reveal_hidden_agenda(0, agenda);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, 0, bear, None);
    assert_eq!(
        g.battlefield_find(bear).expect("bear").counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

#[test]
fn iterative_analysis_draws_off_the_named_spell() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::iterative_analysis(), Some("Lightning Bolt"));
    g.reveal_hidden_agenda(0, agenda);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let library = g.players[0].library.len();
    cast(&mut g, 0, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[0].library.len(), library - 1);
}

#[test]
fn sentinel_dispatch_and_hold_the_perimeter_seed_the_first_upkeep() {
    let mut g = main_phase();
    g.seat_conspiracy(0, catalog::sentinel_dispatch(), None);
    g.seat_conspiracy(0, catalog::hold_the_perimeter(), None);
    g.turn_number = 1;
    upkeep(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).count(), 2);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1 && c.is_token).count(), 1);
}

/// CR 315.5 — a granted activated ability reaches the battlefield from the
/// command zone, and only the named creature gets it.
#[test]
fn incendiary_dissent_arms_only_the_named_creature() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::incendiary_dissent(), Some("Grizzly Bears"));
    g.reveal_hidden_agenda(0, agenda);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    assert_eq!(g.granted_abilities_for(bear).len(), 1);
    assert!(g.granted_abilities_for(giant).is_empty());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 3);
}

#[test]
fn secrets_of_paradise_taps_the_named_creature_for_any_colour() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::secrets_of_paradise(), Some("Grizzly Bears"));
    g.reveal_hidden_agenda(0, agenda);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    assert!(g.players[0].mana_pool.total() > 0);
}

/// The granted trigger fires on the named creature only, and the {W} rider
/// is optional (the AutoDecider declines).
#[test]
fn adrianas_valor_offers_indestructible_on_attack() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::adrianas_valor(), Some("Grizzly Bears"));
    g.reveal_hidden_agenda(0, agenda);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let bear_card = g.battlefield_find(bear).expect("bear").clone();
    let giant_card = g.battlefield_find(giant).expect("giant").clone();
    assert_eq!(g.statics_granted_triggers_for(&bear_card).len(), 1);
    assert!(g.statics_granted_triggers_for(&giant_card).is_empty());
}

#[test]
fn double_stroke_copies_the_named_spell() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::double_stroke(), Some("Lightning Bolt"));
    g.reveal_hidden_agenda(0, agenda);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, 0, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14);
}

#[test]
fn secret_summoning_fetches_the_named_creature() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::secret_summoning(), Some("Grizzly Bears"));
    g.reveal_hidden_agenda(0, agenda);
    let twin = g.add_card_to_library(0, catalog::grizzly_bears());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(vec![
        crabomination::decision::DecisionAnswer::Search(Some(twin)),
    ]));
    cast(&mut g, 0, bear, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == twin));
}

#[test]
fn worldknit_makes_every_land_a_rainbow() {
    let mut g = main_phase();
    g.seat_conspiracy(0, catalog::worldknit(), None);
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    // The printed mana ability plus the granted any-colour one.
    assert_eq!(g.granted_abilities_for(mountain).len(), 1);
}

#[test]
fn hired_heist_and_the_zombie_rider_reach_only_the_named_creature() {
    let mut g = main_phase();
    let heist = g.seat_conspiracy(0, catalog::hired_heist(), Some("Grizzly Bears"));
    let vile = g.seat_conspiracy(0, catalog::assemble_the_rank_and_vile(), Some("Hill Giant"));
    g.reveal_hidden_agenda(0, heist);
    g.reveal_hidden_agenda(0, vile);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let bear_card = g.battlefield_find(bear).expect("bear").clone();
    let giant_card = g.battlefield_find(giant).expect("giant").clone();
    assert_eq!(g.statics_granted_triggers_for(&bear_card).len(), 1);
    assert_eq!(g.statics_granted_triggers_for(&giant_card).len(), 1);
}

#[test]
fn natural_unity_offers_a_counter_each_combat() {
    let mut g = main_phase();
    let agenda = g.seat_conspiracy(0, catalog::natural_unity(), Some("Grizzly Bears"));
    g.reveal_hidden_agenda(0, agenda);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.step = TurnStep::BeginCombat;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(vec![
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).expect("bear").counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

#[test]
fn deathreap_ritual_draws_only_after_a_death() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::deathreap_ritual());
    let before = g.players[0].hand.len();
    g.step = TurnStep::End;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(vec![
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before, "nothing died");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut events = vec![];
    g.destroy_permanent(bear, false, &mut events);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(vec![
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1);
}

#[test]
fn brago_blinks_your_board_on_connect() {
    let mut g = main_phase();
    let brago = g.add_card_to_battlefield(0, catalog::brago_king_eternal());
    g.clear_sickness(brago);
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_omens());
    let before = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: brago, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat {
        g.advance_step(Vec::new()).expect("advance");
        drain_stack(&mut g);
    }
    // The Wall left and re-entered, firing its ETB draw; Brago stayed put.
    assert!(g.battlefield_find(brago).is_some());
    assert!(g.battlefield_find(wall).is_some());
    assert_eq!(g.players[0].hand.len(), before + 1);
}

#[test]
fn canal_dredger_bottoms_a_graveyard_card() {
    let mut g = main_phase();
    let dredger = g.add_card_to_battlefield(0, catalog::canal_dredger());
    g.clear_sickness(dredger);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: dredger,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.is_empty());
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bear));
}

/// Parley counts the nonland tops, runs the body once, then everyone draws.
#[test]
fn parley_counts_nonland_reveals_then_refills() {
    let mut g = main_phase();
    g.players[0].library.clear();
    g.players[1].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears()); // nonland top
    g.add_card_to_library(0, catalog::mountain());
    g.add_card_to_library(1, catalog::mountain()); // land top
    g.add_card_to_library(1, catalog::mountain());
    let spell = g.add_card_to_hand(0, catalog::rousing_of_souls());
    let hands = [g.players[0].hand.len(), g.players[1].hand.len()];
    cast(&mut g, 0, spell, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1);
    // The spell left hand (-1), then both players drew (+1 each).
    assert_eq!(g.players[0].hand.len(), hands[0]);
    assert_eq!(g.players[1].hand.len(), hands[1] + 1);
}

#[test]
fn selvalas_enforcer_grows_on_the_parley() {
    let mut g = main_phase();
    let enforcer = g.add_card_to_hand(0, catalog::selvalas_enforcer());
    cast(&mut g, 0, enforcer, None);
    // Both libraries are all Mountains, so no nonland was revealed.
    assert_eq!(
        g.battlefield_find(enforcer).expect("enforcer").counter_count(CounterType::PlusOnePlusOne),
        0
    );
}

#[test]
fn academy_elite_is_sized_by_the_graveyards() {
    let mut g = main_phase();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.add_card_to_graveyard(1, catalog::grizzly_bears()); // not an I/S
    let elite = g.add_card_to_hand(0, catalog::academy_elite());
    cast(&mut g, 0, elite, None);
    assert_eq!(
        g.battlefield_find(elite).expect("elite").counter_count(CounterType::PlusOnePlusOne),
        2
    );
}

#[test]
fn treasonous_ogre_dethrones_and_burns_life_for_red() {
    let mut g = main_phase();
    let ogre = g.add_card_to_battlefield(0, catalog::treasonous_ogre());
    g.clear_sickness(ogre);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ogre,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    assert_eq!(g.players[0].life, 17);
    assert!(g.players[0].mana_pool.total() > 0);
    // Dethrone: attacking the (tied) most-life player grows it.
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: ogre, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ogre).expect("ogre").counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

// ── Voting, CR 701.38 ──────────────────────────────────────────────────────

use crabomination::decision::{DecisionAnswer, ScriptedDecider};

/// Vote answers, in seat order starting with the controller.
fn ballot(g: &mut GameState, picks: [u32; 2]) {
    g.decider = Box::new(ScriptedDecider::new(picks.map(DecisionAnswer::Amount)));
}

/// CR 701.38 — will of the council: the option with the most votes happens.
#[test]
fn plea_for_power_majority_wins() {
    let mut g = main_phase();
    mana(&mut g, 0);
    let id = g.add_card_to_hand(0, catalog::plea_for_power());
    ballot(&mut g, [1, 1]); // knowledge, knowledge
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "knowledge drew three");
    assert_eq!(g.players[0].extra_turns, 0, "time lost the vote");
}

/// CR 701.38 — a tied will-of-the-council vote resolves to the printed
/// "…or the vote is tied" option, which is always the later one.
#[test]
fn tyrants_choice_tie_goes_to_torture() {
    let mut g = main_phase();
    mana(&mut g, 0);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tyrants_choice());
    ballot(&mut g, [0, 1]); // death, torture
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16, "torture broke the tie");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 1, "no edict");
}

/// CR 701.38 — council's dilemma: every option fires once per vote it drew.
#[test]
fn capital_punishment_dilemma_runs_both_options() {
    let mut g = main_phase();
    mana(&mut g, 0);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::capital_punishment());
    ballot(&mut g, [0, 1]); // death, taxes
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.controller != 1), "the death vote ate the creature");
    assert!(g.players[1].hand.is_empty(), "the taxes vote ate the card");
}

/// Lieutenants of the Guard splits its dilemma across counters and tokens.
#[test]
fn lieutenants_of_the_guard_splits_its_votes() {
    let mut g = main_phase();
    mana(&mut g, 0);
    let card = g.add_card_to_hand(0, catalog::lieutenants_of_the_guard());
    ballot(&mut g, [0, 1]); // strength, numbers
    g.perform_action(GameAction::CastSpell {
        card_id: card, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Lieutenants of the Guard").unwrap().id;
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1, "one Soldier token");
}

/// Messenger Jays loots once per quill vote.
#[test]
fn messenger_jays_loots_per_quill_vote() {
    let mut g = main_phase();
    mana(&mut g, 0);
    let card = g.add_card_to_hand(0, catalog::messenger_jays());
    ballot(&mut g, [1, 1]); // quill, quill
    g.perform_action(GameAction::CastSpell {
        card_id: card, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    let hand = g.players[0].hand.len();
    drain_stack(&mut g);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Messenger Jays").unwrap().id;
    assert_eq!(g.players[0].hand.len(), hand, "two draws, two discards");
    assert_eq!(g.players[0].graveyard.len(), 2, "both loots discarded");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Coercive Portal's carnage vote sacrifices itself and wipes the board.
#[test]
fn coercive_portal_carnage_wipes_the_board() {
    let mut g = main_phase();
    let portal = g.add_card_to_battlefield(0, catalog::coercive_portal());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    ballot(&mut g, [0, 0]); // carnage, carnage
    upkeep(&mut g);
    assert!(g.battlefield_find(portal).is_none(), "Portal sacrificed itself");
    assert!(g.battlefield.iter().all(|c| c.definition.is_land()), "nonlands destroyed");
}

// ── The in-game remainder (`catalog::sets::cns2`) ───────────────────────────

/// Bite of the Black Rose's losing option doesn't happen.
#[test]
fn bite_of_the_black_rose_shrinks_on_a_sickness_majority() {
    let mut g = main_phase();
    mana(&mut g, 0);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::bite_of_the_black_rose());
    ballot(&mut g, [0, 0]); // sickness, sickness
    cast(&mut g, 0, id, None);
    assert!(g.battlefield_find(bear).is_none(), "-2/-2 killed the 2/2");
}

/// Brago's Representative votes twice.
#[test]
fn bragos_representative_votes_twice() {
    let mut g = main_phase();
    mana(&mut g, 0);
    g.add_card_to_battlefield(0, catalog::bragos_representative());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tyrants_choice());
    // Seat 0 casts both its votes for death; seat 1's lone torture vote loses.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(0),
        DecisionAnswer::Amount(0),
        DecisionAnswer::Amount(1),
    ]));
    cast(&mut g, 0, id, None);
    assert_eq!(g.players[1].life, 20, "torture lost 2-1");
    assert!(g.battlefield.iter().all(|c| c.controller != 1), "the edict resolved");
}

/// Council Guardian takes protection from every colour tied for most votes.
#[test]
fn council_guardian_gains_every_winning_protection() {
    let mut g = main_phase();
    let id = g.add_card_to_hand(0, catalog::council_guardian());
    ballot(&mut g, [0, 2]); // blue, red — one vote each
    cast(&mut g, 0, id, None);
    let kw = g.computed_permanent(id).expect("guardian").keywords;
    assert!(kw.contains(&Keyword::Protection(Color::Blue)));
    assert!(kw.contains(&Keyword::Protection(Color::Red)));
    assert!(!kw.contains(&Keyword::Protection(Color::Green)));
}

/// Custodi Squire's ballot returns the winning graveyard card.
#[test]
fn custodi_squire_returns_the_voted_card() {
    let mut g = main_phase();
    let relic = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::custodi_squire());
    cast(&mut g, 0, id, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == relic), "returned to hand");
}

/// Dack Fayden's −2 steals an artifact outright.
#[test]
fn dack_fayden_steals_an_artifact() {
    let mut g = main_phase();
    let dack = g.add_card_to_battlefield(0, catalog::dack_fayden());
    let rock = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: dack, ability_index: 1, target: Some(Target::Permanent(rock)), x_value: None,
    })
    .expect("minus two");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rock).expect("rock").controller, 0);
}

/// Dack's Duplicate copies with haste and dethrone bolted on.
#[test]
fn dacks_duplicate_copies_with_haste_and_dethrone() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::sengir_vampire());
    let id = g.add_card_to_hand(0, catalog::dacks_duplicate());
    cast(&mut g, 0, id, None);
    let cp = g.computed_permanent(id).expect("duplicate");
    assert_eq!((cp.power, cp.toughness), (4, 4), "copied the Vampire");
    assert!(cp.keywords.contains(&Keyword::Haste));
    let card = g.battlefield_find(id).expect("duplicate").clone();
    assert_eq!(card.definition.triggered_abilities.len(), 1, "dethrone bolted on");
}

/// Extract from Darkness mills, then reanimates.
#[test]
fn extract_from_darkness_mills_then_reanimates() {
    let mut g = main_phase();
    let body = g.add_card_to_graveyard(1, catalog::sengir_vampire());
    let id = g.add_card_to_hand(0, catalog::extract_from_darkness());
    cast(&mut g, 0, id, None);
    assert_eq!(g.battlefield_find(body).expect("body").controller, 0);
    assert_eq!(g.players[1].graveyard.len(), 2, "milled two");
}

/// Flamewright makes Constructs, then throws them.
#[test]
fn flamewright_builds_and_fires_constructs() {
    let mut g = main_phase();
    let smith = g.add_card_to_battlefield(0, catalog::flamewright());
    g.clear_sickness(smith);
    activate_n(&mut g, 0, smith, 0, None);
    let token = g.battlefield.iter().find(|c| c.is_token).expect("Construct").id;
    g.battlefield_find_mut(smith).unwrap().tapped = false;
    activate_n(&mut g, 0, smith, 1, Some(Target::Player(1)));
    assert!(g.battlefield_find(token).is_none(), "the Construct was sacrificed");
    assert_eq!(g.players[1].life, 19);
}

/// Grenzo digs the bottom card up when it's small enough.
#[test]
fn grenzo_deploys_a_small_bottom_creature() {
    let mut g = main_phase();
    let grenzo = g.add_card_to_battlefield(0, catalog::grenzo_dungeon_warden());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    activate_n(&mut g, 0, grenzo, 0, None);
    assert!(g.battlefield_find(bear).is_some(), "2 power fits under Grenzo's 2");
}

/// Grudge Keeper drains the opponents who voted the other way.
#[test]
fn grudge_keeper_punishes_dissent() {
    let mut g = main_phase();
    mana(&mut g, 0);
    g.add_card_to_battlefield(0, catalog::grudge_keeper());
    let id = g.add_card_to_hand(0, catalog::plea_for_power());
    ballot(&mut g, [1, 0]); // knowledge vs. time
    cast(&mut g, 0, id, None);
    assert_eq!(g.players[1].life, 18, "voted against you");
}

/// Ignition Team counts tapped lands and animates one.
#[test]
fn ignition_team_counts_tapped_lands() {
    let mut g = main_phase();
    for seat in 0..2 {
        let land = g.add_card_to_battlefield(seat, catalog::mountain());
        g.battlefield_find_mut(land).unwrap().tapped = true;
    }
    let target = g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::ignition_team());
    cast(&mut g, 0, id, None);
    assert_eq!(
        g.battlefield_find(id).expect("team").counter_count(CounterType::PlusOnePlusOne),
        2
    );
    activate_n(&mut g, 0, id, 0, Some(Target::Permanent(target)));
    let cp = g.computed_permanent(target).expect("land");
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
}

/// Magister of Worth's condemnation spares only itself.
#[test]
fn magister_of_worth_condemns_everything_else() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::magister_of_worth());
    ballot(&mut g, [1, 1]); // condemnation
    cast(&mut g, 0, id, None);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(id).is_some(), "the Angel survives");
}

/// Marchesa hands out dethrone and buys back countered creatures.
#[test]
fn marchesa_returns_a_countered_creature() {
    let mut g = main_phase();
    let marchesa = g.add_card_to_battlefield(0, catalog::marchesa_the_black_rose());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.battlefield_find(bear).expect("bear").clone();
    assert_eq!(g.statics_granted_triggers_for(&card).len(), 1, "granted dethrone");
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let mut events = vec![];
    g.destroy_permanent(bear, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "back at the next end step");
    assert!(g.battlefield_find(marchesa).is_some());
}

/// Marchesa's Smuggler slips a creature past the blockers.
#[test]
fn marchesas_smuggler_grants_haste_and_evasion() {
    let mut g = main_phase();
    let smuggler = g.add_card_to_battlefield(0, catalog::marchesas_smuggler());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate_n(&mut g, 0, smuggler, 0, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).expect("bear");
    assert!(cp.keywords.contains(&Keyword::Haste));
    assert!(cp.keywords.contains(&Keyword::Unblockable));
}

/// Muzzio digs as deep as your biggest artifact.
#[test]
fn muzzio_deploys_an_artifact_from_the_top() {
    let mut g = main_phase();
    let muzzio = g.add_card_to_battlefield(0, catalog::muzzio_visionary_architect());
    g.clear_sickness(muzzio);
    g.add_card_to_battlefield(0, catalog::sol_ring()); // mana value 1
    let rock = g.add_card_to_library(0, catalog::sol_ring());
    let top = g.players[0].library.iter().position(|c| c.id == rock).expect("in library");
    let card = g.players[0].library.remove(top);
    g.players[0].library.insert(0, card);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![rock])]));
    activate_n(&mut g, 0, muzzio, 0, None);
    assert!(g.battlefield_find(rock).is_some(), "put onto the battlefield");
}

/// Reign of the Pit's Demon is as big as the bodies it ate.
#[test]
fn reign_of_the_pit_sizes_its_demon() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power
    g.add_card_to_battlefield(1, catalog::sengir_vampire()); // 4 power
    let id = g.add_card_to_hand(0, catalog::reign_of_the_pit());
    cast(&mut g, 0, id, None);
    let demon = g.battlefield.iter().find(|c| c.is_token).expect("Demon");
    assert_eq!((demon.definition.power, demon.definition.toughness), (6, 6));
}

/// Scourge of the Throne untaps the team and buys a second combat.
#[test]
fn scourge_of_the_throne_buys_a_second_combat() {
    let mut g = main_phase();
    let dragon = g.add_card_to_battlefield(0, catalog::scourge_of_the_throne());
    g.clear_sickness(dragon);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: dragon, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(dragon).expect("dragon").tapped, "untapped by its own trigger");
    assert_eq!(g.additional_combat_phases, 1);
}

/// Split Decision counters on a denial majority.
#[test]
fn split_decision_counters_on_denial() {
    let mut g = main_phase();
    mana(&mut g, 1);
    let bolt = g.add_card_to_hand(1, catalog::shock());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast bolt");
    let id = g.add_card_to_hand(0, catalog::split_decision());
    ballot(&mut g, [0, 0]); // denial
    cast(&mut g, 0, id, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, 20, "the bolt was countered");
}

/// CR 100.2 — Advantageous Proclamation shaves five off the minimum deck size.
#[test]
fn advantageous_proclamation_shrinks_the_minimum_deck() {
    use crabomination::format::{Deck, Format, validate_full_deck};
    let main: Vec<_> = (0..55).map(|_| catalog::mountain()).collect();
    let deck = Deck { main: main.clone(), sideboard: vec![], commanders: vec![] };
    assert!(validate_full_deck(&deck, Format::Standard).is_err(), "55 is short");
    let deck = Deck {
        main,
        sideboard: vec![catalog::advantageous_proclamation()],
        commanders: vec![],
    };
    assert!(validate_full_deck(&deck, Format::Standard).is_ok(), "55 clears the reduced floor");
}

/// CR 103.4 — Backup Plan deals an extra opening hand and shuffles the rest back.
#[test]
fn backup_plan_deals_a_second_opening_hand() {
    let mut g = main_phase();
    g.seat_conspiracy(0, catalog::backup_plan(), None);
    let library = g.players[0].library.len();
    g.start_mulligan_phase();
    assert_eq!(g.players[0].hand.len(), 7, "one hand kept");
    assert_eq!(g.players[0].library.len(), library - 7, "the other hand went back");
}

/// Unexpected Potential's chosen name casts off any colour.
#[test]
fn unexpected_potential_pays_the_named_spell_off_colour() {
    let mut g = main_phase();
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 2);
    let cast = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
    };
    assert!(cast(&mut g).is_err(), "no green mana");
    let agenda = g.seat_conspiracy(0, catalog::unexpected_potential(), Some("Grizzly Bears"));
    assert!(cast(&mut g).is_err(), "face down does nothing");
    g.reveal_hidden_agenda(0, agenda);
    assert!(cast(&mut g).is_ok(), "blue mana pays the green pip");
}

/// Deal Broker loots.
#[test]
fn deal_broker_loots() {
    let mut g = main_phase();
    let broker = g.add_card_to_battlefield(0, catalog::deal_broker());
    g.clear_sickness(broker);
    let hand = g.players[0].hand.len();
    activate_n(&mut g, 0, broker, 0, None);
    assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
    assert_eq!(g.players[0].graveyard.len(), 1);
}
