//! Functionality tests for `catalog::sets::decks::recent229` — the optional
//! single-target primitive (`Effect::OptionalTargets`) and its cards.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction, TurnStep};

/// Primal Might pumps your creature by X and fights the enemy target: a 2/2
/// pumped +3/+3 (5/5) kills an opposing 4/4 and survives.
#[test]
fn primal_might_pumps_then_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let theirs = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let ctx = EffectContext {
        targets: vec![Target::Permanent(mine), Target::Permanent(theirs)],
        ..EffectContext::for_spell(0, None, 0, 3)
    };
    g.resolve_effect(&catalog::primal_might().effect.clone(), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "5/5 kills the 3/3");
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!(cp.power, 5, "2 + 3 = 5");
    assert!(g.battlefield.iter().any(|c| c.id == mine), "survives 3 damage on 5 toughness");
}

/// The fight target of Primal Might is slot 1 (optional); slot 0 is required.
/// Resolving with only slot 0 still pumps and no-ops the fight.
#[test]
fn primal_might_fight_slot_is_optional() {
    let def = catalog::primal_might();
    assert!(!def.effect.target_slot_optional(0, None), "the pumped creature is required");
    assert!(def.effect.target_slot_optional(1, None), "the fight target is 'up to one'");

    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext {
        targets: vec![Target::Permanent(mine)],
        ..EffectContext::for_spell(0, None, 0, 2)
    };
    g.resolve_effect(&def.effect.clone(), &ctx).unwrap();
    assert_eq!(g.computed_permanent(mine).unwrap().power, 4, "still pumped without a fight target");
}

/// Boom Box destroys up to one artifact, creature, and land — all three when
/// supplied. Every slot is optional (min 0).
#[test]
fn boom_box_destroys_each_type() {
    let def = catalog::boom_box();
    let ability = def.activated_abilities[0].effect.clone();
    assert!(ability.target_slot_optional(0, None), "up to one — every slot declinable");

    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::mind_stone());
    let cre = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let ctx = EffectContext {
        targets: vec![Target::Permanent(art), Target::Permanent(cre), Target::Permanent(land)],
        ..EffectContext::for_ability(art, 0, None)
    };
    g.resolve_effect(&ability, &ctx).unwrap();
    drain_stack(&mut g);
    for id in [art, cre, land] {
        assert!(!g.battlefield.iter().any(|c| c.id == id), "target destroyed");
    }
}

/// Prizefight fights the two creatures and leaves a Treasure behind.
#[test]
fn prizefight_fights_and_makes_treasure() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let ctx = EffectContext {
        targets: vec![Target::Permanent(mine), Target::Permanent(theirs)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&catalog::prizefight().effect.clone(), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "2/2 dies to the 3/3");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count(),
        1,
        "a Treasure is minted"
    );
}

/// Out Cold can't be countered, taps + stuns up to two creatures, and
/// investigates.
#[test]
fn out_cold_taps_stuns_and_investigates() {
    let mut g = two_player_game();
    assert!(catalog::out_cold().keywords.contains(&Keyword::CantBeCountered));
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    let ctx = EffectContext {
        targets: vec![Target::Permanent(a), Target::Permanent(b)],
        ..EffectContext::for_spell(0, None, 0, 0)
    };
    g.resolve_effect(&catalog::out_cold().effect.clone(), &ctx).unwrap();
    for id in [a, b] {
        assert!(g.battlefield_find(id).unwrap().tapped, "creature tapped");
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::Stun), 1, "stun counter");
    }
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Clue").count(),
        1,
        "a Clue is investigated"
    );
}

/// Harvester of Misery's ETB shrinks every other creature but not itself.
#[test]
fn harvester_etb_shrinks_others() {
    let mut g = two_player_game();
    let harv = g.add_card_to_battlefield(0, catalog::harvester_of_misery());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let enemy = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let effect = catalog::harvester_of_misery().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(harv, 0, None, 0)).unwrap();
    g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == other), "the 2/2 dies to -2/-2");
    assert_eq!(g.computed_permanent(enemy).unwrap().toughness, 1, "3/3 → 1/1");
    assert_eq!(g.computed_permanent(harv).unwrap().power, 5, "Harvester unaffected");
}

/// Harvester of Misery's from-hand ability: {1}{B}, Discard this card: target
/// creature gets -2/-2. Activating discards Harvester and shrinks the target.
#[test]
fn harvester_hand_ability_shrinks_target() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let harv = g.add_card_to_hand(0, catalog::harvester_of_misery());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: harv,
        ability_index: 0,
        target: Some(Target::Permanent(enemy)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("from-hand discard ability");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(enemy).unwrap().toughness, 1, "3/3 → 1/1");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == harv), "Harvester discarded as a cost");
}

/// Krovod Haunch buffs its wearer +2/+0 and, when it dies, its owner may pay
/// {1}{W} to make two Dogs.
#[test]
fn krovod_haunch_buffs_and_makes_dogs() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let haunch = g.add_card_to_battlefield(0, catalog::krovod_haunch());
    g.battlefield_find_mut(haunch).unwrap().attached_to = Some(bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "2 + 2 = 4");

    // Resolve the death-trigger body with mana available and a yes-decider.
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let effect = catalog::krovod_haunch().triggered_abilities[0].effect.clone();
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.resolve_effect(&effect, &EffectContext::for_trigger(haunch, 0, None, 0)).unwrap();
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Dog").count(),
        2,
        "two Dogs created"
    );
}
