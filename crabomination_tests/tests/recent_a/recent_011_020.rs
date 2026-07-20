//! Tests for recentN card batches 11-20 (merged from per-batch micro-files).

mod recent11 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Earthbending Lesson earthbends 4 from a sorcery.
    #[test]
    fn earthbending_lesson_earthbends_four() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::earthbending_lesson());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast_at(&mut g, id, Target::Permanent(land));
        assert_eq!(
            g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
            4
        );
    }

    /// Dai Li Indoctrination's earthbend mode (mode 1) earthbends 2.
    #[test]
    fn dai_li_indoctrination_earthbend_mode() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let id = g.add_card_to_hand(0, catalog::dai_li_indoctrination());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Permanent(land)),
            additional_targets: vec![],
            mode: Some(1),
            x_value: None,
        })
        .expect("cast earthbend mode");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(land).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2
        );
    }
}

mod recent12 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;

    /// Attach `eq` to `creature` directly (test shortcut, bypassing the equip action).
    fn attach(g: &mut GameState, eq: crabomination::card::CardId, creature: crabomination::card::CardId) {
        g.battlefield.iter_mut().find(|c| c.id == eq).unwrap().attached_to = Some(creature);
    }

    /// Leonin Shikari lets equip happen at instant speed (here: during combat).
    #[test]
    fn leonin_shikari_allows_instant_speed_equip() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
        g.add_card_to_battlefield(0, catalog::leonin_shikari());
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::DeclareAttackers; // not sorcery-speed
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::Equip { equipment: boner, target: bear })
            .expect("Leonin Shikari permits equip at instant speed");
        assert_eq!(g.battlefield_find(boner).unwrap().attached_to, Some(bear));
    }

    /// Auriok Steelshaper makes equip cost {1} less.
    #[test]
    fn auriok_steelshaper_reduces_equip_cost() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Vulshok Morningstar's equip is {2}; with Auriok it drops to {1}.
        let star = g.add_card_to_battlefield(0, catalog::vulshok_morningstar());
        g.add_card_to_battlefield(0, catalog::auriok_steelshaper());
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::Equip { equipment: star, target: bear })
            .expect("equip {2} reduced to {1}");
        assert_eq!(g.battlefield_find(star).unwrap().attached_to, Some(bear));
    }

    /// Kemba mints a Cat token for each Equipment attached to her at upkeep.
    #[test]
    fn kemba_makes_cat_per_attached_equipment() {
        let mut g = two_player_game();
        let kemba = g.add_card_to_battlefield(0, catalog::kemba_kha_regent());
        let e1 = g.add_card_to_battlefield(0, catalog::bonesplitter());
        let e2 = g.add_card_to_battlefield(0, catalog::vulshok_morningstar());
        attach(&mut g, e1, kemba);
        attach(&mut g, e2, kemba);
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        let cats = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Cat").count();
        assert_eq!(cats, 2, "one Cat per attached Equipment");
    }

    /// Goblin Gaveleer gains +2/+0 per attached Equipment.
    #[test]
    fn goblin_gaveleer_grows_per_equipment() {
        let mut g = two_player_game();
        let gav = g.add_card_to_battlefield(0, catalog::goblin_gaveleer());
        assert_eq!(g.computed_permanent(gav).unwrap().power, 1, "1/1 with nothing attached");
        // Cathar's Shield is +0/+3, so the +2 power is purely the per-Equipment bonus.
        let eq = g.add_card_to_battlefield(0, catalog::cathars_shield());
        attach(&mut g, eq, gav);
        assert_eq!(g.computed_permanent(gav).unwrap().power, 3, "+2/+0 from one Equipment");
    }

    /// Danitha makes Equipment spells cost {1} less — Leonin Scimitar ({1}) becomes
    /// free.
    #[test]
    fn danitha_reduces_equipment_spell_cost() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::danitha_capashen());
        let scimitar = g.add_card_to_hand(0, catalog::leonin_scimitar());
        // No mana floated; {1} - {1} = {0}, so it should still cast.
        g.priority.player_with_priority = 0;
        cast(&mut g, scimitar);
        assert!(g.battlefield_find(scimitar).is_some(), "discounted Equipment spell resolved");
    }

    /// Maul of the Skyclaves attaches itself to a creature on ETB and grants flying.
    #[test]
    fn maul_etb_attaches_and_grants_flying() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let maul = g.move_card_to_battlefield_for_test(0, catalog::maul_of_the_skyclaves());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(maul).unwrap().attached_to, Some(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
        assert!(cp.keywords.contains(&Keyword::Flying));
    }

    /// Embercleave attaches on ETB and grants double strike + trample.
    #[test]
    fn embercleave_etb_attaches_double_strike() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let cleave = g.move_card_to_battlefield_for_test(0, catalog::embercleave());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(cleave).unwrap().attached_to, Some(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert!(cp.keywords.contains(&Keyword::DoubleStrike) && cp.keywords.contains(&Keyword::Trample));
    }

    /// Armory of Iroas puts a +1/+1 counter on the equipped creature when it attacks.
    #[test]
    fn armory_of_iroas_counter_on_attack() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let armory = g.add_card_to_battlefield(0, catalog::armory_of_iroas());
        attach(&mut g, armory, bear);
        g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().summoning_sick = false;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
            .expect("bear attacks");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Flayer Husk's living weapon mints a 0/0 Germ and attaches itself, making the
    /// Germ a 1/1.
    #[test]
    fn flayer_husk_living_weapon_mints_germ() {
        let mut g = two_player_game();
        let husk = g.move_card_to_battlefield_for_test(0, catalog::flayer_husk());
        drain_stack(&mut g);
        let germ = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Germ").expect("Germ minted");
        let germ_id = germ.id;
        assert_eq!(g.battlefield_find(husk).unwrap().attached_to, Some(germ_id), "Husk attached to its Germ");
        let cp = g.computed_permanent(germ_id).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "0/0 + 1/1 = 1/1");
    }

    /// Lizard Blades is an Equipment creature that grants double strike when attached.
    #[test]
    fn lizard_blades_grants_double_strike_when_attached() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let blades = g.add_card_to_battlefield(0, catalog::lizard_blades());
        attach(&mut g, blades, bear);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike));
    }

    /// Magnetic Theft attaches a target Equipment to a target creature (two-slot
    /// instant).
    #[test]
    fn magnetic_theft_attaches_equipment() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
        let theft = g.add_card_to_hand(0, catalog::magnetic_theft());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: theft,
            target: Some(Target::Permanent(boner)),
            additional_targets: vec![Target::Permanent(bear)],
            mode: None,
            x_value: None,
        })
        .expect("cast Magnetic Theft");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(boner).unwrap().attached_to, Some(bear));
    }

    /// Sram's Expertise makes three Servo tokens.
    #[test]
    fn srams_expertise_makes_three_servos() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::srams_expertise());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        cast(&mut g, id);
        let servos = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Servo").count();
        assert_eq!(servos, 3);
    }

    /// Nahiri's −2 exiles a tapped creature.
    #[test]
    fn nahiri_minus2_exiles_tapped_creature() {
        let mut g = two_player_game();
        let nahiri = g.add_card_to_battlefield(0, catalog::nahiri_the_harbinger());
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield.iter_mut().find(|c| c.id == victim).unwrap().tapped = true;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: nahiri,
            ability_index: 1,
            target: Some(Target::Permanent(victim)),
            x_value: None,
        })
        .expect("activate -2");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "tapped creature exiled");
        assert!(g.players[1].graveyard.iter().all(|c| c.id != victim), "to exile, not graveyard");
    }

    /// Undaunted makes Sublime Exhalation ({6}{W}) cost {1} less per opponent — in
    /// 1v1, {5}{W} — and it wraths the board.
    #[test]
    fn sublime_exhalation_undaunted_and_wraths() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::sublime_exhalation());
        // One opponent → {1} off, so {5}{W} pays the {6}{W} spell.
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(5);
        g.priority.player_with_priority = 0;
        cast(&mut g, id);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.is_creature()).count(),
            0,
            "all creatures destroyed"
        );
    }

    /// Curtains' Call destroys two target creatures.
    #[test]
    fn curtains_call_destroys_two() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::curtains_call());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(4); // {5}{B} - {1} undaunted = {4}{B}
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Permanent(a)),
            additional_targets: vec![Target::Permanent(b)],
            mode: None,
            x_value: None,
        })
        .expect("cast Curtains' Call");
        drain_stack(&mut g);
        assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    }

    /// Balan gains double strike only with two or more Equipment attached, and his
    /// ability rounds up every Equipment.
    #[test]
    fn balan_double_strike_with_two_equipment() {
        let mut g = two_player_game();
        let balan = g.add_card_to_battlefield(0, catalog::balan_wandering_knight());
        let e1 = g.add_card_to_battlefield(0, catalog::bonesplitter());
        let e2 = g.add_card_to_battlefield(0, catalog::bonesplitter());
        assert!(g.computed_permanent(balan).unwrap().keywords.contains(&Keyword::FirstStrike));
        assert!(!g.computed_permanent(balan).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "no double strike unequipped");
        // Activate "attach all Equipment you control" — both pile onto Balan.
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: balan, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("attach all Equipment");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(e1).unwrap().attached_to, Some(balan));
        assert_eq!(g.battlefield_find(e2).unwrap().attached_to, Some(balan));
        assert!(g.computed_permanent(balan).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "double strike with two Equipment");
    }

    /// Valduk mints an Elemental token per attachment at the start of combat.
    #[test]
    fn valduk_makes_token_per_attachment() {
        let mut g = two_player_game();
        let valduk = g.add_card_to_battlefield(0, catalog::valduk_keeper_of_the_flame());
        let eq = g.add_card_to_battlefield(0, catalog::bonesplitter());
        attach(&mut g, eq, valduk);
        g.battlefield.iter_mut().find(|c| c.id == valduk).unwrap().summoning_sick = false;
        g.step = TurnStep::BeginCombat;
        g.priority.player_with_priority = 0;
        g.fire_step_triggers(TurnStep::BeginCombat);
        drain_stack(&mut g);
        let elems = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Elemental").count();
        assert_eq!(elems, 1, "one Elemental per attached Equipment");
        // The transient token is exiled at the next end step.
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        let elems = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Elemental").count();
        assert_eq!(elems, 0, "Elemental exiled at the next end step");
    }
}

mod recent13 {
    use crabomination::card::{CardType, CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;

    /// Misery's Shadow exiles a dying opponent creature instead of letting it hit
    /// the graveyard, and its {1} pump grows it.
    #[test]
    fn miserys_shadow_exiles_dying_opponent_creature_and_pumps() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::miserys_shadow());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.remove_from_battlefield_to_graveyard_raw(foe);
        g.check_state_based_actions();
        assert!(g.players[1].graveyard.iter().all(|c| c.id != foe), "not in graveyard");
        assert!(g.exile.iter().any(|c| c.id == foe), "routed to exile instead");
        // {1} pump.
        let shadow = g.battlefield.iter().find(|c| c.definition.name == "Misery's Shadow").unwrap().id;
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: shadow, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("pump");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(shadow).unwrap().power, 3);
    }

    /// Glarb lets the controller cast a MV-4+ spell off the top of the library.
    #[test]
    fn glarb_casts_high_mv_from_library_top() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::glarb_calamitys_augur());
        // Top of library: a 4-MV creature (Serra Angel is {3}{W}{W} = 5; use a 4-drop).
        g.add_card_to_library(0, catalog::wrath_of_god()); // {2}{W}{W} = MV 4 sorcery
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let top = g.players[0].library[0].id;
        cast(&mut g, top);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "cast from top resolved");
    }

    /// Archfiend enters with four oil counters and an opponent's dying creature
    /// drains its controller 2 life.
    #[test]
    fn archfiend_oil_counters_and_opponent_death_drain() {
        let mut g = two_player_game();
        let arch = g.move_card_to_battlefield_for_test(0, catalog::archfiend_of_the_dross());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(arch).unwrap().counter_count(CounterType::Oil), 4);
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let life = g.players[1].life;
        // Kill the foe through the normal damage/SBA funnel so the death trigger
        // dispatches the same way the live game does.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.priority.player_with_priority = 0;
        cast_at(&mut g, bolt, Target::Permanent(foe));
        assert!(g.battlefield_find(foe).is_none(), "foe died");
        assert_eq!(g.players[1].life, life - 2, "opponent loses 2 when their creature dies");
    }

    /// Archfiend's upkeep removes an oil counter; with one left, removing it makes
    /// the controller lose the game.
    #[test]
    fn archfiend_loses_game_at_zero_oil() {
        let mut g = two_player_game();
        let arch = g.move_card_to_battlefield_for_test(0, catalog::archfiend_of_the_dross());
        drain_stack(&mut g);
        // Drain to a single oil counter, then the next upkeep removal hits zero.
        let inst = g.battlefield.iter_mut().find(|c| c.id == arch).unwrap();
        while inst.counter_count(CounterType::Oil) > 1 {
            inst.remove_counters(CounterType::Oil, 1);
        }
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        g.check_state_based_actions();
        assert!(g.players[0].eliminated, "controller lost the game with no oil counters");
    }

    /// Seeds of Renewal returns up to two cards from the graveyard to hand.
    #[test]
    fn seeds_of_renewal_returns_two_from_graveyard() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        let id = g.add_card_to_hand(0, catalog::seeds_of_renewal());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(5); // {6}{G} - {1} undaunted = {5}{G}
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        cast(&mut g, id);
        assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2, "two cards returned (spell left hand)");
        assert!(g.exile.iter().any(|c| c.definition.name == "Seeds of Renewal"), "self-exiled");
    }

    /// Spara's Headquarters is a GWU Triome that enters tapped with Cycling.
    #[test]
    fn sparas_headquarters_is_a_triome() {
        let d = catalog::sparas_headquarters();
        assert!(d.card_types.contains(&CardType::Land));
        assert_eq!(d.activated_abilities.len(), 3, "taps for three colors");
        assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
    }

    /// Mishra's Foundry animates into a 2/2 Assembly-Worker.
    #[test]
    fn mishras_foundry_animates() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::mishras_foundry());
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        }).expect("animate");
        drain_stack(&mut g);
        let cp = g.computed_permanent(land).unwrap();
        assert!(cp.card_types.contains(&CardType::Creature) && cp.card_types.contains(&CardType::Land));
        assert_eq!((cp.power, cp.toughness), (2, 2));
    }
}

mod recent14 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;

    /// Quirion Beastcaller grows when you cast a creature spell.
    #[test]
    fn quirion_beastcaller_grows_on_creature_cast() {
        let mut g = two_player_game();
        let quirion = g.add_card_to_battlefield(0, catalog::quirion_beastcaller());
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        cast(&mut g, bear);
        assert_eq!(
            g.battlefield_find(quirion).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "a +1/+1 counter per creature spell cast"
        );
    }

    /// Yotian Frontliner buffs another creature you control when it attacks.
    #[test]
    fn yotian_frontliner_buffs_on_attack() {
        let mut g = two_player_game();
        let yotian = g.add_card_to_battlefield(0, catalog::yotian_frontliner());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(yotian);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: yotian, target: AttackTarget::Player(1) }])
            .expect("Yotian attacks");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(ally).unwrap().power, 3, "ally got +1/+1");
    }

    /// Heaped Harvest's ETB fetches a basic land onto the battlefield tapped.
    #[test]
    fn heaped_harvest_etb_fetches_basic_land() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        let id = g.add_card_to_battlefield(0, catalog::heaped_harvest());
        g.fire_self_etb_triggers(id, 0);
        drain_stack(&mut g);
        let lands = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.is_land() && c.tapped).count();
        assert_eq!(lands, 1, "a basic land entered tapped");
    }
}

mod recent15 {
    use crabomination::catalog;
    use crabomination::game::*;

    /// Shaman of the Pack drains the opponent for the number of Elves you control.
    #[test]
    fn shaman_of_the_pack_drains_by_elf_count() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::llanowar_elves()); // an Elf
        let life = g.players[1].life;
        let id = g.add_card_to_battlefield(0, catalog::shaman_of_the_pack()); // a second Elf
        g.fire_self_etb_triggers(id, 0);
        drain_stack(&mut g);
        // Two Elves you control (Llanowar + Shaman) → opponent loses 2.
        assert_eq!(g.players[1].life, life - 2);
    }

    /// Elvish Warmaster mints an Elf token when another Elf enters (once per turn).
    #[test]
    fn elvish_warmaster_makes_token_on_elf_entry() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::elvish_warmaster());
        // Cast the Elf so its entry dispatches to the Warmaster's watcher trigger.
        let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.priority.player_with_priority = 0;
        cast(&mut g, elf);
        drain_stack(&mut g);
        let tokens = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Elf Warrior" && c.is_token).count();
        assert_eq!(tokens, 1, "one Elf Warrior token minted");
    }
}

mod recent16 {
    use crabomination::card::{CardType, CreatureType};
    use crabomination::catalog;
    use crabomination::game::*;

    /// Throne of the God-Pharaoh drains each opponent for the number of tapped
    /// creatures you control at your end step.
    #[test]
    fn throne_drains_by_tapped_creatures() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::throne_of_the_god_pharaoh());
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // untapped — not counted
        g.battlefield.iter_mut().find(|c| c.id == a).unwrap().tapped = true;
        let life = g.players[1].life;
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "one tapped creature → opponent loses 1");
    }

    /// Su-Chi adds four colorless mana when it dies.
    #[test]
    fn su_chi_adds_four_mana_on_death() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::su_chi());
        assert_eq!(g.players[0].mana_pool.total(), 0);
        let _ = g.remove_to_graveyard_with_triggers(id);
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.colorless_amount(), 4, "four colorless on death");
    }

    /// Icon of Ancestry buffs creatures of the chosen type.
    #[test]
    fn icon_of_ancestry_buffs_chosen_type() {
        let mut g = two_player_game();
        let icon = g.add_card_to_battlefield(0, catalog::icon_of_ancestry());
        g.battlefield_find_mut(icon).unwrap().chosen_creature_type = Some(CreatureType::Elf);
        let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves()); // 1/1 Elf
        let cp = g.computed_permanent(elf).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2), "+1/+1 for the chosen type");
    }

    /// Aeolipile sacrifices itself to deal 2 damage to any target.
    #[test]
    fn aeolipile_pings_for_two() {
        let mut g = two_player_game();
        let pile = g.add_card_to_battlefield(0, catalog::aeolipile());
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        let life = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: pile, ability_index: 0,
            target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
        }).expect("activate Aeolipile");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2);
        assert!(g.battlefield_find(pile).is_none(), "sacrificed as a cost");
    }

    /// Phyrexian Vault sacrifices a creature to draw a card.
    #[test]
    fn phyrexian_vault_sacs_for_a_card() {
        let mut g = two_player_game();
        let vault = g.add_card_to_battlefield(0, catalog::phyrexian_vault());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: vault, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Vault");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    }

    /// Vanquisher's Banner buffs creatures of the chosen type.
    #[test]
    fn vanquishers_banner_buffs_chosen_type() {
        let mut g = two_player_game();
        let banner = g.add_card_to_battlefield(0, catalog::vanquishers_banner());
        g.battlefield_find_mut(banner).unwrap().chosen_creature_type = Some(CreatureType::Elf);
        let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves());
        assert_eq!(g.computed_permanent(elf).unwrap().power, 2, "+1/+1 for the chosen type");
    }

    /// Secluded Courtyard is a chosen-type mana land with two mana abilities.
    #[test]
    fn secluded_courtyard_is_a_chosen_type_land() {
        let d = catalog::secluded_courtyard();
        assert!(d.card_types.contains(&CardType::Land));
        assert_eq!(d.activated_abilities.len(), 2, "colorless + chosen-type-restricted mana");
        // ETB chooses a creature type.
        assert!(d.triggered_abilities.iter().any(|t| matches!(
            t.effect,
            crabomination::effect::Effect::NameCreatureType { .. }
        )));
    }
}

mod recent17 {
    use crabomination::card::{ArtifactSubtype, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;

    /// Burglar Rat's ETB makes each opponent discard a card.
    #[test]
    fn burglar_rat_each_opponent_discards() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::grizzly_bears());
        let id = g.add_card_to_battlefield(0, catalog::burglar_rat());
        let before = g.players[1].hand.len();
        g.fire_self_etb_triggers(id, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), before - 1, "opponent discarded one");
    }

    /// Corsair Captain mints a Treasure on ETB and pumps other Pirates.
    #[test]
    fn corsair_captain_treasure_and_pirate_anthem() {
        let mut g = two_player_game();
        let cap = g.add_card_to_battlefield(0, catalog::corsair_captain());
        g.fire_self_etb_triggers(cap, 0);
        drain_stack(&mut g);
        let treasures = g.battlefield.iter()
            .filter(|c| c.controller == 0
                && c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Treasure))
            .count();
        assert_eq!(treasures, 1, "one Treasure minted");
        // Another Pirate gets +1/+1; the Captain itself doesn't (other-only).
        let other = g.add_card_to_battlefield(0, catalog::corsair_captain());
        assert_eq!(g.computed_permanent(other).unwrap().power, 3, "other Pirate buffed");
        assert_eq!(g.computed_permanent(cap).unwrap().power, 3, "buffed by the other Captain");
    }

    /// Crow of Dark Tidings mills two when it enters.
    #[test]
    fn crow_mills_two_on_etb() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let id = g.add_card_to_battlefield(0, catalog::crow_of_dark_tidings());
        let gy = g.players[0].graveyard.len();
        g.fire_self_etb_triggers(id, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), gy + 2, "milled two");
    }

    /// Crusader of Odric's P/T scales with the creatures you control.
    #[test]
    fn crusader_of_odric_scales_with_creatures() {
        let mut g = two_player_game();
        let c = g.add_card_to_battlefield(0, catalog::crusader_of_odric());
        assert_eq!(g.computed_permanent(c).unwrap().power, 1, "just itself");
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert_eq!(g.computed_permanent(c).unwrap().power, 2, "two creatures now");
    }

    /// Angel of Finality exiles the opponent's graveyard on ETB.
    #[test]
    fn angel_of_finality_exiles_opponent_graveyard() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.add_card_to_graveyard(1, catalog::lightning_bolt());
        let id = g.add_card_to_battlefield(0, catalog::angel_of_finality());
        g.fire_self_etb_triggers(id, 0);
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.is_empty(), "opponent graveyard exiled");
    }

    /// Bishop's Soldier has lifelink.
    #[test]
    fn bishops_soldier_has_lifelink() {
        let d = catalog::bishops_soldier();
        assert!(d.keywords.contains(&Keyword::Lifelink));
    }

    /// Affectionate Indrik fights a creature you don't control on ETB.
    #[test]
    fn affectionate_indrik_fights() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_battlefield(0, catalog::affectionate_indrik()); // 4/4
        g.fire_self_etb_triggers(id, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "2/2 died to the 4/4's fight");
    }

    /// Angelic Edict exiles a target creature.
    #[test]
    fn angelic_edict_exiles_creature() {
        let mut g = two_player_game();
        let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::angelic_edict());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(creature)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Angelic Edict");
        drain_stack(&mut g);
        assert!(g.battlefield_find(creature).is_none(), "creature exiled");
        assert!(g.exile.iter().any(|c| c.id == creature));
    }

    /// Broken Wings destroys a creature with flying.
    #[test]
    fn broken_wings_destroys_flyer() {
        let mut g = two_player_game();
        let flyer = g.add_card_to_battlefield(1, catalog::crow_of_dark_tidings()); // 2/1 flyer
        let id = g.add_card_to_hand(0, catalog::broken_wings());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(flyer)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Broken Wings");
        drain_stack(&mut g);
        assert!(g.battlefield_find(flyer).is_none(), "flyer destroyed");
    }

    /// Ambush Wolf has flash and exiles a graveyard card on ETB.
    #[test]
    fn ambush_wolf_flash_and_exiles_gy_card() {
        let mut g = two_player_game();
        assert!(catalog::ambush_wolf().keywords.contains(&Keyword::Flash));
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let id = g.add_card_to_battlefield(0, catalog::ambush_wolf());
        g.fire_self_etb_triggers(id, 0);
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.is_empty(), "the single gy card was exiled");
    }

    /// Crackling Cyclops grows +3/+0 when you cast a noncreature spell.
    #[test]
    fn crackling_cyclops_pumps_on_noncreature_cast() {
        let mut g = two_player_game();
        let cyc = g.add_card_to_battlefield(0, catalog::crackling_cyclops());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Bolt");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(cyc).unwrap().power, 3, "+3/+0 from the noncreature cast");
    }

    /// Angel of Vitality grants +1 life and gets +2/+2 at 25+ life.
    #[test]
    fn angel_of_vitality_lifegain_and_threshold() {
        let mut g = two_player_game();
        let angel = g.add_card_to_battlefield(0, catalog::angel_of_vitality());
        g.players[0].life = 20;
        assert_eq!(g.computed_permanent(angel).unwrap().power, 2, "no buff below 25");
        // Gain 4 → bonus makes it 5 → life 25 → threshold met.
        g.adjust_life_applied(0, 4);
        assert_eq!(g.players[0].life, 25, "gained 4 plus 1");
        assert_eq!(g.computed_permanent(angel).unwrap().power, 4, "+2/+2 at 25 life");
    }

    /// Basilisk Collar grants deathtouch and lifelink to the equipped creature.
    #[test]
    fn basilisk_collar_grants_deathtouch_lifelink() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let collar = g.add_card_to_battlefield(0, catalog::basilisk_collar());
        g.battlefield_find_mut(collar).unwrap().attached_to = Some(bear);
        let kws = &g.computed_permanent(bear).unwrap().keywords;
        assert!(kws.contains(&Keyword::Deathtouch) && kws.contains(&Keyword::Lifelink));
    }

    /// Archway Angel gains 2 life per Gate on ETB.
    #[test]
    fn archway_angel_gains_life_per_gate() {
        let mut g = two_player_game();
        // Azorius Guildgate-style Gate — give it the Gate land type directly.
        let mut gate = catalog::forest();
        gate.subtypes.land_types = vec![crabomination::card::LandType::Gate];
        g.add_card_to_battlefield(0, gate.clone());
        g.add_card_to_battlefield(0, gate);
        let angel = g.add_card_to_battlefield(0, catalog::archway_angel());
        let life = g.players[0].life;
        g.fire_self_etb_triggers(angel, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 4, "2 life per Gate, two Gates");
    }

    /// Cemetery Recruitment returns a creature; a Zombie also draws.
    #[test]
    fn cemetery_recruitment_returns_and_zombie_draws() {
        let mut g = two_player_game();
        let crow = g.add_card_to_graveyard(0, catalog::crow_of_dark_tidings()); // a Zombie
        g.add_card_to_library(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::cemetery_recruitment());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(crow)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Cemetery Recruitment");
        drain_stack(&mut g);
        // +1 returned creature, +1 drawn (Zombie), -1 the spell itself left hand.
        assert_eq!(g.players[0].hand.len(), hand + 1, "returned Zombie + drew a card");
        assert!(g.players[0].graveyard.iter().all(|c| c.definition.name != "Crow of Dark Tidings"));
    }

    /// Seasoned Hallowblade discards a card to gain indestructible.
    #[test]
    fn seasoned_hallowblade_discards_for_indestructible() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
        let blade = g.add_card_to_battlefield(0, catalog::seasoned_hallowblade());
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: blade, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Hallowblade");
        drain_stack(&mut g);
        assert!(g.computed_permanent(blade).unwrap().keywords.contains(&Keyword::Indestructible));
        assert!(g.battlefield_find(blade).unwrap().tapped, "it taps itself");
    }

    /// Dryad Greenseeker reveals the top land into hand.
    #[test]
    fn dryad_greenseeker_reveals_top_land() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let dryad = g.add_card_to_battlefield(0, catalog::dryad_greenseeker());
        g.clear_sickness(dryad);
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: dryad, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("activate Greenseeker");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "top land went to hand");
    }

    /// Aggressive Mammoth grants trample to your other creatures.
    #[test]
    fn aggressive_mammoth_grants_team_trample() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::aggressive_mammoth());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample));
    }

    /// Scrabbling Claws sacrifices to exile a graveyard card and draw.
    #[test]
    fn scrabbling_claws_sac_exiles_and_draws() {
        let mut g = two_player_game();
        let claws = g.add_card_to_battlefield(0, catalog::scrabbling_claws());
        let victim = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: claws, ability_index: 1, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], x_value: None,
        }).expect("activate sac ability");
        drain_stack(&mut g);
        assert!(g.battlefield_find(claws).is_none(), "sacrificed");
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert!(g.players[1].graveyard.is_empty(), "gy card exiled");
    }

    /// A Pirate-typed Cyclops is unaffected — sanity that the Pirate anthem is typed.
    #[test]
    fn corsair_anthem_is_pirate_typed() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::corsair_captain());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // not a Pirate
        assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "non-Pirate unbuffed");
    }

    /// CR 702.38 — Amplify N: the creature enters with N +1/+1 counters for each
    /// matching card revealed in hand. Feral Throwback (Amplify 2, Beast) with two
    /// Beasts in hand enters as a 3/3 + 4 counters = 7/7.
    #[test]
    fn cr_702_38_amplify_counts_revealed_hand_cards() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::canopy_crawler()); // a Beast
        g.add_card_to_hand(0, catalog::feral_throwback()); // also a Beast
        g.add_card_to_hand(0, catalog::lightning_bolt()); // not a Beast — ignored
        let id = g.move_card_to_battlefield_for_test(0, catalog::feral_throwback());
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (7, 7), "3/3 base + 2×2 Beast counters");
    }

    /// CR 702.38 — Amplify with no matching cards in hand leaves the base body.
    #[test]
    fn cr_702_38_amplify_no_reveals_stays_base() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::kilnmouth_dragon());
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5), "no Dragons in hand → base 5/5");
    }

    /// CR 301.5 — "equipped by N": Balan's double strike keys on the attached
    /// Equipment count, dropping when an Equipment is removed.
    #[test]
    fn cr_301_5_equipped_by_count_gates_keyword() {
        let mut g = two_player_game();
        let balan = g.add_card_to_battlefield(0, catalog::balan_wandering_knight());
        let e1 = g.add_card_to_battlefield(0, catalog::bonesplitter());
        let e2 = g.add_card_to_battlefield(0, catalog::bonesplitter());
        g.battlefield_find_mut(e1).unwrap().attached_to = Some(balan);
        g.battlefield_find_mut(e2).unwrap().attached_to = Some(balan);
        assert!(g.computed_permanent(balan).unwrap().keywords.contains(&Keyword::DoubleStrike));
        // Detach one → only one Equipment left → no double strike.
        g.battlefield_find_mut(e2).unwrap().attached_to = None;
        assert!(!g.computed_permanent(balan).unwrap().keywords.contains(&Keyword::DoubleStrike));
    }

    /// CR 119 — a life-total threshold static (Angel of Vitality's +2/+2 at 25+
    /// life) turns on and off as life crosses the boundary.
    #[test]
    fn cr_119_life_threshold_static_toggles() {
        let mut g = two_player_game();
        let angel = g.add_card_to_battlefield(0, catalog::angel_of_vitality());
        g.players[0].life = 24;
        assert_eq!(g.computed_permanent(angel).unwrap().power, 2, "below 25");
        g.players[0].life = 25;
        assert_eq!(g.computed_permanent(angel).unwrap().power, 4, "at 25");
        g.players[0].life = 24;
        assert_eq!(g.computed_permanent(angel).unwrap().power, 2, "back below 25");
    }
}

mod recent18 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;

    /// Charging Bandits pumps itself +2/+0 when it attacks.
    #[test]
    fn charging_bandits_pumps_on_attack() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::charging_bandits());
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }]).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(id).unwrap().power, 5, "3/3 + 2/0 on attack");
    }

    /// Dazzling Angel gains life when another creature enters.
    #[test]
    fn dazzling_angel_gains_life_on_other_etb() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::dazzling_angel());
        let life = g.players[0].life;
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, bear); // real cast so the Angel's ETB watcher fires
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1, "gained 1 from the other creature");
    }

    /// Dragon Trainer makes a 4/4 flying Dragon on ETB.
    #[test]
    fn dragon_trainer_makes_dragon() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::dragon_trainer());
        g.fire_self_etb_triggers(id, 0);
        drain_stack(&mut g);
        let dragon = g.battlefield.iter().find(|c| c.definition.name == "Dragon").expect("a Dragon");
        assert_eq!((dragon.definition.power, dragon.definition.toughness), (4, 4));
        assert!(g.computed_permanent(dragon.id).unwrap().keywords.contains(&Keyword::Flying));
    }

    /// Goblin Tomb Raider gets +1/+0 and haste only while you control an artifact.
    #[test]
    fn goblin_tomb_raider_artifact_gate() {
        let mut g = two_player_game();
        let gob = g.add_card_to_battlefield(0, catalog::goblin_tomb_raider());
        assert_eq!(g.computed_permanent(gob).unwrap().power, 1, "base 1/2 without an artifact");
        assert!(!g.computed_permanent(gob).unwrap().keywords.contains(&Keyword::Haste));
        g.add_card_to_battlefield(0, catalog::bonesplitter()); // an artifact
        assert_eq!(g.computed_permanent(gob).unwrap().power, 2, "+1/+0 with an artifact");
        assert!(g.computed_permanent(gob).unwrap().keywords.contains(&Keyword::Haste));
    }

    /// Sanguine Syphoner drains 1 when it attacks.
    #[test]
    fn sanguine_syphoner_drains_on_attack() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::sanguine_syphoner());
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let (my, opp) = (g.players[0].life, g.players[1].life);
        g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }]).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
        assert_eq!(g.players[0].life, my + 1, "you gained 1");
    }

    /// Sky Crier draws for both players when its ability resolves.
    #[test]
    fn sky_crier_draws_for_both() {
        let mut g = two_player_game();
        let crier = g.add_card_to_battlefield(0, catalog::sky_crier());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
        g.perform_action(GameAction::ActivateAbility {
            card_id: crier, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("activate Sky Crier");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), h0 + 1, "you drew");
        assert_eq!(g.players[1].hand.len(), h1 + 1, "target opponent drew");
    }

    /// Soulmender taps to gain a life.
    #[test]
    fn soulmender_gains_life() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::soulmender());
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
        g.priority.player_with_priority = 0;
        let life = g.players[0].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap Soulmender");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 1);
    }

    /// Stormfist Crusader wheels both players a card and a life at your upkeep.
    #[test]
    fn stormfist_crusader_upkeep_wheel() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::stormfist_crusader());
        for p in 0..2 { g.add_card_to_library(p, catalog::grizzly_bears()); }
        let (h0, l1) = (g.players[0].hand.len(), g.players[1].life);
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), h0 + 1, "each player drew");
        assert_eq!(g.players[1].life, l1 - 1, "each player lost 1");
    }

    /// Run Away Together returns two creatures to their owners' hands.
    #[test]
    fn run_away_together_bounces_two() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::run_away_together());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
        }).expect("cast Run Away Together");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
        assert_eq!(g.players[0].hand.len(), 1, "my creature returned to my hand");
        assert_eq!(g.players[1].hand.len(), 1, "their creature returned to their hand");
    }

    /// Captured by Lagacs stops the enchanted creature from attacking and supports 2.
    #[test]
    fn captured_by_lagacs_locks_and_supports() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_battlefield(0, catalog::captured_by_lagacs());
        g.battlefield_find_mut(aura).unwrap().attached_to = Some(victim);
        g.fire_self_etb_triggers(aura, 0);
        drain_stack(&mut g);
        let kws = g.computed_permanent(victim).unwrap().keywords;
        assert!(kws.contains(&Keyword::CantAttack) && kws.contains(&Keyword::CantBlock));
        // Support 2 landed a +1/+1 counter somewhere friendly (the other bear).
        assert!(g.computed_permanent(other).unwrap().power >= 2, "support buffed a creature");
    }

    /// Battle Screech makes two flying Birds.
    #[test]
    fn battle_screech_makes_two_birds() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::battle_screech());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Battle Screech");
        drain_stack(&mut g);
        let birds = g.battlefield.iter().filter(|c| c.definition.name == "Bird").count();
        assert_eq!(birds, 2, "two 1/1 Birds");
    }

    /// Quag Vampires enters with a +1/+1 counter for each Multikicker payment.
    #[test]
    fn quag_vampires_grows_with_multikicker() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::quag_vampires());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 3); // {B} + two kicks of {1}{B}
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpellMultikicked {
            card_id: id, times: 2, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Quag Vampires kicked twice");
        drain_stack(&mut g);
        let v = g.battlefield.iter().find(|c| c.definition.name == "Quag Vampires").expect("on battlefield");
        assert!(g.computed_permanent(v.id).unwrap().power >= 3, "1/1 base + 2 kicked counters");
    }

    /// Bear Cub and Sworn Guardian are simple vanilla bodies.
    #[test]
    fn vanilla_bodies_have_correct_stats() {
        let cub = catalog::bear_cub();
        assert_eq!((cub.power, cub.toughness), (2, 2));
        assert!(cub.keywords.is_empty() && cub.triggered_abilities.is_empty());
        let guard = catalog::sworn_guardian();
        assert_eq!((guard.power, guard.toughness), (1, 3));
    }

    /// Hunter's Edge counters a friendly creature, then it bites an enemy.
    #[test]
    fn hunters_edge_counters_then_bites() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/3 after counter
        let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, takes 3
        let id = g.add_card_to_hand(0, catalog::hunters_edge());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(enemy)], mode: None, x_value: None,
        }).expect("cast Hunter's Edge");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "+1/+1 counter");
        assert!(g.battlefield_find(enemy).is_none(), "took 3 from a 3-power creature");
    }

    /// Kitsa loots with its tap ability and has prowess.
    #[test]
    fn kitsa_loots_and_has_prowess() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::kitsa_otterball_elite());
        g.battlefield_find_mut(id).unwrap().summoning_sick = false;
        g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
        g.add_card_to_library(0, catalog::lightning_bolt());
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("loot with Kitsa");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one — net zero");
        assert!(catalog::kitsa_otterball_elite().keywords.contains(&Keyword::Prowess));
    }

    /// Kitsa's copy activation is gated on power 3+; with two +1/+1 counters it
    /// copies your instant on the stack.
    #[test]
    fn kitsa_copies_your_spell_at_power_three() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let kitsa = g.add_card_to_battlefield(0, catalog::kitsa_otterball_elite());
        g.clear_sickness(kitsa);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // Under power 3: rejected before costs.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt on the stack");
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        assert!(g.perform_action(GameAction::ActivateAbility {
            card_id: kitsa, ability_index: 1, target: Some(Target::Permanent(bolt)),
            additional_targets: vec![], x_value: None,
        }).is_err(), "power 1 — activation rejected");
        // At power 3 the copy fires: the bolt hits twice.
        g.battlefield_find_mut(kitsa).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
        let life1 = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: kitsa, ability_index: 1, target: Some(Target::Permanent(bolt)),
            additional_targets: vec![], x_value: None,
        }).expect("copy the bolt");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1 - 6, "original + copy resolved");
    }
}

mod recent19 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;

    /// Beast-Kin Ranger pumps itself +1/+0 when another creature enters.
    #[test]
    fn beast_kin_ranger_pumps_on_other_etb() {
        let mut g = two_player_game();
        let ranger = g.add_card_to_battlefield(0, catalog::beast_kin_ranger());
        assert_eq!(g.computed_permanent(ranger).unwrap().power, 3);
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, bear);
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(ranger).unwrap().power, 4, "+1/+0 from the new creature");
    }

    /// Marble Gargoyle firebreathes toughness with {W}.
    #[test]
    fn marble_gargoyle_pumps_toughness() {
        let mut g = two_player_game();
        let gar = g.add_card_to_battlefield(0, catalog::marble_gargoyle());
        assert!(g.computed_permanent(gar).unwrap().keywords.contains(&Keyword::Flying));
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: gar, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("pump toughness");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(gar).unwrap().toughness, 3, "+0/+1");
    }

    /// Coral Colony mills X = your defenders.
    #[test]
    fn coral_colony_mills_by_defender_count() {
        let mut g = two_player_game();
        let colony = g.add_card_to_battlefield(0, catalog::coral_colony());
        g.clear_sickness(colony); // a defender
        g.add_card_to_battlefield(0, catalog::coral_colony()); // a second defender
        for _ in 0..4 { g.add_card_to_library(1, catalog::grizzly_bears()); }
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        let gy = g.players[1].graveyard.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: colony, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("activate Coral Colony");
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), gy + 2, "milled 2 for two defenders");
    }
}

mod recent20 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;
    use crabomination::mana::Color;
    use crabomination::TurnStep;

    /// Player 0 commits a crime by casting Lava Spike at player 1.
    fn commit_crime(g: &mut GameState) {
        let ls = g.add_card_to_hand(0, catalog::lava_spike());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(g, ls, Target::Player(1));
    }

    fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
        g.battlefield.iter().filter(|c| c.controller == controller && c.definition.name == name).count()
    }

    fn add_top(g: &mut GameState, player: usize, def: crabomination::card::CardDefinition) {
        let id = g.next_id();
        g.players[player].add_to_library_top(id, def);
    }

    /// Battle Cry Goblin makes a Goblin when you attack with total power 6+.
    #[test]
    fn battle_cry_goblin_pack_tactics_makes_token() {
        let mut g = two_player_game();
        let bcg = g.add_card_to_battlefield(0, catalog::battle_cry_goblin()); // 2
        let serra = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4 → total 6
        g.clear_sickness(bcg);
        g.clear_sickness(serra);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![
            Attack { attacker: bcg, target: AttackTarget::Player(1) },
            Attack { attacker: serra, target: AttackTarget::Player(1) },
        ]))
        .expect("attack");
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Goblin"), 1, "pack tactics minted a Goblin");
    }

    /// Gisa makes two Zombie Rogues when you commit a crime.
    #[test]
    fn gisa_crime_makes_two_zombies() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::gisa_the_hellraiser());
        commit_crime(&mut g);
        assert_eq!(count_named(&g, 0, "Zombie Rogue"), 2, "two tokens from the crime");
    }

    /// Gisa's crime trigger fires only once each turn.
    #[test]
    fn gisa_crime_once_per_turn() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::gisa_the_hellraiser());
        commit_crime(&mut g);
        commit_crime(&mut g);
        assert_eq!(count_named(&g, 0, "Zombie Rogue"), 2, "second crime same turn does nothing");
    }

    /// Gisa buffs your Zombies and grants them menace.
    #[test]
    fn gisa_anthem_buffs_undead() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::gisa_the_hellraiser());
        let zombie = g.add_card_to_battlefield(0, catalog::gravecrawler());
        let cp = g.computed_permanent(zombie).unwrap();
        assert!(cp.power >= 3, "Gravecrawler 2/1 → 3/2 under Gisa");
        assert!(cp.keywords.contains(&Keyword::Menace), "gains menace");
    }

    /// Magda makes a tapped Treasure when you commit a crime.
    #[test]
    fn magda_crime_makes_treasure() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::magda_the_hoardmaster());
        commit_crime(&mut g);
        let t = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Treasure");
        assert!(t.is_some(), "a Treasure was made");
        assert!(t.unwrap().tapped, "the Treasure is tapped");
    }

    /// Magda sacrifices three Treasures to make a 4/4 flying, hasty Scorpion Dragon.
    #[test]
    fn magda_sacs_three_treasures_for_scorpion_dragon() {
        let mut g = two_player_game();
        let magda = g.add_card_to_battlefield(0, catalog::magda_the_hoardmaster());
        g.clear_sickness(magda);
        let treasure = crabomination::game::effects::treasure_token();
        for _ in 0..3 {
            g.add_token_to_battlefield(0, &treasure);
        }
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: magda,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("sac three Treasures");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").count(),
            0,
            "all three Treasures sacrificed"
        );
        let dragon = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Scorpion Dragon")
            .expect("4/4 Scorpion Dragon minted");
        assert_eq!((dragon.definition.power, dragon.definition.toughness), (4, 4));
        assert!(dragon.definition.keywords.contains(&Keyword::Flying));
        assert!(dragon.definition.keywords.contains(&Keyword::Haste));
    }

    /// Marchesa digs two when you commit a crime and pay {1}.
    #[test]
    fn marchesa_crime_digs() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::marchesa_dealer_of_death());
        // stock the library so the dig has cards.
        add_top(&mut g, 0, catalog::grizzly_bears());
        add_top(&mut g, 0, catalog::grizzly_bears());
        let hand_before = g.players[0].hand.len();
        let gy_before = g.players[0].graveyard.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.players[0].mana_pool.add_colorless(1);
        commit_crime(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "one card to hand");
        assert!(g.players[0].graveyard.len() > gy_before, "one card to graveyard");
    }

    /// Forsaken Miner returns from the graveyard when you commit a crime and pay {B}.
    #[test]
    fn forsaken_miner_returns_on_crime() {
        let mut g = two_player_game();
        let miner = g.add_card_to_graveyard(0, catalog::forsaken_miner());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.players[0].mana_pool.add(Color::Black, 1);
        commit_crime(&mut g);
        assert!(g.battlefield_find(miner).is_some(), "Forsaken Miner returned to the battlefield");
    }

    /// Nimble Brigand is unblockable after you've committed a crime.
    #[test]
    fn nimble_brigand_unblockable_after_crime() {
        let mut g = two_player_game();
        let nb = g.add_card_to_battlefield(0, catalog::nimble_brigand());
        assert!(!g.computed_permanent(nb).unwrap().keywords.contains(&Keyword::Unblockable));
        commit_crime(&mut g);
        assert!(
            g.computed_permanent(nb).unwrap().keywords.contains(&Keyword::Unblockable),
            "unblockable once a crime is committed"
        );
    }

    /// Vial Smasher pings an opponent when another outlaw you control enters.
    #[test]
    fn vial_smasher_pings_on_outlaw_etb() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::vial_smasher_gleeful_grenadier());
        let life_before = g.players[1].life;
        // Treasure Dredger is a Rogue (outlaw).
        let dredger = g.add_card_to_battlefield(0, catalog::treasure_dredger());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: dredger }]);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life_before - 1, "1 damage to the opponent");
    }

    /// Rakish Crew drains when an outlaw you control dies.
    #[test]
    fn rakish_crew_drains_on_outlaw_death() {
        let mut g = two_player_game();
        let crew = g.add_card_to_battlefield(0, catalog::rakish_crew());
        g.fire_self_etb_triggers(crew, 0); // ETB Mercenary token
        drain_stack(&mut g);
        let rogue = g.add_card_to_battlefield(0, catalog::treasure_dredger()); // Rogue
        let opp_life = g.players[1].life;
        let mut evs = g.remove_to_graveyard_with_triggers(rogue);
        evs.push(GameEvent::CreatureDied { card_id: rogue });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_life - 1, "opponent loses 1");
    }

    /// Rictus Robber makes a Zombie when a creature died this turn.
    #[test]
    fn rictus_robber_token_if_creature_died() {
        let mut g = two_player_game();
        let chump = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_to_graveyard_with_triggers(chump);
        drain_stack(&mut g);
        let robber = g.add_card_to_battlefield(0, catalog::rictus_robber());
        g.fire_self_etb_triggers(robber, 0);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Zombie Rogue"), 1, "a creature died → token");
    }

    /// Holy Cow gains 2 life on ETB.
    #[test]
    fn holy_cow_gains_life() {
        let mut g = two_player_game();
        let life = g.players[0].life;
        let cow = g.add_card_to_battlefield(0, catalog::holy_cow());
        g.fire_self_etb_triggers(cow, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained 2 life");
    }

    /// Sterling Keykeeper taps a target creature.
    #[test]
    fn sterling_keykeeper_taps_creature() {
        let mut g = two_player_game();
        let keeper = g.add_card_to_battlefield(0, catalog::sterling_keykeeper());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(keeper);
        g.players[0].mana_pool.add_colorless(2);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: keeper,
            ability_index: 0,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).unwrap().tapped, "bear tapped");
    }

    /// Treasure Dredger mints a Treasure.
    #[test]
    fn treasure_dredger_makes_treasure() {
        let mut g = two_player_game();
        let td = g.add_card_to_battlefield(0, catalog::treasure_dredger());
        g.clear_sickness(td);
        g.players[0].mana_pool.add_colorless(1);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: td,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Treasure"), 1);
    }

    /// Slick Sequence draws if you've cast another spell this turn.
    #[test]
    fn slick_sequence_draws_after_second_spell() {
        let mut g = two_player_game();
        g.players[0].spells_cast_this_turn = 1; // pretend a spell was already cast
        add_top(&mut g, 0, catalog::grizzly_bears());
        let hand_before = g.players[0].hand.len();
        let ss = g.add_card_to_hand(0, catalog::slick_sequence());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, ss, Target::Player(1));
        // cast itself bumps the count to 2; the "another spell" gate (>=1 prior) holds.
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    }

    /// Razzle-Dazzler grows and turns unblockable on your second spell.
    #[test]
    fn razzle_dazzler_grows_on_second_spell() {
        let mut g = two_player_game();
        let rd = g.add_card_to_battlefield(0, catalog::razzle_dazzler());
        g.players[0].spells_cast_this_turn = 1;
        // Casting any spell makes it the second this turn.
        let bolt = g.add_card_to_hand(0, catalog::lava_spike());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, bolt, Target::Player(1));
        let cp = g.computed_permanent(rd).unwrap();
        assert_eq!(cp.power, 2, "got a +1/+1 counter");
        assert!(cp.keywords.contains(&Keyword::Unblockable), "can't be blocked this turn");
    }

    /// Quilled Charger pumps and gains menace when it attacks while saddled.
    #[test]
    fn quilled_charger_pumps_while_saddled() {
        let mut g = two_player_game();
        let qc = g.add_card_to_battlefield(0, catalog::quilled_charger());
        g.battlefield_find_mut(qc).unwrap().saddled = true;
        g.clear_sickness(qc);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: qc,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        let cp = g.computed_permanent(qc).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5), "+1/+2 while saddled");
        assert!(cp.keywords.contains(&Keyword::Menace));
    }

    /// Lassoed by the Law exiles an opponent's permanent and makes a Mercenary.
    #[test]
    fn lassoed_exiles_and_makes_token() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let lasso = g.add_card_to_battlefield(0, catalog::lassoed_by_the_law());
        g.fire_self_etb_triggers(lasso, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "opponent's bear exiled");
        assert_eq!(count_named(&g, 0, "Mercenary"), 1, "made a Mercenary");
    }

    /// Roxanne makes a Meteorite when she enters.
    #[test]
    fn roxanne_makes_meteorite_on_etb() {
        let mut g = two_player_game();
        let roxanne = g.add_card_to_battlefield(0, catalog::roxanne_starfall_savant());
        g.fire_self_etb_triggers(roxanne, 0);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Meteorite"), 1);
    }

    /// Honest Rutstein returns a creature card from your graveyard and cheapens
    /// creature spells.
    #[test]
    fn honest_rutstein_value() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let rutstein = g.add_card_to_battlefield(0, catalog::honest_rutstein());
        g.fire_self_etb_triggers(rutstein, 0);
        drain_stack(&mut g);
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "returned the creature card to hand"
        );
        let bears = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &bears, None), 1, "creature spells cost {{1}} less");
    }

    /// Stoic Sphinx has hexproof until you cast a spell.
    #[test]
    fn stoic_sphinx_hexproof_until_spell() {
        let mut g = two_player_game();
        let sphinx = g.add_card_to_battlefield(0, catalog::stoic_sphinx());
        assert!(g.computed_permanent(sphinx).unwrap().keywords.contains(&Keyword::Hexproof));
        g.players[0].spells_cast_this_turn = 1;
        assert!(
            !g.computed_permanent(sphinx).unwrap().keywords.contains(&Keyword::Hexproof),
            "loses hexproof once you've cast a spell"
        );
    }

    /// Hellspur Brute gets Affinity for outlaws.
    #[test]
    fn hellspur_brute_affinity_for_outlaws() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::treasure_dredger()); // Rogue
        g.add_card_to_battlefield(0, catalog::nimble_brigand()); // Rogue
        let brute = crabomination::card::CardInstance::new(g.next_id(), catalog::hellspur_brute(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &brute, None), 2, "{{1}} less per outlaw");
    }

    /// Bovine Intervention destroys and gives the controller an Ox.
    #[test]
    fn bovine_intervention_destroys_and_makes_ox() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let bovine = g.add_card_to_hand(0, catalog::bovine_intervention());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, bovine, Target::Permanent(bear));
        assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
        assert_eq!(count_named(&g, 1, "Ox"), 1, "its controller made an Ox");
    }
}
