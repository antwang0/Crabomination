//! Functionality tests for `catalog::sets::decks::recent78`.

use crate::card::{Keyword, LandType};
use crate::catalog;
use crate::game::two_player_game;
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

#[test]
fn giant_crab_grants_shroud() {
    let mut g = two_player_game();
    let crab = g.add_card_to_battlefield(0, catalog::giant_crab());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: crab, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{U}: shroud");
    drain_stack(&mut g);
    assert!(g.computed_permanent(crab).unwrap().keywords.contains(&Keyword::Shroud));
}

#[test]
fn wall_of_wonder_can_attack_despite_defender() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_wonder());
    g.clear_sickness(wall);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wall, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("+4/-4 & attack despite defender");
    drain_stack(&mut g);
    let p = g.computed_permanent(wall).unwrap();
    assert_eq!((p.power, p.toughness), (5, 1), "1/5 → 5/1");
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: wall, target: AttackTarget::Player(1),
    }])).expect("Defender attacks after the grant");
}

#[test]
fn instill_energy_untaps_once_per_turn() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::instill_energy());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "granted haste");
    g.battlefield_find(bear).map(|_| ());
    // Tap the bear, then the {0} untap ability should free it.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) { c.tapped = true; }
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{0}: untap");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped by Instill Energy");
    // Second activation the same turn is illegal.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) { c.tapped = true; }
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "once each turn");
}

#[test]
fn erg_raiders_pings_you_if_it_didnt_attack() {
    let mut g = two_player_game();
    let raider = g.add_card_to_battlefield(0, catalog::erg_raiders());
    g.clear_sickness(raider); // pretend it's been around (didn't enter this turn)
    let me = g.players[0].life;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, me - 2, "didn't attack → 2 damage to you");
}

#[test]
fn erg_raiders_no_ping_the_turn_it_enters() {
    let mut g = two_player_game();
    // Cast it this turn so it properly "came under your control this turn".
    let raider = g.add_card_to_hand(0, catalog::erg_raiders());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: raider, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let me = g.players[0].life;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, me, "entered this turn → no self-damage");
}

#[test]
fn foul_familiar_bounces_itself() {
    let mut g = two_player_game();
    let f = g.add_card_to_battlefield(0, catalog::foul_familiar());
    g.clear_sickness(f);
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: f, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{B}, pay 1 life: bounce");
    drain_stack(&mut g);
    assert!(g.battlefield_find(f).is_none(), "returned to hand");
    assert_eq!(g.players[0].hand.len(), hand + 1, "back in hand");
}

#[test]
fn fire_snake_destroys_a_land_on_death() {
    let mut g = two_player_game();
    let snake = g.add_card_to_battlefield(0, catalog::fire_snake()); // 3/1
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let mut events = Vec::new();
    g.deal_damage_to_from(crate::game::effects::EntityRef::Permanent(snake), 1, None, &mut events);
    g.check_state_based_actions();
    // The dies trigger is on the stack targeting the land.
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land destroyed by Fire Snake's death");
}

#[test]
fn dread_reaper_costs_five_life() {
    let mut g = two_player_game();
    let me = g.players[0].life;
    let rd = g.add_card_to_hand(0, catalog::dread_reaper());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: rd, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, me - 5, "ETB lose 5 life");
}

#[test]
fn elven_cache_returns_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_from_battlefield_to_graveyard_raw(bear);
    let cache = g.add_card_to_hand(0, catalog::elven_cache());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: cache, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("regrow the bear");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear back in hand");
    // -1 for the cast Elven Cache, +1 for the returned bear.
    assert_eq!(g.players[0].hand.len(), hand);
}

#[test]
fn dwarven_soldier_toughens_against_orcs() {
    let mut g = two_player_game();
    let soldier = g.add_card_to_battlefield(1, catalog::dwarven_soldier());
    g.clear_sickness(soldier);
    let orc = g.add_card_to_battlefield(0, catalog::ironclaw_orcs()); // Orc attacker
    g.clear_sickness(orc);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: orc, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(soldier, orc)])).expect("block the Orc");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(soldier).unwrap().toughness, 3, "2/1 → 2/3 blocking an Orc");
}

#[test]
fn talas_warrior_is_unblockable_flyer_stats() {
    let w = catalog::talas_warrior();
    assert!(w.keywords.contains(&Keyword::Unblockable));
    assert_eq!((w.power, w.toughness), (2, 2));
}

#[test]
fn fear_grants_fear() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::fear());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Fear));
}

#[test]
fn wanderlust_pings_enchanted_creatures_controller() {
    let mut g = two_player_game();
    let foe_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::wanderlust());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(foe_creature)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant the opponent's creature");
    drain_stack(&mut g);
    let foe = g.players[1].life;
    // It's the enchanted creature's controller (P1) whose upkeep triggers the
    // ping. P0's upkeep must NOT fire it.
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe, "no ping on the caster's upkeep");
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe - 1, "1 damage at the enchanted controller's upkeep");
}

#[test]
fn warp_artifact_and_cursed_land_are_upkeep_ping_auras() {
    // Structural: both are Auras with a single upkeep-triggered ability.
    for card in [catalog::warp_artifact(), catalog::cursed_land()] {
        assert!(card.card_types.contains(&crate::card::CardType::Enchantment));
        assert_eq!(card.triggered_abilities.len(), 1, "one upkeep ping trigger");
    }
}

// ── Comp-rules sections implemented / verified this run ──────────────────────

#[test]
fn cr_702_15_mountainwalk_unblockable_with_a_mountain() {
    // CR 702.15 — landwalk: the attacker can't be blocked while the defending
    // player controls a land of the named type (enforced in declare_blockers).
    let mut g = two_player_game();
    let yeti = g.add_card_to_battlefield(0, catalog::mountain_yeti());
    g.clear_sickness(yeti);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(blocker);
    g.add_card_to_battlefield(1, catalog::mountain());
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: yeti, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(blocker, yeti)])).is_err(),
        "can't block a mountainwalker while defending player controls a Mountain");
}

#[test]
fn cr_702_23_rampage_grows_per_extra_blocker() {
    // CR 702.23 — Rampage N: +N/+N for each blocker beyond the first.
    let mut g = two_player_game();
    let giant = g.add_card_to_battlefield(0, catalog::frost_giant()); // 4/4 rampage 2
    g.clear_sickness(giant);
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(b1);
    g.clear_sickness(b2);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: giant, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, giant), (b2, giant)])).expect("double block");
    drain_stack(&mut g);
    let p = g.computed_permanent(giant).unwrap();
    assert_eq!((p.power, p.toughness), (6, 6), "4/4 + rampage 2 for one extra blocker → 6/6");
}

#[test]
fn cr_603_10_carapace_sac_regenerates_enchanted_via_lki() {
    // CR 603.10 — a sac_cost ability whose body reads the enchanted creature
    // resolves via last-known-information after the Aura leaves the battlefield.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::carapace());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant");
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac to regenerate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "Carapace sacrificed");
    assert_eq!(g.battlefield_find(bear).unwrap().regeneration_shields, 1,
        "enchanted creature gets a regen shield via LKI");
}
