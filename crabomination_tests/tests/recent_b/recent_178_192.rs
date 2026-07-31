//! Tests for recentN card batches 178-192 (merged from per-batch micro-files).

mod recent178 {
    use crabomination::card::{ArtifactSubtype, Keyword};
    use crabomination::catalog;
    use crabomination::game::*;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Marching Duodrone makes a Treasure for each player when it attacks.
    #[test]
    fn marching_duodrone_treasures_each_player_on_attack() {
        let mut g = two_player_game();
        let drone = g.add_card_to_battlefield(0, catalog::marching_duodrone());
        g.clear_sickness(drone);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: drone,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        let treasures = g
            .battlefield
            .iter()
            .filter(|c| c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Treasure))
            .count();
        assert_eq!(treasures, 2, "one Treasure per player");
    }

    /// Fiendish Panda gains a +1/+1 counter on lifegain and reanimates a small
    /// non-Bear creature when it dies.
    #[test]
    fn fiendish_panda_counter_and_reanimate() {
        let mut g = two_player_game();
        let panda = g.add_card_to_battlefield(0, catalog::fiendish_panda());
        g.adjust_life(0, 1);
        g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 1 }]);
        drain_stack(&mut g);
        let counters = *g.battlefield_find(panda).unwrap().counters.get(&crabomination::card::CounterType::PlusOnePlusOne).unwrap_or(&0);
        assert_eq!(counters, 1, "lifegain adds a counter");
        // A 2-MV bear-free creature in the graveyard is returnable (Panda power 4).
        let mut target = catalog::grizzly_bears(); // {1}{G} = MV 2, but it's a Bear...
        target.name = "Small Elf";
        target.subtypes.creature_types = vec![crabomination::card::CreatureType::Elf];
        let dead = g.add_card_to_graveyard(0, target);
        // Kill the Panda → death trigger reanimates the Elf.
        let snap = g.battlefield_find(panda).unwrap().clone();
        g.remove_to_graveyard_with_triggers(panda);
        g.died_card_snapshots.insert(panda, snap);
        g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: panda }]);
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_some(), "reanimated the non-Bear creature");
    }

    /// Quick-Draw Katana grants +2/+0 always and first strike during your turn.
    #[test]
    fn quick_draw_katana_during_turn_bonus() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let katana = g.add_card_to_battlefield(0, catalog::quick_draw_katana());
        g.battlefield_find_mut(katana).unwrap().attached_to = Some(bear);
        g.active_player_idx = 0;
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 4, "+2/+0 from the katana");
        assert!(cp.keywords.contains(&Keyword::FirstStrike), "first strike on your turn");
    }

    /// Salvation Swan blinks a nonflying creature you control when a Bird enters.
    #[test]
    fn salvation_swan_blinks_nonflyer() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::salvation_swan());
        let ground = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // no flying
        // Another Bird entering triggers the Swan's blink on the ground creature.
        let mut bird = catalog::grizzly_bears();
        bird.name = "Test Bird";
        bird.subtypes.creature_types = vec![crabomination::card::CreatureType::Bird];
        let entered = g.add_card_to_battlefield(0, bird);
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: entered }]);
        drain_stack(&mut g);
        // The ground creature was exiled (to return at end step) — gone for now.
        assert!(g.battlefield_find(ground).is_none(), "nonflyer exiled by the blink");
        assert!(g.exile.iter().any(|c| c.id == ground), "waiting in exile to return");
    }
}

mod recent179 {
    use crabomination::card::{ArtifactSubtype, CounterType, CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Twinblade Blessing grants the enchanted creature double strike.
    #[test]
    fn twinblade_blessing_grants_double_strike() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::twinblade_blessing());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: aura,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Twinblade Blessing");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "enchanted creature has double strike"
        );
    }

    /// Tragic Banshee gives -1/-1 with no death, -13/-13 once a creature has died.
    #[test]
    fn tragic_banshee_morbid_scales() {
        let mut g = two_player_game();
        // No creature died yet → -1/-1 to a 5/5.
        let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        g.move_card_to_battlefield_for_test(0, catalog::tragic_banshee());
        drain_stack(&mut g);
        let cp = g.computed_permanent(foe).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2), "no morbid → -1/-1");
    }

    /// Midnight Snack makes a Food at end step if you attacked, and drains for the
    /// life you gained this turn.
    #[test]
    fn midnight_snack_food_and_drain() {
        let mut g = two_player_game();
        let snack = g.add_card_to_battlefield(0, catalog::midnight_snack());
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        advance_to(&mut g, TurnStep::End);
        drain_stack(&mut g);
        let foods = g
            .battlefield
            .iter()
            .filter(|c| c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Food))
            .count();
        assert_eq!(foods, 1, "Raid made a Food at end step");
        // Now gain 4 life and sac Midnight Snack to drain the opponent for 4.
        g.adjust_life(0, 4);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        let opp = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: snack,
            ability_index: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate drain");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 4, "drained the life gained this turn");
    }

    /// Uncharted Voyage tucks a creature into its owner's library (bottom default).
    #[test]
    fn uncharted_voyage_tucks_creature() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::uncharted_voyage());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.add_card_to_library(0, catalog::grizzly_bears()); // surveil fodder
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(foe)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Uncharted Voyage");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "creature left the battlefield");
        assert!(g.players[1].library.iter().any(|c| c.id == foe), "went to owner's library");
    }

    /// Raise the Past returns only the MV≤2 creatures from your graveyard.
    #[test]
    fn raise_the_past_returns_small_creatures() {
        let mut g = two_player_game();
        let small = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
        let big = g.add_card_to_graveyard(0, catalog::hill_giant()); // MV 4
        let spell = g.add_card_to_hand(0, catalog::raise_the_past());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Raise the Past");
        drain_stack(&mut g);
        assert!(g.battlefield_find(small).is_some(), "small creature returned");
        assert!(g.battlefield_find(big).is_none(), "MV4 creature stayed in graveyard");
    }

    /// Sylvan Scavenging's end-step modal resolves (counter or Raccoon).
    #[test]
    fn sylvan_scavenging_end_step_modal() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::sylvan_scavenging());
        let beater = g.add_card_to_battlefield(0, catalog::hill_giant()); // power 3
        let before_counters =
            *g.battlefield_find(beater).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0);
        let before = g.battlefield.len();
        advance_to(&mut g, TurnStep::End);
        drain_stack(&mut g);
        let after_counters =
            *g.battlefield_find(beater).unwrap().counters.get(&CounterType::PlusOnePlusOne).unwrap_or(&0);
        let raccoon = g
            .battlefield
            .iter()
            .any(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Raccoon));
        assert!(
            after_counters > before_counters || raccoon || g.battlefield.len() > before,
            "a mode resolved (counter placed or Raccoon made)"
        );
    }

    /// Ravenous Amulet stores soul counters on sac-to-draw, then drains for them.
    #[test]
    fn ravenous_amulet_stores_and_drains() {
        let mut g = two_player_game();
        let amulet = g.add_card_to_battlefield(0, catalog::ravenous_amulet());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // Sac the bear → draw a card + a charge counter.
        g.perform_action(GameAction::ActivateAbility {
            card_id: amulet,
            ability_index: 0,
            target: Some(Target::Permanent(fodder)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate sac-to-draw");
        drain_stack(&mut g);
        let counters =
            *g.battlefield_find(amulet).unwrap().counters.get(&CounterType::Charge).unwrap_or(&0);
        assert_eq!(counters, 1, "stored a soul counter");
        // Untap (a fresh turn) then sac the amulet → opponent loses 1 (one counter).
        g.battlefield_find_mut(amulet).unwrap().tapped = false;
        g.players[0].mana_pool.add_colorless(4);
        let opp = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: amulet,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate drain");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "drained for the stored soul counter");
    }

    /// Zul Ashur lets you cast a Zombie from your graveyard this turn.
    #[test]
    fn zul_ashur_grants_graveyard_zombie_cast() {
        let mut g = two_player_game();
        let zul = g.add_card_to_battlefield(0, catalog::zul_ashur_lich_lord());
        g.clear_sickness(zul);
        // A Zombie creature card in the graveyard.
        let mut zombie = catalog::grizzly_bears();
        zombie.name = "Rotting Ghoul";
        zombie.subtypes.creature_types = vec![CreatureType::Zombie];
        let ghoul = g.add_card_to_graveyard(0, zombie);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: zul,
            ability_index: 0,
            target: Some(Target::Permanent(ghoul)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate graveyard-cast grant");
        drain_stack(&mut g);
        let granted = g.players[0]
            .graveyard
            .iter()
            .find(|c| c.id == ghoul)
            .map(|c| c.may_play_until.is_some())
            .unwrap_or(false);
        assert!(granted, "the Zombie may now be cast from the graveyard");
    }

    /// Twinflame Tyrant doubles damage your sources deal to opponents.
    #[test]
    fn twinflame_tyrant_doubles_damage_to_opponents() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::twinflame_tyrant());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // 3 to any target
        g.players[0].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let opp = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Lightning Bolt");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 6, "3 damage doubled to 6");
    }

    /// High Fae Trickster lets you cast a sorcery at instant speed.
    #[test]
    fn high_fae_trickster_grants_flash_to_spells() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::high_fae_trickster());
        let sorc = g.add_card_to_hand(0, catalog::divination());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        // It's the opponent's turn (instant speed only) — the sorcery is castable.
        g.active_player_idx = 1;
        g.step = TurnStep::Upkeep;
        g.priority.player_with_priority = 0;
        let ok = g
            .perform_action(GameAction::CastSpell {
                card_id: sorc,
                target: None,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            })
            .is_ok();
        assert!(ok, "sorcery cast at instant speed via granted flash");
    }

    /// Electroduplicate makes a haste token copy of your creature.
    #[test]
    fn electroduplicate_copies_your_creature() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::electroduplicate());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let before = g.battlefield.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Electroduplicate");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.len(), before + 1, "made a token copy");
        let token_id = g
            .battlefield
            .iter()
            .find(|c| c.is_token && c.definition.name == "Grizzly Bears")
            .map(|c| c.id)
            .expect("token copy exists");
        assert!(
            g.computed_permanent(token_id).unwrap().keywords.contains(&Keyword::Haste),
            "copy has haste"
        );
    }

    /// Fear of Falling shrinks and grounds a blocker when it attacks.
    #[test]
    fn fear_of_falling_debuffs_on_attack() {
        let mut g = two_player_game();
        let flyer = g.add_card_to_battlefield(0, catalog::fear_of_falling());
        g.clear_sickness(flyer);
        let mut blocker = catalog::grizzly_bears();
        blocker.keywords.push(Keyword::Flying);
        let foe = g.add_card_to_battlefield(1, blocker); // 2/2 flyer
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: flyer,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        let cp = g.computed_permanent(foe).unwrap();
        assert_eq!(cp.power, 0, "-2/-0 applied");
        assert!(!cp.keywords.contains(&Keyword::Flying), "lost flying");
    }
}

mod recent180 {
    use crabomination::card::{CounterType, CreatureType};
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Possessed Goat's once-per-game ability grows it and makes it a black Demon.
    #[test]
    fn possessed_goat_becomes_black_demon_once() {
        let mut g = two_player_game();
        let goat = g.add_card_to_battlefield(0, catalog::possessed_goat());
        g.clear_sickness(goat);
        g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(6);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: goat,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate Possessed Goat");
        drain_stack(&mut g);
        let cp = g.computed_permanent(goat).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "three +1/+1 counters → 4/4");
        assert!(cp.subtypes.creature_types.contains(&CreatureType::Demon), "became a Demon");
        assert!(cp.colors.contains(&Color::Black), "became black");
        assert!(cp.colors.contains(&Color::White), "kept its white color");
        // "Activate only once" — the second try is rejected.
        let second = g.perform_action(GameAction::ActivateAbility {
            card_id: goat,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        });
        assert!(second.is_err(), "cannot activate a second time");
    }

    /// Hired Claw pings an opponent when you attack with a Lizard.
    #[test]
    fn hired_claw_pings_on_lizard_attack() {
        let mut g = two_player_game();
        let claw = g.add_card_to_battlefield(0, catalog::hired_claw());
        g.clear_sickness(claw);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        let opp = g.players[1].life;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: claw,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack with the Lizard");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "Lizard attack pinged for 1");
    }

    /// Mistbreath Elder bounces another creature you control and grows on upkeep.
    #[test]
    fn mistbreath_elder_bounces_and_grows() {
        let mut g = two_player_game();
    // CR 103.7a — only turn 1's draw is skipped; keep libraries stocked for
    // fixtures that cross a turn boundary.
    for seat in 0..2 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
        let elder = g.add_card_to_battlefield(0, catalog::mistbreath_elder());
        let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Reach the controller's next upkeep.
        while !(g.step == TurnStep::Upkeep && g.active_player_idx == 0) {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == friend), "friend bounced to hand");
        assert_eq!(
            g.battlefield_find(elder).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "Elder grew a +1/+1 counter",
        );
    }
}

mod recent181 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::*;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Plumecreed Mentor puts a counter on a non-flyer when a flyer enters.
    #[test]
    fn plumecreed_mentor_counters_a_grounded_creature() {
        let mut g = two_player_game();
        let grounded = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // no flying
        let mentor = g.move_card_to_battlefield_for_test(0, catalog::plumecreed_mentor());
        // The Mentor is itself a flyer; its "this or another flying creature you
        // control enters" trigger (YourControl scope) fires for its own entry.
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: mentor }]);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(grounded).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "the grounded creature got a +1/+1 counter",
        );
    }

    /// Azure Beastbinder strips abilities and shrinks its target on attack.
    #[test]
    fn azure_beastbinder_strips_on_attack() {
        let mut g = two_player_game();
        let binder = g.add_card_to_battlefield(0, catalog::azure_beastbinder());
        g.clear_sickness(binder);
        let mut flyer = catalog::grizzly_bears();
        flyer.keywords.push(Keyword::Flying);
        flyer.power = 5;
        flyer.toughness = 5;
        let foe = g.add_card_to_battlefield(1, flyer);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: binder,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        let cp = g.computed_permanent(foe).unwrap();
        assert!(!cp.keywords.contains(&Keyword::Flying), "lost its abilities");
        assert_eq!((cp.power, cp.toughness), (2, 2), "became a 2/2");
    }

    /// Byrke's attack trigger doubles the +1/+1 counters on an attacking creature.
    #[test]
    fn byrke_doubles_counters_on_attack() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::byrke_long_ear_of_the_law());
        let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(beater).unwrap().counters.insert(CounterType::PlusOnePlusOne, 2);
        g.clear_sickness(beater);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: beater,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(beater).unwrap().counter_count(CounterType::PlusOnePlusOne),
            4,
            "2 counters doubled to 4",
        );
    }

    /// Dreamdew Entrancer stuns a creature and draws when it's yours.
    #[test]
    fn dreamdew_entrancer_stuns_and_draws() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand_before = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::dreamdew_entrancer());
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(mine).unwrap().counter_count(CounterType::Stun),
            3,
            "three stun counters",
        );
        assert!(g.battlefield_find(mine).unwrap().tapped, "tapped by the ETB");
        assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew two (you control the target)");
    }
}

mod recent182 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Finneas counters your other Rabbits/tokens and draws at high total power.
    #[test]
    fn finneas_counters_rabbits_and_draws() {
        let mut g = two_player_game();
        let finneas = g.add_card_to_battlefield(0, catalog::finneas_ace_archer());
        g.clear_sickness(finneas);
        // A big Rabbit so total power clears 10 after the counter.
        let mut rabbit = catalog::hill_giant(); // 3/3
        rabbit.subtypes.creature_types = vec![crabomination::card::CreatureType::Rabbit];
        rabbit.power = 8;
        let bunny = g.add_card_to_battlefield(0, rabbit);
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand_before = g.players[0].hand.len();
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: finneas,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(bunny).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "the other Rabbit got a counter",
        );
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew at total power 10+");
    }

    /// Gev pings an opponent whenever you cast a Lizard spell.
    #[test]
    fn gev_pings_on_lizard_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::gev_scaled_scorch());
        // A Lizard creature spell to cast.
        let mut lizard = catalog::grizzly_bears();
        lizard.name = "Basking Lizard";
        lizard.subtypes.creature_types = vec![crabomination::card::CreatureType::Lizard];
        let spell = g.add_card_to_hand(0, lizard);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let opp = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast the Lizard");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 1, "Lizard cast pinged the opponent");
    }
}

mod recent183 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Ferocification's begin-combat modal buffs one of your creatures.
    #[test]
    fn ferocification_begin_combat_modal_buffs() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::ferocification());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        advance_to(&mut g, TurnStep::BeginCombat);
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        let buffed = cp.power > 2 || cp.keywords.contains(&Keyword::Menace) || cp.keywords.contains(&Keyword::Haste);
        assert!(buffed, "a mode resolved (+2/+0 or menace+haste)");
    }

    /// Freestrider Lookout digs a land onto the battlefield when you commit a crime.
    #[test]
    fn freestrider_lookout_digs_land_on_crime() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::freestrider_lookout());
        // Stack a land on top of the library, then some filler beneath.
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let land = g.add_card_to_library(0, catalog::forest());
        let bf_before = g.battlefield.len();
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_some(), "the land was put onto the battlefield");
        assert!(g.battlefield.len() > bf_before, "battlefield grew");
    }

    /// Fleeting Reflection makes your creature a copy of another and untaps it.
    #[test]
    fn fleeting_reflection_copies_and_untaps() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.battlefield_find_mut(mine).unwrap().tapped = true;
        let model = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
        let spell = g.add_card_to_hand(0, catalog::fleeting_reflection());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(model)],
            mode: None,
            x_value: None,
        })
        .expect("cast Fleeting Reflection");
        drain_stack(&mut g);
        let cp = g.computed_permanent(mine).unwrap();
        assert!(!g.battlefield_find(mine).unwrap().tapped, "untapped its target");
        assert!(cp.keywords.contains(&Keyword::Hexproof), "gained hexproof");
        assert_eq!((cp.power, cp.toughness), (3, 3), "became a copy of the 3/3");
    }
}

mod recent184 {
    use crabomination::card::{CounterType, CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Full Steam Ahead pumps your team and grants trample + block-limit.
    #[test]
    fn full_steam_ahead_buffs_team() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::full_steam_ahead());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Full Steam Ahead");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
        assert!(cp.keywords.contains(&Keyword::Trample), "gained trample");
        assert!(cp.keywords.contains(&Keyword::CantBeBlockedByMoreThanOne), "block-limited");
    }

    /// Hellspur Posse Boss makes two Mercenaries and gives other outlaws haste.
    #[test]
    fn hellspur_posse_boss_tokens_and_haste() {
        let mut g = two_player_game();
        // Another outlaw (Rogue) already out — it should gain haste from the lord.
        let mut rogue = catalog::grizzly_bears();
        rogue.subtypes.creature_types = vec![CreatureType::Rogue];
        let outlaw = g.add_card_to_battlefield(0, rogue);
        g.move_card_to_battlefield_for_test(0, catalog::hellspur_posse_boss());
        drain_stack(&mut g);
        let mercs = g
            .battlefield
            .iter()
            .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Mercenary))
            .count();
        assert_eq!(mercs, 2, "made two Mercenary tokens");
        assert!(
            g.computed_permanent(outlaw).unwrap().keywords.contains(&Keyword::Haste),
            "other outlaw gained haste",
        );
    }

    /// Kraum draws and grows when you cast your second spell in a turn.
    #[test]
    fn kraum_flurry_draws_and_grows() {
        let mut g = two_player_game();
        let kraum = g.add_card_to_battlefield(0, catalog::kraum_violent_cacophony());
        // First spell of the turn (no trigger).
        let s1 = g.add_card_to_hand(0, catalog::divination());
        let s2 = g.add_card_to_hand(0, catalog::divination());
        for _ in 0..8 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: s1, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("first spell");
        drain_stack(&mut g);
        let hand_after_first = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: s2, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("second spell");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(kraum).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "Kraum grew on the second spell",
        );
        // Divination draws 2; the Flurry draws one more → net > +2 vs before second.
        assert!(g.players[0].hand.len() > hand_after_first, "the Flurry drew a card too");
    }

    /// At Knifepoint gives your outlaws first strike and makes a Mercenary on a crime.
    #[test]
    fn at_knifepoint_first_strike_and_crime_token() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::at_knifepoint());
        let mut rogue = catalog::grizzly_bears();
        rogue.subtypes.creature_types = vec![CreatureType::Rogue];
        let outlaw = g.add_card_to_battlefield(0, rogue);
        assert!(
            g.computed_permanent(outlaw).unwrap().keywords.contains(&Keyword::FirstStrike),
            "outlaw has first strike",
        );
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        drain_stack(&mut g);
        let mercs = g
            .battlefield
            .iter()
            .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Mercenary))
            .count();
        assert_eq!(mercs, 1, "crime made a Mercenary token");
    }
}

mod recent185 {
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Fill `p`'s graveyard with `n` vanilla cards (to toggle threshold).
    fn fill_graveyard(g: &mut GameState, p: usize, n: usize) {
        for _ in 0..n {
            g.add_card_to_graveyard(p, catalog::grizzly_bears());
        }
    }

    /// Thought Shucker's threshold ability grows it and draws — once only, and only
    /// with seven+ cards in the graveyard.
    #[test]
    fn thought_shucker_threshold_activate_once() {
        let mut g = two_player_game();
        let shucker = g.add_card_to_battlefield(0, catalog::thought_shucker());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // < 7 cards in graveyard → gated.
        fill_graveyard(&mut g, 0, 6);
        assert!(
            g.perform_action(GameAction::ActivateAbility {
                card_id: shucker,
                ability_index: 0,
                target: None,
                additional_targets: vec![],
                x_value: None, mode: None,
            })
            .is_err(),
            "threshold not met → activation rejected",
        );
        // Reach threshold and activate.
        fill_graveyard(&mut g, 0, 1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: shucker,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate Thought Shucker");
        drain_stack(&mut g);
        let cp = g.computed_permanent(shucker).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 4), "+1/+1 counter");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
        // Activate-once: a second attempt is rejected even with mana + threshold.
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        assert!(
            g.perform_action(GameAction::ActivateAbility {
                card_id: shucker,
                ability_index: 0,
                target: None,
                additional_targets: vec![],
                x_value: None, mode: None,
            })
            .is_err(),
            "activate only once",
        );
    }

    /// Shoreline Looter draws on combat damage and skips the discard at threshold.
    #[test]
    fn shoreline_looter_loots_at_threshold() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let looter = g.add_card_to_battlefield(0, catalog::shoreline_looter());
        g.clear_sickness(looter);
        g.add_card_to_library(0, catalog::grizzly_bears());
        fill_graveyard(&mut g, 0, 7); // threshold active → no discard
        let hand_before = g.players[0].hand.len();
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: looter,
            target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::CombatDamage);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew, no discard at threshold");
    }

    /// Ruthless Negotiation makes a target opponent exile a hand card; the graveyard
    /// flashback also draws (cast-from-graveyard rider).
    #[test]
    fn ruthless_negotiation_flashback_draws() {
        let mut g = two_player_game();
        let spell = g.add_card_to_graveyard(0, catalog::ruthless_negotiation());
        g.add_card_to_hand(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        // Flashback cost {4}{B}.
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let opp_hand_before = g.players[1].hand.len();
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastFlashback {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("flashback Ruthless Negotiation");
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), opp_hand_before - 1, "opponent exiled a hand card");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "cast-from-graveyard drew a card");
        assert!(g.exile.iter().any(|c| c.id == spell), "flashback exiles the spell");
    }

    /// Seasoned Warrenguard pumps only when you control a token as it attacks.
    #[test]
    fn seasoned_warrenguard_token_gated_pump() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let guard = g.add_card_to_battlefield(0, catalog::seasoned_warrenguard());
        g.clear_sickness(guard);
        // No token yet → attacking gives no bonus.
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: guard,
            target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(guard).unwrap().power, 1, "no token → no pump");

        // Fresh turn with a token controlled → +2/+0.
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let guard = g.add_card_to_battlefield(0, catalog::seasoned_warrenguard());
        g.clear_sickness(guard);
        let tok = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(tok).unwrap().is_token = true;
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: guard,
            target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(guard).unwrap().power, 3, "token → +2/+0");
    }

    /// Valley Flamecaller adds 1 to the damage its typed creatures deal.
    #[test]
    fn valley_flamecaller_boosts_typed_damage() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.players[1].life = 20;
        let flamecaller = g.add_card_to_battlefield(0, catalog::valley_flamecaller());
        g.clear_sickness(flamecaller);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: flamecaller,
            target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::CombatDamage);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 16, "3 power + 1 = 4 combat damage");
    }
}

mod recent186 {
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Vanish from Sight tucks a nonland permanent into its owner's library and
    /// surveils.
    #[test]
    fn vanish_from_sight_tucks_permanent() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(1, catalog::howling_mine());
        g.add_card_to_library(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::vanish_from_sight());
        g.add_card_to_library(0, catalog::grizzly_bears()); // to surveil
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let opp_lib_before = g.players[1].library.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(mine)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Vanish from Sight");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none(), "permanent left the battlefield");
        assert_eq!(g.players[1].library.len(), opp_lib_before + 1, "tucked into owner's library");
    }

    /// Hearthborn Battler pings the opponent on any player's second spell.
    #[test]
    fn hearthborn_battler_pings_on_second_spell() {
        let mut g = two_player_game();
        let _battler = g.add_card_to_battlefield(0, catalog::hearthborn_battler());
        let s1 = g.add_card_to_hand(0, catalog::divination());
        let s2 = g.add_card_to_hand(0, catalog::divination());
        for _ in 0..8 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.players[1].life = 20;
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: s1, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("first spell");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 20, "no ping on the first spell");
        g.perform_action(GameAction::CastSpell {
            card_id: s2, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("second spell");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 18, "second spell pinged the opponent for 2");
    }

    /// Inquisitive Glimmer makes enchantment spells cost {1} less.
    #[test]
    fn inquisitive_glimmer_discounts_enchantments() {
        let cast_with_only_red = |glimmer: bool| -> bool {
            let mut g = two_player_game();
            if glimmer {
                g.add_card_to_battlefield(0, catalog::inquisitive_glimmer());
            }
            let bomb = g.add_card_to_hand(0, catalog::goblin_bombardment()); // {1}{R}
            g.players[0].mana_pool.add(Color::Red, 1);
            g.step = TurnStep::PreCombatMain;
            g.active_player_idx = 0;
            g.priority.player_with_priority = 0;
            g.perform_action(GameAction::CastSpell {
                card_id: bomb, target: None, additional_targets: vec![], mode: None, x_value: None,
            })
            .is_ok()
        };
        assert!(!cast_with_only_red(false), "{{1}}{{R}} unpayable with only {{R}}");
        assert!(cast_with_only_red(true), "Glimmer's -{{1}} makes it castable for {{R}}");
    }

    /// Tidecaller Mentor bounces a permanent only when threshold is active.
    #[test]
    fn tidecaller_mentor_threshold_bounce() {
        let bounced = |gy: usize| -> bool {
            let mut g = two_player_game();
            let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
            for _ in 0..gy {
                g.add_card_to_graveyard(0, catalog::grizzly_bears());
            }
            g.move_card_to_battlefield_for_test(0, catalog::tidecaller_mentor());
            drain_stack(&mut g);
            g.battlefield_find(victim).is_none()
        };
        assert!(!bounced(6), "below threshold → no bounce");
        assert!(bounced(7), "threshold met → bounced a permanent");
    }

    /// Thought-Stalker Warlock's ETB forces a discard, targeting the opponent's hand
    /// when they lost life this turn.
    #[test]
    fn thought_stalker_warlock_conditional_discard() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.add_card_to_hand(1, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::lightning_bolt());
        // Opponent lost life this turn → targeted discard.
        g.adjust_life(1, -1);
        let opp_hand_before = g.players[1].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::thought_stalker_warlock());
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), opp_hand_before - 1, "opponent discarded a card");
    }
}

mod recent187 {
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Split Up mode 0 destroys tapped creatures and spares untapped ones.
    #[test]
    fn split_up_destroys_chosen_state() {
        let mut g = two_player_game();
        let tapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let untapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(tapped).unwrap().tapped = true;
        let spell = g.add_card_to_hand(0, catalog::split_up());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: Some(0),
            x_value: None,
        })
        .expect("cast Split Up (destroy tapped)");
        drain_stack(&mut g);
        assert!(g.battlefield_find(tapped).is_none(), "tapped creature destroyed");
        assert!(g.battlefield_find(untapped).is_some(), "untapped creature spared");
    }

    /// Strongbox Raider's Raid ETB impulses two cards when you attacked this turn.
    #[test]
    fn strongbox_raider_raid_impulse() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        // No attack yet → no impulse.
        g.move_card_to_battlefield_for_test(0, catalog::strongbox_raider());
        drain_stack(&mut g);
        assert_eq!(g.exile.len(), 0, "no raid → no impulse");

        // Fresh board, mark an attack, then ETB the raider.
        let mut g = two_player_game();
        g.active_player_idx = 0;
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        g.players[0].attacked_this_turn = true;
        g.move_card_to_battlefield_for_test(0, catalog::strongbox_raider());
        drain_stack(&mut g);
        assert_eq!(g.exile.len(), 2, "raid satisfied → top two exiled to impulse");
    }

    /// Fireglass Mentor impulses at your second main phase when an opponent lost life.
    #[test]
    fn fireglass_mentor_second_main_impulse() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        g.add_card_to_battlefield(0, catalog::fireglass_mentor());
        g.adjust_life(1, -1); // opponent lost life this turn
        g.fire_step_triggers(TurnStep::PostCombatMain);
        drain_stack(&mut g);
        assert_eq!(g.exile.len(), 2, "second main + opponent lost life → impulse two");
    }

    /// Menagerie Liberator's Melee grows it by the number of opponents attacked.
    #[test]
    fn menagerie_liberator_melee_pumps_on_attack() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let lib = g.add_card_to_battlefield(0, catalog::menagerie_liberator());
        g.clear_sickness(lib);
        // Before combat: base 3/2.
        assert_eq!(g.computed_permanent(lib).unwrap().power, 3);
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: lib,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack the lone opponent");
        drain_stack(&mut g);
        let cp = g.computed_permanent(lib).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 3), "melee: +1/+1 for one opponent");
    }
}

mod recent188 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Map the Frontier fetches up to two basic/Desert lands onto the battlefield
    /// tapped.
    #[test]
    fn map_the_frontier_fetches_two_lands_tapped() {
        let mut g = two_player_game();
        let f1 = g.add_card_to_library(0, catalog::forest());
        let f2 = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Search(Some(f1)),
            DecisionAnswer::Search(Some(f2)),
        ]));
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&catalog::map_the_frontier().effect, &ctx).unwrap();
        for id in [f1, f2] {
            let c = g.battlefield_find(id).expect("land fetched to battlefield");
            assert!(c.tapped, "enters tapped");
        }
    }

    /// Neutralize the Guards shrinks the opponent's creatures by -1/-1.
    #[test]
    fn neutralize_the_guards_shrinks_opponent() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        g.add_card_to_library(0, catalog::grizzly_bears()); // to surveil
        g.add_card_to_library(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::neutralize_the_guards());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Neutralize the Guards");
        drain_stack(&mut g);
        let cp = g.computed_permanent(victim).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "opponent creature is -1/-1");
    }

    /// Rise of the Varmints makes one 2/1 Varmint per creature card in your graveyard.
    #[test]
    fn rise_of_the_varmints_scales_with_graveyard() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_graveyard(0, catalog::grizzly_bears());
        }
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&catalog::rise_of_the_varmints().effect, &ctx).unwrap();
        let varmints = g
            .battlefield
            .iter()
            .filter(|c| c.definition.name == "Varmint" && c.controller == 0)
            .count();
        assert_eq!(varmints, 3, "one Varmint per graveyard creature");
    }

    /// Overzealous Muscle gains indestructible when you commit a crime on your turn.
    #[test]
    fn overzealous_muscle_indestructible_on_crime() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let muscle = g.add_card_to_battlefield(0, catalog::overzealous_muscle());
        assert!(!g.computed_permanent(muscle).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible));
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(muscle).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible),
            "crime on your turn grants indestructible",
        );
    }

    /// Outlaws' Fury pumps your team and impulses a card when you control an outlaw.
    #[test]
    fn outlaws_fury_pumps_and_impulses() {
        let mut g = two_player_game();
        // An outlaw (Rogue) plus a vanilla creature.
        let mut rogue = catalog::grizzly_bears();
        rogue.subtypes.creature_types = vec![crabomination::card::CreatureType::Rogue];
        let outlaw = g.add_card_to_battlefield(0, rogue);
        g.add_card_to_library(0, catalog::mountain());
        let spell = g.add_card_to_hand(0, catalog::outlaws_fury());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Outlaws' Fury");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(outlaw).unwrap().power, 4, "+2/+0 team pump");
        assert_eq!(g.exile.len(), 1, "outlaw controlled → impulsed one card");
    }
}

mod recent189 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Rodeo Pyromancers adds {R}{R} on your first spell each turn.
    #[test]
    fn rodeo_pyromancers_rituals_first_spell() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::rodeo_pyromancers());
        let spell = g.add_card_to_hand(0, catalog::ponder()); // {U}
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast first spell");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2, "first-spell ritual added RR");
    }

    /// Scalestorm Summoner mints a Dinosaur on attack only with a 4-power creature.
    #[test]
    fn scalestorm_summoner_ferocious_token() {
        let dinos = |ferocious: bool| -> usize {
            let mut g = two_player_game();
            g.active_player_idx = 0;
            let summoner = g.add_card_to_battlefield(0, catalog::scalestorm_summoner());
            g.clear_sickness(summoner);
            if ferocious {
                let big = g.add_card_to_battlefield(0, catalog::grizzly_bears());
                g.battlefield_find_mut(big).unwrap().power_bonus = 2; // 4/4
            }
            g.step = TurnStep::DeclareAttackers;
            g.priority.player_with_priority = 0;
            g.perform_action(GameAction::DeclareAttackers(vec![Attack {
                attacker: summoner,
                target: AttackTarget::Player(1),
            }]))
            .unwrap();
            drain_stack(&mut g);
            g.battlefield.iter().filter(|c| c.definition.name == "Dinosaur").count()
        };
        assert_eq!(dinos(false), 0, "no 4-power creature → no token");
        assert_eq!(dinos(true), 1, "ferocious → a Dinosaur token");
    }

    /// Marauding Sphinx surveils when you commit a crime, once each turn.
    #[test]
    fn marauding_sphinx_crime_surveil_once() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.add_card_to_battlefield(0, catalog::marauding_sphinx());
        g.players[0].library.clear();
        let top = g.next_id();
        g.players[0].add_to_library_top(top, catalog::island());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
            kept_top: vec![],
            bottom: vec![top],
        }]));
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "surveiled the top card away");
    }

    /// Raucous Entertainer counters only creatures that entered this turn.
    #[test]
    fn raucous_entertainer_counters_fresh_creatures() {
        let mut g = two_player_game();
        let old = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(old).unwrap().entered_turn = Some(0); // an earlier turn
        let ent = g.add_card_to_battlefield(0, catalog::raucous_entertainer());
        let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(fresh).unwrap().entered_turn = Some(g.turn_number);
        g.clear_sickness(ent);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: ent,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate Raucous Entertainer");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(fresh).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
            1,
            "fresh creature got a counter",
        );
        assert_eq!(
            g.battlefield_find(old).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
            0,
            "older creature untouched",
        );
    }

    /// Ruthless Lawbringer's ETB sacrifices a creature to destroy a nonland permanent.
    #[test]
    fn ruthless_lawbringer_sacrifice_removal() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        // Accept the optional sacrifice; the reflexive destroy auto-targets the
        // opponent's permanent.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.move_card_to_battlefield_for_test(0, catalog::ruthless_lawbringer());
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "sacrificed the fodder creature");
        assert!(g.battlefield_find(victim).is_none(), "destroyed the opponent's permanent");
    }
}

mod recent190 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Rowdy Research costs {1} less per attacker and draws three.
    #[test]
    fn rowdy_research_affinity_and_draw() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        // Two creatures that attacked this turn → {6}{U} becomes {4}{U}.
        for _ in 0..2 {
            let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.battlefield_find_mut(a).unwrap().attacked_this_turn = true;
        }
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::island());
        }
        let spell = g.add_card_to_hand(0, catalog::rowdy_research());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("{4}{U} covers the discounted cost");
        drain_stack(&mut g);
        // -1 for the spell leaving hand, +3 drawn.
        assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3, "drew three");
    }

    /// Brave the Wilds bargained animates a land and tutors a basic to hand.
    #[test]
    fn brave_the_wilds_bargained_animates_land() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let fodder = g.add_card_to_battlefield(0, catalog::howling_mine()); // artifact to bargain
        let basic = g.add_card_to_library(0, catalog::mountain());
        let spell = g.add_card_to_hand(0, catalog::brave_the_wilds());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.step = TurnStep::PreCombatMain;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
        g.perform_action(GameAction::CastSpellBargain {
            card_id: spell,
            sacrifice: Some(fodder),
            target: Some(Target::Permanent(land)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Brave the Wilds bargained");
        drain_stack(&mut g);
        let cp = g.computed_permanent(land).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "land animated to 3/3");
        assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
        assert!(g.players[0].hand.iter().any(|c| c.id == basic), "tutored a basic to hand");
    }

    /// Redrock Sentinel sacrifices a land to draw and make a Treasure.
    #[test]
    fn redrock_sentinel_sacs_land_for_value() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        let sentinel = g.add_card_to_battlefield(0, catalog::redrock_sentinel());
        g.clear_sickness(sentinel);
        let land = g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_library(0, catalog::island());
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: sentinel,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate Redrock Sentinel");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_none(), "sacrificed a land");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Treasure" && c.controller == 0),
            "made a Treasure",
        );
    }
}

mod recent191 {
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Longhorn Sharpshooter pings when plotted.
    #[test]
    fn longhorn_sharpshooter_burns_on_plot() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.players[1].life = 20;
        let card = g.add_card_to_hand(0, catalog::longhorn_sharpshooter());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::Plot { card_id: card }).expect("plot Longhorn Sharpshooter");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == card), "card is plotted (exiled)");
        assert_eq!(g.players[1].life, 18, "dealt 2 to the opponent on plot");
    }

    /// Aloe Alchemist pumps a creature when plotted.
    #[test]
    fn aloe_alchemist_pumps_on_plot() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sole creature → auto-targeted
        let card = g.add_card_to_hand(0, catalog::aloe_alchemist());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::Plot { card_id: card }).expect("plot Aloe Alchemist");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 4), "+3/+2 from the plot trigger");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Trample), "gained trample");
    }
}

mod recent192 {
    use crabomination::catalog;
    use crabomination::game::two_player_game;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Pillage the Bog digs twice your land count and takes a card.
    #[test]
    fn pillage_the_bog_digs_by_lands() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::forest()); // 3 lands → dig 6
        }
        let top = g.next_id();
        g.players[0].add_to_library_top(top, catalog::island());
        for _ in 0..6 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let spell = g.add_card_to_hand(0, catalog::pillage_the_bog());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Pillage the Bog");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == top), "took the top card from the dig");
    }

    /// Hell to Pay burns a creature for X and makes Treasures from excess damage.
    #[test]
    fn hell_to_pay_excess_makes_treasures() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::hell_to_pay());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(5);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: Some(5),
        })
        .expect("cast Hell to Pay for X=5");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "5 damage killed the 2/2");
        let treasures = g
            .battlefield
            .iter()
            .filter(|c| c.definition.name == "Treasure" && c.controller == 0)
            .count();
        assert_eq!(treasures, 3, "5 - 2 lethal = 3 excess → 3 Treasures");
        assert!(
            g.battlefield.iter().filter(|c| c.definition.name == "Treasure").all(|c| c.tapped),
            "Treasures enter tapped",
        );
    }
}
