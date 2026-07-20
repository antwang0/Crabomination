//! Tests for the recent297 Ravnica batch 7 (Boros aggro, Radiance pump, mill).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameState, TurnStep};
use crabomination::mana::Color;

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 6);
    }
    g.players[0].mana_pool.add_colorless(8);
}

#[test]
fn wojek_siren_pumps_the_color_group() {
    let mut g = two_player_game();
    let g1 = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    let g2 = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    let white = g.add_card_to_battlefield(0, catalog::serra_angel()); // white
    let spell = g.add_card_to_hand(0, catalog::wojek_siren());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(g1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(g2).unwrap().power, 3, "shared green → +1/+1");
    assert_eq!(g.computed_permanent(white).unwrap().power, 4, "white unaffected (still 4/4)");
}

#[test]
fn flame_kin_zealot_pumps_and_hastes_the_team() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let zealot = g.add_card_to_hand(0, catalog::flame_kin_zealot());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: zealot, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "2/2 → 3/3");
    assert!(cp.keywords.contains(&Keyword::Haste), "team gained haste");
}

#[test]
fn agrus_kos_pumps_by_color_on_attack() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let agrus = g.add_card_to_battlefield(0, catalog::agrus_kos_wojek_veteran()); // R/W 3/3
    let red = g.add_card_to_battlefield(0, catalog::goblin_guide()); // red
    let white = g.add_card_to_battlefield(0, catalog::serra_angel()); // white
    for id in [agrus, red, white] {
        g.clear_sickness(id);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: agrus, target: AttackTarget::Player(1) },
        Attack { attacker: red, target: AttackTarget::Player(1) },
        Attack { attacker: white, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(red).unwrap().power, 4, "red attacker +2/+0 (2→4)");
    assert_eq!(g.computed_permanent(white).unwrap().toughness, 6, "white attacker +0/+2 (4→6)");
}

#[test]
fn sunhome_guildmage_makes_a_hasty_soldier() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::sunhome_guildmage());
    g.clear_sickness(mage);
    flood(&mut g);
    let before = g.battlefield.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("make token");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), before + 1, "one Soldier token minted");
    let tok = g.battlefield.iter().find(|c| c.definition.name == "Soldier").unwrap();
    assert!(tok.definition.keywords.contains(&Keyword::Haste), "token has haste");
}

#[test]
fn necromancers_assistant_mills_three() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let gy = g.players[0].graveyard.len();
    let nec = g.add_card_to_hand(0, catalog::necromancers_assistant());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: nec, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy + 3, "milled three cards");
}

#[test]
fn mark_of_eviction_bounces_creature_and_itself_on_upkeep() {
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::mark_of_eviction());
    let ctx = EffectContext::for_ability(aura, 0, Some(Target::Permanent(bear)));
    g.resolve_effect(&catalog::mark_of_eviction().effect, &ctx).unwrap();
    drain_stack(&mut g);
    // Fire the upkeep trigger from the aura's equip bonus.
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "aura returned to hand");
}
