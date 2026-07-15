//! Functionality tests for `catalog::sets::decks::recent223`.

use crate::card::Keyword;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::effects::EffectContext;
use crate::game::types::{Attack, AttackTarget, Target, TurnStep};
use crate::game::{drain_stack, two_player_game, GameState};
use crate::mana::Color;

/// Warren Warleader carries Offspring and its attack trigger can mint a Rabbit.
#[test]
fn warren_warleader_attack_makes_a_rabbit() {
    let warleader = catalog::warren_warleader();
    assert!(warleader.keywords.iter().any(|k| matches!(k, Keyword::Offspring(_))), "has Offspring");
    let mut g = two_player_game();
    let wl = g.add_card_to_battlefield(0, catalog::warren_warleader());
    g.clear_sickness(wl);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)])); // make a Rabbit
    let rabbits = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Rabbit").count();
    g.declare_attackers(vec![Attack { attacker: wl, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(rabbits(&g), 1, "attack mode 0 minted a Rabbit token");
}

/// For the Common Good makes X token copies, shields all your tokens, and gains
/// life per token.
#[test]
fn for_the_common_good_copies_and_shields() {
    let mut g = two_player_game();
    let orig = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(orig).unwrap().is_token = true;
    let tokens = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).count();
    let life = g.players[0].life;
    let effect = catalog::for_the_common_good().effect.clone();
    let ctx = EffectContext {
        x_value: 2,
        targets: vec![Target::Permanent(orig)],
        ..EffectContext::for_ability(orig, 0, None)
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(tokens(&g), 3, "two copies join the original token");
    assert_eq!(g.players[0].life, life + 3, "gained 1 life per token");
    assert!(g.computed_permanent(orig).unwrap().keywords.contains(&Keyword::Indestructible), "tokens shielded");
}
