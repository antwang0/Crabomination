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

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// Every wave-2 factory is registered.
#[test]
fn ulg_wave2_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::bouncing_beebles as fn() -> crabomination::card::CardDefinition,
        catalog::gang_of_elk,
        catalog::last_ditch_effort,
        catalog::multanis_presence,
        catalog::palinchron,
        catalog::parch,
        catalog::pyromancy,
        catalog::rank_and_file,
        catalog::raven_familiar,
        catalog::repopulate,
        catalog::rivalry,
        catalog::scrapheap,
        catalog::slow_motion,
        catalog::tethered_skirge,
        catalog::tinker,
        catalog::treefolk_mystic,
        catalog::viashino_bey,
        catalog::viashino_heretic,
        catalog::walking_sponge,
        catalog::weatherseed_elf,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// Last-Ditch Effort converts a board into damage.
#[test]
fn last_ditch_effort_burns_for_the_creatures_sacrificed() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::last_ditch_effort());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(vec![
        crabomination::decision::DecisionAnswer::Amount(3),
    ]));
    let life = g.players[1].life;
    cast(&mut g, spell, Some(Target::Player(1)));
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 0);
    assert_eq!(g.players[1].life, life - 3);
}

/// Parch's second mode doubles up on blue creatures.
#[test]
fn parch_hits_a_blue_creature_for_four() {
    let mut g = two_player_game();
    let faerie = g.add_card_to_battlefield(1, catalog::vigilant_drake());
    let spell = g.add_card_to_hand(0, catalog::parch());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(faerie)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(faerie).is_none(), "4 damage killed the 3/3");
}

/// Gang of Elk grows by twice the number of blockers.
#[test]
fn gang_of_elk_grows_per_blocker() {
    let mut g = two_player_game();
    let gang = g.add_card_to_battlefield(0, catalog::gang_of_elk());
    g.battlefield_find_mut(gang).unwrap().summoning_sick = false;
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking.push(crabomination::game::types::Attack {
        attacker: gang,
        target: crabomination::game::types::AttackTarget::Player(1),
    });
    g.block_map.insert(a, vec![gang]);
    g.block_map.insert(b, vec![gang]);
    g.dispatch_triggers_for_events(&[
        GameEvent::BlockerDeclared { blocker: a, attacker: gang },
        GameEvent::BlockerDeclared { blocker: b, attacker: gang },
    ]);
    drain_stack(&mut g);
    let cp = g.computed_permanent(gang).unwrap();
    assert_eq!((cp.power, cp.toughness), (9, 8), "5/4 plus +4/+4");
}

/// Tethered Skirge bleeds you whenever something targets it.
#[test]
fn tethered_skirge_costs_a_life_per_target() {
    let mut g = two_player_game();
    let skirge = g.add_card_to_battlefield(0, catalog::tethered_skirge());
    let pump = g.add_card_to_hand(0, catalog::iron_will());
    g.players[0].mana_pool.add(Color::White, 1);
    let life = g.players[0].life;
    cast(&mut g, pump, Some(Target::Permanent(skirge)));
    assert_eq!(g.players[0].life, life - 1);
}

/// Scrapheap pays out only for your own artifacts and enchantments.
#[test]
fn scrapheap_gains_life_for_your_artifacts_and_enchantments() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::scrapheap());
    let lens = g.add_card_to_battlefield(0, catalog::thran_lens());
    let theirs = g.add_card_to_battlefield(1, catalog::knighthood());
    let life = g.players[0].life;
    let ctx = crabomination::game::effects::EffectContext::for_ability(theirs, 1, None);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::This },
            &ctx,
        )
        .expect("destroy theirs");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "an opponent's enchantment pays nothing");
    let ctx = crabomination::game::effects::EffectContext::for_ability(lens, 0, None);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::This },
            &ctx,
        )
        .expect("destroy yours");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1);
}

/// Palinchron untaps up to seven lands on entry.
#[test]
fn palinchron_untaps_seven_lands() {
    let mut g = two_player_game();
    for _ in 0..9 {
        let land = g.add_card_to_battlefield(0, catalog::island());
        g.battlefield_find_mut(land).unwrap().tapped = true;
    }
    let pal = g.add_card_to_battlefield(0, catalog::palinchron());
    g.fire_self_etb_triggers(pal, 0);
    drain_stack(&mut g);
    let untapped = g.battlefield.iter().filter(|c| c.definition.is_land() && !c.tapped).count();
    assert_eq!(untapped, 7);
}

/// Viashino Heretic blows up an artifact and burns for its mana value.
#[test]
fn viashino_heretic_burns_for_the_artifacts_mana_value() {
    let mut g = two_player_game();
    let heretic = g.add_card_to_battlefield(0, catalog::viashino_heretic());
    g.battlefield_find_mut(heretic).unwrap().summoning_sick = false;
    let jar = g.add_card_to_battlefield(1, catalog::urzas_blueprints());
    g.players[0].mana_pool.add(Color::Red, 2);
    let life = g.players[1].life;
    activate(&mut g, heretic, 0, Some(Target::Permanent(jar)));
    assert!(g.battlefield_find(jar).is_none());
    assert_eq!(g.players[1].life, life - 6, "Urza's Blueprints costs {{6}}");
}

/// Repopulate shuffles only creature cards back in.
#[test]
fn repopulate_shuffles_creatures_out_of_the_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let land = g.add_card_to_graveyard(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::repopulate());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), 1);
    assert_eq!(g.players[1].graveyard[0].id, land);
}

/// Tinker sacrifices an artifact to tutor a bigger one straight into play.
#[test]
fn tinker_trades_an_artifact_for_a_tutored_one() {
    let mut g = two_player_game();
    let chaff = g.add_card_to_battlefield(0, catalog::jhoiras_toolbox());
    g.add_card_to_library(0, catalog::urzas_blueprints());
    let spell = g.add_card_to_hand(0, catalog::tinker());
    g.players[0].mana_pool.add(Color::Blue, 3);
    let target = g.players[0]
        .library
        .iter()
        .find(|c| c.definition.name == "Urza's Blueprints")
        .expect("in library")
        .id;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new(vec![
        crabomination::decision::DecisionAnswer::Search(Some(target)),
    ]));
    cast(&mut g, spell, None);
    assert!(g.battlefield_find(chaff).is_none(), "sacrificed as a cost");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Urza's Blueprints"),
        "the tutored artifact is in play"
    );
}

/// Treefolk Mystic strips the Auras off whatever it blocks.
#[test]
fn treefolk_mystic_strips_auras_from_its_blocker() {
    let mut g = two_player_game();
    let mystic = g.add_card_to_battlefield(0, catalog::treefolk_mystic());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(1, catalog::granite_grip());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    g.block_map.insert(bear, vec![mystic]);
    g.dispatch_triggers_for_events(&[GameEvent::BlockerDeclared {
        blocker: bear,
        attacker: mystic,
    }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "the Aura was destroyed");
    assert!(g.battlefield_find(bear).is_some());
}

/// Rivalry burns only the player with strictly the most lands.
#[test]
fn rivalry_burns_the_land_leader() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rivalry());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
        g.add_card_to_battlefield(1, catalog::mountain());
    }
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "tied on lands");
    g.add_card_to_battlefield(0, catalog::mountain());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2);
}

/// Walking Sponge strips the chosen evasion keyword.
#[test]
fn walking_sponge_strips_flying() {
    let mut g = two_player_game();
    let sponge = g.add_card_to_battlefield(0, catalog::walking_sponge());
    g.battlefield_find_mut(sponge).unwrap().summoning_sick = false;
    let faerie = g.add_card_to_battlefield(1, catalog::weatherseed_faeries());
    g.perform_action(GameAction::ActivateAbility {
        card_id: sponge,
        ability_index: 0,
        target: Some(Target::Permanent(faerie)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(!g.computed_permanent(faerie).unwrap().keywords.contains(&Keyword::Flying));
}

/// Weatherseed Elf hands out forestwalk.
#[test]
fn weatherseed_elf_grants_forestwalk() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::weatherseed_elf());
    g.battlefield_find_mut(elf).unwrap().summoning_sick = false;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, elf, 0, Some(Target::Permanent(bear)));
    assert!(
        g.computed_permanent(bear)
            .unwrap()
            .keywords
            .contains(&Keyword::Landwalk(crabomination::card::LandType::Forest))
    );
}

/// Rank and File shrinks green creatures as it enters.
#[test]
fn rank_and_file_shrinks_green_creatures() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(1, catalog::weatherseed_elf());
    let faerie = g.add_card_to_battlefield(1, catalog::weatherseed_faeries());
    let rank = g.add_card_to_battlefield(0, catalog::rank_and_file());
    g.fire_self_etb_triggers(rank, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_none(), "the 1/1 Elf died");
    assert!(g.battlefield_find(faerie).is_some(), "the blue Faerie is untouched");
}

/// Multani's Presence replaces a countered spell.
#[test]
fn multanis_presence_draws_when_your_spell_is_countered() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::multanis_presence());
    g.add_card_to_library(0, catalog::forest());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let counter = g.add_card_to_hand(1, catalog::intervene());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 1;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the countered spell replaced itself");
}

// ── Wave 3 ──────────────────────────────────────────────────────────────────

/// No Mercy kills whatever damaged you.
#[test]
fn no_mercy_destroys_the_creature_that_hit_you() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::no_mercy());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        2,
        Some(attacker),
        &mut ev,
    );
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none());
}

/// Opal Champion animates on an opponent's creature spell — and only once.
#[test]
fn opal_champion_wakes_on_an_opponents_creature_spell() {
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::opal_champion());
    assert!(!g.computed_permanent(champ).unwrap().card_types.contains(&CardType::Creature));
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(champ).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// An opponent's noncreature spell leaves Opal Champion asleep.
#[test]
fn opal_champion_ignores_noncreature_spells() {
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::opal_champion());
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
    .expect("cast");
    drain_stack(&mut g);
    assert!(!g.computed_permanent(champ).unwrap().card_types.contains(&CardType::Creature));
}

/// Hidden Gibbons wakes on an opponent's instant.
#[test]
fn hidden_gibbons_wakes_on_an_opponents_instant() {
    let mut g = two_player_game();
    let gibbons = g.add_card_to_battlefield(0, catalog::hidden_gibbons());
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
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(gibbons).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Opal Avenger wakes up once you fall to 10.
#[test]
fn opal_avenger_wakes_at_ten_life() {
    let mut g = two_player_game();
    let avenger = g.add_card_to_battlefield(0, catalog::opal_avenger());
    let mut ev = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 5, None, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(
        !g.computed_permanent(avenger).unwrap().card_types.contains(&CardType::Creature),
        "still at 15"
    );
    let mut ev = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 5, None, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let cp = g.computed_permanent(avenger).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 5));
}

/// Lurking Skirge wakes when a creature dies under an opponent.
#[test]
fn lurking_skirge_wakes_on_an_opponents_creature_dying() {
    let mut g = two_player_game();
    let skirge = g.add_card_to_battlefield(0, catalog::lurking_skirge());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_ability(victim, 1, None);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::This },
            &ctx,
        )
        .expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let cp = g.computed_permanent(skirge).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Crawlspace caps attackers against its controller at two.
#[test]
fn crawlspace_caps_attackers_at_two() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::crawlspace());
    let attackers: Vec<CardId> = (0..3)
        .map(|_| {
            let id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
            g.battlefield_find_mut(id).unwrap().summoning_sick = false;
            id
        })
        .collect();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    let attack = |ids: &[CardId]| {
        ids.iter()
            .map(|id| crabomination::game::types::Attack {
                attacker: *id,
                target: crabomination::game::types::AttackTarget::Player(0),
            })
            .collect::<Vec<_>>()
    };
    assert!(g.declare_attackers(attack(&attackers)).is_err(), "three is over the cap");
    g.declare_attackers(attack(&attackers[..2])).expect("two is legal");
}

/// Treacherous Link moves the host's damage onto its controller.
#[test]
fn treacherous_link_redirects_damage_to_the_hosts_controller() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::treacherous_link());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let life = g.players[1].life;
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(bear),
        3,
        None,
        &mut ev,
    );
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "the controller took it");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "the 2/2 is unmarked");
}

/// Angel's Trumpet grants vigilance and bites the active player for each idle
/// creature it taps at their end step.
#[test]
fn angels_trumpet_taps_idlers_and_bites() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::angels_trumpet());
    let idle = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(idle).unwrap().keywords.contains(&Keyword::Vigilance),
        "the anthem reaches every creature"
    );
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(idle).unwrap().tapped);
    assert!(g.battlefield_find(other).unwrap().tapped);
    assert_eq!(g.players[0].life, life - 2, "one point per creature tapped this way");
}

/// Aura Flux taxes other enchantments — not itself — at their controller's
/// upkeep.
#[test]
fn aura_flux_taxes_other_enchantments() {
    let mut g = two_player_game();
    let flux = g.add_card_to_battlefield(0, catalog::aura_flux());
    let other = g.add_card_to_battlefield(0, catalog::planar_collapse());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(other).is_none(), "unpaid, so it goes");
    assert!(g.battlefield_find(flux).is_some(), "\"other\" excludes the source");
}

/// Damping Engine locks the permanent leader out of lands, and a sacrifice
/// buys the turn back.
#[test]
fn damping_engine_locks_the_leader_until_they_pay() {
    let mut g = two_player_game();
    let engine = g.add_card_to_battlefield(1, catalog::damping_engine());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.can_player_play_land(0), "seat 0 is ahead on permanents");
    assert!(g.can_player_play_land(1), "seat 1 isn't");
    g.priority.player_with_priority = 0;
    activate(&mut g, engine, 0, None);
    assert!(g.battlefield_find(fodder).is_none(), "a permanent was sacrificed");
    assert!(g.can_player_play_land(0), "the pass holds for the turn");
}

/// Martyr's Cause soaks the whole next damage event from the chosen source,
/// wherever it lands.
#[test]
fn martyrs_cause_blanks_the_next_damage_event() {
    let mut g = two_player_game();
    let cause = g.add_card_to_battlefield(0, catalog::martyrs_cause());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, cause, 0, None);
    let life = g.players[0].life;
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        5,
        Some(attacker),
        &mut ev,
    );
    assert_eq!(g.players[0].life, life, "the whole event was prevented");
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        5,
        Some(attacker),
        &mut ev,
    );
    assert_eq!(g.players[0].life, life - 5, "the shield was one-shot");
}

/// Memory Jar swaps every hand for seven, then hands the stash back at the end
/// step.
#[test]
fn memory_jar_lends_seven_and_takes_them_back() {
    let mut g = two_player_game();
    let jar = g.add_card_to_battlefield(0, catalog::memory_jar());
    for seat in 0..2 {
        for _ in 0..2 {
            g.add_card_to_hand(seat, catalog::grizzly_bears());
        }
        for _ in 0..7 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    let stashed: Vec<CardId> = g.players[0].hand.iter().map(|c| c.id).collect();
    activate(&mut g, jar, 0, None);
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[1].hand.len(), 7);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let back: Vec<CardId> = g.players[0].hand.iter().map(|c| c.id).collect();
    assert_eq!(back, stashed, "the loan is returned, the drawn seven discarded");
    assert_eq!(g.players[0].graveyard.len(), 8, "the drawn seven, plus the sacrificed Jar");
}

/// Thran Weaponry's pump lives exactly as long as it stays tapped.
#[test]
fn thran_weaponry_pump_ends_when_it_untaps() {
    let mut g = two_player_game();
    let weaponry = g.add_card_to_battlefield(0, catalog::thran_weaponry());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, weaponry, 0, None);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
    g.battlefield_find_mut(weaponry).unwrap().tapped = false;
    g.check_state_based_actions();
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "the effect fell off");
}
