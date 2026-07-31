//! Zendikar gap closure, wave 3 — the last of the set.

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

fn cast_alt(g: &mut GameState, id: CardId, target: Option<Target>) -> bool {
    let ok = g
        .perform_action(GameAction::CastSpellAlternative {
            card_id: id,
            pitch_card: None,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_ok();
    drain_stack(g);
    ok
}

/// Yes to every "you may" a landfall / Rally trigger asks.
fn always_yes(g: &mut GameState) {
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(8),
    ));
}

/// Play a land for seat 0 and drain the landfall triggers it fires.
fn landfall(g: &mut GameState) -> CardId {
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.dispatch_triggers_for_events(&[GameEvent::LandPlayed { card_id: land, player: 0, played: true }]);
    drain_stack(g);
    land
}

// ── Stat-line sweep ─────────────────────────────────────────────────────────

/// The keyword-only bodies ship their printed stats and keywords.
#[test]
fn zen3_vanilla_bodies() {
    let checks: [(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword]); 6] = [
        (catalog::shatterskull_giant, 4, 3, &[]),
        (
            catalog::shepherd_of_the_lost,
            3,
            3,
            &[Keyword::Flying, Keyword::FirstStrike, Keyword::Vigilance],
        ),
        (catalog::sky_ruin_drake, 2, 5, &[Keyword::Flying]),
        (catalog::stonework_puma, 2, 2, &[]),
        (
            catalog::zendikar_farguide,
            3,
            3,
            &[Keyword::Landwalk(crabomination::card::LandType::Forest)],
        ),
        (catalog::sphinx_of_jwar_isle, 5, 5, &[Keyword::Flying, Keyword::Shroud]),
    ];
    for (f, p, t, kws) in checks {
        let d = f();
        assert_eq!((d.power, d.toughness), (p, t), "{}", d.name);
        for kw in kws {
            assert!(d.keywords.contains(kw), "{} lacks {kw:?}", d.name);
        }
    }
    assert!(catalog::stonework_puma().card_types.contains(&CardType::Artifact));
}

/// Sphinx of Jwar Isle's controller — and only its controller — sees the top
/// card of their library.
#[test]
fn sphinx_of_jwar_isle_shows_its_controller_the_top_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let own = crabomination::server::view::project(&g, 0);
    assert!(own.players[0].library.known_top.is_empty(), "hidden without the Sphinx");
    g.add_card_to_battlefield(0, catalog::sphinx_of_jwar_isle());
    let own = crabomination::server::view::project(&g, 0);
    let theirs = crabomination::server::view::project(&g, 1);
    assert_eq!(own.players[0].library.known_top.len(), 1, "the controller may look");
    assert!(theirs.players[0].library.known_top.is_empty(), "the opponent may not");
}

// ── Landfall ────────────────────────────────────────────────────────────────

/// The landfall pumps/grants fire off a land drop.
#[test]
fn zen3_landfall_riders() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let eel = g.add_card_to_battlefield(0, catalog::windrider_eel());
    let serpent = g.add_card_to_battlefield(0, catalog::shoal_serpent());
    let marauder = g.add_card_to_battlefield(0, catalog::surrakar_marauder());
    assert!(g.computed_permanent(serpent).unwrap().keywords.contains(&Keyword::Defender));
    landfall(&mut g);
    assert_eq!(g.computed_permanent(eel).unwrap().power, 4, "2/2 plus 2/2");
    assert!(!g.computed_permanent(serpent).unwrap().keywords.contains(&Keyword::Defender));
    assert!(g.computed_permanent(marauder).unwrap().keywords.contains(&Keyword::Intimidate));
}

/// Turntimber Basilisk's landfall forces a blocker in front of it.
#[test]
fn turntimber_basilisk_lures_a_blocker() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let basilisk = g.add_card_to_battlefield(0, catalog::turntimber_basilisk());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    landfall(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.must_block == Some(basilisk)),
        "a creature is lured in front of the Basilisk"
    );
    let _ = bear;
}

/// Roil Elemental's landfall steals a creature, and it goes home when the
/// Elemental leaves.
#[test]
fn roil_elemental_steals_while_it_survives() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let roil = g.add_card_to_battlefield(0, catalog::roil_elemental());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    landfall(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "stolen");
    let mut ev = vec![];
    g.destroy_permanent(roil, false, &mut ev);
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 1, "returned");
}

/// The Expeditions bank quest counters off land drops and cash three in.
#[test]
fn zen3_expeditions() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let sun = g.add_card_to_battlefield(0, catalog::sunspring_expedition());
    landfall(&mut g);
    assert_eq!(g.battlefield_find(sun).unwrap().counter_count(CounterType::Quest), 1);
    g.battlefield_find_mut(sun).unwrap().add_counters(CounterType::Quest, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sun,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cash in");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 28);
    assert!(g.battlefield_find(sun).is_none(), "sacrificed as a cost");
}

/// Zektar Shrine Expedition's Elemental is a 7/1 with trample and haste.
#[test]
fn zektar_shrine_expedition_makes_a_hasty_elemental() {
    let mut g = two_player_game();
    let shrine = g.add_card_to_battlefield(0, catalog::zektar_shrine_expedition());
    g.battlefield_find_mut(shrine).unwrap().add_counters(CounterType::Quest, 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shrine,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cash in");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Elemental").expect("token");
    assert_eq!((token.definition.power, token.definition.toughness), (7, 1));
    assert!(token.definition.keywords.contains(&Keyword::Haste));
}

// ── Allies ──────────────────────────────────────────────────────────────────

/// The Rally growers count their own entry, and Turntimber Ranger pays a Wolf
/// on top.
#[test]
fn zen3_rally_payoffs() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let raptor = g.add_card_to_battlefield(0, catalog::umara_raptor());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: raptor }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(raptor).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);

    let ranger = g.add_card_to_battlefield(0, catalog::turntimber_ranger());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ranger }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ranger).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Wolf"));
    assert_eq!(
        g.battlefield_find(raptor).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "the Ranger's entry grew the Raptor too"
    );
}

/// Tajuru Archer snipes a flier for the Ally count.
#[test]
fn tajuru_archer_shoots_for_the_ally_count() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel());
    let archer = g.add_card_to_battlefield(0, catalog::tajuru_archer());
    g.add_card_to_battlefield(0, catalog::stonework_puma());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: archer }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(flier).unwrap().damage, 2, "two Allies");
}

// ── Kicker ──────────────────────────────────────────────────────────────────

/// Torch Slinger and Tempest Owl only fire their ETB when kicked.
#[test]
fn zen3_kicked_etbs() {
    let slinger = |kicked: bool| {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::serra_angel());
        let card = g.add_card_to_hand(0, catalog::torch_slinger());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(3);
        let action = if kicked {
            GameAction::CastSpellKicked {
                card_id: card,
                target: Some(Target::Permanent(bear)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        } else {
            GameAction::CastSpell {
                card_id: card,
                target: Some(Target::Permanent(bear)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        };
        g.perform_action(action).expect("cast");
        drain_stack(&mut g);
        g.battlefield_find(bear).map(|c| c.damage).unwrap_or(0)
    };
    assert_eq!(slinger(false), 0, "unkicked does nothing");
    assert_eq!(slinger(true), 2, "kicked bolts for 2");
}

/// Vampire's Bite pumps either way; only the kicked half grants lifelink.
#[test]
fn vampires_bite_kicker_adds_lifelink() {
    let run = |kicked: bool| {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let card = g.add_card_to_hand(0, catalog::vampires_bite());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(2);
        let target = Some(Target::Permanent(bear));
        let action = if kicked {
            GameAction::CastSpellKicked {
                card_id: card,
                target,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        } else {
            GameAction::CastSpell {
                card_id: card,
                target,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        };
        g.perform_action(action).expect("cast");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        (cp.power, cp.keywords.contains(&Keyword::Lifelink))
    };
    assert_eq!(run(false), (5, false));
    assert_eq!(run(true), (5, true));
}

/// Unstable Footing locks out prevention, and kicked it burns for 5.
#[test]
fn unstable_footing_beats_prevention() {
    let mut g = two_player_game();
    let card = g.add_card_to_hand(0, catalog::unstable_footing());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let shield = g.add_card_to_hand(1, catalog::shieldmates_blessing());
    g.players[1].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, shield, Some(Target::Player(1)));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: card,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 15, "the shield can't prevent it");
}

/// Blood Tribute halves an opponent; the Vampire-tap kicker turns it into a
/// drain.
#[test]
fn blood_tribute_kicker_taps_a_vampire() {
    let run = |kicked: bool, vampires: usize| {
        let mut g = two_player_game();
        let vamps: Vec<CardId> = (0..vampires)
            .map(|_| g.add_card_to_battlefield(0, catalog::vampire_nighthawk()))
            .collect();
        let card = g.add_card_to_hand(0, catalog::blood_tribute());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.players[1].life = 15;
        let target = Some(Target::Player(1));
        let action = if kicked {
            GameAction::CastSpellKicked {
                card_id: card,
                target,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        } else {
            GameAction::CastSpell {
                card_id: card,
                target,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }
        };
        let ok = g.perform_action(action).is_ok();
        drain_stack(&mut g);
        (ok, g.players[0].life, g.players[1].life, vamps.iter().filter(|&&v| g.battlefield_find(v).unwrap().tapped).count())
    };
    assert_eq!(run(false, 0), (true, 20, 7, 0), "8 of 15, rounded up");
    assert_eq!(run(true, 1), (true, 28, 7, 1), "kicked drains and taps the Vampire");
    assert!(!run(true, 0).0, "no Vampire, no kicker");
}

/// Gigantiform makes its host a base 8/8 trampler and fetches the next copy
/// when kicked.
#[test]
fn gigantiform_sets_base_pt_and_fetches_a_copy() {
    let mut g = two_player_game();
    always_yes(&mut g);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let copy = g.add_card_to_library(0, catalog::gigantiform());
    let card = g.add_card_to_hand(0, catalog::gigantiform());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(7);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(copy)),
    ]));
    g.perform_action(GameAction::CastSpellKicked {
        card_id: card,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8));
    assert!(cp.keywords.contains(&Keyword::Trample));
    assert!(g.battlefield_find(copy).is_some(), "the second copy was fetched");
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Seismic Shudder spares fliers.
#[test]
fn seismic_shudder_misses_fliers() {
    let mut g = two_player_game();
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel());
    let card = g.add_card_to_hand(0, catalog::seismic_shudder());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, card, None);
    assert_eq!(g.battlefield_find(ground).unwrap().damage, 1);
    assert_eq!(g.battlefield_find(flier).unwrap().damage, 0);
}

/// Tanglesap lets trample through and stops everything else.
#[test]
fn tanglesap_only_lets_tramplers_through() {
    let mut g = two_player_game();
    let plain = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let trampler = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    let card = g.add_card_to_hand(0, catalog::tanglesap());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, card, None);
    g.attacking = vec![
        Attack { attacker: plain, target: AttackTarget::Player(0) },
        Attack { attacker: trampler, target: AttackTarget::Player(0) },
    ];
    g.resolve_combat().expect("combat");
    assert_eq!(g.players[0].life, 20 - 6, "only the Dreadmaw connects");
}

/// Shieldmate's Blessing soaks up the next 3 damage.
#[test]
fn shieldmates_blessing_prevents_three() {
    let mut g = two_player_game();
    let card = g.add_card_to_hand(0, catalog::shieldmates_blessing());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, card, Some(Target::Player(0)));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 20, "all three prevented");
}

/// Spire Barrage scales with your Mountains.
#[test]
fn spire_barrage_counts_mountains() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let card = g.add_card_to_hand(0, catalog::spire_barrage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, card, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 17);
}

/// Windborne Charge lifts two of your creatures.
#[test]
fn windborne_charge_pumps_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::windborne_charge());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: card,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    for id in [a, b] {
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4));
        assert!(cp.keywords.contains(&Keyword::Flying));
    }
}

/// Summoner's Bane counters a creature spell and leaves an Illusion.
#[test]
fn summoners_bane_counters_and_mints() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bear");
    let card = g.add_card_to_hand(0, catalog::summoners_bane());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    cast(&mut g, card, Some(Target::Permanent(bear)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "countered");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Illusion"));
}

/// Trapmaker's Snare tutors a Trap; Trapfinder's Trick strips them from a hand.
#[test]
fn zen3_trap_tutors_and_hate() {
    let mut g = two_player_game();
    let trap = g.add_card_to_library(0, catalog::cobra_trap());
    g.add_card_to_library(0, catalog::island());
    let snare = g.add_card_to_hand(0, catalog::trapmakers_snare());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(trap))]));
    cast(&mut g, snare, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == trap));

    let mut g = two_player_game();
    let their_trap = g.add_card_to_hand(1, catalog::cobra_trap());
    let keeper = g.add_card_to_hand(1, catalog::grizzly_bears());
    let trick = g.add_card_to_hand(0, catalog::trapfinders_trick());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, trick, Some(Target::Player(1)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == their_trap), "the Trap is discarded");
    assert!(g.players[1].hand.iter().any(|c| c.id == keeper), "the Bear stays");
}

/// Punishing Fire buys itself back out of the graveyard when an opponent gains
/// life and you pay {R}.
#[test]
fn punishing_fire_recurs_on_opponent_lifegain() {
    let mut g = two_player_game();
    let card = g.add_card_to_hand(0, catalog::punishing_fire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, card, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 18);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == card));

    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.adjust_life(1, 3);
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 1, amount: 3 }]);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == card), "bought back");
}

// ── Traps ───────────────────────────────────────────────────────────────────

/// Summoning Trap is free once an opponent countered a creature spell of yours,
/// and digs seven for a creature.
#[test]
fn summoning_trap_free_after_a_countered_creature() {
    let mut g = two_player_game();
    let trap = g.add_card_to_hand(0, catalog::summoning_trap());
    assert!(!cast_alt(&mut g, trap, None), "no counter yet");

    // Their counterspell on your Bear turns it on.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bear");
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    cast(&mut g, counter, Some(Target::Permanent(bear)));
    assert!(g.players[0].creature_spell_countered_by_opponent_this_turn);

    let fatty = g.add_card_to_library(0, catalog::serra_angel());
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(4),
    ));
    assert!(cast_alt(&mut g, trap, None), "free now");
    assert!(g.battlefield_find(fatty).is_some(), "dug out a creature");
}

/// Cobra Trap costs {G} once an opponent's spell destroyed a noncreature
/// permanent of yours.
#[test]
fn cobra_trap_watches_your_noncreature_losses() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(0, catalog::sol_ring());
    let trap = g.add_card_to_hand(0, catalog::cobra_trap());
    g.players[0].mana_pool.add(Color::Green, 1);
    assert!(!cast_alt(&mut g, trap, None), "nothing destroyed yet");

    let naturalize = g.add_card_to_hand(1, catalog::naturalize());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    cast(&mut g, naturalize, Some(Target::Permanent(rock)));
    assert!(g.players[0].noncreature_destroyed_by_opponent_this_turn);

    g.priority.player_with_priority = 0;
    assert!(cast_alt(&mut g, trap, None), "{{G}} now");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Snake").count(), 4);
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// The two Equipment grant their printed riders.
#[test]
fn zen3_equipment() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let net = g.add_card_to_battlefield(0, catalog::spidersilk_net());
    g.battlefield_find_mut(net).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4));
    assert!(cp.keywords.contains(&Keyword::Reach));

    let boots = g.add_card_to_battlefield(0, catalog::trailblazers_boots());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(boots).unwrap().attached_to = Some(other);
    assert!(
        g.computed_permanent(other)
            .unwrap()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::LandwalkFiltered(_)))
    );
}

/// Eternity Vessel banks your life total and landfall restores it.
#[test]
fn eternity_vessel_resets_your_life() {
    let mut g = two_player_game();
    always_yes(&mut g);
    g.players[0].life = 23;
    let vessel = g.add_card_to_hand(0, catalog::eternity_vessel());
    g.players[0].mana_pool.add_colorless(6);
    cast(&mut g, vessel, None);
    assert_eq!(g.battlefield_find(vessel).unwrap().counter_count(CounterType::Charge), 23);
    g.players[0].life = 4;
    landfall(&mut g);
    assert_eq!(g.players[0].life, 23);
}

// ── Creatures with abilities ────────────────────────────────────────────────

/// Timbermaw Larva grows per Forest on the attack.
#[test]
fn timbermaw_larva_grows_per_forest() {
    let mut g = two_player_game();
    let larva = g.add_card_to_battlefield(0, catalog::timbermaw_larva());
    g.clear_sickness(larva);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: larva,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(larva).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Gomazoa sends itself and everything it's blocking to the top of their
/// libraries.
#[test]
fn gomazoa_bounces_the_whole_block_to_libraries() {
    let mut g = two_player_game();
    let gomazoa = g.add_card_to_battlefield(0, catalog::gomazoa());
    g.clear_sickness(gomazoa);
    let attacker = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(0) }];
    g.block_map.insert(gomazoa, vec![attacker]);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gomazoa,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(gomazoa).is_none());
    assert!(g.battlefield_find(attacker).is_none());
    assert!(g.players[1].library.iter().any(|c| c.id == attacker), "back to its owner's library");
}

/// Kalitas leaves a Vampire the size of what he killed.
#[test]
fn kalitas_makes_a_vampire_the_victims_size() {
    let mut g = two_player_game();
    let kalitas = g.add_card_to_battlefield(0, catalog::kalitas_bloodchief_of_ghet());
    g.battlefield_find_mut(kalitas).unwrap().summoning_sick = false;
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kalitas,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Vampire").expect("token");
    assert_eq!((token.power(), token.toughness()), (4, 4), "the Angel's size");
}

/// Lullmage Mentor breeds Merfolk off your counterspells and counters a spell
/// by tapping seven of them.
#[test]
fn lullmage_mentor_breeds_and_counters() {
    let mut g = two_player_game();
    always_yes(&mut g);
    g.add_card_to_battlefield(0, catalog::lullmage_mentor());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bear");
    let counter = g.add_card_to_hand(0, catalog::counterspell());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    cast(&mut g, counter, Some(Target::Permanent(bear)));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Merfolk"), "a Merfolk hatched");
}

/// Lullmage Mentor's tap-seven-Merfolk cost is real: six won't do.
#[test]
fn lullmage_mentor_tap_cost_needs_seven_merfolk() {
    let run = |extra: usize| {
        let mut g = two_player_game();
        let mentor = g.add_card_to_battlefield(0, catalog::lullmage_mentor());
        g.battlefield_find_mut(mentor).unwrap().summoning_sick = false;
        for _ in 0..extra {
            let m = g.add_card_to_battlefield(0, catalog::merfolk_wayfinder());
            g.battlefield_find_mut(m).unwrap().summoning_sick = false;
        }
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
        .expect("bolt");
        g.priority.player_with_priority = 0;
        let ok = g
            .perform_action(GameAction::ActivateAbility {
                card_id: mentor,
                ability_index: 0,
                target: Some(Target::Permanent(bolt)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            })
            .is_ok();
        drain_stack(&mut g);
        (ok, g.players[0].life)
    };
    assert!(!run(5).0, "six Merfolk isn't enough");
    let (ok, life) = run(6);
    assert!(ok, "seven pays the cost");
    assert_eq!(life, 20, "the Bolt was countered");
}

/// World Queller names a type each upkeep and everyone gives one up.
#[test]
fn world_queller_edicts_a_chosen_type() {
    let mut g = two_player_game();
    let queller = g.add_card_to_battlefield(0, catalog::world_queller());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let their_land = g.add_card_to_battlefield(1, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Mode(0),
    ]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none() || g.battlefield_find(queller).is_none());
    assert!(g.battlefield_find(theirs).is_none(), "their creature went");
    assert!(g.battlefield_find(their_land).is_some(), "their land stayed");
}

/// Obsidian Fireheart's blaze counter keeps burning after he's gone.
#[test]
fn obsidian_fireheart_land_burns_without_him() {
    let mut g = two_player_game();
    let heart = g.add_card_to_battlefield(0, catalog::obsidian_fireheart());
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: heart,
        ability_index: 0,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(CounterType::Blaze), 1);

    let mut ev = vec![];
    g.destroy_permanent(heart, false, &mut ev);
    g.check_state_based_actions();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "the land keeps burning");
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// The utility lands enter tapped and fire their ETB rider.
#[test]
fn zen3_utility_lands() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let peaks = g.add_card_to_hand(0, catalog::teetering_peaks());
    g.perform_action(GameAction::PlayLand(peaks)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(peaks).unwrap().tapped, "enters tapped");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);

    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cliff = g.add_card_to_hand(0, catalog::soaring_seacliff());
    g.perform_action(GameAction::PlayLand(cliff)).expect("play");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
}

/// Oran-Rief counters up the green creatures that entered this turn.
#[test]
fn oran_rief_grows_this_turns_green_creatures() {
    let mut g = two_player_game();
    let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let old = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blue = g.add_card_to_battlefield(0, catalog::sky_ruin_drake());
    g.turn_number = 5;
    for id in [fresh, blue] {
        g.battlefield_find_mut(id).unwrap().entered_turn = Some(5);
    }
    g.battlefield_find_mut(old).unwrap().entered_turn = Some(3);
    let land = g.add_card_to_battlefield(0, catalog::oran_rief_the_vastwood());
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let n = |id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!((n(fresh), n(old), n(blue)), (1, 0, 0));
}

/// Magosi banks an eon counter (skipping a turn) and cashes it in for an extra
/// one, bouncing itself as the cost.
#[test]
fn magosi_trades_a_turn_for_a_turn() {
    let mut g = two_player_game();
    let magosi = g.add_card_to_battlefield(0, catalog::magosi_the_waterveil());
    g.battlefield_find_mut(magosi).unwrap().tapped = false;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: magosi,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bank");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(magosi).unwrap().counter_count(CounterType::Eon), 1);
    assert_eq!(g.players[0].skip_turns, 1);

    g.battlefield_find_mut(magosi).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: magosi,
        ability_index: 2,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cash in");
    drain_stack(&mut g);
    assert!(g.battlefield_find(magosi).is_none(), "bounced as a cost");
    assert!(g.players[0].hand.iter().any(|c| c.id == magosi));
    assert_eq!(g.players[0].extra_turns, 1);
}

// ── Planeswalkers ───────────────────────────────────────────────────────────

/// Chandra Ablaze's +1 only burns when the discard was red.
#[test]
fn chandra_ablaze_plus_one_needs_a_red_discard() {
    let run = |red: bool| {
        let mut g = two_player_game();
        let chandra = g.add_card_to_battlefield(0, catalog::chandra_ablaze());
        g.players[0].hand.clear();
        g.add_card_to_hand(0, if red { catalog::lightning_bolt() } else { catalog::island() });
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: chandra,
            ability_index: 0,
            target: Some(Target::Player(1)),
            x_value: None,
        })
        .expect("plus one");
        drain_stack(&mut g);
        g.players[1].life
    };
    assert_eq!(run(true), 16, "a red discard deals 4");
    assert_eq!(run(false), 20, "a blue one doesn't");
}

/// Nissa Revane fetches her Chosen, and Nissa's Chosen goes to the library
/// bottom rather than dying.
#[test]
fn nissa_revane_fetches_her_chosen() {
    let mut g = two_player_game();
    let nissa = g.add_card_to_battlefield(0, catalog::nissa_revane());
    let chosen = g.add_card_to_library(0, catalog::nissas_chosen());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(chosen))]));
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: nissa,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("plus one");
    drain_stack(&mut g);
    assert!(g.battlefield_find(chosen).is_some());

    let mut ev = vec![];
    g.destroy_permanent(chosen, false, &mut ev);
    g.check_state_based_actions();
    assert!(g.players[0].library.iter().any(|c| c.id == chosen), "bottomed, not buried");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == chosen));
}

/// Sorin Markov's −3 sets an opponent to 10.
#[test]
fn sorin_markov_minus_three_sets_life_to_ten() {
    let mut g = two_player_game();
    let sorin = g.add_card_to_battlefield(0, catalog::sorin_markov());
    g.battlefield_find_mut(sorin).unwrap().add_counters(CounterType::Loyalty, 4);
    g.players[1].life = 30;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: sorin,
        ability_index: 1,
        target: Some(Target::Player(1)),
        x_value: None,
    })
    .expect("minus three");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 10);
}
