#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;
use crate::altars_flips_artifacts::kill_with_bolt;

// ── claude/modern_decks: aggro / tribal / aristocrat supplement tests ─────────

#[test]
fn kird_ape_grows_with_a_forest() {
    let mut g = two_player_game();
    let ape = g.add_card_to_battlefield(0, catalog::kird_ape());
    let cp = g.computed_permanent(ape).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "1/1 with no Forest");
    g.add_card_to_battlefield(0, catalog::forest());
    let cp = g.computed_permanent(ape).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "+1/+2 with a Forest");
}

#[test]
fn loam_lion_grows_with_a_forest() {
    let mut g = two_player_game();
    let lion = g.add_card_to_battlefield(0, catalog::loam_lion());
    g.add_card_to_battlefield(0, catalog::forest());
    let cp = g.computed_permanent(lion).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3));
}

#[test]
fn sedge_troll_grows_with_a_swamp() {
    let mut g = two_player_game();
    let troll = g.add_card_to_battlefield(0, catalog::sedge_troll());
    let cp = g.computed_permanent(troll).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    g.add_card_to_battlefield(0, catalog::swamp());
    let cp = g.computed_permanent(troll).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

#[test]
fn young_wolf_returns_with_a_counter_via_undying() {
    let mut g = two_player_game();
    let wolf = g.add_card_to_battlefield(0, catalog::young_wolf());
    kill_with_bolt(&mut g, wolf);
    // Undying: returns as a fresh object with a +1/+1 counter (2/2).
    let back = g.battlefield.iter().find(|c| c.definition.name == "Young Wolf");
    assert!(back.is_some(), "Young Wolf returns via undying");
    let cp = g.computed_permanent(back.unwrap().id).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

#[test]
fn servant_of_the_scale_moves_counters_on_death() {
    let mut g = two_player_game();
    let servant = g.add_card_to_battlefield(0, catalog::servant_of_the_scale());
    g.fire_self_etb_triggers(servant, 0);
    drain_stack(&mut g); // ETB counter
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(servant).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "enters with a +1/+1 counter");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(target))]));
    kill_with_bolt(&mut g, servant);
    let cp = g.computed_permanent(target).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "counter moved to the bear");
}

#[test]
fn champion_of_the_perished_grows_on_zombie_etb() {
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::champion_of_the_perished());
    let cp = g.computed_permanent(champ).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    // Another Zombie enters → +1/+1.
    let champ2 = g.add_card_to_battlefield(0, catalog::champion_of_the_perished());
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::PermanentEntered { card_id: champ2 }]);
    drain_stack(&mut g);
    let cp = g.computed_permanent(champ).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "another Zombie ETB grew the Champion");
}

#[test]
fn furnace_whelp_firebreathes_and_flies() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let whelp = g.add_card_to_battlefield(0, catalog::furnace_whelp());
    assert!(g.computed_permanent(whelp).unwrap().keywords.contains(&Keyword::Flying));
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: whelp, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("firebreathe");
    drain_stack(&mut g);
    let cp = g.computed_permanent(whelp).unwrap();
    assert_eq!(cp.power, 3, "{{R}} gave +1/+0");
}


#[test]
fn falkenrath_noble_drains_on_any_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::falkenrath_noble());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    kill_with_bolt(&mut g, victim);
    assert_eq!(g.players[0].life, l0 + 1, "you gain 1");
    assert_eq!(g.players[1].life, l1 - 1, "target player loses 1");
}

#[test]
fn carrier_thrall_makes_a_scion_on_death() {
    let mut g = two_player_game();
    let thrall = g.add_card_to_battlefield(0, catalog::carrier_thrall());
    kill_with_bolt(&mut g, thrall);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Eldrazi Scion"),
        "dying made an Eldrazi Scion");
}

#[test]
fn blood_seeker_drains_on_opponent_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::blood_seeker());
    let l1 = g.players[1].life;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "opponent's creature ETB drained them 1");
}

#[test]
fn vicious_conquistador_drains_on_attack() {
    let mut g = two_player_game();
    let con = g.add_card_to_battlefield(0, catalog::vicious_conquistador());
    g.battlefield.iter_mut().find(|c| c.id == con).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let l1 = g.players[1].life;
    g.declare_attackers(vec![Attack { attacker: con, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1);
}

#[test]
fn child_of_night_has_lifelink() {
    use crabomination::card::Keyword;
    assert!(catalog::child_of_night().keywords.contains(&Keyword::Lifelink));
}

#[test]
fn vampire_interloper_flies_and_cant_block() {
    use crabomination::card::Keyword;
    let def = catalog::vampire_interloper();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::CantBlock));
}

#[test]
fn bartizan_bats_is_a_three_one_flyer() {
    use crabomination::card::Keyword;
    let def = catalog::bartizan_bats();
    assert_eq!((def.power, def.toughness), (3, 1));
    assert!(def.keywords.contains(&Keyword::Flying));
}

#[test]
fn ajanis_pridemate_grows_on_lifegain() {
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::ajanis_pridemate());
    g.adjust_life(0, 3);
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::LifeGained { player: 0, amount: 3 }]);
    drain_stack(&mut g);
    let cp = g.computed_permanent(cat).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "one +1/+1 counter per lifegain event");
}

#[test]
fn souls_attendant_gains_life_on_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::souls_attendant());
    let l0 = g.players[0].life;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 + 1);
}

#[test]
fn auriok_champion_has_protection_from_black_and_red() {
    use crabomination::card::Keyword;
    let def = catalog::auriok_champion();
    assert!(def.keywords.contains(&Keyword::Protection(Color::Black)));
    assert!(def.keywords.contains(&Keyword::Protection(Color::Red)));
}

#[test]
fn voice_of_the_blessed_flies_at_four_counters() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    let voice = g.add_card_to_battlefield(0, catalog::voice_of_the_blessed());
    assert!(!g.computed_permanent(voice).unwrap().keywords.contains(&Keyword::Flying));
    g.battlefield.iter_mut().find(|c| c.id == voice).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 4);
    let kws = g.computed_permanent(voice).unwrap().keywords.clone();
    assert!(kws.contains(&Keyword::Flying) && kws.contains(&Keyword::Vigilance),
        "4+ counters grant flying and vigilance");
}

#[test]
fn scorch_spitter_pings_on_attack() {
    let mut g = two_player_game();
    let spitter = g.add_card_to_battlefield(0, catalog::scorch_spitter());
    g.battlefield.iter_mut().find(|c| c.id == spitter).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let l1 = g.players[1].life;
    g.declare_attackers(vec![Attack { attacker: spitter, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "1 damage to the defending player on attack");
}

#[test]
fn faerie_miscreant_draws_with_a_twin() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::faerie_miscreant());
    let before = g.players[0].hand.len();
    // Second copy ETB sees the first → draw.
    let m2 = g.add_card_to_battlefield(0, catalog::faerie_miscreant());
    g.fire_self_etb_triggers(m2, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "second Miscreant drew a card");
}

#[test]
fn moan_of_the_unhallowed_makes_two_zombies_and_has_flashback() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::moan_of_the_unhallowed());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Moan");
    drain_stack(&mut g);
    let zombies = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Zombie").count();
    assert_eq!(zombies, 2);
    assert!(catalog::moan_of_the_unhallowed().keywords.iter()
        .any(|k| matches!(k, Keyword::Flashback(_))));
}

// ── claude/modern_decks: beaters batch 2 tests ────────────────────────────────

#[test]
fn batch2_vanilla_and_keyword_bodies() {
    use crabomination::card::Keyword;
    assert_eq!({ let d = catalog::canyon_minotaur(); (d.power, d.toughness) }, (3, 3));
    assert_eq!({ let d = catalog::runeclaw_bear(); (d.power, d.toughness) }, (2, 2));
    assert_eq!({ let d = catalog::alpine_grizzly(); (d.power, d.toughness) }, (4, 2));
    assert_eq!({ let d = catalog::pheres_band_centaurs(); (d.power, d.toughness) }, (3, 7));
    assert_eq!({ let d = catalog::axebane_stag(); (d.power, d.toughness) }, (6, 7));
    assert!(catalog::brazen_scourge().keywords.contains(&Keyword::Haste));
    assert!(catalog::boggart_brute().keywords.contains(&Keyword::Menace));
    assert!(catalog::colossal_dreadmaw().keywords.contains(&Keyword::Trample));
    assert!(catalog::snapping_drake().keywords.contains(&Keyword::Flying));
}

#[test]
fn torch_fiend_sacs_to_destroy_an_artifact() {
    let mut g = two_player_game();
    let fiend = g.add_card_to_battlefield(0, catalog::torch_fiend());
    let art = g.add_card_to_battlefield(1, catalog::worn_powerstone());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fiend, ability_index: 0,
        target: Some(Target::Permanent(art)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Torch Fiend");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    assert!(g.battlefield_find(fiend).is_none(), "Torch Fiend was sacrificed");
}

#[test]
fn goblin_arsonist_pings_on_death() {
    let mut g = two_player_game();
    let arsonist = g.add_card_to_battlefield(0, catalog::goblin_arsonist());
    let l1 = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Player(1)),
    ]));
    kill_with_bolt(&mut g, arsonist);
    assert_eq!(g.players[1].life, l1 - 1, "death ping hit the opponent for 1");
}

#[test]
fn garruks_packleader_draws_on_big_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::garruks_packleader());
    let before = g.players[0].hand.len();
    // A 5/5 (power ≥ 3) entering triggers the draw.
    let ape = g.add_card_to_battlefield(0, catalog::silverback_ape());
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::PermanentEntered { card_id: ape }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "drew off the big creature ETB");
}

#[test]
fn wing_snare_destroys_only_flyers() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::snapping_drake());
    let id = g.add_card_to_hand(0, catalog::wing_snare());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(flyer)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wing Snare a flyer");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flyer).is_none(), "the flyer is destroyed");
}

// ── claude/modern_decks: beaters batch 3 tests ────────────────────────────────

#[test]
fn batch3_bodies_and_keywords() {
    use crabomination::card::Keyword;
    assert_eq!({ let d = catalog::coral_eel(); (d.power, d.toughness) }, (2, 1));
    assert_eq!({ let d = catalog::horned_turtle(); (d.power, d.toughness) }, (1, 4));
    assert_eq!({ let d = catalog::grizzled_outrider(); (d.power, d.toughness) }, (5, 5));
    assert_eq!({ let d = catalog::walking_corpse(); (d.power, d.toughness) }, (2, 2));
    assert!(catalog::highborn_ghoul().keywords.contains(&Keyword::Intimidate));
    assert!(catalog::mist_raven().keywords.contains(&Keyword::Flying));
}

#[test]
fn mist_raven_bounces_a_creature_on_etb() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let raven = g.add_card_to_battlefield(0, catalog::mist_raven());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
    g.fire_self_etb_triggers(raven, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "the bear was bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "bear is back in its owner's hand");
}

// ── Spellseeker / Mystic Snake / Fauna Shaman (this branch) ─────────────────

/// Spellseeker's ETB tutors an instant/sorcery with MV ≤ 2 into hand.
#[test]
fn spellseeker_etb_fetches_cheap_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // {R}, MV 1
    g.add_card_to_library(0, catalog::grizzly_bears()); // creature — not a legal fetch
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));
    let ss = g.add_card_to_battlefield(0, catalog::spellseeker());
    g.fire_self_etb_triggers(ss, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "fetched the cheap instant to hand");
}

/// Mystic Snake's ETB counters a spell on the stack.
#[test]
fn mystic_snake_etb_counters_target_spell() {
    let mut g = two_player_game();
    // P1 casts a spell on their own main phase; it sits on the stack.
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bears on stack");
    // P0's Mystic Snake enters and counters it.
    let snake = g.add_card_to_battlefield(0, catalog::mystic_snake());
    g.fire_self_etb_triggers(snake, 0);
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears),
        "the countered spell goes to its owner's graveyard");
    assert!(!g.battlefield.iter().any(|c| c.id == bears), "it never resolved");
}

/// Fauna Shaman discards a creature card (as cost) to tutor a creature to hand.
#[test]
fn fauna_shaman_discards_creature_to_tutor_creature() {
    let mut g = two_player_game();
    let shaman = g.add_card_to_battlefield(0, catalog::fauna_shaman());
    g.clear_sickness(shaman);
    let pitch = g.add_card_to_hand(0, catalog::grizzly_bears()); // creature to discard
    let target = g.add_card_to_library(0, catalog::serra_angel()); // creature to fetch
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shaman, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Fauna Shaman activates");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pitch),
        "the discarded creature is in the graveyard");
    assert!(g.players[0].hand.iter().any(|c| c.id == target),
        "the tutored creature is in hand");
}

/// Fauna Shaman can't activate with no creature card to discard.
#[test]
fn fauna_shaman_requires_a_creature_to_discard() {
    let mut g = two_player_game();
    let shaman = g.add_card_to_battlefield(0, catalog::fauna_shaman());
    g.clear_sickness(shaman);
    g.add_card_to_hand(0, catalog::lightning_bolt()); // not a creature
    g.players[0].mana_pool.add(Color::Green, 1);
    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: shaman, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    });
    assert!(r.is_err(), "activation rejected with no creature card to discard");
}

// ── Ohran Frostfang / Sheoldred's Edict (this branch) ───────────────────────

/// Ohran Frostfang grants attacking creatures deathtouch and draws when one
/// deals combat damage to a player.
#[test]
fn ohran_frostfang_attackers_have_deathtouch_and_draw_on_damage() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ohran_frostfang());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.add_card_to_library(0, catalog::forest()); // something to draw
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("declare attack");
    let comp = g.computed_permanent(attacker).expect("computed");
    assert!(comp.keywords.contains(&Keyword::Deathtouch),
        "attacking creature gains deathtouch from Ohran's static");
    let hand_before = g.players[0].hand.len();
    for _ in 0..12 {
        if g.players[0].hand.len() > hand_before { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert!(g.players[0].hand.len() > hand_before,
        "drew a card when a creature dealt combat damage to a player");
}

/// Sheoldred's Edict mode 0 makes each opponent sacrifice a nontoken creature.
#[test]
fn sheoldreds_edict_sacrifices_nontoken_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sheoldreds_edict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("castable for {1}{B}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "opponent sacrificed their creature");
}

/// Ragavan: on combat damage to a player, mints a Treasure and exiles the
/// top of that player's library with a cast-this-turn permission.
#[test]
fn ragavan_combat_damage_makes_treasure_and_steals_top_card() {
    let mut g = two_player_game();
    let rag = g.add_card_to_battlefield(0, catalog::ragavan_nimble_pilferer());
    g.clear_sickness(rag);
    let loot = g.add_card_to_library(1, catalog::lightning_bolt()); // opp's top card
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rag, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..12 {
        if g.exile.iter().any(|c| c.id == loot) { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "minted a Treasure token");
    assert!(g.exile.iter().any(|c| c.id == loot),
        "exiled the top card of the damaged player's library");
    let stolen = g.exile.iter().find(|c| c.id == loot).unwrap();
    assert_eq!(stolen.may_play_until.as_ref().map(|m| m.player), Some(0),
        "Ragavan's controller may cast the exiled card");
}

/// Dragon's Rage Channeler: with delirium (4+ card types in graveyard) it's a
/// 3/3 with flying; without delirium it's a vanilla 1/1.
#[test]
fn dragons_rage_channeler_delirium_makes_3_3_flyer() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let drc = g.add_card_to_battlefield(0, catalog::dragons_rage_channeler());
    // No delirium yet: base 1/1, no flying.
    let c0 = g.computed_permanent(drc).expect("computed");
    assert_eq!((c0.power, c0.toughness), (1, 1));
    assert!(!c0.keywords.contains(&Keyword::Flying));
    // Seed 4 distinct card types into P0's graveyard (creature/instant/
    // sorcery/land) to switch on delirium.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::duress());
    g.add_card_to_graveyard(0, catalog::forest());
    let c1 = g.computed_permanent(drc).expect("computed");
    assert_eq!((c1.power, c1.toughness), (3, 3), "delirium grows it to 3/3");
    assert!(c1.keywords.contains(&Keyword::Flying), "delirium grants flying");
    assert!(c1.keywords.contains(&Keyword::MustAttack), "delirium adds attacks-each-combat");
}

// ── Glistener Elf / Imperial Recruiter / Goblin Matron / Loxodon Hierarch ───

#[test]
fn glistener_elf_has_infect() {
    use crabomination::card::Keyword;
    let d = catalog::glistener_elf();
    assert_eq!((d.power, d.toughness), (1, 1));
    assert!(d.keywords.contains(&Keyword::Infect));
}

#[test]
fn imperial_recruiter_fetches_low_power_creature() {
    let mut g = two_player_game();
    let goblin = g.add_card_to_library(0, catalog::grizzly_bears()); // power 2, legal
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(goblin))]));
    let ir = g.add_card_to_battlefield(0, catalog::imperial_recruiter());
    g.fire_self_etb_triggers(ir, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == goblin), "fetched the power-2 creature");
}

#[test]
fn goblin_matron_fetches_a_goblin() {
    let mut g = two_player_game();
    let gob = g.add_card_to_library(0, catalog::goblin_matron()); // a Goblin
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(gob))]));
    let m = g.add_card_to_battlefield(0, catalog::goblin_matron());
    g.fire_self_etb_triggers(m, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == gob), "fetched a Goblin to hand");
}

#[test]
fn loxodon_hierarch_etb_gains_four_life() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let lox = g.add_card_to_battlefield(0, catalog::loxodon_hierarch());
    g.fire_self_etb_triggers(lox, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "gained 4 life on ETB");
}

/// Loxodon Hierarch's sac ability regenerates each creature you control: a
/// regenerated creature survives lethal damage (shield consumed).
#[test]
fn loxodon_hierarch_sac_regenerates_your_creatures() {
    let mut g = two_player_game();
    let lox = g.add_card_to_battlefield(0, catalog::loxodon_hierarch());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lox, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac ability activates");
    drain_stack(&mut g);
    assert!(g.battlefield_find(lox).is_none(), "Hierarch sacrificed itself");
    // Now lethal damage on the bear is replaced by its regen shield.
    g.battlefield_find_mut(bear).unwrap().damage = 2;
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_some(), "regen shield saved the bear from lethal damage");
}

/// Fleecemane Lion gains hexproof + indestructible once it becomes monstrous.
#[test]
fn fleecemane_lion_monstrous_grants_hexproof_indestructible() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let lion = g.add_card_to_battlefield(0, catalog::fleecemane_lion());
    g.clear_sickness(lion);
    let c0 = g.computed_permanent(lion).expect("computed");
    assert!(!c0.keywords.contains(&Keyword::Hexproof), "not monstrous yet");
    // Pay {3}{G}{W} for Monstrosity 1.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lion, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("monstrosity activates");
    drain_stack(&mut g);
    assert!(g.battlefield_find(lion).unwrap().monstrous, "became monstrous");
    let c1 = g.computed_permanent(lion).expect("computed");
    assert!(c1.keywords.contains(&Keyword::Hexproof), "monstrous → hexproof");
    assert!(c1.keywords.contains(&Keyword::Indestructible), "monstrous → indestructible");
}

// ── Mana dorks (this branch) ────────────────────────────────────────────────

#[test]
fn ignoble_hierarch_taps_for_jund_and_has_exalted() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ignoble_hierarch());
    g.clear_sickness(id);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("mana ability");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "tapped for a chosen Jund color");
    assert!(!catalog::ignoble_hierarch().triggered_abilities.is_empty(),
        "has an exalted trigger");
}

#[test]
fn elves_of_deep_shadow_taps_for_black_and_pings_you() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::elves_of_deep_shadow());
    g.clear_sickness(id);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1, "added black");
    assert_eq!(g.players[0].life, life - 1, "dealt 1 damage to you");
}

// ── Generous Ent / Patchwork Automaton (this branch) ────────────────────────

#[test]
fn generous_ent_etb_makes_food_and_has_reach() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    assert!(catalog::generous_ent().keywords.contains(&Keyword::Reach));
    let ent = g.add_card_to_battlefield(0, catalog::generous_ent());
    g.fire_self_etb_triggers(ent, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"),
        "minted a Food token on ETB");
}

#[test]
fn patchwork_automaton_grows_on_artifact_cast() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let auto = g.add_card_to_battlefield(0, catalog::patchwork_automaton());
    // Cast an artifact spell (Sol Ring).
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ring, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sol Ring castable");
    drain_stack(&mut g);
    let a = g.battlefield_find(auto).expect("on battlefield");
    assert_eq!(a.counter_count(CounterType::PlusOnePlusOne), 1,
        "gained a +1/+1 counter when an artifact spell was cast");
}

/// Sleep taps every creature the target player controls and stuns them all
/// (skip-next-untap via a Stun counter).
#[test]
fn sleep_taps_and_stuns_all_target_player_creatures() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let own = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::sleep());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, id, Target::Player(1));
    for c in [a, b] {
        let r = g.battlefield_find(c).unwrap();
        assert!(r.tapped, "opponent creature tapped");
        assert_eq!(r.counter_count(CounterType::Stun), 1, "stun counter applied");
    }
    let mine = g.battlefield_find(own).unwrap();
    assert!(!mine.tapped && mine.counter_count(CounterType::Stun) == 0,
        "our own creature is untouched");
}

/// Crimson Wisps grants haste, makes the target red until end of turn, and
/// cantrips.
#[test]
fn crimson_wisps_grants_haste_and_red_and_draws() {
    use crabomination::card::Keyword;
    use crabomination::mana::Color as C;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears()); // something to draw
    let id = g.add_card_to_hand(0, catalog::crimson_wisps());
    let hand_before = g.players[0].hand.len(); // includes Wisps itself
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, id, Target::Permanent(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Haste), "gained haste");
    assert!(cp.colors.contains(C::Red), "became red");
    // Wisps left hand (−1), draw refilled (+1): net hand size unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before, "cantripped");
}

/// Wild Slash deals 2 damage; with a power-4+ creature out, it also flips the
/// "damage can't be prevented this turn" flag (Ferocious).
#[test]
fn wild_slash_ferocious_locks_out_prevention() {
    let mut g = two_player_game();
    // A 5/5 satisfies Ferocious.
    g.add_card_to_battlefield(0, catalog::grizzled_outrider());
    let id = g.add_card_to_hand(0, catalog::wild_slash());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, 18, "2 damage dealt");
    assert!(g.damage_cant_be_prevented_this_turn, "ferocious set the no-prevent flag");
}

/// Stat/keyword Auras attach to a creature and grant their +P/+T and keywords.
#[test]
fn stat_keyword_auras_grant_their_bonus() {
    use crabomination::card::Keyword;
    let cases: &[(Factory, i32, i32, &[Keyword])] = &[
        (catalog::untamed_hunger as Factory, 4, 3, &[Keyword::Menace]),
        (catalog::mark_of_the_vampire as Factory, 4, 4, &[Keyword::Lifelink]),
        (catalog::hammerhand as Factory, 3, 2, &[Keyword::Haste, Keyword::CantBlock]),
    ];
    for &(factory, p, t, kws) in cases {
        let mut g = two_player_game();
        let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, factory());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bears)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("aura castable");
        drain_stack(&mut g);
        let view = g.compute_battlefield();
        let c = view.iter().find(|c| c.id == bears).unwrap();
        assert_eq!((c.power, c.toughness), (p, t), "{} P/T", c.id.0);
        for kw in kws {
            assert!(c.keywords.contains(kw), "aura grants {kw:?}");
        }
    }
}

/// Claustrophobia taps the enchanted creature on ETB and keeps it tapped
/// through its controller's untap step (aura-anchored CR 502.3 prevention).
#[test]
fn claustrophobia_taps_and_locks_down_the_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    let aura = g.add_card_to_hand(0, catalog::claustrophobia());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, aura, Target::Permanent(bear));
    assert!(g.battlefield_find(bear).unwrap().tapped, "ETB tapped the creature");
    // The enchanted creature's controller's untap step must NOT untap it.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "stays tapped (doesn't untap)");
}

/// Heartless Act mode 0 destroys a creature with no counters; a creature with
/// a counter is not a legal target for that mode (CR 115 + HasNoCounters).
#[test]
fn heartless_act_destroy_requires_no_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let clean = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let counted = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(counted).unwrap()
        .counters.insert(CounterType::PlusOnePlusOne, 1);
    // Mode 0 on the clean creature destroys it.
    let id = g.add_card_to_hand(0, catalog::heartless_act());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(clean)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("clean creature is a legal mode-0 target");
    drain_stack(&mut g);
    assert!(g.battlefield_find(clean).is_none(), "no-counter creature destroyed");
    // Mode 0 cannot target the counter-bearing creature.
    let id2 = g.add_card_to_hand(0, catalog::heartless_act());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let res = g.perform_action(GameAction::CastSpell {
        card_id: id2, target: Some(Target::Permanent(counted)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    });
    assert!(res.is_err(), "counter-bearing creature is an illegal mode-0 target");
}

/// Heartless Act mode 1 strips the counters off a creature.
#[test]
fn heartless_act_mode_one_removes_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap()
        .counters.insert(CounterType::PlusOnePlusOne, 2);
    let id = g.add_card_to_hand(0, catalog::heartless_act());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("mode-1 castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
        "counters removed");
}

/// Landcycling discards the card and fetches a land of the named basic type to
/// hand, leaving non-lands in the library (CR 702.29e). Covers all five types.
#[test]
fn landcycling_fetches_the_named_basic_land() {
    let cases: &[(Factory, Factory, &str)] = &[
        (catalog::wirewood_guardian as Factory, catalog::forest as Factory, "Forest"),
        (catalog::daru_lancer as Factory, catalog::plains as Factory, "Plains"),
        (catalog::shoreline_ranger as Factory, catalog::island as Factory, "Island"),
        (catalog::twisted_abomination as Factory, catalog::swamp as Factory, "Swamp"),
        (catalog::skirk_marauder as Factory, catalog::mountain as Factory, "Mountain"),
    ];
    for &(cycler, land, land_name) in cases {
        let mut g = two_player_game();
        g.add_card_to_library(0, land());
        g.add_card_to_library(0, catalog::grizzly_bears()); // non-land decoy
        let id = g.add_card_to_hand(0, cycler());
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::Landcycle { card_id: id }).expect("landcycle");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "{land_name}: discarded");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == land_name),
            "fetched a {land_name}");
        assert!(g.players[0].library.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "{land_name}: non-land left in library");
    }
}

/// CR 702.29e — a `wants_ui` landcycler with multiple matches picks which
/// card to fetch; the pick suspends before any cost is paid.
#[test]
fn landcycling_ui_picks_among_multiple_matches() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    g.add_card_to_library(0, catalog::swamp());
    let wanted = g.add_card_to_library(0, catalog::swamp());
    let id = g.add_card_to_hand(0, catalog::twisted_abomination());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Landcycle { card_id: id }).expect("raise the pick");
    let pd = g.pending_decision.as_ref().expect("search pick pending");
    assert_eq!(pd.acting_player(), 0);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.id == id).count(), 1,
        "nothing paid or discarded before the pick");
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Search(Some(wanted))))
        .expect("submit the pick");
    assert!(g.players[0].hand.iter().any(|c| c.id == wanted), "the chosen Swamp was fetched");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "cycler discarded");
}

/// Soul Feast drains the target player for 4 and gains the caster 4 life.
#[test]
fn soul_feast_drains_four() {
    let mut g = two_player_game();
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    let id = g.add_card_to_hand(0, catalog::soul_feast());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, l1 - 4, "target lost 4");
    assert_eq!(g.players[0].life, l0 + 4, "caster gained 4");
}

/// Lava Axe deals 5 to a player.
#[test]
fn lava_axe_burns_face_for_five() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lava_axe());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, 15, "5 damage to the dome");
}

/// Demystify destroys an enchantment (and isn't castable at a creature).
#[test]
fn demystify_destroys_an_enchantment() {
    let mut g = two_player_game();
    let aura = g.add_card_to_battlefield(1, catalog::pacifism());
    let id = g.add_card_to_hand(0, catalog::demystify());
    g.players[0].mana_pool.add(Color::White, 1);
    cast_at(&mut g, id, Target::Permanent(aura));
    assert!(g.battlefield_find(aura).is_none(), "enchantment destroyed");
}

/// Befuddle shrinks a creature's power by 4 until end of turn and cantrips.
#[test]
fn befuddle_shrinks_power_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::befuddle());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    cast_at(&mut g, id, Target::Permanent(bear));
    assert_eq!(g.computed_permanent(bear).unwrap().power, -2, "2 - 4 = -2 power");
}

/// Coral Barrier enters as a 1/3 defender and mints a 1/1 Squid on ETB.
#[test]
fn coral_barrier_makes_a_squid() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::coral_barrier());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Coral Barrier castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!((body.power(), body.toughness()), (1, 3));
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Squid"),
        "a Squid token entered");
}

/// Rishadan Airship is a 2/3 flyer that can't block.
#[test]
fn rishadan_airship_flies_and_cant_block() {
    use crabomination::card::Keyword;
    let def = catalog::rishadan_airship();
    assert_eq!((def.power, def.toughness), (3, 1));
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::CantBlock));
}

/// Weakness shrinks the enchanted creature by -2/-1.
#[test]
fn weakness_shrinks_the_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::weakness());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast_at(&mut g, aura, Target::Permanent(bear));
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (0, 1), "2/2 becomes 0/1");
}

/// Lay of the Land tutors a basic land to hand.
#[test]
fn lay_of_the_land_fetches_a_basic() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::lay_of_the_land());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"),
        "basic land fetched to hand");
}

/// Tranquility wipes all enchantments from the board.
#[test]
fn tranquility_destroys_all_enchantments() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(1, catalog::pacifism());
    let ench = g.add_card_to_battlefield(0, catalog::rancor());
    let id = g.add_card_to_hand(0, catalog::tranquility());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert!(g.battlefield_find(aura).is_none() && g.battlefield_find(ench).is_none(),
        "all enchantments destroyed");
    assert!(g.battlefield_find(bear).is_some(), "creature untouched");
}

/// Flashfreeze counters a red spell; it can't target a blue one.
#[test]
fn flashfreeze_counters_red_spell() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt on stack");
    // P0 responds with Flashfreeze targeting the red spell.
    g.priority.player_with_priority = 0;
    let ff = g.add_card_to_hand(0, catalog::flashfreeze());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ff, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("flashfreeze targets the red spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "bolt was countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "bolt in graveyard");
}

/// Smash to Smithereens destroys an artifact and burns its controller for 3.
#[test]
fn smash_to_smithereens_destroys_and_burns() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::smash_to_smithereens());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, id, Target::Permanent(art));
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    assert_eq!(g.players[1].life, 17, "controller took 3");
}

/// Mark of Mutiny steals a creature and pumps it with a +1/+1 counter.
#[test]
fn mark_of_mutiny_steals_and_grows() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mark_of_mutiny());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, id, Target::Permanent(bear));
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.controller, 0, "stolen for the turn");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "got a +1/+1 counter");
}

// ── Aluren ───────────────────────────────────────────────────────────────────

/// Aluren lets any player free-cast a creature with MV ≤ 3, at flash speed.
#[test]
fn aluren_free_casts_cheap_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aluren());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G}, MV 2
    // Empty mana pool — only Aluren makes this castable.
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Aluren free-casts a cheap creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some(), "Grizzly Bears entered for free");
}

/// Aluren grants flash: the creature can be cast outside the main phase.
#[test]
fn aluren_grants_flash_timing() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aluren());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = crabomination::TurnStep::DeclareAttackers; // not a main phase
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Aluren grants flash so a cheap creature casts in combat");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some());
}

/// Aluren only covers MV ≤ 3 — a pricier creature is rejected.
#[test]
fn aluren_rejects_expensive_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aluren());
    let big = g.add_card_to_hand(0, catalog::colossal_dreadmaw()); // MV 6
    assert!(g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: big, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "MV 6 creature is outside Aluren's MV-3 cap");
}

// ── Commander mana lands ──────────────────────────────────────────────────────

/// Command Tower taps for one mana of any color.
#[test]
fn command_tower_taps_for_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::command_tower());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "produced one mana");
}

// ── Allied-color batch (modern_decks) ───────────────────────────────────────

/// Ministrant of Obligation dies into two flying Spirits (Afterlife 2).
#[test]
fn ministrant_of_obligation_afterlife() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ministrant_of_obligation());
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit").count();
    assert_eq!(spirits, 2, "Afterlife 2 → two Spirit tokens");
}

/// Dragon's Eye Sentry is a 1/3 with Defender and first strike.
#[test]
fn dragons_eye_sentry_keywords() {
    use crabomination::card::Keyword;
    let d = catalog::dragons_eye_sentry();
    assert_eq!((d.power, d.toughness), (1, 3));
    assert!(d.keywords.contains(&Keyword::Defender) && d.keywords.contains(&Keyword::FirstStrike));
}

/// Kor Sanctifiers destroys an artifact only when kicked.
#[test]
fn kor_sanctifiers_kicked_destroys_artifact() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(1, catalog::bontus_monument());
    let id = g.add_card_to_hand(0, catalog::kor_sanctifiers());
    g.players[0].mana_pool.add(Color::White, 2); // {2}{W} + kicker {W}
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: id, target: Some(Target::Permanent(relic)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert!(g.battlefield_find(relic).is_none(), "kicked → artifact destroyed");
}

/// Tolarian Terror's cost drops by one per instant/sorcery in the graveyard.
#[test]
fn tolarian_terror_graveyard_affinity() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::lightning_bolt()); }
    let id = g.add_card_to_hand(0, catalog::tolarian_terror());
    // {6}{U} − {3} = {3}{U}.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast with graveyard discount");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "discounted Terror resolved");
}

/// Vodalian Arcanist makes one restricted colorless mana.
#[test]
fn vodalian_arcanist_taps_for_restricted_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::vodalian_arcanist());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1,
        "restricted colorless sits apart from the free pool");
}

/// Reflecting Pool taps for a color a land you control could produce — with a
/// Forest in play, the only legal color is green.
#[test]
fn reflecting_pool_mirrors_your_lands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_battlefield(0, catalog::reflecting_pool());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1,
        "Forest is the only basic-typed land, so the pool produces green");
}

/// Gaea's Cradle taps for {G} equal to the number of creatures you control.
#[test]
fn gaeas_cradle_scales_with_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::gaeas_cradle());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2,
        "two creatures → two green mana");
}

/// Festering Mummy's death drops a -1/-1 counter on a target.
#[test]
fn festering_mummy_death_minus_counter() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::festering_mummy());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(victim).unwrap().counter_count(CounterType::MinusOneMinusOne), 1,
        "-1/-1 counter placed",
    );
}

/// Reassembling Skeleton returns itself from the graveyard tapped.
#[test]
fn reassembling_skeleton_recurs_from_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::reassembling_skeleton());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("reanimate self");
    drain_stack(&mut g);
    let r = g.battlefield_find(id).expect("on battlefield");
    assert!(r.tapped, "returns tapped");
}

/// Dread Wanderer recurs only with one or fewer cards in hand.
#[test]
fn dread_wanderer_gated_recursion() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::dread_wanderer());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // 2 cards → can't activate
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).is_err(),
        "blocked with two cards in hand");
    g.players[0].hand.clear();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("now allowed with empty hand");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "returned to battlefield");
}

/// Mausoleum Turnkey returns a creature card from the graveyard on ETB.
#[test]
fn mausoleum_turnkey_returns_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mausoleum_turnkey());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned to hand");
}

/// Bontu's Monument drains each opponent when you cast a creature spell.
#[test]
fn bontus_monument_drains_on_creature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bontus_monument());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[1].life;
    cast(&mut g, bear);
    assert_eq!(g.players[1].life, before - 1, "opponent lost 1 life");
    assert_eq!(g.players[0].life, 20 + 1, "you gained 1 life");
}

/// Goblin Motivator grants haste to a target creature.
#[test]
fn goblin_motivator_grants_haste() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let mot = g.add_card_to_battlefield(0, catalog::goblin_motivator());
    g.clear_sickness(mot);
    let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: mot, ability_index: 0, target: Some(Target::Permanent(fresh)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("grant haste");
    drain_stack(&mut g);
    assert!(g.computed_permanent(fresh).unwrap().keywords.contains(&Keyword::Haste));
}

/// Goblin Gang Leader makes two Goblins on ETB.
#[test]
fn goblin_gang_leader_makes_goblins() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::goblin_gang_leader());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let goblins = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Goblin").count();
    assert_eq!(goblins, 2, "two Goblin tokens");
}

/// Frenzied Goblin's paid attack trigger keeps a blocker from blocking.
#[test]
fn frenzied_goblin_attack_locks_blocker() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::frenzied_goblin());
    g.clear_sickness(gob);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(blocker)),
    ]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: gob, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(blocker).unwrap().keywords.contains(&Keyword::CantBlock),
        "blocker can't block this turn");
}

/// Llanowar Tribe taps for three green.
#[test]
fn llanowar_tribe_makes_three_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::llanowar_tribe());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("tap for GGG");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3, "three mana");
}

/// Sakura-Tribe Scout puts a land from hand onto the battlefield.
#[test]
fn sakura_tribe_scout_drops_a_land() {
    let mut g = two_player_game();
    let scout = g.add_card_to_battlefield(0, catalog::sakura_tribe_scout());
    g.clear_sickness(scout);
    let land = g.add_card_to_hand(0, catalog::tranquil_cove());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: scout, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("put land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some(), "land entered the battlefield");
}

/// The Khans life-gain taplands enter tapped and gain 1 life.
#[test]
fn gain_taplands_enter_tapped_and_gain_life() {
    let factories: [Factory; 5] = [
        catalog::tranquil_cove, catalog::dismal_backwater, catalog::bloodfell_caves,
        catalog::rugged_highlands, catalog::blossoming_sands,
    ];
    for f in factories {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, f());
        let before = g.players[0].life;
        g.perform_action(GameAction::PlayLand(id)).expect("play land");
        drain_stack(&mut g);
        let land = g.battlefield_find(id).expect("on battlefield");
        assert!(land.tapped, "enters tapped");
        assert_eq!(g.players[0].life, before + 1, "gained 1 life");
    }
}

// ── Addendum (CR 702.124) ────────────────────────────────────────────────────

/// Sphinx's Insight gains 2 life when cast during your main phase.
#[test]
fn sphinxs_insight_addendum_main_phase_gains_life() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::sphinxs_insight());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(g.players[0].life, 22, "drew 2 + gained 2 (addendum active)");
}

/// Cast outside your main phase, Sphinx's Insight only draws — no life.
#[test]
fn sphinxs_insight_no_addendum_off_main_phase() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::sphinxs_insight());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(g.players[0].life, 20, "no addendum off main phase");
}

/// Precognitive Perception always draws three.
#[test]
fn precognitive_perception_draws_three() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::precognitive_perception());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    let before = g.players[0].hand.len();
    cast(&mut g, id);
    assert_eq!(g.players[0].hand.len(), before - 1 + 3, "spell left hand, drew 3");
}

// ── Allied-color batch 3 (modern_decks) ─────────────────────────────────────

/// Tireless Missionaries gains 3 life on ETB.
#[test]
fn tireless_missionaries_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::tireless_missionaries());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, id);
    assert_eq!(g.players[0].life, 23);
}

/// Marauding Blight-Priest drains each opponent whenever you gain life.
#[test]
fn marauding_blight_priest_drains_on_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::marauding_blight_priest());
    let before = g.players[1].life;
    g.adjust_life(0, 2);
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::LifeGained { player: 0, amount: 2 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "opponent loses 1 on your lifegain");
}

/// Vampire Cutthroat has skulk and lifelink.
#[test]
fn vampire_cutthroat_keywords() {
    use crabomination::card::Keyword;
    let d = catalog::vampire_cutthroat();
    assert!(d.keywords.contains(&Keyword::Skulk) && d.keywords.contains(&Keyword::Lifelink));
}

/// Goblin Fireslinger pings a player for 1.
#[test]
fn goblin_fireslinger_pings_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::goblin_fireslinger());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
}

/// Hijack steals an artifact until end of turn.
#[test]
fn hijack_steals_artifact() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(1, catalog::bontus_monument());
    let id = g.add_card_to_hand(0, catalog::hijack());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, id, Target::Permanent(relic));
    assert_eq!(g.battlefield_find(relic).unwrap().controller, 0, "stolen until EOT");
}

/// Greenweaver Druid taps for two green.
#[test]
fn greenweaver_druid_makes_two_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::greenweaver_druid());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

/// Citanul Woodreaders draws two only when kicked.
#[test]
fn citanul_woodreaders_kicked_draws_two() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::citanul_woodreaders());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4); // {2}{G} + kicker {2}{G}
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellKicked {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("kicked");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 2, "spell left hand, drew 2");
}

/// Wickerbough Elder removes its -1/-1 counter to destroy an artifact.
#[test]
fn wickerbough_elder_removes_counter_to_destroy() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(1, catalog::bontus_monument());
    let id = g.add_card_to_hand(0, catalog::wickerbough_elder());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    g.clear_sickness(id);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::MinusOneMinusOne), 1,
        "enters with a -1/-1 counter");
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(relic)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("destroy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(relic).is_none(), "artifact destroyed");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::MinusOneMinusOne), 0,
        "counter removed as cost");
}

// ── Allied-color batch 4 (modern_decks) ─────────────────────────────────────

/// Sandsteppe Outcast's ETB makes a Spirit (mode 1).
#[test]
fn sandsteppe_outcast_makes_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sandsteppe_outcast());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Spirit"),
        "mode 1 → Spirit token");
}

/// Cloudreader Sphinx is a 3/4 flyer that scries on ETB.
#[test]
fn cloudreader_sphinx_etb_scries() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cloudreader_sphinx());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, id);
    let r = g.battlefield_find(id).unwrap();
    assert!(r.definition.keywords.contains(&Keyword::Flying));
}

/// Crippling Blight shrinks and locks down its host.
#[test]
fn crippling_blight_weakens_creature() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::crippling_blight());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast_at(&mut g, aura, Target::Permanent(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "-1/-1");
    assert!(cp.keywords.contains(&Keyword::CantBlock));
}

/// Nimble Mongoose grows with threshold (7+ cards in graveyard).
#[test]
fn nimble_mongoose_threshold() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::nimble_mongoose());
    assert_eq!(g.computed_permanent(id).unwrap().power, 1, "no threshold yet");
    for _ in 0..7 { g.add_card_to_graveyard(0, catalog::lightning_bolt()); }
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "threshold active → +2/+2");
}

/// Aerial Predation destroys a flyer and gains 2 life.
#[test]
fn aerial_predation_kills_flyer_and_gains() {
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(1, catalog::snapping_drake()); // 3/2 flyer
    let id = g.add_card_to_hand(0, catalog::aerial_predation());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, id, Target::Permanent(drake));
    assert!(g.battlefield_find(drake).is_none(), "flyer destroyed");
    assert_eq!(g.players[0].life, 22, "gained 2");
}

/// Centaur Courser and Goblin Roughrider are vanilla beaters with the
/// printed stats.
#[test]
fn vanilla_beaters_have_stats() {
    let cc = catalog::centaur_courser();
    assert_eq!((cc.power, cc.toughness), (3, 3));
    let gr = catalog::goblin_roughrider();
    assert_eq!((gr.power, gr.toughness), (3, 2));
}

// ── Allied-color batch 5 (modern_decks) ─────────────────────────────────────

/// Mistral Singer pumps from prowess on a noncreature cast.
#[test]
fn mistral_singer_prowess() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mistral_singer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "prowess +1/+1");
}

/// Air Servant taps a flyer.
#[test]
fn air_servant_taps_flyer() {
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(1, catalog::snapping_drake());
    let id = g.add_card_to_battlefield(0, catalog::air_servant());
    g.clear_sickness(id);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(drake)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("tap flyer");
    drain_stack(&mut g);
    assert!(g.battlefield_find(drake).unwrap().tapped);
}

/// Ronom Unicorn sacrifices itself to destroy an enchantment.
#[test]
fn ronom_unicorn_destroys_enchantment() {
    let mut g = two_player_game();
    let aura = g.add_card_to_battlefield(1, catalog::crippling_blight());
    let id = g.add_card_to_battlefield(0, catalog::ronom_unicorn());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(aura)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("sac to destroy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "enchantment destroyed");
    assert!(g.battlefield_find(id).is_none(), "unicorn sacrificed");
}

/// Blade Instructor mentors a smaller attacker.
#[test]
fn blade_instructor_mentors() {
    let mut g = two_player_game();
    let mentor = g.add_card_to_battlefield(0, catalog::blade_instructor()); // 3/1
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 (lesser power)
    g.clear_sickness(mentor);
    g.clear_sickness(small);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: mentor, target: AttackTarget::Player(1) },
        Attack { attacker: small, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(small).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "mentor put a +1/+1 counter on the smaller attacker");
}

/// Akroma's Devoted gives all Clerics vigilance.
#[test]
fn akromas_devoted_grants_cleric_vigilance() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::akromas_devoted());
    let pilgrim = g.add_card_to_battlefield(0, catalog::ministrant_of_obligation()); // Human Cleric
    assert!(g.computed_permanent(pilgrim).unwrap().keywords.contains(&Keyword::Vigilance),
        "other Cleric gains vigilance");
}

/// Veteran Swordsmith buffs other Soldiers but not itself.
#[test]
fn veteran_swordsmith_buffs_other_soldiers() {
    let mut g = two_player_game();
    let smith = g.add_card_to_battlefield(0, catalog::veteran_swordsmith()); // 3/2
    let ally = g.add_card_to_battlefield(0, catalog::blade_instructor()); // 3/1 Soldier
    let sc = g.computed_permanent(smith).unwrap();
    assert_eq!((sc.power, sc.toughness), (3, 2), "does not buff itself");
    let ac = g.computed_permanent(ally).unwrap();
    assert_eq!((ac.power, ac.toughness), (4, 1), "other Soldier gets +1/+0");
}

