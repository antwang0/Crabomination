//! Functionality tests for `catalog::sets::decks::recent198`.

use crate::catalog;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};

/// Baseball Bat auto-attaches on ETB (+1/+1) and taps a creature when its
/// wielder attacks.
#[test]
fn baseball_bat_attaches_and_taps_on_attack() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bat = g.add_card_to_battlefield(0, catalog::baseball_bat());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(bearer)),
    ]));
    g.fire_self_etb_triggers(bat, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bat).unwrap().attached_to, Some(bearer), "attached to the bearer");
    assert_eq!(g.computed_permanent(bearer).unwrap().power, 3, "+1/+1 from the Bat");

    // Attack with the bearer → tap the opposing creature.
    g.clear_sickness(bearer);
    g.step = TurnStep::DeclareAttackers;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(foe)),
    ]));
    g.declare_attackers(vec![Attack { attacker: bearer, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "attack trigger tapped the foe");
}
