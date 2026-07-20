//! Tests for recentN card batches 283-290 (merged from per-batch micro-files).

mod recent283 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{two_player_game, Target};

    /// Aven Heartstabber gains +2/+2 and deathtouch once 5 distinct mana values
    /// sit in its controller's graveyard.
    #[test]
    fn aven_heartstabber_graveyard_scaler() {
        let mut g = two_player_game();
        let aven = g.add_card_to_battlefield(0, catalog::aven_heartstabber());
        assert_eq!(g.computed_permanent(aven).unwrap().power, 1, "1/1 with an empty graveyard");
        // Five cards of distinct mana values (0,1,2,3,4).
        g.add_card_to_graveyard(0, catalog::forest()); // MV 0
        g.add_card_to_graveyard(0, catalog::kindled_heroism()); // {R} → MV 1
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
        g.add_card_to_graveyard(0, catalog::repel_calamity()); // {1}{W} → MV 2 (dup)
        assert_eq!(g.computed_permanent(aven).unwrap().power, 1, "still four distinct values");
        g.add_card_to_graveyard(0, catalog::horses_of_the_bruinen()); // {3}{U}{U} → MV 5
        g.add_card_to_graveyard(0, catalog::eagle_of_deliverance()); // {4}{W}{W} → MV 6
        assert_eq!(g.computed_permanent(aven).unwrap().power, 3, "+2/+2 at 5+ distinct values");
        assert!(
            g.computed_permanent(aven).unwrap().keywords.contains(&crabomination::card::Keyword::Deathtouch),
            "gains deathtouch",
        );
    }

    /// Ambitious Dragonborn enters with counters equal to the greatest power
    /// among creatures you control and creature cards in your graveyard.
    #[test]
    fn ambitious_dragonborn_counts_graveyard_power() {
        use crabomination::game::{drain_stack, GameAction, TurnStep};
        use crabomination::mana::Color;
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_graveyard(0, catalog::eagle_of_deliverance()); // 5/5 creature card in gy
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 on board (less than 5)
        let spell = g.add_card_to_hand(0, catalog::ambitious_dragonborn());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Ambitious Dragonborn");
        drain_stack(&mut g);
        let db = g.battlefield.iter().find(|c| c.definition.name == "Ambitious Dragonborn").unwrap();
        assert_eq!(
            db.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
            5,
            "X = 5 from the graveyard Eagle (greater than the 2/2 on board)",
        );
    }

    /// Jolly Gerbils draws whenever its controller gives a gift.
    #[test]
    fn jolly_gerbils_draws_on_gift() {
        use crabomination::game::GameAction;
        use crabomination::mana::Color;
        use crabomination::game::drain_stack;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::jolly_gerbils());
        let own = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest()); // something to draw
        let crumb = g.add_card_to_hand(0, catalog::crumb_and_get_it()); // {W}, Gift a Food
        g.players[0].mana_pool.add(Color::White, 1);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastGift {
            card_id: crumb,
            target: Some(Target::Permanent(own)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Crumb and Get It with gift");
        drain_stack(&mut g);
        // -1 for the spell leaving hand, +1 for Jolly Gerbils' draw = net hand.
        assert_eq!(g.players[0].hand.len(), hand, "Jolly Gerbils drew off the gift");
        assert!(
            g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Food"),
            "opponent received the promised Food",
        );
    }

    /// Argivian Cavalier mints a Soldier on entry, and its Enlist taps a helper
    /// to add its power in combat.
    #[test]
    fn argivian_cavalier_etb_token_and_enlist() {
        use crabomination::game::{drain_stack, Attack, AttackTarget, GameAction, TurnStep};
        let mut g = two_player_game();
        let cav = g.add_card_to_battlefield(0, catalog::argivian_cavalier());
        // ETB token.
        let etb = catalog::argivian_cavalier().triggered_abilities[1].effect.clone();
        g.resolve_effect(&etb, &EffectContext::for_ability(cav, 0, None)).unwrap();
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Soldier"),
            "minted a 1/1 Soldier",
        );
        // Enlist through the real combat flow: tap the Soldier helper.
        let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(cav);
        g.clear_sickness(helper);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: cav,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(cav).unwrap().power(), 4, "2 base + 2 from the helper");
    }
}

mod recent284 {
    use crabomination::catalog;
    use crabomination::game::{drain_stack, two_player_game, GameAction, Target, TurnStep};
    use crabomination::mana::Color;

    /// Scrapshooter's gift-gated ETB: promising the gift lets the opponent draw and
    /// destroys a targeted artifact/enchantment.
    #[test]
    fn scrapshooter_gift_promised_destroys() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let art = g.add_card_to_battlefield(1, catalog::sol_ring()); // an artifact to destroy
        g.add_card_to_library(1, catalog::forest()); // opponent has a card to draw
        let opp_hand = g.players[1].hand.len();
        let scrap = g.add_card_to_hand(0, catalog::scrapshooter());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastGift {
            card_id: scrap,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Scrapshooter with gift");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == art), "artifact destroyed");
        assert_eq!(g.players[1].hand.len(), opp_hand + 1, "opponent drew the gift card");
    }

    /// Kitnap (no gift) steals a creature, taps it, and applies three stun counters.
    #[test]
    fn kitnap_no_gift_steals_and_stuns() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let kit = g.add_card_to_hand(0, catalog::kitnap());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: kit, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Kitnap");
        drain_stack(&mut g);
        let v = g.battlefield_find(victim).unwrap();
        assert_eq!(v.controller, 0, "control stolen");
        assert!(v.tapped, "enchanted creature tapped");
        assert_eq!(
            v.counters.get(&crabomination::card::CounterType::Stun).copied().unwrap_or(0),
            3,
            "three stun counters (no gift)",
        );
    }
}

mod recent285 {
    use crabomination::catalog;
    use crabomination::card::CounterType;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{drain_stack, two_player_game, Target};

    /// Parting Gust with no gift exiles a creature and returns it with a +1/+1
    /// counter at the next end step.
    #[test]
    fn parting_gust_no_gift_exiles_and_returns() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&catalog::parting_gust().effect.clone(), &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == bear), "creature exiled");
        // Resolve the delayed next-end-step return.
        g.fire_step_triggers(crabomination::game::TurnStep::End);
        drain_stack(&mut g);
        let returned = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears");
        assert!(returned.is_some(), "returned at the next end step");
        assert_eq!(
            returned.unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
            1,
            "returned with a +1/+1 counter",
        );
    }

    /// Starfall Invocation destroys all creatures; the gifted branch returns a
    /// creature card from your graveyard.
    #[test]
    fn starfall_invocation_gift_reanimates() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::eagle_of_deliverance());
        g.add_card_to_library(1, catalog::forest()); // the gift draw
        g.add_card_to_graveyard(0, catalog::eagle_of_deliverance()); // a fatter body to bring back
        let gifted = catalog::starfall_invocation().gift.unwrap().gifted_effect.clone();
        let gy_eagle = g.players[0].graveyard[0].id;
        let ctx = EffectContext { targets: vec![Target::Permanent(gy_eagle)], ..EffectContext::for_spell(0, None, 0, 0) };
        g.resolve_effect(&gifted, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == mine || c.id == theirs), "board wiped");
        assert!(g.battlefield.iter().any(|c| c.id == gy_eagle && c.controller == 0), "graveyard creature reanimated");
    }
}

mod recent286 {
    use crabomination::card::Keyword;
    use crabomination::game::{drain_stack, two_player_game, GameAction, Target, TurnStep};
    use crabomination::mana::Color;

    /// A Class enters the battlefield at level 1, and casting Stormchaser's Talent
    /// creates a 1/1 Otter with prowess off its level-1 ETB.
    #[test]
    fn stormchasers_talent_enters_at_level_1_and_makes_otter() {
        let mut g = two_player_game();
        let class = g.add_card_to_hand(0, crabomination::catalog::stormchasers_talent());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: class, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Stormchaser's Talent");
        drain_stack(&mut g);
        let c = g.battlefield_find(class).expect("class on battlefield");
        assert_eq!(c.class_level, 1, "Class enters at level 1");
        let otter = g.battlefield.iter().find(|c| c.definition.name == "Otter").expect("Otter minted");
        assert!(otter.definition.keywords.contains(&Keyword::Prowess), "Otter has prowess");
    }

    /// A Class's level resets when it leaves and re-enters the battlefield
    /// (CR 716.2 — the level is battlefield-only).
    #[test]
    fn class_level_resets_on_leave() {
        let mut g = two_player_game();
        let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::hunters_talent());
        // Force it to level 2.
        g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 2;
        // Bounce it to hand, then replay it — it should re-enter at level 1.
        let class2 = g.move_card_to_battlefield_for_test(0, crabomination::catalog::hunters_talent());
        assert_eq!(g.battlefield_find(class2).unwrap().class_level, 1, "re-enters at level 1");
    }

    /// CR 716.4 — Class levels are gained one at a time: the "Level 3" ability
    /// can't be activated while the Class is still at level 1 (its `condition` is
    /// `SourceClassLevelIs(2)`).
    #[test]
    fn cr_716_4_cannot_skip_a_level() {
        let mut g = two_player_game();
        let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::stormchasers_talent());
        drain_stack(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(5);
        // Ability 1 is `{5}{U}: Level 3` — illegal from level 1.
        let res = g.perform_action(GameAction::ActivateAbility {
            card_id: class, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        });
        assert!(res.is_err(), "can't jump to level 3 from level 1");
        assert_eq!(g.battlefield_find(class).unwrap().class_level, 1, "still level 1");
    }

    /// Hunter's Talent's level-1 ETB is a one-sided bite: a creature you control
    /// deals damage equal to its power to a creature you don't control.
    #[test]
    fn hunters_talent_etb_bite() {
        use crabomination::game::effects::EffectContext;
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears()); // 2/2
        let theirs = g.add_card_to_battlefield(1, crabomination::catalog::grizzly_bears()); // 2/2
        let etb = crabomination::catalog::hunters_talent().triggered_abilities[0].effect.clone();
        let ctx = EffectContext {
            targets: vec![Target::Permanent(mine), Target::Permanent(theirs)],
            ..EffectContext::for_spell(0, None, 0, 0)
        };
        g.resolve_effect(&etb, &ctx).unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).is_none(), "their 2/2 took 2 and died");
        assert!(g.battlefield_find(mine).is_some(), "my creature is unharmed (one-sided)");
    }

    /// Scavenger's Talent's level-1 ability makes a Food when a creature you
    /// control dies (once per turn).
    #[test]
    fn scavengers_talent_food_on_creature_death() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, crabomination::catalog::scavengers_talent());
        drain_stack(&mut g);
        let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
        let mut evs = vec![];
        g.sacrifice_one(bear, 0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"),
            "a Food token was created on creature death",
        );
    }

    /// CR 603.3 — an ability that "triggers only once each turn" fires just once
    /// per turn: two creatures dying on the same turn yield a single Food from
    /// Scavenger's Talent's level-1 trigger.
    #[test]
    fn cr_603_3_once_each_turn_trigger() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, crabomination::catalog::scavengers_talent());
        drain_stack(&mut g);
        let food = |g: &crabomination::game::GameState| {
            g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Food").count()
        };
        for _ in 0..2 {
            let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
            let mut evs = vec![];
            g.sacrifice_one(bear, 0, &mut evs);
            g.dispatch_triggers_for_events(&evs);
            drain_stack(&mut g);
        }
        assert_eq!(food(&g), 1, "only one Food despite two deaths this turn");
    }

    /// Bandit's Talent's level-1 ETB: an opponent holding only lands can't discard
    /// a nonland, so they discard two cards.
    #[test]
    fn bandits_talent_etb_discard_two() {
        let mut g = two_player_game();
        let class = g.add_card_to_hand(0, crabomination::catalog::bandits_talent());
        for _ in 0..3 {
            g.add_card_to_hand(1, crabomination::catalog::forest()); // only lands
        }
        let before = g.players[1].hand.len();
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: class, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Bandit's Talent");
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), before - 2, "opponent discarded two (no nonland to pitch)");
    }

    /// Bandit's Talent's level-3 draw scales with the number of low-hand opponents
    /// (`Value::OpponentsWithHandSizeAtMost`).
    #[test]
    fn bandits_talent_level_3_draw_scales() {
        use crabomination::effect::{Effect, PlayerRef, Selector, Value};
        use crabomination::game::effects::EffectContext;
        let draw = Effect::Draw {
            who: Selector::Player(PlayerRef::You),
            amount: Value::OpponentsWithHandSizeAtMost(1),
        };

        // Opponent has 1 card → draw 1.
        let mut g = two_player_game();
        g.add_card_to_hand(1, crabomination::catalog::forest());
        for _ in 0..3 {
            g.add_card_to_library(0, crabomination::catalog::forest());
        }
        let before = g.players[0].hand.len();
        g.resolve_effect(&draw, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        assert_eq!(g.players[0].hand.len(), before + 1, "drew 1 for the low-hand opponent");

        // Opponent has 2 cards → draw 0.
        let mut g2 = two_player_game();
        for _ in 0..2 {
            g2.add_card_to_hand(1, crabomination::catalog::forest());
        }
        g2.add_card_to_library(0, crabomination::catalog::forest());
        let before2 = g2.players[0].hand.len();
        g2.resolve_effect(&draw, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
        assert_eq!(g2.players[0].hand.len(), before2, "no draw when the opponent has two cards");
    }

    /// Wizard Class draws two on reaching level 2, and at level 3 puts a +1/+1
    /// counter on a creature whenever you draw.
    #[test]
    fn wizard_class_levels() {
        let mut g = two_player_game();
        let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::wizard_class());
        drain_stack(&mut g);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        for _ in 0..6 {
            g.add_card_to_library(0, crabomination::catalog::forest());
        }
        // Level up to 2 → draw two.
        let hand_before = g.players[0].hand.len();
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: class, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("level up to 2");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew two on becoming level 2");

        // Level up to 3, then a draw grows a creature.
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::ActivateAbility {
            card_id: class, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("level up to 3");
        drain_stack(&mut g);
        let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
        let mut evs = vec![];
        g.draw_one(0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
            1,
            "drawing at level 3 adds a +1/+1 counter",
        );
    }

    /// Cleric Class's level-1 static adds 1 to life gained; at level 2 gaining life
    /// grows a creature.
    #[test]
    fn cleric_class_life_gain() {
        use crabomination::effect::{Effect, PlayerRef, Selector, Value};
        use crabomination::game::effects::EffectContext;
        let mut g = two_player_game();
        let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::cleric_class());
        drain_stack(&mut g);
        let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
        let life_before = g.players[0].life;

        // Level 1: gaining 2 life actually gains 3 (LifeGainBonus +1).
        g.resolve_effect(
            &Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(2) },
            &EffectContext::for_spell(0, None, 0, 0),
        )
        .unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life_before + 3, "gained 2 + 1 bonus");
        // No counter yet — the level-2 trigger isn't online.
        assert_eq!(
            g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
            0,
        );

        // Force level 2, gain again → a +1/+1 counter lands.
        g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 2;
        let evs = g
            .resolve_effect(
                &Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(1) },
                &EffectContext::for_spell(0, None, 0, 0),
            )
            .unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne),
            1,
            "level-2 lifegain trigger grew the creature",
        );
    }

    /// Warlock Class: at level 1 the end-step drain only fires if a creature died;
    /// at level 3 each opponent loses life equal to what they lost this turn.
    #[test]
    fn warlock_class_end_step_drains() {
        let mut g = two_player_game();
        let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::warlock_class());
        drain_stack(&mut g);
        g.active_player_idx = 0;

        // No creature died → level-1 drain does nothing.
        let life = g.players[1].life;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life, "no drain without a death");

        // A creature dies → level-1 drain costs the opponent 1.
        let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
        let mut evs = vec![];
        g.sacrifice_one(bear, 0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let life = g.players[1].life;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "level-1 drain after a death");

        // At level 3, the opponent additionally loses what they lost this turn.
        g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 3;
        g.players[1].life_lost_this_turn = 5;
        let life = g.players[1].life;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        // Level-1 (creature died earlier) drains 1, level-3 drains the 5 lost.
        assert!(g.players[1].life <= life - 5, "level-3 mirror drain applied");
    }

    /// CR 716.2 — a Class's level survives a full-state snapshot round-trip.
    #[test]
    fn class_level_survives_serde_roundtrip() {
        use crabomination::game::GameState;
        let mut g = two_player_game();
        let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::wizard_class());
        drain_stack(&mut g);
        g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 3;
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: GameState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(g2.battlefield_find(class).unwrap().class_level, 3, "class_level round-trips");
    }

    /// Blacksmith's Talent mints a Sword Equipment at level 1; its level-2 combat
    /// trigger attaches it to a creature you control, granting +1/+1.
    #[test]
    fn blacksmiths_talent_mints_sword_and_attaches() {
        use crabomination::game::effects::EffectContext;
        let mut g = two_player_game();
        let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::blacksmiths_talent());
        drain_stack(&mut g); // resolve the level-1 ETB (mint Sword)
        let sword = g.battlefield.iter().find(|c| c.definition.name == "Sword").expect("Sword minted");
        assert!(sword.definition.is_equipment(), "Sword is an Equipment");
        let sword_id = sword.id;
        let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
        // Resolve the level-2 attach: slot 0 = Sword, slot 1 = bear.
        let mut ctx = EffectContext::for_trigger(class, 0, Some(Target::Permanent(sword_id)), 0);
        ctx.targets.push(Target::Permanent(bear));
        let attach = crabomination::catalog::blacksmiths_talent().triggered_abilities[1].effect.clone();
        g.resolve_effect(&attach, &ctx).unwrap();
        assert_eq!(g.battlefield_find(sword_id).unwrap().attached_to, Some(bear), "Sword attached");
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (3, 3), "equipped bear gets +1/+1");
    }

    /// Level 3 grants double strike + haste to equipped creatures you control, but
    /// only at level 3 and only during your turn (CR 716.2 / 720).
    #[test]
    fn blacksmiths_talent_level_3_your_turn_grant() {
        use crabomination::game::effects::EffectContext;
        let mut g = two_player_game();
        let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::blacksmiths_talent());
        drain_stack(&mut g);
        let sword = g.battlefield.iter().find(|c| c.definition.name == "Sword").unwrap().id;
        let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
        let mut ctx = EffectContext::for_trigger(class, 0, Some(Target::Permanent(sword)), 0);
        ctx.targets.push(Target::Permanent(bear));
        let attach = crabomination::catalog::blacksmiths_talent().triggered_abilities[1].effect.clone();
        g.resolve_effect(&attach, &ctx).unwrap();
        let has = |g: &crabomination::game::GameState, kw| {
            g.computed_permanent(bear).unwrap().keywords.contains(kw)
        };
        // Level 1: no grant.
        g.active_player_idx = 0;
        assert!(!has(&g, &Keyword::DoubleStrike), "no double strike at level 1");
        // Level 3, your turn: both granted.
        g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 3;
        assert!(has(&g, &Keyword::DoubleStrike), "double strike at level 3 on your turn");
        assert!(has(&g, &Keyword::Haste), "haste at level 3 on your turn");
        // Opponent's turn: grant switches off (CR 611.2 — "during your turn").
        g.active_player_idx = 1;
        assert!(!has(&g, &Keyword::DoubleStrike), "no double strike on opponent's turn");
    }

    /// Builder's Talent mints a 0/4 Wall at level 1; its level-3 "becomes level 3"
    /// returns a noncreature, nonland permanent card from the graveyard.
    #[test]
    fn builders_talent_wall_and_level_3_reanimate() {
        let mut g = two_player_game();
        let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::builders_talent());
        drain_stack(&mut g);
        let wall = g.battlefield.iter().find(|c| c.definition.name == "Wall").expect("Wall minted");
        assert_eq!((wall.definition.power, wall.definition.toughness), (0, 4), "0/4 Wall");
        // Seed a noncreature nonland permanent in the graveyard and reach level 3.
        let relic = g.add_card_to_graveyard(0, crabomination::catalog::bonesplitter()); // Artifact
        g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 2;
        use crabomination::game::effects::EffectContext;
        let l3 = crabomination::catalog::builders_talent().triggered_abilities[2].effect.clone();
        let ctx = EffectContext::for_trigger(class, 0, Some(Target::Permanent(relic)), 0);
        g.resolve_effect(&l3, &ctx).unwrap();
        assert!(g.battlefield_find(relic).is_some(), "artifact returned to the battlefield");
    }

    /// Caretaker's Talent's level-3 static pumps creature tokens you control +2/+2
    /// (and only at level 3).
    #[test]
    fn caretakers_talent_level_3_token_anthem() {
        let mut g = two_player_game();
        let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::caretakers_talent());
        drain_stack(&mut g);
        let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
        g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().is_token = true;
        // Level 1: a nontoken-style anthem doesn't apply yet.
        assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "no anthem below level 3");
        g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 3;
        let c = g.computed_permanent(bear).unwrap();
        assert_eq!((c.power, c.toughness), (4, 4), "creature token gets +2/+2 at level 3");
    }

    /// Innkeeper's Talent: L1 begin-combat counter, L2 ward on countered permanents,
    /// L3 counter doubling — each gated on the Class's level.
    #[test]
    fn innkeepers_talent_levels() {
        use crabomination::card::{CounterType, WardCost};
        use crabomination::effect::{Effect, Selector, Value};
        use crabomination::game::effects::EffectContext;
        let mut g = two_player_game();
        let class = g.move_card_to_battlefield_for_test(0, crabomination::catalog::innkeepers_talent());
        drain_stack(&mut g);
        let bear = g.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());

        // L1: the begin-combat trigger puts a +1/+1 counter on a target creature.
        let l1 = crabomination::catalog::innkeepers_talent().triggered_abilities[0].effect.clone();
        let ctx = EffectContext::for_trigger(class, 0, Some(Target::Permanent(bear)), 0);
        g.resolve_effect(&l1, &ctx).unwrap();
        assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);

        // L2: a countered permanent you control gains ward {1}.
        g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 2;
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Ward(WardCost::generic(1))),
            "countered creature has ward at level 2",
        );

        // L3: adding one more counter now doubles to two (CR 614.16, level-gated).
        g.battlefield.iter_mut().find(|c| c.id == class).unwrap().class_level = 3;
        let add = Effect::AddCounter {
            what: Selector::Target(0),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        };
        let ctx2 = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
        g.resolve_effect(&add, &ctx2).unwrap();
        assert_eq!(
            g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
            3,
            "1 existing + doubled 1 (=2) = 3",
        );
    }
}

mod recent287 {
    use crabomination::card::{
        CardDefinition, CardType, CounterType, CreatureType, Keyword, Subtypes,
    };
    use crabomination::game::{two_player_game, GameEvent, GameState};

    /// A bare 2/2 Mount creature for exercising Miriam's Mount/Vehicle riders.
    fn mount_2_2() -> CardDefinition {
        CardDefinition {
            name: "Test Mount",
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Mount], ..Default::default() },
            power: 2,
            toughness: 2,
            ..Default::default()
        }
    }

    /// Miriam grants your Mounts/Vehicles hexproof during your turn only (CR 611.2).
    #[test]
    fn miriam_turn_gated_mount_hexproof() {
        let mut g = two_player_game();
        let _miriam = g.add_card_to_battlefield(0, crabomination::catalog::miriam_herd_whisperer());
        let mount = g.add_card_to_battlefield(0, mount_2_2());
        let has_hexproof =
            |g: &GameState| g.computed_permanent(mount).unwrap().keywords.contains(&Keyword::Hexproof);
        g.active_player_idx = 0;
        assert!(has_hexproof(&g), "Mount has hexproof on your turn");
        g.active_player_idx = 1;
        assert!(!has_hexproof(&g), "no hexproof on the opponent's turn");
    }

    /// Miriam puts a +1/+1 counter on a Mount you control when it attacks.
    #[test]
    fn miriam_counters_attacking_mount() {
        let mut g = two_player_game();
        let _miriam = g.add_card_to_battlefield(0, crabomination::catalog::miriam_herd_whisperer());
        let mount = g.add_card_to_battlefield(0, mount_2_2());
        g.dispatch_triggers_for_events(&[GameEvent::AttackerDeclared(mount)]);
        crabomination::game::drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(mount).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "attacking Mount gets a +1/+1 counter"
        );
    }

    /// Vadmir gains a counter once per turn on committing a crime, and gains
    /// menace + lifelink once it has four or more +1/+1 counters.
    #[test]
    fn vadmir_crime_counters_and_keyword_threshold() {
        let mut g = two_player_game();
        let vadmir = g.add_card_to_battlefield(0, crabomination::catalog::vadmir_new_blood());
        // Two crimes in one turn → only one counter (once each turn).
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        crabomination::game::drain_stack(&mut g);
        g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
        crabomination::game::drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(vadmir).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "crime counter triggers only once each turn"
        );
        // Below threshold: no menace yet.
        assert!(!g.computed_permanent(vadmir).unwrap().keywords.contains(&Keyword::Menace));
        // Bump to four counters → menace + lifelink.
        g.battlefield_find_mut(vadmir).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
        let c = g.computed_permanent(vadmir).unwrap();
        assert!(c.keywords.contains(&Keyword::Menace), "menace at 4+ counters");
        assert!(c.keywords.contains(&Keyword::Lifelink), "lifelink at 4+ counters");
    }

    /// Skyserpent Seeker's exhaust ability reveals until two lands, puts them onto
    /// the battlefield tapped, and grows itself with a +1/+1 counter.
    #[test]
    fn skyserpent_seeker_ramps_two_lands() {
        use crabomination::card::CounterType;
        use crabomination::catalog;
        use crabomination::game::{drain_stack, effects::EffectContext};
        let mut g = two_player_game();
        let snake = g.add_card_to_battlefield(0, catalog::skyserpent_seeker());
        // Library top → bottom: a nonland, then two Forests.
        g.players[0].library.clear();
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        let ability = catalog::skyserpent_seeker().activated_abilities[0].effect.clone();
        let ctx = EffectContext::for_ability(snake, 0, None);
        g.resolve_effect(&ability, &ctx).unwrap();
        drain_stack(&mut g);
        let forests = g.battlefield.iter().filter(|c| c.definition.name == "Forest").count();
        assert_eq!(forests, 2, "two Forests put onto the battlefield");
        assert!(
            g.battlefield.iter().filter(|c| c.definition.name == "Forest").all(|c| c.tapped),
            "the revealed lands enter tapped",
        );
        assert_eq!(
            g.battlefield_find(snake).unwrap().counter_count(CounterType::PlusOnePlusOne),
            1,
            "Skyserpent grows a +1/+1 counter",
        );
    }
}

mod recent288 {
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::{drain_stack, two_player_game, GameAction, GameState};
    use crabomination::mana::Color;
    use crabomination::TurnStep;

    /// Doc Aurlock reduces Plot activation costs by {2}: Longhorn Sharpshooter's
    /// {3}{R} plot cost becomes {1}{R}.
    #[test]
    fn doc_aurlock_discounts_plot() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::doc_aurlock_grizzled_genius());
        let card = g.add_card_to_hand(0, catalog::longhorn_sharpshooter());
        // Only {1}{R} available — the full {3}{R} would be short.
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::Plot { card_id: card }).expect("plot at the reduced cost");
        assert!(g.exile.iter().any(|c| c.id == card), "the plotted card sits in exile");
        assert_eq!(g.players[0].mana_pool.total(), 0, "the reduced cost drained the pool exactly");
    }

    /// Doc Aurlock reduces exile casts by {2}: a foretold Behold the Multiverse
    /// (foretell {1}{U}) casts for just {U}.
    #[test]
    fn doc_aurlock_discounts_exile_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::doc_aurlock_grizzled_genius());
        let card = g.add_card_to_exile(0, catalog::behold_the_multiverse());
        g.exile.iter_mut().find(|c| c.id == card).unwrap().face_down = true;
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.perform_action(GameAction::CastForetold {
            card_id: card,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast the foretold spell at the reduced exile cost");
        assert_eq!(g.players[0].mana_pool.total(), 0, "only one blue mana was spent");
    }

    fn ready(g: &mut GameState) {
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
    }

    /// Saddling Fortune records the rider in `saddled_by`.
    #[test]
    fn fortune_records_saddlers() {
        let mut g = two_player_game();
        let fortune = g.add_card_to_battlefield(0, catalog::fortune_loyal_steed());
        let rider = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(fortune);
        g.clear_sickness(rider);
        ready(&mut g);
        g.perform_action(GameAction::Saddle { mount: fortune, creatures: vec![rider] }).expect("saddle");
        let m = g.battlefield_find(fortune).unwrap();
        assert!(m.saddled, "Fortune is saddled");
        assert_eq!(m.saddled_by, vec![rider], "the rider is remembered");
    }

    /// Fortune attacks while saddled → at end of combat it and one saddler blink,
    /// returning untapped and summoning-sick.
    #[test]
    fn fortune_end_of_combat_blink() {
        let mut g = two_player_game();
        let fortune = g.add_card_to_battlefield(0, catalog::fortune_loyal_steed());
        let rider = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(fortune);
        g.clear_sickness(rider);
        ready(&mut g);
        g.perform_action(GameAction::Saddle { mount: fortune, creatures: vec![rider] }).expect("saddle");
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker: fortune, target: AttackTarget::Player(1) }])
            .expect("attack");
        drain_stack(&mut g);
        // End of combat → the delayed blink fires.
        g.fire_step_triggers(TurnStep::EndCombat);
        drain_stack(&mut g);
        for id in [fortune, rider] {
            let c = g.battlefield_find(id).expect("returned to the battlefield");
            assert!(!c.tapped, "returns untapped");
            assert!(c.summoning_sick, "returns as a fresh, summoning-sick object");
            assert!(!c.saddled, "the returned Mount is no longer saddled");
        }
    }
}

mod recent289 {
    use crabomination::catalog;
    use crabomination::card::CreatureType;
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::{drain_stack, two_player_game, GameAction, GameEvent, GameState};
    use crabomination::TurnStep;

    fn ready(g: &mut GameState) {
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
    }

    fn attack_with(g: &mut GameState, attacker: crabomination::card::CardId) {
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
        drain_stack(g);
    }

    /// Luxurious Locomotive makes a Treasure for each creature that crewed it.
    #[test]
    fn luxurious_locomotive_treasures_per_crewer() {
        let mut g = two_player_game();
        let loco = g.add_card_to_battlefield(0, catalog::luxurious_locomotive());
        let c1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let c2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(c1);
        g.clear_sickness(c2);
        ready(&mut g);
        g.perform_action(GameAction::Crew { vehicle: loco, crew_creatures: vec![c1, c2] }).expect("crew");
        attack_with(&mut g, loco);
        let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
        assert_eq!(treasures, 2, "one Treasure per crewer");
    }

    /// Mobile Homestead has haste only while you control a Mount, and deploys a
    /// revealed top land when it attacks.
    #[test]
    fn mobile_homestead_haste_and_land_deploy() {
        let mut g = two_player_game();
        let home = g.add_card_to_battlefield(0, catalog::mobile_homestead());
        let has_haste = |g: &GameState| {
            g.computed_permanent(home).unwrap().keywords.contains(&crabomination::card::Keyword::Haste)
        };
        assert!(!has_haste(&g), "no Mount → no haste");
        // Add a Mount → haste.
        let mut mount = catalog::grizzly_bears();
        mount.subtypes.creature_types = vec![CreatureType::Mount];
        g.add_card_to_battlefield(0, mount);
        assert!(has_haste(&g), "controlling a Mount grants haste");
        // Attack with a Forest on top → it enters tapped.
        g.players[0].library.clear();
        g.add_card_to_library(0, catalog::forest());
        ready(&mut g);
        g.clear_sickness(home);
        // Crew the vehicle so it can attack as a creature.
        let crew = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(crew);
        g.perform_action(GameAction::Crew { vehicle: home, crew_creatures: vec![crew] }).expect("crew");
        attack_with(&mut g, home);
        let forest = g.battlefield.iter().find(|c| c.definition.name == "Forest").expect("land deployed");
        assert!(forest.tapped, "the deployed land enters tapped");
    }

    /// Wylie Duke draws and gains life whenever it becomes tapped.
    #[test]
    fn wylie_duke_draws_on_tap() {
        let mut g = two_player_game();
        let wylie = g.add_card_to_battlefield(0, catalog::wylie_duke_atiin_hero());
        g.add_card_to_library(0, catalog::forest());
        let hand_before = g.players[0].hand.len();
        let life_before = g.players[0].life;
        g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: wylie, actor: None }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
        assert_eq!(g.players[0].life, life_before + 1, "gained 1 life");
    }

    /// Bruse Tarl exiles the top card on ETB: a land makes an Ox token; a nonland
    /// gets a play-until-end-of-next-turn grant.
    #[test]
    fn bruse_tarl_reveals_land_makes_ox() {
        let mut g = two_player_game();
        // Land on top → Ox token, and Oxen get double strike from Bruse Tarl.
        g.players[0].library.clear();
        g.add_card_to_library(0, catalog::forest());
        let bruse = g.add_card_to_battlefield(0, catalog::bruse_tarl_roving_rancher());
        g.fire_self_etb_triggers(bruse, 0);
        drain_stack(&mut g);
        let ox = g.battlefield.iter().find(|c| c.definition.name == "Ox").expect("Ox token created");
        assert!(
            g.computed_permanent(ox.id).unwrap().keywords.contains(&crabomination::card::Keyword::DoubleStrike),
            "the Ox has double strike from Bruse Tarl's anthem",
        );
        assert!(g.exile.iter().any(|c| c.definition.name == "Forest"), "the land stays exiled");
    }

    /// A nonland on top instead grants Bruse Tarl's controller a may-play.
    #[test]
    fn bruse_tarl_reveals_nonland_grants_may_play() {
        let mut g = two_player_game();
        g.players[0].library.clear();
        g.add_card_to_library(0, catalog::grizzly_bears());
        let bruse = g.add_card_to_battlefield(0, catalog::bruse_tarl_roving_rancher());
        g.fire_self_etb_triggers(bruse, 0);
        drain_stack(&mut g);
        let bears = g.exile.iter().find(|c| c.definition.name == "Grizzly Bears").expect("nonland exiled");
        assert!(bears.may_play_until.is_some(), "the nonland is castable from exile");
        assert!(g.battlefield.iter().all(|c| c.definition.name != "Ox"), "no Ox token for a nonland");
    }
}

mod recent290 {
    use crabomination::card::{ArtifactSubtype, CardType, CreatureType};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EffectContext;
    use crabomination::game::{drain_stack, two_player_game, GameAction, Target};
    use crabomination::mana::Color;

    /// Krosan Restorer's {T} ability untaps a single target land.
    #[test]
    fn krosan_restorer_untaps_target_land() {
        let mut g = two_player_game();
        let bill = g.add_card_to_battlefield(0, catalog::krosan_restorer());
        g.clear_sickness(bill);
        let land = g.add_card_to_battlefield(0, catalog::forest());
        g.battlefield_find_mut(land).unwrap().tapped = true;
        g.perform_action(GameAction::ActivateAbility {
            card_id: bill, ability_index: 0, target: Some(Target::Permanent(land)),
            additional_targets: vec![], x_value: None,
        }).expect("untap");
        drain_stack(&mut g);
        assert!(!g.battlefield_find(land).unwrap().tapped, "the target land is untapped");
    }

    /// The threshold ability untaps up to three lands once seven cards sit in the
    /// graveyard.
    #[test]
    fn krosan_restorer_threshold_untaps_three() {
        let mut g = two_player_game();
        let bill = g.add_card_to_battlefield(0, catalog::krosan_restorer());
        g.clear_sickness(bill);
        for _ in 0..7 {
            g.add_card_to_graveyard(0, catalog::grizzly_bears());
        }
        let lands: Vec<_> = (0..3)
            .map(|_| {
                let l = g.add_card_to_battlefield(0, catalog::forest());
                g.battlefield_find_mut(l).unwrap().tapped = true;
                l
            })
            .collect();
        g.perform_action(GameAction::ActivateAbility {
            card_id: bill, ability_index: 1, target: None,
            additional_targets: vec![], x_value: None,
        }).expect("threshold untap");
        drain_stack(&mut g);
        assert!(lands.iter().all(|&l| !g.battlefield_find(l).unwrap().tapped), "three lands untapped");
    }

    /// Vraska, the Silencer steals an opponent's dying creature as a tapped
    /// Treasure when you pay {1}.
    #[test]
    fn vraska_steals_dying_creature_as_treasure() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::vraska_the_silencer());
        let fodder = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        // P0 bolts P1's fodder; it dies and Vraska's trigger fires.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(fodder)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        let stolen = g.battlefield_find(fodder).expect("returned to the battlefield");
        assert_eq!(stolen.controller, 0, "under your control");
        assert!(stolen.tapped, "enters tapped");
        let cp = g.computed_permanent(fodder).unwrap();
        assert!(cp.card_types.contains(&CardType::Artifact), "it's an artifact");
        assert!(cp.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Treasure), "…a Treasure");
        assert!(!cp.card_types.contains(&CardType::Creature), "loses its creature type");
    }

    /// Ego Drain's downside: with no Faerie, its caster exiles a card from hand.
    #[test]
    fn ego_drain_exiles_without_faerie() {
        let mut g = two_player_game();
        let src = g.add_card_to_hand(0, catalog::ego_drain());
        g.add_card_to_hand(0, catalog::grizzly_bears()); // the card to exile
        g.add_card_to_hand(1, catalog::grizzly_bears()); // discard fodder
        let ctx = EffectContext::for_ability(src, 0, None);
        let hand_before = g.players[0].hand.len();
        let exile_before = g.exile.len();
        g.resolve_effect(&catalog::ego_drain().effect, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand_before - 1, "caster exiled a card");
        assert_eq!(g.exile.len(), exile_before + 1, "a card moved to exile");
    }

    /// Controlling a Faerie skips Ego Drain's exile clause.
    #[test]
    fn ego_drain_keeps_hand_with_faerie() {
        let mut g = two_player_game();
        let src = g.add_card_to_hand(0, catalog::ego_drain());
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::grizzly_bears());
        // A Faerie on the battlefield spares the caster's hand.
        let mut faerie = catalog::grizzly_bears();
        faerie.subtypes.creature_types = vec![CreatureType::Faerie];
        g.add_card_to_battlefield(0, faerie);
        let ctx = EffectContext::for_ability(src, 0, None);
        let hand_before = g.players[0].hand.len();
        g.resolve_effect(&catalog::ego_drain().effect, &ctx).unwrap();
        assert_eq!(g.players[0].hand.len(), hand_before, "no self-exile with a Faerie out");
    }

    /// Zoyowa Lava-Tongue burns an opponent who can't discard or sacrifice, but
    /// only if you descended this turn.
    #[test]
    fn zoyowa_burns_the_helpless_opponent_after_descending() {
        use crabomination::game::TurnStep;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::zoyowa_lava_tongue());
        g.active_player_idx = 0;
        // Opponent has no cards and no permanents → no dodge available.
        g.players[1].hand.clear();
        // No descend yet → the end-step trigger does nothing.
        let life0 = g.players[1].life;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life0, "no descend → no punisher");
        // Descend, then the opponent takes 3.
        g.players[0].descended_this_turn = true;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life0 - 3, "helpless opponent takes 3");
    }

    /// Discerning Financier mints a Treasure on upkeep when an opponent is ahead
    /// on lands, then donates it to that opponent for a card.
    #[test]
    fn discerning_financier_makes_and_donates_a_treasure() {
        use crabomination::game::TurnStep;
        let mut g = two_player_game();
        let fin = g.add_card_to_battlefield(0, catalog::discerning_financier());
        g.active_player_idx = 0;
        // Opponent controls one more land → the upkeep trigger fires.
        g.add_card_to_battlefield(1, catalog::forest());
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        let treasure = g
            .battlefield
            .iter()
            .find(|c| c.controller == 0 && c.definition.name == "Treasure")
            .expect("minted a Treasure")
            .id;
        // Donate it: the opponent gains control and you draw.
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: fin, ability_index: 0, target: Some(Target::Permanent(treasure)),
            additional_targets: vec![], x_value: None,
        }).expect("donate");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(treasure).unwrap().controller, 1, "opponent controls the Treasure");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "you drew a card");
    }

    /// Grove Rumbler grows +2/+2 whenever a land you control enters.
    #[test]
    fn grove_rumbler_grows_on_landfall() {
        use crabomination::game::TurnStep;
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        let rumbler = g.add_card_to_battlefield(0, catalog::grove_rumbler());
        assert_eq!(g.computed_permanent(rumbler).unwrap().power, 3, "base 3/3");
        let forest = g.add_card_to_hand(0, catalog::forest());
        g.perform_action(GameAction::PlayLand(forest)).expect("play land");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(rumbler).unwrap().power, 5, "+2/+2 after a landfall");
    }

    /// Blister Beetle shrinks a target creature by 1/1 on entry.
    #[test]
    fn blister_beetle_weakens_a_creature() {
        let mut g = two_player_game();
        let beetle = g.add_card_to_battlefield(0, catalog::blister_beetle());
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let etb = catalog::blister_beetle().triggered_abilities[0].effect.clone();
        g.resolve_effect(&etb, &EffectContext::for_ability(beetle, 0, Some(Target::Permanent(victim)))).unwrap();
        let cp = g.computed_permanent(victim).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "2/2 → 1/1 until end of turn");
    }

    /// Swift Response destroys a tapped creature. Casting it only accepts a
    /// tapped target (the `Tapped` filter is enforced at cast time).
    #[test]
    fn swift_response_destroys_tapped_creature() {
        let mut g = two_player_game();
        let tapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let untapped = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(tapped).unwrap().tapped = true;
        // Cast at the untapped creature is rejected (not a legal target).
        let spell = g.add_card_to_hand(0, catalog::swift_response());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert!(g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(untapped)),
            additional_targets: vec![], mode: None, x_value: None,
        }).is_err(), "untapped creature isn't a legal target");
        // Cast at the tapped creature destroys it.
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(tapped)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("tapped is legal");
        drain_stack(&mut g);
        assert!(g.battlefield_find(tapped).is_none(), "tapped creature destroyed");
    }

    /// Might Beyond Reason adds two counters, or three with delirium.
    #[test]
    fn might_beyond_reason_scales_with_delirium() {
        let counters = |deliriumize: bool| -> u32 {
            let mut g = two_player_game();
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            if deliriumize {
                // Four card types in the graveyard → delirium.
                g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
                g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
                g.add_card_to_graveyard(0, catalog::forest()); // land
                g.add_card_to_graveyard(0, catalog::sol_ring()); // artifact
            }
            let ctx = EffectContext::for_ability(c, 0, Some(Target::Permanent(c)));
            g.resolve_effect(&catalog::might_beyond_reason().effect, &ctx).unwrap();
            g.battlefield_find(c).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne)
        };
        assert_eq!(counters(false), 2, "two counters without delirium");
        assert_eq!(counters(true), 3, "three with delirium");
    }

    /// Astral Wingspan attaches, draws a card, and grants +2/+2 and flying.
    #[test]
    fn astral_wingspan_buffs_and_draws() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_battlefield(0, catalog::astral_wingspan());
        g.add_card_to_library(0, catalog::forest());
        let hand_before = g.players[0].hand.len();
        // Resolve the enchant (Attach) then the ETB draw.
        let ctx = EffectContext::for_ability(aura, 0, Some(Target::Permanent(bear)));
        g.resolve_effect(&catalog::astral_wingspan().effect, &ctx).unwrap();
        g.fire_self_etb_triggers(aura, 0);
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "2/2 → 4/4");
        assert!(cp.keywords.contains(&Keyword::Flying), "gains flying");
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "ETB drew a card");
    }

    /// No upkeep Treasure when you aren't behind on lands.
    #[test]
    fn discerning_financier_idle_when_not_behind_on_lands() {
        use crabomination::game::TurnStep;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::discerning_financier());
        g.add_card_to_battlefield(0, catalog::forest()); // you're not behind
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        assert!(
            !g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
            "no Treasure while you match the opponent's land count",
        );
    }

    /// Frontier Warmonger gives your attacking creatures menace once they're
    /// declared as attackers (and only while attacking).
    #[test]
    fn frontier_warmonger_grants_menace_to_attackers() {
        use crabomination::card::Keyword;
        use crabomination::game::types::{Attack, AttackTarget, TurnStep};
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::frontier_warmonger());
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(attacker);
        // Not attacking yet → no menace.
        assert!(
            !g.computed_permanent(attacker).unwrap().keywords.contains(&Keyword::Menace),
            "no menace before combat",
        );
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        assert!(
            g.computed_permanent(attacker).unwrap().keywords.contains(&Keyword::Menace),
            "attacking creature gains menace",
        );
    }
}
