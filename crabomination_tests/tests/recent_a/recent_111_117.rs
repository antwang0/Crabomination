//! Tests for recentN card batches 111-117 (merged from per-batch micro-files).

mod recent111 {
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Silvergill Douser shrinks power by your Merfolk/Faerie count.
    #[test]
    fn silvergill_douser_shrinks_by_tribe_count() {
        let mut g = two_player_game();
        let douser = g.add_card_to_battlefield(0, catalog::silvergill_douser());
        g.add_card_to_battlefield(0, catalog::kumenas_speaker()); // Merfolk
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(douser);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: douser, ability_index: 0, target: Some(Target::Permanent(foe)),
            additional_targets: vec![], x_value: None,
        })
        .expect("douse");
        drain_stack(&mut g);
        let cp = g.computed_permanent(foe).unwrap();
        assert_eq!((cp.power, cp.toughness), (0, 2), "-2/-0 from two Merfolk");
    }

    /// Merfolk Sovereign pumps other Merfolk and grants unblockable.
    #[test]
    fn merfolk_sovereign_lord_and_unblockable() {
        let mut g = two_player_game();
        let sovereign = g.add_card_to_battlefield(0, catalog::merfolk_sovereign());
        let speaker = g.add_card_to_battlefield(0, catalog::kumenas_speaker());
        g.clear_sickness(sovereign);
        let cp = g.computed_permanent(speaker).unwrap();
        // 1/1 base + lord +1/+1 + Speaker's own another-Merfolk +1/+1.
        assert_eq!((cp.power, cp.toughness), (3, 3), "lord pump stacks with self-pump");
        assert_eq!(g.computed_permanent(sovereign).unwrap().power, 2, "not self-pumped");
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: sovereign, ability_index: 0, target: Some(Target::Permanent(speaker)),
            additional_targets: vec![], x_value: None,
        })
        .expect("grant");
        drain_stack(&mut g);
        assert!(g.computed_permanent(speaker).unwrap().keywords.contains(&crabomination::card::Keyword::Unblockable));
    }

    /// Tidebinder Mage taps a red/green creature and locks its next untap.
    #[test]
    fn tidebinder_mage_taps_and_locks() {
        let mut g = two_player_game();
        let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
        let mage = g.add_card_to_battlefield(0, catalog::tidebinder_mage());
        g.fire_self_etb_triggers(mage, 0);
        drain_stack(&mut g);
        let c = g.battlefield_find(dragon).unwrap();
        assert!(c.tapped, "tapped by the ETB");
        assert!(c.skip_next_untap, "untap-locked");
    }

    /// Master of Waves mints Elementals equal to blue devotion, pumped by its
    /// own lord static (1/0 → 2/1).
    #[test]
    fn master_of_waves_devotion_tokens() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::tempest_djinn()); // {U}{U}{U} → devotion 3
        let master = g.add_card_to_battlefield(0, catalog::master_of_waves());
        g.fire_self_etb_triggers(master, 0);
        drain_stack(&mut g);
        let tokens: Vec<_> = g
            .battlefield
            .iter()
            .filter(|c| c.definition.name == "Elemental")
            .map(|c| c.id)
            .collect();
        assert_eq!(tokens.len(), 4, "devotion 3 + Master's own {{U}}");
        let cp = g.computed_permanent(tokens[0]).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 1), "lord-pumped 1/0");
    }

    /// Loaming Shaman shuffles the target player's graveyard into their library.
    #[test]
    fn loaming_shaman_recycles_graveyard() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
        g.add_card_to_graveyard(1, catalog::lightning_bolt());
        let shaman = g.add_card_to_battlefield(0, catalog::loaming_shaman());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
        g.fire_self_etb_triggers(shaman, 0);
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.is_empty(), "graveyard shuffled away");
        assert_eq!(g.players[1].library.len(), 2);
    }

    /// Defense of the Heart fires only against three opposing creatures.
    #[test]
    fn defense_of_the_heart_tutors_two() {
        let mut g = two_player_game();
        let heart = g.add_card_to_battlefield(0, catalog::defense_of_the_heart());
        let big1 = g.add_card_to_library(0, catalog::grizzly_bears());
        let big2 = g.add_card_to_library(0, catalog::savannah_lions());
        g.active_player_idx = 0;
        g.fire_step_triggers(crabomination::game::TurnStep::Upkeep);
        drain_stack(&mut g);
        assert!(g.battlefield_find(heart).is_some(), "no trigger below three creatures");
        for _ in 0..3 {
            g.add_card_to_battlefield(1, catalog::grizzly_bears());
        }
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Search(Some(big1)),
            DecisionAnswer::Search(Some(big2)),
        ]));
        g.fire_step_triggers(crabomination::game::TurnStep::Upkeep);
        drain_stack(&mut g);
        assert!(g.battlefield_find(heart).is_none(), "sacrificed on trigger");
        assert!(g.battlefield_find(big1).is_some() && g.battlefield_find(big2).is_some());
    }

    /// Leaf-Crowned Visionary draws off an Elf cast when {G} is paid.
    #[test]
    fn leaf_crowned_visionary_elf_cantrip() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::leaf_crowned_visionary());
        g.add_card_to_library(0, catalog::forest());
        let elf = g.add_card_to_hand(0, catalog::canopy_tactician());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Elf");
        drain_stack(&mut g);
        // -1 (cast) +1 (paid {G} draw) = unchanged.
        assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "paid {{G}}, drew a card");
    }

    /// Copperhorn Scout untaps the rest of the team on attack.
    #[test]
    fn copperhorn_scout_untaps_team() {
        let mut g = two_player_game();
        let scout = g.add_card_to_battlefield(0, catalog::copperhorn_scout());
        let mate = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(mate).unwrap().tapped = true;
        g.clear_sickness(scout);
        g.active_player_idx = 0;
        g.step = crabomination::game::TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: scout,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert!(!g.battlefield_find(mate).unwrap().tapped, "untapped by the attack trigger");
    }

    /// Genesis Wave deploys permanents with MV ≤ X and bins the rest.
    #[test]
    fn genesis_wave_deploys_cheap_permanents() {
        let mut g = two_player_game();
        // Top 3 (X=3): bear (MV 2, deploy), bolt (nonpermanent, gy), djinn (MV 3, deploy).
        let djinn = g.add_card_to_library(0, catalog::tempest_djinn());
        let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
        let bear = g.add_card_to_library(0, catalog::grizzly_bears());
        let wave = g.add_card_to_hand(0, catalog::genesis_wave());
        g.players[0].mana_pool.add(Color::Green, 3);
        g.players[0].mana_pool.add_colorless(3);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: wave, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
        })
        .expect("cast X=3");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_some());
        assert!(g.battlefield_find(djinn).is_some());
        assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "instant to the graveyard");
    }

    /// Harald digs five for an Elf/Warrior card.
    #[test]
    fn harald_digs_for_an_elf() {
        let mut g = two_player_game();
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::island());
        }
        let elf = g.add_card_to_library(0, catalog::canopy_tactician()); // top
        let harald = g.add_card_to_battlefield(0, catalog::harald_king_of_skemfar());
        g.fire_self_etb_triggers(harald, 0);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == elf), "Elf picked to hand");
    }

    /// Skemfar Shadowsage drains for the largest shared-tribe count.
    #[test]
    fn skemfar_shadowsage_drains_by_tribe() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::canopy_tactician()); // Elf
        g.add_card_to_battlefield(0, catalog::leaf_crowned_visionary()); // Elf
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // Bear
        let sage = g.add_card_to_battlefield(0, catalog::skemfar_shadowsage()); // Elf
        let life1 = g.players[1].life;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
        g.fire_self_etb_triggers(sage, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1 - 3, "three Elves in common");
    }

    /// Rhonas can't attack alone but wakes up next to a power-4 creature.
    #[test]
    fn rhonas_needs_a_big_friend() {
        let mut g = two_player_game();
        let rhonas = g.add_card_to_battlefield(0, catalog::rhonas_the_indomitable());
        g.clear_sickness(rhonas);
        g.active_player_idx = 0;
        g.step = crabomination::game::TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let err = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: rhonas,
            target: AttackTarget::Player(1),
        }]));
        assert!(err.is_err(), "no other power-4 creature → can't attack");
        g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: rhonas,
            target: AttackTarget::Player(1),
        }]))
        .expect("attacks with a power-4 friend");
    }

    /// Oath of Nissa digs three for a creature/land/planeswalker.
    #[test]
    fn oath_of_nissa_digs_three() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::lightning_bolt());
        g.add_card_to_library(0, catalog::counterspell());
        let bear = g.add_card_to_library(0, catalog::grizzly_bears()); // top
        let oath = g.add_card_to_battlefield(0, catalog::oath_of_nissa());
        g.fire_self_etb_triggers(oath, 0);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    }

    /// Trash for Treasure eats an artifact to reanimate a bigger one.
    #[test]
    fn trash_for_treasure_swaps_artifacts() {
        let mut g = two_player_game();
        let fodder = g.add_card_to_battlefield(0, catalog::chrome_mox());
        let big = g.add_card_to_graveyard(0, catalog::metalwork_colossus());
        let tft = g.add_card_to_hand(0, catalog::trash_for_treasure());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: tft, target: Some(Target::Permanent(big)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "artifact sacrificed as a cost");
        assert!(g.battlefield_find(big).is_some(), "Colossus reanimated");
    }

    /// Metalwork Colossus discounts by noncreature-artifact MV and self-recurs.
    #[test]
    fn metalwork_colossus_discount_and_recursion() {
        let mut g = two_player_game();
        // Two Shriekhorns ({1} each) + a Darksteel Garrison ({2}) = MV 4 → {7}.
        g.add_card_to_battlefield(0, catalog::shriekhorn());
        g.add_card_to_battlefield(0, catalog::shriekhorn());
        g.add_card_to_battlefield(0, catalog::darksteel_garrison());
        let colossus = g.add_card_to_hand(0, catalog::metalwork_colossus());
        g.players[0].mana_pool.add_colorless(7);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: colossus, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("castable for {7}");
        drain_stack(&mut g);
        assert!(g.battlefield_find(colossus).is_some());
        // Recursion: dump it in the graveyard, sac two artifacts to return it.
        let evs = g.remove_to_graveyard_with_triggers(colossus);
        g.dispatch_triggers_for_events(&evs);
        g.perform_action(GameAction::ActivateAbility {
            card_id: colossus, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("graveyard recursion");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == colossus), "back to hand");
    }

    /// Jhoira's Familiar discounts historic (artifact/legendary/Saga) spells.
    #[test]
    fn jhoiras_familiar_discounts_historic() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::jhoiras_familiar());
        let mox = g.add_card_to_hand(0, catalog::shriekhorn()); // {1} artifact → free
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: mox, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("{1} artifact is free with the Familiar");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mox).is_some());
    }

    /// Grand Architect pumps blue creatures and funds artifacts off tapping one.
    #[test]
    fn grand_architect_pumps_and_funds() {
        let mut g = two_player_game();
        let architect = g.add_card_to_battlefield(0, catalog::grand_architect());
        let djinn = g.add_card_to_battlefield(0, catalog::tempest_djinn());
        let cp = g.computed_permanent(djinn).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 5), "blue lord pump (0/4 base)");
        // Tap the Djinn for {C}{C} restricted to artifacts.
        g.clear_sickness(djinn);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: architect, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("tap a blue creature for {C}{C}");
        assert!(g.battlefield_find(djinn).unwrap().tapped, "the blue creature tapped");
        assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "restricted {{C}}{{C}} floats");
        let horn = g.add_card_to_hand(0, catalog::shriekhorn());
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: horn, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("artifact funded by the restricted mana");
    }
}

mod recent112 {
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// CR 107.17 — {Q}: the source must be tapped; paying untaps it.
    #[test]
    fn cr_107_17_pili_pala_untap_cost() {
        let mut g = two_player_game();
        let pala = g.add_card_to_battlefield(0, catalog::pili_pala());
        g.clear_sickness(pala);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        // Untapped: {Q} is unpayable.
        let err = g.perform_action(GameAction::ActivateAbility {
            card_id: pala, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        });
        assert!(err.is_err(), "{{Q}} needs a tapped source");
        g.battlefield_find_mut(pala).unwrap().tapped = true;
        g.perform_action(GameAction::ActivateAbility {
            card_id: pala, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("pay {2}, {Q}");
        assert!(!g.battlefield_find(pala).unwrap().tapped, "untapped as the cost");
        assert_eq!(g.players[0].mana_pool.total(), 1, "one mana of any color added");
    }

    /// CR 602.5h — a summoning-sick creature can't pay {Q} either.
    #[test]
    fn cr_602_5h_untap_cost_summoning_sick() {
        let mut g = two_player_game();
        let pala = g.add_card_to_battlefield(0, catalog::pili_pala());
        g.battlefield_find_mut(pala).unwrap().tapped = true;
        g.battlefield_find_mut(pala).unwrap().summoning_sick = true;
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        let err = g.perform_action(GameAction::ActivateAbility {
            card_id: pala, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        });
        assert!(matches!(err, Err(GameError::SummoningSickness(_))));
    }

    /// Phyrexian Unlife — no loss at ≤ 0 life; damage lands as poison instead.
    #[test]
    fn phyrexian_unlife_zero_life_mode() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::phyrexian_unlife());
        g.players[0].life = -3;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        assert!(!g.players[0].eliminated, "no loss from life with Unlife");
        // Damage at ≤ 0 life becomes poison, not further life loss.
        let ctx = crabomination::game::effects::EffectContext::for_spell(1, Some(Target::Player(0)), 0, 0);
        let events = g
            .resolve_effect(
                &crabomination::effect::Effect::DealDamage {
                    to: crabomination::effect::Selector::Target(0),
                    amount: crabomination::effect::Value::Const(4),
                },
                &ctx,
            )
            .unwrap();
        g.dispatch_triggers_for_events(&events);
        assert_eq!(g.players[0].life, -3, "life untouched");
        assert_eq!(g.players[0].poison_counters, 4, "damage became poison");
        // Ten poison still loses through Unlife.
        g.players[0].poison_counters = 10;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        assert!(g.players[0].eliminated, "poison loss is not gated");
    }

    /// Salvage Titan's alternative cost eats three artifacts.
    #[test]
    fn salvage_titan_alt_cost() {
        let mut g = two_player_game();
        let a1 = g.add_card_to_battlefield(0, catalog::chrome_mox());
        let a2 = g.add_card_to_battlefield(0, catalog::shriekhorn());
        let a3 = g.add_card_to_battlefield(0, catalog::darksteel_garrison());
        let titan = g.add_card_to_hand(0, catalog::salvage_titan());
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: titan, target: None, additional_targets: vec![], mode: None,
            x_value: None, pitch_card: None,
        })
        .expect("free via three sacrifices");
        drain_stack(&mut g);
        assert!(g.battlefield_find(titan).is_some());
        assert!(
            g.battlefield_find(a1).is_none()
                && g.battlefield_find(a2).is_none()
                && g.battlefield_find(a3).is_none(),
            "three artifacts sacrificed"
        );
    }

    /// Qasali Ambusher is free (with flash) only under attack with Forest+Plains.
    #[test]
    fn qasali_ambusher_free_when_ambushing() {
        let mut g = two_player_game();
        let cat = g.add_card_to_hand(0, catalog::qasali_ambusher());
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::savannah()); // Forest Plains dual
        g.step = crabomination::game::TurnStep::DeclareBlockers;
        g.active_player_idx = 1;
        g.priority.player_with_priority = 0;
        // No attacker yet → the alternative cast is rejected.
        let err = g.perform_action(GameAction::CastSpellAlternative {
            card_id: cat, target: None, additional_targets: vec![], mode: None,
            x_value: None, pitch_card: None,
        });
        assert!(err.is_err(), "no attacker → no free cast");
        let raider = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.attacking.push(crabomination::game::types::Attack {
            attacker: raider,
            target: crabomination::game::types::AttackTarget::Player(0),
        });
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: cat, target: None, additional_targets: vec![], mode: None,
            x_value: None, pitch_card: None,
        })
        .expect("ambush: free flash cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(cat).is_some());
    }

    /// Boros Reckoner bounces damage dealt to it at any target.
    #[test]
    fn boros_reckoner_bounces_damage() {
        let mut g = two_player_game();
        let reckoner = g.add_card_to_battlefield(0, catalog::boros_reckoner());
        let life1 = g.players[1].life;
        let ctx = crabomination::game::effects::EffectContext::for_spell(1, Some(Target::Permanent(reckoner)), 0, 0);
        let events = g
            .resolve_effect(
                &crabomination::effect::Effect::DealDamage {
                    to: crabomination::effect::Selector::Target(0),
                    amount: crabomination::effect::Value::Const(3),
                },
                &ctx,
            )
            .unwrap();
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1 - 3, "3 damage bounced at the opponent");
    }

    /// Blistercoil Weird grows and untaps on an instant cast.
    #[test]
    fn blistercoil_weird_untaps_on_cast() {
        let mut g = two_player_game();
        let weird = g.add_card_to_battlefield(0, catalog::blistercoil_weird());
        g.battlefield_find_mut(weird).unwrap().tapped = true;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .expect("cast bolt");
        drain_stack(&mut g);
        let c = g.battlefield_find(weird).unwrap();
        assert!(!c.tapped, "untapped by the trigger");
        let cp = g.computed_permanent(weird).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2), "+1/+1 EOT");
    }

    /// Sage of the Falls offers a loot when a non-Human creature enters.
    #[test]
    fn sage_of_the_falls_loots() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_hand(0, catalog::lightning_bolt());
        let sage = g.add_card_to_battlefield(0, catalog::sage_of_the_falls());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: sage }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 1, "drew one, discarded one");
        assert_eq!(g.players[0].graveyard.len(), 1);
    }

    /// Elusive Spellfist pumps and slips through on a noncreature cast.
    #[test]
    fn elusive_spellfist_unblockable_on_cast() {
        let mut g = two_player_game();
        let monk = g.add_card_to_battlefield(0, catalog::elusive_spellfist());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = crabomination::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        let cp = g.computed_permanent(monk).unwrap();
        assert_eq!(cp.power, 2, "+1/+0");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Unblockable));
    }
}

mod recent113 {
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        let mut guard = 0;
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
            guard += 1;
            assert!(guard < 60, "advance_to overran");
        }
    }

    /// CR 601.3e — Void Winnower stops an opponent's even-mana-value spell but not
    /// an odd one (zero is even is covered by the block test).
    #[test]
    fn void_winnower_locks_even_mv_casts() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::void_winnower());
        // Grizzly Bears ({1}{G}) is MV 2 (even) — locked for the opponent.
        let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        let err = g.perform_action(GameAction::CastSpell {
            card_id: bears, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        });
        assert!(err.is_err(), "even-mv spell locked");
        // {R} Lightning Bolt is MV 1 (odd) — castable.
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .expect("odd-mv spell allowed");
    }

    /// CR 509.1 — an opponent can't block with an even-mana-value creature under
    /// Void Winnower, but an odd one blocks fine.
    #[test]
    fn void_winnower_locks_even_mv_blocks() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::void_winnower());
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(attacker);
        // Grizzly Bears (MV 2, even) can't block; Savannah Lions (MV 1, odd) can.
        let even_blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let odd_blk = g.add_card_to_battlefield(1, catalog::savannah_lions());
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker, target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        advance_to(&mut g, TurnStep::DeclareBlockers);
        assert!(
            g.perform_action(GameAction::DeclareBlockers(vec![(even_blk, attacker)])).is_err(),
            "even-mv creature can't block"
        );
        g.perform_action(GameAction::DeclareBlockers(vec![(odd_blk, attacker)]))
            .expect("odd-mv creature blocks");
    }

    /// Conduit of Ruin shaves {2} off the first creature spell each turn, not the
    /// second.
    #[test]
    fn conduit_of_ruin_first_creature_spell_discount() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::conduit_of_ruin());
        // Grizzly Bears is {1}{G}; the discount makes it free (generic-only clamp).
        let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("first creature spell discounted to {G}");
        drain_stack(&mut g);
        // A second creature spell gets no discount.
        let bears2 = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        assert!(
            g.perform_action(GameAction::CastSpell {
                card_id: bears2, target: None, additional_targets: vec![], mode: None, x_value: None,
            })
            .is_err(),
            "second creature spell is full price"
        );
    }

    /// Price of Progress hits each player for twice their nonbasic-land count.
    #[test]
    fn price_of_progress_scales_per_player() {
        let mut g = two_player_game();
        // P0 controls two nonbasics, P1 one; basics don't count.
        g.add_card_to_battlefield(0, catalog::steam_vents());
        g.add_card_to_battlefield(0, catalog::steam_vents());
        g.add_card_to_battlefield(0, catalog::island());
        g.add_card_to_battlefield(1, catalog::steam_vents());
        let pop = g.add_card_to_hand(0, catalog::price_of_progress());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let l0 = g.players[0].life;
        let l1 = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: pop, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, l0 - 4, "2 nonbasics x2");
        assert_eq!(g.players[1].life, l1 - 2, "1 nonbasic x2");
    }

    /// Undead Augur draws and drains a life when a Zombie you control dies.
    #[test]
    fn undead_augur_zombie_death_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::undead_augur());
        let goblin = g.add_card_to_battlefield(0, catalog::putrid_goblin()); // a Zombie
        g.add_card_to_library(0, catalog::island());
        let life = g.players[0].life;
        g.battlefield_find_mut(goblin).unwrap().damage = 2; // lethal → CreatureDied
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 1, "drew on the Zombie death");
        assert_eq!(g.players[0].life, life - 1, "lost 1 life");
    }

    /// King of the Pride buffs other Cats but not itself.
    #[test]
    fn king_of_the_pride_cat_anthem() {
        let mut g = two_player_game();
        let king = g.add_card_to_battlefield(0, catalog::king_of_the_pride());
        let lion = g.add_card_to_battlefield(0, catalog::savannah_lions()); // a Cat
        let kcp = g.computed_permanent(king).unwrap();
        assert_eq!((kcp.power, kcp.toughness), (2, 1), "king unbuffed");
        let lcp = g.computed_permanent(lion).unwrap();
        assert_eq!((lcp.power, lcp.toughness), (4, 2), "other Cat gets +2/+1");
    }

    /// Vesperlark returns a small creature when it leaves the battlefield.
    #[test]
    fn vesperlark_ltb_reanimates_small() {
        let mut g = two_player_game();
        let lark = g.add_card_to_battlefield(0, catalog::vesperlark());
        let lion = g.add_card_to_graveyard(0, catalog::savannah_lions()); // power 2 — illegal
        let goblin = g.add_card_to_graveyard(0, catalog::putrid_goblin()); // power 2 — illegal
        let elf = g.add_card_to_graveyard(0, catalog::llanowar_elves()); // power 1 — legal
        let _ = (lion, goblin);
        let evs = g.remove_to_graveyard_with_triggers(lark);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield_find(elf).is_some(), "power-1 creature returned");
    }

    /// Igneous Elemental costs {2} less with a land in the graveyard.
    #[test]
    fn igneous_elemental_land_discount() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::island());
        let elem = g.add_card_to_hand(0, catalog::igneous_elemental());
        // {4}{R}{R} minus {2} = {2}{R}{R}.
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: elem, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("discounted by the graveyard land");
    }

    /// Mother Bear mints two Bears from the graveyard, exiling itself.
    #[test]
    fn mother_bear_graveyard_bears() {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::mother_bear());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: bear, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate from graveyard");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Bear").count(),
            2,
            "two Bear tokens"
        );
        assert!(g.exile.iter().any(|c| c.id == bear), "Mother Bear exiled as the cost");
    }

    /// Savage Swipe pumps a power-2 creature before it fights.
    #[test]
    fn savage_swipe_conditional_pump_fight() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
        let theirs = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
        let swipe = g.add_card_to_hand(0, catalog::savage_swipe());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: swipe, target: Some(Target::Permanent(mine)),
            additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        // Bears become 4/4, deal 4 to the 3/3 (dies); take 3 back and survive at 4/1.
        assert!(g.battlefield_find(theirs).is_none(), "3/3 dies to the pumped fighter");
        assert!(g.battlefield_find(mine).is_some(), "pumped fighter survives");
    }

    /// Fists of Flame draws, then pumps by cards drawn this turn.
    #[test]
    fn fists_of_flame_draw_scaled_pump() {
        let mut g = two_player_game();
        let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::island());
        let fists = g.add_card_to_hand(0, catalog::fists_of_flame());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: fists, target: Some(Target::Permanent(bears)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bears).unwrap();
        assert_eq!(cp.power, 3, "2 base + one card drawn this turn");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Trample));
    }

    /// Changeling Outcast can't block and can't be blocked.
    #[test]
    fn changeling_outcast_evasion() {
        let mut g = two_player_game();
        let outcast = g.add_card_to_battlefield(0, catalog::changeling_outcast());
        let cp = g.computed_permanent(outcast).unwrap();
        assert!(cp.keywords.contains(&crabomination::card::Keyword::CantBlock));
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Unblockable));
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Changeling));
    }

    /// Irregular Cohort brings a changeling friend.
    #[test]
    fn irregular_cohort_makes_token() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::irregular_cohort());
        drain_stack(&mut g);
        let tok = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Shapeshifter")
            .expect("token minted");
        assert!(tok.definition.keywords.contains(&crabomination::card::Keyword::Changeling));
    }

    /// Rain of Revelation nets two cards (draw three, discard one).
    #[test]
    fn rain_of_revelation_draw_three_discard_one() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        let rain = g.add_card_to_hand(0, catalog::rain_of_revelation());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: rain, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 2, "drew 3, discarded 1");
    }

    /// Martyr's Soul enters with two counters when you control no tapped lands.
    #[test]
    fn martyrs_soul_counters_when_untapped() {
        let mut g = two_player_game();
        let soul = g.move_card_to_battlefield_for_test(0, catalog::martyrs_soul());
        drain_stack(&mut g);
        let cp = g.computed_permanent(soul).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 4), "3/2 + two +1/+1 counters");
    }

    /// Orcish Hellraiser burns a player when it dies (echo aside).
    #[test]
    fn orcish_hellraiser_death_burn() {
        let mut g = two_player_game();
        let devil = g.add_card_to_battlefield(0, catalog::orcish_hellraiser());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Target(Target::Player(1)),
        ]));
        let life = g.players[1].life;
        g.battlefield_find_mut(devil).unwrap().damage = 2;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2, "2 damage on death");
    }

    /// Vengeful Devil's ping is morbid-gated.
    #[test]
    fn vengeful_devil_morbid_gate() {
        let mut g = two_player_game();
        let devil = g.add_card_to_battlefield(0, catalog::vengeful_devil());
        g.clear_sickness(devil);
        g.priority.player_with_priority = 0;
        // No creature has died — activation is illegal.
        assert!(
            g.perform_action(GameAction::ActivateAbility {
                card_id: devil, ability_index: 0, target: Some(Target::Player(1)),
                additional_targets: vec![], x_value: None,
            })
            .is_err(),
            "morbid not active yet"
        );
        // Kill something, then the ping is legal.
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(fodder).unwrap().damage = 2;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        let life = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: devil, ability_index: 0, target: Some(Target::Player(1)),
            additional_targets: vec![], x_value: None,
        })
        .expect("morbid active");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1);
    }

    /// Pondering Mage draws on arrival.
    #[test]
    fn pondering_mage_etb_draw() {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        g.move_card_to_battlefield_for_test(0, catalog::pondering_mage());
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 1, "drew a card");
    }

    /// Graveshifter rescues a creature from the graveyard on arrival.
    #[test]
    fn graveshifter_returns_creature() {
        let mut g = two_player_game();
        let lion = g.add_card_to_graveyard(0, catalog::savannah_lions());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(true),
            crabomination::decision::DecisionAnswer::Target(Target::Permanent(lion)),
        ]));
        g.move_card_to_battlefield_for_test(0, catalog::graveshifter());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == lion), "creature back to hand");
    }

    /// Excavating Anurid grows and gains vigilance once threshold is on.
    #[test]
    fn excavating_anurid_threshold() {
        let mut g = two_player_game();
        let frog = g.add_card_to_battlefield(0, catalog::excavating_anurid());
        let cp = g.computed_permanent(frog).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "no threshold yet");
        for _ in 0..7 {
            g.add_card_to_graveyard(0, catalog::island());
        }
        let cp = g.computed_permanent(frog).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5), "+1/+1 with threshold");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Vigilance));
    }

    /// Goblin War Party's entwine takes both modes: three Goblins and a team pump.
    #[test]
    fn goblin_war_party_entwine_both_modes() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let gwp = g.add_card_to_hand(0, catalog::goblin_war_party());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(5);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpellEntwine {
            card_id: gwp, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("entwine both");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count(),
            3,
            "three Goblins"
        );
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "team +1/+1");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Haste));
    }

    /// Viashino Sandsprinter returns to hand at the end step.
    #[test]
    fn viashino_sandsprinter_end_step_bounce() {
        let mut g = two_player_game();
        let viashino = g.add_card_to_battlefield(0, catalog::viashino_sandsprinter());
        advance_to(&mut g, TurnStep::End);
        drain_stack(&mut g);
        assert!(g.battlefield_find(viashino).is_none(), "left the battlefield");
        assert!(g.players[0].hand.iter().any(|c| c.id == viashino), "back in hand");
    }

    /// Treetop Ambusher pumps a friend when it attacks.
    #[test]
    fn treetop_ambusher_attack_pump() {
        let mut g = two_player_game();
        let ambusher = g.add_card_to_battlefield(0, catalog::treetop_ambusher());
        let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(ambusher);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Target(Target::Permanent(friend)),
        ]));
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ambusher, target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        let cp = g.computed_permanent(friend).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 to the chosen friend");
    }

    /// Zhalfirin Decoy's tap is gated on a creature having entered this turn
    /// (CR 603 — its own arrival satisfies the gate).
    #[test]
    fn zhalfirin_decoy_entered_gate() {
        let mut g = two_player_game();
        let decoy = g.add_card_to_battlefield(0, catalog::zhalfirin_decoy());
        g.clear_sickness(decoy);
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.priority.player_with_priority = 0;
        // Nothing entered this turn (direct-add bypasses the counter) → illegal.
        assert!(
            g.perform_action(GameAction::ActivateAbility {
                card_id: decoy, ability_index: 0, target: Some(Target::Permanent(victim)),
                additional_targets: vec![], x_value: None,
            })
            .is_err(),
            "no creature entered this turn"
        );
        // Mark a creature as having entered, then the tap is legal.
        g.players[0].creatures_entered_this_turn.push(decoy);
        g.perform_action(GameAction::ActivateAbility {
            card_id: decoy, ability_index: 0, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], x_value: None,
        })
        .expect("gate satisfied");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).unwrap().tapped, "victim tapped");
    }

    /// Murasa Behemoth swells while a land sits in the graveyard.
    #[test]
    fn murasa_behemoth_land_in_graveyard() {
        let mut g = two_player_game();
        let beh = g.add_card_to_battlefield(0, catalog::murasa_behemoth());
        let cp = g.computed_permanent(beh).unwrap();
        assert_eq!((cp.power, cp.toughness), (5, 5), "no land in graveyard");
        g.add_card_to_graveyard(0, catalog::island());
        let cp = g.computed_permanent(beh).unwrap();
        assert_eq!((cp.power, cp.toughness), (8, 8), "+3/+3 with a graveyard land");
    }

    /// Knight of Old Benalia rallies the team when it enters.
    #[test]
    fn knight_of_old_benalia_etb_pump() {
        let mut g = two_player_game();
        let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::knight_of_old_benalia());
        drain_stack(&mut g);
        let cp = g.computed_permanent(friend).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "other creature gets +1/+1");
    }

    /// Rank Officer drains each opponent by exiling a creature from the graveyard.
    #[test]
    fn rank_officer_graveyard_drain() {
        let mut g = two_player_game();
        let officer = g.add_card_to_battlefield(0, catalog::rank_officer());
        g.clear_sickness(officer);
        let fodder = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        let life = g.players[1].life;
        g.perform_action(GameAction::ActivateAbility {
            card_id: officer, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate drain");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 2, "opponent lost 2");
        assert!(g.exile.iter().any(|c| c.id == fodder), "creature exiled as the cost");
    }

    /// Silumgar Scavenger grows when another creature you control dies.
    #[test]
    fn silumgar_scavenger_grows_on_death() {
        let mut g = two_player_game();
        let scav = g.add_card_to_battlefield(0, catalog::silumgar_scavenger());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(fodder).unwrap().damage = 2;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        let cp = g.computed_permanent(scav).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 4), "+1/+1 counter from the death");
    }
}

mod recent114 {
    use crabomination::catalog;
    use crabomination::card::CardType;
    use crabomination::game::effects::EntityRef;
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;
    // ── Opalescence / Starfield (NonAuraEnchantmentsAreCreatures) ────────────────

    /// CR 613 — Opalescence makes each *other* non-Aura enchantment an `MV/MV`
    /// creature, and leaves itself alone.
    #[test]
    fn opalescence_animates_other_non_aura_enchantments() {
        let mut g = two_player_game();
        let opal = g.add_card_to_battlefield(0, catalog::opalescence());
        // Enchantress's Presence is {2}{G} — mana value 3.
        let presence = g.add_card_to_battlefield(0, catalog::enchantresss_presence());

        let cp = g.computed_permanent(presence).unwrap();
        assert!(cp.card_types.contains(&CardType::Creature), "non-Aura enchantment becomes a creature");
        assert_eq!((cp.power, cp.toughness), (3, 3), "base P/T = mana value");

        // Opalescence itself ("each *other*") is untouched.
        let self_cp = g.computed_permanent(opal).unwrap();
        assert!(!self_cp.card_types.contains(&CardType::Creature), "Opalescence isn't a creature");
    }

    /// Starfield of Nyx only animates while its controller has 5+ enchantments.
    #[test]
    fn starfield_gates_on_five_enchantments() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::starfield_of_nyx());
        let presence = g.add_card_to_battlefield(0, catalog::enchantresss_presence());
        // Two enchantments total → gate unmet.
        assert!(
            !g.computed_permanent(presence).unwrap().card_types.contains(&CardType::Creature),
            "under five enchantments, nothing animates"
        );
        // Pad up to five enchantments.
        g.add_card_to_battlefield(0, catalog::sigil_of_the_empty_throne());
        g.add_card_to_battlefield(0, catalog::greater_auramancy());
        g.add_card_to_battlefield(0, catalog::nevermore());
        assert!(
            g.computed_permanent(presence).unwrap().card_types.contains(&CardType::Creature),
            "at five enchantments Starfield animates the others"
        );
    }

    /// Enchantress's Presence draws a card when you cast an enchantment spell.
    #[test]
    fn enchantresss_presence_draws_on_enchantment_cast() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::enchantresss_presence());
        let spell = g.add_card_to_hand(0, catalog::sigil_of_the_empty_throne());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(3);
        let hand = g.players[0].hand.len();
        cast(&mut g, spell);
        // Cast one enchantment (-1 hand), the trigger drew one (+1) → net even.
        assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "cast an enchantment, drew a card");
    }

    /// Herald of the Pantheon reduces enchantment spells by {1} and gains life.
    #[test]
    fn herald_of_the_pantheon_reduces_and_gains() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::herald_of_the_pantheon());
        // Enchantress's Presence is {2}{G}; the {1} discount lets {1}{G} pay it.
        let ench = g.add_card_to_hand(0, catalog::enchantresss_presence());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        let life = g.players[0].life;
        cast(&mut g, ench);
        assert!(g.battlefield_find(ench).is_some(), "discounted enchantment resolved");
        assert_eq!(g.players[0].life, life + 1, "gained 1 life on the enchantment cast");
    }

    /// Sigil of the Empty Throne mints a 4/4 flying Angel per enchantment cast.
    #[test]
    fn sigil_of_the_empty_throne_makes_angels() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::sigil_of_the_empty_throne());
        let ench = g.add_card_to_hand(0, catalog::greater_auramancy());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, ench);
        let angels = g.battlefield.iter().filter(|c| c.definition.name == "Angel").count();
        assert_eq!(angels, 1, "one 4/4 Angel from the enchantment cast");
    }

    /// Ajani's Chosen mints a 2/2 Cat when an enchantment enters.
    #[test]
    fn ajanis_chosen_makes_cats() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::ajanis_chosen());
        let ench = g.add_card_to_hand(0, catalog::nevermore());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, ench);
        let cats = g.battlefield.iter().filter(|c| c.definition.name == "Cat").count();
        assert_eq!(cats, 1, "one 2/2 Cat when the enchantment entered");
    }

    /// Privileged Position grants hexproof to your other permanents.
    #[test]
    fn privileged_position_grants_hexproof() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::privileged_position());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        assert!(
            g.computed_permanent(bear).unwrap().keywords.iter().any(|k| matches!(
                k,
                crabomination::card::Keyword::Hexproof
            )),
            "other permanent you control has hexproof"
        );
    }

    /// Greater Auramancy grants shroud to your other enchantments only.
    #[test]
    fn greater_auramancy_grants_shroud_to_enchantments() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::greater_auramancy());
        let other_ench = g.add_card_to_battlefield(0, catalog::enchantresss_presence());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let has_shroud = |cp: &crabomination::game::layers::ComputedPermanent| {
            cp.keywords.iter().any(|k| matches!(k, crabomination::card::Keyword::Shroud))
        };
        assert!(has_shroud(&g.computed_permanent(other_ench).unwrap()), "other enchantment has shroud");
        assert!(!has_shroud(&g.computed_permanent(bear).unwrap()), "non-enchantment is unaffected");
    }

    /// Nevermore stops a named spell from being cast.
    #[test]
    fn nevermore_locks_named_spell() {
        let mut g = two_player_game();
        let nm = g.add_card_to_battlefield(0, catalog::nevermore());
        // Name Lightning Bolt on the enchantment.
        g.battlefield_find_mut(nm).unwrap().named_card = Some("Lightning Bolt".into());
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        let err = g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
            mode: None, x_value: None,
        });
        assert!(err.is_err(), "the named spell can't be cast");
    }

    /// Grasp of Fate exiles an opponent's permanent and returns it when it leaves.
    #[test]
    fn grasp_of_fate_exiles_until_it_leaves() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let grasp = g.add_card_to_hand(0, catalog::grasp_of_fate());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: grasp, target: Some(Target::Permanent(foe)), additional_targets: vec![],
            mode: None, x_value: None,
        })
        .expect("cast Grasp of Fate");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "opponent's creature exiled");
        // Grasp leaves; the exiled card returns.
        let gid = g.battlefield.iter().find(|c| c.definition.name == "Grasp of Fate").unwrap().id;
        let evs = g.remove_to_graveyard_with_triggers(gid);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_some(), "returns when Grasp leaves");
    }

    /// Sigil of the New Dawn returns a dead creature to hand for {1}{W}.
    #[test]
    fn sigil_of_the_new_dawn_returns_dead_creature() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::sigil_of_the_new_dawn());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        // AutoDecider declines optional costs; script the {1}{W} payment.
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(true),
        ]));
        g.battlefield_find_mut(bear).unwrap().damage = 2; // lethal → CreatureDied
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "paid {{1}}{{W}} to return the dead creature to hand"
        );
    }

    /// Calix, Guided by Fate puts a +1/+1 counter on a creature on constellation.
    #[test]
    fn calix_constellation_adds_counter() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::calix_guided_by_fate());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let ench = g.add_card_to_hand(0, catalog::nevermore());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: ench, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast enchantment");
        drain_stack(&mut g);
        let _ = bear;
        let total: u32 = g
            .battlefield
            .iter()
            .map(|c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne))
            .sum();
        assert_eq!(total, 1, "constellation put one +1/+1 counter on a creature");
    }

    /// Angelic Renewal sacrifices itself to reanimate a dead creature.
    #[test]
    fn angelic_renewal_reanimates_dead_creature() {
        let mut g = two_player_game();
        let renewal = g.add_card_to_battlefield(0, catalog::angelic_renewal());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(true),
        ]));
        g.battlefield_find_mut(bear).unwrap().damage = 2; // lethal
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.battlefield_find(renewal).is_none(), "sacrificed itself");
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "the dead creature returned to the battlefield"
        );
    }

    /// Aura Fracture sacrifices a land to destroy an enchantment.
    #[test]
    fn aura_fracture_destroys_enchantment() {
        let mut g = two_player_game();
        let fracture = g.add_card_to_battlefield(0, catalog::aura_fracture());
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let target = g.add_card_to_battlefield(1, catalog::enchantresss_presence());
        g.perform_action(GameAction::ActivateAbility {
            card_id: fracture, ability_index: 0, target: Some(Target::Permanent(target)),
            additional_targets: vec![], x_value: None,
        })
        .expect("activate Aura Fracture");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_none(), "a land was sacrificed");
        assert!(g.battlefield_find(target).is_none(), "target enchantment destroyed");
    }

    /// Solitary Confinement prevents all damage to its controller.
    #[test]
    fn solitary_confinement_prevents_damage() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::solitary_confinement());
        let life = g.players[0].life;
        let mut events = Vec::new();
        g.deal_damage_to_from(EntityRef::Player(0), 4, None, &mut events);
        assert_eq!(g.players[0].life, life, "all damage to the controller prevented");
        assert!(g.player_has_static_hexproof(0), "controller has hexproof");
    }

    /// Shielded by Faith makes the enchanted creature indestructible.
    #[test]
    fn shielded_by_faith_grants_indestructible() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::shielded_by_faith());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        cast_at(&mut g, aura, Target::Permanent(bear));
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::Indestructible),
            "enchanted creature is indestructible"
        );
        // Lethal damage doesn't kill it.
        g.battlefield_find_mut(bear).unwrap().damage = 5;
        g.check_state_based_actions();
        assert!(g.battlefield_find(bear).is_some(), "survives lethal damage");
    }

    /// Unquestioned Authority draws on ETB and grants protection from creatures.
    #[test]
    fn unquestioned_authority_draws_and_protects() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::unquestioned_authority());
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        let hand = g.players[0].hand.len();
        cast_at(&mut g, aura, Target::Permanent(bear));
        assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "drew a card on ETB");
        assert!(
            g.computed_permanent(bear).unwrap().keywords.contains(&crabomination::card::Keyword::ProtectionFromCreatures),
            "enchanted creature has protection from creatures"
        );
    }

    /// Sacred Mesa mints a 1/1 flying Pegasus for {1}{W}.
    #[test]
    fn sacred_mesa_makes_pegasi() {
        let mut g = two_player_game();
        let mesa = g.add_card_to_battlefield(0, catalog::sacred_mesa());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: mesa, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate Sacred Mesa");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Pegasus").count(),
            1,
            "one 1/1 Pegasus token"
        );
    }

    /// Aegis of the Gods gives its controller hexproof.
    #[test]
    fn aegis_of_the_gods_grants_player_hexproof() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::aegis_of_the_gods());
        assert!(g.player_has_static_hexproof(0), "controller has hexproof");
    }

    /// Frozen Aether makes an opponent's creature enter tapped.
    #[test]
    fn frozen_aether_taps_opponent_permanents() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::frozen_aether());
        // A creature entering under the opponent (through the real ETB funnel)
        // should arrive tapped.
        let foe = g.move_card_to_battlefield_for_test(1, catalog::grizzly_bears());
        assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's creature entered tapped");
        // The controller's own creature is unaffected.
        let mine = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
        assert!(!g.battlefield_find(mine).unwrap().tapped, "your own creature is untapped");
    }

    /// Griffin Guide pumps + grants flying, and leaves a Griffin when the host dies.
    #[test]
    fn griffin_guide_pumps_and_leaves_a_griffin() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::griffin_guide());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast_at(&mut g, aura, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
        assert!(cp.keywords.contains(&Keyword::Flying), "granted flying");
        g.battlefield_find_mut(bear).unwrap().damage = 4; // lethal
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.definition.name == "Griffin").count(),
            1,
            "a 2/2 Griffin replaces the fallen host"
        );
    }

    /// Angelic Destiny returns to hand when the enchanted creature dies.
    #[test]
    fn angelic_destiny_returns_on_host_death() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::angelic_destiny());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        cast_at(&mut g, aura, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (6, 6), "+4/+4");
        assert!(cp.keywords.contains(&Keyword::FirstStrike), "first strike");
        g.battlefield_find_mut(bear).unwrap().damage = 6;
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Angelic Destiny"),
            "the Aura returns to hand"
        );
    }

    /// Tranquil Grove destroys every other enchantment, sparing itself.
    #[test]
    fn tranquil_grove_wipes_other_enchantments() {
        let mut g = two_player_game();
        let grove = g.add_card_to_battlefield(0, catalog::tranquil_grove());
        let mine = g.add_card_to_battlefield(0, catalog::enchantresss_presence());
        let theirs = g.add_card_to_battlefield(1, catalog::sigil_of_the_empty_throne());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: grove, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate Tranquil Grove");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(), "all other enchantments gone");
        assert!(g.battlefield_find(grove).is_some(), "Tranquil Grove spares itself");
    }

    /// Cho-Manno's Blessing grants the enchanted creature protection from a color.
    #[test]
    fn cho_mannos_blessing_grants_protection() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::cho_mannos_blessing());
        g.players[0].mana_pool.add(Color::White, 2);
        // Script the color choice → Red.
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Color(Color::Red),
        ]));
        cast_at(&mut g, aura, Target::Permanent(bear));
        let cp = g.computed_permanent(bear).unwrap();
        assert!(
            cp.keywords.iter().any(|k| matches!(k, crabomination::card::Keyword::Protection(Color::Red))),
            "enchanted creature has protection from red: {:?}",
            cp.keywords
        );
    }

    /// Flickering Ward can bounce itself back to hand for {W}.
    #[test]
    fn flickering_ward_returns_to_hand() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::flickering_ward());
        g.players[0].mana_pool.add(Color::White, 1);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Color(Color::Blue),
        ]));
        cast_at(&mut g, aura, Target::Permanent(bear));
        let ward = g.battlefield.iter().find(|c| c.definition.name == "Flickering Ward").unwrap().id;
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: ward, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate Flickering Ward bounce");
        drain_stack(&mut g);
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Flickering Ward"),
            "the Aura returned to hand"
        );
    }

    /// Doomwake Giant shrinks the opponents' creatures on constellation.
    #[test]
    fn doomwake_giant_shrinks_opponents() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::doomwake_giant());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        // A second enchantment entering triggers constellation.
        let ench = g.add_card_to_hand(0, catalog::nevermore());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, ench);
        assert_eq!(g.computed_permanent(foe).unwrap().toughness, 1, "opponent's 2/2 becomes 1/1");
    }

    /// Auramancer / Monk Idealist recur an enchantment card from the graveyard.
    #[test]
    fn monk_idealist_recurs_enchantment() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::enchantresss_presence());
        g.move_card_to_battlefield_for_test(0, catalog::monk_idealist());
        drain_stack(&mut g);
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Enchantress's Presence"),
            "the enchantment returned to hand"
        );
    }

    /// Commune with the Gods digs five deep and bins the rest.
    #[test]
    fn commune_with_the_gods_digs_and_bins() {
        let mut g = two_player_game();
        // First-pushed card is on top, so the creature sits within the top five.
        g.add_card_to_library(0, catalog::grizzly_bears()); // a creature to grab
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::forest());
        }
        let spell = g.add_card_to_hand(0, catalog::commune_with_the_gods());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(
                g.players[0].library.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id,
            )),
        ]));
        cast(&mut g, spell);
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "grabbed the creature");
        assert!(g.players[0].graveyard.len() >= 4, "the other revealed cards were binned");
    }

    /// Wildwood Rebirth returns a creature card from the graveyard.
    #[test]
    fn wildwood_rebirth_returns_creature() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::wildwood_rebirth());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast_at(&mut g, spell, Target::Permanent(dead));
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "creature returned to hand");
    }

    /// Font of Fertility sacrifices for a basic land onto the battlefield tapped.
    #[test]
    fn font_of_fertility_fetches_a_basic() {
        let mut g = two_player_game();
        let font = g.add_card_to_battlefield(0, catalog::font_of_fertility());
        let forest = g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        // AutoDecider declines searches; script the fetch.
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Search(Some(forest)),
        ]));
        g.perform_action(GameAction::ActivateAbility {
            card_id: font, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("activate Font of Fertility");
        drain_stack(&mut g);
        assert!(g.battlefield_find(font).is_none(), "the Font was sacrificed");
        let forest = g.battlefield.iter().find(|c| c.definition.name == "Forest");
        assert!(forest.map(|c| c.tapped).unwrap_or(false), "fetched a basic tapped");
    }

    /// Nylea's Presence draws on ETB and makes the enchanted land every basic type.
    #[test]
    fn nyleas_presence_draws_and_grants_types() {
        use crabomination::card::LandType;
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        let aura = g.add_card_to_hand(0, catalog::nyleas_presence());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        let hand = g.players[0].hand.len();
        cast_at(&mut g, aura, Target::Permanent(land));
        assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "drew on ETB");
        let types = &g.computed_permanent(land).unwrap().subtypes.land_types;
        assert!(
            types.contains(&LandType::Island) && types.contains(&LandType::Mountain),
            "enchanted land gained all basic land types: {types:?}"
        );
    }

    /// Serene Heart destroys all Auras and nothing else.
    #[test]
    fn serene_heart_destroys_auras() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_battlefield(0, catalog::wild_growth());
        let ench = g.add_card_to_battlefield(0, catalog::enchantresss_presence());
        let spell = g.add_card_to_hand(0, catalog::serene_heart());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        cast(&mut g, spell);
        assert!(g.battlefield_find(aura).is_none(), "the Aura is destroyed");
        assert!(g.battlefield_find(ench).is_some(), "a non-Aura enchantment survives");
        assert!(g.battlefield_find(bear).is_some(), "creatures survive");
    }

    /// Winds of Rath spares enchanted creatures and wraths the rest.
    #[test]
    fn winds_of_rath_spares_enchanted() {
        let mut g = two_player_game();
        let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let enchanted = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Give `enchanted` an Aura so it counts as enchanted.
        let aura = g.add_card_to_hand(0, catalog::wild_growth()); // (a land aura won't attach to a creature)
        let _ = aura;
        let armor = g.add_card_to_hand(0, catalog::shielded_by_faith());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(1);
        cast_at(&mut g, armor, Target::Permanent(enchanted));
        let spell = g.add_card_to_hand(0, catalog::winds_of_rath());
        g.players[0].mana_pool.add(Color::White, 2);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, spell);
        g.check_state_based_actions();
        assert!(g.battlefield_find(plain).is_none(), "unenchanted creature destroyed");
        assert!(g.battlefield_find(enchanted).is_some(), "enchanted creature survives");
    }

    /// Calming Verse destroys the opponents' enchantments only.
    #[test]
    fn calming_verse_hits_opponents_only() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::enchantresss_presence());
        let theirs = g.add_card_to_battlefield(1, catalog::sigil_of_the_empty_throne());
        let spell = g.add_card_to_hand(0, catalog::calming_verse());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        cast(&mut g, spell);
        assert!(g.battlefield_find(theirs).is_none(), "opponent's enchantment destroyed");
        assert!(g.battlefield_find(mine).is_some(), "your own enchantment survives");
    }

    /// Root Out destroys an enchantment and leaves a Clue.
    #[test]
    fn root_out_destroys_and_investigates() {
        let mut g = two_player_game();
        let target = g.add_card_to_battlefield(1, catalog::sigil_of_the_empty_throne());
        let spell = g.add_card_to_hand(0, catalog::root_out());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        cast_at(&mut g, spell, Target::Permanent(target));
        assert!(g.battlefield_find(target).is_none(), "the enchantment is destroyed");
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"),
            "a Clue token was created"
        );
    }
}

mod recent115 {
    use crabomination::catalog;
    use crabomination::game::{two_player_game};
    use crabomination::mana::Color;

    /// Upwelling keeps every player's unspent mana through a step/phase end.
    #[test]
    fn upwelling_keeps_all_mana() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::upwelling());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[1].mana_pool.add(Color::Blue, 3);
        g.empty_mana_pools();
        assert_eq!(g.players[0].mana_pool.total(), 3, "controller keeps all colors");
        assert_eq!(g.players[1].mana_pool.total(), 3, "opponent keeps mana too");
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "green preserved as green");
    }

    /// Omnath keeps only green; other colors still empty.
    #[test]
    fn omnath_keeps_only_green() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::omnath_locus_of_mana());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add(Color::Red, 3);
        g.empty_mana_pools();
        assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "green survives");
        assert_eq!(g.players[0].mana_pool.amount(Color::Red), 0, "red empties");
    }

    /// Omnath grows +1/+1 for each unspent green mana (live CDA).
    #[test]
    fn omnath_grows_with_unspent_green() {
        let mut g = two_player_game();
        let omnath = g.add_card_to_battlefield(0, catalog::omnath_locus_of_mana());
        assert_eq!(g.computed_permanent(omnath).unwrap().power, 1, "1/1 with an empty pool");
        g.players[0].mana_pool.add(Color::Green, 3);
        assert_eq!(g.computed_permanent(omnath).unwrap().power, 4, "1 + three unspent green");
        assert_eq!(g.computed_permanent(omnath).unwrap().toughness, 4);
    }
}

mod recent116 {
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Untamed Kavu enters with three +1/+1 counters when kicked (5/5).
    #[test]
    fn untamed_kavu_kicked_is_five_five() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let spell = g.add_card_to_hand(0, catalog::untamed_kavu());
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(4); // {1}{G} + kicker {3}
        g.perform_action(GameAction::CastSpellKicked {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Untamed Kavu kicked");
        drain_stack(&mut g);
        let kavu = g.battlefield.iter().find(|c| c.definition.name == "Untamed Kavu").expect("resolved");
        assert_eq!(g.computed_permanent(kavu.id).unwrap().power, 5, "2/2 + three +1/+1 counters");
    }

    /// Manaform Hellkite mints an X/X Dragon token equal to the mana spent on a
    /// noncreature spell (Divination = 3 → a 3/3).
    #[test]
    fn manaform_hellkite_mints_mana_spent_token() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_battlefield(0, catalog::manaform_hellkite());
        for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let spell = g.add_card_to_hand(0, catalog::divination()); // {2}{U}
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Divination");
        drain_stack(&mut g);
        let token = g.battlefield.iter().find(|c| c.definition.name == "Dragon Illusion").expect("token minted");
        assert_eq!(g.computed_permanent(token.id).unwrap().power, 3, "X = 3 mana spent");
        assert!(token.definition.keywords.contains(&crabomination::card::Keyword::Haste), "has haste");
    }

    /// The Dragon Illusion token is exiled at the next end step.
    #[test]
    fn manaform_token_exiles_at_end_step() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_battlefield(0, catalog::manaform_hellkite());
        for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let spell = g.add_card_to_hand(0, catalog::divination());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Divination");
        drain_stack(&mut g);
        let token = g.battlefield.iter().find(|c| c.definition.name == "Dragon Illusion").unwrap().id;
        g.active_player_idx = 0;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        // The token leaves the battlefield at end step; as a token it then ceases
        // to exist (CR 111.7), so it lingers in no zone.
        assert!(g.battlefield.iter().all(|c| c.id != token), "token exiled at end step");
    }
}

mod recent117 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;
    use crabomination::game::{drain_stack, two_player_game};
    use crabomination::mana::Color;

    /// Bigfin Bouncer's ETB returns an opponent's creature to hand.
    #[test]
    fn bigfin_bouncer_bounces_on_etb() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::bigfin_bouncer());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Bigfin Bouncer");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().all(|c| c.id != victim), "opponent's creature bounced");
        assert!(g.players[1].hand.iter().any(|c| c.id == victim), "back in owner's hand");
    }

    /// Alania's Pathmaker exiles the top card and makes it playable.
    #[test]
    fn alanias_pathmaker_impulses_top() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let top = g.add_card_to_library(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::alanias_pathmaker());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Alania's Pathmaker");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == top), "top card exiled and playable");
    }

    /// Apothecary Stomper's ETB (mode 0) puts two +1/+1 counters on a creature.
    #[test]
    fn apothecary_stomper_etb_counters() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::apothecary_stomper());
        g.players[0].mana_pool.add(Color::Green, 2);
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Apothecary Stomper");
        drain_stack(&mut g);
        let counters: u32 = g.battlefield.iter().filter(|c| c.controller == 0)
            .map(|c| c.counter_count(CounterType::PlusOnePlusOne)).sum();
        assert_eq!(counters, 2, "two +1/+1 counters placed");
    }

    /// Armasaur Guide adds a counter when you attack with three or more creatures.
    #[test]
    fn armasaur_guide_counter_on_three_attackers() {
        let mut g = two_player_game();
        let guide = g.add_card_to_battlefield(0, catalog::armasaur_guide());
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        for id in [guide, a, b] { g.clear_sickness(id); }
        g.step = TurnStep::DeclareAttackers;
        g.declare_attackers(vec![
            Attack { attacker: guide, target: AttackTarget::Player(1) },
            Attack { attacker: a, target: AttackTarget::Player(1) },
            Attack { attacker: b, target: AttackTarget::Player(1) },
        ]).expect("declare three attackers");
        drain_stack(&mut g);
        let counters: u32 = g.battlefield.iter().filter(|c| c.controller == 0)
            .map(|c| c.counter_count(CounterType::PlusOnePlusOne)).sum();
        assert_eq!(counters, 1, "one +1/+1 counter on a three-creature attack");
    }

    /// Battlesong Berserker pumps a creature and grants menace when you attack.
    #[test]
    fn battlesong_berserker_pumps_on_attack() {
        let mut g = two_player_game();
        let zerk = g.add_card_to_battlefield(0, catalog::battlesong_berserker());
        g.clear_sickness(zerk);
        g.step = TurnStep::DeclareAttackers;
        g.declare_attackers(vec![Attack { attacker: zerk, target: AttackTarget::Player(1) }])
            .expect("attack");
        drain_stack(&mut g);
        // The berserker is the only creature; it targets itself.
        let cp = g.computed_permanent(zerk).unwrap();
        assert_eq!(cp.power, 4, "3/4 pumped to 4/4");
        assert!(cp.keywords.contains(&crabomination::card::Keyword::Menace), "gains menace");
    }

    /// Billowing Shriekmass mills three on entry and grows under threshold.
    #[test]
    fn billowing_shriekmass_mills_and_threshold() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        for _ in 0..10 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let spell = g.add_card_to_hand(0, catalog::billowing_shriekmass());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        let gy_before = g.players[0].graveyard.len();
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Billowing Shriekmass");
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), gy_before + 3, "milled three");
        let mass = g.battlefield.iter().find(|c| c.definition.name == "Billowing Shriekmass").unwrap().id;
        assert_eq!(g.computed_permanent(mass).unwrap().power, 2, "no threshold yet");
        for _ in 0..7 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
        assert_eq!(g.computed_permanent(mass).unwrap().power, 4, "threshold → +2/+1");
    }

    /// Bulk Up doubles a creature's power.
    #[test]
    fn bulk_up_doubles_power() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let bear = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
        let spell = g.add_card_to_hand(0, catalog::bulk_up());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Bulk Up");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(bear).unwrap().power, 6, "3 doubled to 6");
    }
}
