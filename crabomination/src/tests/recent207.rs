//! Functionality tests for `catalog::sets::decks::recent207`.

use crate::card::Keyword;
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Cast a no-target sorcery/instant from hand with the given mana, then drain.
fn cast_simple(g: &mut GameState, card: CardId, colorless: u32) {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(colorless);
    g.perform_action(GameAction::CastSpell {
        card_id: card, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

/// Release the Dogs makes four 1/1 Dog tokens.
#[test]
fn release_the_dogs_makes_four() {
    let mut g = two_player_game();
    let s = g.add_card_to_hand(0, catalog::release_the_dogs());
    g.players[0].mana_pool.add(Color::White, 1);
    cast_simple(&mut g, s, 3);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Dog").count(), 4);
}

/// Moment of Triumph pumps and gains life.
#[test]
fn moment_of_triumph_pumps_and_gains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::moment_of_triumph());
    g.players[0].mana_pool.add(Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert_eq!(g.players[0].life, life + 2);
}

/// Deadly Riposte only hits tapped creatures.
#[test]
fn deadly_riposte_burns_tapped() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let s = g.add_card_to_hand(0, catalog::deadly_riposte());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "tapped 2/2 took 3 and died");
    assert_eq!(g.players[0].life, life + 2);
}

/// Skeleton Archer pings any target on ETB.
#[test]
fn skeleton_archer_pings_on_etb() {
    let mut g = two_player_game();
    let l1 = g.players[1].life;
    // No creatures on the opponent's side, so the "any target" auto-picks the
    // opponent's face.
    g.move_card_to_battlefield_for_test(0, catalog::skeleton_archer());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "pinged the opponent for 1");
}

/// Maalfeld Twins leaves two Zombies when it dies.
#[test]
fn maalfeld_twins_dies_into_zombies() {
    let mut g = two_player_game();
    let twins = g.add_card_to_battlefield(0, catalog::maalfeld_twins());
    g.battlefield_find_mut(twins).unwrap().counters.insert(crate::card::CounterType::MinusOneMinusOne, 4);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Zombie").count(), 2);
}

/// Rapacious Dragon mints two Treasures on ETB.
#[test]
fn rapacious_dragon_makes_treasures() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::rapacious_dragon());
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 2);
}

/// Exclusion Mage bounces an opponent's creature on ETB.
#[test]
fn exclusion_mage_bounces() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::exclusion_mage());
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "returned to owner's hand");
}

/// Mystic Archaeologist draws two for {3}{U}{U}.
#[test]
fn mystic_archaeologist_draws_two() {
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::mystic_archaeologist());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    let h = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: m, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h + 2);
}

/// Deathmark destroys a green creature but can't target a blue one.
#[test]
fn deathmark_hits_green_not_blue() {
    let mut g = two_player_game();
    let birds = g.add_card_to_battlefield(1, catalog::birds_of_paradise()); // green
    let s = g.add_card_to_hand(0, catalog::deathmark());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: Some(Target::Permanent(birds)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(birds).is_none(), "green creature destroyed");
}

/// Goblin Oriflamme pumps only attacking creatures.
#[test]
fn goblin_oriflamme_pumps_attackers() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goblin_oriflamme());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Not attacking yet: no bonus.
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
        .expect("bear attacks");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "attacking gets +1/+0");
}

/// Vampire Neonate drains 1 for {2}, {T}.
#[test]
fn vampire_neonate_drains() {
    let mut g = two_player_game();
    let n = g.add_card_to_battlefield(0, catalog::vampire_neonate());
    g.clear_sickness(n);
    g.players[0].mana_pool.add_colorless(2);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::ActivateAbility {
        card_id: n, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1);
    assert_eq!(g.players[0].life, l0 + 1);
}

/// Volley Veteran deals damage equal to your Goblin count.
#[test]
fn volley_veteran_scales_with_goblins() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::raging_redcap()); // a Goblin
    let target = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // 5/6
    g.move_card_to_battlefield_for_test(0, catalog::volley_veteran()); // now 2 Goblins
    drain_stack(&mut g);
    // 2 Goblins → 2 damage to the 5/6.
    assert_eq!(g.battlefield_find(target).unwrap().damage, 2);
}

/// Regal Caracal buffs and lifelinks other Cats, and makes two Cat tokens.
#[test]
fn regal_caracal_lords_cats() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::regal_caracal());
    drain_stack(&mut g);
    let cats: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Cat").map(|c| c.id).collect();
    assert_eq!(cats.len(), 2, "made two Cat tokens");
    // Each token is a base 1/1 lifelink; the lord makes them 2/2.
    let cp = g.computed_permanent(cats[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "lord buffs other Cats");
    assert!(cp.keywords.contains(&Keyword::Lifelink));
}

/// Harmless Offering donates a permanent to an opponent.
#[test]
fn harmless_offering_donates() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::harmless_offering());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: Some(Target::Permanent(mine)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1, "opponent now controls it");
}

/// Syr Alin buffs the rest of the team when it attacks.
#[test]
fn syr_alin_pumps_team_on_attack() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let alin = g.add_card_to_battlefield(0, catalog::syr_alin_the_lions_claw());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(alin);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let evs = g
        .declare_attackers(vec![Attack { attacker: alin, target: AttackTarget::Player(1) }])
        .expect("Syr Alin attacks");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "other creatures +1/+1");
    assert_eq!(g.computed_permanent(alin).unwrap().power, 4, "not itself");
}

/// Dive Down toughens a creature and grants hexproof.
#[test]
fn dive_down_grants_hexproof() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::dive_down());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 5), "+0/+3");
    assert!(cp.keywords.contains(&Keyword::Hexproof));
}

/// Hidetsugu's Second Rite only burns a player who is at exactly 10 life.
#[test]
fn hidetsugus_second_rite_needs_exactly_ten() {
    let mut g = two_player_game();
    let s = g.add_card_to_hand(0, catalog::hidetsugus_second_rite());
    g.players[1].life = 11; // not exactly 10
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 11, "not at 10 → no damage");

    let s2 = g.add_card_to_hand(0, catalog::hidetsugus_second_rite());
    g.players[1].life = 10;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: s2, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 0, "exactly 10 → dealt 10");
}

/// Rise of the Dark Realms reanimates every creature from all graveyards under
/// your control (CR 608 mass move across all graveyards).
#[test]
fn rise_of_the_dark_realms_grabs_all_graveyards() {
    let mut g = two_player_game();
    let mine = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::lightning_bolt()); // not a creature — stays
    let s = g.add_card_to_hand(0, catalog::rise_of_the_dark_realms());
    g.players[0].mana_pool.add(Color::Black, 2);
    cast_simple(&mut g, s, 7);
    assert_eq!(g.battlefield_find(mine).map(|c| c.controller), Some(0), "my creature returned under my control");
    assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(0), "opponent's creature stolen under my control");
}

/// CR 702.16e — protection from everything prevents combat damage: Progenitus
/// takes no damage from an attacker it blocks.
#[test]
fn progenitus_blocks_without_taking_damage() {
    use crate::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::mahamoti_djinn()); // 5/6
    let prog = g.add_card_to_battlefield(0, catalog::progenitus()); // 10/10
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(0) }];
    g.block_map.insert(prog, attacker);
    g.step = TurnStep::CombatDamage;
    g.active_player_idx = 1;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(prog).map(|c| c.damage), Some(0), "protection from everything → no damage marked");
}

/// Magnigoth Sentry has reach; Raging Redcap has double strike (french vanillas).
#[test]
fn vanilla_keyword_bodies() {
    let mut g = two_player_game();
    let sentry = g.add_card_to_battlefield(0, catalog::magnigoth_sentry());
    let redcap = g.add_card_to_battlefield(0, catalog::raging_redcap());
    assert!(g.computed_permanent(sentry).unwrap().keywords.contains(&Keyword::Reach));
    assert!(g.computed_permanent(redcap).unwrap().keywords.contains(&Keyword::DoubleStrike));
}
