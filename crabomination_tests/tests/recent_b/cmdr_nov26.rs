//! EDHREC singles short one card (`decks::cmdr_nov26`, `decks::cmdr_nov26_b`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Selector, Value};
use crabomination::game::effects::EntityRef;
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

fn cast(g: &mut GameState, id: CardId) {
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// Attack `defender` with `attacker`, no blocks, through combat damage.
fn hit(g: &mut GameState, attacker: CardId, defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    for _ in 0..64 {
        if matches!(g.step, TurnStep::EndCombat | TurnStep::PostCombatMain) {
            drain_stack(g);
            return;
        }
        let r = g.perform_action(GameAction::PassPriority);
        if r.is_err() && g.step == TurnStep::DeclareBlockers {
            g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
        }
    }
    panic!("combat never ended: step {:?}, stack {}, pending {:?}", g.step, g.stack.len(), g.pending_decision.is_some());
}

/// Massimo: a creature entering exiles an MV-1 instant from your graveyard and
/// gains "copy it on combat damage to a player"; the copy is cast free. An MV-2
/// instant is not a legal target, so nothing is granted.
#[test]
fn massimo_links_an_exiled_cantrip_to_the_entering_creature() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::massimo_the_magician());
    let opt = g.add_card_to_graveyard(0, catalog::opt());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bear);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != opt), "Opt exiled");
    assert_eq!(g.exile.iter().find(|c| c.id == opt).and_then(|c| c.exiled_with), Some(bear), "linked to the Bears");
    g.clear_sickness(bear);
    let (hand, life) = (g.players[0].hand.len(), g.players[2].life);
    hit(&mut g, bear, 2);
    assert_eq!(g.players[2].life, life - 2);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the free Opt copy drew a card");
    assert!(g.exile.iter().any(|c| c.id == opt), "the original stays exiled");

    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::massimo_the_magician());
    let cn = g.add_card_to_graveyard(0, catalog::counterspell());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bear);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cn), "MV 2: not a target");
    assert!(g.battlefield_find(bear).is_some_and(|c| c.definition.triggered_abilities.is_empty()));
}

// ── decks::cmdr_nov26_b ──────────────────────────────────────────────────────


/// `seat` resolves "`who` loses `n` life" as its own ability, then dispatches.
fn lose(g: &mut GameState, seat: usize, who: PlayerRef, n: i32) {
    let ctx = EffectContext::for_spell(seat, None, 0, 0);
    let evs = g
        .resolve_effect(&Effect::LoseLife { who: Selector::Player(who), amount: Value::Const(n) }, &ctx)
        .expect("lose life");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn plus_ones(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// Sahir: its own eminence ping (1 damage from a source you control) and a
/// 1-life loss your ability causes each grow it; the second resolution this
/// turn draws. A 2-life loss and an opponent's 1-life drain don't count, and
/// the ping is one trigger, not two (its life loss is the damage's).
#[test]
fn sahir_grows_on_exactly_one_from_your_own_sources() {
    let mut g = pod(3);
    let sahir = g.add_card_to_battlefield(0, catalog::sahir_visitor_in_darkness());
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "eminence ping");
    assert_eq!(plus_ones(&g, sahir), 1, "one trigger for one ping");
    assert_eq!(g.players[0].hand.len(), hand);
    lose(&mut g, 0, PlayerRef::You, 1);
    assert_eq!(plus_ones(&g, sahir), 2);
    assert_eq!(g.players[0].hand.len(), hand + 1, "second resolution this turn draws");
    lose(&mut g, 0, PlayerRef::You, 2);
    lose(&mut g, 1, PlayerRef::EachOpponent, 1);
    assert_eq!(plus_ones(&g, sahir), 2, "not exactly 1, or not your ability");
}

/// CR 113.6b — Sahir's eminence pings from the command zone, and the damage is
/// attributed to its owner (a source they control).
#[test]
fn cr_113_6b_sahir_pings_from_the_command_zone_as_its_owners_source() {
    let mut g = pod(2);
    g.seat_commanders(0, vec![catalog::sahir_visitor_in_darkness()]);
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1);
    let id = g.players[0].command[0].id;
    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 1, Some(id), &mut ev);
    assert!(ev.iter().any(|e| matches!(e, GameEvent::DamageDealt { from_controller: Some(0), .. })));
}

/// Peace in Our Time: the active player may draw at their beginning of combat;
/// if they do, their creatures can't attack and a peace counter lands. Twelve
/// counters at its controller's upkeep win the game (CR 603.4 intervening if).
#[test]
fn peace_in_our_time_trades_attacks_for_cards_and_wins_at_twelve() {
    let mut g = pod(3);
    let peace = g.add_card_to_battlefield(1, catalog::peace_in_our_time());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the active player drew");
    assert_eq!(g.battlefield_find(peace).unwrap().counter_count(CounterType::Peace), 1);
    g.step = TurnStep::DeclareAttackers;
    let atk = GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(2) }]);
    assert!(g.perform_action(atk).is_err(), "can't attack this turn");

    g.battlefield_find_mut(peace).unwrap().add_counters(CounterType::Peace, 11);
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].eliminated && g.players[2].eliminated, "twelve counters win");
}

/// Creeping Dread: everyone discards; only an opponent whose discard shares a
/// card type with yours loses 3.
#[test]
fn creeping_dread_punishes_matching_discards() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::creeping_dread());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::llanowar_elves());
    g.add_card_to_hand(2, catalog::island());
    let (l1, l2) = (g.players[1].life, g.players[2].life);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players.iter().all(|p| p.hand.is_empty()), "each player discarded");
    assert_eq!(g.players[1].life, l1 - 3, "creature matches creature");
    assert_eq!(g.players[2].life, l2, "a land doesn't match");
}

/// Dominion Infiltration: X=2 copies two opponents' creatures as hasty
/// changeling tokens for the caster, exiled at the next end step.
#[test]
fn dominion_infiltration_borrows_x_hasty_changeling_copies() {
    let mut g = pod(3);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::llanowar_elves());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::dominion_infiltration());
    g.players[0].mana_pool.add(Color::Red, 5);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    let tokens: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).map(|c| c.id).collect();
    assert_eq!(tokens.len(), 2);
    for t in &tokens {
        let cp = g.computed_permanent(*t).unwrap();
        assert!(cp.keywords().contains(&Keyword::Haste) && cp.keywords().contains(&Keyword::Changeling));
    }
    assert!(g.battlefield_find(mine).is_some());
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(tokens.iter().all(|t| g.battlefield_find(*t).is_none()), "exiled at the end step");
}

/// Bot: Peace in Our Time's draw is taken by its controller (the counter is
/// their win) and by a seat with nothing ready to attack, and declined by a
/// seat that has an attacker.
#[test]
fn bot_stands_down_for_peace_only_without_an_attack() {
    use crabomination::decision::OptionalKind;
    use crabomination::server::bot::decide_optional_trigger;
    let w = crabomination::server::EvalWeights::default();
    let desc = "Draw a card? (You can't attack this turn.)";
    let mut g = pod(3);
    let peace = g.add_card_to_battlefield(1, catalog::peace_in_our_time());
    let ask = |g: &GameState, seat| decide_optional_trigger(g, seat, peace, &OptionalKind::MayBody, desc, &w, false);
    assert!(ask(&g, 1), "the controller draws");
    assert!(ask(&g, 0), "nothing to attack with");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(ask(&g, 0), "a summoning-sick Bear can't attack");
    g.clear_sickness(bear);
    assert!(!ask(&g, 0), "an attacker ready: decline");
}

// ── decks::cmdr_nov26_c ──────────────────────────────────────────────────────

/// Romulan Bird-of-Prey: hexproof and charging off noncreature spells while it
/// isn't a creature; at eight counters it is a 4/4 flier and both stop.
/// Anthropological Mission: quest counter + loot, a pure draw from three on.
#[test]
fn bird_of_prey_cloaks_and_charges_and_mission_loots_then_draws() {
    let mut g = pod(2);
    let bird = g.add_card_to_battlefield(0, catalog::romulan_bird_of_prey());
    let mission = g.add_card_to_battlefield(0, catalog::anthropological_mission());
    assert!(g.computed_permanent(bird).unwrap().keywords().contains(&Keyword::Hexproof));
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    for i in 0..4 {
        let opt = g.add_card_to_hand(0, catalog::opt());
        g.players[0].mana_pool.add(Color::Blue, 1);
        let hand = g.players[0].hand.len();
        cast(&mut g, opt);
        // Opt (-1) scries and draws (+1); the Mission draws (+1) and, below
        // three quest counters, discards (-1).
        let looted = if i < 2 { 0 } else { 1 };
        assert_eq!(g.players[0].hand.len(), hand + looted, "cast {i}");
    }
    assert_eq!(g.battlefield_find(mission).unwrap().counter_count(CounterType::Quest), 4);
    let c = g.battlefield_find(bird).unwrap().counter_count(CounterType::Charge);
    assert_eq!(c, 8, "two a spell until it is a creature");
    let cp = g.computed_permanent(bird).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature) && cp.keywords().contains(&Keyword::Flying));
    assert!(!cp.keywords().contains(&Keyword::Hexproof), "a creature: decloaked");
}

/// Planetary Bombardment deals 4 once a creature died this turn, else 2;
/// Boarding Action makes a hasty 2/1 Klingon Warrior.
#[test]
fn planetary_bombardment_scales_on_a_death_and_boarding_action_makes_a_warrior() {
    let mut g = pod(2);
    let ogre = g.add_card_to_battlefield(1, catalog::hill_giant());
    let bomb = g.add_card_to_hand(0, catalog::planetary_bombardment());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: bomb, target: Some(Target::Permanent(ogre)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ogre).is_some(), "2 damage doesn't kill a 3/3");
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None }).expect("bolt");
    drain_stack(&mut g);
    let ogre2 = g.add_card_to_battlefield(1, catalog::hill_giant());
    let bomb = g.add_card_to_hand(0, catalog::planetary_bombardment());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell { card_id: bomb, target: Some(Target::Permanent(ogre2)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ogre2).is_none(), "4 after a death");

    let ba = g.add_card_to_hand(0, catalog::boarding_action());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, ba);
    let tok = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token).expect("token");
    assert_eq!((tok.definition.power, tok.definition.toughness), (2, 1));
    assert!(tok.definition.keywords.contains(&Keyword::Haste));
}

/// Brazen Bluff: when the targeted spell's controller declines to exile it,
/// the Bluff's caster casts a noncreature spell from hand free; when they
/// exile it, nothing is cast.
#[test]
fn brazen_bluff_exiles_or_grants_a_free_spell() {
    for exile in [false, true] {
        let mut g = pod(2);
        let divination = g.add_card_to_hand(1, catalog::divination());
        let growth = g.add_card_to_hand(0, catalog::opt());
        g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(exile), DecisionAnswer::Cards(vec![growth]), DecisionAnswer::Bool(true)]));
        let bluff = g.add_card_to_hand(0, catalog::brazen_bluff());
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.players[1].mana_pool.add(Color::Blue, 3);
        g.perform_action(GameAction::CastSpell { card_id: divination, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Red, 2);
        g.perform_action(GameAction::CastSpell { card_id: bluff, target: Some(Target::Permanent(divination)), additional_targets: vec![], mode: None, x_value: None }).expect("bluff");
        drain_stack(&mut g);
        assert_eq!(g.exile.iter().any(|c| c.id == divination), exile, "exile={exile}");
        assert_eq!(g.players[0].hand.iter().any(|c| c.id == growth), exile, "free cast iff not exiled");
    }
}

/// Officer makers: M'Ress's landfall makes two, Galileo's one; Academy
/// Instructor counts Officer tokens and makes one per noncreature spell;
/// Subspace Anomaly sacrifices for two Officers and a tapped New Planet.
#[test]
fn officer_makers_and_the_instructor_count_officer_tokens() {
    let officers = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Officer").count();
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mress_crew_coordinator());
    g.add_card_to_battlefield(0, catalog::shuttlecraft_galileo());
    let inst = g.add_card_to_battlefield(0, catalog::academy_instructor());
    let land = g.add_card_to_hand(0, catalog::plains());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    assert_eq!(officers(&g), 3, "two from M'Ress, one from Galileo");
    let opt = g.add_card_to_hand(0, catalog::opt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, opt);
    assert_eq!(officers(&g), 4);
    assert_eq!(g.computed_permanent(inst).unwrap().power, 4);

    let mut g = pod(2);
    let anomaly = g.add_card_to_battlefield(0, catalog::subspace_anomaly());
    g.players[0].mana_pool.add(Color::White, 5);
    g.perform_action(GameAction::ActivateAbility { card_id: anomaly, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None }).expect("sac");
    drain_stack(&mut g);
    assert_eq!(officers(&g), 2);
    let planet = g.battlefield.iter().find(|c| c.definition.name == "New Planet").expect("planet");
    assert!(planet.tapped && g.battlefield_find(anomaly).is_none());
}

/// Rutherford: casting a Spacecraft spell puts a +1/+1 counter on target
/// creature.
#[test]
fn rutherford_counters_on_spacecraft_casts() {
    let mut g = pod(2);
    let ruth = g.add_card_to_battlefield(0, catalog::rutherford_upgraded_ensign());
    let bird = g.add_card_to_hand(0, catalog::romulan_bird_of_prey());
    g.players[0].mana_pool.add(Color::Blue, 2);
    cast(&mut g, bird);
    assert_eq!(plus_ones(&g, ruth), 1);
}

/// Brad Boimler: once tapped, each counter placement on your permanents this
/// turn gets one more of its kind — +1/+1 on a creature, charge on a
/// noncreature Spacecraft — and the bonus ends at cleanup.
#[test]
fn brad_boimler_tapped_adds_one_of_each_kind_this_turn() {
    let mut g = pod(2);
    let brad = g.add_card_to_battlefield(0, catalog::brad_boimler_eager_ensign());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ship = g.add_card_to_battlefield(0, catalog::shuttlecraft_galileo());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let tap = g.resolve_effect(&Effect::Tap { what: Selector::ExactObjects(vec![brad]) }, &ctx).expect("tap");
    g.dispatch_triggers_for_events(&tap);
    drain_stack(&mut g);
    for (id, kind) in [(bear, CounterType::PlusOnePlusOne), (ship, CounterType::Charge)] {
        g.resolve_effect(&Effect::AddCounter { what: Selector::ExactObjects(vec![id]), kind, amount: Value::ONE }, &ctx)
            .expect("counter");
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(kind), 2, "{kind:?}: one plus one");
    }
    assert_eq!(g.players[0].extra_counters_this_turn, 1);
}

/// Itazura: three cards are exiled, the opponents bid (APNAP: seat 1, seat 2);
/// the highest bidder takes that much damage from Itazura and casts its pick
/// free, and Itazura's controller keeps a card that wasn't cast. The third
/// card stays in exile.
#[test]
fn itazura_sells_a_free_spell_to_the_highest_bidder() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let opt = g.add_card_to_library(0, catalog::opt());
    let island = g.add_card_to_library(0, catalog::island());
    for seat in 0..3 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g.add_card_to_battlefield(0, catalog::itazura_lingering_wick());
    let (one, two) = (g.players[1].life, g.players[2].life);
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Amount(2),
        DecisionAnswer::Amount(1),
        DecisionAnswer::Cards(vec![bears]),
        DecisionAnswer::Cards(vec![opt]),
        DecisionAnswer::Bool(true),
    ]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!((g.players[1].life, g.players[2].life), (one - 2, two), "the high bid is the damage");
    let b = g.battlefield.iter().find(|c| c.id == bears).expect("bears cast free");
    assert_eq!(b.controller, 1);
    assert!(g.players[0].hand.iter().any(|c| c.id == opt), "the controller keeps Opt");
    assert!(g.exile.iter().any(|c| c.id == island), "the third card stays exiled");
}

/// Itazura with a headless (Auto) decider: both opponents bid 0, the tie goes
/// to the lower life total, nobody is damaged, and the winner still casts.
#[test]
fn itazura_ties_go_to_the_lowest_life_and_zero_deals_nothing() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::opt());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::itazura_lingering_wick());
    g.players[2].life -= 3;
    let (one, two) = (g.players[1].life, g.players[2].life);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!((g.players[1].life, g.players[2].life), (one, two));
    assert_eq!(g.battlefield.iter().find(|c| c.id == bears).map(|c| c.controller), Some(2));
    assert_eq!(g.players[0].hand.len(), 1, "one card kept");
}

/// Miss Highwater: combat damage offers the wheel; accepting draws seven and
/// takes a contract counter, so the next hit doesn't trigger. When the
/// contracted player loses (CR 800.4a — before their objects leave), Highwater's
/// controller gets a token copy of each artifact and creature they controlled;
/// their lands are not copied.
#[test]
fn miss_highwater_collects_on_a_contract_when_the_player_loses() {
    let mut g = pod(3);
    let hw = g.add_card_to_battlefield(0, catalog::miss_highwater());
    g.battlefield.iter_mut().for_each(|c| c.summoning_sick = false);
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    hit(&mut g, hw, 1);
    assert_eq!(g.players[1].hand.len(), 7, "wheeled into seven");
    assert_eq!(g.players[1].graveyard.len(), 3);
    assert_eq!(g.players[1].contracts.len(), 1);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.battlefield.iter_mut().for_each(|c| c.tapped = false);
    hit(&mut g, hw, 1);
    assert_eq!(g.players[1].hand.len(), 7, "a contract holder isn't offered again");
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::sol_ring());
    g.add_card_to_battlefield(1, catalog::forest());
    g.players[1].life = 0;
    g.check_state_based_actions();
    assert!(!g.players[1].is_alive());
    drain_stack(&mut g);
    let mine = |name: &str| g.battlefield.iter().any(|c| c.controller == 0 && c.is_token && c.definition.name == name);
    assert!(mine("Grizzly Bears") && mine("Sol Ring"), "the creature and the artifact are copied");
    assert!(!mine("Forest"), "lands are not");
}

/// Salinae: an artifact spell's copy resolves as a 1/1 Fish artifact creature
/// token (CR 707.9b), and devotion to blue discounts her.
#[test]
fn salinae_copies_an_artifact_spell_as_a_one_one_fish() {
    use crabomination::card::{CardType, CreatureType};
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::salinae_the_ocean_mother());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, ring);
    let fish = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token).expect("copy token");
    let cp = g.computed_permanent(fish.id).unwrap();
    assert!(cp.card_types().contains(&CardType::Artifact) && cp.card_types().contains(&CardType::Creature));
    assert!(fish.definition.subtypes.creature_types.contains(&CreatureType::Fish));
    assert_eq!((cp.power, cp.toughness), (1, 1));

    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::merfolk_of_the_pearl_trident());
    }
    let sal = g.add_card_to_hand(0, catalog::salinae_the_ocean_mother());
    g.players[0].mana_pool.add(Color::Blue, 5);
    cast(&mut g, sal);
    assert!(g.battlefield_find(sal).is_some(), "eight less three devotion");
}

/// Invasion of Kamigawa taps an opponent's creature and stuns it.
#[test]
fn invasion_of_kamigawa_taps_and_stuns() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let inv = g.add_card_to_hand(0, catalog::invasion_of_kamigawa());
    g.players[0].mana_pool.add(Color::Blue, 4);
    cast(&mut g, inv);
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.tapped && b.counter_count(CounterType::Stun) == 1);
}

// ── decks::cmdr_nov27_b ───────────────────────────────────────────────────────

/// McCoy: his entry (and a Doctor's) puts a +1/+1 counter on each of your
/// creatures, and your creatures with counters have lifelink.
#[test]
fn mccoy_counters_the_team_and_grants_lifelink_to_countered_creatures() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mccoy = g.add_card_to_hand(0, catalog::mccoy_not_a_bricklayer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, mccoy);
    assert_eq!(plus_ones(&g, bear), 1);
    assert_eq!(plus_ones(&g, mccoy), 1);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Lifelink));
    let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(fresh).unwrap().keywords().contains(&Keyword::Lifelink), "no counter, no lifelink");
}

/// Gone Missing tops the target on its owner's library and investigates.
#[test]
fn gone_missing_tops_a_permanent_and_makes_a_clue() {
    let mut g = pod(2);
    let ogre = g.add_card_to_battlefield(1, catalog::hill_giant());
    let gm = g.add_card_to_hand(0, catalog::gone_missing());
    g.players[0].mana_pool.add(Color::Blue, 5);
    g.perform_action(GameAction::CastSpell { card_id: gm, target: Some(Target::Permanent(ogre)), additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[1].library.iter().any(|c| c.id == ogre));
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"));
}

/// The Needs of the Many: sacrifice a creature; the rest become indestructible
/// and hexproof, damage to you is prevented, and the spell is exiled.
#[test]
fn the_needs_of_the_many_shields_the_team_and_exiles_itself() {
    let mut g = pod(2);
    let fodder = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::the_needs_of_the_many());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Cards(vec![fodder])]));
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, spell);
    assert!(g.battlefield_find(fodder).is_none() && g.battlefield_find(bear).is_some(), "one creature sacrificed");
    let kws = g.computed_permanent(bear).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::Indestructible) && kws.contains(&Keyword::Hexproof));
    assert!(g.exile.iter().any(|c| c.id == spell), "exiled on resolution");
    let life = g.players[0].life;
    let mut ev = Vec::new();
    g.deal_damage_to_from(EntityRef::Player(0), 3, None, &mut ev);
    assert_eq!(g.players[0].life, life, "damage to you prevented");
}

/// Troubling Tribbles copies itself each upkeep; Gul Dukat doubles an
/// opponent's life loss this turn at the end step and drains it.
#[test]
fn tribbles_multiply_and_gul_dukat_doubles_the_turns_loss() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::troubling_tribbles());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Troubling Tribbles").count(), 2);

    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::gul_dukat_colonial_prefect());
    lose(&mut g, 0, PlayerRef::Seat(1), 3);
    let (me, l1, l2) = (g.players[0].life, g.players[1].life, g.players[2].life);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 3);
    assert_eq!(g.players[2].life, l2, "lost nothing this turn");
    assert_eq!(g.players[0].life, me + 3);
}

/// Sash and Waistcoat: an opponent may activate the manifest ability (it
/// manifests from the activator's own library, CR 602.2), and a face-down
/// creature attacking one of Sash's opponents — anyone's — can't be blocked.
#[test]
fn sash_and_waistcoat_lets_anyone_manifest_and_face_downs_slip_past() {
    let mut g = pod(3);
    let sash = g.add_card_to_battlefield(0, catalog::sash_and_waistcoat_unmen());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Blue, 3);
    g.perform_action(GameAction::ActivateAbility { card_id: sash, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("seat 1 activates Sash's ability");
    drain_stack(&mut g);
    let fd = g.battlefield.iter().find(|c| c.face_down).expect("manifested");
    assert_eq!(fd.controller, 1, "the activator manifests from their own library");
    let fd = fd.id;
    g.battlefield.iter_mut().for_each(|c| c.summoning_sick = false);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: fd, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(fd).unwrap().keywords().contains(&Keyword::Unblockable));
}

/// Write into Being: one of the top two is manifested (a creature card when
/// there is one), the other stays in the library.
#[test]
fn write_into_being_manifests_one_of_two() {
    let mut g = multi_player_game(2);
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let island = g.add_card_to_library(0, catalog::island());
    let w = g.add_card_to_hand(0, catalog::write_into_being());
    g.players[0].mana_pool.add(Color::Blue, 3);
    cast(&mut g, w);
    assert!(g.battlefield.iter().any(|c| c.id == bears && c.face_down));
    assert!(g.players[0].library.iter().any(|c| c.id == island));
}

fn cast_x(g: &mut GameState, id: CardId, x: u32) {
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(x) })
        .expect("cast");
    drain_stack(g);
}

/// Glava: the first {X} permanent spell of the turn has X become 5; the
/// second keeps its own X (once each turn).
#[test]
fn glava_makes_the_first_x_five_once_each_turn() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::glava_five_advents_mage());
    let a = g.add_card_to_hand(0, catalog::endless_one());
    let b = g.add_card_to_hand(0, catalog::endless_one());
    g.players[0].mana_pool.add_colorless(2);
    cast_x(&mut g, a, 1);
    cast_x(&mut g, b, 1);
    assert_eq!(plus_ones(&g, a), 5, "X became 5");
    assert_eq!(plus_ones(&g, b), 1, "only once each turn");
}

/// Savage Summoning: the next creature spell has flash (cast on an opponent's
/// turn) and enters with an additional +1/+1 counter.
#[test]
fn savage_summoning_flashes_in_a_bigger_creature() {
    let mut g = pod(2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    let s = g.add_card_to_hand(0, catalog::savage_summoning());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 3);
    cast(&mut g, s);
    g.priority.player_with_priority = 0;
    cast(&mut g, bear);
    assert_eq!(plus_ones(&g, bear), 1);
}

/// Matoc: its own ability (exile two graveyard cards) deals exactly 2, so its
/// trigger exiles the top card with a play permission. Bug fix: "any other
/// target" (`any_target().and(OtherThanSource)`) rejected every player.
#[test]
fn matoc_pings_for_two_and_impulses_off_it() {
    let mut g = pod(2);
    let m = g.add_card_to_battlefield(0, catalog::matoc_lavamancer());
    g.battlefield.iter_mut().for_each(|c| c.summoning_sick = false);
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::island());
    let top = g.players[0].library[0].id;
    g.players[0].mana_pool.add(Color::Red, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: m,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
    assert!(g.players[0].graveyard.is_empty(), "two cards exiled as the cost");
    assert!(g.exile.iter().any(|c| c.id == top && c.may_play_until.is_some()));
}

/// Leyline of Combustion: an opponent's Lightning Bolt aimed at you costs its
/// caster 2.
#[test]
fn leyline_of_combustion_burns_whoever_targets_you() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::leyline_of_combustion());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (mine - 3, theirs - 2));
}

/// The Madcap Jester: its attack exiles each opponent's top card, and since
/// its own ability did it, Jester's controller may play each. Bitter Ordeal
/// (searching an opponent's library — CR 400.7, the search now announces the
/// library origin) is "a spell you control" too.
#[test]
fn madcap_jester_plays_what_your_effects_exile_from_opponents_libraries() {
    let mut g = pod(3);
    let j = g.add_card_to_battlefield(0, catalog::the_madcap_jester());
    g.battlefield.iter_mut().for_each(|c| c.summoning_sick = false);
    let (t1, t2) = (g.players[1].library[0].id, g.players[2].library[0].id);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: j, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    for t in [t1, t2] {
        let c = g.exile.iter().find(|c| c.id == t).expect("exiled");
        assert!(c.may_play_until.is_some(), "playable by Jester's controller");
    }
    let ordeal = g.add_card_to_hand(0, catalog::bitter_ordeal());
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: ordeal,
        target: Some(Target::Player(2)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("ordeal");
    drain_stack(&mut g);
    let granted = g.exile.iter().filter(|c| c.owner == 2 && c.may_play_until.is_some()).count();
    assert_eq!(granted, 2, "the searched card too");
}

fn treasures(g: &GameState, seat: usize) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == "Treasure").count()
}

/// Chira: Tibalt's +1 (draw, then discard at random) is one random act — a
/// counter and a tapped Treasure; one effect flipping three coins is one more
/// ("one or more coins"). A plain draw is none.
#[test]
fn chira_grows_on_random_acts_once_each() {
    let mut g = pod(2);
    let chira = g.add_card_to_battlefield(0, catalog::chira_all_in());
    let tibalt = g.add_card_to_battlefield(0, catalog::tibalt_the_fiend_blooded());
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: tibalt, ability_index: 0, target: None, x_value: None })
        .expect("+1");
    drain_stack(&mut g);
    assert_eq!(plus_ones(&g, chira), 1, "discarded at random");
    assert_eq!(treasures(&g, 0), 1);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.tapped));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let flips = Effect::FlipCoin { count: Value::Const(3), on_heads: Box::new(Effect::Noop), on_tails: Box::new(Effect::Noop) };
    let ev = g.resolve_effect(&flips, &ctx).expect("flip");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(plus_ones(&g, chira), 2, "three coins in one effect: one trigger");
    let ev = g.resolve_effect(&Effect::Draw { who: Selector::You, amount: Value::ONE }, &ctx).expect("draw");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(plus_ones(&g, chira), 2);
}
