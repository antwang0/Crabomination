//! Tests for recentN card batches 268-282 (merged from per-batch micro-files).

mod recent268 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

    /// Aether Channeler's draw mode draws a card.
    #[test]
    fn aether_channeler_modal_draw() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let ac = g.add_card_to_hand(0, catalog::aether_channeler());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        // Choose mode 2 (draw a card).
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(2)]));
        cast(&mut g, ac, None);
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Forest"),
            "drew the library card via modal ETB"
        );
    }

    /// Aggressive Sabotage discards two, and burns for 3 when kicked.
    #[test]
    fn aggressive_sabotage_kicked_burns() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::forest());
        g.add_card_to_hand(1, catalog::island());
        g.add_card_to_hand(1, catalog::mountain());
        let effect = catalog::aggressive_sabotage().effect;
        let mut ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
        ctx.kicked = true;
        let life = g.players[1].life;
        let hand = g.players[1].hand.len();
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[1].hand.len(), hand - 2, "discarded two");
        assert_eq!(g.players[1].life, life - 3, "kicked burn for 3");
    }

    /// Argivian Phalanx's affinity for creatures cuts its cost.
    #[test]
    fn argivian_phalanx_affinity() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ph = g.add_card_to_hand(0, catalog::argivian_phalanx());
        // {5}{W} minus {2} for two creatures = {3}{W}.
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, ph, None);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Argivian Phalanx"),
            "cast for the affinity-reduced cost"
        );
    }

    /// Artillery Blast scales with domain and only hits tapped creatures.
    #[test]
    fn artillery_blast_domain_damage() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::island());
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        g.battlefield_find_mut(victim).unwrap().tapped = true;
        let effect = catalog::artillery_blast().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(victim)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        // 1 + 2 basic land types = 3 damage → the 2/2 dies.
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "took 3, died");
    }

    /// Automatic Librarian scries on ETB.
    #[test]
    fn automatic_librarian_scries() {
        let mut g = two_player_game();
        let lib = g.add_card_to_hand(0, catalog::automatic_librarian());
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add_colorless(3);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
            kept_top: vec![],
            bottom: vec![],
        }]));
        cast(&mut g, lib, None);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Automatic Librarian"));
    }

    /// Antagonize pumps +4/+3.
    #[test]
    fn antagonize_pumps() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::antagonize().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (6, 5));
    }

    /// Attended Socialite grows when another creature enters.
    #[test]
    fn attended_socialite_alliance() {
        let mut g = two_player_game();
        let soc = g.add_card_to_battlefield(0, catalog::attended_socialite());
        let other = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, other, None);
        let cp = g.computed_permanent(soc).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 2), "socialite grew +1/+1");
    }

    /// Backup Agent puts a +1/+1 counter on a creature on ETB.
    #[test]
    fn backup_agent_counters() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let agent = g.add_card_to_hand(0, catalog::backup_agent());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, agent, Some(Target::Permanent(bear)));
        assert_eq!(
            g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1
        );
    }

    /// Armor of Shadows grants indestructible and +1/+0.
    #[test]
    fn armor_of_shadows_grants_indestructible() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::armor_of_shadows().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 3, "+1/+0");
        assert!(cp.keywords.contains(&Keyword::Indestructible));
    }

    /// Arms of Hadar shrinks all of a player's creatures.
    #[test]
    fn arms_of_hadar_mass_shrink() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let effect = catalog::arms_of_hadar().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Player(1)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert!(g.battlefield_find(a).is_none(), "shrank to 0/0 and died");
        assert!(g.battlefield_find(b).is_none(), "shrank to 0/0 and died");
    }

    /// A Little Chat digs two deep for a card.
    #[test]
    fn a_little_chat_digs() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::island());
        let effect = catalog::a_little_chat().effect;
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let hand = g.players[0].hand.len();
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 1, "one card to hand");
    }
}

mod recent269 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
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

    /// Gilded Scuttler taps and stuns an opponent's creature on ETB.
    #[test]
    fn gilded_scuttler_taps_and_stuns() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let scut = g.add_card_to_hand(0, catalog::gilded_scuttler());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast(&mut g, scut, Some(Target::Permanent(victim)));
        let v = g.battlefield_find(victim).unwrap();
        assert!(v.tapped, "tapped");
        assert_eq!(v.counter_count(CounterType::Stun), 1, "stunned");
        assert!(
            g.battlefield_find(scut).unwrap().has_keyword(&Keyword::Unblockable),
            "unblockable"
        );
    }

    /// Go Forth can tutor a basic land to hand.
    #[test]
    fn go_forth_tutors_basic() {
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        let effect = catalog::go_forth().effect;
        // Resolving the effect directly runs mode 0 (the tutor); ctx.mode defaults to 0.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Forest"),
            "basic land tutored to hand"
        );
    }

    /// Hearts on Fire can pump two creatures.
    #[test]
    fn hearts_on_fire_pumps_two() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::hearts_on_fire().effect;
        let ctx = EffectContext {
            targets: vec![Target::Permanent(a), Target::Permanent(b)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.computed_permanent(a).unwrap().power, 4, "+2/+1");
        assert_eq!(g.computed_permanent(b).unwrap().power, 4, "+2/+1");
    }

    /// Hungry Megasloth grows itself with its mana ability.
    #[test]
    fn hungry_megasloth_grows() {
        let mut g = two_player_game();
        let sloth = g.add_card_to_battlefield(0, catalog::hungry_megasloth());
        g.clear_sickness(sloth);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: sloth,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .unwrap();
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(sloth).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1
        );
    }

    /// Phantasmal Shieldback sacrifices itself when targeted, then draws.
    #[test]
    fn phantasmal_shieldback_sacs_when_targeted() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let shield = g.add_card_to_battlefield(0, catalog::phantasmal_shieldback());
        // Target it with a pump spell → it sacrifices itself, then draws.
        let bolt = g.add_card_to_hand(0, catalog::antagonize());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        let hand_before = g.players[0].hand.len();
        cast(&mut g, bolt, Some(Target::Permanent(shield)));
        assert!(g.battlefield_find(shield).is_none(), "sacrificed itself on being targeted");
        // Bolt left hand (-1) and the death draw added one (+1) → net unchanged.
        assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1, "drew on death");
    }

    /// Battlefield Butcher's activation cost drops {1} per creature card in the
    /// graveyard (new `cost_reduction_per_graveyard` primitive).
    #[test]
    fn battlefield_butcher_graveyard_discount() {
        let mut g = two_player_game();
        let butcher = g.add_card_to_battlefield(0, catalog::battlefield_butcher());
        g.clear_sickness(butcher);
        // Two creature cards + a noncreature in the graveyard → {5} - {2} = {3}.
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add_colorless(3);
        let life = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: butcher,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activated for the reduced {3}");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2, "opponent lost 2 life");
    }

    /// Razorgrass Invoker pumps itself and one other creature.
    #[test]
    fn razorgrass_invoker_pumps_pair() {
        let mut g = two_player_game();
        let inv = g.add_card_to_battlefield(0, catalog::razorgrass_invoker());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(inv);
        g.players[0].mana_pool.add_colorless(8);
        g.perform_action(GameAction::ActivateAbility {
            card_id: inv,
            ability_index: 0,
            target: Some(Target::Permanent(ally)),
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .unwrap();
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(inv).unwrap().power, 7, "self +3/+3");
        assert_eq!(g.computed_permanent(ally).unwrap().power, 5, "ally +3/+3");
    }
}

mod recent270 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target};
    use crabomination::game::{drain_stack, two_player_game};

    /// Black Market Tycoon mints a Treasure, and its upkeep bites for 2 per Treasure.
    #[test]
    fn black_market_tycoon_treasure_and_bite() {
        let mut g = two_player_game();
        let tycoon = g.add_card_to_battlefield(0, catalog::black_market_tycoon());
        g.clear_sickness(tycoon);
        // Tap for a Treasure.
        g.perform_action(GameAction::ActivateAbility {
            card_id: tycoon,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .unwrap();
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
            "made a Treasure"
        );
        // The upkeep trigger deals 2 per Treasure (one here → 2 damage).
        let effect = catalog::black_market_tycoon().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(tycoon, 0, None);
        let life = g.players[0].life;
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[0].life, life - 2, "2 damage for one Treasure");
    }

    /// Balduvian Atrocity reanimates a small creature only when kicked.
    #[test]
    fn balduvian_atrocity_kicked_reanimates() {
        let mut g = two_player_game();
        // A bear in the graveyard to reanimate.
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let effect = catalog::balduvian_atrocity().triggered_abilities[0].effect.clone();
        // Unkicked: nothing returns.
        let ctx = EffectContext::for_ability(
            crabomination::game::CardId(999),
            0,
            None,
        );
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(
            !g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "unkicked: no reanimation"
        );
        // Kicked: the bear returns with haste.
        let target = g.players[0].graveyard.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
        let mut kctx = EffectContext::for_ability(crabomination::game::CardId(999), 0, Some(Target::Permanent(target)));
        kctx.kicked = true;
        g.resolve_effect(&effect, &kctx).unwrap();
        drain_stack(&mut g);
        let bear = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("reanimated");
        assert!(bear.has_keyword(&crabomination::card::Keyword::Haste), "gained haste");
    }
}

mod recent271 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{GameAction, Target};
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Body Dropper's sac ability grants menace and its own sacrifice trigger
    /// grows it with a +1/+1 counter.
    #[test]
    fn body_dropper_sac_ability_grows_and_menaces() {
        let mut g = two_player_game();
        let dropper = g.add_card_to_battlefield(0, catalog::body_dropper());
        g.clear_sickness(dropper);
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // fodder to sacrifice
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dropper,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .expect("activate sac-menace");
        drain_stack(&mut g);
        let cp = g.computed_permanent(dropper).unwrap();
        assert!(cp.keywords.contains(&Keyword::Menace), "gained menace");
        assert_eq!(
            g.battlefield_find(dropper).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "the sacrifice grew it"
        );
    }

    /// Boon of Safety shields a creature.
    #[test]
    fn boon_of_safety_shields() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
            kept_top: vec![],
            bottom: vec![],
        }]));
        let effect = catalog::boon_of_safety().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Shield), 1);
    }

    /// Brokers Initiate becomes a 5/5 with its hybrid ability.
    #[test]
    fn brokers_initiate_becomes_5_5() {
        let mut g = two_player_game();
        let init = g.add_card_to_battlefield(0, catalog::brokers_initiate());
        g.clear_sickness(init);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::ActivateAbility {
            card_id: init,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .unwrap();
        drain_stack(&mut g);
        let cp = g.computed_permanent(init).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5));
    }

    /// Brokers Veteran shields a creature you control when it dies.
    #[test]
    fn brokers_veteran_death_shield() {
        let mut g = two_player_game();
        let vet = g.add_card_to_battlefield(0, catalog::brokers_veteran());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(vet).unwrap().damage = 100;
        let evs = g.check_state_based_actions();
        // Target the ally with the death trigger.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(
            Target::Permanent(ally),
        )]));
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::Shield), 1);
    }

    /// Battle-Rage Blessing grants deathtouch and indestructible.
    #[test]
    fn battle_rage_blessing_grants_both() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let effect = catalog::battle_rage_blessing().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        let cp = g.computed_permanent(bear).unwrap();
        assert!(cp.keywords.contains(&Keyword::Deathtouch));
        assert!(cp.keywords.contains(&Keyword::Indestructible));
    }

    /// Benalish Sleeper's kicked ETB forces an edict on each player.
    #[test]
    fn benalish_sleeper_kicked_edict() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let effect = catalog::benalish_sleeper().triggered_abilities[0].effect.clone();
        let mut ctx = EffectContext::for_ability(mine, 0, None);
        ctx.kicked = true;
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none(), "you sacrificed a creature");
        assert!(g.battlefield_find(theirs).is_none(), "opponent sacrificed a creature");
    }

    /// Argivian Avenger shrinks and grants a chosen keyword.
    #[test]
    fn argivian_avenger_modal_grant() {
        let mut g = two_player_game();
        let av = g.add_card_to_battlefield(0, catalog::argivian_avenger());
        g.clear_sickness(av);
        g.players[0].mana_pool.add_colorless(1);
        // Choose mode 0 (flying).
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
        g.perform_action(GameAction::ActivateAbility {
            card_id: av,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .unwrap();
        drain_stack(&mut g);
        let cp = g.computed_permanent(av).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "-1/-1");
        assert!(cp.keywords.contains(&Keyword::Flying), "gained the chosen keyword");
    }
}

mod recent272 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::Target;
    use crabomination::game::{drain_stack, two_player_game};

    /// Ambitious Assault pumps the team and draws when a modified creature is out.
    #[test]
    fn ambitious_assault_draws_when_modified() {
        // No modified creature → no draw.
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        let effect = catalog::ambitious_assault().effect;
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let hand = g.players[0].hand.len();
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "team +2/+0");
        assert_eq!(g.players[0].hand.len(), hand, "no modified creature → no draw");

        // Put a +1/+1 counter on the bear (a modification) → the draw happens.
        g.battlefield_find_mut(bear)
            .unwrap()
            .add_counters(crabomination::card::CounterType::PlusOnePlusOne, 1);
        let hand = g.players[0].hand.len();
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 1, "modified creature → draw");
    }

    /// Revenge of the Drowned tucks a creature and makes a decayed Zombie.
    #[test]
    fn revenge_of_the_drowned_tucks_and_spawns() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let lib_before = g.players[1].library.len();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // top
        let effect = catalog::revenge_of_the_drowned().effect;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(victim)), 0, 0);
        g.resolve_effect(&effect, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "creature tucked away");
        assert_eq!(g.players[1].library.len(), lib_before + 1, "returned to its owner's library");
        let zombie = g
            .battlefield
            .iter()
            .find(|c| c.controller == 0 && c.definition.name == "Zombie")
            .expect("made a Zombie");
        assert!(zombie.definition.keywords.contains(&crabomination::card::Keyword::Decayed));
    }
}

mod recent273 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{drain_stack, two_player_game};

    /// Academy Wall loots when you cast an instant or sorcery.
    #[test]
    fn academy_wall_loots_on_spell() {
        let mut g = two_player_game();
        let wall = g.add_card_to_battlefield(0, catalog::academy_wall());
        g.add_card_to_library(0, catalog::forest());
        let discard = g.add_card_to_hand(0, catalog::island()); // something to discard
        // Accept the "may draw then discard" and bin the Island.
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Discard(vec![discard]),
        ]));
        // Fire the trigger effect directly (a cast would drain the stack too).
        let effect = catalog::academy_wall().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(wall, 0, None);
        let hand = g.players[0].hand.len();
        g.resolve_effect(&effect, &ctx).unwrap();
        // +1 drawn, -1 discarded → net unchanged, but the loot happened.
        assert_eq!(g.players[0].hand.len(), hand, "drew then discarded");
        assert_eq!(g.players[0].graveyard.len(), 1, "one card discarded");
    }

    /// Battlewing Mystic wheels only when kicked.
    #[test]
    fn battlewing_mystic_kicked_wheels() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::forest());
        }
        g.add_card_to_hand(0, catalog::island());
        g.add_card_to_hand(0, catalog::mountain());
        let effect = catalog::battlewing_mystic().triggered_abilities[0].effect.clone();
        let mut ctx = EffectContext::for_ability(crabomination::game::CardId(500), 0, None);
        ctx.kicked = true;
        g.resolve_effect(&effect, &ctx).unwrap();
        // Discarded the two-card hand, drew two fresh.
        assert_eq!(g.players[0].hand.len(), 2, "new hand of two");
        assert!(
            g.players[0].hand.iter().all(|c| c.definition.name == "Forest"),
            "the wheel replaced the old hand"
        );
    }

    /// Brazen Upstart digs for a creature when it dies.
    #[test]
    fn brazen_upstart_death_dig() {
        let mut g = two_player_game();
        let up = g.add_card_to_battlefield(0, catalog::brazen_upstart());
        let bear = g.add_card_to_library(0, catalog::grizzly_bears());
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::forest());
        }
        g.battlefield_find_mut(up).unwrap().damage = 100;
        let evs = g.check_state_based_actions();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(
            g.players[0].hand.iter().any(|c| c.id == bear),
            "revealed a creature to hand"
        );
    }
}

mod recent274 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::two_player_game;

    /// Emergent Haunting flips into a 3/3 flying Spirit when the end-step trigger
    /// resolves (it starts as a noncreature enchantment, so the gate passes).
    #[test]
    fn emergent_haunting_self_animates() {
        let mut g = two_player_game();
        let e = g.add_card_to_battlefield(0, catalog::emergent_haunting());
        let effect = catalog::emergent_haunting().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(e, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        let p = g.computed_permanent(e).unwrap();
        assert!(p.card_types.contains(&crabomination::card::CardType::Creature), "now a creature");
        assert_eq!((p.power, p.toughness), (3, 3), "3/3 body");
        assert!(p.keywords.contains(&crabomination::card::Keyword::Flying), "gains flying");
    }

    /// Jolene's attack trigger only fires when a power-4+ attacker is declared, and
    /// mints a Treasure when it does.
    #[test]
    fn jolene_makes_treasure_on_beefy_attack() {
        let mut g = two_player_game();
        let jolene = g.add_card_to_battlefield(0, catalog::jolene_plundering_pugilist());
        let effect = catalog::jolene_plundering_pugilist().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(jolene, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
            "a Treasure token entered"
        );
    }

    /// Jolene's sacrifice-a-Treasure ping targets any target.
    #[test]
    fn jolene_ping_ability_costs_a_treasure() {
        let def = catalog::jolene_plundering_pugilist();
        let ab = &def.activated_abilities[0];
        assert!(ab.sac_other_filter.is_some(), "requires sacrificing a Treasure");
    }
}

mod recent275 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{drain_stack, two_player_game};

    /// Stargaze at X=2 digs 4, banks 2, bins 2, and costs 2 life.
    #[test]
    fn stargaze_digs_and_pays_life() {
        let mut g = two_player_game();
        let mut ids = vec![];
        for _ in 0..4 {
            ids.push(g.add_card_to_library(0, catalog::grizzly_bears()));
        }
        let life = g.players[0].life;
        // Pick the first two of the four looked at.
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(ids[..2].to_vec())]));
        let ctx = EffectContext { x_value: 2, ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::stargaze().effect.clone(), &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), 2, "banked X=2 cards");
        assert_eq!(g.players[0].graveyard.len(), 2, "binned the other two");
        assert_eq!(g.players[0].life, life - 2, "lost X life");
    }

    /// Axgard Artisan mints a Treasure the first time counters land on it each turn.
    #[test]
    fn axgard_artisan_counter_makes_treasure() {
        let mut g = two_player_game();
        let ax = g.add_card_to_battlefield(0, catalog::axgard_artisan());
        let effect = catalog::axgard_artisan().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(ax, 0, None);
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "a Treasure entered");
    }

    /// Bloated Processor incubates its power when it dies.
    #[test]
    fn bloated_processor_death_incubates() {
        let mut g = two_player_game();
        let bp = g.add_card_to_battlefield(0, catalog::bloated_processor());
        g.battlefield_find_mut(bp).unwrap().damage = 100;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name.contains("Incubator")),
            "an Incubator token was created"
        );
    }

    /// Harvestrite Host draws only on the second resolution in a turn.
    #[test]
    fn harvestrite_host_second_resolution_draws() {
        let mut g = two_player_game();
        let host = g.add_card_to_battlefield(0, catalog::harvestrite_host());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let effect = catalog::harvestrite_host().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![crabomination::game::Target::Permanent(host)], ..EffectContext::for_ability(host, 0, None) };
        let start = g.players[0].hand.len();
        g.resolve_effect(&effect, &ctx).unwrap(); // 1st: pump only
        assert_eq!(g.players[0].hand.len(), start, "no draw on the first resolution");
        g.resolve_effect(&effect, &ctx).unwrap(); // 2nd: pump + draw
        assert_eq!(g.players[0].hand.len(), start + 1, "draws on the second resolution");
    }
}

mod recent276 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{drain_stack, two_player_game, Target};

    /// Converter Beast incubates 5 on entry.
    #[test]
    fn converter_beast_incubates() {
        let mut g = two_player_game();
        let cb = g.add_card_to_battlefield(0, catalog::converter_beast());
        let effect = catalog::converter_beast().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_ability(cb, 0, None)).unwrap();
        let inc = g.battlefield.iter().find(|c| c.definition.name.contains("Incubator")).unwrap();
        assert_eq!(inc.counters.get(&crabomination::card::CounterType::PlusOnePlusOne).copied().unwrap_or(0), 5);
    }

    /// Carrion Locust drains 1 when the exiled graveyard card is a creature.
    #[test]
    fn carrion_locust_drains_on_creature() {
        let mut g = two_player_game();
        let cl = g.add_card_to_battlefield(0, catalog::carrion_locust());
        let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let life = g.players[1].life;
        let effect = catalog::carrion_locust().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(dead)], ..EffectContext::for_ability(cl, 0, None) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(g.exile.iter().any(|c| c.id == dead), "the creature card is exiled");
        assert_eq!(g.players[1].life, life - 1, "owner lost 1 life for a creature card");
    }

    /// Coastal Bulwark grows to 3/3 while its controller has an Island.
    #[test]
    fn coastal_bulwark_islandwalk_pump() {
        let mut g = two_player_game();
        let cb = g.add_card_to_battlefield(0, catalog::coastal_bulwark());
        assert_eq!(g.computed_permanent(cb).unwrap().power, 1, "1/3 with no Island");
        g.add_card_to_battlefield(0, catalog::island());
        assert_eq!(g.computed_permanent(cb).unwrap().power, 3, "+2/+0 with an Island");
    }

    /// Emergency Weld returns a creature card and mints a Soldier.
    #[test]
    fn emergency_weld_returns_and_makes_token() {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::emergency_weld().effect.clone(), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear), "returned to hand");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Soldier"), "made a Soldier");
    }

    /// Burning Sun's Fury has convoke and pumps up to two creatures with haste.
    #[test]
    fn burning_suns_fury_pumps() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(a)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::burning_suns_fury().effect.clone(), &ctx).unwrap();
        let p = g.computed_permanent(a).unwrap();
        assert_eq!(p.power, 4, "+2/+0");
        assert!(p.keywords.contains(&crabomination::card::Keyword::Haste), "gains haste");
        assert!(catalog::burning_suns_fury().keywords.contains(&crabomination::card::Keyword::Convoke));
    }
}

mod recent277 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{two_player_game, Target};

    /// Calamity's Wake empties graveyards, locks noncreature casting, and exiles
    /// itself.
    #[test]
    fn calamitys_wake_nukes_graveyards_and_locks() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.resolve_effect(&catalog::calamitys_wake().effect.clone(), &EffectContext::for_spell(0, None, 0, 0))
            .unwrap();
        assert!(g.players[0].graveyard.is_empty() && g.players[1].graveyard.is_empty(), "all graveyards exiled");
        assert!(g.players[0].cant_cast_noncreature_this_turn, "you locked");
        assert!(g.players[1].cant_cast_noncreature_this_turn, "opponent locked");
    }

    /// Attentive Skywarden's combat-damage trigger flips an Incubator token.
    #[test]
    fn attentive_skywarden_flips_incubator() {
        let mut g = two_player_game();
        // Give the controller an Incubator token via Converter Beast's incubate.
        let cb = g.add_card_to_battlefield(0, catalog::converter_beast());
        let inc_effect = catalog::converter_beast().triggered_abilities[0].effect.clone();
        g.resolve_effect(&inc_effect, &EffectContext::for_ability(cb, 0, None)).unwrap();
        let inc = g.battlefield.iter().find(|c| c.definition.name.contains("Incubator")).unwrap().id;
        let warden = g.add_card_to_battlefield(0, catalog::attentive_skywarden());
        let effect = catalog::attentive_skywarden().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(inc)], ..EffectContext::for_ability(warden, 0, None) };
        g.resolve_effect(&effect, &ctx).unwrap();
        // The Incubator transformed into its Phyrexian creature back face.
        let flipped = g.battlefield_find(inc).unwrap();
        assert!(flipped.definition.card_types.contains(&crabomination::card::CardType::Creature), "now a creature");
    }

    /// Molten Collapse is descend-gated: one mode by default, up to both when the
    /// caster descended this turn.
    #[test]
    fn molten_collapse_descend_widens_to_both() {
        use crabomination::effect::Effect;
        let def = catalog::molten_collapse();
        let Effect::If { cond, then, else_ } = &def.effect else { panic!("expected a descend-gated modal") };
        assert!(
            matches!(cond, crabomination::card::Predicate::DescendedThisTurn { .. }),
            "gated on descend",
        );
        assert!(matches!(**else_, Effect::ChooseModesCast { min: 1, max: 1, .. }), "one mode by default");
        let Effect::ChooseModesCast { modes, min: 1, max: 2, .. } = &**then else {
            panic!("both modes available once descended")
        };
        assert_eq!(modes.len(), 2, "the two printed destroy modes");

        // The default (non-descended) branch destroys a targeted creature.
        let mut g = two_player_game();
        let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(creature)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&modes[0].clone(), &ctx).unwrap();
        assert!(!g.battlefield.iter().any(|c| c.id == creature), "creature destroyed by mode 0");
    }
}

mod recent278 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{drain_stack, two_player_game, Target};

    /// Bog Badger grants your team menace only when kicked.
    #[test]
    fn bog_badger_kicked_grants_menace() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let badger = g.add_card_to_battlefield(0, catalog::bog_badger());
        let effect = catalog::bog_badger().triggered_abilities[0].effect.clone();
        let mut ctx = EffectContext::for_ability(badger, 0, None);
        ctx.kicked = true;
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::Menace),
            "team gained menace when kicked",
        );
    }

    /// Colossal Growth is +3/+3 unkicked and +4/+4 with trample/haste kicked.
    #[test]
    fn colossal_growth_kicker_scales() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::colossal_growth().effect.clone(), &ctx).unwrap();
        assert_eq!(g.computed_permanent(bear).unwrap().power, 5, "unkicked +3/+3");

        let mut g2 = two_player_game();
        let bear2 = g2.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut kctx = EffectContext { targets: vec![Target::Permanent(bear2)], ..EffectContext::for_spell(0, None, 0, 0) };
        kctx.kicked = true;
        g2.resolve_effect(&catalog::colossal_growth().effect.clone(), &kctx).unwrap();
        let p = g2.computed_permanent(bear2).unwrap();
        assert_eq!(p.power, 6, "kicked +4/+4");
        assert!(p.keywords.contains(&crabomination::card::Keyword::Trample), "kicked grants trample");
    }

    /// Civic Gardener untaps a target on attack.
    #[test]
    fn civic_gardener_untaps_on_attack() {
        let mut g = two_player_game();
        let cg = g.add_card_to_battlefield(0, catalog::civic_gardener());
        let land = g.add_card_to_battlefield(0, catalog::forest());
        g.battlefield_find_mut(land).unwrap().tapped = true;
        let effect = catalog::civic_gardener().triggered_abilities[0].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(land)], ..EffectContext::for_ability(cg, 0, None) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert!(!g.battlefield_find(land).unwrap().tapped, "the land untapped");
    }

    /// Celebrity Fencer grows when another creature enters.
    #[test]
    fn celebrity_fencer_alliance_grows() {
        let mut g = two_player_game();
        let fencer = g.add_card_to_battlefield(0, catalog::celebrity_fencer());
        let effect = catalog::celebrity_fencer().triggered_abilities[0].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_ability(fencer, 0, None)).unwrap();
        assert_eq!(g.computed_permanent(fencer).unwrap().power, 4, "3/2 → 4/3");
    }

    /// Commune with Spirits digs an enchantment or land into hand.
    #[test]
    fn commune_with_spirits_digs() {
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Cards(vec![forest]),
        ]));
        g.resolve_effect(&catalog::commune_with_spirits().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        assert!(g.players[0].hand.iter().any(|c| c.id == forest), "the land went to hand");
    }

    /// Buy Your Silence exiles a nonland permanent and gives its controller a Treasure.
    #[test]
    fn buy_your_silence_exiles_and_compensates() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::buy_your_silence().effect.clone(), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == bear), "the creature is exiled");
        assert!(
            g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Treasure"),
            "its controller got a Treasure",
        );
    }

    /// Case the Joint draws two.
    #[test]
    fn case_the_joint_draws_two() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        for _ in 0..3 {
            g.add_card_to_library(1, catalog::forest());
        }
        let hand = g.players[0].hand.len();
        g.resolve_effect(&catalog::case_the_joint().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 2, "drew two");
    }
}

mod recent279 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{two_player_game, Target};

    /// Sunrise Cavalier's day/night flip grows a creature.
    #[test]
    fn sunrise_cavalier_flip_adds_counter() {
        let mut g = two_player_game();
        let cav = g.add_card_to_battlefield(0, catalog::sunrise_cavalier());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // The second triggered ability is the day/night payoff.
        let effect = catalog::sunrise_cavalier().triggered_abilities[1].effect.clone();
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_ability(cav, 0, None) };
        g.resolve_effect(&effect, &ctx).unwrap();
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "2/2 → 3/3");
    }

    /// Celestus Sanctifier bins one of the top two on a day/night flip.
    #[test]
    fn celestus_sanctifier_flip_mills_choice() {
        let mut g = two_player_game();
        let cel = g.add_card_to_battlefield(0, catalog::celestus_sanctifier());
        let keep = g.add_card_to_library(0, catalog::forest());
        let bin = g.add_card_to_library(0, catalog::island());
        let effect = catalog::celestus_sanctifier().triggered_abilities[1].effect.clone();
        g.resolve_effect(&effect, &EffectContext::for_ability(cel, 0, None)).unwrap();
        // Exactly one of the top two is binned; the other stays in the library.
        let two = [keep, bin];
        assert_eq!(g.players[0].graveyard.len(), 1, "one of the two binned");
        let binned = two.iter().filter(|id| g.players[0].graveyard.iter().any(|c| c.id == **id)).count();
        let kept = two.iter().filter(|id| g.players[0].library.iter().any(|c| c.id == **id)).count();
        assert_eq!((binned, kept), (1, 1), "one binned, one kept on top");
    }

    /// Cartographer's Survey ramps up to two lands tapped.
    #[test]
    fn cartographers_survey_ramps_lands() {
        let mut g = two_player_game();
        for _ in 0..2 {
            g.add_card_to_library(0, catalog::forest());
        }
        for _ in 0..5 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let bf = g.battlefield.len();
        g.resolve_effect(&catalog::cartographers_survey().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        let lands_in = g.battlefield.len() - bf;
        assert!((1..=2).contains(&lands_in), "put up to two lands onto the battlefield");
        assert!(g.battlefield.iter().filter(|c| c.definition.name == "Forest").all(|c| c.tapped), "entered tapped");
    }

    /// Markov Retribution's team-pump mode buffs your board.
    #[test]
    fn markov_retribution_team_pump_mode() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Resolve mode 0 (team +1/+0) directly.
        let crabomination::effect::Effect::ChooseModesCast { modes, .. } = &catalog::markov_retribution().effect
        else {
            panic!("modal");
        };
        g.resolve_effect(&modes[0].clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "2/2 → 3/2");
    }
}

mod recent280 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{drain_stack, two_player_game, Target};

    /// Brandywine Farmer makes a Food on entry and another on leaving.
    #[test]
    fn brandywine_farmer_food_on_etb_and_ltb() {
        let mut g = two_player_game();
        let bf = g.add_card_to_battlefield(0, catalog::brandywine_farmer());
        g.resolve_effect(&catalog::brandywine_farmer().triggered_abilities[0].effect.clone(), &EffectContext::for_ability(bf, 0, None)).unwrap();
        g.resolve_effect(&catalog::brandywine_farmer().triggered_abilities[1].effect.clone(), &EffectContext::for_ability(bf, 0, None)).unwrap();
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Food").count(), 2, "two Food tokens");
    }

    /// Captain of Umbar loots.
    #[test]
    fn captain_of_umbar_loots() {
        let mut g = two_player_game();
        let cap = g.add_card_to_battlefield(0, catalog::captain_of_umbar());
        g.add_card_to_library(0, catalog::forest());
        let dump = g.add_card_to_hand(0, catalog::island());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Discard(vec![dump]),
        ]));
        let hand = g.players[0].hand.len();
        g.resolve_effect(&catalog::captain_of_umbar().activated_abilities[0].effect.clone(), &EffectContext::for_ability(cap, 0, None)).unwrap();
        assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
        assert_eq!(g.players[0].graveyard.len(), 1, "one card discarded");
    }

    /// Chance-Met Elves grows when its controller scries.
    #[test]
    fn chance_met_elves_grows_on_scry() {
        let mut g = two_player_game();
        let e = g.add_card_to_battlefield(0, catalog::chance_met_elves());
        g.resolve_effect(&catalog::chance_met_elves().triggered_abilities[0].effect.clone(), &EffectContext::for_ability(e, 0, None)).unwrap();
        assert_eq!(g.computed_permanent(e).unwrap().power, 4, "3/2 → 4/3");
    }

    /// Claim the Precious destroys and tempts the Ring.
    #[test]
    fn claim_the_precious_destroys_and_tempts() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(victim)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::claim_the_precious().effect.clone(), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == victim), "creature destroyed");
        assert!(g.players[0].ring_temptations > 0, "the Ring tempted");
    }

    /// Dreadful as the Storm sets base P/T to 5/5.
    #[test]
    fn dreadful_as_the_storm_sets_base_pt() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::dreadful_as_the_storm().effect.clone(), &ctx).unwrap();
        let p = g.computed_permanent(bear).unwrap();
        assert_eq!((p.power, p.toughness), (5, 5), "base P/T becomes 5/5");
    }

    /// Cirith Ungol Patrol sacrifices a creature to draw and make Food.
    #[test]
    fn cirith_ungol_patrol_sac_draws_and_makes_food() {
        let mut g = two_player_game();
        let patrol = g.add_card_to_battlefield(0, catalog::cirith_ungol_patrol());
        g.add_card_to_library(0, catalog::forest());
        let hand = g.players[0].hand.len();
        g.resolve_effect(&catalog::cirith_ungol_patrol().activated_abilities[0].effect.clone(), &EffectContext::for_ability(patrol, 0, None)).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"), "made a Food");
    }

    /// Breaking of the Fellowship turns an opponent's creature on another.
    #[test]
    fn breaking_of_the_fellowship_forces_a_fight() {
        let mut g = two_player_game();
        let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let ctx = EffectContext { targets: vec![Target::Permanent(attacker), Target::Permanent(victim)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::breaking_of_the_fellowship().effect.clone(), &ctx).unwrap();
        g.check_state_based_actions();
        assert!(!g.battlefield.iter().any(|c| c.id == victim), "victim took 2 and died");
        assert!(g.players[0].ring_temptations > 0, "the Ring tempted");
    }

    /// Deceive the Messenger weakens a creature and amasses Orcs.
    #[test]
    fn deceive_the_messenger_debuffs_and_amasses() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::deceive_the_messenger().effect.clone(), &ctx).unwrap();
        assert_eq!(g.computed_permanent(bear).unwrap().power, -1, "2/2 → -3/-0 leaves power -1");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Army)), "an Orc Army exists");
    }
}

mod recent281 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{drain_stack, two_player_game, Target};

    /// Enraged Huorn tempts the Ring on entry.
    #[test]
    fn enraged_huorn_tempts_the_ring() {
        let mut g = two_player_game();
        let h = g.add_card_to_battlefield(0, catalog::enraged_huorn());
        g.resolve_effect(&catalog::enraged_huorn().triggered_abilities[0].effect.clone(), &EffectContext::for_ability(h, 0, None)).unwrap();
        assert!(g.players[0].ring_temptations > 0, "the Ring tempted");
    }

    /// Ithilien Kingfisher cantrips on death.
    #[test]
    fn ithilien_kingfisher_death_draws() {
        let mut g = two_player_game();
        let k = g.add_card_to_battlefield(0, catalog::ithilien_kingfisher());
        g.add_card_to_library(0, catalog::forest());
        let hand = g.players[0].hand.len();
        g.battlefield_find_mut(k).unwrap().damage = 100;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew on death");
    }

    /// Escape from Orthanc pumps toughness, grants flying, and untaps.
    #[test]
    fn escape_from_orthanc_pumps_and_untaps() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::escape_from_orthanc().effect.clone(), &ctx).unwrap();
        let p = g.computed_permanent(bear).unwrap();
        assert_eq!((p.power, p.toughness), (3, 5), "+1/+3");
        assert!(p.keywords.contains(&crabomination::card::Keyword::Flying), "gains flying");
        assert!(!g.battlefield_find(bear).unwrap().tapped, "untapped");
    }

    /// Gimli's Fury grants trample only to a legendary target.
    #[test]
    fn gimlis_fury_trample_for_legends_only() {
        // Nonlegendary: no trample.
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::gimlis_fury().effect.clone(), &ctx).unwrap();
        let p = g.computed_permanent(bear).unwrap();
        assert_eq!(p.power, 5, "+3/+2");
        assert!(!p.keywords.contains(&crabomination::card::Keyword::Trample), "no trample for a nonlegend");
    }

    /// East-Mark Cavalier destroys the Goblin/Orc it damages in combat, and its
    /// trigger is gated to those types.
    #[test]
    fn east_mark_cavalier_slays_orcs() {
        let def = catalog::east_mark_cavalier();
        // Event is filtered to a Goblin or Orc trigger target.
        assert!(
            def.triggered_abilities[0].event.filter.is_some(),
            "the destroy is gated on the damaged creature being a Goblin or Orc",
        );
        // The destroy body kills the damaged creature (an Orc Soldier).
        let mut g = two_player_game();
        let cav = g.add_card_to_battlefield(0, catalog::east_mark_cavalier());
        let orc = g.add_card_to_battlefield(1, catalog::cirith_ungol_patrol()); // Orc Soldier
        assert!(catalog::cirith_ungol_patrol().subtypes.creature_types.contains(&crabomination::card::CreatureType::Orc));
        let ctx = EffectContext { targets: vec![Target::Permanent(orc)], ..EffectContext::for_ability(cav, 0, None) };
        g.resolve_effect(&def.triggered_abilities[0].effect.clone(), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == orc), "the damaged Orc is destroyed");
    }
}

mod recent282 {
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{two_player_game, Target};

    /// Elven Farsight draws when the top card (post-scry) is a creature.
    #[test]
    fn elven_farsight_reveals_and_draws_creature() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::grizzly_bears()); // top after scry
        let hand = g.players[0].hand.len();
        g.resolve_effect(&catalog::elven_farsight().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        assert_eq!(g.players[0].hand.len(), hand + 1, "revealed a creature and drew");
    }

    /// Eagle of Deliverance grants indestructibility and cantrips on a small target.
    #[test]
    fn eagle_of_deliverance_shields_and_draws() {
        let mut g = two_player_game();
        let eagle = g.add_card_to_battlefield(0, catalog::eagle_of_deliverance());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → power ≤ 2
        g.add_card_to_library(0, catalog::forest()); // something to draw
        let hand = g.players[0].hand.len();
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_ability(eagle, 0, None) };
        g.resolve_effect(&catalog::eagle_of_deliverance().triggered_abilities[0].effect.clone(), &ctx).unwrap();
        assert_eq!(
            g.battlefield_find(bear).unwrap().counters.get(&crabomination::card::CounterType::Indestructible).copied().unwrap_or(0),
            1,
            "indestructible counter placed",
        );
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew for a power-2 target");
    }

    /// Horses of the Bruinen bounces two creatures and tempts the Ring.
    #[test]
    fn horses_of_the_bruinen_bounces_and_tempts() {
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(a), Target::Permanent(b)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::horses_of_the_bruinen().effect.clone(), &ctx).unwrap();
        assert!(!g.battlefield.iter().any(|c| c.id == a || c.id == b), "both creatures bounced");
        assert_eq!(g.players[1].hand.len(), 2, "returned to owner's hand");
        assert!(g.players[0].ring_temptations > 0, "the Ring tempted");
    }
}
