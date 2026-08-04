//! The Dark (DRK) — `catalog::sets::drk`.

use crabomination::card::{CardId, Keyword};
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

fn activate(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    activate_n(g, seat, id, 0, target)
}

#[test]
fn keyword_bodies_are_printed_correctly() {
    let mut g = main_phase();
    for (def, kw) in [
        (catalog::carnivorous_plant(), Keyword::Defender),
        (catalog::land_leeches(), Keyword::FirstStrike),
        (catalog::knights_of_thorn(), Keyword::Protection(Color::Red)),
        (catalog::pikemen(), Keyword::Banding),
        (catalog::goblins_of_the_flarg(), Keyword::Landwalk(crabomination::card::LandType::Mountain)),
    ] {
        let name = def.name;
        let id = g.add_card_to_battlefield(0, def);
        assert!(
            g.computed_permanent(id).expect(name).keywords.contains(&kw),
            "{name} is missing {kw:?}",
        );
    }
}

#[test]
fn apprentice_wizard_turns_one_blue_into_three() {
    let mut g = main_phase();
    let wiz = g.add_card_to_battlefield(0, catalog::apprentice_wizard());
    g.clear_sickness(wiz);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: wiz,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

#[test]
fn cave_people_trade_toughness_for_power_when_attacking() {
    let mut g = main_phase();
    let people = g.add_card_to_battlefield(0, catalog::cave_people());
    g.clear_sickness(people);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: people, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(people).expect("cave people");
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

#[test]
fn coal_golem_burns_into_three_red() {
    let mut g = main_phase();
    let golem = g.add_card_to_battlefield(0, catalog::coal_golem());
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: golem,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(golem).is_none());
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

#[test]
fn electric_eel_shocks_you_on_entry_and_on_pump() {
    let mut g = main_phase();
    let eel = g.add_card_to_hand(0, catalog::electric_eel());
    cast(&mut g, 0, eel, None);
    assert_eq!(g.players[0].life, 19);
    activate(&mut g, 0, eel, None);
    assert_eq!(g.players[0].life, 18);
    assert_eq!(g.computed_permanent(eel).expect("eel").power, 3);
}

#[test]
fn exorcist_only_kills_black_creatures() {
    let mut g = main_phase();
    let ex = g.add_card_to_battlefield(0, catalog::exorcist());
    g.clear_sickness(ex);
    let green = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: ex,
            ability_index: 0,
            target: Some(Target::Permanent(green)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
    );
    let black = g.add_card_to_battlefield(1, catalog::uncle_istvan());
    activate(&mut g, 0, ex, Some(Target::Permanent(black)));
    assert!(g.battlefield_find(black).is_none());
}

#[test]
fn goblins_of_the_flarg_wont_serve_beside_a_dwarf() {
    let mut g = main_phase();
    let gob = g.add_card_to_battlefield(0, catalog::goblins_of_the_flarg());
    let dwarf = g.add_card_to_hand(0, catalog::dwarven_patrol());
    cast(&mut g, 0, dwarf, None);
    assert!(g.battlefield_find(gob).is_none(), "the Dwarf drove them off");
}

#[test]
fn murk_dwellers_swell_when_unblocked() {
    let mut g = main_phase();
    let dwellers = g.add_card_to_battlefield(0, catalog::murk_dwellers());
    g.clear_sickness(dwellers);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: dwellers, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(dwellers).expect("dwellers").power, 4);
}

#[test]
fn people_of_the_woods_scale_with_forests() {
    let mut g = main_phase();
    let folk = g.add_card_to_battlefield(0, catalog::people_of_the_woods());
    assert_eq!(g.computed_permanent(folk).expect("folk").toughness, 0);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let cp = g.computed_permanent(folk).expect("folk");
    assert_eq!((cp.power, cp.toughness), (1, 3));
}

#[test]
fn scavenger_folk_eat_an_artifact() {
    let mut g = main_phase();
    let folk = g.add_card_to_battlefield(0, catalog::scavenger_folk());
    g.clear_sickness(folk);
    let chalice = g.add_card_to_battlefield(1, catalog::urzas_chalice());
    activate(&mut g, 0, folk, Some(Target::Permanent(chalice)));
    assert!(g.battlefield_find(chalice).is_none());
    assert!(g.battlefield_find(folk).is_none(), "it sacrificed itself");
}

#[test]
fn uncle_istvan_shrugs_off_creature_damage() {
    let mut g = main_phase();
    let uncle = g.add_card_to_battlefield(1, catalog::uncle_istvan());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(uncle, bear)])).expect("block");
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(uncle).expect("uncle").damage, 0);
}

#[test]
fn water_wurm_grows_off_an_opponents_island() {
    let mut g = main_phase();
    let wurm = g.add_card_to_battlefield(0, catalog::water_wurm());
    assert_eq!(g.computed_permanent(wurm).expect("wurm").toughness, 1);
    g.add_card_to_battlefield(1, catalog::island());
    assert_eq!(g.computed_permanent(wurm).expect("wurm").toughness, 2);
}

#[test]
fn witch_hunter_pings_and_bounces() {
    let mut g = main_phase();
    let hunter = g.add_card_to_battlefield(0, catalog::witch_hunter());
    g.clear_sickness(hunter);
    activate(&mut g, 0, hunter, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 19);
    g.battlefield_find_mut(hunter).expect("hunter").tapped = false;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate_n(&mut g, 0, hunter, 1, Some(Target::Permanent(bear)));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

#[test]
fn wormwood_treefolk_buys_forestwalk_with_life() {
    let mut g = main_phase();
    let tree = g.add_card_to_battlefield(0, catalog::wormwood_treefolk());
    activate(&mut g, 0, tree, None);
    assert!(
        g.computed_permanent(tree)
            .expect("treefolk")
            .keywords
            .contains(&Keyword::Landwalk(crabomination::card::LandType::Forest)),
    );
    assert_eq!(g.players[0].life, 18);
}

// ── Spells ─────────────────────────────────────────────────────────────────

#[test]
fn amnesia_leaves_only_lands() {
    let mut g = main_phase();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::amnesia());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 1);
    assert!(g.players[1].hand[0].definition.is_land());
}

#[test]
fn ashes_to_ashes_exiles_two_and_bleeds_you() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::drowned());
    let spell = g.add_card_to_hand(0, catalog::ashes_to_ashes());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert_eq!(g.players[0].life, 15);
}

#[test]
fn marsh_gas_shrinks_every_creature() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::marsh_gas());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 0);
}

#[test]
fn riptide_taps_only_blue() {
    let mut g = main_phase();
    let blue = g.add_card_to_battlefield(1, catalog::drowned());
    let green = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::riptide());
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield_find(blue).expect("blue").tapped);
    assert!(!g.battlefield_find(green).expect("green").tapped);
}

#[test]
fn tivadars_crusade_wipes_the_goblins() {
    let mut g = main_phase();
    let gob = g.add_card_to_battlefield(1, catalog::scarwood_goblins());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::tivadars_crusade());
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield_find(gob).is_none());
    assert!(g.battlefield_find(bear).is_some());
}

// ── Enchantments / artifacts ───────────────────────────────────────────────

#[test]
fn dark_heart_of_the_wood_eats_forests_for_life() {
    let mut g = main_phase();
    let heart = g.add_card_to_battlefield(0, catalog::dark_heart_of_the_wood());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, 0, heart, None);
    assert!(g.battlefield_find(forest).is_none());
    assert_eq!(g.players[0].life, 23);
}

#[test]
fn hidden_path_walks_every_green_creature() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::hidden_path());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(bear)
            .expect("bear")
            .keywords
            .contains(&Keyword::Landwalk(crabomination::card::LandType::Forest)),
    );
}

#[test]
fn sunken_city_pumps_blue_then_sinks() {
    let mut g = main_phase();
    let city = g.add_card_to_battlefield(0, catalog::sunken_city());
    let blue = g.add_card_to_battlefield(1, catalog::drowned());
    assert_eq!(g.computed_permanent(blue).expect("blue").power, 2);
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(city).is_none(), "the rent went unpaid");
}

#[test]
fn bone_flute_dulls_the_board() {
    let mut g = main_phase();
    let flute = g.add_card_to_battlefield(0, catalog::bone_flute());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, flute, None);
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 1);
}

#[test]
fn book_of_rass_trades_life_for_cards() {
    let mut g = main_phase();
    let book = g.add_card_to_battlefield(0, catalog::book_of_rass());
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, book, None);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].life, 18);
}

#[test]
fn standing_stones_pays_a_life_for_any_colour() {
    let mut g = main_phase();
    let stones = g.add_card_to_battlefield(0, catalog::standing_stones());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: stones,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

#[test]
fn stone_calendar_discounts_your_spells() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::stone_calendar());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G}
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("the generic pip was discounted away");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}

#[test]
fn tower_of_coireall_walks_past_walls() {
    let mut g = main_phase();
    let tower = g.add_card_to_battlefield(0, catalog::tower_of_coireall());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, tower, Some(Target::Permanent(bear)));
    assert!(
        g.computed_permanent(bear)
            .expect("bear")
            .keywords
            .contains(&Keyword::CantBeBlockedByCreatureType(
                crabomination::card::CreatureType::Wall,
            )),
    );
}

#[test]
fn fountain_of_youth_drips_life() {
    let mut g = main_phase();
    let fount = g.add_card_to_battlefield(0, catalog::fountain_of_youth());
    activate(&mut g, 0, fount, None);
    assert_eq!(g.players[0].life, 21);
}

#[test]
fn skull_of_orm_buys_back_an_enchantment() {
    let mut g = main_phase();
    let skull = g.add_card_to_battlefield(0, catalog::skull_of_orm());
    let city = g.add_card_to_graveyard(0, catalog::sunken_city());
    activate(&mut g, 0, skull, Some(Target::Permanent(city)));
    assert!(g.players[0].hand.iter().any(|c| c.id == city));
}
