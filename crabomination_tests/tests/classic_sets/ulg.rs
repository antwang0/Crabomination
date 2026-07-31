//! Urza's Legacy (ULG) gap closure.

use crabomination::card::{CardType, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
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

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Every ULG factory is registered under its printed name.
#[test]
fn ulg_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::about_face as fn() -> crabomination::card::CardDefinition,
        catalog::beast_of_burden,
        catalog::engineered_plague,
        catalog::intervene,
        catalog::iron_maiden,
        catalog::molten_hydra,
        catalog::multani_maro_sorcerer,
        catalog::phyrexian_plaguelord,
        catalog::planar_collapse,
        catalog::purify,
        catalog::radiant_archangel,
        catalog::second_chance,
        catalog::thran_lens,
        catalog::viashino_cutthroat,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// The cycling cycle all carry Cycling {2}.
#[test]
fn ulg_cycling_cards_carry_cycling_two() {
    for f in [
        catalog::bloated_toad as fn() -> crabomination::card::CardDefinition,
        catalog::darkwatch_elves,
        catalog::iron_will,
        catalog::radiants_judgment,
        catalog::swat,
        catalog::rebuild,
    ] {
        let def = f();
        let cost = def
            .keywords
            .iter()
            .find_map(|k| match k {
                Keyword::Cycling(c) => Some(c.cmc()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{} has no cycling", def.name));
        assert_eq!(cost, 2, "{}", def.name);
    }
}

/// About Face swaps power and toughness for the turn.
#[test]
fn about_face_switches_power_and_toughness() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::yavimaya_wurm());
    let spell = g.add_card_to_hand(0, catalog::about_face());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, spell, Some(Target::Permanent(wurm)));
    let cp = g.computed_permanent(wurm).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 6), "the 6/4 flipped");
}

/// Intervene only counters spells that target a creature.
#[test]
fn intervene_counters_a_spell_that_targets_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bolt");
    let counter = g.add_card_to_hand(0, catalog::intervene());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    cast(&mut g, counter, Some(Target::Permanent(bolt)));
    assert!(g.battlefield_find(bear).is_some(), "the Bolt was countered");
}

/// A spell aimed at a player isn't a legal Intervene target.
#[test]
fn intervene_cant_counter_a_spell_aimed_at_a_player() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bolt");
    let counter = g.add_card_to_hand(0, catalog::intervene());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: counter,
            target: Some(Target::Permanent(bolt)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the Bolt targets a player, not a creature"
    );
}

/// Beast of Burden counts every creature on the battlefield, both sides.
#[test]
fn beast_of_burden_counts_all_creatures() {
    let mut g = two_player_game();
    let beast = g.add_card_to_battlefield(0, catalog::beast_of_burden());
    assert_eq!(g.computed_permanent(beast).unwrap().power, 1, "itself");
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(beast).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Multani reads every player's hand.
#[test]
fn multani_counts_every_hand() {
    let mut g = two_player_game();
    let multani = g.add_card_to_battlefield(0, catalog::multani_maro_sorcerer());
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::forest());
    }
    let cp = g.computed_permanent(multani).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(cp.keywords.contains(&Keyword::Shroud));
}

/// Radiant counts other flyers on either side, not her own flying.
#[test]
fn radiant_grows_with_other_flyers() {
    let mut g = two_player_game();
    let radiant = g.add_card_to_battlefield(0, catalog::radiant_archangel());
    assert_eq!(g.computed_permanent(radiant).unwrap().power, 3, "alone");
    g.add_card_to_battlefield(1, catalog::weatherseed_faeries());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(radiant).unwrap().power, 4, "only the flyer counts");
}

/// Engineered Plague shrinks the chosen type on both sides of the table.
#[test]
fn engineered_plague_hits_every_players_chosen_type() {
    let mut g = two_player_game();
    let plague = g.add_card_to_battlefield(0, catalog::engineered_plague());
    g.battlefield_find_mut(plague).unwrap().chosen_creature_type = Some(CreatureType::Insect);
    let mine = g.add_card_to_battlefield(0, catalog::giant_cockroach());
    let theirs = g.add_card_to_battlefield(1, catalog::giant_cockroach());
    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for id in [mine, theirs] {
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 1));
    }
    assert_eq!(g.computed_permanent(other).unwrap().power, 2, "non-Insects are untouched");
}

/// Knighthood and Levitation are one-line team anthems.
#[test]
fn knighthood_and_levitation_grant_their_keyword() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::knighthood());
    g.add_card_to_battlefield(0, catalog::levitation());
    let cp = g.computed_permanent(mine).unwrap();
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
    assert!(cp.keywords.contains(&Keyword::Flying));
    assert!(!g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::Flying));
}

/// The Phyrexian Carrier cycle shrinks by its printed amount and eats itself.
#[test]
fn phyrexian_carriers_shrink_by_their_size() {
    for (f, n) in [
        (catalog::phyrexian_denouncer as fn() -> crabomination::card::CardDefinition, 1),
        (catalog::phyrexian_debaser, 2),
        (catalog::phyrexian_defiler, 3),
        (catalog::phyrexian_plaguelord, 4),
    ] {
        let mut g = two_player_game();
        let carrier = g.add_card_to_battlefield(0, f());
        g.battlefield_find_mut(carrier).unwrap().summoning_sick = false;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        activate(&mut g, carrier, 0, Some(Target::Permanent(bear)));
        assert!(g.battlefield_find(carrier).is_none(), "sacrificed as a cost");
        match g.computed_permanent(bear) {
            Some(cp) => assert_eq!((cp.power, cp.toughness), (2 - n, 2 - n)),
            None => assert!(n >= 2, "only -2/-2 or worse kills a 2/2"),
        }
    }
}

/// Plaguelord's second ability is a free sacrifice outlet.
#[test]
fn phyrexian_plaguelord_sacrifices_for_a_minus_one() {
    let mut g = two_player_game();
    let lord = g.add_card_to_battlefield(0, catalog::phyrexian_plaguelord());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, lord, 1, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(fodder).is_none(), "the Bears were eaten");
    assert!(g.battlefield_find(lord).is_some(), "the Plaguelord stayed");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
}

/// Ghitu Fire-Eater trades itself for damage equal to its power.
#[test]
fn ghitu_fire_eater_burns_for_its_power() {
    let mut g = two_player_game();
    let eater = g.add_card_to_battlefield(0, catalog::ghitu_fire_eater());
    g.battlefield_find_mut(eater).unwrap().summoning_sick = false;
    let life = g.players[1].life;
    activate(&mut g, eater, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 2);
}

/// Goblin Medics pings whenever it becomes tapped.
#[test]
fn goblin_medics_pings_on_tap() {
    let mut g = two_player_game();
    let medics = g.add_card_to_battlefield(0, catalog::goblin_medics());
    let life = g.players[1].life;
    g.battlefield_find_mut(medics).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped {
        card_id: medics,
        actor: None,
        as_attacker: false,
    }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
}

/// Delusions of Mediocrity is a 10-life loan that comes due when it leaves.
#[test]
fn delusions_of_mediocrity_lends_and_reclaims_ten_life() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::delusions_of_mediocrity());
    g.players[0].mana_pool.add(Color::Blue, 4);
    let life = g.players[0].life;
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].life, life + 10);
    let ctx = crabomination::game::effects::EffectContext::for_ability(spell, 0, None);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::This },
            &ctx,
        )
        .expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "the loan came due");
}

/// The sticky Auras bounce back to hand instead of staying in the graveyard.
#[test]
fn sticky_auras_return_to_hand_from_the_graveyard() {
    for f in [
        catalog::cessation as fn() -> crabomination::card::CardDefinition,
        catalog::sluggishness,
        catalog::sleepers_guile,
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = g.add_card_to_battlefield(0, f());
        g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
        let ctx = crabomination::game::effects::EffectContext::for_ability(aura, 0, None);
        let evs = g
            .resolve_effect(
                &crabomination::effect::Effect::Destroy {
                    what: crabomination::effect::Selector::This,
                },
                &ctx,
            )
            .expect("destroy");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let name = f().name;
        assert!(
            g.players[0].hand.iter().any(|c| c.id == aura),
            "{name} should be back in hand"
        );
    }
}

/// Cessation's host can't attack.
#[test]
fn cessation_stops_the_host_attacking() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    let aura = g.add_card_to_battlefield(0, catalog::cessation());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::CantAttack));
}

/// Granite Grip scales with your Mountains.
#[test]
fn granite_grip_scales_with_mountains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::granite_grip());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "no Mountains yet");
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 2), "power only");
}

/// Iron Maiden burns for hand size over four; Wheel of Torture for three minus
/// hand size. Neither ever heals.
#[test]
fn hand_size_artifacts_burn_the_active_opponent() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::iron_maiden());
    g.add_card_to_battlefield(0, catalog::wheel_of_torture());
    for _ in 0..6 {
        g.add_card_to_hand(1, catalog::forest());
    }
    g.active_player_idx = 1;
    let life = g.players[1].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    // Six cards: Iron Maiden deals 2, Wheel of Torture is floored at 0.
    assert_eq!(g.players[1].life, life - 2);
}

/// Planar Collapse waits for a fourth creature, then wraths.
#[test]
fn planar_collapse_wraths_at_four_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::planar_collapse());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_creature()).count(), 3, "held");
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.turn_number += 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_creature()).count(), 0);
}

/// Impending Disaster waits for seven lands, then destroys them all.
#[test]
fn impending_disaster_destroys_every_land_at_seven() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::impending_disaster());
    for _ in 0..7 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 0);
}

/// Brink of Madness fires only on an empty hand.
#[test]
fn brink_of_madness_needs_an_empty_hand() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::brink_of_madness());
    g.add_card_to_hand(0, catalog::forest());
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::forest());
    }
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 3, "you still held a card");
    g.players[0].hand.clear();
    g.turn_number += 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty());
}

/// Second Chance trades itself for an extra turn at 5 or less life.
#[test]
fn second_chance_buys_an_extra_turn_when_low() {
    let mut g = two_player_game();
    let chance = g.add_card_to_battlefield(0, catalog::second_chance());
    g.players[0].life = 5;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(chance).is_none(), "sacrificed itself");
    assert!(g.players[0].extra_turns > 0, "an extra turn is queued");
}

/// Anthroplasm resets its counters to the X paid.
#[test]
fn anthroplasm_resets_its_counters_to_x() {
    let mut g = two_player_game();
    let plasm = g.add_card_to_battlefield_with_counters(0, catalog::anthroplasm());
    g.battlefield_find_mut(plasm).unwrap().summoning_sick = false;
    assert_eq!(g.computed_permanent(plasm).unwrap().power, 2);
    g.players[0].mana_pool.add(Color::Blue, 5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: plasm,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(5),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(plasm).unwrap().counter_count(CounterType::PlusOnePlusOne),
        5
    );
}

/// Molten Hydra cashes its counters in for damage.
#[test]
fn molten_hydra_converts_counters_into_damage() {
    let mut g = two_player_game();
    let hydra = g.add_card_to_battlefield(0, catalog::molten_hydra());
    g.battlefield_find_mut(hydra).unwrap().summoning_sick = false;
    g.players[0].mana_pool.add(Color::Red, 6);
    for _ in 0..2 {
        activate(&mut g, hydra, 0, None);
    }
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    let life = g.players[1].life;
    activate(&mut g, hydra, 1, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 2);
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Thran Lens blanks every permanent's colour.
#[test]
fn thran_lens_makes_every_permanent_colorless() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).unwrap().colors.is_empty());
    g.add_card_to_battlefield(0, catalog::thran_lens());
    assert!(g.computed_permanent(bear).unwrap().colors.is_empty());
}

/// The self-bouncing Viashinos go home at end of turn.
#[test]
fn viashino_scouts_bounce_at_end_of_turn() {
    for f in [
        catalog::viashino_sandscout as fn() -> crabomination::card::CardDefinition,
        catalog::viashino_cutthroat,
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, f());
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == id), "{} went home", f().name);
    }
}

/// The recursive fatties return to hand rather than staying dead.
#[test]
fn recursive_fatties_return_to_hand_on_death() {
    for f in [
        catalog::shivan_phoenix as fn() -> crabomination::card::CardDefinition,
        catalog::weatherseed_treefolk,
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, f());
        let ctx = crabomination::game::effects::EffectContext::for_ability(id, 0, None);
        let evs = g
            .resolve_effect(
                &crabomination::effect::Effect::Destroy {
                    what: crabomination::effect::Selector::This,
                },
                &ctx,
            )
            .expect("destroy");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == id), "{} came back", f().name);
    }
}

/// The Echo artifacts carry their printed echo cost.
#[test]
fn echo_artifacts_carry_their_echo_cost() {
    for (f, cmc) in [
        (catalog::urzas_blueprints as fn() -> crabomination::card::CardDefinition, 6),
        (catalog::ring_of_gix, 3),
        (catalog::thran_war_machine, 4),
        (catalog::simian_grunts, 3),
    ] {
        let def = f();
        let echo = def
            .keywords
            .iter()
            .find_map(|k| match k {
                Keyword::Echo(c) => Some(c.cmc()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{} has no echo", def.name));
        assert_eq!(echo, cmc, "{}", def.name);
    }
}

/// Purify sweeps artifacts and enchantments and leaves creatures alone.
#[test]
fn purify_destroys_artifacts_and_enchantments_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::thran_lens());
    g.add_card_to_battlefield(1, catalog::knighthood());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::purify());
    g.players[0].mana_pool.add(Color::White, 5);
    cast(&mut g, spell, None);
    assert!(g.battlefield_find(bear).is_some());
    assert!(
        !g.battlefield
            .iter()
            .any(|c| c.definition.card_types.contains(&CardType::Artifact)
                || c.definition.card_types.contains(&CardType::Enchantment))
    );
}

/// Harmonic Convergence tucks every enchantment on top of its owner's library.
#[test]
fn harmonic_convergence_tucks_every_enchantment() {
    let mut g = two_player_game();
    let theirs = g.add_card_to_battlefield(1, catalog::knighthood());
    let spell = g.add_card_to_hand(0, catalog::harmonic_convergence());
    g.players[0].mana_pool.add(Color::Green, 3);
    cast(&mut g, spell, None);
    assert!(g.battlefield_find(theirs).is_none());
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(theirs));
}

/// Subversion drains each opponent on your upkeep.
#[test]
fn subversion_drains_on_your_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::subversion());
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, theirs - 1);
    assert_eq!(g.players[0].life, mine + 1);
}
