//! Commander: the Quick Draw precon (OTC, Stella Lee, `decks::cmdr_stella`).

use crabomination::card::{CardDefinition, CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
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
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn cast_action(id: CardId, target: Option<Target>) -> GameAction {
    GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None }
}

/// Cast from a flooded pool and resolve everything.
fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g);
    g.perform_action(cast_action(id, target)).expect("cast");
    drain_stack(g);
}

fn bolt_face(g: &mut GameState) {
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(g, bolt, Some(Target::Player(1)));
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

fn library(g: &mut GameState, cards: Vec<CardDefinition>) -> Vec<CardId> {
    cards.into_iter().map(|c| g.add_card_to_library(0, c)).collect()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

// ── Stella Lee, Wild Card ──────────────────────────────────────────────────

/// The second spell each turn exiles the top card, playable through your
/// next turn at its own cost (CR 603.2 — one trigger, on exactly the second).
#[test]
fn stella_lee_exiles_the_top_card_on_the_second_spell() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::stella_lee_wild_card());
    let top = library(&mut g, vec![catalog::divination(), catalog::forest(), catalog::forest()])[0];
    bolt_face(&mut g);
    assert!(g.exile.iter().all(|c| c.id != top), "one spell: nothing exiled");
    bolt_face(&mut g);
    let card = g.exile.iter().find(|c| c.id == top).expect("the top card was exiled");
    assert!(card.may_play_until.is_some(), "and may be played");
    assert!(card.granted_alt_cast_cost_eot.is_some(), "paying its own cost, not free");
    bolt_face(&mut g);
    assert_eq!(g.exile.len(), 1, "the third spell doesn't trigger");
}

/// CR 602.5b — the copy ability is legal only after three spells this turn.
#[test]
fn stella_lee_copies_a_spell_once_three_are_cast() {
    let mut g = main_phase();
    let stella = g.add_card_to_battlefield(0, catalog::stella_lee_wild_card());
    g.clear_sickness(stella);
    library(&mut g, vec![catalog::forest(), catalog::forest()]);
    bolt_face(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    g.perform_action(cast_action(bolt, Some(Target::Player(1)))).expect("second bolt");
    assert!(activate(&mut g, stella, 0, Some(Target::Permanent(bolt))).is_err(), "two spells");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    g.perform_action(cast_action(bolt, Some(Target::Player(1)))).expect("third bolt");
    activate(&mut g, stella, 0, Some(Target::Permanent(bolt))).expect("three spells: copy");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20 - 3 * 4, "three bolts and a copy");
}

// ── Eris and Octavia ───────────────────────────────────────────────────────

/// CR 601.2f — {2} less per distinct mana value among instants and sorceries
/// in your graveyard; a land's mana value doesn't count.
#[test]
fn eris_costs_two_less_per_distinct_mana_value() {
    let mut g = main_phase();
    for c in [catalog::opt(), catalog::lightning_bolt(), catalog::divination(), catalog::forest()] {
        g.add_card_to_graveyard(0, c);
    }
    let eris = g.add_card_to_hand(0, catalog::eris_roar_of_the_storm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert!(g.perform_action(cast_action(eris, None)).is_err(), "MV 1 and 3: {{4}}{{U}}{{R}}");
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(cast_action(eris, None)).expect("six mana is enough");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Eris, Roar of the Storm"), 1);
}

/// The second spell each turn makes a 4/4 Dragon Elemental.
#[test]
fn eris_makes_a_dragon_on_the_second_spell() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::eris_roar_of_the_storm());
    bolt_face(&mut g);
    assert_eq!(count_named(&g, 0, "Dragon Elemental"), 0);
    bolt_face(&mut g);
    bolt_face(&mut g);
    assert_eq!(count_named(&g, 0, "Dragon Elemental"), 1, "only the second spell");
}

/// {8} less with eight instants and sorceries in the graveyard; magecraft
/// sets a creature's base P/T to 8/8 (CR 613.4b).
#[test]
fn octavia_is_cheap_with_eight_spells_and_makes_an_eight_eight() {
    let mut g = main_phase();
    let octavia = g.add_card_to_hand(0, catalog::octavia_living_thesis());
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::opt());
    }
    g.players[0].mana_pool.add(Color::Blue, 2);
    assert!(g.perform_action(cast_action(octavia, None)).is_err(), "seven spells: full price");
    g.add_card_to_graveyard(0, catalog::opt());
    g.perform_action(cast_action(octavia, None)).expect("eight spells: {U}{U}");
    drain_stack(&mut g);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    bolt_face(&mut g);
    assert_eq!(pt(&g, bears), (8, 8), "magecraft targets the bears");
}

// ── Cost reducers ──────────────────────────────────────────────────────────

/// Kaza's discount counts Wizards as the ability resolves.
#[test]
fn kaza_discounts_the_next_spell_per_wizard() {
    let mut g = main_phase();
    let kaza = g.add_card_to_battlefield(0, catalog::kaza_roil_chaser());
    g.add_card_to_battlefield(0, catalog::crackling_spellslinger());
    library(&mut g, vec![catalog::forest(), catalog::forest()]);
    activate(&mut g, kaza, 0, None).expect("tap Kaza");
    drain_stack(&mut g);
    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(cast_action(div, None)).expect("two Wizards: {U}");
}

/// Instants and sorceries cost {1} less; the sacrifice copies the next one
/// once per command-zone cast of your commander (CR 903.8).
#[test]
fn thunderclap_drake_discounts_and_copies_per_commander_cast() {
    let mut g = main_phase();
    let drake = g.add_card_to_battlefield(0, catalog::thunderclap_drake());
    let cmdr = g.add_card_to_battlefield(0, catalog::stella_lee_wild_card());
    g.players[0].commanders.push(cmdr);
    g.commander_cast_count.insert(cmdr, 2);
    library(&mut g, (0..12).map(|_| catalog::forest()).collect());
    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(cast_action(div, None)).expect("{1}{U}");
    drain_stack(&mut g);
    flood(&mut g);
    activate(&mut g, drake, 0, None).expect("sacrifice");
    drain_stack(&mut g);
    let hand = g.players[0].hand.len();
    let div = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, div, None);
    assert_eq!(g.players[0].hand.len(), hand + 2 * 3, "the spell and two copies");
}

// ── Storm and cascade ──────────────────────────────────────────────────────

/// Cast, Spellslinger gives the next instant or sorcery storm (CR 702.40a);
/// put onto the battlefield some other way, it doesn't (CR 603.4).
#[test]
fn crackling_spellslinger_gives_the_next_spell_storm() {
    let mut g = main_phase();
    let sling = g.add_card_to_hand(0, catalog::crackling_spellslinger());
    cast(&mut g, sling, None);
    bolt_face(&mut g);
    assert_eq!(g.players[1].life, 20 - 3 * 2, "one spell before it: one copy");
    bolt_face(&mut g);
    assert_eq!(g.players[1].life, 20 - 3 * 3, "the storm was used up");
    let other = g.add_card_to_battlefield_entering(0, catalog::crackling_spellslinger());
    g.fire_self_etb_triggers(other, 0);
    drain_stack(&mut g);
    bolt_face(&mut g);
    assert_eq!(g.players[1].life, 20 - 3 * 4, "not cast: no storm");
}

/// Storm copies a token-making sorcery (CR 702.40a).
#[test]
fn elemental_eruption_storms_up_dragons() {
    let mut g = main_phase();
    bolt_face(&mut g);
    bolt_face(&mut g);
    let erupt = g.add_card_to_hand(0, catalog::elemental_eruption());
    cast(&mut g, erupt, None);
    assert_eq!(count_named(&g, 0, "Dragon Elemental"), 3);
}

/// Cascade (CR 702.85a) casts the Divination first, so X counts both.
#[test]
fn volcanic_torrent_cascades_then_counts_every_spell() {
    let mut g = main_phase();
    library(&mut g, vec![catalog::forest(), catalog::divination(), catalog::forest(), catalog::forest()]);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let torrent = g.add_card_to_hand(0, catalog::volcanic_torrent());
    cast(&mut g, torrent, None);
    assert_eq!(g.players[0].hand.len(), hand + 2, "Divination was cast free");
    assert!(g.battlefield.find_by_id(bears).is_none(), "two damage kills the bears");
    assert!(g.battlefield.find_by_id(giant).is_some(), "the giant survives two");
    assert!(g.battlefield.find_by_id(mine).is_some(), "only opponents' creatures");
}

/// Attacking arms one cascade for the next instant and one for the next
/// sorcery (CR 702.85a); casting a sorcery leaves the instant's armed.
#[test]
fn smoldering_stagecoach_arms_cascade_per_spell_type() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::opt());
    }
    let coach = g.add_card_to_battlefield(0, catalog::smoldering_stagecoach());
    let crew = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(crew);
    g.clear_sickness(coach);
    let mut lib = vec![catalog::forest(), catalog::lava_spike(), catalog::forest(), catalog::forest()];
    lib.push(catalog::ornithopter());
    lib.extend((0..4).map(|_| catalog::forest()));
    library(&mut g, lib);
    g.perform_action(GameAction::Crew { vehicle: coach, crew_creatures: vec![crew] }).expect("crew");
    assert_eq!(pt(&g, coach), (3, 5), "power = instants and sorceries in the graveyard");
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: coach,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    let div = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, div, None);
    assert_eq!(g.players[1].life, 20 - 3, "the sorcery cascaded into Lava Spike");
    assert_eq!(count_named(&g, 0, "Ornithopter"), 0, "the instant's cascade is still armed");
    bolt_face(&mut g);
    assert_eq!(count_named(&g, 0, "Ornithopter"), 1, "the instant cascaded into Ornithopter");
}

// ── Card flow ──────────────────────────────────────────────────────────────

/// One card, plus one per OTHER instant and sorcery cast this turn.
#[test]
fn lock_and_load_draws_per_other_instant_and_sorcery() {
    let mut g = main_phase();
    library(&mut g, (0..6).map(|_| catalog::forest()).collect());
    bolt_face(&mut g);
    bolt_face(&mut g);
    let hand = g.players[0].hand.len();
    let lnl = g.add_card_to_hand(0, catalog::lock_and_load());
    cast(&mut g, lnl, None);
    assert_eq!(g.players[0].hand.len(), hand + 3);
}

/// Discard the hand, draw four, +1/+0 per discarded card.
#[test]
fn pyretic_charge_pumps_per_discarded_card() {
    let mut g = main_phase();
    library(&mut g, (0..6).map(|_| catalog::forest()).collect());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let charge = g.add_card_to_hand(0, catalog::pyretic_charge());
    cast(&mut g, charge, None);
    assert_eq!(g.players[0].hand.len(), 4);
    assert_eq!(pt(&g, bears), (5, 2), "three discarded");
}

/// {R} per card in the target opponent's hand, then CR 702.62a — it exiles
/// itself with three time counters, suspended again.
#[test]
fn rousing_refrain_adds_mana_and_suspends_itself() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::forest());
    }
    let refrain = g.add_card_to_hand(0, catalog::rousing_refrain());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(cast_action(refrain, Some(Target::Player(1)))).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 4);
    let exiled = g.exile.iter().find(|c| c.id == refrain).expect("exiled");
    assert_eq!(exiled.counter_count(CounterType::Time), 3);
}

/// Mill one; an instant or sorcery may go to hand. Tapping a legendary
/// creature untaps it.
#[test]
fn leyline_dowser_digs_and_untaps_off_a_legend() {
    let mut g = main_phase();
    let dowser = g.add_card_to_battlefield(0, catalog::leyline_dowser());
    let kaza = g.add_card_to_battlefield(0, catalog::kaza_roil_chaser());
    let opt = library(&mut g, vec![catalog::opt(), catalog::forest()])[0];
    flood(&mut g);
    activate(&mut g, dowser, 0, None).expect("dig");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == opt), "Opt to hand");
    activate(&mut g, dowser, 1, None).expect("untap");
    drain_stack(&mut g);
    assert!(!g.battlefield.find_by_id(dowser).unwrap().tapped);
    assert!(g.battlefield.find_by_id(kaza).unwrap().tapped);
}

/// CR 106.6 — the Foundry's mana exiles the cheap spell it funds with the
/// Foundry, and the second ability casts it again for free.
#[test]
fn forgers_foundry_exiles_what_it_funds_and_recasts_it() {
    let mut g = main_phase();
    let foundry = g.add_card_to_battlefield(0, catalog::forgers_foundry());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let shock = g.add_card_to_hand(0, catalog::shock());
    // A Shock paid with ordinary mana goes to the graveyard.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(cast_action(shock, Some(Target::Player(1)))).expect("shock");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == shock));
    activate(&mut g, foundry, 0, None).expect("tap for {U}");
    g.players[0].mana_pool.add(Color::Red, 1);
    // Bolt costs {R}; the rider mana pays nothing here, so fund an Opt.
    let opt = g.add_card_to_hand(0, catalog::opt());
    library(&mut g, vec![catalog::forest(), catalog::forest()]);
    g.perform_action(cast_action(opt, None)).expect("opt off the Foundry");
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == opt).expect("Opt exiled instead");
    assert_eq!(exiled.exiled_with, Some(foundry));
    g.perform_action(cast_action(bolt, Some(Target::Player(1)))).expect("bolt, plain {R}");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "ordinary mana: graveyard");
    g.battlefield_find_mut(foundry).unwrap().tapped = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    flood(&mut g);
    let hand = g.players[0].hand.len();
    activate(&mut g, foundry, 1, None).expect("recast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "Opt drew again");
}

/// The first instant or sorcery each turn exiles one at random from the
/// graveyard and offers a free copy of each card exiled with it (CR 707.12).
#[test]
fn arcane_bombardment_builds_a_free_copy_pile() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::arcane_bombardment());
    let old = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 20 - 2 - 3, "the Shock and a copy of the Bolt");
    let exiled = g.exile.iter().find(|c| c.id == old).expect("exiled with it");
    assert!(exiled.exiled_with.is_some());
    bolt_face(&mut g);
    assert_eq!(g.players[1].life, 20 - 5 - 3, "only the first spell each turn");
}

/// CR 601.2b — Finale of Promise's slots read the cast's X ("mana value X or
/// less"): with X given, the all-slots picker finds the graveyard instant and
/// sorcery; without it that atom matches nothing, which is why no bot cast it.
#[test]
fn finale_of_promise_targets_read_the_casts_x() {
    let mut g = main_phase();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let div = g.add_card_to_graveyard(0, catalog::divination());
    let finale = catalog::finale_of_promise().effect;
    let (t, extra) = g.auto_targets_for_effect_all_slots_x(&finale, 0, None, false, None, Some(3));
    assert_eq!(t, Some(Target::Permanent(bolt)));
    assert_eq!(extra, vec![Target::Permanent(div)]);
    let (t, _) = g.auto_targets_for_effect_all_slots_x(&finale, 0, None, false, None, Some(2));
    assert_eq!(t, Some(Target::Permanent(bolt)), "X = 2: the Bolt still fits");
    assert_eq!(g.auto_targets_for_effect_all_slots(&finale, 0, None).0, None, "no X: nothing");
}
