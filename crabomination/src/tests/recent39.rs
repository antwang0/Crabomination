//! Functionality tests for `catalog::sets::decks::recent39` — defensive walls
//! and the new prevent-all-combat-damage-to-self static (CR 615).

use crate::card::Keyword;
use crate::catalog;
use crate::game::two_player_game;
use crate::game::types::{Attack, AttackTarget, TurnStep};
use crate::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// P0 attacks with `attacker`; P1's `wall` blocks; resolve through combat.
fn attack_into_wall(g: &mut GameState, attacker: CardId, wall: CardId) {
    g.clear_sickness(attacker);
    advance_to(g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(g);
    advance_to(g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, attacker)])).expect("block");
    drain_stack(g);
    advance_to(g, TurnStep::PostCombatMain);
}

#[test]
fn wall_of_denial_has_defender_flying_shroud() {
    let kw = catalog::wall_of_denial().keywords;
    assert!(kw.contains(&Keyword::Defender) && kw.contains(&Keyword::Flying) && kw.contains(&Keyword::Shroud));
}

#[test]
fn guard_gomazoa_takes_no_combat_damage() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let gomazoa = g.add_card_to_battlefield(1, catalog::guard_gomazoa()); // 1/3
    attack_into_wall(&mut g, attacker, gomazoa);
    assert!(g.battlefield_find(gomazoa).is_some(), "Gomazoa survives a 4-power hit");
    assert_eq!(g.battlefield_find(gomazoa).unwrap().damage, 0, "no combat damage marked");
}

#[test]
fn fog_bank_takes_no_combat_damage_and_deals_none() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let fog = g.add_card_to_battlefield(1, catalog::fog_bank()); // 0/2
    attack_into_wall(&mut g, attacker, fog);
    assert_eq!(g.battlefield_find(fog).unwrap().damage, 0, "Fog Bank takes no combat damage");
    assert!(catalog::fog_bank().keywords.contains(&Keyword::DealsNoCombatDamage));
}
