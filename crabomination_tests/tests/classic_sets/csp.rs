//! Coldsnap (CSP) — `catalog::sets::csp`.

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword, Supertype};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(
    g: &mut GameState,
    id: CardId,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

fn activate(
    g: &mut GameState,
    id: CardId,
    index: usize,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

/// The snow tapland cycle enters tapped and is itself snow.
#[test]
fn snow_taplands_enter_tapped() {
    for land in [
        catalog::arctic_flats as fn() -> CardDefinition,
        catalog::boreal_shelf,
        catalog::frost_marsh,
        catalog::highland_weald,
    ] {
        let def = land();
        assert!(def.supertypes.contains(&Supertype::Snow), "{} is snow", def.name);
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, land());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::PlayLand(id)).expect("play");
        assert!(g.battlefield_find(id).unwrap().tapped, "{} entered tapped", def.name);
    }
}

/// The {S} pumps run off snow mana, and Boreal Centaur's is once a turn.
#[test]
fn boreal_centaur_pumps_once_per_turn_off_snow() {
    let mut g = two_player_game();
    let centaur = ready(&mut g, 0, catalog::boreal_centaur());
    let land = ready(&mut g, 0, catalog::arctic_flats());
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.players[0].mana_pool.add_snow(Color::Green, 2);
    activate(&mut g, centaur, 0, None).expect("first pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(centaur).unwrap().power, 3);
    assert!(activate(&mut g, centaur, 0, None).is_err(), "only once each turn");
}

/// Frost Raptor buys shroud with two snow pips.
#[test]
fn frost_raptor_buys_shroud() {
    let mut g = two_player_game();
    let raptor = ready(&mut g, 0, catalog::frost_raptor());
    g.players[0].mana_pool.add_snow(Color::Blue, 2);
    activate(&mut g, raptor, 0, None).expect("shroud");
    drain_stack(&mut g);
    assert!(g.computed_permanent(raptor).unwrap().keywords.contains(&Keyword::Shroud));
}

/// Chill to the Bone can't touch a snow creature.
#[test]
fn chill_to_the_bone_spares_snow() {
    let mut g = two_player_game();
    let snow = ready(&mut g, 1, catalog::chilling_shade());
    let plain = ready(&mut g, 1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::chill_to_the_bone());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert!(cast(&mut g, spell, Some(Target::Permanent(snow))).is_err(), "snow is off-limits");
    cast(&mut g, spell, Some(Target::Permanent(plain))).expect("nonsnow is fair game");
    drain_stack(&mut g);
    assert!(g.battlefield_find(plain).is_none());
}

/// Cryoclasm blows up a Plains and burns its controller.
#[test]
fn cryoclasm_burns_the_land_owner() {
    let mut g = two_player_game();
    let plains = g.add_card_to_battlefield(1, catalog::plains());
    let spell = g.add_card_to_hand(0, catalog::cryoclasm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, Some(Target::Permanent(plains))).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(plains).is_none());
    assert_eq!(g.players[1].life, 17);
}

/// Into the North fetches a snow land tapped.
#[test]
fn into_the_north_fetches_a_snow_land() {
    let mut g = two_player_game();
    let shelf = g.add_card_to_library(0, catalog::boreal_shelf());
    g.add_card_to_library(0, catalog::plains());
    let spell = g.add_card_to_hand(0, catalog::into_the_north());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    let land = g.battlefield_find(shelf).expect("fetched");
    assert!(land.tapped, "tapped, per the printed text");
}

/// Bull Aurochs swells for each other attacking Aurochs.
#[test]
fn bull_aurochs_counts_the_herd() {
    let mut g = two_player_game();
    let a = ready(&mut g, 0, catalog::bull_aurochs());
    let b = ready(&mut g, 0, catalog::bull_aurochs());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(a).unwrap().power, 3, "one other Aurochs");
}

/// Earthen Goo grows one age counter at a time.
#[test]
fn earthen_goo_grows_with_its_upkeep() {
    let mut g = two_player_game();
    let goo = g.add_card_to_battlefield(0, catalog::earthen_goo());
    g.battlefield_find_mut(goo).unwrap().add_counters(CounterType::Age, 2);
    let cp = g.computed_permanent(goo).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Arctic Nishoba pays two life per age counter when it dies.
#[test]
fn arctic_nishoba_pays_out_on_death() {
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::arctic_nishoba());
    g.battlefield_find_mut(cat).unwrap().add_counters(CounterType::Age, 3);
    let evs = g.remove_to_graveyard_with_triggers(cat);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 26, "2 per age counter");
}

/// Jötun Owl Keeper leaves one Bird per age counter.
#[test]
fn jotun_owl_keeper_leaves_birds() {
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(0, catalog::jotun_owl_keeper());
    g.battlefield_find_mut(giant).unwrap().add_counters(CounterType::Age, 2);
    let evs = g.remove_to_graveyard_with_triggers(giant);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Bird").count(), 2);
}

/// Feast of Flesh scales with the copies already in graveyards.
#[test]
fn feast_of_flesh_counts_its_own_copies() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::feast_of_flesh());
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::feast_of_flesh());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, spell, Some(Target::Permanent(victim))).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22, "1 + one copy in a graveyard");
    assert!(g.battlefield_find(victim).is_none(), "2 damage killed the 2/2");
}

/// Gelid Shackles locks its host out of blocking and tapping.
#[test]
fn gelid_shackles_locks_the_host() {
    let mut g = two_player_game();
    let host = ready(&mut g, 1, catalog::prodigal_sorcerer());
    let aura = g.add_card_to_hand(0, catalog::gelid_shackles());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, aura, Some(Target::Permanent(host))).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(host).unwrap();
    assert!(cp.keywords.contains(&Keyword::CantBlock));
    assert!(cp.keywords.contains(&Keyword::CantActivateTapAbilities));
}

/// Freyalise's Radiance keeps snow permanents tapped.
#[test]
fn freyalises_radiance_freezes_snow() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::freyalises_radiance());
    let snow = ready(&mut g, 0, catalog::chilling_shade());
    let plain = ready(&mut g, 0, catalog::grizzly_bears());
    for id in [snow, plain] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(snow).unwrap().tapped, "snow stays down");
    assert!(!g.battlefield_find(plain).unwrap().tapped);
}

/// Kjeldoran Javelineer throws for its age-counter count.
#[test]
fn kjeldoran_javelineer_throws_its_age() {
    let mut g = two_player_game();
    let jav = ready(&mut g, 0, catalog::kjeldoran_javelineer());
    g.battlefield_find_mut(jav).unwrap().add_counters(CounterType::Age, 2);
    let attacker = ready(&mut g, 1, catalog::serra_angel());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    activate(&mut g, jav, 0, Some(Target::Permanent(attacker))).expect("throw");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(attacker).unwrap().damage, 2);
}

/// Adarkar Windform grounds a flier for the turn.
#[test]
fn adarkar_windform_grounds_a_flier() {
    let mut g = two_player_game();
    let wind = ready(&mut g, 0, catalog::adarkar_windform());
    let flier = ready(&mut g, 1, catalog::serra_angel());
    g.players[0].mana_pool.add_snow(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, wind, 0, Some(Target::Permanent(flier))).expect("ground it");
    drain_stack(&mut g);
    assert!(!g.computed_permanent(flier).unwrap().keywords.contains(&Keyword::Flying));
}

/// Karplusan Strider can't be targeted by blue or black.
#[test]
fn karplusan_strider_dodges_blue_and_black() {
    let mut g = two_player_game();
    let strider = ready(&mut g, 1, catalog::karplusan_strider());
    let kill = g.add_card_to_hand(0, catalog::chill_to_the_bone());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert!(cast(&mut g, kill, Some(Target::Permanent(strider))).is_err());
}

/// Gutless Ghoul trades a creature for two life.
#[test]
fn gutless_ghoul_eats_a_creature() {
    let mut g = two_player_game();
    let ghoul = ready(&mut g, 0, catalog::gutless_ghoul());
    let food = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, ghoul, 0, None).expect("eat");
    drain_stack(&mut g);
    assert!(g.battlefield_find(food).is_none());
    assert_eq!(g.players[0].life, 22);
}

/// Disciple of Tevesh Szat shrinks a creature, then shrinks one to death.
#[test]
fn disciple_of_tevesh_szat_shrinks_twice() {
    let mut g = two_player_game();
    let disciple = ready(&mut g, 0, catalog::disciple_of_tevesh_szat());
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    activate(&mut g, disciple, 0, Some(Target::Permanent(victim))).expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(victim).unwrap().power, 1);
    g.battlefield_find_mut(disciple).unwrap().tapped = false;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    activate(&mut g, disciple, 1, Some(Target::Permanent(victim))).expect("sac");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
}

/// Kjeldoran Gargoyle turns its damage into life.
#[test]
fn kjeldoran_gargoyle_drinks_its_damage() {
    let mut g = two_player_game();
    let gargoyle = ready(&mut g, 0, catalog::kjeldoran_gargoyle());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: gargoyle,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, 17);
    assert_eq!(g.players[0].life, 23, "3 damage, 3 life");
}

/// Kjeldoran War Cry scales with the copies already binned.
#[test]
fn kjeldoran_war_cry_counts_its_copies() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::kjeldoran_war_cry());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::kjeldoran_war_cry());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+2");
}

/// Balduvian Rage pumps an attacker now and draws next upkeep.
#[test]
fn balduvian_rage_pumps_and_replaces_itself() {
    let mut g = two_player_game();
    let attacker = ready(&mut g, 0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let spell = g.add_card_to_hand(0, catalog::balduvian_rage());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(attacker)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(attacker).unwrap().power, 4);
}

/// Drelnoch and Karplusan Wolverine both punish a block.
#[test]
fn blocked_triggers_fire() {
    for def in
        [catalog::drelnoch as fn() -> CardDefinition, catalog::karplusan_wolverine]
    {
        let mut g = two_player_game();
        let attacker = ready(&mut g, 0, def());
        let blocker = ready(&mut g, 1, catalog::grizzly_bears());
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::plains());
        }
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
            drain_stack(&mut g);
        }
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(true),
        ]));
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
        drain_stack(&mut g);
        let name = def().name;
        if name == "Drelnoch" {
            assert_eq!(g.players[0].hand.len(), 2, "{name} drew two");
        } else {
            assert!(
                g.players[1].life < 20 || g.battlefield_find(blocker).unwrap().damage > 0,
                "{name} pinged something"
            );
        }
    }
}
