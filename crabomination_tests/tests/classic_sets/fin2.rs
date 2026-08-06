//! Final Fantasy (FIN) gap closure: Town // Adventure lands and Sidequests.

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

/// Advance to `step` on seat 0's next turn (games start mid-turn, so the naive
/// walk to Upkeep lands on the opponent's).
fn advance_to_your(g: &mut GameState, step: TurnStep) {
    for p in 0..g.players.len() {
        for i in 0..40 {
            g.players[p].library.push(CardInstance::new(CardId(5000 + (p * 100 + i) as u32), catalog::forest(), p));
        }
    }
    while !(g.step == step && g.active_player_idx == 0) {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
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

fn cast_adventure(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastAdventure {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast adventure");
    drain_stack(g);
}

/// Every new FIN factory is registered under its printed name.
#[test]
fn fin2_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::ishgard_the_holy_see as fn() -> crabomination::card::CardDefinition,
        catalog::jidoor_aristocratic_capital,
        catalog::lindblum_industrial_regency,
        catalog::midgar_city_of_mako,
        catalog::zanarkand_ancient_metropolis,
        catalog::balamb_garden_seed_academy,
        catalog::sidequest_card_collection,
        catalog::sidequest_catch_a_fish,
        catalog::sidequest_hunt_the_mark,
        catalog::sidequest_play_blitzball,
        catalog::sidequest_raise_a_chocobo,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

// ── Town // Adventure lands (CR 715.3d) ─────────────────────────────────────

/// The land half is played out of adventure exile — not cast — and it costs the
/// turn's land drop.
#[test]
fn adventure_land_half_is_played_from_exile() {
    let mut g = two_player_game();
    let ishgard = g.add_card_to_hand(0, catalog::ishgard_the_holy_see());
    g.add_card_to_graveyard(0, catalog::sol_ring());
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    g.players[0].mana_pool.add(Color::White, 5);
    let sol_ring = g.players[0].graveyard[0].id;
    g.perform_action(GameAction::CastAdventure {
        card_id: ishgard,
        target: Some(Target::Permanent(sol_ring)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Faith & Grief");
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.id == ishgard && c.on_adventure),
        "Faith & Grief exiled the land half on the adventure"
    );
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Sol Ring"));
    g.perform_action(GameAction::PlayLand(ishgard)).expect("play the land half");
    let land = g.battlefield_find(ishgard).expect("Ishgard is on the battlefield");
    assert!(land.tapped, "Towns enter tapped");
    assert_eq!(g.players[0].lands_played_this_turn, 1);
}

/// The exiled land half is surfaced to the client as a playable affordance.
#[test]
fn adventure_land_half_is_offered_as_an_affordance() {
    let mut g = two_player_game();
    let midgar = g.add_card_to_hand(0, catalog::midgar_city_of_mako());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    g.players[0].mana_pool.add(Color::Black, 3);
    cast_adventure(&mut g, midgar, None);
    assert_eq!(g.compute_hand_affordances(0).adventure_exile, vec![midgar]);
    // A spent land drop takes it back off the list.
    g.players[0].lands_played_this_turn = 1;
    assert!(g.compute_hand_affordances(0).adventure_exile.is_empty());
}

/// Overture mills half the target opponent's library, rounded down.
#[test]
fn overture_mills_half_rounded_down() {
    let mut g = two_player_game();
    let jidoor = g.add_card_to_hand(0, catalog::jidoor_aristocratic_capital());
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    g.players[0].mana_pool.add(Color::Blue, 6);
    let before = g.players[1].library.len();
    cast_adventure(&mut g, jidoor, Some(Target::Player(1)));
    assert_eq!(g.players[1].library.len(), before - before / 2);
}

/// Lasting Fayth's Hero scales with the lands you control.
#[test]
fn lasting_fayth_hero_counts_your_lands() {
    let mut g = two_player_game();
    let zanarkand = g.add_card_to_hand(0, catalog::zanarkand_ancient_metropolis());
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.players[0].mana_pool.add(Color::Green, 6);
    cast_adventure(&mut g, zanarkand, None);
    let hero = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Hero")
        .expect("Hero token");
    assert_eq!(hero.counters.get(&CounterType::PlusOnePlusOne).copied(), Some(6));
}

/// Balamb Garden's transform cost drops {1} per other Town you control.
#[test]
fn balamb_garden_transform_is_discounted_by_towns() {
    let mut g = two_player_game();
    let garden = g.add_card_to_battlefield(0, catalog::balamb_garden_seed_academy());
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::treno_dark_city());
    }
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.battlefield_find_mut(garden).unwrap().tapped = false;
    // {5}{G}{U} less {2} for the two other Towns = {3}{G}{U}.
    g.players[0].mana_pool.add(Color::Green, 4);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: garden, ability_index: 2,
        target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("transform");
    drain_stack(&mut g);
    let bf = g.battlefield_find(garden).expect("still on the battlefield");
    assert_eq!(bf.definition.name, "Balamb Garden, Airborne");
    assert!(bf.definition.keywords.contains(&Keyword::Flying));
}

// ── Sidequests ──────────────────────────────────────────────────────────────

/// Card Collection draws three and discards two on entry, then transforms at
/// your end step once the graveyard holds eight cards.
#[test]
fn sidequest_card_collection_transforms_on_eight_cards() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_card_collection());
    for _ in 0..8 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    advance_to(&mut g, TurnStep::End);
    let bf = g.battlefield_find(quest).expect("still around");
    assert_eq!(bf.definition.name, "Magicked Card");
    assert!(bf.definition.card_types.contains(&CardType::Artifact));
}

/// Under eight cards the quest stays face up.
#[test]
fn sidequest_card_collection_waits_for_the_graveyard() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_card_collection());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Sidequest: Card Collection");
}

/// Catch a Fish's "if you put a card into your hand this way" rider makes the
/// Food and flips the quest.
#[test]
fn sidequest_catch_a_fish_rider_fires_on_a_pick() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_catch_a_fish());
    g.players[0].library.insert(0, CardInstance::new(CardId(900), catalog::grizzly_bears(), 0));
    advance_to_your(&mut g, TurnStep::Upkeep);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"));
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Cooking Campsite");
}

/// No eligible card on top → no Food, no transform.
#[test]
fn sidequest_catch_a_fish_rider_skips_on_a_miss() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_catch_a_fish());
    g.players[0].library.insert(0, CardInstance::new(CardId(901), catalog::lightning_bolt(), 0));
    advance_to_your(&mut g, TurnStep::Upkeep);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Food"));
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Sidequest: Catch a Fish");
}

/// Hunt the Mark makes a Treasure when an opponent's creature died, and flips
/// on the third.
#[test]
fn sidequest_hunt_the_mark_transforms_on_three_treasures() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_hunt_the_mark());
    for _ in 0..2 {
        g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
    }
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(victim)));
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Yiazmat, Ultimate Mark");
}

/// Play Blitzball flips at end of combat once a player took 6+ combat damage.
#[test]
fn sidequest_play_blitzball_transforms_on_six_combat_damage() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_play_blitzball());
    g.players[1].combat_damage_taken_this_turn = 6;
    advance_to(&mut g, TurnStep::EndCombat);
    let bf = g.battlefield_find(quest).expect("still around");
    assert_eq!(bf.definition.name, "World Champion, Celestial Weapon");
}

/// Non-combat damage doesn't count toward Blitzball's threshold.
#[test]
fn sidequest_play_blitzball_ignores_noncombat_damage() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::sidequest_play_blitzball());
    g.players[1].damage_taken_this_turn = 9;
    advance_to(&mut g, TurnStep::EndCombat);
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Sidequest: Play Blitzball");
}

/// Raise a Chocobo makes a Bird on entry and flips at your main phase with four.
#[test]
fn sidequest_raise_a_chocobo_transforms_on_four_birds() {
    let mut g = two_player_game();
    let quest = g.add_card_to_hand(0, catalog::sidequest_raise_a_chocobo());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::birds_of_paradise());
    }
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, quest, None);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Bird"), "ETB Bird token");
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Sidequest: Raise a Chocobo");
    g.add_card_to_battlefield(0, catalog::birds_of_paradise());
    advance_to_your(&mut g, TurnStep::PreCombatMain);
    assert_eq!(g.battlefield_find(quest).unwrap().definition.name, "Black Chocobo");
}

// ── Transforming legends ────────────────────────────────────────────────────

/// Every transforming-legend factory is registered.
#[test]
fn fin2_transformers_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::exdeath_void_warlock as fn() -> crabomination::card::CardDefinition,
        catalog::emet_selch_unsundered,
        catalog::kuja_genome_sorcerer,
        catalog::the_emperor_of_palamecia,
        catalog::vincent_valentine,
        catalog::ultimecia_time_sorceress,
    ] {
        let name = f().name;
        assert!(names.contains(&name), "{name} is not registered");
    }
}

/// Exdeath flips on six permanent cards in the graveyard; Neo Exdeath's power
/// reads that pile.
#[test]
fn exdeath_transforms_and_scales_off_the_graveyard() {
    let mut g = two_player_game();
    let ex = g.add_card_to_battlefield(0, catalog::exdeath_void_warlock());
    for _ in 0..6 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    advance_to(&mut g, TurnStep::End);
    let neo = g.battlefield_find(ex).expect("still around");
    assert_eq!(neo.definition.name, "Neo Exdeath, Dimension's End");
    assert_eq!(g.computed_permanent(ex).unwrap().power, 6);
}

/// Five permanent cards isn't enough.
#[test]
fn exdeath_waits_for_six_permanent_cards() {
    let mut g = two_player_game();
    let ex = g.add_card_to_battlefield(0, catalog::exdeath_void_warlock());
    for _ in 0..5 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(ex).unwrap().definition.name, "Exdeath, Void Warlock");
}

/// Hades exiles what would hit your graveyard and lets you cast out of it.
#[test]
fn hades_exiles_your_graveyard_and_opens_it_up() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hades = g.add_card_to_battlefield(0, catalog::emet_selch_unsundered());
    let mut evs = vec![];
    g.transform_permanent(hades, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(hades).unwrap().definition.name, "Hades, Sorcerer of Eld");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    assert!(g.players[0].graveyard.is_empty(), "the Bear and the Bolt were exiled instead");
}

/// Kuja's end-step Wizard flips him once you have four.
#[test]
fn kuja_transforms_on_four_wizards() {
    let mut g = two_player_game();
    let kuja = g.add_card_to_battlefield(0, catalog::kuja_genome_sorcerer());
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::snapcaster_mage());
    }
    advance_to(&mut g, TurnStep::End);
    assert_eq!(g.battlefield_find(kuja).unwrap().definition.name, "Trance Kuja, Fate Defied");
}

/// Trance Kuja doubles a Wizard's combat damage but not anyone else's.
#[test]
fn trance_kuja_doubles_only_wizard_damage() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let kuja = g.add_card_to_battlefield(0, catalog::kuja_genome_sorcerer());
    let mut evs = vec![];
    g.transform_permanent(kuja, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let wizard = g.add_card_to_battlefield(0, catalog::snapcaster_mage()); // 2/1 Wizard
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, no Wizard
    for id in [wizard, bear] {
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    }
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: wizard, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    advance_to(&mut g, TurnStep::PostCombatMain);
    // 2 (Bear) + 2×2 (Wizard) = 6.
    assert_eq!(g.players[1].life, 14);
}

/// Vincent grows by the power of a dying opponent creature.
#[test]
fn vincent_valentine_eats_the_power_of_the_dead() {
    let mut g = two_player_game();
    let vincent = g.add_card_to_battlefield(0, catalog::vincent_valentine());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let wrath = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 3);
    cast(&mut g, wrath, Some(Target::Permanent(angel)));
    assert_eq!(
        g.battlefield_find(vincent).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(4),
        "Serra Angel's power"
    );
}

/// The Emperor's mana only funds noncreature spells.
#[test]
fn the_emperor_taps_for_noncreature_only_mana() {
    let d = catalog::the_emperor_of_palamecia();
    assert_eq!(d.activated_abilities.len(), 1);
    assert!(matches!(
        d.activated_abilities[0].effect,
        crabomination::effect::Effect::AddMana {
            pool: crabomination::effect::ManaPayload::Restricted(_, _),
            ..
        }
    ));
    let back = d.back_face.as_ref().expect("back face");
    assert_eq!(back.name, "The Lord Master of Hell");
}

/// Ultimecia's flip takes an extra turn.
#[test]
fn ultimecia_transforms_into_an_extra_turn() {
    let mut g = two_player_game();
    let ulti = g.add_card_to_battlefield(0, catalog::ultimecia_time_sorceress());
    let mut evs = vec![];
    g.transform_permanent(ulti, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ulti).unwrap().definition.name, "Ultimecia, Omnipotent");
    assert!(g.players[0].extra_turns > 0, "Time Compression banked an extra turn");
}

// ── The Dominant cycle ──────────────────────────────────────────────────────

/// Every Dominant is registered and carries a Saga-creature back face.
#[test]
fn dominants_are_registered_with_saga_creature_backs() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for f in [
        catalog::clive_ifrits_dominant as fn() -> crabomination::card::CardDefinition,
        catalog::dion_bahamuts_dominant,
        catalog::jill_shivas_dominant,
        catalog::joshua_phoenixs_dominant,
        catalog::jecht_reluctant_guardian,
    ] {
        let d = f();
        assert!(names.contains(&d.name), "{} is not registered", d.name);
        let back = d.back_face.as_ref().expect("back face");
        assert!(!back.saga_chapters.is_empty(), "{} flips into a Saga", d.name);
        assert!(back.is_creature(), "{} flips into a creature", d.name);
    }
}

/// The flip line exiles and returns the front face transformed, and the Saga
/// back enters with its first lore counter (CR 714.2b).
#[test]
fn dominant_flip_returns_a_saga_creature_with_a_lore_counter() {
    let mut g = two_player_game();
    let jill = g.add_card_to_battlefield(0, catalog::jill_shivas_dominant());
    g.battlefield_find_mut(jill).unwrap().summoning_sick = false;
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    g.players[0].mana_pool.add(Color::Blue, 5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: jill, ability_index: 0,
        target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("flip Jill");
    drain_stack(&mut g);
    let shiva = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Shiva, Warden of Ice")
        .expect("Shiva on the battlefield");
    assert_eq!(shiva.counters.get(&CounterType::Lore).copied(), Some(1));
    assert!(shiva.definition.is_creature() && !shiva.definition.saga_chapters.is_empty());
}

/// CR 714.4 — a Saga creature is still sacrificed at its final chapter, so the
/// cycle's last chapter resets itself front face up instead.
#[test]
fn saga_creature_resets_itself_on_its_last_chapter() {
    let mut g = two_player_game();
    let jill = g.add_card_to_battlefield(0, catalog::jill_shivas_dominant());
    g.battlefield_find_mut(jill).unwrap().summoning_sick = false;
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    g.add_card_to_battlefield(1, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: jill, ability_index: 0,
        target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("flip Jill");
    drain_stack(&mut g);
    // Push it to chapter III by hand — the turn-based lore counter only lands
    // on the controller's own precombat main.
    for _ in 0..2 {
        g.saga_advance(jill);
        drain_stack(&mut g);
    }
    let back = g.battlefield_find(jill).expect("survived chapter III");
    assert_eq!(back.definition.name, "Jill, Shiva's Dominant", "reset front face up");
    assert_eq!(back.counter_count(CounterType::Lore), 0);
}

/// Serah's discount covers only the turn's first legendary creature spell.
#[test]
fn serah_farron_discounts_the_first_legend_each_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::serah_farron());
    let legend_a = g.add_card_to_hand(0, catalog::vincent_valentine());
    let legend_b = g.add_card_to_hand(0, catalog::vincent_valentine());
    // Vincent is {2}{B}{B}; the discount makes the first one {B}{B}.
    g.players[0].mana_pool.add(Color::Black, 2);
    cast(&mut g, legend_a, None);
    assert!(g.battlefield_find(legend_a).is_some(), "the first legend was discounted");
    g.players[0].mana_pool.add(Color::Black, 2);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: legend_b,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the discount is spent for the turn"
    );
}

/// Crystallized Serah anthems your legends and keeps the discount.
#[test]
fn crystallized_serah_anthems_legends() {
    let mut g = two_player_game();
    let serah = g.add_card_to_battlefield(0, catalog::serah_farron());
    let legend = g.add_card_to_battlefield(0, catalog::vincent_valentine());
    let mut evs = vec![];
    g.transform_permanent(serah, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let back = g.battlefield_find(serah).unwrap();
    assert_eq!(back.definition.name, "Crystallized Serah");
    assert_eq!(g.computed_permanent(legend).unwrap().power, 4, "2/2 plus the anthem");
}

/// Venat draws off your first legendary spell each turn, once.
#[test]
fn venat_draws_off_the_first_legendary_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::venat_heart_of_hydaelyn());
    for i in 0..6 {
        g.players[0].library.push(CardInstance::new(CardId(600 + i), catalog::forest(), 0));
    }
    let a = g.add_card_to_hand(0, catalog::vincent_valentine());
    let b = g.add_card_to_hand(0, catalog::vincent_valentine());
    g.players[0].mana_pool.add(Color::Black, 8);
    let hand = g.players[0].hand.len();
    cast(&mut g, a, None);
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    cast(&mut g, b, None);
    assert_eq!(g.players[0].hand.len(), hand - 1, "the second legend draws nothing");
}

/// Garland returns from the graveyard transformed into Chaos, and Chaos bottoms
/// itself when it dies.
#[test]
fn garland_returns_transformed_and_chaos_bottoms_itself() {
    let mut g = two_player_game();
    let garland = g.add_card_to_graveyard(0, catalog::garland_knight_of_cornelia());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: garland,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("return Garland transformed");
    drain_stack(&mut g);
    let chaos = g.battlefield_find(garland).expect("back on the battlefield");
    assert_eq!(chaos.definition.name, "Chaos, the Endless");
    assert!(chaos.definition.keywords.contains(&Keyword::Flying));

    let mut evs = vec![];
    g.destroy_permanent(garland, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].library.last().map(|c| c.id),
        Some(garland),
        "Chaos went to the bottom of its owner's library"
    );
}

/// Crystal Fragments equips for +1/+1, then flips into the Saga creature with
/// its first lore counter.
#[test]
fn crystal_fragments_flips_into_alexander() {
    let mut g = two_player_game();
    let gear = g.add_card_to_battlefield(0, catalog::crystal_fragments());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(gear).unwrap().attached_to = Some(bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "2/2 plus the Equipment");

    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("flip Crystal Fragments");
    drain_stack(&mut g);
    let saga = g.battlefield_find(gear).expect("returned transformed");
    assert_eq!(saga.definition.name, "Summon: Alexander");
    assert_eq!(saga.counter_count(CounterType::Lore), 1, "CR 714.2b first chapter");
}

/// Zenos spares his chosen creature from the sweep, then transforms when it
/// leaves the battlefield.
#[test]
fn zenos_spares_the_chosen_creature_then_transforms() {
    let mut g = two_player_game();
    let chosen = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bystander = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let zenos = g.move_card_to_battlefield_for_test(0, catalog::zenos_yae_galvus());
    drain_stack(&mut g);
    // The ETB picks one of the opponent's creatures; the other one dies to -2/-2.
    let survivors: Vec<_> = [chosen, bystander]
        .into_iter()
        .filter(|id| g.battlefield_find(*id).is_some())
        .collect();
    assert_eq!(survivors.len(), 1, "the unchosen creature took -2/-2");
    assert_eq!(
        g.battlefield_find(zenos).unwrap().chosen_permanent,
        Some(survivors[0]),
        "the survivor is the remembered choice"
    );
    assert_eq!(g.computed_permanent(zenos).unwrap().power, 4, "Zenos spares himself");

    let mut evs = vec![];
    g.destroy_permanent(survivors[0], false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let back = g.battlefield_find(zenos).expect("still around");
    assert_eq!(back.definition.name, "Shinryu, Transcendent Rival");
    assert!(back.definition.keywords.contains(&Keyword::Flying));
}

/// The fourth "another creature dies" drain flips Sephiroth and mints the
/// Super Nova emblem, which keeps draining afterwards.
#[test]
fn sephiroth_transforms_on_the_fourth_drain() {
    let mut g = two_player_game();
    let seph = g.add_card_to_battlefield(0, catalog::sephiroth_fabled_soldier());
    let fodder: Vec<_> =
        (0..5).map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears())).collect();
    for (i, id) in fodder.iter().enumerate() {
        let mut evs = vec![];
        g.destroy_permanent(*id, false, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let flipped = g.battlefield_find(seph).unwrap().transformed;
        assert_eq!(flipped, i >= 3, "flips on the fourth death, not before");
    }
    assert_eq!(
        g.battlefield_find(seph).unwrap().definition.name,
        "Sephiroth, One-Winged Angel"
    );
    assert_eq!(g.players[0].emblems.len(), 1, "Super Nova landed with the flip");
    // Four front-face drains plus the emblem's on the fifth death.
    assert_eq!(g.players[1].life, 20 - 5);
    assert_eq!(g.players[0].life, 20 + 5);
}

/// Terra flips into the Saga, whose chapter copies a nonlegendary enchantment
/// and hands the copy three lore counters when it's a Saga.
#[test]
fn esper_terra_copies_a_saga_with_lore_counters() {
    let mut g = two_player_game();
    let terra = g.add_card_to_battlefield(0, catalog::terra_magical_adept());
    g.battlefield_find_mut(terra).unwrap().summoning_sick = false;
    let saga = g.add_card_to_battlefield(0, catalog::crystal_fragments());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Flip the Equipment to its Saga back face so there's a legal copy target.
    let mut evs = vec![];
    g.transform_permanent(saga, &mut evs);
    drain_stack(&mut g);

    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: terra,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("Trance");
    drain_stack(&mut g);
    let back = g.battlefield_find(terra).expect("returned transformed");
    assert_eq!(back.definition.name, "Esper Terra");
    // Chapter I minted a hasty copy of the Saga with three lore counters, so
    // the copy ran all three of its own chapters (III taps the opponent's
    // board) and was sacrificed to CR 714.4 on the way out.
    assert!(g.battlefield_find(victim).unwrap().tapped, "the copy's chapter III fired");
    assert!(
        !g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Summon: Alexander"),
        "the copy finished its final chapter and was sacrificed"
    );
}

/// Esper Origins is a plain surveil-and-gain from hand; flashed back from the
/// graveyard it returns as the Saga creature with a finality counter.
#[test]
fn esper_origins_returns_transformed_from_the_graveyard() {
    let mut g = two_player_game();
    let card = g.add_card_to_graveyard(0, catalog::esper_origins());
    for i in 0..6 {
        g.players[0].library.push(CardInstance::new(CardId(7100 + i), catalog::forest(), 0));
    }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFlashback {
        card_id: card,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flashback");
    drain_stack(&mut g);
    let maduin = g.battlefield_find(card).expect("on the battlefield, not exiled");
    assert_eq!(maduin.definition.name, "Summon: Esper Maduin");
    assert_eq!(maduin.counter_count(CounterType::Finality), 1);
    assert_eq!(maduin.counter_count(CounterType::Lore), 1, "CR 714.2b first chapter");
    assert_eq!(g.players[0].life, 22);
}
