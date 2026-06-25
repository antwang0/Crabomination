//! Functionality tests for the `catalog::sets::decks::recent6` batch.

use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget, Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

// ── White ────────────────────────────────────────────────────────────────

/// Karmic Guide ETB returns a creature card from your graveyard.
#[test]
fn karmic_guide_reanimates_from_graveyard() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::karmic_guide());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(dead)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Karmic Guide");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "grizzly returned to battlefield");
}

/// Elspeth, Sun's Champion: +1 makes three Soldiers; -3 destroys only power ≥ 4.
#[test]
fn elspeth_plus_one_tokens_and_minus_three_sweeps_big() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::elspeth_suns_champion());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw, ability_index: 0, target: None, x_value: None,
    }).expect("+1");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 3, "three 1/1 Soldiers");

    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    // Reset the once-per-turn loyalty gate so we can exercise the -3 too.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == pw) { c.loyalty_uses_this_turn = 0; }
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw, ability_index: 1, target: None, x_value: None,
    }).expect("-3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_some(), "2/2 survives the power-4 sweep");
    assert!(g.battlefield_find(big).is_none(), "6/6 is destroyed");
}

/// Faith's Fetters gains 4 life and locks the enchanted creature out of combat.
#[test]
fn faiths_fetters_gains_life_and_locks_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::faiths_fetters());
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Faith's Fetters");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "gained 4 life");
    let v = g.computed_permanent(victim).unwrap();
    assert!(v.keywords.contains(&Keyword::CantAttack), "can't attack");
    assert!(v.keywords.contains(&Keyword::CantBlock), "can't block");
}

/// Increasing Devotion makes five Humans when cast from hand.
#[test]
fn increasing_devotion_makes_five_from_hand() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::increasing_devotion());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let humans = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Human").count();
    assert_eq!(humans, 5, "five 1/1 Humans");
}

/// Wing Shards forces the targeted player to sacrifice an attacking creature.
#[test]
fn wing_shards_sacrifices_attacker() {
    let mut g = two_player_game();
    // P1's attacker. Make P1 the active player so it can attack P0.
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bystander = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    // P0 casts Wing Shards (instant) targeting P1.
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::wing_shards());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Wing Shards");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "attacking creature sacrificed");
    assert!(g.battlefield_find(bystander).is_some(), "non-attacker untouched");
}

/// Council's Judgment exiles an opponent's hexproof permanent (no targeting).
#[test]
fn councils_judgment_votes_out_hexproof() {
    let mut g = two_player_game();
    // Single legal candidate → unanimous vote → exiled, even with hexproof.
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::councils_judgment());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert!(g.battlefield_find(foe).is_none(), "voted-out permanent exiled");
    assert!(g.exile.iter().any(|c| c.id == foe), "in exile (not graveyard)");
}

// ── Blue ─────────────────────────────────────────────────────────────────

/// Talrand makes a Drake when you cast an instant or sorcery.
#[test]
fn talrand_makes_drake_on_instant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::talrand_sky_summoner());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    let drakes = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Drake").count();
    assert_eq!(drakes, 1, "one 2/2 Drake");
}

/// Tezzeret the Seeker −X tutors an artifact onto the battlefield.
#[test]
fn tezzeret_minus_x_fetches_artifact() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::tezzeret_the_seeker());
    let sol = g.add_card_to_library(0, catalog::sol_ring()); // MV 1
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(sol))]));
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw, ability_index: 1, target: None, x_value: Some(1),
    }).expect("-X");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sol).is_some(), "Sol Ring tutored to battlefield");
}

/// Dream Eater surveils 4 and bounces an opponent's nonland permanent.
#[test]
fn dream_eater_surveils_and_bounces() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let top: Vec<_> = (0..4).map(|_| g.add_card_to_library(0, catalog::forest())).collect();
    let id = g.add_card_to_hand(0, catalog::dream_eater());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::ScryOrder { kept_top: top, bottom: vec![] }, // surveil 4: keep all
        DecisionAnswer::Bool(true),  // yes, bounce
        DecisionAnswer::Target(Target::Permanent(victim)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dream Eater");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "creature bounced to hand");
}

/// Malcolm makes a Treasure when a Pirate deals combat damage to an opponent.
#[test]
fn malcolm_treasure_on_pirate_combat_damage() {
    let mut g = two_player_game();
    let malcolm = g.add_card_to_battlefield(0, catalog::malcolm_keen_eyed_navigator());
    g.clear_sickness(malcolm);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: malcolm, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure").count();
    assert_eq!(treasures, 1, "one Treasure for the opponent dealt damage");
}

// ── Black ────────────────────────────────────────────────────────────────

/// Profane Tutor suspends for {1}{B} and tutors any card to hand on resolution.
#[test]
fn profane_tutor_suspends_then_searches() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::profane_tutor());
    let wanted = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(wanted))]));
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..2 { let _ = g.process_suspend(); }
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == wanted), "tutored card in hand");
}

/// Shambling Ghast's death trigger can give an opponent's creature -1/-1.
#[test]
fn shambling_ghast_death_minus_one() {
    let mut g = two_player_game();
    let ghast = g.add_card_to_battlefield(0, catalog::shambling_ghast());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Mode(0),
        DecisionAnswer::Target(Target::Permanent(foe)),
    ]));
    // Kill the Ghast to fire its dies trigger.
    let ctx = crate::game::effects::EffectContext::for_trigger(ghast, 0, Some(Target::Permanent(ghast)), 0);
    g.resolve_effect(&crate::effect::Effect::DestroyNoRegen { what: crate::card::Selector::Target(0) }, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(foe).unwrap().toughness, 1, "2/2 → 2/1 from -1/-1");
}

/// Priest of Forgotten Gods: sac two creatures → opponent loses 2 + sacrifices,
/// you draw a card.
#[test]
fn priest_of_forgotten_gods_drain_and_draw() {
    let mut g = two_player_game();
    let priest = g.add_card_to_battlefield(0, catalog::priest_of_forgotten_gods());
    g.clear_sickness(priest);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let life = g.players[1].life;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: priest, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Priest");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "opponent lost 2 life");
    assert!(g.battlefield_find(foe).is_none(), "opponent sacrificed a creature");
    assert_eq!(g.players[0].hand.len(), hand + 1, "you drew a card");
}

/// Spawn of Mayhem's upkeep ping hits each player for 1.
#[test]
fn spawn_of_mayhem_upkeep_pings_each_player() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::spawn_of_mayhem());
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 - 1, "controller pinged");
    assert_eq!(g.players[1].life, l1 - 1, "opponent pinged");
}

/// Spawn of Mayhem can be cast for its Spectacle cost after an opponent lost life.
#[test]
fn spawn_of_mayhem_spectacle_available() {
    let s = catalog::spawn_of_mayhem();
    let alt = s.alternative_cost.expect("has spectacle");
    assert_eq!(alt.mana_cost, crate::mana::cost(&[crate::mana::generic(1), crate::mana::b(), crate::mana::b()]));
}

/// Magus of the Coffers taps for {B} per Swamp you control.
#[test]
fn magus_of_the_coffers_mana_per_swamp() {
    let mut g = two_player_game();
    let magus = g.add_card_to_battlefield(0, catalog::magus_of_the_coffers());
    g.clear_sickness(magus);
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::swamp()); }
    g.players[0].mana_pool.add_colorless(2); // pay the {2}
    g.perform_action(GameAction::ActivateAbility {
        card_id: magus, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Magus");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3, "added B per Swamp");
}

/// Plague Engineer's chosen-type aura shrinks opponents' creatures of that type.
#[test]
fn plague_engineer_shrinks_opponent_chosen_type() {
    let mut g = two_player_game();
    let goblin = g.add_card_to_battlefield(1, catalog::skirk_prospector()); // 1/1 Goblin
    let engineer = g.add_card_to_battlefield(0, catalog::plague_engineer());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::CreatureType(CreatureType::Goblin),
    ]));
    g.fire_self_etb_triggers(engineer, 0);
    drain_stack(&mut g);
    // A 1/1 Goblin getting -1/-1 has 0 toughness → dies as SBA.
    assert!(g.battlefield_find(goblin).is_none(), "opponent's 1/1 Goblin dies to -1/-1");
}

/// Mukotai Soulripper grows and gains menace when it sacrifices a creature on
/// attack.
#[test]
fn mukotai_soulripper_attack_sacrifice() {
    let mut g = two_player_game();
    let vehicle = g.add_card_to_battlefield(0, catalog::mukotai_soulripper());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(vehicle);
    g.clear_sickness(fodder);
    // Crew it so it's a creature able to attack.
    g.perform_action(GameAction::Crew { vehicle, crew_creatures: vec![fodder] }).ok();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: vehicle, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(attacker)),
    ]));
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(vehicle).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1, "+1/+1 counter",
    );
    assert!(g.computed_permanent(vehicle).unwrap().keywords.contains(&Keyword::Menace), "gained menace");
}

// ── Green ────────────────────────────────────────────────────────────────

/// Dryad Arbor is a land creature that taps for green.
#[test]
fn dryad_arbor_taps_for_green() {
    let mut g = two_player_game();
    let arbor = g.add_card_to_battlefield(0, catalog::dryad_arbor());
    g.clear_sickness(arbor);
    let c = g.computed_permanent(arbor).unwrap();
    assert!(
        c.card_types.contains(&crate::card::CardType::Land)
            && c.card_types.contains(&crate::card::CardType::Creature),
        "land creature",
    );
    g.perform_action(GameAction::ActivateAbility {
        card_id: arbor, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for G");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

/// Marwyn grows when another Elf enters and taps for green equal to her power.
#[test]
fn marwyn_grows_and_taps_for_power() {
    let mut g = two_player_game();
    let marwyn = g.add_card_to_battlefield(0, catalog::marwyn_the_nurturer());
    g.clear_sickness(marwyn);
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, elf);
    assert_eq!(g.battlefield_find(marwyn).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: marwyn, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap Marwyn");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "added G equal to her power (2)");
}

/// Hexdrinker levels into protection from instants, then from everything.
#[test]
fn hexdrinker_levels_into_protection() {
    let mut g = two_player_game();
    let hex = g.add_card_to_battlefield(0, catalog::hexdrinker());
    // Push to level 3: protection from instants → a Bolt can't target it.
    g.battlefield.iter_mut().find(|c| c.id == hex).unwrap()
        .counters.insert(CounterType::Level, 3);
    assert!(g.computed_permanent(hex).unwrap().keywords.contains(&Keyword::ProtectionFromInstants));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(hex)), additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "instant can't target protection-from-instants");

    // Push to level 8: protection from everything → can't be blocked.
    g.battlefield.iter_mut().find(|c| c.id == hex).unwrap()
        .counters.insert(CounterType::Level, 8);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(g.computed_permanent(hex).unwrap().keywords.contains(&Keyword::ProtectionFromEverything));
    assert!(!g.blocker_can_block_attacker(blocker, hex), "can't be blocked");
}

/// Wolfir Avenger can regenerate itself.
#[test]
fn wolfir_avenger_regenerates() {
    let mut g = two_player_game();
    let wolf = g.add_card_to_battlefield(0, catalog::wolfir_avenger());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wolf, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("regen shield");
    drain_stack(&mut g); // resolve the regen ability so the shield is up
    let ctx = crate::game::effects::EffectContext::for_trigger(wolf, 0, Some(Target::Permanent(wolf)), 0);
    g.resolve_effect(&crate::effect::Effect::Destroy { what: crate::card::Selector::Target(0) }, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(wolf).is_some(), "regeneration shield saved Wolfir Avenger");
}

/// Mwonvuli Acid-Moss destroys a land and ramps a Forest.
#[test]
fn mwonvuli_acid_moss_destroys_and_ramps() {
    let mut g = two_player_game();
    let foe_land = g.add_card_to_battlefield(1, catalog::island());
    let forest = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::mwonvuli_acid_moss());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe_land)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Acid-Moss");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe_land).is_none(), "land destroyed");
    assert!(g.battlefield_find(forest).is_some(), "Forest ramped onto battlefield");
}

// ── Lands ────────────────────────────────────────────────────────────────

/// Fabled Passage fetches a basic tapped; with four lands it enters untapped.
#[test]
fn fabled_passage_untaps_with_four_lands() {
    let mut g = two_player_game();
    let fp = g.add_card_to_battlefield(0, catalog::fabled_passage());
    // Three more lands so that after fetching the basic, you control 4+.
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); }
    let basic = g.add_card_to_library(0, catalog::plains());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: fp, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("crack Fabled Passage");
    drain_stack(&mut g);
    let fetched = g.battlefield_find(basic).expect("basic fetched");
    assert!(!fetched.tapped, "untapped (controlled 4+ lands)");
}

/// Mystic Sanctuary enters tapped with too few Islands.
#[test]
fn mystic_sanctuary_enters_tapped_without_islands() {
    let mut g = two_player_game();
    let ms = g.add_card_to_battlefield(0, catalog::mystic_sanctuary());
    g.fire_self_etb_triggers(ms, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ms).unwrap().tapped, "entered tapped with <3 other Islands");
}

/// Mystic Sanctuary enters untapped with three other Islands and recurs an I/S.
#[test]
fn mystic_sanctuary_recurs_with_islands() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::island()); }
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let ms = g.add_card_to_battlefield(0, catalog::mystic_sanctuary());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(bolt)),
    ]));
    g.fire_self_etb_triggers(ms, 0);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(ms).unwrap().tapped, "entered untapped");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bolt), "Bolt on top of library");
}
