//! Commander: the Paradox Power precon (WHO, The Thirteenth Doctor + Yasmin
//! Khan, `decks::cmdr_thirteenth`), and its primitives: spells cast from
//! anywhere other than your hand (Paradox), drawing from the bottom, a
//! turn-gated graveyard flashback grant, two-type spend restriction, exiling
//! a permanent foretold (CR 702.143) and a granted demonstrate (CR 702.150).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.turn_number = 3;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..12 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn flashback(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastFlashback { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

/// CR 702.124m — The Thirteenth Doctor pairs with a Doctor's companion.
#[test]
fn cr_702_124m_the_doctor_and_yasmin_pair() {
    use crabomination::format::commanders_may_pair;
    assert!(commanders_may_pair(&catalog::the_thirteenth_doctor(), &catalog::yasmin_khan()));
    assert!(!commanders_may_pair(&catalog::yasmin_khan(), &catalog::dan_lewis()), "two companions");
}

/// Paradox + Return the Past — a spell flashed back from the graveyard on
/// your turn triggers The Thirteenth Doctor; on another turn there is no
/// flashback to use.
#[test]
fn return_the_past_grants_flashback_on_your_turn_only() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::return_the_past());
    let doc = g.add_card_to_battlefield(0, catalog::the_thirteenth_doctor());
    let loop_ = g.add_card_to_graveyard(0, catalog::flatline());
    flood(&mut g, 0);
    flashback(&mut g, 0, loop_, None).expect("flashback on your turn");
    assert_eq!(counters(&g, doc), 1, "paradox: a counter on the only creature");

    let other = g.add_card_to_graveyard(0, catalog::flatline());
    g.active_player_idx = 1;
    assert!(flashback(&mut g, 0, other, None).is_err(), "not during an opponent's turn");
}

/// Impending Flux — X is 1 plus the spells cast not from hand this turn.
#[test]
fn impending_flux_counts_paradox_spells() {
    let mut g = pod(2);
    g.players[0].spells_cast_this_turn = 3;
    g.players[0].spells_cast_this_game_turn = 3;
    g.players[0].spells_cast_from_hand_this_turn = 1;
    flood(&mut g, 0);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flux = g.add_card_to_hand(0, catalog::impending_flux());
    cast(&mut g, 0, flux, None).expect("Impending Flux");
    // Cast from hand: now 4 cast, 2 from hand → X = 1 + 2 = 3.
    assert_eq!(g.players[1].life, 17);
    assert!(g.battlefield_find(bear).is_none());
}

/// River Song — you draw from the bottom of your library.
#[test]
fn river_song_draws_from_the_bottom() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::river_song());
    let bottom = g.add_card_to_library(0, catalog::grizzly_bears());
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(bottom), "added at the bottom");
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::Draw { who: Selector::You, amount: crabomination::card::Value::ONE }, &ctx).expect("draw");
    assert!(g.players[0].hand.iter().any(|c| c.id == bottom));
}

/// Gallifrey Council Chamber — its colored mana pays for a Time Lord, not a
/// Bear.
#[test]
fn gallifrey_mana_is_for_time_lords_and_aliens() {
    let mut g = pod(2);
    let gcc = g.add_card_to_battlefield(0, catalog::gallifrey_council_chamber());
    activate(&mut g, 0, gcc, 1).expect("any color");
    g.players[0].mana_pool.add(Color::Green, 1);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(cast(&mut g, 0, bear, None).is_err(), "not for a Bear");
    g.players[0].mana_pool.add(Color::Blue, 1);
    let doc = g.add_card_to_hand(0, catalog::the_thirteenth_doctor());
    cast(&mut g, 0, doc, None).expect("The Thirteenth Doctor on Gallifrey mana");
}

/// CR 702.143 — The Foretold Soldier dealing damage is exiled foretold, and
/// can be cast for its foretell cost on a later turn.
#[test]
fn cr_702_143_foretold_soldier_exiles_itself_foretold() {
    let mut g = pod(2);
    let fs = g.add_card_to_battlefield(0, catalog::the_foretold_soldier());
    let ctx = EffectContext::for_ability(fs, 0, None);
    let evs = g
        .resolve_effect(
            &Effect::DealDamage {
                to: Selector::Player(crabomination::effect::PlayerRef::Seat(1)),
                amount: crabomination::card::Value::Const(6),
            },
            &ctx,
        )
        .expect("damage");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let ex = g.exile.iter().find(|c| c.id == fs).expect("exiled");
    assert!(ex.face_down, "foretold");
}

/// Clara Oswald — a Doctor's paradox trigger triggers an additional time.
#[test]
fn clara_doubles_the_doctors_triggers() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::return_the_past());
    g.add_card_to_battlefield(0, catalog::clara_oswald());
    let doc = g.add_card_to_battlefield(0, catalog::the_thirteenth_doctor());
    let fl = g.add_card_to_graveyard(0, catalog::flatline());
    flood(&mut g, 0);
    flashback(&mut g, 0, fl, None).expect("flashback");
    let total: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(total, 2, "the Doctor's paradox fired twice");
    let _ = doc;
}

/// Sisterhood of Karn — paradox doubles its counters.
#[test]
fn sisterhood_of_karn_doubles() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::return_the_past());
    flood(&mut g, 0);
    let sk = g.add_card_to_hand(0, catalog::sisterhood_of_karn());
    cast(&mut g, 0, sk, None).expect("Sisterhood");
    assert_eq!(counters(&g, sk), 1);
    let fl = g.add_card_to_graveyard(0, catalog::flatline());
    flashback(&mut g, 0, fl, None).expect("flashback");
    assert_eq!(counters(&g, sk), 2);
}

/// CR 702.150 — The Twelfth Doctor: the first not-from-hand spell is
/// demonstrated, and each copy of yours grows it.
#[test]
fn cr_702_150_twelfth_doctor_demonstrates() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::return_the_past());
    let twelve = g.add_card_to_battlefield(0, catalog::the_twelfth_doctor());
    let sob = g.add_card_to_graveyard(0, catalog::surge_of_brilliance());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    flashback(&mut g, 0, sob, None).expect("flashback Surge");
    assert!(counters(&g, twelve) >= 1, "your copy grew it");
    assert!(g.players[0].hand.len() >= hand + 2, "the original and your copy each drew");
}

/// Twice Upon a Time — can't be cast with fewer than two Doctors.
#[test]
fn twice_upon_a_time_needs_two_doctors() {
    let mut g = pod(2);
    flood(&mut g, 0);
    g.add_card_to_battlefield(0, catalog::the_thirteenth_doctor());
    let tut = g.add_card_to_hand(0, catalog::twice_upon_a_time());
    assert!(cast(&mut g, 0, tut, None).is_err(), "one Doctor");
    g.add_card_to_battlefield(0, catalog::the_twelfth_doctor());
    cast(&mut g, 0, tut, None).expect("two Doctors");
    assert!(g.exile.iter().any(|c| c.id == tut), "exiled as it resolves");
}

/// Heaven Sent III — with every opponent above 0, it exiles itself and may be
/// cast again this turn.
#[test]
fn heaven_sent_recurs_itself() {
    let mut g = pod(2);
    let hs = g.add_card_to_battlefield(0, catalog::heaven_sent());
    g.battlefield_find_mut(hs).unwrap().add_counters(CounterType::Lore, 2);
    let ctx = EffectContext::for_ability(hs, 0, None);
    let chapter = catalog::heaven_sent().saga_chapters[2].1.clone();
    g.resolve_effect(&chapter, &ctx).expect("III");
    assert_eq!(g.players[1].life, 19);
    let ex = g.exile.iter().find(|c| c.id == hs).expect("exiled");
    assert!(ex.may_play_until.is_some(), "and playable this turn");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: hs,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from exile this turn, paying {U}{R}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hs).is_some());
    assert_eq!(g.players[0].mana_pool.total(), 0, "its own cost was paid");
}

/// Truth or Consequences chooses ONE opponent at random for every
/// consequences vote's 3 damage.
#[test]
fn truth_or_consequences_hits_one_random_opponent() {
    for seed in 0..6u64 {
        let mut g = pod(4);
        g.rng = crabomination::game::rng::GameRng::seeded(seed);
        let before: Vec<i32> = g.players.iter().map(|p| p.life).collect();
        let spell = g.add_card_to_hand(0, catalog::truth_or_consequences());
        flood(&mut g, 0);
        cast(&mut g, 0, spell, None).expect("cast");
        let hit: Vec<usize> = (1..4).filter(|&p| g.players[p].life < before[p]).collect();
        assert!(hit.len() <= 1, "seed {seed}: {hit:?}");
        for p in hit {
            assert_eq!((before[p] - g.players[p].life) % 3, 0);
        }
    }
}

/// The Fugitive Doctor's granted flashback costs {2}{R}{G}, not the card's
/// own mana cost.
#[test]
fn the_fugitive_doctor_grants_flashback_for_2rg() {
    let mut g = pod(2);
    let doc = g.add_card_to_battlefield(0, catalog::the_fugitive_doctor());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let Effect::MaySacrifice { then, .. } = g.battlefield_find(doc).unwrap().definition.triggered_abilities[1].effect.clone()
    else {
        panic!("attack trigger")
    };
    let ctx = EffectContext::for_trigger(doc, 0, Some(Target::Permanent(bolt)), 0);
    g.resolve_effect(&then, &ctx).expect("grant");
    flood(&mut g, 0);
    let (total, green) = (g.players[0].mana_pool.total(), g.players[0].mana_pool.amount(Color::Green));
    flashback(&mut g, 0, bolt, Some(Target::Player(1))).expect("flashback");
    assert_eq!(total - g.players[0].mana_pool.total(), 4, "{{2}}{{R}}{{G}}");
    assert_eq!(green - g.players[0].mana_pool.amount(Color::Green), 1);
}

/// Strax's Grenades: the player is chosen at random, then a reflexive
/// trigger TARGETS another creature that player controls — with nothing of
/// the chosen player's to target, nothing fights.
#[test]
fn strax_grenades_target_the_random_players_creature() {
    let mut killed = 0;
    for seed in 0..10u64 {
        let mut g = pod(2);
        g.rng = crabomination::game::rng::GameRng::seeded(seed);
        let strax = g.add_card_to_battlefield(0, catalog::strax_sontaran_nurse());
        g.clear_sickness(strax);
        g.add_card_to_battlefield(0, catalog::sol_ring());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        flood(&mut g, 0);
        activate(&mut g, 0, strax, 0).expect("grenades");
        if g.battlefield_find(bear).is_none() {
            killed += 1;
            assert_eq!(counters(&g, strax), 1, "Glory of Battle");
        } else {
            assert_eq!(counters(&g, strax), 0, "no creature of seat 0's to fight");
        }
    }
    assert!(killed > 0 && killed < 10, "the random pick lands on both seats ({killed}/10)");
}

/// Ryan Sinclair stops at the FIRST nonland card: a Hill Giant above his
/// power ends the reveal (no cast) and a Grizzly Bears under it isn't reached.
#[test]
fn ryan_sinclair_stops_at_the_first_nonland_card() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = pod(2);
    let ryan = g.add_card_to_battlefield(0, catalog::ryan_sinclair());
    g.clear_sickness(ryan);
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let giant = g.add_card_to_library(0, catalog::hill_giant());
    let island = g.add_card_to_library(0, catalog::island());
    for id in [bears, giant, island] {
        let i = g.players[0].library.iter().position(|c| c.id == id).unwrap();
        let c = g.players[0].library.remove(i);
        g.players[0].library.insert(0, c);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: ryan, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none() && g.battlefield_find(bears).is_none());
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bears), "the Bears was never reached");
    assert!(g.players[0].library.iter().rev().take(2).any(|c| c.id == giant), "the Giant went to the bottom");
}

/// CR 509.1b — Become the Pilot's creature can't be blocked unless it's
/// attacking its owner: a third player can't block it, its owner can.
#[test]
fn become_the_pilot_can_be_blocked_only_by_its_owner() {
    use crabomination::game::types::{Attack, AttackTarget};
    for (defender, blockable) in [(2usize, false), (1, true)] {
        let mut g = pod(3);
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().controller = 0;
        g.clear_sickness(bear);
        let pilot = g.add_card_to_battlefield(0, catalog::become_the_pilot());
        g.battlefield_find_mut(pilot).unwrap().attached_to = Some(bear);
        let wall = g.add_card_to_battlefield(defender, catalog::hill_giant());
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(defender) }]))
            .expect("attack");
        drain_stack(&mut g);
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = defender;
        let r = g.perform_action(GameAction::DeclareBlockers(vec![(wall, bear)]));
        assert_eq!(r.is_ok(), blockable, "defender {defender}");
    }
}

/// CR 605.1a — Bigger on the Inside's granted ability targets a player, so
/// it isn't a mana ability: that player adds the mana and their next spell
/// gets cascade.
#[test]
fn cr_605_1a_bigger_on_the_inside_gives_target_player_the_mana() {
    let mut g = pod(3);
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let aura = g.add_card_to_battlefield(0, catalog::bigger_on_the_inside());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(land);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    assert_eq!(g.players[1].mana_pool.total(), 0, "the ability uses the stack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].mana_pool.total(), 2);
    assert_eq!(g.players[0].mana_pool.total(), 0);
    assert!(g.delayed_triggers.iter().any(|d| d.controller == 1));
}

fn escape(g: &mut GameState, id: CardId, exile_cards: Vec<CardId>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastEscape {
        card_id: id,
        exile_cards,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// CR 702.138 — Lunar Hatchling's escape also exiles a land you control: the
/// named land when one is given, the cheapest otherwise, and none means no
/// cast.
#[test]
fn cr_702_138_lunar_hatchling_escape_exiles_a_land() {
    for named in [false, true] {
        let mut g = pod(2);
        let hatchling = g.add_card_to_graveyard(0, catalog::lunar_hatchling());
        let fodder: Vec<CardId> = (0..5).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        flood(&mut g, 0);
        let mut picks = fodder.clone();
        if named {
            picks.push(land);
        }
        escape(&mut g, hatchling, picks).expect("escape");
        assert!(g.battlefield_find(hatchling).is_some());
        assert!(g.battlefield_find(land).is_none(), "the land is exiled");
        assert!(g.exile.iter().any(|c| c.id == land));
        assert!(fodder.iter().all(|id| g.exile.iter().any(|c| c.id == *id)));
    }
    // No land you control: the cost can't be paid.
    let mut g = pod(2);
    let hatchling = g.add_card_to_graveyard(0, catalog::lunar_hatchling());
    let fodder: Vec<CardId> = (0..5).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    let theirs = g.add_card_to_battlefield(1, catalog::forest());
    flood(&mut g, 0);
    let mut picks = fodder.clone();
    picks.push(theirs);
    assert!(escape(&mut g, hatchling, picks).is_err(), "an opponent's land isn't yours");
    assert!(escape(&mut g, hatchling, fodder).is_err(), "no land to exile");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == hatchling));
}

/// The bot escapes Lunar Hatchling, paying the land half.
#[test]
fn lunar_hatchling_bot_escapes() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = pod(2);
    g.step = TurnStep::PostCombatMain;
    let hatchling = g.add_card_to_graveyard(0, catalog::lunar_hatchling());
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::island());
    }
    let mut bot = HeuristicBot::new();
    for _ in 0..20 {
        g.priority.player_with_priority = 0;
        let Some(action) = bot.next_action(&g, 0) else { break };
        let pass = matches!(action, GameAction::PassPriority);
        let _ = g.perform_action(action);
        drain_stack(&mut g);
        if g.battlefield_find(hatchling).is_some() || pass {
            break;
        }
    }
    assert!(g.battlefield_find(hatchling).is_some(), "the bot escaped it");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count(), 7);
}

/// CR 122.2 (and Me, the Immortal's exception) — its counters stay with it
/// into the graveyard and back onto the battlefield when it's cast from
/// there by discarding two cards.
#[test]
fn me_the_immortal_keeps_its_counters_across_zones() {
    let mut g = pod(2);
    let me = g.add_card_to_battlefield(0, catalog::me_the_immortal());
    g.battlefield_find_mut(me).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let blade = g.add_card_to_hand(0, catalog::doom_blade());
    flood(&mut g, 0);
    cast(&mut g, 0, blade, Some(Target::Permanent(me))).expect("doom blade");
    let in_gy = g.players[0].graveyard.iter().find(|c| c.id == me).expect("in the graveyard");
    assert_eq!(in_gy.counter_count(CounterType::PlusOnePlusOne), 2);
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    flood(&mut g, 0);
    flashback(&mut g, 0, me, None).expect("cast from the graveyard");
    assert_eq!(counters(&g, me), 2, "it comes back with them");
    assert!(g.players[0].hand.is_empty(), "two cards discarded");
}
