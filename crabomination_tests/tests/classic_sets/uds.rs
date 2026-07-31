//! Urza's Destiny (UDS) gap closure.

use crabomination::card::{CardType, CounterType, Keyword};
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

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

/// Every UDS factory is registered under its printed name.
#[test]
fn uds_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::goblin_berserker as fn() -> crabomination::card::CardDefinition,
        catalog::masticore,
        catalog::metalworker,
        catalog::powder_keg,
        catalog::quash,
        catalog::replenish,
        catalog::yawgmoths_bargain,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// The printed keyword lines land on the vanilla-ish bodies.
#[test]
fn uds_keyword_bodies_carry_their_keywords() {
    for (f, kw) in [
        (catalog::goliath_beetle as fn() -> crabomination::card::CardDefinition, Keyword::Trample),
        (catalog::plated_spider, Keyword::Reach),
        (catalog::squirming_mass, Keyword::Fear),
        (catalog::elvish_lookout, Keyword::Shroud),
        (catalog::hulking_ogre, Keyword::CantBlock),
        (catalog::metathran_soldier, Keyword::Unblockable),
        (catalog::taunting_elf, Keyword::MustBeBlocked),
        (catalog::thorn_elemental, Keyword::AssignsDamageAsThoughUnblocked),
        (catalog::voice_of_duty, Keyword::Protection(Color::Green)),
        (catalog::voice_of_reason, Keyword::Protection(Color::Blue)),
    ] {
        let def = f();
        assert!(def.keywords.contains(&kw), "{} is missing {kw:?}", def.name);
    }
}

// ── The Scent / Seer cycle ──────────────────────────────────────────────────

/// Scent of Cinder scales with the red cards revealed from hand.
#[test]
fn scent_of_cinder_scales_with_revealed_red_cards() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::lightning_bolt()); // red
    }
    g.add_card_to_hand(0, catalog::counterspell()); // blue, not revealable
    let spell = g.add_card_to_hand(0, catalog::scent_of_cinder());
    mana(&mut g, 0);
    let life = g.players[1].life;
    cast(&mut g, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 3, "three red cards left in hand");
}

/// Jasmine Seer converts revealed white cards into two life apiece.
#[test]
fn jasmine_seer_pays_two_life_per_white_card() {
    let mut g = two_player_game();
    let seer = g.add_card_to_battlefield(0, catalog::jasmine_seer());
    g.battlefield_find_mut(seer).unwrap().summoning_sick = false;
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::swords_to_plowshares()); // white
    }
    mana(&mut g, 0);
    let life = g.players[0].life;
    activate(&mut g, seer, 0, None);
    assert_eq!(g.players[0].life, life + 4);
}

/// Metalworker turns revealed artifacts into two colorless each.
#[test]
fn metalworker_makes_two_mana_per_revealed_artifact() {
    let mut g = two_player_game();
    let worker = g.add_card_to_battlefield(0, catalog::metalworker());
    g.battlefield_find_mut(worker).unwrap().summoning_sick = false;
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::sol_ring());
    }
    activate(&mut g, worker, 0, None);
    assert_eq!(g.players[0].mana_pool.total(), 4);
}

// ── The name-hate cycle ─────────────────────────────────────────────────────

/// Eradicate exiles the target and every copy in its controller's other zones.
#[test]
fn eradicate_strips_every_copy_of_the_name() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::eradicate());
    mana(&mut g, 0);
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 4);
    assert_eq!(g.players[1].library.len(), 1, "the Hill Giant stays");
}

/// Quash lifts the spell off the stack and exiles the rest of its name.
#[test]
fn quash_counters_and_strips_the_name() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let quash = g.add_card_to_hand(0, catalog::quash());
    mana(&mut g, 0);
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Bolt");
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    cast(&mut g, quash, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, life, "the Bolt never resolved");
    assert_eq!(g.exile.iter().filter(|c| c.definition.name == "Lightning Bolt").count(), 2);
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Powder Keg sweeps artifacts and creatures at its fuse count and leaves the
/// rest of the board alone.
#[test]
fn powder_keg_sweeps_at_its_fuse_count() {
    let mut g = two_player_game();
    let keg = g.add_card_to_battlefield(0, catalog::powder_keg());
    g.battlefield_find_mut(keg).unwrap().add_counters(CounterType::Fuse, 2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant()); // MV 4
    activate(&mut g, keg, 0, None);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(giant).is_some());
}

/// Masticore eats a card each upkeep and dies when the hand is empty.
#[test]
fn masticore_starves_without_a_card_to_discard() {
    let mut g = two_player_game();
    let core = g.add_card_to_battlefield(0, catalog::masticore());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(core).is_none(), "no card to feed it");
}

/// Caltrops pricks every attacker, whoever declared it.
#[test]
fn caltrops_pricks_each_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::caltrops());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(attacker).unwrap().summoning_sick = false;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    let evs = g
        .declare_attackers(vec![crabomination::game::types::Attack {
            attacker,
            target: crabomination::game::types::AttackTarget::Player(0),
        }])
        .expect("attack");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(attacker).unwrap().damage, 1);
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Yawgmoth's Bargain skips your draw step and sells cards for life.
#[test]
fn yawgmoths_bargain_trades_the_draw_step_for_life() {
    let mut g = two_player_game();
    let bargain = g.add_card_to_battlefield(0, catalog::yawgmoths_bargain());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.turn_number = 3;
    g.step = TurnStep::Upkeep;
    g.advance_step(vec![]).expect("advance");
    assert_eq!(g.step, TurnStep::Draw);
    assert!(g.players[0].hand.is_empty(), "the draw step was skipped");
    let life = g.players[0].life;
    activate(&mut g, bargain, 0, None);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].life, life - 1);
}

/// Repercussion mirrors creature damage onto that creature's controller.
#[test]
fn repercussion_echoes_creature_damage_onto_its_controller() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::repercussion());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(bear),
        2,
        None,
        &mut ev,
    );
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
}

/// Aether Sting bleeds an opponent for each creature spell they cast.
#[test]
fn aether_sting_pings_creature_casters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aether_sting());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let life = g.players[1].life;
    cast(&mut g, bear, None);
    assert_eq!(g.players[1].life, life - 1);
}

/// Attrition turns a spare body into a dead nonblack creature.
#[test]
fn attrition_eats_a_body_to_kill_a_nonblack_creature() {
    let mut g = two_player_game();
    let attrition = g.add_card_to_battlefield(0, catalog::attrition());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::hill_giant());
    mana(&mut g, 0);
    activate(&mut g, attrition, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(fodder).is_none());
    assert!(g.battlefield_find(victim).is_none());
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Momentum's growth counters scale the host.
#[test]
fn momentum_scales_with_its_growth_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::momentum());
    mana(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    let aura = g.battlefield.iter().find(|c| c.definition.name == "Momentum").unwrap().id;
    g.battlefield_find_mut(aura).unwrap().add_counters(CounterType::Growth, 3);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Twisted Experiment is a straight +3/-1.
#[test]
fn twisted_experiment_trades_toughness_for_power() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::twisted_experiment());
    mana(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 1));
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Flicker blinks a permanent through exile and back under its owner.
#[test]
fn flicker_blinks_a_permanent_home() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::flicker());
    mana(&mut g, 0);
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    assert!(g.exile.iter().all(|c| c.definition.name != "Grizzly Bears"));
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count(),
        1,
        "it came right back"
    );
}

/// Donate hands a permanent to the targeted player.
#[test]
fn donate_gives_a_permanent_away() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::donate());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("cast Donate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 1);
}

/// Replenish rebuilds every enchantment in your graveyard.
#[test]
fn replenish_returns_every_graveyard_enchantment() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::attrition());
    g.add_card_to_graveyard(0, catalog::aether_sting());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::replenish());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_enchantment()).count(), 2);
    assert_eq!(g.players[0].graveyard.len(), 2, "the Bears and Replenish itself");
}

/// Multani's Decree wraths enchantments and pays two life each.
#[test]
fn multanis_decree_pays_two_life_per_enchantment() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::attrition());
    g.add_card_to_battlefield(1, catalog::aether_sting());
    let spell = g.add_card_to_hand(0, catalog::multanis_decree());
    mana(&mut g, 0);
    let life = g.players[0].life;
    cast(&mut g, spell, None);
    assert!(!g.battlefield.iter().any(|c| c.definition.is_enchantment()));
    assert_eq!(g.players[0].life, life + 4);
}

/// Emperor Crocodile eats itself once it's alone.
#[test]
fn emperor_crocodile_needs_company() {
    let mut g = two_player_game();
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let croc = g.add_card_to_battlefield(0, catalog::emperor_crocodile());
    g.check_state_based_actions();
    assert!(g.battlefield_find(croc).is_some());
    g.battlefield.retain(|c| c.id != friend);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(croc).is_none());
}

/// Lurking Jackals wakes up as a 3/2 once an opponent falls to ten.
#[test]
fn lurking_jackals_wakes_at_ten_life() {
    let mut g = two_player_game();
    let jackals = g.add_card_to_battlefield(0, catalog::lurking_jackals());
    assert!(!g.computed_permanent(jackals).unwrap().card_types.contains(&CardType::Creature));
    g.players[1].life = 12;
    g.adjust_life(1, -3);
    g.dispatch_triggers_for_events(&[GameEvent::LifeLost { player: 1, amount: 3 }]);
    drain_stack(&mut g);
    let cp = g.computed_permanent(jackals).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (3, 2));
}
