//! Functionality tests for the `catalog::sets::decks::recent12` Equipment batch.

use crate::card::Keyword;
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;

/// Attach `eq` to `creature` directly (test shortcut, bypassing the equip action).
fn attach(g: &mut GameState, eq: crate::card::CardId, creature: crate::card::CardId) {
    g.battlefield.iter_mut().find(|c| c.id == eq).unwrap().attached_to = Some(creature);
}

/// Leonin Shikari lets equip happen at instant speed (here: during combat).
#[test]
fn leonin_shikari_allows_instant_speed_equip() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.add_card_to_battlefield(0, catalog::leonin_shikari());
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::DeclareAttackers; // not sorcery-speed
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: boner, target: bear })
        .expect("Leonin Shikari permits equip at instant speed");
    assert_eq!(g.battlefield_find(boner).unwrap().attached_to, Some(bear));
}

/// Auriok Steelshaper makes equip cost {1} less.
#[test]
fn auriok_steelshaper_reduces_equip_cost() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Vulshok Morningstar's equip is {2}; with Auriok it drops to {1}.
    let star = g.add_card_to_battlefield(0, catalog::vulshok_morningstar());
    g.add_card_to_battlefield(0, catalog::auriok_steelshaper());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: star, target: bear })
        .expect("equip {2} reduced to {1}");
    assert_eq!(g.battlefield_find(star).unwrap().attached_to, Some(bear));
}

/// Kemba mints a Cat token for each Equipment attached to her at upkeep.
#[test]
fn kemba_makes_cat_per_attached_equipment() {
    let mut g = two_player_game();
    let kemba = g.add_card_to_battlefield(0, catalog::kemba_kha_regent());
    let e1 = g.add_card_to_battlefield(0, catalog::bonesplitter());
    let e2 = g.add_card_to_battlefield(0, catalog::vulshok_morningstar());
    attach(&mut g, e1, kemba);
    attach(&mut g, e2, kemba);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let cats = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Cat").count();
    assert_eq!(cats, 2, "one Cat per attached Equipment");
}

/// Goblin Gaveleer gains +2/+0 per attached Equipment.
#[test]
fn goblin_gaveleer_grows_per_equipment() {
    let mut g = two_player_game();
    let gav = g.add_card_to_battlefield(0, catalog::goblin_gaveleer());
    assert_eq!(g.computed_permanent(gav).unwrap().power, 1, "1/1 with nothing attached");
    // Cathar's Shield is +0/+3, so the +2 power is purely the per-Equipment bonus.
    let eq = g.add_card_to_battlefield(0, catalog::cathars_shield());
    attach(&mut g, eq, gav);
    assert_eq!(g.computed_permanent(gav).unwrap().power, 3, "+2/+0 from one Equipment");
}

/// Danitha makes Equipment spells cost {1} less — Leonin Scimitar ({1}) becomes
/// free.
#[test]
fn danitha_reduces_equipment_spell_cost() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::danitha_capashen());
    let scimitar = g.add_card_to_hand(0, catalog::leonin_scimitar());
    // No mana floated; {1} - {1} = {0}, so it should still cast.
    g.priority.player_with_priority = 0;
    cast(&mut g, scimitar);
    assert!(g.battlefield_find(scimitar).is_some(), "discounted Equipment spell resolved");
}

/// Maul of the Skyclaves attaches itself to a creature on ETB and grants flying.
#[test]
fn maul_etb_attaches_and_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let maul = g.move_card_to_battlefield_for_test(0, catalog::maul_of_the_skyclaves());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(maul).unwrap().attached_to, Some(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Embercleave attaches on ETB and grants double strike + trample.
#[test]
fn embercleave_etb_attaches_double_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cleave = g.move_card_to_battlefield_for_test(0, catalog::embercleave());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cleave).unwrap().attached_to, Some(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::DoubleStrike) && cp.keywords.contains(&Keyword::Trample));
}

/// Armory of Iroas puts a +1/+1 counter on the equipped creature when it attacks.
#[test]
fn armory_of_iroas_counter_on_attack() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let armory = g.add_card_to_battlefield(0, catalog::armory_of_iroas());
    attach(&mut g, armory, bear);
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("bear attacks");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Flayer Husk's living weapon mints a 0/0 Germ and attaches itself, making the
/// Germ a 1/1.
#[test]
fn flayer_husk_living_weapon_mints_germ() {
    let mut g = two_player_game();
    let husk = g.move_card_to_battlefield_for_test(0, catalog::flayer_husk());
    drain_stack(&mut g);
    let germ = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Germ").expect("Germ minted");
    let germ_id = germ.id;
    assert_eq!(g.battlefield_find(husk).unwrap().attached_to, Some(germ_id), "Husk attached to its Germ");
    let cp = g.computed_permanent(germ_id).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "0/0 + 1/1 = 1/1");
}

/// Lizard Blades is an Equipment creature that grants double strike when attached.
#[test]
fn lizard_blades_grants_double_strike_when_attached() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blades = g.add_card_to_battlefield(0, catalog::lizard_blades());
    attach(&mut g, blades, bear);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// Magnetic Theft attaches a target Equipment to a target creature (two-slot
/// instant).
#[test]
fn magnetic_theft_attaches_equipment() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
    let theft = g.add_card_to_hand(0, catalog::magnetic_theft());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: theft,
        target: Some(Target::Permanent(boner)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("cast Magnetic Theft");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(boner).unwrap().attached_to, Some(bear));
}

/// Sram's Expertise makes three Servo tokens.
#[test]
fn srams_expertise_makes_three_servos() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::srams_expertise());
    g.players[0].mana_pool.add(crate::mana::Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    cast(&mut g, id);
    let servos = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Servo").count();
    assert_eq!(servos, 3);
}

/// Nahiri's −2 exiles a tapped creature.
#[test]
fn nahiri_minus2_exiles_tapped_creature() {
    let mut g = two_player_game();
    let nahiri = g.add_card_to_battlefield(0, catalog::nahiri_the_harbinger());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == victim).unwrap().tapped = true;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: nahiri,
        ability_index: 1,
        target: Some(Target::Permanent(victim)),
        x_value: None,
    })
    .expect("activate -2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "tapped creature exiled");
    assert!(g.players[1].graveyard.iter().all(|c| c.id != victim), "to exile, not graveyard");
}

/// Valduk mints an Elemental token per attachment at the start of combat.
#[test]
fn valduk_makes_token_per_attachment() {
    let mut g = two_player_game();
    let valduk = g.add_card_to_battlefield(0, catalog::valduk_keeper_of_the_flame());
    let eq = g.add_card_to_battlefield(0, catalog::bonesplitter());
    attach(&mut g, eq, valduk);
    g.battlefield.iter_mut().find(|c| c.id == valduk).unwrap().summoning_sick = false;
    g.step = TurnStep::BeginCombat;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let elems = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Elemental").count();
    assert_eq!(elems, 1, "one Elemental per attached Equipment");
}
