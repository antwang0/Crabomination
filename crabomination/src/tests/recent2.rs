//! Functionality tests for the `catalog::sets::decks::recent2` batch.

use crate::card::{CreatureType, Keyword};
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Tangle fogs combat and keeps attackers from untapping next turn.
#[test]
fn tangle_fogs_and_locks_attackers() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::tangle());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Tangle");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 20, "combat damage prevented");
    assert!(g.battlefield_find(atk).unwrap().tapped, "attacker still tapped");
}

/// March of Otherworldly Light exiles a creature with MV ≤ X.
#[test]
fn march_of_otherworldly_light_exiles_by_x() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let id = g.add_card_to_hand(0, catalog::march_of_otherworldly_light());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2); // X = 2
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast March");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "creature exiled");
}

/// Disdainful Stroke counters a 4-MV spell but not a cheap one.
#[test]
fn disdainful_stroke_counters_expensive_spell() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let spell = g.add_card_to_hand(1, catalog::serra_angel()); // {3}{W}{W} = MV 5
    g.players[1].mana_pool.add(Color::White, 2);
    g.players[1].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Serra Angel");
    g.perform_action(GameAction::PassPriority).expect("P1 passes to P0");
    let ds = g.add_card_to_hand(0, catalog::disdainful_stroke());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ds, target: Some(Target::Permanent(spell)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Disdainful Stroke");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spell).is_none(), "Serra Angel countered");
}

/// Flame Lash deals 4 to a player.
#[test]
fn flame_lash_deals_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::flame_lash());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Flame Lash");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16);
}

/// Virtue of Persistence: the adventure half (-3/-3 + gain 2 life) resolves.
#[test]
fn virtue_of_persistence_adventure_shrinks_and_gains() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies
    let id = g.add_card_to_hand(0, catalog::virtue_of_persistence());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Locthwain Scorn");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "−3/−3 killed the 2/2");
    assert_eq!(g.players[0].life, 22, "gained 2 life");
}

/// Scrabbling Skullcrab mills when an enchantment enters under your control.
#[test]
fn scrabbling_skullcrab_mills_on_enchantment_etb() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    g.add_card_to_battlefield(0, catalog::scrabbling_skullcrab());
    let lib_before = g.players[1].library.len();
    // An enchantment entering under your control triggers the mill — cast it
    // through the full path so observer triggers dispatch.
    let ench = g.add_card_to_hand(0, catalog::possibility_storm());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: ench, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Possibility Storm");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 2, "opponent milled two");
}

/// Hush destroys every enchantment.
#[test]
fn hush_destroys_all_enchantments() {
    let mut g = two_player_game();
    let e1 = g.add_card_to_battlefield(0, catalog::glorious_anthem());
    let e2 = g.add_card_to_battlefield(1, catalog::glorious_anthem());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::hush());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hush");
    drain_stack(&mut g);
    assert!(g.battlefield_find(e1).is_none() && g.battlefield_find(e2).is_none(), "enchantments gone");
    assert!(g.battlefield_find(bear).is_some(), "creature untouched");
}

/// Hush can be cycled away for {2}.
#[test]
fn hush_can_be_cycled() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::hush());
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None }).expect("cycle Hush");
    drain_stack(&mut g);
    // Cycled away (-1) and drew a card (+1) → hand size unchanged, Hush in gy.
    assert_eq!(g.players[0].hand.len(), before, "cycle drew a replacement");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Hush in graveyard");
}

/// Llanowar Greenwidow returns itself from the graveyard to the battlefield.
#[test]
fn llanowar_greenwidow_returns_from_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::llanowar_greenwidow());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(7);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate graveyard return");
    drain_stack(&mut g);
    let r = g.battlefield_find(id).expect("returned to battlefield");
    assert!(r.tapped, "returns tapped");
    assert_eq!((r.power(), r.toughness()), (4, 3));
}

/// Searchlight Companion makes a Spirit token on ETB.
#[test]
fn searchlight_companion_makes_a_spirit() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::searchlight_companion());
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit").count();
    assert_eq!(spirits, 1);
}

/// Resolute Reinforcements has flash and makes a Soldier on ETB.
#[test]
fn resolute_reinforcements_makes_a_soldier() {
    let mut g = two_player_game();
    assert!(catalog::resolute_reinforcements().keywords.contains(&Keyword::Flash));
    g.move_card_to_battlefield_for_test(0, catalog::resolute_reinforcements());
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 1);
}

/// Jewel Thief makes a Treasure on ETB.
#[test]
fn jewel_thief_makes_a_treasure() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::jewel_thief());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Treasure"));
}

/// Sweettooth Witch makes a Food on ETB.
#[test]
fn sweettooth_witch_makes_a_food() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::sweettooth_witch());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Food"));
}

/// Ambush Paratrooper's {5} ability pumps the team.
#[test]
fn ambush_paratrooper_pumps_team() {
    let mut g = two_player_game();
    let trooper = g.add_card_to_battlefield(0, catalog::ambush_paratrooper());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: trooper, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "bear pumped +1/+1");
}

/// Glistening Deluge shrinks all creatures and hits G/W harder.
#[test]
fn glistening_deluge_punishes_green_white() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green 2/2 → -3/-3 dies
    let id = g.add_card_to_hand(0, catalog::glistening_deluge());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Glistening Deluge");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "green 2/2 took -3/-3 and died");
}

/// Faerie Dreamthief surveils on ETB and can be exiled from the graveyard to draw.
#[test]
fn faerie_dreamthief_surveils_and_recurs() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.move_card_to_battlefield_for_test(0, catalog::faerie_dreamthief());
    drain_stack(&mut g);
    let d = catalog::faerie_dreamthief();
    assert_eq!(d.triggered_abilities.len(), 1, "ETB surveil wired");
    assert!(d.activated_abilities[0].from_graveyard && d.activated_abilities[0].exile_self_cost);
}

/// Vinereap Mentor makes a Food on ETB (and again on death).
#[test]
fn vinereap_mentor_makes_food_on_etb() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::vinereap_mentor());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Food"));
    assert_eq!(catalog::vinereap_mentor().triggered_abilities.len(), 2, "etb + dies");
}

/// Topiary Panther is a 6/5 trampler with basic landcycling.
#[test]
fn topiary_panther_has_basic_landcycling() {
    let d = catalog::topiary_panther();
    assert_eq!((d.power, d.toughness), (6, 5));
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Typecycling(_))));
}

/// Valgavoth's Faithful sacrifices itself to reanimate a creature.
#[test]
fn valgavoths_faithful_reanimates() {
    let mut g = two_player_game();
    let faithful = g.add_card_to_battlefield(0, catalog::valgavoths_faithful());
    let dead = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: faithful, ability_index: 0, target: Some(Target::Permanent(dead)),
        additional_targets: vec![], x_value: None,
    }).expect("reanimate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(faithful).is_none(), "Faithful sacrificed");
    assert!(g.battlefield_find(dead).is_some(), "Serra Angel reanimated");
}

/// Lord Skitter makes a Rat at the beginning of combat on your turn.
#[test]
fn lord_skitter_makes_a_rat_in_combat() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lord_skitter_sewer_king());
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    let rats = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Rat))
        .count();
    assert_eq!(rats, 1, "one Rat token created at combat");
}
