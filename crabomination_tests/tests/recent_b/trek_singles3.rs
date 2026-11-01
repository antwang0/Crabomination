//! Star Trek: the third batch of EDHREC closers (`decks::trek_singles3`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn counters(g: &GameState, id: CardId, kind: CounterType) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(kind))
}

fn play_land(g: &mut GameState, seat: usize) {
    let land = g.add_card_to_hand(seat, catalog::forest());
    g.players[seat].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(g);
}

fn resolve(g: &mut GameState, effect: Effect, ctx: &EffectContext) {
    let evs = g.resolve_effect(&effect, ctx).expect("resolve");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// Lwaxana Troi (CR 603.2 / 611.2c): while she stays tapped, the chosen
/// creature's landfall triggers twice; once she untaps it triggers once again.
#[test]
fn lwaxana_doubles_the_chosen_creatures_triggers_while_tapped() {
    let mut g = pod(2);
    let lwaxana = ready(&mut g, 0, catalog::lwaxana_troi_charming_guest());
    let liq = ready(&mut g, 0, catalog::syndicate_liquidators());
    flood(&mut g, 0);
    activate(&mut g, lwaxana, 0, Some(Target::Permanent(liq)));
    // The landfall target is the auto-pick's; count the counters it placed.
    let placed = |g: &GameState| counters(g, liq, CounterType::PlusOnePlusOne) + counters(g, lwaxana, CounterType::PlusOnePlusOne);
    play_land(&mut g, 0);
    assert_eq!(placed(&g), 4, "landfall fired twice");
    g.battlefield_find_mut(lwaxana).unwrap().tapped = false;
    g.check_state_based_actions();
    play_land(&mut g, 0);
    assert_eq!(placed(&g), 6, "untapped: once again");
}

/// Kira Nerys (CR 614.1c / 508.4): a creature entering while you control an
/// attacker enters tapped and attacking that attacker's defender — never
/// declared — and gets +1/+1 until end of turn.
#[test]
fn kira_nerys_puts_entering_creatures_into_the_attack() {
    let mut g = pod(3);
    let kira = ready(&mut g, 0, catalog::kira_nerys_firebrand());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: kira, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(&mut g);
    let viper = g.add_card_to_hand(0, catalog::ambush_viper());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: viper,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flash");
    drain_stack(&mut g);
    let v = g.battlefield_find(viper).expect("entered");
    assert!(v.tapped);
    assert!(g.attacking.iter().any(|a| a.attacker == viper && a.target == AttackTarget::Player(2)));
    let cp = g.computed_permanent(viper).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "Kira's +1/+1");
    // Outside combat a creature enters normally.
    let mut g = pod(2);
    ready(&mut g, 0, catalog::kira_nerys_firebrand());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.battlefield_find(bear).unwrap().tapped);
}

/// Planet Killer: combat damage to a player adds a +1/+1 counter per land
/// *that* player controls.
#[test]
fn planet_killer_grows_by_the_damaged_players_lands() {
    let mut g = pod(3);
    let pk = ready(&mut g, 0, catalog::planet_killer());
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::island());
    }
    g.add_card_to_battlefield(2, catalog::island());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: pk, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(counters(&g, pk, CounterType::PlusOnePlusOne), 3);
}

/// Artificial Anomaly at four seats: each opponent loses 2, you gain 2 once,
/// and a tapped New Planet that taps for any color arrives.
#[test]
fn artificial_anomaly_drains_each_opponent_and_makes_a_planet() {
    let mut g = pod(4);
    let land = ready(&mut g, 0, catalog::artificial_anomaly());
    flood(&mut g, 0);
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    activate(&mut g, land, 1, None);
    assert_eq!(g.players[0].life, life[0] + 2);
    for (seat, before) in life.iter().enumerate().skip(1) {
        assert_eq!(g.players[seat].life, before - 2);
    }
    assert!(g.battlefield_find(land).is_none(), "sacrificed");
    let planet = g.battlefield.iter().find(|c| c.definition.name == "New Planet").expect("planet");
    assert!(planet.tapped && planet.controller == 0);
}

/// Beckett Mariner: every tap adds a promotion counter; the third removes
/// them all and deals 2 to each opponent.
#[test]
fn beckett_mariner_cashes_in_three_promotions() {
    let mut g = pod(3);
    let beckett = ready(&mut g, 0, catalog::beckett_mariner_impetuous_ensign());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let life1 = g.players[1].life;
    for n in 1..=3 {
        g.battlefield_find_mut(beckett).unwrap().tapped = false;
        resolve(&mut g, Effect::Tap { what: Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature) }, &ctx);
        if n < 3 {
            assert_eq!(counters(&g, beckett, CounterType::Promotion), n);
            assert_eq!(g.players[1].life, life1);
        }
    }
    assert_eq!(counters(&g, beckett, CounterType::Promotion), 0);
    assert_eq!(g.players[1].life, life1 - 2);
    assert_eq!(g.players[2].life, life1 - 2);
}

/// Urza's Tome: with a historic card in the graveyard you may exile it rather
/// than discard; without one, the discard is forced.
#[test]
fn urzas_tome_exiles_a_historic_card_instead_of_discarding() {
    let mut g = pod(2);
    let tome = ready(&mut g, 0, catalog::urzas_tome());
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, tome, 0, None);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew, kept it");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != ring), "Sol Ring exiled");
    let mut g = pod(2);
    let tome = ready(&mut g, 0, catalog::urzas_tome());
    g.add_card_to_graveyard(0, catalog::island());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    activate(&mut g, tome, 0, None);
    assert_eq!(g.players[0].hand.len(), hand, "no historic card: drew and discarded");
}

/// Troi, Winging It: an opponent drawing during *your* turn makes a Faerie;
/// during their own turn it doesn't.
#[test]
fn troi_winging_it_only_on_your_turn() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::troi_winging_it());
    let faeries = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Faerie").count();
    let mut ev = Vec::new();
    g.draw_one(1, &mut ev);
    g.draw_one(2, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(faeries(&g), 2);
    g.active_player_idx = 1;
    let mut ev = Vec::new();
    g.draw_one(1, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(faeries(&g), 2);
}

/// Sarcophagus: a creature dying charges it only while it isn't a creature
/// (an intervening-if, CR 603.4); your other creatures get +1/+0.
#[test]
fn sarcophagus_charges_on_deaths_while_not_a_creature() {
    let mut g = pod(2);
    let ship = ready(&mut g, 0, catalog::sarcophagus_ship_of_the_dead());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(foe)), 0, 0);
    resolve(&mut g, Effect::Destroy { what: Selector::Target(0) }, &ctx);
    assert_eq!(counters(&g, ship, CounterType::Charge), 1);
}

/// Test of Conviction's second mode: the target opponent draws one, you two.
#[test]
fn test_of_conviction_negotiations_draws() {
    let mut g = pod(3);
    let spell = g.add_card_to_hand(0, catalog::test_of_conviction());
    flood(&mut g, 0);
    let (h0, h2) = (g.players[0].hand.len(), g.players[2].hand.len());
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(2)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 - 1 + 2);
    assert_eq!(g.players[2].hand.len(), h2 + 1);
}

/// Chief Kyle: landfall exiles another creature of yours, which returns under
/// its owner's control at the next end step.
#[test]
fn chief_kyle_blinks_until_the_end_step() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::chief_kyle_transporter_tech());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    play_land(&mut g, 0);
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Grizzly Bears"), "exiled");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0), "{bear:?} back");
}

/// Federation Resupply's Beam me up (CR 702.34a flashback with an additional
/// cost): cast from the graveyard, a creature you control returns to hand,
/// the search finds an Equipment, and the card is exiled.
#[test]
fn federation_resupply_beams_up_from_the_graveyard() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.add_card_to_graveyard(0, catalog::federation_resupply());
    g.add_card_to_library(0, catalog::bonesplitter());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastFlashback {
        card_id: card,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("beam me up");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "returned as the cost");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Bonesplitter"));
    assert!(g.exile.iter().any(|c| c.id == card));
}

/// T'Pring: the chosen creatures fight; the one that dies this turn draws a
/// card (each creature that fought is watched, CR 603.7c).
#[test]
fn tpring_fight_draws_for_the_creature_that_died() {
    let mut g = pod(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tp = g.add_card_to_hand(0, catalog::tpring_never_parted());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell { card_id: tp, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let _ = giant;
    assert!(g.battlefield_find(bear).is_none(), "the Bears died fighting");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "one creature died: one card");
}

/// Dominion Saboteur (CR 707.9b + 614.1c): a copy of a legend that isn't
/// legendary, entering with the copied permanent's counters, kind for kind.
#[test]
fn dominion_saboteur_copies_the_counters_and_drops_legendary() {
    let mut g = pod(2);
    let kira = g.add_card_to_battlefield(1, catalog::kira_nerys_firebrand());
    {
        let k = g.battlefield_find_mut(kira).unwrap();
        k.counters.add(CounterType::PlusOnePlusOne, 2);
        k.counters.add(CounterType::Charge, 1);
    }
    let sab = g.add_card_to_hand(0, catalog::dominion_saboteur());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: sab, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let c = g.battlefield_find(sab).expect("entered");
    assert_eq!(c.definition.name, "Kira Nerys, Firebrand");
    assert!(!c.definition.supertypes.contains(&crabomination::card::Supertype::Legendary));
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(c.counter_count(CounterType::Charge), 1);
}

/// The Guardian of Forever: the sacrificed nontoken creature card returns at
/// the beginning of the next end step.
#[test]
fn guardian_of_forever_returns_the_sacrificed_card_at_end_step() {
    let mut g = pod(2);
    let guardian = ready(&mut g, 0, catalog::the_guardian_of_forever());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    activate(&mut g, guardian, 0, None);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear), "sacrificed");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0), "returned");
}

/// Deep Space Scan: a card per differently named land, and no maximum hand
/// size until your next turn — seven again at your next upkeep.
#[test]
fn deep_space_scan_lifts_the_hand_limit_until_your_next_turn() {
    let mut g = pod(2);
    for land in [catalog::island(), catalog::forest(), catalog::forest()] {
        g.add_card_to_battlefield(0, land);
    }
    let scan = g.add_card_to_hand(0, catalog::deep_space_scan());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell { card_id: scan, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "two land names");
    assert_eq!(g.players[0].max_hand_size, None);
    g.turn_number += 2;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].max_hand_size, Some(7));
}

/// Command Chair (CR 903.3): its second ability makes X mana of one color, X
/// the greatest mana value among commanders you control.
#[test]
fn command_chair_taps_for_your_commanders_mana_value() {
    let mut g = pod(2);
    let cmd = g.seat_commanders(0, vec![catalog::tpring_never_parted()])[0];
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
    assert!(g.battlefield_find(cmd).is_some());
    g.players[0].mana_pool = Default::default();
    let chair = ready(&mut g, 0, catalog::command_chair());
    g.battlefield_find_mut(chair).unwrap().tapped = false;
    activate(&mut g, chair, 1, None);
    assert_eq!(g.players[0].mana_pool.total(), 5, "T'Pring's mana value");
}

/// Hologram Stalker: its own return is a card leaving your graveyard, which
/// makes an Illusion — once each turn (CR 603.3d... "only once each turn").
#[test]
fn hologram_stalker_illusion_once_each_turn() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let stalker = g.add_card_to_hand(0, catalog::hologram_stalker());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: stalker, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let illusions = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Illusion").count();
    assert_eq!(g.players[0].graveyard.len(), 1, "one card returned");
    assert_eq!(illusions(&g), 1);
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(giant)), 0, 0);
    resolve(
        &mut g,
        Effect::Move {
            what: Selector::Target(0),
            to: crabomination::effect::ZoneDest::Hand(crabomination::effect::PlayerRef::You),
        },
        &ctx,
    );
    assert_eq!(illusions(&g), 1, "only once each turn");
}

/// Replicator: a Food on arrival, then a tapped copy of a token you control.
#[test]
fn replicator_copies_a_token_tapped() {
    let mut g = pod(2);
    let rep = g.add_card_to_hand(0, catalog::replicator());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: rep, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let food = g.battlefield.iter().find(|c| c.definition.name == "Food").expect("food").id;
    g.clear_sickness(rep);
    activate(&mut g, rep, 0, Some(Target::Permanent(food)));
    let foods: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Food").collect();
    assert_eq!(foods.len(), 2);
    assert!(foods.iter().any(|c| c.tapped));
}

/// U.S.S. Farragut: landfall puts a creature from hand in with haste; at the
/// next end step it goes to the bottom of its owner's library and you draw.
#[test]
fn farragut_borrows_a_creature_until_the_end_step() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::u_s_s_farragut_bold_explorer());
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    play_land(&mut g, 0);
    assert!(g.battlefield_find(giant).is_some(), "put onto the battlefield");
    let hand = g.players[0].hand.len();
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(giant), "on the bottom");
    assert_eq!(g.players[0].hand.len(), hand + 1, "and you draw");
}

/// S.S. Botany Bay: its tap exiles a card from each opponent's graveyard (and
/// one of yours); stationed, combat damage returns your creature cards exiled
/// with it.
#[test]
fn botany_bay_exiles_and_returns_your_creatures() {
    let mut g = pod(3);
    let ship = ready(&mut g, 0, catalog::s_s_botany_bay());
    g.add_card_to_graveyard(1, catalog::island());
    g.add_card_to_graveyard(2, catalog::forest());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, ship, 0, None);
    assert!(g.players[1].graveyard.is_empty() && g.players[2].graveyard.is_empty());
    assert!(g.exile.iter().any(|c| c.id == giant));
    g.battlefield_find_mut(ship).unwrap().counters.add(CounterType::Charge, 8);
    g.battlefield_find_mut(ship).unwrap().tapped = false;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: ship, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == giant && c.controller == 0), "the Giant returns");
}
