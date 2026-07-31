//! Tests for recentN card batches 31-40 (merged from per-batch micro-files).

mod recent31 {
    use crabomination::card::{CardType, CreatureType, Effect, Keyword};
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn ctx0(_g: &GameState) -> EffectContext {
        EffectContext::for_ability(crabomination::card::CardId(0), 0, None)
    }

    fn modes(def: crabomination::card::CardDefinition) -> Vec<Effect> {
        match def.effect {
            Effect::ChooseMode(m) => m,
            Effect::ChooseN { modes, .. } => modes,
            other => panic!("not a modal card: {other:?}"),
        }
    }

    #[test]
    fn gruul_charm_burns_flyers() {
        let mut g = two_player_game();
        let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
        let m = modes(catalog::gruul_charm());
        g.resolve_effect(&m[2], &ctx0(&g)).unwrap();
        drain_stack(&mut g);
        // 3 damage marked on a 4-toughness flyer survives; assert the damage landed.
        assert_eq!(g.battlefield_find(flyer).unwrap().damage, 3, "3 damage to each flyer");
    }

    #[test]
    fn dimir_charm_destroys_small_creature() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mut ctx = ctx0(&g);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&modes(catalog::dimir_charm())[1], &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "power-2 creature destroyed");
    }

    #[test]
    fn orzhov_charm_destroy_loses_toughness_life() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let mut ctx = ctx0(&g);
        ctx.targets = vec![Target::Permanent(angel)];
        g.resolve_effect(&modes(catalog::orzhov_charm())[1], &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(angel).is_none(), "creature destroyed");
        assert_eq!(g.players[0].life, 16, "lost life equal to its toughness (4)");
    }

    #[test]
    fn naya_charm_burns_a_creature() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mut ctx = ctx0(&g);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&modes(catalog::naya_charm())[0], &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.battlefield_find(bear).is_none(), "3 damage kills a 2/2");
    }

    #[test]
    fn naya_charm_taps_only_target_players_creatures() {
        // CR 107.3 / 508 — "target player controls": mode 3 taps the chosen
        // player's creatures and leaves everyone else's untapped.
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let mut ctx = ctx0(&g);
        ctx.targets = vec![Target::Player(1)];
        g.resolve_effect(&modes(catalog::naya_charm())[2], &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).unwrap().tapped, "target player's creature tapped");
        assert!(!g.battlefield_find(mine).unwrap().tapped, "our own creature untouched");
    }

    #[test]
    fn jund_charm_adds_two_counters() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = ctx0(&g);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&modes(catalog::jund_charm())[2], &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "two +1/+1 counters");
    }

    #[test]
    fn grixis_charm_shrinks_creature() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let mut ctx = ctx0(&g);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&modes(catalog::grixis_charm())[1], &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.battlefield_find(bear).is_none(), "-4/-4 kills a 2/2");
    }

    #[test]
    fn silumgars_command_minus_three() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let mut ctx = ctx0(&g);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&modes(catalog::silumgars_command())[2], &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.battlefield_find(bear).is_none(), "-3/-3 kills a 2/2");
    }

    #[test]
    fn ojutais_command_gains_life() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        g.resolve_effect(&modes(catalog::ojutais_command())[1], &ctx0(&g)).unwrap();
        assert_eq!(g.players[0].life, 24);
    }

    #[test]
    fn atarkas_command_burns_opponent() {
        let mut g = two_player_game();
        let before = g.players[1].life;
        g.resolve_effect(&modes(catalog::atarkas_command())[1], &ctx0(&g)).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, before - 3, "3 damage to each opponent");
    }

    #[test]
    fn lhurgoyf_counts_all_graveyard_creatures() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let id = g.add_card_to_battlefield(0, catalog::lhurgoyf());
        let cp = g.computed_permanent(id).unwrap();
        // 2 creature cards across all graveyards → power 2, toughness 2+1.
        assert_eq!((cp.power, cp.toughness), (2, 3));
    }

    #[test]
    fn boneyard_wurm_counts_only_your_graveyard() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(1, catalog::grizzly_bears()); // opponent's — ignored
        let id = g.add_card_to_battlefield(0, catalog::boneyard_wurm());
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "only your graveyard counts");
    }

    #[test]
    fn splinterfright_is_a_trampling_goyf() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let id = g.add_card_to_battlefield(0, catalog::splinterfright());
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2));
        assert!(cp.keywords.contains(&Keyword::Trample));
    }

    #[test]
    fn disciple_of_bolas_sacrifices_for_value() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 sac fodder
        let hand_before = g.players[0].hand.len();
        let id = g.add_card_to_battlefield(0, catalog::disciple_of_bolas());
        g.fire_self_etb_triggers(id, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 22, "gained life = sacrificed power (2)");
        assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew = sacrificed power (2)");
    }

    #[test]
    fn agony_warp_splits_minus_across_two_targets() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mut ctx = ctx0(&g);
        ctx.targets = vec![Target::Permanent(a), Target::Permanent(b)];
        g.resolve_effect(&catalog::agony_warp().effect, &ctx).unwrap();
        g.check_state_based_actions();
        // a got -3/-0 (power 2→-1, survives as 0); b got -0/-3 (toughness 2→-1, dies).
        assert!(g.battlefield_find(b).is_none(), "-0/-3 kills the second target");
        assert!(g.battlefield_find(a).is_some(), "-3/-0 doesn't kill the first");
    }

    #[test]
    fn savage_knuckleblade_pumps_itself() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::savage_knuckleblade());
        let mut ctx = ctx0(&g);
        ctx.source = Some(id);
        g.resolve_effect(&catalog::savage_knuckleblade().activated_abilities[0].effect, &ctx).unwrap();
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (6, 6), "the firebreathing ability pumps +2/+2");
    }

    #[test]
    fn butcher_grants_chosen_keyword_for_a_sacrifice() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::butcher_of_the_horde());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
        let mut ctx = ctx0(&g);
        ctx.source = Some(id);
        // Default decider picks mode 0 (vigilance).
        g.resolve_effect(&catalog::butcher_of_the_horde().activated_abilities[0].effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Vigilance));
        assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 1,
            "sacrificed another creature");
    }

    #[test]
    fn demonic_dread_has_cascade_and_grants_fear() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut ctx = ctx0(&g);
        ctx.targets = vec![Target::Permanent(bear)];
        g.resolve_effect(&catalog::demonic_dread().effect, &ctx).unwrap();
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Fear));
        // Cascade rides a printed cast trigger.
        assert!(catalog::demonic_dread().triggered_abilities.iter()
            .any(|t| matches!(t.effect, Effect::Cascade { .. })));
    }

    #[test]
    fn glory_activates_from_graveyard() {
        let def = catalog::glory();
        assert!(def.keywords.contains(&Keyword::Flying));
        assert!(def.activated_abilities[0].from_graveyard, "protection grant is graveyard-only");
    }

    #[test]
    fn foul_tongue_invocation_sacs_and_gains_with_dragon() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        let dragon = crabomination::card::CardDefinition {
            name: "Test Dragon",
            cost: crabomination::mana::cost(&[crabomination::mana::r()]),
            card_types: vec![CardType::Creature],
            subtypes: crabomination::card::Subtypes {
                creature_types: vec![CreatureType::Dragon], ..Default::default()
            },
            power: 4, toughness: 4,
            ..Default::default()
        };
        g.add_card_to_battlefield(0, dragon);
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let mut ctx = ctx0(&g);
        ctx.targets = vec![Target::Player(1)];
        g.resolve_effect(&catalog::foul_tongue_invocation().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "target player sacrificed a creature");
        assert_eq!(g.players[0].life, 24, "gained 4 — you control a Dragon");
    }

    #[test]
    fn first_sliver_grants_sliver_spells_cascade() {
        let def = catalog::the_first_sliver();
        // Its own printed cascade plus the battlefield "Sliver spells have cascade".
        let cascades = def.triggered_abilities.iter()
            .filter(|t| matches!(t.effect, Effect::Cascade { .. })).count();
        assert_eq!(cascades, 2);
        assert!(def.subtypes.creature_types.contains(&CreatureType::Sliver));
    }

    #[test]
    fn mortivore_regenerates() {
        let def = catalog::mortivore();
        assert!(matches!(def.activated_abilities[0].effect, Effect::Regenerate { .. }));
    }
}

mod recent32 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn ctx_for(source: CardId) -> EffectContext {
        EffectContext::for_ability(source, 0, None)
    }

    fn ability0_effect(def: crabomination::card::CardDefinition) -> crabomination::card::Effect {
        def.activated_abilities.into_iter().next().unwrap().effect
    }

    #[test]
    fn bloodflow_connoisseur_grows_on_sacrifice() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::bloodflow_connoisseur());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder
        g.resolve_effect(&ability0_effect(catalog::bloodflow_connoisseur()), &ctx_for(id)).unwrap();
        drain_stack(&mut g);
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2), "a +1/+1 counter from the sacrifice");
    }

    #[test]
    fn vampire_aristocrat_pumps_on_sacrifice() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::vampire_aristocrat());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.resolve_effect(&ability0_effect(catalog::vampire_aristocrat()), &ctx_for(id)).unwrap();
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4));
    }

    #[test]
    fn cartel_aristocrat_gains_protection() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::cartel_aristocrat());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.resolve_effect(&ability0_effect(catalog::cartel_aristocrat()), &ctx_for(id)).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "another creature was sacrificed");
        assert!(g.computed_permanent(id).unwrap().keywords.iter()
            .any(|k| matches!(k, Keyword::Protection(_))), "gained protection from a color");
    }

    #[test]
    fn yahenni_grows_when_opponent_creature_dies() {
        let mut g = two_player_game();
        let yah = g.add_card_to_battlefield(0, catalog::yahenni_undying_partisan());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(foe).unwrap().damage = 2; // lethal → CreatureDied
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(yah).unwrap().power, 3, "+1/+1 counter from the opponent's death");
    }

    #[test]
    fn bontu_gate_opens_after_a_creature_dies() {
        let mut g = two_player_game();
        let bontu = g.add_card_to_battlefield(0, catalog::bontu_the_glorified());
        assert!(g.computed_permanent(bontu).unwrap().keywords
            .contains(&Keyword::CantAttackOrBlockUnlessCreatureDiedThisTurn));
        // Make Bontu an otherwise-legal attacker.
        g.active_player_idx = 0;
        g.step = crabomination::TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.battlefield_find_mut(bontu).unwrap().summoning_sick = false;
        assert!(!g.legal_attackers(0).contains(&bontu), "gate shut with no death this turn");
        g.players[0].creatures_died_this_turn = 1;
        assert!(g.legal_attackers(0).contains(&bontu), "gate opens once a creature died under your control");
    }

    #[test]
    fn bontu_ability_drains_each_opponent() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        let bontu = g.add_card_to_battlefield(0, catalog::bontu_the_glorified());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
        let before = g.players[1].life;
        // Bontu's ability is the second activated ability slot here (the only one).
        g.resolve_effect(&ability0_effect(catalog::bontu_the_glorified()), &ctx_for(bontu)).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, before - 1, "each opponent loses 1");
        assert_eq!(g.players[0].life, 21, "you gain 1");
    }

    #[test]
    fn smothering_abomination_draws_when_you_sacrifice() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_battlefield(0, catalog::smothering_abomination());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let hand_before = g.players[0].hand.len();
        // The upkeep trigger sacrifices a creature; that sacrifice fires the draw.
        let upkeep = catalog::smothering_abomination().triggered_abilities[0].effect.clone();
        let evs = g.resolve_effect(&upkeep, &EffectContext::for_ability(crabomination::card::CardId(0), 0, None)).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.definition.is_creature()),
            "a creature was sacrificed at upkeep");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "the sacrifice drew a card");
    }

    #[test]
    fn butcher_ghoul_returns_via_undying() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::butcher_ghoul());
        let mut ctx = ctx_for(id);
        ctx.targets = vec![Target::Permanent(id)];
        g.resolve_effect(&crabomination::card::Effect::Destroy { what: crabomination::effect::Selector::Target(0) }, &ctx).unwrap();
        drain_stack(&mut g);
        let ghoul = g.battlefield.iter().find(|c| c.definition.name == "Butcher Ghoul")
            .expect("Undying returned it to the battlefield");
        assert_eq!(ghoul.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1, "returns with a +1/+1 counter");
    }

    #[test]
    fn elas_il_kor_gains_life_on_creature_etb() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        g.add_card_to_battlefield(0, catalog::elas_il_kor_sadistic_pilgrim());
        // Cast another creature so its ETB event dispatches to Elas's trigger.
        g.active_player_idx = 0;
        g.step = crabomination::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("Grizzly Bears castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 21, "gained 1 from another creature entering");
    }

    #[test]
    fn mahadi_makes_treasures_for_the_dead() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::mahadi_emporium_master());
        g.players[0].creatures_died_this_turn = 2; // two creatures died this turn
        let end_step = catalog::mahadi_emporium_master().triggered_abilities[0].effect.clone();
        g.resolve_effect(&end_step, &ctx_for(id)).unwrap();
        drain_stack(&mut g);
        let treasures = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Treasure").count();
        assert_eq!(treasures, 2, "one Treasure per creature that died this turn");
    }

    #[test]
    fn heartless_summoning_shrinks_your_creatures() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::heartless_summoning());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "-1/-1 to your creatures");
    }
}

mod recent33 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn ctx_for(source: CardId) -> EffectContext {
        EffectContext::for_ability(source, 0, None)
    }

    #[test]
    fn endless_cockroaches_returns_to_hand_on_death() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::endless_cockroaches());
        g.battlefield_find_mut(id).unwrap().damage = 1; // lethal
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).is_none(), "left the battlefield");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Endless Cockroaches"),
            "returned to its owner's hand");
    }

    #[test]
    fn poison_tip_archer_drains_on_any_other_death() {
        let mut g = two_player_game();
        g.players[1].life = 20;
        g.add_card_to_battlefield(0, catalog::poison_tip_archer());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(foe).unwrap().damage = 2; // lethal
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "each opponent loses 1 when another creature dies");
    }

    #[test]
    fn altar_of_dementia_mills_equal_to_power() {
        let mut g = two_player_game();
        for _ in 0..10 { g.add_card_to_library(1, catalog::island()); }
        let altar = g.add_card_to_battlefield(0, catalog::altar_of_dementia());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 power
        let lib_before = g.players[1].library.len();
        let mut ctx = ctx_for(altar);
        ctx.targets = vec![Target::Player(1)];
        g.resolve_effect(&catalog::altar_of_dementia().activated_abilities[0].effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].library.len(), lib_before - 2, "milled = sacrificed creature's power");
    }

    #[test]
    fn sadistic_hypnotist_discards_two() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
        let hyp = g.add_card_to_battlefield(0, catalog::sadistic_hypnotist());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder
        let hand_before = g.players[1].hand.len();
        let mut ctx = ctx_for(hyp);
        ctx.targets = vec![Target::Player(1)];
        g.resolve_effect(&catalog::sadistic_hypnotist().activated_abilities[0].effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), hand_before - 2, "target player discards two");
    }

    #[test]
    fn sprout_swarm_makes_a_saproling() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::sprout_swarm());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("Sprout Swarm castable");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(), 1);
    }
}

mod recent34 {
    use crabomination::card::{CardType, CounterType, CreatureType};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::mana::Color;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    #[test]
    fn quest_for_the_goblin_lord_anthem_at_five_counters() {
        let mut g = two_player_game();
        let quest = g.add_card_to_battlefield(0, catalog::quest_for_the_goblin_lord());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        // Below threshold: no bonus.
        g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 4);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "no anthem under five counters");
        g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 1);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0 at five counters");
        assert_eq!(g.computed_permanent(bear).unwrap().toughness, 2, "toughness unchanged");
    }

    #[test]
    fn quest_for_the_gravelord_accrues_on_death_and_makes_zombie() {
        let mut g = two_player_game();
        let quest = g.add_card_to_battlefield(0, catalog::quest_for_the_gravelord());
        // A creature dies → a quest counter.
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(foe).unwrap().damage = 2;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(quest).unwrap().counters.get(&CounterType::Quest).copied().unwrap_or(0),
            1,
            "quest counter on a creature dying"
        );
        // Top up to three and activate the sacrifice ability.
        g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: quest, ability_index: 0, target: None, additional_targets: Vec::new(),
            x_value: None, mode: None,
        }).expect("remove 3 quest counters + sacrifice");
        drain_stack(&mut g);
        assert!(g.battlefield_find(quest).is_none(), "quest sacrificed");
        let zombie = g.battlefield.iter().find(|c| c.definition.name == "Zombie Giant")
            .expect("5/5 Zombie Giant minted");
        assert_eq!((zombie.power(), zombie.toughness()), (5, 5));
        assert!(zombie.definition.subtypes.creature_types.contains(&CreatureType::Zombie));
    }

    #[test]
    fn quest_for_the_gemblades_drops_four_counters_on_target() {
        let mut g = two_player_game();
        let quest = g.add_card_to_battlefield(0, catalog::quest_for_the_gemblades());
        g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 1);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: quest, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("remove a quest counter + sacrifice");
        drain_stack(&mut g);
        assert!(g.battlefield_find(quest).is_none(), "quest sacrificed");
        assert_eq!(g.computed_permanent(bear).unwrap().power, 6, "four +1/+1 counters → 6/6");
    }

    #[test]
    fn quest_for_ancient_secrets_shuffles_graveyard() {
        let mut g = two_player_game();
        let quest = g.add_card_to_battlefield(0, catalog::quest_for_ancient_secrets());
        g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 5);
        for _ in 0..4 { g.add_card_to_graveyard(0, catalog::island()); }
        let gy_before = g.players[0].graveyard.len();
        assert!(gy_before >= 4);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: quest, ability_index: 0, target: Some(Target::Player(0)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("remove 5 quest counters + sacrifice");
        drain_stack(&mut g);
        // The quest itself is sacrificed (so it's now in the graveyard), but the
        // four islands shuffled into the library.
        assert!(g.players[0].library.len() >= 4, "graveyard shuffled into library");
    }

    #[test]
    fn quest_for_the_holy_relic_tutors_an_equipment_to_play() {
        let mut g = two_player_game();
        let quest = g.add_card_to_battlefield(0, catalog::quest_for_the_holy_relic());
        g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 5);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let equip = g.add_card_to_library(0, catalog::bonesplitter()); // an Equipment to find
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(equip))]));
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: quest, ability_index: 0, target: None, additional_targets: Vec::new(),
            x_value: None, mode: None,
        }).expect("remove 5 quest counters + sacrifice");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(equip).expect("Equipment fetched onto the battlefield").attached_to,
            Some(bear),
            "fetched Equipment enters attached to your creature"
        );
    }

    #[test]
    fn magebane_lizard_burns_the_caster_per_noncreature_spell() {
        let mut g = two_player_game();
        g.players[0].life = 20;
        g.add_card_to_battlefield(1, catalog::magebane_lizard());
        let opt = g.add_card_to_hand(0, catalog::opt());
        for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: opt, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Opt");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 19, "1 noncreature spell this turn → 1 damage to caster");
    }

    #[test]
    fn atog_sacrifices_an_artifact_for_plus_two() {
        let mut g = two_player_game();
        let atog = g.add_card_to_battlefield(0, catalog::atog());
        let art = g.add_card_to_battlefield(0, catalog::ornithopter()); // an artifact
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: atog, ability_index: 0, target: None, additional_targets: Vec::new(),
            x_value: None, mode: None,
        }).expect("Sacrifice an artifact: +2/+2");
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_none(), "artifact sacrificed");
        assert_eq!(g.computed_permanent(atog).unwrap().power, 3, "1 + 2 = 3");
    }

    #[test]
    fn origin_spellbomb_sacrifices_for_a_myr() {
        let mut g = two_player_game();
        let bomb = g.add_card_to_battlefield(0, catalog::origin_spellbomb());
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: bomb, ability_index: 0, target: None, additional_targets: Vec::new(),
            x_value: None, mode: None,
        }).expect("{1}, {T}, Sacrifice: make a Myr");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bomb).is_none(), "spellbomb sacrificed");
        let myr = g.battlefield.iter().find(|c| c.definition.name == "Myr").expect("Myr minted");
        assert!(myr.definition.card_types.contains(&CardType::Artifact));
    }

    #[test]
    fn land_tax_fetches_basics_when_behind_on_lands() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::land_tax());
        // Opponent controls more lands than us (we control none).
        for _ in 0..2 { g.add_card_to_battlefield(1, catalog::island()); }
        let mut basics = Vec::new();
        for _ in 0..3 { basics.push(g.add_card_to_library(0, catalog::plains())); }
        g.decider = Box::new(ScriptedDecider::new(
            basics.iter().map(|&id| DecisionAnswer::Search(Some(id))).collect::<Vec<_>>(),
        ));
        let hand_before = g.players[0].hand.len();
        g.step = TurnStep::Upkeep;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 3, "tutored three basics to hand");
    }
}

mod recent35 {
    use crabomination::card::{Keyword, Value};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::mana::Color;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: idx, target, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("ability activates");
        drain_stack(g);
    }

    #[test]
    fn spike_weaver_enters_with_three_counters() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::spike_weaver());
        // Simulate the enters-with-counters replacement.
        let n = catalog::spike_weaver().enters_with_counters.unwrap();
        if let Value::Const(c) = n.1 {
            g.battlefield_find_mut(id).unwrap().add_counters(n.0, c as u32);
        }
        assert_eq!(g.computed_permanent(id).unwrap().power, 3, "0/0 + three +1/+1 = 3/3");
    }

    #[test]
    fn glimmerpoint_stag_blinks_a_permanent() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let stag = g.add_card_to_battlefield(0, catalog::glimmerpoint_stag());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(foe))]));
        g.fire_self_etb_triggers(stag, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "target exiled by the blink");
        assert!(g.exile.iter().any(|c| c.id == foe), "sitting in exile until end step");
    }

    #[test]
    fn weathered_wayfarer_only_works_when_behind() {
        let mut g = two_player_game();
        let way = g.add_card_to_battlefield(0, catalog::weathered_wayfarer());
        g.clear_sickness(way);
        g.add_card_to_library(0, catalog::plains());
        g.players[0].mana_pool.add(Color::White, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        // No opponent lands → activation illegal.
        let res = g.perform_action(GameAction::ActivateAbility {
            card_id: way, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        });
        assert!(res.is_err(), "can't activate while not behind on lands");
    }

    #[test]
    fn plea_for_guidance_tutors_two_enchantments() {
        let mut g = two_player_game();
        let plea = g.add_card_to_hand(0, catalog::plea_for_guidance());
        let e1 = g.add_card_to_library(0, catalog::pacifism());
        let e2 = g.add_card_to_library(0, catalog::narcolepsy());
        g.add_card_to_library(0, catalog::island());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(5);
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Search(Some(e1)), DecisionAnswer::Search(Some(e2)),
        ]));
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: plea, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Plea for Guidance");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == e1));
        assert!(g.players[0].hand.iter().any(|c| c.id == e2));
    }

    #[test]
    fn fleetfoot_dancer_has_the_three_keywords() {
        let g = two_player_game();
        let _ = g;
        let kw = catalog::fleetfoot_dancer().keywords;
        assert!(kw.contains(&Keyword::Trample) && kw.contains(&Keyword::Lifelink) && kw.contains(&Keyword::Haste));
    }

    #[test]
    fn stormscape_apprentice_taps_a_creature() {
        let mut g = two_player_game();
        let app = g.add_card_to_battlefield(0, catalog::stormscape_apprentice());
        g.clear_sickness(app);
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::White, 1);
        activate(&mut g, app, 0, Some(Target::Permanent(foe)));
        assert!(g.battlefield_find(foe).unwrap().tapped, "target creature tapped");
    }

    #[test]
    fn stonecloaker_exiles_a_graveyard_card() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // a creature to bounce
        let cloak = g.add_card_to_battlefield(0, catalog::stonecloaker());
        // Answer the bounce-own (first ETB) and the gy-exile (second ETB).
        let own = g.battlefield.iter()
            .find(|c| c.controller == 0 && c.definition.name == "Grizzly Bears").unwrap().id;
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Target(Target::Permanent(own)),
            DecisionAnswer::Target(Target::Permanent(dead)),
        ]));
        g.fire_self_etb_triggers(cloak, 0);
        drain_stack(&mut g);
        assert!(!g.players[1].graveyard.iter().any(|c| c.id == dead), "graveyard card exiled");
    }

    #[test]
    fn stonehorn_dignitary_makes_opponent_skip_combat() {
        let mut g = two_player_game();
        let dig = g.add_card_to_battlefield(0, catalog::stonehorn_dignitary());
        g.fire_self_etb_triggers(dig, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].skip_next_combat, 1, "opponent will skip their next combat");
    }

    #[test]
    fn skip_next_combat_jumps_over_the_combat_phase() {
        let mut g = two_player_game();
        // Player 0 is the active player; give them a skip charge.
        g.active_player_idx = 0;
        g.players[0].skip_next_combat = 1;
        g.step = TurnStep::PreCombatMain;
        let evs = g.advance_step(Vec::new()).expect("advance from precombat main");
        let _ = evs;
        assert_eq!(g.step, TurnStep::PostCombatMain, "combat phase skipped");
        assert_eq!(g.players[0].skip_next_combat, 0, "skip charge consumed");
    }

    #[test]
    fn bile_blight_hits_all_same_name() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let blight = g.add_card_to_hand(0, catalog::bile_blight());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: blight, target: Some(Target::Permanent(a)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast Bile Blight");
        drain_stack(&mut g);
        // Both 2/2 Bears take -3/-3 and die.
        assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(),
            "both same-named creatures destroyed");
    }
}

mod recent36 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::mana::Color;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn drain_dies(g: &mut GameState, id: CardId) {
        g.battlefield_find_mut(id).unwrap().damage = 999;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }

    #[test]
    fn hour_of_promise_fetches_two_lands_tapped() {
        let mut g = two_player_game();
        let hop = g.add_card_to_hand(0, catalog::hour_of_promise());
        let l1 = g.add_card_to_library(0, catalog::forest());
        let l2 = g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Search(Some(l1)), DecisionAnswer::Search(Some(l2)),
        ]));
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: hop, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Hour of Promise");
        drain_stack(&mut g);
        assert!(g.battlefield_find(l1).is_some_and(|c| c.tapped));
        assert!(g.battlefield_find(l2).is_some_and(|c| c.tapped));
    }

    #[test]
    fn pirs_whim_makes_opponent_sacrifice() {
        let mut g = two_player_game();
        let whim = g.add_card_to_hand(0, catalog::pirs_whim());
        let land = g.add_card_to_library(0, catalog::forest());
        let art = g.add_card_to_battlefield(1, catalog::ornithopter());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(land))]));
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: whim, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Pir's Whim");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_some(), "we fetched a land");
        assert!(g.battlefield_find(art).is_none(), "opponent sacrificed their artifact");
    }

    #[test]
    fn wayward_swordtooth_gated_until_city_blessing() {
        let mut g = two_player_game();
        let dino = g.add_card_to_battlefield(0, catalog::wayward_swordtooth());
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.battlefield_find_mut(dino).unwrap().summoning_sick = false;
        assert!(!g.legal_attackers(0).contains(&dino), "gated without the city's blessing");
        g.players[0].city_blessing = true;
        assert!(g.legal_attackers(0).contains(&dino), "can attack with the city's blessing");
    }

    #[test]
    fn gather_the_pack_takes_a_creature() {
        let mut g = two_player_game();
        let gtp = g.add_card_to_hand(0, catalog::gather_the_pack());
        let bear = g.add_card_to_library(0, catalog::grizzly_bears());
        for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: gtp, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Gather the Pack");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature to hand");
    }

    #[test]
    fn trackers_instincts_has_flashback() {
        assert!(catalog::trackers_instincts().keywords.iter()
            .any(|k| matches!(k, Keyword::Flashback(_))));
    }

    #[test]
    fn dictate_of_kruphix_draws_extra_on_draw_step() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::dictate_of_kruphix());
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        g.active_player_idx = 0;
        let hand_before = g.players[0].hand.len();
        g.fire_step_triggers(TurnStep::Draw);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "active player drew the extra card");
    }

    #[test]
    fn mogg_flunkies_cant_act_alone() {
        assert!(catalog::mogg_flunkies().keywords.contains(&Keyword::CantAttackOrBlockAlone));
    }

    #[test]
    fn wily_goblin_makes_a_treasure() {
        let mut g = two_player_game();
        let gob = g.add_card_to_battlefield(0, catalog::wily_goblin());
        g.fire_self_etb_triggers(gob, 0);
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "Treasure minted");
    }

    #[test]
    fn hunted_witness_leaves_a_lifelink_soldier() {
        let mut g = two_player_game();
        let w = g.add_card_to_battlefield(0, catalog::hunted_witness());
        drain_dies(&mut g, w);
        let sol = g.battlefield.iter().find(|c| c.definition.name == "Soldier").expect("Soldier token");
        assert!(sol.definition.keywords.contains(&Keyword::Lifelink));
    }

    #[test]
    fn brindle_shoat_leaves_a_three_three_boar() {
        let mut g = two_player_game();
        let s = g.add_card_to_battlefield(0, catalog::brindle_shoat());
        drain_dies(&mut g, s);
        let boar = g.battlefield.iter().find(|c| c.definition.name == "Boar").expect("Boar token");
        assert_eq!((boar.power(), boar.toughness()), (3, 3));
    }

    #[test]
    fn goblin_assault_mints_a_hasty_goblin_each_upkeep() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::goblin_assault());
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        let gob = g.battlefield.iter().find(|c| c.definition.name == "Goblin").expect("Goblin token");
        assert!(gob.definition.keywords.contains(&Keyword::Haste));
    }

    #[test]
    fn goblin_rally_makes_four() {
        let mut g = two_player_game();
        let rally = g.add_card_to_hand(0, catalog::goblin_rally());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: rally, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Goblin Rally");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count(), 4);
    }

    #[test]
    fn bottomless_pit_discards_at_each_upkeep() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::bottomless_pit());
        for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); }
        g.active_player_idx = 0;
        let hand_before = g.players[0].hand.len();
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before - 1, "active player discarded one");
    }
}

mod recent37 {
    use crabomination::card::{CardType, Supertype};
    use crabomination::catalog;
    use crabomination::mana::Color;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn cast(g: &mut GameState, id: CardId) {
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("spell castable");
        drain_stack(g);
    }

    #[test]
    fn mesa_enchantress_draws_on_enchantment_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::mesa_enchantress());
        g.add_card_to_library(0, catalog::island()); // something to draw
        let aura = g.add_card_to_hand(0, catalog::pacifism());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        let hand_before = g.players[0].hand.len();
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(foe)), additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("cast Pacifism");
        drain_stack(&mut g);
        // Cast the aura (−1 hand) then drew a card (+1) → net unchanged.
        assert_eq!(g.players[0].hand.len(), hand_before, "enchantress replaced the cast aura");
    }

    #[test]
    fn femeref_enchantress_draws_when_enchantment_dies() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::femeref_enchantress());
        g.add_card_to_library(0, catalog::island());
        let ench = g.add_card_to_battlefield(0, catalog::pacifism());
        let hand_before = g.players[0].hand.len();
        g.remove_from_battlefield_to_graveyard_raw(ench);
        // The enchantment now sits in the graveyard; fire the put-into-graveyard event.
        g.dispatch_triggers_for_events(&[GameEvent::CardPutIntoGraveyard {
            player: 0, card_id: ench, is_land: false,
        }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew when an enchantment hit the graveyard");
    }

    #[test]
    fn eidolon_of_blossoms_draws_on_its_own_entry() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let eid = g.add_card_to_battlefield(0, catalog::eidolon_of_blossoms());
        let hand_before = g.players[0].hand.len();
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: eid }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "constellation fires on its own entry");
    }

    #[test]
    fn mutilate_scales_with_swamps() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_battlefield(0, catalog::swamp()); }
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let mut0 = g.add_card_to_hand(0, catalog::mutilate());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, mut0);
        // -3/-3 (three Swamps) kills the 2/2.
        assert!(g.battlefield_find(foe).is_none(), "three Swamps → -3/-3 wipe");
    }

    #[test]
    fn golden_demise_minus_two() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let gd = g.add_card_to_hand(0, catalog::golden_demise());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, gd);
        assert!(g.battlefield_find(foe).is_none(), "-2/-2 kills a 2/2");
    }

    #[test]
    fn yahennis_expertise_minus_three() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        let ye = g.add_card_to_hand(0, catalog::yahennis_expertise());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, ye);
        // 4/4 → 1/1, survives; assert it took the shrink (toughness now 1).
        assert_eq!(g.computed_permanent(foe).unwrap().toughness, 1, "-3/-3 applied");
    }

    #[test]
    fn sword_of_the_animist_is_legendary() {
        assert!(catalog::sword_of_the_animist().supertypes.contains(&Supertype::Legendary));
    }

    #[test]
    fn dawn_of_hope_makes_a_soldier() {
        let mut g = two_player_game();
        let dawn = g.add_card_to_battlefield(0, catalog::dawn_of_hope());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: dawn, ability_index: 0, target: None, additional_targets: Vec::new(),
            x_value: None, mode: None,
        }).expect("{3}{W}: make a Soldier");
        drain_stack(&mut g);
        let sol = g.battlefield.iter().find(|c| c.definition.name == "Soldier")
            .expect("Soldier minted");
        assert!(sol.definition.keywords.contains(&crabomination::card::Keyword::Lifelink));
        assert!(sol.definition.card_types.contains(&CardType::Creature));
    }
}

mod recent38 {
    use crabomination::catalog;
    use crabomination::card::StaticEffect;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn ctx_for(source: CardId) -> EffectContext {
        EffectContext::for_ability(source, 0, None)
    }

    fn rider(def: crabomination::card::CardDefinition) -> Effect {
        def.triggered_abilities[0].effect.clone()
    }

    #[test]
    fn monuments_reduce_their_color() {
        for (def, _) in [
            (catalog::oketras_monument(), 'W'),
            (catalog::kefnets_monument(), 'U'),
            (catalog::hazorets_monument(), 'R'),
            (catalog::rhonass_monument(), 'G'),
        ] {
            assert!(def.static_abilities.iter().any(|s| matches!(
                s.effect, StaticEffect::CostReduction { amount: 1, .. }
            )), "{} reduces creature spells", def.name);
        }
    }

    #[test]
    fn oketras_monument_mints_a_vigilant_warrior() {
        let mut g = two_player_game();
        let mon = g.add_card_to_battlefield(0, catalog::oketras_monument());
        g.resolve_effect(&rider(catalog::oketras_monument()), &ctx_for(mon)).unwrap();
        drain_stack(&mut g);
        let w = g.battlefield.iter().find(|c| c.definition.name == "Warrior").expect("Warrior token");
        assert!(w.definition.keywords.contains(&crabomination::card::Keyword::Vigilance));
    }

    #[test]
    fn kefnets_monument_locks_an_opponents_untap() {
        let mut g = two_player_game();
        let mon = g.add_card_to_battlefield(0, catalog::kefnets_monument());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let mut ctx = ctx_for(mon);
        ctx.targets = vec![Target::Permanent(foe)];
        g.resolve_effect(&rider(catalog::kefnets_monument()), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).unwrap().skip_next_untap, "target won't untap next untap step");
    }

    #[test]
    fn rhonass_monument_pumps_and_tramples() {
        let mut g = two_player_game();
        let mon = g.add_card_to_battlefield(0, catalog::rhonass_monument());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let mut ctx = ctx_for(mon);
        ctx.targets = vec![Target::Permanent(mine)];
        g.resolve_effect(&rider(catalog::rhonass_monument()), &ctx).unwrap();
        drain_stack(&mut g);
        let cp = g.computed_permanent(mine).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Trample), "gained trample");
    }

    #[test]
    fn hazorets_monument_can_loot() {
        let mut g = two_player_game();
        let mon = g.add_card_to_battlefield(0, catalog::hazorets_monument());
        g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to pitch
        g.add_card_to_library(0, catalog::island()); // a card to draw
        let hand_before = g.players[0].hand.len();
        g.resolve_effect(&rider(catalog::hazorets_monument()), &ctx_for(mon)).unwrap();
        drain_stack(&mut g);
        // Loot is net-neutral on hand size (discard one, draw one).
        assert_eq!(g.players[0].hand.len(), hand_before, "looted: −1 +1");
    }
}

mod recent39 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    use crabomination::game::*;

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
}

mod recent40 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::effects::EntityRef;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn activate(g: &mut GameState, id: CardId, idx: usize) {
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: idx, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("ability activates");
        drain_stack(g);
    }

    #[test]
    fn ancient_tomb_pings_its_controller() {
        let mut g = two_player_game();
        let tomb = g.add_card_to_battlefield(0, catalog::ancient_tomb());
        let life = g.players[0].life;
        activate(&mut g, tomb, 0);
        assert_eq!(g.players[0].life, life - 2, "tapping Ancient Tomb deals 2 to you");
        assert_eq!(g.players[0].mana_pool.total(), 2, "added two colorless");
    }

    #[test]
    fn tarnished_citadel_painful_rainbow() {
        let mut g = two_player_game();
        let cit = g.add_card_to_battlefield(0, catalog::tarnished_citadel());
        let life = g.players[0].life;
        // ability 0 = {C} (painless), ability 1 = any color + 3 damage.
        activate(&mut g, cit, 1);
        assert_eq!(g.players[0].life, life - 3, "the any-color mode deals 3 to you");
    }

    #[test]
    fn castle_locthwain_draws_and_drains() {
        let mut g = two_player_game();
        let castle = g.add_card_to_battlefield(0, catalog::castle_locthwain());
        for _ in 0..3 { g.add_card_to_library(0, catalog::swamp()); }
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
        g.players[0].mana_pool.add_colorless(1);
        let life = g.players[0].life;
        let hand = g.players[0].hand.len();
        activate(&mut g, castle, 1); // draw-then-lose-life
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        // After drawing, hand has `hand + 1` cards → lose that much life.
        assert_eq!(g.players[0].life, life - (hand as i32 + 1), "lose life = cards in hand");
    }

    #[test]
    fn castle_ardenvale_makes_a_human() {
        let mut g = two_player_game();
        let castle = g.add_card_to_battlefield(0, catalog::castle_ardenvale());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        activate(&mut g, castle, 1);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Human"),
            "minted a 1/1 Human token"
        );
    }

    #[test]
    fn faceless_haven_animates_to_a_changeling() {
        let mut g = two_player_game();
        let haven = g.add_card_to_battlefield(0, catalog::faceless_haven());
        g.players[0].mana_pool.add_snow(crabomination::mana::Color::White, 3);
        activate(&mut g, haven, 1);
        let cp = g.computed_permanent(haven).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 3), "becomes a 4/3");
        assert!(cp.keywords.contains(&Keyword::Vigilance), "with vigilance");
        assert!(cp.keywords.contains(&Keyword::Changeling), "and all creature types");
        assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
    }

    #[test]
    fn crawling_barrens_grows_and_animates() {
        let mut g = two_player_game();
        let barrens = g.add_card_to_battlefield(0, catalog::crawling_barrens());
        g.players[0].mana_pool.add_colorless(4);
        activate(&mut g, barrens, 1);
        let cp = g.computed_permanent(barrens).unwrap();
        // 0/0 base + two +1/+1 counters = 2/2.
        assert_eq!((cp.power, cp.toughness), (2, 2), "two +1/+1 counters on the 0/0 land");
    }

    #[test]
    fn field_of_the_dead_makes_a_zombie_at_seven_names() {
        let mut g = two_player_game();
        // Six distinct lands + Field of the Dead = seven differently-named lands.
        g.add_card_to_battlefield(0, catalog::plains());
        g.add_card_to_battlefield(0, catalog::island());
        g.add_card_to_battlefield(0, catalog::swamp());
        g.add_card_to_battlefield(0, catalog::mountain());
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::ancient_tomb());
        g.add_card_to_battlefield(0, catalog::field_of_the_dead());
        // An eighth distinct land enters → trigger sees ≥7 names.
        let newcomer = g.add_card_to_battlefield(0, catalog::city_of_traitors());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: newcomer }]);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Zombie"),
            "made a 2/2 Zombie"
        );
    }

    #[test]
    fn glacial_chasm_prevents_all_damage_to_you() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::glacial_chasm());
        let life = g.players[0].life;
        let mut events = Vec::new();
        g.deal_damage_to_from(EntityRef::Player(0), 5, None, &mut events);
        assert_eq!(g.players[0].life, life, "all damage to the controller is prevented");
        // The opponent (no Chasm) still takes damage.
        let foe_life = g.players[1].life;
        g.deal_damage_to_from(EntityRef::Player(1), 5, None, &mut events);
        assert_eq!(g.players[1].life, foe_life - 5, "the other player is unaffected");
    }
}
