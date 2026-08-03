//! Legends (LEG) wave 3 — landwalk hosers, the "becomes [color]" cycle and
//! the utility legends (`catalog::sets::leg2`).

use crabomination::card::{CardId, CounterType, Keyword, LandType, Supertype};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
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

/// Try to block `attacker` with `blocker`; returns whether the block was legal.
fn try_block(g: &mut GameState, attacker: CardId, blocker: CardId) -> bool {
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).is_ok()
}

/// CR 509.1b — Deadfall blanks forestwalk for everyone.
#[test]
fn deadfall_turns_off_forestwalk() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::forest());
    let cat = g.add_card_to_battlefield(0, catalog::cat_warriors()); // forestwalk
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(!try_block(&mut g, cat, blocker), "forestwalk is live");

    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::forest());
    g.add_card_to_battlefield(0, catalog::deadfall());
    let cat = g.add_card_to_battlefield(0, catalog::cat_warriors());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(try_block(&mut g, cat, blocker), "Deadfall blanks it");
}

/// The rest of the hoser cycle names the right land type.
#[test]
fn the_landwalk_hosers_cover_the_cycle() {
    use crabomination::effect::StaticEffect;
    for (def, lt) in [
        (catalog::great_wall(), LandType::Plains),
        (catalog::quagmire(), LandType::Swamp),
        (catalog::crevasse(), LandType::Mountain),
        (catalog::gosta_dirk(), LandType::Island),
    ] {
        assert!(
            def.static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::LandwalkIgnored(t) if t == lt)),
            "{} hoses {lt:?}",
            def.name
        );
    }
    assert_eq!(catalog::lord_magnus().static_abilities.len(), 2, "plains + forest");
}

/// Hammerheim strips every landwalk the target has.
#[test]
fn hammerheim_strips_landwalk() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::forest());
    let land = g.add_card_to_battlefield(0, catalog::hammerheim());
    let cat = g.add_card_to_battlefield(0, catalog::cat_warriors());
    activate(&mut g, 0, land, 1, Some(Target::Permanent(cat)));
    assert!(
        !g.computed_permanent(cat)
            .unwrap()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::Landwalk(_)))
    );
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(try_block(&mut g, cat, blocker));
}




/// Indestructible Aura is a one-creature fog.
#[test]
fn indestructible_aura_shields_the_creature() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::indestructible_aura());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0);
}

/// Acid Rain only eats Forests.
#[test]
fn acid_rain_destroys_forests_only() {
    let mut g = main_phase();
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let island = g.add_card_to_battlefield(1, catalog::island());
    let rain = g.add_card_to_hand(0, catalog::acid_rain());
    cast(&mut g, 0, rain, None);
    assert!(g.battlefield_find(forest).is_none());
    assert!(g.battlefield_find(island).is_some());
}

/// Cleanse sweeps black creatures and leaves the rest.
#[test]
fn cleanse_sweeps_black_creatures() {
    let mut g = main_phase();
    let black = g.add_card_to_battlefield(1, catalog::lost_soul());
    let green = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::cleanse());
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield_find(black).is_none());
    assert!(g.battlefield_find(green).is_some());
}

/// Active Volcano's first mode kills a blue permanent.
#[test]
fn active_volcano_kills_a_blue_permanent() {
    let mut g = main_phase();
    let merfolk = g.add_card_to_battlefield(1, catalog::devouring_deep());
    let spell = g.add_card_to_hand(0, catalog::active_volcano());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(merfolk)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(merfolk).is_none());
}

/// Flash Flood's bounce mode returns a Mountain.
#[test]
fn flash_flood_bounces_a_mountain() {
    let mut g = main_phase();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let spell = g.add_card_to_hand(0, catalog::flash_flood());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mountain).is_none());
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Jovial Evil burns for twice the white creature count.
#[test]
fn jovial_evil_doubles_the_white_creature_count() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_battlefield(1, catalog::keepers_of_the_faith());
    }
    let spell = g.add_card_to_hand(0, catalog::jovial_evil());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 16);
}

/// Moat grounds the ground.
#[test]
fn moat_stops_nonfliers_attacking() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::moat());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(1),
        }]))
        .is_err()
    );
}

/// Gravity Sphere grounds fliers globally.
#[test]
fn gravity_sphere_strips_flying() {
    let mut g = main_phase();
    let bees = g.add_card_to_battlefield(0, catalog::killer_bees());
    assert!(g.computed_permanent(bees).unwrap().keywords.contains(&Keyword::Flying));
    g.add_card_to_battlefield(1, catalog::gravity_sphere());
    assert!(!g.computed_permanent(bees).unwrap().keywords.contains(&Keyword::Flying));
    assert!(catalog::gravity_sphere().supertypes.contains(&Supertype::World));
}

/// Greed pays life for cards.
#[test]
fn greed_draws_for_two_life() {
    let mut g = main_phase();
    let greed = g.add_card_to_battlefield(0, catalog::greed());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    activate(&mut g, 0, greed, 0, None);
    assert_eq!(g.players[0].hand.len(), before + 1);
    assert_eq!(g.players[0].life, 18);
}

/// Planar Gate and Mana Matrix each shave {2} off their spell class.
#[test]
fn the_cost_reducers_shave_two() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::planar_gate());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G}
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{1}{G} minus {2} is {G}");
    assert_eq!(catalog::mana_matrix().static_abilities.len(), 1);
}

/// Arena of the Ancients taps the legends and keeps them down.
#[test]
fn arena_of_the_ancients_locks_legends_down() {
    let mut g = main_phase();
    let legend = g.add_card_to_battlefield(0, catalog::lady_orca());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let arena = g.add_card_to_hand(0, catalog::arena_of_the_ancients());
    cast(&mut g, 0, arena, None);
    assert!(g.battlefield_find(legend).unwrap().tapped, "ETB taps legends");
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    while g.step != TurnStep::Upkeep {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.active_player_idx, 0);
    assert!(g.battlefield_find(legend).unwrap().tapped, "still locked");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "the bear untapped");
}

/// Dakkon Blackblade is as big as your lands.
#[test]
fn dakkon_blackblade_counts_lands() {
    let mut g = main_phase();
    let dakkon = g.add_card_to_battlefield(0, catalog::dakkon_blackblade());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    let cp = g.computed_permanent(dakkon).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Fallen Angel eats a creature for a pump.
#[test]
fn fallen_angel_eats_a_creature() {
    let mut g = main_phase();
    let angel = g.add_card_to_battlefield(0, catalog::fallen_angel());
    let snack = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, angel, 0, None);
    assert!(g.battlefield_find(snack).is_none());
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 4));
}

/// Cyclopean Mummy exiles itself out of the graveyard.
#[test]
fn cyclopean_mummy_exiles_itself() {
    let mut g = main_phase();
    let mummy = g.add_card_to_battlefield(0, catalog::cyclopean_mummy());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(mummy)));
    assert!(g.exile.iter().any(|c| c.definition.name == "Cyclopean Mummy"));
    assert!(!g.players[0].graveyard.iter().any(|c| c.definition.name == "Cyclopean Mummy"));
}

/// Crimson Manticore only shoots creatures in combat.
#[test]
fn crimson_manticore_needs_a_combatant() {
    let mut g = main_phase();
    let manticore = g.add_card_to_battlefield(0, catalog::crimson_manticore());
    g.clear_sickness(manticore);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: manticore,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the bear isn't attacking or blocking"
    );
}

/// Ramses Overdark assassinates anything wearing an Aura.
#[test]
fn ramses_overdark_kills_the_enchanted() {
    let mut g = main_phase();
    let ramses = g.add_card_to_battlefield(0, catalog::ramses_overdark());
    g.clear_sickness(ramses);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::divine_transformation());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "+3/+3");
    activate(&mut g, 0, ramses, 0, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none());
}

/// Elven Riders only gets stopped by Walls and fliers.
#[test]
fn elven_riders_dodges_ground_creatures() {
    let mut g = main_phase();
    let riders = g.add_card_to_battlefield(0, catalog::elven_riders());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(!try_block(&mut g, riders, ground));

    let mut g = main_phase();
    let riders = g.add_card_to_battlefield(0, catalog::elven_riders());
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_earth());
    assert!(try_block(&mut g, riders, wall));
}

/// Osai Vultures banks a carrion counter when something died.
#[test]
fn osai_vultures_banks_carrion_counters() {
    let mut g = main_phase();
    let vultures = g.add_card_to_battlefield(0, catalog::osai_vultures());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bear)));
    g.step = TurnStep::EndCombat;
    while g.step != TurnStep::End {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(vultures).unwrap().counter_count(CounterType::Carrion), 1);
}

/// Boris Devilboon mints a Minor Demon.
#[test]
fn boris_devilboon_mints_a_minor_demon() {
    let mut g = main_phase();
    let boris = g.add_card_to_battlefield(0, catalog::boris_devilboon());
    g.clear_sickness(boris);
    activate(&mut g, 0, boris, 0, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Minor Demon"));
}

/// Gwendlyn Di Corci only works on your own turn.
#[test]
fn gwendlyn_di_corci_is_your_turn_only() {
    let mut g = main_phase();
    let gwendlyn = g.add_card_to_battlefield(0, catalog::gwendlyn_di_corci());
    g.clear_sickness(gwendlyn);
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: gwendlyn,
            ability_index: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err()
    );
    g.active_player_idx = 0;
    activate(&mut g, 0, gwendlyn, 0, Some(Target::Player(1)));
    assert!(g.players[1].hand.is_empty());
}

/// Psionic Entity pays 3 of its own to deal 2.
#[test]
fn psionic_entity_hurts_itself() {
    let mut g = main_phase();
    let entity = g.add_card_to_battlefield(0, catalog::psionic_entity());
    g.clear_sickness(entity);
    activate(&mut g, 0, entity, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 18);
    assert!(g.battlefield_find(entity).is_none(), "a 2/2 taking 3 dies");
}

/// The wave's plain bodies ship with their printed stats.
#[test]
fn leg2_bodies_have_their_printed_stats() {
    for (def, p, t) in [
        (catalog::barktooth_warbeard(), 6, 5),
        (catalog::jasmine_boreal(), 4, 5),
        (catalog::jedit_ojanen(), 5, 5),
        (catalog::jerrard_of_the_closed_fist(), 6, 5),
        (catalog::kasimir_the_lone_wolf(), 5, 3),
        (catalog::lady_orca(), 7, 4),
        (catalog::ramirez_depietro(), 4, 3),
        (catalog::keepers_of_the_faith(), 2, 3),
        (catalog::moss_monster(), 3, 6),
        (catalog::raging_bull(), 2, 2),
        (catalog::lost_soul(), 2, 1),
        (catalog::hunding_gjornersen(), 5, 4),
        (catalog::marhault_elsdragon(), 4, 6),
        (catalog::aerathi_berserker(), 2, 4),
        (catalog::pavel_maliki(), 5, 3),
        (catalog::princess_lucrezia(), 5, 4),
        (catalog::lady_caleria(), 3, 6),
        (catalog::kei_takahashi(), 2, 2),
        (catalog::ragnar(), 2, 2),
        (catalog::pradesh_gypsies(), 1, 1),
        (catalog::pixie_queen(), 1, 1),
        (catalog::hyperion_blacksmith(), 2, 2),
        (catalog::jacques_le_vert(), 3, 2),
        (catalog::gosta_dirk(), 4, 4),
        (catalog::lord_magnus(), 4, 3),
    ] {
        assert_eq!((def.power, def.toughness), (p, t), "{}", def.name);
    }
}

/// Rampage and the legendary supertype ride along where they should.
#[test]
fn leg2_keywords_and_supertypes() {
    assert!(catalog::aerathi_berserker().keywords.contains(&Keyword::Rampage(3)));
    assert!(catalog::hunding_gjornersen().keywords.contains(&Keyword::Rampage(1)));
    assert!(catalog::lost_soul().keywords.contains(&Keyword::Landwalk(LandType::Swamp)));
    assert!(catalog::ramirez_depietro().supertypes.contains(&Supertype::Legendary));
    assert!(!catalog::keepers_of_the_faith().supertypes.contains(&Supertype::Legendary));
}

/// Jacques le Vert only pumps green creatures; Fortified Area only Walls.
#[test]
fn the_filtered_anthems_only_hit_their_filter() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::jacques_le_vert());
    let green = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let white = g.add_card_to_battlefield(0, catalog::keepers_of_the_faith());
    assert_eq!(g.computed_permanent(green).unwrap().toughness, 4);
    assert_eq!(g.computed_permanent(white).unwrap().toughness, 3);

    g.add_card_to_battlefield(0, catalog::fortified_area());
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_earth());
    let cp = g.computed_permanent(wall).unwrap();
    assert_eq!(cp.power, 1);
    assert!(cp.keywords.contains(&Keyword::Banding));
}

/// Kei Takahashi's shield eats the first 2 damage.
#[test]
fn kei_takahashi_prevents_two() {
    let mut g = main_phase();
    let kei = g.add_card_to_battlefield(0, catalog::kei_takahashi());
    g.clear_sickness(kei);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, kei, 0, Some(Target::Permanent(bear)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1, "3 minus the 2 prevented");
}

/// Pradesh Gypsies shaves power; Pixie Queen hands out flying.
#[test]
fn pradesh_gypsies_and_pixie_queen() {
    let mut g = main_phase();
    let gypsies = g.add_card_to_battlefield(0, catalog::pradesh_gypsies());
    g.clear_sickness(gypsies);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, gypsies, 0, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 0);

    let queen = g.add_card_to_battlefield(0, catalog::pixie_queen());
    g.clear_sickness(queen);
    activate(&mut g, 0, queen, 0, Some(Target::Permanent(bear)));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
}

/// Ragnar regenerates; Killer Bees grows.
#[test]
fn ragnar_regenerates_and_killer_bees_grow() {
    let mut g = main_phase();
    let ragnar = g.add_card_to_battlefield(0, catalog::ragnar());
    g.clear_sickness(ragnar);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, ragnar, 0, Some(Target::Permanent(bear)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_some(), "the shield ate it");

    let bees = g.add_card_to_battlefield(0, catalog::killer_bees());
    activate(&mut g, 0, bees, 0, None);
    let cp = g.computed_permanent(bees).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 2));
}

/// Hyperion Blacksmith jams an opponent's artifact.
#[test]
fn hyperion_blacksmith_taps_an_opposing_artifact() {
    let mut g = main_phase();
    let smith = g.add_card_to_battlefield(0, catalog::hyperion_blacksmith());
    g.clear_sickness(smith);
    let rock = g.add_card_to_battlefield(1, catalog::sol_ring());
    activate(&mut g, 0, smith, 0, Some(Target::Permanent(rock)));
    assert!(g.battlefield_find(rock).unwrap().tapped);
}
