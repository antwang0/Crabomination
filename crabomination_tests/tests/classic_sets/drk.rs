//! The Dark (DRK) — `catalog::sets::drk`.

use crabomination::card::{CardId, Keyword};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

// ── Closing wave (`catalog::sets::drk2`) ───────────────────────────────────

#[test]
fn banshee_splits_x_between_a_target_and_you() {
    let mut g = main_phase();
    let banshee = g.add_card_to_battlefield(0, catalog::banshee());
    g.clear_sickness(banshee);
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: banshee,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: Some(5),
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    // Half of 5 rounded down out, rounded up back.
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 17);
}

#[test]
fn barls_cage_keeps_a_creature_down_through_its_untap_step() {
    let mut g = main_phase();
    let cage = g.add_card_to_battlefield(0, catalog::barls_cage());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).expect("bear").tapped = true;
    activate(&mut g, 0, cage, Some(Target::Permanent(bear)));
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bear).expect("bear").tapped);
}

#[test]
fn city_of_shadows_banks_creatures_into_colorless() {
    let mut g = main_phase();
    let city = g.add_card_to_battlefield(0, catalog::city_of_shadows());
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        activate_n(&mut g, 0, city, 0, None);
        g.battlefield_find_mut(city).expect("city").tapped = false;
    }
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: city,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for banked mana");
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

#[test]
fn eater_of_the_dead_untaps_by_eating_a_corpse() {
    let mut g = main_phase();
    let eater = g.add_card_to_battlefield(0, catalog::eater_of_the_dead());
    g.clear_sickness(eater);
    g.battlefield_find_mut(eater).expect("eater").tapped = true;
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    activate(&mut g, 0, eater, Some(Target::Permanent(corpse)));
    assert!(!g.battlefield_find(eater).expect("eater").tapped);
    assert!(g.exile.iter().any(|c| c.id == corpse));
}

#[test]
fn giant_shark_dies_without_an_island() {
    let mut g = main_phase();
    let shark = g.add_card_to_battlefield(0, catalog::giant_shark());
    drain_stack(&mut g);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(shark).is_none());
}

#[test]
fn goblin_wizard_deploys_a_goblin_from_hand() {
    let mut g = main_phase();
    let wiz = g.add_card_to_battlefield(0, catalog::goblin_wizard());
    g.clear_sickness(wiz);
    let sled = g.add_card_to_hand(0, catalog::goblin_rock_sled());
    activate_n(&mut g, 0, wiz, 0, None);
    assert!(g.battlefield_find(sled).is_some());
}

#[test]
fn orc_general_eats_a_goblin_to_pump_the_orcs() {
    let mut g = main_phase();
    let fodder = g.add_card_to_battlefield(0, catalog::goblin_wizard()); // 1/1
    let general = g.add_card_to_battlefield(0, catalog::orc_general());
    g.clear_sickness(general);
    let other = g.add_card_to_battlefield(0, catalog::orc_general());
    activate(&mut g, 0, general, None);
    assert!(g.battlefield_find(fodder).is_none());
    assert_eq!(g.computed_permanent(other).expect("orc").power, 3);
    // "Other Orc creatures" — the general itself misses out.
    assert_eq!(g.computed_permanent(general).expect("general").power, 2);
}

#[test]
fn rag_man_rips_a_creature_at_random() {
    let mut g = main_phase();
    let rag = g.add_card_to_battlefield(0, catalog::rag_man());
    g.clear_sickness(rag);
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::mountain());
    activate(&mut g, 0, rag, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 1);
    assert!(g.players[1].hand[0].definition.is_land());
}

#[test]
fn scarwood_hag_grants_then_strips_forestwalk() {
    use crabomination::card::LandType;
    let mut g = main_phase();
    let hag = g.add_card_to_battlefield(0, catalog::scarwood_hag());
    g.clear_sickness(hag);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate_n(&mut g, 0, hag, 0, Some(Target::Permanent(bear)));
    assert!(
        g.computed_permanent(bear)
            .expect("bear")
            .keywords
            .contains(&Keyword::Landwalk(LandType::Forest))
    );
    g.battlefield_find_mut(hag).expect("hag").tapped = false;
    activate_n(&mut g, 0, hag, 1, Some(Target::Permanent(bear)));
    assert!(
        !g.computed_permanent(bear)
            .expect("bear")
            .keywords
            .contains(&Keyword::Landwalk(LandType::Forest))
    );
}

#[test]
fn tracker_trades_damage_both_ways() {
    let mut g = main_phase();
    let tracker = g.add_card_to_battlefield(0, catalog::tracker());
    g.clear_sickness(tracker);
    let wall = g.add_card_to_battlefield(1, catalog::carnivorous_plant()); // 4/5
    activate(&mut g, 0, tracker, Some(Target::Permanent(wall)));
    assert_eq!(g.battlefield_find(wall).expect("plant").damage, 2);
    assert!(g.battlefield_find(tracker).is_none(), "the 4/5 swings back for lethal");
}

#[test]
fn whippoorwill_exiles_its_mark_instead_of_burying_it() {
    let mut g = main_phase();
    let bird = g.add_card_to_battlefield(0, catalog::whippoorwill());
    g.clear_sickness(bird);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, bird, Some(Target::Permanent(bear)));
    g.destroy_permanent(bear, false, &mut vec![]);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear));
}

#[test]
fn living_armor_scales_with_the_targets_mana_value() {
    let mut g = main_phase();
    let armor = g.add_card_to_battlefield(0, catalog::living_armor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // {1}{G}
    activate(&mut g, 0, armor, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).expect("bear").toughness, 4);
    assert_eq!(g.computed_permanent(bear).expect("bear").power, 2);
}

#[test]
fn necropolis_grows_on_exiled_corpses() {
    let mut g = main_phase();
    let necro = g.add_card_to_battlefield(0, catalog::necropolis());
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // mv 2
    activate(&mut g, 0, necro, None);
    assert_eq!(g.computed_permanent(necro).expect("necropolis").toughness, 3);
}

#[test]
fn safe_haven_gives_the_exiles_back_when_it_goes() {
    let mut g = main_phase();
    let haven = g.add_card_to_battlefield(0, catalog::safe_haven());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, haven, Some(Target::Permanent(bear)));
    assert!(g.exile.iter().any(|c| c.id == bear));
    g.step = TurnStep::Upkeep;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert!(g.battlefield_find(haven).is_none());
}

#[test]
fn eternal_flame_burns_by_mountain_count() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let flame = g.add_card_to_hand(0, catalog::eternal_flame());
    cast(&mut g, 0, flame, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 16);
    assert_eq!(g.players[0].life, 18); // half of 4, rounded up
}

#[test]
fn inquisition_counts_white_cards() {
    let mut g = main_phase();
    g.add_card_to_hand(1, catalog::angry_mob());
    g.add_card_to_hand(1, catalog::brainwash());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let inq = g.add_card_to_hand(0, catalog::inquisition());
    cast(&mut g, 0, inq, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 18);
}

#[test]
fn martyrs_cry_exiles_the_white_board_and_pays_for_it() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::angry_mob());
    g.add_card_to_battlefield(1, catalog::angry_mob());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let before = g.players[1].hand.len();
    let cry = g.add_card_to_hand(0, catalog::martyrs_cry());
    cast(&mut g, 0, cry, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_creature()).count(), 1);
    assert_eq!(g.players[1].hand.len(), before + 2);
}

#[test]
fn word_of_binding_taps_exactly_x_creatures() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let word = g.add_card_to_hand(0, catalog::word_of_binding());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: word,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).expect("a").tapped);
    assert!(g.battlefield_find(b).expect("b").tapped);
}

#[test]
fn gaeas_touch_drops_a_forest_once_a_turn() {
    let mut g = main_phase();
    let touch = g.add_card_to_battlefield(0, catalog::gaeas_touch());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    activate_n(&mut g, 0, touch, 0, None);
    assert!(g.battlefield_find(forest).is_some());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: touch,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "once each turn",
    );
}

#[test]
fn curse_artifact_taxes_the_hosts_controller() {
    let mut g = main_phase();
    let rock = g.add_card_to_battlefield(1, catalog::diabolic_machine());
    let curse = g.add_card_to_hand(0, catalog::curse_artifact());
    cast(&mut g, 0, curse, Some(Target::Permanent(rock)));
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    // The auto-decider declines the sacrifice, so the damage lands.
    assert_eq!(g.players[1].life, 18);
    assert!(g.battlefield_find(rock).is_some());
}

#[test]
fn erosion_erodes_the_land_when_nobody_pays() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::mountain());
    let erosion = g.add_card_to_hand(0, catalog::erosion());
    cast(&mut g, 0, erosion, Some(Target::Permanent(land)));
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none());
}

#[test]
fn angry_mob_only_counts_swamps_on_your_turn() {
    let mut g = main_phase();
    let mob = g.add_card_to_battlefield(0, catalog::angry_mob());
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::swamp());
    }
    assert_eq!(g.computed_permanent(mob).expect("mob").power, 5);
    g.active_player_idx = 1;
    assert_eq!(g.computed_permanent(mob).expect("mob").power, 2);
}

#[test]
fn brainwash_taxes_the_attack() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let wash = g.add_card_to_hand(0, catalog::brainwash());
    cast(&mut g, 0, wash, Some(Target::Permanent(bear)));
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.players[1].mana_pool = Default::default();
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }])
            .is_err(),
        "an unpaid {{3}} keeps it home",
    );
}

#[test]
fn lurker_dodges_spells_until_it_fights() {
    let mut g = main_phase();
    let lurker = g.add_card_to_battlefield(1, catalog::lurker());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(lurker)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
    );
    g.battlefield_find_mut(lurker).expect("lurker").attacked_this_turn = true;
    cast(&mut g, 0, bolt, Some(Target::Permanent(lurker)));
    assert!(g.battlefield_find(lurker).is_none());
}

#[test]
fn goblin_rock_sled_sits_out_the_turn_after_it_attacks() {
    let mut g = main_phase();
    let sled = g.add_card_to_battlefield(0, catalog::goblin_rock_sled());
    g.clear_sickness(sled);
    g.add_card_to_battlefield(1, catalog::mountain());
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: sled, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    g.do_untap();
    assert!(g.battlefield_find(sled).expect("sled").tapped);
}

#[test]
fn tangle_kelp_taps_on_entry() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let kelp = g.add_card_to_hand(0, catalog::tangle_kelp());
    cast(&mut g, 0, kelp, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).expect("bear").tapped);
    assert!(
        g.computed_permanent(bear)
            .expect("bear")
            .keywords
            .contains(&Keyword::DoesntUntapIfAttackedLastTurn)
    );
}

#[test]
fn goblin_caves_only_pumps_over_a_basic_mountain() {
    let mut g = main_phase();
    let sled = g.add_card_to_battlefield(0, catalog::goblin_rock_sled());
    let wastes = g.add_card_to_battlefield(0, catalog::city_of_shadows());
    let caves = g.add_card_to_hand(0, catalog::goblin_caves());
    cast(&mut g, 0, caves, Some(Target::Permanent(wastes)));
    assert_eq!(g.computed_permanent(sled).expect("sled").toughness, 1);
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    g.battlefield_find_mut(caves).expect("caves").attached_to = Some(mountain);
    assert_eq!(g.computed_permanent(sled).expect("sled").toughness, 3);
}

#[test]
fn goblin_shrine_burns_the_tribe_on_its_way_out() {
    let mut g = main_phase();
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    let sled = g.add_card_to_battlefield(0, catalog::goblin_rock_sled());
    let shrine = g.add_card_to_hand(0, catalog::goblin_shrine());
    cast(&mut g, 0, shrine, Some(Target::Permanent(mountain)));
    assert_eq!(g.computed_permanent(sled).expect("sled").power, 4);
    g.destroy_permanent(shrine, false, &mut vec![]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(sled).is_none(), "1 damage is lethal to a 3/1");
}

#[test]
fn worms_of_the_earth_stops_lands_by_every_route() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::worms_of_the_earth());
    let land = g.add_card_to_hand(1, catalog::mountain());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::PlayLand(land)).is_err());
    // …and the "put onto the battlefield" back door is shut too.
    let mut events = vec![];
    let ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
    g.move_card_to(
        land,
        &crabomination::effect::ZoneDest::Battlefield {
            controller: crabomination::effect::PlayerRef::Seat(1),
            tapped: false,
        },
        &ctx,
        &mut events,
    );
    assert!(g.battlefield_find(land).is_none());
}

#[test]
fn festival_calls_off_the_combat() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let festival = g.add_card_to_hand(0, catalog::festival());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    cast(&mut g, 0, festival, None);
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }])
            .is_err(),
    );
}

#[test]
fn deep_water_turns_every_land_blue_for_the_turn() {
    let mut g = main_phase();
    let water = g.add_card_to_battlefield(0, catalog::deep_water());
    let mountain = g.add_card_to_battlefield(0, catalog::mountain());
    activate(&mut g, 0, water, None);
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mountain,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 0);
}

#[test]
fn mind_bomb_hits_for_three_when_nobody_discards() {
    let mut g = main_phase();
    let bomb = g.add_card_to_hand(0, catalog::mind_bomb());
    cast(&mut g, 0, bomb, None);
    assert_eq!(g.players[0].life, 17);
    assert_eq!(g.players[1].life, 17);
}

#[test]
fn leviathan_pays_two_islands_to_attack() {
    let mut g = main_phase();
    let lev = g.add_card_to_battlefield(0, catalog::leviathan());
    g.clear_sickness(lev);
    g.battlefield_find_mut(lev).expect("leviathan").tapped = false;
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.declare_attackers(vec![Attack { attacker: lev, target: AttackTarget::Player(1) }])
            .is_err(),
        "no Islands, no attack",
    );
    let a = g.add_card_to_battlefield(0, catalog::island());
    let b = g.add_card_to_battlefield(0, catalog::island());
    g.declare_attackers(vec![Attack { attacker: lev, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

#[test]
fn season_of_the_witch_reaps_the_creatures_that_stayed_home() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::season_of_the_witch());
    let idle = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let swung = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(swung).expect("swung").attacked_this_turn = true;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(idle).is_none());
    assert!(g.battlefield_find(swung).is_some());
}

#[test]
fn psychic_allergy_bleeds_the_chosen_colour() {
    let mut g = main_phase();
    let allergy = g.add_card_to_hand(0, catalog::psychic_allergy());
    cast(&mut g, 0, allergy, None);
    g.battlefield_find_mut(allergy).expect("allergy").chosen_color = Some(Color::Green);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

#[test]
fn scarwood_bandits_take_the_artifact_when_the_toll_goes_unpaid() {
    let mut g = main_phase();
    let bandits = g.add_card_to_battlefield(0, catalog::scarwood_bandits());
    g.clear_sickness(bandits);
    let rock = g.add_card_to_battlefield(1, catalog::diabolic_machine());
    activate(&mut g, 0, bandits, Some(Target::Permanent(rock)));
    assert_eq!(g.battlefield_find(rock).expect("rock").controller, 0);
}

#[test]
fn cleansing_sweeps_the_lands_nobody_pays_for() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::mountain());
    let b = g.add_card_to_battlefield(1, catalog::mountain());
    let cleansing = g.add_card_to_hand(0, catalog::cleansing());
    cast(&mut g, 0, cleansing, None);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

#[test]
fn spitting_slug_hands_out_first_strike_when_the_toll_is_declined() {
    let mut g = main_phase();
    let slug = g.add_card_to_battlefield(1, catalog::spitting_slug());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    let evs = g.declare_blockers(vec![(slug, bear)]).expect("block");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    // Seat 1 can't pay {1}{G}, so the attacker gets first strike too.
    assert!(g.computed_permanent(bear).expect("bear").keywords.contains(&Keyword::FirstStrike));
}

#[test]
fn venom_kills_whatever_the_enchanted_creature_meets() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(0, catalog::goblin_rock_sled());
    g.clear_sickness(attacker);
    g.add_card_to_battlefield(1, catalog::mountain());
    g.active_player_idx = 1;
    let venom = g.add_card_to_hand(1, catalog::venom());
    cast(&mut g, 1, venom, Some(Target::Permanent(bear)));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    let evs = g.declare_blockers(vec![(bear, attacker)]).expect("block");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat {
        g.advance_step(Vec::new()).expect("advance");
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none());
}
