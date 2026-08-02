//! Invasion (INV) gap wave 3 — the Apprentice/Master cycles, the Auras, the
//! legends and the utility shell.

use crabomination::card::{Keyword, LandType, Supertype};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
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

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Untapped, unsick copy of `f` on `seat`'s battlefield.
fn ready(g: &mut GameState, seat: usize, f: fn() -> crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, f());
    g.clear_sickness(id);
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    id
}

// ── The Apprentice / Master cycles ──────────────────────────────────────────

/// Each Apprentice/Master carries exactly its two printed off-colour taps.
#[test]
fn inv_apprentices_and_masters_have_two_activated_abilities() {
    for f in [
        catalog::sunscape_apprentice as fn() -> _,
        catalog::sunscape_master,
        catalog::nightscape_apprentice,
        catalog::nightscape_master,
        catalog::thornscape_apprentice,
        catalog::thornscape_master,
        catalog::thunderscape_apprentice,
        catalog::thunderscape_master,
        catalog::stormscape_master,
    ] {
        let def = f();
        assert_eq!(def.activated_abilities.len(), 2, "{} ability count", def.name);
        assert!(
            def.activated_abilities.iter().all(|a| a.tap_cost),
            "{} — both halves cost {{T}}",
            def.name
        );
    }
}

/// Thunderscape Master's {G}{G} half pumps the whole team.
#[test]
fn thunderscape_master_pumps_the_team() {
    let mut g = main_phase();
    let master = ready(&mut g, 0, catalog::thunderscape_master);
    let friend = ready(&mut g, 0, catalog::noble_panther);
    activate(&mut g, 0, master, 1, None);
    let cp = g.computed_permanent(friend).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Nightscape Master's {R}{R} half is a 2-damage ping.
#[test]
fn nightscape_master_pings_for_two() {
    let mut g = main_phase();
    let master = ready(&mut g, 0, catalog::nightscape_master);
    let victim = ready(&mut g, 1, catalog::noble_panther);
    activate(&mut g, 0, master, 1, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(victim).unwrap().damage, 2);
}

/// Sunscape Apprentice's {U} half tucks your own creature on top.
#[test]
fn sunscape_apprentice_tucks_your_creature() {
    let mut g = main_phase();
    let apprentice = ready(&mut g, 0, catalog::sunscape_apprentice);
    let friend = ready(&mut g, 0, catalog::noble_panther);
    activate(&mut g, 0, apprentice, 1, Some(Target::Permanent(friend)));
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(friend));
}

// ── Colour-restricted weavers ───────────────────────────────────────────────

/// Each weaver only reaches its two printed colours.
#[test]
fn inv_weavers_respect_their_color_gate() {
    // Might Weaver grants trample to a red or white creature — not a blue one.
    let mut g = main_phase();
    let weaver = ready(&mut g, 0, catalog::might_weaver);
    let red = ready(&mut g, 0, catalog::ruby_leech);
    let blue = ready(&mut g, 0, catalog::sapphire_leech);
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: weaver,
            ability_index: 0,
            target: Some(Target::Permanent(blue)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a blue creature isn't a legal target"
    );
    activate(&mut g, 0, weaver, 0, Some(Target::Permanent(red)));
    assert!(g.computed_permanent(red).unwrap().keywords.contains(&Keyword::Trample));
}

// ── Statics gated on the opposing board ─────────────────────────────────────

/// Skittish Kavu and Kavu Runner both switch off once a white or blue creature
/// shows up across the table.
#[test]
fn inv_kavu_shrink_when_the_opponent_goes_white_or_blue() {
    let mut g = main_phase();
    let skittish = g.add_card_to_battlefield(0, catalog::skittish_kavu());
    let runner = g.add_card_to_battlefield(0, catalog::kavu_runner());
    let cp = g.computed_permanent(skittish).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(g.computed_permanent(runner).unwrap().keywords.contains(&Keyword::Haste));

    g.add_card_to_battlefield(1, catalog::sapphire_leech());
    let cp = g.computed_permanent(skittish).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(!g.computed_permanent(runner).unwrap().keywords.contains(&Keyword::Haste));
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Wings of Hope is +1/+3 and flying on its host.
#[test]
fn wings_of_hope_grants_flight_and_toughness() {
    let mut g = main_phase();
    let host = ready(&mut g, 0, catalog::noble_panther);
    let aura = g.add_card_to_hand(0, catalog::wings_of_hope());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    let cp = g.computed_permanent(host).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 6));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Whip Silk grants reach and rebuys itself for {G}.
#[test]
fn whip_silk_grants_reach_and_returns_itself() {
    let mut g = main_phase();
    let host = ready(&mut g, 0, catalog::noble_panther);
    let aura = g.add_card_to_hand(0, catalog::whip_silk());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    assert!(g.computed_permanent(host).unwrap().keywords.contains(&Keyword::Reach));
    activate(&mut g, 0, aura, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "the Aura went home");
    assert!(!g.computed_permanent(host).unwrap().keywords.contains(&Keyword::Reach));
}

/// Tainted Well cantrips and turns its host land into a Swamp.
#[test]
fn tainted_well_makes_a_swamp_and_cantrips() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::mountain());
    g.add_card_to_library(0, catalog::opt());
    let aura = g.add_card_to_hand(0, catalog::tainted_well());
    let before = g.players[0].hand.len();
    cast(&mut g, 0, aura, Some(Target::Permanent(land)));
    let cp = g.computed_permanent(land).unwrap();
    assert!(cp.subtypes.land_types.contains(&LandType::Swamp));
    assert!(!cp.subtypes.land_types.contains(&LandType::Mountain), "the type is replaced");
    assert_eq!(g.players[0].hand.len(), before, "aura left hand, ETB drew one back");
}

// ── Legends ─────────────────────────────────────────────────────────────────

/// Captain Sisay's tutor only offers legendary cards.
#[test]
fn captain_sisay_tutors_a_legend() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    let sisay = ready(&mut g, 0, catalog::captain_sisay);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let legend = g.add_card_to_library(0, catalog::tsabo_tavoc());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(legend))]));
    activate(&mut g, 0, sisay, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == legend), "the legend came to hand");
    assert!(
        g.players[0]
            .hand
            .iter()
            .all(|c| c.definition.supertypes.contains(&Supertype::Legendary)),
        "a nonlegendary card was never eligible"
    );
}

/// Tsabo Tavoc destroys another legend and is protected from them.
#[test]
fn tsabo_tavoc_hunts_legends() {
    let mut g = main_phase();
    let tsabo = ready(&mut g, 0, catalog::tsabo_tavoc);
    let victim = g.add_card_to_battlefield(1, catalog::captain_sisay());
    let plain = g.add_card_to_battlefield(1, catalog::noble_panther());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: tsabo,
            ability_index: 0,
            target: Some(Target::Permanent(plain)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a nonlegendary creature isn't a legal target"
    );
    activate(&mut g, 0, tsabo, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield.iter().all(|c| c.id != victim));
}

/// Empress Galina steals a legendary permanent outright.
#[test]
fn empress_galina_steals_a_legend() {
    let mut g = main_phase();
    let galina = ready(&mut g, 0, catalog::empress_galina);
    let prize = g.add_card_to_battlefield(1, catalog::captain_sisay());
    activate(&mut g, 0, galina, 0, Some(Target::Permanent(prize)));
    assert_eq!(g.battlefield_find(prize).unwrap().controller, 0);
}

// ── Utility ─────────────────────────────────────────────────────────────────

/// Tsabo's Web cantrips and keeps utility lands tapped through untap.
#[test]
fn tsabos_web_locks_utility_lands() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::tsabos_web());
    let utility = g.add_card_to_battlefield(0, catalog::keldon_necropolis());
    let basic = g.add_card_to_battlefield(0, catalog::mountain());
    for id in [utility, basic] {
        g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = true;
    }
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(utility).unwrap().tapped, "Necropolis stayed tapped");
    assert!(!g.battlefield_find(basic).unwrap().tapped, "a plain Mountain untapped");
}

/// Winnow only reaches a permanent whose name is duplicated on the battlefield.
#[test]
fn winnow_needs_a_duplicate_name() {
    let mut g = main_phase();
    let lone = g.add_card_to_battlefield(1, catalog::noble_panther());
    let spell = g.add_card_to_hand(0, catalog::winnow());
    g.add_card_to_library(0, catalog::opt());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(lone)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a unique name isn't a legal target"
    );
    let twin = g.add_card_to_battlefield(1, catalog::noble_panther());
    cast(&mut g, 0, spell, Some(Target::Permanent(lone)));
    assert!(g.battlefield.iter().all(|c| c.id != lone));
    assert!(g.battlefield.iter().any(|c| c.id == twin), "only the target dies");
}

/// Shivan Harvest eats a creature to break a nonbasic land.
#[test]
fn shivan_harvest_eats_a_creature_for_a_nonbasic() {
    let mut g = main_phase();
    let harvest = g.add_card_to_battlefield(0, catalog::shivan_harvest());
    let fodder = ready(&mut g, 0, catalog::noble_panther);
    let nonbasic = g.add_card_to_battlefield(1, catalog::salt_marsh());
    activate(&mut g, 0, harvest, 0, Some(Target::Permanent(nonbasic)));
    assert!(g.battlefield.iter().all(|c| c.id != nonbasic));
    assert!(g.battlefield.iter().all(|c| c.id != fodder), "the creature was the cost");
}

/// Obliterate can't be countered and clears every artifact, creature and land.
#[test]
fn obliterate_wipes_everything_permanent() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::noble_panther());
    g.add_card_to_battlefield(1, catalog::mountain());
    g.add_card_to_battlefield(1, catalog::tsabos_web());
    let survivor = g.add_card_to_battlefield(1, catalog::dueling_grounds());
    let spell = g.add_card_to_hand(0, catalog::obliterate());
    assert!(catalog::obliterate().keywords.contains(&Keyword::CantBeCountered));
    cast(&mut g, 0, spell, None);
    assert_eq!(g.battlefield.len(), 1, "only the enchantment survives");
    assert_eq!(g.battlefield[0].id, survivor);
}

/// Twilight's Call returns every graveyard creature to its owner's battlefield.
#[test]
fn twilights_call_reanimates_for_everyone() {
    let mut g = main_phase();
    for seat in 0..2 {
        g.players[seat].graveyard.push(crabomination::card::CardInstance::new(
            crabomination::card::CardId(8000 + seat as u32),
            catalog::noble_panther(),
            seat,
        ));
    }
    let spell = g.add_card_to_hand(0, catalog::twilights_call());
    cast(&mut g, 0, spell, None);
    for seat in 0..2 {
        assert!(
            g.battlefield.iter().any(|c| c.controller == seat && c.definition.name == "Noble Panther"),
            "seat {seat} got its creature back"
        );
    }
}

/// Plague Spitter pings the whole table on upkeep and again when it dies.
#[test]
fn plague_spitter_pings_on_upkeep_and_death() {
    let mut g = main_phase();
    let spitter = g.add_card_to_battlefield(0, catalog::plague_spitter());
    let other = g.add_card_to_battlefield(1, catalog::noble_panther());
    let life = [g.players[0].life, g.players[1].life];
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life[0] - 1);
    assert_eq!(g.players[1].life, life[1] - 1);
    assert_eq!(g.battlefield_find(other).unwrap().damage, 1);
    assert_eq!(g.battlefield_find(spitter).unwrap().damage, 1);
}

/// Reckless Assault turns life into reach.
#[test]
fn reckless_assault_pays_life_for_damage() {
    let mut g = main_phase();
    let engine = g.add_card_to_battlefield(0, catalog::reckless_assault());
    let before = g.players[0].life;
    activate(&mut g, 0, engine, 0, Some(Target::Player(1)));
    assert_eq!(g.players[0].life, before - 2);
    assert_eq!(g.players[1].life, 19);
}
