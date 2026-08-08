//! Tests for recentN card batches 238-252 (merged from per-batch micro-files).

mod recent238 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::{Effect, SpreeMode};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{drain_stack, two_player_game};

    fn spree_modes(def: &crabomination::card::CardDefinition) -> Vec<SpreeMode> {
        match &def.effect {
            Effect::Spree { modes } => modes.clone(),
            _ => panic!("not a spree card"),
        }
    }

    /// Prized Griffin is a 3/4 flier.
    #[test]
    fn prized_griffin_stats() {
        let def = catalog::prized_griffin();
        assert_eq!((def.power, def.toughness), (3, 4));
        assert!(def.keywords.contains(&Keyword::Flying));
    }

    /// Abhorrent Oculus can only be cast after exiling six graveyard cards, and
    /// manifests dread on each opponent's upkeep.
    #[test]
    fn abhorrent_oculus_shape() {
        use crabomination::card::AdditionalCastCost;
        let def = catalog::abhorrent_oculus();
        match &def.additional_cast_cost[0] {
            AdditionalCastCost::ExileFromGraveyard { count, .. } => assert_eq!(*count, 6),
            other => panic!("unexpected cost: {other:?}"),
        }
        assert!(!def.triggered_abilities.is_empty(), "has the upkeep manifest trigger");
    }

    /// Lively Dirge's second mode reanimates creatures totalling MV<=4.
    #[test]
    fn lively_dirge_reanimates() {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
        let modes = spree_modes(&catalog::lively_dirge());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
        g.resolve_effect(&modes[1].effect, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == bear), "bear reanimated");
    }

    /// Smuggler's Surprise mode 3 grants hexproof + indestructible to big creatures.
    #[test]
    fn smugglers_surprise_protects_big() {
        let mut g = two_player_game();
        let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        let modes = spree_modes(&catalog::smugglers_surprise());
        g.resolve_effect(&modes[2].effect, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        let kws = g.computed_permanent(big).unwrap().keywords.clone();
        assert!(kws.contains(&Keyword::Hexproof) && kws.contains(&Keyword::Indestructible));
    }

    /// Prairie Dog grows at end step only if you haven't cast a spell from hand,
    /// and its {4}{W} adds an extra counter to placements this turn.
    #[test]
    fn prairie_dog_from_hand_and_counter_bonus() {
        let mut g = two_player_game();
        let dog = g.add_card_to_battlefield(0, catalog::prairie_dog());
        // No from-hand cast this turn → the end-step trigger's filter holds.
        let trig = &catalog::prairie_dog().triggered_abilities[0];
        let ctx = EffectContext::for_trigger(dog, 0, None, 0);
        assert!(
            g.evaluate_predicate(trig.event.filter.as_ref().unwrap(), &ctx),
            "haven't cast from hand → trigger fires",
        );
        // Activate {4}{W}: counter placements now get +1.
        let act = catalog::prairie_dog().activated_abilities[0].effect.clone();
        g.resolve_effect(&act, &ctx).unwrap();
        g.resolve_effect(
            &Effect::AddCounter {
                what: crabomination::effect::Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: crabomination::effect::Value::ONE,
            },
            &ctx,
        )
        .unwrap();
        assert_eq!(
            g.battlefield_find(dog).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2,
            "1 placed + 1 bonus",
        );
    }
}

mod recent239 {
    use crabomination::card::{AdditionalCastCost, CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::{Effect, Predicate};
    use crabomination::game::GameAction;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Betrayer's Bargain deals 5 and exiles the lethal creature instead of
    /// burying it, and carries the sacrifice-or-pay additional cost.
    #[test]
    fn betrayers_bargain_exiles_lethal() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let def = catalog::betrayers_bargain();
        assert!(matches!(
            def.additional_cast_cost[0],
            AdditionalCastCost::SacrificeOrPay { pay: 2, .. }
        ));
        let ctx = EffectContext { targets: vec![Target::Permanent(victim)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&def.effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == victim), "lethal creature exiled, not buried");
        assert!(!g.players[1].graveyard.iter().any(|c| c.id == victim));
    }

    /// Untimely Malfunction's third mode keeps one or two creatures from blocking.
    #[test]
    fn untimely_malfunction_cant_block_mode() {
        let mut g = two_player_game();
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let modes = match &catalog::untimely_malfunction().effect {
            Effect::ChooseMode(m) => m.clone(),
            _ => panic!("not modal"),
        };
        let ctx = EffectContext { targets: vec![Target::Permanent(blocker)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&modes[2], &ctx).unwrap();
        assert!(g.computed_permanent(blocker).unwrap().keywords.contains(&Keyword::CantBlock));
    }

    /// With delirium, Omnivorous Flytrap's ETB distributes two +1/+1 counters; at
    /// six card types it doubles them on the same creatures.
    #[test]
    fn omnivorous_flytrap_delirium_counters() {
        let mut g = two_player_game();
        // Six distinct card types in the graveyard.
        g.add_card_to_graveyard(0, catalog::forest()); // Land
        g.add_card_to_graveyard(0, catalog::lightning_strike()); // Instant
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // Creature
        g.add_card_to_graveyard(0, catalog::sol_ring()); // Artifact
        g.add_card_to_graveyard(0, catalog::divination()); // Sorcery
        g.add_card_to_graveyard(0, catalog::pacifism()); // Enchantment
        assert!(g.distinct_card_types_in_graveyard(0) >= 6);
        let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let etb = catalog::omnivorous_flytrap().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(target)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&etb, &ctx).unwrap();
        // Two counters distributed onto the single target, then doubled to four.
        assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
    }

    /// Norin can't block, and his blocked-creature trigger exiles the trigger
    /// source and grants a play-from-exile window.
    #[test]
    fn norin_exiles_blocked_creature() {
        let mut g = two_player_game();
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let norin = catalog::norin_swift_survivalist();
        assert!(norin.keywords.contains(&Keyword::CantBlock));
        let effect = norin.triggered_abilities[0].effect.clone();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let ctx = EffectContext::for_trigger(ally, 0, None, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.exile.iter().any(|c| c.id == ally), "blocked creature exiled");
    }

    /// Rootwise Survivor's Survival animates a target land with three +1/+1
    /// counters into a creature, and its trigger is a tapped-gated second main.
    #[test]
    fn rootwise_survivor_survival_animates_land() {
        use crabomination::effect::{EventKind, EventScope};
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let def = catalog::rootwise_survivor();
        assert_eq!(def.triggered_abilities[0].event.kind, EventKind::StepBegins(crabomination::game::TurnStep::PostCombatMain));
        assert!(matches!(def.triggered_abilities[0].event.scope, EventScope::YourControl));
        let ctx = EffectContext { targets: vec![Target::Permanent(land)], ..EffectContext::for_trigger(land, 0, None, 0) };
        g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).unwrap();
        let l = g.computed_permanent(land).unwrap();
        assert_eq!((l.power, l.toughness), (3, 3), "0/0 Elemental + three +1/+1");
        assert!(l.card_types.contains(&crabomination::card::CardType::Creature));
    }

    /// Reluctant Role Model's Survival grants a flying counter, and its death
    /// trigger relocates the counters to another creature.
    #[test]
    fn reluctant_role_model_counters_and_relocation() {
        let mut g = two_player_game();
        let model = g.add_card_to_battlefield(0, catalog::reluctant_role_model());
        let heir = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let survival = match &catalog::reluctant_role_model().triggered_abilities[0].effect {
            Effect::ChooseMode(m) => m[0].clone(), // flying counter
            _ => panic!("not modal"),
        };
        g.resolve_effect(&survival, &EffectContext::for_trigger(model, 0, None, 0)).unwrap();
        assert!(g.computed_permanent(model).unwrap().keywords.contains(&Keyword::Flying));
        // Death trigger moves the counters onto the heir.
        let death = catalog::reluctant_role_model().triggered_abilities[1].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(heir)], ..EffectContext::for_trigger(model, 0, None, 0) };
        g.resolve_effect(&death, &ctx).unwrap();
        assert!(g.computed_permanent(heir).unwrap().keywords.contains(&Keyword::Flying),
            "flying keyword counter relocated to the heir");
        assert!(!g.computed_permanent(model).unwrap().keywords.contains(&Keyword::Flying),
            "the model's counter left it");
    }

    /// Kutzil's Flanker's second mode gains life and scries; its third exiles a
    /// target player's graveyard.
    #[test]
    fn kutzils_flanker_modes() {
        let mut g = two_player_game();
        let flanker = g.add_card_to_battlefield(0, catalog::kutzils_flanker());
        assert!(catalog::kutzils_flanker().keywords.contains(&Keyword::Flash));
        let modes = match &catalog::kutzils_flanker().triggered_abilities[0].effect {
            Effect::ChooseMode(m) => m.clone(),
            _ => panic!("not modal"),
        };
        let life0 = g.players[0].life;
        g.resolve_effect(&modes[1], &EffectContext::for_trigger(flanker, 0, None, 0)).unwrap();
        assert_eq!(g.players[0].life, life0 + 2, "gained 2 life");
        // Mode 3 exiles a target player's graveyard.
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.resolve_effect(&modes[2], &EffectContext::for_trigger(flanker, 0, Some(Target::Player(1)), 0)).unwrap();
        assert!(g.players[1].graveyard.is_empty(), "opponent graveyard exiled");
    }

    /// Stubborn Burrowfiend's saddle trigger mills two and pumps by graveyard
    /// creatures, and it fires only once per turn.
    #[test]
    fn stubborn_burrowfiend_saddle_mill_and_pump() {
        use crabomination::effect::EventKind;
        let mut g = two_player_game();
        let fiend = g.add_card_to_battlefield(0, catalog::stubborn_burrowfiend()); // 2/2
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // 2 creatures → X=2
        for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); } // mill lands, X unchanged
        let def = catalog::stubborn_burrowfiend();
        assert_eq!(def.triggered_abilities[0].event.kind, EventKind::CrewsOrSaddles);
        assert!(def.triggered_abilities[0].event.once_per_turn, "first-time-each-turn gate");
        g.resolve_effect(&def.triggered_abilities[0].effect, &EffectContext::for_trigger(fiend, 0, None, 0)).unwrap();
        let c = g.computed_permanent(fiend).unwrap();
        assert_eq!((c.power, c.toughness), (4, 4), "2/2 + X/X (X = 2 graveyard creatures)");
    }

    /// Unscrupulous Contractor's ETB sacrifice draws two and drains the target.
    #[test]
    fn unscrupulous_contractor_sacrifice_draws_and_drains() {
        let mut g = two_player_game();
        let src = g.add_card_to_battlefield(0, catalog::unscrupulous_contractor());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        let (life0, hand0) = (g.players[0].life, g.players[0].hand.len());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let etb = catalog::unscrupulous_contractor().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_trigger(src, 0, Some(Target::Player(0)), 0);
        g.resolve_effect(&etb, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
        assert_eq!(g.players[0].hand.len(), hand0 + 2, "target drew 2");
        assert_eq!(g.players[0].life, life0 - 2, "target lost 2");
    }

    /// Outlaw Stitcher makes a 2/2 Zombie Rogue that grows by two per extra spell
    /// cast this turn, and it's plottable.
    #[test]
    fn outlaw_stitcher_token_scales_with_spells() {
        let mut g = two_player_game();
        g.players[0].spells_cast_this_turn = 3; // Stitcher + 2 others → 2 extra
        assert!(catalog::outlaw_stitcher().plot_cost.is_some());
        let src = g.add_card_to_battlefield(0, catalog::outlaw_stitcher());
        let etb = catalog::outlaw_stitcher().triggered_abilities[0].effect.clone();
        g.resolve_effect(&etb, &EffectContext::for_trigger(src, 0, None, 0)).unwrap();
        let token = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Zombie Rogue").expect("token made");
        // base 2/2 + 2 counters * (3 - 1) = four counters.
        assert_eq!(token.counter_count(CounterType::PlusOnePlusOne), 4);
    }

    /// Tumbleweed Rising makes an Elemental whose power tracks your biggest
    /// creature, and it's plottable.
    #[test]
    fn tumbleweed_rising_makes_dynamic_token() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 → X = 4
        assert!(catalog::tumbleweed_rising().plot_cost.is_some());
        g.resolve_effect(&catalog::tumbleweed_rising().effect, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        let token = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Elemental").expect("token made");
        let id = token.id;
        assert_eq!(g.computed_permanent(id).unwrap().power, 4, "X/X = greatest power you control");
    }

    /// Bite Down on Crime pumps your creature and fights an enemy for its power.
    #[test]
    fn bite_down_on_crime_pumps_and_fights() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 4/2
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies to 4
        let ctx = EffectContext {
            targets: vec![Target::Permanent(mine), Target::Permanent(enemy)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        g.resolve_effect(&catalog::bite_down_on_crime().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(mine).unwrap().power, 4, "+2/+0");
        assert!(g.battlefield_find(enemy).is_none(), "took 4 and died");
    }

    /// Trial of Agony burns one creature and locks the other out of blocking.
    #[test]
    fn trial_of_agony_burns_and_locks() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies to 5
        let b = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 survives
        let ctx = EffectContext {
            targets: vec![Target::Permanent(a), Target::Permanent(b)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        g.resolve_effect(&catalog::trial_of_agony().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(a).is_none(), "took 5 and died");
        assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::CantBlock));
    }

    /// Getaway Glamer's first mode blinks a creature (exiles it now).
    #[test]
    fn getaway_glamer_blink_mode() {
        let mut g = two_player_game();
        let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let blink = match &catalog::getaway_glamer().effect {
            Effect::Spree { modes } => modes[0].effect.clone(),
            _ => panic!("not spree"),
        };
        let ctx = EffectContext { targets: vec![Target::Permanent(c)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&blink, &ctx).unwrap();
        assert!(g.battlefield_find(c).is_none(), "creature exiled by the blink");
    }

    /// Come Back Wrong destroys a creature, reanimates it under your control, and
    /// schedules a sacrifice at your next end step.
    #[test]
    fn come_back_wrong_steals_the_corpse() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(victim)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::come_back_wrong().effect, &ctx).unwrap();
        drain_stack(&mut g);
        let c = g.battlefield_find(victim).expect("reanimated onto the battlefield");
        assert_eq!(c.controller, 0, "under your control now");
    }

    /// Valgavoth's Onslaught (X=2) manifests two 2/2s, each with two +1/+1
    /// counters (making them 4/4 face-down creatures).
    #[test]
    fn valgavoths_onslaught_manifests_and_counters() {
        let mut g = two_player_game();
        for _ in 0..5 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let ctx = EffectContext::for_spell(0, None, 0, 2); // X = 2
        g.resolve_effect(&catalog::valgavoths_onslaught().effect, &ctx).unwrap();
        let facedown: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).collect();
        assert_eq!(facedown.len(), 2, "two creatures manifested");
        for c in facedown {
            assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2, "each got X=2 counters");
        }
    }

    /// Altanak's channel ability returns a target land card from the graveyard to
    /// the battlefield tapped, and it carries the opponent-target draw trigger.
    #[test]
    fn altanak_channel_returns_land_tapped() {
        let mut g = two_player_game();
        let land = g.add_card_to_graveyard(0, catalog::forest());
        let def = catalog::altanak_the_thrice_called();
        assert!(def.keywords.contains(&Keyword::Trample));
        assert_eq!(def.triggered_abilities[0].event.kind, crabomination::effect::EventKind::BecameTarget);
        let effect = def.activated_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(land)], ..EffectContext::for_ability(land, 0, None) };
        g.resolve_effect(&effect, &ctx).unwrap();
        let l = g.battlefield_find(land).expect("land returned to battlefield");
        assert!(l.tapped, "enters tapped");
    }

    /// Behind the Mask makes the target 4/3 with no evidence, 1/1 with evidence.
    #[test]
    fn behind_the_mask_evidence_flips_pt() {
        // No evidence → 4/3.
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::behind_the_mask());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Behind the Mask");
        drain_stack(&mut g);
        let c = g.computed_permanent(target).unwrap();
        assert_eq!((c.power, c.toughness), (4, 3), "4/3 without evidence");

        // Evidence collected → 1/1.
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); } // MV 6
        let spell = g.add_card_to_hand(0, catalog::behind_the_mask());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Behind the Mask with evidence");
        drain_stack(&mut g);
        let c = g.computed_permanent(target).unwrap();
        assert_eq!((c.power, c.toughness), (1, 1), "1/1 with evidence");
    }

    /// Analyze the Pollen fetches a basic land normally, but with evidence its
    /// `If` branch widens the search filter to creature-or-land — so a Grizzly
    /// Bears in the library becomes a legal pick.
    #[test]
    fn analyze_the_pollen_evidence_widens_search() {
        use crabomination::effect::Effect;
        // Evidence collected → creature is a legal search pick.
        let mut g = two_player_game();
        let bear = g.add_card_to_library(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
        let mut ctx = EffectContext::for_spell(0, None, 0, 0);
        ctx.cast_collected_evidence = true;
        g.resolve_effect(&catalog::analyze_the_pollen().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature fetched with evidence");

        // Without evidence the same creature pick is illegal (basic-land only).
        let mut g = two_player_game();
        let bear = g.add_card_to_library(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear))]));
        let else_ = match catalog::analyze_the_pollen().effect {
            Effect::If { else_, .. } => *else_,
            _ => panic!("not an If"),
        };
        g.resolve_effect(&else_, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        drain_stack(&mut g);
        assert!(!g.players[0].hand.iter().any(|c| c.id == bear), "creature is not a basic land");
    }

    /// Paranormal Analyst returns the card milled by manifest dread to hand.
    #[test]
    fn paranormal_analyst_returns_milled_card() {
        use crabomination::effect::{Effect, PlayerRef};
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::paranormal_analyst());
        // Two cards on top: the analyst manifests one, mills the other, and its
        // trigger returns that milled card to hand.
        let manifested = g.add_card_to_library(0, catalog::grizzly_bears());
        let milled = g.add_card_to_library(0, catalog::forest());
        // Library is a stack — ensure `manifested` is on top so `forest` is the mill.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![manifested])]));
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let events = g.resolve_effect(&Effect::ManifestDread { who: PlayerRef::You }, &ctx).unwrap();
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == milled),
            "milled card returned to hand by Paranormal Analyst");
        assert!(g.battlefield.iter().any(|c| c.id == manifested && c.face_down),
            "the chosen card is manifested face down");
    }

    /// Oblivious Bookworm draws then discards when you had no face-down activity,
    /// but keeps the drawn card when a permanent entered face down this turn.
    #[test]
    fn oblivious_bookworm_discard_unless_face_down_activity() {
        // No face-down activity → draw then discard (net hand unchanged).
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        let effect = catalog::oblivious_bookworm().triggered_abilities[0].effect.clone();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let before = g.players[0].hand.len();
        g.resolve_effect(&effect, &EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0)).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before, "drew one, discarded one");

        // A permanent entered face down this turn → no discard (net +1 hand).
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].face_down_activity_this_turn = true;
        let effect = catalog::oblivious_bookworm().triggered_abilities[0].effect.clone();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let before = g.players[0].hand.len();
        g.resolve_effect(&effect, &EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0)).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 1, "kept the drawn card");
    }

    /// Monstrous Emergence deals damage equal to the chosen creature's power (its
    /// choose-a-creature additional cost picks the highest-power creature you
    /// control).
    #[test]
    fn monstrous_emergence_deals_chosen_power() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let big = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
        let _ = big;
        let victim = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        let spell = g.add_card_to_hand(0, catalog::monstrous_emergence());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Monstrous Emergence");
        drain_stack(&mut g);
        // Highest-power creature is the 3/3 Hill Giant → 3 damage kills the 3/3.
        assert!(g.battlefield_find(victim).is_none(), "3 damage (chosen creature's power) killed the 3/3");
    }

    /// Leyline of Hope may begin in play, boosts life gain by 1, and anthems your
    /// team once you're 7+ life above your starting total.
    #[test]
    fn leyline_of_hope_lifegain_and_anthem() {
        use crabomination::effect::OpeningHandEffect;
        let def = catalog::leyline_of_hope();
        assert!(matches!(def.opening_hand, Some(OpeningHandEffect::StartInPlay { .. })));
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::leyline_of_hope());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        // Life gain of 3 becomes 4 (bonus +1).
        let start = g.players[0].life;
        g.adjust_life(0, 3);
        assert_eq!(g.players[0].life, start + 4, "life gain boosted by 1");
        // Below the +7 threshold the team isn't pumped.
        assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "no anthem yet");
        // Push to 7+ above starting → +2/+2 anthem.
        g.players[0].life = g.players[0].starting_life + 7;
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (4, 4), "anthem online at 7+ above starting");
    }

    /// Creeping Peeper taps for {U} that only casts enchantment spells.
    #[test]
    fn creeping_peeper_enchantment_only_mana() {
        use crabomination::effect::{Effect, ManaPayload};
        use crabomination::mana::SpendRestriction;
        let def = catalog::creeping_peeper();
        // The ability adds enchantment-restricted blue mana.
        match &def.activated_abilities[0].effect {
            Effect::AddMana { pool: ManaPayload::Restricted(_, r), .. } => {
                assert_eq!(*r, SpendRestriction::EnchantmentSpell);
            }
            _ => panic!("not enchantment-restricted mana"),
        }
        // The restriction admits an enchantment spell but not a creature spell.
        assert!(SpendRestriction::EnchantmentSpell.allows(&catalog::pacifism().spell_kind()));
        assert!(!SpendRestriction::EnchantmentSpell.allows(&catalog::grizzly_bears().spell_kind()));
    }

    /// Fear of Burning Alive burns each opponent for 4 on ETB, and with delirium its
    /// noncombat-damage trigger copies the damage onto an opponent's creature.
    #[test]
    fn fear_of_burning_alive_etb_and_delirium_copy() {
        use crabomination::effect::{EventKind, EventScope};
        let mut g = two_player_game();
        let etb = catalog::fear_of_burning_alive().triggered_abilities[0].effect.clone();
        let before = g.players[1].life;
        g.resolve_effect(&etb, &EffectContext::for_trigger(crabomination::card::CardId(0), 0, None, 0)).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, before - 4, "each opponent burned for 4");

        // Delirium trigger is a noncombat-damage listener; it copies the amount.
        let def = catalog::fear_of_burning_alive();
        assert_eq!(def.triggered_abilities[1].event.kind, EventKind::PlayerDealtNoncombatDamage);
        assert!(matches!(def.triggered_abilities[1].event.scope, EventScope::OpponentControl));
        let mut g = two_player_game();
        let src = g.add_card_to_battlefield(0, catalog::fear_of_burning_alive());
        let victim = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        let copy = def.triggered_abilities[1].effect.clone();
        let mut ctx = EffectContext { targets: vec![Target::Permanent(victim)], ..EffectContext::for_trigger(src, 0, None, 0) };
        ctx.event_amount = 5; // an earlier source dealt 5 noncombat
        g.resolve_effect(&copy, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "5 copied damage killed the 3/3");
    }

    /// Mudflat Village's sac ability returns a Rat card from the graveyard to hand;
    /// its {B} ability is creature-spell restricted.
    #[test]
    fn mudflat_village_returns_kindred_and_restricts_mana() {
        use crabomination::effect::{Effect, ManaPayload};
        use crabomination::mana::SpendRestriction;
        let def = catalog::mudflat_village();
        // The {B} ability is creature-only.
        match &def.activated_abilities[1].effect {
            Effect::AddMana { pool: ManaPayload::Restricted(_, r), .. } => {
                assert_eq!(*r, SpendRestriction::CreatureOnly);
            }
            _ => panic!("not creature-restricted mana"),
        }
        // Sac ability (tap + sacrifice + {1}{B}) returns a matching Rat to hand.
        let ab = &def.activated_abilities[2];
        assert!(ab.tap_cost && ab.sac_cost, "tap + sacrifice cost");
        assert_eq!(ab.mana_cost.cmc(), 2, "one-generic-plus-black activation");
        let mut g = two_player_game();
        let rat = g.add_card_to_graveyard(0, catalog::typhoid_rats());
        let ctx = EffectContext { targets: vec![Target::Permanent(rat)], ..EffectContext::for_ability(rat, 0, None) };
        g.resolve_effect(&ab.effect, &ctx).unwrap();
        assert!(g.players[0].hand.iter().any(|c| c.id == rat), "Rat returned to hand");
    }

    /// Oakhollow Village's {G} ability counters only your kindred creatures that
    /// entered this turn.
    #[test]
    fn oakhollow_village_counters_kindred_entered_this_turn() {
        let mut g = two_player_game();
        g.turn_number = 5;
        // A Frog that entered this turn (kindred), a Rat this turn (not kindred),
        // and a Frog that entered earlier.
        let fresh_frog = g.add_card_to_battlefield(0, catalog::spore_frog());
        let rat = g.add_card_to_battlefield(0, catalog::typhoid_rats());
        let old_frog = g.add_card_to_battlefield(0, catalog::spore_frog());
        g.battlefield_find_mut(fresh_frog).unwrap().entered_turn = Some(5);
        g.battlefield_find_mut(rat).unwrap().entered_turn = Some(5);
        g.battlefield_find_mut(old_frog).unwrap().entered_turn = Some(1);
        let ab = catalog::oakhollow_village().activated_abilities[2].effect.clone();
        g.resolve_effect(&ab, &EffectContext::for_ability(crabomination::card::CardId(0), 0, None)).unwrap();
        assert_eq!(g.battlefield_find(fresh_frog).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
            "kindred creature that entered this turn gets a counter");
        assert_eq!(g.battlefield_find(rat).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
            "non-kindred creature untouched");
        assert_eq!(g.battlefield_find(old_frog).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
            "kindred creature that entered earlier untouched");
    }

    /// Lupinflower Village's sac ability digs six deep for a Rabbit and bottoms the
    /// rest; its {W} ability is creature-restricted.
    #[test]
    fn lupinflower_village_digs_for_kindred() {
        use crabomination::effect::{Effect, ManaPayload};
        use crabomination::mana::SpendRestriction;
        let def = catalog::lupinflower_village();
        match &def.activated_abilities[1].effect {
            Effect::AddMana { pool: ManaPayload::Restricted(_, r), .. } => {
                assert_eq!(*r, SpendRestriction::CreatureOnly);
            }
            _ => panic!("not creature-restricted mana"),
        }
        // Sac ability digs six deep for a kindred card, bottoming the rest at random.
        let ab = &def.activated_abilities[2];
        assert!(ab.tap_cost && ab.sac_cost);
        match &ab.effect {
            Effect::LookPickToHand(lp)
                if lp.pick_filter.is_some() && lp.rest_bottom_random && lp.optional =>
            {
                assert!(matches!(lp.count, crabomination::effect::Value::Const(6)));
            }
            _ => panic!("not a six-deep kindred dig"),
        }
    }

    /// Lilypad Village's surveil ability is gated on controlling a kindred creature
    /// that entered this turn.
    #[test]
    fn lilypad_village_surveil_gate() {
        use crabomination::effect::Effect;
        let def = catalog::lilypad_village();
        let ab = &def.activated_abilities[2];
        assert!(matches!(ab.effect, Effect::Surveil { .. }));
        assert!(ab.condition.is_some(), "gated on a kindred entered-this-turn");
        // The gate is unmet with no kindred on the board.
        let mut g = two_player_game();
        g.turn_number = 3;
        let ctx = EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
        assert!(!g.evaluate_predicate(ab.condition.as_ref().unwrap(), &ctx), "no kindred → gate closed");
        // With a Frog that entered this turn, the gate opens.
        let frog = g.add_card_to_battlefield(0, catalog::spore_frog());
        g.battlefield_find_mut(frog).unwrap().entered_turn = Some(3);
        assert!(g.evaluate_predicate(ab.condition.as_ref().unwrap(), &ctx), "kindred present → gate open");
    }

    /// Rockface Village's sorcery-speed ability pumps and hastes a kindred creature.
    #[test]
    fn rockface_village_pumps_and_hastes_kindred() {
        let mut g = two_player_game();
        let lizard = g.add_card_to_battlefield(0, catalog::viashino_pyromancer()); // Lizard 2/1
        let ab = catalog::rockface_village().activated_abilities[2].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(lizard)], ..EffectContext::for_ability(crabomination::card::CardId(0), 0, None) };
        g.resolve_effect(&ab, &ctx).unwrap();
        let c = g.computed_permanent(lizard).unwrap();
        assert_eq!(c.power, 3, "+1/+0 applied");
        assert!(c.keywords.contains(&Keyword::Haste), "gained haste");
    }

    /// Whiskervale Forerunner's Valiant trigger digs five deep for a small creature
    /// and deploys it; the trigger is once-per-turn on becoming your target.
    #[test]
    fn whiskervale_forerunner_valiant_dig() {
        use crabomination::effect::{Effect, EventKind};
        let def = catalog::whiskervale_forerunner();
        let t = &def.triggered_abilities[0];
        assert_eq!(t.event.kind, EventKind::BecameTarget);
        assert!(t.event.once_per_turn, "first time each turn");
        match &t.effect {
            Effect::LookPickToHand(lp)
                if lp.pick_filter.is_some() && lp.to_battlefield && lp.rest_bottom_random =>
            {
                assert!(matches!(lp.count, crabomination::effect::Value::Const(5)));
            }
            _ => panic!("not a five-deep creature dig"),
        }
        // The dig deploys a small creature from the top of the library.
        let mut g = two_player_game();
        let src = g.add_card_to_battlefield(0, catalog::whiskervale_forerunner());
        let small = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 creature
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![small])]));
        g.resolve_effect(&t.effect, &EffectContext::for_trigger(src, 0, None, 0)).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == small), "small creature deployed");
    }

    /// Hollow Marauder costs less per graveyard creature, and its ETB draws only
    /// when the opponent's discard was cheap (MV ≤ 3).
    #[test]
    fn hollow_marauder_cost_and_conditional_draw() {
        use crabomination::card::SelectionRequirement as R;
        let def = catalog::hollow_marauder();
        assert_eq!(def.affinity_graveyard_filter, Some(R::Creature));
        assert!(def.keywords.contains(&Keyword::Flying));
        // Cheap discard → draw.
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::grizzly_bears()); // MV 2
        let src = g.add_card_to_battlefield(0, catalog::hollow_marauder());
        let etb = def.triggered_abilities[0].effect.clone();
        let before = g.players[0].hand.len();
        g.resolve_effect(&etb, &EffectContext::for_trigger(src, 0, None, 0)).unwrap();
        assert_eq!(g.players[0].hand.len(), before + 1, "drew after a cheap discard");
        // Expensive discard → no draw.
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::serra_angel()); // MV 5
        let src = g.add_card_to_battlefield(0, catalog::hollow_marauder());
        let before = g.players[0].hand.len();
        g.resolve_effect(&def.triggered_abilities[0].effect.clone(), &EffectContext::for_trigger(src, 0, None, 0)).unwrap();
        assert_eq!(g.players[0].hand.len(), before, "no draw after an expensive discard");
    }

    /// Feed the Cycle forages (exiling three graveyard cards) to pay its additional
    /// cost, then destroys a creature; with no forage material the pay is folded.
    #[test]
    fn feed_the_cycle_forage_or_pay() {
        use crabomination::card::AdditionalCastCost;
        let def = catalog::feed_the_cycle();
        assert!(matches!(def.additional_cast_cost[0], AdditionalCastCost::ForageOrPay { pay: 1 }));
        // With three graveyard cards, cast forages (no extra mana) and destroys.
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::feed_the_cycle());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1); // exactly {1}{B}
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("forage pays the additional cost");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "creature destroyed");
        assert_eq!(g.exile.iter().filter(|c| c.owner == 0).count(), 3, "three cards foraged");

        // With no forage material the pay ({1}) is folded — {1}{B} is one short.
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::feed_the_cycle());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        assert!(g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        }).is_err(), "no forage material means the folded generic makes it unaffordable");
    }

    /// Freestrider Commando: a hardcast (mana spent) enters as a vanilla 3/3; a
    /// free/plotted cast (no mana spent) enters with two +1/+1 counters.
    #[test]
    fn freestrider_commando_counters_only_when_free() {
        // Hardcast for {2}{G} → no counters.
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let spell = g.add_card_to_hand(0, catalog::freestrider_commando());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("hardcast");
        drain_stack(&mut g);
        let c = g.battlefield_find(spell).unwrap();
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 0, "hardcast is a vanilla 3/3");

        // The ETB gates the counters on "no mana spent" (Not CastSpellManaSpentAtLeast
        // 1) — so a free/plotted cast (0 spent) or a non-cast entry qualifies. That
        // hardcast above proves the gate reads the cast's mana spend at ETB time.
        let def = catalog::freestrider_commando();
        assert!(def.plot_cost.is_some(), "has a Plot cost");
        match &def.triggered_abilities[0].effect {
            Effect::If { cond: Predicate::Not(inner), .. } => {
                assert!(matches!(**inner, Predicate::CastSpellManaSpentAtLeast(1)));
            }
            _ => panic!("not a no-mana-spent gate"),
        }
    }

    /// Crimestopper Sprite's ETB taps a creature; with collect-evidence paid, it
    /// also stuns it. The self-ETB trigger reads the cast's collect-evidence flag.
    #[test]
    fn crimestopper_sprite_taps_and_conditionally_stuns() {
        use crabomination::card::CardId;
        // Evidence collected → tap + stun.
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let src = g.add_card_to_battlefield(0, catalog::crimestopper_sprite());
        g.battlefield_find_mut(src).unwrap().cast_collected_evidence = true;
        let etb = catalog::crimestopper_sprite().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(victim)], ..EffectContext::for_trigger(src, 0, None, 0) };
        // Stamp the flag on the trigger ctx as the engine's trigger driver does.
        let mut ctx = ctx; ctx.cast_collected_evidence = true;
        g.resolve_effect(&etb, &ctx).unwrap();
        let c = g.battlefield_find(victim).unwrap();
        assert!(c.tapped, "tapped by the ETB");
        assert_eq!(c.counter_count(CounterType::Stun), 1, "stunned when evidence was collected");

        // No evidence → tap only.
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let src = g.add_card_to_battlefield(0, catalog::crimestopper_sprite());
        let _ = src;
        let etb = catalog::crimestopper_sprite().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(victim)], ..EffectContext::for_trigger(CardId(0), 0, None, 0) };
        g.resolve_effect(&etb, &ctx).unwrap();
        let c = g.battlefield_find(victim).unwrap();
        assert!(c.tapped, "tapped");
        assert_eq!(c.counter_count(CounterType::Stun), 0, "no stun without evidence");
    }
}

mod recent240 {
    use crabomination::card::AdditionalCastCost;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, two_player_game};

    /// Fear of Abduction exiles an opponent's creature until it leaves, then hands
    /// it back to its owner — and it carries the exile-a-creature additional cost.
    #[test]
    fn fear_of_abduction_exiles_until_it_leaves() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let fear = g.add_card_to_battlefield(0, catalog::fear_of_abduction());
        assert!(matches!(
            catalog::fear_of_abduction().additional_cast_cost[0],
            AdditionalCastCost::ExilePermanent { count: 1, .. }
        ));
        g.fire_self_etb_triggers(fear, 0);
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == victim), "opponent's creature exiled");
        g.remove_from_battlefield_to_graveyard_raw(fear);
        drain_stack(&mut g);
        assert!(g.players[1].hand.iter().any(|c| c.id == victim), "returns to owner's hand on leave");
    }

    /// Say Its Name mills three, then returns a creature (or land) from the
    /// graveyard to hand.
    #[test]
    fn say_its_name_mills_then_returns() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::forest());
        }
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&catalog::say_its_name().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(
            g.players[0].graveyard.iter().filter(|c| c.definition.name == "Forest").count(),
            3,
            "milled three cards"
        );
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned to hand");
    }

    /// Veteran Survivor gains +3/+3 and hexproof once three cards are exiled with
    /// it via its Survival ability.
    #[test]
    fn veteran_survivor_buffs_at_three_exiled() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let vet = g.add_card_to_battlefield(0, catalog::veteran_survivor());
        // Baseline: 2/1, no hexproof.
        let c = g.computed_permanent(vet).unwrap();
        assert_eq!((c.power, c.toughness), (2, 1));
        assert!(!c.keywords.contains(&Keyword::Hexproof));
        // Exile three cards stamped with the survivor as their source.
        for _ in 0..3 {
            let card = g.add_card_to_exile(1, catalog::grizzly_bears());
            g.exile.iter_mut().find(|c| c.id == card).unwrap().exiled_with = Some(vet);
        }
        let c = g.computed_permanent(vet).unwrap();
        assert_eq!((c.power, c.toughness), (5, 4), "+3/+3 at three exiled");
        assert!(c.keywords.contains(&Keyword::Hexproof), "hexproof at three exiled");
    }

    /// Coordinated Clobbering taps both chosen creatures and makes each deal its
    /// power to the opponent's creature.
    #[test]
    fn coordinated_clobbering_two_creatures() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let opp = g.add_card_to_battlefield(1, catalog::avenger_of_zendikar()); // 5/5
        // Slots: 0 = a, 1 = opp, 2 = b.
        let ctx = EffectContext {
            targets: vec![Target::Permanent(a), Target::Permanent(opp), Target::Permanent(b)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        let body = match &catalog::coordinated_clobbering().effect {
            Effect::OptionalTargets { body, .. } => (**body).clone(),
            _ => panic!("not OptionalTargets"),
        };
        g.resolve_effect(&body, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(a).unwrap().tapped, "first creature tapped");
        assert!(g.battlefield_find(b).unwrap().tapped, "second creature tapped");
        assert_eq!(g.battlefield_find(opp).unwrap().damage, 4, "2 + 2 damage dealt");
    }

    /// Waltz of Rage's chosen creature deals its power to every other creature.
    #[test]
    fn waltz_of_rage_radiates() {
        let mut g = two_player_game();
        let hero = g.add_card_to_battlefield(0, catalog::avenger_of_zendikar()); // 5/5
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let ctx = EffectContext {
            targets: vec![Target::Permanent(hero)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        g.resolve_effect(&catalog::waltz_of_rage().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(hero).is_some(), "source is not hit by itself");
        assert!(g.battlefield_find(ally).is_none(), "friendly creature took 5 and died");
        assert!(g.battlefield_find(enemy).is_none(), "enemy creature took 5 and died");
    }
}

mod recent241 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::{EffectContext, EntityRef};
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::{drain_stack, two_player_game, GameEvent};

    fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
        g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
    }

    /// Sanitation Automaton's ETB surveil bins the top card to the graveyard.
    #[test]
    fn sanitation_automaton_surveils() {
        let mut g = two_player_game();
        let top = g.add_card_to_library(0, catalog::forest());
        let auto = g.add_card_to_battlefield(0, catalog::sanitation_automaton());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
            kept_top: vec![],
            bottom: vec![top],
        }]));
        g.fire_self_etb_triggers(auto, 0);
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "top card surveilled to graveyard");
    }

    /// Loxodon Eavesdropper investigates on ETB and grows on the second draw.
    #[test]
    fn loxodon_eavesdropper_investigates_and_grows() {
        let mut g = two_player_game();
        let lox = g.add_card_to_battlefield(0, catalog::loxodon_eavesdropper());
        g.fire_self_etb_triggers(lox, 0);
        drain_stack(&mut g);
        assert_eq!(clues(&g, 0), 1, "ETB investigate made a Clue");
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::forest());
        }
        let mut evs = vec![];
        g.draw_one(0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        let mut evs = vec![];
        g.draw_one(0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let c = g.computed_permanent(lox).unwrap();
        assert_eq!((c.power, c.toughness), (4, 4), "+1/+1 on the second draw");
        assert!(c.keywords.contains(&Keyword::Vigilance), "gains vigilance on the second draw");
    }

    /// Jaded Analyst sheds defender and gains vigilance on the second draw.
    #[test]
    fn jaded_analyst_loses_defender_on_second_draw() {
        let mut g = two_player_game();
        let jaded = g.add_card_to_battlefield(0, catalog::jaded_analyst());
        assert!(g.computed_permanent(jaded).unwrap().keywords.contains(&Keyword::Defender));
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::island());
        }
        for _ in 0..2 {
            let mut evs = vec![];
            g.draw_one(0, &mut evs);
            g.dispatch_triggers_for_events(&evs);
        }
        drain_stack(&mut g);
        let c = g.computed_permanent(jaded).unwrap();
        assert!(!c.keywords.contains(&Keyword::Defender), "defender removed");
        assert!(c.keywords.contains(&Keyword::Vigilance), "vigilance gained");
    }

    /// Innocent Bystander investigates when dealt three or more damage.
    #[test]
    fn innocent_bystander_investigates_on_big_hit() {
        let mut g = two_player_game();
        let bystander = g.add_card_to_battlefield(0, catalog::innocent_bystander());
        let mut evs = vec![];
        g.deal_damage_to_from(EntityRef::Permanent(bystander), 3, None, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(clues(&g, 0), 1, "3 damage triggered investigate");
    }

    /// Rot Farm Mortipede pumps when a creature card leaves the graveyard.
    #[test]
    fn rot_farm_mortipede_pumps_on_graveyard_departure() {
        let mut g = two_player_game();
        let mort = g.add_card_to_battlefield(0, catalog::rot_farm_mortipede());
        let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.dispatch_triggers_for_events(&[GameEvent::CardLeftGraveyard { player: 0, card_id: gy }]);
        drain_stack(&mut g);
        let c = g.computed_permanent(mort).unwrap();
        assert_eq!(c.power, 4, "+1/+0 until end of turn");
        assert!(c.keywords.contains(&Keyword::Menace) && c.keywords.contains(&Keyword::Lifelink));
    }

    /// Dog Walker mints two tapped Dog tokens when turned face up.
    #[test]
    fn dog_walker_makes_dogs_face_up() {
        let mut g = two_player_game();
        let walker = g.add_card_to_battlefield(0, catalog::dog_walker());
        let effect = catalog::dog_walker().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(walker, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        let dogs: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Dog").collect();
        assert_eq!(dogs.len(), 2, "two Dog tokens");
        assert!(dogs.iter().all(|d| d.tapped), "tokens enter tapped");
    }

    /// Forum Familiar bounces another permanent you control and grows when turned
    /// face up.
    #[test]
    fn forum_familiar_bounces_and_grows() {
        let mut g = two_player_game();
        let fam = g.add_card_to_battlefield(0, catalog::forum_familiar());
        let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::forum_familiar().triggered_abilities[0].effect.clone();
        let ctx = EffectContext {
            targets: vec![Target::Permanent(other)],
            ..EffectContext::for_ability(fam, 0, None)
        };
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == other), "other permanent returned to hand");
        // Forum Familiar is 1/1; the +1/+1 counter makes it 2/2.
        let c = g.computed_permanent(fam).unwrap();
        assert_eq!((c.power, c.toughness), (2, 2), "gained a +1/+1 counter");
    }

    /// Sanguine Savior grants lifelink to another creature when turned face up.
    #[test]
    fn sanguine_savior_grants_lifelink() {
        let mut g = two_player_game();
        let savior = g.add_card_to_battlefield(0, catalog::sanguine_savior());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::sanguine_savior().triggered_abilities[0].effect.clone();
        let ctx = EffectContext {
            targets: vec![Target::Permanent(ally)],
            ..EffectContext::for_ability(savior, 0, None)
        };
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Lifelink));
    }

    /// Sample Collector collects evidence on attack and counters a creature.
    #[test]
    fn sample_collector_collects_and_counters() {
        let mut g = two_player_game();
        let sample = g.add_card_to_battlefield(0, catalog::sample_collector());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Graveyard fuel: two 2-MV cards (total 4 ≥ 3) to collect evidence 3.
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        // The collect is a "may" — accept it; the counter then auto-targets a creature.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let base: i32 = g.computed_permanent(sample).unwrap().power + g.computed_permanent(ally).unwrap().power;
        let effect = catalog::sample_collector().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(sample, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.exile.len(), 2, "collected evidence exiled graveyard cards");
        let after: i32 = g.computed_permanent(sample).unwrap().power + g.computed_permanent(ally).unwrap().power;
        assert_eq!(after, base + 1, "a +1/+1 counter landed on a creature you control");
    }

    /// Drag the Canal makes a Detective and, if a creature died, gains life +
    /// investigates.
    #[test]
    fn drag_the_canal_death_bonus() {
        let mut g = two_player_game();
        // Register a creature death this turn.
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(victim).unwrap().damage = 5;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        let life = g.players[0].life;
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&catalog::drag_the_canal().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Detective" && c.controller == 0),
            "Detective token created"
        );
        assert_eq!(g.players[0].life, life + 2, "gained 2 (a creature died this turn)");
        assert_eq!(clues(&g, 0), 1, "investigated");
    }

    /// Harried Dronesmith makes a hasty Thopter at the beginning of combat.
    #[test]
    fn harried_dronesmith_makes_hasty_thopter() {
        let mut g = two_player_game();
        let smith = g.add_card_to_battlefield(0, catalog::harried_dronesmith());
        let effect = catalog::harried_dronesmith().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(smith, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        let thopter = g.battlefield.iter().find(|c| c.definition.name == "Thopter").expect("thopter minted");
        assert!(thopter.definition.keywords.contains(&Keyword::Haste), "thopter has haste");
    }

    /// Vengeful Tracker pings an opponent who sacrifices an artifact.
    #[test]
    fn vengeful_tracker_pings_on_opponent_sacrifice() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::vengeful_tracker());
        let art = g.add_card_to_battlefield(1, catalog::sol_ring());
        let life = g.players[1].life;
        g.dispatch_triggers_for_events(&[GameEvent::PermanentSacrificed { card_id: art, who: 1 }]);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2, "2 damage to the sacrificing opponent");
    }

    /// Essence of Antiquity untaps your team and grants hexproof when turned up.
    #[test]
    fn essence_of_antiquity_protects_team() {
        let mut g = two_player_game();
        let essence = g.add_card_to_battlefield(0, catalog::essence_of_antiquity());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(ally).unwrap().tapped = true;
        let effect = catalog::essence_of_antiquity().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(essence, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield_find(ally).unwrap().tapped, "team untapped");
        assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Hexproof), "team hexproof");
    }

    /// Meddling Youths' investigate trigger is gated on attacking with 3+ creatures.
    #[test]
    fn meddling_youths_gated_on_three_attackers() {
        use crabomination::effect::{EventKind, Predicate};
        let def = catalog::meddling_youths();
        assert!(def.keywords.contains(&Keyword::Haste));
        let ta = &def.triggered_abilities[0];
        assert_eq!(ta.event.kind, EventKind::YouAttack);
        assert!(matches!(
            ta.event.filter,
            Some(Predicate::AttackedWithCountAtLeast { at_least: 3, .. })
        ));
    }

    /// Gleaming Geardrake investigates on ETB.
    #[test]
    fn gleaming_geardrake_investigates() {
        let mut g = two_player_game();
        let drake = g.add_card_to_battlefield(0, catalog::gleaming_geardrake());
        g.fire_self_etb_triggers(drake, 0);
        drain_stack(&mut g);
        assert_eq!(clues(&g, 0), 1, "ETB investigate");
    }

    /// Private Eye is a Detective lord.
    #[test]
    fn private_eye_boosts_other_detectives() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::private_eye());
        let other = g.add_card_to_battlefield(0, catalog::loxodon_eavesdropper()); // 3/3 Detective
        let c = g.computed_permanent(other).unwrap();
        assert_eq!((c.power, c.toughness), (4, 4), "other Detective gets +1/+1");
    }

    /// Gadget Technician makes a Thopter when it enters.
    #[test]
    fn gadget_technician_makes_thopter() {
        let mut g = two_player_game();
        let tech = g.add_card_to_battlefield(0, catalog::gadget_technician());
        g.fire_self_etb_triggers(tech, 0);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count(),
            1,
            "one Thopter token"
        );
    }

    /// CR 701.60 — a suspected creature has menace and can't block.
    #[test]
    fn cr_701_60_suspected_creature_has_menace_and_cant_block() {
        use crabomination::effect::{Effect, Selector};
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ctx = EffectContext {
            targets: vec![Target::Permanent(bear)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        g.resolve_effect(&Effect::Suspect { what: Selector::Target(0) }, &ctx).unwrap();
        let c = g.computed_permanent(bear).unwrap();
        assert!(c.keywords.contains(&Keyword::Menace), "suspected -> menace");
        assert!(c.keywords.contains(&Keyword::CantBlock), "suspected -> can't block");
    }

    /// CR 701.13 — an investigated Clue sacrifices for a card.
    #[test]
    fn cr_701_13_clue_sacrifices_to_draw() {
        use crabomination::game::GameAction;
        use crabomination::mana::Color;
        let mut g = two_player_game();
        let lox = g.add_card_to_battlefield(0, catalog::loxodon_eavesdropper());
        g.add_card_to_library(0, catalog::forest());
        g.fire_self_etb_triggers(lox, 0);
        drain_stack(&mut g);
        let clue = g.battlefield.iter().find(|c| c.definition.name == "Clue").unwrap().id;
        let hand_before = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Green, 2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: clue,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None, mode: None,
        })
        .expect("sacrifice the Clue to draw");
        drain_stack(&mut g);
        assert!(g.battlefield_find(clue).is_none(), "Clue sacrificed");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    }

    /// Mistway Spy, once turned face up, investigates whenever a creature you
    /// control deals combat damage to a player this turn.
    #[test]
    fn mistway_spy_investigates_on_combat_damage() {
        let mut g = two_player_game();
        let spy = g.add_card_to_battlefield(0, catalog::mistway_spy());
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::mistway_spy().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(spy, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        // A creature you control deals combat damage to player 1.
        g.fire_combat_damage_to_player_triggers(attacker, 1, 2);
        drain_stack(&mut g);
        assert_eq!(clues(&g, 0), 1, "investigated on the combat damage");
    }

    /// Glint Weaver distributes three +1/+1 counters and gains life for greatest
    /// toughness.
    #[test]
    fn glint_weaver_counters_and_lifegain() {
        let mut g = two_player_game();
        let weaver = g.add_card_to_battlefield(0, catalog::glint_weaver()); // 3/3
        let big = g.add_card_to_battlefield(0, catalog::avenger_of_zendikar()); // 5/5
        let effect = catalog::glint_weaver().triggered_abilities[0].effect.clone();
        let ctx = EffectContext {
            targets: vec![Target::Permanent(big)],
            ..EffectContext::for_ability(weaver, 0, None)
        };
        let life = g.players[0].life;
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        // All three counters land on the sole target (5/5 -> 8/8), greatest toughness
        // among the OTHER creatures (Avenger) is then 8.
        assert_eq!(g.computed_permanent(big).unwrap().toughness, 8, "three counters distributed");
        assert_eq!(g.players[0].life, life + 8, "gained life = greatest toughness");
    }

    /// Exit Specialist can't be blocked by big creatures and bounces one when
    /// turned face up.
    #[test]
    fn exit_specialist_evasion_and_bounce() {
        let mut g = two_player_game();
        let exit = g.add_card_to_battlefield(0, catalog::exit_specialist());
        assert!(g
            .computed_permanent(exit)
            .unwrap()
            .keywords
            .contains(&Keyword::CantBeBlockedByPowerAtLeast(3)));
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let effect = catalog::exit_specialist().triggered_abilities[0].effect.clone();
        let ctx = EffectContext {
            targets: vec![Target::Permanent(victim)],
            ..EffectContext::for_ability(exit, 0, None)
        };
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.players[1].hand.iter().any(|c| c.id == victim), "creature returned to hand");
    }

    /// Projektor Inspector loots when a Detective you control enters.
    #[test]
    fn projektor_inspector_loots_on_detective() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::projektor_inspector());
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::island());
        }
        let to_pitch = g.add_card_to_hand(0, catalog::forest());
        // AutoDecider declines "may" — script the yes + the discard choice.
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Discard(vec![to_pitch]),
        ]));
        let other = g.add_card_to_battlefield(0, catalog::loxodon_eavesdropper()); // a Detective
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: other }]);
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == to_pitch), "looted (drew then discarded)");
    }

    /// Hotshot Investigators bounces a creature and investigates when it was yours.
    #[test]
    fn hotshot_investigators_bounces_and_investigates_own() {
        let mut g = two_player_game();
        let hot = g.add_card_to_battlefield(0, catalog::hotshot_investigators());
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::hotshot_investigators().triggered_abilities[0].effect.clone();
        let ctx = EffectContext {
            targets: vec![Target::Permanent(mine)],
            ..EffectContext::for_ability(hot, 0, None)
        };
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == mine), "own creature returned to hand");
        assert_eq!(clues(&g, 0), 1, "controlled it -> investigate");
    }

    /// Frantic Scapegoat suspects itself on entry.
    #[test]
    fn frantic_scapegoat_suspects_itself() {
        let mut g = two_player_game();
        let goat = g.add_card_to_battlefield(0, catalog::frantic_scapegoat());
        g.fire_self_etb_triggers(goat, 0);
        drain_stack(&mut g);
        let c = g.computed_permanent(goat).unwrap();
        // Suspected creatures have menace and can't block.
        assert!(c.keywords.contains(&Keyword::Menace), "suspected -> menace");
    }

    /// Slice from the Shadows gives target creature -X/-X and can't be countered.
    #[test]
    fn slice_from_the_shadows_shrinks() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::avenger_of_zendikar()); // 5/5
        let def = catalog::slice_from_the_shadows();
        assert!(def.keywords.contains(&Keyword::CantBeCountered));
        let ctx = EffectContext {
            targets: vec![Target::Permanent(victim)],
            ..EffectContext::for_spell(0, None, 0, 3)
        };
        g.resolve_effect(&def.effect, &ctx).unwrap();
        drain_stack(&mut g);
        let c = g.computed_permanent(victim).unwrap();
        assert_eq!((c.power, c.toughness), (2, 2), "-3/-3 applied");
    }

    /// Cerebral Confiscation's first mode makes the opponent discard two cards.
    #[test]
    fn cerebral_confiscation_discards_two() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_hand(1, catalog::grizzly_bears());
        }
        let modes = match &catalog::cerebral_confiscation().effect {
            crabomination::effect::Effect::ChooseMode(m) => m.clone(),
            _ => panic!("not modal"),
        };
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let before = g.players[1].hand.len();
        g.resolve_effect(&modes[0], &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), before - 2, "opponent discarded two");
    }

    /// Caught Red-Handed steals a creature for the turn and suspects it.
    #[test]
    fn caught_red_handed_steals_and_suspects() {
        let mut g = two_player_game();
        let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ctx = EffectContext {
            targets: vec![Target::Permanent(creature)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        g.resolve_effect(&catalog::caught_red_handed().effect, &ctx).unwrap();
        drain_stack(&mut g);
        let c = g.computed_permanent(creature).unwrap();
        assert_eq!(c.controller, 0, "control gained");
        assert!(c.keywords.contains(&Keyword::Haste), "gains haste");
        assert!(c.keywords.contains(&Keyword::Menace), "suspected -> menace");
    }

    /// Snarling Gorehound surveils when a small creature you control enters.
    #[test]
    fn snarling_gorehound_surveils_on_small_creature() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::snarling_gorehound());
        let top = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
            kept_top: vec![],
            bottom: vec![top],
        }]));
        // A 2/2 (power ≤ 2) entering triggers the surveil.
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "surveilled to graveyard");
    }
}

mod recent242 {
    use crabomination::card::{
        CardDefinition, CardType, CounterType, CreatureType, Subtypes,
    };
    use crabomination::catalog;
    use crabomination::game::types::TurnStep;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::{b, cost, g, r, u, w, ManaSymbol};

    /// A vanilla 1/1 in one color, for board-state solve conditions.
    fn mono(name: &'static str, pip: ManaSymbol) -> CardDefinition {
        CardDefinition {
            name,
            cost: cost(&[pip]),
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
            power: 1,
            toughness: 1,
            ..Default::default()
        }
    }

    fn solve_now(g: &mut crabomination::game::GameState) {
        let mut evs = vec![];
        g.process_case_solves(&mut evs);
        drain_stack(g);
    }

    fn is_solved(g: &crabomination::game::GameState, id: crabomination::card::CardId) -> bool {
        g.battlefield.iter().find(|c| c.id == id).map(|c| c.case_solved).unwrap_or(false)
    }

    /// ETB discards a card, then draws two (net +1, one card binned).
    #[test]
    fn crimson_pulse_etb_loots() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_hand(0, catalog::island());
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::mountain());
        }
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
        g.fire_self_etb_triggers(case, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 3, "discard one, draw two");
        assert_eq!(g.players[0].graveyard.len(), 1, "one card binned");
    }

    /// Solves at the end step once the controller's hand is empty.
    #[test]
    fn crimson_pulse_solves_on_empty_hand() {
        let mut g = two_player_game();
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
        assert!(g.players[0].hand.is_empty());
        solve_now(&mut g);
        assert!(is_solved(&g, case), "empty hand solves the Case");
    }

    /// Does not solve while a card remains in hand.
    #[test]
    fn crimson_pulse_unsolved_with_cards_in_hand() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::forest());
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
        solve_now(&mut g);
        assert!(!is_solved(&g, case), "a card in hand blocks the solve");
    }

    /// Once solved, the upkeep ability wheels the whole hand into two fresh cards.
    #[test]
    fn crimson_pulse_solved_wheels_at_upkeep() {
        let mut g = two_player_game();
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
        solve_now(&mut g);
        assert!(is_solved(&g, case));
        g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_hand(0, catalog::forest());
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::island());
        }
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 2, "discard hand, draw two");
        assert!(
            g.players[0].hand.iter().all(|c| c.definition.name == "Island"),
            "new hand is the two drawn cards"
        );
    }

    /// ETB distributes two +1/+1 counters onto a creature you control.
    #[test]
    fn trampled_garden_etb_distributes_counters() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, mono("Bear", g_pip()));
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_trampled_garden());
        g.fire_self_etb_triggers(case, 0);
        drain_stack(&mut g);
        let counters = g.battlefield.iter().find(|c| c.id == bear).unwrap()
            .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        assert_eq!(counters, 2, "two +1/+1 counters distributed");
    }

    /// Solves once your creatures' total power reaches eight.
    #[test]
    fn trampled_garden_solves_on_total_power() {
        let mut g = two_player_game();
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_trampled_garden());
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::grizzly_bears());
        }
        solve_now(&mut g);
        assert!(!is_solved(&g, case), "total power 6 is not enough");
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        solve_now(&mut g);
        assert!(is_solved(&g, case), "total power 8 solves the Case");
    }

    /// ETB fetches a basic land card into hand.
    #[test]
    fn shattered_pact_etb_fetches_basic() {
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(forest)),
        ]));
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_shattered_pact());
        g.fire_self_etb_triggers(case, 0);
        drain_stack(&mut g);
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Forest"),
            "basic land fetched to hand"
        );
    }

    /// Solves once there are five colors among permanents you control.
    #[test]
    fn shattered_pact_solves_on_five_colors() {
        let mut g = two_player_game();
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_shattered_pact());
        for (n, pip) in [("W", w()), ("U", u()), ("B", b()), ("R", r())] {
            g.add_card_to_battlefield(0, mono(n, pip));
        }
        solve_now(&mut g);
        assert!(!is_solved(&g, case), "four colors is not enough");
        g.add_card_to_battlefield(0, mono("G", g_pip()));
        solve_now(&mut g);
        assert!(is_solved(&g, case), "five colors solves the Case");
    }

    /// ETB investigates; solving needs three artifacts and switches on the {2}{U},
    /// Sacrifice activated ability.
    #[test]
    fn filched_falcon_solves_on_three_artifacts_and_arms_ability() {
        let mut g = two_player_game();
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_filched_falcon());
        g.fire_self_etb_triggers(case, 0);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count(),
            1,
            "ETB investigate made a Clue"
        );
        // The Clue is one artifact; add two more to reach three.
        g.add_card_to_battlefield(0, catalog::ornithopter());
        solve_now(&mut g);
        assert!(!is_solved(&g, case), "two artifacts is not enough");
        g.add_card_to_battlefield(0, catalog::ornithopter());
        solve_now(&mut g);
        assert!(is_solved(&g, case), "three artifacts solves the Case");
        let armed = g.battlefield.iter().find(|c| c.id == case).unwrap()
            .definition.activated_abilities.len();
        assert_eq!(armed, 1, "solved Case gains its activated ability");
    }

    /// Each creature you control entering gains 1 life; five such gains solves it.
    #[test]
    fn uneaten_feast_gains_life_and_solves() {
        let mut g = two_player_game();
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_uneaten_feast());
        let start = g.players[0].life;
        for _ in 0..5 {
            let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentEntered {
                card_id: bear,
            }]);
            drain_stack(&mut g);
        }
        assert_eq!(g.players[0].life, start + 5, "gained 1 life per creature");
        solve_now(&mut g);
        assert!(is_solved(&g, case), "gaining 5 life this turn solves the Case");
    }

    /// Solves on seven lands and arms the play-from-top statics.
    #[test]
    fn locked_hothouse_solves_on_seven_lands_and_arms_top_play() {
        let mut g = two_player_game();
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_locked_hothouse());
        let armed_before =
            g.battlefield.iter().find(|c| c.id == case).unwrap().definition.static_abilities.len();
        for _ in 0..6 {
            g.add_card_to_battlefield(0, catalog::forest());
        }
        solve_now(&mut g);
        assert!(!is_solved(&g, case), "six lands is not enough");
        g.add_card_to_battlefield(0, catalog::forest());
        solve_now(&mut g);
        assert!(is_solved(&g, case), "seven lands solves the Case");
        let armed_after =
            g.battlefield.iter().find(|c| c.id == case).unwrap().definition.static_abilities.len();
        assert_eq!(armed_after, armed_before + 2, "solved Case gains its two top-play statics");
    }

    /// ETB: each creature you control pings the chosen enemy creature. Solved: your
    /// creatures get +1/+0.
    #[test]
    fn gateway_express_pings_and_anthems() {
        let mut g = two_player_game();
        // Two 2/2s ping the enemy 0/4 for 2 total on ETB.
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let wall = g.add_card_to_battlefield(1, catalog::wall_of_omens());
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_gateway_express());
        let effect = catalog::case_of_the_gateway_express().triggered_abilities[0].effect.clone();
        let ctx = crabomination::game::effects::EffectContext::for_ability(
            case,
            0,
            Some(crabomination::game::types::Target::Permanent(wall)),
        );
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.battlefield.iter().find(|c| c.id == wall).unwrap().damage, 2, "two pings");
        // Solve via three attackers this turn, then the anthem applies.
        g.players[0].creatures_attacked_this_turn = 3;
        solve_now(&mut g);
        assert!(is_solved(&g, case));
        let bear = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+0 anthem while solved");
    }

    /// "Whenever you solve a Case" fires the Auditor's look-six.
    #[test]
    fn case_file_auditor_triggers_on_solve() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::case_file_auditor());
        let ench = g.add_card_to_library(0, catalog::case_of_the_shattered_pact());
        // Auditor looks at the top six; a Case (enchantment) is on top to reveal.
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
        assert!(g.players[0].hand.is_empty());
        solve_now(&mut g);
        assert!(is_solved(&g, case));
        assert!(
            g.players[0].hand.iter().any(|c| c.id == ench),
            "solving a Case let the Auditor pull an enchantment to hand"
        );
    }

    /// A Case's solved designation clears when it leaves the battlefield.
    #[test]
    fn solved_case_resets_on_leave() {
        let mut g = two_player_game();
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_crimson_pulse());
        solve_now(&mut g);
        assert!(is_solved(&g, case));
        let ctx = crabomination::game::effects::EffectContext::for_trigger(case, 0, None, 0);
        let mut evs = vec![];
        g.move_card_to(case, &crabomination::effect::ZoneDest::Graveyard, &ctx, &mut evs);
        drain_stack(&mut g);
        let in_gy = g.players[0].graveyard.iter().find(|c| c.id == case);
        assert!(in_gy.map(|c| !c.case_solved).unwrap_or(true), "solved flag cleared off-battlefield");
    }

    fn g_pip() -> ManaSymbol {
        g()
    }
}

mod recent243 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target, TurnStep};
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
        g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
    }

    /// The Chase Is On pumps +3/+0, grants first strike, and investigates.
    #[test]
    fn the_chase_is_on_pumps_and_investigates() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::the_chase_is_on());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast The Chase Is On");
        drain_stack(&mut g);
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (5, 2), "+3/+0");
        assert!(c.keywords.contains(&Keyword::FirstStrike), "gains first strike");
        assert_eq!(clues(&g, 0), 1, "investigated");
    }

    /// Galvanize deals 3, or 5 once you've drawn two cards this turn.
    #[test]
    fn galvanize_scales_with_draws() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        // A 0/4 dies only to the 5-damage mode.
        let wall = g.add_card_to_battlefield(1, catalog::wall_of_omens());
        let spell = g.add_card_to_hand(0, catalog::galvanize());
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::forest());
        }
        let mut evs = vec![];
        g.draw_one(0, &mut evs);
        g.draw_one(0, &mut evs);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(wall)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Galvanize");
        drain_stack(&mut g);
        assert!(g.battlefield_find(wall).is_none(), "5 damage after two draws kills the 0/4");
    }

    /// Red Herring has haste + must-attack, and sacs itself to draw.
    #[test]
    fn red_herring_keywords_and_sac_draw() {
        let mut g = two_player_game();
        let rh = g.add_card_to_battlefield(0, catalog::red_herring());
        g.add_card_to_library(0, catalog::forest());
        let c = g.computed_permanent(rh).unwrap();
        assert!(c.keywords.contains(&Keyword::Haste) && c.keywords.contains(&Keyword::MustAttack));
        let before = g.players[0].hand.len();
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: rh,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("sacrifice to draw");
        drain_stack(&mut g);
        assert!(g.battlefield_find(rh).is_none(), "sacrificed");
        assert_eq!(g.players[0].hand.len(), before + 1, "drew a card");
    }

    /// Vengeful Creeper, when turned face up, destroys an opponent's artifact.
    #[test]
    fn vengeful_creeper_face_up_destroys() {
        let mut g = two_player_game();
        let creeper = g.add_card_to_battlefield(0, catalog::vengeful_creeper());
        let orn = g.add_card_to_battlefield(1, catalog::ornithopter());
        let effect = catalog::vengeful_creeper().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(creeper, 0, Some(Target::Permanent(orn)));
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(orn).is_none(), "opponent artifact destroyed");
    }

    /// Rubblebelt Maverick surveils 2 on ETB.
    #[test]
    fn rubblebelt_maverick_surveils_two() {
        let mut g = two_player_game();
        let a = g.add_card_to_library(0, catalog::forest());
        let b = g.add_card_to_library(0, catalog::forest());
        let mav = g.add_card_to_battlefield(0, catalog::rubblebelt_maverick());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
            kept_top: vec![],
            bottom: vec![a, b],
        }]));
        g.fire_self_etb_triggers(mav, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), 2, "surveilled two cards to graveyard");
    }

    /// Leering Onlooker's graveyard ability mints two tapped flying Bats.
    #[test]
    fn leering_onlooker_makes_two_tapped_bats() {
        let mut g = two_player_game();
        let src = g.add_card_to_graveyard(0, catalog::leering_onlooker());
        let effect = catalog::leering_onlooker().activated_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(src, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        let bats: Vec<_> =
            g.battlefield.iter().filter(|c| c.definition.name == "Bat" && c.controller == 0).collect();
        assert_eq!(bats.len(), 2, "two Bats");
        assert!(bats.iter().all(|c| c.tapped && c.definition.keywords.contains(&Keyword::Flying)));
    }

    /// Tunnel Tipster grows at end step after a face-down creature entered.
    #[test]
    fn tunnel_tipster_grows_after_facedown() {
        let mut g = two_player_game();
        let tip = g.add_card_to_battlefield(0, catalog::tunnel_tipster());
        g.players[0].face_down_activity_this_turn = true;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        let c = g.computed_permanent(tip).unwrap();
        assert_eq!((c.power, c.toughness), (2, 2), "+1/+1 counter after a face-down entered");
    }

    /// Gravestone Strider exiles a card from a graveyard.
    #[test]
    fn gravestone_strider_exiles_from_graveyard() {
        let mut g = two_player_game();
        let strider = g.add_card_to_battlefield(0, catalog::gravestone_strider());
        let victim = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let effect = catalog::gravestone_strider().activated_abilities[1].effect.clone();
        let ctx = EffectContext::for_ability(strider, 0, Some(Target::Permanent(victim)));
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.exile.iter().any(|c| c.id == victim), "graveyard card exiled");
        assert!(g.players[1].graveyard.iter().all(|c| c.id != victim));
    }
}

mod recent244 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target, TurnStep};
    use crabomination::game::{drain_stack, two_player_game, GameEvent};
    use crabomination::mana::Color;

    fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
        g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
    }

    /// Wispdrinker Vampire drains on a small creature's entry and its activated
    /// ability grants deathtouch + lifelink to your small creatures.
    #[test]
    fn wispdrinker_vampire_drains_and_buffs_small_creatures() {
        let mut g = two_player_game();
        let _wisp = g.add_card_to_battlefield(0, catalog::wispdrinker_vampire());
        let life = g.players[0].life;
        let opp = g.players[1].life;
        let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves()); // 1/1
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: elf }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "drained 1 into your life");
        assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
        // Activate the anthem-grant.
        let wisp_id = g.battlefield.iter().find(|c| c.definition.name == "Wispdrinker Vampire").unwrap().id;
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(5);
        g.perform_action(GameAction::ActivateAbility {
            card_id: wisp_id,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate the deathtouch/lifelink grant");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(elf).unwrap().keywords.contains(&Keyword::Deathtouch),
            "small creature gains deathtouch"
        );
    }

    /// Torch the Witness deals twice X and investigates when excess damage lands.
    #[test]
    fn torch_the_witness_double_x_and_excess_investigate() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::torch_the_witness());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3); // X = 3 → 6 damage
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: Some(3),
        })
        .expect("cast Torch the Witness for X=3");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "6 damage kills the 2/2");
        assert_eq!(clues(&g, 0), 1, "excess damage investigated");
    }

    /// Extract a Confession edicts each opponent (greatest-power with evidence).
    #[test]
    fn extract_a_confession_edicts_opponent() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::extract_a_confession());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Extract a Confession");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "opponent sacrificed their creature");
    }

    /// Vitu-Ghazi Inspector rewards a collected evidence: +1/+1 on a creature and 2
    /// life. Without evidence, its ETB does nothing.
    #[test]
    fn vitu_ghazi_inspector_rewards_collected_evidence() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for _ in 0..3 {
            g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 6 to collect
        }
        let spell = g.add_card_to_hand(0, catalog::vitu_ghazi_inspector());
        let life = g.players[0].life;
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Vitu-Ghazi Inspector collecting evidence");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained 2 life with evidence");
        let counters: u32 = g
            .battlefield
            .iter()
            .filter(|c| c.controller == 0)
            .map(|c| c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0))
            .sum();
        assert_eq!(counters, 1, "a +1/+1 counter was placed");
    }

    /// Novice Inspector investigates on ETB.
    #[test]
    fn novice_inspector_investigates() {
        let mut g = two_player_game();
        let ni = g.add_card_to_battlefield(0, catalog::novice_inspector());
        g.fire_self_etb_triggers(ni, 0);
        drain_stack(&mut g);
        assert_eq!(clues(&g, 0), 1, "ETB investigate");
    }

    /// Curious Cadaver returns from the graveyard when you sacrifice a Clue.
    #[test]
    fn curious_cadaver_returns_on_clue_sacrifice() {
        let mut g = two_player_game();
        // A Clue on the battlefield, and the Cadaver waiting in the graveyard.
        let inspector = g.add_card_to_battlefield(0, catalog::novice_inspector());
        g.fire_self_etb_triggers(inspector, 0);
        drain_stack(&mut g);
        let clue = g.battlefield.iter().find(|c| c.definition.name == "Clue").unwrap().id;
        let cadaver = g.add_card_to_graveyard(0, catalog::curious_cadaver());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentSacrificed { card_id: clue, who: 0 }]);
        drain_stack(&mut g);
        assert!(
            g.players[0].hand.iter().any(|c| c.id == cadaver),
            "Cadaver returned to hand on Clue sacrifice"
        );
    }

    /// They Went This Way ramps a tapped basic and investigates.
    #[test]
    fn they_went_this_way_ramps_and_investigates() {
        let mut g = two_player_game();
        let src = g.add_card_to_hand(0, catalog::they_went_this_way());
        let forest = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        let effect = catalog::they_went_this_way().effect.clone();
        let ctx = EffectContext::for_ability(src, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        let land = g.battlefield.iter().find(|c| c.id == forest).unwrap();
        assert!(land.tapped, "basic entered tapped");
        assert_eq!(clues(&g, 0), 1, "investigated");
    }

    /// Undercover Crocodelf has Disguise and investigates on combat connect.
    #[test]
    fn undercover_crocodelf_disguise_and_connect_investigate() {
        let mut g = two_player_game();
        let croc = g.add_card_to_battlefield(0, catalog::undercover_crocodelf());
        assert!(
            catalog::undercover_crocodelf()
                .keywords
                .iter()
                .any(|k| matches!(k, Keyword::Disguise(_))),
            "has Disguise"
        );
        let effect = catalog::undercover_crocodelf().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(croc, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(clues(&g, 0), 1, "investigated on combat damage");
    }

    /// Sharp-Eyed Rookie grows and investigates when a bigger creature enters, but
    /// ignores a smaller one.
    #[test]
    fn sharp_eyed_rookie_grows_on_bigger_creature() {
        let mut g = two_player_game();
        let rookie = g.add_card_to_battlefield(0, catalog::sharp_eyed_rookie());
        // A 1/1 is not bigger — no trigger.
        let small = g.add_card_to_battlefield(0, catalog::llanowar_elves());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: small }]);
        drain_stack(&mut g);
        assert_eq!(clues(&g, 0), 0, "small creature does not trigger");
        // A 3/3 is bigger in both stats — +1/+1 and investigate.
        let big = g.add_card_to_battlefield(0, catalog::hill_giant());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: big }]);
        drain_stack(&mut g);
        let c = g.computed_permanent(rookie).unwrap();
        assert_eq!((c.power, c.toughness), (3, 3), "grew from the counter");
        assert_eq!(clues(&g, 0), 1, "investigated");
    }
}

mod recent245 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::GameAction;
    use crabomination::game::{drain_stack, two_player_game, GameEvent};

    fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
        g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
    }

    /// Wrench grants +1/+1, vigilance, and a "{3}, {T}: Tap target creature"
    /// activated ability to the equipped creature.
    #[test]
    fn wrench_buffs_and_grants_tap_ability() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let wrench = g.add_card_to_battlefield(0, catalog::wrench());
        g.battlefield_find_mut(wrench).unwrap().attached_to = Some(bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
        assert!(cp.keywords.contains(&Keyword::Vigilance), "gains vigilance");
        assert_eq!(g.granted_abilities_for(bear).len(), 1, "granted the tap ability");
    }

    /// Rope grants +1/+2, reach, and can't-be-blocked-by-more-than-one.
    #[test]
    fn rope_buffs_reach_and_lone_block() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let rope = g.add_card_to_battlefield(0, catalog::rope());
        g.battlefield_find_mut(rope).unwrap().attached_to = Some(bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 4), "+1/+2");
        assert!(cp.keywords.contains(&Keyword::Reach));
        assert!(cp.keywords.contains(&Keyword::CantBeBlockedByMoreThanOne));
    }

    /// Knife's +1/+0 and first strike apply only during the controller's turn.
    #[test]
    fn knife_only_during_your_turn() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let knife = g.add_card_to_battlefield(0, catalog::knife());
        g.battlefield_find_mut(knife).unwrap().attached_to = Some(bear);
        // Player 0's turn: bonus active.
        g.active_player_idx = 0;
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 3, "+1/+0 during your turn");
        assert!(cp.keywords.contains(&Keyword::FirstStrike));
        // Opponent's turn: bonus gone.
        g.active_player_idx = 1;
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 2, "no bonus off your turn");
        assert!(!cp.keywords.contains(&Keyword::FirstStrike));
    }

    /// The shared "{2}, Sacrifice this Equipment: Draw a card" ability draws and
    /// sacrifices the Equipment.
    #[test]
    fn clue_equipment_sac_draws() {
        let mut g = two_player_game();
        let candlestick = g.add_card_to_battlefield(0, catalog::candlestick());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: candlestick,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("sac Candlestick to draw");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert!(g.battlefield_find(candlestick).is_none(), "Equipment sacrificed");
    }

    /// Surveillance Monitor mints a Thopter whenever its controller collects
    /// evidence.
    #[test]
    fn surveillance_monitor_thopter_on_evidence() {
        let mut g = two_player_game();
        let _mon = g.add_card_to_battlefield(0, catalog::surveillance_monitor());
        g.dispatch_triggers_for_events(&[GameEvent::EvidenceCollected { player: 0 }]);
        drain_stack(&mut g);
        let thopters = g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count();
        assert_eq!(thopters, 1, "collecting evidence made a Thopter");
    }

    /// Evidence Examiner investigates whenever its controller collects evidence.
    #[test]
    fn evidence_examiner_investigates_on_collection() {
        let mut g = two_player_game();
        let _examiner = g.add_card_to_battlefield(0, catalog::evidence_examiner());
        g.dispatch_triggers_for_events(&[GameEvent::EvidenceCollected { player: 0 }]);
        drain_stack(&mut g);
        assert_eq!(clues(&g, 0), 1, "collecting evidence investigated");
    }

    /// Collecting evidence via `Effect::CollectEvidence` emits an
    /// `EvidenceCollected` event (the wiring behind the two payoffs above).
    #[test]
    fn collect_evidence_emits_event() {
        use crabomination::effect::{Effect, Value};
        let mut g = two_player_game();
        let src = g.add_card_to_battlefield(0, catalog::surveillance_monitor());
        for _ in 0..3 {
            g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 6, ample for 4
        }
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let effect = Effect::CollectEvidence { amount: Value::Const(4), then: Box::new(Effect::Noop) };
        let events = g.resolve_effect(&effect, &EffectContext::for_ability(src, 0, None)).unwrap();
        assert!(
            events.iter().any(|e| matches!(e, GameEvent::EvidenceCollected { player: 0 })),
            "collection emitted the EvidenceCollected event"
        );
    }

    /// Unscrupulous Agent makes an opponent exile a card from hand on ETB.
    #[test]
    fn unscrupulous_agent_exiles_from_hand() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::grizzly_bears());
        let opp_hand = g.players[1].hand.len();
        let agent = g.add_card_to_battlefield(0, catalog::unscrupulous_agent());
        g.fire_self_etb_triggers(agent, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent exiled a card");
    }

    /// Undercity Eliminator sacrifices a permanent to exile an opponent's creature.
    #[test]
    fn undercity_eliminator_sacs_to_exile() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let _fodder = g.add_card_to_battlefield(0, catalog::llanowar_elves());
        let elim = g.add_card_to_battlefield(0, catalog::undercity_eliminator());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.fire_self_etb_triggers(elim, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "opponent's creature exiled");
    }

    /// Furtive Courier loots (draw then discard) when it attacks.
    #[test]
    fn furtive_courier_attack_loots() {
        let mut g = two_player_game();
        let courier = g.add_card_to_battlefield(0, catalog::furtive_courier());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::grizzly_bears());
        let effect = catalog::furtive_courier().triggered_abilities[0].effect.clone();
        let before = g.players[0].graveyard.len();
        g.resolve_effect(&effect, &EffectContext::for_trigger(courier, 0, None, 0)).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), before + 1, "discarded one card after drawing");
    }
}

mod recent246 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target};
    use crabomination::game::{drain_stack, two_player_game, GameEvent};

    fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
        g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
    }

    /// Rune-Brand Juggler suspects a creature on ETB, then sacrifices a suspected
    /// creature to shrink a target -5/-5.
    #[test]
    fn rune_brand_juggler_suspects_and_sacs() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let juggler = g.add_card_to_battlefield(0, catalog::rune_brand_juggler());
        let effect = catalog::rune_brand_juggler().triggered_abilities[0].effect.clone();
        let mut ctx = EffectContext::for_trigger(juggler, 0, None, 0);
        ctx.targets = vec![Target::Permanent(mine)];
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.battlefield_find(mine).unwrap().suspected, "ETB suspected our creature");
        // Now activate: sacrifice the suspected creature to shrink the opponent's.
        let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: juggler,
            ability_index: 0,
            target: Some(Target::Permanent(foe)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("sac the suspected creature");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none(), "suspected creature sacrificed");
        assert!(g.battlefield_find(foe).is_none(), "-5/-5 killed the 3/3");
    }

    /// Chalk Outline mints a Detective and investigates when a creature card leaves
    /// your graveyard.
    #[test]
    fn chalk_outline_detective_and_investigate() {
        let mut g = two_player_game();
        let _outline = g.add_card_to_battlefield(0, catalog::chalk_outline());
        let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::CardLeftGraveyard { player: 0, card_id: gy }]);
        drain_stack(&mut g);
        let detectives = g.battlefield.iter().filter(|c| c.definition.name == "Detective").count();
        assert_eq!(detectives, 1, "made a Detective");
        assert_eq!(clues(&g, 0), 1, "investigated");
    }

    /// Soul Enervation drains when a creature card leaves your graveyard.
    #[test]
    fn soul_enervation_drains_on_graveyard_leave() {
        let mut g = two_player_game();
        let _ener = g.add_card_to_battlefield(0, catalog::soul_enervation());
        let life = g.players[0].life;
        let opp = g.players[1].life;
        let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::CardLeftGraveyard { player: 0, card_id: gy }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "gained 1");
        assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
    }

    /// Convenient Target suspects the creature it enchants and buffs it +1/+1.
    #[test]
    fn convenient_target_suspects_and_buffs() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_battlefield(0, catalog::convenient_target());
        g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
        // Fire the ETB suspect trigger.
        g.fire_self_etb_triggers(aura, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).unwrap().suspected, "enchanted creature suspected");
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    }

    /// Curious Inquiry buffs +1/+1 and grants a combat-damage investigate trigger.
    #[test]
    fn curious_inquiry_buffs_and_grants_investigate() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_battlefield(0, catalog::curious_inquiry());
        g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
        // The Aura grants a combat-damage investigate trigger to the creature.
        let bonus = catalog::curious_inquiry().equipped_bonus.unwrap();
        assert_eq!(bonus.triggered_abilities.len(), 1, "grants one triggered ability");
    }

    /// Due Diligence grants the enchanted creature +2/+2 and vigilance.
    #[test]
    fn due_diligence_buffs_enchanted() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_battlefield(0, catalog::due_diligence());
        g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
        assert!(cp.keywords.contains(&Keyword::Vigilance));
    }

    /// Suspected filter: the {3}{B}{R} sac ability rejects when no suspected
    /// creature is available.
    #[test]
    fn juggler_sac_needs_a_suspected_creature() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        let _plain = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // not suspected
        let juggler = g.add_card_to_battlefield(0, catalog::rune_brand_juggler());
        let foe = g.add_card_to_battlefield(1, catalog::hill_giant());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        let res = g.perform_action(GameAction::ActivateAbility {
            card_id: juggler,
            ability_index: 0,
            target: Some(Target::Permanent(foe)),
            additional_targets: vec![],
            x_value: None, mode: None,
        });
        assert!(res.is_err(), "no suspected creature to sacrifice → activation rejected");
    }
}

mod recent247 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::{GameAction, Target};
    use crabomination::game::{drain_stack, two_player_game};

    fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
        g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
    }

    /// Magnifying Glass taps for {C} and its {4}, {T} ability investigates.
    #[test]
    fn magnifying_glass_taps_and_investigates() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        let glass = g.add_card_to_battlefield(0, catalog::magnifying_glass());
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::ActivateAbility {
            card_id: glass,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate the investigate ability");
        drain_stack(&mut g);
        assert_eq!(clues(&g, 0), 1, "investigated");
    }

    /// Escape Tunnel sacrifices to make a small creature unblockable.
    #[test]
    fn escape_tunnel_grants_unblockable() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let tunnel = g.add_card_to_battlefield(0, catalog::escape_tunnel());
        g.perform_action(GameAction::ActivateAbility {
            card_id: tunnel,
            ability_index: 1,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate the unblockable ability");
        drain_stack(&mut g);
        assert!(g.battlefield_find(tunnel).is_none(), "Escape Tunnel sacrificed");
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable),
            "small creature can't be blocked"
        );
    }

    /// Scene of the Crime enters tapped and sacrifices to draw a card.
    #[test]
    fn scene_of_the_crime_enters_tapped_and_draws() {
        let mut g = two_player_game();
        g.priority.player_with_priority = 0;
        let scene = g.add_card_to_battlefield(0, catalog::scene_of_the_crime());
        // (The enters-tapped static is exercised through the real ETB path in play;
        // `add_card_to_battlefield` skips replacement effects.)
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand = g.players[0].hand.len();
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: scene,
            ability_index: 2,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("sac to draw");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert!(g.battlefield_find(scene).is_none(), "sacrificed");
    }

    /// Massacre Girl grants wither to your creatures and draws when an opponent's
    /// creature dies with toughness less than 1.
    #[test]
    fn massacre_girl_wither_and_death_draw() {
        let mut g = two_player_game();
        let girl = g.add_card_to_battlefield(0, catalog::massacre_girl_known_killer());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let _ = girl;
        assert!(
            g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Wither),
            "your creatures have wither"
        );
        // An opponent 1/1 reduced to 0 toughness dies → draw.
        g.add_card_to_library(0, catalog::grizzly_bears());
        let foe = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // 1/1
        g.battlefield_find_mut(foe).unwrap().counters.insert(CounterType::MinusOneMinusOne, 1);
        let hand = g.players[0].hand.len();
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "0-toughness creature died");
        assert_eq!(g.players[0].hand.len(), hand + 1, "Massacre Girl drew a card");

        // A creature that dies to lethal damage (toughness still ≥ 1) does not draw.
        let hardy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand2 = g.players[0].hand.len();
        g.battlefield_find_mut(hardy).unwrap().damage = 2; // lethal, toughness stays 2
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield_find(hardy).is_none(), "took lethal damage and died");
        assert_eq!(g.players[0].hand.len(), hand2, "toughness was 2 → no draw");
    }
}

mod recent248 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::{GameAction, Target, TurnStep};
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Sacrificing an artifact bumps the per-turn artifact-sacrifice tally; a
    /// creature sacrifice does not.
    #[test]
    fn artifact_sacrifice_tracker_counts_only_artifacts() {
        let mut g = two_player_game();
        let clue_src = g.add_card_to_battlefield(0, catalog::magnifying_glass()); // an artifact
        let mut evs = vec![];
        g.sacrifice_one(clue_src, 0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        assert_eq!(g.players[0].artifacts_sacrificed_this_turn, 1, "artifact counted");
        // A creature sacrifice bumps the permanent tally but not the artifact one.
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut evs = vec![];
        g.sacrifice_one(bear, 0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        assert_eq!(g.players[0].artifacts_sacrificed_this_turn, 1, "creature not counted");
        assert_eq!(g.players[0].permanents_sacrificed_this_turn, 2, "both permanents counted");
    }

    /// Suspicious Detonation costs {3} less once you've sacrificed an artifact this
    /// turn — castable for {1}{R} instead of {4}{R}.
    #[test]
    fn suspicious_detonation_cost_reduction_after_artifact_sac() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].artifacts_sacrificed_this_turn = 1;
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::suspicious_detonation());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1); // only {1}{R} — the reduced cost
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast for the reduced {1}{R}");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "4 damage killed the 2/2");
    }

    /// Furtive Courier is unblockable only while you've sacrificed an artifact this
    /// turn.
    #[test]
    fn furtive_courier_unblockable_after_artifact_sac() {
        let mut g = two_player_game();
        let courier = g.add_card_to_battlefield(0, catalog::furtive_courier());
        assert!(
            !g.computed_permanent(courier).unwrap().keywords.contains(&Keyword::Unblockable),
            "not unblockable without an artifact sacrifice"
        );
        g.players[0].artifacts_sacrificed_this_turn = 1;
        assert!(
            g.computed_permanent(courier).unwrap().keywords.contains(&Keyword::Unblockable),
            "unblockable once an artifact was sacrificed"
        );
    }

    /// Deadly Complication's destroy mode kills a target creature.
    #[test]
    fn deadly_complication_destroys() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::deadly_complication());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Deadly Complication");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "destroy mode killed the creature");
    }
}

mod recent249 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target, TurnStep};
    use crabomination::game::{drain_stack, two_player_game};

    fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
        g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
    }

    /// Clandestine Meddler suspects another creature you control on ETB (not itself).
    #[test]
    fn clandestine_meddler_suspects_another_creature() {
        let mut g = two_player_game();
        let meddler = g.add_card_to_battlefield(0, catalog::clandestine_meddler());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::clandestine_meddler().triggered_abilities[0].effect.clone();
        let mut ctx = EffectContext::for_trigger(meddler, 0, None, 0);
        ctx.targets = vec![Target::Permanent(ally)];
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.battlefield_find(ally).unwrap().suspected, "the other creature is suspected");
        assert!(!g.battlefield_find(meddler).unwrap().suspected, "not itself");
    }

    /// Forensic Gadgeteer investigates whenever you cast an artifact spell.
    #[test]
    fn forensic_gadgeteer_investigates_on_artifact_cast() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let _gadgeteer = g.add_card_to_battlefield(0, catalog::forensic_gadgeteer());
        let artifact = g.add_card_to_hand(0, catalog::magnifying_glass()); // {3} artifact
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: artifact,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast an artifact spell");
        drain_stack(&mut g);
        assert_eq!(clues(&g, 0), 1, "casting an artifact investigated");
    }

    /// Pompous Gadabout has hexproof only during its controller's turn.
    #[test]
    fn pompous_gadabout_hexproof_your_turn_only() {
        let mut g = two_player_game();
        let gad = g.add_card_to_battlefield(0, catalog::pompous_gadabout());
        g.active_player_idx = 0;
        assert!(
            g.computed_permanent(gad).unwrap().keywords.contains(&Keyword::Hexproof),
            "hexproof on your turn"
        );
        g.active_player_idx = 1;
        assert!(
            !g.computed_permanent(gad).unwrap().keywords.contains(&Keyword::Hexproof),
            "no hexproof on the opponent's turn"
        );
    }
}

mod recent250 {
    use crabomination::card::{CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::game::{drain_stack, two_player_game};

    /// Coerced to Kill steals the enchanted creature and makes it a 1/1 deathtouch
    /// Assassin; control reverts when the Aura leaves.
    #[test]
    fn coerced_to_kill_steals_and_reshapes() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer
        let aura = g.add_card_to_battlefield(0, catalog::coerced_to_kill());
        g.battlefield_find_mut(aura).unwrap().attached_to = Some(victim);
        g.fire_self_etb_triggers(aura, 0);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(victim).unwrap().controller, 0, "stole control");
        let cp = g.computed_permanent(victim).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "base P/T 1/1");
        assert!(cp.keywords.contains(&Keyword::Deathtouch), "gains deathtouch");
        assert!(cp.subtypes.creature_types.contains(&CreatureType::Assassin), "is an Assassin");
        // Removing the Aura reverts control.
        g.remove_to_graveyard_with_triggers(aura);
        g.check_state_based_actions();
        assert_eq!(g.battlefield_find(victim).unwrap().controller, 1, "control reverts");
    }

    /// Airtight Alibi buffs +2/+2, untaps, grants hexproof, and clears suspicion.
    #[test]
    fn airtight_alibi_buffs_and_cleanses() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        g.battlefield_find_mut(bear).unwrap().suspected = true;
        let aura = g.add_card_to_battlefield(0, catalog::airtight_alibi());
        g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
        g.fire_self_etb_triggers(aura, 0);
        drain_stack(&mut g);
        let c = g.battlefield_find(bear).unwrap();
        assert!(!c.tapped, "untapped on ETB");
        assert!(!c.suspected, "no longer suspected");
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
        assert!(cp.keywords.contains(&Keyword::Hexproof), "gains hexproof");
    }
}

mod recent251 {
    use crabomination::catalog;
    use crabomination::game::types::{GameAction, Target, TurnStep};
    use crabomination::game::{drain_stack, two_player_game};

    /// Kraul Whipcracker destroys an opponent's token on ETB (and can't hit a
    /// nontoken creature).
    #[test]
    fn kraul_whipcracker_destroys_opponent_token() {
        use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
        let mut g = two_player_game();
        // A real token for the opponent.
        let tok = TokenDefinition {
            name: "Bird".into(),
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
            power: 1,
            toughness: 1,
            ..Default::default()
        };
        let token = g.add_token_to_battlefield(1, &tok);
        let whip = g.add_card_to_battlefield(0, catalog::kraul_whipcracker());
        g.fire_self_etb_triggers(whip, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(token).is_none(), "opponent's token destroyed");
    }

    /// Forensic Researcher untaps another target permanent you control.
    #[test]
    fn forensic_researcher_untaps_your_permanent() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let researcher = g.add_card_to_battlefield(0, catalog::forensic_researcher());
        g.clear_sickness(researcher);
        let land = g.add_card_to_battlefield(0, catalog::forest());
        g.battlefield_find_mut(land).unwrap().tapped = true;
        g.perform_action(GameAction::ActivateAbility {
            card_id: researcher,
            ability_index: 0,
            target: Some(Target::Permanent(land)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate the untap ability");
        drain_stack(&mut g);
        assert!(!g.battlefield_find(land).unwrap().tapped, "the land was untapped");
    }
}

mod recent252 {
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
            x_value: None, mode: None,
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
            x_value: None, mode: None,
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
            x_value: None, mode: None,
        })
        .expect("activate the sacrifice draw ability");
        drain_stack(&mut g);
        assert!(g.battlefield_find(cryptex).is_none(), "Cryptex sacrificed");
        assert_eq!(g.players[0].hand.len(), hand_before + 3, "drew three");
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
            x_value: None, mode: None,
        });
        assert!(err.is_err(), "gated before an artifact sacrifice");
        // After sacrificing an artifact it makes a Thopter.
        g.players[0].artifacts_sacrificed_this_turn = 1;
        g.perform_action(GameAction::ActivateAbility {
            card_id: satchel,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
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
            x_value: None, mode: None,
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
            x_value: None, mode: None,
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
}
