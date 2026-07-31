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

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// The cycling cards all carry Cycling {2}.
#[test]
fn uds_cycling_cards_carry_cycling_two() {
    for f in [
        catalog::flame_jet as fn() -> crabomination::card::CardDefinition,
        catalog::fend_off,
        catalog::rapid_decay,
    ] {
        let def = f();
        assert!(
            def.keywords.iter().any(|k| matches!(k, Keyword::Cycling(c) if c.cmc() == 2)),
            "{} is missing Cycling {{2}}",
            def.name
        );
    }
}

/// Fatigue eats the target's next draw step.
#[test]
fn fatigue_skips_the_next_draw_step() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::fatigue());
    mana(&mut g, 0);
    cast(&mut g, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].skip_next_draw_step, 1);
    g.active_player_idx = 1;
    g.turn_number = 3;
    g.step = TurnStep::Upkeep;
    g.advance_step(vec![]).expect("advance");
    assert!(g.players[1].hand.is_empty(), "the draw was skipped");
    assert_eq!(g.players[1].skip_next_draw_step, 0, "and the charge was spent");
}

/// Wake of Destruction takes every land sharing the target's name.
#[test]
fn wake_of_destruction_takes_every_land_of_that_name() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::mountain());
    let b = g.add_card_to_battlefield(1, catalog::mountain());
    let island = g.add_card_to_battlefield(1, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::wake_of_destruction());
    mana(&mut g, 0);
    cast(&mut g, spell, Some(Target::Permanent(a)));
    assert!(g.battlefield_find(a).is_none());
    assert!(g.battlefield_find(b).is_none());
    assert!(g.battlefield_find(island).is_some());
}

/// Goblin Marshal brings two Goblins both coming and going.
#[test]
fn goblin_marshal_makes_goblins_twice() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::goblin_marshal());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    let goblins = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count()
    };
    assert_eq!(goblins(&g), 2);
    let marshal =
        g.battlefield.iter().find(|c| c.definition.name == "Goblin Marshal").unwrap().id;
    let ctx = crabomination::game::effects::EffectContext::for_ability(marshal, 0, None);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::This },
            &ctx,
        )
        .expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(goblins(&g), 4);
}

/// Phyrexian Negator pays a permanent for every point it takes.
#[test]
fn phyrexian_negator_sacrifices_per_damage() {
    let mut g = two_player_game();
    let negator = g.add_card_to_battlefield(0, catalog::phyrexian_negator());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let mut ev = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(negator),
        2,
        None,
        &mut ev,
    );
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), before - 2);
}

/// Bloodshot Cyclops flings the sacrificed creature's power.
#[test]
fn bloodshot_cyclops_flings_a_creature() {
    let mut g = two_player_game();
    let cyclops = g.add_card_to_battlefield(0, catalog::bloodshot_cyclops());
    g.battlefield_find_mut(cyclops).unwrap().summoning_sick = false;
    g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let life = g.players[1].life;
    activate(&mut g, cyclops, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 3);
}

/// Thran Golem suits up only while something is attached to it.
#[test]
fn thran_golem_grows_while_enchanted() {
    let mut g = two_player_game();
    let golem = g.add_card_to_battlefield(0, catalog::thran_golem());
    let cp = g.computed_permanent(golem).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    let aura = g.add_card_to_hand(0, catalog::twisted_experiment());
    mana(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(golem)));
    let cp = g.computed_permanent(golem).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 4), "+3/-1 from the Aura, +2/+2 from itself");
    assert!(cp.keywords.contains(&Keyword::Flying));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Treachery steals the enchanted creature and untaps five lands.
#[test]
fn treachery_steals_and_untaps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..6 {
        let land = g.add_card_to_battlefield(0, catalog::island());
        g.battlefield_find_mut(land).unwrap().tapped = true;
    }
    let aura = g.add_card_to_hand(0, catalog::treachery());
    mana(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land() && !c.tapped).count(),
        5
    );
}

/// Archery Training's arrow counters arm the enchanted creature.
#[test]
fn archery_training_shoots_for_its_arrow_counters() {
    let mut g = two_player_game();
    let archer = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(archer).unwrap().summoning_sick = false;
    let aura = g.add_card_to_hand(0, catalog::archery_training());
    mana(&mut g, 0);
    cast(&mut g, aura, Some(Target::Permanent(archer)));
    let aura =
        g.battlefield.iter().find(|c| c.definition.name == "Archery Training").unwrap().id;
    g.battlefield_find_mut(aura).unwrap().add_counters(CounterType::Arrow, 2);
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking.push(crabomination::game::types::Attack {
        attacker,
        target: crabomination::game::types::AttackTarget::Player(0),
    });
    activate(&mut g, archer, 0, Some(Target::Permanent(attacker)));
    assert!(g.battlefield_find(attacker).is_none(), "two arrows killed the 2/2");
}

/// Compost draws off black cards hitting an opponent's graveyard.
#[test]
fn compost_draws_off_opposing_black_cards() {
    let mut g = two_player_game();
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.add_card_to_battlefield(0, catalog::compost());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::phyrexian_negator()); // black
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Mill {
                who: crabomination::effect::Selector::Player(
                    crabomination::effect::PlayerRef::Seat(1),
                ),
                amount: crabomination::effect::Value::ONE,
            },
            &ctx,
        )
        .expect("mill");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}

// ── Wave 3: the last of the set ─────────────────────────────────────────────

fn say_yes(g: &mut GameState) {
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
}

/// Academy Rector exiles itself to fetch an enchantment onto the battlefield.
#[test]
fn academy_rector_trades_itself_for_an_enchantment() {
    let mut g = two_player_game();
    let rector = g.add_card_to_battlefield(0, catalog::academy_rector());
    let fetched = g.add_card_to_library(0, catalog::attrition());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Search(Some(fetched)),
    ]));
    let ctx = crabomination::game::effects::EffectContext::for_ability(rector, 0, None);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::This },
            &ctx,
        )
        .expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.definition.name == "Academy Rector"));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Attrition"));
}

/// Gamekeeper digs past the chaff for the first creature.
#[test]
fn gamekeeper_reanimates_off_the_top() {
    let mut g = two_player_game();
    say_yes(&mut g);
    let keeper = g.add_card_to_battlefield(0, catalog::gamekeeper());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::hill_giant());
    let ctx = crabomination::game::effects::EffectContext::for_ability(keeper, 0, None);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::This },
            &ctx,
        )
        .expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Hill Giant"));
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Island"));
}

/// Body Snatcher exiles itself on arrival when there's no creature to pitch.
#[test]
fn body_snatcher_exiles_itself_without_a_creature_to_discard() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::body_snatcher());
    mana(&mut g, 0);
    cast(&mut g, spell, None);
    assert!(g.exile.iter().any(|c| c.definition.name == "Body Snatcher"));
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Body Snatcher"));
}

/// Iridescent Drake arrives wearing an Aura off the graveyard.
#[test]
fn iridescent_drake_wears_a_dead_aura() {
    let mut g = two_player_game();
    let aura = g.add_card_to_graveyard(0, catalog::twisted_experiment());
    let drake = g.add_card_to_battlefield(0, catalog::iridescent_drake());
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        drake,
        0,
        Some(Target::Permanent(aura)),
    );
    let def = catalog::iridescent_drake();
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).expect("etb");
    assert_eq!(g.battlefield_find(aura).unwrap().attached_to, Some(drake));
    let cp = g.computed_permanent(drake).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 1), "+3/-1 from the Aura");
}

/// Bubbling Muck doubles Swamps for the turn and stops at cleanup.
#[test]
fn bubbling_muck_doubles_swamps_for_the_turn() {
    let mut g = two_player_game();
    let swamp = g.add_card_to_battlefield(0, catalog::swamp());
    let spell = g.add_card_to_hand(0, catalog::bubbling_muck());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast(&mut g, spell, None);
    g.players[0].mana_pool = Default::default();
    activate(&mut g, swamp, 0, None);
    assert_eq!(g.players[0].mana_pool.total(), 2, "the Swamp paid twice");
}

/// Storage Matrix lets a player untap only one card type per untap step.
#[test]
fn storage_matrix_limits_the_untap_step_to_one_type() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::storage_matrix());
    let land = g.add_card_to_battlefield(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land2 = g.add_card_to_battlefield(0, catalog::island());
    for id in [land, bear, land2] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g.active_player_idx = 0;
    g.do_untap();
    assert!(!g.battlefield_find(land).unwrap().tapped, "lands were the biggest pile");
    assert!(!g.battlefield_find(land2).unwrap().tapped);
    assert!(g.battlefield_find(bear).unwrap().tapped, "the creature stayed down");
}

/// Goblin Festival pings, and a lost flip hands it to an opponent.
#[test]
fn goblin_festival_changes_hands_on_a_lost_flip() {
    let mut g = two_player_game();
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(false), // lose the flip
    ]));
    let festival = g.add_card_to_battlefield(0, catalog::goblin_festival());
    mana(&mut g, 0);
    let life = g.players[1].life;
    activate(&mut g, festival, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 1);
    assert_eq!(g.battlefield_find(festival).unwrap().controller, 1);
}
