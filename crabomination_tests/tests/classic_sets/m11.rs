//! Magic 2011 (M11) gap closure.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

fn always_yes(g: &mut GameState) {
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(8),
    ));
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) -> bool {
    let ok = g
        .perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: idx,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok();
    drain_stack(g);
    ok
}

// ── Stat-line sweep ─────────────────────────────────────────────────────────

/// The keyword-only bodies ship their printed stats.
#[test]
fn m11_vanilla_bodies() {
    let checks: [(fn() -> crabomination::card::CardDefinition, i32, i32); 7] = [
        (catalog::armored_cancrix, 2, 5),
        (catalog::maritime_guard, 1, 3),
        (catalog::nether_horror, 4, 2),
        (catalog::stone_golem, 4, 4),
        (catalog::cloud_crusader, 2, 3),
        (catalog::sacred_wolf, 3, 1),
        (catalog::wall_of_vines, 0, 3),
    ];
    for (f, p, t) in checks {
        let d = f();
        assert_eq!((d.power, d.toughness), (p, t), "{}", d.name);
    }
    assert!(catalog::sacred_wolf().keywords.contains(&Keyword::Hexproof));
    assert!(catalog::wall_of_vines().keywords.contains(&Keyword::Reach));
}

/// Rotting Legion enters tapped.
#[test]
fn rotting_legion_enters_tapped() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rotting_legion());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, id, None);
    assert!(g.battlefield_find(id).unwrap().tapped);
}

// ── Attack gates ────────────────────────────────────────────────────────────

/// Bloodcrazed Goblin waits for first blood.
#[test]
fn bloodcrazed_goblin_needs_first_blood() {
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::bloodcrazed_goblin());
    g.clear_sickness(gob);
    g.step = TurnStep::DeclareAttackers;
    let swing = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: gob,
            target: AttackTarget::Player(1),
        }]))
        .is_ok()
    };
    assert!(!swing(&mut g), "nobody has been hit yet");
    g.players[1].was_dealt_damage_this_turn = true;
    assert!(swing(&mut g), "first blood unlocks it");
}

/// Harbor Serpent needs five Islands on the table.
#[test]
fn harbor_serpent_wants_five_islands() {
    let swing = |islands: usize| {
        let mut g = two_player_game();
        let serpent = g.add_card_to_battlefield(0, catalog::harbor_serpent());
        g.clear_sickness(serpent);
        for _ in 0..islands {
            g.add_card_to_battlefield(0, catalog::island());
        }
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: serpent,
            target: AttackTarget::Player(1),
        }]))
        .is_ok()
    };
    assert!(!swing(4), "four isn't enough");
    assert!(swing(5), "five unlocks the attack");
}

// ── Creatures with abilities ────────────────────────────────────────────────

/// Earth Servant grows a toughness per Mountain.
#[test]
fn earth_servant_scales_with_mountains() {
    let mut g = two_player_game();
    let servant = g.add_card_to_battlefield(0, catalog::earth_servant());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let cp = g.computed_permanent(servant).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 7));
}

/// Water Servant shifts its stats in either direction.
#[test]
fn water_servant_shifts_both_ways() {
    let mut g = two_player_game();
    let servant = g.add_card_to_battlefield(0, catalog::water_servant());
    g.players[0].mana_pool.add(Color::Blue, 2);
    assert!(activate(&mut g, servant, 0, None));
    assert!(activate(&mut g, servant, 1, None));
    let cp = g.computed_permanent(servant).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "+1/-1 then -1/+1 cancels out");
}

/// Gargoyle Sentinel trades defender for flight.
#[test]
fn gargoyle_sentinel_takes_off() {
    let mut g = two_player_game();
    let gargoyle = g.add_card_to_battlefield(0, catalog::gargoyle_sentinel());
    assert!(g.computed_permanent(gargoyle).unwrap().keywords.contains(&Keyword::Defender));
    g.players[0].mana_pool.add_colorless(3);
    assert!(activate(&mut g, gargoyle, 0, None));
    let cp = g.computed_permanent(gargoyle).unwrap();
    assert!(!cp.keywords.contains(&Keyword::Defender));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Arc Runner burns out at the end step.
#[test]
fn arc_runner_burns_out() {
    let mut g = two_player_game();
    let runner = g.add_card_to_battlefield(0, catalog::arc_runner());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(runner).is_none());
}

/// Phantom Beast pops the moment anything points at it.
#[test]
fn phantom_beast_pops_when_targeted() {
    let mut g = two_player_game();
    let beast = g.add_card_to_battlefield(0, catalog::phantom_beast());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Permanent(beast)));
    assert!(g.battlefield_find(beast).is_none());
}

/// Roc Egg hatches a 3/3 flier.
#[test]
fn roc_egg_hatches_a_bird() {
    let mut g = two_player_game();
    let egg = g.add_card_to_battlefield(0, catalog::roc_egg());
    let mut ev = vec![];
    g.destroy_permanent(egg, false, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let bird = g.battlefield.iter().find(|c| c.definition.name == "Bird").expect("token");
    assert_eq!((bird.definition.power, bird.definition.toughness), (3, 3));
    assert!(bird.definition.keywords.contains(&Keyword::Flying));
}

/// Mitotic Slime splits twice on the way out.
#[test]
fn mitotic_slime_splits_twice() {
    let mut g = two_player_game();
    let slime = g.add_card_to_battlefield(0, catalog::mitotic_slime());
    let mut ev = vec![];
    g.destroy_permanent(slime, false, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let big: Vec<CardId> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Ooze")
        .map(|c| c.id)
        .collect();
    assert_eq!(big.len(), 2, "two 2/2 Oozes");
    let mut ev = vec![];
    for id in &big {
        g.destroy_permanent(*id, false, &mut ev);
    }
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Ooze").count(),
        4,
        "each 2/2 leaves two 1/1s"
    );
}

/// Hoarding Dragon buries an artifact and hands it back when it dies.
#[test]
fn hoarding_dragon_hoards_and_returns() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let rock = g.add_card_to_library(0, catalog::sol_ring());
    let dragon = g.add_card_to_hand(0, catalog::hoarding_dragon());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(rock)),
    ]));
    cast(&mut g, dragon, None);
    assert!(g.exile.iter().any(|c| c.id == rock), "exiled by the Dragon");

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut ev = vec![];
    g.destroy_permanent(dragon, false, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == rock), "returned to hand");
}

/// Ancient Hellkite only machine-guns while it's attacking.
#[test]
fn ancient_hellkite_only_fires_while_attacking() {
    let mut g = two_player_game();
    let kite = g.add_card_to_battlefield(0, catalog::ancient_hellkite());
    g.clear_sickness(kite);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 2);
    assert!(!activate(&mut g, kite, 0, Some(Target::Permanent(bear))), "not attacking yet");
    g.attacking = vec![Attack { attacker: kite, target: AttackTarget::Player(1) }];
    assert!(activate(&mut g, kite, 0, Some(Target::Permanent(bear))));
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1);
}

/// Cyclops Gladiator duels a blocker on the swing.
#[test]
fn cyclops_gladiator_duels_on_attack() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let cyclops = g.add_card_to_battlefield(0, catalog::cyclops_gladiator());
    g.clear_sickness(cyclops);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cyclops,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the Bear ate 4");
    assert_eq!(g.battlefield_find(cyclops).unwrap().damage, 2, "and hit back for 2");
}

/// Gaea's Revenge dodges every nongreen answer.
#[test]
fn gaeas_revenge_only_answers_to_green() {
    let mut g = two_player_game();
    let revenge = g.add_card_to_battlefield(0, catalog::gaeas_revenge());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(revenge)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a red Bolt can't target it"
    );
    assert!(catalog::gaeas_revenge().keywords.contains(&Keyword::CantBeCountered));
}

/// Stormtide Leviathan floods the board and grounds everything.
#[test]
fn stormtide_leviathan_floods_the_board() {
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_battlefield(0, catalog::stormtide_leviathan());
    assert!(
        g.computed_permanent(forest)
            .unwrap()
            .subtypes
            .land_types
            .contains(&crabomination::card::LandType::Island),
        "every land is an Island too"
    );
    assert!(g.computed_permanent(ground).unwrap().keywords.contains(&Keyword::CantAttack));
    assert!(!g.computed_permanent(flier).unwrap().keywords.contains(&Keyword::CantAttack));
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Ajani's Mantra pays a life each upkeep.
#[test]
fn ajanis_mantra_gains_a_life() {
    let mut g = two_player_game();
    always_yes(&mut g);
    g.add_card_to_battlefield(0, catalog::ajanis_mantra());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21);
}

/// Dark Tutelage trades life for the top card.
#[test]
fn dark_tutelage_bills_you_for_the_card() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dark_tutelage());
    let top = g.add_card_to_library(0, catalog::serra_angel()); // MV 5
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == top));
    assert_eq!(g.players[0].life, 15);
}

/// Jace's Erasure mills off your draws.
#[test]
fn jaces_erasure_mills_on_draw() {
    let mut g = two_player_game();
    always_yes(&mut g);
    g.add_card_to_battlefield(0, catalog::jaces_erasure());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(1, catalog::island());
    let before: usize = g.players.iter().map(|p| p.library.len()).sum();
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let after: usize = g.players.iter().map(|p| p.library.len()).sum();
    assert_eq!(after, before - 2, "one drawn, one milled");
}

/// The three Leylines start on the battlefield from an opening hand.
#[test]
fn m11_leylines_start_in_play() {
    for f in [
        catalog::leyline_of_anticipation as fn() -> crabomination::card::CardDefinition,
        catalog::leyline_of_punishment,
        catalog::leyline_of_vitality,
    ] {
        assert!(f().opening_hand.is_some(), "{} starts in play", f().name);
    }
}

/// Leyline of Punishment shuts off life gain and prevention.
#[test]
fn leyline_of_punishment_locks_life_and_prevention() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leyline_of_punishment());
    let shield = g.add_card_to_hand(1, catalog::shieldmates_blessing());
    g.players[1].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, shield, Some(Target::Player(1)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 17, "the shield can't prevent it");
    g.adjust_life(1, 5);
    assert_eq!(g.players[1].life, 17, "and nobody gains life");
}

/// Leyline of Vitality toughens the team and pays a life per arrival.
#[test]
fn leyline_of_vitality_toughens_and_gains() {
    let mut g = two_player_game();
    always_yes(&mut g);
    g.add_card_to_battlefield(0, catalog::leyline_of_vitality());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 3);
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21);
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// The M11 Auras grant their printed riders.
#[test]
fn m11_auras() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let strength = g.add_card_to_battlefield(0, catalog::volcanic_strength());
    g.battlefield_find_mut(strength).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.keywords.contains(&Keyword::Landwalk(crabomination::card::LandType::Mountain)));

    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sickness = g.add_card_to_battlefield(0, catalog::quag_sickness());
    g.battlefield_find_mut(sickness).unwrap().attached_to = Some(other);
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    let cp = g.computed_permanent(other).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 0), "-1/-1 per Swamp");
}

/// Primal Cocoon grows its host and falls off when it fights.
#[test]
fn primal_cocoon_grows_then_falls_off() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let cocoon = g.add_card_to_battlefield(0, catalog::primal_cocoon());
    g.battlefield_find_mut(cocoon).unwrap().attached_to = Some(bear);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cocoon).is_none(), "sacrificed on the attack");
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Crystal Ball scries 2.
#[test]
fn crystal_ball_scries_two() {
    let mut g = two_player_game();
    let ball = g.add_card_to_battlefield(0, catalog::crystal_ball());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.players[0].mana_pool.add_colorless(1);
    assert!(activate(&mut g, ball, 0, None));
    assert!(g.battlefield_find(ball).unwrap().tapped);
}

/// Brittle Effigy exiles itself along with its victim.
#[test]
fn brittle_effigy_trades_itself_for_a_creature() {
    let mut g = two_player_game();
    let effigy = g.add_card_to_battlefield(0, catalog::brittle_effigy());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.players[0].mana_pool.add_colorless(4);
    assert!(activate(&mut g, effigy, 0, Some(Target::Permanent(angel))));
    assert!(g.exile.iter().any(|c| c.id == angel), "the Angel is exiled");
    assert!(g.battlefield_find(effigy).is_none(), "and the Effigy went with it");
}

/// Warlord's Axe hits for +3/+1.
#[test]
fn warlords_axe_swings_big() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let axe = g.add_card_to_battlefield(0, catalog::warlords_axe());
    g.battlefield_find_mut(axe).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 3));
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Blood Tithe drains for 3.
#[test]
fn blood_tithe_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blood_tithe());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id, None);
    assert_eq!((g.players[0].life, g.players[1].life), (23, 17));
}

/// Call to Mind buys back a spell.
#[test]
fn call_to_mind_returns_a_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::call_to_mind());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id, Some(Target::Permanent(bolt)));
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
}

/// Diminish shrinks a fatty to 1/1.
#[test]
fn diminish_shrinks_to_one_one() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::diminish());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, id, Some(Target::Permanent(angel)));
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
}

/// Stabbing Pain shrinks and taps.
#[test]
fn stabbing_pain_shrinks_and_taps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::stabbing_pain());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, id, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Thunder Strike pumps and grants first strike.
#[test]
fn thunder_strike_pumps_and_strikes_first() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::thunder_strike());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4);
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// Incite recolors a creature and forces it into the red zone.
#[test]
fn incite_recolors_and_compels() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::incite());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, id, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.colors, vec![Color::Red]);
    assert!(cp.keywords.contains(&Keyword::MustAttack));
}

/// Combust burns a white or blue creature through a shield.
#[test]
fn combust_burns_through_prevention() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let shield = g.add_card_to_hand(1, catalog::shieldmates_blessing());
    g.players[1].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, shield, Some(Target::Permanent(angel)));
    let id = g.add_card_to_hand(0, catalog::combust());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    cast(&mut g, id, Some(Target::Permanent(angel)));
    assert!(g.battlefield_find(angel).is_none(), "5 unpreventable damage kills it");
}

/// Combust can only point at white or blue.
#[test]
fn combust_only_hits_white_or_blue() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::combust());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "a green Bear isn't a legal target"
    );
}

/// Autumn's Veil makes your spells uncounterable by the blue/black half.
#[test]
fn autumns_veil_dodges_blue_and_black() {
    let mut g = two_player_game();
    let veil = g.add_card_to_hand(0, catalog::autumns_veil());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, veil, None);
    assert!(g.players[0].spells_uncounterable_this_turn);
    assert!(g.players[0].hexproof_from_colors_this_turn.contains(&Color::Blue));
    assert!(g.players[0].hexproof_from_colors_this_turn.contains(&Color::Black));
}

/// Hunters' Feast hands out 6 life.
#[test]
fn hunters_feast_gains_six() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::hunters_feast());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 26);
}

/// Destructive Force strips five lands each and sweeps the board.
#[test]
fn destructive_force_wrecks_lands_and_creatures() {
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::mountain());
        g.add_card_to_battlefield(1, catalog::forest());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::destructive_force());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, id, None);
    let lands = |g: &GameState, p: usize| {
        g.battlefield.iter().filter(|c| c.controller == p && c.definition.is_land()).count()
    };
    assert_eq!((lands(&g, 0), lands(&g, 1)), (1, 1));
    assert!(g.battlefield_find(bear).is_none());
}

/// Time Reversal refills both hands and exiles itself.
#[test]
fn time_reversal_refills_and_exiles() {
    let mut g = two_player_game();
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::time_reversal());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id, None);
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[1].hand.len(), 7);
    assert!(g.exile.iter().any(|c| c.id == id), "exiles itself");
}

/// Mystifying Maze blinks an attacker back tapped.
#[test]
fn mystifying_maze_blinks_an_attacker() {
    let mut g = two_player_game();
    let maze = g.add_card_to_battlefield(0, catalog::mystifying_maze());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(0) }];
    g.players[0].mana_pool.add_colorless(4);
    assert!(activate(&mut g, maze, 1, Some(Target::Permanent(bear))));
    assert!(g.battlefield_find(bear).is_none(), "exiled");
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let back = g.battlefield_find(bear).expect("returned at the end step");
    assert!(back.tapped, "and it comes back tapped");
}

/// Scroll Thief and Merfolk Spy both pay off on connecting.
#[test]
fn m11_combat_damage_payoffs() {
    let mut g = two_player_game();
    let thief = g.add_card_to_battlefield(0, catalog::scroll_thief());
    g.clear_sickness(thief);
    g.add_card_to_library(0, catalog::island());
    let before = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: thief,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.resolve_combat().expect("combat");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "the Thief drew");
    assert!(catalog::merfolk_spy()
        .keywords
        .contains(&Keyword::Landwalk(crabomination::card::LandType::Island)));
}

/// Nightwing Shade pumps itself.
#[test]
fn nightwing_shade_pumps() {
    let mut g = two_player_game();
    let shade = g.add_card_to_battlefield(0, catalog::nightwing_shade());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(activate(&mut g, shade, 0, None));
    let cp = g.computed_permanent(shade).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Dryad's Favor grants forestwalk.
#[test]
fn dryads_favor_grants_forestwalk() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let favor = g.add_card_to_battlefield(0, catalog::dryads_favor());
    g.battlefield_find_mut(favor).unwrap().attached_to = Some(bear);
    assert!(g
        .computed_permanent(bear)
        .unwrap()
        .keywords
        .contains(&Keyword::Landwalk(crabomination::card::LandType::Forest)));
}

/// Every M11 gap card is registered by name.
#[test]
fn m11_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for name in ["Crystal Ball", "Time Reversal", "Stormtide Leviathan", "Leyline of Vitality"] {
        assert!(names.contains(&name), "{name} is registered");
    }
    assert!(catalog::stone_golem().card_types.contains(&CardType::Artifact));
}

/// Fire Servant doubles your red burn — but only spells, and only red ones.
#[test]
fn fire_servant_doubles_red_spell_damage() {
    let burn = |servant: bool, spell: fn() -> crabomination::card::CardDefinition| {
        let mut g = two_player_game();
        if servant {
            g.add_card_to_battlefield(0, catalog::fire_servant());
        }
        let id = g.add_card_to_hand(0, spell());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(4);
        cast(&mut g, id, Some(Target::Player(1)));
        20 - g.players[1].life
    };
    assert_eq!(burn(false, catalog::lightning_bolt), 3);
    assert_eq!(burn(true, catalog::lightning_bolt), 6, "red burn is doubled");
    // Its own combat damage is untouched — the static only reads spells.
    let mut g = two_player_game();
    let servant = g.add_card_to_battlefield(0, catalog::fire_servant());
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(1),
        4,
        Some(servant),
        &mut ev,
    );
    assert_eq!(g.players[1].life, 16, "combat damage from the body isn't doubled");
}

/// Demon of Death's Gate comes down for 6 life and three black creatures.
#[test]
fn demon_of_deaths_gate_alternative_cost() {
    let try_alt = |blacks: usize| {
        let mut g = two_player_game();
        for _ in 0..blacks {
            g.add_card_to_battlefield(0, catalog::vampire_nighthawk());
        }
        let demon = g.add_card_to_hand(0, catalog::demon_of_deaths_gate());
        let ok = g
            .perform_action(GameAction::CastSpellAlternative {
                card_id: demon,
                pitch_card: None,
                target: None,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            })
            .is_ok();
        drain_stack(&mut g);
        (ok, g.players[0].life, g.battlefield_find(demon).is_some())
    };
    assert!(!try_alt(2).0, "two black creatures isn't enough");
    assert_eq!(try_alt(3), (true, 14, true), "6 life and three bodies");
}

/// Vengeful Archon turns damage aimed at you around onto a chosen player.
#[test]
fn vengeful_archon_redirects_damage() {
    let mut g = two_player_game();
    let archon = g.add_card_to_battlefield(0, catalog::vengeful_archon());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: archon,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("activate");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 20, "the Archon soaked all three");
    assert_eq!(g.players[1].life, 17, "and sent them back");
}

/// Wild Evocation flips a random hand card into play each upkeep — a land
/// lands, anything else is cast free.
#[test]
fn wild_evocation_deploys_a_random_hand_card() {
    // Land-only hand: it hits the battlefield.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wild_evocation());
    g.players[0].hand.clear();
    let land = g.add_card_to_hand(0, catalog::mountain());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some(), "the land is put onto the battlefield");

    // Spell-only hand: it's cast without paying.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wild_evocation());
    g.players[0].hand.clear();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "cast for free");
    assert_eq!(g.players[1].life, 17);
}

/// Phylactery Lich anchors itself to one artifact and dies with it.
#[test]
fn phylactery_lich_lives_and_dies_by_its_counter() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(0, catalog::sol_ring());
    let lich = g.add_card_to_hand(0, catalog::phylactery_lich());
    g.players[0].mana_pool.add(Color::Black, 3);
    cast(&mut g, lich, None);
    g.check_state_based_actions();
    assert!(g.battlefield_find(lich).is_some(), "anchored to the Sol Ring");
    assert_eq!(
        g.battlefield_find(rock).unwrap().counter_count(CounterType::Phylactery),
        1
    );
    let mut ev = vec![];
    g.destroy_permanent(rock, false, &mut ev);
    g.check_state_based_actions();
    assert!(g.battlefield_find(lich).is_none(), "the anchor is gone, so is the Lich");
}

/// With no artifact to anchor to, the Lich never sticks.
#[test]
fn phylactery_lich_without_an_artifact_dies_immediately() {
    let mut g = two_player_game();
    let lich = g.add_card_to_hand(0, catalog::phylactery_lich());
    g.players[0].mana_pool.add(Color::Black, 3);
    cast(&mut g, lich, None);
    g.check_state_based_actions();
    assert!(g.battlefield_find(lich).is_none());
}

/// Mass Polymorph swaps your board for the same number of creatures off the
/// top, shuffling the misses back.
#[test]
fn mass_polymorph_trades_the_board_for_the_top() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].library.clear();
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    let miss = g.add_card_to_library(0, catalog::island());
    let drake = g.add_card_to_library(0, catalog::sky_ruin_drake());
    let id = g.add_card_to_hand(0, catalog::mass_polymorph());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, id, None);
    for old in [a, b] {
        assert!(g.exile.iter().any(|c| c.id == old), "the old board is exiled");
    }
    assert!(g.battlefield_find(angel).is_some());
    assert!(g.battlefield_find(drake).is_some());
    assert!(g.players[0].library.iter().any(|c| c.id == miss), "the land is shuffled back");
}

/// Angelic Arbiter locks an opponent out of attacking once they've cast a
/// spell, and out of casting once they've attacked.
#[test]
fn angelic_arbiter_forces_the_choice() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::angelic_arbiter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let swing = |g: &mut GameState| {
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Player(1),
        }]))
        .is_ok()
    };
    g.players[0].spells_cast_this_turn = 1;
    assert!(!swing(&mut g), "a spell this turn shuts off the attack");
    g.players[0].spells_cast_this_turn = 0;
    assert!(swing(&mut g), "no spell cast, the attack is legal");

    // Having attacked, the same player can no longer cast.
    g.step = TurnStep::PostCombatMain;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "attacking locks out spells"
    );
}

/// Conundrum Sphinx: a correct guess goes to hand, a miss to the bottom.
#[test]
fn conundrum_sphinx_hits_and_misses() {
    let mut g = two_player_game();
    let sphinx = g.add_card_to_battlefield(0, catalog::conundrum_sphinx());
    g.clear_sickness(sphinx);
    for p in [0, 1] {
        g.players[p].library.clear();
    }
    let mine = g.add_card_to_library(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_library(1, catalog::island());
    let filler = g.add_card_to_library(1, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::NamedCard("Grizzly Bears".into()),
        DecisionAnswer::NamedCard("Grizzly Bears".into()),
    ]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sphinx,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "the hit goes to hand");
    assert_eq!(
        g.players[1].library.last().map(|c| c.id),
        Some(theirs),
        "the miss goes to the bottom"
    );
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(filler));
}

/// Necrotic Plague eats its host at that player's upkeep, then hops to a
/// creature they don't control.
#[test]
fn necrotic_plague_eats_and_hops() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let next = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let plague = g.add_card_to_hand(0, catalog::necrotic_plague());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, plague, Some(Target::Permanent(victim)));
    assert_eq!(g.battlefield_find(plague).and_then(|c| c.attached_to), Some(victim));

    // Player 1's upkeep: the enchanted creature sacrifices itself.
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "the host is sacrificed");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(plague).and_then(|c| c.attached_to),
        Some(next),
        "the Plague hops to a creature its victim's controller doesn't control"
    );
}
