//! Functionality tests for `catalog::sets::decks::recent252`.

use crabomination::card::{AdditionalCastCost, CounterType, Keyword, SelectionRequirement as R};
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game};

/// Treacherous Greed carries the "sacrifice a creature that dealt damage"
/// additional cost and draws three / drains three on resolution.
#[test]
fn treacherous_greed_draws_and_drains() {
    // Additional cost is a filtered sacrifice.
    assert!(matches!(
        &catalog::treacherous_greed().additional_cast_cost[0],
        AdditionalCastCost::SacrificePermanent { count: 1, filter }
            if *filter == R::Creature.and(R::DealtDamageThisTurn)
    ));
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand_before = g.players[0].hand.len();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::treacherous_greed().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 3, "drew three");
    assert_eq!(g.players[0].life, 23, "you gained three");
    assert_eq!(g.players[1].life, 17, "opponent lost three");
}

/// Flourishing Bloom-Kin gets +1/+1 for each Forest you control.
#[test]
fn flourishing_bloom_kin_scales_with_forests() {
    let mut g = two_player_game();
    let kin = g.add_card_to_battlefield(0, catalog::flourishing_bloom_kin());
    // 0/0 with no Forests.
    let c = g.computed_permanent(kin).unwrap();
    assert_eq!((c.power, c.toughness), (0, 0));
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let c = g.computed_permanent(kin).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "+1/+1 per Forest");
}

/// Concealed Weapon's turn-face-up trigger attaches it to a creature you control.
#[test]
fn concealed_weapon_attaches_on_turn_up() {
    let mut g = two_player_game();
    let weapon = g.add_card_to_battlefield(0, catalog::concealed_weapon());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Resolve the real turn-face-up trigger effect (attach to the bear).
    let ctx = EffectContext::for_trigger(weapon, 0, Some(Target::Permanent(bear)), 0);
    let trig = catalog::concealed_weapon().triggered_abilities[0].effect.clone();
    g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!(g.battlefield_find(weapon).unwrap().attached_to, Some(bear), "attached");
    // +3/+0 from the Equipment now applies.
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 5, "equipped creature gets +3/+0");
}

/// Lumbering Laundry is a 4/5 Golem with Disguise.
#[test]
fn lumbering_laundry_has_disguise() {
    let def = catalog::lumbering_laundry();
    assert_eq!((def.power, def.toughness), (4, 5));
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Disguise(_))));
}

/// Audience with Trostani makes a Plant and draws per differently-named
/// creature token you control.
#[test]
fn audience_with_trostani_draws_per_distinct_token() {
    use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
    let mut g = two_player_game();
    // Pre-existing tokens: two "Clue"? no — two distinct creature-token names.
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    };
    g.add_token_to_battlefield(0, &spirit);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand_before = g.players[0].hand.len();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::audience_with_trostani().effect, &ctx).unwrap();
    drain_stack(&mut g);
    // Distinct token names now: Spirit + Plant = 2 → drew 2.
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew per distinct token name");
}

/// Krenko's sacrifice ability puts a +1/+1 counter on each Goblin you control.
#[test]
fn krenko_baron_counters_each_goblin() {
    use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let krenko = g.add_card_to_battlefield(0, catalog::krenko_baron_of_tin_street());
    g.clear_sickness(krenko);
    let goblin_tok = TokenDefinition {
        name: "Goblin".into(),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    };
    let other = g.add_token_to_battlefield(0, &goblin_tok);
    let fodder = g.add_card_to_battlefield(0, catalog::ornithopter());
    g.perform_action(GameAction::ActivateAbility {
        card_id: krenko,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate sac-artifact ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "artifact sacrificed");
    for gob in [krenko, other] {
        assert_eq!(
            g.battlefield_find(gob).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
            1,
            "each Goblin got a +1/+1 counter",
        );
    }
}

/// Cryptex's collect-evidence ability adds mana and an unlock counter; the
/// sacrifice ability is gated on five unlock counters and draws three.
#[test]
fn cryptex_collect_evidence_and_unlock_sacrifice() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let cryptex = g.add_card_to_battlefield(0, catalog::cryptex());
    // Graveyard cards worth ≥ 3 to collect evidence.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: cryptex,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate collect-evidence mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "produced one mana");
    assert_eq!(
        g.battlefield_find(cryptex).unwrap().counters.get(&CounterType::Unlock).copied().unwrap_or(0),
        1,
        "gained an unlock counter",
    );
    // With five unlock counters the sacrifice ability draws three.
    g.battlefield_find_mut(cryptex).unwrap().counters.insert(CounterType::Unlock, 5);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: cryptex,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate the sacrifice draw ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cryptex).is_none(), "Cryptex sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 3, "drew three");
}

/// Cryptex's sacrifice ability is rejected without five unlock counters.
#[test]
fn cryptex_sacrifice_gated_on_five_counters() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let cryptex = g.add_card_to_battlefield(0, catalog::cryptex());
    g.battlefield_find_mut(cryptex).unwrap().counters.insert(CounterType::Unlock, 4);
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: cryptex,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
    });
    assert!(err.is_err(), "rejected with only four unlock counters");
    assert!(g.battlefield_find(cryptex).is_some(), "not sacrificed");
}

/// Detective's Satchel investigates twice on ETB, and its Thopter ability is
/// gated on having sacrificed an artifact this turn.
#[test]
fn detectives_satchel_investigate_and_gated_thopter() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let satchel = g.add_card_to_battlefield(0, catalog::detectives_satchel());
    g.fire_self_etb_triggers(satchel, 0);
    drain_stack(&mut g);
    let clues = g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count();
    assert_eq!(clues, 2, "investigated twice");
    // Without a sacrificed artifact this turn the Thopter ability is rejected.
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: satchel,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    });
    assert!(err.is_err(), "gated before an artifact sacrifice");
    // After sacrificing an artifact it makes a Thopter.
    g.players[0].artifacts_sacrificed_this_turn = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: satchel,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("Thopter ability now activatable");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Thopter" && c.controller == 0),
        "created a Thopter",
    );
}

/// Polygraph Orb digs on ETB (two to hand, two to graveyard, lose 2), and its
/// activated ability drains an opponent who can't discard or sacrifice.
#[test]
fn polygraph_orb_etb_dig_and_punisher() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let hand_before = g.players[0].hand.len();
    let orb = g.add_card_to_battlefield(0, catalog::polygraph_orb());
    g.fire_self_etb_triggers(orb, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "two cards to hand");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Forest").count(), 2, "two milled");
    assert_eq!(g.players[0].life, 18, "lost two life");
    // Opponent with empty hand and no creatures must lose 3 life.
    g.players[1].hand.clear();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: orb,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate the collect-evidence punisher");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "opponent lost three (no dodge available)");
}
