//! Commander: the Sworn to Darkness precon (C14, Ob Nixilis of the Black
//! Oath, `decks::cmdr_ob_nixilis`) and the primitives it needed.

use crabomination::card::{CardId, Keyword, SelectionRequirement as R, StaticAbility, Value};
use crabomination::catalog;
use crabomination::effect::{ActivatedAbility, Effect, PlayerRef, Selector, StaticEffect};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::{Color, b, cost, generic};

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn activate(g: &mut GameState, id: CardId, index: usize) -> Result<(), String> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) -> Result<(), String> {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

/// CR 508.1d — "attacks that player this combat if able": the creature must
/// be declared, and only against the opponent stamped on it.
#[test]
fn cr_508_1d_must_attack_chosen_player_binds_attacker_and_defender() {
    let mut g = pod(3);
    let mut def = catalog::grizzly_bears();
    def.keywords.push(Keyword::MustAttackChosenPlayer);
    let bear = g.add_card_to_battlefield(0, def);
    g.clear_sickness(bear);
    g.battlefield_find_mut(bear).unwrap().chosen_player = Some(2);

    let snapshot = g.clone();
    assert!(declare(&mut g, vec![]).is_err(), "it must attack");
    let mut g = snapshot.clone();
    assert!(declare(&mut g, vec![(bear, 1)]).is_err(), "and only the chosen opponent");
    let mut g = snapshot.clone();
    declare(&mut g, vec![(bear, 2)]).expect("attacking the chosen opponent is legal");
}

/// CR 508.1d — with its chosen seat gone (CR 800.4a), nothing binds it.
#[test]
fn cr_508_1d_must_attack_chosen_player_lapses_when_that_player_left() {
    let mut g = pod(3);
    let mut def = catalog::grizzly_bears();
    def.keywords.push(Keyword::MustAttackChosenPlayer);
    let bear = g.add_card_to_battlefield(0, def);
    g.clear_sickness(bear);
    g.battlefield_find_mut(bear).unwrap().chosen_player = Some(2);
    g.players[2].eliminated = true;
    declare(&mut g, vec![]).expect("no live chosen opponent — no requirement");
}

/// CR 114.4 — an emblem's "creatures you control have '…'" works from the
/// command zone: each of its owner's creatures gains the activated ability,
/// and nobody else's does.
#[test]
fn cr_114_4_emblem_grants_an_activated_ability_to_its_owners_creatures() {
    let mut g = pod(3);
    let grant = StaticAbility {
        description: "Creatures you control have \"{1}{B}, Sacrifice this creature: gain X, draw X.\"",
        effect: StaticEffect::GrantActivatedAbility {
            applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            ability: ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::SacrificedPower },
                    Effect::Draw { who: Selector::You, amount: Value::SacrificedPower },
                ]),
                ..Default::default()
            },
            condition: None,
        },
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::CreateEmblem { who: PlayerRef::You, name: "Test".into(), triggered: vec![], statics: vec![grant] },
        &ctx,
    )
    .expect("emblem");
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    flood(&mut g, 0);
    activate(&mut g, mine, 0).expect("the emblem's ability is on my creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "sacrificed as the cost");
    assert_eq!(g.players[0].life, life + 2, "X is the sacrificed creature's power");
    assert_eq!(g.players[0].hand.len(), hand + 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    flood(&mut g, 1);
    assert!(activate(&mut g, theirs, 0).is_err(), "not on an opponent's creature");
}
