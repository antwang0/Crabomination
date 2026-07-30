//! Tests for recentN card batches 41-50 (merged from per-batch micro-files).

mod recent41 {
    use crabomination::catalog;
    use crabomination::game::effects::{EffectContext, EntityRef};
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    #[test]
    fn mark_of_asylum_prevents_noncombat_damage_to_your_creatures() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::mark_of_asylum());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let mut events = Vec::new();
        g.deal_damage_to_from(EntityRef::Permanent(mine), 3, None, &mut events);
        g.deal_damage_to_from(EntityRef::Permanent(theirs), 3, None, &mut events);
        assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "your creature's noncombat damage is prevented");
        assert_eq!(g.battlefield_find(theirs).unwrap().damage, 3, "the opponent's creature is unaffected");
    }

    #[test]
    fn dryad_militant_exiles_instants_and_sorceries_only() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::dryad_militant());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // instant
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // creature
        let bolt_card = g.players[0].hand.iter().find(|c| c.id == bolt).unwrap().clone();
        let bear_card = g.players[0].hand.iter().find(|c| c.id == bear).unwrap().clone();
        assert!(g.graveyard_exiled_for(&bolt_card), "an instant bound for a graveyard is exiled");
        assert!(!g.graveyard_exiled_for(&bear_card), "a creature card is not");
    }

    #[test]
    fn plated_geopede_grows_on_landfall() {
        let mut g = two_player_game();
        let pede = g.add_card_to_battlefield(0, catalog::plated_geopede());
        let land = g.add_card_to_battlefield(0, catalog::forest());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: land }]);
        drain_stack(&mut g);
        let cp = g.computed_permanent(pede).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "1/1 + landfall +2/+2 = 3/3");
    }

    #[test]
    fn scale_up_makes_a_six_four_wurm() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(&catalog::scale_up().effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (6, 4), "becomes a 6/4");
        assert!(cp.subtypes.creature_types.contains(&crabomination::card::CreatureType::Wurm), "and a Wurm");
    }

    #[test]
    fn spawning_pool_animates_to_a_skeleton() {
        let mut g = two_player_game();
        let pool = g.add_card_to_battlefield(0, catalog::spawning_pool());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: pool, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("animate");
        drain_stack(&mut g);
        let cp = g.computed_permanent(pool).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "1/1 Skeleton");
        assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
    }
}

mod recent42 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn activate(g: &mut GameState, id: CardId, idx: usize) {
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: idx, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("ability activates");
        drain_stack(g);
    }

    #[test]
    fn ratchet_bomb_blows_up_matching_mana_value() {
        let mut g = two_player_game();
        let bomb = g.add_card_to_battlefield(0, catalog::ratchet_bomb());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // {1}{G} = MV 2
        let one_drop = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // MV 1
        let land = g.add_card_to_battlefield(1, catalog::forest()); // nonland-only filter spares it
        // Tick to two charge counters, then detonate.
        g.battlefield_find_mut(bomb).unwrap().add_counters(CounterType::Charge, 2);
        activate(&mut g, bomb, 1); // {T}, Sacrifice: destroy each nonland with MV == 2
        assert!(g.battlefield_find(bear).is_none(), "MV-2 creature destroyed");
        assert!(g.battlefield_find(one_drop).is_some(), "MV-1 creature spared");
        assert!(g.battlefield_find(land).is_some(), "lands are never hit");
        assert!(g.battlefield_find(bomb).is_none(), "bomb sacrificed itself");
    }

    /// Two distinct colors of mana → two charge counters (CR 702.44).
    #[test]
    fn engineered_explosives_enters_with_sunburst_counters() {
        let mut g = two_player_game();
        g.step = crabomination::game::types::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let bomb = g.add_card_to_hand(0, catalog::engineered_explosives());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.perform_action(crabomination::game::types::GameAction::CastSpell {
            card_id: bomb, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
        })
        .expect("cast for X=2 with two colors");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(bomb).unwrap().counter_count(CounterType::Charge),
            2
        );
    }

    #[test]
    fn sphere_of_the_suns_taps_for_any_color_off_a_charge_counter() {
        let mut g = two_player_game();
        let sphere = g.add_card_to_battlefield(0, catalog::sphere_of_the_suns());
        g.battlefield_find_mut(sphere).unwrap().add_counters(CounterType::Charge, 3);
        activate(&mut g, sphere, 0);
        assert_eq!(g.players[0].mana_pool.total(), 1, "added one mana");
        assert_eq!(
            g.battlefield_find(sphere).unwrap().counter_count(CounterType::Charge),
            2,
            "spent a charge counter"
        );
    }

    #[test]
    fn gaddock_teeg_locks_expensive_noncreature_spells() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::gaddock_teeg());
        assert!(g.noncreature_spell_cast_locked(&catalog::wrath_of_god()), "MV-4 sorcery is locked");
        assert!(g.noncreature_spell_cast_locked(&catalog::engineered_explosives()), "an X-cost artifact is locked");
        assert!(!g.noncreature_spell_cast_locked(&catalog::grizzly_bears()), "creatures are exempt");
        assert!(!g.noncreature_spell_cast_locked(&catalog::lightning_bolt()), "cheap noncreature spells are fine");
    }

    #[test]
    fn tabernacle_grants_every_creature_an_upkeep_tax() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::the_tabernacle_at_pendrell_vale());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let granted = g.statics_granted_triggers_for(g.battlefield_find(bear).unwrap());
        assert!(!granted.is_empty(), "the bear inherits the Tabernacle's upkeep trigger");
    }

    #[test]
    fn tabernacle_destroys_an_unpaid_creature() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::the_tabernacle_at_pendrell_vale());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Resolve the granted "pay {1} or destroy" with no mana available.
        let granted = g.statics_granted_triggers_for(g.battlefield_find(bear).unwrap());
        let effect = granted[0].effect.clone();
        let ctx = EffectContext::for_ability(bear, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "unpaid creature is destroyed");
    }
}

mod recent43 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: idx, target, additional_targets: Vec::new(), x_value: None,
        }).expect("ability activates");
        drain_stack(g);
    }

    #[test]
    fn blast_zone_charges_then_detonates() {
        let mut g = two_player_game();
        let bz = g.add_card_to_battlefield(0, catalog::blast_zone());
        // Pump to two charge counters via the {X}{X} ability (X=2 → costs 4).
        g.battlefield_find_mut(bz).unwrap().add_counters(CounterType::Charge, 1); // simulate ETB charge
        g.players[0].mana_pool.add_colorless(4);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: bz, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: Some(1),
        }).expect("charge");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(bz).unwrap().counter_count(CounterType::Charge), 2,
            "1 (ETB) + 1 (X=1) charge counters"
        );
        // Untap (the charge ability tapped it) and detonate (MV 2): a {1}{G} bear
        // dies, a 1-drop lives.
        g.battlefield_find_mut(bz).unwrap().tapped = false;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
        g.players[0].mana_pool.add_colorless(3);
        activate(&mut g, bz, 2, None);
        assert!(g.battlefield_find(bear).is_none(), "MV-2 permanent destroyed");
        assert!(g.battlefield_find(elf).is_some(), "MV-1 permanent spared");
    }

    #[test]
    fn encroaching_wastes_destroys_a_nonbasic_land() {
        let mut g = two_player_game();
        let edge = g.add_card_to_battlefield(0, catalog::encroaching_wastes());
        let target = g.add_card_to_battlefield(1, catalog::wasteland()); // nonbasic
        g.players[0].mana_pool.add_colorless(4);
        activate(&mut g, edge, 1, Some(Target::Permanent(target)));
        assert!(g.battlefield_find(target).is_none(), "nonbasic land destroyed");
        assert!(g.battlefield_find(edge).is_none(), "Encroaching Wastes sacrificed itself");
    }

    #[test]
    fn tectonic_edge_needs_an_opponent_with_four_lands() {
        let mut g = two_player_game();
        let edge = g.add_card_to_battlefield(0, catalog::tectonic_edge());
        let victim = g.add_card_to_battlefield(1, catalog::wasteland());
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        // Opponent controls only one land → activation illegal.
        let res = g.perform_action(GameAction::ActivateAbility {
            card_id: edge, ability_index: 1, target: Some(Target::Permanent(victim)),
            additional_targets: Vec::new(), x_value: None,
        });
        assert!(res.is_err(), "can't blow up a land until the opponent has four");
        // Give them three more lands (four total) → now legal.
        for _ in 0..3 { g.add_card_to_battlefield(1, catalog::island()); }
        g.players[0].mana_pool.add_colorless(1);
        activate(&mut g, edge, 1, Some(Target::Permanent(victim)));
        assert!(g.battlefield_find(victim).is_none(), "now destroyed");
    }

    #[test]
    fn buried_ruin_returns_an_artifact_from_the_graveyard() {
        let mut g = two_player_game();
        let ruin = g.add_card_to_battlefield(0, catalog::buried_ruin());
        let art = g.add_card_to_graveyard(0, catalog::ratchet_bomb());
        g.players[0].mana_pool.add_colorless(2);
        activate(&mut g, ruin, 1, Some(Target::Permanent(art)));
        assert!(g.players[0].hand.iter().any(|c| c.id == art), "artifact returned to hand");
    }

    #[test]
    fn serras_sanctum_scales_with_enchantments() {
        let mut g = two_player_game();
        let sanctum = g.add_card_to_battlefield(0, catalog::serras_sanctum());
        g.add_card_to_battlefield(0, catalog::mark_of_asylum()); // enchantment
        g.add_card_to_battlefield(0, catalog::glacial_chasm()); // a land, not counted
        activate(&mut g, sanctum, 0, None);
        assert_eq!(g.players[0].mana_pool.total(), 1, "one W per enchantment (one enchantment)");
    }

    #[test]
    fn tolarian_academy_scales_with_artifacts() {
        let mut g = two_player_game();
        let academy = g.add_card_to_battlefield(0, catalog::tolarian_academy());
        g.add_card_to_battlefield(0, catalog::ratchet_bomb());
        g.add_card_to_battlefield(0, catalog::sphere_of_the_suns());
        activate(&mut g, academy, 0, None);
        assert_eq!(g.players[0].mana_pool.total(), 2, "one U per artifact (two artifacts)");
    }
}

mod recent44 {
    use crabomination::card::StaticEffect;
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: idx, target, additional_targets: Vec::new(), x_value: None,
        }).expect("ability activates");
        drain_stack(g);
    }

    #[test]
    fn uktabi_orangutan_smashes_an_artifact_on_etb() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::ratchet_bomb());
        let ape = g.add_card_to_battlefield(0, catalog::uktabi_orangutan());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Target(Target::Permanent(art)),
        ]));
        g.fire_self_etb_triggers(ape, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_none(), "the artifact is destroyed");
    }

    #[test]
    fn viridian_zealot_sacs_to_destroy_artifact_or_enchantment() {
        let mut g = two_player_game();
        let zealot = g.add_card_to_battlefield(0, catalog::viridian_zealot());
        let ench = g.add_card_to_battlefield(1, catalog::mark_of_asylum());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        activate(&mut g, zealot, 0, Some(Target::Permanent(ench)));
        assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
        assert!(g.battlefield_find(zealot).is_none(), "Zealot sacrificed itself");
    }

    #[test]
    fn glowrider_taxes_noncreature_spells() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::glowrider());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        let bolt = g.players[0].hand.iter().find(|c| c.id == bolt).unwrap().clone();
        let bear = g.players[0].hand.iter().find(|c| c.id == bear).unwrap().clone();
        assert_eq!(crabomination::game::actions::extra_cost_for_spell(&g, 0, &bolt, None), 1, "noncreature taxed");
        assert_eq!(crabomination::game::actions::extra_cost_for_spell(&g, 0, &bear, None), 0, "creature untaxed");
    }

    #[test]
    fn ingot_chewer_has_an_evoke_cost() {
        let alt = catalog::ingot_chewer().alternative_cost.unwrap();
        assert!(alt.evoke_sacrifice, "evoke sacrifices the body on ETB");
    }

    #[test]
    fn energy_flux_grants_artifacts_an_upkeep_tax() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::energy_flux());
        let art = g.add_card_to_battlefield(0, catalog::ratchet_bomb());
        let granted = g.statics_granted_triggers_for(g.battlefield_find(art).unwrap());
        assert!(!granted.is_empty(), "the artifact inherits Energy Flux's upkeep tax");
    }

    #[test]
    fn hushwing_gryff_suppresses_creature_etb_triggers() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::hushwing_gryff());
        let art = g.add_card_to_battlefield(1, catalog::ratchet_bomb());
        // Another Uktabi Orangutan enters: its ETB destroy is suppressed.
        let ape = g.add_card_to_battlefield(0, catalog::uktabi_orangutan());
        g.fire_self_etb_triggers(ape, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_some(), "the ETB trigger never fired");
        // Sanity: the suppressor's static is present.
        assert!(catalog::hushwing_gryff().static_abilities.iter().any(|s|
            matches!(s.effect, StaticEffect::SuppressCreatureEtbTriggers { .. })));
    }

    #[test]
    fn harsh_mentor_punishes_opponent_ability_activations() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::harsh_mentor());
        // An opponent activates a non-mana ability (Ratchet Bomb's charge).
        let bomb = g.add_card_to_battlefield(1, catalog::ratchet_bomb());
        let life = g.players[1].life;
        g.priority.player_with_priority = 1;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: bomb, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("charge ability");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2, "Harsh Mentor deals 2 to the activating opponent");
    }
}

mod recent45 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    /// Resolve a single-target spell effect against `target`.
    fn resolve_on(g: &mut GameState, def: crabomination::card::CardDefinition, target: Target) {
        let ctx = EffectContext::for_spell(0, Some(target), 0, 0);
        g.resolve_effect(&def.effect, &ctx).unwrap();
        drain_stack(g);
    }

    #[test]
    fn fragmentize_only_hits_cheap_artifacts_and_enchantments() {
        let mut g = two_player_game();
        let cheap = g.add_card_to_battlefield(1, catalog::ratchet_bomb()); // MV 2 artifact
        resolve_on(&mut g, catalog::fragmentize(), Target::Permanent(cheap));
        assert!(g.battlefield_find(cheap).is_none(), "MV-2 artifact destroyed");
    }

    #[test]
    fn erase_exiles_an_enchantment() {
        let mut g = two_player_game();
        let ench = g.add_card_to_battlefield(1, catalog::mark_of_asylum());
        resolve_on(&mut g, catalog::erase(), Target::Permanent(ench));
        assert!(g.exile.iter().any(|c| c.id == ench), "enchantment exiled");
    }

    #[test]
    fn rebuke_destroys_an_attacker() {
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.attacking.push(crabomination::game::types::Attack {
            attacker: atk,
            target: crabomination::game::types::AttackTarget::Player(0),
        });
        resolve_on(&mut g, catalog::rebuke(), Target::Permanent(atk));
        assert!(g.battlefield_find(atk).is_none(), "attacking creature destroyed");
    }

    #[test]
    fn depopulate_spares_tokens() {
        let mut g = two_player_game();
        let real = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // A token survives the nontoken-only wipe.
        let bear_token = crabomination::card::TokenDefinition {
            name: "Bear".into(),
            power: 2,
            toughness: 2,
            card_types: vec![crabomination::card::CardType::Creature],
            ..Default::default()
        };
        let tok = g.add_token_to_battlefield(0, &bear_token);
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&catalog::depopulate().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(real).is_none(), "nontoken creature destroyed");
        assert!(g.battlefield_find(tok).is_some(), "token survives");
    }

    #[test]
    fn crib_swap_exiles_and_gifts_a_shapeshifter() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        resolve_on(&mut g, catalog::crib_swap(), Target::Permanent(victim));
        assert!(g.exile.iter().any(|c| c.id == victim), "target exiled");
        assert!(
            g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Shapeshifter"),
            "its controller gets a 1/1 Shapeshifter"
        );
    }
}

mod recent46 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    #[test]
    fn greenwarden_etb_returns_a_graveyard_card() {
        let mut g = two_player_game();
        let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let gw = g.add_card_to_battlefield(0, catalog::greenwarden_of_murasa());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.fire_self_etb_triggers(gw, 0);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == gy), "gy card returned to hand");
    }

    #[test]
    fn greenwarden_dies_exiles_self_and_returns_a_card() {
        let mut g = two_player_game();
        let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let gw = g.add_card_to_battlefield(0, catalog::greenwarden_of_murasa());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let mut evs = g.remove_to_graveyard_with_triggers(gw);
        evs.push(GameEvent::CreatureDied { card_id: gw });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == gy), "other gy card returned");
        assert!(g.exile.iter().any(|c| c.id == gw), "Greenwarden exiled itself");
    }

    #[test]
    fn nantuko_vigilante_face_up_destroys_artifact() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::ratchet_bomb());
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(art)), 0, 0);
        let trig = &catalog::nantuko_vigilante().triggered_abilities[0].effect;
        g.resolve_effect(trig, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_none(), "artifact destroyed on turn face up");
    }

    #[test]
    fn bramble_sovereign_copies_an_entering_creature() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::bramble_sovereign());
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        // {1}{G} for the bear + {1}{G} for the copy.
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bear");
        drain_stack(&mut g);
        let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
        assert_eq!(bears, 2, "original plus a token copy");
    }

    #[test]
    fn verdurous_gearhulk_distributes_four_counters() {
        let mut g = two_player_game();
        let vg = g.add_card_to_battlefield(0, catalog::verdurous_gearhulk());
        g.fire_self_etb_triggers(vg, 0);
        drain_stack(&mut g);
        let total: u32 = g
            .battlefield
            .iter()
            .filter(|c| c.controller == 0)
            .map(|c| c.counters.get(&crabomination::card::CounterType::PlusOnePlusOne).copied().unwrap_or(0))
            .sum();
        assert_eq!(total, 4, "four +1/+1 counters distributed");
    }

    #[test]
    fn pathbreaker_ibex_pumps_team_by_greatest_power() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::pathbreaker_ibex()); // 3/3
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let trig = &catalog::pathbreaker_ibex().triggered_abilities[0].effect;
        g.resolve_effect(trig, &ctx).unwrap();
        drain_stack(&mut g);
        // Greatest power is 3 (the Ibex), so the bear becomes 5/5 with trample.
        let b = g.computed_permanent(bear).unwrap();
        assert_eq!(b.power, 5, "bear pumped +3/+3");
        assert!(b.keywords.contains(&crabomination::card::Keyword::Trample), "bear gained trample");
    }

    #[test]
    fn ghalta_costs_less_per_total_power() {
        let mut g = two_player_game();
        // Two 5-power bodies → total power 10 → {10}{G}{G} becomes {G}{G}.
        g.add_card_to_battlefield(0, catalog::serra_angel()); // 4 power
        g.add_card_to_battlefield(0, catalog::serra_angel()); // 4 power
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power → total 10
        let ghalta = g.add_card_to_hand(0, catalog::ghalta_primal_hunger());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: ghalta, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("Ghalta castable for {G}{G} with 10 total power");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Ghalta, Primal Hunger"));
    }

    #[test]
    fn lifecrafters_bestiary_draws_on_creature_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::lifecrafters_bestiary());
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        // {1}{G} for the bear + {G} for the draw.
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let lib0 = g.players[0].library.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bear");
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib0 - 1, "drew a card off the creature cast");
    }
}

mod recent47 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: idx, target, additional_targets: Vec::new(), x_value: None,
        }).expect("ability activates");
        drain_stack(g);
    }

    #[test]
    fn multani_grows_with_lands_on_board_and_in_graveyard() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_graveyard(0, catalog::forest());
        let m = g.add_card_to_battlefield(0, catalog::multani_yavimayas_avatar());
        let cp = g.computed_permanent(m).unwrap();
        assert_eq!(cp.power, 3, "2 lands on board + 1 in graveyard");
        assert_eq!(cp.toughness, 3);
    }

    #[test]
    fn nullmage_shepherd_taps_four_to_destroy() {
        let mut g = two_player_game();
        let shep = g.add_card_to_battlefield(0, catalog::nullmage_shepherd());
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::grizzly_bears());
        }
        let ench = g.add_card_to_battlefield(1, catalog::mark_of_asylum());
        activate(&mut g, shep, 0, Some(Target::Permanent(ench)));
        assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
    }

    #[test]
    fn magus_of_the_wheel_refills_both_hands() {
        let mut g = two_player_game();
        let magus = g.add_card_to_battlefield(0, catalog::magus_of_the_wheel());
        g.clear_sickness(magus);
        for p in 0..2 {
            for _ in 0..3 { g.add_card_to_hand(p, catalog::forest()); }
            for _ in 0..10 { g.add_card_to_library(p, catalog::forest()); }
        }
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        activate(&mut g, magus, 0, None);
        assert_eq!(g.players[0].hand.len(), 7, "P0 drew a fresh seven");
        assert_eq!(g.players[1].hand.len(), 7, "P1 drew a fresh seven");
    }

    #[test]
    fn bankrupt_in_blood_sacs_two_for_three_cards() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        let bib = g.add_card_to_hand(0, catalog::bankrupt_in_blood());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        let hand0 = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bib, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Bankrupt in Blood");
        drain_stack(&mut g);
        // -Bankrupt (cast) +3 drawn.
        assert_eq!(g.players[0].hand.len(), hand0 - 1 + 3, "drew three");
        let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
        assert_eq!(bears, 0, "both creatures sacrificed as the additional cost");
    }

    #[test]
    fn sidisi_exploit_tutors() {
        let mut g = two_player_game();
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let wanted = g.add_card_to_library(0, catalog::lightning_bolt());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),            // yes, exploit the fodder
            DecisionAnswer::Search(Some(wanted)),  // tutor the bolt
        ]));
        let etb = catalog::sidisi_undead_vizier().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_trigger(crabomination::card::CardId(99), 0, None, 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "fodder exploited");
        assert!(g.players[0].hand.iter().any(|c| c.id == wanted), "tutored card in hand");
    }

    #[test]
    fn nighthawk_scavenger_scales_off_opponent_graveyard_types() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(1, catalog::lightning_bolt()); // instant
        g.add_card_to_graveyard(1, catalog::grizzly_bears()); // creature
        let nh = g.add_card_to_battlefield(0, catalog::nighthawk_scavenger());
        let cp = g.computed_permanent(nh).unwrap();
        assert_eq!(cp.power, 3, "1 + two card types (instant, creature)");
        assert_eq!(cp.toughness, 3);
    }

    #[test]
    fn speaker_of_the_heavens_makes_angel_only_when_high_on_life() {
        let mut g = two_player_game();
        let speaker = g.add_card_to_battlefield(0, catalog::speaker_of_the_heavens());
        g.clear_sickness(speaker);
        // At 20 life the ability is illegal — no Angel.
        g.players[0].life = 20;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        assert!(g.perform_action(GameAction::ActivateAbility {
            card_id: speaker, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).is_err(), "blocked below the +7 life threshold");
        // Untap and climb to 27 — now it fires.
        g.battlefield_find_mut(speaker).unwrap().tapped = false;
        g.players[0].life = 27;
        activate(&mut g, speaker, 0, None);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Angel"), "Angel token created");
    }
}

mod recent48 {
    use crabomination::card::{CounterType, Keyword, TokenDefinition};
    use crabomination::catalog;
    use crabomination::game::effects::{EffectContext, EntityRef};
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn counters(g: &GameState, id: CardId) -> u32 {
        g.battlefield_find(id)
            .and_then(|c| c.counters.get(&CounterType::PlusOnePlusOne).copied())
            .unwrap_or(0)
    }

    fn flyer() -> TokenDefinition {
        TokenDefinition {
            name: "Bird".into(),
            power: 2,
            toughness: 2,
            card_types: vec![crabomination::card::CardType::Creature],
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }
    }

    #[test]
    fn predator_ooze_grows_on_attack() {
        let mut g = two_player_game();
        let ooze = g.add_card_to_battlefield(0, catalog::predator_ooze());
        let ctx = EffectContext::for_trigger(ooze, 0, None, 0);
        let trig = catalog::predator_ooze().triggered_abilities[0].effect.clone();
        g.resolve_effect(&trig, &ctx).unwrap();
        assert_eq!(counters(&g, ooze), 1, "attack adds a +1/+1 counter");
    }

    #[test]
    fn hornet_nest_spawns_insects_when_damaged() {
        let mut g = two_player_game();
        let nest = g.add_card_to_battlefield(0, catalog::hornet_nest());
        let mut events = Vec::new();
        g.deal_damage_to_from(EntityRef::Permanent(nest), 1, None, &mut events);
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        let insects = g.battlefield.iter().filter(|c| c.definition.name == "Insect" && c.controller == 0).count();
        assert_eq!(insects, 1, "one Insect per point of damage");
    }

    #[test]
    fn aerie_ouphes_sacs_to_shoot_a_flier() {
        let mut g = two_player_game();
        let ouphe = g.add_card_to_battlefield(0, catalog::aerie_ouphes());
        let bird = g.add_token_to_battlefield(1, &flyer());
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: ouphe, ability_index: 0, target: Some(Target::Permanent(bird)),
            additional_targets: vec![], x_value: None,
        }).expect("sac ability");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bird).is_none(), "the flier took 3 and died");
    }

    #[test]
    fn walking_atlas_drops_a_land() {
        let mut g = two_player_game();
        let atlas = g.add_card_to_battlefield(0, catalog::walking_atlas());
        let forest = g.add_card_to_hand(0, catalog::forest());
        g.clear_sickness(atlas);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Cards(vec![forest]),
        ]));
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: atlas, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap for a land");
        drain_stack(&mut g);
        assert!(g.battlefield_find(forest).is_some(), "land entered from hand");
    }

    #[test]
    fn rishkar_counters_two_creatures() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let rish = g.add_card_to_battlefield(0, catalog::rishkar_peema_renegade());
        g.fire_self_etb_triggers(rish, 0);
        drain_stack(&mut g);
        assert_eq!(counters(&g, a) + counters(&g, b), 2, "two counters distributed");
    }

    #[test]
    fn rishkar_grants_mana_to_counter_creatures() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
        g.add_card_to_battlefield(0, catalog::rishkar_peema_renegade());
        g.clear_sickness(bear);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: bear, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("granted mana ability");
        assert!(g.players[0].mana_pool.amount(Color::Green) >= 1, "tapped the counter-bearing bear for green");
    }

    #[test]
    fn gnarlid_colony_kicked_enters_with_counters() {
        let mut g = two_player_game();
        let gn = g.add_card_to_hand(0, catalog::gnarlid_colony());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpellKicked {
            card_id: gn, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast kicked");
        drain_stack(&mut g);
        assert_eq!(counters(&g, gn), 2, "kicked → two +1/+1 counters");
        let cp = g.computed_permanent(gn).unwrap();
        assert!(cp.keywords.contains(&Keyword::Trample), "counter-bearing creature has trample");
    }
}

mod recent49 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    #[test]
    fn ghoultree_costs_less_per_graveyard_creature() {
        let mut g = two_player_game();
        for _ in 0..7 {
            g.add_card_to_graveyard(0, catalog::grizzly_bears());
        }
        let tree = g.add_card_to_hand(0, catalog::ghoultree());
        g.players[0].mana_pool.add(Color::Green, 1); // {7}{G} - 7 creatures = {G}
        g.perform_action(GameAction::CastSpell {
            card_id: tree, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("Ghoultree castable for {G} with 7 creatures in the yard");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Ghoultree"));
    }

    #[test]
    fn nyx_weaver_mills_on_upkeep() {
        let mut g = two_player_game();
        let nyx = g.add_card_to_battlefield(0, catalog::nyx_weaver());
        for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
        let gy0 = g.players[0].graveyard.len();
        let trig = catalog::nyx_weaver().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_trigger(nyx, 0, None, 0);
        g.resolve_effect(&trig, &ctx).unwrap();
        assert_eq!(g.players[0].graveyard.len(), gy0 + 2, "milled two");
    }

    #[test]
    fn genesis_returns_a_creature_from_the_yard() {
        let mut g = two_player_game();
        let gcard = g.add_card_to_graveyard(0, catalog::genesis());
        let want = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let trig = catalog::genesis().triggered_abilities[0].effect.clone();
        let mut ctx = EffectContext::for_trigger(gcard, 0, None, 0);
        ctx.targets = vec![Target::Permanent(want)];
        g.resolve_effect(&trig, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == want), "creature card returned to hand");
    }

    #[test]
    fn elephant_guide_pumps_and_leaves_an_elephant() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::elephant_guide());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast aura on bear");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "+3/+3 from the Guide");
        // Bear dies (lethal SBA records the aura link) → 3/3 Elephant.
        g.battlefield_find_mut(bear).unwrap().damage = 99;
        let events = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Elephant"), "Elephant token created");
    }

    #[test]
    fn moldervine_cloak_pumps_and_has_dredge() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::moldervine_cloak());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast aura on bear");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().toughness, 5, "+3/+3 from the Cloak");
        assert!(catalog::moldervine_cloak().keywords.iter().any(|k| matches!(k, crabomination::card::Keyword::Dredge(2))));
    }
}

mod recent50 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    #[test]
    fn enigma_drake_power_tracks_graveyard_spells() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature — not counted
        let drake = g.add_card_to_battlefield(0, catalog::enigma_drake());
        let cp = g.computed_permanent(drake).unwrap();
        assert_eq!(cp.power, 2, "two instants in the graveyard");
        assert_eq!(cp.toughness, 4);
    }

    #[test]
    fn niblis_of_frost_taps_and_locks_on_spellcast() {
        let mut g = two_player_game();
        let niblis = g.add_card_to_battlefield(0, catalog::niblis_of_frost());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let trig = catalog::niblis_of_frost().triggered_abilities[0].effect.clone();
        let mut ctx = EffectContext::for_trigger(niblis, 0, None, 0);
        ctx.targets = vec![Target::Permanent(foe)];
        g.resolve_effect(&trig, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's creature tapped");
        assert!(g.battlefield_find(foe).unwrap().skip_next_untap, "and locked down for its next untap");
    }

    #[test]
    fn wavesifter_investigates_twice() {
        let mut g = two_player_game();
        let ws = g.add_card_to_battlefield(0, catalog::wavesifter());
        g.fire_self_etb_triggers(ws, 0);
        drain_stack(&mut g);
        let clues = g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count();
        assert_eq!(clues, 2, "two Clue tokens");
    }
}
