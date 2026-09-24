//! CR 613 / 611.3 — a static's affected set can depend on live state (an
//! attacking creature, an enchanted one, a card's name). Such a filter has no
//! printed-characteristics route through the layer system; the gather resolves
//! it on the board. Attacking, modified and equipped filters already took that
//! path; eight cards whose leaf it didn't know (an enchanted creature, a name,
//! the source's choice, …) had their static dropped whole.

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn power(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).expect("on the battlefield").power
}

/// Goblin Oriflamme — "Attacking creatures you control get +1/+0": the pump
/// applies once the creature attacks, and not before.
#[test]
fn cr_611_3_goblin_oriflamme_pumps_only_attackers() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::goblin_oriflamme());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let home = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(power(&g, bear), 2);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(power(&g, bear), 3);
    assert_eq!(power(&g, home), 2, "a creature at home isn't attacking");
}

/// Song of Serenity — "Creatures that are enchanted can't attack or block":
/// a creature wearing Pacifism's neighbour picks up the restriction, an
/// unenchanted one doesn't.
#[test]
fn cr_611_3_song_of_serenity_binds_enchanted_creatures() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::song_of_serenity());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::pacifism());
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enchant");
    drain_stack(&mut g);
    let kws = g.computed_permanent(bear).unwrap().keywords().to_vec();
    assert!(kws.contains(&Keyword::CantAttack) && kws.contains(&Keyword::CantBlock), "{kws:?}");
    assert!(!g.computed_permanent(other).unwrap().keywords().contains(&Keyword::CantBlock));
}

/// Shield of Kaldra — Equipment *named* Sword/Shield/Helm of Kaldra have
/// indestructible; another Equipment doesn't.
#[test]
fn cr_611_3_shield_of_kaldra_protects_the_named_set() {
    let mut g = main_phase();
    let shield = g.add_card_to_battlefield(0, catalog::shield_of_kaldra());
    assert!(g.computed_permanent(shield).unwrap().keywords().contains(&Keyword::Indestructible));
    let boots = g.add_card_to_battlefield(0, catalog::swiftfoot_boots());
    assert!(!g.computed_permanent(boots).unwrap().keywords().contains(&Keyword::Indestructible));
}
