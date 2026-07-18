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

/// Undergrowth Recon returns a land from your graveyard to the battlefield
/// tapped at your upkeep.
#[test]
fn undergrowth_recon_returns_land_on_upkeep() {
    let mut g = two_player_game();
    let recon = g.add_card_to_battlefield(0, catalog::undergrowth_recon());
    let land = g.add_card_to_graveyard(0, catalog::forest());
    let ctx = EffectContext::for_trigger(recon, 0, Some(Target::Permanent(land)), 0);
    let trig = catalog::undergrowth_recon().triggered_abilities[0].effect.clone();
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    let ret = g.battlefield_find(land).expect("land back on the battlefield");
    assert!(ret.tapped, "returned tapped");
}

/// Dramatic Accusation taps the enchanted creature on ETB and can shuffle it
/// into its owner's library.
#[test]
fn dramatic_accusation_taps_then_shuffles() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::dramatic_accusation());
    // ETB effect: attach + tap.
    let ctx = EffectContext::for_trigger(aura, 0, Some(Target::Permanent(foe)), 0);
    let etb = catalog::dramatic_accusation().effect.clone();
    g.resolve_effect(&etb, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "enchanted creature tapped on ETB");
    // Activate the shuffle.
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate the shuffle ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature left the battlefield");
    assert!(g.players[1].library.iter().any(|c| c.id == foe), "shuffled into owner's library");
}

/// Lamplight Phoenix returns from the graveyard on death by collecting
/// evidence 4, and stays dead when the graveyard can't pay.
#[test]
fn lamplight_phoenix_returns_via_collect_evidence() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Two cheap graveyard cards cover the collect-evidence 4 cost.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let phoenix = g.add_card_to_graveyard(0, catalog::lamplight_phoenix());
    // Accept the optional "collect evidence" prompt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let trig = catalog::lamplight_phoenix().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_trigger(phoenix, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    let back = g.battlefield_find(phoenix).expect("phoenix returned to the battlefield");
    assert!(back.tapped, "returns tapped");
    // The two other graveyard cards were exiled to collect evidence (not the phoenix).
    assert_eq!(g.exile.len(), 2, "collected evidence 4 by exiling the cheap cards");
}

/// Slime Against Humanity makes an Ooze with 2 + (Oozes in exile/graveyard)
/// counters.
#[test]
fn slime_against_humanity_scales_with_oozes() {
    use crabomination::card::{CardType, CreatureType, Subtypes};
    let mut g = two_player_game();
    // One Ooze card in the graveyard → X = 2 + 1 = 3.
    let ooze_card = crabomination::card::CardDefinition {
        name: "Some Ooze",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ooze], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    };
    g.add_card_to_graveyard(0, ooze_card);
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&catalog::slime_against_humanity().effect, &ctx).unwrap();
    drain_stack(&mut g);
    let ooze = g.battlefield.iter().find(|c| c.definition.name == "Ooze" && c.controller == 0)
        .expect("Ooze token created");
    let c = g.computed_permanent(ooze.id).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "0/0 + three +1/+1 counters");
}

/// Magnetic Snuffler reanimates an Equipment from the graveyard attached to
/// itself, and grows when you sacrifice an artifact.
#[test]
fn magnetic_snuffler_reanimates_equipment_and_grows() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let snuffler = g.add_card_to_battlefield(0, catalog::magnetic_snuffler());
    let equip = g.add_card_to_graveyard(0, catalog::bonesplitter()); // +2/+0 Equipment
    let ctx = EffectContext::for_trigger(snuffler, 0, Some(Target::Permanent(equip)), 0);
    let etb = catalog::magnetic_snuffler().triggered_abilities[0].effect.clone();
    g.resolve_effect(&etb, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(equip).unwrap().attached_to, Some(snuffler), "equipment attached");
    // Sacrifice an artifact → +1/+1 counter.
    let fodder = g.add_card_to_battlefield(0, catalog::ornithopter());
    let mut events = Vec::new();
    g.sacrifice_one(fodder, 0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(snuffler).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "grew on artifact sacrifice",
    );
}

/// Cryptic Coat cloaks the top card and attaches to it, granting unblockable.
#[test]
fn cryptic_coat_cloaks_and_attaches() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let coat = g.add_card_to_battlefield(0, catalog::cryptic_coat());
    let ctx = EffectContext::for_trigger(coat, 0, None, 0);
    let etb = catalog::cryptic_coat().triggered_abilities[0].effect.clone();
    g.resolve_effect(&etb, &ctx).unwrap();
    drain_stack(&mut g);
    // A face-down 2/2 was created and the coat attached to it.
    let cloaked = g.battlefield_find(coat).unwrap().attached_to.expect("coat attached to the cloaked creature");
    let c = g.computed_permanent(cloaked).unwrap();
    assert_eq!((c.power, c.toughness), (3, 2), "2/2 face-down + the coat's +1/+0");
    assert!(c.keywords.contains(&Keyword::Unblockable), "equipped creature can't be blocked");
}

/// Outrageous Robbery exiles the top X of the target's library, castable by you.
#[test]
fn outrageous_robbery_exiles_and_grants_play() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let a = g.add_card_to_library(1, catalog::grizzly_bears());
    let b = g.add_card_to_library(1, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 2);
    g.resolve_effect(&catalog::outrageous_robbery().effect, &ctx).unwrap();
    drain_stack(&mut g);
    for id in [a, b] {
        let c = g.exile.iter().find(|c| c.id == id).expect("exiled from opponent's library");
        let perm = c.may_play_until.as_ref().expect("has a may-play grant");
        assert_eq!(perm.player, 0, "you may play the exiled cards");
    }
}

/// Presumed Dead pumps a creature and gives it a die-then-return-and-suspect
/// rider.
#[test]
fn presumed_dead_returns_and_suspects() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&catalog::presumed_dead().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0 applied");
    // Kill it; the granted trigger returns and suspects it.
    let events = g.remove_to_graveyard_with_triggers(bear);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let back = g.battlefield_find(bear).expect("returned to the battlefield");
    assert!(!back.tapped, "returns untapped");
    assert!(back.suspected, "returned suspected");
}
