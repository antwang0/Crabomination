//! Commander: the Heads I Win, Tails You Lose deck's missing cards
//! (`decks::cmdr_zndrsplt`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::Blue, Color::Red] {
        g.players[0].mana_pool.add(c, 10);
    }
    g.players[0].mana_pool.add_colorless(10);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    flood(g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .unwrap_or_else(|e| panic!("activate: {e:?}"));
    drain_stack(g);
}

fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

fn keywords(g: &GameState, id: CardId) -> Vec<Keyword> {
    g.computed_permanent(id).unwrap().keywords().to_vec()
}

/// Bloodsworn Steward pumps and hastes your commander creatures only.
#[test]
fn bloodsworn_steward_boosts_commanders() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::bloodsworn_steward());
    let okaun = g.add_card_to_battlefield(0, catalog::okaun_eye_of_chaos());
    g.players[0].commanders.push(okaun);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(okaun).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(keywords(&g, okaun).contains(&Keyword::Haste));
    assert_eq!(g.computed_permanent(bears).unwrap().power, 2, "not a commander");
}

/// Boompile wipes nonland permanents on a won flip and does nothing on a
/// lost one.
#[test]
fn boompile_destroys_nonland_permanents_on_a_win() {
    let mut g = main_phase();
    let pile = g.add_card_to_battlefield(0, catalog::boompile());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(1, catalog::island());
    script(&mut g, vec![DecisionAnswer::Bool(false)]);
    activate(&mut g, pile, 0, None);
    assert!(g.battlefield_find(bears).is_some(), "lost the flip");
    g.battlefield_find_mut(pile).unwrap().tapped = false;
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    activate(&mut g, pile, 0, None);
    assert!(g.battlefield_find(bears).is_none() && g.battlefield_find(pile).is_none());
    assert!(g.battlefield_find(land).is_some(), "lands stay");
}

/// Desolate Lighthouse loots.
#[test]
fn desolate_lighthouse_loots() {
    let mut g = main_phase();
    let house = g.add_card_to_battlefield(0, catalog::desolate_lighthouse());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, house, 1, None);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Flamekin Village enters tapped unless an Elemental is revealed, and hastes a
/// creature for {R}.
#[test]
fn flamekin_village_reveal_and_haste() {
    let mut g = main_phase();
    let land = g.add_card_to_hand(0, catalog::flamekin_village());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "no Elemental to reveal");
    g.battlefield_find_mut(land).unwrap().tapped = false;
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, land, 1, Some(Target::Permanent(bears)));
    assert!(keywords(&g, bears).contains(&Keyword::Haste));
}

/// Footfall Crater gives its land "{T}: target creature gains trample and haste".
#[test]
fn footfall_crater_grants_its_land_an_ability() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    let crater = g.add_card_to_hand(0, catalog::footfall_crater());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: crater,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let granted = g.granted_abilities_for(land);
    assert_eq!(granted.len(), 1);
    let index = catalog::mountain().activated_abilities.len();
    activate(&mut g, land, index, Some(Target::Permanent(bears)));
    let kws = keywords(&g, bears);
    assert!(kws.contains(&Keyword::Trample) && kws.contains(&Keyword::Haste));
}

/// Frenetic Sliver gives every Sliver its {0} flip: a win exiles it until the
/// next end step, a loss sacrifices it.
#[test]
fn frenetic_sliver_flickers_or_sacrifices() {
    let mut g = main_phase();
    let frenetic = g.add_card_to_battlefield(0, catalog::frenetic_sliver());
    let other = g.add_card_to_battlefield(1, catalog::frenetic_sliver());
    // "All Slivers", either side — and one instance per Frenetic Sliver.
    assert_eq!(g.granted_abilities_for(other).len(), 2);
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    activate(&mut g, frenetic, 0, None);
    assert!(g.exile.iter().any(|c| c.id == frenetic), "won: exiled");
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(frenetic).is_some(), "back at the end step");

    let mut g = main_phase();
    let frenetic = g.add_card_to_battlefield(0, catalog::frenetic_sliver());
    script(&mut g, vec![DecisionAnswer::Bool(false)]);
    activate(&mut g, frenetic, 0, None);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == frenetic), "lost: sacrificed");
}

/// Daretti: +2 loots up to two, −2 trades an artifact for one in the yard, and
/// the −10 emblem brings back an artifact that went to the graveyard at the
/// next end step.
#[test]
fn daretti_scrap_savant_abilities() {
    let loyalty = |g: &mut GameState, id, index, target| {
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: id,
            ability_index: index,
            target,
            x_value: None,
        })
        .expect("loyalty");
        drain_stack(g);
    };
    let mut g = main_phase();
    let daretti = g.add_card_to_battlefield(0, catalog::daretti_scrap_savant());
    let fodder = g.add_card_to_battlefield(0, catalog::boompile());
    let back = g.add_card_to_graveyard(0, catalog::sol_ring());
    loyalty(&mut g, daretti, 1, Some(Target::Permanent(back)));
    assert!(g.battlefield_find(back).is_some(), "Sol Ring returned");
    assert!(g.battlefield_find(fodder).is_none(), "Boompile was the cost");

    let mut g = main_phase();
    let daretti = g.add_card_to_battlefield(0, catalog::daretti_scrap_savant());
    g.battlefield_find_mut(daretti)
        .unwrap()
        .add_counters(crabomination::card::CounterType::Loyalty, 10);
    loyalty(&mut g, daretti, 2, None);
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let murder_like = crabomination::effect::Effect::Destroy {
        what: crabomination::effect::Selector::EachPermanent(
            crabomination::card::SelectionRequirement::Artifact,
        ),
    };
    let ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&murder_like, &ctx).expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == ring));
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_some(), "the emblem returned it");
}

// ── Goblin Storm (`decks::cmdr_zada`) ─────────────────────────────────────────

fn cast(g: &mut GameState, card_id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell {
        card_id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .unwrap_or_else(|e| panic!("cast: {e:?}"));
    drain_stack(g);
}

/// Battle Hymn adds {R} per creature you control.
#[test]
fn battle_hymn_counts_your_creatures() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hymn = g.add_card_to_hand(0, catalog::battle_hymn());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, hymn, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
}

/// CR 702.142 — Broadside Bombardiers boasts only after attacking: 2 plus the
/// sacrificed permanent's mana value to any target.
#[test]
fn broadside_bombardiers_boasts_for_two_plus_mana_value() {
    let mut g = main_phase();
    let bomb = g.add_card_to_battlefield(0, catalog::broadside_bombardiers());
    let signet = g.add_card_to_battlefield(0, catalog::arcane_signet());
    let boast = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: bomb,
            ability_index: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(boast(&mut g).is_err(), "hasn't attacked");
    g.battlefield_find_mut(bomb).unwrap().attacked_this_turn = true;
    let life = g.players[1].life;
    boast(&mut g).expect("boast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(signet).is_none());
    assert_eq!(g.players[1].life, life - 4, "2 + Arcane Signet's 2");
}

/// Castle Embereth enters tapped without a Mountain (a CR 614.12 replacement)
/// and pumps your team.
#[test]
fn castle_embereth_needs_a_mountain_and_pumps() {
    let mut g = main_phase();
    let castle = g.add_card_to_hand(0, catalog::castle_embereth());
    g.perform_action(GameAction::PlayLand(castle)).expect("play");
    assert!(g.battlefield_find(castle).unwrap().tapped);

    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mountain());
    let castle = g.add_card_to_hand(0, catalog::castle_embereth());
    g.perform_action(GameAction::PlayLand(castle)).expect("play");
    assert!(!g.battlefield_find(castle).unwrap().tapped, "a Mountain: untapped");
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, castle, 1, None);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 3);
}

/// Frontline Heroism makes a hasty Soldier and aims a copy of a
/// single-creature spell at it.
#[test]
fn frontline_heroism_copies_onto_a_new_soldier() {
    let mut g = main_phase();
    let heroism = g.add_card_to_hand(0, catalog::frontline_heroism());
    flood(&mut g);
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, heroism, None);
    let soldiers = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.definition.name == "Soldier").map(|c| c.id).collect::<Vec<_>>()
    };
    assert_eq!(soldiers(&g).len(), 1);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    cast(&mut g, growth, Some(Target::Permanent(bears)));
    assert_eq!(g.computed_permanent(bears).unwrap().power, 5);
    let fresh = *soldiers(&g).last().unwrap();
    assert_eq!(soldiers(&g).len(), 2);
    assert_eq!(g.computed_permanent(fresh).unwrap().power, 4, "the copy hit the new Soldier");
}

/// General Kreat pings each opponent when another creature enters.
#[test]
fn general_kreat_pings_on_entry() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::general_kreat_the_boltbringer());
    let life = g.players[1].life;
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g);
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bears, None);
    assert_eq!(g.players[1].life, life - 1);
}

/// Kher Keep makes the named Kobold token.
#[test]
fn kher_keep_makes_kobolds() {
    let mut g = main_phase();
    let keep = g.add_card_to_battlefield(0, catalog::kher_keep());
    activate(&mut g, keep, 1, None);
    let kobold = g.battlefield.iter().find(|c| c.is_token).expect("a token");
    assert_eq!(kobold.definition.name, "Kobolds of Kher Keep");
    assert_eq!((kobold.definition.power, kobold.definition.toughness), (0, 1));
}

/// Rundvelt Hordemaster pumps the other Goblins, and a Goblin's death exiles
/// the top card — castable when it's a Goblin creature.
#[test]
fn rundvelt_hordemaster_lords_and_digs_on_goblin_death() {
    let mut g = main_phase();
    let master = g.add_card_to_battlefield(0, catalog::rundvelt_hordemaster());
    let kreat = g.add_card_to_battlefield(0, catalog::general_kreat_the_boltbringer());
    assert_eq!(g.computed_permanent(kreat).unwrap().power, 3);
    assert_eq!(g.computed_permanent(master).unwrap().power, 1, "other Goblins only");
    let top = g.add_card_to_library(0, catalog::broadside_bombardiers());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    cast(&mut g, bolt, Some(Target::Permanent(kreat)));
    let exiled = g.exile.iter().find(|c| c.id == top).expect("exiled the top card");
    assert!(exiled.may_play_until.is_some(), "a Goblin creature: castable");
}

/// Siege-Gang Lieutenant's lieutenant trigger needs your commander.
#[test]
fn siege_gang_lieutenant_needs_the_commander() {
    let goblins = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count();
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::siege_gang_lieutenant());
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(goblins(&g), 0, "no commander");
    let zada = g.add_card_to_battlefield(0, catalog::zada_hedron_grinder());
    g.players[0].commanders.push(zada);
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(goblins(&g), 2);
    let tok = g.battlefield.iter().find(|c| c.definition.name == "Goblin").unwrap().id;
    assert!(keywords(&g, tok).contains(&Keyword::Haste));
}

/// Throne of Eldraine's four mana pay only for a monocolored spell of the
/// chosen color.
#[test]
fn throne_of_eldraine_mana_is_chosen_color_monocolored_only() {
    let mut g = main_phase();
    let throne = g.add_card_to_battlefield(0, catalog::throne_of_eldraine());
    g.battlefield_find_mut(throne).unwrap().chosen_color = Some(Color::Red);
    g.perform_action(GameAction::ActivateAbility {
        card_id: throne,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for four");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 4);
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: ring,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a colorless spell can't use it"
    );
    let bombardiers = g.add_card_to_hand(0, catalog::broadside_bombardiers());
    cast(&mut g, bombardiers, None);
    assert!(g.battlefield_find(bombardiers).is_some(), "a mono-red spell can");
}
