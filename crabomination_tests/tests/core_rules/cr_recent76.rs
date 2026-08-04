//! CR conformance: 115.7c multi-slot retargeting, 601.2c distinct slots,
//! 706.10 activated-ability copies, and the 104.4b / 732.3 loop guards.

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

/// CR 115.7c — "choose new targets" repoints *every* declared slot, not just
/// slot 0: Redirect pulls both halves of Snow Day off my creatures.
#[test]
fn cr_115_7c_redirect_repoints_every_slot() {
    let mut g = two_player_game();
    let mine_a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mine_b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let snow = g.add_card_to_hand(1, catalog::snow_day());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.players[1].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: snow,
        target: Some(Target::Permanent(mine_a)),
        additional_targets: vec![Target::Permanent(mine_b)],
        mode: None,
        x_value: None,
    })
    .expect("opponent freezes both my creatures");

    let redirect = g.add_card_to_hand(0, catalog::redirect());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: redirect,
        target: Some(Target::Permanent(snow)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Redirect");
    drain_stack(&mut g);

    assert!(!g.battlefield_find(mine_a).unwrap().tapped, "slot 0 moved off my creature");
    assert!(!g.battlefield_find(mine_b).unwrap().tapped, "slot 1 moved off my creature too");
    let theirs_tapped =
        g.battlefield.iter().filter(|c| c.controller == 1 && c.tapped).count();
    // CR 601.2c — the two slots can't name the same object.
    assert_eq!(theirs_tapped, 2, "both slots landed on distinct opposing creatures");
}

/// CR 706.10 — a copied activated ability may choose new targets; the copy
/// takes the scripted pick rather than inheriting the original's.
#[test]
fn cr_706_10_copied_activated_ability_takes_new_targets() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let tim = g.add_card_to_battlefield(0, catalog::prodigal_sorcerer());
    g.clear_sickness(tim);
    let bracers = g.add_card_to_battlefield(0, catalog::illusionists_bracers());
    g.battlefield_find_mut(bracers).unwrap().attached_to = Some(tim);
    let first = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let second = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // The copy's target prompt is the only `ChooseTarget` in flight.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(
        second,
    ))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: tim,
        ability_index: 0,
        target: Some(Target::Permanent(first)),
        additional_targets: Vec::new(),
        x_value: None,
        mode: None,
    })
    .expect("ping");
    drain_stack(&mut g);

    assert_eq!(g.battlefield_find(first).unwrap().damage, 1, "the original ping stayed put");
    assert_eq!(g.battlefield_find(second).unwrap().damage, 1, "the copy was re-aimed");
}

/// CR 104.4b — a cost-free activated ability that changes nothing is a
/// mandatory loop the watchdog has to catch; a real cost line is not.
#[test]
fn cr_104_4b_free_activation_is_watched() {
    use crabomination::effect::ActivatedAbility;
    let free = ActivatedAbility { effect: crabomination::effect::Effect::Noop, ..Default::default() };
    assert!(free.is_free(), "{{0}}: with no cost line loops without bound");
    assert!(!ActivatedAbility { tap_cost: true, ..free.clone() }.is_free(), "{{T}} bounds it");
    assert!(!ActivatedAbility { once_per_turn: true, ..free }.is_free(), "a per-turn cap bounds it");
}


/// CR 732.3 — a fragmented loop of free activations is broken by rejecting
/// the repeat, not by drawing the game (that's 732.4's mandatory case).
#[test]
fn cr_732_3_repeated_free_activation_is_rejected() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::effect::{ActivatedAbility, Effect};
    use crabomination::game::types::GameError;
    // "{0}: This creature gains flying" — no cost, no state change.
    let looper = CardDefinition {
        name: "Looper",
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::GrantKeyword {
                what: crabomination::effect::Selector::This,
                keyword: crabomination::card::Keyword::Flying,
                duration: crabomination::effect::Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_battlefield(0, looper);
    g.clear_sickness(id);
    let activate = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
            mode: None,
        })
    };
    for _ in 0..50 {
        activate(&mut g).expect("under the cap");
        drain_stack(&mut g);
    }
    assert!(
        matches!(activate(&mut g), Err(GameError::LoopMustBeBroken)),
        "the repeat past the cap is refused"
    );
    assert!(g.game_over.is_none(), "a fragmented loop is not a draw");
}
