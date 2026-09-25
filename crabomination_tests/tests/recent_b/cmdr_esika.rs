//! Commander: the From Cute to Brute Secret Lair deck (SLD, Esika, God of
//! the Tree, `decks::cmdr_esika`) — almost every card double-faced.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn loyalty(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .expect("loyalty");
    drain_stack(g);
}

fn on_board(g: &GameState, seat: usize, name: &str) -> Option<CardId> {
    g.battlefield.iter().find(|c| c.controller == seat && c.definition.name == name).map(|c| c.id)
}

fn count(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn untap(g: &mut GameState, id: CardId) {
    if let Some(c) = g.battlefield_find_mut(id) {
        c.tapped = false;
    }
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

/// CR 712 — Treasure Map scries and collects landmark counters; the third
/// transforms it into Treasure Cove with three Treasures.
#[test]
fn treasure_map_finds_the_cove() {
    let mut g = main_phase();
    library(&mut g, 0, 5);
    let map = g.add_card_to_battlefield(0, catalog::treasure_map());
    for n in 1..=2 {
        activate(&mut g, map, 0, None).expect("{1}, {T}");
        untap(&mut g, map);
        assert_eq!(g.battlefield_find(map).unwrap().counter_count(CounterType::Landmark), n);
    }
    activate(&mut g, map, 0, None).expect("the third");
    let cove = g.battlefield_find(map).expect("still on the battlefield");
    assert_eq!(cove.definition.name, "Treasure Cove");
    assert_eq!(cove.counter_count(CounterType::Landmark), 0);
    assert_eq!(count(&g, 0, "Treasure"), 3);
}

/// Garruk Relentless fights down to two loyalty and transforms (CR 704 /
/// the state trigger).
#[test]
fn garruk_relentless_becomes_veil_cursed() {
    let mut g = main_phase();
    let garruk = g.add_card_to_battlefield(0, catalog::garruk_relentless());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    loyalty(&mut g, garruk, 0, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "3 damage kills the bear");
    let cursed = g.battlefield_find(garruk).expect("survives on 1 loyalty");
    assert_eq!(cursed.definition.name, "Garruk, the Veil-Cursed");
}

/// Jace, Vryn's Prodigy loots and, with five cards in the graveyard, comes
/// back as Jace, Telepath Unbound (CR 701.28 — exiled and returned).
#[test]
fn jace_flips_with_five_in_the_graveyard() {
    let mut g = main_phase();
    library(&mut g, 0, 3);
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    let jace = g.add_card_to_battlefield(0, catalog::jace_vryns_prodigy());
    g.clear_sickness(jace);
    activate(&mut g, jace, 0, None).expect("{T}: loot");
    let unbound = on_board(&g, 0, "Jace, Telepath Unbound").expect("the planeswalker side");
    assert_eq!(g.battlefield_find(unbound).unwrap().counter_count(CounterType::Loyalty), 5);
}

/// Esika gives other legendary creatures vigilance and a mana ability; her
/// back, The Prismatic Bridge, is cast for {W}{U}{B}{R}{G} (CR 712.12).
#[test]
fn esika_and_the_prismatic_bridge() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::esika_god_of_the_tree());
    let sisay = g.add_card_to_battlefield(0, catalog::sisay_weatherlight_captain());
    let c = g.computed_permanent(sisay).unwrap();
    assert!(c.keywords().contains(&Keyword::Vigilance));
    let bridge = g.add_card_to_hand(0, catalog::esika_god_of_the_tree());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellBack { card_id: bridge, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("the MDFC back");
    drain_stack(&mut g);
    assert!(on_board(&g, 0, "The Prismatic Bridge").is_some());
}

/// Sisay grows by the colors among your other legendary permanents.
#[test]
fn sisay_counts_legendary_colors() {
    let mut g = main_phase();
    let sisay = g.add_card_to_battlefield(0, catalog::sisay_weatherlight_captain());
    assert_eq!(pt(&g, sisay), (2, 2));
    g.add_card_to_battlefield(0, catalog::kinnan_bonder_prodigy());
    assert_eq!(pt(&g, sisay), (4, 4), "green and blue");
}

/// Hadana's Climb's third counter on one creature transforms it into the
/// Winged Temple of Orazca.
#[test]
fn hadanas_climb_reaches_the_temple() {
    let mut g = main_phase();
    let climb = g.add_card_to_battlefield(0, catalog::hadanas_climb());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    assert_eq!(g.battlefield_find(climb).unwrap().definition.name, "Winged Temple of Orazca");
}

/// Kinnan: a nonland mana permanent makes one more mana.
#[test]
fn kinnan_doubles_up_mana_rocks() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::kinnan_bonder_prodigy());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.players[0].mana_pool = Default::default();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ring,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap Sol Ring");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3, "two colorless and one more");
}

/// Voldaren Pariah sacrifices three others to become the Abolisher, which
/// makes an opponent sacrifice three creatures.
#[test]
fn voldaren_pariah_abolishes() {
    let mut g = main_phase();
    let pariah = g.add_card_to_battlefield(0, catalog::voldaren_pariah());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    activate(&mut g, pariah, 0, None).expect("sacrifice three");
    assert_eq!(g.battlefield_find(pariah).unwrap().definition.name, "Abolisher of Bloodlines");
    assert_eq!(count(&g, 0, "Grizzly Bears"), 0);
    assert_eq!(count(&g, 1, "Grizzly Bears"), 0, "the opponent sacrifices three");
}

/// Liliana: another nontoken creature of yours dying flips her and makes a
/// 2/2 Zombie.
#[test]
fn liliana_becomes_the_defiant_necromancer() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::liliana_heretical_healer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt the bear");
    assert!(on_board(&g, 0, "Liliana, Defiant Necromancer").is_some());
    assert_eq!(count(&g, 0, "Zombie"), 1);
}

/// Dowsing Dagger gives an opponent two 0/2 Plants; connecting may
/// transform it into Lost Vale.
#[test]
fn dowsing_dagger_finds_the_lost_vale() {
    let mut g = main_phase();
    let dagger = g.add_card_to_hand(0, catalog::dowsing_dagger());
    cast(&mut g, dagger, Some(Target::Player(1))).expect("cast");
    assert_eq!(count(&g, 1, "Plant"), 2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g);
    g.perform_action(GameAction::Equip { equipment: dagger, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bear), (4, 3));
    g.clear_sickness(bear);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(dagger).map(|c| c.definition.name), Some("Lost Vale"));
}

/// Nissa, Vastwood Seer fetches a Forest and flips on the seventh land.
#[test]
fn nissa_flips_on_the_seventh_land() {
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::forest());
    let nissa = g.add_card_to_hand(0, catalog::nissa_vastwood_seer());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([crabomination::decision::DecisionAnswer::Bool(true)]));
    cast(&mut g, nissa, None).expect("cast");
    let forest = g.players[0].hand.iter().find(|c| c.definition.name == "Forest").map(|c| c.id).expect("fetched");
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.perform_action(GameAction::PlayLand(forest)).expect("the seventh land");
    drain_stack(&mut g);
    assert!(on_board(&g, 0, "Nissa, Sage Animist").is_some());
}

/// Kolvori gets +4/+2 and vigilance with three legendary creatures.
#[test]
fn kolvori_with_three_legends() {
    let mut g = main_phase();
    let kolvori = g.add_card_to_battlefield(0, catalog::kolvori_god_of_kinship());
    g.add_card_to_battlefield(0, catalog::kinnan_bonder_prodigy());
    assert_eq!(pt(&g, kolvori), (2, 4));
    g.add_card_to_battlefield(0, catalog::sisay_weatherlight_captain());
    assert_eq!(pt(&g, kolvori), (6, 6));
    assert!(g.computed_permanent(kolvori).unwrap().keywords().contains(&Keyword::Vigilance));
}

/// The snow duals enter tapped (table-driven).
#[test]
fn snow_duals_enter_tapped() {
    for def in [catalog::rimewood_falls(), catalog::woodland_chasm()] {
        let mut g = main_phase();
        let id = g.add_card_to_hand(0, def);
        g.perform_action(GameAction::PlayLand(id)).expect("play");
        assert!(g.battlefield_find(id).unwrap().tapped);
    }
}
