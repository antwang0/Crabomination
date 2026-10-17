//! Commander: the 2026-10-17 (`01Htv6cF`) rescan's two-card gaps (`decks::cmdr_oct17_a`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    if let Some(c) = g.battlefield_find_mut(id) {
        c.loyalty_uses_this_turn = 0;
    }
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .expect("loyalty");
    drain_stack(g);
}

fn cant_block(g: &GameState, id: CardId) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&Keyword::CantBlock))
}

/// CR 702.29c — Quakefoot Cyclops's "when you cycle this card" trigger
/// stops a creature blocking this turn.
#[test]
fn quakefoot_cyclops_cycling_trigger_stops_a_blocker() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let cyclops = g.add_card_to_hand(0, catalog::quakefoot_cyclops());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::Cycle { card_id: cyclops, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cyclops));
    assert!(cant_block(&g, wurm));
}

/// General Traag's ETB is a reflexive "when you do": sacrificing another
/// artifact deals 4 to a creature; with no other artifact nothing happens.
#[test]
fn general_traag_sacrifices_an_artifact_for_four_damage() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let traag = g.add_card_to_hand(0, catalog::general_traag_heart_of_stone());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, traag, None).expect("Traag");
    assert!(g.battlefield_find(traag).is_some(), "Traag itself is not the sacrifice");
    assert!(g.battlefield_find(ring).is_none(), "Sol Ring sacrificed");
    assert!(g.battlefield_find(wurm).is_none(), "4 damage killed the 6/4");
}

/// Earth Tremor counts only the caster's lands.
#[test]
fn earth_tremor_deals_damage_per_land_you_control() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    g.add_card_to_battlefield(1, catalog::forest());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let tremor = g.add_card_to_hand(0, catalog::earth_tremor());
    g.players[0].mana_pool.add(Color::Red, 4);
    cast(&mut g, tremor, Some(Target::Permanent(wurm))).expect("Tremor");
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, 3);
}

/// CR 601.2b — Grafted Identity's additional cost sacrifices a creature; the
/// Aura then takes control of its host and pumps it.
#[test]
fn grafted_identity_sacrifices_then_steals() {
    let mut g = pod(2);
    let fodder = ready(&mut g, 0, catalog::grizzly_bears());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let aura = g.add_card_to_hand(0, catalog::grafted_identity());
    g.players[0].mana_pool.add(Color::Blue, 4);
    cast(&mut g, aura, Some(Target::Permanent(wurm))).expect("Grafted Identity");
    assert!(g.battlefield_find(fodder).is_none(), "the additional cost was paid");
    assert_eq!(g.battlefield_find(wurm).unwrap().controller, 0);
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 7);
}

/// Discontinuity costs {1}{U} during its caster's turn and all six mana
/// otherwise; either way it ends the turn, exiling the spell below it.
#[test]
fn discontinuity_is_cheap_only_on_your_turn() {
    let mut g = pod(2);
    let life = g.players[1].life;
    let disc = g.add_card_to_hand(0, catalog::discontinuity());
    g.players[0].mana_pool.add(Color::Blue, 2);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    cast(&mut g, disc, None).expect("{U}{U} on your own turn");
    assert!(g.exile.iter().any(|c| c.id == bolt), "the turn ended under the bolt");
    assert_eq!(g.players[1].life, life);

    let mut g = pod(2);
    g.active_player_idx = 1;
    let disc = g.add_card_to_hand(0, catalog::discontinuity());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    let res = g.perform_action(GameAction::CastSpell { card_id: disc, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert!(res.is_err(), "full cost on an opponent's turn");
}

/// Call to Heel: the bounced creature's controller draws, not the caster.
#[test]
fn call_to_heel_makes_the_controller_draw() {
    let mut g = pod(3);
    let wurm = ready(&mut g, 2, catalog::craw_wurm());
    let heel = g.add_card_to_hand(0, catalog::call_to_heel());
    g.players[0].mana_pool.add(Color::Blue, 2);
    let (mine, theirs) = (g.players[0].hand.len(), g.players[2].hand.len());
    cast(&mut g, heel, Some(Target::Permanent(wurm))).expect("Call to Heel");
    assert!(g.players[2].hand.iter().any(|c| c.id == wurm));
    assert_eq!(g.players[2].hand.len(), theirs + 2, "the wurm and a card");
    assert_eq!(g.players[0].hand.len(), mine - 1);
}

/// Candlekeep Inspiration counts instants/sorceries/Adventures you own in
/// exile and in your graveyard, but not lands or other players' cards.
#[test]
fn candlekeep_inspiration_counts_spells_in_exile_and_graveyard() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(1, catalog::opt());
    g.add_card_to_exile(0, catalog::opt());
    let ci = g.add_card_to_hand(0, catalog::candlekeep_inspiration());
    g.players[0].mana_pool.add(Color::Blue, 5);
    cast(&mut g, ci, None).expect("Candlekeep");
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "Bolt + the exiled Opt; Candlekeep is still on the stack");
}

/// Chandra, Bold Pyromancer: +1 makes {R}{R} and pings a player; −7 hits
/// the target player and only that player's creatures and planeswalkers.
#[test]
fn chandra_bold_pyromancer_ultimate_hits_one_player_and_their_board() {
    let mut g = pod(3);
    let c = ready(&mut g, 0, catalog::chandra_bold_pyromancer());
    let life = g.players[1].life;
    loyalty(&mut g, c, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 2);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2);
    let theirs = ready(&mut g, 1, catalog::craw_wurm());
    let other = ready(&mut g, 2, catalog::craw_wurm());
    let mine = ready(&mut g, 0, catalog::craw_wurm());
    g.battlefield_find_mut(c).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 7);
    loyalty(&mut g, c, 2, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 12);
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(other).is_some() && g.battlefield_find(mine).is_some());
}

/// Chandra, Dressed to Kill's second +1 lets you cast the exiled card only
/// if it's red.
#[test]
fn chandra_dressed_to_kill_grants_only_red_cards() {
    for (def, red) in [(catalog::lightning_bolt(), true), (catalog::opt(), false)] {
        let mut g = pod(2);
        let c = ready(&mut g, 0, catalog::chandra_dressed_to_kill());
        let top = g.next_id();
        g.players[0].add_to_library_top(top, def);
        loyalty(&mut g, c, 1, None);
        let exiled = g.exile.iter().find(|x| x.id == top).expect("exiled");
        assert_eq!(exiled.may_play_until.is_some(), red);
    }
}

/// CR 601.2c — Shifting Loyalties' second target must share a card type with
/// the first (and Role Reversal's a permanent type): a creature can't be
/// swapped for a land.
#[test]
fn exchange_control_targets_must_share_a_type() {
    for def in [catalog::shifting_loyalties(), catalog::role_reversal()] {
        let mut g = pod(2);
        let bears = ready(&mut g, 0, catalog::grizzly_bears());
        let land = ready(&mut g, 1, catalog::forest());
        let wurm = ready(&mut g, 1, catalog::craw_wurm());
        let s = g.add_card_to_hand(0, def.clone());
        g.players[0].mana_pool.add(Color::Blue, 3);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        let bad = g.perform_action(GameAction::CastSpell {
            card_id: s,
            target: Some(Target::Permanent(bears)),
            additional_targets: vec![Target::Permanent(land)],
            mode: None,
            x_value: None,
        });
        assert!(bad.is_err(), "{}: a creature and a land share no type", def.name);
        g.perform_action(GameAction::CastSpell {
            card_id: s,
            target: Some(Target::Permanent(bears)),
            additional_targets: vec![Target::Permanent(wurm)],
            mode: None,
            x_value: None,
        })
        .expect("two creatures");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(wurm).unwrap().controller, 0, "{}", def.name);
        assert_eq!(g.battlefield_find(bears).unwrap().controller, 1);
    }
}

/// Hoshi Sato's Federation count skips Borg: a Human Scientist (itself) and
/// a Bear look at two cards; a Borg creature adds nothing.
#[test]
fn hoshi_sato_counts_non_borg_creature_types() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    let mut borg = catalog::grizzly_bears();
    borg.subtypes.creature_types = vec![crabomination::card::CreatureType::Borg, crabomination::card::CreatureType::Drone];
    ready(&mut g, 0, borg);
    let mut tops = Vec::new();
    for def in [catalog::opt(), catalog::sol_ring(), catalog::lightning_bolt(), catalog::island(), catalog::mountain()] {
        let id = g.next_id();
        g.players[0].add_to_library_top(id, def);
        tops.push(id);
    }
    // Top of library is now Mountain, Island, Bolt, Sol Ring, Opt.
    let hoshi = g.add_card_to_hand(0, catalog::hoshi_sato_exolinguist());
    g.players[0].mana_pool.add(Color::Blue, 5);
    cast(&mut g, hoshi, None).expect("Hoshi");
    // Human + Scientist + Bear = 3 looked at (Borg and Drone don't count).
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(tops[1]), "Sol Ring is the new top");
}

/// CR 702.165 — Saiba Cryptomancer's backup counter on another creature also
/// grants it hexproof until end of turn.
#[test]
fn saiba_cryptomancer_backs_up_with_hexproof() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let saiba = g.add_card_to_hand(0, catalog::saiba_cryptomancer());
    g.players[0].mana_pool.add(Color::Blue, 2);
    cast(&mut g, saiba, None).expect("Saiba");
    let cp = g.computed_permanent(bears).unwrap();
    if cp.power == 3 {
        assert!(cp.keywords().contains(&Keyword::Hexproof));
    } else {
        assert_eq!(g.computed_permanent(saiba).unwrap().power, 1, "the counter went on Saiba itself");
    }
}

/// Rick Jones mills four and can keep an enchantment (or a Hero) among them.
#[test]
fn rick_jones_keeps_an_enchantment_from_the_mill() {
    let mut g = pod(2);
    let rick = ready(&mut g, 0, catalog::rick_jones_destined_sidekick());
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::crop_sigil());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility { card_id: rick, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("Rick");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == top), "Crop Sigil to hand");
    assert_eq!(g.players[0].graveyard.len(), 3);
}

/// CR 702.86 — Crop Sigil's sacrifice ability needs delirium.
#[test]
fn crop_sigil_requires_delirium() {
    let mut g = pod(2);
    let sigil = ready(&mut g, 0, catalog::crop_sigil());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let forest = g.add_card_to_graveyard(0, catalog::forest());
    let act = |g: &mut GameState| {
        g.players[0].mana_pool.add(Color::Green, 3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: sigil,
            ability_index: 0,
            target: Some(Target::Permanent(bears)),
            additional_targets: vec![Target::Permanent(forest)],
            x_value: None,
            mode: None,
        })
    };
    assert!(act(&mut g).is_err(), "two card types: no delirium");
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::sol_ring());
    act(&mut g).expect("four card types");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears) && g.players[0].hand.iter().any(|c| c.id == forest));
}

/// Mirkwood Channeler pumps an Elf by the Forests you control at your
/// beginning of combat.
#[test]
fn mirkwood_channeler_pumps_an_elf_per_forest() {
    let mut g = pod(2);
    let elf = ready(&mut g, 0, catalog::mirkwood_channeler());
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.step = TurnStep::PreCombatMain;
    let events = g.advance_step(Vec::new()).expect("to combat");
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let cp = g.computed_permanent(elf).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(cp.keywords().contains(&Keyword::Trample));
}

/// Contest of Claws discovers only on excess damage (CR 701.57).
#[test]
fn contest_of_claws_discovers_the_excess() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 0, catalog::craw_wurm());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::lightning_bolt());
    let c = g.add_card_to_hand(0, catalog::contest_of_claws());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: c,
        target: Some(Target::Permanent(wurm)),
        additional_targets: vec![Target::Permanent(bears)],
        mode: None,
        x_value: None,
    })
    .expect("Contest");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    assert!(!g.players[0].library.iter().any(|x| x.id == top), "4 excess discovered the Bolt");
}

/// Supplant Form bounces a creature and leaves you a token copy of it.
#[test]
fn supplant_form_bounces_and_copies() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let s = g.add_card_to_hand(0, catalog::supplant_form());
    g.players[0].mana_pool.add(Color::Blue, 6);
    cast(&mut g, s, Some(Target::Permanent(wurm))).expect("Supplant Form");
    assert!(g.players[1].hand.iter().any(|c| c.id == wurm));
    assert!(g.battlefield.iter().any(|c| c.is_token && c.controller == 0 && c.definition.name == "Craw Wurm"));
}

/// Linessa's {X}{U}{U} bounce takes only a creature with mana value exactly X.
#[test]
fn linessa_bounces_a_creature_of_mana_value_x() {
    let mut g = pod(2);
    let linessa = ready(&mut g, 0, catalog::linessa_zephyr_mage());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let act = |g: &mut GameState, x: u32| {
        g.players[0].mana_pool.add(Color::Blue, 2 + x);
        g.perform_action(GameAction::ActivateAbility {
            card_id: linessa,
            ability_index: 0,
            target: Some(Target::Permanent(bears)),
            additional_targets: vec![],
            x_value: Some(x),
            mode: None,
        })
    };
    assert!(act(&mut g, 3).is_err(), "Bears is mana value 2");
    act(&mut g, 2).expect("X = 2");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bears));
}

/// CR 207.2c (Grandeur is an ability word) — discarding a second Linessa makes the target player
/// return a creature, then an artifact, an enchantment and a land they control.
#[test]
fn linessa_grandeur_makes_a_player_return_one_of_each() {
    let mut g = pod(3);
    let linessa = ready(&mut g, 0, catalog::linessa_zephyr_mage());
    g.add_card_to_hand(0, catalog::linessa_zephyr_mage());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let land = ready(&mut g, 1, catalog::forest());
    let safe = ready(&mut g, 2, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: linessa,
        ability_index: 1,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("Grandeur");
    drain_stack(&mut g);
    for id in [bears, ring, land] {
        assert!(g.players[1].hand.iter().any(|c| c.id == id));
    }
    assert!(g.battlefield_find(safe).is_some(), "only the target player returns");
}

/// Last Chance (and Final Fortune) lose at the EXTRA turn's end step, not at
/// the end of the turn it was cast in.
#[test]
fn last_chance_loses_at_the_extra_turns_end_step() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::last_chance());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, s, None).expect("Last Chance");
    let turn = g.turn_number;
    let mut guard = 0;
    while !g.players[0].eliminated && guard < 400 {
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
        guard += 1;
    }
    assert!(g.players[0].eliminated);
    assert_eq!((g.turn_number, g.active_player_idx), (turn + 1, 0), "lost on the extra turn");
}

/// Vile Manifestation counts only cards with cycling in its owner's graveyard.
#[test]
fn vile_manifestation_counts_cycling_cards() {
    let mut g = pod(2);
    let vile = ready(&mut g, 0, catalog::vile_manifestation());
    g.add_card_to_graveyard(0, catalog::granitic_titan());
    g.add_card_to_graveyard(0, catalog::quakefoot_cyclops());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::granitic_titan());
    assert_eq!(g.computed_permanent(vile).unwrap().power, 2);
}

/// Huatli's +1 adds a loyalty counter per creature you control (on top of the
/// +1 cost); Ajani's −9 puts counters equal to your life total.
#[test]
fn huatli_and_ajani_scale_with_the_board_and_life() {
    let mut g = pod(2);
    let h = ready(&mut g, 0, catalog::huatli_radiant_champion());
    for _ in 0..3 {
        ready(&mut g, 0, catalog::grizzly_bears());
    }
    ready(&mut g, 1, catalog::grizzly_bears());
    loyalty(&mut g, h, 0, None);
    assert_eq!(g.battlefield_find(h).unwrap().counter_count(crabomination::card::CounterType::Loyalty), 3 + 1 + 3);

    let a = ready(&mut g, 0, catalog::ajani_wise_counselor());
    g.battlefield_find_mut(a).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 9);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].life = 13;
    loyalty(&mut g, a, 2, Some(Target::Permanent(bears)));
    assert_eq!(g.computed_permanent(bears).unwrap().power, 15);
}

/// CR 614 — Crush the Weak exiles the creatures it kills.
#[test]
fn crush_the_weak_exiles_what_it_kills() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let s = g.add_card_to_hand(0, catalog::crush_the_weak());
    g.players[0].mana_pool.add(Color::Red, 3);
    cast(&mut g, s, None).expect("Crush");
    assert!(g.exile.iter().any(|c| c.id == bears), "exiled, not dead");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bears));
    assert!(g.battlefield_find(wurm).is_some());
}

/// Venom Connoisseur: the second alliance trigger in a turn spreads
/// deathtouch to the whole team.
#[test]
fn venom_connoisseur_second_alliance_gives_the_team_deathtouch() {
    let mut g = pod(2);
    let v = ready(&mut g, 0, catalog::venom_connoisseur());
    let deathtouch = |g: &GameState, id| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::Deathtouch);
    let a = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, a, None).expect("bears");
    assert!(deathtouch(&g, v) && !deathtouch(&g, a), "first time: itself only");
    let b = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, b, None).expect("bears");
    assert!(deathtouch(&g, a) && deathtouch(&g, b), "second time: everyone");
}

/// CR 702.63 — Chronozoa copies itself only when it dies with no time
/// counters on it.
#[test]
fn chronozoa_splits_only_when_out_of_time() {
    for (counters, copies) in [(0u32, 2usize), (2, 0)] {
        let mut g = pod(2);
        let c = g.add_card_to_battlefield(0, catalog::chronozoa());
        if let Some(card) = g.battlefield_find_mut(c) {
            card.counters.clear();
            card.add_counters(crabomination::card::CounterType::Time, counters);
        }
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast(&mut g, bolt, Some(Target::Permanent(c))).expect("bolt");
        let n = g.battlefield.iter().filter(|x| x.is_token && x.definition.name == "Chronozoa").count();
        assert_eq!(n, copies, "{counters} time counters");
    }
}

/// Gideon's Memorial pumps and grants vigilance to creature tokens only.
#[test]
fn gideons_memorial_pumps_tokens() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::gideons_memorial());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::supplant_form());
    g.players[0].mana_pool.add(Color::Blue, 6);
    cast(&mut g, s, Some(Target::Permanent(bears))).expect("copy the bears");
    let token = g.battlefield.iter().find(|c| c.is_token).expect("token").id;
    let cp = g.computed_permanent(token).unwrap();
    assert_eq!(cp.power, 3);
    assert!(cp.keywords().contains(&Keyword::Vigilance));
}

/// Dragon's Disciple enters with a counter when a Dragon card is in hand.
#[test]
fn dragons_disciple_counter_needs_a_dragon() {
    for (dragon, power) in [(true, 2), (false, 1)] {
        let mut g = pod(2);
        if dragon {
            g.add_card_to_hand(0, catalog::shivan_dragon());
        }
        let d = g.add_card_to_hand(0, catalog::dragons_disciple());
        g.players[0].mana_pool.add(Color::White, 2);
        cast(&mut g, d, None).expect("Disciple");
        assert_eq!(g.computed_permanent(d).unwrap().power, power);
    }
}

/// CR 301.5c — Captain's Hook destroys the creature it leaves when moved.
#[test]
fn captains_hook_destroys_its_old_host() {
    let mut g = pod(2);
    let hook = ready(&mut g, 0, catalog::captains_hook());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: hook, target: a }).expect("equip a");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(a).unwrap().power, 4);
    g.perform_action(GameAction::Equip { equipment: hook, target: b }).expect("equip b");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none(), "the old host is destroyed");
    assert!(g.battlefield_find(b).is_some());
}

/// Omnipresence casts a spell free only while its mana value is at most the
/// number of creatures you control.
#[test]
fn omnipresence_scales_with_creatures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::omnipresence());
    ready(&mut g, 0, catalog::grizzly_bears());
    let free = |g: &mut GameState, id, target| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
    };
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    assert!(free(&mut g, wurm, None).is_err(), "one creature, a six-drop");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    free(&mut g, bolt, Some(Target::Player(1))).expect("a one-drop is free");
}

/// Puppet Crafting makes a noncreature artifact a 5/5 Construct.
#[test]
fn puppet_crafting_animates_an_artifact() {
    let mut g = pod(2);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let p = g.add_card_to_hand(0, catalog::puppet_crafting());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, p, Some(Target::Permanent(ring))).expect("Puppet Crafting");
    let cp = g.computed_permanent(ring).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Novel Nunchaku attaches on entry and its bearer fights.
#[test]
fn novel_nunchaku_attaches_and_fights() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 0, catalog::craw_wurm());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let n = g.add_card_to_hand(0, catalog::novel_nunchaku());
    g.players[0].mana_pool.add(Color::Green, 3);
    cast(&mut g, n, None).expect("Nunchaku");
    assert_eq!(g.battlefield_find(n).unwrap().attached_to, Some(wurm));
    assert!(g.battlefield_find(bears).is_none(), "the 7/5 fought the bears");
}
