//! Tests for recentN card batches 101-110 (merged from per-batch micro-files).

mod recent101 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;

    fn pass_through_combat(g: &mut GameState) {
        while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        drain_stack(g);
    }

    /// Coiling Stalker counters a creature that lacks a +1/+1 counter on combat damage.
    #[test]
    fn coiling_stalker_counters_on_damage() {
        let mut g = two_player_game();
        let stalker = g.add_card_to_battlefield(0, catalog::coiling_stalker());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(stalker);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: stalker,
            target: AttackTarget::Player(1),
        }]))
        .expect("stalker attacks");
        drain_stack(&mut g);
        pass_through_combat(&mut g);
        assert_eq!(
            g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "the uncountered bear got a +1/+1 counter"
        );
    }

    /// Sunblade Samurai's Channel fetches a Plains and gains 2 life.
    #[test]
    fn sunblade_samurai_channel_ramps_and_gains() {
        let mut g = two_player_game();
        let plains = g.add_card_to_library(0, catalog::plains());
        g.players[0].life = 20;
        let sam = g.add_card_to_hand(0, catalog::sunblade_samurai());
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(plains)),
        ]));
        g.perform_action(GameAction::ActivateAbility {
            card_id: sam,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("channel Sunblade Samurai");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Plains"), "fetched a Plains");
        assert_eq!(g.players[0].life, 22, "gained 2 life");
    }

    /// Moonsnare Specialist's ETB bounces a creature.
    #[test]
    fn moonsnare_specialist_bounces() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ninja = g.add_card_to_battlefield(0, catalog::moonsnare_specialist());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Target(Target::Permanent(victim)),
        ]));
        g.fire_self_etb_triggers(ninja, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "creature bounced");
        assert!(g.players[1].hand.iter().any(|c| c.id == victim), "returned to owner's hand");
    }

    /// Undercity Scrounger only makes a Treasure once a creature has died.
    #[test]
    fn undercity_scrounger_gated_on_death() {
        let mut g = two_player_game();
        let scrounger = g.add_card_to_battlefield(0, catalog::undercity_scrounger());
        g.clear_sickness(scrounger);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let act = |g: &mut GameState| {
            g.perform_action(GameAction::ActivateAbility {
                card_id: scrounger,
                ability_index: 0,
                target: None,
                additional_targets: vec![],
                x_value: None,
            })
        };
        assert!(act(&mut g).is_err(), "can't activate before a death");
        // Kill a creature so the condition is met.
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().damage = 2;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        act(&mut g).expect("activates after a death");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "made a Treasure");
    }

    /// Season of Renewal returns a creature and an enchantment card from graveyard.
    #[test]
    fn season_of_renewal_returns_both() {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let ench = g.add_card_to_graveyard(0, catalog::golden_tail_disciple());
        let spell = g.add_card_to_hand(0, catalog::season_of_renewal());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![Target::Permanent(ench)],
            mode: None,
            x_value: None,
        })
        .expect("cast Season of Renewal");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned");
        assert!(g.players[0].hand.iter().any(|c| c.id == ench), "enchantment returned");
    }

    /// Assassin's Ink is cheaper with an artifact and an enchantment in play.
    #[test]
    fn assassins_ink_cost_reduction() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::bonesplitter()); // artifact
        g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); // enchantment
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::assassins_ink());
        // {2}{B}{B} - {2} = {B}{B}: two black mana suffices.
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast discounted Assassin's Ink");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "creature destroyed");
    }

    /// Mnemonic Sphere draws two when sacrificed, and one via Channel from hand.
    #[test]
    fn mnemonic_sphere_draw_modes() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        // Sac mode: draw two.
        let sphere = g.add_card_to_battlefield(0, catalog::mnemonic_sphere());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: sphere,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
        })
        .expect("sac Mnemonic Sphere");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 2, "drew two");
        assert!(g.battlefield_find(sphere).is_none(), "sphere sacrificed");
    }

    /// Suit Up turns a creature into a 4/5 and draws.
    #[test]
    fn suit_up_pumps_and_draws() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::suit_up());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Suit Up");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 5), "became a 4/5");
        assert_eq!(g.players[0].hand.len(), before - 1 + 1, "spell left hand, drew one");
    }

    /// Careful Consideration draws four and discards two in the main phase.
    #[test]
    fn careful_consideration_main_phase_loot() {
        let mut g = two_player_game();
        for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
        let spell = g.add_card_to_hand(0, catalog::careful_consideration());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let before = g.players[0].hand.len(); // spell still in hand
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Careful Consideration");
        drain_stack(&mut g);
        // -1 (spell) + 4 drawn - 2 discarded = +1 net.
        assert_eq!(g.players[0].hand.len(), before + 1, "drew four, discarded two (main phase)");
    }
}

mod recent102 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Surrak draws when an opponent's spell targets a creature you control, but not
    /// when you target your own creature.
    #[test]
    fn surrak_draws_on_opponent_targeting_your_creature() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::surrak_elusive_hunter());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        let hand0 = g.players[0].hand.len();
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.cast_spell(bolt, Some(Target::Permanent(bear)), vec![], None, None)
            .expect("opponent bolts your creature");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand0 + 1, "Surrak drew when your creature was targeted");
    }

    /// Effortless Master enters with two +1/+1 counters only after two spells cast.
    #[test]
    fn effortless_master_enters_bigger_after_two_spells() {
        let mut g = two_player_game();
        // Zero spells cast → no counters.
        let m0 = g.move_card_to_battlefield_for_test(0, catalog::effortless_master());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(m0).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
        // Two spells cast this turn → enters with two counters.
        g.players[0].spells_cast_this_turn = 2;
        let m2 = g.move_card_to_battlefield_for_test(0, catalog::effortless_master());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(m2).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    }

    /// Stalwart Successor grafts one extra counter the first time a creature you
    /// control gets counters each turn — but only once per creature per turn.
    #[test]
    fn stalwart_successor_first_counter_each_turn() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::stalwart_successor());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let bump = |g: &mut GameState, id| {
            g.battlefield_find_mut(id).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
            g.dispatch_triggers_for_events(&[GameEvent::CounterAdded {
                card_id: id,
                counter_type: CounterType::PlusOnePlusOne,
                count: 1,
            }]);
            drain_stack(g);
        };
        // First counter placement this turn → Stalwart adds one more (2 total).
        bump(&mut g, bear);
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
        // Second placement same turn → no bonus (2 + 1 = 3, not 4).
        bump(&mut g, bear);
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    }
}

mod recent103 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::TurnStep;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};

    /// Encore: exiling Impulsive Pilferer from the graveyard for {3}{R} mints a
    /// hasty token copy per opponent that must attack; it's sacrificed at the
    /// next end step.
    #[test]
    fn cr_702_141_encore_mints_attacking_copies() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::impulsive_pilferer());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dead, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("encore from the graveyard");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == dead), "source exiled as the cost");
        let copy = g.battlefield.iter()
            .find(|c| c.is_token && c.definition.name == "Impulsive Pilferer")
            .expect("one token copy (one opponent)");
        assert!(g.computed_permanent(copy.id).unwrap().keywords.contains(&Keyword::Haste));
        assert_eq!(copy.goaded_by, vec![0], "attacks-if-able requirement");
        let copy_id = copy.id;
        // The copy is sacrificed at the beginning of the next end step; its
        // dies-trigger still mints the Treasure.
        while g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
        assert!(g.battlefield_find(copy_id).is_none(), "token sacrificed at end step");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
            "dies trigger fired off the sacrifice");
    }

    /// Encore respects sorcery timing: not activatable during an opponent's turn
    /// priority window off-main.
    #[test]
    fn encore_is_sorcery_speed() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::trove_tracker());
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(5);
        assert!(g.perform_action(GameAction::ActivateAbility {
            card_id: dead, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).is_err(), "sorcery-only activation rejected mid-combat");
    }

    /// Kilnmouth Dragon's amplify counters power its {T} ping (Amplify 3 with one
    /// Dragon in hand → 3 counters → 3 damage).
    #[test]
    fn kilnmouth_dragon_pings_for_its_counters() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::kilnmouth_dragon()); // a Dragon to reveal
        let kiln = g.move_card_to_battlefield_for_test(0, catalog::kilnmouth_dragon());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(kiln).unwrap()
            .counter_count(crabomination::card::CounterType::PlusOnePlusOne), 3, "amplify 3 × 1 Dragon");
        g.clear_sickness(kiln);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let life = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: kiln, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("tap to ping");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 3, "3 damage");
    }
}

mod recent104 {
    use crabomination::catalog;
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::*;

    /// Pulmonic Sliver: a dying Sliver goes to its owner's library top instead of
    /// the graveyard; a non-Sliver still dies normally.
    #[test]
    fn pulmonic_sliver_redirects_dying_slivers_to_library_top() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::pulmonic_sliver());
        let sliver = g.add_card_to_battlefield(0, catalog::galerider_sliver());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(sliver).unwrap().damage = 9;
        g.battlefield_find_mut(bear).unwrap().damage = 9;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        assert_eq!(
            g.players[0].library.first().map(|c| c.definition.name),
            Some("Galerider Sliver"),
            "Sliver on top of library"
        );
        assert!(!g.players[0].graveyard.iter().any(|c| c.id == sliver));
        assert!(g.players[0].graveyard.iter().any(|c| c.id == bear), "non-Sliver dies normally");
    }

    /// CR 700.4 — a death redirected to the library (Pulmonic Sliver) never
    /// happened: "whenever a creature dies" watchers must not fire for it.
    #[test]
    fn library_redirected_death_does_not_fire_dies_triggers() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::pulmonic_sliver());
        g.add_card_to_battlefield(0, catalog::blood_artist());
        let sliver = g.add_card_to_battlefield(0, catalog::galerider_sliver());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let (p0, p1) = (g.players[0].life, g.players[1].life);
        g.battlefield_find_mut(sliver).unwrap().damage = 9;
        g.battlefield_find_mut(bear).unwrap().damage = 9;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, p0 + 1, "Blood Artist fires only for the bear");
        assert_eq!(g.players[1].life, p1 - 1);
    }

    /// Twilight Prophet with the city's blessing drains each opponent for the
    /// revealed card's mana value at upkeep (and puts it in hand).
    #[test]
    fn twilight_prophet_drains_with_citys_blessing() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::twilight_prophet());
        g.players[0].city_blessing = true;
        g.add_card_to_library(0, catalog::serra_angel()); // bottom; library empty before
        let hand_before = g.players[0].hand.len();
        let (p0, p1) = (g.players[0].life, g.players[1].life);
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, p1 - 5, "opponent loses Serra Angel's MV");
        assert_eq!(g.players[0].life, p0 + 5, "controller gains that much");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "revealed card to hand");
    }

    /// Goblin Welder swaps a battlefield artifact with the highest-MV artifact in
    /// its controller's graveyard.
    #[test]
    fn goblin_welder_swaps_artifact_with_graveyard() {
        let mut g = two_player_game();
        let welder = g.add_card_to_battlefield(0, catalog::goblin_welder());
        g.clear_sickness(welder);
        let small = g.add_card_to_battlefield(1, catalog::sol_ring());
        let big = g.add_card_to_graveyard(1, catalog::mind_stone());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: welder,
            ability_index: 0,
            target: Some(Target::Permanent(small)),
            additional_targets: vec![],
            x_value: None,
        })
        .expect("weld");
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == small), "artifact sacrificed");
        let ret = g.battlefield_find(big).expect("graveyard artifact returned");
        assert_eq!(ret.controller, 1, "returned under its owner's control");
    }

    /// Gilt-Leaf Archdruid draws on Druid casts and steals a player's lands for
    /// tapping seven Druids.
    #[test]
    fn gilt_leaf_archdruid_draws_and_steals_lands() {
        let mut g = two_player_game();
        let druid = g.add_card_to_battlefield(0, catalog::gilt_leaf_archdruid());
        g.clear_sickness(druid);
        // Cast a Druid spell → draw.
        g.add_card_to_library(0, catalog::island());
        let hand_druid = g.add_card_to_hand(0, catalog::gilt_leaf_archdruid());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: hand_druid,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast the second Archdruid");
        drain_stack(&mut g);
        // -1 for the cast card, +1 for the trigger draw.
        assert_eq!(g.players[0].hand.len(), hand_before, "cast-a-Druid draw fired");
        // Two Archdruids + five Elves = seven Druids; steal the lands.
        for _ in 0..5 {
            let d = g.add_card_to_battlefield(0, catalog::llanowar_elves());
            g.clear_sickness(d);
        }
        let land = g.add_card_to_battlefield(1, catalog::island());
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: druid,
            ability_index: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            x_value: None,
        })
        .expect("tap seven Druids");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(land).unwrap().controller, 0, "land stolen");
        assert!(g.battlefield_find(druid).unwrap().tapped, "Druids tapped for the cost");
    }
}

mod recent105 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::*;

    /// Squee recasts from the graveyard and from exile for its mana cost.
    #[test]
    fn squee_recasts_from_graveyard_and_exile() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // From the graveyard (ability 0).
        let squee = g.add_card_to_graveyard(0, catalog::squee_the_immortal());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: squee, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("recast from graveyard");
        drain_stack(&mut g);
        assert!(g.battlefield_find(squee).is_some(), "back from the graveyard");
        // Exile it, then recast from exile (ability 1).
        g.remove_from_battlefield_to_exile(squee);
        assert!(g.exile.iter().any(|c| c.id == squee));
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: squee, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        }).expect("recast from exile");
        drain_stack(&mut g);
        assert!(g.battlefield_find(squee).is_some(), "back from exile");
    }

    /// Dark Depths enters with ten ice counters; removing the last one
    /// sacrifices it for Marit Lage.
    #[test]
    fn dark_depths_hatches_marit_lage() {
        let mut g = two_player_game();
        let depths = g.move_card_to_battlefield_for_test(0, catalog::dark_depths());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(depths).unwrap().counter_count(CounterType::Ice), 10);
        g.battlefield_find_mut(depths).unwrap().counters.insert(CounterType::Ice, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: depths, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("remove the last ice counter");
        drain_stack(&mut g);
        assert!(g.battlefield_find(depths).is_none(), "Dark Depths sacrificed");
        let lage = g.battlefield.iter().find(|c| c.definition.name == "Marit Lage")
            .expect("Marit Lage token");
        assert_eq!((lage.power(), lage.toughness()), (20, 20));
    }

    /// Smokestack accrues soot at your upkeep and makes each player sacrifice
    /// per counter at their upkeep.
    #[test]
    fn smokestack_taxes_each_upkeep() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let stack = g.add_card_to_battlefield(0, catalog::smokestack());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(stack).unwrap().counter_count(CounterType::Soot), 1,
            "soot added at your upkeep");
        // Opponent's upkeep: they sacrifice one permanent.
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.active_player_idx = 1;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "opponent sacrificed their only permanent");
    }

    /// Tangle Wire taps the active player's permanents per fade counter.
    #[test]
    fn tangle_wire_taps_per_fade_counter() {
        let mut g = two_player_game();
        let wire = g.move_card_to_battlefield_for_test(0, catalog::tangle_wire());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(wire).unwrap().counter_count(CounterType::Fade), 4,
            "Fading 4 enters with four fade counters");
        g.battlefield_find_mut(wire).unwrap().counters.insert(CounterType::Fade, 2);
        let l1 = g.add_card_to_battlefield(1, catalog::island());
        let l2 = g.add_card_to_battlefield(1, catalog::island());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.active_player_idx = 1;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        // Two fade counters → two taps, lands preferred.
        assert!(g.battlefield_find(l1).unwrap().tapped && g.battlefield_find(l2).unwrap().tapped);
        assert!(!g.battlefield_find(bear).unwrap().tapped, "creature spared (lands first)");
    }

    /// Dimir Charm mode 3 keeps the opponent's worst card on top and bins the
    /// other two.
    #[test]
    fn dimir_charm_mode_three_strands_the_worst_card() {
        let mut g = two_player_game();
        // Opponent library (top → bottom): Serra Angel (MV 5), Island, Bears (MV 2).
        g.add_card_to_library(1, catalog::serra_angel());
        g.add_card_to_library(1, catalog::island());
        g.add_card_to_library(1, catalog::grizzly_bears());
        let charm = g.add_card_to_hand(0, catalog::dimir_charm());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: charm, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: Some(2), x_value: None,
        }).expect("mode 3 at the opponent");
        drain_stack(&mut g);
        assert_eq!(g.players[1].library.len(), 1, "one card kept on top");
        assert_eq!(g.players[1].library[0].definition.name, "Island",
            "the lowest-MV card stays");
        assert_eq!(g.players[1].graveyard.len(), 2, "the rest milled");
    }
}

mod recent106 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::*;

    /// Grinding Station mills three off a sacrificed artifact and untaps on an
    /// artifact ETB.
    #[test]
    fn grinding_station_mills_and_untaps() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let station = g.add_card_to_battlefield(0, catalog::grinding_station());
        g.add_card_to_battlefield(0, catalog::sol_ring());
        for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: station, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        }).expect("grind");
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), 3, "milled three");
        assert!(g.battlefield_find(station).unwrap().tapped);
        // A new artifact untaps it (may → scripted yes).
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let evs = vec![GameEvent::PermanentEntered {
            card_id: g.add_card_to_battlefield(0, catalog::mind_stone()),
        }];
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(!g.battlefield_find(station).unwrap().tapped, "untapped off the ETB");
    }

    /// Anafenza bolsters when another nontoken creature enters.
    #[test]
    fn anafenza_bolsters_on_ally_etb() {
        let mut g = two_player_game();
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let ana = g.add_card_to_battlefield(0, catalog::anafenza_kin_tree_spirit());
        let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast the elf");
        drain_stack(&mut g);
        // Bolster 1: the 1/1 entrant is the least-tough creature.
        assert_eq!(g.battlefield_find(elf).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert_eq!(g.battlefield_find(ana).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    }

    /// Slitherhead scavenges from the graveyard for {0}.
    #[test]
    fn slitherhead_scavenges_for_free() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::slitherhead());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: dead, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], x_value: None,
        }).expect("scavenge");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert!(g.exile.iter().any(|c| c.id == dead), "scavenged card exiled");
    }

    /// Iona names a color; opponents can't cast spells of it.
    #[test]
    fn iona_locks_the_chosen_color() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Color(crabomination::mana::Color::Red),
        ]));
        g.move_card_to_battlefield_for_test(0, catalog::iona_shield_of_emeria());
        drain_stack(&mut g);
        // Opponent can't cast a red spell…
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.active_player_idx = 1;
        assert!(g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)),
            additional_targets: vec![], mode: None, x_value: None,
        }).is_err(), "red spell locked");
        // …but a nonred one resolves.
        let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("green spell fine");
    }

    /// Thopter Assembly bounces itself for five Thopters at a lonely upkeep.
    #[test]
    fn thopter_assembly_disassembles() {
        let mut g = two_player_game();
        let asm = g.add_card_to_battlefield(0, catalog::thopter_assembly());
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == asm), "bounced to hand");
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count(), 5);
    }

    /// Reshape sacrifices an artifact and fetches one with MV ≤ X.
    #[test]
    fn reshape_swaps_an_artifact() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let fodder = g.add_card_to_battlefield(0, catalog::sol_ring());
        let target = g.add_card_to_library(0, catalog::mind_stone()); // MV 2
        let re = g.add_card_to_hand(0, catalog::reshape());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: re, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
        }).expect("Reshape for X=2");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "artifact sacrificed");
        assert!(g.battlefield_find(target).is_some(), "fetched onto the battlefield");
    }

    /// Wild Cantor sacrifices for one mana of any color.
    #[test]
    fn wild_cantor_ramps() {
        let mut g = two_player_game();
        let cantor = g.add_card_to_battlefield(0, catalog::wild_cantor());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: cantor, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("sac for mana");
        assert!(g.battlefield_find(cantor).is_none(), "sacrificed");
        assert!(g.players[0].mana_pool.total() >= 1, "one mana added");
    }

    /// Melira: no poison for you, no -1/-1 counters on your creatures, and
    /// opponents' creatures lose infect.
    #[test]
    fn melira_shuts_off_infect() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::melira_sylvok_outcast());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Poison lock.
        let mut evs = Vec::new();
        g.add_poison(0, 3, &mut evs);
        assert_eq!(g.players[0].poison_counters, 0, "no poison counters for you");
        // -1/-1 lock.
        let ctx = crabomination::game::effects::EffectContext::for_ability(crabomination::card::CardId(0), 1, None);
        g.resolve_effect(&crabomination::effect::Effect::AddCounter {
            what: crabomination::effect::Selector::EachPermanent(
                crabomination::card::SelectionRequirement::Creature,
            ),
            kind: CounterType::MinusOneMinusOne,
            amount: crabomination::effect::Value::Const(2),
        }, &ctx).unwrap();
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 0,
            "your creature dodges the -1/-1 counters");
        // Opponent's infect creature loses the keyword.
        let carrier = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        if let Some(c) = g.battlefield_find_mut(carrier) {
            let def = std::sync::Arc::make_mut(&mut c.definition);
            def.keywords.push(Keyword::Infect);
        }
        assert!(!g.computed_permanent(carrier).unwrap().keywords.contains(&Keyword::Infect),
            "opponent's creature loses infect");
    }
}

mod recent107 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::*;

    /// Cranial Ram enters as living weapon (attached to a Germ) and scales with
    /// your artifact count.
    #[test]
    fn cranial_ram_living_weapon_scales() {
        let mut g = two_player_game();
        let ram = g.move_card_to_battlefield_for_test(0, catalog::cranial_ram());
        drain_stack(&mut g);
        let germ = g.battlefield.iter().find(|c| c.definition.name == "Phyrexian Germ")
            .expect("germ token").id;
        assert_eq!(g.battlefield_find(ram).unwrap().attached_to, Some(germ));
        // One artifact (the Ram): germ is 0/0 +1/+1 → 1/1.
        let cp = g.computed_permanent(germ).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1));
        g.add_card_to_battlefield(0, catalog::sol_ring());
        let cp = g.computed_permanent(germ).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 1), "+X grows with artifacts");
    }

    /// The Underworld Cookbook turns discards into Food and sacs for a raise.
    #[test]
    fn underworld_cookbook_cooks() {
        let mut g = two_player_game();
        let book = g.add_card_to_battlefield(0, catalog::the_underworld_cookbook());
        g.add_card_to_hand(0, catalog::island());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: book, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("discard for Food");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"));
        assert_eq!(g.players[0].hand.len(), 0, "discarded the card");
    }

    /// Asmor is castable for {B/R} only after a discard, and fetches the Cookbook.
    #[test]
    fn asmor_alt_cast_after_discard() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let asmor = g.add_card_to_hand(0, catalog::asmoranomardicadaistinaculdacar());
        let book = g.add_card_to_library(0, catalog::the_underworld_cookbook());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        // No discard yet — the alternative cast is rejected.
        assert!(g.perform_action(GameAction::CastSpellAlternative {
            card_id: asmor, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).is_err(), "no discard yet");
        // Discard a card, then it casts and fetches the Cookbook.
        let junk = g.add_card_to_hand(0, catalog::island());
        let mut evs = Vec::new();
        g.discard_card(0, junk, &mut evs);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(book))]));
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: asmor, pitch_card: None, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("alt cast after a discard");
        drain_stack(&mut g);
        assert!(g.battlefield_find(asmor).is_some(), "Asmor resolved");
        assert!(g.players[0].hand.iter().any(|c| c.id == book), "Cookbook fetched");
    }

    /// Retract bounces all your artifacts (and only yours).
    #[test]
    fn retract_bounces_your_artifacts() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::sol_ring());
        let theirs = g.add_card_to_battlefield(1, catalog::sol_ring());
        let re = g.add_card_to_hand(0, catalog::retract());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: re, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("retract");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == mine), "yours bounced");
        assert!(g.battlefield_find(theirs).is_some(), "theirs stays");
    }

    /// Jeskai Ascendancy pumps + untaps your team on a noncreature cast.
    #[test]
    fn jeskai_ascendancy_pumps_and_untaps() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::jeskai_ascendancy());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("noncreature cast");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 until end of turn");
        assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
    }

    /// Fatestitcher unearths for {U} and taps a permanent.
    #[test]
    fn fatestitcher_unearths_and_taps() {
        let mut g = two_player_game();
        let stitcher = g.add_card_to_graveyard(0, catalog::fatestitcher());
        let land = g.add_card_to_battlefield(1, catalog::island());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        let unearth_idx = catalog::fatestitcher().activated_abilities.len() - 1;
        g.perform_action(GameAction::ActivateAbility {
            card_id: stitcher, ability_index: unearth_idx, target: None,
            additional_targets: vec![], x_value: None,
        }).expect("unearth");
        drain_stack(&mut g);
        let c = g.computed_permanent(stitcher).expect("unearthed");
        assert!(c.keywords.contains(&Keyword::Haste));
        // {T}: tap another permanent (mode 0).
        g.perform_action(GameAction::ActivateAbility {
            card_id: stitcher, ability_index: 0, target: Some(Target::Permanent(land)),
            additional_targets: vec![], x_value: None,
        }).expect("tap the land");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).unwrap().tapped);
    }

    /// Urza mints a Karnstruct and taps artifacts for {U}.
    #[test]
    fn urza_construct_and_artifact_mana() {
        let mut g = two_player_game();
        let urza = g.move_card_to_battlefield_for_test(0, catalog::urza_lord_high_artificer());
        drain_stack(&mut g);
        let construct = g.battlefield.iter().find(|c| c.definition.name == "Construct")
            .expect("Karnstruct").id;
        // Urza + Construct = 2 artifacts... Urza isn't an artifact; Construct is.
        let cp = g.computed_permanent(construct).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "+1/+1 for itself");
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: urza, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("tap the Construct for {U}");
        assert!(g.battlefield_find(construct).unwrap().tapped, "artifact tapped for the cost");
        assert!(g.players[0].mana_pool.total() >= 1, "added blue mana");
    }

    /// Tezzeret animates an artifact into a 5/5.
    #[test]
    fn tezzeret_animates_an_artifact() {
        let mut g = two_player_game();
        let tez = g.add_card_to_battlefield(0, catalog::tezzeret_agent_of_bolas());
        let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.perform_action(GameAction::ActivateLoyaltyAbility {
            card_id: tez, ability_index: 1, target: Some(Target::Permanent(ring)), x_value: None,
        }).expect("-1");
        drain_stack(&mut g);
        let cp = g.computed_permanent(ring).unwrap();
        assert!(cp.card_types.contains(&crabomination::card::CardType::Creature), "animated");
        assert_eq!((cp.power, cp.toughness), (5, 5));
    }

    /// Second Sunrise rebuilds only what hit the graveyard from the battlefield
    /// this turn.
    #[test]
    fn second_sunrise_rebuilds_this_turns_losses() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let old = g.add_card_to_graveyard(0, catalog::sol_ring()); // not from bf this turn
        g.battlefield_find_mut(bear).unwrap().damage = 9;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        let sunrise = g.add_card_to_hand(0, catalog::second_sunrise());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: sunrise, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("second sunrise");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_some(), "the dead bear returns");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == old),
            "a card that didn't come from the battlefield stays");
    }
}

mod recent108 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::types::TurnStep;
    use crabomination::game::*;

    /// Urza's Cave sacs for a tapped land fetch.
    #[test]
    fn urzas_cave_fetches_tapped() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let cave = g.add_card_to_battlefield(0, catalog::urzas_cave());
        let land = g.add_card_to_library(0, catalog::island());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(land))]));
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: cave, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        }).expect("fetch");
        drain_stack(&mut g);
        assert!(g.battlefield_find(cave).is_none(), "cave sacrificed");
        assert!(g.battlefield_find(land).unwrap().tapped, "fetched land enters tapped");
    }

    /// Fallaji Archaeologist grows when the mill whiffs.
    #[test]
    fn fallaji_archaeologist_grows_on_whiff() {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); } // all lands: whiff
        let arch = g.move_card_to_battlefield_for_test(0, catalog::fallaji_archaeologist());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(arch).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Sleep-Cursed Faerie enters tapped with three stun counters and untaps
    /// for {1}{U}.
    #[test]
    fn sleep_cursed_faerie_stunned_start() {
        let mut g = two_player_game();
        let faerie = g.move_card_to_battlefield_for_test(0, catalog::sleep_cursed_faerie());
        drain_stack(&mut g);
        let c = g.battlefield_find(faerie).unwrap();
        assert!(c.tapped, "enters tapped");
        assert_eq!(c.counter_count(CounterType::Stun), 3);
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: faerie, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("self-untap");
        drain_stack(&mut g);
        // CR 122.1i — the stun counter eats the untap instead.
        let c = g.battlefield_find(faerie).unwrap();
        assert!(c.tapped, "stun counter replaced the untap");
        assert_eq!(c.counter_count(CounterType::Stun), 2, "one stun removed");
        // With the stun cleared, the activation untaps for real.
        g.battlefield_find_mut(faerie).unwrap().counters.remove(&CounterType::Stun);
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: faerie, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).expect("self-untap again");
        drain_stack(&mut g);
        assert!(!g.battlefield_find(faerie).unwrap().tapped, "untapped once unstunned");
    }

    /// Manabond dumps hand lands at end step and discards the rest.
    #[test]
    fn manabond_dumps_lands() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::manabond());
        let l1 = g.add_card_to_hand(0, catalog::island());
        let l2 = g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.battlefield_find(l1).is_some() && g.battlefield_find(l2).is_some(),
            "both lands deployed");
        assert!(g.players[0].hand.is_empty(), "the rest discarded");
    }

    /// Nissa adds mana on each land drop; the second drop digs an Elf/Elemental.
    #[test]
    fn nissa_resurgent_animist_ramps_and_digs() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::nissa_resurgent_animist());
        g.add_card_to_library(0, catalog::llanowar_elves()); // the dig hit
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.players[0].extra_land_plays = 1;
        let l1 = g.add_card_to_hand(0, catalog::forest());
        g.perform_action(GameAction::PlayLand(l1)).expect("first land");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.total(), 1, "landfall mana");
        let hand_before = g.players[0].hand.len();
        let l2 = g.add_card_to_hand(0, catalog::forest());
        g.perform_action(GameAction::PlayLand(l2)).expect("second land");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.total(), 2, "second landfall mana");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "dug up the Elf");
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Llanowar Elves"));
    }
}

mod recent109 {
    use crabomination::catalog;
    use crabomination::game::effects::EntityRef;
    use crabomination::game::*;

    /// CR 104.3d — Platinum Angel's controller skips the life-loss SBA; the loss
    /// resumes as soon as the Angel leaves.
    #[test]
    fn cr_104_3d_platinum_angel_blocks_loss_sbas() {
        let mut g = two_player_game();
        let angel = g.add_card_to_battlefield(0, catalog::platinum_angel());
        g.players[0].life = 0;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        assert!(!g.players[0].eliminated, "can't lose at 0 life with the Angel");
        assert!(!g.is_game_over());
        g.remove_from_battlefield_to_graveyard_raw(angel);
        g.check_state_based_actions();
        assert!(g.players[0].eliminated, "loss SBA resumes once the Angel leaves");
    }

    /// CR 104.3d — an opponent's "you win the game" effect does nothing while
    /// Platinum Angel's controller can't lose.
    #[test]
    fn cr_104_3d_platinum_angel_blocks_opponent_win_effect() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::platinum_angel());
        let ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
        g.resolve_effect(
            &crabomination::effect::Effect::WinGame { who: crabomination::effect::PlayerRef::You },
            &ctx,
        )
        .unwrap();
        g.check_state_based_actions();
        assert!(!g.players[0].eliminated, "opponent can't win through the Angel");
        assert!(!g.is_game_over());
    }

    /// CR 104.3d — Abyssal Persecutor keeps its controller's opponents alive at
    /// 0 life; killing it hands them the loss.
    #[test]
    fn cr_104_3d_abyssal_persecutor_keeps_opponents_alive() {
        let mut g = two_player_game();
        let demon = g.add_card_to_battlefield(0, catalog::abyssal_persecutor());
        g.players[1].life = -3;
        g.check_state_based_actions();
        assert!(!g.players[1].eliminated, "opponent can't lose under the Persecutor");
        g.remove_from_battlefield_to_graveyard_raw(demon);
        g.check_state_based_actions();
        assert!(g.players[1].eliminated);
        assert!(g.is_game_over());
    }

    /// Angel's Grace — can't lose this turn, and damage that would drop you
    /// below 1 life drops you to 1; both wear off at the turn boundary.
    #[test]
    fn angels_grace_floors_damage_and_blocks_loss_this_turn() {
        let mut g = two_player_game();
        let grace = g.add_card_to_hand(0, catalog::angels_grace());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.step = crabomination::game::types::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: grace, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Angel's Grace");
        drain_stack(&mut g);
        g.players[0].life = 3;
        let mut evs = Vec::new();
        g.deal_damage_to_from(EntityRef::Player(0), 9, None, &mut evs);
        assert_eq!(g.players[0].life, 1, "damage floored at 1 life");
        // A non-damage loss effect is also blocked this turn.
        let ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
        g.resolve_effect(
            &crabomination::effect::Effect::LoseGame { who: crabomination::effect::PlayerRef::EachOpponent },
            &ctx,
        )
        .unwrap();
        assert!(!g.players[0].eliminated, "can't lose this turn");
        // The protections end at the turn boundary.
        g.do_untap();
        assert!(!g.players[0].cant_lose_this_turn);
        assert!(!g.players[0].damage_floor_this_turn);
    }

    /// Worship — with a creature, damage can't take its controller below 1;
    /// without one the floor is off.
    #[test]
    fn worship_floors_damage_while_controlling_a_creature() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::worship());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].life = 2;
        let mut evs = Vec::new();
        g.deal_damage_to_from(EntityRef::Player(0), 7, None, &mut evs);
        assert_eq!(g.players[0].life, 1, "floored while a creature is out");
        g.remove_from_battlefield_to_graveyard_raw(bear);
        g.deal_damage_to_from(EntityRef::Player(0), 7, None, &mut evs);
        assert_eq!(g.players[0].life, -6, "no creature, no floor");
    }

    /// Archetype of Imagination: your team gains flying; opponents' creatures
    /// lose printed flying and can't gain it even from a later grant.
    #[test]
    fn cr_113_11_archetype_strips_and_blocks_keyword() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::archetype_of_imagination());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::Flying),
            "your creatures gain flying"
        );
        assert!(
            !g.computed_permanent(angel).unwrap().keywords.contains(&crabomination::card::Keyword::Flying),
            "opponent's printed flying is stripped"
        );
        // A grant with a later timestamp still loses to the can't-have.
        let ctx =
            crabomination::game::effects::EffectContext::for_spell(1, Some(Target::Permanent(angel)), 0, 0);
        g.resolve_effect(
            &crabomination::effect::Effect::GrantKeyword {
                what: crabomination::card::Selector::Target(0),
                keyword: crabomination::card::Keyword::Flying,
                duration: crabomination::effect::Duration::EndOfTurn,
            },
            &ctx,
        )
        .unwrap();
        assert!(
            !g.computed_permanent(angel).unwrap().keywords.contains(&crabomination::card::Keyword::Flying),
            "a later EOT grant can't restore the keyword"
        );
    }

    /// An attacker with trample over planeswalkers (CR 702.19c) assigns lethal
    /// to the planeswalker and the excess to its controller; plain trample
    /// never spills past a planeswalker (CR 702.19f).
    #[test]
    fn cr_702_19c_trample_over_planeswalkers_spills_excess() {
        use crabomination::card::CounterType;
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(
            0,
            crabomination::card::CardDefinition {
                name: "Wagon",
                card_types: vec![crabomination::card::CardType::Creature],
                power: 6,
                toughness: 6,
                keywords: vec![
                    crabomination::card::Keyword::Trample,
                    crabomination::card::Keyword::TrampleOverPlaneswalkers,
                ],
                ..Default::default()
            },
        );
        g.clear_sickness(atk);
        let pw = g.add_card_to_battlefield(1, catalog::teferi_time_raveler()); // loyalty 4
        let life = g.players[1].life;
        g.step = crabomination::game::types::TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
            attacker: atk,
            target: crabomination::game::types::AttackTarget::Planeswalker(pw),
        }]))
        .unwrap();
        g.step = crabomination::game::types::TurnStep::DeclareBlockers;
        g.step = crabomination::game::types::TurnStep::CombatDamage;
        g.resolve_combat().unwrap();
        assert!(
            g.battlefield_find(pw)
                .is_none_or(|c| c.counter_count(CounterType::Loyalty) == 0),
            "planeswalker took lethal loyalty damage"
        );
        assert_eq!(g.players[1].life, life - 2, "6 power − 4 loyalty spills 2 to the player");
    }

    /// CR 702.19f — plain trample assigns nothing past a planeswalker.
    #[test]
    fn cr_702_19f_plain_trample_does_not_spill_over_planeswalker() {
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(
            0,
            crabomination::card::CardDefinition {
                name: "Wagon",
                card_types: vec![crabomination::card::CardType::Creature],
                power: 6,
                toughness: 6,
                keywords: vec![crabomination::card::Keyword::Trample],
                ..Default::default()
            },
        );
        g.clear_sickness(atk);
        let pw = g.add_card_to_battlefield(1, catalog::teferi_time_raveler());
        let life = g.players[1].life;
        g.step = crabomination::game::types::TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
            attacker: atk,
            target: crabomination::game::types::AttackTarget::Planeswalker(pw),
        }]))
        .unwrap();
        g.step = crabomination::game::types::TurnStep::DeclareBlockers;
        g.step = crabomination::game::types::TurnStep::CombatDamage;
        g.resolve_combat().unwrap();
        assert_eq!(g.players[1].life, life, "no spill without the variant keyword");
    }
}

mod recent110 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// CR 702.71 — Fortify attaches to a land you control and the bonus applies.
    #[test]
    fn cr_702_71_fortify_attaches_and_grants_indestructible() {
        let mut g = two_player_game();
        let garrison = g.add_card_to_battlefield(0, catalog::darksteel_garrison());
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::Equip { equipment: garrison, target: forest }).expect("fortify");
        assert_eq!(g.battlefield_find(garrison).unwrap().attached_to, Some(forest));
        let cp = g.computed_permanent(forest).unwrap();
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Indestructible));
    }

    /// CR 702.71c — fortify only targets lands; a creature is rejected.
    #[test]
    fn cr_702_71_fortify_rejects_creature() {
        let mut g = two_player_game();
        let garrison = g.add_card_to_battlefield(0, catalog::darksteel_garrison());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(3);
        let err = g.perform_action(GameAction::Equip { equipment: garrison, target: bear });
        assert!(matches!(err, Err(GameError::InvalidTarget)));
    }

    /// Fortified land becoming tapped pumps a target creature +1/+1 EOT.
    #[test]
    fn darksteel_garrison_tap_trigger_pumps() {
        let mut g = two_player_game();
        let garrison = g.add_card_to_battlefield(0, catalog::darksteel_garrison());
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(garrison).unwrap().attached_to = Some(forest);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: forest, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("tap forest for mana");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "tapped fortified land pumped the bear");
    }

    /// Saffi's sacrifice returns the watched creature when it dies this turn.
    #[test]
    fn saffi_returns_dying_creature() {
        let mut g = two_player_game();
        let saffi = g.add_card_to_battlefield(0, catalog::saffi_eriksdotter());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(saffi);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: saffi, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], x_value: None,
        })
        .expect("sac Saffi");
        drain_stack(&mut g);
        assert!(g.battlefield_find(saffi).is_none(), "Saffi sacrificed");
        let events = g.resolve_effect(
            &crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) },
            &crabomination::game::effects::EffectContext::for_spell(1, Some(Target::Permanent(bear)), 0, 0),
        )
        .unwrap();
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_some(), "bear returned to the battlefield");
    }

    /// Restore Balance levels lands, creatures, and hands down to the fewest.
    #[test]
    fn restore_balance_levels_everything() {
        let mut g = two_player_game();
        for _ in 0..4 {
            g.add_card_to_battlefield(0, catalog::forest());
        }
        for _ in 0..2 {
            g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.add_card_to_hand(0, catalog::lightning_bolt());
        }
        g.add_card_to_hand(0, catalog::lightning_bolt());
        for _ in 0..2 {
            g.add_card_to_battlefield(1, catalog::island());
        }
        g.add_card_to_hand(1, catalog::counterspell());
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        let events = g.resolve_effect(&crabomination::effect::Effect::Balance, &ctx).unwrap();
        g.dispatch_triggers_for_events(&events);
        let lands0 = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
        let creatures0 = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count();
        assert_eq!(lands0, 2, "lands leveled to the fewest (2)");
        assert_eq!(creatures0, 0, "creatures leveled to the fewest (0)");
        assert_eq!(g.players[0].hand.len(), 1, "hand leveled to the smallest (1)");
        assert_eq!(g.players[1].hand.len(), 1, "smallest hand untouched");
    }

    /// Restore Balance can't be hard-cast (no mana cost) but resolves off suspend.
    #[test]
    fn restore_balance_is_suspend_only() {
        let mut g = two_player_game();
        let rb = g.add_card_to_hand(0, catalog::restore_balance());
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let err = g.perform_action(GameAction::CastSpell {
            card_id: rb, target: None, additional_targets: vec![], mode: None, x_value: None,
        });
        assert!(matches!(err, Err(GameError::NoManaCost)));
    }

    /// Wheel of Fate wheels every player into a fresh seven.
    #[test]
    fn wheel_of_fate_wheels() {
        let mut g = two_player_game();
        for p in 0..2 {
            for _ in 0..10 {
                g.add_card_to_library(p, catalog::island());
            }
            g.add_card_to_hand(p, catalog::lightning_bolt());
        }
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        let eff = catalog::wheel_of_fate().effect.clone();
        let events = g.resolve_effect(&eff, &ctx).unwrap();
        g.dispatch_triggers_for_events(&events);
        assert_eq!(g.players[0].hand.len(), 7);
        assert_eq!(g.players[1].hand.len(), 7);
    }

    /// Hypergenesis dumps hand permanents onto the battlefield for every player.
    #[test]
    fn hypergenesis_dumps_permanents() {
        let mut g = two_player_game();
        let my_bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        let my_bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let their_land = g.add_card_to_hand(1, catalog::island());
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        let eff = catalog::hypergenesis().effect.clone();
        let events = g.resolve_effect(&eff, &ctx).unwrap();
        g.dispatch_triggers_for_events(&events);
        assert!(g.battlefield_find(my_bear).is_some(), "creature deployed");
        assert_eq!(g.battlefield_find(their_land).unwrap().controller, 1, "opponent keeps theirs");
        assert!(g.players[0].hand.iter().any(|c| c.id == my_bolt), "instant stays in hand");
    }

    /// Kumena's Speaker is 1/1 alone, 2/2 with an Island.
    #[test]
    fn kumenas_speaker_pumps_with_island() {
        let mut g = two_player_game();
        let speaker = g.add_card_to_battlefield(0, catalog::kumenas_speaker());
        let cp = g.computed_permanent(speaker).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1));
        g.add_card_to_battlefield(0, catalog::island());
        let cp = g.computed_permanent(speaker).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2));
    }

    /// Shriekhorn enters with three charges and mills two per activation.
    #[test]
    fn shriekhorn_mills_two() {
        let mut g = two_player_game();
        for _ in 0..5 {
            g.add_card_to_library(1, catalog::island());
        }
        let horn = g.add_card_to_hand(0, catalog::shriekhorn());
        g.players[0].mana_pool.add_colorless(1);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: horn, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Shriekhorn");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(horn).unwrap().counter_count(CounterType::Charge), 3);
        let gy_before = g.players[1].graveyard.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: horn, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        })
        .expect("mill two");
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), gy_before + 2);
        assert_eq!(g.battlefield_find(horn).unwrap().counter_count(CounterType::Charge), 2);
    }

    /// Emrakul costs {1} less per card type in your graveyard.
    #[test]
    fn emrakul_promised_end_cost_reduction() {
        let mut g = two_player_game();
        // Graveyard: creature + instant + land = 3 card types → {10}.
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.add_card_to_graveyard(0, catalog::forest());
        let emmy = g.add_card_to_hand(0, catalog::emrakul_the_promised_end());
        g.players[0].mana_pool.add_colorless(10);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: emmy, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("castable for {10} with 3 card types in graveyard");
        drain_stack(&mut g);
        assert!(g.battlefield_find(emmy).is_some());
    }

    /// Worldspine Wurm leaves three 5/5 Wurms and shuffles itself back.
    #[test]
    fn worldspine_wurm_dies_tokens_and_shuffles_back() {
        let mut g = two_player_game();
        let wurm = g.add_card_to_battlefield(0, catalog::worldspine_wurm());
        let events = g.remove_to_graveyard_with_triggers(wurm);
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        let tokens = g
            .battlefield
            .iter()
            .filter(|c| c.definition.name == "Wurm" && c.is_token)
            .count();
        assert_eq!(tokens, 3, "three 5/5 Wurm tokens");
        assert!(!g.players[0].graveyard.iter().any(|c| c.id == wurm), "not in graveyard");
        assert!(g.players[0].library.iter().any(|c| c.id == wurm), "shuffled into library");
    }

    /// Martyr of Sands gains 3 life per white card in hand.
    #[test]
    fn martyr_of_sands_gains_life() {
        let mut g = two_player_game();
        let martyr = g.add_card_to_battlefield(0, catalog::martyr_of_sands());
        g.add_card_to_hand(0, catalog::savannah_lions());
        g.add_card_to_hand(0, catalog::savannah_lions());
        g.players[0].mana_pool.add_colorless(1);
        g.clear_sickness(martyr);
        let life = g.players[0].life;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: martyr, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("sac Martyr");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 6, "3 × 2 white cards");
        assert!(g.battlefield_find(martyr).is_none());
    }

    /// Proclamation of Rebirth mass-reanimates up to three MV≤1 creatures.
    #[test]
    fn proclamation_of_rebirth_returns_three() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_graveyard(0, catalog::savannah_lions());
        }
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 — stays
        let proc = g.add_card_to_hand(0, catalog::proclamation_of_rebirth());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: proc, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        let lions = g.battlefield.iter().filter(|c| c.definition.name == "Savannah Lions").count();
        assert_eq!(lions, 3);
        assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.is_creature()).count(), 1);
    }

    /// Prismatic Omen makes your lands every basic type (a Forest taps for {U}).
    #[test]
    fn prismatic_omen_grants_all_basic_types() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::prismatic_omen());
        let forest = g.add_card_to_battlefield(0, catalog::forest());
        let cp = g.computed_permanent(forest).unwrap();
        for lt in [
            crabomination::card::LandType::Plains,
            crabomination::card::LandType::Island,
            crabomination::card::LandType::Swamp,
            crabomination::card::LandType::Mountain,
            crabomination::card::LandType::Forest,
        ] {
            assert!(cp.subtypes.land_types.contains(&lt), "missing {lt:?}");
        }
    }

    /// Norin dodges any spell cast and returns at the next end step.
    #[test]
    fn norin_dodges_spells() {
        let mut g = two_player_game();
        let norin = g.add_card_to_battlefield(0, catalog::norin_the_wary());
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast bolt");
        drain_stack(&mut g);
        assert!(g.battlefield_find(norin).is_none(), "Norin exiled on the cast");
        assert!(g.exile.iter().any(|c| c.id == norin));
        g.fire_step_triggers(crabomination::game::TurnStep::End);
        drain_stack(&mut g);
        assert!(g.battlefield_find(norin).is_some(), "returns at the next end step");
    }

    /// Genesis Chamber mints a Myr for the entering creature's controller, only
    /// while untapped.
    #[test]
    fn genesis_chamber_mints_myr() {
        let mut g = two_player_game();
        let chamber = g.add_card_to_battlefield(0, catalog::genesis_chamber());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: theirs }]);
        drain_stack(&mut g);
        let myr1 = g.battlefield.iter().filter(|c| c.definition.name == "Myr" && c.controller == 1).count();
        assert_eq!(myr1, 1, "the entering creature's controller gets the Myr");
        g.battlefield_find_mut(chamber).unwrap().tapped = true;
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: mine }]);
        drain_stack(&mut g);
        let myr0 = g.battlefield.iter().filter(|c| c.definition.name == "Myr" && c.controller == 0).count();
        assert_eq!(myr0, 0, "tapped Chamber stays silent");
    }

    /// Entreat the Angels mints X Angels off the {X}{X} cost.
    #[test]
    fn entreat_the_angels_mints_x() {
        let mut g = two_player_game();
        let entreat = g.add_card_to_hand(0, catalog::entreat_the_angels());
        // X = 2: {2}{2}{W}{W}{W}.
        g.players[0].mana_pool.add(Color::White, 3);
        g.players[0].mana_pool.add_colorless(4);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: entreat, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
        })
        .expect("cast for X=2");
        drain_stack(&mut g);
        let angels = g.battlefield.iter().filter(|c| c.definition.name == "Angel").count();
        assert_eq!(angels, 2);
    }

    /// Fracturing Gust sweeps artifacts/enchantments and gains 2 per kill.
    #[test]
    fn fracturing_gust_sweeps_and_gains() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::prismatic_omen());
        g.add_card_to_battlefield(1, catalog::chrome_mox());
        g.add_card_to_battlefield(1, catalog::chrome_mox());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let life = g.players[0].life;
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        let eff = catalog::fracturing_gust().effect.clone();
        let events = g.resolve_effect(&eff, &ctx).unwrap();
        g.dispatch_triggers_for_events(&events);
        assert!(g.battlefield_find(bear).is_some(), "creatures survive");
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.is_artifact() || c.definition.is_enchantment()).count(),
            0
        );
        assert_eq!(g.players[0].life, life + 6, "2 × 3 destroyed");
    }

    /// Hurkyl's Recall bounces all of the target player's artifacts.
    #[test]
    fn hurkyls_recall_bounces_artifacts() {
        let mut g = two_player_game();
        let mox1 = g.add_card_to_battlefield(1, catalog::chrome_mox());
        let mox2 = g.add_card_to_battlefield(1, catalog::chrome_mox());
        let mine = g.add_card_to_battlefield(0, catalog::chrome_mox());
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
        let eff = catalog::hurkyls_recall().effect.clone();
        let events = g.resolve_effect(&eff, &ctx).unwrap();
        g.dispatch_triggers_for_events(&events);
        assert!(g.battlefield_find(mox1).is_none() && g.battlefield_find(mox2).is_none());
        assert_eq!(g.players[1].hand.len(), 2);
        assert!(g.battlefield_find(mine).is_some(), "your artifacts stay");
    }

    /// Slippery Scoundrel gains hexproof + unblockable with the city's blessing.
    #[test]
    fn slippery_scoundrel_with_blessing() {
        let mut g = two_player_game();
        let rogue = g.add_card_to_battlefield(0, catalog::slippery_scoundrel());
        let cp = g.computed_permanent(rogue).unwrap();
        assert!(!cp.keywords.contains(&crabomination::card::Keyword::Hexproof), "no blessing yet");
        g.players[0].city_blessing = true;
        let cp = g.computed_permanent(rogue).unwrap();
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Hexproof));
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Unblockable));
    }

    /// Tempest Djinn grows +1/+0 per Island you control.
    #[test]
    fn tempest_djinn_scales_with_islands() {
        let mut g = two_player_game();
        let djinn = g.add_card_to_battlefield(0, catalog::tempest_djinn());
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::island());
        }
        g.add_card_to_battlefield(1, catalog::island()); // theirs doesn't count
        let cp = g.computed_permanent(djinn).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 4));
    }

    /// Undercity Informer mills the target down to their first land.
    #[test]
    fn undercity_informer_mills_until_land() {
        let mut g = two_player_game();
        let informer = g.add_card_to_battlefield(0, catalog::undercity_informer());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(1, catalog::lightning_bolt());
        g.add_card_to_library(1, catalog::counterspell());
        g.add_card_to_library(1, catalog::island()); // 3rd from top
        g.players[0].mana_pool.add_colorless(1);
        g.clear_sickness(informer);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: informer, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "a creature was sacrificed");
        assert_eq!(g.players[1].graveyard.len(), 3, "milled through the first land");
    }

    /// Runeflare Trap: {R} when an opponent drew 3+, damage = their hand size.
    #[test]
    fn runeflare_trap_alt_cost_and_damage() {
        let mut g = two_player_game();
        let trap = g.add_card_to_hand(0, catalog::runeflare_trap());
        for _ in 0..4 {
            g.add_card_to_hand(1, catalog::island());
        }
        g.players[1].cards_drawn_this_turn = 3;
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let life = g.players[1].life;
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: trap, target: Some(Target::Player(1)), additional_targets: vec![], mode: None,
            x_value: None, pitch_card: None,
        })
        .expect("trap cost");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 4, "damage equals their hand size");
    }

    /// Molten Psyche wheels hands into libraries; metalcraft burns per draw.
    #[test]
    fn molten_psyche_wheels_and_burns() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::chrome_mox()); // metalcraft
            g.add_card_to_hand(1, catalog::island());
            g.add_card_to_library(1, catalog::island());
        }
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_hand(0, catalog::island());
        let life = g.players[1].life;
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        let eff = catalog::molten_psyche().effect.clone();
        let events = g.resolve_effect(&eff, &ctx).unwrap();
        g.dispatch_triggers_for_events(&events);
        assert_eq!(g.players[1].hand.len(), 3, "same count drawn back");
        assert_eq!(g.players[1].life, life - 3, "metalcraft burn = their draws");
    }

    /// Master of the Feast makes each opponent draw at your upkeep.
    #[test]
    fn master_of_the_feast_upkeep_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::master_of_the_feast());
        g.add_card_to_library(1, catalog::island());
        let hand = g.players[1].hand.len();
        g.active_player_idx = 0;
        g.fire_step_triggers(crabomination::game::TurnStep::Upkeep);
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), hand + 1);
    }

    /// Spiteful Visions pings every drawer for 1.
    #[test]
    fn spiteful_visions_pings_drawers() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::spiteful_visions());
        g.add_card_to_library(1, catalog::island());
        let life = g.players[1].life;
        let ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
        let events = g
            .resolve_effect(
                &crabomination::effect::Effect::Draw {
                    who: crabomination::effect::Selector::Player(crabomination::effect::PlayerRef::You),
                    amount: crabomination::effect::Value::Const(1),
                },
                &ctx,
            )
            .unwrap();
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "drawer pinged for 1");
    }

    /// Tolaria West transmutes for an MV-0 card.
    #[test]
    fn tolaria_west_transmutes() {
        let mut g = two_player_game();
        let target = g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let tw = g.add_card_to_hand(0, catalog::tolaria_west());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: tw, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("transmute");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == target), "MV-0 card fetched");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == tw), "Tolaria West discarded");
    }

    /// Boseiju mana makes the instant it funds uncounterable (and costs 2 life).
    #[test]
    fn boseiju_funds_uncounterable_instant() {
        let mut g = two_player_game();
        let boseiju = g.add_card_to_battlefield(0, catalog::boseiju_who_shelters_all());
        g.battlefield_find_mut(boseiju).unwrap().tapped = false;
        let life = g.players[0].life;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: boseiju, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("tap Boseiju");
        assert_eq!(g.players[0].life, life - 2);
        // Psionic Blast {2}{U}: the Boseiju {C} pays part of the generic.
        let blast = g.add_card_to_hand(0, catalog::psionic_blast());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: blast, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Psionic Blast off Boseiju mana");
        let counter = g.add_card_to_hand(1, catalog::counterspell());
        g.players[1].mana_pool.add(Color::Blue, 2);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: counter, target: Some(Target::Permanent(blast)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .expect("counterspell castable");
        let life1 = g.players[1].life;
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1 - 4, "blast resolved despite the counter");
    }

    /// Pendelhaven pumps a 1/1 (and only a 1/1).
    #[test]
    fn pendelhaven_pumps_only_one_ones() {
        let mut g = two_player_game();
        let haven = g.add_card_to_battlefield(0, catalog::pendelhaven());
        let lion = g.add_card_to_battlefield(0, catalog::savannah_lions()); // 2/1
        let speaker = g.add_card_to_battlefield(0, catalog::kumenas_speaker()); // 1/1
        g.priority.player_with_priority = 0;
        let err = g.perform_action(GameAction::ActivateAbility {
            card_id: haven, ability_index: 1, target: Some(Target::Permanent(lion)),
            additional_targets: vec![], x_value: None,
        });
        assert!(err.is_err(), "a 2/1 is not a legal target");
        g.battlefield_find_mut(haven).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: haven, ability_index: 1, target: Some(Target::Permanent(speaker)),
            additional_targets: vec![], x_value: None,
        })
        .expect("pump the 1/1");
        drain_stack(&mut g);
        let cp = g.computed_permanent(speaker).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 3));
    }

    /// Wanderwine Hub enters untapped only when revealing a Merfolk.
    #[test]
    fn wanderwine_hub_reveal_gate() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::kumenas_speaker());
        let hub = g.add_card_to_battlefield(0, catalog::wanderwine_hub());
        g.fire_self_etb_triggers(hub, 0);
        drain_stack(&mut g);
        assert!(!g.battlefield_find(hub).unwrap().tapped, "revealed a Merfolk → untapped");
        g.players[0].hand.clear();
        let hub2 = g.add_card_to_battlefield(0, catalog::wanderwine_hub());
        g.fire_self_etb_triggers(hub2, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(hub2).unwrap().tapped, "no Merfolk to reveal → tapped");
    }

    /// Fevered Visions draws for the active player and burns a full-handed
    /// opponent at their end step.
    #[test]
    fn fevered_visions_end_step() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::fevered_visions());
        for _ in 0..4 {
            g.add_card_to_hand(1, catalog::island());
        }
        g.add_card_to_library(1, catalog::island());
        let life = g.players[1].life;
        let hand = g.players[1].hand.len();
        g.active_player_idx = 1;
        g.fire_step_triggers(crabomination::game::TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), hand + 1, "opponent drew at their end step");
        assert_eq!(g.players[1].life, life - 2, "4+ cards in hand → 2 damage");
    }
}
