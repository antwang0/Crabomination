//! Tests for recentN card batches 253-267 (merged from per-batch micro-files).

mod recent253 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target, TurnStep};
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Trostani, Three Whispers grants deathtouch to a target creature.
    #[test]
    fn trostani_grants_deathtouch() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let trostani = g.add_card_to_battlefield(0, catalog::trostani_three_whispers());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: trostani,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate the deathtouch ability");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch),
            "bear gained deathtouch",
        );
    }

    /// Ezrim investigates twice on ETB and gains a chosen keyword by sacrificing an
    /// artifact.
    #[test]
    fn ezrim_investigates_and_grants_chosen_keyword() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let ezrim = g.add_card_to_battlefield(0, catalog::ezrim_agency_chief());
        g.fire_self_etb_triggers(ezrim, 0);
        drain_stack(&mut g);
        let clues = g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count();
        assert_eq!(clues, 2, "investigated twice");
        // The keyword-grant ability carries a "sacrifice an artifact" cost.
        let ability = &catalog::ezrim_agency_chief().activated_abilities[0];
        assert!(ability.sac_other_filter.is_some(), "sacrifice-an-artifact cost present");
        // Resolve the modal grant, choosing lifelink (mode 1).
        let ctx = EffectContext::for_trigger(ezrim, 0, None, 1);
        g.resolve_effect(&ability.effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(ezrim).unwrap().keywords.contains(&Keyword::Lifelink),
            "Ezrim gained the chosen keyword (lifelink)",
        );
    }

    /// Agrus Kos suspects a clean creature, then exiles it once it's suspected.
    #[test]
    fn agrus_kos_suspects_then_exiles() {
        let mut g = two_player_game();
        let agrus = g.add_card_to_battlefield(0, catalog::agrus_kos_spirit_of_justice());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let interrogate = catalog::agrus_kos_spirit_of_justice().triggered_abilities[0].effect.clone();
        // First interrogation: not suspected → suspect it.
        let ctx = EffectContext::for_trigger(agrus, 0, Some(Target::Permanent(foe)), 0);
        g.resolve_effect(&interrogate, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).unwrap().suspected, "creature suspected");
        // Second interrogation: already suspected → exile it.
        let ctx = EffectContext::for_trigger(agrus, 0, Some(Target::Permanent(foe)), 0);
        g.resolve_effect(&interrogate, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "suspected creature exiled");
        assert!(g.exile.iter().any(|c| c.id == foe), "moved to exile");
    }

    /// Aurelia draws on a 3-creature attack and drains on a 5-creature attack.
    #[test]
    fn aurelia_law_above_attack_triggers() {
        use crabomination::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let aurelia = g.add_card_to_battlefield(0, catalog::aurelia_the_law_above());
        let mut atk = vec![aurelia];
        for _ in 0..4 {
            atk.push(g.add_card_to_battlefield(0, catalog::grizzly_bears()));
        }
        for &id in &atk { g.clear_sickness(id); }
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        let hand_before = g.players[0].hand.len();
        let foe_life = g.players[1].life;
        let my_life = g.players[0].life;
        g.step = TurnStep::DeclareAttackers;
        g.declare_attackers(atk.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect())
            .expect("declare five attackers");
        drain_stack(&mut g);
        // 5 attackers ≥ 3 (draw) and ≥ 5 (drain) both fire.
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew from the 3+ attack trigger");
        assert_eq!(g.players[1].life, foe_life - 3, "opponent took 3 from the 5+ trigger");
        assert_eq!(g.players[0].life, my_life + 3, "gained 3 from the 5+ trigger");
    }

    /// Rakdos draws two when the opponent has nothing to sacrifice at your end step.
    #[test]
    fn rakdos_patron_draws_when_no_sacrifice() {
        use crabomination::game::TurnStep;
        let mut g = two_player_game();
        let rakdos = g.add_card_to_battlefield(0, catalog::rakdos_patron_of_chaos());
        for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
        // Opponent controls only a token (nontoken required) → can't pay.
        use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
        let tok = TokenDefinition {
            name: "Bird".into(),
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
            power: 1, toughness: 1, ..Default::default()
        };
        g.add_token_to_battlefield(1, &tok);
        let hand_before = g.players[0].hand.len();
        let ctx = EffectContext::for_trigger(rakdos, 0, None, 0);
        let trig = catalog::rakdos_patron_of_chaos().triggered_abilities[0].effect.clone();
        g.step = TurnStep::End;
        g.resolve_effect(&trig, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew two (no sacrifice available)");
    }

    /// Voja pumps each creature by the Elf count and draws per Wolf on attack.
    #[test]
    fn voja_attack_counters_and_draw() {
        use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
        let mut g = two_player_game();
        let voja = g.add_card_to_battlefield(0, catalog::voja_jaws_of_the_conclave()); // a Wolf
        let elf = TokenDefinition {
            name: "Elf".into(),
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
            power: 1, toughness: 1, ..Default::default()
        };
        let e1 = g.add_token_to_battlefield(0, &elf);
        g.add_token_to_battlefield(0, &elf);
        for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
        let hand_before = g.players[0].hand.len();
        let ctx = EffectContext::for_trigger(voja, 0, None, 0);
        let trig = catalog::voja_jaws_of_the_conclave().triggered_abilities[0].effect.clone();
        g.resolve_effect(&trig, &ctx).unwrap();
        drain_stack(&mut g);
        // Two Elves → +2 counters on each creature; one Wolf (Voja) → draw 1.
        assert_eq!(g.battlefield_find(e1).unwrap().counters.get(&crabomination::card::CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2, "each creature got 2 counters");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew one per Wolf");
    }
}

mod recent254 {
    use crabomination::card::CardInstance;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::actions::cost_reduction_for_spell;
    use crabomination::game::{drain_stack, two_player_game};

    /// Melek's power/toughness is twice the instant and sorcery cards in his
    /// controller's graveyard.
    #[test]
    fn melek_pt_scales_with_instants_and_sorceries() {
        let mut g = two_player_game();
        let melek = g.add_card_to_battlefield(0, catalog::melek_reforged_researcher());
        assert_eq!(g.computed_permanent(melek).map(|c| (c.power, c.toughness)), Some((0, 0)));
        g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
        g.add_card_to_graveyard(0, catalog::divination()); // sorcery
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature — ignored
        assert_eq!(
            g.computed_permanent(melek).map(|c| (c.power, c.toughness)),
            Some((4, 4)),
            "2 I/S cards × 2",
        );
    }

    /// Melek makes the first instant/sorcery spell each turn cost {3} less, but not
    /// creature spells or the second I/S spell.
    #[test]
    fn melek_discounts_first_instant_or_sorcery() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::melek_reforged_researcher());
        let bolt = CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
        let bear = CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 3, "first I/S → {{3}} off");
        assert_eq!(cost_reduction_for_spell(&g, 0, &bear, None), 0, "creature spell unaffected");
        g.players[0].instants_or_sorceries_cast_this_turn = 1;
        assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 0, "second I/S → no discount");
    }

    /// Incinerator of the Guilty collects evidence on combat damage and deals X to
    /// each creature/planeswalker the damaged player controls, where X is the total
    /// mana value exiled.
    #[test]
    fn incinerator_collects_evidence_and_burns_the_board() {
        let mut g = two_player_game();
        let incinerator = g.add_card_to_battlefield(0, catalog::incinerator_of_the_guilty());
        // Fuel: two MV-2 cards → X = 4 when the bot exiles the whole graveyard.
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        // Opponent's board: a 2/2 and a 5/5 (survives X=4).
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
        // Opt into the "collect evidence X" reflexive (bot path exiles the whole gy).
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.fire_combat_damage_to_player_triggers(incinerator, 1, 6);
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "2/2 took 4 and died");
        assert!(g.battlefield_find(big).is_some(), "6/6 survived 4 damage");
        assert_eq!(g.players[0].graveyard.len(), 0, "evidence exiled the graveyard");
    }
}

mod recent255 {
    use crabomination::card::{CardDefinition, CardType, CounterType, CreatureType, Subtypes};
    use crabomination::catalog;
    use crabomination::game::actions::cost_reduction_for_spell;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::{cost, w};

    fn detective(name: &'static str) -> CardDefinition {
        CardDefinition {
            name,
            cost: cost(&[w()]),
            card_types: vec![CardType::Creature],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Detective],
                ..Default::default()
            },
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

    /// Ransacked Lab discounts instants/sorceries and solves after four are cast.
    #[test]
    fn ransacked_lab_discounts_and_solves() {
        let mut g = two_player_game();
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_ransacked_lab());
        let bolt = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 1, "I/S cost {{1}} less");
        solve_now(&mut g);
        assert!(!is_solved(&g, case), "unsolved with 0 I/S cast");
        g.players[0].instants_or_sorceries_cast_this_turn = 4;
        solve_now(&mut g);
        assert!(is_solved(&g, case), "solved after casting 4 I/S");
    }

    /// Stashed Skeleton mints a suspected Skeleton on ETB; it stays unsolved while
    /// that Skeleton lives and solves once none remain.
    #[test]
    fn stashed_skeleton_etb_and_solve() {
        let mut g = two_player_game();
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_stashed_skeleton());
        g.fire_self_etb_triggers(case, 0);
        drain_stack(&mut g);
        let skele = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Skeleton" && c.controller == 0)
            .expect("Skeleton token minted");
        assert!(skele.suspected, "the Skeleton token is suspected");
        let skele_id = skele.id;
        solve_now(&mut g);
        assert!(!is_solved(&g, case), "unsolved while a suspected Skeleton lives");
        g.battlefield.retain(|c| c.id != skele_id);
        solve_now(&mut g);
        assert!(is_solved(&g, case), "solved once no suspected Skeletons remain");
    }

    /// Pilfered Proof counters entering Detectives and solves at three.
    #[test]
    fn pilfered_proof_counters_detectives_and_solves() {
        let mut g = two_player_game();
        let case = g.add_card_to_battlefield(0, catalog::case_of_the_pilfered_proof());
        let d1 = g.add_card_to_battlefield(0, detective("Sleuth A"));
        g.dispatch_triggers_for_events(&[crabomination::game::GameEvent::PermanentEntered {
            card_id: d1,
        }]);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(d1).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "entering Detective got a +1/+1 counter",
        );
        g.add_card_to_battlefield(0, detective("Sleuth B"));
        g.add_card_to_battlefield(0, detective("Sleuth C"));
        solve_now(&mut g);
        assert!(is_solved(&g, case), "solved with three Detectives");
    }
}

mod recent256 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::types::{GameAction, TurnStep};
    use crabomination::game::{drain_stack, two_player_game, GameEvent};
    use crabomination::mana::Color;

    /// When a creature card leaves your graveyard, Insidious Roots makes a Plant and
    /// grows every Plant you control.
    #[test]
    fn insidious_roots_makes_plants_on_graveyard_departure() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::insidious_roots());
        let leaver = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::CardLeftGraveyard { player: 0, card_id: leaver }]);
        drain_stack(&mut g);
        let plant = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Plant" && c.controller == 0)
            .expect("a Plant token was created");
        assert_eq!(
            plant.counter_count(CounterType::PlusOnePlusOne),
            1,
            "the new Plant got a +1/+1 counter",
        );
    }

    /// Assemble the Players lets you cast one small creature from the top of your
    /// library each turn; a second top-of-library cast is blocked.
    #[test]
    fn assemble_the_players_casts_one_creature_from_top() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::assemble_the_players());
        let bear1 = g.add_card_to_library(0, catalog::grizzly_bears()); // power 2 — castable
        let bear2 = g.add_card_to_library(0, catalog::grizzly_bears());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(Color::Green, 4);
        g.perform_action(GameAction::CastSpell {
            card_id: bear1,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("first small creature cast from the top succeeds");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear1).is_some(), "first bear resolved onto the battlefield");
        assert!(
            g.perform_action(GameAction::CastSpell {
                card_id: bear2,
                target: None,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            })
            .is_err(),
            "second top-of-library cast is blocked by the once-per-turn cap",
        );
    }
}

mod recent257 {
    use crabomination::catalog;
    use crabomination::game::types::{GameAction, TurnStep};
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Alquist Proft investigates on ETB, then converts a Clue + {X}{W}{U}{U} into
    /// X cards drawn and X life.
    #[test]
    fn alquist_proft_investigates_then_draws_x() {
        let mut g = two_player_game();
        let proft = g.add_card_to_battlefield(0, catalog::alquist_proft_master_sleuth());
        g.fire_self_etb_triggers(proft, 0);
        drain_stack(&mut g);
        let clues = g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count();
        assert_eq!(clues, 1, "ETB investigated for one Clue");

        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.battlefield_find_mut(proft).unwrap().summoning_sick = false; // able to tap
        g.players[0].mana_pool.add_colorless(2); // the {X}=2 generic
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Blue, 2);
        let hand_before = g.players[0].hand.len();
        let life_before = g.players[0].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: proft,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: Some(2),
        })
        .expect("activate the draw-X ability with X=2");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew X=2 cards");
        assert_eq!(g.players[0].life, life_before + 2, "gained X=2 life");
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count(),
            0,
            "the Clue was sacrificed as a cost",
        );
    }
}

mod recent258 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
    use crabomination::game::{drain_stack, two_player_game, GameState};

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Fuss (the left half) grows only your attacking creatures.
    #[test]
    fn fuss_pumps_attackers() {
        let mut g = two_player_game();
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let idle = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(attacker);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker,
            target: AttackTarget::Player(1),
        }]))
        .expect("declare attacker");
        drain_stack(&mut g);
        let def = catalog::fuss_bother();
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let evs = g.resolve_effect(&def.effect, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        assert_eq!(
            g.battlefield_find(attacker).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "the attacker got a +1/+1 counter",
        );
        assert_eq!(
            g.battlefield_find(idle).unwrap().counter_count(CounterType::PlusOnePlusOne),
            0,
            "the non-attacker was untouched",
        );
    }

    /// Bother (the right half) makes three Thopters and surveils.
    #[test]
    fn bother_makes_thopters() {
        let mut g = two_player_game();
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let right = catalog::fuss_bother().split.unwrap().right.effect.clone();
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&right, &ctx).unwrap();
        drain_stack(&mut g);
        let thopters = g.battlefield.iter().filter(|c| c.definition.name == "Thopter" && c.controller == 0).count();
        assert_eq!(thopters, 3, "created three Thopters");
    }

    /// Cease (the left half) exiles graveyard cards and gives the target player 2
    /// life plus a card.
    #[test]
    fn cease_exiles_and_refills() {
        let mut g = two_player_game();
        let gy1 = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let gy2 = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::mountain());
        let life = g.players[0].life;
        let hand = g.players[0].hand.len();
        // Choose both graveyard cards to exile.
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Cards(vec![gy1, gy2]),
        ]));
        let ctx = EffectContext::for_spell(0, Some(Target::Player(0)), 0, 0);
        g.resolve_effect(&catalog::cease_desist().effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), 0, "up to two graveyard cards exiled");
        assert_eq!(g.players[0].life, life + 2, "target player gained 2 life");
        assert_eq!(g.players[0].hand.len(), hand + 1, "target player drew a card");
    }

    /// Desist (the right half) destroys all artifacts and enchantments.
    #[test]
    fn desist_wipes_artifacts_and_enchantments() {
        let mut g = two_player_game();
        let clue = g.add_card_to_battlefield(0, catalog::insidious_roots()); // enchantment
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // creature — spared
        let artifact = g.add_card_to_battlefield(1, catalog::assemble_the_players()); // enchantment
        let right = catalog::cease_desist().split.unwrap().right.effect.clone();
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let evs = g.resolve_effect(&right, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        g.check_state_based_actions();
        assert!(g.battlefield_find(clue).is_none(), "enchantment destroyed");
        assert!(g.battlefield_find(artifact).is_none(), "opponent enchantment destroyed");
        assert!(g.battlefield_find(bear).is_some(), "creature spared");
    }
}

mod recent259 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::two_player_game;

    /// Living Conundrum is a 2/5 with a full library and a 10/10 flying, vigilant
    /// beater once the library is empty.
    #[test]
    fn living_conundrum_wakes_on_empty_library() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::living_conundrum());
        g.add_card_to_library(0, catalog::island());
        let base = g.computed_permanent(id).unwrap();
        assert_eq!((base.power, base.toughness), (2, 5), "2/5 while the library has cards");
        assert!(!base.keywords.contains(&Keyword::Flying), "no flying yet");

        g.players[0].library.clear();
        let big = g.computed_permanent(id).unwrap();
        assert_eq!((big.power, big.toughness), (10, 10), "10/10 with an empty library");
        assert!(big.keywords.contains(&Keyword::Flying), "gained flying");
        assert!(big.keywords.contains(&Keyword::Vigilance), "gained vigilance");
        assert!(big.keywords.contains(&Keyword::Hexproof), "still hexproof");
    }
}

mod recent260 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, TurnStep};
    use crabomination::game::{drain_stack, two_player_game};

    /// When Anzrag becomes blocked it untaps your creatures and grants an extra
    /// combat phase.
    #[test]
    fn anzrag_untaps_and_adds_combat_on_block() {
        let mut g = two_player_game();
        let anzrag = g.add_card_to_battlefield(0, catalog::anzrag_the_quake_mole());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(ally).unwrap().tapped = true;
        let before = g.additional_combat_phases;
        let effect = catalog::anzrag_the_quake_mole().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_trigger(anzrag, 0, None, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield_find(ally).unwrap().tapped, "your creatures untapped");
        assert_eq!(g.additional_combat_phases, before + 1, "an extra combat phase was queued");
    }

    /// Anzrag's activated ability makes it must-be-blocked.
    #[test]
    fn anzrag_forces_a_block() {
        let mut g = two_player_game();
        let anzrag = g.add_card_to_battlefield(0, catalog::anzrag_the_quake_mole());
        g.clear_sickness(anzrag);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(3);
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: anzrag,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate the must-be-blocked ability");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(anzrag).unwrap().keywords.contains(&Keyword::MustBeBlocked),
            "Anzrag must be blocked this turn",
        );
    }
}

mod recent261 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target};
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// The ETB exiles a nonland permanent an opponent controls until the Aura leaves.
    #[test]
    fn buried_in_the_garden_exiles_on_etb() {
        let mut g = two_player_game();
        let aura = g.add_card_to_battlefield(0, catalog::buried_in_the_garden());
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let effect = catalog::buried_in_the_garden().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_trigger(aura, 0, Some(Target::Permanent(victim)), 0);
        let evs = g.resolve_effect(&effect, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        assert!(g.battlefield_find(victim).is_none(), "opponent's creature exiled");
    }
}

mod recent262 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::two_player_game;

    /// X damage hits the chosen target and up to X lands come back tapped, drawn
    /// preferentially from the graveyard so hand lands stay playable.
    #[test]
    fn worldsouls_rage_burns_and_ramps() {
        let mut g = two_player_game();
        // Two graveyard lands + one hand land available; X = 2 deploys the two
        // graveyard lands and leaves the hand land untouched.
        g.add_card_to_graveyard(0, catalog::forest());
        g.add_card_to_graveyard(0, catalog::mountain());
        let hand_land = g.add_card_to_hand(0, catalog::forest());
        let start_life = g.players[1].life;

        let effect = catalog::worldsouls_rage().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 2);
        g.resolve_effect(&effect, &ctx).unwrap();

        assert_eq!(g.players[1].life, start_life - 2, "X=2 damage to the player");
        let lands_out = g.battlefield.iter().filter(|c| c.controller == 0 && c.tapped).count();
        assert_eq!(lands_out, 2, "two graveyard lands deployed tapped");
        assert!(g.players[0].graveyard.iter().all(|c| !c.definition.is_land()), "graveyard lands consumed");
        assert!(g.players[0].hand.iter().any(|c| c.id == hand_land), "hand land untouched at X=2");
    }

    /// Discarding a 5-drop and a 2-drop deals 5 (the greatest MV, not the
    /// last-discarded 2) to each creature — a 4/4 dies.
    #[test]
    fn ill_timed_explosion_scales_by_greatest_discarded_mv() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        // Hand's two highest-MV cards (auto-discarded): a 5-drop + a 2-drop.
        g.add_card_to_hand(0, catalog::serra_angel()); // MV 5
        g.add_card_to_hand(0, catalog::grizzly_bears()); // MV 2
        // Library feeds the "draw two" so the discard picks from the intended pair.
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

        let effect = catalog::ill_timed_explosion().effect;
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.battlefield_find(victim).is_none(), "5 damage (greatest MV) killed the 4/4");
    }
}

mod recent263 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;

    /// Discarding a nonland card fires the reflexive 3-damage bolt (a 2/2 dies).
    #[test]
    fn glacial_dragonhunt_bolts_on_nonland_discard() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        g.add_card_to_hand(0, catalog::serra_angel()); // MV 5 nonland — auto-discarded
        g.add_card_to_library(0, catalog::forest()); // the "draw a card"
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

        let effect = catalog::glacial_dragonhunt().effect;
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.battlefield_find(victim).is_none(), "3 damage killed the 2/2");
    }
}

mod recent264 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target};
    use crabomination::game::{drain_stack, two_player_game};

    fn kw(g: &crabomination::game::GameState, id: crabomination::card::CardId, k: Keyword) -> bool {
        g.computed_permanent(id).is_some_and(|cp| cp.keywords.contains(&k))
    }

    /// Alabaster Host Sanctifier is a 2/2 with lifelink.
    #[test]
    fn alabaster_host_sanctifier_has_lifelink() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::alabaster_host_sanctifier());
        assert!(kw(&g, id, Keyword::Lifelink));
    }

    /// Nezumi Informant's ETB makes each opponent discard a card.
    #[test]
    fn nezumi_informant_opponent_discards() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::forest());
        let before = g.players[1].hand.len();
        let effect = catalog::nezumi_informant().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[1].hand.len(), before - 1, "opponent discarded one");
    }

    /// Preening Champion flies and mints a 1/1 U/R Elemental on ETB.
    #[test]
    fn preening_champion_makes_elemental() {
        let mut g = two_player_game();
        let champ = g.add_card_to_battlefield(0, catalog::preening_champion());
        assert!(kw(&g, champ, Keyword::Flying));
        let effect = catalog::preening_champion().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Elemental"),
            "Elemental token created"
        );
    }

    /// Knight of the New Coalition's ETB makes a vigilant Knight token.
    #[test]
    fn knight_of_new_coalition_makes_vigilant_knight() {
        let mut g = two_player_game();
        let effect = catalog::knight_of_the_new_coalition().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        let tok = g.battlefield.iter().find(|c| c.definition.name == "Knight").expect("Knight token");
        assert!(tok.definition.keywords.contains(&Keyword::Vigilance), "vigilant Knight");
    }

    /// Conscripted Infantry's real death makes a 1/1 Soldier artifact creature.
    #[test]
    fn conscripted_infantry_dies_into_soldier() {
        let mut g = two_player_game();
        let inf = g.add_card_to_battlefield(0, catalog::conscripted_infantry());
        // Destroy via the effect path, then dispatch the self-death trigger.
        let effect = crabomination::effect::Effect::Destroy {
            what: crabomination::effect::Selector::Target(0),
        };
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(inf)), 0, 0);
        let evs = g.resolve_effect(&effect, &ctx).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let tok = g.battlefield.iter().find(|c| c.definition.name == "Soldier").expect("Soldier token");
        assert!(tok.definition.card_types.contains(&crabomination::card::CardType::Artifact));
    }

    /// Burrowing Razormaw mills four when it dies.
    #[test]
    fn burrowing_razormaw_dies_mills_four() {
        let mut g = two_player_game();
        for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
        let before = g.players[0].graveyard.len();
        let effect = catalog::burrowing_razormaw().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[0].graveyard.len(), before + 4, "milled four");
    }

    /// Hoarding Recluse has reach + deathtouch and bottoms a graveyard card on death.
    #[test]
    fn hoarding_recluse_bottoms_graveyard_card() {
        let mut g = two_player_game();
        let recluse = g.add_card_to_battlefield(0, catalog::hoarding_recluse());
        assert!(kw(&g, recluse, Keyword::Reach) && kw(&g, recluse, Keyword::Deathtouch));
        let buried = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let lib_before = g.players[1].library.len();
        let effect = catalog::hoarding_recluse().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(buried)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[1].library.len(), lib_before + 1, "card moved to owner's library");
        assert!(g.players[1].graveyard.iter().all(|c| c.id != buried), "left the graveyard");
    }

    /// Fallaji Chaindancer buys double strike with {2}.
    #[test]
    fn fallaji_chaindancer_grants_double_strike() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::fallaji_chaindancer());
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert!(kw(&g, id, Keyword::DoubleStrike));
    }

    /// Iridescent Blademaster pumps itself +2/+2 with {3}{G}.
    #[test]
    fn iridescent_blademaster_firebreathes() {
        use crabomination::mana::Color;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::iridescent_blademaster());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "2/2 became 4/4");
    }

    /// Air Marshal grants flying to a target Soldier.
    #[test]
    fn air_marshal_grants_flying_to_soldier() {
        let mut g = two_player_game();
        let marshal = g.add_card_to_battlefield(0, catalog::air_marshal());
        let ally = g.add_card_to_battlefield(0, catalog::alabaster_host_sanctifier()); // not a Soldier
        let soldier = g.add_card_to_battlefield(0, catalog::conscripted_infantry()); // Soldier
        let _ = ally;
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: marshal, ability_index: 0,
            target: Some(Target::Permanent(soldier)), additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert!(kw(&g, soldier, Keyword::Flying), "the Soldier gained flying");
    }

    /// Onakke Javelineer taps to deal 2 to a player.
    #[test]
    fn onakke_javelineer_pings_a_player() {
        let mut g = two_player_game();
        let jav = g.add_card_to_battlefield(0, catalog::onakke_javelineer());
        g.battlefield_find_mut(jav).unwrap().summoning_sick = false;
        let before = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: jav, ability_index: 0,
            target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, before - 2);
    }

    /// CR 310.10 — Onakke Javelineer's noncombat ping removes defense counters
    /// from a battle (the noncombat analogue of combat damage to a battle).
    #[test]
    fn onakke_javelineer_damages_a_battle() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let battle = g.add_card_to_battlefield(0, catalog::invasion_of_zendikar());
        {
            let b = g.battlefield_find_mut(battle).unwrap();
            b.counters.insert(CounterType::Defense, 3);
            b.protected_by = Some(1);
        }
        let jav = g.add_card_to_battlefield(0, catalog::onakke_javelineer());
        g.battlefield_find_mut(jav).unwrap().summoning_sick = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: jav, ability_index: 0,
            target: Some(Target::Permanent(battle)), additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(battle).unwrap().counter_count(CounterType::Defense),
            1,
            "2 damage removed two defense counters"
        );
    }

    /// Dreg Recycler sacrifices to drain one.
    #[test]
    fn dreg_recycler_drains_one() {
        let mut g = two_player_game();
        let dreg = g.add_card_to_battlefield(0, catalog::dreg_recycler());
        g.battlefield_find_mut(dreg).unwrap().summoning_sick = false;
        let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let (my_life, opp_life) = (g.players[0].life, g.players[1].life);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dreg, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_life - 1, "opponent lost one");
        assert_eq!(g.players[0].life, my_life + 1, "you gained one");
    }

    /// Coming In Hot pumps +1/+0, grants first strike, and scries.
    #[test]
    fn coming_in_hot_pumps_and_grants_first_strike() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.add_card_to_library(0, catalog::forest());
        let effect = catalog::coming_in_hot().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 3, "+1/+0");
        assert!(cp.keywords.contains(&Keyword::FirstStrike));
    }

    /// Cosmic Hunger makes your creature deal its power to another creature.
    #[test]
    fn cosmic_hunger_bites_with_power() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let effect = catalog::cosmic_hunger().effect;
        let mut ctx = EffectContext::for_spell(0, None, 0, 0);
        ctx.targets = vec![Target::Permanent(mine), Target::Permanent(foe)];
        g.resolve_effect(&effect, &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.battlefield_find(foe).is_none(), "4 damage killed the 2/2");
    }

    /// Mirrodin Avenged destroys a damaged creature and draws.
    #[test]
    fn mirrodin_avenged_destroys_damaged_and_draws() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.battlefield_find_mut(foe).unwrap().dealt_damage_this_turn = true;
        g.add_card_to_library(0, catalog::forest());
        let hand_before = g.players[0].hand.len();
        let effect = catalog::mirrodin_avenged().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(foe)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.battlefield_find(foe).is_none(), "destroyed the damaged creature");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    }

    /// Atraxa's Fall destroys a flyer.
    #[test]
    fn atraxas_fall_destroys_flyer() {
        let mut g = two_player_game();
        let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // flying
        let effect = catalog::atraxas_fall().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(flyer)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        g.check_state_based_actions();
        assert!(g.battlefield_find(flyer).is_none(), "destroyed the flyer");
    }

    /// Furnace Host Charger has haste and mountaincycling.
    #[test]
    fn furnace_host_charger_haste_and_landcycling() {
        let g = two_player_game();
        let def = catalog::furnace_host_charger();
        assert!(def.keywords.contains(&Keyword::Haste));
        assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Landcycling(..))));
        let _ = g;
    }

    /// Phyrexian Pegasus grants flying to a nonflying attacker.
    #[test]
    fn phyrexian_pegasus_lifts_a_grounded_attacker() {
        let mut g = two_player_game();
        let _peg = g.add_card_to_battlefield(0, catalog::phyrexian_pegasus());
        let grunt = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // no flying
        let effect = catalog::phyrexian_pegasus().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(grunt)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(kw(&g, grunt, Keyword::Flying), "grounded attacker gained flying");
    }
}

mod recent265 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target};
    use crabomination::game::{drain_stack, two_player_game};

    fn kw(g: &crabomination::game::GameState, id: crabomination::card::CardId, k: Keyword) -> bool {
        g.computed_permanent(id).is_some_and(|cp| cp.keywords.contains(&k))
    }

    /// A resolution context whose source (and kicked flag) is `id`.
    fn src_ctx(controller: usize, id: crabomination::card::CardId, kicked: bool) -> EffectContext {
        let mut ctx = EffectContext::for_spell(controller, None, 0, 0);
        ctx.source = Some(id);
        ctx.kicked = kicked;
        ctx
    }

    /// Bonebreaker Giant is a vanilla 4/4.
    #[test]
    fn bonebreaker_giant_is_a_vanilla_4_4() {
        let d = catalog::bonebreaker_giant();
        assert_eq!((d.power, d.toughness), (4, 4));
        assert!(d.keywords.is_empty() && d.triggered_abilities.is_empty());
    }

    /// Gnottvold Recluse has reach.
    #[test]
    fn gnottvold_recluse_has_reach() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::gnottvold_recluse());
        assert!(kw(&g, id, Keyword::Reach));
    }

    /// Deathbloom Gardener taps for any color.
    #[test]
    fn deathbloom_gardener_makes_mana() {
        use crabomination::mana::Color;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::deathbloom_gardener());
        g.clear_sickness(id);
        assert!(kw(&g, id, Keyword::Deathtouch));
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0,
            target: None, additional_targets: vec![], x_value: None,
        }).unwrap();
        let total: u32 = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
            .iter().map(|c| g.players[0].mana_pool.amount(*c)).sum::<u32>()
            + g.players[0].mana_pool.colorless_amount();
        assert_eq!(total, 1, "produced one mana");
    }

    /// Battlefly Swarm buys deathtouch with {B}.
    #[test]
    fn battlefly_swarm_grants_deathtouch() {
        use crabomination::mana::Color;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::battlefly_swarm());
        assert!(kw(&g, id, Keyword::Flying));
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert!(kw(&g, id, Keyword::Deathtouch));
    }

    /// Duct Crawler stops a creature from blocking it this turn.
    #[test]
    fn duct_crawler_locks_a_blocker() {
        use crabomination::game::types::{Attack, AttackTarget, TurnStep};
        let mut g = two_player_game();
        let crawler = g.add_card_to_battlefield(0, catalog::duct_crawler());
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(crawler);
        g.players[0].mana_pool.add_colorless(1);
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: crawler, ability_index: 0,
            target: Some(Target::Permanent(blocker)), additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: crawler, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        assert!(
            g.perform_action(GameAction::DeclareBlockers(vec![(blocker, crawler)])).is_err(),
            "the locked creature can't block the crawler"
        );
    }

    /// Charismatic Vanguard pumps the whole team +1/+1.
    #[test]
    fn charismatic_vanguard_pumps_team() {
        use crabomination::mana::Color;
        let mut g = two_player_game();
        let van = g.add_card_to_battlefield(0, catalog::charismatic_vanguard());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::ActivateAbility {
            card_id: van, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        let cp = g.computed_permanent(ally).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "ally pumped to 3/3");
    }

    /// Cabaretti Initiate buys double strike with its hybrid ability.
    #[test]
    fn cabaretti_initiate_gains_double_strike() {
        use crabomination::mana::Color;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::cabaretti_initiate());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert!(kw(&g, id, Keyword::DoubleStrike));
    }

    /// Serpent-Blade Assailant's Backup puts a counter on and grants deathtouch.
    #[test]
    fn serpent_blade_backup_buffs_ally() {
        let mut g = two_player_game();
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::serpent_blade_assailant().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(ally)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(
            g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
            "+1/+1 counter placed"
        );
        assert!(kw(&g, ally, Keyword::Deathtouch), "granted deathtouch");
    }

    /// Rhox Pikemaster gives other Soldiers first strike.
    #[test]
    fn rhox_pikemaster_soldier_anthem() {
        let mut g = two_player_game();
        let _rhox = g.add_card_to_battlefield(0, catalog::rhox_pikemaster());
        let soldier = g.add_card_to_battlefield(0, catalog::conscripted_infantry()); // Soldier
        let nonsoldier = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert!(kw(&g, soldier, Keyword::FirstStrike), "other Soldier gets first strike");
        assert!(!kw(&g, nonsoldier, Keyword::FirstStrike), "non-Soldier unaffected");
    }

    /// Witty Roastmaster pings each opponent when another creature enters.
    #[test]
    fn witty_roastmaster_pings_on_creature_etb() {
        use crabomination::mana::Color;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::witty_roastmaster());
        let before = g.players[1].life;
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a creature");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, before - 1, "1 damage to the opponent");
    }

    /// Yavimaya Iconoclast pumps itself when kicked.
    #[test]
    fn yavimaya_iconoclast_kicked_pumps() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::yavimaya_iconoclast());
        let effect = catalog::yavimaya_iconoclast().triggered_abilities[0].effect.clone();
        let ctx = src_ctx(0, id, true);
        g.resolve_effect(&effect, &ctx).unwrap();
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 3), "+1/+1 when kicked");
        assert!(cp.keywords.contains(&Keyword::Haste));
    }

    /// Vineshaper Prodigy digs three when kicked.
    #[test]
    fn vineshaper_prodigy_kicked_digs() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::vineshaper_prodigy());
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        let hand_before = g.players[0].hand.len();
        let effect = catalog::vineshaper_prodigy().triggered_abilities[0].effect.clone();
        let ctx = src_ctx(0, id, true);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "took one card to hand");
    }

    /// Shield-Wall Sentinel has defender and can tutor a defender to hand.
    #[test]
    fn shield_wall_sentinel_tutors_defender() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let wall = g.add_card_to_library(0, catalog::wall_of_omens());
        let sentinel_def = catalog::shield_wall_sentinel();
        assert!(sentinel_def.keywords.contains(&Keyword::Defender));
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Search(Some(wall)),
        ]));
        let effect = sentinel_def.triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.players[0].hand.iter().any(|c| c.id == wall), "defender fetched to hand");
    }

    /// Kami of Industry reanimates a cheap artifact, hastes it, and sacs it at end.
    #[test]
    fn kami_of_industry_reanimates_artifact() {
        use crabomination::game::types::TurnStep;
        let mut g = two_player_game();
        let relic = g.add_card_to_graveyard(0, catalog::mind_stone()); // MV 2 artifact
        let effect = catalog::kami_of_industry().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(relic)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.battlefield_find(relic).is_some(), "artifact reanimated");
        assert!(kw(&g, relic, Keyword::Haste), "gained haste");
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.battlefield_find(relic).is_none(), "sacrificed at the end step");
    }

    /// Wingmantle Chaplain mints a Bird per defender on ETB.
    #[test]
    fn wingmantle_chaplain_makes_birds_per_defender() {
        let mut g = two_player_game();
        // Two defenders already out (plus the Chaplain would be a third once it enters).
        g.add_card_to_battlefield(0, catalog::wall_of_omens());
        g.add_card_to_battlefield(0, catalog::shield_wall_sentinel());
        let chaplain = g.add_card_to_battlefield(0, catalog::wingmantle_chaplain());
        let effect = catalog::wingmantle_chaplain().triggered_abilities[0].effect.clone();
        let ctx = src_ctx(0, chaplain, false);
        g.resolve_effect(&effect, &ctx).unwrap();
        let birds = g.battlefield.iter().filter(|c| c.definition.name == "Bird").count();
        assert_eq!(birds, 3, "one Bird per defender (two walls + the Chaplain)");
    }
}

mod recent266 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target};
    use crabomination::game::{drain_stack, two_player_game};

    fn kw(g: &crabomination::game::GameState, id: crabomination::card::CardId, k: Keyword) -> bool {
        g.computed_permanent(id).is_some_and(|cp| cp.keywords.contains(&k))
    }

    /// Fungal Infection shrinks a creature and makes a Saproling.
    #[test]
    fn fungal_infection_shrinks_and_spawns() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let effect = catalog::fungal_infection().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(victim)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        let cp = g.computed_permanent(victim).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "-1/-1 applied");
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Saproling"),
            "Saproling token created"
        );
    }

    /// Prakhata Pillar-Bug buys lifelink with {B}.
    #[test]
    fn prakhata_pillar_bug_gains_lifelink() {
        use crabomination::mana::Color;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::prakhata_pillar_bug());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).unwrap();
        drain_stack(&mut g);
        assert!(kw(&g, id, Keyword::Lifelink));
    }

    /// Savai Sabertooth is a vanilla 3/1.
    #[test]
    fn savai_sabertooth_is_vanilla() {
        let d = catalog::savai_sabertooth();
        assert_eq!((d.power, d.toughness), (3, 1));
        assert!(d.keywords.is_empty() && d.triggered_abilities.is_empty());
    }

    /// Territorial Boar grows when a big creature enters under your control.
    #[test]
    fn territorial_boar_grows_on_big_creature() {
        use crabomination::mana::Color;
        let mut g = two_player_game();
        let boar = g.add_card_to_battlefield(0, catalog::territorial_boar());
        // A 4/4 entering (cast, so the watcher trigger fires) grows the boar.
        let angel = g.add_card_to_hand(0, catalog::serra_angel()); // 4/4, {3}{W}{W}
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast the angel");
        drain_stack(&mut g);
        let cp = g.computed_permanent(boar).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "boar pumped to 3/3");
        assert!(cp.keywords.contains(&Keyword::Vigilance), "gained vigilance");
    }

    /// Might of Murasa gives +3/+3, or +5/+5 when kicked.
    #[test]
    fn might_of_murasa_pumps_scaled_by_kicker() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let effect = catalog::might_of_murasa().effect;
        // Unkicked: +3/+3 → 5/5.
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "+3/+3 base");
        // Kicked: +5/+5 instead → another +5 power on top.
        let mut kicked = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        kicked.kicked = true;
        g.resolve_effect(&effect, &kicked).unwrap();
        assert_eq!(g.computed_permanent(bear).unwrap().power, 10, "kicked adds +5/+5");
    }
}

mod recent267 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::effect::{Effect, Selector};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target};
    use crabomination::game::{drain_stack, two_player_game, CardId, GameState};
    use crabomination::mana::Color;

    fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(g);
    }

    /// Akki Scrapchomper sacrifices an artifact to draw.
    #[test]
    fn akki_scrapchomper_sacs_for_a_card() {
        let mut g = two_player_game();
        let akki = g.add_card_to_battlefield(0, catalog::akki_scrapchomper());
        g.clear_sickness(akki); // {T} cost needs no summoning sickness
        let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: akki,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        assert!(g.battlefield_find(stone).is_none(), "artifact sacrificed");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    }

    /// Argothian Opportunist makes a tapped Powerstone on ETB.
    #[test]
    fn argothian_opportunist_makes_tapped_powerstone() {
        let mut g = two_player_game();
        let opp = g.add_card_to_hand(0, catalog::argothian_opportunist());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, opp, None);
        let ps = g
            .battlefield
            .iter()
            .find(|c| c.controller == 0 && c.definition.name == "Powerstone")
            .expect("Powerstone token");
        assert!(ps.tapped, "Powerstone enters tapped");
    }

    /// Ashnod's Intervention pumps and returns the creature to hand when it dies.
    #[test]
    fn ashnods_intervention_returns_on_death() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let effect = catalog::ashnods_intervention().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0");
        // Destroy it — the granted trigger returns it to its owner's hand.
        let dctx = EffectContext::for_ability(bear, 0, Some(Target::Permanent(bear)));
        g.resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &dctx).unwrap();
        g.dispatch_triggers_for_events(&[]);
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "creature left the battlefield");
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "returned to owner's hand"
        );
    }

    /// Gnawing Crescendo pumps the team and spawns a Rat when a nontoken dies.
    #[test]
    fn gnawing_crescendo_pumps_and_makes_rats() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let effect = catalog::gnawing_crescendo().effect;
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "team +2/+0");
        // The nontoken bear dies (via SBA) → a Rat appears.
        g.battlefield_find_mut(bear).unwrap().damage = 100;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let rat = g
            .battlefield
            .iter()
            .find(|c| c.controller == 0 && c.definition.name == "Rat")
            .expect("Rat token");
        assert!(rat.definition.keywords.contains(&Keyword::CantBlock), "Rat can't block");
    }

    /// Angelic Intervention grants protection and a +1/+1 counter.
    #[test]
    fn angelic_intervention_protects_and_counters() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Black)]));
        let effect = catalog::angelic_intervention().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 counter");
        assert!(
            g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Protection(Color::Black)),
            "protected from black"
        );
    }

    /// Alabaster Host Intercessor exiles an opponent's creature until it leaves.
    #[test]
    fn alabaster_host_intercessor_exiles_until_leaves() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // opponent 4/4
        let inter = g.add_card_to_hand(0, catalog::alabaster_host_intercessor());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(5);
        cast(&mut g, inter, Some(Target::Permanent(victim)));
        assert!(g.battlefield_find(victim).is_none(), "opponent creature exiled");
        // Destroy the Intercessor → the exiled creature returns.
        let iid = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Alabaster Host Intercessor")
            .unwrap()
            .id;
        let dctx = EffectContext::for_ability(iid, 0, Some(Target::Permanent(iid)));
        g.resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &dctx).unwrap();
        g.dispatch_triggers_for_events(&[]);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Serra Angel"),
            "exiled creature returned"
        );
    }
}
