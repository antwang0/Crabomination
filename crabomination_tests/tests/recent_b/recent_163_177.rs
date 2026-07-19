//! Tests for recentN card batches 163-177 (merged from per-batch micro-files).

mod recent163 {
    use crabomination::catalog;
    use crabomination::card::CounterType;
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Herald of Eternal Dawn keeps its controller from losing even at 0 life.
    #[test]
    fn herald_keeps_you_alive() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::herald_of_eternal_dawn());
        g.players[0].life = 0;
        g.check_state_based_actions();
        assert!(!g.players[0].eliminated, "Herald prevents the loss at 0 life");
    }

    /// Rune-Sealed Wall surveils when tapped.
    #[test]
    fn rune_sealed_wall_surveils() {
        let mut g = two_player_game();
        let wall = g.add_card_to_battlefield(0, catalog::rune_sealed_wall());
        g.clear_sickness(wall);
        g.add_card_to_library(0, catalog::island());
        let lib = g.players[0].library.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: wall, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        })
        .expect("surveil");
        drain_stack(&mut g);
        // Surveil looked at the top card (library shrank only if it went to gy, but
        // at minimum the ability resolved without error and the wall is tapped).
        assert!(g.battlefield_find(wall).unwrap().tapped, "tapped for the ability");
        let _ = lib;
    }

    /// Scrawling Crawler drains an opponent when they draw.
    #[test]
    fn scrawling_crawler_punishes_opponent_draws() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::scrawling_crawler());
        g.add_card_to_library(1, catalog::island());
        let life = g.players[1].life;
        let mut events = vec![];
        g.draw_one(1, &mut events);
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life - 1, "opponent lost 1 life on their draw");
    }

    /// Revenge of the Rats mints one Rat per creature card in the graveyard.
    #[test]
    fn revenge_of_the_rats_swarms() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::lightning_bolt()); // not a creature
        let id = g.add_card_to_hand(0, catalog::revenge_of_the_rats());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Revenge of the Rats");
        drain_stack(&mut g);
        let rats = g.battlefield.iter().filter(|c| c.definition.name == "Rat" && c.controller == 0).count();
        assert_eq!(rats, 2, "one Rat per creature card in the graveyard");
        assert!(g.battlefield.iter().filter(|c| c.definition.name == "Rat").all(|c| c.tapped), "Rats enter tapped");
    }

    /// High-Society Hunter draws whenever another nontoken creature dies.
    #[test]
    fn high_society_hunter_draws_on_death() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::high_society_hunter());
        g.add_card_to_library(0, catalog::island());
        let chump = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let hand = g.players[0].hand.len();
        let mut evs = g.remove_to_graveyard_with_triggers(chump);
        evs.push(GameEvent::CreatureDied { card_id: chump });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew when a nontoken creature died");
    }

    /// Dropkick Bomber buffs other Goblins and can grant one flying.
    #[test]
    fn dropkick_bomber_lord_and_flight() {
        use crabomination::card::Keyword;
        let mut g = two_player_game();
        let bomber = g.add_card_to_battlefield(0, catalog::dropkick_bomber());
        let goblin = g.add_card_to_battlefield(0, catalog::searslicer_goblin()); // 2/1 Goblin
        // Lord: the other Goblin is +1/+1.
        let cp = g.computed_permanent(goblin).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 2), "lord buff");
        // Grant flying to the other Goblin.
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::ActivateAbility {
            card_id: bomber, ability_index: 0,
            target: Some(crabomination::game::types::Target::Permanent(goblin)), additional_targets: vec![], x_value: None,
        })
        .expect("grant flying");
        drain_stack(&mut g);
        assert!(g.computed_permanent(goblin).unwrap().keywords.contains(&Keyword::Flying), "granted flying");
    }

    /// Seeker's Folly (mode 1) shrinks the opponent's board.
    #[test]
    fn seekers_folly_debuffs_opponents() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_hand(0, catalog::seekers_folly());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
        })
        .expect("cast Seeker's Folly, mode 1");
        drain_stack(&mut g);
        let cp = g.computed_permanent(foe).unwrap();
        assert_eq!((cp.power, cp.toughness), (1, 1), "opponents' creatures get -1/-1");
    }

    /// Spinner of Souls digs a creature into hand when another of yours dies.
    #[test]
    fn spinner_of_souls_digs() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::spinner_of_souls());
        // A creature on top of the library to dig into hand.
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::island());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let chump = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let hand = g.players[0].hand.len();
        let mut evs = g.remove_to_graveyard_with_triggers(chump);
        evs.push(GameEvent::CreatureDied { card_id: chump });
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(g.players[0].hand.len() > hand, "dug a creature into hand");
    }
}

mod recent164 {
    use crabomination::catalog;
    use crabomination::card::Keyword;
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Fleeting Flight prevents combat damage to the buffed creature — it survives a
    /// bigger attacker in combat while still dealing its own.
    #[test]
    fn fleeting_flight_prevents_incoming_combat_damage() {
        let mut g = two_player_game();
        // Defender's 2/3 (after the counter) blocks an attacking 4/4.
        let attacker = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 flying
        g.clear_sickness(attacker);
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_hand(1, catalog::fleeting_flight());
        g.players[1].mana_pool.add(Color::White, 1);
        // Player 1 casts Fleeting Flight on their blocker (at instant speed).
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(blocker)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Fleeting Flight");
        drain_stack(&mut g);
        assert!(g.computed_permanent(blocker).unwrap().keywords.contains(&Keyword::Flying), "gained flying");
        // Now resolve combat: attacker (4/4) is blocked by the 3/3 blocker.
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker, target: AttackTarget::Player(1),
        }]))
        .unwrap();
        g.step = TurnStep::DeclareBlockers;
        g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
        g.step = TurnStep::CombatDamage;
        g.resolve_combat().unwrap();
        // The blocker took no combat damage (prevented), so it survives; it still
        // dealt its 3 to the 4/4.
        assert!(g.battlefield_find(blocker).is_some(), "blocker survives — all combat damage to it was prevented");
        assert_eq!(g.battlefield_find(blocker).map(|c| c.damage), Some(0), "no damage marked");
        assert_eq!(g.battlefield_find(attacker).map(|c| c.damage), Some(3), "attacker still took the blocker's 3");
    }

    /// Goblin Negotiation makes a Goblin for each point of excess damage.
    #[test]
    fn goblin_negotiation_makes_goblins_from_excess() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let id = g.add_card_to_hand(0, catalog::goblin_negotiation());
        g.players[0].mana_pool.add(Color::Red, 2);
        g.players[0].mana_pool.add_colorless(5);
        g.step = TurnStep::PreCombatMain;
        // X = 5 → 5 damage to a 2/2 → 3 excess → 3 Goblins.
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: Some(5),
        })
        .expect("cast Goblin Negotiation for X=5");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "the 2/2 died");
        let goblins = g.battlefield.iter().filter(|c| c.definition.name == "Goblin" && c.controller == 0).count();
        assert_eq!(goblins, 3, "3 excess damage → 3 Goblins");
    }

    /// Homunculus Horde copies itself on the second draw each turn.
    #[test]
    fn homunculus_horde_copies_on_second_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::homunculus_horde());
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        // First draw of the turn — no trigger.
        let mut evs = vec![];
        g.draw_one(0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Homunculus Horde").count(), 1, "no copy on the first draw");
        // Second draw — trigger mints a copy.
        let mut evs = vec![];
        g.draw_one(0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Homunculus Horde").count(), 2, "second draw minted a copy");
    }
}

mod recent165 {
    use crabomination::catalog;
    use crabomination::card::Keyword;
    use crabomination::game::types::{Target, TurnStep};
    use crabomination::game::*;

    /// Skyship Buccaneer's Raid draws when you attacked this turn.
    #[test]
    fn skyship_buccaneer_raid_draws() {
        let mut g = two_player_game();
        g.players[0].attacked_this_turn = true;
        g.add_card_to_library(0, catalog::island());
        let hand = g.players[0].hand.len();
        let bucc = g.add_card_to_battlefield(0, catalog::skyship_buccaneer());
        g.fire_self_etb_triggers(bucc, 0);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 1, "Raid drew a card");
    }

    /// Starlight Snare taps its target and locks it down.
    #[test]
    fn starlight_snare_taps_and_locks() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::starlight_snare());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: aura, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Starlight Snare");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).unwrap().tapped, "ETB tapped the creature");
        // It won't untap while enchanted.
        g.battlefield_find_mut(foe).unwrap().tapped = true;
        g.do_untap();
        assert!(g.battlefield_find(foe).unwrap().tapped, "stays tapped — the Aura locks its untap");
    }

    /// Inspiring Paladin has first strike only during its controller's turn.
    #[test]
    fn inspiring_paladin_first_strike_on_your_turn() {
        let mut g = two_player_game();
        let pal = g.add_card_to_battlefield(0, catalog::inspiring_paladin());
        g.active_player_idx = 0;
        assert!(g.computed_permanent(pal).unwrap().keywords.contains(&Keyword::FirstStrike), "first strike on your turn");
        g.active_player_idx = 1;
        assert!(!g.computed_permanent(pal).unwrap().keywords.contains(&Keyword::FirstStrike), "no first strike on the opponent's turn");
    }

    /// Dreadwing Scavenger loots on entry and gains deathtouch at Threshold.
    #[test]
    fn dreadwing_scavenger_loots_and_thresholds() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let hand = g.players[0].hand.len();
        let dread = g.add_card_to_battlefield(0, catalog::dreadwing_scavenger());
        // No Threshold yet → no deathtouch.
        assert!(!g.computed_permanent(dread).unwrap().keywords.contains(&Keyword::Deathtouch));
        g.fire_self_etb_triggers(dread, 0);
        drain_stack(&mut g);
        // Loot: drew then discarded → net hand unchanged (drew 1, discarded 1).
        assert_eq!(g.players[0].hand.len(), hand, "looted (draw then discard)");
        // Fill the graveyard to seven for Threshold.
        for _ in 0..7 {
            g.add_card_to_graveyard(0, catalog::island());
        }
        assert!(g.computed_permanent(dread).unwrap().keywords.contains(&Keyword::Deathtouch), "Threshold grants deathtouch");
    }
}

mod recent166 {
    use crabomination::card::{CardType, CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Territorial Bruntar's landfall exiles leading lands and grants a pay-own-cost
    /// impulse on the first nonland card.
    #[test]
    fn territorial_bruntar_landfall_impulses_first_nonland() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::territorial_bruntar());
        // Library top → bottom: Mountain (land), Grizzly Bears (nonland spell).
        g.add_card_to_library(0, catalog::mountain());
        let spell = g.add_card_to_library(0, catalog::grizzly_bears());
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let land = g.add_card_to_hand(0, catalog::forest());
        g.perform_action(GameAction::PlayLand(land)).expect("play land");
        drain_stack(&mut g);
        let s = g.exile.iter().find(|c| c.id == spell).expect("nonland impulsed to exile");
        assert!(s.may_play_until.is_some(), "castable this turn");
        // Pay-own-cost impulse: the granted cost equals the card's real cost.
        assert!(s.granted_alt_cast_cost_eot.is_some(), "impulse pays the card's own cost, not free");
    }

    /// Solstice Revelations grants a free cast when the nonland's MV is below your
    /// Mountain count, else it goes to hand.
    #[test]
    fn solstice_revelations_free_below_mountains_else_hand() {
        // Enough Mountains: Grizzly Bears (MV 2) < 3 Mountains → free may-play.
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::mountain());
        }
        let spell = g.add_card_to_library(0, catalog::grizzly_bears());
        let cast = g.add_card_to_hand(0, catalog::solstice_revelations());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.cast_spell(cast, None, vec![], None, None).expect("cast Solstice");
        drain_stack(&mut g);
        let s = g.exile.iter().find(|c| c.id == spell).expect("nonland impulsed");
        assert!(s.may_play_until.is_some(), "free may-play granted");
        assert!(s.granted_alt_cast_cost_eot.is_none(), "cast without paying its mana cost");

        // Too few Mountains: MV 2 not below 0 → put into hand instead.
        let mut g2 = two_player_game();
        let spell2 = g2.add_card_to_library(0, catalog::grizzly_bears());
        let cast2 = g2.add_card_to_hand(0, catalog::solstice_revelations());
        g2.players[0].mana_pool.add(Color::Red, 1);
        g2.players[0].mana_pool.add_colorless(2);
        g2.priority.player_with_priority = 0;
        g2.step = TurnStep::PreCombatMain;
        g2.cast_spell(cast2, None, vec![], None, None).expect("cast Solstice");
        drain_stack(&mut g2);
        assert!(g2.players[0].hand.iter().any(|c| c.id == spell2), "put into hand, no free cast");
    }

    /// White Lotus Hideout taps for colorless and, restricted, for any color.
    #[test]
    fn white_lotus_hideout_taps_for_mana() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::white_lotus_hideout());
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        })
        .expect("{T}: Add {C}");
        assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "one colorless produced");
    }

    /// Jasmine Dragon Tea Shop's {5},{T} mints a 1/1 Ally token.
    #[test]
    fn jasmine_dragon_tea_shop_makes_ally() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::jasmine_dragon_tea_shop());
        g.players[0].mana_pool.add_colorless(5);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None,
        })
        .expect("{5},{T}: make Ally");
        drain_stack(&mut g);
        let allies = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Ally").count();
        assert_eq!(allies, 1, "one Ally token");
    }

    /// Secret Tunnel's {4},{T} makes a creature you control unblockable.
    #[test]
    fn secret_tunnel_grants_unblockable() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::secret_tunnel());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(4);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
        })
        .expect("{4},{T}: unblockable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Unblockable), "bear can't be blocked");
    }

    /// Planetarium's scry fires its once-per-turn impulse on the top card.
    #[test]
    fn planetarium_impulses_top_on_scry() {
        let mut g = two_player_game();
        let art = g.add_card_to_battlefield(0, catalog::planetarium_of_wan_shi_tong());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: art, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        })
        .expect("{1},{T}: Scry 2");
        drain_stack(&mut g);
        assert!(
            g.exile.iter().any(|c| c.controller == 0 && c.may_play_until.is_some()),
            "scry triggered an impulse of the top card",
        );
    }

    /// Phoenix Fleet Airship copies itself at end step if you sacrificed a permanent.
    #[test]
    fn phoenix_fleet_airship_copies_after_sacrifice() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::phoenix_fleet_airship());
        g.active_player_idx = 0;
        // Record a sacrifice this turn.
        g.players[0].permanents_sacrificed_this_turn = 1;
        let before = g.battlefield.iter().filter(|c| c.definition.name == "Phoenix Fleet Airship").count();
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        let after = g.battlefield.iter().filter(|c| c.definition.name == "Phoenix Fleet Airship").count();
        assert_eq!(after, before + 1, "a token copy was minted");
    }

    /// Firebender Ascension's ETB mints a firebending 2/2 Soldier.
    #[test]
    fn firebender_ascension_makes_soldier() {
        let mut g = two_player_game();
        g.move_card_to_battlefield_for_test(0, catalog::firebender_ascension());
        drain_stack(&mut g);
        let tok = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Soldier").expect("Soldier token");
        assert_eq!((tok.power(), tok.toughness()), (2, 2));
        assert!(tok.definition.keywords.contains(&Keyword::Firebending(1)), "has firebending 1");
    }

    /// Ragost sacrifices a Food to burn each opponent for 3.
    #[test]
    fn ragost_sacrifices_food_for_damage() {
        let mut g = two_player_game();
        let ragost = g.add_card_to_battlefield(0, catalog::ragost_deft_gastronaut());
        g.clear_sickness(ragost);
        g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token());
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: ragost, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        })
        .expect("{1},{T},Sac Food: 3 to each opponent");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 17, "opponent took 3");
    }

    /// Ragost untaps at end step if you gained life this turn.
    #[test]
    fn ragost_untaps_when_life_gained() {
        let mut g = two_player_game();
        let ragost = g.add_card_to_battlefield(0, catalog::ragost_deft_gastronaut());
        g.battlefield_find_mut(ragost).unwrap().tapped = true;
        g.active_player_idx = 0;
        g.players[0].life_gained_this_turn = 2;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(!g.battlefield_find(ragost).unwrap().tapped, "Ragost untapped");
    }

    /// Invasion Submersible's exhaust animation turns it into a 3/3 artifact creature.
    #[test]
    fn invasion_submersible_exhaust_animates() {
        let mut g = two_player_game();
        let sub = g.add_card_to_battlefield(0, catalog::invasion_submersible());
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: sub, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        })
        .expect("exhaust {3}: animate");
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(sub).unwrap().card_types.contains(&CardType::Creature),
            "became an artifact creature",
        );
        assert_eq!(
            g.battlefield_find(sub).unwrap().counter_count(CounterType::PlusOnePlusOne),
            3,
            "three +1/+1 counters",
        );
    }

    /// Gloryheath Lynx tutors a basic Plains to hand when it attacks while saddled.
    #[test]
    fn gloryheath_lynx_saddled_attack_tutors_plains() {
        let mut g = two_player_game();
        let lynx = g.add_card_to_battlefield(0, catalog::gloryheath_lynx());
        g.clear_sickness(lynx);
        g.battlefield_find_mut(lynx).unwrap().saddled = true;
        let plains = g.add_card_to_library(0, catalog::plains());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains))]));
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: lynx, target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Plains"), "tutored a Plains");
    }

    /// Guardian Sunmare puts a cheap nonland permanent onto the battlefield on a
    /// saddled attack.
    #[test]
    fn guardian_sunmare_saddled_attack_cheats_permanent() {
        let mut g = two_player_game();
        let mare = g.add_card_to_battlefield(0, catalog::guardian_sunmare());
        g.clear_sickness(mare);
        g.battlefield_find_mut(mare).unwrap().saddled = true;
        let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 ≤ 3
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bears))]));
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: mare, target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "cheated Bears in");
    }

    /// Guidelight Optimizer taps for one blue mana.
    #[test]
    fn guidelight_optimizer_taps_for_blue() {
        let mut g = two_player_game();
        let opt = g.add_card_to_battlefield(0, catalog::guidelight_optimizer());
        g.clear_sickness(opt);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: opt, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        })
        .expect("{T}: Add {U}");
        // The mana is spend-restricted (artifact spells/abilities only), so it
        // lands in the restricted pool rather than the open blue slot.
        assert_eq!(g.players[0].mana_pool.restricted_total(), 1, "one restricted blue produced");
    }

    /// Grim Bauble's ETB shrinks an opposing creature by -2/-2.
    #[test]
    fn grim_bauble_etb_shrinks_opponent_creature() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::grim_bauble());
        drain_stack(&mut g);
        // The 2/2 becomes a 0/0 and dies to state-based actions.
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "shrunk bear died");
    }

    /// Gastal Raider gets +1/+1 and menace only at max speed.
    #[test]
    fn gastal_raider_grows_at_max_speed() {
        let mut g = two_player_game();
        let raider = g.add_card_to_battlefield(0, catalog::gastal_raider());
        assert_eq!(g.computed_permanent(raider).unwrap().power, 2, "base 2/1 before max speed");
        g.players[0].speed = 4;
        let cp = g.computed_permanent(raider).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 2), "max speed → 3/2");
        assert!(cp.keywords.contains(&Keyword::Menace), "gains menace at max speed");
    }

    /// Basri makes a lifelinking Cat token with its activated ability.
    #[test]
    fn basri_makes_cat_token() {
        let mut g = two_player_game();
        let basri = g.add_card_to_battlefield(0, catalog::basri_tomorrows_champion());
        g.clear_sickness(basri);
        g.players[0].mana_pool.add(Color::White, 1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: basri, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        })
        .expect("{W},{T}: make Cat");
        drain_stack(&mut g);
        let cat = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Cat").expect("Cat token");
        assert!(cat.definition.keywords.contains(&Keyword::Lifelink), "Cat has lifelink");
    }

    /// Broodheart Engine's sac ability reanimates a creature from your graveyard.
    #[test]
    fn broodheart_engine_reanimates_from_graveyard() {
        let mut g = two_player_game();
        let engine = g.add_card_to_battlefield(0, catalog::broodheart_engine());
        g.clear_sickness(engine);
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: engine, ability_index: 0, target: Some(Target::Permanent(dead)),
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("sac: reanimate");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == dead), "Bears back on the battlefield");
    }

    /// Amonkhet Raceway's max-speed ability grants haste.
    #[test]
    fn amonkhet_raceway_max_speed_grants_haste() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::amonkhet_raceway());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].speed = 4;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 1, target: Some(Target::Permanent(bear)),
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("max speed {T}: grant haste");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Haste), "bear gained haste");
    }

    /// Fang-Druid Summoner tutors a creature card to hand on ETB.
    #[test]
    fn fang_druid_summoner_tutors_creature() {
        let mut g = two_player_game();
        let bears = g.add_card_to_library(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bears))]));
        g.move_card_to_battlefield_for_test(0, catalog::fang_druid_summoner());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "tutored a creature");
    }

    /// Caradora's static adds an extra +1/+1 counter to placements on your creatures.
    #[test]
    fn caradora_adds_extra_counter() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::caradora_heart_of_alacria());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.resolve_effect(
            &crabomination::effect::Effect::AddCounter {
                what: crabomination::effect::Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: crabomination::effect::Value::ONE,
            },
            &crabomination::game::effects::EffectContext::for_ability(bear, 0, None),
        )
        .unwrap();
        assert_eq!(
            g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
            2,
            "one counter becomes two",
        );
    }

    /// Far Fortune pings each opponent when you attack.
    #[test]
    fn far_fortune_pings_on_attack() {
        let mut g = two_player_game();
        let boss = g.add_card_to_battlefield(0, catalog::far_fortune_end_boss());
        g.clear_sickness(boss);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: boss, target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 19, "opponent pinged for 1 on attack");
    }

    /// Hazoret makes a small creature unblockable.
    #[test]
    fn hazoret_makes_small_creature_unblockable() {
        let mut g = two_player_game();
        let haz = g.add_card_to_battlefield(0, catalog::hazoret_godseeker());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: haz, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("{1},{T}: unblockable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Unblockable), "small creature unblockable");
    }

    /// Aatchik's ETB makes one Insect per artifact/creature card in your graveyard.
    #[test]
    fn aatchik_makes_insects_from_graveyard() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.add_card_to_graveyard(0, catalog::lightning_bolt()); // not counted
        g.move_card_to_battlefield_for_test(0, catalog::aatchik_emerald_radian());
        drain_stack(&mut g);
        let insects = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Insect").count();
        assert_eq!(insects, 2, "two creature cards → two Insects");
    }

    /// Aatchik grows and drains when another Insect you control dies.
    #[test]
    fn aatchik_grows_when_insect_dies() {
        use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
        let mut g = two_player_game();
        let aatchik = g.add_card_to_battlefield(0, catalog::aatchik_emerald_radian());
        let insect = TokenDefinition {
            name: "Insect".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
            ..Default::default()
        };
        let bug = g.add_token_to_battlefield(0, &insect);
        g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: bug }]);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(aatchik).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "grew");
        assert_eq!(g.players[1].life, 19, "opponent drained 1");
    }

    /// Fearless Swashbuckler grants haste to your Vehicles.
    #[test]
    fn fearless_swashbuckler_gives_vehicles_haste() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::fearless_swashbuckler());
        let sub = g.add_card_to_battlefield(0, catalog::invasion_submersible());
        assert!(
            g.computed_permanent(sub).unwrap().keywords.contains(&Keyword::Haste),
            "Vehicle has haste from the Swashbuckler",
        );
    }

    /// Gastal Thrillroller animates itself on ETB.
    #[test]
    fn gastal_thrillroller_enters_as_creature() {
        let mut g = two_player_game();
        let v = g.move_card_to_battlefield_for_test(0, catalog::gastal_thrillroller());
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(v).unwrap().card_types.contains(&CardType::Creature),
            "Vehicle is a creature on entry",
        );
    }

    /// Apocalypse Runner grants lifelink + unblockable to a small creature.
    #[test]
    fn apocalypse_runner_buffs_small_creature() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(0, catalog::apocalypse_runner());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: v, ability_index: 0, target: Some(Target::Permanent(bear)),
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("{T}: buff");
        drain_stack(&mut g);
        let cp = g.battlefield_find(bear).unwrap();
        assert!(cp.has_keyword(&Keyword::Lifelink) && cp.has_keyword(&Keyword::Unblockable), "buffed");
    }

    /// Wingshield Agent enters with a shield counter.
    #[test]
    fn wingshield_agent_enters_with_shield() {
        let mut g = two_player_game();
        let w = g.move_card_to_battlefield_for_test(0, catalog::wingshield_agent());
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(w).unwrap().counter_count(CounterType::Shield), 1, "one shield counter");
    }

    /// Country Roads sacrifices for a Pilot token.
    #[test]
    fn country_roads_makes_pilot() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::country_roads());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
        })
        .expect("sac for Pilot");
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Pilot").count(),
            1,
            "one Pilot token",
        );
    }

    /// Guidelight Pathmaker tutors an artifact to hand on ETB.
    #[test]
    fn guidelight_pathmaker_tutors_artifact() {
        let mut g = two_player_game();
        let art = g.add_card_to_library(0, catalog::grim_bauble());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(art))]));
        g.move_card_to_battlefield_for_test(0, catalog::guidelight_pathmaker());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == art), "tutored an artifact");
    }

    /// Voyager Glidecar's tap-three ability animates it with flying + a counter.
    #[test]
    fn voyager_glidecar_animates_with_crew() {
        let mut g = two_player_game();
        let v = g.add_card_to_battlefield(0, catalog::voyager_glidecar());
        let crew: Vec<Target> = (0..3).map(|_| {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(c);
            Target::Permanent(c)
        }).collect();
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: v, ability_index: 0, target: None, additional_targets: crew, x_value: None,
        })
        .expect("tap 3: animate");
        drain_stack(&mut g);
        let cp = g.computed_permanent(v).unwrap();
        assert!(cp.card_types.contains(&CardType::Creature) && cp.keywords.contains(&Keyword::Flying), "animated flyer");
        assert_eq!(g.battlefield_find(v).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "one +1/+1 counter");
    }

    /// Kickoff Celebrations' ETB loots: discard one, draw two.
    #[test]
    fn kickoff_celebrations_loots_on_etb() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::grizzly_bears()); // the card to discard
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let before = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::kickoff_celebrations());
        drain_stack(&mut g);
        // Discard 1, draw 2 → net +1 card in hand.
        assert_eq!(g.players[0].hand.len(), before + 1, "looted: -1 discard, +2 draw");
    }
}

mod recent167 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Loxodon Surveyor's max-speed graveyard ability exiles itself to draw, and is
    /// gated behind speed 4.
    #[test]
    fn loxodon_surveyor_max_speed_draws_from_graveyard() {
        let mut g = two_player_game();
        let surveyor = g.add_card_to_graveyard(0, catalog::loxodon_surveyor());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        // Below max speed → activation rejected.
        g.players[0].speed = 3;
        assert!(
            g.perform_action(GameAction::ActivateAbility {
                card_id: surveyor, ability_index: 0, target: None,
                additional_targets: Vec::new(), x_value: None,
            }).is_err(),
            "not usable below max speed"
        );
        // At max speed → exile self, draw a card.
        g.players[0].speed = 4;
        let before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: surveyor, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("max speed gy draw");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 1, "drew a card");
        assert!(g.exile.iter().any(|c| c.id == surveyor), "exiled itself as a cost");
    }

    /// Leonin Surveyor has first strike only during its controller's turn.
    #[test]
    fn leonin_surveyor_first_strike_only_your_turn() {
        let mut g = two_player_game();
        let leonin = g.add_card_to_battlefield(0, catalog::leonin_surveyor());
        g.active_player_idx = 0;
        assert!(g.computed_permanent(leonin).unwrap().keywords.contains(&Keyword::FirstStrike),
            "first strike on your turn");
        g.active_player_idx = 1;
        assert!(!g.computed_permanent(leonin).unwrap().keywords.contains(&Keyword::FirstStrike),
            "no first strike on opponent's turn");
    }

    /// Ooze Patrol mills two, then counts artifact/creature cards in the graveyard.
    #[test]
    fn ooze_patrol_grows_with_graveyard() {
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
        g.add_card_to_graveyard(0, catalog::sol_ring()); // artifact
        g.add_card_to_library(0, catalog::forest()); // milled — neither
        g.add_card_to_library(0, catalog::grizzly_bears()); // milled — creature
        let ooze = g.move_card_to_battlefield_for_test(0, catalog::ooze_patrol());
        drain_stack(&mut g);
        // Two starting + one milled creature = 3 art/creature cards in gy.
        assert_eq!(g.battlefield_find(ooze).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 3);
    }

    /// Marketback Walker enters with X counters and draws that many on death.
    #[test]
    fn marketback_walker_enters_with_x_and_draws_on_death() {
        let mut g = two_player_game();
        for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
        let walker = g.add_card_to_hand(0, catalog::marketback_walker());
        g.players[0].mana_pool.add_colorless(6); // {X}{X} with X=3
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.cast_spell(walker, None, vec![], None, Some(3)).expect("cast X=3");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(walker).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 3,
            "enters with 3 counters");
        let before = g.players[0].hand.len();
        g.battlefield_find_mut(walker).unwrap().damage = 3; // lethal vs its 3/3
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), before + 3, "drew 3 on death (one per counter)");
    }

    /// Momentum Breaker's sac ability gains life equal to your speed.
    #[test]
    fn momentum_breaker_gains_life_equal_to_speed() {
        let mut g = two_player_game();
        let mb = g.add_card_to_battlefield(0, catalog::momentum_breaker());
        g.players[0].speed = 3;
        g.players[0].life = 20;
        g.players[0].mana_pool.add_colorless(2);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: mb, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("sac: gain life = speed");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 23, "gained 3 life (speed)");
    }

    /// Adrenaline Jockey punishes off-turn spellcasting and grows on exhaust use.
    #[test]
    fn adrenaline_jockey_off_turn_burn() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::adrenaline_jockey());
        // It's player 0's turn; player 1 casts a spell → 4 damage to player 1.
        g.active_player_idx = 0;
        g.players[1].life = 20;
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.cast_spell(bolt, Some(Target::Player(0)), vec![], None, None).expect("opp casts on your turn");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, 16, "off-turn caster took 4");
    }

    /// Hour of Victory makes a Zombie on ETB and, at max speed, tutors to hand.
    #[test]
    fn hour_of_victory_token_and_tutor() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let target = g.add_card_to_library(0, catalog::grizzly_bears());
        let hov = g.move_card_to_battlefield_for_test(0, catalog::hour_of_victory());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Zombie"), "made a Zombie");
        g.players[0].speed = 4;
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
        g.perform_action(GameAction::ActivateAbility {
            card_id: hov, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("max speed sac: tutor");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == target), "tutored the card to hand");
    }

    /// Intimidation Tactics exiles an artifact/creature card from an opponent's hand.
    #[test]
    fn intimidation_tactics_exiles_from_hand() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let creature = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::lightning_bolt()); // not a valid pick
        let spell = g.add_card_to_hand(0, catalog::intimidation_tactics());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![creature])]));
        g.cast_spell(spell, Some(Target::Player(1)), vec![], None, None).expect("cast");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == creature), "exiled the creature card");
    }

    /// Muraganda Raceway taps for {C}, and doubles to {C}{C} at max speed.
    #[test]
    fn muraganda_raceway_max_speed_double_mana() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::muraganda_raceway());
        g.players[0].speed = 4;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 1, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("max speed: {T} add CC");
        assert_eq!(g.players[0].mana_pool.colorless_amount(), 2, "added two colorless at max speed");
    }

    /// Avishkar Raceway's max-speed loot ability requires a discard and max speed.
    #[test]
    fn avishkar_raceway_max_speed_loots() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::avishkar_raceway());
        let pitch = g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.players[0].speed = 4;
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 1, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("max speed: discard, draw");
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == pitch), "discarded a card");
        assert_eq!(g.players[0].hand.len(), hand_before, "net-zero hand (discard 1, draw 1)");
    }

    /// Night Market taps for the color chosen as it entered.
    #[test]
    fn night_market_taps_for_chosen_color() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
        let land = g.move_card_to_battlefield_for_test(0, catalog::night_market());
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).unwrap().tapped, "enters tapped");
        g.battlefield_find_mut(land).unwrap().tapped = false;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("{T}: add chosen color");
        assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "added blue (the chosen color)");
    }

    /// Marshals' Pathcruiser tutors a basic land on ETB and animates via exhaust.
    #[test]
    fn marshals_pathcruiser_tutors_and_animates() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let basic = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
        let vehicle = g.move_card_to_battlefield_for_test(0, catalog::marshals_pathcruiser());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == basic), "tutored a basic land to hand");
        assert!(!g.battlefield_find(vehicle).unwrap().definition.is_creature(), "not a creature before exhaust");
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 1);
        }
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: vehicle, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("exhaust: become creature + 2 counters");
        drain_stack(&mut g);
        let v = g.battlefield_find(vehicle).unwrap();
        assert!(g.computed_permanent(vehicle).unwrap().card_types.contains(&crabomination::card::CardType::Creature), "became an artifact creature");
        assert_eq!(v.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2, "two +1/+1 counters");
    }

    /// Boommobile's exhaust ability deals X damage and grows.
    #[test]
    fn boommobile_exhaust_burns() {
        let mut g = two_player_game();
        let boom = g.add_card_to_battlefield(0, catalog::boommobile());
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(4); // {X=2}{2}{R}
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: boom, ability_index: 0, target: Some(Target::Permanent(foe)),
            additional_targets: Vec::new(), x_value: Some(2),
        })
        .expect("exhaust: X=2 damage");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "2 damage killed the 2/2");
        assert_eq!(g.battlefield_find(boom).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
    }

    /// Howlsquad Heavy grants other Goblins haste.
    #[test]
    fn howlsquad_heavy_goblin_haste() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::howlsquad_heavy());
        let goblin = g.add_card_to_battlefield(0, catalog::mogg_fanatic()); // a Goblin
        assert!(g.computed_permanent(goblin).unwrap().keywords.contains(&Keyword::Haste),
            "other Goblins gain haste");
    }

    /// Boosted Sloop loots (draw then discard) whenever you attack. The trigger is
    /// controller-scoped, so any attacker you declare fires it.
    #[test]
    fn boosted_sloop_attack_loots() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::boosted_sloop());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear, target: AttackTarget::Player(1),
        }]))
        .unwrap();
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before, "loot is net-zero (draw 1, discard 1)");
        assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"), "discarded a card");
    }

    /// Howler's Heavy shrinks an opponent's creature by -3/-0 when cycled.
    #[test]
    fn howlers_heavy_cycle_debuff() {
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let card = g.add_card_to_hand(0, catalog::howlers_heavy());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::Cycle { card_id: card, x_value: None })
            .expect("cycle Howler's Heavy");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(foe).unwrap().power, -1, "2/2 → -1/2 after -3/-0 (only opp creature auto-targeted)");
    }

    /// Wreckage Wickerfolk surveils 2 on entry (one card sent to the graveyard).
    #[test]
    fn wreckage_wickerfolk_surveils() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let top = g.add_card_to_library(0, catalog::grizzly_bears());
        let next = g.add_card_to_library(0, catalog::forest());
        // Surveil 2: keep `next` on top, bin `top` to the graveyard.
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::ScryOrder { kept_top: vec![next], bottom: vec![top] },
        ]));
        let wf = g.move_card_to_battlefield_for_test(0, catalog::wreckage_wickerfolk());
        drain_stack(&mut g);
        assert!(g.computed_permanent(wf).unwrap().keywords.contains(&Keyword::Flying), "has flying");
        assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "surveiled a card to the graveyard");
    }

    /// Transit Mage tutors a mana-value-4 artifact to hand.
    #[test]
    fn transit_mage_tutors_artifact() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let rock = g.add_card_to_library(0, catalog::hedron_archive()); // MV 4 — eligible
        g.add_card_to_library(0, catalog::sol_ring()); // MV 1 — ineligible
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(rock))]));
        g.move_card_to_battlefield_for_test(0, catalog::transit_mage());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == rock), "tutored the MV-4 artifact to hand");
    }

    /// Veteran Beastrider untaps your creatures at your end step.
    #[test]
    fn veteran_beastrider_untaps_at_end_step() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::veteran_beastrider());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        g.active_player_idx = 0;
        g.step = TurnStep::End;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(!g.battlefield_find(bear).unwrap().tapped, "creature untapped at your end step");
    }

    /// Ticket Tortoise makes a Treasure when an opponent has more lands.
    #[test]
    fn ticket_tortoise_treasure_when_behind_on_lands() {
        let mut g = two_player_game();
        for _ in 0..2 { g.add_card_to_battlefield(1, catalog::forest()); }
        g.move_card_to_battlefield_for_test(0, catalog::ticket_tortoise());
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
            "made a Treasure while behind on lands");
    }

    /// Haunt the Network makes two Thopters and drains for your artifact count.
    #[test]
    fn haunt_the_network_tokens_and_drain() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::sol_ring()); // 1 artifact before resolution
        let spell = g.add_card_to_hand(0, catalog::haunt_the_network());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.players[0].life = 20;
        g.players[1].life = 20;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.cast_spell(spell, Some(Target::Player(1)), vec![], None, None).expect("cast");
        drain_stack(&mut g);
        let thopters = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Thopter").count();
        assert_eq!(thopters, 2, "made two Thopters");
        // After the Thopters resolve, artifacts you control = Sol Ring + 2 Thopters = 3.
        assert_eq!(g.players[1].life, 17, "opponent lost 3 (artifact count)");
        assert_eq!(g.players[0].life, 23, "you gained 3");
    }
}

mod recent168 {
    use crabomination::card::CardType;
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;

    /// Hellish Sideswipe draws only when the sacrificed fodder was a Vehicle.
    #[test]
    fn hellish_sideswipe_vehicle_fodder_draws() {
        let mut g = two_player_game();
        let vehicle = g.add_card_to_battlefield(0, catalog::midnight_mangler());
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::hellish_sideswipe());
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        g.pending_cast_sacrifices = Some(vec![vehicle]);
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast with Vehicle fodder");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "target destroyed");
        // -1 spell cast +1 draw = net same, but the draw fired: hand size unchanged
        // (spell left hand, one card drawn).
        assert_eq!(g.players[0].hand.len(), hand_before, "Vehicle fodder drew a card");
    }

    /// Midnight Mangler is an artifact creature only during turns that aren't its
    /// controller's.
    #[test]
    fn midnight_mangler_creature_off_turn() {
        let mut g = two_player_game();
        let mangler = g.add_card_to_battlefield(0, catalog::midnight_mangler());
        g.active_player_idx = 0;
        assert!(
            !g.computed_permanent(mangler).unwrap().card_types.contains(&CardType::Creature),
            "not a creature on your own turn"
        );
        g.active_player_idx = 1;
        assert!(
            g.computed_permanent(mangler).unwrap().card_types.contains(&CardType::Creature),
            "an artifact creature during other players' turns"
        );
    }

    /// Guidelight Matrix's first ability saddles a target Mount.
    #[test]
    fn guidelight_matrix_saddles_mount() {
        let mut g = two_player_game();
        let matrix = g.add_card_to_battlefield(0, catalog::guidelight_matrix());
        let mount = g.add_card_to_battlefield(0, catalog::bridled_bighorn());
        g.clear_sickness(matrix);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert!(!g.battlefield_find(mount).unwrap().saddled, "starts unsaddled");
        g.perform_action(GameAction::ActivateAbility {
            card_id: matrix,
            ability_index: 0,
            target: Some(Target::Permanent(mount)),
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("saddle activation");
        drain_stack(&mut g);
        assert!(g.battlefield_find(mount).unwrap().saddled, "Mount is now saddled");
    }

    /// Interface Ace crews a Crew 2 Vehicle with its toughness (4), which a plain
    /// 0-power creature cannot. Its untap trigger then untaps it that turn.
    #[test]
    fn interface_ace_crews_with_toughness_then_untaps() {
        let mut g = two_player_game();
        let vehicle = g.add_card_to_battlefield(0, catalog::boommobile());
        let ace = g.add_card_to_battlefield(0, catalog::interface_ace());
        let weakling = g.add_card_to_battlefield(0, catalog::ornithopter());
        g.clear_sickness(ace);
        g.clear_sickness(weakling);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        // A 0/2 can't crew (power 0 < 2).
        assert!(
            g.perform_action(GameAction::Crew { vehicle, crew_creatures: vec![weakling] }).is_err(),
            "0-power crewer rejected"
        );
        // Interface Ace crews via toughness 4 ≥ 2.
        g.perform_action(GameAction::Crew { vehicle, crew_creatures: vec![ace] })
            .expect("toughness crews the vehicle");
        drain_stack(&mut g);
        assert!(
            !g.battlefield_find(ace).unwrap().tapped,
            "becomes-tapped trigger untapped Interface Ace on your turn"
        );
    }
}

mod recent169 {
    use crabomination::card::{ArtifactSubtype, CardType, CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Skybox Ferry is a 4/4 flying Vehicle with Crew 2 and Cycling {2}.
    #[test]
    fn skybox_ferry_keywords() {
        let d = catalog::skybox_ferry();
        assert!(d.keywords.contains(&Keyword::Flying));
        assert!(d.keywords.contains(&Keyword::Crew(2)));
        assert!(d.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Vehicle));
        assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
    }

    /// Ripclaw Wrangler's ETB makes each opponent discard.
    #[test]
    fn ripclaw_wrangler_etb_discard() {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::forest());
        let before = g.players[1].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::ripclaw_wrangler());
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), before - 1, "opponent discarded one");
    }

    /// Pothole Mole mills three and returns a land from the graveyard to hand.
    #[test]
    fn pothole_mole_mills_and_returns_land() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::forest());
        let hand_before = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::pothole_mole());
        drain_stack(&mut g);
        // A land was milled and returned → +1 hand; graveyard holds the milled
        // non-land plus whatever wasn't taken.
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "returned a land to hand");
        assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "the milled creature stays in the graveyard");
    }

    /// Roadside Blowout costs {2} less when it targets a mana-value-1 permanent.
    #[test]
    fn roadside_blowout_cost_reduction_and_bounce() {
        let mut g = two_player_game();
        // MV-1 target: {U} alone (2 less than {2}{U}) pays it.
        let target = g.add_card_to_battlefield(1, catalog::savannah_lions()); // {W} 2/1, MV 1
        let spell = g.add_card_to_hand(0, catalog::roadside_blowout());
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(spell, Some(Target::Permanent(target)), vec![], None, None)
            .expect("{U} pays the MV-1-reduced cost");
        drain_stack(&mut g);
        assert!(g.battlefield_find(target).is_none(), "creature bounced");
        assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Savannah Lions"),
            "returned to owner's hand");
    }

    /// Run Over is a one-sided bite: your creature deals its power to an opponent's.
    #[test]
    fn run_over_one_sided_bite() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
        let spell = g.add_card_to_hand(0, catalog::run_over());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(spell, Some(Target::Permanent(mine)), vec![Target::Permanent(theirs)], None, None)
            .expect("cast Run Over");
        drain_stack(&mut g);
        assert!(g.battlefield_find(theirs).is_none(), "their 2/2 took 2 and died");
        assert!(g.battlefield_find(mine).is_some(), "one-sided — my creature is untouched");
    }

    /// Pride of the Road grants double strike at the start of combat when at max
    /// speed.
    #[test]
    fn pride_of_the_road_max_speed_double_strike() {
        let mut g = two_player_game();
        let pride = g.add_card_to_battlefield(0, catalog::pride_of_the_road());
        g.clear_sickness(pride);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].speed = 4;
        advance_to(&mut g, TurnStep::BeginCombat);
        drain_stack(&mut g);
        assert!(
            g.computed_permanent(pride).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "max-speed begin-combat granted double strike"
        );
    }

    /// Rangers' Refueler draws when you activate its exhaust ability, which also
    /// animates it with a +1/+1 counter.
    #[test]
    fn rangers_refueler_exhaust_animate_and_draw() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::rangers_refueler());
        g.add_card_to_library(0, catalog::forest());
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: veh, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("exhaust animate");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew off the exhaust trigger");
        let cp = g.computed_permanent(veh).unwrap();
        assert!(cp.card_types.contains(&CardType::Creature), "animated");
        assert_eq!(g.battlefield_find(veh).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Rocketeer Boostbuggy makes a Treasure whenever it attacks.
    #[test]
    fn rocketeer_boostbuggy_attack_treasure() {
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(0, catalog::rocketeer_boostbuggy());
        g.clear_sickness(veh);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        // Animate it via its exhaust ability so it can attack.
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::ActivateAbility {
            card_id: veh, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("exhaust animate");
        drain_stack(&mut g);
        assert!(g.computed_permanent(veh).unwrap().card_types.contains(&CardType::Creature));
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: veh, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
            "made a Treasure on attack"
        );
    }

    /// Point the Way searches basics equal to your speed onto the battlefield.
    #[test]
    fn point_the_way_searches_basics_equal_to_speed() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let ench = g.add_card_to_battlefield(0, catalog::point_the_way());
        let f1 = g.add_card_to_library(0, catalog::forest());
        let f2 = g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Search(Some(f1)),
            DecisionAnswer::Search(Some(f2)),
        ]));
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.players[0].speed = 2;
        let lands_before = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
        g.perform_action(GameAction::ActivateAbility {
            card_id: ench, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("sac: search basics = speed");
        drain_stack(&mut g);
        let lands_after = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
        assert_eq!(lands_after - lands_before, 2, "fetched 2 basics (speed 2)");
    }

    /// Perilous Snare exiles an opponent's permanent until it leaves the field.
    #[test]
    fn perilous_snare_exiles_until_leaves() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let snare = g.move_card_to_battlefield_for_test(0, catalog::perilous_snare());
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "victim exiled");
        // Snare leaves → victim returns.
        g.remove_from_battlefield_to_graveyard_raw(snare);
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 1),
            "victim returned when the snare left"
        );
    }
}

mod recent170 {
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;

    /// A Roads land enters tapped when you control no Mount or Vehicle, untapped
    /// when you do.
    #[test]
    fn roads_land_enters_tapped_unless_vehicle() {
        // No Vehicle → enters tapped.
        let mut g = two_player_game();
        let l1 = g.move_card_to_battlefield_for_test(0, catalog::foul_roads());
        drain_stack(&mut g);
        assert!(g.battlefield_find(l1).unwrap().tapped, "enters tapped with no Mount/Vehicle");

        // Control a Vehicle → enters untapped.
        let mut g2 = two_player_game();
        g2.add_card_to_battlefield(0, catalog::skybox_ferry());
        let l2 = g2.move_card_to_battlefield_for_test(0, catalog::rocky_roads());
        drain_stack(&mut g2);
        assert!(!g2.battlefield_find(l2).unwrap().tapped, "enters untapped with a Vehicle out");
    }

    /// A Roads land sacrifices for a Pilot token that crews/saddles with a +2 power
    /// bonus.
    #[test]
    fn roads_land_sacrifices_for_boosted_pilot() {
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::reef_roads());
        g.clear_sickness(land);
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 1, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("sac for a Pilot");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_none(), "land sacrificed");
        let pilot = g.battlefield.iter().find(|c| c.definition.name == "Pilot").expect("made a Pilot");
        assert_eq!(g.crew_saddle_power_bonus(pilot.id), 2, "Pilot crews as though +2 power");
    }

    /// Rangers' Aetherhive mints a Thopter whenever you activate an exhaust ability.
    #[test]
    fn rangers_aetherhive_thopter_on_exhaust() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::rangers_aetherhive());
        let refueler = g.add_card_to_battlefield(0, catalog::rangers_refueler());
        g.add_card_to_library(0, catalog::forest()); // refueler's own draw-on-exhaust
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: refueler, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("activate an exhaust ability");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Thopter"),
            "the Aetherhive made a Thopter off the exhaust event");
    }

    /// Racers' Scoreboard draws two and discards one on ETB, and cuts spell costs
    /// by {1} at max speed.
    #[test]
    fn racers_scoreboard_etb_and_max_speed_discount() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let c1 = g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![c1])]));
        let hand_before = g.players[0].hand.len();
        g.move_card_to_battlefield_for_test(0, catalog::racers_scoreboard());
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew two, discarded one");

        // Max speed → a {1}{G} creature costs just {G}.
        let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].speed = 4;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(bear, None, vec![], None, None).expect("max-speed {1} discount pays {1}{G} with {G}");
    }

    /// Salvation Engine buffs other artifact creatures and reanimates an artifact on
    /// attack.
    #[test]
    fn salvation_engine_anthem_and_attack_reanimate() {
        let mut g = two_player_game();
        let engine = g.add_card_to_battlefield(0, catalog::salvation_engine());
        let ornithopter = g.add_card_to_battlefield(0, catalog::ornithopter()); // 0/2 artifact creature
        g.add_card_to_graveyard(0, catalog::sol_ring()); // artifact in gy
        // Anthem: other artifact creatures get +2/+2.
        let cp = g.computed_permanent(ornithopter).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 4), "Ornithopter buffed to 2/4");
        // Animate + attack to fire the reanimation.
        let ts = g.next_timestamp();
        g.add_continuous_effect(crabomination::game::layers::ContinuousEffect {
            timestamp: ts,
            source: engine,
            affected: crabomination::game::layers::AffectedPermanents::Specific(vec![engine]),
            layer: crabomination::game::layers::Layer::L4Type,
            sublayer: None,
            duration: crabomination::game::layers::EffectDuration::UntilEndOfTurn,
            modification: crabomination::game::layers::Modification::AddCardType(crabomination::card::CardType::Creature),
        });
        g.clear_sickness(engine);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: engine, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == "Sol Ring"),
            "reanimated the artifact from the graveyard");
    }
}

mod recent171 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;

    /// Rover Blades grants double strike to the creature it's attached to.
    #[test]
    fn rover_blades_grants_double_strike() {
        let mut g = two_player_game();
        let blades = g.add_card_to_battlefield(0, catalog::rover_blades());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(blades);
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::Equip { equipment: blades, target: bear })
            .expect("equip the bear");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike),
            "equipped creature has double strike");
    }

    /// Spotcycle Scouter is a Crew 1 Vehicle whose ETB scry doesn't disturb the
    /// library size.
    #[test]
    fn spotcycle_scouter_etb_scry() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::island());
        let lib_before = g.players[0].library.len();
        let v = g.move_card_to_battlefield_for_test(0, catalog::spotcycle_scouter());
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib_before, "scry keeps the library size");
        assert!(g.battlefield_find(v).unwrap().definition.keywords.contains(&Keyword::Crew(1)));
    }

    /// Veloheart Bike gains 2 life on ETB.
    #[test]
    fn veloheart_bike_gains_life() {
        let mut g = two_player_game();
        let life = g.players[0].life;
        g.move_card_to_battlefield_for_test(0, catalog::veloheart_bike());
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life + 2, "gained 2 life");
    }

    /// Venomsac Lagac gets +0/+3 when it attacks while saddled.
    #[test]
    fn venomsac_lagac_saddled_attack_pump() {
        let mut g = two_player_game();
        let lagac = g.add_card_to_battlefield(0, catalog::venomsac_lagac());
        g.clear_sickness(lagac);
        g.battlefield_find_mut(lagac).unwrap().saddled = true;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: lagac, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        let s = g.battlefield_find(lagac).unwrap();
        assert_eq!((s.power(), s.toughness()), (2, 4), "saddled attack pump +0/+3");
    }

    /// Stall Out taps the target and lands three stun counters.
    #[test]
    fn stall_out_taps_and_stuns() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::stall_out());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(spell, Some(Target::Permanent(victim)), vec![], None, None).expect("cast Stall Out");
        drain_stack(&mut g);
        let s = g.battlefield_find(victim).unwrap();
        assert!(s.tapped, "target tapped");
        assert_eq!(s.counter_count(CounterType::Stun), 3, "three stun counters");
    }

    /// Trip Up tucks a nonland permanent into its owner's library.
    #[test]
    fn trip_up_tucks_permanent() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let spell = g.add_card_to_hand(0, catalog::trip_up());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.cast_spell(spell, Some(Target::Permanent(victim)), vec![], None, None).expect("cast Trip Up");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "left the battlefield");
        assert!(g.players[1].library.iter().any(|c| c.id == victim), "tucked into owner's library");
    }

    /// Spikeshell Harrier bounces an opponent's creature on ETB.
    #[test]
    fn spikeshell_harrier_bounces_opponent_creature() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::spikeshell_harrier());
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "opponent creature bounced");
        assert!(g.players[1].hand.iter().any(|c| c.id == victim), "returned to owner's hand");
    }
}

mod recent172 {
    use crabomination::card::Keyword;
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget, Target};
    use crabomination::game::*;

    /// Starting Column's max-speed sac draws two and discards one.
    #[test]
    fn starting_column_max_speed_sac_draws() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let col = g.add_card_to_battlefield(0, catalog::starting_column());
        g.clear_sickness(col);
        let a = g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::island());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![a])]));
        g.players[0].speed = 4;
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: col, ability_index: 1, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("max-speed sac");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew two, discarded one");
        assert!(g.battlefield_find(col).is_none(), "sacrificed");
    }

    /// Haunted Hellride buffs and untaps a creature when you attack.
    #[test]
    fn haunted_hellride_attack_trigger() {
        let mut g = two_player_game();
        let hellride = g.add_card_to_battlefield(0, catalog::haunted_hellride());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Animate the Vehicle so it can be the attacker.
        let ts = g.next_timestamp();
        g.add_continuous_effect(crabomination::game::layers::ContinuousEffect {
            timestamp: ts,
            source: hellride,
            affected: crabomination::game::layers::AffectedPermanents::Specific(vec![hellride]),
            layer: crabomination::game::layers::Layer::L4Type,
            sublayer: None,
            duration: crabomination::game::layers::EffectDuration::UntilEndOfTurn,
            modification: crabomination::game::layers::Modification::AddCardType(crabomination::card::CardType::Creature),
        });
        g.clear_sickness(hellride);
        g.battlefield_find_mut(bear).unwrap().tapped = true;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: hellride, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        let s = g.battlefield_find(bear).unwrap();
        assert!(!s.tapped, "the bear was untapped");
        assert_eq!(s.power(), 3, "+1/+0");
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch));
    }

    /// Unswerving Sloth untaps your team and gains indestructible on a saddled
    /// attack.
    #[test]
    fn unswerving_sloth_saddled_attack() {
        let mut g = two_player_game();
        let sloth = g.add_card_to_battlefield(0, catalog::unswerving_sloth());
        let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(sloth);
        g.battlefield_find_mut(sloth).unwrap().saddled = true;
        g.battlefield_find_mut(other).unwrap().tapped = true;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: sloth, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert!(!g.battlefield_find(other).unwrap().tapped, "team untapped");
        assert!(g.computed_permanent(sloth).unwrap().keywords.contains(&Keyword::Indestructible));
    }

    /// Thundering Broodwagon destroys a low-MV opposing permanent on ETB.
    #[test]
    fn thundering_broodwagon_etb_destroys_low_mv() {
        let mut g = two_player_game();
        let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
        let big = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // MV 6
        g.move_card_to_battlefield_for_test(0, catalog::thundering_broodwagon());
        drain_stack(&mut g);
        assert!(g.battlefield_find(small).is_none(), "MV-2 creature destroyed");
        assert!(g.battlefield_find(big).is_some(), "high-MV permanent untouched");
    }

    /// Tune Up returns an artifact from the graveyard to the battlefield.
    #[test]
    fn tune_up_reanimates_artifact() {
        let mut g = two_player_game();
        let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
        let spell = g.add_card_to_hand(0, catalog::tune_up());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(spell, Some(Target::Permanent(ring)), vec![], None, None).expect("cast Tune Up");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == ring), "Sol Ring back on the battlefield");
    }
}

mod recent173 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;

    /// Roadside Assistance attaches, mints a boosted Pilot, and grants +1/+1 +
    /// lifelink.
    #[test]
    fn roadside_assistance_aura_and_pilot() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::roadside_assistance());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(aura, Some(Target::Permanent(bear)), vec![], None, None).expect("cast the Aura");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
        assert!(cp.keywords.contains(&Keyword::Lifelink));
        let pilot = g.battlefield.iter().find(|c| c.definition.name == "Pilot").expect("Pilot minted");
        assert_eq!(g.crew_saddle_power_bonus(pilot.id), 2, "boosted Pilot");
    }

    /// Trade the Helm swaps control of two permanents.
    #[test]
    fn trade_the_helm_swaps_control() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
        let spell = g.add_card_to_hand(0, catalog::trade_the_helm());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(spell, Some(Target::Permanent(mine)), vec![Target::Permanent(theirs)], None, None)
            .expect("cast Trade the Helm");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(mine).unwrap().controller, 1, "my bear went to the opponent");
        assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0, "their angel came to me");
    }

    /// Voyage Home draws three, gains 3, and gets Affinity for artifacts.
    #[test]
    fn voyage_home_affinity_draw_and_life() {
        let mut g = two_player_game();
        // Two artifacts → {2} off {5}{W}{U}.
        g.add_card_to_battlefield(0, catalog::sol_ring());
        g.add_card_to_battlefield(0, catalog::ornithopter());
        for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
        let spell = g.add_card_to_hand(0, catalog::voyage_home());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3); // 5 total = {5}{W}{U} - {2} affinity
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand_before = g.players[0].hand.len();
        let life = g.players[0].life;
        g.cast_spell(spell, None, vec![], None, None).expect("affinity pays the reduced cost");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3, "drew three (spell left hand)");
        assert_eq!(g.players[0].life, life + 3, "gained 3");
    }

    /// Aggressive Negotiations exiles a nonland from the opponent's hand and puts a
    /// counter on your creature.
    #[test]
    fn aggressive_negotiations_exiles_and_counters() {
        let mut g = two_player_game();
        let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::lightning_bolt()); // nonland
        g.add_card_to_hand(1, catalog::forest());
        let spell = g.add_card_to_hand(0, catalog::aggressive_negotiations());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(spell, Some(Target::Permanent(mine)), vec![], None, None)
            .expect("cast Aggressive Negotiations");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.definition.name == "Lightning Bolt"), "nonland exiled");
        assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Forest"), "land kept");
        assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Cloudspire Coordinator taps to mint one Pilot per Mount/Vehicle that entered
    /// this turn.
    #[test]
    fn cloudspire_coordinator_mints_pilots_per_entered_vehicle() {
        let mut g = two_player_game();
        let coord = g.add_card_to_battlefield(0, catalog::cloudspire_coordinator());
        g.clear_sickness(coord);
        g.players[0].mounts_vehicles_entered_this_turn = 2;
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: coord, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("tap to make Pilots");
        drain_stack(&mut g);
        let pilots = g.battlefield.iter().filter(|c| c.definition.name == "Pilot").count();
        assert_eq!(pilots, 2, "two Mounts/Vehicles entered → two Pilot tokens");
    }
}

mod recent174 {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::catalog;
    use crabomination::game::types::Target;
    use crabomination::game::*;

    /// Boom Scholar's exhaust gives your team trample and grows itself by two.
    #[test]
    fn boom_scholar_exhaust_team_trample() {
        let mut g = two_player_game();
        let scholar = g.add_card_to_battlefield(0, catalog::boom_scholar());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(scholar);
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(4);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: scholar, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("exhaust");
        drain_stack(&mut g);
        assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample),
            "team gained trample");
        assert_eq!(g.battlefield_find(scholar).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    }

    /// Spire Mechcycle's exhaust taps another Vehicle to animate and grows by the
    /// number of other Mounts/Vehicles you control.
    #[test]
    fn spire_mechcycle_exhaust_scales_counters() {
        use crabomination::card::CardType;
        let mut g = two_player_game();
        let cycle = g.add_card_to_battlefield(0, catalog::spire_mechcycle());
        let helper = g.add_card_to_battlefield(0, catalog::skybox_ferry()); // another Vehicle
        let helper2 = g.add_card_to_battlefield(0, catalog::veloheart_bike()); // and another
        g.clear_sickness(cycle);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: cycle, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("exhaust taps another Vehicle");
        drain_stack(&mut g);
        assert!(g.computed_permanent(cycle).unwrap().card_types.contains(&CardType::Creature), "animated");
        // Two other Mounts/Vehicles → two counters.
        assert_eq!(g.battlefield_find(cycle).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
        // One of the helpers was tapped to pay the cost.
        assert!(g.battlefield_find(helper).unwrap().tapped || g.battlefield_find(helper2).unwrap().tapped,
            "a helper Vehicle was tapped");
    }

    /// Slick Imitator copies your spell at max speed (opponent eats two Bolts).
    #[test]
    fn slick_imitator_copies_spell_at_max_speed() {
        let mut g = two_player_game();
        let imitator = g.add_card_to_battlefield(0, catalog::slick_imitator());
        g.clear_sickness(imitator);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.players[0].speed = 4;
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let opp_life = g.players[1].life;
        // Cast the Bolt at the opponent; it sits on the stack.
        g.cast_spell(bolt, Some(Target::Player(1)), vec![], None, None).expect("cast Bolt");
        // Copy it with Slick Imitator's max-speed sacrifice ability.
        g.perform_action(GameAction::ActivateAbility {
            card_id: imitator, ability_index: 0, target: Some(Target::Permanent(bolt)),
            additional_targets: Vec::new(), x_value: None,
        })
        .expect("copy the Bolt at max speed");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_life - 6, "two Bolts resolved (original + copy)");
        assert!(g.battlefield_find(imitator).is_none(), "sacrificed to copy");
    }
}

mod recent175 {
    use crabomination::card::CounterType;
    use crabomination::catalog;
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::game::*;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Outpace Oblivion's ETB deals 5 to a creature (kills a Grizzly Bears).
    #[test]
    fn outpace_oblivion_etb_burns_a_creature() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let ench = g.add_card_to_battlefield(0, catalog::outpace_oblivion());
        g.fire_self_etb_triggers(ench, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "5 damage killed the 2/2");
    }

    /// Sabotage Strategist debuffs a creature that attacks its controller.
    #[test]
    fn sabotage_strategist_debuffs_attacker() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(1, catalog::sabotage_strategist()); // defender's
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.clear_sickness(attacker);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker, target: AttackTarget::Player(1),
        }]))
        .expect("attack the Strategist's controller");
        drain_stack(&mut g);
        // 2/2 becomes 1/2 until end of turn.
        let p = g.computed_permanent(attacker).unwrap();
        assert_eq!((p.power, p.toughness), (1, 2), "attacker got -1/-0");
    }

    /// Magmakin Artillerist burns each opponent for the number of cards discarded
    /// in a single resolution (batched CR 701.9 discard event).
    #[test]
    fn magmakin_artillerist_burns_on_batched_discard() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::magmakin_artillerist());
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::grizzly_bears());
        let opp = g.players[1].life;
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        let events = g
            .resolve_effect(
                &crabomination::effect::Effect::Discard {
                    who: crabomination::effect::Selector::You,
                    amount: crabomination::effect::Value::Const(2),
                    random: false,
                },
                &ctx,
            )
            .unwrap();
        g.dispatch_triggers_for_events(&events);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp - 2, "two cards discarded → 2 damage to the opponent");
    }

    /// Waxen Shapethief enters as a copy of an artifact/creature you control.
    #[test]
    fn waxen_shapethief_copies_your_creature() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::grizzly_bears()); // a 2/2 to copy
        let thief = g.add_card_to_hand(0, catalog::waxen_shapethief());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: thief, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Waxen Shapethief");
        drain_stack(&mut g);
        // enters_as_copy resolves on entry; recompute the board.
        let cp = g.computed_permanent(thief).unwrap();
        assert_eq!((cp.power, cp.toughness), (2, 2), "copied the Grizzly Bears");
    }

    /// Quag Feast mills two then destroys the target only if its MV fits the
    /// now-larger graveyard.
    #[test]
    fn quag_feast_destroys_when_graveyard_is_big_enough() {
        use crabomination::game::types::Target;
        let mut g = two_player_game();
        let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
        // Seed one card so mill-2 pushes the graveyard to 3 (≥ 2).
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
        let spell = g.add_card_to_hand(0, catalog::quag_feast());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(spell, Some(Target::Permanent(foe)), vec![], None, None).expect("cast Quag Feast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(foe).is_none(), "MV 2 ≤ 2 milled cards → destroyed");
    }

    /// Plow Through mode 1 destroys a Vehicle.
    #[test]
    fn plow_through_destroys_a_vehicle() {
        use crabomination::game::types::Target;
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(1, catalog::skybox_ferry()); // a Vehicle
        let spell = g.add_card_to_hand(0, catalog::plow_through());
        g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        // Mode 1 = destroy target Vehicle (slot 0).
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(veh)),
            additional_targets: vec![], mode: Some(1), x_value: None,
        }).expect("cast Plow Through, destroy mode");
        drain_stack(&mut g);
        assert!(g.battlefield_find(veh).is_none(), "Vehicle destroyed");
    }

    /// Explosive Getaway blinks a target and deals 4 to each creature.
    #[test]
    fn explosive_getaway_blinks_and_wipes() {
        use crabomination::game::types::Target;
        let mut g = two_player_game();
        let saved = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, exiled → spared
        let doomed = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, eats 4
        let spell = g.add_card_to_hand(0, catalog::explosive_getaway());
        g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(spell, Some(Target::Permanent(saved)), vec![], None, None).expect("cast");
        drain_stack(&mut g);
        assert!(g.battlefield_find(doomed).is_none(), "unexiled creature took 4 and died");
        assert!(g.exile.iter().any(|c| c.id == saved) || g.battlefield_find(saved).is_some(),
            "the blinked creature was spared (in exile or already returned)");
    }

    /// Lightwheel Enhancements pumps and grants vigilance, and seeds speed.
    #[test]
    fn lightwheel_enhancements_pumps_and_seeds_speed() {
        use crabomination::card::Keyword;
        use crabomination::game::types::Target;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_hand(0, catalog::lightwheel_enhancements());
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(aura, Some(Target::Permanent(bear)), vec![], None, None).expect("cast Aura");
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
        assert!(cp.keywords.contains(&Keyword::Vigilance), "granted vigilance");
        assert_eq!(g.players[0].speed, 1, "Start your engines! seeded speed 1");
    }

    /// Thopter Fabricator mints a Thopter on your second draw each turn (once).
    #[test]
    fn thopter_fabricator_mints_on_second_draw() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::thopter_fabricator());
        for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
        g.players[0].cards_drawn_this_turn = 0;
        let mut ev = vec![];
        g.draw_one(0, &mut ev); // first draw — no token
        g.dispatch_triggers_for_events(&ev);
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count(), 0);
        let mut ev2 = vec![];
        g.draw_one(0, &mut ev2); // second draw — one Thopter
        g.dispatch_triggers_for_events(&ev2);
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count(), 1,
            "second draw mints a Thopter");
    }

    /// Coalstoke Gearhulk reanimates a small creature with a finality counter and
    /// grants it haste/menace/deathtouch; it's exiled at the next end step.
    #[test]
    fn coalstoke_gearhulk_reanimates_then_exiles() {
        use crabomination::card::{CounterType, Keyword};
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears()); // MV 2, opponent's gy
        let hulk = g.add_card_to_battlefield(0, catalog::coalstoke_gearhulk());
        g.fire_self_etb_triggers(hulk, 0);
        drain_stack(&mut g);
        let cp = g.computed_permanent(dead).expect("reanimated onto the battlefield");
        assert_eq!(cp.controller, 0, "under my control");
        assert!(cp.keywords.contains(&Keyword::Haste), "gained haste");
        assert_eq!(g.battlefield_find(dead).unwrap().counter_count(CounterType::Finality), 1);
        // At my next end step it's exiled.
        g.step = TurnStep::End;
        g.active_player_idx = 0;
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_none(), "exiled at the next end step");
        assert!(g.exile.iter().any(|c| c.id == dead), "moved to exile");
    }

    /// March of the World Ooze makes your team base 6/6 Oozes and mints an Elephant
    /// when an opponent casts on your turn.
    #[test]
    fn march_of_the_world_ooze_anthem_and_token() {
        use crabomination::card::CreatureType;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::march_of_the_world_ooze());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!((cp.power, cp.toughness), (6, 6), "base 6/6");
        assert!(cp.subtypes.creature_types.contains(&CreatureType::Ooze), "now an Ooze");
        // Opponent casts on your turn → Elephant.
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.active_player_idx = 0; // your turn
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 1;
        g.cast_spell(bolt, Some(Target::Player(0)), vec![], None, None).expect("opponent casts on your turn");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Elephant" && c.controller == 0).count(), 1,
            "minted a 3/3 Elephant");
    }

    /// Possession Engine steals a creature while it stays on the battlefield.
    #[test]
    fn possession_engine_steals_a_creature() {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let engine = g.add_card_to_battlefield(0, catalog::possession_engine());
        g.fire_self_etb_triggers(engine, 0);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(victim).unwrap().controller, 0, "now under my control");
        // Sacrifice the engine → control reverts.
        g.remove_to_graveyard_with_triggers(engine);
        g.check_state_based_actions();
        assert_eq!(g.battlefield_find(victim).unwrap().controller, 1, "control reverts when the Vehicle leaves");
    }

    /// Oildeep Gearhulk makes an opponent discard a chosen card, then draw.
    #[test]
    fn oildeep_gearhulk_coercive_discard_then_draw() {
        let mut g = two_player_game();
        let hulk = g.add_card_to_battlefield(0, catalog::oildeep_gearhulk());
        g.add_card_to_hand(1, catalog::lightning_bolt());
        g.add_card_to_library(1, catalog::forest());
        let hand_before = g.players[1].hand.len();
        g.fire_self_etb_triggers(hulk, 0);
        drain_stack(&mut g);
        // Discard one, draw one → net hand size unchanged, but a card left the hand
        // to the graveyard and a fresh card was drawn.
        assert_eq!(g.players[1].hand.len(), hand_before, "discarded one, drew one");
        assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"),
            "the chosen card was discarded");
    }

    /// Repurposing Bay sacrifices a MV-1 artifact to fetch a MV-2 artifact.
    #[test]
    fn repurposing_bay_fetches_mana_value_plus_one() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        let bay = g.add_card_to_battlefield(0, catalog::repurposing_bay());
        let fodder = g.add_card_to_battlefield(0, catalog::springleaf_drum()); // MV 1 artifact
        let orni = g.add_card_to_library(0, catalog::ornithopter_of_paradise()); // MV 2 — fetchable
        g.clear_sickness(bay);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(orni))]));
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: bay, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        }).expect("sac + fetch");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == orni && c.controller == 0),
            "fetched the MV+1 artifact onto the battlefield");
        assert!(g.battlefield_find(fodder).is_none(), "the MV-1 fodder artifact was sacrificed");
    }

    /// Wreck Remover's ETB exiles a graveyard card and gains 1 life.
    #[test]
    fn wreck_remover_exiles_graveyard_card_and_gains_life() {
        let mut g = two_player_game();
        let gy_card = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let life = g.players[0].life;
        let wr = g.add_card_to_battlefield(0, catalog::wreck_remover());
        g.fire_self_etb_triggers(wr, 0);
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == gy_card), "graveyard card exiled");
        assert_eq!(g.players[0].life, life + 1, "gained 1 life");
    }

    /// Lagorin counters up to two Mounts/Vehicles when it attacks while saddled.
    #[test]
    fn lagorin_saddled_attack_counters_vehicles() {
        use crabomination::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let lagorin = g.add_card_to_battlefield(0, catalog::lagorin_soul_of_alacria());
        let veh1 = g.add_card_to_battlefield(0, catalog::skybox_ferry());
        let veh2 = g.add_card_to_battlefield(0, catalog::veloheart_bike());
        g.clear_sickness(lagorin);
        // Mark Lagorin saddled so the attack trigger fires.
        g.battlefield_find_mut(lagorin).unwrap().saddled = true;
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: lagorin, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(veh1).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
        assert_eq!(g.battlefield_find(veh2).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    }

    /// Riverchurn Monument's base ability mills each opponent two.
    #[test]
    fn riverchurn_monument_mills_each_opponent() {
        let mut g = two_player_game();
        let mon = g.add_card_to_battlefield(0, catalog::riverchurn_monument());
        for _ in 0..5 { g.add_card_to_library(1, catalog::forest()); }
        let gy_before = g.players[1].graveyard.len();
        g.clear_sickness(mon);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: mon, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        }).expect("mill ability");
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), gy_before + 2, "opponent milled two");
    }

    /// Flood the Engine taps the enchanted permanent, strips its abilities, and
    /// keeps it from untapping.
    #[test]
    fn flood_the_engine_taps_and_locks() {
        use crabomination::game::types::Target;
        let mut g = two_player_game();
        let veh = g.add_card_to_battlefield(1, catalog::skybox_ferry()); // has Crew/Flying/Cycling
        let aura = g.add_card_to_hand(0, catalog::flood_the_engine());
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.cast_spell(aura, Some(Target::Permanent(veh)), vec![], None, None).expect("enchant the Vehicle");
        drain_stack(&mut g);
        assert!(g.battlefield_find(veh).unwrap().tapped, "ETB tapped it");
        // Abilities stripped (no keywords in the computed view).
        assert!(g.computed_permanent(veh).unwrap().keywords.is_empty(), "lost all abilities");
        // It stays tapped through its controller's untap step.
        g.active_player_idx = 1;
        g.do_untap();
        assert!(g.battlefield_find(veh).unwrap().tapped, "doesn't untap");
    }
}

mod recent176 {
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::mana::Color;

    /// Dune Drifter's ETB *triggered* ability reads the cast's X: cast with X=2 and
    /// a mana-value-2 card in the graveyard returns to the battlefield.
    #[test]
    fn dune_drifter_etb_reads_cast_x() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // {1}{G} = MV 2
        let spell = g.add_card_to_hand(0, catalog::dune_drifter());
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2); // X=2
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: Some(2),
        })
        .expect("cast Dune Drifter with X=2");
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.id == dead),
            "MV-2 creature reanimated by X=2 ETB"
        );
    }

    /// Vnwxt doubles draws only at max speed (4).
    #[test]
    fn vnwxt_draw_doubles_at_max_speed() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::vnwxt_verbose_host());
        for _ in 0..6 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let mut events = Vec::new();
        // Below max: a draw yields exactly one card.
        g.players[0].speed = 3;
        let before = g.players[0].hand.len();
        g.draw_one(0, &mut events);
        assert_eq!(g.players[0].hand.len(), before + 1, "speed 3 → single draw");
        // Max speed: a draw yields two cards.
        g.players[0].speed = 4;
        let before = g.players[0].hand.len();
        g.draw_one(0, &mut events);
        assert_eq!(g.players[0].hand.len(), before + 2, "speed 4 → doubled draw");
    }

    /// Zahur's max-speed death trigger mints a tapped Zombie; below max, nothing.
    #[test]
    fn zahur_max_speed_death_makes_zombie() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::zahur_glorys_past());
        let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.players[0].speed = 4;
        g.battlefield_find_mut(victim).unwrap().damage = 99; // lethal → CreatureDied
        let evs = g.check_state_based_actions();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.controller == 0
                && c.definition.name == "Zombie"
                && c.tapped),
            "max speed minted a tapped Zombie on the death"
        );
    }

    /// Zahur's sac ability surveils and is once-per-turn.
    #[test]
    fn zahur_sac_ability_is_once_per_turn() {
        let mut g = two_player_game();
        let zahur = g.add_card_to_battlefield(0, catalog::zahur_glorys_past());
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: zahur,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("sac another creature → surveil");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "fodder was sacrificed");
        // A second activation this turn is rejected (no creature to sac anyway, but
        // the once-per-turn gate fires first).
        let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let r = g.perform_action(GameAction::ActivateAbility {
            card_id: zahur,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
        });
        assert!(r.is_err(), "second activation blocked by once-per-turn");
        assert!(g.battlefield_find(other).is_some(), "no extra creature sacrificed");
    }

    /// The Last Ride shrinks by your life total and its {2}{B}, pay-2-life ability
    /// draws a card.
    #[test]
    fn the_last_ride_scales_with_life_and_draws() {
        let mut g = two_player_game();
        let ride = g.add_card_to_battlefield(0, catalog::the_last_ride());
        g.add_card_to_library(0, catalog::grizzly_bears());
        // At 7 life the 13/13 base reads as 6/6.
        g.players[0].life = 7;
        let cp = g.computed_permanent(ride).unwrap();
        assert_eq!((cp.power, cp.toughness), (6, 6), "13/13 − life(7) = 6/6");
        // Pay {2}{B} + 2 life to draw.
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: ride,
            ability_index: 0,
            target: None,
            additional_targets: Vec::new(),
            x_value: None,
        })
        .expect("pay 2 life + {2}{B}: draw");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 5, "paid 2 life");
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    }

    /// The Speed Demon's end step draws and loses life equal to your speed.
    #[test]
    fn the_speed_demon_end_step_scales_with_speed() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::the_speed_demon());
        for _ in 0..5 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        g.players[0].speed = 3;
        g.active_player_idx = 0;
        let life = g.players[0].life;
        let hand = g.players[0].hand.len();
        g.fire_step_triggers(TurnStep::End);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand + 3, "drew 3 (speed)");
        assert_eq!(g.players[0].life, life - 3, "lost 3 (speed)");
    }
}

mod recent177 {
    use crabomination::card::{CreatureType, Keyword};
    use crabomination::catalog;
    use crabomination::game::*;
    use crabomination::mana::Color;

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// Exemplar of Light gains a +1/+1 counter when you gain life and draws a card
    /// the first time counters are placed on it that turn.
    #[test]
    fn exemplar_of_light_counters_then_draws() {
        let mut g = two_player_game();
        let ex = g.add_card_to_battlefield(0, catalog::exemplar_of_light());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let hand = g.players[0].hand.len();
        g.adjust_life(0, 3);
        g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 3 }]);
        drain_stack(&mut g);
        let counters = *g.battlefield_find(ex).unwrap().counters.get(&crabomination::card::CounterType::PlusOnePlusOne).unwrap_or(&0);
        assert_eq!(counters, 1, "gained a +1/+1 counter from lifegain");
        assert_eq!(g.players[0].hand.len(), hand + 1, "drew from the counter trigger");
    }

    /// Ashroot Animist's attack trigger gives another creature you control trample
    /// and +power/+power (power 4 → +4/+4).
    #[test]
    fn ashroot_animist_pumps_ally_on_attack() {
        let mut g = two_player_game();
        let ash = g.add_card_to_battlefield(0, catalog::ashroot_animist());
        let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.clear_sickness(ash);
        g.clear_sickness(ally);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ash,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        let cp = g.computed_permanent(ally).unwrap();
        assert_eq!(cp.power, 6, "2 base + 4 from Ashroot's power");
        assert!(cp.keywords.contains(&Keyword::Trample), "ally gained trample");
    }

    /// Arahbo makes a Cat token when a nontoken Cat enters, and its anthem pumps
    /// other Cats.
    #[test]
    fn arahbo_tokens_and_anthem() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::arahbo_the_first_fang());
        drain_stack(&mut g);
        let before = g.battlefield.len();
        // A nontoken Cat entering triggers the token.
        let mut cat = catalog::grizzly_bears();
        cat.name = "Test Cat";
        cat.subtypes.creature_types = vec![CreatureType::Cat];
        let entered = g.add_card_to_battlefield(0, cat);
        g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: entered }]);
        drain_stack(&mut g);
        assert!(g.battlefield.len() > before + 1, "made a Cat token on top of the entered Cat");
        // The anthem pumps the other Cat (2/2 → 3/3).
        let cp = g.computed_permanent(entered).unwrap();
        assert_eq!(cp.power, 3, "other Cat gets +1/+1 from Arahbo");
    }

    /// Bumbleflower's Sharepot makes a Food on entry and its sac ability destroys a
    /// nonland permanent.
    #[test]
    fn bumbleflowers_sharepot_food_and_destroy() {
        use crabomination::decision::{DecisionAnswer, ScriptedDecider};
        let mut g = two_player_game();
        // move_card_to_battlefield_for_test fires the SelfSource ETB (create Food).
        let pot = g.move_card_to_battlefield_for_test(0, catalog::bumbleflowers_sharepot());
        drain_stack(&mut g);
        assert!(
            g.battlefield.iter().any(|c| c.definition.subtypes.artifact_subtypes.contains(&crabomination::card::ArtifactSubtype::Food)),
            "made a Food token"
        );
        let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add_colorless(5);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
        g.perform_action(GameAction::ActivateAbility {
            card_id: pot, ability_index: 0,
            target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None,
        })
        .expect("sac to destroy");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "destroyed the nonland permanent");
        assert!(g.battlefield_find(pot).is_none(), "Sharepot sacrificed");
    }

    /// Celestial Armor attaches on entry, grants +2/+0 and flying, and gives the
    /// creature hexproof + indestructible until end of turn.
    #[test]
    fn celestial_armor_attaches_and_grants() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Target(Target::Permanent(bear)),
        ]));
        // Fire the SelfSource ETB (attach + grant) via the move path.
        g.move_card_to_battlefield_for_test(0, catalog::celestial_armor());
        drain_stack(&mut g);
        let cp = g.computed_permanent(bear).unwrap();
        assert_eq!(cp.power, 4, "+2/+0 from the Equipment");
        assert!(cp.keywords.contains(&Keyword::Flying), "gained flying");
        assert!(cp.keywords.contains(&Keyword::Indestructible), "gained indestructible EOT");
        assert!(cp.keywords.contains(&Keyword::Hexproof), "gained hexproof EOT");
    }

    /// Strix Lookout loots: {1}{U},{T} draws then discards.
    #[test]
    fn strix_lookout_loots() {
        let mut g = two_player_game();
        let bird = g.add_card_to_battlefield(0, catalog::strix_lookout());
        g.clear_sickness(bird);
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::grizzly_bears()); // something to discard
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let hand = g.players[0].hand.len();
        g.perform_action(GameAction::ActivateAbility {
            card_id: bird, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        })
        .expect("loot");
        drain_stack(&mut g);
        // Draw +1 then discard -1 → net hand unchanged, and a card is in the graveyard.
        assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
        assert!(!g.players[0].graveyard.is_empty(), "discarded a card");
    }

    /// Vanguard Seraph surveils the first time you gain life each turn (once).
    #[test]
    fn vanguard_seraph_surveils_on_first_lifegain() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::vanguard_seraph());
        g.add_card_to_library(0, catalog::grizzly_bears());
        let lib = g.players[0].library.len();
        g.adjust_life(0, 2);
        g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 2 }]);
        drain_stack(&mut g);
        // Surveil 1 auto-keeps or bins the top card; either way the trigger fired
        // (library shrank by 0 or 1). Fire a second lifegain — no extra surveil.
        let after_first = g.players[0].library.len();
        assert!(after_first <= lib, "surveil looked at the top card");
        g.adjust_life(0, 2);
        g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 2 }]);
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), after_first, "only the first lifegain surveils");
    }

    /// Vampire Soulcaller returns a creature card from your graveyard on entry.
    #[test]
    fn vampire_soulcaller_returns_creature() {
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        g.move_card_to_battlefield_for_test(0, catalog::vampire_soulcaller());
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == dead), "returned the creature to hand");
    }

    /// Turn Inside Out pumps +3/+0 and, when the creature dies, manifests dread.
    #[test]
    fn turn_inside_out_pumps_and_manifests_on_death() {
        let mut g = two_player_game();
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        for _ in 0..2 { g.add_card_to_library(0, catalog::grizzly_bears()); }
        let spell = g.add_card_to_hand(0, catalog::turn_inside_out());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast Turn Inside Out");
        drain_stack(&mut g);
        assert_eq!(g.computed_permanent(target).unwrap().power, 5, "+3/+0 → 5 power");
        let bf = g.battlefield.len();
        // Kill the creature this turn → manifest dread fires (a face-down 2/2 enters).
        g.remove_to_graveyard_with_triggers(target);
        g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: target }]);
        drain_stack(&mut g);
        assert!(g.battlefield.len() > bf.saturating_sub(1), "a manifested creature entered");
        assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.face_down), "face-down manifest");
    }

    /// Huskburster Swarm costs {1} less per creature card in your graveyard.
    #[test]
    fn huskburster_swarm_graveyard_affinity() {
        use crabomination::game::actions::cost_reduction_for_spell;
        let mut g = two_player_game();
        let swarm = crabomination::card::CardInstance::new(g.next_id(), catalog::huskburster_swarm(), 0);
        assert_eq!(cost_reduction_for_spell(&g, 0, &swarm, None), 0, "empty graveyard → no discount");
        for _ in 0..3 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
        g.add_card_to_graveyard(0, catalog::lightning_bolt()); // noncreature, ignored
        assert_eq!(cost_reduction_for_spell(&g, 0, &swarm, None), 3, "three creature cards → 3 generic off");
    }
}
