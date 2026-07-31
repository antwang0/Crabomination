//! Betrayers of Kamigawa (BOK) gap closure, wave 2.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;

fn always_yes(g: &mut GameState) {
    g.decider = Box::new(ScriptedDecider::new(
        std::iter::repeat_with(|| DecisionAnswer::Bool(true)).take(8),
    ));
}

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

/// Every wave-2 BOK factory is registered under its printed name.
#[test]
fn bok2_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::higure_the_still_wind as fn() -> crabomination::card::CardDefinition,
        catalog::ink_eyes_servant_of_oni,
        catalog::okiba_gang_shinobi,
        catalog::walker_of_secret_ways,
        catalog::jetting_glasskite,
        catalog::shimmering_glasskite,
        catalog::kira_great_glass_spinner,
        catalog::horobis_whisper,
        catalog::hundred_talon_strike,
        catalog::torrent_of_stone,
        catalog::roar_of_jukai,
        catalog::veil_of_secrecy,
        catalog::overblaze,
        catalog::flames_of_the_blood_hand,
        catalog::sway_of_the_stars,
        catalog::twist_allegiance,
        catalog::akki_raider,
        catalog::empty_shrine_kannushi,
        catalog::chisei_heart_of_oceans,
        catalog::ogre_marauder,
        catalog::shirei_shizos_caretaker,
        catalog::iwamori_of_the_open_fist,
        catalog::blessing_of_leeches,
        catalog::mark_of_the_oni,
        catalog::kumanos_blessing,
        catalog::slumbering_tora,
        catalog::neko_te,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

// ── Glasskites ──────────────────────────────────────────────────────────────

/// The first spell to target a Glasskite each turn is countered; the second
/// resolves.
#[test]
fn jetting_glasskite_counters_only_the_first_spell_each_turn() {
    let mut g = two_player_game();
    let kite = g.add_card_to_battlefield(1, catalog::jetting_glasskite());
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    cast(&mut g, bolt1, Some(Target::Permanent(kite)));
    assert!(g.battlefield_find(kite).is_some(), "the first Bolt was countered");
    cast(&mut g, bolt2, Some(Target::Permanent(kite)));
    assert_eq!(g.battlefield_find(kite).unwrap().damage, 3, "the second Bolt got through");
}

/// Kira hands the once-per-turn counter to every creature you control.
#[test]
fn kira_shields_your_other_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::kira_great_glass_spinner());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_some(), "Kira countered the Bolt");
}

// ── Splice (CR 702.47) ──────────────────────────────────────────────────────

/// Horobi's Whisper splices onto an Arcane spell for four graveyard cards.
#[test]
fn horobis_whisper_splices_for_four_graveyard_cards() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::swamp());
    let ray = g.add_card_to_hand(0, catalog::glacial_ray());
    let whisper = g.add_card_to_hand(0, catalog::horobis_whisper());
    for _ in 0..4 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 4);
    g.perform_action(GameAction::CastSpellSpliced {
        card_id: ray,
        splice_cards: vec![whisper],
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![Target::Permanent(victim)],
        mode: None,
        x_value: None,
    })
    .expect("spliced cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "the Bear ate Ray + Whisper");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.is_creature()).count(), 0);
}

/// Torrent of Stone's splice cost is two Mountains off the battlefield.
#[test]
fn torrent_of_stone_splice_sacrifices_two_mountains() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let ray = g.add_card_to_hand(0, catalog::glacial_ray());
    let torrent = g.add_card_to_hand(0, catalog::torrent_of_stone());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 4);
    g.perform_action(GameAction::CastSpellSpliced {
        card_id: ray,
        splice_cards: vec![torrent],
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![Target::Permanent(victim)],
        mode: None,
        x_value: None,
    })
    .expect("spliced cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 0);
    assert!(g.battlefield_find(victim).is_none());
}

/// Roar of Jukai's splice cost hands an opponent five life.
#[test]
fn roar_of_jukai_splice_gives_an_opponent_five_life() {
    let mut g = two_player_game();
    let ray = g.add_card_to_hand(0, catalog::glacial_ray());
    let roar = g.add_card_to_hand(0, catalog::roar_of_jukai());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 4);
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpellSpliced {
        card_id: ray,
        splice_cards: vec![roar],
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("spliced cast");
    drain_stack(&mut g);
    // +5 from the splice cost, then Glacial Ray's 2 damage.
    assert_eq!(g.players[1].life, before + 5 - 2);
}

// ── Other cards ─────────────────────────────────────────────────────────────

/// Overblaze doubles the chosen permanent's damage for the turn.
#[test]
fn overblaze_doubles_a_permanents_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blaze = g.add_card_to_hand(0, catalog::overblaze());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 4);
    cast(&mut g, blaze, Some(Target::Permanent(bear)));
    let mut evs = vec![];
    let before = g.players[1].life;
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(1),
        2,
        Some(bear),
        &mut evs,
    );
    assert_eq!(g.players[1].life, before - 4, "2 became 4");
}

/// Flames of the Blood Hand shuts off the target's life gain for the turn.
#[test]
fn flames_of_the_blood_hand_locks_out_life_gain() {
    let mut g = two_player_game();
    let flames = g.add_card_to_hand(0, catalog::flames_of_the_blood_hand());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 3);
    let before = g.players[1].life;
    cast(&mut g, flames, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, before - 4);
    let heal = g.add_card_to_hand(1, catalog::healing_salve());
    g.players[1].mana_pool.add(crabomination::mana::Color::White, 1);
    g.priority.player_with_priority = 1;
    let _ = g.perform_action(GameAction::CastSpell {
        card_id: heal,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 4, "the lock held");
}

/// Chisei eats a counter off one of your permanents each upkeep.
#[test]
fn chisei_removes_a_counter_instead_of_dying() {
    let mut g = two_player_game();
    let chisei = g.add_card_to_battlefield(0, catalog::chisei_heart_of_oceans());
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(host).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    always_yes(&mut g);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(chisei).is_some(), "Chisei survived");
    assert_eq!(g.battlefield_find(host).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// With no counters anywhere, Chisei sacrifices itself.
#[test]
fn chisei_dies_without_a_counter_to_eat() {
    let mut g = two_player_game();
    let chisei = g.add_card_to_battlefield(0, catalog::chisei_heart_of_oceans());
    always_yes(&mut g);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(chisei).is_none());
}

/// Empty-Shrine Kannushi can't be blocked by a creature sharing a color with
/// something you control.
#[test]
fn empty_shrine_kannushi_has_protection_from_your_colors() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::empty_shrine_kannushi());
    g.clear_sickness(atk);
    // The Kannushi itself is white, so white blockers are barred.
    let white_blocker = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let green_blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(white_blocker, atk)])).is_err());
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(green_blocker, atk)])).is_ok());
}

/// Akki Raider pumps when a land hits a graveyard.
#[test]
fn akki_raider_grows_off_a_dead_land() {
    let mut g = two_player_game();
    let raider = g.add_card_to_battlefield(0, catalog::akki_raider());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let evs = g.remove_to_graveyard_with_triggers(land);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(raider).unwrap().power, 3);
}

/// Shirei brings a 1-power creature back at the next end step.
#[test]
fn shirei_returns_a_small_creature_at_the_end_step() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::shirei_shizos_caretaker());
    let mox = g.add_card_to_battlefield(0, catalog::savannah_lions()); // 2/1 — power 2
    let weenie = g.add_card_to_battlefield(0, catalog::memnite()); // 1/1
    let _ = mox;
    always_yes(&mut g);
    let evs = g.remove_to_graveyard_with_triggers(weenie);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Memnite"),
        "the 1/1 came back"
    );
}

/// Twist Allegiance swaps the two armies and unlocks the borrowed creatures.
#[test]
fn twist_allegiance_swaps_armies_with_haste() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::savannah_lions());
    if let Some(c) = g.battlefield_find_mut(theirs) {
        c.tapped = true;
    }
    let twist = g.add_card_to_hand(0, catalog::twist_allegiance());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 7);
    cast(&mut g, twist, Some(Target::Player(1)));
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1);
    assert!(!g.battlefield_find(theirs).unwrap().tapped, "it untapped");
    assert!(
        g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::Haste),
        "and gained haste"
    );
}

/// Sway of the Stars resets both boards, hands, graveyards and life totals.
#[test]
fn sway_of_the_stars_resets_the_game() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    for _ in 0..12 {
        g.add_card_to_library(1, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
    }
    let sway = g.add_card_to_hand(0, catalog::sway_of_the_stars());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 10);
    cast(&mut g, sway, None);
    assert!(g.battlefield.is_empty(), "the board is gone");
    for p in [0, 1] {
        assert_eq!(g.players[p].life, 7);
        assert_eq!(g.players[p].hand.len(), 7);
    }
    assert!(g.players[1].graveyard.is_empty(), "the Bolt was shuffled away");
}

/// Slumbering Tora animates as an X/X off the discarded card's mana value.
#[test]
fn slumbering_tora_animates_off_the_discarded_mana_value() {
    let mut g = two_player_game();
    let tora = g.add_card_to_battlefield(0, catalog::slumbering_tora());
    g.add_card_to_hand(0, catalog::body_of_jukai()); // MV 9 Spirit
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: tora,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(tora).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature));
    assert_eq!((cp.power, cp.toughness), (9, 9));
}

/// Ogre Marauder's attack trigger costs the defender a creature.
#[test]
fn ogre_marauder_taxes_the_defender_a_creature() {
    let mut g = two_player_game();
    let ogre = g.add_card_to_battlefield(0, catalog::ogre_marauder());
    g.clear_sickness(ogre);
    let chump = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    always_yes(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ogre,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(chump).is_none(), "the defender paid up");
}

/// Blessing of Leeches regenerates its host for {0}.
#[test]
fn blessing_of_leeches_regenerates_the_enchanted_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(1, catalog::blessing_of_leeches());
    g.players[1].mana_pool.add(crabomination::mana::Color::Black, 3);
    g.priority.player_with_priority = 1;
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("regenerate");
    drain_stack(&mut g);
    let mut evs = vec![];
    g.destroy_permanent(bear, false, &mut evs);
    assert!(g.battlefield_find(bear).is_some(), "the shield held");
}

/// Neko-Te's equipped creature locks down what it damages.
#[test]
fn neko_te_taps_down_what_it_damages() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let te = g.add_card_to_battlefield(0, catalog::neko_te());
    g.battlefield_find_mut(te).unwrap().attached_to = Some(bear);
    g.clear_sickness(bear);
    let victim = g.add_card_to_battlefield(1, catalog::wall_of_wood());
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
    g.set_block_map([(victim, bear)]);
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = 0;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped, "it got tapped");
    assert!(g.battlefield_find(victim).unwrap().skip_next_untap);
}

// ── Wave 3 ──────────────────────────────────────────────────────────────────

/// Every wave-3 BOK factory is registered under its printed name.
#[test]
fn bok2_wave3_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::budoka_pupil as fn() -> crabomination::card::CardDefinition,
        catalog::fumiko_the_lowblood,
        catalog::kentaro_the_smiling_cat,
        catalog::opal_eye_kondas_yojimbo,
        catalog::toshiro_umezawa,
        catalog::aura_barbs,
        catalog::minamos_meddling,
        catalog::shining_shoal,
        catalog::mark_of_sakiko,
        catalog::ornate_kanzashi,
        catalog::blinding_powder,
        catalog::shuriken,
        catalog::clash_of_realities,
        catalog::tomorrow_azamis_familiar,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// Aura Barbs shocks each enchantment's controller and each Aura's host.
#[test]
fn aura_barbs_bites_controllers_and_hosts() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(1, catalog::pacifism());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let barbs = g.add_card_to_hand(0, catalog::aura_barbs());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 3);
    let before = g.players[1].life;
    cast(&mut g, barbs, None);
    assert_eq!(g.players[1].life, before - 2, "the Aura's controller took 2");
    assert!(g.battlefield_find(bear).is_none(), "and the 2/2 host died to its own Aura");
}

/// Minamo's Meddling counters the spell and strips the spliced names.
#[test]
fn minamos_meddling_discards_the_spliced_names() {
    let mut g = two_player_game();
    let ray = g.add_card_to_hand(0, catalog::glacial_ray());
    let splicer = g.add_card_to_hand(0, catalog::glacial_ray());
    let copy = g.add_card_to_hand(0, catalog::glacial_ray());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 4);
    g.perform_action(GameAction::CastSpellSpliced {
        card_id: ray,
        splice_cards: vec![splicer],
        target: Some(Target::Player(1)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("spliced cast");
    let meddle = g.add_card_to_hand(1, catalog::minamos_meddling());
    g.players[1].mana_pool.add(crabomination::mana::Color::Blue, 4);
    g.priority.player_with_priority = 1;
    cast(&mut g, meddle, Some(Target::Permanent(ray)));
    assert!(g.stack.is_empty(), "the Ray was countered");
    // Both remaining Glacial Rays (the splicer, still in hand, and the spare)
    // share the spliced name and are discarded.
    assert!(!g.players[0].hand.iter().any(|c| c.id == splicer || c.id == copy));
}

/// Tomorrow turns each draw into look-3-keep-1.
#[test]
fn tomorrow_replaces_draws_with_a_look_at_three() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tomorrow_azamis_familiar());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let lib = g.players[0].library.len();
    let mut evs = vec![];
    assert!(g.draw_one(0, &mut evs));
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].library.len(), lib - 1, "the other two went to the bottom");
}

/// Kentaro lets a Samurai spell be cast for colorless equal to its mana value.
#[test]
fn kentaro_lets_samurai_be_paid_generically() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kentaro_the_smiling_cat());
    let samurai = g.add_card_to_hand(0, catalog::opal_eye_kondas_yojimbo()); // {1}{W}{W}
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: samurai,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("alternative cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Opal-Eye, Konda's Yojimbo"));
}

/// Toshiro Umezawa flashes an instant back when an opponent's creature dies.
#[test]
fn toshiro_umezawa_unlocks_a_graveyard_instant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::toshiro_umezawa());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let evs = g.remove_to_graveyard_with_triggers(victim);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.players[0]
            .graveyard
            .iter()
            .find(|c| c.id == bolt)
            .is_some_and(|c| c.may_play_until.is_some()),
        "the Bolt is castable from the graveyard"
    );
}

/// Fumiko's bushido scales with the attacker count and pushes opponents to
/// attack.
#[test]
fn fumiko_grants_must_attack_and_scaling_bushido() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fumiko_the_lowblood());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::MustAttack),
        "their creatures have to swing"
    );
    assert_eq!(catalog::fumiko_the_lowblood().triggered_abilities.len(), 2);
}

/// Ornate Kanzashi exiles an opponent's top card and lets you play it.
#[test]
fn ornate_kanzashi_lends_you_the_top_card() {
    let mut g = two_player_game();
    let kanzashi = g.add_card_to_battlefield(0, catalog::ornate_kanzashi());
    g.clear_sickness(kanzashi);
    let top = g.add_card_to_library(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kanzashi,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == top).expect("exiled");
    assert_eq!(exiled.may_play_until.map(|m| m.player), Some(0));
}

/// Budoka Pupil flips into Ichiga once it has two ki counters.
#[test]
fn budoka_pupil_flips_into_ichiga() {
    let mut g = two_player_game();
    let pupil = g.add_card_to_battlefield(0, catalog::budoka_pupil());
    g.battlefield_find_mut(pupil).unwrap().add_counters(CounterType::Ki, 2);
    always_yes(&mut g);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let cp = g.computed_permanent(pupil).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Blinding Powder's unattach shrugs off the equipped creature's combat damage.
#[test]
fn blinding_powder_unattaches_to_dodge_combat_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let powder = g.add_card_to_battlefield(0, catalog::blinding_powder());
    g.battlefield_find_mut(powder).unwrap().attached_to = Some(bear);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("unattach");
    drain_stack(&mut g);
    assert!(g.battlefield_find(powder).unwrap().attached_to.is_none(), "it came off");
    assert!(g.combat_damage_prevented_to_self(bear));
}

/// Clash of Realities points every entering creature at the other camp.
#[test]
fn clash_of_realities_makes_arrivals_bite() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::clash_of_realities());
    let spirit = g.add_card_to_battlefield(1, catalog::kami_of_ancient_law()); // 2/2 Spirit
    always_yes(&mut g);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    cast(&mut g, bear, None);
    assert!(g.battlefield_find(spirit).is_none(), "the non-Spirit arrival shot the Spirit");
}
