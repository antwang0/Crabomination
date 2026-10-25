//! Commander: the 2026-10-24 (`01M2Ffkh`) Isu the Abominable closers
//! (`decks::cmdr_01m2_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, GameError, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

/// Isu plays a snow land and casts a snow spell off the top of the library
/// (never a non-snow one), and a snow arrival buys a +1/+1 counter for the
/// chosen colour.
#[test]
fn isu_plays_snow_off_the_top_and_grows() {
    let mut g = pod(2);
    let isu = ready(&mut g, 0, catalog::isu_the_abominable());
    let land = g.add_card_to_library(0, catalog::snow_covered_forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0), DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::PlayLand(land)).expect("a snow land off the top");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some());
    assert_eq!(g.battlefield_find(isu).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "paid {{G}}");

    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    assert!(cast(&mut g, bears).is_err(), "not a snow spell");
    g.players[0].library.retain(|c| c.id != bears);
    let yeti = g.add_card_to_library(0, catalog::frostpeak_yeti());
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(2), DecisionAnswer::Bool(false)]));
    cast(&mut g, yeti).expect("a snow spell off the top");
    assert!(g.battlefield_find(yeti).is_some());
    assert_eq!(g.battlefield_find(isu).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "declined");
}

/// Diamond Faerie's {1}{S} pumps snow creatures only, paid from snow lands.
#[test]
fn diamond_faerie_pumps_only_snow_creatures() {
    let mut g = pod(2);
    let faerie = ready(&mut g, 0, catalog::diamond_faerie());
    let yeti = ready(&mut g, 0, catalog::frostpeak_yeti());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..2 {
        ready(&mut g, 0, catalog::snow_covered_island());
    }
    activate(&mut g, faerie).expect("{1}{S} from two snow lands");
    assert_eq!(pt(&g, faerie), (4, 4));
    assert_eq!(pt(&g, yeti), (4, 4));
    assert_eq!(pt(&g, bears), (2, 2));
}

/// Snow Devil: flying always; first strike only while blocking with a snow
/// land under its controller.
#[test]
fn snow_devil_first_strike_needs_a_block_and_a_snow_land() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let devil = g.add_card_to_battlefield(0, catalog::snow_devil());
    g.battlefield_find_mut(devil).unwrap().attached_to = Some(bear);
    let has = |g: &GameState, k: Keyword| g.computed_permanent(bear).unwrap().keywords().contains(&k);
    assert!(has(&g, Keyword::Flying));
    let attacker = ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::snow_covered_island());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(!has(&g, Keyword::FirstStrike), "not blocking yet");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, attacker)])).expect("block");
    drain_stack(&mut g);
    assert!(has(&g, Keyword::FirstStrike));
}

/// Glacial Plating: +3/+3 for each age counter on it.
#[test]
fn glacial_plating_scales_with_age_counters() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let plating = g.add_card_to_battlefield(0, catalog::glacial_plating());
    g.battlefield_find_mut(plating).unwrap().attached_to = Some(bear);
    assert_eq!(pt(&g, bear), (2, 2));
    g.battlefield_find_mut(plating).unwrap().add_counters(CounterType::Age, 2);
    assert_eq!(pt(&g, bear), (8, 8));
}

/// CR 615 — Cover of Winter prevents X of each attacking creature's combat
/// damage, X its age counters.
#[test]
fn cover_of_winter_shaves_creature_combat_damage() {
    let mut g = pod(2);
    let cover = g.add_card_to_battlefield(0, catalog::cover_of_winter());
    g.battlefield_find_mut(cover).unwrap().add_counters(CounterType::Age, 1);
    let attacker = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    let life = g.players[0].life;
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    assert_eq!(g.players[0].life, life - 1, "2 power, 1 prevented");
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(6);
}

/// Bladeback Sliver: hellbent — only an empty hand gives Slivers the ping.
#[test]
fn bladeback_sliver_pings_only_when_hellbent() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let sliver = ready(&mut g, 0, catalog::bladeback_sliver());
    g.add_card_to_hand(0, catalog::island());
    let ping = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: sliver,
            ability_index: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(ping(&mut g).is_err(), "a card in hand");
    g.players[0].hand.clear();
    let life = g.players[1].life;
    ping(&mut g).expect("hellbent");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
}

/// CR 506.2 — Hellkite Whelp's attack ping hits a creature of the player it
/// attacks, not another opponent's (three seats).
#[test]
fn hellkite_whelp_pings_the_defending_players_creature() {
    let mut g = pod(3);
    let whelp = ready(&mut g, 0, catalog::hellkite_whelp());
    let theirs = ready(&mut g, 2, catalog::llanowar_elves());
    let other = ready(&mut g, 1, catalog::llanowar_elves());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: whelp, target: AttackTarget::Player(2) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "the defending player's 1/1");
    assert!(g.battlefield_find(other).is_some());
}

/// Prophetic Ravings gives its host haste and a {T}, discard: draw loot.
#[test]
fn prophetic_ravings_grants_haste_and_a_loot() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ravings = g.add_card_to_battlefield(0, catalog::prophetic_ravings());
    g.battlefield_find_mut(ravings).unwrap().attached_to = Some(bear);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Haste));
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, bear).expect("summoning sick, but hasty");
    assert_eq!(g.players[0].hand.len(), 1, "discarded one, drew one");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

/// Volatile Claws: +2/+0 and every creature type until end of turn.
#[test]
fn volatile_claws_pumps_and_grants_every_type() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let claws = g.add_card_to_hand(0, catalog::volatile_claws());
    flood(&mut g, 0);
    cast(&mut g, claws).expect("cast");
    assert_eq!(pt(&g, bear), (4, 2));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Changeling));
}

/// Vandalize — "choose one or both": an artifact and a land in one cast.
#[test]
fn vandalize_destroys_an_artifact_and_a_land() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let land = g.add_card_to_battlefield(1, catalog::island());
    let v = g.add_card_to_hand(0, catalog::vandalize());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: v,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![Target::Permanent(land)],
        x_value: None,
    })
    .expect("both modes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none() && g.battlefield_find(land).is_none());
}

/// Sun-Crowned Hunters' enrage answers damage with 3 to an opponent.
#[test]
fn sun_crowned_hunters_enrage_burns_an_opponent() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let hunters = ready(&mut g, 0, catalog::sun_crowned_hunters());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g, 0);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(hunters)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt our own");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
}

/// Stampeding Horncrest has haste only beside another Dinosaur.
#[test]
fn stampeding_horncrest_needs_another_dinosaur_for_haste() {
    let mut g = pod(2);
    let horn = g.add_card_to_battlefield(0, catalog::stampeding_horncrest());
    let haste = |g: &GameState| g.computed_permanent(horn).unwrap().keywords().contains(&Keyword::Haste);
    assert!(!haste(&g));
    g.add_card_to_battlefield(0, catalog::nest_robber());
    assert!(haste(&g));
}

/// War-Trained Slasher doubles its power attacking a battle, not a player.
#[test]
fn war_trained_slasher_doubles_only_against_a_battle() {
    let mut g = pod(2);
    let slasher = ready(&mut g, 0, catalog::war_trained_slasher());
    // Your own Siege, protected by the opponent (CR 310.9b).
    let battle = g.add_card_to_battlefield(0, catalog::invasion_of_zendikar());
    {
        let b = g.battlefield_find_mut(battle).unwrap();
        b.counters.insert(CounterType::Defense, 3);
        b.protected_by = Some(1);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: slasher, target: AttackTarget::Battle(battle) }]))
        .expect("attack the battle");
    drain_stack(&mut g);
    assert_eq!(pt(&g, slasher), (8, 3));

    let mut g = pod(2);
    let slasher = ready(&mut g, 0, catalog::war_trained_slasher());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: slasher, target: AttackTarget::Player(1) }]))
        .expect("attack a player");
    drain_stack(&mut g);
    assert_eq!(pt(&g, slasher), (4, 3));
}

/// Tilonalli's Crown pings its host as it enters, then gives +3/+0 and trample.
#[test]
fn tilonallis_crown_pings_then_pumps() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let crown = g.add_card_to_hand(0, catalog::tilonallis_crown());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: crown,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("2 toughness survives 1");
    assert_eq!(c.damage, 1);
    assert_eq!(pt(&g, bear), (5, 2));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Trample));
}

/// CR 701.68 — Cinder Strike deals 4 with the optional blight paid, 2 without.
#[test]
fn cinder_strike_hits_harder_blighted() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    let wall = ready(&mut g, 1, catalog::wall_of_stone());
    let cs = g.add_card_to_hand(0, catalog::cinder_strike());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: cs,
        target: Some(Target::Permanent(wall)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("blighted");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wall).unwrap().damage, 4);
}

/// Feed the Flames exiles the creature its 5 damage kills.
#[test]
fn feed_the_flames_exiles_what_it_kills() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let f = g.add_card_to_hand(0, catalog::feed_the_flames());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: f,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear));
}

fn lands(g: &mut GameState, seat: usize, defs: Vec<crabomination::card::CardDefinition>) {
    for d in defs {
        g.add_card_to_battlefield(seat, d);
    }
}

/// CR 702.43 — domain: Territorial Maro is twice the basic land types among
/// your lands, Might of Alara pumps once per type, Manaforce Mace likewise.
#[test]
fn domain_cards_count_basic_land_types() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    lands(&mut g, 0, vec![catalog::plains(), catalog::island(), catalog::forest(), catalog::forest()]);
    let maro = ready(&mut g, 0, catalog::territorial_maro());
    assert_eq!(pt(&g, maro), (6, 6), "three types, doubled");
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let mace = g.add_card_to_battlefield(0, catalog::manaforce_mace());
    g.battlefield_find_mut(mace).unwrap().attached_to = Some(bear);
    assert_eq!(pt(&g, bear), (5, 5));
    let might = g.add_card_to_hand(0, catalog::might_of_alara());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: might, target: Some(Target::Permanent(maro)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(pt(&g, maro), (9, 9));
}

/// Herd Migration: a 3/3 Beast per basic land type; from hand, {1}{G} and a
/// discard fetch a basic land and gain 3 life.
#[test]
fn herd_migration_beasts_and_its_hand_cycling() {
    let mut g = pod(2);
    lands(&mut g, 0, vec![catalog::swamp(), catalog::mountain()]);
    let hm = g.add_card_to_hand(0, catalog::herd_migration());
    flood(&mut g, 0);
    cast(&mut g, hm).expect("cast");
    let beasts = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Beast").count();
    assert_eq!(beasts, 2);

    let mut g = pod(2);
    let basic = g.add_card_to_library(0, catalog::forest());
    let hm = g.add_card_to_hand(0, catalog::herd_migration());
    flood(&mut g, 0);
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
    activate(&mut g, hm).expect("{1}{G}, discard it");
    assert!(g.players[0].hand.iter().any(|c| c.id == basic));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == hm));
    assert_eq!(g.players[0].life, life + 3);
}

/// Pixie Illusionist kicked enters with two +1/+1 counters.
#[test]
fn pixie_illusionist_kicked_enters_with_two_counters() {
    let mut g = pod(2);
    let pixie = g.add_card_to_hand(0, catalog::pixie_illusionist());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked { card_id: pixie, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("kicked");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(pixie).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Nael's combat damage digs and, with all five basic land types, draws.
#[test]
fn nael_draws_with_five_basic_land_types() {
    let mut g = pod(2);
    lands(&mut g, 0, vec![catalog::plains(), catalog::island(), catalog::swamp(), catalog::mountain(), catalog::forest()]);
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let nael = ready(&mut g, 0, catalog::nael_avizoa_aeronaut());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: nael, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    let hand = g.players[0].hand.len();
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

fn cast_kicked(g: &mut GameState, id: CardId, target: Option<crabomination::game::types::Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpellKicked { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

/// Verazol enters with a counter per mana spent; a kicked spell may cost it
/// two counters to copy (CR 707.10) — Field Research kicked twice over.
#[test]
fn verazol_copies_a_kicked_spell_for_two_counters() {
    let mut g = pod(2);
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    let v = g.add_card_to_hand(0, catalog::verazol_the_split_current());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: v, target: None, additional_targets: vec![], mode: None, x_value: Some(2) })
        .expect("X = 2");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(v).unwrap().counter_count(CounterType::PlusOnePlusOne), 4, "four mana spent");
    let fr = g.add_card_to_hand(0, catalog::field_research());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast_kicked(&mut g, fr, None).expect("kicked");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 6, "three and a copy's three");
    assert_eq!(g.battlefield_find(v).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Roost of Drakes: kicked, a Drake as it enters; each later kicked spell, another.
#[test]
fn roost_of_drakes_makes_drakes_off_kicker() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let roost = g.add_card_to_hand(0, catalog::roost_of_drakes());
    flood(&mut g, 0);
    cast_kicked(&mut g, roost, None).expect("kicked");
    let drakes = |g: &GameState| g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Drake").count();
    assert_eq!(drakes(&g), 1);
    let fr = g.add_card_to_hand(0, catalog::field_research());
    flood(&mut g, 0);
    cast_kicked(&mut g, fr, None).expect("kicked");
    assert_eq!(drakes(&g), 2);
}

/// Territorial Allosaurus kicked fights another target creature.
#[test]
fn territorial_allosaurus_kicked_fights() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let allo = g.add_card_to_hand(0, catalog::territorial_allosaurus());
    flood(&mut g, 0);
    cast_kicked(&mut g, allo, None).expect("kicked; the trigger aims at the only other creature");
    assert!(g.battlefield_find(bear).is_none(), "5 damage kills the Bears");
    assert_eq!(g.battlefield_find(allo).unwrap().damage, 2);
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<crabomination::game::types::Target>, x: Option<u32>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: x })
        .expect("loyalty ability");
    drain_stack(g);
}

/// CR 614.1a — Pyromancer's Gauntlet adds 2 to your red sorcery and your red
/// planeswalker's damage, never to an opponent's spell.
#[test]
fn pyromancers_gauntlet_adds_two_to_your_red_spells_and_walkers() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::pyromancers_gauntlet());
    let pb = g.add_card_to_hand(0, catalog::precision_bolt());
    flood(&mut g, 0);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell { card_id: pb, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 5, "3 + 2");
    let nalaar = g.add_card_to_battlefield(0, catalog::chandra_nalaar());
    g.battlefield_find_mut(nalaar).unwrap().add_counters(CounterType::Loyalty, 6);
    let life = g.players[1].life;
    loyalty(&mut g, nalaar, 0, Some(Target::Player(1)), None);
    assert_eq!(g.players[1].life, life - 3, "1 + 2 from a red planeswalker");
    // An opponent's red spell is theirs, not the Gauntlet controller's.
    let theirs = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    let mine = g.players[0].life;
    g.perform_action(GameAction::CastSpell { card_id: theirs, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, mine - 3, "a Lightning Bolt, unboosted");
}

/// Chandra's Fury: 4 to the targeted player and 1 to each creature that
/// player controls — not another opponent's (three seats).
#[test]
fn chandras_fury_sweeps_only_the_targeted_players_creatures() {
    use crabomination::game::types::Target;
    let mut g = pod(3);
    let theirs = ready(&mut g, 1, catalog::llanowar_elves());
    let other = ready(&mut g, 2, catalog::llanowar_elves());
    let fury = g.add_card_to_hand(0, catalog::chandras_fury());
    flood(&mut g, 0);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell { card_id: fury, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 4);
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(other).is_some());
}

/// Keral Keep Disciples pings each opponent when a Chandra's loyalty ability
/// is activated.
#[test]
fn keral_keep_disciples_ping_on_chandra_loyalty() {
    use crabomination::game::types::Target;
    let mut g = pod(3);
    ready(&mut g, 0, catalog::keral_keep_disciples());
    let fb = g.add_card_to_battlefield(0, catalog::chandra_the_firebrand());
    g.battlefield_find_mut(fb).unwrap().add_counters(CounterType::Loyalty, 3);
    let lives: Vec<i32> = (1..3).map(|p| g.players[p].life).collect();
    loyalty(&mut g, fb, 0, Some(Target::Player(1)), None);
    assert_eq!(g.players[1].life, lives[0] - 2, "the +1 and the Disciples");
    assert_eq!(g.players[2].life, lives[1] - 1);
}

/// Chandra, Heart of Fire's −9 exiles every red instant and sorcery from the
/// graveyard and library, castable this turn, and adds six red.
#[test]
fn chandra_heart_of_fire_ultimate_gathers_red_spells() {
    let mut g = pod(2);
    let gy = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let lib = g.add_card_to_library(0, catalog::precision_bolt());
    let keep = g.add_card_to_library(0, catalog::island());
    let hof = g.add_card_to_battlefield(0, catalog::chandra_heart_of_fire());
    g.battlefield_find_mut(hof).unwrap().add_counters(CounterType::Loyalty, 9);
    loyalty(&mut g, hof, 2, None, None);
    assert!([gy, lib].iter().all(|id| g.exile.iter().any(|c| c.id == *id)));
    assert!(g.players[0].library.iter().any(|c| c.id == keep));
    assert_eq!(g.players[0].mana_pool.amount(crabomination::mana::Color::Red), 6);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: lib,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("castable from exile this turn");
}

/// Chandra, Pyromaster's −7 casts an exiled instant three times for free.
#[test]
fn chandra_pyromaster_ultimate_casts_three_copies() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::lightning_bolt());
    for _ in 0..9 {
        g.add_card_to_library(0, catalog::island());
    }
    let pm = g.add_card_to_battlefield(0, catalog::chandra_pyromaster());
    g.battlefield_find_mut(pm).unwrap().add_counters(CounterType::Loyalty, 7);
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    loyalty(&mut g, pm, 2, None, None);
    assert_eq!(g.players[1].life, life - 9, "three Bolts at the opponent");
}
