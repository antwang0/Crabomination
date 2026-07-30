//! Tests for recentN card batches 61-70 (merged from per-batch micro-files).

mod recent61 {
    use crabomination::card::{CardDefinition, CardType, CreatureType, Keyword, Subtypes};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;

    fn human(name: &'static str) -> CardDefinition {
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
            power: 2,
            toughness: 2,
            ..Default::default()
        }
    }

    fn vanilla(name: &'static str, p: i32, t: i32) -> CardDefinition {
        CardDefinition { name, card_types: vec![CardType::Creature], power: p, toughness: t, ..Default::default() }
    }

    #[test]
    fn kessig_malcontents_burns_for_humans() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, human("Villager A"));
        g.add_card_to_battlefield(0, human("Villager B"));
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        let life = g.players[1].life;
        // Kessig is itself a Human → 3 Humans total → 3 damage.
        let k = g.add_card_to_battlefield(0, catalog::kessig_malcontents());
        g.fire_self_etb_triggers(k, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 3, "3 damage = number of Humans");
    }

    #[test]
    fn somberwald_vigilante_pings_its_blocker() {
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, catalog::somberwald_vigilante());
        g.clear_sickness(atk);
        let blocker = g.add_card_to_battlefield(1, vanilla("Weakling", 1, 1));
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(1),
        }])).unwrap();
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, atk)])).unwrap();
        drain_stack(&mut g);
        g.check_state_based_actions();
        assert!(g.battlefield_find(blocker).is_none(), "1/1 blocker dies to the 1-damage ping");
    }

    #[test]
    fn ash_zealot_punishes_graveyard_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::ash_zealot());
        // A flashback sorcery in the opponent's graveyard.
        fn flashy() -> CardDefinition {
            CardDefinition {
                name: "Flashy Bolt",
                cost: crabomination::mana::cost(&[crabomination::mana::r()]),
                card_types: vec![CardType::Sorcery],
                keywords: vec![Keyword::Flashback(crabomination::mana::cost(&[crabomination::mana::r()]))],
                effect: crabomination::effect::Effect::Noop,
                ..Default::default()
            }
        }
        let sp = g.add_card_to_hand(1, flashy());
        // Move it to the graveyard.
        let pos = g.players[1].hand.iter().position(|c| c.id == sp).unwrap();
        let card = g.players[1].hand.remove(pos);
        g.players[1].graveyard.push(card);
        g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        let life = g.players[1].life;
        g.perform_action(GameAction::CastFlashback {
            card_id: sp, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("flashback cast");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 3, "Ash Zealot deals 3 to the graveyard caster");
    }

    #[test]
    fn perimeter_captain_gains_on_defender_block() {
        let mut g = two_player_game();
        let cap = g.add_card_to_battlefield(0, catalog::perimeter_captain());
        let atk = g.add_card_to_battlefield(1, vanilla("Raider", 2, 2));
        g.clear_sickness(atk);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let life = g.players[0].life;
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(0),
        }])).unwrap();
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareBlockers(vec![(cap, atk)])).unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained 2 when the defender blocked");
    }

    #[test]
    fn firefist_striker_battalion_locks_a_blocker() {
        let mut g = two_player_game();
        let striker = g.add_card_to_battlefield(0, catalog::firefist_striker());
        let a = g.add_card_to_battlefield(0, vanilla("A", 1, 1));
        let b = g.add_card_to_battlefield(0, vanilla("B", 1, 1));
        for id in [striker, a, b] { g.clear_sickness(id); }
        let foe = g.add_card_to_battlefield(1, vanilla("Wall", 2, 2));
        // Auto-target picks the opponent's creature for the "can't block" debuff.
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![
            Attack { attacker: striker, target: AttackTarget::Player(1) },
            Attack { attacker: a, target: AttackTarget::Player(1) },
            Attack { attacker: b, target: AttackTarget::Player(1) },
        ])).unwrap();
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::CantBlock),
            "battalion granted the target creature can't-block",
        );
    }

    #[test]
    fn scab_clan_berserker_punishes_only_once_renowned() {
        let mut g = two_player_game();
        let scab = g.add_card_to_battlefield(0, catalog::scab_clan_berserker());
        // Opponent's noncreature spell before renown: no damage.
        fn bolt() -> CardDefinition {
            CardDefinition {
                name: "Zap",
                cost: crabomination::mana::cost(&[crabomination::mana::r()]),
                card_types: vec![CardType::Instant],
                effect: crabomination::effect::Effect::Noop,
                ..Default::default()
            }
        }
        let cast_zap = |g: &mut GameState| {
            let z = g.add_card_to_hand(1, bolt());
            g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
            g.step = TurnStep::PreCombatMain;
            g.active_player_idx = 1;
            g.priority.player_with_priority = 1;
            g.perform_action(GameAction::CastSpell {
                card_id: z, target: None, additional_targets: vec![], mode: None, x_value: None,
            }).expect("cast");
            drain_stack(g);
        };
        let life0 = g.players[1].life;
        cast_zap(&mut g);
        assert_eq!(g.players[1].life, life0, "no punish before renown");
        // Make it renowned, then cast again → 2 damage.
        g.battlefield_find_mut(scab).unwrap().renowned = true;
        let life1 = g.players[1].life;
        cast_zap(&mut g);
        assert_eq!(g.players[1].life, life1 - 2, "renowned Scab-Clan deals 2 to the caster");
    }

    #[test]
    fn fireblade_charger_dies_deals_its_power() {
        let mut g = two_player_game();
        let fc = g.add_card_to_battlefield(0, catalog::fireblade_charger());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        let life = g.players[1].life;
        // Lethal damage + SBA kills it, firing the death trigger.
        g.battlefield_find_mut(fc).unwrap().damage = 1;
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "dealt damage equal to its power (1)");
    }
}

mod recent62 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;

    fn servos(g: &GameState, p: usize) -> usize {
        g.battlefield.iter().filter(|c| c.controller == p && c.definition.name == "Servo").count()
    }

    #[test]
    fn servo_schematic_makes_servo_on_enter_and_death() {
        let mut g = two_player_game();
        let id = g.move_card_to_battlefield_for_test(0, catalog::servo_schematic());
        drain_stack(&mut g);
        assert_eq!(servos(&g, 0), 1, "one Servo on enter");
        // Destroy it → a second Servo on leaving the battlefield.
        g.remove_to_graveyard_with_triggers(id);
        drain_stack(&mut g);
        assert_eq!(servos(&g, 0), 2, "second Servo when put into the graveyard");
    }

    #[test]
    fn cogworkers_puzzleknot_etb_and_sac() {
        let mut g = two_player_game();
        let id = g.move_card_to_battlefield_for_test(0, catalog::cogworkers_puzzleknot());
        drain_stack(&mut g);
        assert_eq!(servos(&g, 0), 1, "ETB Servo");
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("sac ability");
        drain_stack(&mut g);
        assert_eq!(servos(&g, 0), 2, "sac makes a second Servo");
        assert!(g.battlefield_find(id).is_none(), "the Puzzleknot was sacrificed");
    }

    #[test]
    fn renegade_freighter_pumps_and_tramples_on_attack() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::renegade_freighter());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(veh);
        g.clear_sickness(bear);
        g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![bear] }).expect("crew");
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: veh, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        let cp = g.compute_battlefield();
        let v = cp.iter().find(|c| c.id == veh).unwrap();
        assert_eq!((v.power, v.toughness), (5, 4), "4/3 → 5/4 on attack");
        assert!(v.keywords.contains(&Keyword::Trample), "gains trample");
    }

    #[test]
    fn bomat_bazaar_barge_draws_on_enter() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let hand = g.players[0].hand.len();
        let id = g.add_card_to_battlefield(0, catalog::bomat_bazaar_barge());
        g.fire_self_etb_triggers(id, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew on enter");
    }

    #[test]
    fn peema_outrider_fabricate_makes_servo() {
        let mut g = two_player_game();
        // Fabricate mode 1 = create Servos instead of the +1/+1 counter.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
        g.move_card_to_battlefield_for_test(0, catalog::peema_outrider());
        drain_stack(&mut g);
        assert_eq!(servos(&g, 0), 1, "fabricate 1 minted a Servo");
    }

    #[test]
    fn deadeye_harpooner_destroys_tapped_creature_with_revolt() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(foe).unwrap().tapped = true;
        // Trigger revolt: a permanent left the battlefield under our control.
        let sac = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_to_graveyard_with_triggers(sac);
        drain_stack(&mut g);
        let dh = g.add_card_to_battlefield(0, catalog::deadeye_harpooner());
        g.fire_self_etb_triggers(dh, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "revolt destroyed the tapped creature");
    }

    #[test]
    fn gearshift_ace_grants_first_strike_on_crew() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::renegade_freighter());
        let ace = g.add_card_to_battlefield(0, catalog::gearshift_ace());
        g.clear_sickness(ace);
        g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![ace] }).expect("crew");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(veh).unwrap().keywords.contains(&Keyword::FirstStrike),
            "the crewed Vehicle gains first strike",
        );
    }

    #[test]
    fn veteran_motorist_scries_and_pumps_crewed_vehicle() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::renegade_freighter());
        let vm = g.add_card_to_battlefield(0, catalog::veteran_motorist());
        g.clear_sickness(vm);
        g.perform_action(GameAction::Crew { vehicle: veh, crew_creatures: vec![vm] }).expect("crew");
        drain_stack(&mut g);
        let v = g.compute_battlefield().into_iter().find(|c| c.id == veh).unwrap();
        assert_eq!((v.power, v.toughness), (5, 4), "crewed Vehicle got +1/+1");
    }

    #[test]
    fn aether_chaser_energy_then_servo_on_attack() {
        let mut g = two_player_game();
        let ac = g.move_card_to_battlefield_for_test(0, catalog::aether_chaser());
        drain_stack(&mut g);
        assert_eq!(g.players[0].energy, 2, "ETB gave two energy");
        g.clear_sickness(ac);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ac, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert_eq!(servos(&g, 0), 1, "paid {{E}}{{E}} for a Servo");
        assert_eq!(g.players[0].energy, 0, "energy spent");
    }
}

mod recent63 {
    use crabomination::card::{CardType, CreatureType, Keyword, Subtypes};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::*;

    fn bear(name: &'static str) -> crabomination::card::CardDefinition {
        crabomination::card::CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
            power: 2,
            toughness: 2,
            ..Default::default()
        }
    }

    fn cast_instant_at(g: &mut GameState, controller: usize, id: CardId, target: CardId) {
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = controller;
        g.priority.player_with_priority = controller;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Permanent(target)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).expect("cast");
        drain_stack(g);
    }

    #[test]
    fn scion_of_the_wild_scales_with_creatures() {
        let mut g = two_player_game();
        let scion = g.add_card_to_battlefield(0, catalog::scion_of_the_wild());
        g.add_card_to_battlefield(0, bear("A"));
        g.add_card_to_battlefield(0, bear("B"));
        let c = g.compute_battlefield();
        let s = c.iter().find(|c| c.id == scion).unwrap();
        // Scion + 2 bears = 3 creatures you control.
        assert_eq!((s.power, s.toughness), (3, 3));
    }

    #[test]
    fn grazing_gladehart_landfall_gains_life() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grazing_gladehart());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let land = g.add_card_to_hand(0, catalog::forest());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let life = g.players[0].life;
        g.perform_action(GameAction::PlayLand(land)).expect("play land");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "landfall gained 2");
    }

    #[test]
    fn snapping_sailback_enrages() {
        let mut g = two_player_game();
        let sb = g.add_card_to_battlefield(0, catalog::snapping_sailback());
        g.dispatch_triggers_for_events(&[GameEvent::DamageDealt {
            amount: 2, to_card: Some(sb), to_player: None, combat: false, from_controller: None, from_card: None, }]);
        drain_stack(&mut g);
        let c = g.compute_battlefield();
        let s = c.iter().find(|c| c.id == sb).unwrap();
        assert_eq!((s.power, s.toughness), (5, 5), "one +1/+1 counter from enrage");
    }

    #[test]
    fn baloth_woodcrasher_landfall_pumps() {
        let mut g = two_player_game();
        let baloth = g.add_card_to_battlefield(0, catalog::baloth_woodcrasher());
        let land = g.add_card_to_hand(0, catalog::forest());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::PlayLand(land)).expect("play land");
        drain_stack(&mut g);
        let c = g.compute_battlefield();
        let b = c.iter().find(|c| c.id == baloth).unwrap();
        assert_eq!((b.power, b.toughness), (8, 8), "4/4 → 8/8 on landfall");
        assert!(b.keywords.contains(&Keyword::Trample));
    }

    #[test]
    fn kavu_climber_draws_on_enter() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let hand = g.players[0].hand.len();
        let id = g.add_card_to_battlefield(0, catalog::kavu_climber());
        g.fire_self_etb_triggers(id, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1);
    }

    #[test]
    fn might_of_oaks_pumps_seven() {
        let mut g = two_player_game();
        let target = g.add_card_to_battlefield(0, bear("Grunt"));
        let spell = g.add_card_to_hand(0, catalog::might_of_oaks());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast_instant_at(&mut g, 0, spell, target);
        let c = g.compute_battlefield();
        let t = c.iter().find(|c| c.id == target).unwrap();
        assert_eq!((t.power, t.toughness), (9, 9), "2/2 + 7/7");
    }

    #[test]
    fn wildsize_pumps_tramples_and_draws() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let target = g.add_card_to_battlefield(0, bear("Grunt"));
        let spell = g.add_card_to_hand(0, catalog::wildsize());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        cast_instant_at(&mut g, 0, spell, target);
        let c = g.compute_battlefield();
        let t = c.iter().find(|c| c.id == target).unwrap();
        assert_eq!((t.power, t.toughness), (4, 4), "2/2 + 2/2");
        assert!(t.keywords.contains(&Keyword::Trample));
        // -1 cast + 1 draw = net unchanged vs pre-cast hand.
        assert_eq!(g.players[0].hand.len(), hand);
    }

    #[test]
    fn broken_bond_destroys_and_ramps() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::sol_ring());
        let land = g.add_card_to_hand(0, catalog::forest());
        let spell = g.add_card_to_hand(0, catalog::broken_bond());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        // Destroy target passed in the cast; the land pick is the only prompt.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![land])]));
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(art)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
        assert!(g.battlefield_find(land).is_some(), "land put onto the battlefield");
    }

    #[test]
    fn woodfall_primus_destroys_noncreature_and_has_persist() {
        let def = catalog::woodfall_primus();
        assert!(def.keywords.contains(&Keyword::Persist) && def.keywords.contains(&Keyword::Trample));
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(1, catalog::sol_ring());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(art))]));
        let wp = g.add_card_to_battlefield(0, catalog::woodfall_primus());
        g.fire_self_etb_triggers(wp, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).is_none(), "ETB destroyed the noncreature");
    }
}

mod recent64 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;

    #[test]
    fn peregrine_drake_untaps_five_lands() {
        let mut g = two_player_game();
        let lands: Vec<CardId> =
            (0..5).map(|_| g.add_card_to_battlefield(0, catalog::island())).collect();
        for &l in &lands { g.battlefield_find_mut(l).unwrap().tapped = true; }
        let drake = g.add_card_to_battlefield(0, catalog::peregrine_drake());
        g.fire_self_etb_triggers(drake, 0);
        drain_stack(&mut g);
        assert!(lands.iter().all(|&l| !g.battlefield_find(l).unwrap().tapped), "all five untapped");
    }

    #[test]
    fn cloud_elemental_can_block_only_flying() {
        let mut g = two_player_game();
        let ce = g.add_card_to_battlefield(0, catalog::cloud_elemental());
        assert!(g.computed_permanent(ce).unwrap().keywords.contains(&Keyword::CanBlockOnlyFlying));
    }

    #[test]
    fn thought_courier_loots() {
        let mut g = two_player_game();
        for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
        let extra = g.add_card_to_hand(0, catalog::forest());
        let tc = g.add_card_to_battlefield(0, catalog::thought_courier());
        g.clear_sickness(tc);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![extra])]));
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: tc, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("loot");
        drain_stack(&mut g);
        // +1 draw, -1 discard = net unchanged.
        assert_eq!(g.players[0].hand.len(), hand);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == extra), "discarded the chosen card");
    }

    #[test]
    fn jhessian_thief_draws_on_combat_damage() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let jt = g.add_card_to_battlefield(0, catalog::jhessian_thief());
        let hand = g.players[0].hand.len();
        g.fire_combat_damage_to_player_triggers(jt, 1, 1);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew on combat damage");
    }

    #[test]
    fn sky_spirit_flies_and_first_strikes() {
        let def = catalog::sky_spirit();
        assert!(def.keywords.contains(&Keyword::Flying) && def.keywords.contains(&Keyword::FirstStrike));
    }

    #[test]
    fn cephalid_broker_target_player_wheels_two() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(1, catalog::forest()); }
        let discardable: Vec<CardId> =
            (0..2).map(|_| g.add_card_to_hand(1, catalog::mountain())).collect();
        let cb = g.add_card_to_battlefield(0, catalog::cephalid_broker());
        g.clear_sickness(cb);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(discardable.clone())]));
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[1].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: cb, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        // Opponent drew 2 and discarded 2 → net hand unchanged.
        assert_eq!(g.players[1].hand.len(), hand);
        assert!(discardable.iter().all(|d| g.players[1].graveyard.iter().any(|c| c.id == *d)));
    }

    #[test]
    fn riverwise_augur_draws_three_puts_two_back() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
        let ra = g.add_card_to_battlefield(0, catalog::riverwise_augur());
        let hand = g.players[0].hand.len();
        let lib = g.players[0].library.len();
        // Put the first two hand cards back on top.
        let picks: Vec<CardId> = g.players[0].hand.iter().take(2).map(|c| c.id).collect();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::PutOnLibrary(picks)]));
        g.fire_self_etb_triggers(ra, 0);
        drain_stack(&mut g);
        // +3 drawn, -2 put back = net +1 hand; library net -1.
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew 3, put 2 back");
        assert_eq!(g.players[0].library.len(), lib - 1);
    }
}

mod recent65 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::*;

    fn pt(g: &GameState, id: CardId) -> (i32, i32) {
        let c = g.compute_battlefield();
        let c = c.iter().find(|c| c.id == id).unwrap();
        (c.power, c.toughness)
    }

    #[test]
    fn ruthless_cullblade_swings_on_low_life() {
        let mut g = two_player_game();
        let cb = g.add_card_to_battlefield(0, catalog::ruthless_cullblade());
        assert_eq!(pt(&g, cb), (2, 1), "base while opponent is above 10");
        g.players[1].life = 10;
        assert_eq!(pt(&g, cb), (4, 2), "+2/+1 while opponent at 10 or less");
    }

    #[test]
    fn guul_draz_vampire_gains_intimidate_on_low_life() {
        let mut g = two_player_game();
        let gd = g.add_card_to_battlefield(0, catalog::guul_draz_vampire());
        assert!(!g.computed_permanent(gd).unwrap().keywords.contains(&Keyword::Intimidate));
        g.players[1].life = 8;
        assert_eq!(pt(&g, gd), (3, 2));
        assert!(g.computed_permanent(gd).unwrap().keywords.contains(&Keyword::Intimidate));
    }

    #[test]
    fn bloodrite_invoker_drains_three() {
        let mut g = two_player_game();
        let bi = g.add_card_to_battlefield(0, catalog::bloodrite_invoker());
        g.clear_sickness(bi);
        g.players[0].mana_pool.add_colorless(8);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let (my, opp) = (g.players[0].life, g.players[1].life);
        g.perform_action(GameAction::ActivateAbility {
            card_id: bi, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
        }).expect("drain");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 3);
        assert_eq!(g.players[0].life, my + 3);
    }

    #[test]
    fn nip_gwyllion_has_lifelink() {
        assert!(catalog::nip_gwyllion().keywords.contains(&Keyword::Lifelink));
    }

    #[test]
    fn barony_vampire_is_a_three_two_vampire() {
        let d = catalog::barony_vampire();
        assert_eq!((d.power, d.toughness), (3, 2));
        assert!(d.subtypes.creature_types.contains(&crabomination::card::CreatureType::Vampire));
    }

    #[test]
    fn nested_shambler_leaves_squirrels_equal_to_power() {
        let mut g = two_player_game();
        let ns = g.add_card_to_battlefield(0, catalog::nested_shambler());
        // Pump to power 3 → 3 Squirrels on death.
        g.battlefield_find_mut(ns).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
        g.remove_to_graveyard_with_triggers(ns);
        drain_stack(&mut g);
        let sq: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").collect();
        assert_eq!(sq.len(), 3, "X = its (pumped) power");
        assert!(sq.iter().all(|c| c.tapped), "Squirrels enter tapped");
    }

    #[test]
    fn duty_bound_dead_regenerates() {
        let mut g = two_player_game();
        let d = g.add_card_to_battlefield(0, catalog::duty_bound_dead());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: d, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("regen shield");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(d).unwrap().regeneration_shields, 1, "shield stamped");
        // Lethal damage is soaked by the regeneration shield.
        g.battlefield_find_mut(d).unwrap().damage = 5;
        g.check_state_based_actions();
        assert!(g.battlefield_find(d).is_some(), "regen shield saved it");
    }
}

mod recent66 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::mana::Color;
    use crabomination::TurnStep;

    fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
        g.battlefield.iter().filter(|c| c.controller == controller && c.definition.name == name).count()
    }

    /// Cast Lava Spike at the opponent to commit a crime.
    fn commit_crime(g: &mut GameState) {
        let ls = g.add_card_to_hand(0, catalog::lava_spike());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(g, ls, Target::Player(1));
    }

    fn attack_saddled(g: &mut GameState, id: CardId) {
        g.battlefield_find_mut(id).unwrap().saddled = true;
        g.clear_sickness(id);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: id,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(g);
    }

    #[test]
    fn vengeful_townsfolk_grows_when_others_die() {
        let mut g = two_player_game();
        let vt = g.add_card_to_battlefield(0, catalog::vengeful_townsfolk());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Kill the ally through the full damage→SBA→dispatch path.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, bolt, Target::Permanent(ally));
        assert_eq!(g.battlefield_find(vt).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    #[test]
    fn loan_shark_draws_when_two_spells_cast() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        g.players[0].spells_cast_this_turn = 2;
        let ls = g.add_card_to_battlefield(0, catalog::loan_shark());
        let hand = g.players[0].hand.len();
        g.fire_self_etb_triggers(ls, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "ETB drew with 2 spells cast");
        assert!(catalog::loan_shark().plot_cost.is_some());
    }

    #[test]
    fn rattleback_apothecary_grants_menace_on_crime() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::rattleback_apothecary());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Mode 0 = menace.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Modes(vec![0])]));
        commit_crime(&mut g);
        assert!(g
            .computed_permanent(bear)
            .unwrap()
            .keywords
            .contains(&Keyword::Menace));
    }

    #[test]
    fn servant_of_the_stinger_has_deathtouch() {
        let d = catalog::servant_of_the_stinger();
        assert!(d.keywords.contains(&Keyword::Deathtouch));
        assert_eq!((d.power, d.toughness), (1, 3));
    }

    #[test]
    fn deserts_due_scales_with_deserts() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
        // Two Deserts → -2/-2 plus -2/-2 = -4/-4 → 0/0, dies.
        g.add_card_to_battlefield(0, catalog::conduit_pylons());
        g.add_card_to_battlefield(0, catalog::conduit_pylons());
        let id = g.add_card_to_hand(0, catalog::deserts_due());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, id, Target::Permanent(victim));
        assert!(g.battlefield_find(victim).is_none(), "-4/-4 killed the 4/4");
    }

    #[test]
    fn quick_draw_pumps_own_and_strips_opponent() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::combat_thresher()); // has double strike
        let id = g.add_card_to_hand(0, catalog::quick_draw());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Player(1)],
            mode: None,
            x_value: None,
        })
        .expect("cast Quick Draw");
        drain_stack(&mut g);
        let cp = g.computed_permanent(mine).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
        assert!(cp.keywords.contains(&Keyword::FirstStrike));
        assert!(
            !g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "opponent's double strike stripped"
        );
    }

    #[test]
    fn prickly_pair_makes_a_mercenary() {
        let mut g = two_player_game();
        let pp = g.add_card_to_battlefield(0, catalog::prickly_pair());
        g.fire_self_etb_triggers(pp, 0);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Mercenary"), 1);
    }

    #[test]
    fn bounding_felidar_buffs_team_when_saddled_attacks() {
        let mut g = two_player_game();
        let felidar = g.add_card_to_battlefield(0, catalog::bounding_felidar());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let life = g.players[0].life;
        attack_saddled(&mut g, felidar);
        assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert_eq!(g.players[0].life, life + 1, "gained 1 per other creature");
    }

    #[test]
    fn trained_arynx_first_strike_when_saddled_attacks() {
        let mut g = two_player_game();
        let arynx = g.add_card_to_battlefield(0, catalog::trained_arynx());
        attack_saddled(&mut g, arynx);
        assert!(g.computed_permanent(arynx).unwrap().keywords.contains(&Keyword::FirstStrike));
    }

    fn cast_weatherseed(g: &mut GameState) -> CardId {
        let id = g.add_card_to_hand(0, catalog::the_weatherseed_treaty());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast The Weatherseed Treaty");
        drain_stack(g);
        id
    }

    #[test]
    fn weatherseed_treaty_read_ahead_starts_on_chosen_chapter() {
        let mut g = two_player_game();
        // Read ahead → start on chapter II (make a Saproling), skipping chapter I.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
        let saga = cast_weatherseed(&mut g);
        assert_eq!(
            g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore),
            2,
            "entered with 2 lore counters"
        );
        assert_eq!(count_named(&g, 0, "Saproling"), 1, "chapter II fired");
    }

    #[test]
    fn weatherseed_treaty_read_ahead_defaults_to_chapter_one() {
        let mut g = two_player_game();
        // AutoDecider declines the amount → falls back to chapter I.
        let saga = cast_weatherseed(&mut g);
        assert_eq!(
            g.battlefield_find(saga).unwrap().counter_count(CounterType::Lore),
            1,
            "default start is chapter I"
        );
        assert_eq!(count_named(&g, 0, "Saproling"), 0, "chapter II not fired");
    }

    #[test]
    fn frenzy_sliver_grants_frenzy_to_unblocked_slivers() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::frenzy_sliver());
        let gale = g.add_card_to_battlefield(0, catalog::galerider_sliver()); // 1/1
        g.clear_sickness(gale);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: gale,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
        drain_stack(&mut g);
        // Unblocked → Frenzy 1 from Frenzy Sliver: +1/+0.
        assert_eq!(g.computed_permanent(gale).unwrap().power, 2);
    }

    #[test]
    fn rambling_possum_pumps_when_saddled_attacks() {
        let mut g = two_player_game();
        let possum = g.add_card_to_battlefield(0, catalog::rambling_possum());
        attack_saddled(&mut g, possum);
        let cp = g.computed_permanent(possum).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 5), "+1/+2 while saddled");
    }
}

mod recent67 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;
    use crabomination::TurnStep;

    fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
        g.battlefield.iter().filter(|c| c.controller == controller && c.definition.name == name).count()
    }

    #[test]
    fn nezumi_linkbreaker_dies_into_a_mercenary() {
        let mut g = two_player_game();
        let nz = g.add_card_to_battlefield(0, catalog::nezumi_linkbreaker());
        g.remove_to_graveyard_with_triggers(nz);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Mercenary"), 1);
    }

    #[test]
    fn gold_rush_makes_treasure_and_pumps_per_treasure() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_hand(0, catalog::gold_rush());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        cast_at(&mut g, id, Target::Permanent(bear));
        // One Treasure made → +2/+2 → 4/4.
        assert_eq!(count_named(&g, 0, "Treasure"), 1);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4));
    }

    #[test]
    fn prosperity_tycoon_etb_mercenary_and_sac_for_indestructible() {
        let mut g = two_player_game();
        let pt = g.add_card_to_battlefield(0, catalog::prosperity_tycoon());
        g.fire_self_etb_triggers(pt, 0);
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Mercenary"), 1, "ETB made a Mercenary");
        // Sac the token for indestructible.
        g.clear_sickness(pt);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: pt, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate sac-token ability");
        drain_stack(&mut g);
        assert_eq!(count_named(&g, 0, "Mercenary"), 0, "the token was sacrificed");
        assert!(
            g.computed_permanent(pt).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible)
        );
    }

    #[test]
    fn ambuscade_pumps_then_fights() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/2 after +1/+0
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_hand(0, catalog::ambuscade());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(foe)],
            mode: None,
            x_value: None,
        })
        .expect("cast Ambuscade");
        drain_stack(&mut g);
        // Pumped to power 3, dealt 3 to the 2/2 → it dies.
        assert!(g.battlefield_find(foe).is_none(), "3 damage killed the opposing 2/2");
    }

    #[test]
    fn nyxborn_unicorn_bestows_plus_two_two() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_hand(0, catalog::nyxborn_unicorn());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // Cast for bestow, enchanting the bear.
        g.perform_action(GameAction::CastBestow {
            card_id: id,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bestow onto the bear");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2 from the bestowed Unicorn");
    }

    #[test]
    fn iron_fist_pulverizer_burns_on_second_spell() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::iron_fist_pulverizer());
        g.players[1].life = 20;
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // First spell: no trigger.
        let s1 = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast_at(&mut g, s1, Target::Player(1));
        // Second spell triggers Iron-Fist for 2 to the opponent (auto-targeted).
        let s2 = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast_at(&mut g, s2, Target::Player(1));
        // Bolt 1 (3) + Bolt 2 (3) + Iron-Fist (2) = 8.
        assert_eq!(g.players[1].life, 12);
    }
}

mod recent68 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::effect::{Effect, EventKind, Selector};
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn resolve_spell(g: &mut GameState, def: crabomination::card::CardDefinition, targets: Vec<Target>) {
        let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        ctx.targets = targets;
        g.resolve_effect(&def.effect, &ctx).unwrap();
    }

    fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: idx, target, additional_targets: Vec::new(), x_value: None,
        }).expect("ability activates");
        drain_stack(g);
    }

    #[test]
    fn chrome_steed_metalcraft_pump() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::chrome_steed());
        assert_eq!(g.computed_permanent(id).unwrap().power, 2, "steed is one artifact → no metalcraft");
        for _ in 0..2 {
            g.add_card_to_battlefield(0, catalog::ornithopter());
        }
        assert_eq!(g.computed_permanent(id).unwrap().power, 4, "three artifacts → +2/+2");
    }

    #[test]
    fn vulshok_replica_sac_burns() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::vulshok_replica());
        let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        activate(&mut g, id, 0, Some(Target::Permanent(foe)));
        assert!(g.battlefield_find(id).is_none(), "sacrificed as a cost");
        assert!(g.battlefield_find(foe).is_none(), "3 damage kills the 3/3");
    }

    #[test]
    fn bloodhall_ooze_has_two_color_gated_upkeep_growers() {
        use crabomination::card::{Predicate, SelectionRequirement as R};
        use crabomination::mana::Color;
        let d = catalog::bloodhall_ooze();
        assert_eq!(d.triggered_abilities.len(), 2, "one per color");
        for (color, ab) in [Color::Black, Color::Green].iter().zip(&d.triggered_abilities) {
            assert!(matches!(ab.event.kind, EventKind::StepBegins(TurnStep::Upkeep)));
            let want = Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::HasColor(*color).and(R::ControlledByYou)),
                n: crabomination::card::Value::Const(1),
            };
            assert_eq!(ab.event.filter.as_ref(), Some(&want), "gated on controlling that color");
            assert!(matches!(ab.effect, Effect::MayDo { .. }), "may add a +1/+1 counter");
        }
    }

    #[test]
    fn sylvan_might_pumps_and_grants_trample() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        resolve_spell(&mut g, catalog::sylvan_might(), vec![Target::Permanent(mine)]);
        let cp = g.computed_permanent(mine).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
        assert!(cp.keywords.contains(&Keyword::Trample));
        assert!(catalog::sylvan_might().keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))));
    }

    #[test]
    fn nimble_innovator_draws_on_etb() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island()); // the card to draw
        let id = g.add_card_to_hand(0, catalog::nimble_innovator());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Nimble Innovator");
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).is_some(), "resolved onto the battlefield");
        assert_eq!(g.players[0].hand.len(), 1, "ETB drew the island");
    }

    #[test]
    fn barrage_ogre_sacs_artifact_to_burn() {
        let mut g = two_player_game();
        let ogre = g.add_card_to_battlefield(0, catalog::barrage_ogre());
        let art = g.add_card_to_battlefield(0, catalog::ornithopter());
        g.clear_sickness(ogre);
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        activate(&mut g, ogre, 0, Some(Target::Permanent(foe)));
        assert!(g.battlefield_find(art).is_none(), "sacrificed the artifact");
        assert!(g.battlefield_find(foe).is_none(), "2 damage kills the 2/2");
    }

    #[test]
    fn reckless_imp_flies_and_cant_block() {
        let imp = catalog::reckless_imp();
        assert!(imp.keywords.contains(&Keyword::Flying));
        assert!(imp.keywords.contains(&Keyword::CantBlock));
        assert!(imp.alternative_cost.is_some(), "has Dash");
    }

    #[test]
    fn colossodon_yearling_is_a_beast() {
        let c = catalog::colossodon_yearling();
        assert_eq!((c.power, c.toughness), (2, 4));
        assert!(c.subtypes.creature_types.contains(&crabomination::card::CreatureType::Beast));
    }
}

mod recent69 {
    use crabomination::card::{Keyword, LandType};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    #[test]
    fn frost_giant_has_rampage_2() {
        assert!(catalog::frost_giant().keywords.contains(&Keyword::Rampage(2)));
    }

    #[test]
    fn highland_game_gains_life_on_death() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::highland_game());
        let before = g.players[0].life;
        let evs = g.remove_to_graveyard_with_triggers(id);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, before + 2, "dies → gain 2 life");
    }

    #[test]
    fn rushwood_dryad_has_forestwalk() {
        assert!(catalog::rushwood_dryad().keywords.contains(&Keyword::Landwalk(LandType::Forest)));
    }

    #[test]
    fn ainok_tracker_first_strike_and_morph() {
        let d = catalog::ainok_tracker();
        assert!(d.keywords.contains(&Keyword::FirstStrike));
        assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Morph(_))));
    }

    #[test]
    fn charging_slateback_cant_block_and_morph() {
        let d = catalog::charging_slateback();
        assert!(d.keywords.contains(&Keyword::CantBlock));
        assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Morph(_))));
    }

    #[test]
    fn auriok_transfixer_taps_target_artifact() {
        let mut g = two_player_game();
        let tf = g.add_card_to_battlefield(0, catalog::auriok_transfixer());
        g.clear_sickness(tf);
        let art = g.add_card_to_battlefield(1, catalog::ornithopter());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: tf, ability_index: 0, target: Some(Target::Permanent(art)),
            additional_targets: Vec::new(), x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert!(g.battlefield_find(art).unwrap().tapped, "target artifact tapped");
    }

    #[test]
    fn snapping_creeper_gains_vigilance_on_landfall() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::snapping_creeper());
        assert!(!g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Vigilance));
        let land = g.add_card_to_battlefield(0, catalog::forest());
        g.dispatch_triggers_for_events(&[GameEvent::LandPlayed { player: 0, card_id: land, played: true }]);
        drain_stack(&mut g);
        assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Vigilance),
            "landfall grants vigilance until end of turn");
    }

    #[test]
    fn nyxborn_rollicker_bestows_plus_one() {
        let d = catalog::nyxborn_rollicker();
        assert!(d.bestow.is_some(), "has Bestow");
        let bonus = d.equipped_bonus.expect("bestow bonus");
        assert_eq!((bonus.power, bonus.toughness), (1, 1));
    }
}

mod recent70 {
    use crabomination::card::{CreatureType, Keyword, LandType};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    #[test]
    fn krosan_archer_discards_to_pump_toughness() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::krosan_archer());
        g.add_card_to_hand(0, catalog::island()); // card to discard
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(id).unwrap().toughness, 5, "2/3 → 2/5");
        assert!(catalog::krosan_archer().keywords.contains(&Keyword::Reach));
    }

    #[test]
    fn dwarven_grunt_has_mountainwalk() {
        assert!(catalog::dwarven_grunt().keywords.contains(&Keyword::Landwalk(LandType::Mountain)));
    }

    #[test]
    fn vengeful_firebrand_haste_gated_on_warrior_in_graveyard() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::vengeful_firebrand());
        assert!(!g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Haste),
            "no Warrior in graveyard → no haste");
        g.add_card_to_graveyard(0, catalog::sabertooth_outrider()); // a Human Warrior card
        assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Haste),
            "Warrior card in graveyard → haste");
    }

    #[test]
    fn anaba_shaman_pings_any_target() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::anaba_shaman());
        g.clear_sickness(id);
        let foe_life = g.players[1].life;
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: Vec::new(), x_value: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, foe_life - 1, "1 damage to the opponent");
    }

    #[test]
    fn balduvian_barbarians_is_a_vanilla_3_2() {
        let c = catalog::balduvian_barbarians();
        assert_eq!((c.power, c.toughness), (3, 2));
        assert!(c.subtypes.creature_types.contains(&CreatureType::Barbarian));
    }

    #[test]
    fn zephyr_falcon_flies_with_vigilance() {
        let d = catalog::zephyr_falcon();
        assert!(d.keywords.contains(&Keyword::Flying) && d.keywords.contains(&Keyword::Vigilance));
    }

    #[test]
    fn regal_unicorn_is_a_vanilla_2_3() {
        let c = catalog::regal_unicorn();
        assert_eq!((c.power, c.toughness), (2, 3));
        assert!(c.subtypes.creature_types.contains(&CreatureType::Unicorn));
    }
}
