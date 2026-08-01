//! Prophecy (PCY), closing wave — the untapped-lands shell and the rares.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
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

fn end_step(g: &mut GameState) {
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(g);
}

fn upkeep(g: &mut GameState) {
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

/// The printed-keyword bodies.
#[test]
fn pcy4_keyword_bodies_carry_their_printed_keywords() {
    let cases: &[(fn() -> crabomination::card::CardDefinition, &[Keyword])] = &[
        (catalog::stormwatch_eagle, &[Keyword::Flying]),
        (catalog::troublesome_spirit, &[Keyword::Flying]),
        (catalog::windscouter, &[Keyword::Flying]),
        (catalog::vintara_elephant, &[Keyword::Trample]),
        (catalog::zerapa_minotaur, &[Keyword::FirstStrike]),
        (catalog::wall_of_vipers, &[Keyword::Defender]),
        (catalog::keldon_battlewagon, &[Keyword::Trample, Keyword::CantBlock]),
        (
            catalog::veteran_brawlers,
            &[Keyword::CantAttackIfDefenderHasUntappedLand, Keyword::CantBlockIfYouHaveUntappedLand],
        ),
    ];
    for (factory, expected) in cases {
        let def = factory();
        for kw in *expected {
            assert!(def.keywords.contains(kw), "{} is missing {kw:?}", def.name);
        }
    }
}

/// Spur Grappler is a 4/2 only while you're tapped out.
#[test]
fn spur_grappler_grows_while_tapped_out() {
    let mut g = two_player_game();
    let beast = g.add_card_to_battlefield(0, catalog::spur_grappler());
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    assert_eq!(g.computed_permanent(beast).unwrap().power, 2);
    g.battlefield_find_mut(land).unwrap().tapped = true;
    let cp = g.computed_permanent(beast).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2));
}

/// Vintara Snapper picks up shroud once every land is tapped.
#[test]
fn vintara_snapper_gains_shroud_while_tapped_out() {
    let mut g = two_player_game();
    let turtle = g.add_card_to_battlefield(0, catalog::vintara_snapper());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    assert!(!g.computed_permanent(turtle).unwrap().keywords.contains(&Keyword::Shroud));
    g.battlefield_find_mut(land).unwrap().tapped = true;
    assert!(g.computed_permanent(turtle).unwrap().keywords.contains(&Keyword::Shroud));
}

/// Well of Life pays out only on an end step with no untapped lands.
#[test]
fn well_of_life_needs_you_tapped_out() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::well_of_life());
    let land = g.add_card_to_battlefield(0, catalog::plains());
    end_step(&mut g);
    assert_eq!(g.players[0].life, 20, "an untapped land blanks it");
    g.battlefield_find_mut(land).unwrap().tapped = true;
    end_step(&mut g);
    assert_eq!(g.players[0].life, 22);
}

/// Well of Discovery draws on the same condition.
#[test]
fn well_of_discovery_draws_while_tapped_out() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::well_of_discovery());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    end_step(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1);
}

/// Vintara Elephant's trample can be switched off by the opponent.
#[test]
fn vintara_elephant_loses_trample_to_any_player() {
    let mut g = main_phase();
    let elephant = g.add_card_to_battlefield(0, catalog::vintara_elephant());
    activate(&mut g, 1, elephant, 0, None);
    assert!(!g.computed_permanent(elephant).unwrap().keywords.contains(&Keyword::Trample));
}

/// Zerapa Minotaur's first strike is likewise for sale.
#[test]
fn zerapa_minotaur_loses_first_strike_to_any_player() {
    let mut g = main_phase();
    let minotaur = g.add_card_to_battlefield(0, catalog::zerapa_minotaur());
    activate(&mut g, 1, minotaur, 0, None);
    assert!(!g.computed_permanent(minotaur).unwrap().keywords.contains(&Keyword::FirstStrike));
}

/// Wall of Vipers trades itself for the creature it's blocking.
#[test]
fn wall_of_vipers_trades_with_what_it_blocks() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_vipers());
    let ogre = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(ogre);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: ogre, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, ogre)])).expect("block");
    drain_stack(&mut g);
    activate(&mut g, 0, wall, 0, Some(Target::Permanent(ogre)));
    assert!(g.battlefield_find(ogre).is_none(), "the blocked attacker died");
    assert!(g.battlefield_find(wall).is_none(), "the Wall went with it");
}

/// Whipstitched Zombie walks off without its upkeep {B}.
#[test]
fn whipstitched_zombie_needs_its_upkeep_payment() {
    let mut g = main_phase();
    let zombie = g.add_card_to_battlefield(0, catalog::whipstitched_zombie());
    g.players[0].mana_pool.empty();
    upkeep(&mut g);
    assert!(g.battlefield_find(zombie).is_none());
}

/// Whip Sergeant rents haste out for {R}.
#[test]
fn whip_sergeant_grants_haste() {
    let mut g = main_phase();
    let sergeant = g.add_card_to_battlefield(0, catalog::whip_sergeant());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, sergeant, 0, Some(Target::Permanent(bear)));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
}

/// Vitalizing Wind is a team-wide +7/+7.
#[test]
fn vitalizing_wind_pumps_your_team() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wind = g.add_card_to_hand(0, catalog::vitalizing_wind());
    cast(&mut g, 0, wind, None);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 9);
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 2, "only yours");
}

/// Wild Might's second half lands when nobody buys it off.
#[test]
fn wild_might_gives_the_full_bonus_when_unpaid() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let might = g.add_card_to_hand(0, catalog::wild_might());
    cast(&mut g, 0, might, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
}

/// Withdraw bounces both creatures when the tax goes unpaid.
#[test]
fn withdraw_bounces_both_when_unpaid() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let withdraw = g.add_card_to_hand(0, catalog::withdraw());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: withdraw,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Steal Strength moves a point of stats between two creatures.
#[test]
fn steal_strength_pumps_one_and_shrinks_the_other() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::steal_strength());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3);
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 1);
}

/// Stormwatch Eagle buys itself back for a land.
#[test]
fn stormwatch_eagle_returns_itself_for_a_land() {
    let mut g = main_phase();
    let eagle = g.add_card_to_battlefield(0, catalog::stormwatch_eagle());
    g.add_card_to_battlefield(0, catalog::island());
    activate(&mut g, 0, eagle, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == eagle));
    assert!(!g.battlefield.iter().any(|c| c.definition.is_land()), "the land paid for it");
}

/// Trenching Steed eats a land to survive.
#[test]
fn trenching_steed_trades_a_land_for_toughness() {
    let mut g = main_phase();
    let steed = g.add_card_to_battlefield(0, catalog::trenching_steed());
    g.add_card_to_battlefield(0, catalog::plains());
    activate(&mut g, 0, steed, 0, None);
    assert_eq!(g.computed_permanent(steed).unwrap().toughness, 6);
}

/// Troubled Healer's land turns into a two-point shield.
#[test]
fn troubled_healer_prevents_two_damage() {
    let mut g = main_phase();
    let healer = g.add_card_to_battlefield(0, catalog::troubled_healer());
    g.add_card_to_battlefield(0, catalog::plains());
    activate(&mut g, 0, healer, 0, Some(Target::Player(0)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 19, "two of the three were prevented");
}

/// Troublesome Spirit taps you out every end step.
#[test]
fn troublesome_spirit_taps_your_lands() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::troublesome_spirit());
    let land = g.add_card_to_battlefield(0, catalog::island());
    end_step(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped);
}

/// Sword Dancer chips power off an attacker.
#[test]
fn sword_dancer_shrinks_an_attacker() {
    let mut g = two_player_game();
    let dancer = g.add_card_to_battlefield(0, catalog::sword_dancer());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }])
        .expect("attack");
    drain_stack(&mut g);
    activate(&mut g, 0, dancer, 0, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 1);
}

/// Squirrel Wrangler turns a land into two Squirrels.
#[test]
fn squirrel_wrangler_makes_squirrels() {
    let mut g = main_phase();
    let wrangler = g.add_card_to_battlefield(0, catalog::squirrel_wrangler());
    g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, 0, wrangler, 0, None);
    let squirrels = g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count();
    assert_eq!(squirrels, 2);
}

/// Squirrel Wrangler's second ability is a Squirrel anthem.
#[test]
fn squirrel_wrangler_pumps_squirrels() {
    let mut g = main_phase();
    let wrangler = g.add_card_to_battlefield(0, catalog::squirrel_wrangler());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, 0, wrangler, 0, None);
    activate(&mut g, 0, wrangler, 1, None);
    let squirrel = g.battlefield.iter().find(|c| c.definition.name == "Squirrel").unwrap().id;
    assert_eq!(g.computed_permanent(squirrel).unwrap().power, 2);
}

/// Thresher Beast bills the blocker's controller a land.
#[test]
fn thresher_beast_eats_a_defending_land() {
    let mut g = two_player_game();
    let beast = g.add_card_to_battlefield(0, catalog::thresher_beast());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::forest());
    g.clear_sickness(beast);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: beast, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, beast)])).expect("block");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.is_land()), "the defender lost their land");
}

/// Thrive puts a counter on each of X creatures.
#[test]
fn thrive_counters_scale_with_x() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let thrive = g.add_card_to_hand(0, catalog::thrive());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: thrive,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    for id in [a, b] {
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }
}

/// Alexi bounces X creatures for two cards.
#[test]
fn alexi_bounces_x_creatures() {
    let mut g = main_phase();
    let alexi = g.add_card_to_battlefield(0, catalog::alexi_zephyr_mage());
    g.clear_sickness(alexi);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: alexi,
        ability_index: 0,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Wing Storm bills each player two per flier they control.
#[test]
fn wing_storm_scales_with_each_players_fliers() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::shivan_dragon());
    g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let storm = g.add_card_to_hand(0, catalog::wing_storm());
    cast(&mut g, 0, storm, None);
    assert_eq!(g.players[1].life, 16);
    assert_eq!(g.players[0].life, 20, "you control no fliers");
}

/// Wintermoon Mesa taps two lands on the way out.
#[test]
fn wintermoon_mesa_taps_two_lands() {
    let mut g = main_phase();
    let mesa = g.add_card_to_battlefield(0, catalog::wintermoon_mesa());
    let a = g.add_card_to_battlefield(1, catalog::island());
    let b = g.add_card_to_battlefield(1, catalog::island());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mesa,
        ability_index: 1,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
    assert!(g.battlefield_find(mesa).is_none(), "it sacrificed itself");
}

/// Windscouter goes home at end of combat after attacking.
#[test]
fn windscouter_returns_after_attacking() {
    let mut g = two_player_game();
    let scout = g.add_card_to_battlefield(0, catalog::windscouter());
    g.clear_sickness(scout);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: scout, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == scout));
}

/// Denying Wind exiles seven cards out of a library.
#[test]
fn denying_wind_exiles_seven() {
    let mut g = main_phase();
    for _ in 0..8 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let picks: Vec<_> = g.players[1].library.iter().take(7).map(|c| c.id).collect();
    let wind = g.add_card_to_hand(0, catalog::denying_wind());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(
        picks.iter().map(|id| crabomination::decision::DecisionAnswer::Search(Some(*id))),
    ));
    cast(&mut g, 0, wind, Some(Target::Player(1)));
    assert_eq!(g.exile.iter().filter(|c| c.owner == 1).count(), 7);
}

/// Forgotten Harvest turns a graveyard land into a +1/+1 counter.
#[test]
fn forgotten_harvest_trades_a_land_for_a_counter() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::forgotten_harvest());
    g.add_card_to_graveyard(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    upkeep(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.exile.iter().any(|c| c.definition.name == "Forest"), "the land paid for it");
}

/// Sunken Field turns the enchanted land into a Mana Leak battery.
#[test]
fn sunken_field_taxes_a_spell() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::island());
    let field = g.add_card_to_hand(0, catalog::sunken_field());
    cast(&mut g, 0, field, Some(Target::Permanent(land)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    // Index 0 is the Island's own mana ability; the Aura grants index 1.
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    g.players[1].mana_pool.empty();
    activate(&mut g, 0, land, 1, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, 20, "countered by the enchanted land");
}

/// Verdant Field's land pumps a creature.
#[test]
fn verdant_field_pumps_from_the_land() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let field = g.add_card_to_hand(0, catalog::verdant_field());
    cast(&mut g, 0, field, Some(Target::Permanent(land)));
    activate(&mut g, 0, land, 1, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Keldon Battlewagon borrows the power of a creature it taps.
#[test]
fn keldon_battlewagon_borrows_power() {
    let mut g = main_phase();
    let wagon = g.add_card_to_battlefield(0, catalog::keldon_battlewagon());
    g.add_card_to_battlefield(0, catalog::shivan_dragon());
    activate(&mut g, 0, wagon, 0, None);
    assert_eq!(g.computed_permanent(wagon).unwrap().power, 5, "0 + the Dragon's 5");
}

/// Keldon Battlewagon sacrifices itself at end of combat after attacking.
#[test]
fn keldon_battlewagon_dies_after_attacking() {
    let mut g = two_player_game();
    let wagon = g.add_card_to_battlefield(0, catalog::keldon_battlewagon());
    g.clear_sickness(wagon);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: wagon, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(wagon).is_none());
}

/// Fickle Efreet flips at end of combat; a lost flip hands it over.
#[test]
fn fickle_efreet_changes_hands_on_a_lost_flip() {
    let mut g = two_player_game();
    let efreet = g.add_card_to_battlefield(0, catalog::fickle_efreet());
    g.clear_sickness(efreet);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(false),
    ]));
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: efreet, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(efreet).unwrap().controller, 1);
}
