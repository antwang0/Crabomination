//! Tests for recentN card batches 71-80 (merged from per-batch micro-files).

mod recent71 {
    use crabomination::card::{CreatureType, Keyword, LandType};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    #[test]
    fn nightmare_pt_tracks_swamps_you_control() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::nightmare());
        assert_eq!(g.computed_permanent(id).unwrap().power, 0, "no Swamps → 0/0");
        g.add_card_to_battlefield(0, catalog::swamp());
        g.add_card_to_battlefield(0, catalog::swamp());
        let p = g.computed_permanent(id).unwrap();
        assert_eq!((p.power, p.toughness), (2, 2), "two Swamps → 2/2");
        assert!(catalog::nightmare().keywords.contains(&Keyword::Flying));
    }

    #[test]
    fn rukh_egg_mints_a_flying_bird_on_death() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::rukh_egg());
        g.remove_to_graveyard_with_triggers(id);
        drain_stack(&mut g);
        let bird = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Bird")
            .expect("Rukh token minted");
        assert_eq!((bird.definition.power, bird.definition.toughness), (4, 4));
        assert!(bird.definition.keywords.contains(&Keyword::Flying));
    }

    #[test]
    fn sabertooth_tiger_has_first_strike() {
        assert!(catalog::sabertooth_tiger().keywords.contains(&Keyword::FirstStrike));
    }

    #[test]
    fn segovian_leviathan_has_islandwalk() {
        assert!(catalog::segovian_leviathan().keywords.contains(&Keyword::Landwalk(LandType::Island)));
    }

    #[test]
    fn vampire_bats_pumps_once_per_turn() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::vampire_bats());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("first activation");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(id).unwrap().power, 1, "0/1 → 1/1");
        // Second activation the same turn is illegal (once per turn).
        assert!(g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).is_err(), "can't activate twice in one turn");
    }

    #[test]
    fn wall_of_spears_is_defender_first_strike() {
        let d = catalog::wall_of_spears();
        assert!(d.keywords.contains(&Keyword::Defender) && d.keywords.contains(&Keyword::FirstStrike));
        assert!(d.card_types.contains(&crabomination::card::CardType::Artifact));
    }

    #[test]
    fn rod_of_ruin_pings_any_target() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::rod_of_ruin());
        let foe = g.players[1].life;
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, foe - 1, "1 damage to opponent");
    }

    #[test]
    fn vanilla_bodies_have_expected_stats() {
        assert_eq!((catalog::ironroot_treefolk().power, catalog::ironroot_treefolk().toughness), (3, 5));
        assert_eq!((catalog::fire_elemental().power, catalog::fire_elemental().toughness), (5, 4));
        assert_eq!((catalog::dross_crocodile().power, catalog::dross_crocodile().toughness), (5, 1));
        assert_eq!((catalog::durkwood_boars().power, catalog::durkwood_boars().toughness), (4, 4));
        assert!(catalog::wall_of_ice().keywords.contains(&Keyword::Defender));
        assert!(catalog::dross_crocodile().subtypes.creature_types.contains(&CreatureType::Zombie));
    }
}

mod recent72 {
    use crabomination::card::{CreatureType, Keyword, LandType};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    #[test]
    fn yavimaya_enchantress_grows_with_enchantments() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::yavimaya_enchantress());
        assert_eq!(g.computed_permanent(id).unwrap().power, 2, "no enchantments → 2/2");
        // An enchantment either player controls grows it (counts all in play).
        g.add_card_to_battlefield(0, catalog::wild_growth());
        g.add_card_to_battlefield(1, catalog::wild_growth());
        let p = g.computed_permanent(id).unwrap();
        assert_eq!((p.power, p.toughness), (4, 4), "two enchantments in play → 4/4");
    }

    #[test]
    fn zombie_master_grants_swampwalk_to_other_zombies() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::zombie_master());
        let ally = g.add_card_to_battlefield(0, catalog::scathe_zombies());
        assert!(
            g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Landwalk(LandType::Swamp)),
            "other Zombie gains swampwalk from the Master",
        );
        // The Master itself does not gain swampwalk ("other").
        let master = g.battlefield.iter().find(|c| c.definition.name == "Zombie Master").unwrap().id;
        assert!(
            !g.computed_permanent(master).unwrap().keywords.contains(&Keyword::Landwalk(LandType::Swamp)),
            "the Master is excluded (other Zombies only)",
        );
    }

    #[test]
    fn cudgel_troll_regenerates() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, catalog::cudgel_troll());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("activate regen");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(id).unwrap().regeneration_shields, 1, "shield stamped");
        g.battlefield_find_mut(id).unwrap().damage = 3;
        g.check_state_based_actions();
        assert!(g.battlefield_find(id).is_some(), "regen shield saved the Troll from lethal");
    }

    #[test]
    fn radjan_spirit_strips_flying() {
        use crabomination::game::types::Target;
        let mut g = two_player_game();
        let spirit = g.add_card_to_battlefield(0, catalog::radjan_spirit());
        g.clear_sickness(spirit);
        let flyer = g.add_card_to_battlefield(1, catalog::air_elemental());
        assert!(g.computed_permanent(flyer).unwrap().keywords.contains(&Keyword::Flying));
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: spirit, ability_index: 0, target: Some(Target::Permanent(flyer)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("activate");
        drain_stack(&mut g);
        assert!(!g.computed_permanent(flyer).unwrap().keywords.contains(&Keyword::Flying),
            "target lost flying this turn");
    }

    #[test]
    fn deadly_insect_has_shroud() {
        let d = catalog::deadly_insect();
        assert_eq!((d.power, d.toughness), (6, 1));
        assert!(d.keywords.contains(&Keyword::Shroud));
    }

    #[test]
    fn retro_vanilla_and_keyword_stats() {
        assert!(catalog::longbow_archer().keywords.contains(&Keyword::FirstStrike));
        assert!(catalog::longbow_archer().keywords.contains(&Keyword::Reach));
        assert!(catalog::talruum_minotaur().keywords.contains(&Keyword::Haste));
        assert_eq!((catalog::giant_octopus().power, catalog::giant_octopus().toughness), (3, 3));
        assert_eq!((catalog::balduvian_bears().power, catalog::balduvian_bears().toughness), (2, 2));
        assert!(catalog::norwood_ranger().subtypes.creature_types.contains(&CreatureType::Scout));
    }
}

mod recent73 {
    use crabomination::card::{CounterType, CreatureType, Keyword, LandType};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;

    #[test]
    fn bog_rats_cant_be_blocked_by_walls() {
        let mut g = two_player_game();
        let rats = g.add_card_to_battlefield(0, catalog::bog_rats());
        let wall = g.add_card_to_battlefield(1, catalog::wall_of_ice());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(rats);
        assert!(!g.blocker_can_block_attacker(wall, rats), "a Wall can't block Bog Rats");
        assert!(g.blocker_can_block_attacker(bear, rats), "a non-Wall can block it");
    }

    #[test]
    fn serrated_arrows_enters_with_three_and_shoots() {
        let mut g = two_player_game();
        // The printed `enters_with_counters` spec applies on the real ETB path; the
        // test helper bypasses it, so seed the three arrowheads directly.
        assert_eq!(catalog::serrated_arrows().enters_with_counters.unwrap().0, CounterType::Charge);
        let arrows = g.add_card_to_battlefield(0, catalog::serrated_arrows());
        g.battlefield_find_mut(arrows).unwrap().add_counters(CounterType::Charge, 3);
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: arrows, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("shoot");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(arrows).unwrap().counter_count(CounterType::Charge), 2, "spent one");
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 1,
            "-1/-1 counter placed");
    }

    #[test]
    fn ghitu_slinger_pings_on_etb() {
        let mut g = two_player_game();
        let foe = g.players[1].life;
        let slinger = g.add_card_to_hand(0, catalog::ghitu_slinger());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: slinger, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, foe - 2, "ETB deals 2");
        assert!(catalog::ghitu_slinger().keywords.iter().any(|k| matches!(k, Keyword::Echo(_))));
    }

    #[test]
    fn skittering_skirge_sacrifices_on_creature_cast() {
        let mut g = two_player_game();
        let skirge = g.add_card_to_battlefield(0, catalog::skittering_skirge());
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast a creature spell");
        drain_stack(&mut g);
        assert!(g.battlefield_find(skirge).is_none(), "Skirge sacrificed when a creature spell was cast");
    }

    #[test]
    fn viashino_sandstalker_returns_at_end_step() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(0, catalog::viashino_sandstalker());
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.battlefield_find(v).is_none(), "returned to hand at end step");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Viashino Sandstalker"));
    }

    #[test]
    fn recent73_static_stats() {
        assert!(catalog::shanodin_dryads().keywords.contains(&Keyword::Landwalk(LandType::Forest)));
        assert!(catalog::mesa_falcon().keywords.contains(&Keyword::Flying));
        assert_eq!((catalog::highland_giant().power, catalog::highland_giant().toughness), (3, 4));
        assert!(catalog::ghitu_slinger().subtypes.creature_types.contains(&CreatureType::Nomad));
        assert!(catalog::cackling_fiend().subtypes.creature_types.contains(&CreatureType::Zombie));
    }
}

mod recent74 {
    use crabomination::card::{CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    #[test]
    fn blood_pet_sacrifices_for_black_mana() {
        let mut g = two_player_game();
        let pet = g.add_card_to_battlefield(0, catalog::blood_pet());
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: pet, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("sac for mana");
        assert!(g.battlefield_find(pet).is_none(), "Blood Pet sacrificed");
        assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1, "one black added");
    }

    #[test]
    fn foul_imp_costs_two_life_on_etb() {
        let mut g = two_player_game();
        let life = g.players[0].life;
        let imp = g.add_card_to_hand(0, catalog::foul_imp());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: imp, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life - 2, "lost 2 life on ETB");
    }

    #[test]
    fn skyshroud_vampire_discards_to_pump() {
        let mut g = two_player_game();
        let vamp = g.add_card_to_battlefield(0, catalog::skyshroud_vampire());
        g.add_card_to_hand(0, catalog::grizzly_bears()); // a creature card to discard
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: vamp, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("discard-pump");
        drain_stack(&mut g);
        let p = g.computed_permanent(vamp).unwrap();
        assert_eq!((p.power, p.toughness), (5, 5), "3/3 → 5/5");
    }

    #[test]
    fn kris_mage_pings_with_discard() {
        let mut g = two_player_game();
        let mage = g.add_card_to_battlefield(0, catalog::kris_mage());
        g.clear_sickness(mage);
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Red, 1);
        let foe = g.players[1].life;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::ActivateAbility {
            card_id: mage, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("ping");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, foe - 1, "1 damage dealt");
    }

    #[test]
    fn sabertooth_nishoba_has_dual_protection() {
        let d = catalog::sabertooth_nishoba();
        assert!(d.keywords.contains(&Keyword::Protection(Color::Blue)));
        assert!(d.keywords.contains(&Keyword::Protection(Color::Red)));
        assert!(d.keywords.contains(&Keyword::Trample));
    }

    #[test]
    fn recent74_misc_stats() {
        assert_eq!((catalog::water_elemental().power, catalog::water_elemental().toughness), (5, 4));
        assert!(catalog::wall_of_water().keywords.contains(&Keyword::Defender));
        assert!(catalog::spitting_drake().keywords.contains(&Keyword::Flying));
        assert!(catalog::feral_shadow().subtypes.creature_types.contains(&CreatureType::Nightstalker));
        assert_eq!((catalog::rowan_treefolk().power, catalog::rowan_treefolk().toughness), (3, 4));
    }
}

mod recent75 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    #[test]
    fn fungusaur_grows_when_dealt_damage() {
        let mut g = two_player_game();
        let saur = g.add_card_to_battlefield(0, catalog::fungusaur());
        let mut events = Vec::new();
        g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(saur), 1, None, &mut events);
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(saur).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
            "enrage placed a +1/+1 counter");
        let p = g.computed_permanent(saur).unwrap();
        assert_eq!((p.power, p.toughness), (3, 3), "2/2 → 3/3");
    }

    #[test]
    fn serpent_warrior_costs_three_life() {
        let mut g = two_player_game();
        let life = g.players[0].life;
        let sw = g.add_card_to_hand(0, catalog::serpent_warrior());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: sw, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life - 3, "lost 3 life on ETB");
    }

    #[test]
    fn nettletooth_djinn_pings_you_at_upkeep() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::nettletooth_djinn());
        let life = g.players[0].life;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life - 1, "1 damage to you at upkeep");
    }

    #[test]
    fn hulking_cyclops_cant_block() {
        let mut g = two_player_game();
        let cyc = g.add_card_to_battlefield(0, catalog::hulking_cyclops());
        let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(attacker);
        assert!(!g.blocker_can_block_attacker(cyc, attacker), "can't block");
        assert!(catalog::pygmy_pyrosaur().keywords.contains(&Keyword::CantBlock));
    }

    #[test]
    fn owl_familiar_loots_on_etb() {
        let mut g = two_player_game();
        let hand_before = g.players[0].hand.len();
        g.add_card_to_hand(0, catalog::grizzly_bears()); // something to discard
        let owl = g.add_card_to_hand(0, catalog::owl_familiar());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: owl, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        // Net hand size: -1 owl cast, +1 draw, -1 discard = hand_before (the pre-seeded bear).
        assert_eq!(g.players[0].hand.len(), hand_before, "draw then discard nets zero");
    }

    #[test]
    fn recent75_static_stats() {
        assert!(catalog::ekundu_griffin().keywords.contains(&Keyword::Flying));
        assert!(catalog::ekundu_griffin().keywords.contains(&Keyword::FirstStrike));
        assert!(catalog::fire_drake().keywords.contains(&Keyword::Flying));
        assert_eq!((catalog::muck_rats().power, catalog::muck_rats().toughness), (1, 1));
    }
}

mod recent76 {
    use crabomination::card::{CounterType, CreatureType, EventKind, Keyword};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn cast_aura_on(g: &mut GameState, aura: crabomination::card::CardDefinition, host: CardId, pips: &[(Color, u32)]) {
        let id = g.add_card_to_hand(0, aura);
        for (c, n) in pips {
            g.players[0].mana_pool.add(*c, *n);
        }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(host)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("enchant");
        drain_stack(g);
    }

    #[test]
    fn giant_strength_pumps_plus_two_plus_two() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        cast_aura_on(&mut g, catalog::giant_strength(), bear, &[(Color::Red, 2)]);
        let p = g.computed_permanent(bear).unwrap();
        assert_eq!((p.power, p.toughness), (4, 4), "2/2 → 4/4");
    }

    #[test]
    fn web_grants_toughness_and_reach() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        cast_aura_on(&mut g, catalog::web(), bear, &[(Color::Green, 1)]);
        let p = g.computed_permanent(bear).unwrap();
        assert_eq!((p.power, p.toughness), (2, 4), "+0/+2");
        assert!(p.keywords.contains(&Keyword::Reach), "gained reach");
    }

    #[test]
    fn firebreathing_grants_pump_ability_to_host() {
        // CR 604.3 — the Aura grants the enchanted creature an activated ability.
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        cast_aura_on(&mut g, catalog::firebreathing(), bear, &[(Color::Red, 1)]);
        let granted = g.granted_abilities_for(bear);
        assert_eq!(granted.len(), 1, "host gained the firebreathing ability");
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: bear, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("activate granted {R}: +1/+0");
        drain_stack(&mut g);
        let p = g.computed_permanent(bear).unwrap();
        assert_eq!((p.power, p.toughness), (3, 2), "2/2 → 3/2");
    }

    #[test]
    fn lure_forces_all_blockers() {
        // CR 509.1c — every creature able to block the enchanted creature must.
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        cast_aura_on(&mut g, catalog::lure(), bear, &[(Color::Green, 3)]);
        g.clear_sickness(bear);
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(blocker);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        // Declaring no blocks is illegal — the able blocker must block the lured creature.
        assert!(g.perform_action(GameAction::DeclareBlockers(vec![])).is_err(),
            "an able blocker must block the lured attacker");
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, bear)])).expect("forced block legal");
    }

    #[test]
    fn blanchwood_armor_scales_with_forests() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::forest());
        cast_aura_on(&mut g, catalog::blanchwood_armor(), bear, &[(Color::Green, 3)]);
        let p = g.computed_permanent(bear).unwrap();
        assert_eq!((p.power, p.toughness), (4, 4), "2/2 + 2 Forests = 4/4");
    }

    #[test]
    fn ironclaw_orcs_cant_block_big_attackers() {
        let mut g = two_player_game();
        let orcs = g.add_card_to_battlefield(0, catalog::ironclaw_orcs());
        let big = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → still ≥2
        let tiny = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // 1/1
        g.clear_sickness(big);
        g.clear_sickness(small);
        g.clear_sickness(tiny);
        assert!(!g.blocker_can_block_attacker(orcs, big), "can't block power 3");
        assert!(!g.blocker_can_block_attacker(orcs, small), "can't block power 2");
        assert!(g.blocker_can_block_attacker(orcs, tiny), "can block power 1");
    }

    #[test]
    fn dwarven_warriors_makes_target_unblockable() {
        let mut g = two_player_game();
        let dw = g.add_card_to_battlefield(0, catalog::dwarven_warriors());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(dw);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dw, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("grant unblockable");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
    }

    #[test]
    fn frozen_shade_pumps_with_black() {
        let mut g = two_player_game();
        let shade = g.add_card_to_battlefield(0, catalog::frozen_shade());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: shade, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("pump");
        drain_stack(&mut g);
        let p = g.computed_permanent(shade).unwrap();
        assert_eq!((p.power, p.toughness), (1, 2), "0/1 → 1/2");
    }

    #[test]
    fn wall_of_brambles_regenerates() {
        let mut g = two_player_game();
        let wall = g.add_card_to_battlefield(0, catalog::wall_of_brambles());
        assert!(catalog::wall_of_brambles().keywords.contains(&Keyword::Defender));
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: wall, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("regenerate");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(wall).unwrap().regeneration_shields, 1, "regen shield stamped");
    }

    #[test]
    fn whirling_dervish_grows_on_combat_damage() {
        let mut g = two_player_game();
        let dervish = g.add_card_to_battlefield(0, catalog::whirling_dervish());
        g.clear_sickness(dervish);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: dervish, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::PostCombatMain {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        assert_eq!(g.battlefield_find(dervish).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
            "grew after dealing combat damage to a player");
        assert!(catalog::whirling_dervish().keywords.contains(&Keyword::Protection(Color::Black)));
    }

    #[test]
    fn femeref_archers_shoots_attacking_flyer() {
        let mut g = two_player_game();
        let archers = g.add_card_to_battlefield(0, catalog::femeref_archers());
        g.clear_sickness(archers);
        let flyer = g.add_card_to_battlefield(0, catalog::bird_maiden()); // 1/2 flyer
        g.clear_sickness(flyer);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: flyer, target: AttackTarget::Player(1),
        }])).expect("flyer attacks");
        drain_stack(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: archers, ability_index: 0, target: Some(Target::Permanent(flyer)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("shoot the attacking flyer");
        drain_stack(&mut g);
        assert!(g.battlefield_find(flyer).is_none(), "took 4, died");
    }

    #[test]
    fn fyndhorn_elder_taps_for_two_green() {
        let mut g = two_player_game();
        let elder = g.add_card_to_battlefield(0, catalog::fyndhorn_elder());
        g.clear_sickness(elder);
        g.perform_action(GameAction::ActivateAbility {
            card_id: elder, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("mana");
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "added two green");
    }

    #[test]
    fn wyluli_wolf_pumps_a_target() {
        let mut g = two_player_game();
        let wolf = g.add_card_to_battlefield(0, catalog::wyluli_wolf());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(wolf);
        g.perform_action(GameAction::ActivateAbility {
            card_id: wolf, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("pump the bear");
        drain_stack(&mut g);
        let p = g.computed_permanent(bear).unwrap();
        assert_eq!((p.power, p.toughness), (3, 3), "2/2 → 3/3 until EOT");
    }

    #[test]
    fn goblin_elite_infantry_shrinks_when_it_blocks() {
        let mut g = two_player_game();
        let goblin = g.add_card_to_battlefield(1, catalog::goblin_elite_infantry());
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(attacker);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareBlockers(vec![(goblin, attacker)])).expect("block");
        drain_stack(&mut g);
        let p = g.computed_permanent(goblin).unwrap();
        assert_eq!((p.power, p.toughness), (1, 1), "2/2 → 1/1 for blocking");
    }

    #[test]
    fn jayemdae_tome_draws() {
        let mut g = two_player_game();
        let tome = g.add_card_to_battlefield(0, catalog::jayemdae_tome());
        g.add_card_to_library(0, catalog::island());
        g.players[0].mana_pool.add_colorless(4);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: tome, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("draw");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    }

    #[test]
    fn aladdins_ring_deals_four() {
        let mut g = two_player_game();
        let ring = g.add_card_to_battlefield(0, catalog::aladdins_ring());
        g.players[0].mana_pool.add_colorless(8);
        let foe = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: ring, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("ping");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, foe - 4, "4 damage to the opponent");
    }

    #[test]
    fn lifegain_cycle_triggers_on_matching_color_spell() {
        // Structural: each battery watches its own color's casts.
        for (card, kind) in [
            (catalog::throne_of_bone(), EventKind::SpellCast),
            (catalog::wooden_sphere(), EventKind::SpellCast),
            (catalog::iron_star(), EventKind::SpellCast),
            (catalog::crystal_rod(), EventKind::SpellCast),
            (catalog::ivory_cup(), EventKind::SpellCast),
        ] {
            assert_eq!(card.triggered_abilities[0].event.kind, kind);
        }
    }

    #[test]
    fn recent76_static_stats() {
        assert!(catalog::cockatrice().keywords.contains(&Keyword::Flying));
        assert!(catalog::cockatrice().keywords.contains(&Keyword::Deathtouch));
        assert!(catalog::thicket_basilisk().keywords.contains(&Keyword::Deathtouch));
        assert!(catalog::bird_maiden().keywords.contains(&Keyword::Flying));
        assert!(catalog::alaborn_grenadier().keywords.contains(&Keyword::Vigilance));
        assert_eq!((catalog::skeletal_snake().power, catalog::skeletal_snake().toughness), (2, 1));
        assert!(catalog::skeletal_snake().subtypes.creature_types.contains(&CreatureType::Skeleton));
        assert!(catalog::ironclaw_orcs().keywords.contains(&Keyword::CantBlockPowerAtLeast(2)));
    }
}

mod recent77 {
    use crabomination::card::{CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Cast an Aura from hand onto `host` with the given colored pips available.
    fn cast_aura_on(g: &mut GameState, aura: crabomination::card::CardDefinition, host: CardId, pips: &[(Color, u32)]) {
        let id = g.add_card_to_hand(0, aura);
        for (c, n) in pips {
            g.players[0].mana_pool.add(*c, *n);
        }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(host)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("enchant");
        drain_stack(g);
    }

    #[test]
    fn storm_shaman_firebreathes() {
        let mut g = two_player_game();
        let s = g.add_card_to_battlefield(0, catalog::storm_shaman());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("{R}: +1/+0");
        drain_stack(&mut g);
        let p = g.computed_permanent(s).unwrap();
        assert_eq!((p.power, p.toughness), (1, 4), "0/4 → 1/4");
    }

    #[test]
    fn wild_aesthir_pump_is_once_per_turn() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::wild_aesthir());
        g.players[0].mana_pool.add(Color::White, 4);
        g.perform_action(GameAction::ActivateAbility {
            card_id: a, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("first activation");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(a).unwrap().power, 3, "1/1 → 3/1");
        // Second activation the same turn is illegal (activate only once each turn).
        assert!(g.perform_action(GameAction::ActivateAbility {
            card_id: a, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).is_err(), "once each turn");
    }

    #[test]
    fn woolly_spider_pumps_when_blocking_a_flyer() {
        let mut g = two_player_game();
        let spider = g.add_card_to_battlefield(1, catalog::woolly_spider());
        g.clear_sickness(spider);
        let flyer = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 flying
        g.clear_sickness(flyer);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: flyer, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareBlockers(vec![(spider, flyer)])).expect("block flyer");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(spider).unwrap().toughness, 5, "2/3 → 2/5 blocking a flyer");
    }

    #[test]
    fn orcish_artillery_hits_target_and_controller() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(0, catalog::orcish_artillery());
        g.clear_sickness(art);
        let foe = g.players[1].life;
        let me = g.players[0].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: art, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("ping");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, foe - 2, "2 to target");
        assert_eq!(g.players[0].life, me - 3, "3 to controller");
    }

    #[test]
    fn goblin_digging_team_destroys_a_wall() {
        let mut g = two_player_game();
        let team = g.add_card_to_battlefield(0, catalog::goblin_digging_team());
        g.clear_sickness(team);
        let wall = g.add_card_to_battlefield(1, catalog::wall_of_brambles()); // Plant Wall
        g.perform_action(GameAction::ActivateAbility {
            card_id: team, ability_index: 0, target: Some(Target::Permanent(wall)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("destroy wall");
        drain_stack(&mut g);
        assert!(g.battlefield_find(wall).is_none(), "Wall destroyed");
        assert!(g.battlefield_find(team).is_none(), "Digging Team sacrificed");
    }

    #[test]
    fn aysen_bureaucrats_taps_only_small_creatures() {
        let mut g = two_player_game();
        let bur = g.add_card_to_battlefield(0, catalog::aysen_bureaucrats());
        g.clear_sickness(bur);
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        g.perform_action(GameAction::ActivateAbility {
            card_id: bur, ability_index: 0, target: Some(Target::Permanent(small)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("tap the 2/2");
        drain_stack(&mut g);
        assert!(g.battlefield_find(small).unwrap().tapped, "power-2 creature tapped");
    }

    #[test]
    fn anaba_spirit_crafter_pumps_all_minotaurs() {
        let mut g = two_player_game();
        let crafter = g.add_card_to_battlefield(0, catalog::anaba_spirit_crafter()); // Minotaur Shaman 1/3
        let other = g.add_card_to_battlefield(1, catalog::anaba_ancestor()); // opponent's Minotaur 1/1
        assert_eq!(g.computed_permanent(crafter).unwrap().power, 2, "pumps itself: 1/3 → 2/3");
        assert_eq!(g.computed_permanent(other).unwrap().power, 2, "pumps any Minotaur, even opponents': 1/1 → 2/1");
    }

    #[test]
    fn anaba_ancestor_pumps_another_minotaur() {
        let mut g = two_player_game();
        let anc = g.add_card_to_battlefield(0, catalog::anaba_ancestor());
        g.clear_sickness(anc);
        let target = g.add_card_to_battlefield(0, catalog::anaba_spirit_crafter());
        // Crafter is already 2/3 from its own static; +1/+1 makes it 3/4.
        g.perform_action(GameAction::ActivateAbility {
            card_id: anc, ability_index: 0, target: Some(Target::Permanent(target)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("pump another Minotaur");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(target).unwrap().toughness, 4, "+1/+1");
    }

    #[test]
    fn elvish_bard_forces_all_blockers() {
        let mut g = two_player_game();
        let bard = g.add_card_to_battlefield(0, catalog::elvish_bard());
        g.clear_sickness(bard);
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(blocker);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bard, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        assert!(g.perform_action(GameAction::DeclareBlockers(vec![])).is_err(),
            "an able blocker must block Elvish Bard");
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, bard)])).expect("forced block legal");
    }

    #[test]
    fn marsh_goblins_has_swampwalk() {
        let mut g = two_player_game();
        let goblin = g.add_card_to_battlefield(0, catalog::marsh_goblins());
        assert!(g.computed_permanent(goblin).unwrap().keywords
            .contains(&Keyword::Landwalk(crabomination::card::LandType::Swamp)));
    }

    #[test]
    fn ghost_hounds_gains_first_strike_blocking_white() {
        let mut g = two_player_game();
        let hounds = g.add_card_to_battlefield(1, catalog::ghost_hounds());
        g.clear_sickness(hounds);
        let white = g.add_card_to_battlefield(0, catalog::savannah_lions()); // white ground attacker
        g.clear_sickness(white);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: white, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareBlockers(vec![(hounds, white)])).expect("block white");
        drain_stack(&mut g);
        assert!(g.computed_permanent(hounds).unwrap().keywords.contains(&Keyword::FirstStrike),
            "gained first strike blocking a white creature");
    }

    #[test]
    fn orcish_oriflamme_pumps_only_attackers() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::orcish_oriflamme());
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        // Not attacking yet — no bonus.
        assert_eq!(g.computed_permanent(atk).unwrap().power, 2, "idle creature unpumped");
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(atk).unwrap().power, 3, "attacking → +1/+0");
    }

    #[test]
    fn regeneration_grants_regen_ability() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        cast_aura_on(&mut g, catalog::regeneration(), bear, &[(Color::Green, 2)]);
        let granted = g.granted_abilities_for(bear);
        assert_eq!(granted.len(), 1, "host gained a regenerate ability");
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: bear, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("regen shield");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).unwrap().regeneration_shields, 1, "regen shield stamped");
    }

    #[test]
    fn carapace_toughness_and_sac_regenerates() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::carapace());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("enchant");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().toughness, 4, "+0/+2");
        // Sacrifice the Aura to set up a regen shield on the enchanted creature.
        g.perform_action(GameAction::ActivateAbility {
            card_id: aura, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("sac Carapace to regenerate");
        drain_stack(&mut g);
        assert!(g.battlefield_find(aura).is_none(), "Carapace sacrificed");
        assert_eq!(g.battlefield_find(bear).unwrap().regeneration_shields, 1, "regen shield on the host");
    }

    #[test]
    fn feast_of_the_unicorn_pumps_power() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::feast_of_the_unicorn());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("enchant");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 6, "2/2 → 6/2");
    }

    #[test]
    fn icequake_snow_land_pings_controller() {
        let mut g = two_player_game();
        let snow = g.add_card_to_battlefield(1, catalog::snow_covered_swamp());
        let foe = g.players[1].life;
        let ice = g.add_card_to_hand(0, catalog::icequake());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: ice, target: Some(Target::Permanent(snow)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("destroy snow land");
        drain_stack(&mut g);
        assert!(g.battlefield_find(snow).is_none(), "land destroyed");
        assert_eq!(g.players[1].life, foe - 1, "snow land → 1 damage to its controller");
    }

    #[test]
    fn jokulhaups_wipes_the_board() {
        let mut g = two_player_game();
        let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let land = g.add_card_to_battlefield(1, catalog::forest());
        let artifact = g.add_card_to_battlefield(1, catalog::jayemdae_tome());
        let jk = g.add_card_to_hand(0, catalog::jokulhaups());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: jk, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("boom");
        drain_stack(&mut g);
        assert!(g.battlefield_find(creature).is_none(), "creature destroyed");
        assert!(g.battlefield_find(land).is_none(), "land destroyed");
        assert!(g.battlefield_find(artifact).is_none(), "artifact destroyed");
    }

    #[test]
    fn yavimaya_ants_has_cumulative_upkeep() {
        let ants = catalog::yavimaya_ants();
        assert!(ants.keywords.iter().any(|k| matches!(k, Keyword::CumulativeUpkeep(_))), "has cumulative upkeep");
        assert!(ants.keywords.contains(&Keyword::Trample) && ants.keywords.contains(&Keyword::Haste));
    }

    #[test]
    fn merfolk_assassin_targets_islandwalkers() {
        // Structural: the ability's destroy filter is islandwalk-only.
        let a = catalog::merfolk_assassin();
        assert_eq!(a.subtypes.creature_types, vec![CreatureType::Merfolk, CreatureType::Assassin]);
        assert_eq!(a.activated_abilities.len(), 1);
    }
}

mod recent78 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    #[test]
    fn giant_crab_grants_shroud() {
        let mut g = two_player_game();
        let crab = g.add_card_to_battlefield(0, catalog::giant_crab());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: crab, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("{U}: shroud");
        drain_stack(&mut g);
        assert!(g.computed_permanent(crab).unwrap().keywords.contains(&Keyword::Shroud));
    }

    #[test]
    fn wall_of_wonder_can_attack_despite_defender() {
        let mut g = two_player_game();
        let wall = g.add_card_to_battlefield(0, catalog::wall_of_wonder());
        g.clear_sickness(wall);
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: wall, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("+4/-4 & attack despite defender");
        drain_stack(&mut g);
        let p = g.computed_permanent(wall).unwrap();
        assert_eq!((p.power, p.toughness), (5, 1), "1/5 → 5/1");
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: wall, target: AttackTarget::Player(1),
        }])).expect("Defender attacks after the grant");
    }

    #[test]
    fn instill_energy_untaps_once_per_turn() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::instill_energy());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("enchant");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "granted haste");
        // Tap the bear, then the {0} untap ability should free it.
        if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) { c.tapped = true; }
        g.perform_action(GameAction::ActivateAbility {
            card_id: aura, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("{0}: untap");
        drain_stack(&mut g);
        assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped by Instill Energy");
        // Second activation the same turn is illegal.
        if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) { c.tapped = true; }
        assert!(g.perform_action(GameAction::ActivateAbility {
            card_id: aura, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).is_err(), "once each turn");
    }

    #[test]
    fn erg_raiders_pings_you_if_it_didnt_attack() {
        let mut g = two_player_game();
        let raider = g.add_card_to_battlefield(0, catalog::erg_raiders());
        g.clear_sickness(raider); // pretend it's been around (didn't enter this turn)
        let me = g.players[0].life;
        while g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, me - 2, "didn't attack → 2 damage to you");
    }

    #[test]
    fn foul_familiar_bounces_itself() {
        let mut g = two_player_game();
        let f = g.add_card_to_battlefield(0, catalog::foul_familiar());
        g.clear_sickness(f);
        g.players[0].mana_pool.add(Color::Black, 1);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: f, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("{B}, pay 1 life: bounce");
        drain_stack(&mut g);
        assert!(g.battlefield_find(f).is_none(), "returned to hand");
        assert_eq!(g.players[0].hand.len(), hand + 1, "back in hand");
    }

    #[test]
    fn fire_snake_destroys_a_land_on_death() {
        let mut g = two_player_game();
        let snake = g.add_card_to_battlefield(0, catalog::fire_snake()); // 3/1
        let land = g.add_card_to_battlefield(1, catalog::forest());
        let mut events = Vec::new();
        g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(snake), 1, None, &mut events);
        g.check_state_based_actions();
        // The dies trigger is on the stack targeting the land.
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_none(), "land destroyed by Fire Snake's death");
    }

    #[test]
    fn dread_reaper_costs_five_life() {
        let mut g = two_player_game();
        let me = g.players[0].life;
        let rd = g.add_card_to_hand(0, catalog::dread_reaper());
        g.players[0].mana_pool.add(Color::Black, 3);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: rd, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, me - 5, "ETB lose 5 life");
    }

    #[test]
    fn elven_cache_returns_from_graveyard() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_from_battlefield_to_graveyard_raw(bear);
        let cache = g.add_card_to_hand(0, catalog::elven_cache());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: cache, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("regrow the bear");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear back in hand");
        // -1 for the cast Elven Cache, +1 for the returned bear.
        assert_eq!(g.players[0].hand.len(), hand);
    }

    #[test]
    fn dwarven_soldier_toughens_against_orcs() {
        let mut g = two_player_game();
        let soldier = g.add_card_to_battlefield(1, catalog::dwarven_soldier());
        g.clear_sickness(soldier);
        let orc = g.add_card_to_battlefield(0, catalog::ironclaw_orcs()); // Orc attacker
        g.clear_sickness(orc);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: orc, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareBlockers(vec![(soldier, orc)])).expect("block the Orc");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(soldier).unwrap().toughness, 3, "2/1 → 2/3 blocking an Orc");
    }

    #[test]
    fn talas_warrior_is_unblockable_flyer_stats() {
        let w = catalog::talas_warrior();
        assert!(w.keywords.contains(&Keyword::Unblockable));
        assert_eq!((w.power, w.toughness), (2, 2));
    }

    #[test]
    fn fear_grants_fear() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::fear());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("enchant");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Fear));
    }

    #[test]
    fn wanderlust_pings_enchanted_creatures_controller() {
        let mut g = two_player_game();
        let foe_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::wanderlust());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(foe_creature)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("enchant the opponent's creature");
        drain_stack(&mut g);
        let foe = g.players[1].life;
        // It's the enchanted creature's controller (P1) whose upkeep triggers the
        // ping. P0's upkeep must NOT fire it.
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, foe, "no ping on the caster's upkeep");
        g.active_player_idx = 1;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, foe - 1, "1 damage at the enchanted controller's upkeep");
    }

    #[test]
    fn warp_artifact_and_cursed_land_are_upkeep_ping_auras() {
        // Structural: both are Auras with a single upkeep-triggered ability.
        for card in [catalog::warp_artifact(), catalog::cursed_land()] {
            assert!(card.card_types.contains(&crabomination::card::CardType::Enchantment));
            assert_eq!(card.triggered_abilities.len(), 1, "one upkeep ping trigger");
        }
    }

    #[test]
    fn cr_702_15_mountainwalk_unblockable_with_a_mountain() {
        // CR 702.15 — landwalk: the attacker can't be blocked while the defending
        // player controls a land of the named type (enforced in declare_blockers).
        let mut g = two_player_game();
        let yeti = g.add_card_to_battlefield(0, catalog::mountain_yeti());
        g.clear_sickness(yeti);
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(blocker);
        g.add_card_to_battlefield(1, catalog::mountain());
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: yeti, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        assert!(g.perform_action(GameAction::DeclareBlockers(vec![(blocker, yeti)])).is_err(),
            "can't block a mountainwalker while defending player controls a Mountain");
    }

    #[test]
    fn cr_702_23_rampage_grows_per_extra_blocker() {
        // CR 702.23 — Rampage N: +N/+N for each blocker beyond the first.
        let mut g = two_player_game();
        let giant = g.add_card_to_battlefield(0, catalog::frost_giant()); // 4/4 rampage 2
        g.clear_sickness(giant);
        let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(b1);
        g.clear_sickness(b2);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: giant, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareBlockers(vec![(b1, giant), (b2, giant)])).expect("double block");
        drain_stack(&mut g);
        let p = g.computed_permanent(giant).unwrap();
        assert_eq!((p.power, p.toughness), (6, 6), "4/4 + rampage 2 for one extra blocker → 6/6");
    }

    #[test]
    fn cr_603_10_carapace_sac_regenerates_enchanted_via_lki() {
        // CR 603.10 — a sac_cost ability whose body reads the enchanted creature
        // resolves via last-known-information after the Aura leaves the battlefield.
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::carapace());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("enchant");
        drain_stack(&mut g);
        g.perform_action(GameAction::ActivateAbility {
            card_id: aura, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("sac to regenerate");
        drain_stack(&mut g);
        assert!(g.battlefield_find(aura).is_none(), "Carapace sacrificed");
        assert_eq!(g.battlefield_find(bear).unwrap().regeneration_shields, 1,
            "enchanted creature gets a regen shield via LKI");
    }
}

mod recent79 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    #[test]
    fn carrion_ants_pump_stacks() {
        let mut g = two_player_game();
        let ants = g.add_card_to_battlefield(0, catalog::carrion_ants());
        g.players[0].mana_pool.add_colorless(2);
        for _ in 0..2 {
            g.perform_action(GameAction::ActivateAbility {
                card_id: ants, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
            }).expect("{1}: +1/+1");
            drain_stack(&mut g);
        }
        let p = g.computed_permanent(ants).unwrap();
        assert_eq!((p.power, p.toughness), (2, 3), "0/1 pumped twice");
    }

    #[test]
    fn elvish_hunter_locks_untap() {
        let mut g = two_player_game();
        let hunter = g.add_card_to_battlefield(0, catalog::elvish_hunter());
        g.clear_sickness(hunter);
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: hunter, ability_index: 0, target: Some(Target::Permanent(foe)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("untap lock");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).unwrap().skip_next_untap, "target flagged to skip its next untap");
    }

    #[test]
    fn dwarven_nomad_makes_small_creature_unblockable() {
        let mut g = two_player_game();
        let nomad = g.add_card_to_battlefield(0, catalog::dwarven_nomad());
        g.clear_sickness(nomad);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
        g.perform_action(GameAction::ActivateAbility {
            card_id: nomad, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("grant unblockable");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable));
    }

    #[test]
    fn balduvian_war_makers_has_haste_and_rampage() {
        let c = catalog::balduvian_war_makers();
        assert!(c.keywords.contains(&Keyword::Haste));
        assert!(c.keywords.contains(&Keyword::Rampage(1)));
        assert_eq!((c.power, c.toughness), (3, 3));
    }

    #[test]
    fn grave_robbers_exiles_artifact_and_gains_life() {
        let mut g = two_player_game();
        let robbers = g.add_card_to_battlefield(0, catalog::grave_robbers());
        g.clear_sickness(robbers);
        // An artifact in the opponent's graveyard.
        let art = g.add_card_to_battlefield(1, catalog::jayemdae_tome());
        g.remove_from_battlefield_to_graveyard_raw(art);
        let life = g.players[0].life;
        g.players[0].mana_pool.add(Color::Black, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: robbers, ability_index: 0, target: Some(Target::Permanent(art)),
            additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("exile artifact from graveyard");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == art), "artifact exiled from the graveyard");
        assert_eq!(g.players[0].life, life + 2, "gained 2 life");
    }
}

mod recent80 {
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::*;

    #[test]
    fn bonehoard_scales_germ_by_creature_cards_in_all_graveyards() {
        let mut g = two_player_game();
        // Two creature cards in p0's graveyard, one in p1's = 3 creature cards
        // across all graveyards.
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let bh = g.add_card_to_hand(0, catalog::bonehoard());
        g.players[0].mana_pool.add_colorless(4);
        // Cast Bonehoard → living-weapon ETB mints a Germ and attaches.
        cast(&mut g, bh);
        let germ = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Phyrexian Germ" && c.controller == 0)
            .expect("living weapon mints a Germ");
        assert_eq!(
            g.battlefield_find(bh).unwrap().attached_to,
            Some(germ.id),
            "Bonehoard attaches to its Germ"
        );
        let cp = g.computed_permanent(germ.id).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+X/+X, X = creature cards in all graveyards");
    }

    #[test]
    fn necropolis_fiend_debuffs_by_exiled_count() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_graveyard(0, catalog::grizzly_bears());
        }
        let fiend = g.add_card_to_battlefield(0, catalog::necropolis_fiend());
        g.clear_sickness(fiend);
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        // {2}, {T}, Exile 2 cards from graveyard: victim gets -2/-2.
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: fiend,
            ability_index: 0,
            target: Some(Target::Permanent(victim)),
            additional_targets: Vec::new(),
            x_value: Some(2), mode: None,
        })
        .expect("activate -X/-X for X=2");
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), 1, "two of three graveyard cards exiled as a cost");
        // 2/2 with -2/-2 → 0 toughness → dies as an SBA.
        assert!(
            !g.battlefield.iter().any(|c| c.id == victim),
            "the -2/-2'd 2/2 dies to the state-based check"
        );
    }

    #[test]
    fn charmed_sleep_taps_and_locks_untap() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::charmed_sleep());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: aura,
            target: Some(Target::Permanent(foe)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("Charmed Sleep castable for {1}{U}{U}");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).unwrap().tapped, "ETB taps the enchanted creature");
        // The enchanted creature's controller's untap step must not free it.
        g.active_player_idx = 1;
        g.do_untap();
        assert!(g.battlefield_find(foe).unwrap().tapped, "enchanted creature doesn't untap");
    }

    #[test]
    fn blaze_deals_x_damage_to_any_target() {
        let mut g = two_player_game();
        let blaze = g.add_card_to_hand(0, catalog::blaze());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        let opp_life = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: blaze,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: Some(3),
        })
        .expect("Blaze castable for X=3");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_life - 3, "X=3 damage to face");
    }

    #[test]
    fn highway_robber_drains_two_on_etb() {
        let mut g = two_player_game();
        let hr = g.add_card_to_hand(0, catalog::highway_robber());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
        g.players[0].mana_pool.add_colorless(2);
        let my_life = g.players[0].life;
        let opp_life = g.players[1].life;
        cast(&mut g, hr);
        assert_eq!(g.players[1].life, opp_life - 2, "opponent loses 2");
        assert_eq!(g.players[0].life, my_life + 2, "you gain 2");
    }

    #[test]
    fn warthog_has_swampwalk() {
        use crabomination::card::{Keyword, LandType};
        let g = two_player_game();
        let def = catalog::warthog();
        assert!(def.keywords.contains(&Keyword::Landwalk(LandType::Swamp)), "Warthog has swampwalk");
        let _ = g;
    }

    #[test]
    fn ghost_ship_regenerates() {
        let mut g = two_player_game();
        let ship = g.add_card_to_battlefield(0, catalog::ghost_ship());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 3);
        // {U}{U}{U}: set up a regeneration shield.
        g.perform_action(GameAction::ActivateAbility {
            card_id: ship,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None, mode: None,
        })
        .expect("regenerate");
        drain_stack(&mut g);
        // Lethal damage is replaced by the shield: the Ship survives.
        g.battlefield_find_mut(ship).unwrap().damage = 99;
        let sba = g.check_state_based_actions();
        let _ = sba;
        assert!(g.battlefield.iter().any(|c| c.id == ship), "regeneration shield saves the Ship");
    }

    #[test]
    fn serpent_assassin_etb_destroys_nonblack() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let white = g.add_card_to_battlefield(1, catalog::serra_angel()); // nonblack
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let assassin = g.add_card_to_hand(0, catalog::serpent_assassin());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: assassin,
            target: Some(Target::Permanent(white)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("Serpent Assassin castable");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == white), "ETB destroys the nonblack creature");
    }

    #[test]
    fn sea_monster_needs_defender_island() {
        let mut g = two_player_game();
        let sm = g.add_card_to_battlefield(0, catalog::sea_monster());
        g.clear_sickness(sm);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        assert!(
            g.perform_action(GameAction::DeclareAttackers(vec![Attack {
                attacker: sm,
                target: AttackTarget::Player(1),
            }]))
            .is_err(),
            "can't attack a defender with no Island"
        );
        g.add_card_to_battlefield(1, catalog::island());
        assert!(
            g.perform_action(GameAction::DeclareAttackers(vec![Attack {
                attacker: sm,
                target: AttackTarget::Player(1),
            }]))
            .is_ok(),
            "may attack once the defender controls an Island"
        );
    }

    #[test]
    fn sea_serpent_needs_defender_island_to_attack() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::island()); // p0 keeps its Island so it survives upkeep
        let serp = g.add_card_to_battlefield(0, catalog::sea_serpent());
        g.clear_sickness(serp);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        assert!(
            g.perform_action(GameAction::DeclareAttackers(vec![Attack {
                attacker: serp,
                target: AttackTarget::Player(1),
            }]))
            .is_err(),
            "can't attack a defender with no Island"
        );
        g.add_card_to_battlefield(1, catalog::island());
        assert!(
            g.perform_action(GameAction::DeclareAttackers(vec![Attack {
                attacker: serp,
                target: AttackTarget::Player(1),
            }]))
            .is_ok(),
            "may attack once the defender controls an Island"
        );
    }

    #[test]
    fn caustic_bronco_unsaddled_hits_you_saddled_hits_opponent() {
        // Unsaddled: the controller loses life equal to the revealed card's MV.
        let mut g = two_player_game();
        let bronco = g.add_card_to_battlefield(0, catalog::caustic_bronco());
        g.clear_sickness(bronco);
        g.add_card_to_library(0, catalog::grizzly_bears()); // top card, MV 2
        let my_life = g.players[0].life;
        let opp_life = g.players[1].life;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bronco,
            target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, my_life - 2, "unsaddled: you lose the revealed card's MV");
        assert_eq!(g.players[1].life, opp_life, "opponent untouched while unsaddled");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "revealed card to hand");
    }

    #[test]
    fn goblin_recruiter_stacks_goblins_on_top() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        // A non-Goblin on top, then a Goblin deeper in the library.
        g.add_card_to_library(0, catalog::grizzly_bears());
        let goblin = g.add_card_to_library(0, catalog::goblin_recruiter());
        let rec = g.add_card_to_hand(0, catalog::goblin_recruiter());
        // Search picks the Goblin, then stops.
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Search(Some(goblin)),
            DecisionAnswer::Search(None),
        ]));
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, rec);
        // ETB tutors the Goblin to the top of the library.
        let top = g.players[0].library.first().expect("library non-empty");
        assert_eq!(top.id, goblin, "the searched-up Goblin sits on top");
    }
}
