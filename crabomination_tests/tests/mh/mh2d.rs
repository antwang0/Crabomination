//! Functionality tests for `catalog::sets::decks::mh2d` — MH2 sweep batch 5.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn resolve_spell(g: &mut GameState, def: crabomination::card::CardDefinition, targets: Vec<Target>) {
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = targets;
    let events = g.resolve_effect(&def.effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
}

fn activate(g: &mut GameState, id: crabomination::card::CardId, idx: usize, target: Option<Target>) {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: idx, target, additional_targets: vec![], x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}


fn destroy(g: &mut GameState, id: crabomination::card::CardId) {
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(id)];
    let events = g
        .resolve_effect(
            &crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// Burdened Aerialist gains flying when a token is sacrificed, not a card.
#[test]
fn burdened_aerialist_token_sac() {
    let mut g = two_player_game();
    let pirate = g.add_card_to_battlefield(0, catalog::burdened_aerialist());
    g.add_card_to_battlefield(0, catalog::parcel_myr());
    // Sacrificing the nontoken Myr: no flying.
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let sac = crabomination::effect::Effect::Sacrifice {
        who: crabomination::effect::Selector::You,
        count: crabomination::effect::Value::ONE,
        filter: crabomination::card::SelectionRequirement::Artifact,
    };
    let events = g.resolve_effect(&sac, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(!g.computed_permanent(pirate).unwrap().keywords.contains(&Keyword::Flying));
    // Sacrificing a Treasure token: flying.
    resolve_spell(&mut g, catalog::crack_open(), vec![]); // stray Treasure? no target -> skip
    let treasure = crabomination::effect::shortcut::mint_treasures(1);
    let events = g.resolve_effect(&treasure, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    let events = g.resolve_effect(&sac, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.computed_permanent(pirate).unwrap().keywords.contains(&Keyword::Flying));
}

/// Combine Chrysalis grants creature tokens flying and trades one for a Beast.
#[test]
fn combine_chrysalis() {
    let mut g = two_player_game();
    let chrys = g.add_card_to_battlefield(0, catalog::combine_chrysalis());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(&crabomination::effect::shortcut::mint_treasures(1), &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    // A Treasure isn't a creature token — no flying grant on it.
    let squirrel_def = catalog::chatterfang_squirrel_general(); // any creature
    let bear = g.add_card_to_battlefield(0, squirrel_def);
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying),
        "nontoken creature unaffected");
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, chrys, 0, None);
    let beast = g.battlefield.iter().find(|c| c.definition.name == "Beast").expect("Beast");
    assert!(g.computed_permanent(beast.id).unwrap().keywords.contains(&Keyword::Flying),
        "creature token has flying");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "token paid");
}

/// Dihada's Ploy loots and pays life per discard this turn.
#[test]
fn dihadas_ploy_life_per_discard() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].cards_discarded_this_turn = 1; // one earlier discard
    let life = g.players[0].life;
    resolve_spell(&mut g, catalog::dihadas_ploy(), vec![]);
    assert_eq!(g.players[0].life, life + 2, "1 earlier + 1 from the spell");
    assert!(catalog::dihadas_ploy().keywords.contains(&Keyword::JumpStart));
}

/// Fae Offering pays out only after both a creature and a noncreature cast.
#[test]
fn fae_offering_gate() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fae_offering());
    g.active_player_idx = 0;
    g.players[0].creatures_cast_this_turn = 1;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Clue"), "creature only: no");
    g.players[0].noncreature_spells_cast_this_game_turn = 1;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    for name in ["Clue", "Food", "Treasure"] {
        assert!(g.battlefield.iter().any(|c| c.definition.name == name), "{name} minted");
    }
}

/// Flay Essence banks the counters before exiling.
#[test]
fn flay_essence_counter_life() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::glinting_creeper());
    g.battlefield_find_mut(target).unwrap().add_counters(CounterType::PlusOnePlusOne, 4);
    let life = g.players[0].life;
    resolve_spell(&mut g, catalog::flay_essence(), vec![Target::Permanent(target)]);
    assert!(g.battlefield_find(target).is_none(), "exiled");
    assert_eq!(g.players[0].life, life + 4, "4 counters = 4 life");
}

/// Gilt-Blade Prowler only draws once you've discarded.
#[test]
fn gilt_blade_prowler_gate() {
    let mut g = two_player_game();
    let prowler = g.add_card_to_battlefield(0, catalog::gilt_blade_prowler());
    g.clear_sickness(prowler);
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: prowler, ability_index: 0, target: None, additional_targets: vec![],
            x_value: None,
        })
        .is_err(),
        "locked before a discard"
    );
    g.players[0].discarded_this_turn.insert(crabomination::card::CardId(999_999));
    activate(&mut g, prowler, 0, None);
    assert_eq!(g.players[0].hand.len(), 1, "drew");
}

/// Glinting Creeper's converge counters double per color.
#[test]
fn glinting_creeper_converge() {
    let mut g = two_player_game();
    let creeper = g.add_card_to_hand(0, catalog::glinting_creeper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::White, 2);
    g.priority.player_with_priority = 0;
    cast(&mut g, creeper);
    let cp = g.computed_permanent(creeper).unwrap();
    // 4 colors spent → 8 counters (0/0 base).
    assert_eq!((cp.power, cp.toughness), (8, 8), "two counters per color");
}

/// Glorious Enforcer doubles up only while ahead on life.
#[test]
fn glorious_enforcer_life_gate() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::glorious_enforcer());
    g.players[1].life = 20;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(!g.computed_permanent(angel).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "tied life: no double strike");
    g.players[1].life = 10;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(g.computed_permanent(angel).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// Junk Winder's affinity counts tokens, and a token ETB stuns a permanent.
#[test]
fn junk_winder() {
    let mut g = two_player_game();
    assert_eq!(catalog::junk_winder().affinity_filter,
        Some(crabomination::card::SelectionRequirement::IsToken));
    g.add_card_to_battlefield(0, catalog::junk_winder());
    let their_land = g.add_card_to_battlefield(1, catalog::batterbone());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(&crabomination::effect::shortcut::mint_treasures(1), &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let c = g.battlefield_find(their_land).unwrap();
    assert!(c.tapped, "tapped");
    assert_eq!(c.counter_count(CounterType::Stun), 1, "stunned");
}

/// Lazotep Chancellor amasses on discard when the {1} is paid.
#[test]
fn lazotep_chancellor_amass() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lazotep_chancellor());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(
            &crabomination::effect::Effect::Discard {
                who: crabomination::effect::Selector::You,
                amount: crabomination::effect::Value::ONE,
                random: true,
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let army = g.battlefield.iter().find(|c| c.definition.name == "Army");
    assert!(army.is_some(), "Army minted");
    assert_eq!(army.unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "amass 2");
}

/// Lucid Dreams draws the delirium count.
#[test]
fn lucid_dreams_draws_card_types() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
    g.add_card_to_graveyard(0, catalog::island()); // land
    g.add_card_to_graveyard(0, catalog::lucid_dreams()); // sorcery
    resolve_spell(&mut g, catalog::lucid_dreams(), vec![]);
    assert_eq!(g.players[0].hand.len(), 3, "three card types → three draws");
}

/// Magus of the Bridge: your nontoken deaths mint Zombies; enemy deaths
/// exile it.
#[test]
fn magus_of_the_bridge() {
    let mut g = two_player_game();
    let magus = g.add_card_to_battlefield(0, catalog::magus_of_the_bridge());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    destroy(&mut g, mine);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Zombie"), "Zombie minted");
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    destroy(&mut g, theirs);
    assert!(g.battlefield_find(magus).is_none(), "magus exiled");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != magus), "exiled, not dead");
}

/// Mystic Redaction mills opponents on your discards.
#[test]
fn mystic_redaction_mills() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mystic_redaction());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::island());
    }
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(
            &crabomination::effect::Effect::Discard {
                who: crabomination::effect::Selector::You,
                amount: crabomination::effect::Value::ONE,
                random: false,
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 2, "opponent milled 2");
}

/// Necromancer's Familiar: hellbent lifelink flips with hand size.
#[test]
fn necromancers_familiar_hellbent() {
    let mut g = two_player_game();
    let bird = g.add_card_to_battlefield(0, catalog::necromancers_familiar());
    g.players[0].hand.clear();
    assert!(g.computed_permanent(bird).unwrap().keywords.contains(&Keyword::Lifelink));
    g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bird).unwrap().keywords.contains(&Keyword::Lifelink));
}

/// Nykthos Paragon converts a life gain into team counters, once a turn.
#[test]
fn nykthos_paragon_counters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nykthos_paragon());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(
            &crabomination::effect::Effect::GainLife {
                who: crabomination::effect::Selector::You,
                amount: crabomination::effect::Value::Const(3),
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "3 life → 3 counters"
    );
}

/// Prophetic Titan takes both modes under delirium.
#[test]
fn prophetic_titan_delirium_both() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::lucid_dreams());
    g.add_card_to_graveyard(0, catalog::batterbone());
    let titan = g.add_card_to_hand(0, catalog::prophetic_titan());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let opp_life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: titan,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 4, "bolt half fired");
    assert_eq!(g.players[0].hand.len(), 1, "dig half took a card");
}

/// Radiant Epicure drains for the converge count.
#[test]
fn radiant_epicure_converge_drain() {
    let mut g = two_player_game();
    let epicure = g.add_card_to_hand(0, catalog::radiant_epicure());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Red, 2);
    let my_life = g.players[0].life;
    let opp_life = g.players[1].life;
    g.priority.player_with_priority = 0;
    cast(&mut g, epicure);
    assert_eq!(g.players[1].life, opp_life - 4, "4 colors spent");
    assert_eq!(g.players[0].life, my_life + 4);
}

/// Raving Visionary's second ability is delirium-locked.
#[test]
fn raving_visionary_delirium_lock() {
    let mut g = two_player_game();
    let vis = g.add_card_to_battlefield(0, catalog::raving_visionary());
    g.clear_sickness(vis);
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: vis, ability_index: 1, target: None, additional_targets: vec![],
            x_value: None,
        })
        .is_err(),
        "no delirium yet"
    );
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::lucid_dreams());
    g.add_card_to_graveyard(0, catalog::batterbone());
    activate(&mut g, vis, 1, None);
    assert_eq!(g.players[0].hand.len(), 1, "delirium draw");
}

/// Recalibrate bounces, drawing only after a discard.
#[test]
fn recalibrate() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve_spell(&mut g, catalog::recalibrate(), vec![Target::Permanent(bear)]);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bounced to owner");
    assert!(g.players[0].hand.is_empty(), "no discard: no draw");
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].discarded_this_turn.insert(crabomination::card::CardId(999_999));
    resolve_spell(&mut g, catalog::recalibrate(), vec![Target::Permanent(bear2)]);
    assert_eq!(g.players[0].hand.len(), 1, "draw after a discard");
}

/// Revolutionist regrows an instant or sorcery on entry; madness registered.
#[test]
fn revolutionist_regrowth() {
    let mut g = two_player_game();
    assert!(catalog::revolutionist()
        .keywords
        .iter()
        .any(|k| matches!(k, Keyword::Madness(_))));
    let spell = g.add_card_to_graveyard(0, catalog::lucid_dreams());
    let rev = g.add_card_to_hand(0, catalog::revolutionist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, rev, Target::Permanent(spell));
    assert!(g.players[0].hand.iter().any(|c| c.id == spell), "regrown");
}

/// Sanctuary Raptor pumps only with three tokens back home.
#[test]
fn sanctuary_raptor_token_gate() {
    let mut g = two_player_game();
    let raptor = g.add_card_to_battlefield(0, catalog::sanctuary_raptor());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let events = g
        .resolve_effect(&crabomination::effect::shortcut::mint_treasures(3), &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&events);
    g.clear_sickness(raptor);
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: raptor,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(raptor).unwrap();
    assert_eq!(cp.power, 4, "+2/+0 with 3 tokens");
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// Scour the Desert converts a graveyard body into toughness-many Birds.
#[test]
fn scour_the_desert() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::serra_angel()); // 4/4
    resolve_spell(&mut g, catalog::scour_the_desert(), vec![Target::Permanent(dead)]);
    let birds = g.battlefield.iter().filter(|c| c.definition.name == "Bird").count();
    assert_eq!(birds, 4, "toughness 4 → 4 Birds");
    assert!(g.players[0].graveyard.is_empty(), "exiled");
}

/// Scuttletide's Crabs get delirium anthem.
#[test]
fn scuttletide_delirium_anthem() {
    let mut g = two_player_game();
    let tide = g.add_card_to_battlefield(0, catalog::scuttletide());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, tide, 0, None);
    let crab = g.battlefield.iter().find(|c| c.definition.name == "Crab").unwrap().id;
    assert_eq!(g.computed_permanent(crab).unwrap().toughness, 3, "no delirium: 0/3");
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::lucid_dreams());
    g.add_card_to_graveyard(0, catalog::batterbone());
    let cp = g.computed_permanent(crab).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 4), "delirium anthem");
}

/// Skyblade's Boon returns to hand from the battlefield and the graveyard.
#[test]
fn skyblades_boon_recursion() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::skyblades_boon());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, dead, 1, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "back from the graveyard");
}

/// Smell Fear proliferates before the fight.
#[test]
fn smell_fear_proliferate_then_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::glinting_creeper());
    g.battlefield_find_mut(mine).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    resolve_spell(
        &mut g,
        catalog::smell_fear(),
        vec![Target::Permanent(mine), Target::Permanent(theirs)],
    );
    g.check_state_based_actions();
    // Proliferate bumps 3 → 4 counters, so the 4/4s trade.
    assert!(g.battlefield_find(theirs).is_none(), "4 damage kills the Angel");
    assert!(g.battlefield_find(mine).is_none(), "Angel's 4 back kills the 4/4 creeper");
}

/// Specimen Collector mints its menagerie and copies a token on death.
#[test]
fn specimen_collector() {
    let mut g = two_player_game();
    let coll = g.add_card_to_hand(0, catalog::specimen_collector());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    cast(&mut g, coll);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Squirrel"));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Crab"));
    destroy(&mut g, coll);
    let crabs = g.battlefield.iter().filter(|c| c.definition.name == "Crab").count()
        + g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count();
    assert_eq!(crabs, 3, "death copied one of the tokens");
}

/// Spreading Insurrection threatens with storm.
#[test]
fn spreading_insurrection_storm() {
    let mut g = two_player_game();
    assert!(catalog::spreading_insurrection().keywords.contains(&Keyword::Storm));
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    resolve_spell(&mut g, catalog::spreading_insurrection(), vec![Target::Permanent(theirs)]);
    let c = g.battlefield_find(theirs).unwrap();
    assert_eq!(c.controller, 0, "stolen");
    assert!(!c.tapped, "untapped");
}

/// Sweep the Skies mints converge-many Thopters.
#[test]
fn sweep_the_skies_converge() {
    let mut g = two_player_game();
    let sweep = g.add_card_to_hand(0, catalog::sweep_the_skies());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: sweep, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("cast for X=2");
    drain_stack(&mut g);
    let thopters = g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count();
    assert_eq!(thopters, 3, "three colors spent");
}

/// Tourach's Canticle takes a chosen card and a random one.
#[test]
fn tourachs_canticle_double_discard() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    resolve_spell(&mut g, catalog::tourachs_canticle(), vec![Target::Player(1)]);
    assert_eq!(g.players[1].hand.len(), 1, "chosen + random discard");
}

/// Unbounded Potential spreads counters across two targets.
#[test]
fn unbounded_potential_counters() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(a), Target::Permanent(b)];
    ctx.mode = 0;
    let def = catalog::unbounded_potential();
    let events = g.resolve_effect(&def.effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Sanctum Weaver taps for enchantment-many mana.
#[test]
fn sanctum_weaver_mana() {
    let mut g = two_player_game();
    let weaver = g.add_card_to_battlefield(0, catalog::sanctum_weaver());
    g.clear_sickness(weaver);
    g.add_card_to_battlefield(0, catalog::fae_offering());
    g.add_card_to_battlefield(0, catalog::mystic_redaction());
    // Weaver itself is an enchantment creature: 3 total.
    activate(&mut g, weaver, 0, None);
    assert_eq!(g.players[0].mana_pool.total(), 3, "X = enchantments you control");
}

/// Batch-5 stat spot checks.
#[test]
fn batch5_stats() {
    assert!(catalog::flourishing_strike().keywords.iter().any(|k| matches!(k, Keyword::Entwine(_))));
    assert!(catalog::unbounded_potential().keywords.iter().any(|k| matches!(k, Keyword::Entwine(_))));
    assert_eq!(catalog::sanctum_weaver().toughness, 2);
    assert_eq!(catalog::radiant_epicure().power, 5);
    assert!(catalog::gouged_zealot().keywords.contains(&Keyword::Reach));
    assert_eq!(catalog::magus_of_the_bridge().cost.cmc(), 3);
}
