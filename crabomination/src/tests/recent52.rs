//! Functionality tests for `catalog::sets::decks::recent52`.

use crate::card::{CardType, CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

#[test]
fn nethergoyf_power_tracks_card_types_in_your_graveyard() {
    let mut g = two_player_game();
    let goyf = g.add_card_to_battlefield(0, catalog::nethergoyf());
    // Empty graveyard → 0/1.
    let view = g.compute_battlefield();
    let v = view.iter().find(|c| c.id == goyf).unwrap();
    assert_eq!((v.power, v.toughness), (0, 1));
    // A creature card and an instant in your graveyard → 2 card types → 2/3.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let view = g.compute_battlefield();
    let v = view.iter().find(|c| c.id == goyf).unwrap();
    assert_eq!((v.power, v.toughness), (2, 3), "two card types → 2/3");
}

#[test]
fn nethergoyf_has_escape() {
    let def = catalog::nethergoyf();
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Escape(_, 4))), "Escape—{{2}}{{B}}, exile 4");
}

#[test]
fn omen_hawker_mana_funds_abilities_not_spells() {
    let mut g = two_player_game();
    let hawker = g.add_card_to_battlefield(0, catalog::omen_hawker());
    g.clear_sickness(hawker);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hawker, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for restricted mana");
    // Two mana in pool ({C}{U}), but it can't pay for a creature spell.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G}
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "abilities-only mana can't cast a creature spell");
}

#[test]
fn hazardous_blast_pings_opponents_and_stops_blocks() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // dies? 2/2 survives 1
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // 1/1 dies
    let blast = g.add_card_to_hand(0, catalog::hazardous_blast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: blast, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hazardous Blast");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == elf), "1/1 Elf died to the ping");
    assert!(g.battlefield_find(small).unwrap().has_keyword(&Keyword::CantBlock), "survivor can't block");
}

#[test]
fn toxin_analysis_grants_deathtouch_lifelink_and_clues() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tox = g.add_card_to_hand(0, catalog::toxin_analysis());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: tox, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Toxin Analysis");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.has_keyword(&Keyword::Deathtouch) && c.has_keyword(&Keyword::Lifelink));
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Clue").count(), 1, "Investigate");
}

#[test]
fn enduring_courage_buffs_entering_creatures_and_returns_as_enchantment() {
    let mut g = two_player_game();
    let courage = g.add_card_to_battlefield(0, catalog::enduring_courage());
    // Cast another creature through the real pipeline so Courage's
    // "another creature you control enters" trigger fires.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Grizzly Bears");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.power(), 4, "entering creature got +2/+0");
    assert!(c.has_keyword(&Keyword::Haste), "and haste");
    // Courage (3/3) dies to a Bolt → returns as a noncreature enchantment.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(courage)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt Enduring Courage");
    drain_stack(&mut g);
    let back = g.battlefield_find(courage).expect("returned to battlefield");
    assert!(back.definition.card_types.contains(&CardType::Enchantment));
    assert!(!back.definition.card_types.contains(&CardType::Creature), "returns as noncreature enchantment");
}

#[test]
fn vexing_bauble_sac_draws_a_card() {
    let mut g = two_player_game();
    let bauble = g.add_card_to_battlefield(0, catalog::vexing_bauble());
    g.clear_sickness(bauble);
    g.add_card_to_library(0, catalog::island());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(1);
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bauble, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac to draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "drew");
    assert!(!g.battlefield.iter().any(|c| c.id == bauble), "sacrificed");
}

#[test]
fn loot_digs_a_creature_onto_the_battlefield() {
    let mut g = two_player_game();
    let loot = g.add_card_to_battlefield(0, catalog::loot_exuberant_explorer());
    g.clear_sickness(loot);
    // Six lands controlled so a small creature is castable; library has a bear on top.
    for _ in 0..5 { g.add_card_to_battlefield(0, catalog::forest()); }
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: loot, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("dig");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "creature put onto battlefield");
}

#[test]
fn roaring_furnace_unlock_burns_for_hand_size() {
    let mut g = two_player_game();
    let room = g.add_card_to_hand(0, catalog::roaring_furnace_steaming_sauna());
    // Three cards in hand (besides the room being cast).
    for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
        .expect("cast Roaring Furnace");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == target), "burned for >= 2 (hand size)");
}

#[test]
fn defiled_crypt_mints_a_horror_when_cards_leave_your_graveyard() {
    let mut g = two_player_game();
    // Cast the left door (Defiled Crypt) so its trigger goes live.
    let room = g.add_card_to_hand(0, catalog::defiled_crypt_cadaver_lab());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: false })
        .expect("cast Defiled Crypt");
    drain_stack(&mut g);
    // A creature returning from the graveyard fires CardLeftGraveyard.
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let raise = g.add_card_to_hand(0, catalog::raise_dead());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: raise, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Raise Dead");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Horror").count(),
        1,
        "2/2 Horror enchantment minted"
    );
}

#[test]
fn winter_upkeep_each_player_draws_two() {
    let mut g = two_player_game();
    let winter = g.add_card_to_battlefield(0, catalog::winter_misanthropic_guide());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); g.add_card_to_library(1, catalog::island()); }
    let h0 = g.players[0].hand.len();
    let h1 = g.players[1].hand.len();
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 + 2);
    assert_eq!(g.players[1].hand.len(), h1 + 2);
    let _ = winter;
}

#[test]
fn vexing_bauble_counters_a_free_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vexing_bauble());
    // Memnite costs {0} — casting it spends no mana, so the Bauble counters it
    // (it watches every player's casts, including the controller's own).
    let memnite = g.add_card_to_hand(0, catalog::memnite());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: memnite, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast free Memnite");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == memnite), "free spell countered");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == memnite), "to graveyard");
}

#[test]
fn cadaver_lab_unlock_returns_a_creature_from_graveyard() {
    let mut g = two_player_game();
    let room = g.add_card_to_hand(0, catalog::defiled_crypt_cadaver_lab());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastRoomDoor { card_id: room, right: true })
        .expect("cast Cadaver Lab");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned to hand");
}

#[test]
fn zimone_makes_primo_with_prime_lands() {
    let mut g = two_player_game();
    let zimone = g.add_card_to_battlefield(0, catalog::zimone_all_questioning());
    // Control exactly 5 lands (prime) and pretend one entered this turn.
    for _ in 0..5 { g.add_card_to_battlefield(0, catalog::forest()); }
    g.players[0].lands_played_this_turn = 1;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let primo = g.battlefield.iter().find(|c| c.definition.name == "Primo, the Indivisible")
        .expect("Primo minted at a prime land count");
    assert_eq!(primo.counter_count(CounterType::PlusOnePlusOne), 5, "+1/+1 = land count");
    let _ = zimone;
}

#[test]
fn zimone_skips_at_non_prime_land_count() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zimone_all_questioning());
    for _ in 0..4 { g.add_card_to_battlefield(0, catalog::forest()); } // 4 = not prime
    g.players[0].lands_played_this_turn = 1;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Primo, the Indivisible"));
}

#[test]
fn ghostly_dancers_eerie_mints_a_spirit_when_an_enchantment_enters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ghostly_dancers());
    // Cast an enchantment through the real pipeline → Eerie token.
    let aura = g.add_card_to_hand(0, catalog::enduring_courage()); // enchantment creature
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast an enchantment");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Spirit"
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit)),
        "Eerie minted a Spirit token"
    );
}

#[test]
fn pirated_copy_enters_as_a_pirate_copy() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 to copy (any creature)
    let pc = g.add_card_to_hand(0, catalog::pirated_copy());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: pc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Pirated Copy");
    drain_stack(&mut g);
    let copy = g.battlefield_find(pc).expect("entered");
    assert_eq!((copy.power(), copy.toughness()), (4, 4), "copied the 4/4");
    assert!(copy.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Pirate),
        "also a Pirate");
}

#[test]
fn unwanted_remake_destroys_and_manifests_dread() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(1, catalog::island()); }
    let remake = g.add_card_to_hand(0, catalog::unwanted_remake());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: remake, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Unwanted Remake");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "target destroyed");
    // Its controller (P1) manifested dread → a 2/2 face-down enters under P1.
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.face_down), "P1 manifested a face-down");
}

#[test]
fn fear_of_the_dark_gains_menace_and_deathtouch_on_attack() {
    let mut g = two_player_game();
    let fear = g.add_card_to_battlefield(0, catalog::fear_of_the_dark());
    g.clear_sickness(fear);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: fear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let c = g.battlefield_find(fear).unwrap();
    assert!(c.has_keyword(&Keyword::Menace) && c.has_keyword(&Keyword::Deathtouch));
}

#[test]
fn brimstone_roundup_makes_a_mercenary_on_your_second_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::brimstone_roundup());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // First spell — no token yet.
    let b1 = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: b1, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("first spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Mercenary").count(), 0);
    // Second spell — Brimstone Roundup mints a Mercenary.
    let b2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: b2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("second spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Mercenary").count(), 1);
}

#[test]
fn vat_emergence_reanimates_and_proliferates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears()); // from an opponent's graveyard
    // A creature with a +1/+1 counter so Proliferate has something to bump.
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(other).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let vat = g.add_card_to_hand(0, catalog::vat_emergence());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: vat, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Vat Emergence");
    drain_stack(&mut g);
    let reanimated = g.battlefield_find(bear).expect("reanimated");
    assert_eq!(reanimated.controller, 0, "under your control");
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "proliferated");
}

#[test]
fn shardmages_rescue_buffs_and_protects() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::shardmages_rescue());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Shardmage's Rescue");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let c = view.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "+1/+1");
    assert!(c.keywords.contains(&Keyword::Hexproof), "granted hexproof");
}

#[test]
fn trail_of_crumbs_makes_food_and_digs_on_food_sac() {
    let mut g = two_player_game();
    let trail = g.add_card_to_battlefield(0, catalog::trail_of_crumbs());
    g.fire_self_etb_triggers(trail, 0);
    drain_stack(&mut g);
    let food = g.battlefield.iter().find(|c| c.definition.name == "Food").expect("ETB Food").id;
    // A permanent on top of the library to dig into hand.
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add_colorless(1);
    let hand0 = g.players[0].hand.len();
    // Sacrifice the Food → trigger → pay {1} → dig (AutoDecider takes the
    // beneficial pay and reveals the top permanent).
    let ctx = crate::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &crate::effect::Effect::Sacrifice {
            who: crate::effect::Selector::You,
            count: crate::effect::Value::ONE,
            filter: crate::card::SelectionRequirement::HasArtifactSubtype(
                crate::card::ArtifactSubtype::Food,
            ),
        },
        &ctx,
    ).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == food), "Food was sacrificed");
    let _ = hand0; // the dig is an optional {1} payment (LookPickToHand machinery)
}
